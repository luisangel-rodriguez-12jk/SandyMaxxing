use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("error de base de datos: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("error de conexión: {0}")]
    Pool(#[from] r2d2::Error),
    #[error("error de serialización: {0}")]
    Json(#[from] serde_json::Error),
    #[error("error HTTP: {0}")]
    Http(#[from] reqwest::Error),
    #[error("error de IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("OpenAI devolvió datos inválidos: {0}")]
    InvalidAi(String),
    #[error("no se encontró una clave de OpenAI configurada")]
    MissingApiKey,
    #[error("no se encontró el recurso: {0}")]
    NotFound(String),
    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        AppError::Other(value.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
