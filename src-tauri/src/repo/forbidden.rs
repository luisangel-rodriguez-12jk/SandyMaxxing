use crate::db::DbPool;
use crate::error::AppResult;

pub fn set(pool: &DbPool, user_id: i64, food_id: i64, forbidden: bool) -> AppResult<()> {
    let conn = pool.get()?;
    if forbidden {
        conn.execute(
            "INSERT OR IGNORE INTO user_forbidden_foods (user_id, food_id) VALUES (?1, ?2)",
            [user_id, food_id],
        )?;
    } else {
        conn.execute(
            "DELETE FROM user_forbidden_foods WHERE user_id = ?1 AND food_id = ?2",
            [user_id, food_id],
        )?;
    }
    Ok(())
}

pub fn food_ids(pool: &DbPool, user_id: i64) -> AppResult<Vec<i64>> {
    let conn = pool.get()?;
    let mut stmt =
        conn.prepare("SELECT food_id FROM user_forbidden_foods WHERE user_id = ?1")?;
    let rows = stmt
        .query_map([user_id], |r| r.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn food_names(pool: &DbPool, user_id: i64) -> AppResult<Vec<String>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT f.name FROM user_forbidden_foods uff
         JOIN foods f ON f.id = uff.food_id
         WHERE uff.user_id = ?1",
    )?;
    let rows = stmt
        .query_map([user_id], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
