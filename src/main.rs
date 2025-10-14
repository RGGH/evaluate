mod api;
mod config;
mod errors;
mod runner;
mod models;
mod database;
mod banner;
 
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, middleware, Responder};
use actix_cors::Cors;
use api::{configure_routes, AppState};
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAssets;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Print the startup banner
    banner::print_banner();

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
    
    let app_config = config::AppConfig::from_env()
        .expect("Failed to load app configuration from environment");
    
    let state = AppState::new(app_config).await;
    
    println!("🚀 Starting server...");
    println!("📊 Frontend available at http://127.0.0.1:8080");

    HttpServer::new(move || {
        let cors = Cors::permissive();
        
        App::new()
            .app_data(actix_web::web::Data::new(state.clone()))
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
        // trim leading '/'
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