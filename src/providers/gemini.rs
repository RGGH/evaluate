// src/providers/gemini.rs

use reqwest::Client;
use serde_json::json;
use std::time::Instant;

use crate::config::GeminiConfig;
use crate::errors::{EvalError, Result};
use crate::providers::LlmProvider;

/// A provider for interacting with Google's Gemini models.
pub struct GeminiProvider {
    client: Client,
    config: GeminiConfig,
}

impl GeminiProvider {
    /// Creates a new `GeminiProvider`.
    pub fn new(client: Client, config: GeminiConfig) -> Self {
        Self { client, config }
    }
}

impl LlmProvider for GeminiProvider {
    /// Calls the Gemini API with a given prompt and returns the model's response text and latency.
    async fn generate(&self, model: &str, prompt: &str) -> Result<(String, u64)> {
        let url = format!(
            "{}/v1beta/models/{}:generateContent",
            self.config.api_base.trim_end_matches('/'),
            model
        );

        println!("📡 Calling Gemini: {} with model: {}", url, model);

        let body = json!({
            "contents": [{"parts": [{"text": prompt}]}]
        });

        let start = Instant::now();

        let resp = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.config.api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let latency_ms = start.elapsed().as_millis() as u64;

        println!("📥 Gemini response status: {} ({}ms)", status, latency_ms);

        if !status.is_success() {
            let error_body = resp
                .text()
                .await
                .unwrap_or_else(|_| "Could not read error body".to_string());
            return Err(EvalError::ApiError {
                status: status.as_u16(),
                body: error_body,
            });
        }

        let response_json: serde_json::Value = resp.json().await?;

        if let Some(error) = response_json.get("error") {
            return Err(EvalError::ApiResponse(error.to_string()));
        }

        let output = response_json
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| EvalError::UnexpectedResponse(response_json.to_string()))?;

        if output.is_empty() {
            return Err(EvalError::EmptyResponse);
        }

        Ok((output.to_string(), latency_ms))
    }
}