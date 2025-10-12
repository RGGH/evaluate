// src/config.rs
use std::fs;
use serde::Deserialize;
use anyhow::Result;

/// High-level application configuration.
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// Base API URL for Gemini
    pub api_base: String,
    /// API key for Gemini
    pub api_key: String,
    /// List of JSON eval files
    pub evals: Vec<String>,
}

/// Contains all the information needed to run one prompt against a model
#[derive(Deserialize, Debug)]
pub struct EvalConfig {
    pub model: String,
    pub prompt: String,
    #[serde(default)]
    pub expected: Option<String>,
    #[serde(default)]
    pub judge_model: Option<String>,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        let cfg: AppConfig = toml::from_str(&data)?;
        Ok(cfg)
    }
}

impl EvalConfig {
    /// Load a single EvalConfig from a JSON file
    pub fn from_file(path: &str) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        let cfg: EvalConfig = serde_json::from_str(&data)?;
        Ok(cfg)
    }
}