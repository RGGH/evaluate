// src/runner.rs
use crate::config::{AppConfig, EvalConfig};
use crate::errors::{EvalError, Result};
use crate::providers::{anthropic::AnthropicProvider, gemini::GeminiProvider, ollama::OllamaProvider, openai::OpenAIProvider, LlmProvider, TokenUsage};
use futures::future;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::time::Instant;
use regex::Regex;
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvalResult {
    pub model: String,
    pub prompt: String,
    pub model_output: String,
    pub parsed_output: Option<JsonValue>,
    pub expected: Option<String>,
    pub judge_result: Option<JudgeResult>,
    pub timestamp: String,
    pub latency_ms: u64,
    pub judge_latency_ms: Option<u64>,
    pub token_usage: Option<TokenUsage>,
    pub judge_token_usage: Option<TokenUsage>,
    pub total_latency_ms: u64,
    pub judge_prompt_version: Option<i64>,  // NEW: Track which judge prompt was used
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

/// Default judge prompt template (fallback if database is unavailable)
fn get_default_judge_prompt_template() -> String {
    r#"You are an expert evaluator comparing two text outputs.

EVALUATION CRITERIA:
{{criteria}}

EXPECTED OUTPUT:
{{expected}}

ACTUAL OUTPUT:
{{actual}}

INSTRUCTIONS:
1. Carefully compare both outputs
2. Consider semantic equivalence, not just exact wording
3. Provide your verdict as the first line: "Verdict: PASS" or "Verdict: FAIL"
4. Then explain your reasoning in 2-3 sentences

Your evaluation:"#.to_string()
}

/// Render judge prompt template with actual values
fn render_judge_prompt(template: &str, expected: &str, actual: &str, criteria: Option<&str>) -> String {
    let base_criteria = criteria.unwrap_or(
        "The outputs should convey the same core meaning, even if phrased differently."
    );
    
    template
        .replace("{{criteria}}", base_criteria)
        .replace("{{expected}}", expected)
        .replace("{{actual}}", actual)
}

/// Load judge prompt from database or use default
async fn get_judge_prompt_template(db_pool: Option<&SqlitePool>) -> (String, Option<i64>) {
    if let Some(pool) = db_pool {
        match crate::database::get_active_judge_prompt(pool).await {
            Ok(prompt) => {
                println!("📋 Using judge prompt v{}: {}", prompt.version, prompt.name);
                return (prompt.template, Some(prompt.version));
            }
            Err(e) => {
                log::warn!("Could not load judge prompt from database: {}. Using default.", e);
            }
        }
    }
    
    println!("📋 Using default judge prompt template");
    (get_default_judge_prompt_template(), None)
}

/// Enhanced judge prompt with better structure (DEPRECATED - kept for compatibility)
#[deprecated(note = "Use get_judge_prompt_template and render_judge_prompt instead")]
fn create_judge_prompt(expected: &str, actual: &str, criteria: Option<&str>) -> String {
    let template = get_default_judge_prompt_template();
    render_judge_prompt(&template, expected, actual, criteria)
}

/// Attempt to parse model output into structured JSON
pub fn parse_model_output(raw_output: &str) -> Option<JsonValue> {
    if let Ok(json) = serde_json::from_str::<JsonValue>(raw_output) {
        return Some(json);
    }

    if let Some(json) = extract_json_code_block(raw_output) {
        if let Ok(parsed) = serde_json::from_str::<JsonValue>(&json) {
            return Some(parsed);
        }
    }

    if let Some(number) = extract_number(raw_output) {
        return Some(serde_json::json!({ "answer": number }));
    }

    if let Some(boolean) = extract_boolean(raw_output) {
        return Some(serde_json::json!({ "answer": boolean }));
    }

    if let Some(choice) = extract_multiple_choice(raw_output) {
        return Some(serde_json::json!({ "answer": choice }));
    }

    if let Some(structured) = extract_key_value_pairs(raw_output) {
        return Some(structured);
    }

    None
}

fn extract_json_code_block(text: &str) -> Option<String> {
    let re = Regex::new(r"```(?:json)?\s*\n([\s\S]*?)\n```").ok()?;
    re.captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

fn extract_number(text: &str) -> Option<f64> {
    let patterns = [
        r"(?:answer|result|solution)(?:\s+is)?[:\s]+(-?\d+\.?\d*)",
        r"^(-?\d+\.?\d*)$",
        r"\b(-?\d+\.?\d*)\b(?:\s*$)",
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

fn extract_multiple_choice(text: &str) -> Option<String> {
    let re = Regex::new(r"(?:answer|choice)(?:\s+is)?[:\s]+([A-Za-z])").ok()?;
    re.captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_uppercase())
}

fn extract_key_value_pairs(text: &str) -> Option<JsonValue> {
    let mut map = serde_json::Map::new();
    let re = Regex::new(r"(?m)^([A-Za-z\s]+):\s*(.+)$").ok()?;
    
    for caps in re.captures_iter(text) {
        if let (Some(key), Some(value)) = (caps.get(1), caps.get(2)) {
            let key_str = key.as_str().trim().to_lowercase().replace(' ', "_");
            let value_str = value.as_str().trim();
            
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

fn parse_model_string(model_str: &str) -> (String, String) {
    match model_str.split_once(':') {
        Some((provider, model)) => (provider.to_string(), model.to_string()),
        None => ("gemini".to_string(), model_str.to_string()),
    }
}

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
    run_eval_with_pool(config, eval, client, None).await
}

/// Run eval with optional database pool for judge prompt loading
pub async fn run_eval_with_pool(
    config: &AppConfig,
    eval: &EvalConfig,
    client: &reqwest::Client,
    db_pool: Option<&SqlitePool>,
) -> Result<EvalResult> {
    let rendered_eval = eval.render()?;
    let eval_start = Instant::now();
    let separator = "=".repeat(60);
    
    println!("\n{}", separator);
    println!("🎯 Starting evaluation for model: {}", rendered_eval.model);
    println!("{}\n", separator);

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
    
    let parsed_output = parse_model_output(&model_output_str);
    if let Some(ref parsed) = parsed_output {
        println!("📊 Parsed Output: {}", serde_json::to_string_pretty(parsed).unwrap_or_else(|_| "Unable to display".to_string()));
    } else {
        println!("⚠️  Could not parse output into structured format");
    }

    // Step 2: Run judge evaluation with dynamic prompt loading
    let mut judge_latency_ms = None;
    let mut judge_token_usage = None;
    let mut judge_prompt_version = None;
    
    let judge_result = if let (Some(expected), Some(judge_model)) =
        (&rendered_eval.expected, &rendered_eval.judge_model) {
        
        println!("⚖️  Running judge evaluation with model: {}", judge_model);
        
        // 🆕 Load judge prompt from database
        let (judge_prompt_template, version) = get_judge_prompt_template(db_pool).await;
        judge_prompt_version = version;
        
        // Render the template with actual values
        let judge_prompt = render_judge_prompt(
            &judge_prompt_template,
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
        judge_prompt_version,  // 🆕 Store which version was used
    })
}

/// Run multiple evals and aggregate results concurrently
pub async fn run_batch_evals(
    config: &AppConfig,
    evals: Vec<EvalConfig>,
    client: &reqwest::Client,
) -> Vec<Result<EvalResult>> {
    run_batch_evals_with_pool(config, evals, client, None).await
}

/// Run batch evals with optional database pool
pub async fn run_batch_evals_with_pool(
    config: &AppConfig,
    evals: Vec<EvalConfig>,
    client: &reqwest::Client,
    db_pool: Option<&SqlitePool>,
) -> Vec<Result<EvalResult>> {
    let batch_start = Instant::now();
    let total_evals = evals.len();

    let futures: Vec<_> = evals
        .iter()
        .map(|eval| run_eval_with_pool(config, eval, client, db_pool))
        .collect();

    let results = future::join_all(futures).await;

    let batch_total_ms = batch_start.elapsed().as_millis() as u64;
    println!("\n📊 Batch of {} completed concurrently in {}ms", total_evals, batch_total_ms);

    results
}