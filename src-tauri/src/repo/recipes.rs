use rusqlite::params;

use crate::db::models::{Recipe, RecipeIngredient};
use crate::db::DbPool;
use crate::error::AppResult;

/// Lista todas las recetas guardadas (favoritas). Si se pasa `meal_type`,
/// filtra solo las de ese tiempo de comida. Orden: más recientes primero.
pub fn list(pool: &DbPool, meal_type: Option<&str>) -> AppResult<Vec<Recipe>> {
    let conn = pool.get()?;
    let order = " ORDER BY COALESCE(created_at, '') DESC, id DESC";

    let row_mapper = |r: &rusqlite::Row| -> rusqlite::Result<(
        i64,
        String,
        String,
        bool,
        Option<String>,
        Option<String>,
    )> {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)? != 0,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, Option<String>>(5)?,
        ))
    };

    let recipes: Vec<(i64, String, String, bool, Option<String>, Option<String>)> = match meal_type {
        Some(mt) => {
            let sql = format!(
                "SELECT id, name, instructions, created_by_ai, meal_type, created_at
                 FROM recipes WHERE meal_type = ?1{}",
                order
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt
                .query_map(params![mt], row_mapper)?
                .collect::<Result<Vec<_>, _>>()?;
            rows
        }
        None => {
            let sql = format!(
                "SELECT id, name, instructions, created_by_ai, meal_type, created_at
                 FROM recipes{}",
                order
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt
                .query_map([], row_mapper)?
                .collect::<Result<Vec<_>, _>>()?;
            rows
        }
    };

    let mut ing_stmt = conn.prepare(
        "SELECT ri.food_id, ri.free_name, COALESCE(f.name, ri.free_name, ''), ri.quantity, ri.unit
         FROM recipe_ingredients ri
         LEFT JOIN foods f ON f.id = ri.food_id
         WHERE ri.recipe_id = ?1
         ORDER BY ri.id",
    )?;
    let mut out = Vec::new();
    for (id, name, instructions, by_ai, mt, created) in recipes {
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
            meal_type: mt,
            created_at: created,
            ingredients,
        });
    }
    Ok(out)
}

/// Guarda una receta (favorita). Para cada ingrediente intenta resolver el
/// food_id contra la tabla 'foods' por nombre (case-insensitive). Si no
/// encuentra match, lo guarda como free_name.
pub fn save_single(
    pool: &DbPool,
    name: &str,
    instructions: &str,
    meal_type: &str,
    created_by_ai: bool,
    ingredients: &[(String, f64, String)],
) -> AppResult<i64> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO recipes (name, instructions, created_by_ai, meal_type, created_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        params![name, instructions, created_by_ai as i64, meal_type],
    )?;
    let id = tx.last_insert_rowid();
    {
        let mut lookup = tx.prepare(
            "SELECT id FROM foods WHERE LOWER(name) = LOWER(?1) LIMIT 1",
        )?;
        let mut ins = tx.prepare(
            "INSERT INTO recipe_ingredients
               (recipe_id, food_id, free_name, quantity, unit)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for (iname, qty, unit) in ingredients {
            let food_id: Option<i64> = lookup
                .query_row(params![iname], |r| r.get(0))
                .ok();
            let free_name: Option<&str> = if food_id.is_some() { None } else { Some(iname) };
            ins.execute(params![id, food_id, free_name, qty, unit])?;
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
