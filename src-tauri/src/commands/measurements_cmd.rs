use tauri::State;

use crate::app_state::SharedState;
use crate::db::models::Measurement;
use crate::error::AppResult;
use crate::repo;

#[tauri::command]
pub fn measurements_list(state: State<SharedState>, user_id: i64) -> AppResult<Vec<Measurement>> {
    repo::measurements::list(&state.pool, user_id)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn measurements_add(
    state: State<SharedState>,
    user_id: i64,
    date: String,
    weight: Option<f64>,
    back_cm: Option<f64>,
    waist_cm: Option<f64>,
    abdomen_cm: Option<f64>,
    hip_cm: Option<f64>,
) -> AppResult<i64> {
    repo::measurements::insert(
        &state.pool, user_id, &date, weight, back_cm, waist_cm, abdomen_cm, hip_cm,
    )
}

#[tauri::command]
pub fn measurements_delete(state: State<SharedState>, id: i64) -> AppResult<()> {
    repo::measurements::delete(&state.pool, id)
}
