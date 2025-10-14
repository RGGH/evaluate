// src/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum EvalError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to parse TOML config: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Failed to parse JSON config: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API request failed with status {status}: {body}")]
    ApiError { status: u16, body: String },

    #[error("API returned an error: {0}")]
    ApiResponse(String),

    #[error("Unexpected response structure: {0}")]
    UnexpectedResponse(String),

    #[error("Received empty text response from model")]
    EmptyResponse,

    #[error("Model '{model}' failed to respond")]
    ModelFailure { model: String },

    #[error("Judge model '{model}' failed: {source}")]
    JudgeFailure {
        model: String,
        #[source]
        source: Box<EvalError>,
    },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider '{0}' not found")]
    ProviderNotFound(String),
}

pub type Result<T> = std::result::Result<T, EvalError>;