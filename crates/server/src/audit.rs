//! Audit logging helpers.
//!
//! Audit failures must never break the user's request, so write errors are logged
//! and swallowed. Entries store message keys plus JSON parameters, never rendered
//! prose, so the log follows the instance language.

use serde_json::json;
use unipanel_storage::audit::NewAuditEntry;
use unipanel_storage::User;

use crate::extract::ClientMeta;
use crate::state::AppState;

/// Audit event keys, kept in one place so the UI can translate them.
pub mod events {
    /// First-run setup completed.
    pub const SETUP_COMPLETED: &str = "audit.setup.completed";
    /// Successful login.
    pub const LOGIN: &str = "audit.auth.login";
    /// Failed login attempt.
    pub const LOGIN_FAILED: &str = "audit.auth.login_failed";
    /// Explicit logout.
    pub const LOGOUT: &str = "audit.auth.logout";
}

/// Writes one audit entry.
pub async fn record(
    state: &AppState,
    meta: &ClientMeta,
    actor: Option<&User>,
    event: &str,
    success: bool,
    detail: serde_json::Value,
    request_id: &str,
) {
    let ip = meta.ip_string();
    let entry = NewAuditEntry {
        actor_user_id: actor.map(|user| user.id),
        actor_label: actor
            .map(|user| user.username.as_str())
            .or_else(|| detail.get("username").and_then(|value| value.as_str())),
        event,
        detail: Some(&detail),
        ip: ip.as_deref(),
        success,
        request_id: Some(request_id),
    };
    if let Err(err) = state.db.insert_audit(&entry).await {
        tracing::warn!(error = %err, event, "could not write audit entry");
    }
}

/// Detail payload for login-related events; never includes the password.
#[must_use]
pub fn login_detail(username: &str, reason: Option<&str>) -> serde_json::Value {
    match reason {
        Some(reason) => json!({ "username": username, "reason": reason }),
        None => json!({ "username": username }),
    }
}
