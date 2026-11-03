//! Pool setup, health checks and migrations.

use std::str::FromStr as _;
use std::time::Duration as StdDuration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Executor as _, SqlitePool};
use tracing::{info, instrument};

/// Errors returned by the storage layer.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    /// A unique constraint was violated.
    #[error("conflict: {0}")]
    Conflict(&'static str),
    /// The requested row does not exist.
    #[error("not found: {0}")]
    NotFound(&'static str),
    /// The connection string is not a valid SQLite URL.
    #[error("invalid database url: {0}")]
    InvalidUrl(String),
    /// Migrations failed to apply.
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    /// The instance is already set up.
    #[error("instance already set up")]
    AlreadySetup,
}

/// Handle around the SQLx pool.
#[derive(Debug, Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    /// Opens (and creates if needed) a SQLite database at `url`.
    ///
    /// # Errors
    /// Returns [`StorageError::InvalidUrl`] when the URL cannot be parsed and
    /// [`StorageError::Database`] when the connection fails.
    #[instrument(skip_all, fields(url = %redact_url(url)))]
    pub async fn connect(url: &str) -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::from_str(url)
            .map_err(|_| StorageError::InvalidUrl(redact_url(url).into_owned()))?;
        Self::connect_with(options).await
    }

    /// Opens (and creates if needed) a SQLite database file.
    ///
    /// This bypasses URL parsing entirely, which keeps Windows paths with drive
    /// letters working.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] when the connection fails.
    pub async fn connect_file(path: &std::path::Path) -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::new().filename(path);
        Self::connect_with(options).await
    }

    async fn connect_with(options: SqliteConnectOptions) -> Result<Self, StorageError> {
        let options = options
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(StdDuration::from_secs(5));

        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .acquire_timeout(StdDuration::from_secs(10))
            .connect_with(options)
            .await?;

        info!("database connected");
        Ok(Self { pool })
    }

    /// Opens a private in-memory database, used by tests.
    ///
    /// SQLite in-memory databases are per connection, so the pool is limited to
    /// a single connection to make migrations and queries share state.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] if the pool cannot be created.
    pub async fn connect_in_memory() -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(|_| StorageError::InvalidUrl("sqlite::memory:".to_owned()))?
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_with(options)
            .await?;
        Ok(Self { pool })
    }

    /// Applies all pending migrations.
    ///
    /// # Errors
    /// Returns [`StorageError::Migrate`] when a migration fails.
    #[instrument(skip(self))]
    pub async fn migrate(&self) -> Result<(), StorageError> {
        sqlx::migrate!("../../migrations").run(&self.pool).await?;
        info!("migrations applied");
        Ok(())
    }

    /// Returns the underlying pool for repositories in this crate.
    #[must_use]
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Cheap connectivity check used by `/healthz` and `doctor`.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] when the probe query fails.
    pub async fn health(&self) -> Result<(), StorageError> {
        self.pool.execute("SELECT 1").await?;
        Ok(())
    }

    /// Closes the pool, waiting for in-flight queries.
    pub async fn close(self) {
        self.pool.close().await;
    }
}

/// Removes credentials and query strings from a database URL for logging.
fn redact_url(url: &str) -> std::borrow::Cow<'_, str> {
    if let Some((scheme, rest)) = url.split_once("://") {
        if let Some(at) = rest.find('@') {
            if rest[..at].contains(':') {
                return format!("{scheme}://***@{rest_after_at}", rest_after_at = &rest[at + 1..])
                    .into();
            }
        }
    }
    if let Some(base) = url.split('?').next() {
        return base.to_owned().into();
    }
    url.to_owned().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_database_runs_migrations_and_answers_health() {
        let db = Db::connect_in_memory().await.expect("connect");
        db.migrate().await.expect("migrate");
        db.health().await.expect("health");
    }

    #[test]
    fn redacts_credentials_and_query() {
        assert_eq!(
            redact_url("sqlite://user:pass@host/db?mode=rwc"),
            "sqlite://***@host/db"
        );
        assert_eq!(redact_url("sqlite:///data/unipanel.db?mode=rwc"), "sqlite:///data/unipanel.db");
    }
}
