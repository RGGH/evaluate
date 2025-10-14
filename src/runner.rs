// src/runner.rs
use crate::config::{AppConfig, EvalConfig};
use crate::errors::{EvalError, Result};
use futures::future;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Instant;

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Deserialize)]
struct Part {
    text: String,
}

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

/// Calls the Gemini API with a given prompt and returns the model's response text and latency.
async fn call_gemini_api(
    client: &Client,
    api_base: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
) -> Result<(String, u64)> {
    let url = format!(
        "{}/v1beta/models/{}:generateContent",
        api_base.trim_end_matches('/'),
        model
    );

    println!("📡 Calling: {} with model: {}", url, model);

    let body = json!({
        "contents": [{"parts": [{"text": prompt}]}]
    });

    let start = Instant::now();
    
    let resp = client
        .post(&url)
        .header("x-goog-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    let latency_ms = start.elapsed().as_millis() as u64;
    
    println!("📥 Response status: {} ({}ms)", status, latency_ms);

    if !status.is_success() {
        let error_body = resp.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
        return Err(EvalError::ApiError {
            status: status.as_u16(),
            body: error_body,
        });
    }

    let response_json: Value = resp.json().await?;

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

/// Parse judge response to extract verdict and reasoning
fn parse_judge_response(response: &str) -> JudgeResult {
    let response_lower = response.to_lowercase();
    
    // Try to extract structured response if available
    let verdict = if response_lower.contains("verdict: pass") || 
                     (response_lower.starts_with("yes") || response_lower.contains("yes, they")) {
        JudgeVerdict::Pass
    } else if response_lower.contains("verdict: fail") || 
              (response_lower.starts_with("no") || response_lower.contains("no, they")) {
        JudgeVerdict::Fail
    } else {
        JudgeVerdict::Uncertain
    };

    // Extract reasoning if present (look for lines after the verdict)
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

/// Run a single eval with comprehensive LLM-as-a-judge evaluation
pub async fn run_eval(
    app: &AppConfig, 
    eval: &EvalConfig, 
    client: &Client
) -> Result<EvalResult> {
    // Step 0: Render the eval config to substitute variables from metadata
    let rendered_eval = eval.render()?;

    let eval_start = Instant::now();
    let separator = "=".repeat(60);
    println!("\n{}", separator);
    println!("🎯 Starting evaluation for model: {}", rendered_eval.model);
    println!("{}\n", separator);

    // Step 1: Call the target model
    println!("📝 Prompt: {}", rendered_eval.prompt);
    
    let (model_output, latency_ms) = call_gemini_api(
        client,
        &app.api_base,
        &app.api_key,
        &rendered_eval.model,
        &rendered_eval.prompt,
    )
    .await
    .map_err(|e| {
        eprintln!("❌ Model failed: {}", e);
        EvalError::ModelFailure {
            model: rendered_eval.model.clone(),
        }
    })?;

    println!("\n✅ Model Output ({}ms):\n{}\n", latency_ms, model_output);

    // Step 2: Run judge evaluation if expected output provided
    let mut judge_latency_ms = None;
    let judge_result = if let (Some(expected), Some(judge_model)) =
        (&rendered_eval.expected, &rendered_eval.judge_model) {
        
        println!("⚖️  Running judge evaluation with model: {}", judge_model);
        
        let judge_prompt = create_judge_prompt(
            expected,
            &model_output,
            rendered_eval.criteria.as_deref()
        );

        match call_gemini_api(
            client,
            &app.api_base,
            &app.api_key,
            judge_model,
            &judge_prompt,
        )
        .await
        {
            Ok((judge_response, judge_latency)) => {
                judge_latency_ms = Some(judge_latency);
                println!("\n⚖️  Judge Response ({}ms):\n{}\n", judge_latency, judge_response);
                
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
        model_output,
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
    app: &AppConfig,
    evals: Vec<EvalConfig>,
    client: &Client,
) -> Vec<Result<EvalResult>> {
    let batch_start = Instant::now();
    let total_evals = evals.len();

    // list of futures and run them concurrently within the same task **
    let futures: Vec<_> = evals
        .iter()
        .map(|eval| run_eval(app, eval, client))
        .collect();

    let results = future::join_all(futures).await;

    let batch_total_ms = batch_start.elapsed().as_millis() as u64;
    println!("\n📊 Batch of {} completed concurrently in {}ms", total_evals, batch_total_ms);

    results
}