//! UniPanel HTTP server: routing, middleware, embedded SPA and startup.

pub mod api;
pub mod app;
pub mod assets;
pub mod audit;
pub mod config;
pub mod cookies;
pub mod dto;
pub mod error;
pub mod extract;
pub mod middleware;
pub mod state;

#[cfg(test)]
pub mod test_support;

pub use app::{ServerError, build_app, open_database, run, serve, shutdown_signal};
pub use config::{Config, ConfigError};
pub use error::{ApiError, ApiErrorBody, ApiErrorPayload};
pub use state::AppState;
