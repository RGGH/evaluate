// src/providers/openai.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::config::OpenAIConfig;
use crate::errors::{EvalError, Result};
use crate::providers::{LlmProvider, TokenUsage};

/// A provider for interacting with OpenAI models.
pub struct OpenAIProvider {
    client: Client,
    config: OpenAIConfig,
}

#[derive(Serialize)]
struct OpenAIRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    temperature: f32,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
    usage: Option<ApiUsage>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

#[derive(Deserialize)]
struct ApiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

impl OpenAIProvider {
    /// Creates a new `OpenAIProvider`.
    pub fn new(client: Client, config: OpenAIConfig) -> Self {
        Self { client, config }
    }
}

impl LlmProvider for OpenAIProvider {
    /// Calls the OpenAI API with a given prompt and returns the model's response text and latency.
    async fn generate(&self, model: &str, prompt: &str) -> Result<(String, u64, TokenUsage)> {
        let url = format!("{}/chat/completions", self.config.api_base.trim_end_matches('/'));

        println!("📡 Calling OpenAI: {} with model: {}", url, model);

        let body = OpenAIRequest {
            model,
            messages: vec![Message {
                role: "user",
                content: prompt,
            }],
            temperature: 0.7,
        };

        let start = Instant::now();

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let latency_ms = start.elapsed().as_millis() as u64;

        println!("📥 OpenAI response status: {} ({}ms)", status, latency_ms);

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

        let openai_resp: OpenAIResponse = resp.json().await?;

        let token_usage = openai_resp.usage.map_or_else(
            TokenUsage::default,
            |u| TokenUsage {
                input_tokens: Some(u.prompt_tokens),
                output_tokens: Some(u.completion_tokens),
            },
        );
        
        let output = openai_resp
            .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .ok_or_else(|| EvalError::UnexpectedResponse("No choices in response".to_string()))?;

        if output.is_empty() {
            return Err(EvalError::EmptyResponse);
        }

        Ok((output, latency_ms, token_usage))
    }
}
