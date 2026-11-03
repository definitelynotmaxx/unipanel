//! Bootstrapping: everything the SPA needs before it can render.
//!
//! `/bootstrap` is intentionally unauthenticated: it tells the frontend whether
//! to show the setup wizard, the login screen or the app shell, which language to
//! load and which CSRF token to send with the first mutating request.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse as _, Response};
use unipanel_auth::CSRF_COOKIE;
use unipanel_core::{Locale, brand};
use unipanel_core::ErrorKey;

use crate::cookies::{CookieOptions, cookie_value, set_cookie};
use crate::dto::{BootstrapResponse, UserDto};
use crate::error::ApiError;
use crate::extract::{ClientMeta, OptionalSession, RequestId};
use crate::state::AppState;

/// Anonymous CSRF cookies survive long enough for a slow setup wizard.
const ANON_CSRF_TTL_SECONDS: i64 = 86_400;

/// Returns locale, setup state, session and CSRF token.
///
/// # Errors
/// Returns an [`ApiError`] when the database cannot be read.
#[utoipa::path(
    get,
    path = "/api/v1/bootstrap",
    tag = "system",
    responses((status = 200, body = BootstrapResponse))
)]
pub async fn bootstrap(
    State(state): State<AppState>,
    meta: ClientMeta,
    OptionalSession(found): OptionalSession,
    RequestId(request_id): RequestId,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let _ = &meta;
    let configured_locale = state.instance_locale().await;
    let suggested = suggest_locale(&headers);

    let setup_required = state
        .db
        .is_setup_complete()
        .await
        .map_err(|err| ApiError::internal(&err).with_request_id(Some(request_id.as_str())))?;

    let instance_name = if setup_required {
        brand().display_name.to_owned()
    } else {
        state
            .db
            .instance_name()
            .await
            .map_err(|err| ApiError::internal(&err).with_request_id(Some(request_id.as_str())))?
    };

    let (user, session_csrf) = match found {
        Some((session, user)) if user.is_active => (Some(UserDto::from(&user)), Some(session.csrf_token)),
        _ => (None, None),
    };

    let mut new_anonymous_cookie = false;
    let csrf_token = match session_csrf {
        Some(token) => token,
        None => match cookie_value(&headers, CSRF_COOKIE) {
            Some(existing) => existing,
            None => {
                new_anonymous_cookie = true;
                unipanel_auth::token::random_token()
            }
        },
    };

    let body = BootstrapResponse {
        setup_required,
        authenticated: user.is_some(),
        locale: configured_locale.unwrap_or(suggested).as_str().to_owned(),
        suggested_locale: suggested.as_str().to_owned(),
        instance_name,
        version: unipanel_core::version().to_owned(),
        user,
        csrf_token: csrf_token.clone(),
    };

    let mut response = (StatusCode::OK, Json(body)).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    if new_anonymous_cookie {
        append_cookie(
            &mut response,
            &set_cookie(
                CSRF_COOKIE,
                &csrf_token,
                CookieOptions {
                    http_only: true,
                    secure: state.secure_cookies(),
                    max_age_seconds: Some(ANON_CSRF_TTL_SECONDS),
                },
            ),
        );
    }
    Ok(response)
}

/// Picks a locale from `Accept-Language`, defaulting to English.
fn suggest_locale(headers: &HeaderMap) -> Locale {
    headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
        .and_then(Locale::from_accept_language)
        .unwrap_or_default()
}

pub(crate) fn append_cookie(response: &mut Response, cookie: &str) {
    if let Ok(value) = HeaderValue::from_str(cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    } else {
        tracing::error!("generated an invalid Set-Cookie header");
    }
}

/// Validates that the instance is still unconfigured, mapping to the right error.
pub(crate) async fn ensure_setup_required(state: &AppState) -> Result<(), ApiError> {
    if state
        .db
        .is_setup_complete()
        .await
        .map_err(|err| ApiError::internal(&err))?
    {
        return Err(ApiError::new(ErrorKey::SetupAlreadyCompleted));
    }
    Ok(())
}
