use rusqlite::params;

use crate::db::models::FamilyPlan;
use crate::db::DbPool;
use crate::error::AppResult;

pub fn list(pool: &DbPool) -> AppResult<Vec<FamilyPlan>> {
    let conn = pool.get()?;
    let mut stmt =
        conn.prepare("SELECT id, name, week_start FROM family_plans ORDER BY week_start DESC")?;
    let plans = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut users_stmt =
        conn.prepare("SELECT user_id FROM family_plan_users WHERE family_plan_id = ?1")?;
    let mut out = Vec::new();
    for (id, name, week_start) in plans {
        let user_ids = users_stmt
            .query_map([id], |r| r.get::<_, i64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        out.push(FamilyPlan {
            id,
            name,
            week_start,
            user_ids,
        });
    }
    Ok(out)
}

pub fn create(
    pool: &DbPool,
    name: &str,
    week_start: &str,
    user_ids: &[i64],
) -> AppResult<i64> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO family_plans (name, week_start) VALUES (?1, ?2)",
        params![name, week_start],
    )?;
    let id = tx.last_insert_rowid();
    {
        let mut ins = tx.prepare(
            "INSERT INTO family_plan_users (family_plan_id, user_id) VALUES (?1, ?2)",
        )?;
        for uid in user_ids {
            ins.execute(params![id, uid])?;
        }
    }
    tx.commit()?;
    Ok(id)
}

pub fn delete(pool: &DbPool, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM family_plans WHERE id = ?1", [id])?;
    Ok(())
}
