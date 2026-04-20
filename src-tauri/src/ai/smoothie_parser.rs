use crate::db::models::ParsedSmoothie;
use crate::error::{AppError, AppResult};

const SYSTEM: &str = "Eres un asistente experto en nutrición que recibe una receta de licuado en español \
y devuelve exclusivamente JSON con el esquema: \
{\"ingredients\":[{\"name\":<string>,\"quantity\":<number>,\"unit\":<string>}]}. \
Normaliza las unidades a: taza, cda, cdta, pieza, gramos, ml, scoop. \
Usa nombres comunes en español mexicano. No incluyas comentarios ni texto fuera del JSON.";

pub async fn parse(api_key: &str, raw_text: &str) -> AppResult<ParsedSmoothie> {
    let content = super::chat_json(api_key, SYSTEM, raw_text).await?;
    match serde_json::from_str::<ParsedSmoothie>(&content) {
        Ok(v) => Ok(v),
        Err(_) => {
            let retry_user = format!(
                "Re-intenta. Solo responde con JSON válido ({}) para este licuado:\n{}",
                "{\"ingredients\":[{\"name\":string,\"quantity\":number,\"unit\":string}]}",
                raw_text
            );
            let content = super::chat_json(api_key, SYSTEM, &retry_user).await?;
            serde_json::from_str::<ParsedSmoothie>(&content)
                .map_err(|e| AppError::InvalidAi(e.to_string()))
        }
    }
}
