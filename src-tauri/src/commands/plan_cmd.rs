use tauri::{AppHandle, Emitter, State};

use crate::ai::plan_generator::{
    self, AllowedGroup, MealOptions, PlanMeal, PlanResult, PlanUser, SingleMeal,
};
use crate::app_state::SharedState;
use crate::db::models::SavedPlan;
use crate::error::{AppError, AppResult};
use crate::repo;
use crate::services::meal_planner;

/// Pequeño helper para emitir eventos "start" / "done" / "error" alrededor de
/// la llamada a la IA, de modo que el frontend pueda mostrar una barra de
/// progreso informativa mientras la generación se ejecuta. Los eventos
/// intermedios ('requesting', 'validating', 'retrying') los emite el propio
/// plan_generator durante el ciclo de chat + validación.
fn emit_start(app: &AppHandle, label: &str) {
    let _ = app.emit(
        "ai_progress",
        serde_json::json!({
            "stage": "start",
            "label": label,
        }),
    );
}

fn emit_done(app: &AppHandle, label: &str) {
    let _ = app.emit(
        "ai_progress",
        serde_json::json!({
            "stage": "done",
            "label": label,
        }),
    );
}

fn emit_error(app: &AppHandle, label: &str, err: &AppError) {
    let _ = app.emit(
        "ai_progress",
        serde_json::json!({
            "stage": "error",
            "label": label,
            "message": err.to_string(),
        }),
    );
}

#[tauri::command]
pub async fn plan_generate(
    app: AppHandle,
    state: State<'_, SharedState>,
    user_ids: Vec<i64>,
    week_start: String,
    end_date: Option<String>,
    notes: Option<String>,
) -> AppResult<PlanResult> {
    let label = "plan semanal";
    emit_start(&app, label);
    let api_key = match repo::settings::get_openai_key(&state.pool) {
        Ok(k) => k,
        Err(e) => {
            emit_error(&app, label, &e);
            return Err(e);
        }
    };
    let end = end_date.unwrap_or_else(|| default_end_date(&week_start));
    let req = match meal_planner::build_request(
        &state.pool,
        &user_ids,
        &week_start,
        Some(end.as_str()),
        notes,
    ) {
        Ok(r) => r,
        Err(e) => {
            emit_error(&app, label, &e);
            return Err(e);
        }
    };
    match plan_generator::generate(&app, &api_key, &req).await {
        Ok(plan) => {
            emit_done(&app, label);
            Ok(plan)
        }
        Err(e) => {
            emit_error(&app, label, &e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn meal_design(
    app: AppHandle,
    state: State<'_, SharedState>,
    user_ids: Vec<i64>,
    week_start: String,
    notes: Option<String>,
    meal_type: Option<String>,
) -> AppResult<SingleMeal> {
    let label = "comida individual";
    emit_start(&app, label);
    let api_key = match repo::settings::get_openai_key(&state.pool) {
        Ok(k) => k,
        Err(e) => {
            emit_error(&app, label, &e);
            return Err(e);
        }
    };
    let req = match meal_planner::build_request(&state.pool, &user_ids, &week_start, None, notes) {
        Ok(r) => r,
        Err(e) => {
            emit_error(&app, label, &e);
            return Err(e);
        }
    };
    let meal_type = meal_type.unwrap_or_else(|| "comida".to_string());
    if let Err(e) = meal_planner::preflight_meal_type(&req, &meal_type) {
        emit_error(&app, label, &e);
        return Err(e);
    }
    match plan_generator::generate_single_meal(&app, &api_key, &req, &meal_type).await {
        Ok(meal) => {
            emit_done(&app, label);
            Ok(meal)
        }
        Err(e) => {
            emit_error(&app, label, &e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn meal_options(
    app: AppHandle,
    state: State<'_, SharedState>,
    user_ids: Vec<i64>,
    week_start: String,
    notes: Option<String>,
    meal_type: String,
    count: u32,
    exclude_names: Vec<String>,
) -> AppResult<MealOptions> {
    let label = "opciones de comida";
    emit_start(&app, label);
    let api_key = match repo::settings::get_openai_key(&state.pool) {
        Ok(k) => k,
        Err(e) => {
            emit_error(&app, label, &e);
            return Err(e);
        }
    };
    let req = match meal_planner::build_request(&state.pool, &user_ids, &week_start, None, notes) {
        Ok(r) => r,
        Err(e) => {
            emit_error(&app, label, &e);
            return Err(e);
        }
    };
    if let Err(e) = meal_planner::preflight_meal_type(&req, &meal_type) {
        emit_error(&app, label, &e);
        return Err(e);
    }
    match plan_generator::generate_meal_options(
        &app,
        &api_key,
        &req,
        &meal_type,
        count,
        &exclude_names,
    )
    .await
    {
        Ok(opts) => {
            emit_done(&app, label);
            Ok(opts)
        }
        Err(e) => {
            emit_error(&app, label, &e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn plan_tweak_meal(
    app: AppHandle,
    state: State<'_, SharedState>,
    user_ids: Vec<i64>,
    week_start: String,
    day: String,
    original: PlanMeal,
    user_instruction: String,
) -> AppResult<PlanMeal> {
    let label = "ajuste de comida";
    emit_start(&app, label);
    let api_key = match repo::settings::get_openai_key(&state.pool) {
        Ok(k) => k,
        Err(e) => {
            emit_error(&app, label, &e);
            return Err(e);
        }
    };
    let req = match meal_planner::build_request(&state.pool, &user_ids, &week_start, None, None) {
        Ok(r) => r,
        Err(e) => {
            emit_error(&app, label, &e);
            return Err(e);
        }
    };
    if let Err(e) = meal_planner::preflight_meal_type(&req, &original.meal_type) {
        emit_error(&app, label, &e);
        return Err(e);
    }
    let users: Vec<PlanUser> = req.users;
    let allowed: Vec<AllowedGroup> = req.allowed_foods_by_group;
    match plan_generator::tweak_meal(
        &app,
        &api_key,
        &users,
        &allowed,
        &original,
        &user_instruction,
        &day,
    )
    .await
    {
        Ok(meal) => {
            emit_done(&app, label);
            Ok(meal)
        }
        Err(e) => {
            emit_error(&app, label, &e);
            Err(e)
        }
    }
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
