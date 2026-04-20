use tauri::State;

use crate::ai::plan_generator::PlanResult;
use crate::app_state::SharedState;
use crate::db::models::ShoppingItem;
use crate::error::AppResult;
use crate::repo;
use crate::services::pdf_export;

#[tauri::command]
pub fn pdf_plan(plan: PlanResult, title: String) -> AppResult<Vec<u8>> {
    pdf_export::plan_to_pdf(&plan, &title)
}

#[tauri::command]
pub fn pdf_shopping(items: Vec<ShoppingItem>, title: String) -> AppResult<Vec<u8>> {
    pdf_export::shopping_to_pdf(&items, &title)
}

#[tauri::command]
pub fn pdf_measurements(state: State<SharedState>, user_id: i64) -> AppResult<Vec<u8>> {
    let user = repo::users::get(&state.pool, user_id)?;
    let list = repo::measurements::list(&state.pool, user_id)?;
    pdf_export::measurements_to_pdf(&user.name, &list)
}
