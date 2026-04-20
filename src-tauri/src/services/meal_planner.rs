use std::collections::BTreeMap;

use chrono::{Datelike, Duration, NaiveDate};

use crate::ai::plan_generator::{
    AllowedGroup, PlanPortion, PlanRequest, PlanSmoothie, PlanUser,
};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::repo;
use crate::services::family_compat;

const WEEKDAYS_ES: [&str; 7] = [
    "Lunes",
    "Martes",
    "Miércoles",
    "Jueves",
    "Viernes",
    "Sábado",
    "Domingo",
];

const MONTHS_ES: [&str; 12] = [
    "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
];

/// Construye un PlanRequest con smoothies estructurados, porciones y
/// (si aplica) las etiquetas de día correspondientes al rango start..end.
/// Pasa `end_date = None` para comandos de una sola comida.
pub fn build_request(
    pool: &DbPool,
    user_ids: &[i64],
    start_date: &str,
    end_date: Option<&str>,
    notes: Option<String>,
) -> AppResult<PlanRequest> {
    let compat = family_compat::overlap(pool, user_ids)?;

    let mut by_group: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for f in &compat.allowed {
        by_group
            .entry(f.group_name.clone())
            .or_default()
            .push(f.name.clone());
    }
    let allowed_foods_by_group = by_group
        .into_iter()
        .map(|(group, foods)| AllowedGroup { group, foods })
        .collect();

    let mut users = Vec::new();
    let mut smoothies: Vec<PlanSmoothie> = Vec::new();
    for uid in user_ids {
        let user = repo::users::get(pool, *uid)?;
        // Primero probamos la dieta de la fecha solicitada.
        let diet = repo::diets::get_or_create(pool, *uid, start_date)?;
        // Si esa dieta está vacía (caso común: el usuario configuró su dieta para una
        // semana distinta y hoy abrió "Cocinar"/"Mi plan" en otra fecha), hacemos
        // fallback a la dieta más reciente que sí tenga porciones asignadas.
        let effective_portions = if diet.portions.is_empty() {
            match repo::diets::get_latest_with_portions(pool, *uid)? {
                Some(fallback) => fallback.portions,
                None => {
                    return Err(AppError::InvalidAi(format!(
                        "El usuario '{}' no tiene ninguna dieta configurada todavía. \
                         Ve a 'Mi plan', selecciona una semana y asigna las porciones \
                         por tiempo de comida antes de generar comidas con IA.",
                        user.name
                    )));
                }
            }
        } else {
            diet.portions
        };
        let portions: Vec<PlanPortion> = effective_portions
            .iter()
            .map(|p| PlanPortion {
                meal_type: p.meal_type.clone(),
                group: p.group_name.clone(),
                portions: p.portions,
            })
            .collect();
        let forbidden = repo::forbidden::food_names(pool, *uid)?;
        let user_name = user.name.clone();
        users.push(PlanUser {
            name: user.name,
            portions,
            forbidden,
        });

        for s in repo::smoothies::list(pool, *uid).unwrap_or_default() {
            smoothies.push(PlanSmoothie {
                user: user_name.clone(),
                meal_type: s.meal_type,
                raw_text: s.raw_text,
            });
        }
    }

    let day_labels = match end_date {
        Some(end) => build_day_labels(start_date, end),
        None => Vec::new(),
    };

    Ok(PlanRequest {
        users,
        allowed_foods_by_group,
        smoothies,
        day_labels,
        notes,
    })
}

/// Verifica que, para un comando de UNA sola comida, todos los usuarios tengan al
/// menos UNA porción asignada para ese meal_type. Así evitamos mandar peticiones a
/// la IA que el validador rechazará seguro (ahorrando tokens y dando un error claro).
pub fn preflight_meal_type(req: &PlanRequest, meal_type: &str) -> AppResult<()> {
    let mut missing: Vec<String> = Vec::new();
    for u in &req.users {
        let has_any = u.portions.iter().any(|p| p.meal_type == meal_type);
        if !has_any {
            missing.push(u.name.clone());
        }
    }
    if !missing.is_empty() {
        let names = missing.join(", ");
        return Err(AppError::InvalidAi(format!(
            "Los siguientes usuarios no tienen porciones configuradas para '{meal_type}': \
             {names}. Ve a 'Mi plan' y asigna sus porciones para ese tiempo de comida antes \
             de generar con IA."
        )));
    }
    Ok(())
}

fn build_day_labels(start_iso: &str, end_iso: &str) -> Vec<String> {
    let start = match NaiveDate::parse_from_str(start_iso, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let end = match NaiveDate::parse_from_str(end_iso, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    if end < start {
        return Vec::new();
    }
    let span_days = (end - start).num_days();
    // Limitamos a 31 días para evitar explosiones en el prompt.
    let total = (span_days + 1).min(31);
    let mut out = Vec::with_capacity(total as usize);
    for i in 0..total {
        let d = start + Duration::days(i);
        let weekday = d.weekday().num_days_from_monday() as usize;
        let name = WEEKDAYS_ES[weekday];
        let day_num = d.day();
        let month_idx = (d.month() as usize).saturating_sub(1).min(11);
        let month_name = MONTHS_ES[month_idx];
        out.push(format!("{name} {day_num} {month_name}"));
    }
    out
}
