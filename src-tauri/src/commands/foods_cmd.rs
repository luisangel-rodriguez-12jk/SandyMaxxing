use tauri::State;

use crate::app_state::SharedState;
use crate::db::models::{Food, FoodGroup};
use crate::error::AppResult;
use crate::repo;

#[tauri::command]
pub fn food_groups_list(state: State<SharedState>) -> AppResult<Vec<FoodGroup>> {
    repo::foods::groups(&state.pool)
}

#[tauri::command]
pub fn food_groups_create(state: State<SharedState>, name: String) -> AppResult<FoodGroup> {
    repo::foods::create_group(&state.pool, &name)
}

#[tauri::command]
pub fn food_groups_delete(state: State<SharedState>, id: i64) -> AppResult<()> {
    repo::foods::delete_group(&state.pool, id)
}

#[tauri::command]
pub fn foods_list(state: State<SharedState>, user_id: Option<i64>) -> AppResult<Vec<Food>> {
    repo::foods::list(&state.pool, user_id)
}

#[tauri::command]
pub fn foods_create(
    state: State<SharedState>,
    group_id: i64,
    name: String,
    portion_quantity: f64,
    portion_unit: String,
) -> AppResult<i64> {
    repo::foods::create(&state.pool, group_id, &name, portion_quantity, &portion_unit)
}

#[tauri::command]
pub fn foods_update(
    state: State<SharedState>,
    id: i64,
    group_id: i64,
    name: String,
    portion_quantity: f64,
    portion_unit: String,
) -> AppResult<()> {
    repo::foods::update(&state.pool, id, group_id, &name, portion_quantity, &portion_unit)
}

#[tauri::command]
pub fn foods_delete(state: State<SharedState>, id: i64) -> AppResult<()> {
    repo::foods::delete(&state.pool, id)
}

#[tauri::command]
pub fn forbidden_set(
    state: State<SharedState>,
    user_id: i64,
    food_id: i64,
    forbidden: bool,
) -> AppResult<()> {
    repo::forbidden::set(&state.pool, user_id, food_id, forbidden)
}
