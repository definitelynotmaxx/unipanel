//! Login, logout and session lookup.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse as _, Response};
use unipanel_core::ErrorKey;

use crate::api::bootstrap::append_cookie;
use crate::api::session::{cleared_cookies, issue_session};
use crate::audit;
use crate::dto::{LoginRequest, SessionResponse};
use crate::error::{ApiError, ApiJson};
use crate::extract::{ClientMeta, CurrentSession, OptionalSession, RequestId, verify_csrf};
use crate::state::AppState;

/// Verifies credentials and opens a session.
///
/// The response is identical for unknown users and wrong passwords, and both
/// paths spend the same Argon2 time, so the endpoint does not leak which accounts
/// exist.
///
/// # Errors
/// Returns [`ErrorKey::AuthInvalidCredentials`], [`ErrorKey::AuthRateLimited`] (with
/// a `retry_after` parameter) or [`ErrorKey::AuthAccountDisabled`].
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, body = SessionResponse),
        (status = 401, body = crate::error::ApiErrorBody),
        (status = 429, body = crate::error::ApiErrorBody)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    meta: ClientMeta,
    RequestId(request_id): RequestId,
    headers: HeaderMap,
    ApiJson(request): ApiJson<LoginRequest>,
) -> Result<Response, ApiError> {
    verify_csrf(&headers, None)?;

    let username = unipanel_storage::users::normalize_username(&request.username);
    if username.is_empty() || request.password.is_empty() {
        return Err(ApiError::new(ErrorKey::AuthInvalidCredentials)
            .with_request_id(Some(request_id.as_str())));
    }

    if let Some(retry_after) = state.throttle.check(meta.ip, &username) {
        return Err(ApiError::new(ErrorKey::AuthRateLimited)
            .with("retry_after", retry_after)
            .with_request_id(Some(request_id.as_str())));
    }

    let user = state
        .db
        .find_user_by_username(&username)
        .await
        .map_err(|err| ApiError::internal(&err).with_request_id(Some(request_id.as_str())))?;

    let stored_hash = user
        .as_ref()
        .and_then(|user| user.password_hash.clone())
        .unwrap_or_else(|| unipanel_auth::dummy_hash().to_owned());
    let password = request.password.clone();
    let password_ok = tokio::task::spawn_blocking(move || {
        unipanel_auth::verify_password(&stored_hash, &password)
    })
    .await
    .map_err(|err| ApiError::internal(&err).with_request_id(Some(request_id.as_str())))?;

    let Some(user) = user else {
        state.throttle.record_failure(meta.ip, &username);
        audit::record(
            &state,
            &meta,
            None,
            audit::events::LOGIN_FAILED,
            false,
            audit::login_detail(&username, Some("unknown_user")),
            request_id.as_str(),
        )
        .await;
        return Err(ApiError::new(ErrorKey::AuthInvalidCredentials)
            .with_request_id(Some(request_id.as_str())));
    };

    if !user.is_active {
        audit::record(
            &state,
            &meta,
            Some(&user),
            audit::events::LOGIN_FAILED,
            false,
            audit::login_detail(&username, Some("disabled")),
            request_id.as_str(),
        )
        .await;
        return Err(ApiError::new(ErrorKey::AuthAccountDisabled)
            .with_request_id(Some(request_id.as_str())));
    }

    if !password_ok {
        state.throttle.record_failure(meta.ip, &username);
        audit::record(
            &state,
            &meta,
            Some(&user),
            audit::events::LOGIN_FAILED,
            false,
            audit::login_detail(&username, Some("bad_password")),
            request_id.as_str(),
        )
        .await;
        return Err(ApiError::new(ErrorKey::AuthInvalidCredentials)
            .with_request_id(Some(request_id.as_str())));
    }

    state.throttle.record_success(meta.ip, &username);
    let issued = issue_session(&state, &user, &meta).await?;
    audit::record(
        &state,
        &meta,
        Some(&user),
        audit::events::LOGIN,
        true,
        audit::login_detail(&username, None),
        request_id.as_str(),
    )
    .await;

    let mut response = (StatusCode::OK, Json(issued.body)).into_response();
    for cookie in issued.cookies {
        append_cookie(&mut response, &cookie);
    }
    Ok(response)
}

/// Ends the current session. Always succeeds, so a stale cookie can be cleared.
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "auth",
    responses((status = 204))
)]
pub async fn logout(
    State(state): State<AppState>,
    meta: ClientMeta,
    OptionalSession(found): OptionalSession,
    RequestId(request_id): RequestId,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if let Some((session, user)) = found {
        verify_csrf(&headers, Some(&session))?;
        state
            .db
            .delete_session(session.id)
            .await
            .map_err(|err| ApiError::internal(&err))?;
        audit::record(
            &state,
            &meta,
            Some(&user),
            audit::events::LOGOUT,
            true,
            serde_json::json!({}),
            request_id.as_str(),
        )
        .await;
    }

    let mut response = StatusCode::NO_CONTENT.into_response();
    for cookie in cleared_cookies(state.secure_cookies()) {
        append_cookie(&mut response, &cookie);
    }
    Ok(response)
}

/// Returns the signed-in user and the session's CSRF token.
///
/// # Errors
/// Returns [`ErrorKey::AuthRequired`] without a valid session.
#[utoipa::path(
    get,
    path = "/api/v1/auth/session",
    tag = "auth",
    responses(
        (status = 200, body = SessionResponse),
        (status = 401, body = crate::error::ApiErrorBody)
    )
)]
pub async fn session(
    CurrentSession { session, user }: CurrentSession,
) -> Json<SessionResponse> {
    Json(SessionResponse {
        user: crate::dto::UserDto::from(&user),
        csrf_token: session.csrf_token,
    })
}
