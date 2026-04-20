use rusqlite::params;

use crate::db::models::{DietPortion, WeeklyDiet};
use crate::db::DbPool;
use crate::error::AppResult;

pub fn get_or_create(pool: &DbPool, user_id: i64, week_start: &str) -> AppResult<WeeklyDiet> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT OR IGNORE INTO weekly_diets (user_id, week_start) VALUES (?1, ?2)",
        params![user_id, week_start],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM weekly_diets WHERE user_id = ?1 AND week_start = ?2",
        params![user_id, week_start],
        |r| r.get(0),
    )?;
    let mut stmt = conn.prepare(
        "SELECT dp.meal_type, dp.group_id, g.name, dp.portions
         FROM diet_portions dp JOIN food_groups g ON g.id = dp.group_id
         WHERE dp.diet_id = ?1",
    )?;
    let portions = stmt
        .query_map([id], |r| {
            Ok(DietPortion {
                meal_type: r.get(0)?,
                group_id: r.get(1)?,
                group_name: r.get(2)?,
                portions: r.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(WeeklyDiet {
        id,
        user_id,
        week_start: week_start.into(),
        portions,
    })
}

/// Devuelve la dieta más reciente (por week_start DESC) que tenga al menos UNA porción
/// asignada. Se usa como fallback cuando la dieta de la semana solicitada está vacía,
/// para que el usuario no tenga que volver a configurar su dieta cada vez que cambia
/// el calendario.
pub fn get_latest_with_portions(
    pool: &DbPool,
    user_id: i64,
) -> AppResult<Option<WeeklyDiet>> {
    let conn = pool.get()?;
    // Tomamos el diet_id más reciente (week_start DESC) que tenga al menos una porción.
    let row: Option<(i64, String)> = conn
        .query_row(
            "SELECT wd.id, wd.week_start
             FROM weekly_diets wd
             WHERE wd.user_id = ?1
               AND EXISTS (
                 SELECT 1 FROM diet_portions dp WHERE dp.diet_id = wd.id
               )
             ORDER BY wd.week_start DESC
             LIMIT 1",
            params![user_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();
    let Some((id, week_start)) = row else {
        return Ok(None);
    };
    let mut stmt = conn.prepare(
        "SELECT dp.meal_type, dp.group_id, g.name, dp.portions
         FROM diet_portions dp JOIN food_groups g ON g.id = dp.group_id
         WHERE dp.diet_id = ?1",
    )?;
    let portions = stmt
        .query_map([id], |r| {
            Ok(DietPortion {
                meal_type: r.get(0)?,
                group_id: r.get(1)?,
                group_name: r.get(2)?,
                portions: r.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(WeeklyDiet {
        id,
        user_id,
        week_start,
        portions,
    }))
}

pub fn set_portion(
    pool: &DbPool,
    diet_id: i64,
    meal_type: &str,
    group_id: i64,
    portions: f64,
) -> AppResult<()> {
    let conn = pool.get()?;
    if portions <= 0.0 {
        conn.execute(
            "DELETE FROM diet_portions WHERE diet_id = ?1 AND meal_type = ?2 AND group_id = ?3",
            params![diet_id, meal_type, group_id],
        )?;
    } else {
        conn.execute(
            "INSERT INTO diet_portions (diet_id, meal_type, group_id, portions)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(diet_id, meal_type, group_id)
             DO UPDATE SET portions = excluded.portions",
            params![diet_id, meal_type, group_id, portions],
        )?;
    }
    Ok(())
}
