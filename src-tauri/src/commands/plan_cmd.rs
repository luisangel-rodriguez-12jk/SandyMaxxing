use tauri::State;

use crate::ai::plan_generator::{
    self, AllowedGroup, MealOptions, PlanMeal, PlanRequest, PlanResult, PlanUser, SingleMeal,
};
use crate::app_state::SharedState;
use crate::db::models::SavedPlan;
use crate::error::AppResult;
use crate::repo;
use crate::services::meal_planner;

#[tauri::command]
pub async fn plan_generate(
    state: State<'_, SharedState>,
    user_ids: Vec<i64>,
    week_start: String,
    end_date: Option<String>,
    notes: Option<String>,
) -> AppResult<PlanResult> {
    let api_key = repo::settings::get_openai_key(&state.pool)?;
    // Si el frontend no mandó end_date, asumimos semana (start + 6 días).
    let end = end_date.unwrap_or_else(|| default_end_date(&week_start));
    let req = meal_planner::build_request(
        &state.pool,
        &user_ids,
        &week_start,
        Some(end.as_str()),
        notes,
    )?;
    plan_generator::generate(&api_key, &req).await
}

#[tauri::command]
pub async fn meal_design(
    state: State<'_, SharedState>,
    user_ids: Vec<i64>,
    week_start: String,
    notes: Option<String>,
) -> AppResult<SingleMeal> {
    let api_key = repo::settings::get_openai_key(&state.pool)?;
    let req = meal_planner::build_request(&state.pool, &user_ids, &week_start, None, notes)?;
    plan_generator::generate_single_meal(&api_key, &req).await
}

#[tauri::command]
pub async fn meal_options(
    state: State<'_, SharedState>,
    user_ids: Vec<i64>,
    week_start: String,
    notes: Option<String>,
    meal_type: String,
    count: u32,
    exclude_names: Vec<String>,
) -> AppResult<MealOptions> {
    let api_key = repo::settings::get_openai_key(&state.pool)?;
    let req = meal_planner::build_request(&state.pool, &user_ids, &week_start, None, notes)?;
    plan_generator::generate_meal_options(&api_key, &req, &meal_type, count, &exclude_names).await
}

#[tauri::command]
pub async fn plan_tweak_meal(
    state: State<'_, SharedState>,
    user_ids: Vec<i64>,
    week_start: String,
    day: String,
    original: PlanMeal,
    user_instruction: String,
) -> AppResult<PlanMeal> {
    let api_key = repo::settings::get_openai_key(&state.pool)?;
    let req = meal_planner::build_request(&state.pool, &user_ids, &week_start, None, None)?;
    // Extraemos componentes relevantes para el tweak.
    let users: Vec<PlanUser> = req.users;
    let allowed: Vec<AllowedGroup> = req.allowed_foods_by_group;
    plan_generator::tweak_meal(&api_key, &users, &allowed, &original, &user_instruction, &day).await
}

fn default_end_date(start: &str) -> String {
    use chrono::{Duration, NaiveDate};
    match NaiveDate::parse_from_str(start, "%Y-%m-%d") {
        Ok(d) => (d + Duration::days(6))
            .format("%Y-%m-%d")
            .to_string(),
        Err(_) => start.to_string(),
    }
}

// ---- Planes guardados ----

#[tauri::command]
pub fn saved_plans_list(state: State<SharedState>) -> AppResult<Vec<SavedPlan>> {
    repo::saved_plans::list(&state.pool)
}

#[tauri::command]
pub fn saved_plans_get(state: State<SharedState>, id: i64) -> AppResult<SavedPlan> {
    repo::saved_plans::get(&state.pool, id)
}

#[tauri::command]
pub fn saved_plans_upsert(
    state: State<SharedState>,
    id: Option<i64>,
    name: String,
    week_start: String,
    user_ids: Vec<i64>,
    plan: PlanResult,
    notes: Option<String>,
) -> AppResult<i64> {
    let user_ids_json = serde_json::to_string(&user_ids)?;
    let plan_json = serde_json::to_string(&plan)?;
    repo::saved_plans::upsert(
        &state.pool,
        id,
        &name,
        &week_start,
        &user_ids_json,
        &plan_json,
        notes.as_deref(),
    )
}

#[tauri::command]
pub fn saved_plans_delete(state: State<SharedState>, id: i64) -> AppResult<()> {
    repo::saved_plans::delete(&state.pool, id)
}
