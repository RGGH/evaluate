// src/providers/anthropic.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::config::AnthropicConfig;
use crate::errors::{EvalError, Result};
use crate::providers::{LlmProvider, TokenUsage};

/// A provider for interacting with Anthropic Claude models.
pub struct AnthropicProvider {
    client: Client,
    config: AnthropicConfig,
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    usage: ApiUsage,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct ApiUsage {
    input_tokens: u32,
    output_tokens: u32,
}

impl AnthropicProvider {
    /// Creates a new `AnthropicProvider`.
    pub fn new(client: Client, config: AnthropicConfig) -> Self {
        Self { client, config }
    }
}

impl LlmProvider for AnthropicProvider {
    /// Calls the Anthropic API with a given prompt and returns the model's response text and latency.
    async fn generate(&self, model: &str, prompt: &str) -> Result<(String, u64, TokenUsage)> {
        let url = format!("{}/v1/messages", self.config.api_base.trim_end_matches('/'));

        println!("📡 Calling Anthropic: {} with model: {}", url, model);

        let body = AnthropicRequest {
            model,
            messages: vec![Message {
                role: "user",
                content: prompt,
            }],
            max_tokens: 4096,
            temperature: Some(0.7),
        };

        let start = Instant::now();

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let latency_ms = start.elapsed().as_millis() as u64;

        println!("📥 Anthropic response status: {} ({}ms)", status, latency_ms);

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

        let anthropic_resp: AnthropicResponse = resp.json().await?;

        let token_usage = TokenUsage {
            input_tokens: Some(anthropic_resp.usage.input_tokens),
            output_tokens: Some(anthropic_resp.usage.output_tokens),
        };
        
        let output = anthropic_resp
            .content
            .iter()
            .find(|block| block.content_type == "text")
            .and_then(|block| block.text.as_ref())
            .ok_or_else(|| EvalError::UnexpectedResponse("No text content in response".to_string()))?;

        if output.is_empty() {
            return Err(EvalError::EmptyResponse);
        }

        Ok((output.to_string(), latency_ms, token_usage))
    }
}
