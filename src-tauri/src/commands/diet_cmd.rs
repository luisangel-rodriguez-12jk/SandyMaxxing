use tauri::State;

use crate::app_state::SharedState;
use crate::db::models::WeeklyDiet;
use crate::error::AppResult;
use crate::repo;

#[tauri::command]
pub fn diet_get(
    state: State<SharedState>,
    user_id: i64,
    week_start: String,
) -> AppResult<WeeklyDiet> {
    repo::diets::get_or_create(&state.pool, user_id, &week_start)
}

#[tauri::command]
pub fn diet_set_portion(
    state: State<SharedState>,
    diet_id: i64,
    meal_type: String,
    group_id: i64,
    portions: f64,
) -> AppResult<()> {
    repo::diets::set_portion(&state.pool, diet_id, &meal_type, group_id, portions)
}
