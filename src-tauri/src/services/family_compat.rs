use std::collections::HashSet;

use crate::db::models::Food;
use crate::db::DbPool;
use crate::error::AppResult;
use crate::repo;

pub struct CompatResult {
    pub allowed: Vec<Food>,
    pub forbidden_by_user: Vec<(i64, Vec<String>)>,
}

pub fn overlap(pool: &DbPool, user_ids: &[i64]) -> AppResult<CompatResult> {
    let all_foods = repo::foods::list(pool, None)?;
    let mut union_forbidden: HashSet<i64> = HashSet::new();
    let mut per_user = Vec::new();
    for uid in user_ids {
        let ids = repo::forbidden::food_ids(pool, *uid)?;
        let names = repo::forbidden::food_names(pool, *uid)?;
        for id in &ids {
            union_forbidden.insert(*id);
        }
        per_user.push((*uid, names));
    }
    let allowed = all_foods
        .into_iter()
        .filter(|f| !union_forbidden.contains(&f.id))
        .collect();
    Ok(CompatResult {
        allowed,
        forbidden_by_user: per_user,
    })
}
