//! Error responses.
//!
//! Every failure leaves the server as a message key plus parameters. The
//! frontend translates it, which keeps the API language-neutral and lets a
//! German instance read its own logs without storing prose in the database.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use unipanel_core::ErrorKey;
use unipanel_core::error::Message;
use utoipa::ToSchema;

/// Error payload shared by every endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiErrorBody {
    /// The translatable error.
    pub error: ApiErrorPayload,
    /// Correlation id, also present in the `x-request-id` response header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Message key and interpolation parameters.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiErrorPayload {
    /// Stable message key, e.g. `auth.required`.
    pub key: String,
    /// Values interpolated by the frontend.
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub params: serde_json::Map<String, serde_json::Value>,
}

/// An API error ready to be returned.
#[derive(Debug)]
pub struct ApiError {
    /// HTTP status code.
    pub status: StatusCode,
    /// Translatable message.
    pub message: Message,
    /// Correlation id.
    pub request_id: Option<String>,
}

impl ApiError {
    /// Builds an error with the default status for the given key.
    #[must_use]
    pub fn new(key: ErrorKey) -> Self {
        let status =
            StatusCode::from_u16(key.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        Self {
            status,
            message: Message::new(key),
            request_id: None,
        }
    }

    /// Builds an error with a custom message key and status.
    #[must_use]
    pub fn keyed(status: StatusCode, key: &str) -> Self {
        Self {
            status,
            message: Message::key(key),
            request_id: None,
        }
    }

    /// Adds an interpolation parameter.
    #[must_use]
    pub fn with(mut self, name: &str, value: impl Into<serde_json::Value>) -> Self {
        self.message = self.message.with(name, value);
        self
    }

    /// Attaches the request id from the current request.
    #[must_use]
    pub fn with_request_id(mut self, request_id: Option<&str>) -> Self {
        self.request_id = request_id.map(ToOwned::to_owned);
        self
    }

    /// Logs an internal failure and returns a generic error to the client.
    pub fn internal(err: &dyn std::error::Error) -> Self {
        tracing::error!(error = %err, "internal server error");
        Self::new(ErrorKey::Internal)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorBody {
            error: ApiErrorPayload {
                key: self.message.key,
                params: self.message.params,
            },
            request_id: self.request_id,
        };
        (self.status, Json(body)).into_response()
    }
}

impl From<unipanel_storage::StorageError> for ApiError {
    fn from(value: unipanel_storage::StorageError) -> Self {
        use unipanel_storage::StorageError;
        match &value {
            StorageError::Conflict(_) => Self::new(ErrorKey::RequestInvalid)
                .with("reason", "conflict"),
            StorageError::NotFound(_) => Self::new(ErrorKey::RequestNotFound),
            StorageError::AlreadySetup => Self::new(ErrorKey::SetupAlreadyCompleted),
            StorageError::InvalidUrl(_) | StorageError::Migrate(_) | StorageError::Database(_) => {
                Self::internal(&value)
            }
        }
    }
}

impl From<unipanel_auth::password::PasswordError> for ApiError {
    fn from(value: unipanel_auth::password::PasswordError) -> Self {
        Self::new(value.error_key())
    }
}

/// JSON extractor that reports rejections with the standard error envelope.
#[derive(Debug, Clone, Copy)]
pub struct ApiJson<T>(pub T);

impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection: JsonRejection| {
                ApiError::new(ErrorKey::RequestInvalid)
                    .with("reason", rejection.status().as_u16())
            })?;
        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn serialises_the_standard_envelope() {
        let error = ApiError::new(ErrorKey::AuthRequired)
            .with_request_id(Some("req-1"))
            .with("hint", "login");
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let bytes = to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("body");
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(json["error"]["key"], "auth.required");
        assert_eq!(json["error"]["params"]["hint"], "login");
        assert_eq!(json["request_id"], "req-1");
    }

    #[test]
    fn maps_storage_errors() {
        let error: ApiError = unipanel_storage::StorageError::AlreadySetup.into();
        assert_eq!(error.status, StatusCode::CONFLICT);
        let error: ApiError = unipanel_storage::StorageError::NotFound("user").into();
        assert_eq!(error.status, StatusCode::NOT_FOUND);
    }
}
