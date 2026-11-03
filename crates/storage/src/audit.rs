//! Append-only audit trail.
//!
//! Login attempts and control operations are recorded as message keys plus JSON
//! parameters, never as rendered text, so an instance that switches locale keeps
//! a readable history.

use serde::{Deserialize, Serialize};
use sqlx::Row as _;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::{Db, StorageError};
use crate::time::{from_unix, now, unix};

/// One audit entry to be written.
#[derive(Debug, Clone)]
pub struct NewAuditEntry<'a> {
    /// Account responsible, if any (failed logins have none).
    pub actor_user_id: Option<Uuid>,
    /// Login/display name snapshot, so deleted accounts stay traceable.
    pub actor_label: Option<&'a str>,
    /// Event key, e.g. `audit.auth.login`.
    pub event: &'a str,
    /// Message parameters as JSON.
    pub detail: Option<&'a serde_json::Value>,
    /// Client address.
    pub ip: Option<&'a str>,
    /// Whether the action succeeded.
    pub success: bool,
    /// Correlation id, also emitted in server logs.
    pub request_id: Option<&'a str>,
}

/// A stored audit entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Monotonic row id.
    pub id: i64,
    /// Timestamp (UTC).
    pub created_at: OffsetDateTime,
    /// Account id, if known.
    pub actor_user_id: Option<Uuid>,
    /// Account label snapshot.
    pub actor_label: Option<String>,
    /// Event key.
    pub event: String,
    /// Parameters.
    pub detail: Option<serde_json::Value>,
    /// Client address.
    pub ip: Option<String>,
    /// Outcome.
    pub success: bool,
    /// Correlation id.
    pub request_id: Option<String>,
}

impl Db {
    /// Appends an entry. Never fails the caller's request: auditing problems are
    /// logged by the server layer.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn insert_audit(&self, entry: &NewAuditEntry<'_>) -> Result<(), StorageError> {
        let detail = entry
            .detail
            .map(|value| value.to_string())
            .unwrap_or_default();
        sqlx::query(
            "INSERT INTO audit_log (created_at, actor_user_id, actor_label, event, detail, ip, success, request_id)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(unix(now()))
        .bind(entry.actor_user_id.map(|id| id.to_string()))
        .bind(entry.actor_label)
        .bind(entry.event)
        .bind(if detail.is_empty() { None } else { Some(detail) })
        .bind(entry.ip)
        .bind(i64::from(entry.success))
        .bind(entry.request_id)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    /// Returns the most recent entries, newest first.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn recent_audit(&self, limit: i64) -> Result<Vec<AuditEntry>, StorageError> {
        let rows = sqlx::query("SELECT * FROM audit_log ORDER BY id DESC LIMIT ?")
            .bind(limit.clamp(1, 500))
            .fetch_all(self.pool())
            .await?;
        Ok(rows.iter().map(audit_from_row).collect())
    }
}

fn audit_from_row(row: &sqlx::sqlite::SqliteRow) -> AuditEntry {
    AuditEntry {
        id: row.try_get("id").unwrap_or_default(),
        created_at: from_unix(row.try_get("created_at").unwrap_or_default()),
        actor_user_id: row
            .try_get::<Option<String>, _>("actor_user_id")
            .ok()
            .flatten()
            .and_then(|raw| Uuid::parse_str(&raw).ok()),
        actor_label: row.try_get("actor_label").ok(),
        event: row.try_get("event").unwrap_or_default(),
        detail: row
            .try_get::<Option<String>, _>("detail")
            .ok()
            .flatten()
            .and_then(|raw| serde_json::from_str(&raw).ok()),
        ip: row.try_get("ip").ok(),
        success: row.try_get::<i64, _>("success").unwrap_or(0) == 1,
        request_id: row.try_get("request_id").ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn appends_and_reads_entries() {
        let db = Db::connect_in_memory().await.expect("connect");
        db.migrate().await.expect("migrate");
        let detail = serde_json::json!({ "username": "admin" });
        db.insert_audit(&NewAuditEntry {
            actor_user_id: None,
            actor_label: Some("admin"),
            event: "audit.auth.login",
            detail: Some(&detail),
            ip: Some("10.0.0.5"),
            success: false,
            request_id: Some("req-1"),
        })
        .await
        .expect("insert");

        let entries = db.recent_audit(10).await.expect("list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].event, "audit.auth.login");
        assert!(!entries[0].success);
        assert_eq!(entries[0].detail.as_ref().expect("detail")["username"], "admin");
    }
}
