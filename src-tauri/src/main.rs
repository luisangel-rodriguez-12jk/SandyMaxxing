#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use directories::ProjectDirs;

mod app_state;
mod ai;
mod commands;
mod crypto;
mod db;
mod error;
mod repo;
mod services;

use app_state::{AppState, SharedState};

fn main() {
    tracing_subscriber::fmt::init();

    let dirs = ProjectDirs::from("com", "sandymaxxing", "SandyMaxxing")
        .expect("no se pudo obtener el directorio de datos");
    let db_path = dirs.data_dir().join("sandymaxxing.sqlite");

    let pool = db::open_pool(db_path).expect("no se pudo abrir la base de datos");
    let state: SharedState = Arc::new(AppState { pool });

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::users_cmd::users_list,
            commands::users_cmd::users_create,
            commands::users_cmd::users_update,
            commands::users_cmd::users_delete,
            commands::measurements_cmd::measurements_list,
            commands::measurements_cmd::measurements_add,
            commands::measurements_cmd::measurements_delete,
            commands::foods_cmd::food_groups_list,
            commands::foods_cmd::food_groups_create,
            commands::foods_cmd::food_groups_delete,
            commands::foods_cmd::foods_list,
            commands::foods_cmd::foods_create,
            commands::foods_cmd::foods_update,
            commands::foods_cmd::foods_delete,
            commands::foods_cmd::forbidden_set,
            commands::diet_cmd::diet_get,
            commands::diet_cmd::diet_set_portion,
            commands::smoothie_cmd::smoothies_list,
            commands::smoothie_cmd::smoothie_parse_and_save,
            commands::smoothie_cmd::smoothie_save_manual,
            commands::smoothie_cmd::smoothie_delete,
            commands::plan_cmd::plan_generate,
            commands::plan_cmd::meal_design,
            commands::plan_cmd::meal_options,
            commands::plan_cmd::plan_tweak_meal,
            commands::plan_cmd::saved_plans_list,
            commands::plan_cmd::saved_plans_get,
            commands::plan_cmd::saved_plans_upsert,
            commands::plan_cmd::saved_plans_delete,
            commands::family_cmd::family_plans_list,
            commands::family_cmd::family_plans_create,
            commands::family_cmd::family_plans_delete,
            commands::family_cmd::family_compatibility,
            commands::shopping_cmd::shopping_build,
            commands::pdf_cmd::pdf_plan,
            commands::pdf_cmd::pdf_shopping,
            commands::pdf_cmd::pdf_measurements,
            commands::settings_cmd::settings_set_openai_key,
            commands::settings_cmd::settings_has_openai_key,
            commands::settings_cmd::settings_clear_openai_key,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar SandyMaxxing");
}
