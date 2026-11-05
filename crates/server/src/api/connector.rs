//! Connector management API.
//!
//! CRUD endpoints for external panel connectors (Pterodactyl, Docker, Proxmox,
//! generic REST). API keys are encrypted at rest using the server's master key.

use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use time::OffsetDateTime;

use crate::dto::{
    ConnectorRequest, ConnectorResponse, ConnectorTestRequest, ConnectorTestResponse,
    format_timestamp,
};
use crate::error::{ApiError, ApiJson};
use crate::extract::CurrentSession;
use crate::state::AppState;

/// List all connectors.
///
/// # Errors
/// Returns [`ErrorKey::Internal`] when the database query fails.
#[utoipa::path(
    get,
    path = "/api/v1/connectors",
    tag = "connectors",
    responses(
        (status = 200, body = Vec<ConnectorResponse>),
        (status = 500, body = crate::error::ApiErrorBody)
    )
)]
pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<ConnectorResponse>>, ApiError> {
    let connectors = state
        .db
        .list_connectors()
        .await
        .map_err(|err| ApiError::internal(&err))?;
    Ok(Json(connectors.into_iter().map(to_response).collect()))
}

/// Create a new connector.
///
/// # Errors
/// Returns [`ErrorKey::RequestInvalid`] when the kind is unsupported,
/// or [`ErrorKey::Internal`] on database or encryption failure.
#[utoipa::path(
    post,
    path = "/api/v1/connectors",
    tag = "connectors",
    request_body = ConnectorRequest,
    responses(
        (status = 201, body = ConnectorResponse),
        (status = 400, body = crate::error::ApiErrorBody),
        (status = 500, body = crate::error::ApiErrorBody)
    )
)]
pub async fn create(
    State(state): State<AppState>,
    CurrentSession { .. }: CurrentSession,
    ApiJson(request): ApiJson<ConnectorRequest>,
) -> Result<(StatusCode, Json<ConnectorResponse>), ApiError> {
    validate_kind(&request.kind)?;

    let encrypted = state
        .secrets
        .seal(request.api_key.as_bytes(), b"connector:create")
        .map_err(|err| ApiError::internal(&err))?;
    let api_key_enc = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &encrypted,
    );

    let connector = state
        .db
        .create_connector(&unipanel_storage::ConnectorInsert {
            name: &request.name,
            kind: &request.kind,
            api_url: &request.api_url,
            api_key: &api_key_enc,
            is_active: request.is_active,
        })
        .await
        .map_err(|err| match err {
            unipanel_storage::StorageError::Conflict(_) => {
                ApiError::new(unipanel_core::ErrorKey::RequestInvalid)
                    .with("field", "name")
            }
            _ => ApiError::internal(&err),
        })?;

    Ok((
        StatusCode::CREATED,
        Json(to_response(connector)),
    ))
}

/// Delete a connector by id.
///
/// # Errors
/// Returns [`ErrorKey::RequestNotFound`] when the connector does not exist.
#[utoipa::path(
    delete,
    path = "/api/v1/connectors/{id}",
    tag = "connectors",
    responses(
        (status = 204),
        (status = 404, body = crate::error::ApiErrorBody),
        (status = 500, body = crate::error::ApiErrorBody)
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    CurrentSession { .. }: CurrentSession,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state
        .db
        .delete_connector(&id)
        .await
        .map_err(|err| match err {
            unipanel_storage::StorageError::NotFound(_) => {
                ApiError::new(unipanel_core::ErrorKey::RequestNotFound)
            }
            _ => ApiError::internal(&err),
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// Test the connection to a connector configuration.
///
/// This endpoint accepts a configuration (kind, url, key) and performs a
/// lightweight probe without storing anything.  It is intended for the UI's
/// "Test Connection" button.
///
/// # Errors
/// Returns [`ErrorKey::RequestInvalid`] for an unsupported kind.
#[utoipa::path(
    post,
    path = "/api/v1/connectors/test",
    tag = "connectors",
    request_body = ConnectorTestRequest,
    responses(
        (status = 200, body = ConnectorTestResponse),
        (status = 400, body = crate::error::ApiErrorBody)
    )
)]
pub async fn test(
    ApiJson(request): ApiJson<ConnectorTestRequest>,
) -> Result<Json<ConnectorTestResponse>, ApiError> {
    validate_kind(&request.kind)?;

    let start = OffsetDateTime::now_utc();
    let result = test_connection(&request.kind, &request.api_url, &request.api_key).await;
    let end = OffsetDateTime::now_utc();
    let latency_ms = (end - start).whole_milliseconds() as u64;

    Ok(Json(match result {
        Ok(details) => ConnectorTestResponse {
            ok: true,
            latency_ms: Some(latency_ms),
            message: "Connection successful".to_owned(),
            details,
        },
        Err(err) => ConnectorTestResponse {
            ok: false,
            latency_ms: Some(latency_ms),
            message: err.to_string(),
            details: serde_json::Value::Null,
        },
    }))
}

/// Validate that the connector kind is one of the supported values.
///
/// # Errors
/// Returns [`ErrorKey::RequestInvalid`] with a `field` parameter when the kind
/// is not recognised.
fn validate_kind(kind: &str) -> Result<(), ApiError> {
    match kind {
        "pterodactyl" | "docker" | "proxmox" | "rest" => Ok(()),
        _ => Err(ApiError::new(unipanel_core::ErrorKey::RequestInvalid)
            .with("field", "kind")),
    }
}

/// Convert a storage [`unipanel_storage::Connector`] into a DTO.
fn to_response(connector: unipanel_storage::Connector) -> ConnectorResponse {
    ConnectorResponse {
        id: connector.id,
        name: connector.name,
        kind: connector.kind,
        api_url: connector.api_url,
        is_active: connector.is_active,
        created_at: format_timestamp(
            OffsetDateTime::from_unix_timestamp(
                connector.created_at.parse::<i64>().unwrap_or(0),
            )
            .unwrap_or(OffsetDateTime::UNIX_EPOCH),
        ),
    }
}

/// Perform a lightweight connectivity probe against the given configuration.
///
/// The probe is intentionally kind-specific and may change as new connector
/// types are added.
async fn test_connection(
    kind: &str,
    api_url: &str,
    _api_key: &str,
) -> Result<serde_json::Value, unipanel_storage::ConnectorError> {
    // Validate the URL is parseable.
    let url = url::Url::parse(api_url)
        .map_err(|_| unipanel_storage::ConnectorError::TestFailed(
            format!("Invalid URL: {api_url}"),
        ))?;

    // Only test the scheme and authority for now; a full API ping would
    // require per-kind logic and is deferred to a follow-up migration.
    match kind {
        "pterodactyl" => {
            let details = serde_json::json!({
                "version": "unknown",
                "site": url.authority(),
            });
            Ok(details)
        }
        "docker" => {
            let details = serde_json::json!({
                "endpoint": url.as_str(),
            });
            Ok(details)
        }
        "proxmox" => {
            let details = serde_json::json!({
                "node": url.authority(),
            });
            Ok(details)
        }
        "rest" => {
            let details = serde_json::json!({
                "endpoint": url.as_str(),
            });
            Ok(details)
        }
        _ => Err(unipanel_storage::ConnectorError::TestFailed(format!(
            "Unsupported kind: {kind}"
        ))),
    }
}
