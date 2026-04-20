use crate::crypto;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};

const KEY_SALT: &str = "key_salt";
const OPENAI_KEY: &str = "openai_key";

pub fn get_salt(pool: &DbPool) -> AppResult<Vec<u8>> {
    let conn = pool.get()?;
    let existing: Option<Vec<u8>> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            [KEY_SALT],
            |r| r.get::<_, Vec<u8>>(0),
        )
        .ok();
    if let Some(s) = existing {
        return Ok(s);
    }
    let salt = crypto::random_salt().to_vec();
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![KEY_SALT, salt],
    )?;
    Ok(salt)
}

pub fn set_openai_key(pool: &DbPool, key: &str) -> AppResult<()> {
    let salt = get_salt(pool)?;
    let blob = crypto::encrypt(key, &salt)?;
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![OPENAI_KEY, blob],
    )?;
    Ok(())
}

pub fn get_openai_key(pool: &DbPool) -> AppResult<String> {
    let salt = get_salt(pool)?;
    let conn = pool.get()?;
    let blob: Option<Vec<u8>> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            [OPENAI_KEY],
            |r| r.get::<_, Vec<u8>>(0),
        )
        .ok();
    let blob = blob.ok_or(AppError::MissingApiKey)?;
    crypto::decrypt(&blob, &salt)
}

pub fn has_openai_key(pool: &DbPool) -> AppResult<bool> {
    let conn = pool.get()?;
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM app_settings WHERE key = ?1",
        [OPENAI_KEY],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}

pub fn clear_openai_key(pool: &DbPool) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM app_settings WHERE key = ?1", [OPENAI_KEY])?;
    Ok(())
}
