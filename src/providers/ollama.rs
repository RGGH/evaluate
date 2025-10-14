// src/providers/ollama.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::config::OllamaConfig;
use crate::errors::{EvalError, Result};
use crate::providers::LlmProvider;

/// A provider for interacting with local Ollama models.
pub struct OllamaProvider {
    client: Client,
    config: OllamaConfig,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl OllamaProvider {
    /// Creates a new `OllamaProvider`.
    pub fn new(client: Client, config: OllamaConfig) -> Self {
        Self { client, config }
    }
}

impl LlmProvider for OllamaProvider {
    /// Calls the Ollama API with a given prompt and returns the model's response text and latency.
    async fn generate(&self, model: &str, prompt: &str) -> Result<(String, u64)> {
        let url = format!("{}/api/generate", self.config.api_base.trim_end_matches('/'));

        println!("📡 Calling Ollama: {} with model: {}", url, model);

        let body = OllamaRequest {
            model,
            prompt,
            stream: false,
        };

        let start = Instant::now();

        let resp = self.client.post(&url).json(&body).send().await?;

        let status = resp.status();
        let latency_ms = start.elapsed().as_millis() as u64;

        println!("📥 Ollama response status: {} ({}ms)", status, latency_ms);

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

        let ollama_resp: OllamaResponse = resp.json().await?;
        if ollama_resp.response.is_empty() {
            return Err(EvalError::EmptyResponse);
        }

        Ok((ollama_resp.response, latency_ms))
    }
}