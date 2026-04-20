use rusqlite::params;

use crate::db::models::SavedPlan;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};

pub fn list(pool: &DbPool) -> AppResult<Vec<SavedPlan>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, week_start, user_ids_json, plan_json, notes, created_at
         FROM saved_plans ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SavedPlan {
                id: r.get(0)?,
                name: r.get(1)?,
                week_start: r.get(2)?,
                user_ids_json: r.get(3)?,
                plan_json: r.get(4)?,
                notes: r.get(5)?,
                created_at: r.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(pool: &DbPool, id: i64) -> AppResult<SavedPlan> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT id, name, week_start, user_ids_json, plan_json, notes, created_at
         FROM saved_plans WHERE id = ?1",
        [id],
        |r| {
            Ok(SavedPlan {
                id: r.get(0)?,
                name: r.get(1)?,
                week_start: r.get(2)?,
                user_ids_json: r.get(3)?,
                plan_json: r.get(4)?,
                notes: r.get(5)?,
                created_at: r.get(6)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("plan {id}")),
        other => AppError::Db(other),
    })
}

pub fn upsert(
    pool: &DbPool,
    id: Option<i64>,
    name: &str,
    week_start: &str,
    user_ids_json: &str,
    plan_json: &str,
    notes: Option<&str>,
) -> AppResult<i64> {
    let conn = pool.get()?;
    if let Some(plan_id) = id {
        conn.execute(
            "UPDATE saved_plans
             SET name = ?1, week_start = ?2, user_ids_json = ?3, plan_json = ?4, notes = ?5
             WHERE id = ?6",
            params![name, week_start, user_ids_json, plan_json, notes, plan_id],
        )?;
        Ok(plan_id)
    } else {
        conn.execute(
            "INSERT INTO saved_plans (name, week_start, user_ids_json, plan_json, notes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, week_start, user_ids_json, plan_json, notes],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

pub fn delete(pool: &DbPool, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM saved_plans WHERE id = ?1", [id])?;
    Ok(())
}
