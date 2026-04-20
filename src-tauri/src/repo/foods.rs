use rusqlite::params;

use crate::db::models::{Food, FoodGroup};
use crate::db::DbPool;
use crate::error::AppResult;

pub fn groups(pool: &DbPool) -> AppResult<Vec<FoodGroup>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT id, name FROM food_groups ORDER BY sort_order, id")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(FoodGroup {
                id: r.get(0)?,
                name: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn create_group(pool: &DbPool, name: &str) -> AppResult<FoodGroup> {
    let conn = pool.get()?;
    let next_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM food_groups",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    conn.execute(
        "INSERT INTO food_groups (name, sort_order) VALUES (?1, ?2)",
        params![name, next_order],
    )?;
    let id = conn.last_insert_rowid();
    Ok(FoodGroup { id, name: name.to_string() })
}

pub fn delete_group(pool: &DbPool, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM food_groups WHERE id = ?1", [id])?;
    Ok(())
}

pub fn list(pool: &DbPool, user_id: Option<i64>) -> AppResult<Vec<Food>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT f.id, f.group_id, g.name, f.name, f.portion_quantity, f.portion_unit,
                CASE WHEN uff.id IS NOT NULL THEN 1 ELSE 0 END
         FROM foods f
         JOIN food_groups g ON g.id = f.group_id
         LEFT JOIN user_forbidden_foods uff
           ON uff.food_id = f.id AND uff.user_id = ?1
         ORDER BY g.sort_order, g.id, f.sort_order, f.id",
    )?;
    let uid = user_id.unwrap_or(-1);
    let rows = stmt
        .query_map([uid], |r| {
            Ok(Food {
                id: r.get(0)?,
                group_id: r.get(1)?,
                group_name: r.get(2)?,
                name: r.get(3)?,
                portion_quantity: r.get(4)?,
                portion_unit: r.get(5)?,
                forbidden: r.get::<_, i64>(6)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn create(
    pool: &DbPool,
    group_id: i64,
    name: &str,
    portion_quantity: f64,
    portion_unit: &str,
) -> AppResult<i64> {
    let conn = pool.get()?;
    let next_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM foods WHERE group_id = ?1",
            [group_id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    conn.execute(
        "INSERT INTO foods (group_id, name, portion_quantity, portion_unit, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![group_id, name, portion_quantity, portion_unit, next_order],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(
    pool: &DbPool,
    id: i64,
    group_id: i64,
    name: &str,
    portion_quantity: f64,
    portion_unit: &str,
) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute(
        "UPDATE foods SET group_id = ?1, name = ?2, portion_quantity = ?3, portion_unit = ?4 WHERE id = ?5",
        params![group_id, name, portion_quantity, portion_unit, id],
    )?;
    Ok(())
}

pub fn delete(pool: &DbPool, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM foods WHERE id = ?1", [id])?;
    Ok(())
}
