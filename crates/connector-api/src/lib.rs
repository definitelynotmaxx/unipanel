//! The connector contract.
//!
//! Connectors hide every upstream API behind one normalized surface. The rest of
//! the server only ever sees [`Resource`]s, [`Capabilities`] and message keys, so
//! adding a new system (or a declarative REST template) does not touch the UI.
//!
//! This crate intentionally contains traits and plain data only. Implementations
//! live in `crates/connectors/*` (M1 onwards).

use std::pin::Pin;

use bytes::Bytes;
use futures::Stream;
use schemars::schema::RootSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use unipanel_core::{
    Action, ActionKind, ActionResult, Metrics, Resource, ResourceId,
};

/// A streamed byte sequence. Used for file downloads/uploads and console output
/// so that transfers never buffer whole payloads in memory.
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, ConnectorError>> + Send>>;

/// What a connector can do. The UI renders tabs and buttons from this struct
/// instead of special-casing connector kinds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    /// Can enumerate resources.
    pub resources: bool,
    /// Can report live metrics.
    pub metrics: bool,
    /// Can run power/control actions.
    pub actions: bool,
    /// Can stream a console or log output.
    pub console: bool,
    /// Can browse and edit files.
    pub files: bool,
    /// Can list and restore backups.
    pub backups: bool,
}

impl Capabilities {
    /// Everything M0 knows about, useful for well-behaved connectors in tests.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            resources: true,
            metrics: true,
            actions: true,
            console: true,
            files: true,
            backups: true,
        }
    }
}

/// Result of a "Test connection" click.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestReport {
    /// Whether the upstream answered as expected.
    pub ok: bool,
    /// Round-trip time in milliseconds, if measured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    /// Translable summary, e.g. `connector.test.ok`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<unipanel_core::error::Message>,
    /// Upstream version or other useful details.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub details: serde_json::Value,
}

/// Errors a connector can produce.
#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    /// The upstream rejected the request with a status code.
    #[error("upstream rejected the request with status {status}")]
    Upstream {
        /// HTTP status code, when applicable.
        status: u16,
        /// Short upstream message, safe to log but not necessarily to show.
        message: String,
    },
    /// Credentials are wrong or expired.
    #[error("authentication with the upstream system failed")]
    Unauthorized,
    /// The resource does not exist upstream.
    #[error("resource not found upstream")]
    NotFound,
    /// The connector cannot do this.
    #[error("the connector does not support this operation")]
    Unsupported,
    /// Network/transport failure.
    #[error("could not reach the upstream system: {0}")]
    Transport(String),
    /// The upstream payload did not match the expected shape.
    #[error("unexpected upstream payload: {0}")]
    Protocol(String),
    /// Configuration is invalid.
    #[error("invalid connector configuration: {0}")]
    Config(String),
}

impl ConnectorError {
    /// Maps the error onto a stable message key for the UI.
    #[must_use]
    pub const fn error_key(&self) -> unipanel_core::ErrorKey {
        use unipanel_core::ErrorKey;
        match self {
            Self::Unauthorized => ErrorKey::AuthInvalidCredentials,
            Self::NotFound => ErrorKey::RequestNotFound,
            Self::Upstream { .. } | Self::Transport(_) | Self::Protocol(_) => ErrorKey::Internal,
            Self::Unsupported | Self::Config(_) => ErrorKey::RequestInvalid,
        }
    }
}

/// One entry in a remote directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    /// Base name.
    pub name: String,
    /// Absolute or connector-relative path.
    pub path: String,
    /// Whether this is a directory.
    pub is_dir: bool,
    /// Size in bytes (0 for directories).
    pub size: u64,
    /// Last modification time, when reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<OffsetDateTime>,
    /// Whether the entry is a symlink.
    #[serde(default)]
    pub is_symlink: bool,
}

/// A live console session, returned by [`Connector::open_console`].
#[async_trait::async_trait]
pub trait ConsoleSession: Send {
    /// Sends one line or chunk of input to the remote console.
    async fn send(&mut self, line: &str) -> Result<(), ConnectorError>;
    /// Receives the next chunk of output, or `None` when the stream ended.
    async fn recv(&mut self) -> Option<Result<Bytes, ConnectorError>>;
    /// Closes the session and releases upstream resources.
    async fn close(&mut self);
}

/// File operations for connectors that report [`Capabilities::files`].
#[async_trait::async_trait]
pub trait FileProvider: Send + Sync {
    /// Lists a directory.
    async fn list(&self, id: &ResourceId, dir: &str) -> Result<Vec<FileEntry>, ConnectorError>;
    /// Streams a file's contents.
    async fn read(&self, id: &ResourceId, path: &str) -> Result<ByteStream, ConnectorError>;
    /// Streams a file to the upstream, replacing it.
    async fn write(
        &self,
        id: &ResourceId,
        path: &str,
        body: ByteStream,
    ) -> Result<(), ConnectorError>;
    /// Creates a directory.
    async fn mkdir(&self, id: &ResourceId, path: &str) -> Result<(), ConnectorError>;
    /// Deletes several paths at once.
    async fn delete(&self, id: &ResourceId, paths: &[String]) -> Result<(), ConnectorError>;
    /// Renames or moves a path.
    async fn rename(&self, id: &ResourceId, from: &str, to: &str) -> Result<(), ConnectorError>;
}

/// The main connector trait.
///
/// Implementations are cheap handles (configuration plus an HTTP client); all
/// methods take `&self` and are called from many tasks, so any mutable state
/// must be behind interior synchronisation.
#[async_trait::async_trait]
pub trait Connector: Send + Sync {
    /// Stable kind, e.g. `pterodactyl` or `rest`. Matches the registry key.
    fn kind(&self) -> &'static str;
    /// JSON Schema that drives the generated configuration form.
    fn config_schema(&self) -> RootSchema;
    /// Feature flags used to decide which UI affordances to show.
    fn capabilities(&self) -> Capabilities;
    /// Verifies configuration and reachability.
    async fn test(&self) -> Result<TestReport, ConnectorError>;
    /// Lists every resource this connector exposes.
    async fn list_resources(&self) -> Result<Vec<Resource>, ConnectorError>;
    /// Returns the latest metrics for one resource.
    async fn metrics(&self, _id: &ResourceId) -> Result<Metrics, ConnectorError> {
        Err(ConnectorError::Unsupported)
    }
    /// Runs one action.
    async fn action(&self, _id: &ResourceId, _action: ActionKind) -> Result<ActionResult, ConnectorError> {
        Err(ConnectorError::Unsupported)
    }
    /// Opens a console session.
    async fn open_console(
        &self,
        _id: &ResourceId,
    ) -> Result<Box<dyn ConsoleSession>, ConnectorError> {
        Err(ConnectorError::Unsupported)
    }
    /// Returns the file provider, when supported.
    fn files(&self) -> Option<&dyn FileProvider> {
        None
    }
}

/// Convenience for connector implementations that accept [`Action`] values.
///
/// # Errors
/// Propagates connector errors.
pub async fn run_action<C: Connector + ?Sized>(
    connector: &C,
    action: &Action,
) -> Result<ActionResult, ConnectorError> {
    connector.action(&action.resource, action.kind.clone()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_default_to_nothing() {
        let caps = Capabilities::default();
        assert!(!caps.resources);
        assert!(Capabilities::all().files);
    }

    #[test]
    fn unsupported_maps_to_a_client_error() {
        assert_eq!(
            ConnectorError::Unsupported.error_key(),
            unipanel_core::ErrorKey::RequestInvalid
        );
    }
}
