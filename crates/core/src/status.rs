//! Normalized resource status.

use serde::{Deserialize, Serialize};

/// The lifecycle state of a resource, as far as the connector can tell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Up and healthy.
    Running,
    /// Down cleanly.
    Stopped,
    /// Transitioning into [`Status::Running`].
    Starting,
    /// Transitioning into [`Status::Stopped`].
    Stopping,
    /// Up, but the connector reports a problem.
    Degraded,
    /// The connector could not determine a status.
    #[default]
    Unknown,
}

impl Status {
    /// Stable API string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Stopping => "stopping",
            Self::Degraded => "degraded",
            Self::Unknown => "unknown",
        }
    }

    /// Whether the status represents a settled, healthy-enough state.
    #[must_use]
    pub const fn is_live(self) -> bool {
        matches!(self, Self::Running | Self::Degraded)
    }

    /// Whether a transition is in flight.
    #[must_use]
    pub const fn is_transitional(self) -> bool {
        matches!(self, Self::Starting | Self::Stopping)
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_json() {
        for status in [
            Status::Running,
            Status::Stopped,
            Status::Starting,
            Status::Stopping,
            Status::Degraded,
            Status::Unknown,
        ] {
            let json = serde_json::to_string(&status).expect("serialise");
            let back: Status = serde_json::from_str(&json).expect("deserialise");
            assert_eq!(status, back);
        }
    }

    #[test]
    fn live_and_transitional_are_disjoint() {
        assert!(Status::Running.is_live());
        assert!(!Status::Starting.is_live());
        assert!(Status::Starting.is_transitional());
    }
}
