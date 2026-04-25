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
    /// meal_types para los que AL MENOS UN usuario tiene un grupo relajado.
    /// En esos casos no exigimos que cada ingrediente esté en el catálogo
    /// (solo que no coincida con ningún forbidden). Esto permite a la IA
    /// improvisar un alimento genérico cuando toda la familia tiene prohibido
    /// el grupo (p. ej. Azúcares) pero el usuario sí tiene porción asignada.
    relaxed_meal_types: HashSet<String>,
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
        let relaxed_meal_types: HashSet<String> = req
            .users
            .iter()
            .flat_map(|u| u.relaxed_groups.iter().map(|r| r.meal_type.clone()))
            .collect();
        Self {
            allowed_norm,
            forbidden_per_user,
            relaxed_meal_types,
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
    //    Excepción: si el meal_type tiene algún grupo relajado (porque un
    //    usuario tiene porciones en un grupo donde toda la familia tiene
    //    todo prohibido), la IA improvisa alimentos genéricos y no exigimos
    //    que estén en el catálogo. La regla de "no prohibidos" sigue viva.
    let strict_allowed_check = !ctx.relaxed_meal_types.contains(meal_type);
    if strict_allowed_check {
        for ing in &meal.ingredients {
            if !ingredient_is_allowed(&ing.name, &ctx.allowed_norm) {
                issues.push(format!(
                    "{prefix} El ingrediente '{}' NO está en allowed_foods_by_group. \
                     Reemplázalo por un alimento que sí aparezca textualmente en la lista.",
                    ing.name
                ));
            }
        }
    }

    // 2. Ningún ingrediente puede ser prohibido para ningún usuario.
    //    Usamos un match ASIMÉTRICO por tokens: los tokens del alimento prohibido
    //    deben estar TODOS presentes como palabras en el ingrediente. Así evitamos
    //    falsos positivos: si el usuario tiene "Costilla de res" prohibida pero "Res"
    //    permitida, un ingrediente llamado "Res" NO debe marcarse como prohibido.
    for (user_name, forbidden_list) in &ctx.forbidden_per_user {
        for ing in &meal.ingredients {
            let ing_norm = normalize(&ing.name);
            for fb in forbidden_list {
                if forbidden_matches(&ing_norm, fb) {
                    issues.push(format!(
                        "{prefix} El ingrediente '{}' está PROHIBIDO para {} (coincide con \
                         '{}'). Quítalo o reemplázalo por otro alimento permitido.",
                        ing.name, user_name, fb
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
    allowed_norm.iter().any(|a| allowed_matches(&n, a))
}

/// Match BIDIRECCIONAL por tokens usado para validar si un ingrediente figura en
/// la lista de permitidos. Acepta tanto 'Pechuga de pollo' (ingrediente más
/// específico) cuando 'Pollo' está en el catálogo como 'Pollo' cuando en el
/// catálogo aparece 'Pechuga de pollo'. Trabaja por palabras completas para
/// evitar que 'Fresa' coincida con 'Res'.
fn allowed_matches(ingredient_norm: &str, allowed_norm: &str) -> bool {
    if ingredient_norm.is_empty() || allowed_norm.is_empty() {
        return false;
    }
    let ing_tokens: HashSet<&str> = ingredient_norm.split_whitespace().collect();
    let al_tokens: HashSet<&str> = allowed_norm.split_whitespace().collect();
    if ing_tokens.is_empty() || al_tokens.is_empty() {
        return false;
    }
    ing_tokens.is_subset(&al_tokens) || al_tokens.is_subset(&ing_tokens)
}

/// Match ASIMÉTRICO por tokens usado para la lista de prohibidos. La entrada
/// prohibida (p. ej. "Costilla de res") coincide con un ingrediente SOLO si
/// TODAS sus palabras aparecen en el ingrediente. De esa forma:
///   - prohibido="Res" e ingrediente="Costilla de res" → coincide ✓
///   - prohibido="Costilla de res" e ingrediente="Res" → NO coincide ✓
///   - prohibido="Res" e ingrediente="Fresa" → NO coincide ✓ (tokens distintos)
pub(crate) fn forbidden_matches(ingredient_norm: &str, forbidden_norm: &str) -> bool {
    if ingredient_norm.is_empty() || forbidden_norm.is_empty() {
        return false;
    }
    let ing_tokens: HashSet<&str> = ingredient_norm.split_whitespace().collect();
    let fb_tokens: Vec<&str> = forbidden_norm.split_whitespace().collect();
    if fb_tokens.is_empty() {
        return false;
    }
    fb_tokens.iter().all(|t| ing_tokens.contains(t))
}

pub(crate) fn normalize(s: &str) -> String {
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
