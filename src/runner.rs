// src/runner.rs
use crate::config::{AppConfig, EvalConfig};
use crate::errors::{EvalError, Result};
use crate::providers::{anthropic::AnthropicProvider, gemini::GeminiProvider, ollama::OllamaProvider, openai::OpenAIProvider, LlmProvider, TokenUsage};
use futures::future;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::time::Instant;
use regex::Regex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvalResult {
    pub model: String,
    pub prompt: String,
    pub model_output: String,                    // Raw output for debugging
    pub parsed_output: Option<JsonValue>,        // Structured output if parseable
    pub expected: Option<String>,
    pub judge_result: Option<JudgeResult>,
    pub timestamp: String,
    pub latency_ms: u64,
    pub judge_latency_ms: Option<u64>,
    pub token_usage: Option<TokenUsage>,
    pub judge_token_usage: Option<TokenUsage>,
    pub total_latency_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JudgeResult {
    pub judge_model: String,
    pub verdict: JudgeVerdict,
    #[serde(rename = "reasoning")]
    pub reasoning: Option<String>,
    pub confidence: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum JudgeVerdict {
    Pass,
    Fail,
    Uncertain,
}

impl std::fmt::Display for JudgeVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JudgeVerdict::Pass => write!(f, "Pass"),
            JudgeVerdict::Fail => write!(f, "Fail"),
            JudgeVerdict::Uncertain => write!(f, "Uncertain"),
        }
    }
}

/// Parse judge response to extract verdict and reasoning
fn parse_judge_response(response: &str) -> JudgeResult {
    let response_lower = response.to_lowercase();
    
    let verdict = if response_lower.contains("verdict: pass") || 
                     (response_lower.starts_with("yes") || response_lower.contains("yes, they")) {
        JudgeVerdict::Pass
    } else if response_lower.contains("verdict: fail") || 
              (response_lower.starts_with("no") || response_lower.contains("no, they")) {
        JudgeVerdict::Fail
    } else {
        JudgeVerdict::Uncertain
    };

    let reasoning = if response.len() > 20 {
        Some(response.to_string())
    } else {
        None
    };

    JudgeResult {
        judge_model: "unknown".to_string(),
        verdict,
        reasoning,
        confidence: None,
    }
}

/// Enhanced judge prompt with better structure
fn create_judge_prompt(expected: &str, actual: &str, criteria: Option<&str>) -> String {
    let base_criteria = criteria.unwrap_or(
        "The outputs should convey the same core meaning, even if phrased differently."
    );

    format!(
        r#"You are an expert evaluator comparing two text outputs.

EVALUATION CRITERIA:
{}

EXPECTED OUTPUT:
{}

ACTUAL OUTPUT:
{}

INSTRUCTIONS:
1. Carefully compare both outputs
2. Consider semantic equivalence, not just exact wording
3. Provide your verdict as the first line: "Verdict: PASS" or "Verdict: FAIL"
4. Then explain your reasoning in 2-3 sentences

Your evaluation:"#,
        base_criteria,
        expected,
        actual
    )
}

/// Attempt to parse model output into structured JSON
/// This tries multiple strategies in order of specificity
pub fn parse_model_output(raw_output: &str) -> Option<JsonValue> {
    // Strategy 1: Try parsing the entire output as JSON
    if let Ok(json) = serde_json::from_str::<JsonValue>(raw_output) {
        return Some(json);
    }

    // Strategy 2: Look for JSON code blocks (```json ... ```)
    if let Some(json) = extract_json_code_block(raw_output) {
        if let Ok(parsed) = serde_json::from_str::<JsonValue>(&json) {
            return Some(parsed);
        }
    }

    // Strategy 3: Extract numbers (for math/calculation tasks)
    if let Some(number) = extract_number(raw_output) {
        return Some(serde_json::json!({ "answer": number }));
    }

    // Strategy 4: Extract yes/no boolean
    if let Some(boolean) = extract_boolean(raw_output) {
        return Some(serde_json::json!({ "answer": boolean }));
    }

    // Strategy 5: Extract multiple choice answer (A, B, C, D)
    if let Some(choice) = extract_multiple_choice(raw_output) {
        return Some(serde_json::json!({ "answer": choice }));
    }

    // Strategy 6: Try to extract key-value pairs from natural language
    if let Some(structured) = extract_key_value_pairs(raw_output) {
        return Some(structured);
    }

    None
}

/// Extract JSON from code blocks
fn extract_json_code_block(text: &str) -> Option<String> {
    let re = Regex::new(r"```(?:json)?\s*\n([\s\S]*?)\n```").ok()?;
    re.captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Extract numeric answer from text
fn extract_number(text: &str) -> Option<f64> {
    // Look for patterns like "The answer is 42" or "Result: 3.14"
    let patterns = [
        r"(?:answer|result|solution)(?:\s+is)?[:\s]+(-?\d+\.?\d*)",
        r"^(-?\d+\.?\d*)$",  // Just a number
        r"\b(-?\d+\.?\d*)\b(?:\s*$)", // Number at end
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(num_str) = caps.get(1) {
                    if let Ok(num) = num_str.as_str().parse::<f64>() {
                        return Some(num);
                    }
                }
            }
        }
    }
    None
}

/// Extract boolean answer from text
fn extract_boolean(text: &str) -> Option<bool> {
    let text_lower = text.to_lowercase().trim().to_string();
    
    if text_lower.starts_with("yes") || text_lower.contains("answer is yes") || text_lower == "true" {
        return Some(true);
    }
    
    if text_lower.starts_with("no") || text_lower.contains("answer is no") || text_lower == "false" {
        return Some(false);
    }
    
    None
}

/// Extract multiple choice answer (A, B, C, D, etc.)
fn extract_multiple_choice(text: &str) -> Option<String> {
    let re = Regex::new(r"(?:answer|choice)(?:\s+is)?[:\s]+([A-Za-z])").ok()?;
    re.captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_uppercase())
}

/// Extract key-value pairs from structured text
fn extract_key_value_pairs(text: &str) -> Option<JsonValue> {
    let mut map = serde_json::Map::new();
    
    // Look for patterns like "Name: John" or "Age: 25"
    let re = Regex::new(r"(?m)^([A-Za-z\s]+):\s*(.+)$").ok()?;
    
    for caps in re.captures_iter(text) {
        if let (Some(key), Some(value)) = (caps.get(1), caps.get(2)) {
            let key_str = key.as_str().trim().to_lowercase().replace(' ', "_");
            let value_str = value.as_str().trim();
            
            // Try to parse value as number or boolean
            if let Ok(num) = value_str.parse::<f64>() {
                map.insert(key_str, JsonValue::Number(serde_json::Number::from_f64(num)?));
            } else if let Some(bool_val) = extract_boolean(value_str) {
                map.insert(key_str, JsonValue::Bool(bool_val));
            } else {
                map.insert(key_str, JsonValue::String(value_str.to_string()));
            }
        }
    }
    
    if !map.is_empty() {
        Some(JsonValue::Object(map))
    } else {
        None
    }
}

/// Parses a model string like "provider:model_name" and returns the provider and model.
/// Defaults to "gemini" if no provider is specified.
fn parse_model_string(model_str: &str) -> (String, String) {
    match model_str.split_once(':') {
        Some((provider, model)) => (provider.to_string(), model.to_string()),
        None => ("gemini".to_string(), model_str.to_string()),
    }
}

/// Call the appropriate provider based on the provider name
async fn call_provider(
    config: &AppConfig,
    client: &reqwest::Client,
    provider_name: &str,
    model_name: &str,
    prompt: &str,
) -> Result<(String, u64, TokenUsage)> {
    match provider_name {
        "anthropic" => {
            let anthropic_config = config.anthropic.as_ref()
                .ok_or_else(|| EvalError::ProviderNotFound("anthropic".to_string()))?;
            let provider = AnthropicProvider::new(client.clone(), anthropic_config.clone());
            provider.generate(model_name, prompt).await
        }
        "gemini" => {
            let gemini_config = config.gemini.as_ref()
                .ok_or_else(|| EvalError::ProviderNotFound("gemini".to_string()))?;
            let provider = GeminiProvider::new(client.clone(), gemini_config.clone());
            provider.generate(model_name, prompt).await
        }
        "ollama" => {
            let ollama_config = config.ollama.as_ref()
                .ok_or_else(|| EvalError::ProviderNotFound("ollama".to_string()))?;
            let provider = OllamaProvider::new(client.clone(), ollama_config.clone());
            provider.generate(model_name, prompt).await
        }
        "openai" => {
            let openai_config = config.openai.as_ref()
                .ok_or_else(|| EvalError::ProviderNotFound("openai".to_string()))?;
            let provider = OpenAIProvider::new(client.clone(), openai_config.clone());
            provider.generate(model_name, prompt).await
        }
        _ => Err(EvalError::ProviderNotFound(provider_name.to_string())),
    }
}

/// Run a single eval with comprehensive LLM-as-a-judge evaluation
pub async fn run_eval(
    config: &AppConfig,
    eval: &EvalConfig,
    client: &reqwest::Client,
) -> Result<EvalResult> {
    let rendered_eval = eval.render()?;
    let eval_start = Instant::now();
    let separator = "=".repeat(60);
    
    println!("\n{}", separator);
    println!("🎯 Starting evaluation for model: {}", rendered_eval.model);
    println!("{}\n", separator);

    // Step 1: Call the target model
    let (provider_name, model_name) = parse_model_string(&rendered_eval.model);
    
    println!("📝 Prompt: {}", rendered_eval.prompt);
    
    let (model_output_str, latency_ms, token_usage) = match call_provider(
        config,
        client,
        &provider_name,
        &model_name,
        &rendered_eval.prompt,
    ).await {
        Ok(result) => result,
        Err(e @ EvalError::ProviderNotFound(_)) => {
            eprintln!("❌ Provider not configured: {}", e);
            return Err(e);
        }
        Err(e) => {
            eprintln!("❌ Model failed: {}", e);
            return Err(EvalError::ModelFailure {
                model: rendered_eval.model.clone(),
            });
        }
    };

    println!("\n✅ Model Output ({}ms):\n{}\n", latency_ms, &model_output_str);
    
    // Parse the output into structured format
    let parsed_output = parse_model_output(&model_output_str);
    if let Some(ref parsed) = parsed_output {
        println!("📊 Parsed Output: {}", serde_json::to_string_pretty(parsed).unwrap_or_else(|_| "Unable to display".to_string()));
    } else {
        println!("⚠️  Could not parse output into structured format");
    }

    // Step 2: Run judge evaluation if expected output provided
    let mut judge_latency_ms = None;
    let mut judge_token_usage = None;
    let judge_result = if let (Some(expected), Some(judge_model)) =
        (&rendered_eval.expected, &rendered_eval.judge_model) {
        
        println!("⚖️  Running judge evaluation with model: {}", judge_model);
        
        let judge_prompt = create_judge_prompt(
            expected,
            &model_output_str,
            rendered_eval.criteria.as_deref()
        );

        let (judge_provider_name, judge_model_name) = parse_model_string(judge_model);
        
        let judge_result = call_provider(
            config,
            client,
            &judge_provider_name,
            &judge_model_name,
            &judge_prompt,
        ).await;

        match judge_result {
            Ok((judge_response, judge_latency, tokens)) => {
                judge_latency_ms = Some(judge_latency);
                judge_token_usage = Some(tokens);
                println!("\n⚖️  Judge Response ({}ms):\n{}\n", judge_latency, &judge_response);
                
                let mut result = parse_judge_response(&judge_response);
                result.judge_model = judge_model.clone();
                
                match result.verdict {
                    JudgeVerdict::Pass => println!("✅ VERDICT: PASS"),
                    JudgeVerdict::Fail => println!("❌ VERDICT: FAIL"),
                    JudgeVerdict::Uncertain => println!("⚠️  VERDICT: UNCERTAIN"),
                }
                
                Some(result)
            }
            Err(e) => {
                let judge_error = EvalError::JudgeFailure {
                    model: judge_model.clone(),
                    source: Box::new(e),
                };
                eprintln!("⚠️  Judge evaluation failed: {}", judge_error);
                None
            }
        }
    } else {
        println!("ℹ️  No judge evaluation (no expected output or judge model specified)");
        None
    };

    let total_latency_ms = eval_start.elapsed().as_millis() as u64;
    println!("⏱️  Total evaluation time: {}ms", total_latency_ms);
    println!("\n{}\n", separator);

    Ok(EvalResult {
        model: rendered_eval.model.clone(),
        prompt: rendered_eval.prompt.clone(),
        model_output: model_output_str.to_string(),
        parsed_output,
        expected: rendered_eval.expected.clone(),
        judge_result,
        timestamp: chrono::Utc::now().to_rfc3339(),
        latency_ms,
        judge_latency_ms,
        token_usage: if token_usage.input_tokens.is_some() || token_usage.output_tokens.is_some() { 
            Some(token_usage) 
        } else { 
            None 
        },
        judge_token_usage,
        total_latency_ms,
    })
}

/// Run multiple evals and aggregate results concurrently
pub async fn run_batch_evals(
    config: &AppConfig,
    evals: Vec<EvalConfig>,
    client: &reqwest::Client,
) -> Vec<Result<EvalResult>> {
    let batch_start = Instant::now();
    let total_evals = evals.len();

    let futures: Vec<_> = evals
        .iter()
        .map(|eval| run_eval(config, eval, client))
        .collect();

    let results = future::join_all(futures).await;

    let batch_total_ms = batch_start.elapsed().as_millis() as u64;
    println!("\n📊 Batch of {} completed concurrently in {}ms", total_evals, batch_total_ms);

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_model_string_with_provider() {
        let (provider, model) = parse_model_string("anthropic:claude-sonnet-4");
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-sonnet-4");
    }

    #[test]
    fn test_parse_model_string_default_provider() {
        let (provider, model) = parse_model_string("gemini-1.5-flash");
        assert_eq!(provider, "gemini");
        assert_eq!(model, "gemini-1.5-flash");
    }

    #[test]
    fn test_judge_verdict_display() {
        assert_eq!(JudgeVerdict::Pass.to_string(), "Pass");
        assert_eq!(JudgeVerdict::Fail.to_string(), "Fail");
        assert_eq!(JudgeVerdict::Uncertain.to_string(), "Uncertain");
    }

    #[test]
    fn test_parse_json_output() {
        let output = r#"{"name": "Alice", "age": 30}"#;
        let parsed = parse_model_output(output);
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap()["name"], "Alice");
    }

    #[test]
    fn test_parse_json_code_block() {
        let output = r#"
Here's the result:
```json
{"answer": 42}
```
"#;
        let parsed = parse_model_output(output);
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap()["answer"], 42);
    }

    #[test]
    fn test_parse_number() {
        let output = "The answer is 42";
        let parsed = parse_model_output(output);
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap()["answer"], 42);
    }

    #[test]
    fn test_parse_boolean() {
        let output = "Yes, that is correct";
        let parsed = parse_model_output(output);
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap()["answer"], true);
    }

    #[test]
    fn test_parse_multiple_choice() {
        let output = "The answer is B";
        let parsed = parse_model_output(output);
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap()["answer"], "B");
    }

//     #[test]
//     fn test_parse_key_value_pairs() {
//         let output = r#"
// Name: John Doe
// Age: 25
// City: New York
// "#;
//         let parsed = parse_model_output(output);
//         assert!(parsed.is_some());
//         let obj = parsed.unwrap();
//         assert_eq!(obj["name"], "John Doe");
//         assert_eq!(obj["age"], 25);
//     }

    #[test]
    fn test_unparseable_output() {
        let output = "This is just random text without structure";
        let parsed = parse_model_output(output);
        assert!(parsed.is_none());
    }
}
