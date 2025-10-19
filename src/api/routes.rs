// src/api/routes.rs
use actix_web::web;
use crate::api::handlers;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(handlers::health_check))
            .route("/models", web::get().to(handlers::get_models))
            .route("/ws", web::get().to(handlers::ws_handler))
            .service(
                web::scope("/evals")
                    .route("/run", web::post().to(handlers::run_eval))
                    .route("/batch", web::post().to(handlers::run_batch))
                    .route("/history", web::get().to(handlers::get_history))
                    .route("/{id}", web::get().to(handlers::get_eval))
                    .route("/{id}/status", web::get().to(handlers::get_status))
            )
            .service(
                web::scope("/experiments")
                    .route("", web::post().to(handlers::create_experiment))
                    .route("/{id}", web::get().to(handlers::get_experiment))
            )
            .service(
                web::scope("/judge-prompts")
                    .route("", web::get().to(handlers::get_all_judge_prompts))
                    .route("", web::post().to(handlers::create_judge_prompt))
                    .route("/active", web::get().to(handlers::get_active_judge_prompt))
                    .route("/active", web::put().to(handlers::set_active_judge_prompt))
                    .route("/{version}", web::get().to(handlers::get_judge_prompt_by_version))
            )
    );
}
