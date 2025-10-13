// src/main.rs
mod api;
mod config;
mod errors;
mod runner;

use actix_web::{middleware, App, HttpServer};
use actix_files as fs;
use actix_cors::Cors;
use api::{configure_routes, AppState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let app_config = config::AppConfig::from_file("src/config.toml")
        .expect("Failed to load config");
    
    let state = AppState::new(app_config);
    
    println!("🚀 Starting server at http://127.0.0.1:8080");
    println!("📊 Frontend available at http://127.0.0.1:8080");
    println!("🔌 API available at http://127.0.0.1:8080/api/v1");
    
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::permissive();
        
        App::new()
            .app_data(actix_web::web::Data::new(state.clone()))
            .wrap(cors)
            .wrap(middleware::Logger::default())
            // API routes (register first so they take precedence)
            .configure(configure_routes)
            // Serve static files from the static directory
            .service(fs::Files::new("/static", "./static").show_files_listing())
            // Serve index.html at root
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}