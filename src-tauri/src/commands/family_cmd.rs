use serde::Serialize;
use tauri::State;

use crate::app_state::SharedState;
use crate::db::models::{FamilyPlan, Food};
use crate::error::AppResult;
use crate::repo;
use crate::services::family_compat;

#[tauri::command]
pub fn family_plans_list(state: State<SharedState>) -> AppResult<Vec<FamilyPlan>> {
    repo::family::list(&state.pool)
}

#[tauri::command]
pub fn family_plans_create(
    state: State<SharedState>,
    name: String,
    week_start: String,
    user_ids: Vec<i64>,
) -> AppResult<i64> {
    repo::family::create(&state.pool, &name, &week_start, &user_ids)
}

#[tauri::command]
pub fn family_plans_delete(state: State<SharedState>, id: i64) -> AppResult<()> {
    repo::family::delete(&state.pool, id)
}

#[derive(Serialize)]
pub struct CompatPayload {
    pub allowed: Vec<Food>,
    pub forbidden_by_user: Vec<ForbiddenEntry>,
}

#[derive(Serialize)]
pub struct ForbiddenEntry {
    pub user_id: i64,
    pub foods: Vec<String>,
}

#[tauri::command]
pub fn family_compatibility(
    state: State<SharedState>,
    user_ids: Vec<i64>,
) -> AppResult<CompatPayload> {
    let r = family_compat::overlap(&state.pool, &user_ids)?;
    Ok(CompatPayload {
        allowed: r.allowed,
        forbidden_by_user: r
            .forbidden_by_user
            .into_iter()
            .map(|(user_id, foods)| ForbiddenEntry { user_id, foods })
            .collect(),
    })
}
