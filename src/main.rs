// src/main.rs
mod runner;
mod config;

use crate::runner::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let app_config = config::AppConfig::from_file("src/config.toml")?;
    
    for eval_path in &app_config.evals {
        let eval = config::EvalConfig::from_file(eval_path)?;
        run_eval(&app_config, &eval).await?;
    }
    
    Ok(())
}