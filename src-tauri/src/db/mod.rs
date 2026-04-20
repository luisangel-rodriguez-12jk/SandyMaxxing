pub mod migrations;
pub mod models;

use std::path::PathBuf;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::error::AppResult;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn open_pool(db_path: PathBuf) -> AppResult<DbPool> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let manager = SqliteConnectionManager::file(db_path).with_init(|c| {
        c.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
    });
    let pool = Pool::builder().max_size(8).build(manager)?;
    {
        let conn = pool.get()?;
        migrations::run(&conn)?;
    }
    Ok(pool)
}
