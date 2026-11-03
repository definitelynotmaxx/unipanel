//! Domain types shared by every UniPanel crate.
//!
//! This crate deliberately has no I/O and no framework dependencies: it is the
//! vocabulary that the server, storage, connectors and CLI all speak.

pub mod action;
pub mod brand;
pub mod error;
pub mod locale;
pub mod permission;
pub mod resource;
pub mod status;

pub use action::{Action, ActionKind, ActionResult};
pub use brand::{Brand, SCHEMA_VERSION, brand, version};
pub use error::{ErrorKey, Message};
pub use locale::Locale;
pub use permission::{Permission, PermissionSet};
pub use resource::{ConnectorId, Metrics, Resource, ResourceId, ResourceKind};
pub use status::Status;
