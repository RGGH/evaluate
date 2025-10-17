// src/database.rs - Fixed directory creation
use crate::models::{ApiResponse, EvalResult};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::path::PathBuf;

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let db_path = get_db_path()?;
    
    // Create parent directory BEFORE attempting to connect
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(sqlx::Error::Io)?;
        println!("✅ Created database directory: {}", parent.display());
    }
    
    // Ensure the path is absolute and properly formatted
    let absolute_path = if db_path.is_relative() {
        std::env::current_dir()
            .map_err(sqlx::Error::Io)?
            .join(&db_path)
    } else {
        db_path.clone()
    };
    
    println!("📦 Database file path: {}", absolute_path.display());
    
    // SQLite connection string needs to be properly formatted
    let db_url = format!("sqlite://{}?mode=rwc", absolute_path.display());
    println!("📦 Connecting to: {}", db_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    println!("✅ Database connected successfully");

    // Run migrations from the migrations directory
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    println!("✅ Database migrations completed");

    Ok(pool)
}

fn get_db_path() -> Result<PathBuf, sqlx::Error> {
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| sqlx::Error::Configuration("DATABASE_URL must be set".into()))?;
    
    let db_path_str = db_url.strip_prefix("sqlite:").ok_or_else(|| {
        sqlx::Error::Configuration("DATABASE_URL must start with 'sqlite:'".into())
    })?;
    
    Ok(PathBuf::from(db_path_str))
}

pub async fn save_evaluation(pool: &SqlitePool, response: &ApiResponse) -> Result<(), sqlx::Error> {
    let id = &response.id;
    let status = response.status.to_string();

    let (
        model,
        prompt,
        model_output,
        expected,
        judge_model,
        judge_verdict,
        judge_reasoning,
        error_message,
        latency_ms,
        judge_latency_ms,
        input_tokens,
        output_tokens,
        judge_input_tokens,
        judge_output_tokens,
        created_at,
    ) = match &response.result {
            EvalResult::Success(res) => (
                Some(res.model.clone()),
                Some(res.prompt.clone()),
                Some(res.model_output.clone()),
                res.expected.clone(),
                res.judge_result.as_ref().map(|j| j.judge_model.clone()),
                res.judge_result.as_ref().map(|j| j.verdict.to_string()),
                res.judge_result.as_ref().map(|j| j.reasoning.clone()),
                None,
                // Model Latency & Tokens
                Some(res.latency_ms as i64),
                res.judge_latency_ms.map(|l| l as i64),
                res.token_usage.as_ref().and_then(|u| u.input_tokens.map(|t| t as i64)),
                res.token_usage.as_ref().and_then(|u| u.output_tokens.map(|t| t as i64)),
                res.judge_token_usage.as_ref().and_then(|u| u.input_tokens.map(|t| t as i64)),
                res.judge_token_usage.as_ref().and_then(|u| u.output_tokens.map(|t| t as i64)),
                Some(res.timestamp.clone()),
            ),
            EvalResult::Error(err) => (
                // All fields are None except error_message
                // The order here must match the tuple declaration above
                None, // model
                None, // prompt
                None, // model_output
                None, // expected
                None, // judge_model
                None, // judge_verdict
                None, // judge_reasoning
                Some(err.message.clone()),
                None, // latency_ms
                None, // judge_latency_ms
                None, // input_tokens
                None, // output_tokens
                None, // judge_input_tokens
                None, // judge_output_tokens
                None, // created_at
            ),
        };

    let created_at_str = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    sqlx::query(
        r#" 
        INSERT INTO evaluations (id, status, model, prompt, model_output, expected, judge_model, judge_verdict, judge_reasoning, error_message, latency_ms, judge_latency_ms, input_tokens, output_tokens, judge_input_tokens, judge_output_tokens, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(id)
    .bind(&status)
    .bind(&model)
    .bind(&prompt)
    .bind(&model_output)
    .bind(&expected)
    .bind(&judge_model)
    .bind(&judge_verdict)
    .bind(&judge_reasoning)
    .bind(&error_message)
    .bind(latency_ms)
    .bind(judge_latency_ms)
    .bind(input_tokens)
    .bind(output_tokens)
    .bind(judge_input_tokens)
    .bind(judge_output_tokens)
    .bind(&created_at_str)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all_evaluations(pool: &SqlitePool) -> Result<Vec<HistoryEntry>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, status, model, prompt, model_output, expected, judge_model, judge_verdict, judge_reasoning, error_message, latency_ms, judge_latency_ms, input_tokens, output_tokens, judge_input_tokens, judge_output_tokens, created_at
        FROM evaluations
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| HistoryEntry {
        id: row.get(0),
        status: row.get(1),
        model: row.get(2),
        prompt: row.get(3),
        model_output: row.get(4),
        expected: row.get(5),
        judge_model: row.get(6),
        judge_verdict: row.get(7),
        judge_reasoning: row.get(8),
        error_message: row.get(9),
        latency_ms: row.get(10),
        judge_latency_ms: row.get(11),
        input_tokens: row.get(12),
        output_tokens: row.get(13),
        judge_input_tokens: row.get(14),
        judge_output_tokens: row.get(15),
        created_at: row.get(16),
    }).collect())
}

#[derive(serde::Serialize, Clone)]
pub struct HistoryEntry {
    pub id: String,
    pub status: Option<String>,
    pub model: Option<String>,
    pub prompt: Option<String>,
    pub model_output: Option<String>,
    pub expected: Option<String>,
    pub judge_model: Option<String>,
    pub judge_verdict: Option<String>,
    pub judge_reasoning: Option<String>,
    pub error_message: Option<String>,
    pub latency_ms: Option<i64>,
    pub judge_latency_ms: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub judge_input_tokens: Option<i64>,
    pub judge_output_tokens: Option<i64>,
    pub created_at: String,
}