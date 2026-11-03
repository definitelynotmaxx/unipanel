//! Wire types for the public API.
//!
//! These structs are the contract the generated TypeScript client is built from,
//! so changes here must keep `openapi.json` in sync.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

use unipanel_storage::User;

/// A user as exposed over HTTP.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct UserDto {
    /// Account id.
    pub id: String,
    /// Login name.
    pub username: String,
    /// Display name.
    pub display_name: String,
    /// Role name (`admin` or `member`).
    pub role: String,
    /// Creation timestamp (RFC 3339).
    pub created_at: String,
}

impl From<&User> for UserDto {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.clone(),
            display_name: user.display_name.clone(),
            role: user.role.clone(),
            created_at: format_timestamp(user.created_at),
        }
    }
}

/// Formats a timestamp for JSON output.
#[must_use]
pub fn format_timestamp(value: OffsetDateTime) -> String {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| value.unix_timestamp().to_string())
}

/// Boot information fetched before anything else renders.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BootstrapResponse {
    /// Whether the first-run wizard must be shown.
    pub setup_required: bool,
    /// Whether the request carried a valid session.
    pub authenticated: bool,
    /// Instance locale, or the suggested one while unconfigured.
    pub locale: String,
    /// Locale guessed from `Accept-Language`, used to preselect the wizard.
    pub suggested_locale: String,
    /// Instance name for the document title.
    pub instance_name: String,
    /// Server version.
    pub version: String,
    /// Signed-in user, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserDto>,
    /// CSRF token to send with unsafe requests.
    pub csrf_token: String,
}

/// First-run setup payload.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SetupRequest {
    /// `en` or `de`; immutable afterwards.
    pub locale: String,
    /// Administrator login name.
    pub username: String,
    /// Administrator display name.
    pub display_name: String,
    /// Administrator password (plain, over TLS in production).
    pub password: String,
    /// Optional instance name.
    #[serde(default)]
    pub instance_name: Option<String>,
}

/// Login payload.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// Login name.
    pub username: String,
    /// Password.
    pub password: String,
}

/// Result of a successful login, setup or session lookup.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionResponse {
    /// Signed-in account.
    pub user: UserDto,
    /// CSRF token bound to the new session.
    pub csrf_token: String,
}

/// Instance information for the app shell.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemInfoResponse {
    /// Server version.
    pub version: String,
    /// Instance locale.
    pub locale: String,
    /// Instance name.
    pub instance_name: String,
    /// Server start time (RFC 3339).
    pub started_at: String,
    /// Seconds since start.
    pub uptime_seconds: i64,
    /// Storage schema version this build expects.
    pub schema_version: i64,
    /// Database backend description.
    pub database: String,
}

/// Liveness/readiness payload.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Always `ok` when the process can answer.
    pub status: String,
    /// Server version.
    pub version: String,
    /// Seconds since start.
    pub uptime_seconds: i64,
    /// Whether the database answered a probe query.
    pub database_ok: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_dto_formats_timestamps_as_rfc3339() {
        let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("time");
        assert_eq!(format_timestamp(now), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn setup_request_defaults_instance_name() {
        let request: SetupRequest = serde_json::from_value(serde_json::json!({
            "locale": "en",
            "username": "admin",
            "display_name": "Ada",
            "password": "a-long-password"
        }))
        .expect("parse");
        assert!(request.instance_name.is_none());
    }
}
