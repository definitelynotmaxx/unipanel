//! User accounts.

use sqlx::Row as _;
use sqlx::sqlite::SqliteRow;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::{Db, StorageError};
use crate::time::{from_unix, now, unix};

/// Role name of the built-in administrator.
pub const ROLE_ADMIN: &str = "admin";
/// Role name for regular users.
pub const ROLE_MEMBER: &str = "member";

/// A local account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// Primary key.
    pub id: Uuid,
    /// Lowercased login name, unique across the instance.
    pub username: String,
    /// Shown in the UI.
    pub display_name: String,
    /// Argon2id PHC string; `None` for accounts that only sign in via OIDC.
    pub password_hash: Option<String>,
    /// Role name; `admin` or `member` in M0.
    pub role: String,
    /// Disabled accounts cannot log in and their sessions are revoked.
    pub is_active: bool,
    /// Creation timestamp.
    pub created_at: OffsetDateTime,
    /// Last modification timestamp.
    pub updated_at: OffsetDateTime,
}

impl User {
    /// Whether the account has full instance administration rights.
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.role == ROLE_ADMIN
    }
}

/// Data needed to insert a new account.
#[derive(Debug, Clone)]
pub struct NewUser<'a> {
    /// Login name, normalized with [`normalize_username`] before insertion.
    pub username: &'a str,
    /// Display name as typed by the operator.
    pub display_name: &'a str,
    /// Argon2id hash produced by `unipanel-auth`.
    pub password_hash: Option<&'a str>,
    /// Role name.
    pub role: &'a str,
}

/// Lowercases and trims a username so `Admin` and `admin` are the same account.
#[must_use]
pub fn normalize_username(raw: &str) -> String {
    raw.trim().to_lowercase()
}

/// Validates the shape of a username: 2-32 chars of `a-z0-9._-`, starting with
/// a letter or digit.
#[must_use]
pub fn is_valid_username(raw: &str) -> bool {
    let len = raw.chars().count();
    if !(2..=32).contains(&len) {
        return false;
    }
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
}

impl Db {
    /// Inserts a user and returns the stored row.
    ///
    /// # Errors
    /// Returns [`StorageError::Conflict`] if the username is taken.
    pub async fn create_user(&self, new: &NewUser<'_>) -> Result<User, StorageError> {
        let username = normalize_username(new.username);
        let id = Uuid::new_v4();
        let ts = now();
        let result = sqlx::query(
            "INSERT INTO users (id, username, display_name, password_hash, role, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, 1, ?, ?)",
        )
        .bind(id.to_string())
        .bind(&username)
        .bind(new.display_name.trim())
        .bind(new.password_hash)
        .bind(new.role)
        .bind(unix(ts))
        .bind(unix(ts))
        .execute(self.pool())
        .await;

        match result {
            Ok(_) => self
                .find_user_by_id(id)
                .await?
                .ok_or(StorageError::NotFound("user")),
            Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
                Err(StorageError::Conflict("user.username"))
            }
            Err(err) => Err(StorageError::Database(err)),
        }
    }

    /// Looks up a user by primary key.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>, StorageError> {
        let row = sqlx::query("SELECT * FROM users WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(self.pool())
            .await?;
        Ok(row.map(|row| user_from_row(&row)))
    }

    /// Looks up a user by (normalized) username.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>, StorageError> {
        let row = sqlx::query("SELECT * FROM users WHERE username = ?")
            .bind(normalize_username(username))
            .fetch_optional(self.pool())
            .await?;
        Ok(row.map(|row| user_from_row(&row)))
    }

    /// Number of accounts in the instance.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn count_users(&self) -> Result<i64, StorageError> {
        let row = sqlx::query("SELECT COUNT(*) AS count FROM users")
            .fetch_one(self.pool())
            .await?;
        Ok(row.try_get::<i64, _>("count")?)
    }

    /// Replaces the password hash of an account.
    ///
    /// # Errors
    /// Returns [`StorageError::NotFound`] when the user does not exist.
    pub async fn set_password(&self, user_id: Uuid, hash: &str) -> Result<(), StorageError> {
        let result = sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
            .bind(hash)
            .bind(unix(now()))
            .bind(user_id.to_string())
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound("user"));
        }
        Ok(())
    }

    /// Updates the display name of an account.
    ///
    /// # Errors
    /// Returns [`StorageError::NotFound`] when the user does not exist.
    pub async fn set_display_name(&self, user_id: Uuid, display_name: &str) -> Result<(), StorageError> {
        let result = sqlx::query("UPDATE users SET display_name = ?, updated_at = ? WHERE id = ?")
            .bind(display_name.trim())
            .bind(unix(now()))
            .bind(user_id.to_string())
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound("user"));
        }
        Ok(())
    }
}

fn user_from_row(row: &SqliteRow) -> User {
    User {
        id: row
            .try_get::<String, _>("id")
            .ok()
            .and_then(|raw| Uuid::parse_str(&raw).ok())
            .unwrap_or_else(Uuid::nil),
        username: row.try_get("username").unwrap_or_default(),
        display_name: row.try_get("display_name").unwrap_or_default(),
        password_hash: row.try_get("password_hash").ok(),
        role: row.try_get("role").unwrap_or_else(|_| ROLE_MEMBER.to_owned()),
        is_active: row.try_get::<i64, _>("is_active").unwrap_or(0) == 1,
        created_at: from_unix(row.try_get("created_at").unwrap_or_default()),
        updated_at: from_unix(row.try_get("updated_at").unwrap_or_default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn db() -> Db {
        let db = Db::connect_in_memory().await.expect("connect");
        db.migrate().await.expect("migrate");
        db
    }

    #[test]
    fn normalizes_and_validates_usernames() {
        assert_eq!(normalize_username("  Admin "), "admin");
        assert!(is_valid_username("admin"));
        assert!(is_valid_username("ops.team-1"));
        assert!(!is_valid_username("a"));
        assert!(!is_valid_username("-dash"));
        assert!(!is_valid_username("with space"));
        assert!(!is_valid_username("umlaut-ä"));
    }

    #[tokio::test]
    async fn creates_and_finds_users() {
        let db = db().await;
        let user = db
            .create_user(&NewUser {
                username: "Admin",
                display_name: " Ada ",
                password_hash: Some("$argon2id$hash"),
                role: ROLE_ADMIN,
            })
            .await
            .expect("create");
        assert_eq!(user.username, "admin");
        assert_eq!(user.display_name, "Ada");
        assert!(user.is_admin());
        assert_eq!(db.count_users().await.expect("count"), 1);

        let found = db
            .find_user_by_username("ADMIN")
            .await
            .expect("find")
            .expect("present");
        assert_eq!(found.id, user.id);
    }

    #[tokio::test]
    async fn rejects_duplicate_username() {
        let db = db().await;
        let new = NewUser {
            username: "admin",
            display_name: "Ada",
            password_hash: None,
            role: ROLE_ADMIN,
        };
        db.create_user(&new).await.expect("first");
        let err = db.create_user(&new).await.expect_err("duplicate");
        assert!(matches!(err, StorageError::Conflict("user.username")));
    }
}
