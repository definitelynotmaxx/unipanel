//! Server-side sessions.
//!
//! The browser holds a random 32 byte token in an `HttpOnly` cookie. Only the
//! SHA-256 hash of that token is persisted, so a database leak does not hand out
//! live sessions. Each session also owns a CSRF token which the frontend must
//! echo in the `X-CSRF-Token` header for state-changing requests.

use sqlx::Row as _;
use sqlx::sqlite::SqliteRow;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::db::{Db, StorageError};
use crate::time::{from_unix, now, unix};
use crate::users::User;

/// A persisted login session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Primary key.
    pub id: Uuid,
    /// Owning account.
    pub user_id: Uuid,
    /// CSRF token bound to this session.
    pub csrf_token: String,
    /// When the session was created.
    pub created_at: OffsetDateTime,
    /// Last request that carried this session.
    pub last_seen_at: OffsetDateTime,
    /// Absolute expiry; sessions are not extended indefinitely.
    pub expires_at: OffsetDateTime,
    /// Client address at login time, if known.
    pub ip: Option<String>,
    /// Client user agent at login time.
    pub user_agent: Option<String>,
}

impl Session {
    /// Whether the session is past its absolute expiry.
    #[must_use]
    pub fn is_expired_at(&self, at: OffsetDateTime) -> bool {
        self.expires_at <= at
    }
}

/// A session together with its account, as needed by request extractors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionWithUser {
    /// The session row.
    pub session: Session,
    /// The owning account.
    pub user: User,
}

/// Parameters for [`Db::create_session`].
#[derive(Debug, Clone)]
pub struct NewSession<'a> {
    /// Owning account.
    pub user_id: Uuid,
    /// SHA-256 of the cookie token.
    pub token_hash: &'a [u8],
    /// CSRF token handed to the frontend.
    pub csrf_token: &'a str,
    /// Absolute expiry.
    pub expires_at: OffsetDateTime,
    /// Client address.
    pub ip: Option<&'a str>,
    /// Client user agent, truncated by the caller.
    pub user_agent: Option<&'a str>,
}

impl Db {
    /// Creates a session row.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn create_session(&self, new: &NewSession<'_>) -> Result<Session, StorageError> {
        let id = Uuid::new_v4();
        let ts = now();
        sqlx::query(
            "INSERT INTO sessions (id, user_id, token_hash, csrf_token, created_at, last_seen_at, expires_at, ip, user_agent)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(new.user_id.to_string())
        .bind(new.token_hash)
        .bind(new.csrf_token)
        .bind(unix(ts))
        .bind(unix(ts))
        .bind(unix(new.expires_at))
        .bind(new.ip)
        .bind(new.user_agent)
        .execute(self.pool())
        .await?;

        Ok(Session {
            id,
            user_id: new.user_id,
            csrf_token: new.csrf_token.to_owned(),
            created_at: ts,
            last_seen_at: ts,
            expires_at: new.expires_at,
            ip: new.ip.map(ToOwned::to_owned),
            user_agent: new.user_agent.map(ToOwned::to_owned),
        })
    }

    /// Finds a live session by token hash and loads its account.
    ///
    /// Expired rows are deleted lazily and reported as `None`.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn find_session_by_token_hash(
        &self,
        token_hash: &[u8],
    ) -> Result<Option<SessionWithUser>, StorageError> {
        let current = now();
        let row = sqlx::query("SELECT * FROM sessions WHERE token_hash = ?")
            .bind(token_hash)
            .fetch_optional(self.pool())
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let session = session_from_row(&row);
        if session.is_expired_at(current) {
            self.delete_session(session.id).await?;
            return Ok(None);
        }
        let Some(user) = self.find_user_by_id(session.user_id).await? else {
            self.delete_session(session.id).await?;
            return Ok(None);
        };
        Ok(Some(SessionWithUser { session, user }))
    }

    /// Updates `last_seen_at`, but only if it is older than `interval` to avoid
    /// a write on every request.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn touch_session(
        &self,
        session_id: Uuid,
        interval: Duration,
    ) -> Result<(), StorageError> {
        let current = now();
        sqlx::query("UPDATE sessions SET last_seen_at = ? WHERE id = ? AND last_seen_at < ?")
            .bind(unix(current))
            .bind(session_id.to_string())
            .bind(unix(current - interval))
            .execute(self.pool())
            .await?;
        Ok(())
    }

    /// Deletes a single session.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn delete_session(&self, session_id: Uuid) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(session_id.to_string())
            .execute(self.pool())
            .await?;
        Ok(())
    }

    /// Deletes every session of a user (used on logout-everywhere and password
    /// changes).
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn delete_user_sessions(&self, user_id: Uuid) -> Result<u64, StorageError> {
        let result = sqlx::query("DELETE FROM sessions WHERE user_id = ?")
            .bind(user_id.to_string())
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected())
    }

    /// Removes expired sessions. Called on startup and periodically.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn purge_expired_sessions(&self) -> Result<u64, StorageError> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
            .bind(unix(now()))
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected())
    }
}

fn session_from_row(row: &SqliteRow) -> Session {
    Session {
        id: row
            .try_get::<String, _>("id")
            .ok()
            .and_then(|raw| Uuid::parse_str(&raw).ok())
            .unwrap_or_else(Uuid::nil),
        user_id: row
            .try_get::<String, _>("user_id")
            .ok()
            .and_then(|raw| Uuid::parse_str(&raw).ok())
            .unwrap_or_else(Uuid::nil),
        csrf_token: row.try_get("csrf_token").unwrap_or_default(),
        created_at: from_unix(row.try_get("created_at").unwrap_or_default()),
        last_seen_at: from_unix(row.try_get("last_seen_at").unwrap_or_default()),
        expires_at: from_unix(row.try_get("expires_at").unwrap_or_default()),
        ip: row.try_get("ip").ok(),
        user_agent: row.try_get("user_agent").ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::users::{normalize_username, NewUser};

    async fn db_with_user() -> (Db, User) {
        let db = Db::connect_in_memory().await.expect("connect");
        db.migrate().await.expect("migrate");
        let user = db
            .create_user(&NewUser {
                username: &normalize_username("admin"),
                display_name: "Ada",
                password_hash: Some("hash"),
                role: "admin",
            })
            .await
            .expect("user");
        (db, user)
    }

    #[tokio::test]
    async fn creates_and_resolves_sessions() {
        let (db, user) = db_with_user().await;
        let expires = now() + Duration::hours(1);
        let session = db
            .create_session(&NewSession {
                user_id: user.id,
                token_hash: b"hash-value",
                csrf_token: "csrf",
                expires_at: expires,
                ip: Some("127.0.0.1"),
                user_agent: Some("test"),
            })
            .await
            .expect("session");
        assert_eq!(session.user_id, user.id);

        let found = db
            .find_session_by_token_hash(b"hash-value")
            .await
            .expect("find")
            .expect("present");
        assert_eq!(found.session.id, session.id);
        assert_eq!(found.user.id, user.id);

        db.delete_session(session.id).await.expect("delete");
        assert!(
            db.find_session_by_token_hash(b"hash-value")
                .await
                .expect("find")
                .is_none()
        );
    }

    #[tokio::test]
    async fn expired_sessions_are_rejected_and_removed() {
        let (db, user) = db_with_user().await;
        db.create_session(&NewSession {
            user_id: user.id,
            token_hash: b"stale",
            csrf_token: "csrf",
            expires_at: now() - Duration::minutes(1),
            ip: None,
            user_agent: None,
        })
        .await
        .expect("session");

        assert!(
            db.find_session_by_token_hash(b"stale")
                .await
                .expect("find")
                .is_none()
        );
        assert_eq!(db.purge_expired_sessions().await.expect("purge"), 0);
    }

    #[tokio::test]
    async fn deleting_a_user_removes_sessions() {
        let (db, user) = db_with_user().await;
        db.create_session(&NewSession {
            user_id: user.id,
            token_hash: b"t",
            csrf_token: "c",
            expires_at: now() + Duration::days(1),
            ip: None,
            user_agent: None,
        })
        .await
        .expect("session");

        assert_eq!(db.delete_user_sessions(user.id).await.expect("delete"), 1);
    }
}
