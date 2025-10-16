// src/providers/mod.rs

use crate::errors::Result;

pub mod anthropic;
pub mod gemini;
pub mod ollama;
pub mod openai;

/// A common trait for Large Language Model (LLM) providers.
/// This allows for a unified interface to different model backends like Gemini, Ollama, OpenAI, Anthropic, etc.
/// 
/// Note: We're not using async_trait here, so implementers must handle async directly.
pub trait LlmProvider: Send + Sync {
    /// Generates a response from the LLM based on a given prompt.
    ///
    /// # Arguments
    /// * `model` - The specific model to use for generation (e.g., "gemini-1.5-flash-latest", "gpt-4o", "claude-sonnet-4").
    /// * `prompt` - The input prompt to send to the model.
    ///
    /// # Returns
    /// A `Result` containing a tuple of the generated `String` and the latency in milliseconds (`u64`).
    fn generate(&self, model: &str, prompt: &str) -> impl std::future::Future<Output = Result<(String, u64)>> + Send;
}
