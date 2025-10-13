// src/database.rs - Fixed directory creation
use crate::models::{ApiResponse, EvalResult};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::path::PathBuf;

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let db_path = get_db_path()?;
    
    // Create parent directory BEFORE attempting to connect
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| sqlx::Error::Io(e))?;
        println!("✅ Created database directory: {}", parent.display());
    }
    
    // Ensure the path is absolute and properly formatted
    let absolute_path = if db_path.is_relative() {
        std::env::current_dir()
            .map_err(|e| sqlx::Error::Io(e))?
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
                Some(res.timestamp.clone()),
            ),
            EvalResult::Error(err) => (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(err.message.clone()),
                None,
            ),
        };

    let created_at_str = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    sqlx::query(
        r#"
        INSERT INTO evaluations (id, status, model, prompt, model_output, expected, judge_model, judge_verdict, judge_reasoning, error_message, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
    .bind(&created_at_str)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all_evaluations(pool: &SqlitePool) -> Result<Vec<HistoryEntry>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, status, model, prompt, model_output, expected, judge_model, judge_verdict, judge_reasoning, error_message, created_at
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
        created_at: row.get(10),
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
    pub created_at: String,
}