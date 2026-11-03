//! Request extractors: client metadata, the current session and CSRF checks.

use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::request::Parts;
use unipanel_auth::{CSRF_COOKIE, CSRF_HEADER, constant_time_eq};
use unipanel_core::ErrorKey;
use unipanel_storage::{Session, User};

use crate::cookies::cookie_value;
use crate::error::ApiError;
use crate::state::AppState;

/// Client information used for auditing, throttling and session binding.
#[derive(Debug, Clone, Default)]
pub struct ClientMeta {
    /// Best-effort client address.
    pub ip: Option<IpAddr>,
    /// Truncated user agent.
    pub user_agent: Option<String>,
}

impl ClientMeta {
    /// Address as a string for storage.
    #[must_use]
    pub fn ip_string(&self) -> Option<String> {
        self.ip.map(|ip| ip.to_string())
    }

    /// User agent for storage (limited to 256 characters).
    #[must_use]
    pub fn user_agent_string(&self) -> Option<String> {
        self.user_agent.as_ref().map(|agent| {
            agent
                .chars()
                .take(256)
                .collect::<String>()
        })
    }
}

impl FromRequestParts<AppState> for ClientMeta {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_agent = parts
            .headers
            .get(axum::http::header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);

        let mut ip = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip());

        // Only trust forwarding headers when the operator opted in. A wrong
        // `trust_proxy` setting makes auditing spoofable, so it is off by default.
        if state.config.trust_proxy {
            ip = forwarded_ip(parts).or(ip);
        }

        Ok(Self { ip, user_agent })
    }
}

fn forwarded_ip(parts: &Parts) -> Option<IpAddr> {
    let value = parts
        .headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .and_then(|value| value.parse::<IpAddr>().ok());
    value.or_else(|| {
        parts
            .headers
            .get("x-real-ip")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .and_then(|value| value.parse::<IpAddr>().ok())
    })
}

/// Loads the session referenced by the session cookie, without rejecting.
#[derive(Debug, Clone)]
pub struct OptionalSession(pub Option<(Session, User)>);

impl FromRequestParts<AppState> for OptionalSession {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(token) = cookie_value(&parts.headers, unipanel_auth::SESSION_COOKIE) else {
            return Ok(Self(None));
        };
        let hash = unipanel_auth::token::hash_token(&token);
        let found = state
            .db
            .find_session_by_token_hash(&hash)
            .await
            .map_err(|err| ApiError::internal(&err))?;

        // Opportunistically refresh `last_seen_at`; failures are not fatal.
        if let Some(found) = &found {
            let _ = state
                .db
                .touch_session(found.session.id, unipanel_auth::SESSION_TOUCH_INTERVAL)
                .await;
        }
        Ok(Self(found.map(|found| (found.session, found.user))))
    }
}

/// A session that must exist.
#[derive(Debug, Clone)]
pub struct CurrentSession {
    /// Session row.
    pub session: Session,
    /// Account.
    pub user: User,
}

impl FromRequestParts<AppState> for CurrentSession {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let OptionalSession(found) = OptionalSession::from_request_parts(parts, state).await?;
        let request_id = parts.extensions.get::<RequestId>();
        match found {
            Some((session, user)) if user.is_active => Ok(Self { session, user }),
            Some(_) => Err(ApiError::new(ErrorKey::AuthAccountDisabled)
                .with_request_id(request_id.map(|id| id.as_str()))),
            None => Err(ApiError::new(ErrorKey::AuthRequired)
                .with_request_id(request_id.map(|id| id.as_str()))),
        }
    }
}

/// Correlation id for the current request.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl RequestId {
    /// Returns the id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromRequestParts<AppState> for RequestId {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<Self>()
            .cloned()
            .unwrap_or_else(|| Self("unknown".to_owned())))
    }
}

/// Verifies the double-submit CSRF token.
///
/// Authenticated requests compare the header against the token bound to the
/// session. Pre-login requests (setup, login) compare against the anonymous CSRF
/// cookie that `/bootstrap` hands out.
///
/// # Errors
/// Returns [`ErrorKey::RequestCsrf`] when the token is missing or wrong.
pub fn verify_csrf(
    headers: &axum::http::HeaderMap,
    session: Option<&Session>,
) -> Result<(), ApiError> {
    let header = headers
        .get(CSRF_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let expected = session
        .map(|session| Some(session.csrf_token.clone()))
        .unwrap_or_else(|| cookie_value(headers, CSRF_COOKIE));

    match (header, expected) {
        (Some(header), Some(expected)) if constant_time_eq(header, &expected) => Ok(()),
        _ => Err(ApiError::new(ErrorKey::RequestCsrf)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use axum::http::header::{COOKIE, HeaderMap};

    fn parts_with(headers: HeaderMap) -> Parts {
        let mut request = axum::http::Request::builder()
            .uri("/api/v1/test")
            .body(())
            .expect("request");
        *request.headers_mut() = headers;
        let (parts, ()) = request.into_parts();
        parts
    }

    fn session(csrf: &str) -> Session {
        Session {
            id: uuid::Uuid::nil(),
            user_id: uuid::Uuid::nil(),
            csrf_token: csrf.to_owned(),
            created_at: time::OffsetDateTime::UNIX_EPOCH,
            last_seen_at: time::OffsetDateTime::UNIX_EPOCH,
            expires_at: time::OffsetDateTime::UNIX_EPOCH,
            ip: None,
            user_agent: None,
        }
    }

    #[test]
    fn accepts_matching_session_token() {
        let mut headers = HeaderMap::new();
        headers.insert(CSRF_HEADER, HeaderValue::from_static("abc"));
        let parts = parts_with(headers);
        assert!(verify_csrf(&parts.headers, Some(&session("abc"))).is_ok());
    }

    #[test]
    fn rejects_missing_or_wrong_token() {
        let parts = parts_with(HeaderMap::new());
        assert!(verify_csrf(&parts.headers, Some(&session("abc"))).is_err());

        let mut headers = HeaderMap::new();
        headers.insert(CSRF_HEADER, HeaderValue::from_static("nope"));
        let parts = parts_with(headers);
        assert!(verify_csrf(&parts.headers, Some(&session("abc"))).is_err());
    }

    #[test]
    fn falls_back_to_the_anonymous_cookie_before_login() {
        let mut headers = HeaderMap::new();
        headers.insert(CSRF_HEADER, HeaderValue::from_static("anon"));
        headers.insert(COOKIE, HeaderValue::from_static("unipanel_csrf=anon"));
        let parts = parts_with(headers);
        assert!(verify_csrf(&parts.headers, None).is_ok());
    }

    #[test]
    fn trims_long_user_agents() {
        let meta = ClientMeta {
            ip: Some("127.0.0.1".parse().expect("ip")),
            user_agent: Some("x".repeat(1000)),
        };
        assert_eq!(meta.user_agent_string().expect("agent").chars().count(), 256);
        assert_eq!(meta.ip_string().as_deref(), Some("127.0.0.1"));
    }
}
