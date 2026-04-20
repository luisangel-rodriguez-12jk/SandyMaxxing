pub mod smoothie_parser;
pub mod plan_generator;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const BASE_URL: &str = "https://api.openai.com/v1/chat/completions";
const MODEL: &str = "gpt-4o-mini";

#[derive(Serialize)]
struct ChatReq<'a> {
    model: &'a str,
    messages: Vec<ChatMsg<'a>>,
    response_format: ResponseFormat,
    temperature: f32,
}

#[derive(Serialize)]
struct ChatMsg<'a> {
    role: &'a str,
    content: &'a str,
}

/// Versión dueña (owned) usada cuando construimos la conversación dinámicamente
/// con reintentos: el modelo puede ver su respuesta anterior + nuestro feedback.
#[derive(Clone)]
pub struct ChatMsgOwned {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Deserialize)]
struct ChatResp {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatRespMsg,
}

#[derive(Deserialize)]
struct ChatRespMsg {
    content: String,
}

/// Llamada simple de un system + un user.
pub async fn chat_json(api_key: &str, system: &str, user: &str) -> AppResult<String> {
    let messages = vec![
        ChatMsgOwned { role: "system".into(), content: system.to_string() },
        ChatMsgOwned { role: "user".into(), content: user.to_string() },
    ];
    chat_json_messages(api_key, &messages).await
}

/// Llamada multi-turno, para flujos con reintentos donde le pasamos al modelo
/// su propia respuesta previa + un mensaje de corrección.
pub async fn chat_json_messages(
    api_key: &str,
    messages: &[ChatMsgOwned],
) -> AppResult<String> {
    let req = ChatReq {
        model: MODEL,
        messages: messages
            .iter()
            .map(|m| ChatMsg { role: &m.role, content: &m.content })
            .collect(),
        response_format: ResponseFormat { kind: "json_object" },
        temperature: 0.4,
    };
    let client = reqwest::Client::new();
    let resp = client
        .post(BASE_URL)
        .bearer_auth(api_key)
        .json(&req)
        .send()
        .await?;
    if !resp.status().is_success() {
        let code = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::InvalidAi(format!("HTTP {code}: {body}")));
    }
    let parsed: ChatResp = resp.json().await?;
    parsed
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| AppError::InvalidAi("respuesta vacía".into()))
}
