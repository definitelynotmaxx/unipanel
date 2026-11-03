//! HTTP API, version 1.

pub mod auth;
pub mod bootstrap;
pub mod session;
pub mod setup;
pub mod system;

use axum::Json;
use axum::routing::{get, post};
use utoipa::OpenApi as _;

use crate::error::{ApiErrorBody, ApiErrorPayload};
use crate::state::AppState;

/// The API router, mounted under `/api/v1` by the application.
pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/bootstrap", get(bootstrap::bootstrap))
        .route("/setup", post(setup::setup))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/session", get(auth::session))
        .route("/system/health", get(system::health))
        .route("/system/info", get(system::info))
        .route("/openapi.json", get(openapi))
}

/// Machine-readable API description; the TypeScript client is generated from it.
#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "UniPanel API",
        version = env!("CARGO_PKG_VERSION"),
        description = "Self-hosted control panel API. All errors use the message-key envelope.",
        license(name = "MIT")
    ),
    paths(
        bootstrap::bootstrap,
        setup::setup,
        auth::login,
        auth::logout,
        auth::session,
        system::health,
        system::info
    ),
    components(schemas(
        crate::dto::BootstrapResponse,
        crate::dto::SetupRequest,
        crate::dto::LoginRequest,
        crate::dto::SessionResponse,
        crate::dto::UserDto,
        crate::dto::SystemInfoResponse,
        crate::dto::HealthResponse,
        ApiErrorBody,
        ApiErrorPayload
    ))
)]
pub struct ApiDoc;

async fn openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Serialises the OpenAPI document, used by `unipanel openapi`.
///
/// # Errors
/// Returns an error if the document cannot be serialised.
pub fn openapi_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&ApiDoc::openapi())
}

#[cfg(test)]
mod tests;
