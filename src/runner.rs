// src/runner.rs
use crate::config::{AppConfig, EvalConfig};
use crate::errors::{EvalError, Result};
use crate::providers::{anthropic::AnthropicProvider, gemini::GeminiProvider, ollama::OllamaProvider, openai::OpenAIProvider, LlmProvider};
use futures::future;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvalResult {
    pub model: String,
    pub prompt: String,
    pub model_output: String,
    pub expected: Option<String>,
    pub judge_result: Option<JudgeResult>,
    pub timestamp: String,
    pub latency_ms: u64,
    pub judge_latency_ms: Option<u64>,
    pub total_latency_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JudgeResult {
    pub judge_model: String,
    pub verdict: JudgeVerdict,
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
) -> Result<(String, u64)> {
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
    
    let (model_output_str, latency_ms) = call_provider(
        config,
        client,
        &provider_name,
        &model_name,
        &rendered_eval.prompt,
    ).await.map_err(|e| {
        eprintln!("❌ Model failed: {}", e);
        EvalError::ModelFailure {
            model: rendered_eval.model.clone(),
        }
    })?;

    println!("\n✅ Model Output ({}ms):\n{}\n", latency_ms, &model_output_str);

    // Step 2: Run judge evaluation if expected output provided
    let mut judge_latency_ms = None;
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
            Ok((judge_response, judge_latency)) => {
                judge_latency_ms = Some(judge_latency);
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
        expected: rendered_eval.expected.clone(),
        judge_result,
        timestamp: chrono::Utc::now().to_rfc3339(),
        latency_ms,
        judge_latency_ms,
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
