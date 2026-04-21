use serde::Deserialize;
use tauri::State;

use crate::app_state::SharedState;
use crate::db::models::Recipe;
use crate::error::AppResult;
use crate::repo;

/// Ingrediente tal cual lo manda el frontend (nombre + cantidad + unidad).
/// En el backend se intenta resolver el `food_id` contra la tabla foods; si
/// no encuentra match, se guarda como free_name.
#[derive(Debug, Deserialize)]
pub struct RecipeIngredientInput {
    pub name: String,
    pub quantity: f64,
    pub unit: String,
}

#[tauri::command]
pub fn recipes_list(
    state: State<SharedState>,
    meal_type: Option<String>,
) -> AppResult<Vec<Recipe>> {
    repo::recipes::list(&state.pool, meal_type.as_deref())
}

#[tauri::command]
pub fn recipes_save(
    state: State<SharedState>,
    name: String,
    instructions: String,
    meal_type: String,
    ingredients: Vec<RecipeIngredientInput>,
    created_by_ai: Option<bool>,
) -> AppResult<i64> {
    let tuples: Vec<(String, f64, String)> = ingredients
        .into_iter()
        .map(|i| (i.name, i.quantity, i.unit))
        .collect();
    repo::recipes::save_single(
        &state.pool,
        &name,
        &instructions,
        &meal_type,
        created_by_ai.unwrap_or(true),
        &tuples,
    )
}

#[tauri::command]
pub fn recipes_delete(state: State<SharedState>, id: i64) -> AppResult<()> {
    repo::recipes::delete(&state.pool, id)
}
