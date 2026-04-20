use std::collections::HashMap;

use crate::ai::plan_generator::PlanResult;
use crate::db::models::{ParsedSmoothie, ShoppingItem};
use crate::db::DbPool;
use crate::error::AppResult;
use crate::repo;

fn normalize_unit(unit: &str) -> String {
    let u = unit.trim().to_lowercase();
    match u.as_str() {
        "tza" | "taza" | "tazas" => "taza".into(),
        "cda" | "cdas" | "cucharada" | "cucharadas" => "cda".into(),
        "cdta" | "cdtas" | "cucharadita" | "cucharaditas" => "cdta".into(),
        "pz" | "pza" | "pzs" | "pzas" | "pieza" | "piezas" => "pieza".into(),
        "g" | "gr" | "gramos" | "gramo" => "g".into(),
        "ml" | "mililitros" => "ml".into(),
        "reb" | "rebanada" | "rebanadas" => "reb".into(),
        other => other.to_string(),
    }
}

pub fn aggregate(
    pool: &DbPool,
    plan: Option<&PlanResult>,
    user_ids: &[i64],
) -> AppResult<Vec<ShoppingItem>> {
    let mut totals: HashMap<(String, String), f64> = HashMap::new();

    if let Some(plan) = plan {
        for day in &plan.days {
            for meal in &day.meals {
                for ing in &meal.ingredients {
                    let key = (ing.name.to_lowercase(), normalize_unit(&ing.unit));
                    *totals.entry(key).or_insert(0.0) += ing.quantity;
                }
            }
        }
    }

    for uid in user_ids {
        let smoothies = repo::smoothies::list(pool, *uid)?;
        for s in smoothies {
            if let Some(ParsedSmoothie { ingredients }) = s.parsed {
                for ing in ingredients {
                    let key = (ing.name.to_lowercase(), normalize_unit(&ing.unit));
                    *totals.entry(key).or_insert(0.0) += ing.quantity * 7.0;
                }
            }
        }
    }

    let all_foods = repo::foods::list(pool, None)?;
    let group_lookup: HashMap<String, String> = all_foods
        .iter()
        .map(|f| (f.name.to_lowercase(), f.group_name.clone()))
        .collect();

    let mut out: Vec<ShoppingItem> = totals
        .into_iter()
        .map(|((name, unit), quantity)| {
            let group_name = group_lookup
                .get(&name)
                .cloned()
                .unwrap_or_else(|| "Otros".into());
            ShoppingItem {
                name,
                group_name,
                quantity,
                unit,
            }
        })
        .collect();
    out.sort_by(|a, b| a.group_name.cmp(&b.group_name).then(a.name.cmp(&b.name)));
    Ok(out)
}
