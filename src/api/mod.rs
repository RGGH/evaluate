// src/api/mod.rs
pub mod routes;
pub mod handlers;
pub mod state;

pub use routes::configure_routes;
pub use state::AppState;
