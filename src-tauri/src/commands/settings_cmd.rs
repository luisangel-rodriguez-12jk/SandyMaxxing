use tauri::State;

use crate::app_state::SharedState;
use crate::error::AppResult;
use crate::repo;

#[tauri::command]
pub fn settings_set_openai_key(state: State<SharedState>, key: String) -> AppResult<()> {
    repo::settings::set_openai_key(&state.pool, &key)
}

#[tauri::command]
pub fn settings_has_openai_key(state: State<SharedState>) -> AppResult<bool> {
    repo::settings::has_openai_key(&state.pool)
}

#[tauri::command]
pub fn settings_clear_openai_key(state: State<SharedState>) -> AppResult<()> {
    repo::settings::clear_openai_key(&state.pool)
}
