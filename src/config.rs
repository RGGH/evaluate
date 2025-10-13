// src/config.rs
use std::fs;
use serde::Deserialize;
use crate::errors::Result;

/// High-level application configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    /// Base API URL for Gemini
    pub api_base: String,
    /// API key for Gemini
    pub api_key: String,
    /// List of JSON eval files
    #[serde(default)]
    pub evals: Vec<String>,
}

/// Contains all the information needed to run one prompt against a model
#[derive(Deserialize, Debug, Clone)]
pub struct EvalConfig {
    /// The model to evaluate
    pub model: String,
    
    /// The prompt to send to the model
    pub prompt: String,
    
    /// Expected output for comparison (optional)
    #[serde(default)]
    pub expected: Option<String>,
    
    /// Judge model for LLM-as-a-judge evaluation (optional)
    #[serde(default)]
    pub judge_model: Option<String>,
    
    /// Custom evaluation criteria (optional)
    /// If not provided, default semantic equivalence criteria will be used
    #[serde(default)]
    pub criteria: Option<String>,
    
    /// Tags for organizing evals
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Metadata for the eval
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
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
    
    /// Load multiple EvalConfigs from a JSON array file
    pub fn batch_from_file(path: &str) -> Result<Vec<Self>> {
        let data = fs::read_to_string(path)?;
        let configs: Vec<EvalConfig> = serde_json::from_str(&data)?;
        Ok(configs)
    }
}