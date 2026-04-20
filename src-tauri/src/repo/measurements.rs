use rusqlite::params;

use crate::db::models::Measurement;
use crate::db::DbPool;
use crate::error::AppResult;

pub fn list(pool: &DbPool, user_id: i64) -> AppResult<Vec<Measurement>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, user_id, date, weight, back_cm, waist_cm, abdomen_cm, hip_cm
         FROM body_measurements WHERE user_id = ?1 ORDER BY date ASC",
    )?;
    let rows = stmt
        .query_map([user_id], |r| {
            Ok(Measurement {
                id: r.get(0)?,
                user_id: r.get(1)?,
                date: r.get(2)?,
                weight: r.get(3)?,
                back_cm: r.get(4)?,
                waist_cm: r.get(5)?,
                abdomen_cm: r.get(6)?,
                hip_cm: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[allow(clippy::too_many_arguments)]
pub fn insert(
    pool: &DbPool,
    user_id: i64,
    date: &str,
    weight: Option<f64>,
    back_cm: Option<f64>,
    waist_cm: Option<f64>,
    abdomen_cm: Option<f64>,
    hip_cm: Option<f64>,
) -> AppResult<i64> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO body_measurements
           (user_id, date, weight, back_cm, waist_cm, abdomen_cm, hip_cm)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![user_id, date, weight, back_cm, waist_cm, abdomen_cm, hip_cm],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete(pool: &DbPool, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM body_measurements WHERE id = ?1", [id])?;
    Ok(())
}
