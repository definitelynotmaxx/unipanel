//! The normalized resource model.
//!
//! Every connector maps its own payload onto these types so that widgets,
//! search, the command palette and permission checks work identically for a
//! Minecraft server, a Docker container and a Proxmox VM.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::status::Status;

/// Identifier of a configured connector instance (a row in `connectors`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConnectorId(pub Uuid);

impl ConnectorId {
    /// Wraps a UUID.
    #[must_use]
    pub const fn new(id: Uuid) -> Self {
        Self(id)
    }

    /// Returns the inner UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for ConnectorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Globally unique resource identifier: connector id plus the remote id.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId {
    /// Which connector owns this resource.
    pub connector: ConnectorId,
    /// The id as the upstream system knows it, opaque to UniPanel.
    pub remote: String,
}

impl ResourceId {
    /// Builds a new resource id.
    #[must_use]
    pub fn new(connector: ConnectorId, remote: impl Into<String>) -> Self {
        Self {
            connector,
            remote: remote.into(),
        }
    }

    /// The stable, URL-safe form used in REST paths: `<connector>/<remote>`.
    #[must_use]
    pub fn composite(&self) -> String {
        format!("{}/{}", self.connector, self.remote)
    }
}

/// What kind of thing a resource is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    /// A game server, e.g. a Pterodactyl server.
    GameServer,
    /// A Docker container.
    Container,
    /// A full virtual machine, e.g. Proxmox QEMU.
    Vm,
    /// A Linux container, e.g. Proxmox LXC.
    Lxc,
    /// A system service or daemon.
    Service,
    /// A cluster or hypervisor node.
    Node,
    /// A monitoring check.
    Check,
    /// Anything else, described by `meta`.
    Generic,
}

impl ResourceKind {
    /// Stable string used in the API and translations.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GameServer => "game_server",
            Self::Container => "container",
            Self::Vm => "vm",
            Self::Lxc => "lxc",
            Self::Service => "service",
            Self::Node => "node",
            Self::Check => "check",
            Self::Generic => "generic",
        }
    }
}

/// A single metric sample for a resource. All fields are optional because no
/// two connectors report the same set.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Metrics {
    /// CPU utilisation in percent (0-100, may exceed 100 on multi-core hosts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_percent: Option<f64>,
    /// Resident memory in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_bytes: Option<u64>,
    /// Memory limit in bytes, when the upstream knows one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit_bytes: Option<u64>,
    /// Disk usage in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_bytes: Option<u64>,
    /// Disk limit in bytes, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_limit_bytes: Option<u64>,
    /// Received bytes per second.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_rx_bps: Option<u64>,
    /// Transmitted bytes per second.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_tx_bps: Option<u64>,
    /// Uptime in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_seconds: Option<u64>,
}

/// One resource, normalized across connectors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// Global id.
    pub id: ResourceId,
    /// Normalized kind.
    pub kind: ResourceKind,
    /// Display name.
    pub name: String,
    /// Normalized status.
    pub status: Status,
    /// Latest known metrics, if the connector reports any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Metrics>,
    /// Actions the caller may attempt on this resource.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<crate::ActionKind>,
    /// Connector-specific extras (node, image, tags, ...).
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub meta: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_id_is_url_safe() {
        let id = ResourceId::new(ConnectorId::new(Uuid::nil()), "abc-123");
        assert_eq!(composite_id(&id), "00000000-0000-0000-0000-000000000000/abc-123");
    }

    fn composite_id(id: &ResourceId) -> String {
        id.composite()
    }

    #[test]
    fn metrics_skip_unknown_fields() {
        let metrics = Metrics {
            cpu_percent: Some(12.5),
            ..Metrics::default()
        };
        let json = serde_json::to_value(&metrics).expect("serialise");
        assert_eq!(json["cpu_percent"], 12.5);
        assert!(json.get("memory_bytes").is_none());
    }
}
