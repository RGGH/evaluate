// src/api/routes.rs
use actix_web::web;
use super::handlers;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(handlers::health_check))
            .service(
                web::scope("/evals")
                    .route("/run", web::post().to(handlers::run_eval))
                    .route("/batch", web::post().to(handlers::run_batch))
                    .route("/{id}", web::get().to(handlers::get_eval))
                    .route("/{id}/status", web::get().to(handlers::get_status))
            )
            .service(
                web::scope("/experiments")
                    .route("", web::post().to(handlers::create_experiment))
                    .route("/{id}", web::get().to(handlers::get_experiment))
            )
    );
}
