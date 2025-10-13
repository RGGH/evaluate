// src/main.rs
mod api;
mod config;
mod errors;
mod runner;

use actix_web::{middleware, App, HttpServer};
use api::{configure_routes, AppState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let app_config = config::AppConfig::from_file("src/config.toml")
        .expect("Failed to load config");
    
    let state = AppState::new(app_config);
    
    println!("🚀 Starting server at http://127.0.0.1:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(state.clone()))
            .wrap(middleware::Logger::default())
            .configure(configure_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
