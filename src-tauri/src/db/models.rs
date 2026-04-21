use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub age: Option<i64>,
    pub height: Option<f64>,
    pub sex: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Measurement {
    pub id: i64,
    pub user_id: i64,
    pub date: String,
    pub weight: Option<f64>,
    pub back_cm: Option<f64>,
    pub waist_cm: Option<f64>,
    pub abdomen_cm: Option<f64>,
    pub hip_cm: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FoodGroup {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Food {
    pub id: i64,
    pub group_id: i64,
    pub group_name: String,
    pub name: String,
    pub portion_quantity: f64,
    pub portion_unit: String,
    pub forbidden: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DietPortion {
    pub meal_type: String,
    pub group_id: i64,
    pub group_name: String,
    pub portions: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeeklyDiet {
    pub id: i64,
    pub user_id: i64,
    pub week_start: String,
    pub portions: Vec<DietPortion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmoothieIngredient {
    pub name: String,
    pub quantity: f64,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedSmoothie {
    pub ingredients: Vec<SmoothieIngredient>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Smoothie {
    pub id: i64,
    pub user_id: i64,
    pub meal_type: String,
    pub raw_text: String,
    pub parsed: Option<ParsedSmoothie>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipeIngredient {
    pub food_id: Option<i64>,
    pub free_name: Option<String>,
    pub name: String,
    pub quantity: f64,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Recipe {
    pub id: i64,
    pub name: String,
    pub instructions: String,
    pub created_by_ai: bool,
    pub meal_type: Option<String>,
    pub created_at: Option<String>,
    pub ingredients: Vec<RecipeIngredient>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FamilyPlan {
    pub id: i64,
    pub name: String,
    pub week_start: String,
    pub user_ids: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShoppingItem {
    pub name: String,
    pub group_name: String,
    pub quantity: f64,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SavedPlan {
    pub id: i64,
    pub name: String,
    pub week_start: String,
    pub user_ids_json: String,
    pub plan_json: String,
    pub notes: Option<String>,
    pub created_at: String,
}
