// src/main.rs
mod config;
mod api;
mod errors;
mod providers;
mod runner;
mod models;
mod database;
mod banner;
 
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, middleware, Responder};
use actix_cors::Cors;
use api::{configure_routes, AppState};
use api::handlers::WsBroker;
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAssets;

/// Load environment variables with .env file taking priority over system env vars
fn load_env_with_priority() {
    // Load from .env file with override
    match dotenvy::from_filename_override(".env") {
        Ok(_) => println!("✅ Loaded .env file (overriding system environment variables)"),
        Err(e) => {
            eprintln!("⚠️  Warning: Could not load .env file: {}", e);
            eprintln!("   Using system environment variables only");
        }
    }
    
    // Verify critical env vars
    match std::env::var("DATABASE_URL") {
        Ok(url) => println!("✅ DATABASE_URL set to: {}", url),
        Err(_) => eprintln!("❌ DATABASE_URL not set!"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    banner::print_banner();

    load_env_with_priority();
    
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info,actix_web=warn"));
    
    let app_config = config::AppConfig::from_env()
        .expect("Failed to load app configuration from environment");
    
    let state = AppState::new(app_config).await;
    let ws_broker = WsBroker::new();
    
    println!("🚀 Starting server...");
    println!("📊 Frontend available at http://127.0.0.1:8080");
    println!("🔌 WebSocket endpoint at ws://127.0.0.1:8080/api/v1/ws");

    HttpServer::new(move || {
        let cors = Cors::permissive();
        
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(ws_broker.clone()))
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(configure_routes)
            .route("/{_:.*}", web::get().to(static_file_handler))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

async fn static_file_handler(req: HttpRequest) -> impl Responder {
    let path = if req.path() == "/" {
        "index.html"
    } else {
        &req.path()[1..]
    };

    match StaticAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            HttpResponse::Ok().content_type(mime.as_ref()).body(Cow::into_owned(content.data))
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}
