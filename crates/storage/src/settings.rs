//! Instance settings and the atomic first-run setup.
//!
//! Settings are a flat key/value table. The only value with special semantics in
//! M0 is `instance.locale`: it is written exactly once, inside the setup
//! transaction, and is therefore also the "instance is set up" flag.

use sqlx::Row as _;
use unipanel_core::Locale;

use crate::db::{Db, StorageError};
use crate::time::{now, unix};
use crate::users::{normalize_username, User, ROLE_ADMIN};

/// Locale of the instance (`en` or `de`). Immutable after setup.
pub const KEY_LOCALE: &str = "instance.locale";
/// Human readable instance name, editable later.
pub const KEY_NAME: &str = "instance.name";
/// Creation timestamp (UNIX seconds as string).
pub const KEY_CREATED_AT: &str = "instance.created_at";
/// Schema version this database was last migrated with.
pub const KEY_SCHEMA_VERSION: &str = "instance.schema_version";

/// Everything needed to create the instance and its first administrator.
#[derive(Debug, Clone)]
pub struct InstanceSetup<'a> {
    /// Language chosen in the wizard. Immutable afterwards.
    pub locale: Locale,
    /// Administrator username (normalized on insert).
    pub username: &'a str,
    /// Administrator display name.
    pub display_name: &'a str,
    /// Argon2id hash of the administrator password.
    pub password_hash: &'a str,
    /// Instance name shown in the UI and document title.
    pub instance_name: &'a str,
}

impl Db {
    /// Returns a setting value, or `None` when unset.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        let row = sqlx::query("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(self.pool())
            .await?;
        Ok(row.and_then(|row| row.try_get::<String, _>("value").ok()))
    }

    /// Inserts or replaces a setting.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(value)
        .bind(unix(now()))
        .execute(self.pool())
        .await?;
        Ok(())
    }

    /// Whether the instance has completed first-run setup.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn is_setup_complete(&self) -> Result<bool, StorageError> {
        Ok(self.get_setting(KEY_LOCALE).await?.is_some())
    }

    /// The instance locale, once configured.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure. An unparsable stored
    /// value is treated as missing rather than crashing the process.
    pub async fn instance_locale(&self) -> Result<Option<Locale>, StorageError> {
        Ok(self
            .get_setting(KEY_LOCALE)
            .await?
            .and_then(|raw| raw.parse().ok()))
    }

    /// The instance name, falling back to the product name.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn instance_name(&self) -> Result<String, StorageError> {
        Ok(self
            .get_setting(KEY_NAME)
            .await?
            .unwrap_or_else(|| unipanel_core::brand().display_name.to_owned()))
    }

    /// Creates the instance, its administrator and the immutable locale in a
    /// single transaction.
    ///
    /// # Errors
    /// Returns [`StorageError::AlreadySetup`] when setup already ran, and
    /// [`StorageError::Conflict`] when the username is taken.
    pub async fn setup_instance(&self, setup: &InstanceSetup<'_>) -> Result<User, StorageError> {
        let mut tx = self.pool().begin().await?;

        let already = sqlx::query("SELECT 1 FROM settings WHERE key = ?")
            .bind(KEY_LOCALE)
            .fetch_optional(&mut *tx)
            .await?;
        if already.is_some() {
            return Err(StorageError::AlreadySetup);
        }

        let id = uuid::Uuid::new_v4();
        let ts = unix(now());
        let username = normalize_username(setup.username);
        let insert = sqlx::query(
            "INSERT INTO users (id, username, display_name, password_hash, role, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, 1, ?, ?)",
        )
        .bind(id.to_string())
        .bind(&username)
        .bind(setup.display_name.trim())
        .bind(setup.password_hash)
        .bind(ROLE_ADMIN)
        .bind(ts)
        .bind(unix(now()))
        .execute(&mut *tx)
        .await;

        match insert {
            Ok(_) => {}
            Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
                return Err(StorageError::Conflict("user.username"));
            }
            Err(err) => return Err(StorageError::Database(err)),
        }

        for (key, value) in [
            (KEY_LOCALE, setup.locale.as_str().to_owned()),
            (KEY_NAME, setup.instance_name.trim().to_owned()),
            (KEY_CREATED_AT, ts.to_string()),
            (KEY_SCHEMA_VERSION, unipanel_core::SCHEMA_VERSION.to_string()),
        ] {
            sqlx::query("INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)")
                .bind(key)
                .bind(value)
                .bind(ts)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        self.find_user_by_id(id)
            .await?
            .ok_or(StorageError::NotFound("user"))
    }

    /// Changes the instance locale. Only the CLI exposes this; the web UI never
    /// does. The caller is responsible for the "this is permanent" warning.
    ///
    /// # Errors
    /// Returns [`StorageError::NotFound`] when the instance is not set up yet.
    pub async fn set_instance_locale(&self, locale: Locale) -> Result<(), StorageError> {
        if !self.is_setup_complete().await? {
            return Err(StorageError::NotFound("instance"));
        }
        self.set_setting(KEY_LOCALE, locale.as_str()).await
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

    fn setup<'a>(locale: Locale, username: &'a str) -> InstanceSetup<'a> {
        InstanceSetup {
            locale,
            username,
            display_name: "Ada",
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA",
            instance_name: "Homelab",
        }
    }

    #[tokio::test]
    async fn setup_is_atomic_and_sets_locale() {
        let db = db().await;
        assert!(!db.is_setup_complete().await.expect("flag"));
        let user = db
            .setup_instance(&setup(Locale::De, "Admin"))
            .await
            .expect("setup");
        assert_eq!(user.username, "admin");
        assert!(user.is_admin());
        assert!(db.is_setup_complete().await.expect("flag"));
        assert_eq!(db.instance_locale().await.expect("locale"), Some(Locale::De));
        assert_eq!(db.instance_name().await.expect("name"), "Homelab");
    }

    #[tokio::test]
    async fn setup_runs_only_once() {
        let db = db().await;
        db.setup_instance(&setup(Locale::En, "admin"))
            .await
            .expect("setup");
        let err = db
            .setup_instance(&setup(Locale::De, "other"))
            .await
            .expect_err("second setup");
        assert!(matches!(err, StorageError::AlreadySetup));
        assert_eq!(db.count_users().await.expect("count"), 1);
    }

    #[tokio::test]
    async fn settings_are_upserted() {
        let db = db().await;
        db.set_setting("custom.key", "a").await.expect("set");
        db.set_setting("custom.key", "b").await.expect("set");
        assert_eq!(
            db.get_setting("custom.key").await.expect("get"),
            Some("b".to_owned())
        );
    }

    #[tokio::test]
    async fn locale_change_requires_an_instance() {
        let db = db().await;
        assert!(db.set_instance_locale(Locale::De).await.is_err());
        db.setup_instance(&setup(Locale::En, "admin"))
            .await
            .expect("setup");
        db.set_instance_locale(Locale::De).await.expect("change");
        assert_eq!(db.instance_locale().await.expect("locale"), Some(Locale::De));
    }
}
