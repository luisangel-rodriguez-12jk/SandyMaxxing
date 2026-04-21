use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::ai::ChatMsgOwned;
use crate::error::{AppError, AppResult};
use crate::services::meal_validator;

/// Número máximo de intentos con feedback correctivo para la IA.
/// 1 intento original + hasta 2 reintentos = 3 llamadas como máximo por operación.
const MAX_RETRIES: usize = 2;

#[derive(Serialize)]
pub struct PlanRequest {
    pub users: Vec<PlanUser>,
    pub allowed_foods_by_group: Vec<AllowedGroup>,
    pub smoothies: Vec<PlanSmoothie>,
    pub day_labels: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Serialize)]
pub struct PlanUser {
    pub name: String,
    pub portions: Vec<PlanPortion>,
    pub forbidden: Vec<String>,
}

#[derive(Serialize)]
pub struct PlanPortion {
    pub meal_type: String,
    pub group: String,
    pub portions: f64,
}

#[derive(Serialize)]
pub struct AllowedGroup {
    pub group: String,
    pub foods: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct PlanSmoothie {
    pub user: String,
    pub meal_type: String,
    pub raw_text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanDay {
    pub day: String,
    pub meals: Vec<PlanMeal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanMeal {
    pub meal_type: String,
    pub name: String,
    pub instructions: String,
    pub ingredients: Vec<PlanIngredient>,
    pub per_user_portions: Vec<PlanUserPortion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanIngredient {
    pub name: String,
    pub quantity: f64,
    pub unit: String,
}

/// Cuántas porciones de un grupo alimenticio consume un usuario en UNA comida.
/// Por ejemplo: {group:"Proteínas", portions:2.0}.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupPortion {
    pub group: String,
    pub portions: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanUserPortion {
    pub user: String,
    pub notes: String,
    /// Conteo explícito de porciones por grupo que ESTE usuario se come en ESTA comida.
    /// Debe coincidir EXACTO con user.portions filtradas por el meal_type correspondiente.
    /// Con serde(default) para que planes viejos (sin este campo) sigan cargando.
    #[serde(default)]
    pub portions_consumed: Vec<GroupPortion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanResult {
    pub days: Vec<PlanDay>,
}

// Bloque reutilizable: la regla DURA sobre porciones y cómo expresarlas.
const PORTIONS_RULE: &str = "\
REGLA DURA DE PORCIONES (NO NEGOCIABLE, se valida por código y si fallas se te rechaza la respuesta): \
cada usuario tiene en 'portions' una lista con {meal_type, group, portions}. Para CADA comida que \
propongas, los ingredientes para CADA usuario deben sumar EXACTAMENTE las porciones que ese usuario \
tiene asignadas para ese meal_type específico, por grupo, sin pasarse ni quedarse corto. Si un \
usuario NO tiene asignaciones de un grupo en ese meal_type, NO incluyas ese grupo para ese usuario. \
\
EN CADA 'per_user_portions[i]' DEBES incluir dos cosas: \
1) 'notes' (texto en español) con el detalle humano: 'Proteína: 2 (120 g pechuga de pollo) · \
   Cereales: 2 (1 taza arroz) · Verduras: 2 (2 tazas ensalada) · Grasas: 1 (1 cdita aceite)'. \
2) 'portions_consumed': arreglo con {group, portions} por cada grupo que ese usuario consume en \
   esa comida. Debe reflejar EXACTAMENTE las porciones asignadas. \
NO inventes un grupo que el usuario no tenga. NO omitas un grupo que el usuario sí tenga. Los \
nombres de los grupos deben coincidir con los de 'allowed_foods_by_group' y 'user.portions'. \
\
PROHIBIDOS (NO NEGOCIABLE): si un ingrediente aparece en el arreglo 'forbidden' de CUALQUIER \
usuario, NO puedes ponerlo en 'ingredients' (ni como sustituto ni como opcional). Revisa cada \
ingrediente contra TODOS los forbidden de TODOS los usuarios antes de escribirlo. ";

// Regla para respetar ingredientes concretos mencionados por el usuario en
// 'notes' o 'user_instruction'. Cuando alguien pide 'algo con picaña', la IA
// debía caer en 'Res' (o el alimento más cercano de allowed_foods_by_group) y
// nombrar el platillo como 'Picaña ...', NO cambiar a pollo por default.
const SPECIFIC_INGREDIENT_RULE: &str = "\
INGREDIENTE / CORTE / PREPARACIÓN ESPECÍFICA (NO NEGOCIABLE): si el usuario menciona en 'notes' o \
'user_instruction' un ingrediente, corte o platillo concreto (ejemplos: 'picaña', 'arrachera', \
'aguja', 't-bone', 'salmón', 'camarón', 'atún', 'lengua', 'chuleta', 'molleja', 'fajitas', \
'tinga', 'cochinita', 'barbacoa', 'birria', 'al pastor', 'tamales', 'enchiladas', 'pozole'), \
DEBES respetarlo como protagonista del platillo: \
1) Identifica el alimento base más cercano en allowed_foods_by_group. Mapeo típico: \
picaña/arrachera/aguja/t-bone/fajitas/birria → 'Res' o 'Costilla de res' o 'Carne deshebrada'; \
salmón → 'Salmón fresco' (o 'Pescado'); camarón → 'Camarón'; atún → 'Atún fresco' o 'Atún en \
agua'/'Atún light'; lengua → 'Lengua'; chuleta → 'Chuleta cerdo' o 'Cerdo'; molleja → 'Molleja'; \
tinga/barbacoa → 'Carne deshebrada' o 'Res'; cochinita/al pastor → 'Cerdo'. \
2) Arma el platillo usando EXACTAMENTE ese alimento como proteína principal, con las cantidades \
que cuadren con user.portions. \
3) Nombra el platillo con el término que usó el usuario — p. ej. 'Picaña a la plancha con arroz y \
ensalada', NO 'Carne de res con arroz'. El 'name' es la etiqueta visible: puede sonar a \
restaurante aunque el ingrediente técnico en 'ingredients' sea genérico ('Res', 'Cerdo'). \
4) JAMÁS reemplaces la proteína pedida por otra sin relación (no devuelvas 'Tacos de pollo' \
cuando el usuario pidió picaña). Si el pedido es imposible por prohibidos, usa el sustituto \
animal más cercano permitido y avísalo entre paréntesis en 'instructions'. \
5) Si se piden VARIAS opciones y el usuario nombra un ingrediente específico, TODAS las opciones \
deben ser VARIACIONES de platillos con ese mismo ingrediente (distintas preparaciones, cortes, \
guarniciones). Ejemplo 'algo con picaña' count=3 → 'Picaña a la plancha con ensalada', 'Picaña \
al horno con vegetales', 'Tacos de picaña con guacamole'. NO mezcles con pollo, salmón u otras \
proteínas en ese caso. ";

const SYSTEM: &str = "Eres un nutriólogo experto que diseña planes de alimentación para familias. \
Recibirás un objeto JSON con: (a) la lista de usuarios con sus porciones asignadas por grupo y sus \
alimentos prohibidos, (b) la lista de alimentos permitidos agrupados, (c) los licuados habituales \
(con user + meal_type + raw_text), (d) 'day_labels' con los nombres exactos de cada día a planear, \
(e) un campo 'notes' con PETICIONES ESPECÍFICAS del usuario final (p. ej. 'algo con carne', \
'comida rápida', 'recetas mexicanas'). \
\
PRIORIDAD ABSOLUTA: el campo 'notes' refleja la intención del usuario y DEBES seguirla en todas \
las comidas que propongas siempre que no contradiga los prohibidos ni exceda porciones. Si pide \
'algo con carne' usa proteínas animales (res, cerdo, pollo, pescado) varias veces en la semana. \
Si pide 'comida rápida' propón platillos al estilo de restaurantes conocidos (hamburguesa casera, \
tacos, pizza casera, bowl tipo chipotle) usando SOLO los ingredientes permitidos. \
\
LICUADOS: si un usuario tiene un licuado registrado con meal_type X (por ejemplo colacion1), \
para ESE usuario propón ese licuado en la comida X usando exactamente los ingredientes del \
licuado. En el name del platillo pon 'Licuado de ...'. Los demás usuarios de la familia pueden \
tomar una variante (otra fruta/leche) o el mismo licuado si es compatible. \
\
Devuelve EXCLUSIVAMENTE un objeto JSON con este esquema: \
{\"days\":[{\"day\":string,\"meals\":[{\"meal_type\":string,\"name\":string,\"instructions\":string,\
\"ingredients\":[{\"name\":string,\"quantity\":number,\"unit\":string}],\
\"per_user_portions\":[{\"user\":string,\"notes\":string,\
\"portions_consumed\":[{\"group\":string,\"portions\":number}]}]}]}]}. \
El arreglo 'days' DEBE tener EXACTAMENTE la misma longitud y el mismo ORDEN que 'day_labels', y \
cada entrada 'day' DEBE coincidir textualmente con el label correspondiente. \
Cada día DEBE tener 5 comidas con meal_type en este orden: desayuno, colacion1, comida, colacion2, cena. \
El 'name' del platillo puede ser un nombre común de cocina casera o de restaurante (tacos de res, \
hamburguesa, bowl de pollo, pizza margarita casera) — es SOLO una etiqueta. \
'ingredients' en cambio SÍ debe listar únicamente alimentos que aparezcan textualmente en \
allowed_foods_by_group y NUNCA los prohibidos de ningún usuario. \
El campo 'instructions' debe contener pasos numerados con tiempos aproximados de cocción, utensilios \
y tips. \
En 'per_user_portions' incluye UN elemento por cada usuario. \
";

// ---------- helpers de progreso ----------

fn emit_progress(app: &AppHandle, stage: &str, extra: serde_json::Value) {
    let mut payload = match extra {
        serde_json::Value::Object(m) => serde_json::Value::Object(m),
        _ => serde_json::json!({}),
    };
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("stage".into(), serde_json::Value::String(stage.to_string()));
    }
    let _ = app.emit("ai_progress", payload);
}

// ---------- helpers de porciones pre-computadas ----------

/// Construye un bloque de texto que le dice a la IA, con valores exactos,
/// qué debe ir en portions_consumed para cada usuario en UN meal_type específico.
/// Reduce drásticamente los reintentos porque la IA ya no tiene que derivar
/// las porciones: las copia tal cual. Si un usuario no tiene porciones en ese
/// meal_type, se indica portions_consumed=[] para ese usuario.
fn required_portions_block(users: &[PlanUser], meal_type: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for u in users {
        let pieces: Vec<String> = u
            .portions
            .iter()
            .filter(|p| p.meal_type == meal_type)
            .map(|p| {
                format!(
                    "{{\"group\":\"{}\",\"portions\":{}}}",
                    p.group.replace('"', "\\\""),
                    format_portion_num(p.portions)
                )
            })
            .collect();
        lines.push(format!("- \"{}\": [{}]", u.name, pieces.join(",")));
    }
    format!(
        "VALORES EXACTOS que DEBES usar en portions_consumed para cada usuario en meal_type='{}' \
         (copia estos arreglos EXACTAMENTE — mismos grupos, mismas cantidades, sin inventar ni \
         omitir; si el arreglo está vacío, devuelve portions_consumed=[] para ese usuario):\n{}",
        meal_type,
        lines.join("\n")
    )
}

/// Para plan semanal: bloque con las porciones por usuario × meal_type de todo el día.
fn required_portions_block_week(users: &[PlanUser]) -> String {
    let meal_types = ["desayuno", "colacion1", "comida", "colacion2", "cena"];
    let mut out: Vec<String> = Vec::new();
    out.push(
        "VALORES EXACTOS que DEBES usar en portions_consumed (copia EXACTAMENTE estos arreglos \
         para cada usuario en cada meal_type; si el arreglo está vacío, devuelve portions_consumed=[] \
         para ese usuario en ese meal_type):"
            .to_string(),
    );
    for mt in meal_types {
        out.push(format!("-- meal_type='{mt}' --"));
        for u in users {
            let pieces: Vec<String> = u
                .portions
                .iter()
                .filter(|p| p.meal_type == mt)
                .map(|p| {
                    format!(
                        "{{\"group\":\"{}\",\"portions\":{}}}",
                        p.group.replace('"', "\\\""),
                        format_portion_num(p.portions)
                    )
                })
                .collect();
            out.push(format!("- \"{}\": [{}]", u.name, pieces.join(",")));
        }
    }
    out.join("\n")
}

fn format_portion_num(n: f64) -> String {
    // Para no mandar "2.0" cuando es entero — evita ambigüedades.
    if (n - n.round()).abs() < 1e-9 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// Devuelve un preámbulo que repite la petición del usuario al inicio del
/// user_content, antes del JSON. Sacar el texto del JSON y dejarlo visible al
/// principio aumenta mucho que el modelo lo respete (si pide 'picaña', no lo
/// confundirá con otra proteína).
fn notes_preamble(notes: &Option<String>) -> String {
    match notes {
        Some(n) if !n.trim().is_empty() => format!(
            "PETICIÓN EXPLÍCITA DEL USUARIO (léela primero, respétala en TODAS las comidas y \
             aplica la regla SPECIFIC_INGREDIENT_RULE si menciona algún ingrediente, corte o \
             platillo concreto): \"{}\"\n\n",
            n.trim().replace('"', "'")
        ),
        _ => String::new(),
    }
}

/// Igual que notes_preamble, pero para user_instruction de tweak_meal.
fn instruction_preamble(instruction: &str) -> String {
    if instruction.trim().is_empty() {
        String::new()
    } else {
        format!(
            "PETICIÓN EXPLÍCITA DEL USUARIO PARA AJUSTAR LA COMIDA (respétala al pie de la letra y \
             aplica SPECIFIC_INGREDIENT_RULE si menciona algún ingrediente o platillo concreto): \
             \"{}\"\n\n",
            instruction.trim().replace('"', "'")
        )
    }
}

// ---------- helpers de reintento ----------

fn build_feedback_message(issues: &[String]) -> String {
    // Cap para no reventar el contexto si son muchísimos errores.
    let capped: Vec<&String> = issues.iter().take(25).collect();
    let bullets = capped
        .iter()
        .map(|i| format!("- {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Tu respuesta anterior violó estas reglas duras (las valida el sistema, NO son opcionales). \
         Regenera TODO el JSON con el MISMO esquema corrigiendo exactamente estos puntos, sin \
         introducir nuevos errores:\n\n{bullets}\n\n\
         Recuerda: portions_consumed debe cuadrar EXACTO con user.portions para ese meal_type, \
         los ingredientes deben estar en allowed_foods_by_group y NO estar en forbidden de ningún \
         usuario. Devuelve SOLO el JSON final válido."
    )
}

/// Ejecuta una llamada con reintentos: si el validador encuentra problemas, le pide a la IA
/// que corrija el JSON. `validate` devuelve la lista de issues (vacía si OK).
async fn chat_with_validation<T, V>(
    app: &AppHandle,
    api_key: &str,
    system: &str,
    user_content: &str,
    parse_label: &str,
    mut validate: V,
) -> AppResult<T>
where
    T: for<'de> Deserialize<'de>,
    V: FnMut(&T) -> Vec<String>,
{
    let mut messages = vec![
        ChatMsgOwned { role: "system".into(), content: system.to_string() },
        ChatMsgOwned { role: "user".into(), content: user_content.to_string() },
    ];
    let mut last_err = String::from("sin respuesta");

    for attempt in 0..=MAX_RETRIES {
        emit_progress(
            app,
            "requesting",
            serde_json::json!({
                "attempt": attempt + 1,
                "total": MAX_RETRIES + 1,
            }),
        );
        let content = super::chat_json_messages(api_key, &messages).await?;
        emit_progress(
            app,
            "validating",
            serde_json::json!({"attempt": attempt + 1}),
        );
        match serde_json::from_str::<T>(&content) {
            Ok(parsed) => {
                let issues = validate(&parsed);
                if issues.is_empty() {
                    return Ok(parsed);
                }
                last_err = issues.join(" | ");
                if attempt == MAX_RETRIES {
                    return Err(AppError::InvalidAi(format!(
                        "{parse_label}: la IA no respetó las reglas después de {} intentos. \
                         Problemas restantes: {last_err}",
                        MAX_RETRIES + 1
                    )));
                }
                emit_progress(
                    app,
                    "retrying",
                    serde_json::json!({
                        "attempt": attempt + 2,
                        "reason": format!("corrigiendo {} problema(s)", issues.len()),
                    }),
                );
                messages.push(ChatMsgOwned { role: "assistant".into(), content });
                messages.push(ChatMsgOwned {
                    role: "user".into(),
                    content: build_feedback_message(&issues),
                });
            }
            Err(e) => {
                last_err = e.to_string();
                if attempt == MAX_RETRIES {
                    return Err(AppError::InvalidAi(format!("{parse_label}: {last_err}")));
                }
                emit_progress(
                    app,
                    "retrying",
                    serde_json::json!({
                        "attempt": attempt + 2,
                        "reason": "JSON inválido, pidiendo regenerar",
                    }),
                );
                messages.push(ChatMsgOwned { role: "assistant".into(), content });
                messages.push(ChatMsgOwned {
                    role: "user".into(),
                    content: format!(
                        "Tu respuesta no es JSON válido según el esquema pedido. Error de parseo: \
                         {last_err}. Regenera la respuesta cumpliendo EXACTAMENTE el esquema \
                         indicado y sin texto fuera del JSON."
                    ),
                });
            }
        }
    }
    Err(AppError::InvalidAi(format!(
        "{parse_label}: ciclo de reintentos terminó sin éxito. Último error: {last_err}"
    )))
}

// ---------- generate (plan semanal) ----------

pub async fn generate(app: &AppHandle, api_key: &str, req: &PlanRequest) -> AppResult<PlanResult> {
    let system = format!(
        "{SYSTEM}\n\n{PORTIONS_RULE}\n\n{SPECIFIC_INGREDIENT_RULE}Todo en español, natural y claro."
    );
    let portions_block = required_portions_block_week(&req.users);
    let preamble = notes_preamble(&req.notes);
    let user_content = format!(
        "{preamble}{}\n\n---\n{}",
        serde_json::to_string(req)?,
        portions_block
    );
    chat_with_validation::<PlanResult, _>(
        app,
        api_key,
        &system,
        &user_content,
        "plan inválido",
        |plan| meal_validator::validate_plan(plan, req),
    )
    .await
}

// ---------- single meal ----------

const SINGLE_MEAL_SYSTEM: &str = "Eres un nutriólogo que diseña UNA sola comida compatible para una familia. \
Recibirás un objeto JSON con usuarios, sus porciones, alimentos permitidos agrupados, licuados \
habituales y un campo 'notes' con PETICIONES ESPECÍFICAS del usuario final (p. ej. 'algo con carne', \
'comida rápida', 'ligero', 'recetas mexicanas'). \
\
PRIORIDAD ABSOLUTA: el campo 'notes' es la intención del usuario y DEBES seguirla siempre que no \
choque con prohibidos ni exceda porciones. Si pide 'comida rápida' propón platillos al estilo de \
restaurantes conocidos (hamburguesa casera, tacos, pizza casera, bowl tipo chipotle, sándwich \
tipo subway) usando SOLO los ingredientes permitidos para armarlos. Si pide 'algo con carne' elige \
proteínas animales (res, cerdo, pollo, pescado) que estén en la lista de permitidos. \
\
Devuelve EXCLUSIVAMENTE un objeto JSON con el esquema: \
{\"name\":string,\"instructions\":string,\
\"ingredients\":[{\"name\":string,\"quantity\":number,\"unit\":string}],\
\"per_user_portions\":[{\"user\":string,\"notes\":string,\
\"portions_consumed\":[{\"group\":string,\"portions\":number}]}]}. \
El 'name' puede ser un nombre común de cocina casera o de restaurante (tacos de res, hamburguesa, \
bowl de pollo, pizza margarita casera) — es SOLO una etiqueta. \
'ingredients' en cambio SÍ debe listar únicamente alimentos que aparezcan textualmente en \
allowed_foods_by_group y NUNCA los prohibidos de ningún usuario. \
El campo instructions debe contener pasos numerados claros con tiempos y utensilios. \
Incluye UN elemento en per_user_portions por cada usuario. Todo en español.";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SingleMeal {
    pub name: String,
    pub instructions: String,
    pub ingredients: Vec<PlanIngredient>,
    pub per_user_portions: Vec<PlanUserPortion>,
}

#[derive(Serialize)]
struct SingleMealRequest<'a> {
    #[serde(flatten)]
    base: &'a PlanRequest,
    meal_type: String,
}

pub async fn generate_single_meal(
    app: &AppHandle,
    api_key: &str,
    req: &PlanRequest,
    meal_type: &str,
) -> AppResult<SingleMeal> {
    let system = format!(
        "{SINGLE_MEAL_SYSTEM}\n\n{PORTIONS_RULE}\n\n{SPECIFIC_INGREDIENT_RULE}Todo en español."
    );
    let wrapped = SingleMealRequest {
        base: req,
        meal_type: meal_type.to_string(),
    };
    let portions_block = required_portions_block(&req.users, meal_type);
    let preamble = notes_preamble(&req.notes);
    let user_content = format!(
        "{preamble}{}\n\n---\n{}",
        serde_json::to_string(&wrapped)?,
        portions_block
    );
    chat_with_validation::<SingleMeal, _>(
        app,
        api_key,
        &system,
        &user_content,
        "comida inválida",
        |meal| meal_validator::validate_single_meal(meal, req, meal_type),
    )
    .await
}

// ---- Múltiples opciones de comida ----

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MealOptions {
    pub options: Vec<SingleMeal>,
}

#[derive(Serialize)]
pub struct MealOptionsRequest<'a> {
    #[serde(flatten)]
    pub base: &'a PlanRequest,
    pub meal_type: String,
    pub count: u32,
    pub exclude_names: Vec<String>,
}

const MEAL_OPTIONS_SYSTEM: &str = "Eres un nutriólogo que propone múltiples opciones de UNA comida para una familia. \
Recibirás un objeto JSON con: usuarios y sus porciones, alimentos permitidos agrupados, licuados \
habituales, un campo 'notes' con PETICIONES ESPECÍFICAS del usuario final (p. ej. 'algo con carne', \
'comida rápida', 'ligero', 'recetas mexicanas', 'sugiereme algo que pueda pedir de comida rápida', \
etc.), y además 'meal_type', 'count' y 'exclude_names'. \
\
PRIORIDAD ABSOLUTA: el campo 'notes' refleja la intención explícita del usuario y DEBES seguirla \
en TODAS las opciones que propongas, siempre que no contradiga los prohibidos ni exceda porciones. \
- Si pide 'comida rápida' o 'algo que pueda pedir': propón platillos al estilo de cadenas conocidas \
  (hamburguesa casera, tacos, pizza casera, bowl tipo chipotle, sándwich tipo subway, quesadillas, \
  burritos, hot dog casero) armados SOLO con alimentos de allowed_foods_by_group. \
- Si pide 'algo con carne': usa proteínas animales (res, cerdo, pollo, pescado) permitidas, en la \
  mayoría de las opciones. \
- Si pide 'ligero': reduce grasas y cereales dentro de lo permitido. \
- Si pide 'mexicano' o 'recetas mexicanas': tacos, sopes, enchiladas, tinga, pozole, chilaquiles. \
NUNCA respondas con un array vacío ni te rehúses a proponer opciones: siempre encuentra \
combinaciones compatibles. \
\
Devuelve EXCLUSIVAMENTE un objeto JSON con el esquema: \
{\"options\":[{\"name\":string,\"instructions\":string,\
\"ingredients\":[{\"name\":string,\"quantity\":number,\"unit\":string}],\
\"per_user_portions\":[{\"user\":string,\"notes\":string,\
\"portions_consumed\":[{\"group\":string,\"portions\":number}]}]}]}. \
Debes proponer EXACTAMENTE la cantidad que se te pida en el campo 'count'. \
Cada opción DEBE ser DIFERENTE entre sí y DIFERENTE de los nombres listados en 'exclude_names'. \
El meal_type indica el momento del día (desayuno, colacion1, comida, colacion2 o cena) — ajusta \
la propuesta al momento. \
El 'name' puede ser un nombre común de cocina casera o de restaurante — es SOLO una etiqueta. \
'ingredients' SÍ debe listar únicamente alimentos que aparezcan textualmente en \
allowed_foods_by_group y NUNCA los prohibidos de ningún usuario. \
El campo instructions debe contener pasos numerados claros con tiempos y utensilios. \
Incluye UN elemento en per_user_portions por cada usuario.";

pub async fn generate_meal_options(
    app: &AppHandle,
    api_key: &str,
    req: &PlanRequest,
    meal_type: &str,
    count: u32,
    exclude_names: &[String],
) -> AppResult<MealOptions> {
    let wrapped = MealOptionsRequest {
        base: req,
        meal_type: meal_type.to_string(),
        count,
        exclude_names: exclude_names.to_vec(),
    };
    let system = format!(
        "{MEAL_OPTIONS_SYSTEM}\n\n{PORTIONS_RULE}\n\n{SPECIFIC_INGREDIENT_RULE}Todo en español."
    );
    let portions_block = required_portions_block(&req.users, meal_type);
    let preamble = notes_preamble(&req.notes);
    let user_content = format!(
        "{preamble}{}\n\n---\n{}",
        serde_json::to_string(&wrapped)?,
        portions_block
    );
    chat_with_validation::<MealOptions, _>(
        app,
        api_key,
        &system,
        &user_content,
        "opciones inválidas",
        |opts| meal_validator::validate_meal_options(opts, req, meal_type),
    )
    .await
}

// ---- Reemplazar/ajustar una comida de un plan existente ----

#[derive(Serialize)]
pub struct TweakRequest<'a> {
    pub users: &'a [PlanUser],
    pub allowed_foods_by_group: &'a [AllowedGroup],
    pub original_meal: &'a PlanMeal,
    pub user_instruction: String,
    pub day: String,
}

const TWEAK_SYSTEM: &str = "Eres un nutriólogo que ajusta UNA comida existente de un plan semanal de \
acuerdo con la petición del usuario en el campo 'user_instruction' (por ejemplo 'cámbialo por algo \
con pollo', 'que sea más ligero', 'algo de comida rápida', 'algo con carne', 'recetas mexicanas'). \
\
PRIORIDAD ABSOLUTA: 'user_instruction' es la intención del usuario y DEBES seguirla al pie de la \
letra siempre que no choque con prohibidos ni exceda porciones. \
- Si pide 'comida rápida' o 'algo que pueda pedir': propón un platillo al estilo de restaurantes \
  conocidos armado SOLO con alimentos permitidos. \
- Si pide 'algo con carne': usa una proteína animal permitida. \
- Si pide 'más ligero': baja grasas y cereales dentro de lo que indiquen las porciones. \
NUNCA te rehúses a proponer la comida: siempre encuentra la mejor combinación compatible. \
\
Devuelve EXCLUSIVAMENTE un objeto JSON con el esquema: \
{\"meal_type\":string,\"name\":string,\"instructions\":string,\
\"ingredients\":[{\"name\":string,\"quantity\":number,\"unit\":string}],\
\"per_user_portions\":[{\"user\":string,\"notes\":string,\
\"portions_consumed\":[{\"group\":string,\"portions\":number}]}]}. \
Mantén el mismo meal_type que 'original_meal'. \
El 'name' puede ser un nombre común de cocina casera o de restaurante — es SOLO una etiqueta. \
'ingredients' SÍ debe listar únicamente alimentos que aparezcan textualmente en \
allowed_foods_by_group y NUNCA los prohibidos de ningún usuario. \
El campo instructions debe contener pasos numerados claros con tiempos y utensilios.";

pub async fn tweak_meal(
    app: &AppHandle,
    api_key: &str,
    users: &[PlanUser],
    allowed: &[AllowedGroup],
    original: &PlanMeal,
    user_instruction: &str,
    day: &str,
) -> AppResult<PlanMeal> {
    let wrapped = TweakRequest {
        users,
        allowed_foods_by_group: allowed,
        original_meal: original,
        user_instruction: user_instruction.to_string(),
        day: day.to_string(),
    };
    let system = format!(
        "{TWEAK_SYSTEM}\n\n{PORTIONS_RULE}\n\n{SPECIFIC_INGREDIENT_RULE}Todo en español."
    );
    let portions_block = required_portions_block(users, &original.meal_type);
    let preamble = instruction_preamble(user_instruction);
    let user_content = format!(
        "{preamble}{}\n\n---\n{}",
        serde_json::to_string(&wrapped)?,
        portions_block
    );

    // Para validar, armamos un PlanRequest sintético con la misma info que ve el modelo.
    let synthetic_users: Vec<PlanUser> = users
        .iter()
        .map(|u| PlanUser {
            name: u.name.clone(),
            portions: u
                .portions
                .iter()
                .map(|p| PlanPortion {
                    meal_type: p.meal_type.clone(),
                    group: p.group.clone(),
                    portions: p.portions,
                })
                .collect(),
            forbidden: u.forbidden.clone(),
        })
        .collect();
    let synthetic_allowed: Vec<AllowedGroup> = allowed
        .iter()
        .map(|g| AllowedGroup {
            group: g.group.clone(),
            foods: g.foods.clone(),
        })
        .collect();
    let synthetic_req = PlanRequest {
        users: synthetic_users,
        allowed_foods_by_group: synthetic_allowed,
        smoothies: vec![],
        day_labels: vec![],
        notes: None,
    };
    let original_meal_type = original.meal_type.clone();

    chat_with_validation::<PlanMeal, _>(
        app,
        api_key,
        &system,
        &user_content,
        "comida ajustada inválida",
        move |meal| {
            let mut issues = meal_validator::validate_plan_meal(meal, &synthetic_req);
            // Además el meal_type debe mantenerse.
            if meal.meal_type != original_meal_type {
                issues.push(format!(
                    "El meal_type debe mantenerse como '{}', no '{}'.",
                    original_meal_type, meal.meal_type
                ));
            }
            issues
        },
    )
    .await
}
