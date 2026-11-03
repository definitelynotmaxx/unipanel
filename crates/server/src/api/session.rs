//! Session issuance and cookie handling shared by setup and login.

use time::Duration;
use unipanel_auth::{
    SESSION_COOKIE, SESSION_TTL, CSRF_COOKIE, SessionTokens,
};
use unipanel_storage::{NewSession, User};

use crate::cookies::{CookieOptions, set_cookie};
use crate::dto::{SessionResponse, UserDto};
use crate::error::ApiError;
use crate::extract::ClientMeta;
use crate::state::AppState;

/// A freshly issued session plus the cookies that carry it.
#[derive(Debug)]
pub struct IssuedSession {
    /// Body returned to the client.
    pub body: SessionResponse,
    /// `Set-Cookie` values to attach to the response.
    pub cookies: [String; 2],
}

/// Creates a session row and the matching cookie headers.
///
/// # Errors
/// Returns an [`ApiError`] when the session cannot be persisted.
pub async fn issue_session(
    state: &AppState,
    user: &User,
    meta: &ClientMeta,
) -> Result<IssuedSession, ApiError> {
    let tokens = SessionTokens::generate();
    let expires_at = unipanel_storage::time::now() + SESSION_TTL;
    let max_age = SESSION_TTL.whole_seconds();

    state
        .db
        .create_session(&NewSession {
            user_id: user.id,
            token_hash: &tokens.token_hash,
            csrf_token: &tokens.csrf_token,
            expires_at,
            ip: meta.ip_string().as_deref(),
            user_agent: meta.user_agent_string().as_deref(),
        })
        .await
        .map_err(|err| ApiError::internal(&err))?;

    Ok(IssuedSession {
        body: SessionResponse {
            user: UserDto::from(user),
            csrf_token: tokens.csrf_token.clone(),
        },
        cookies: [
            set_cookie(
                SESSION_COOKIE,
                &tokens.token,
                CookieOptions::session(state.secure_cookies(), max_age),
            ),
            set_cookie(
                CSRF_COOKIE,
                &tokens.csrf_token,
                CookieOptions {
                    http_only: true,
                    secure: state.secure_cookies(),
                    max_age_seconds: Some(max_age),
                },
            ),
        ],
    })
}

/// Cookies that clear both session and CSRF state on logout.
#[must_use]
pub fn cleared_cookies(secure: bool) -> [String; 2] {
    [
        set_cookie(
            SESSION_COOKIE,
            "",
            CookieOptions::cleared(secure),
        ),
        set_cookie(CSRF_COOKIE, "", CookieOptions::cleared(secure)),
    ]
}

/// Session lifetime as a duration, exported for tests.
#[must_use]
pub const fn session_ttl() -> Duration {
    SESSION_TTL
}
