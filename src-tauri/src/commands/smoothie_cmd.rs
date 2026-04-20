use tauri::State;

use crate::ai::smoothie_parser;
use crate::app_state::SharedState;
use crate::db::models::{ParsedSmoothie, Smoothie};
use crate::error::AppResult;
use crate::repo;

#[tauri::command]
pub fn smoothies_list(state: State<SharedState>, user_id: i64) -> AppResult<Vec<Smoothie>> {
    repo::smoothies::list(&state.pool, user_id)
}

#[tauri::command]
pub async fn smoothie_parse_and_save(
    state: State<'_, SharedState>,
    user_id: i64,
    meal_type: String,
    raw_text: String,
) -> AppResult<Smoothie> {
    let api_key = repo::settings::get_openai_key(&state.pool)?;
    let parsed = smoothie_parser::parse(&api_key, &raw_text).await?;
    let id = repo::smoothies::insert(&state.pool, user_id, &meal_type, &raw_text, &parsed)?;
    Ok(Smoothie {
        id,
        user_id,
        meal_type,
        raw_text,
        parsed: Some(parsed),
    })
}

#[tauri::command]
pub fn smoothie_save_manual(
    state: State<SharedState>,
    user_id: i64,
    meal_type: String,
    raw_text: String,
    parsed: ParsedSmoothie,
) -> AppResult<i64> {
    repo::smoothies::insert(&state.pool, user_id, &meal_type, &raw_text, &parsed)
}

#[tauri::command]
pub fn smoothie_delete(state: State<SharedState>, id: i64) -> AppResult<()> {
    repo::smoothies::delete(&state.pool, id)
}
