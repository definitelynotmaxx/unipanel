//! Single source of truth for the project name, binary name and env prefix.
//!
//! The public name "UniPanel" collides with an unrelated mobile client, so all
//! user-visible naming and every environment variable funnels through this
//! module. Renaming the project should be a mechanical change here plus the
//! package metadata in `Cargo.toml`.

/// Static, compile-time branding information.
#[derive(Debug, Clone, Copy)]
pub struct Brand {
    /// Human readable product name shown in the UI and CLI.
    pub display_name: &'static str,
    /// Lowercase slug used for the binary, image and directory names.
    pub slug: &'static str,
    /// Uppercase prefix for environment variables, including the trailing `_`.
    pub env_prefix: &'static str,
}

impl Brand {
    /// Builds an environment variable name, e.g. `var("DATA_DIR")` -> `UNIPANEL_DATA_DIR`.
    #[must_use]
    pub fn var(self, suffix: &str) -> String {
        format!("{}{suffix}", self.env_prefix)
    }
}

/// The one and only branding constant.
pub const BRAND: Brand = Brand {
    display_name: "UniPanel",
    slug: "unipanel",
    env_prefix: "UNIPANEL_",
};

/// Returns the process-wide [`Brand`].
#[must_use]
pub const fn brand() -> Brand {
    BRAND
}

/// Crate version taken from `CARGO_PKG_VERSION` at compile time.
#[must_use]
pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Version of the storage schema that this build expects.
///
/// The value mirrors the highest migration number in `migrations/` and is
/// reported by `/healthz` variants and the CLI `doctor` command.
pub const SCHEMA_VERSION: i64 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_prefix_is_used_consistently() {
        assert_eq!(brand().var("DATA_DIR"), "UNIPANEL_DATA_DIR");
    }

    #[test]
    fn slug_is_lowercase() {
        assert_eq!(brand().slug, brand().slug.to_lowercase());
    }
}
