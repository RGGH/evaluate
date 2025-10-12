// src/runner.rs
use crate::config::EvalConfig;
use reqwest::Client;
use serde_json::json;
use anyhow::{Result, Context};

/// Run a single eval using Gemini directly
/// Sends the prompt to the target model and optionally asks the judge model
pub async fn run_eval(app: &crate::config::AppConfig, eval: &EvalConfig) -> Result<()> {
    let client = Client::new();
    
    // Step 1: Call the target model
    let model_url = format!(
        "{}/v1beta/models/{}:generateContent",
        app.api_base.trim_end_matches('/'),
        eval.model
    );
    
    println!("📡 Calling: {}", model_url);
    
    let body = json!({
        "contents": [{
            "parts": [{
                "text": eval.prompt
            }]
        }]
    });
    
    let resp = client.post(&model_url)
        .header("x-goog-api-key", &app.api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("Failed to send request")?;
    
    let status = resp.status();
    println!("📥 Response status: {}", status);
    
    let response_text = resp.text().await?;
    println!("📄 Response body: {}", response_text);
    
    if response_text.is_empty() {
        anyhow::bail!("Empty response from API");
    }
    
    let response_json: serde_json::Value = serde_json::from_str(&response_text)
        .context("Failed to parse response as JSON")?;
    
    // Check for error in response
    if let Some(error) = response_json.get("error") {
        anyhow::bail!("API Error: {}", error);
    }
    
    let model_output = response_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("No response")
        .trim()
        .to_string();
    
    println!("🔹 Model Output [{}]: {}", eval.model, model_output);
    
    // Step 2: Optional judge model for semantic equivalence
    if let (Some(expected), Some(judge_model)) = (&eval.expected, &eval.judge_model) {
        let judge_url = format!(
            "{}/v1beta/models/{}:generateContent",
            app.api_base.trim_end_matches('/'),
            judge_model
        );
        
        let judge_prompt = format!(
            "Compare the following two texts. Do they have the same meaning?\n\
             Expected: {}\nModel Output: {}\nReply only with 'YES' or 'NO'.",
            expected, model_output
        );
        
        let judge_body = json!({
            "contents": [{
                "parts": [{
                    "text": judge_prompt
                }]
            }]
        });
        
        let judge_resp = client.post(&judge_url)
            .header("x-goog-api-key", &app.api_key)
            .header("Content-Type", "application/json")
            .json(&judge_body)
            .send()
            .await?;
        
        let judge_status = judge_resp.status();
        let judge_text = judge_resp.text().await?;
        
        if !judge_status.is_success() {
            println!("⚠️ Judge request failed: {} - {}", judge_status, judge_text);
            return Ok(());
        }
        
        let judge_json: serde_json::Value = serde_json::from_str(&judge_text)?;
        let judge_result = judge_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("No judge response")
            .trim();
        
        println!("🔸 Judge Output [{}]: {}", judge_model, judge_result);
    }
    
    Ok(())
}