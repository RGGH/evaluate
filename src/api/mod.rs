// src/api/mod.rs
pub mod handlers;
mod routes;
mod state;

pub use routes::configure_routes;
pub use state::AppState;
