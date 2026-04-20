use std::sync::Arc;

use crate::db::DbPool;

pub struct AppState {
    pub pool: DbPool,
}

pub type SharedState = Arc<AppState>;
