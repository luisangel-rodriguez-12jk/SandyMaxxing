use rusqlite::params;

use crate::db::models::{ParsedSmoothie, Smoothie};
use crate::db::DbPool;
use crate::error::AppResult;

pub fn list(pool: &DbPool, user_id: i64) -> AppResult<Vec<Smoothie>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, user_id, meal_type, raw_text, parsed_json
         FROM smoothies WHERE user_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map([user_id], |r| {
            let parsed_json: Option<String> = r.get(4)?;
            let parsed = parsed_json.and_then(|s| serde_json::from_str::<ParsedSmoothie>(&s).ok());
            Ok(Smoothie {
                id: r.get(0)?,
                user_id: r.get(1)?,
                meal_type: r.get(2)?,
                raw_text: r.get(3)?,
                parsed,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn insert(
    pool: &DbPool,
    user_id: i64,
    meal_type: &str,
    raw_text: &str,
    parsed: &ParsedSmoothie,
) -> AppResult<i64> {
    let conn = pool.get()?;
    let parsed_json = serde_json::to_string(parsed)?;
    conn.execute(
        "INSERT INTO smoothies (user_id, meal_type, raw_text, parsed_json)
         VALUES (?1, ?2, ?3, ?4)",
        params![user_id, meal_type, raw_text, parsed_json],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete(pool: &DbPool, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM smoothies WHERE id = ?1", [id])?;
    Ok(())
}
