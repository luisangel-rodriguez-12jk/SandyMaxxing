use tauri::State;

use crate::app_state::SharedState;
use crate::db::models::User;
use crate::error::AppResult;
use crate::repo;

#[tauri::command]
pub fn users_list(state: State<SharedState>) -> AppResult<Vec<User>> {
    repo::users::list(&state.pool)
}

#[tauri::command]
pub fn users_create(
    state: State<SharedState>,
    name: String,
    age: Option<i64>,
    height: Option<f64>,
    sex: Option<String>,
) -> AppResult<User> {
    repo::users::create(&state.pool, &name, age, height, sex)
}

#[tauri::command]
pub fn users_update(
    state: State<SharedState>,
    id: i64,
    name: String,
    age: Option<i64>,
    height: Option<f64>,
    sex: Option<String>,
) -> AppResult<User> {
    repo::users::update(&state.pool, id, &name, age, height, sex)
}

#[tauri::command]
pub fn users_delete(state: State<SharedState>, id: i64) -> AppResult<()> {
    repo::users::delete(&state.pool, id)
}
