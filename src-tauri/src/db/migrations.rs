use rusqlite::Connection;

use crate::error::AppResult;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  age INTEGER,
  height REAL,
  sex TEXT
);

CREATE TABLE IF NOT EXISTS body_measurements (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  date TEXT NOT NULL,
  weight REAL, back_cm REAL, waist_cm REAL, abdomen_cm REAL, hip_cm REAL
);
CREATE INDEX IF NOT EXISTS idx_measurements_user_date ON body_measurements(user_id, date);

CREATE TABLE IF NOT EXISTS food_groups (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS foods (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  group_id INTEGER NOT NULL REFERENCES food_groups(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  portion_quantity REAL NOT NULL,
  portion_unit TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_foods_group ON foods(group_id);

CREATE TABLE IF NOT EXISTS user_forbidden_foods (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  food_id INTEGER NOT NULL REFERENCES foods(id) ON DELETE CASCADE,
  UNIQUE(user_id, food_id)
);

CREATE TABLE IF NOT EXISTS weekly_diets (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  week_start TEXT NOT NULL,
  UNIQUE(user_id, week_start)
);

CREATE TABLE IF NOT EXISTS diet_portions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  diet_id INTEGER NOT NULL REFERENCES weekly_diets(id) ON DELETE CASCADE,
  meal_type TEXT NOT NULL,
  group_id INTEGER NOT NULL REFERENCES food_groups(id),
  portions REAL NOT NULL,
  UNIQUE(diet_id, meal_type, group_id)
);

CREATE TABLE IF NOT EXISTS smoothies (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  meal_type TEXT NOT NULL,
  raw_text TEXT NOT NULL,
  parsed_json TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS recipes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  instructions TEXT NOT NULL,
  created_by_ai INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS recipe_ingredients (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
  food_id INTEGER REFERENCES foods(id) ON DELETE SET NULL,
  free_name TEXT,
  quantity REAL NOT NULL,
  unit TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS family_plans (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  week_start TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS family_plan_users (
  family_plan_id INTEGER NOT NULL REFERENCES family_plans(id) ON DELETE CASCADE,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  PRIMARY KEY (family_plan_id, user_id)
);

CREATE TABLE IF NOT EXISTS family_meals (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  family_plan_id INTEGER NOT NULL REFERENCES family_plans(id) ON DELETE CASCADE,
  day TEXT NOT NULL,
  meal_type TEXT NOT NULL,
  recipe_id INTEGER REFERENCES recipes(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS family_meal_user_portions (
  family_meal_id INTEGER NOT NULL REFERENCES family_meals(id) ON DELETE CASCADE,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  food_id INTEGER NOT NULL REFERENCES foods(id) ON DELETE CASCADE,
  portions REAL NOT NULL,
  PRIMARY KEY (family_meal_id, user_id, food_id)
);

CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value BLOB
);

CREATE TABLE IF NOT EXISTS saved_plans (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  week_start TEXT NOT NULL,
  user_ids_json TEXT NOT NULL,
  plan_json TEXT NOT NULL,
  notes TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_saved_plans_week ON saved_plans(week_start);
"#;

pub fn run(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(SCHEMA)?;
    upgrade_existing_schema(conn)?;
    seed_food_groups(conn)?;
    seed_foods(conn)?;
    Ok(())
}

/// Añade columnas nuevas a tablas que ya existían en versiones anteriores de la app.
/// Se ejecuta después de CREATE TABLE IF NOT EXISTS para cubrir el caso de upgrade
/// (la tabla ya existe con un esquema viejo y no le añade la columna nueva).
fn upgrade_existing_schema(conn: &Connection) -> AppResult<()> {
    ensure_column(conn, "food_groups", "sort_order", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(conn, "foods", "sort_order", "INTEGER NOT NULL DEFAULT 0")?;
    // Favoritas: añadimos meal_type y created_at a la tabla recipes existente.
    // En ALTER TABLE de SQLite el default no puede ser una expresión, así que
    // dejamos ambas columnas nullables y las rellenamos en el INSERT.
    ensure_column(conn, "recipes", "meal_type", "TEXT")?;
    ensure_column(conn, "recipes", "created_at", "TEXT")?;
    reseed_if_bundled_foods(conn)?;
    Ok(())
}

/// Detecta el caso "ya existen alimentos viejos con comas dentro del nombre"
/// (p. ej. "Plátano, membrillo, pera, zapote") y los borra para que el seed
/// posterior los vuelva a insertar uno por uno. También cubre el caso donde
/// los nombres venían combinados con " y " (p. ej. "Uva verde y morada").
fn reseed_if_bundled_foods(conn: &Connection) -> AppResult<()> {
    // Un alimento "bundled" es aquel cuyo nombre contiene una coma seguida de
    // espacio — los nombres individuales del folleto nunca llevan coma.
    let bundled: i64 = conn.query_row(
        "SELECT COUNT(*) FROM foods WHERE name LIKE '%, %'",
        [],
        |r| r.get(0),
    )?;
    if bundled == 0 {
        return Ok(());
    }

    // Borramos en orden para respetar foreign keys.
    conn.execute("DELETE FROM user_forbidden_foods", [])?;
    conn.execute("DELETE FROM family_meal_user_portions", [])?;
    conn.execute(
        "DELETE FROM recipe_ingredients WHERE food_id IS NOT NULL",
        [],
    )?;
    conn.execute("DELETE FROM foods", [])?;
    // Resetea el autoincrement para que los IDs arranquen de nuevo en 1
    // (opcional pero mantiene la base limpia).
    let _ = conn.execute("DELETE FROM sqlite_sequence WHERE name = 'foods'", []);
    Ok(())
}

fn ensure_column(
    conn: &Connection,
    table: &str,
    column: &str,
    decl: &str,
) -> AppResult<()> {
    let exists: bool = {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
        let mut rows = stmt.query([])?;
        let mut found = false;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name == column {
                found = true;
                break;
            }
        }
        found
    };
    if !exists {
        let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, decl);
        conn.execute(&sql, [])?;
    }
    Ok(())
}

fn seed_food_groups(conn: &Connection) -> AppResult<()> {
    // Orden exacto del folleto.
    let groups = [
        "Grasas",
        "Verduras",
        "Leguminosas",
        "Frutas",
        "Proteínas",
        "Leche",
        "Azúcares",
        "Cereales",
    ];
    let mut stmt = conn.prepare(
        "INSERT INTO food_groups (name, sort_order) VALUES (?1, ?2)
         ON CONFLICT(name) DO UPDATE SET sort_order = excluded.sort_order",
    )?;
    for (i, g) in groups.iter().enumerate() {
        stmt.execute(rusqlite::params![g, i as i64])?;
    }
    Ok(())
}

// Una fila del folleto: (nombres_separados_por_coma, porción, unidad)
type Row = (&'static str, f64, &'static str);

fn seed_foods(conn: &Connection) -> AppResult<()> {
    // Alimentos transcritos del folleto. Cada string que contiene comas se expande
    // a varios alimentos independientes con la misma porción y unidad.
    let data: &[(&str, &[Row])] = &[
        ("Grasas", &[
            ("Crema", 1.0, "cda"),
            ("Aceite de oliva", 1.0, "cda"),
            ("Aceite vegetal", 1.0, "cda"),
            ("Aceite de aguacate", 1.0, "cda"),
            ("Aceite de coco", 1.0, "cda"),
            ("Vinagreta", 1.0, "cda"),
            ("Mayonesa", 1.0, "cda"),
            ("Mantequilla", 1.0, "cda"),
            ("Queso crema", 1.0, "cda"),
            ("Margarina", 1.0, "cda"),
            ("Crema de cacahuate", 1.0, "cda"),
            ("Semillas secas", 1.0, "cda"),
            ("Aderezo", 1.0, "cda"),
            ("Pistaches", 4.0, "pzs"),
            ("Almendras", 4.0, "pzs"),
            ("Piñones", 4.0, "pzs"),
            ("Avellana", 8.0, "pzs"),
            ("Cacahuates", 6.0, "pzs"),
            ("Aguacate", 0.25, "pz"),
            ("Chorizo", 10.0, "g"),
            ("Nuez", 3.0, "pzs"),
            ("Nuez de la India", 3.0, "pzs"),
            ("Tocino", 1.0, "reb"),
            ("Queso amarillo", 1.0, "reb"),
            ("Pata de cerdo pequeña", 1.0, "pz"),
            ("Chicharrón de cerdo", 30.0, "g"),
            ("Ajonjolí", 4.0, "cdas"),
            ("Cacahuate horneado", 10.0, "pzs"),
            ("Aceite aerosol", 2.0, "pres"),
            ("Semillas de hemp", 2.0, "cdas"),
            ("Semilla girasol", 1.5, "cdas"),
        ]),
        ("Verduras", &[
            ("Berenjena", 1.0, "tza"),
            ("Acelgas", 1.0, "tza"),
            ("Brócoli", 1.0, "tza"),
            ("Ejote", 1.0, "tza"),
            ("Espinacas", 1.0, "tza"),
            ("Pepino", 1.0, "tza"),
            ("Jícama", 1.0, "tza"),
            ("Calabacita", 1.0, "tza"),
            ("Col", 1.0, "tza"),
            ("Coliflor", 1.0, "tza"),
            ("Chayote", 1.0, "tza"),
            ("Chiles", 1.0, "tza"),
            ("Rábano", 1.0, "tza"),
            ("Verdolagas", 1.0, "tza"),
            ("Berros", 1.0, "tza"),
            ("Tomates", 1.0, "tza"),
            ("Nopales", 1.0, "tza"),
            ("Lechugas", 1.0, "tza"),
            ("Jitomate", 1.0, "tza"),
            ("Flor de calabaza", 1.0, "tza"),
            ("Champiñones", 1.0, "tza"),
            ("Romeritos (taza)", 1.0, "tza"),
            ("Betabel", 0.5, "tza"),
            ("Calabaza amarilla", 0.5, "tza"),
            ("Cebolla blanca", 0.5, "tza"),
            ("Perejil", 0.5, "tza"),
            ("Zanahoria", 0.5, "tza"),
            ("Chaya", 0.5, "tza"),
            ("Germinado de soya o alfalfa", 0.5, "tza"),
            ("Puré de tomate", 0.5, "tza"),
            ("Aceitunas", 0.5, "tza"),
            ("Rábano rebanado", 0.5, "tza"),
            ("Acelga cruda", 1.5, "tza"),
            ("Arúgula", 1.5, "tza"),
            ("Pimiento morrón", 0.5, "pz"),
            ("Chile poblano", 0.5, "pz"),
            ("Apio", 1.5, "bara"),
            ("Espárragos", 4.0, "pzs"),
            ("Zanahoria baby", 4.0, "pzs"),
            ("Germen", 0.33, "tza"),
            ("Romeritos (gramos)", 120.0, "g"),
        ]),
        ("Leguminosas", &[
            ("Frijoles de la olla", 0.5, "tza"),
            ("Lentejas", 0.5, "tza"),
            ("Garbanzo", 0.5, "tza"),
            ("Alubias", 0.5, "tza"),
            ("Habas", 0.5, "tza"),
            ("Chícharos", 0.5, "tza"),
            ("Frijoles machacados sin aceite", 0.33, "tza"),
            ("Soya", 0.33, "tza"),
        ]),
        ("Frutas", &[
            ("Plátano", 0.5, "pza"),
            ("Membrillo", 0.5, "pza"),
            ("Pera", 0.5, "pza"),
            ("Zapote", 0.5, "pza"),
            ("Guanábana", 0.5, "pza"),
            ("Manzana verde", 0.5, "pza"),
            ("Pitahaya", 0.5, "pza"),
            ("Mango mediano", 0.5, "pza"),
            ("Guayaba", 3.0, "pzs"),
            ("Tuna", 3.0, "pzs"),
            ("Lima mediana", 3.0, "pzs"),
            ("Ciruela amarilla", 3.0, "pzs"),
            ("Lichis", 3.0, "pzs"),
            ("Mandarina", 2.0, "pzs"),
            ("Ciruela pasa", 2.0, "pzs"),
            ("Higos", 2.0, "pzs"),
            ("Durazno", 2.0, "pzs"),
            ("Dátil", 2.0, "pzs"),
            ("Ciruela negra", 2.0, "pzs"),
            ("Granados", 2.0, "pzs"),
            ("Pitaya", 2.0, "pzs"),
            ("Tamarindo", 2.0, "pzs"),
            ("Chabacano", 4.0, "pzs"),
            ("Fresa", 1.0, "tza"),
            ("Papaya", 1.0, "tza"),
            ("Melón", 1.0, "tza"),
            ("Piña", 1.0, "tza"),
            ("Sandía", 1.0, "tza"),
            ("Manzana roja", 1.0, "pza"),
            ("Naranja", 1.0, "pza"),
            ("Granada roja", 1.0, "pza"),
            ("Toronja chica", 1.0, "pza"),
            ("Kiwi", 1.0, "pza"),
            ("Zarzamoras", 0.75, "tza"),
            ("Frambuesa", 0.75, "tza"),
            ("Blueberry", 0.75, "tza"),
            ("Mamey chico", 0.33, "pza"),
            ("Plátano macho", 0.25, "pza"),
            ("Tejocote", 8.0, "pzs"),
            ("Pasas", 10.0, "pzs"),
            ("Cereza", 12.0, "pzs"),
            ("Jugos naturales", 0.5, "tza"),
            ("Capulín", 0.5, "tza"),
            ("Coco", 1.0, "reb"),
            ("Uva verde", 15.0, "pzs"),
            ("Uva morada", 15.0, "pzs"),
            ("Arándano seco", 15.0, "pzs"),
            ("Agua de coco", 1.0, "vso"),
        ]),
        ("Proteínas", &[
            ("Conejo", 30.0, "g"),
            ("Pavo", 30.0, "g"),
            ("Pollo", 30.0, "g"),
            ("Lomo", 30.0, "g"),
            ("Cerdo", 30.0, "g"),
            ("Res", 30.0, "g"),
            ("Pescado", 30.0, "g"),
            ("Mariscos", 30.0, "g"),
            ("Atún fresco", 30.0, "g"),
            ("Salchicha de pavo", 30.0, "g"),
            ("Lengua", 30.0, "g"),
            ("Corazón", 30.0, "g"),
            ("Riñón", 30.0, "g"),
            ("Hígado", 30.0, "g"),
            ("Carne deshebrada", 30.0, "g"),
            ("Pollo con piel", 30.0, "g"),
            ("Molleja", 30.0, "g"),
            ("Costilla de res", 30.0, "g"),
            ("Espinazo", 30.0, "g"),
            ("Queso manchego", 30.0, "g"),
            ("Queso cotija", 30.0, "g"),
            ("Queso adobera", 30.0, "g"),
            ("Queso de mesa", 30.0, "g"),
            ("Queso fresco", 30.0, "g"),
            ("Queso panela", 30.0, "g"),
            ("Queso chihuahua", 30.0, "g"),
            ("Filete de pescado", 40.0, "g"),
            ("Pescado blanco", 40.0, "g"),
            ("Filete lenguado", 45.0, "g"),
            ("Quesos añejos", 25.0, "g"),
            ("Queso oaxaca", 25.0, "g"),
            ("Queso parmesano", 25.0, "g"),
            ("Adobera para fundir", 15.0, "g"),
            ("Queso mozzarella", 15.0, "g"),
            ("Pechuga de pollo", 30.0, "g"),
            ("Milanesa de pollo", 30.0, "g"),
            ("Salmón fresco", 30.0, "g"),
            ("Atún en agua", 0.5, "lta"),
            ("Atún light", 0.5, "lta"),
            ("Chuleta cerdo", 0.5, "pza"),
            ("Queso ricotta", 3.0, "cds"),
            ("Requesón", 2.0, "cds"),
            ("Cottage", 2.0, "cds"),
            ("Charales", 10.0, "pzs"),
            ("Jamón de pavo", 1.0, "reb"),
            ("Clara de huevo", 2.0, "pzs"),
            ("Clara de huevo líquida", 0.75, "tza"),
            ("Sardina drenada grande", 1.0, "pz"),
            ("Huevo entero", 1.0, "pz"),
            ("Camarón", 5.0, "pzs"),
        ]),
        ("Leche", &[
            ("Leche evaporada", 1.0, "tza"),
            ("Leche descremada light", 1.0, "tza"),
            ("Leche semi descremada light", 1.0, "tza"),
            ("Leche vegetal (marca Silk)", 0.5, "tza"),
            ("Leche de almendras", 0.5, "tza"),
            ("Leche de coco", 0.5, "tza"),
            ("Yogurt de sabor", 0.5, "tza"),
            ("Jocoque", 0.5, "tza"),
            ("Yogurt natural", 0.75, "tza"),
            ("Yogurt griego sin azúcar", 0.75, "tza"),
            ("Fage", 0.75, "tza"),
            ("Chobani", 0.75, "tza"),
        ]),
        ("Azúcares", &[
            ("Helado de agua", 0.5, "tza"),
            ("Gelatina marca Art", 0.5, "tza"),
            ("Chocolate tablilla zero sugar", 8.0, "g"),
            ("Mole", 3.0, "cds"),
            ("Pipián", 3.0, "cds"),
            ("Refresco", 0.33, "tza"),
            ("Sopa condensada", 0.33, "tza"),
            ("Helado crema", 0.25, "tza"),
            ("Frutas en almíbar", 0.25, "tza"),
            ("Azúcar blanca", 2.0, "cds"),
            ("Azúcar morena", 2.0, "cds"),
            ("Miel", 2.0, "cds"),
            ("Cajeta", 2.0, "cds"),
            ("Cátsup", 2.0, "cds"),
            ("Chocomilk", 2.0, "cds"),
            ("Vinos y licores", 0.5, "oz"),
            ("Mermelada", 1.0, "cda"),
            ("Jaleas", 1.0, "cda"),
            ("Piloncillo", 1.0, "cda"),
            ("Malvavisco chico", 3.0, "pzs"),
        ]),
        ("Cereales", &[
            ("Tortilla de maíz", 1.0, "pza"),
            ("Tortilla de harina", 1.0, "pza"),
            ("Tortilla de nopal", 1.0, "pza"),
            ("Bimbo doble cero", 1.0, "pza"),
            ("Tostadas", 1.0, "pza"),
            ("Galletas habaneras", 1.0, "pza"),
            ("Papa", 1.0, "pza"),
            ("Camote", 1.0, "pza"),
            ("Pan de hot dog", 1.0, "pza"),
            ("Galletas marías", 4.0, "pzas"),
            ("Papa cambray", 4.0, "pzas"),
            ("Pizza", 1.0, "reb"),
            ("Bolillo", 0.5, "pz"),
            ("Pan de hamburguesa", 0.5, "pz"),
            ("Hot cakes mediano", 0.5, "pz"),
            ("Elote amarillo o blanco", 0.5, "pz"),
            ("Tamal", 0.5, "pz"),
            ("Pan baguette", 0.5, "pz"),
            ("Germen de trigo", 2.0, "cds"),
            ("Harina de trigo", 2.0, "cds"),
            ("Avena", 2.0, "cds"),
            ("Amaranto natural", 2.0, "cds"),
            ("Masa", 2.0, "cds"),
            ("Maicena", 2.0, "cds"),
            ("Quinoa", 2.0, "cds"),
            ("Galleta de arroz inflado", 2.0, "pzs"),
            ("Granola", 3.0, "cds"),
            ("Salvado de trigo", 3.0, "cds"),
            ("Maíz pozolero", 0.5, "tza"),
            ("Cereal de caja sin azúcar", 0.5, "tza"),
            ("Croutones", 0.5, "tza"),
            ("Pasta integral cocida", 0.25, "tza"),
            ("Arroz blanco", 1.5, "tza"),
            ("Arroz integral", 1.5, "tza"),
            ("Harina de arroz", 0.75, "tza"),
            ("Palomitas caseras", 0.75, "tza"),
            ("Avena cocida", 0.5, "tza"),
            ("Palomitas naturales", 4.0, "tza"),
            ("Galletas saladas", 1.0, "paq"),
            ("Galletas salmas", 1.0, "paq"),
            ("Pan thins", 1.0, "pza"),
            ("Pan tostado cero cero", 1.0, "pza"),
        ]),
    ];

    let mut check = conn.prepare("SELECT COUNT(*) FROM foods")?;
    let count: i64 = check.query_row([], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    let mut grp_stmt = conn.prepare("SELECT id FROM food_groups WHERE name = ?1")?;
    let mut ins = conn.prepare(
        "INSERT INTO foods (group_id, name, portion_quantity, portion_unit, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    let mut order: i64 = 0;
    for (group_name, rows) in data {
        let group_id: i64 = grp_stmt.query_row([group_name], |r| r.get(0))?;
        for (name, qty, unit) in *rows {
            ins.execute(rusqlite::params![group_id, name, qty, unit, order])?;
            order += 1;
        }
    }
    Ok(())
}
