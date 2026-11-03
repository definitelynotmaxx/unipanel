//! Control actions and their results.

use serde::{Deserialize, Serialize};

use crate::resource::ResourceId;
use crate::status::Status;
/// A power-style action the user can trigger on a resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "name")]
pub enum ActionKind {
    /// Start the resource.
    Start,
    /// Gracefully stop the resource.
    Stop,
    /// Restart the resource.
    Restart,
    /// Kill the resource immediately.
    Kill,
    /// A connector-specific action, identified by its own name.
    Custom(String),
}

impl ActionKind {
    /// Stable identifier used in audit entries and permissions.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
            Self::Kill => "kill",
            Self::Custom(name) => name,
        }
    }

    /// Whether the action is destructive enough to deserve a confirmation.
    #[must_use]
    pub const fn is_destructive(&self) -> bool {
        matches!(self, Self::Kill | Self::Stop)
    }
}

/// A request to run an action against a resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    /// Target resource.
    pub resource: ResourceId,
    /// Which action.
    pub kind: ActionKind,
}

/// Outcome of an action, as reported by the connector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionResult {
    /// Whether the upstream accepted the request.
    pub accepted: bool,
    /// Status the resource is expected to reach, if the connector knows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_status: Option<Status>,
    /// Optional message key for a human-readable explanation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<crate::error::Message>,
}

impl ActionResult {
    /// A plain successful result.
    #[must_use]
    pub fn accepted(expected_status: Option<Status>) -> Self {
        Self {
            accepted: true,
            expected_status,
            message: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_actions_serialise_with_name() {
        let json = serde_json::to_value(ActionKind::Custom("backup".to_owned())).expect("json");
        assert_eq!(json["kind"], "custom");
        assert_eq!(json["name"], "backup");
    }

    #[test]
    fn simple_actions_serialise_as_json_objects() {
        let json = serde_json::to_value(ActionKind::Start).expect("json");
        // With tag = "kind", unit variants serialize as {"kind": "start"}.
        // (The `content = "name"` only appears when a variant carries data.)
        assert_eq!(json, serde_json::json!({"kind": "start"}));
    }

    #[test]
    fn stop_action_serialises_correctly() {
        let json = serde_json::to_value(ActionKind::Stop).expect("json");
        assert_eq!(json, serde_json::json!({"kind": "stop", "name": null}));
    }
}
