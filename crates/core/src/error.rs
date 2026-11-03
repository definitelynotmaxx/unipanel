//! User-facing errors as message keys.
//!
//! The backend never stores or returns rendered prose. Every error carries a
//! stable [`ErrorKey`] plus optional parameters; the frontend owns the actual
//! translations. Audit entries and notifications use the same mechanism, which
//! is why `ErrorKey` serialises as a plain string.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Stable error identifiers shared with the frontend translation bundles.
///
/// Keys are grouped by area (`auth.*`, `setup.*`, `request.*`) and are part of
/// the public API contract. Never renumber or reuse a key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKey {
    /// The request body was not valid JSON or failed validation.
    RequestInvalid,
    /// The referenced entity does not exist.
    RequestNotFound,
    /// The request is missing a valid CSRF token.
    RequestCsrf,
    /// The caller is not authenticated.
    AuthRequired,
    /// The caller is authenticated but not allowed to do this.
    AuthForbidden,
    /// Username or password is wrong (deliberately indistinguishable).
    AuthInvalidCredentials,
    /// Too many failed attempts, try again later.
    AuthRateLimited,
    /// The account is disabled.
    AuthAccountDisabled,
    /// The session has expired or was revoked.
    AuthSessionExpired,
    /// First-run setup has already been completed.
    SetupAlreadyCompleted,
    /// The setup payload was rejected.
    SetupInvalid,
    /// A password does not meet the policy.
    PasswordTooShort,
    /// A password is in the list of commonly used passwords.
    PasswordTooCommon,
    /// Two passwords do not match.
    PasswordMismatch,
    /// Internal server error; details are logged, not returned.
    Internal,
}

impl ErrorKey {
    /// The wire representation, e.g. `auth.required`.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::RequestInvalid => "request.invalid",
            Self::RequestNotFound => "request.not_found",
            Self::RequestCsrf => "request.csrf",
            Self::AuthRequired => "auth.required",
            Self::AuthForbidden => "auth.forbidden",
            Self::AuthInvalidCredentials => "auth.invalid_credentials",
            Self::AuthRateLimited => "auth.rate_limited",
            Self::AuthAccountDisabled => "auth.account_disabled",
            Self::AuthSessionExpired => "auth.session_expired",
            Self::SetupAlreadyCompleted => "setup.already_completed",
            Self::SetupInvalid => "setup.invalid",
            Self::PasswordTooShort => "password.too_short",
            Self::PasswordTooCommon => "password.too_common",
            Self::PasswordMismatch => "password.mismatch",
            Self::Internal => "internal",
        }
    }

    /// Default HTTP status code for this error.
    #[must_use]
    pub const fn status(&self) -> u16 {
        match self {
            Self::RequestInvalid | Self::SetupInvalid => 400,
            Self::PasswordTooShort | Self::PasswordTooCommon | Self::PasswordMismatch => 422,
            Self::RequestCsrf | Self::AuthForbidden | Self::AuthAccountDisabled => 403,
            Self::AuthRequired | Self::AuthSessionExpired | Self::AuthInvalidCredentials => 401,
            Self::RequestNotFound => 404,
            Self::SetupAlreadyCompleted => 409,
            Self::AuthRateLimited => 429,
            Self::Internal => 500,
        }
    }
}

impl std::fmt::Display for ErrorKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A translatable error: message key plus interpolation parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Stable message key, e.g. `auth.required`.
    pub key: String,
    /// Interpolation parameters (never pre-rendered text).
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub params: serde_json::Map<String, serde_json::Value>,
}

impl Message {
    /// Builds a message from a typed error key.
    #[must_use]
    pub fn new(key: ErrorKey) -> Self {
        Self {
            key: key.as_str().to_owned(),
            params: serde_json::Map::new(),
        }
    }

    /// Builds a message from a free-form key (used by audit entries).
    #[must_use]
    pub fn key(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            params: serde_json::Map::new(),
        }
    }

    /// Adds or replaces one interpolation parameter.
    #[must_use]
    pub fn with(mut self, name: &str, value: impl Into<serde_json::Value>) -> Self {
        self.params.insert(name.to_owned(), value.into());
        self
    }
}

impl From<ErrorKey> for Message {
    fn from(value: ErrorKey) -> Self {
        Self::new(value)
    }
}

/// Widely used field names, so typos in parameters surface at compile time.
pub mod params {
    /// Field name for a parameter name.
    pub const FIELD: &str = "field";
    /// Minimum length of a password.
    pub const MIN: &str = "min";
    /// Maximum allowed size in bytes.
    pub const MAX: &str = "max";
    /// Seconds until a retry is allowed.
    pub const RETRY_AFTER: &str = "retry_after";
}

/// Borrowed or owned message, useful for streaming responses.
pub type MessageRef<'a> = Cow<'a, Message>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_namespaced() {
        assert_eq!(ErrorKey::AuthRequired.as_str(), "auth.required");
        assert!(ErrorKey::Internal.status() == 500);
    }

    #[test]
    fn message_serialises_key_and_params() {
        let msg = Message::new(ErrorKey::PasswordTooShort).with(params::MIN, 10);
        let json = serde_json::to_value(&msg).expect("serialise");
        assert_eq!(json["key"], "password.too_short");
        assert_eq!(json["params"]["min"], 10);
    }

    #[test]
    fn empty_params_are_omitted() {
        let json = serde_json::to_value(Message::new(ErrorKey::Internal)).expect("serialise");
        assert!(json.get("params").is_none());
    }
}
