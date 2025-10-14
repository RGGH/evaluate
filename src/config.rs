// src/config.rs
use serde::Deserialize;
use regex::Regex;
use crate::errors::Result;

/// High-level application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Base API URL for Gemini
    pub api_base: String,
    /// API key for Gemini
    pub api_key: String,
    /// Available models
    pub models: Vec<String>,
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
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let api_base = std::env::var("GEMINI_API_BASE")
            .unwrap_or_else(|_| "https://generativelanguage.googleapis.com".to_string());
        
        let api_key = std::env::var("GEMINI_API_KEY")
            .map_err(|_| crate::errors::EvalError::ApiResponse(
                "GEMINI_API_KEY environment variable must be set".to_string()
            ))?;

        let models_str = std::env::var("GEMINI_MODELS").unwrap_or_else(|_| {
            "gemini-2.5-pro,gemini-2.5-flash".to_string()
        });
        let models = models_str.split(',').map(|s| s.trim().to_string()).collect();
        
        Ok(AppConfig {
            api_base,
            api_key,
            models,
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
            .unwrap_or_else(|| caps[0].to_string()) // Keep original placeholder if key not found
    }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_eval_config_render() {
        // 1. Create an EvalConfig with placeholders
        let eval_config = EvalConfig {
            model: "gemini-2.5-flash".to_string(),
            prompt: "What is the capital of {{country}}?".to_string(),
            expected: Some("The capital is {{capital}}.".to_string()),
            judge_model: Some("gemini-2.5-pro".to_string()),
            criteria: None,
            tags: vec!["geography".to_string()],
            // 2. Add metadata with the values to substitute
            metadata: Some(json!({
                "country": "France",
                "capital": "Paris"
            })),
        };

        // 3. Call the render function
        let rendered_config = eval_config.render().unwrap();

        // 4. Assert that the placeholders have been replaced
        assert_eq!(rendered_config.prompt, "What is the capital of France?");
        assert_eq!(
            rendered_config.expected,
            Some("The capital is Paris.".to_string())
        );

        // Metadata and other fields should remain unchanged
        assert_eq!(rendered_config.model, eval_config.model);
        assert_eq!(rendered_config.metadata, eval_config.metadata);
    }
}