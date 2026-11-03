//! Instance information and health.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse as _, Response};
use crate::dto::{HealthResponse, SystemInfoResponse};
use crate::error::ApiError;
use crate::extract::CurrentSession;
use crate::state::AppState;

/// Unauthenticated liveness probe used by Docker and the installer.
pub async fn healthz(State(state): State<AppState>) -> Response {
    let database_ok = state.db.health().await.is_ok();
    if !database_ok {
        return (StatusCode::SERVICE_UNAVAILABLE, "degraded\n").into_response();
    }
    (
        StatusCode::OK,
        [
            ("content-type", "text/plain; charset=utf-8"),
            ("cache-control", "no-store"),
        ],
        "ok\n",
    )
        .into_response()
}

/// JSON health details for the UI status pill.
///
/// # Errors
/// Returns [`ErrorKey::Internal`] when the database probe fails.
#[utoipa::path(
    get,
    path = "/api/v1/system/health",
    tag = "system",
    responses((status = 200, body = HealthResponse))
)]
pub async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    state
        .db
        .health()
        .await
        .map_err(|err| ApiError::internal(&err))?;
    Ok(Json(HealthResponse {
        status: "ok".to_owned(),
        version: unipanel_core::version().to_owned(),
        uptime_seconds: state.uptime_seconds(),
        database_ok: true,
    }))
}

/// Instance metadata for the app shell.
///
/// # Errors
/// Returns [`ErrorKey::Internal`] when instance settings cannot be read.
#[utoipa::path(
    get,
    path = "/api/v1/system/info",
    tag = "system",
    responses(
        (status = 200, body = SystemInfoResponse),
        (status = 401, body = crate::error::ApiErrorBody)
    )
)]
pub async fn info(
    State(state): State<AppState>,
    CurrentSession { user, .. }: CurrentSession,
) -> Result<Json<SystemInfoResponse>, ApiError> {
    let _ = user;
    let locale = state.instance_locale().await.unwrap_or_default();
    let instance_name = state
        .db
        .instance_name()
        .await
        .map_err(|err| ApiError::internal(&err))?;
    Ok(Json(SystemInfoResponse {
        version: unipanel_core::version().to_owned(),
        locale: locale.as_str().to_owned(),
        instance_name,
        started_at: crate::dto::format_timestamp(state.started_at),
        uptime_seconds: state.uptime_seconds(),
        schema_version: unipanel_core::SCHEMA_VERSION,
        database: state
            .config
            .database_path()
            .map_or_else(|| "external".to_owned(), |_| "sqlite".to_owned()),
    }))
}
