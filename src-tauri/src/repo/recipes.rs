use rusqlite::params;

use crate::db::models::{Recipe, RecipeIngredient};
use crate::db::DbPool;
use crate::error::AppResult;

pub fn list(pool: &DbPool) -> AppResult<Vec<Recipe>> {
    let conn = pool.get()?;
    let mut stmt =
        conn.prepare("SELECT id, name, instructions, created_by_ai FROM recipes ORDER BY id DESC")?;
    let recipes = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)? != 0,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut ing_stmt = conn.prepare(
        "SELECT ri.food_id, ri.free_name, COALESCE(f.name, ri.free_name, ''), ri.quantity, ri.unit
         FROM recipe_ingredients ri
         LEFT JOIN foods f ON f.id = ri.food_id
         WHERE ri.recipe_id = ?1",
    )?;
    let mut out = Vec::new();
    for (id, name, instructions, by_ai) in recipes {
        let ingredients = ing_stmt
            .query_map([id], |r| {
                Ok(RecipeIngredient {
                    food_id: r.get(0)?,
                    free_name: r.get(1)?,
                    name: r.get(2)?,
                    quantity: r.get(3)?,
                    unit: r.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        out.push(Recipe {
            id,
            name,
            instructions,
            created_by_ai: by_ai,
            ingredients,
        });
    }
    Ok(out)
}

pub fn create(
    pool: &DbPool,
    name: &str,
    instructions: &str,
    created_by_ai: bool,
    ingredients: &[RecipeIngredient],
) -> AppResult<i64> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO recipes (name, instructions, created_by_ai) VALUES (?1, ?2, ?3)",
        params![name, instructions, created_by_ai as i64],
    )?;
    let id = tx.last_insert_rowid();
    {
        let mut ins = tx.prepare(
            "INSERT INTO recipe_ingredients
               (recipe_id, food_id, free_name, quantity, unit)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for i in ingredients {
            ins.execute(params![id, i.food_id, i.free_name, i.quantity, i.unit])?;
        }
    }
    tx.commit()?;
    Ok(id)
}

pub fn delete(pool: &DbPool, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM recipes WHERE id = ?1", [id])?;
    Ok(())
}
