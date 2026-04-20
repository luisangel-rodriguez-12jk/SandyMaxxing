use tauri::State;

use crate::ai::plan_generator::PlanResult;
use crate::app_state::SharedState;
use crate::db::models::ShoppingItem;
use crate::error::AppResult;
use crate::services::shopping_list;

#[tauri::command]
pub fn shopping_build(
    state: State<SharedState>,
    user_ids: Vec<i64>,
    plan: Option<PlanResult>,
) -> AppResult<Vec<ShoppingItem>> {
    shopping_list::aggregate(&state.pool, plan.as_ref(), &user_ids)
}
