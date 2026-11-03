//! First-run setup: choose the language, create the administrator, log in.
//!
//! The endpoint is only reachable while `instance.locale` is unset. Creating the
//! user and writing the locale happen in one transaction, so a crash can never
//! leave a half-configured instance that still allows a second setup.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse as _, Response};
use unipanel_core::{ErrorKey, Locale};
use unipanel_storage::settings::InstanceSetup;

use crate::api::bootstrap::{append_cookie, ensure_setup_required};
use crate::api::session::issue_session;
use crate::audit;
use crate::dto::{SetupRequest, SessionResponse};
use crate::error::{ApiError, ApiJson};
use crate::extract::{ClientMeta, RequestId, verify_csrf};
use crate::state::AppState;

/// Longest accepted display/instance name, in characters.
const MAX_NAME_LEN: usize = 64;

/// Performs first-run setup and signs the new administrator in.
///
/// # Errors
/// Returns [`ErrorKey::SetupAlreadyCompleted`] when the instance is configured,
/// [`ErrorKey::SetupInvalid`] for malformed input and the password policy keys
/// for weak passwords.
#[utoipa::path(
    post,
    path = "/api/v1/setup",
    tag = "system",
    request_body = SetupRequest,
    responses(
        (status = 200, body = SessionResponse),
        (status = 400, body = crate::error::ApiErrorBody),
        (status = 409, body = crate::error::ApiErrorBody)
    )
)]
pub async fn setup(
    State(state): State<AppState>,
    meta: ClientMeta,
    RequestId(request_id): RequestId,
    headers: HeaderMap,
    ApiJson(request): ApiJson<SetupRequest>,
) -> Result<Response, ApiError> {
    ensure_setup_required(&state).await?;

    // Pre-login CSRF uses the anonymous cookie handed out by `/bootstrap`.
    verify_csrf(&headers, None)?;

    let locale = Locale::parse_tag(&request.locale)
        .ok_or_else(|| ApiError::new(ErrorKey::SetupInvalid).with("field", "locale"))?;
    let username = unipanel_storage::users::normalize_username(&request.username);
    if !unipanel_storage::users::is_valid_username(&username) {
        return Err(ApiError::new(ErrorKey::SetupInvalid).with("field", "username"));
    }
    let display_name = request.display_name.trim();
    if display_name.is_empty() || display_name.chars().count() > MAX_NAME_LEN {
        return Err(ApiError::new(ErrorKey::SetupInvalid).with("field", "display_name"));
    }
    let instance_name = request
        .instance_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(unipanel_core::brand().display_name);
    if instance_name.chars().count() > MAX_NAME_LEN {
        return Err(ApiError::new(ErrorKey::SetupInvalid).with("field", "instance_name"));
    }

    unipanel_auth::PasswordPolicy::default()
        .validate(&request.password, &[&username, display_name])
        .map_err(ApiError::from)?;

    let password = request.password.clone();
    let hash = tokio::task::spawn_blocking(move || unipanel_auth::hash_password(&password))
        .await
        .map_err(|err| ApiError::internal(&err))?
        .map_err(ApiError::from)?;

    let created = state
        .db
        .setup_instance(&InstanceSetup {
            locale,
            username: &username,
            display_name,
            password_hash: &hash,
            instance_name,
        })
        .await;

    let user = match created {
        Ok(user) => user,
        Err(unipanel_storage::StorageError::AlreadySetup) => {
            return Err(ApiError::new(ErrorKey::SetupAlreadyCompleted));
        }
        Err(unipanel_storage::StorageError::Conflict(_)) => {
            return Err(ApiError::new(ErrorKey::SetupInvalid).with("field", "username"));
        }
        Err(err) => return Err(ApiError::internal(&err)),
    };

    state.locale.set(Some(locale)).await;
    audit::record(
        &state,
        &meta,
        Some(&user),
        audit::events::SETUP_COMPLETED,
        true,
        serde_json::json!({ "locale": locale.as_str(), "instance_name": instance_name }),
        request_id.as_str(),
    )
    .await;

    let issued = issue_session(&state, &user, &meta).await?;
    let mut response = (StatusCode::OK, Json(issued.body)).into_response();
    for cookie in issued.cookies {
        append_cookie(&mut response, &cookie);
    }
    tracing::info!(user = %user.username, locale = %locale, "instance configured");
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_limits_are_sane() {
        assert!(MAX_NAME_LEN >= 32);
    }
}
