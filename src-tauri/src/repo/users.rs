use rusqlite::params;

use crate::db::models::User;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};

pub fn list(pool: &DbPool) -> AppResult<Vec<User>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT id, name, age, height, sex FROM users ORDER BY name")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(User {
                id: r.get(0)?,
                name: r.get(1)?,
                age: r.get(2)?,
                height: r.get(3)?,
                sex: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn create(
    pool: &DbPool,
    name: &str,
    age: Option<i64>,
    height: Option<f64>,
    sex: Option<String>,
) -> AppResult<User> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO users (name, age, height, sex) VALUES (?1, ?2, ?3, ?4)",
        params![name, age, height, sex],
    )?;
    let id = conn.last_insert_rowid();
    get(pool, id)
}

pub fn update(
    pool: &DbPool,
    id: i64,
    name: &str,
    age: Option<i64>,
    height: Option<f64>,
    sex: Option<String>,
) -> AppResult<User> {
    let conn = pool.get()?;
    conn.execute(
        "UPDATE users SET name = ?1, age = ?2, height = ?3, sex = ?4 WHERE id = ?5",
        params![name, age, height, sex, id],
    )?;
    get(pool, id)
}

pub fn get(pool: &DbPool, id: i64) -> AppResult<User> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT id, name, age, height, sex FROM users WHERE id = ?1",
        [id],
        |r| {
            Ok(User {
                id: r.get(0)?,
                name: r.get(1)?,
                age: r.get(2)?,
                height: r.get(3)?,
                sex: r.get(4)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("usuario {id}")),
        other => other.into(),
    })
}

pub fn delete(pool: &DbPool, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM users WHERE id = ?1", [id])?;
    Ok(())
}
