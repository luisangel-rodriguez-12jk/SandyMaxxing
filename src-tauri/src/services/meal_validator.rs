//! Validador de respuestas de IA contra las reglas duras:
//! - Los ingredientes DEBEN existir en allowed_foods_by_group.
//! - Ningún ingrediente puede aparecer en `forbidden` de NINGÚN usuario.
//! - `per_user_portions` debe tener un elemento por cada usuario.
//! - Para cada usuario × meal_type, `portions_consumed` debe coincidir
//!   EXACTAMENTE con la cuota en `user.portions`, ni más ni menos.
//!
//! Devuelve una lista de mensajes en español listos para re-inyectar como
//! feedback al modelo en un reintento.

use std::collections::{HashMap, HashSet};

use crate::ai::plan_generator::{
    MealOptions, PlanMeal, PlanRequest, PlanResult, SingleMeal,
};

const TOLERANCE: f64 = 0.01;

pub fn validate_plan(plan: &PlanResult, req: &PlanRequest) -> Vec<String> {
    let ctx = Context::new(req);
    let mut out = Vec::new();
    for day in &plan.days {
        for meal in &day.meals {
            out.extend(validate_meal_inner(
                meal,
                req,
                &ctx,
                Some(&day.day),
                &meal.meal_type,
            ));
        }
    }
    // Si el plan trae menos/más días de los pedidos, marcamos eso.
    if !req.day_labels.is_empty() {
        if plan.days.len() != req.day_labels.len() {
            out.push(format!(
                "El plan tiene {} días pero 'day_labels' pide exactamente {} días.",
                plan.days.len(),
                req.day_labels.len()
            ));
        }
        for (i, want) in req.day_labels.iter().enumerate() {
            if let Some(got) = plan.days.get(i) {
                if got.day != *want {
                    out.push(format!(
                        "El día #{} debe llamarse '{}' (tal cual en day_labels), no '{}'.",
                        i + 1,
                        want,
                        got.day
                    ));
                }
            }
        }
    }
    out
}

pub fn validate_single_meal(
    meal: &SingleMeal,
    req: &PlanRequest,
    meal_type: &str,
) -> Vec<String> {
    let ctx = Context::new(req);
    let pseudo = PlanMeal {
        meal_type: meal_type.to_string(),
        name: meal.name.clone(),
        instructions: meal.instructions.clone(),
        ingredients: meal.ingredients.clone(),
        per_user_portions: meal.per_user_portions.clone(),
    };
    validate_meal_inner(&pseudo, req, &ctx, None, meal_type)
}

pub fn validate_meal_options(
    opts: &MealOptions,
    req: &PlanRequest,
    meal_type: &str,
) -> Vec<String> {
    let ctx = Context::new(req);
    let mut out = Vec::new();
    for (idx, opt) in opts.options.iter().enumerate() {
        let label = format!("opción #{} ({})", idx + 1, opt.name);
        let pseudo = PlanMeal {
            meal_type: meal_type.to_string(),
            name: opt.name.clone(),
            instructions: opt.instructions.clone(),
            ingredients: opt.ingredients.clone(),
            per_user_portions: opt.per_user_portions.clone(),
        };
        out.extend(validate_meal_inner(&pseudo, req, &ctx, Some(&label), meal_type));
    }
    out
}

pub fn validate_plan_meal(meal: &PlanMeal, req: &PlanRequest) -> Vec<String> {
    let ctx = Context::new(req);
    validate_meal_inner(meal, req, &ctx, None, &meal.meal_type)
}

// ---------- Internals ----------

struct Context {
    allowed_norm: Vec<String>,
    forbidden_per_user: Vec<(String, Vec<String>)>,
}

impl Context {
    fn new(req: &PlanRequest) -> Self {
        let allowed_norm = req
            .allowed_foods_by_group
            .iter()
            .flat_map(|g| g.foods.iter().map(|f| normalize(f)))
            .collect();
        let forbidden_per_user = req
            .users
            .iter()
            .map(|u| {
                (
                    u.name.clone(),
                    u.forbidden.iter().map(|f| normalize(f)).collect(),
                )
            })
            .collect();
        Self {
            allowed_norm,
            forbidden_per_user,
        }
    }
}

fn validate_meal_inner(
    meal: &PlanMeal,
    req: &PlanRequest,
    ctx: &Context,
    day_label: Option<&str>,
    meal_type: &str,
) -> Vec<String> {
    let mut issues = Vec::new();
    let prefix = match day_label {
        Some(d) => format!("[{d} · {} ({})]", meal.meal_type, meal.name),
        None => format!("[{} ({})]", meal.meal_type, meal.name),
    };

    // 1. Cada ingrediente debe existir en allowed_foods_by_group.
    for ing in &meal.ingredients {
        if !ingredient_is_allowed(&ing.name, &ctx.allowed_norm) {
            issues.push(format!(
                "{prefix} El ingrediente '{}' NO está en allowed_foods_by_group. \
                 Reemplázalo por un alimento que sí aparezca textualmente en la lista.",
                ing.name
            ));
        }
    }

    // 2. Ningún ingrediente puede ser prohibido para ningún usuario.
    for (user_name, forbidden_list) in &ctx.forbidden_per_user {
        for ing in &meal.ingredients {
            let ing_norm = normalize(&ing.name);
            for fb in forbidden_list {
                if name_matches(&ing_norm, fb) {
                    issues.push(format!(
                        "{prefix} El ingrediente '{}' está PROHIBIDO para {}. \
                         Quítalo o reemplázalo por otro alimento permitido.",
                        ing.name, user_name
                    ));
                }
            }
        }
    }

    // 3. per_user_portions debe tener un elemento por cada usuario.
    let users_with_entry: HashSet<&str> = meal
        .per_user_portions
        .iter()
        .map(|p| p.user.as_str())
        .collect();
    for u in &req.users {
        if !users_with_entry.contains(u.name.as_str()) {
            issues.push(format!(
                "{prefix} Falta per_user_portions para '{}'. Debes incluir UN elemento por cada usuario.",
                u.name
            ));
        }
    }

    // 4. Para cada usuario × meal_type, portions_consumed debe coincidir EXACTO.
    for user in &req.users {
        let expected: HashMap<String, f64> = user
            .portions
            .iter()
            .filter(|p| p.meal_type == meal_type)
            .map(|p| (normalize(&p.group), p.portions))
            .collect();

        let Some(pup) = meal
            .per_user_portions
            .iter()
            .find(|p| p.user == user.name)
        else {
            continue;
        };

        let actual: HashMap<String, f64> = pup
            .portions_consumed
            .iter()
            .map(|gp| (normalize(&gp.group), gp.portions))
            .collect();

        // Grupos que la IA reportó pero el usuario no tiene.
        for (group_norm, val) in &actual {
            if !expected.contains_key(group_norm) {
                issues.push(format!(
                    "{prefix} Para '{}' incluiste {} porciones del grupo '{}' pero ese usuario \
                     NO tiene asignaciones de ese grupo en {}. Quítalo de portions_consumed o \
                     ajusta los ingredientes.",
                    user.name, val, group_norm, meal_type
                ));
            }
        }

        // Grupos que el usuario tiene pero la IA no cuadró.
        for (group_norm, exp_val) in &expected {
            let act_val = actual.get(group_norm).copied().unwrap_or(0.0);
            if (exp_val - act_val).abs() > TOLERANCE {
                issues.push(format!(
                    "{prefix} Para '{}' en el grupo '{}' se esperaban {} porciones (según \
                     user.portions), pero portions_consumed dice {}. Ajusta los ingredientes \
                     para ese usuario y corrige portions_consumed a EXACTAMENTE {} porciones.",
                    user.name, group_norm, exp_val, act_val, exp_val
                ));
            }
        }

        // Si notes está vacío, también es un problema.
        if pup.notes.trim().is_empty() {
            issues.push(format!(
                "{prefix} El 'notes' de per_user_portions para '{}' está vacío. Escribe una \
                 explicación breve con cantidades concretas (gramos, tazas, piezas).",
                user.name
            ));
        }
    }

    issues
}

fn ingredient_is_allowed(ingredient: &str, allowed_norm: &[String]) -> bool {
    let n = normalize(ingredient);
    if n.is_empty() {
        return false;
    }
    allowed_norm.iter().any(|a| name_matches(&n, a))
}

/// Match tolerante: coincide si uno contiene al otro (ambos ya normalizados).
/// Así 'pechuga de pollo' matchea 'pollo' y viceversa.
fn name_matches(a: &str, b: &str) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    a.contains(b) || b.contains(a)
}

fn normalize(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' | 'ã' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            _ => c,
        })
        .collect()
}
