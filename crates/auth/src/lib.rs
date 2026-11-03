//! Authentication primitives.
//!
//! M0 ships local accounts only: Argon2id password hashing, opaque server-side
//! sessions, a session-bound CSRF token and a small in-process login throttle.
//! OIDC lands in the access milestone but reuses the session machinery here.

pub mod password;
pub mod rate_limit;
pub mod token;

pub use password::{PasswordPolicy, dummy_hash, hash_password, verify_password};
pub use rate_limit::LoginThrottle;
pub use token::{SessionTokens, constant_time_eq};

/// Absolute lifetime of a browser session.
pub const SESSION_TTL: time::Duration = time::Duration::days(30);
/// Do not write `last_seen_at` more often than this.
pub const SESSION_TOUCH_INTERVAL: time::Duration = time::Duration::minutes(5);
/// Prefix used for every cookie set by UniPanel.
pub const COOKIE_PREFIX: &str = "unipanel";
/// Name of the session cookie (without prefix).
pub const SESSION_COOKIE: &str = "unipanel_session";
/// Name of the CSRF cookie (readable by JavaScript on purpose).
pub const CSRF_COOKIE: &str = "unipanel_csrf";
/// Header the frontend must send for unsafe methods.
pub const CSRF_HEADER: &str = "x-csrf-token";
