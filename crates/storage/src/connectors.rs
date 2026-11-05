//! Connector persistence: CRUD over the `connectors` table.
//!
//! API keys are stored as opaque encrypted blobs (the server encrypts them
//! before insert and decrypts after fetch).

use sqlx::Row as _;
use sqlx::sqlite::SqliteRow;

use crate::db::{Db, StorageError};
use crate::time::{now, unix};

/// A single connector configuration row as loaded from storage.
#[derive(Debug, Clone)]
pub struct Connector {
    /// UUID primary key.
    pub id: String,
    /// Human-readable name shown in the UI.
    pub name: String,
    /// Panel type.
    pub kind: String,
    /// Base URL of the panel API.
    pub api_url: String,
    /// Whether the connector is enabled for resource syncing.
    pub is_active: bool,
    /// Creation timestamp (RFC 3339).
    pub created_at: String,
    /// Last-modified timestamp (RFC 3339).
    pub updated_at: String,
}

/// Parameters for creating a new connector.
#[derive(Debug, Clone)]
pub struct ConnectorInsert<'a> {
    /// Human-readable name.
    pub name: &'a str,
    /// Panel type (`pterodactyl`, `docker`, `proxmox`, `rest`).
    pub kind: &'a str,
    /// Base URL of the panel API.
    pub api_url: &'a str,
    /// Pre-encrypted API key blob (opaque string).
    pub api_key: &'a str,
    /// Whether the connector should be active on creation.
    pub is_active: bool,
}

/// Errors specific to connector operations.
#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    /// The connection test did not succeed.
    #[error("connection test failed: {0}")]
    TestFailed(String),
}

fn connector_from_row(row: &SqliteRow) -> Connector {
    Connector {
        id: row.try_get("id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        kind: row.try_get("kind").unwrap_or_default(),
        api_url: row.try_get("api_url").unwrap_or_default(),
        is_active: row.try_get::<i64, _>("is_active").unwrap_or(1) == 1,
        created_at: row.try_get("created_at").unwrap_or_default(),
        updated_at: row.try_get("updated_at").unwrap_or_default(),
    }
}

impl Db {
    /// Lists all connector configurations.
    ///
    /// # Errors
    /// Returns [`StorageError::Database`] on query failure.
    pub async fn list_connectors(&self) -> Result<Vec<Connector>, StorageError> {
        let rows = sqlx::query(
            "SELECT id, name, kind, api_url, is_active, created_at, updated_at \
             FROM connectors ORDER BY name",
        )
        .fetch_all(self.pool())
        .await?;
        Ok(rows.into_iter().map(|row| connector_from_row(&row)).collect())
    }

    /// Creates a new connector. The `api_key` column must already be encrypted.
    ///
    /// # Errors
    /// Returns [`StorageError::Conflict`] if `name` is duplicated,
    /// or [`StorageError::Database`] on other SQL errors.
    pub async fn create_connector(
        &self,
        insert: &ConnectorInsert<'_>,
    ) -> Result<Connector, StorageError> {
        let id = uuid::Uuid::new_v4().to_string();
        let ts = unix(now());

        let result = sqlx::query(
            "INSERT INTO connectors \
             (id, name, kind, api_url, api_key, is_active, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(insert.name)
        .bind(insert.kind)
        .bind(insert.api_url)
        .bind(insert.api_key)
        .bind(if insert.is_active { 1 } else { 0 })
        .bind(ts)
        .bind(ts)
        .execute(self.pool())
        .await;

        match result {
            Ok(_) => {}
            Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
                return Err(StorageError::Conflict("connector.name"));
            }
            Err(err) => return Err(StorageError::Database(err)),
        }

        let row = sqlx::query(
            "SELECT id, name, kind, api_url, is_active, created_at, updated_at \
             FROM connectors WHERE id = ?",
        )
        .bind(&id)
        .fetch_one(self.pool())
        .await?;

        Ok(connector_from_row(&row))
    }

    /// Deletes a connector by id.
    ///
    /// # Errors
    /// Returns [`StorageError::NotFound`] when no connector with that id exists.
    pub async fn delete_connector(&self, id: &str) -> Result<(), StorageError> {
        let rows_affected = sqlx::query("DELETE FROM connectors WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?
            .rows_affected();
        if rows_affected == 0 {
            return Err(StorageError::NotFound("connector"));
        }
        Ok(())
    }
}
