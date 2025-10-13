mod api;
mod config;
mod errors;
mod runner;
mod models;
mod database;

use actix_web::{middleware, App, HttpServer};
use actix_files as fs;
use actix_cors::Cors;
use api::{configure_routes, AppState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file - fail loudly if it doesn't exist
    if let Err(e) = dotenv::dotenv() {
        eprintln!("⚠️  Warning: Could not load .env file: {}", e);
        eprintln!("   Make sure DATABASE_URL is set in your environment");
    }
    
    // Debug: Check if DATABASE_URL is set
    match std::env::var("DATABASE_URL") {
        Ok(url) => println!("✅ DATABASE_URL set to: {}", url),
        Err(_) => eprintln!("❌ DATABASE_URL not set!"),
    }
    
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let app_config = config::AppConfig::from_file("src/config.toml")
        .expect("Failed to load config");
    
    let state = AppState::new(app_config).await;
    
    println!("🚀 Starting server at http://127.0.0.1:8080");
    println!("📊 Frontend available at http://127.0.0.1:8080");
    println!("🔌 API available at http://127.0.0.1:8080/api/v1");
    
    HttpServer::new(move || {
        let cors = Cors::permissive();
        
        App::new()
            .app_data(actix_web::web::Data::new(state.clone()))
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(configure_routes)
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}