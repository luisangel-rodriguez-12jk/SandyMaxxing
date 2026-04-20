use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanUserPortion {
    pub user: String,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanResult {
    pub days: Vec<PlanDay>,
}

// Bloque reutilizable: la regla DURA sobre porciones y cómo expresarlas.
const PORTIONS_RULE: &str = "\
REGLA DURA DE PORCIONES (no negociable): cada usuario tiene en 'portions' una lista con \
{meal_type, group, portions}. Para CADA comida que propongas, la distribución de ingredientes para \
CADA usuario debe sumar EXACTAMENTE las porciones que ese usuario tiene asignadas para ese meal_type, \
sin pasarse ni quedarse corto — por grupo. Ejemplo: si un usuario tiene en 'comida' {Proteínas:2, \
Cereales:2, Verduras:2, Grasas:1} entonces la ración de ese usuario en la comida debe equivaler a \
2 proteínas + 2 cereales + 2 verduras + 1 grasa. \
En el campo 'per_user_portions[i].notes' DEBES escribir explícitamente el conteo por grupo que le \
toca a ese usuario, con este formato: 'Proteína: 2 (120 g pechuga de pollo) · Cereales: 2 (1 taza \
arroz) · Verduras: 2 (2 tazas ensalada) · Grasas: 1 (1 cdita aceite)'. Si el tamaño del platillo es \
distinto para cada usuario, indícalo claramente en esas notas. ";

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
tomar una variante (otra fruta/leche) o el mismo licuado si es compatible. Si el mismo meal_type \
ya tiene una comida sugerida, respeta el licuado para quien lo tenga registrado y describe en \
per_user_portions qué toma cada uno. \
\
Devuelve EXCLUSIVAMENTE un objeto JSON con este esquema: \
{\"days\":[{\"day\":string,\"meals\":[{\"meal_type\":string,\"name\":string,\"instructions\":string,\
\"ingredients\":[{\"name\":string,\"quantity\":number,\"unit\":string}],\
\"per_user_portions\":[{\"user\":string,\"notes\":string}]}]}]}. \
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
\
PORCIONES: La suma total de porciones (por grupo) consumidas por UN usuario a lo largo de los 5 \
tiempos de comida de UN día DEBE coincidir con la suma de sus 'portions' diarias. No uses un grupo \
que ese usuario no tenga asignado. No excedas la cuota. \
";

// Nota: concatenamos PORTIONS_RULE al final.
pub async fn generate(api_key: &str, req: &PlanRequest) -> AppResult<PlanResult> {
    let system = format!("{SYSTEM}\n\n{PORTIONS_RULE}Todo en español, natural y claro.");
    let user_content = serde_json::to_string(req)?;
    let content = super::chat_json(api_key, &system, &user_content).await?;
    serde_json::from_str::<PlanResult>(&content)
        .map_err(|e| AppError::InvalidAi(format!("plan inválido: {e}")))
}

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
\"per_user_portions\":[{\"user\":string,\"notes\":string}]}. \
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

pub async fn generate_single_meal(api_key: &str, req: &PlanRequest) -> AppResult<SingleMeal> {
    let system = format!("{SINGLE_MEAL_SYSTEM}\n\n{PORTIONS_RULE}Todo en español.");
    let user_content = serde_json::to_string(req)?;
    let content = super::chat_json(api_key, &system, &user_content).await?;
    serde_json::from_str::<SingleMeal>(&content)
        .map_err(|e| AppError::InvalidAi(format!("comida inválida: {e}")))
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
\"per_user_portions\":[{\"user\":string,\"notes\":string}]}]}. \
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
    let system = format!("{MEAL_OPTIONS_SYSTEM}\n\n{PORTIONS_RULE}Todo en español.");
    let user_content = serde_json::to_string(&wrapped)?;
    let content = super::chat_json(api_key, &system, &user_content).await?;
    serde_json::from_str::<MealOptions>(&content)
        .map_err(|e| AppError::InvalidAi(format!("opciones inválidas: {e}")))
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
\"per_user_portions\":[{\"user\":string,\"notes\":string}]}. \
Mantén el mismo meal_type que 'original_meal'. \
El 'name' puede ser un nombre común de cocina casera o de restaurante — es SOLO una etiqueta. \
'ingredients' SÍ debe listar únicamente alimentos que aparezcan textualmente en \
allowed_foods_by_group y NUNCA los prohibidos de ningún usuario. \
El campo instructions debe contener pasos numerados claros con tiempos y utensilios.";

pub async fn tweak_meal(
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
    let system = format!("{TWEAK_SYSTEM}\n\n{PORTIONS_RULE}Todo en español.");
    let user_content = serde_json::to_string(&wrapped)?;
    let content = super::chat_json(api_key, &system, &user_content).await?;
    serde_json::from_str::<PlanMeal>(&content)
        .map_err(|e| AppError::InvalidAi(format!("comida ajustada inválida: {e}")))
}
