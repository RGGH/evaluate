// src/config.rs
use serde::Deserialize;
use regex::Regex;
use crate::errors::{Result, EvalError};

/// Configuration for the Gemini provider.
#[derive(Debug, Clone)]
pub struct GeminiConfig {
    pub api_base: String,
    pub api_key: String,
    pub models: Vec<String>,
}

/// Configuration for the Ollama provider.
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub api_base: String,
    pub models: Vec<String>,
}

/// Configuration for the OpenAI provider.
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_base: String,
    pub api_key: String,
    pub models: Vec<String>,
}

/// High-level application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub gemini: Option<GeminiConfig>,
    pub ollama: Option<OllamaConfig>,
    pub openai: Option<OpenAIConfig>,
    pub models: Vec<String>,
}

/// Contains all the information needed to run one prompt against a model
/// The model string is expected to be in the format `provider:model_name`,
/// e.g., `gemini:gemini-1.5-flash`, `ollama:llama3`, or `openai:gpt-4`.
/// If no provider is specified, it will default to `gemini`.
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
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut all_models = Vec::new();
        
        // Gemini configuration
        let gemini_config = if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            let api_base = std::env::var("GEMINI_API_BASE")
                .unwrap_or_else(|_| "https://generativelanguage.googleapis.com".to_string());
            let models_str = std::env::var("GEMINI_MODELS").unwrap_or_else(|_| {
                "gemini-1.5-pro-latest,gemini-1.5-flash-latest".to_string()
            });
            let models: Vec<String> = models_str.split(',').map(|s| s.trim().to_string()).collect();
            all_models.extend(models.iter().map(|m| format!("gemini:{}", m)));
            Some(GeminiConfig { api_base, api_key, models })
        } else {
            None
        };

        // Ollama configuration
        let ollama_config = if let Ok(api_base) = std::env::var("OLLAMA_API_BASE") {
            let models_str = std::env::var("OLLAMA_MODELS").unwrap_or_else(|_| {
                "llama3,gemma".to_string()
            });
            let models: Vec<String> = models_str.split(',').map(|s| s.trim().to_string()).collect();
            all_models.extend(models.iter().map(|m| format!("ollama:{}", m)));
            Some(OllamaConfig { api_base, models })
        } else {
            None
        };

        // OpenAI configuration
        let openai_config = if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let api_base = std::env::var("OPENAI_API_BASE")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
            let models_str = std::env::var("OPENAI_MODELS").unwrap_or_else(|_| {
                "gpt-4o,gpt-4o-mini,gpt-3.5-turbo".to_string()
            });
            let models: Vec<String> = models_str.split(',').map(|s| s.trim().to_string()).collect();
            all_models.extend(models.iter().map(|m| format!("openai:{}", m)));
            Some(OpenAIConfig { api_base, api_key, models })
        } else {
            None
        };

        if gemini_config.is_none() && ollama_config.is_none() && openai_config.is_none() {
            return Err(EvalError::Config(
                "No LLM providers configured. Please set at least one of: GEMINI_API_KEY, OLLAMA_API_BASE, or OPENAI_API_KEY.".to_string()
            ));
        }

        Ok(AppConfig { 
            gemini: gemini_config, 
            ollama: ollama_config,
            openai: openai_config,
            models: all_models 
        })
    }
}

impl EvalConfig {
    /// Creates a new `EvalConfig` by substituting placeholders from its metadata.
    /// Placeholders are in the format `{{key}}`.
    pub fn render(&self) -> Result<Self> {
        let mut rendered_config = self.clone();

        if let Some(metadata) = &self.metadata {
            rendered_config.prompt = render_template(&self.prompt, metadata);
            if let Some(expected) = &self.expected {
                rendered_config.expected = Some(render_template(expected, metadata));
            }
        }

        Ok(rendered_config)
    }
}

/// Simple template renderer using regex.
fn render_template(template: &str, data: &serde_json::Value) -> String {
    let re = Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap();
    re.replace_all(template, |caps: &regex::Captures| {
        let key = &caps[1];
        data.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| caps[0].to_string())
    }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_eval_config_render() {
        let eval_config = EvalConfig {
            model: "gemini-2.5-flash".to_string(),
            prompt: "What is the capital of {{country}}?".to_string(),
            expected: Some("The capital is {{capital}}.".to_string()),
            judge_model: Some("gemini-2.5-pro".to_string()),
            criteria: None,
            tags: vec!["geography".to_string()],
            metadata: Some(json!({
                "country": "France",
                "capital": "Paris"
            })),
        };

        let rendered_config = eval_config.render().unwrap();

        assert_eq!(rendered_config.prompt, "What is the capital of France?");
        assert_eq!(
            rendered_config.expected,
            Some("The capital is Paris.".to_string())
        );

        assert_eq!(rendered_config.model, eval_config.model);
        assert_eq!(rendered_config.metadata, eval_config.metadata);
    }
}