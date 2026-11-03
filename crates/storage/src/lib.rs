//! Persistence for UniPanel.
//!
//! The M0 schema targets SQLite; Postgres support is planned behind the same
//! repository functions. Timestamps are stored as UNIX seconds in `INTEGER`
//! columns and converted at the crate boundary, so the storage layer does not
//! depend on any SQLx-specific date encoding.

pub mod audit;
pub mod db;
pub mod sessions;
pub mod settings;
pub mod time;
pub mod users;

pub use db::{Db, StorageError};
pub use sessions::{NewSession, Session, SessionWithUser};
pub use settings::InstanceSetup;
pub use users::{NewUser, User};
