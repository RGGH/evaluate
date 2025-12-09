// src/database.rs

use crate::models::{ApiResponse, EvalResult};
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
};
use std::{
    path::PathBuf,
    str::FromStr,
};
use chrono::Utc; // Import chrono::Utc for use in structs and functions

// =======================================================
// Database Initialization
// =======================================================

/// Initializes the SQLite database connection pool.
/// It ensures the necessary parent directory exists and runs migrations.
pub async fn init_db() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let db_path = get_db_path_for_fs()?;
    
    // 1. Extract and create the directory FIRST
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            println!("💾 Database directory does not exist, creating: {}", parent.display());
            std::fs::create_dir_all(parent)?;
        }
    }
    
    // 2. Build the connection options using the original URL
    let db_url = std::env::var("DATABASE_URL")?;
    
    // We connect with the original URL, which sqlx handles, after ensuring the directory exists.
    let connection_options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true);

    println!("📦 Connecting to database using URL: {}", db_url);

    // 3. Connect and create pool
    let pool = SqlitePoolOptions::new()
        .connect_with(connection_options)
        .await?;
        
    // 4. Run migrations
    run_migrations(&pool).await?;

    println!("✅ Database connection successful and migrations applied.");
    
    Ok(pool)
}

/// Helper function to retrieve and clean the database file path from the DATABASE_URL 
/// for **File System (FS) operations** (i.e., directory creation).
fn get_db_path_for_fs() -> Result<PathBuf, sqlx::Error> {
    let db_url = std::env::var("DATABASE_URL").map_err(|e| {
        eprintln!("❌ DATABASE_URL environment variable not set: {}", e);
        sqlx::Error::Configuration("DATABASE_URL must be set".into())
    })?;
    
    // Remove the "sqlite:" prefix
    let db_path_str = db_url.strip_prefix("sqlite:").ok_or_else(|| {
        eprintln!("❌ DATABASE_URL must start with 'sqlite:' but got: {}", db_url);
        sqlx::Error::Configuration("DATABASE_URL must start with 'sqlite:'".into())
    })?;
    
    // We return a simple PathBuf, which is what std::fs::create_dir_all expects.
    Ok(PathBuf::from(db_path_str))
}

/// Runs the database migrations located in the 'migrations' directory.
async fn run_migrations(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let migrator = Migrator::new(std::path::Path::new("./migrations")).await?;
    migrator.run(pool).await?;
    Ok(())
}


// =======================================================
// Save and retrieve evaluations
// =======================================================

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
        judge_prompt_version,
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
            Some(res.latency_ms as i64),
            res.judge_latency_ms.map(|l| l as i64),
            res.token_usage.as_ref().and_then(|u| u.input_tokens.map(|t| t as i64)),
            res.token_usage.as_ref().and_then(|u| u.output_tokens.map(|t| t as i64)),
            res.judge_token_usage.as_ref().and_then(|u| u.input_tokens.map(|t| t as i64)),
            res.judge_token_usage.as_ref().and_then(|u| u.output_tokens.map(|t| t as i64)),
            Some(res.timestamp.clone()),
            res.judge_prompt_version,
        ),
        EvalResult::Error(err) => (
            None, None, None, None, None, None, None,
            Some(err.message.clone()),
            None, None, None, None, None, None, None, None,
        ),
    };

    let created_at_str = created_at.unwrap_or_else(|| Utc::now().to_rfc3339());

    sqlx::query(
        r#" 
        INSERT INTO evaluations (
            id, status, model, prompt, model_output, expected, 
            judge_model, judge_verdict, judge_reasoning, error_message, 
            latency_ms, judge_latency_ms, input_tokens, output_tokens, 
            judge_input_tokens, judge_output_tokens, created_at, judge_prompt_version
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
    .bind(judge_prompt_version)
    .execute(pool)
    .await?;

    Ok(())
}

// =======================================================
// Query evaluations
// =======================================================

pub async fn get_all_evaluations(pool: &SqlitePool) -> Result<Vec<HistoryEntry>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT 
            id, status, model, prompt, model_output, expected, 
            judge_model, judge_verdict, judge_reasoning, error_message, 
            latency_ms, judge_latency_ms, input_tokens, output_tokens, 
            judge_input_tokens, judge_output_tokens, created_at, judge_prompt_version
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
        judge_prompt_version: row.get(17),
    }).collect())
}

// =======================================================
// Structs (Needed for compilation)
// =======================================================

// NOTE: These structs must be defined here as they are not explicitly imported
// in the provided code snippet.
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
    pub judge_prompt_version: Option<i64>,
}

#[derive(serde::Serialize, Clone)]
pub struct JudgePrompt {
    pub version: i64,
    pub name: String,
    pub template: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

// =======================================================
// Judge prompt functions
// =======================================================

pub async fn get_all_judge_prompts(pool: &SqlitePool) -> Result<Vec<JudgePrompt>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT version, name, template, description, is_active, created_at
        FROM judge_prompts
        ORDER BY version DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| JudgePrompt {
        version: row.get(0),
        name: row.get(1),
        template: row.get(2),
        description: row.get(3),
        is_active: row.get(4),
        created_at: row.get(5),
    }).collect())
}

pub async fn get_active_judge_prompt(pool: &SqlitePool) -> Result<JudgePrompt, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT version, name, template, description, is_active, created_at
        FROM judge_prompts
        WHERE is_active = TRUE
        LIMIT 1
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(JudgePrompt {
        version: row.get(0),
        name: row.get(1),
        template: row.get(2),
        description: row.get(3),
        is_active: row.get(4),
        created_at: row.get(5),
    })
}

pub async fn get_judge_prompt_by_version(pool: &SqlitePool, version: i64) -> Result<JudgePrompt, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT version, name, template, description, is_active, created_at
        FROM judge_prompts
        WHERE version = ?
        "#
    )
    .bind(version)
    .fetch_one(pool)
    .await?;

    Ok(JudgePrompt {
        version: row.get(0),
        name: row.get(1),
        template: row.get(2),
        description: row.get(3),
        is_active: row.get(4),
        created_at: row.get(5),
    })
}

pub async fn create_judge_prompt(
    pool: &SqlitePool,
    name: String,
    template: String,
    description: Option<String>,
    set_active: bool,
) -> Result<JudgePrompt, sqlx::Error> {
    let created_at = Utc::now().to_rfc3339();
    
    let mut tx = pool.begin().await?;
    
    if set_active {
        sqlx::query("UPDATE judge_prompts SET is_active = FALSE")
            .execute(&mut *tx)
            .await?;
    }
    
    let result = sqlx::query(
        r#"
        INSERT INTO judge_prompts (name, template, description, is_active, created_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING version, name, template, description, is_active, created_at
        "#
    )
    .bind(&name)
    .bind(&template)
    .bind(&description)
    .bind(set_active)
    .bind(&created_at)
    .fetch_one(&mut *tx)
    .await?;
    
    tx.commit().await?;
    
    Ok(JudgePrompt {
        version: result.get(0),
        name: result.get(1),
        template: result.get(2),
        description: result.get(3),
        is_active: result.get(4),
        created_at: result.get(5),
    })
}

pub async fn set_active_judge_prompt(pool: &SqlitePool, version: i64) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    // Check if the version exists
    sqlx::query("SELECT version FROM judge_prompts WHERE version = ?")
        .bind(version)
        .fetch_one(&mut *tx)
        .await?;
    
    // Deactivate all others
    sqlx::query("UPDATE judge_prompts SET is_active = FALSE")
        .execute(&mut *tx)
        .await?;
    
    // Activate the specified one
    sqlx::query("UPDATE judge_prompts SET is_active = TRUE WHERE version = ?")
        .bind(version)
        .execute(&mut *tx)
        .await?;
    
    tx.commit().await?;
    
    Ok(())
}

// =======================================================
// Prompt Version Management
// =======================================================

#[derive(serde::Serialize, Clone)]
pub struct PromptVersion {
    pub version: i64,
    pub name: String,
    pub prompt_template: String,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub metadata: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub created_by: Option<String>,
}

pub async fn get_all_prompt_versions(pool: &SqlitePool) -> Result<Vec<PromptVersion>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT version, name, prompt_template, description, tags, metadata, 
               is_active, created_at, created_by
        FROM prompt_versions
        ORDER BY version DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| PromptVersion {
        version: row.get(0),
        name: row.get(1),
        prompt_template: row.get(2),
        description: row.get(3),
        tags: row.get(4),
        metadata: row.get(5),
        is_active: row.get(6),
        created_at: row.get(7),
        created_by: row.get(8),
    }).collect())
}

pub async fn get_active_prompt_version(pool: &SqlitePool) -> Result<PromptVersion, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT version, name, prompt_template, description, tags, metadata,
               is_active, created_at, created_by
        FROM prompt_versions
        WHERE is_active = TRUE
        LIMIT 1
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(PromptVersion {
        version: row.get(0),
        name: row.get(1),
        prompt_template: row.get(2),
        description: row.get(3),
        tags: row.get(4),
        metadata: row.get(5),
        is_active: row.get(6),
        created_at: row.get(7),
        created_by: row.get(8),
    })
}

pub async fn create_prompt_version(
    pool: &SqlitePool,
    name: String,
    prompt_template: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
    set_active: bool,
) -> Result<PromptVersion, sqlx::Error> {
    let created_at = Utc::now().to_rfc3339();
    let tags_json = tags.map(|t| serde_json::to_string(&t).unwrap());
    
    let mut tx = pool.begin().await?;
    
    if set_active {
        sqlx::query("UPDATE prompt_versions SET is_active = FALSE")
            .execute(&mut *tx)
            .await?;
    }
    
    let result = sqlx::query(
        r#"
        INSERT INTO prompt_versions (name, prompt_template, description, tags, is_active, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING version, name, prompt_template, description, tags, metadata, is_active, created_at, created_by
        "#
    )
    .bind(&name)
    .bind(&prompt_template)
    .bind(&description)
    .bind(&tags_json)
    .bind(set_active)
    .bind(&created_at)
    .fetch_one(&mut *tx)
    .await?;
    
    tx.commit().await?;
    
    Ok(PromptVersion {
        version: result.get(0),
        name: result.get(1),
        prompt_template: result.get(2),
        description: result.get(3),
        tags: result.get(4),
        metadata: result.get(5),
        is_active: result.get(6),
        created_at: result.get(7),
        created_by: result.get(8),
    })
}

pub async fn link_evaluation_to_prompt(
    pool: &SqlitePool,
    evaluation_id: &str,
    prompt_version: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO prompt_evaluations (evaluation_id, prompt_version)
        VALUES (?, ?)
        "#
    )
    .bind(evaluation_id)
    .bind(prompt_version)
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn get_prompt_version_stats(pool: &SqlitePool, version: i64) -> Result<PromptStats, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT 
            COUNT(*) as total_evals,
            SUM(CASE WHEN judge_verdict = 'Pass' THEN 1 ELSE 0 END) as passed,
            AVG(latency_ms) as avg_latency,
            AVG(judge_latency_ms) as avg_judge_latency
        FROM evaluations e
        JOIN prompt_evaluations pe ON e.id = pe.evaluation_id
        WHERE pe.prompt_version = ?
        "#
    )
    .bind(version)
    .fetch_one(pool)
    .await?;
    
    Ok(PromptStats {
        version,
        total_evaluations: row.get(0),
        passed: row.get(1),
        avg_latency_ms: row.get::<Option<f64>, _>(2).unwrap_or(0.0),
        avg_judge_latency_ms: row.get::<Option<f64>, _>(3).unwrap_or(0.0),
    })
}

#[derive(serde::Serialize)]
pub struct PromptStats {
    pub version: i64,
    pub total_evaluations: i64,
    pub passed: i64,
    pub avg_latency_ms: f64,
    pub avg_judge_latency_ms: f64,
}
