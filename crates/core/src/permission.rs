//! RBAC permissions.
//!
//! Only the vocabulary lives here for M0; role storage and evaluation arrive
//! with the access milestone. Keeping the type in `core` now means connectors
//! and the UI can already reason about capabilities.

use serde::{Deserialize, Serialize};

/// One thing a role may be allowed to do on a connector or resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// See the resource and its metrics.
    View,
    /// Run power/control actions.
    Control,
    /// Open a console or log stream.
    Console,
    /// List, read, write and delete files.
    Files,
}

impl Permission {
    /// All permissions, in escalating order.
    pub const ALL: [Self; 4] = [Self::View, Self::Control, Self::Console, Self::Files];

    /// Stable API string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Control => "control",
            Self::Console => "console",
            Self::Files => "files",
        }
    }

    /// The bit used in [`PermissionSet`].
    #[must_use]
    const fn bit(self) -> u8 {
        match self {
            Self::View => 1,
            Self::Control => 1 << 1,
            Self::Console => 1 << 2,
            Self::Files => 1 << 3,
        }
    }
}

/// A compact set of permissions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PermissionSet(u8);

impl PermissionSet {
    /// The empty set.
    #[must_use]
    pub const fn none() -> Self {
        Self(0)
    }

    /// A set containing every permission.
    #[must_use]
    pub const fn all() -> Self {
        Self(0b1111)
    }

    /// Builds a set from an iterator of permissions.
    #[must_use]
    pub fn from_permissions(perms: impl IntoIterator<Item = Permission>) -> Self {
        perms.into_iter().fold(Self::none(), |set, perm| set.with(perm))
    }

    /// Adds a permission.
    #[must_use]
    pub const fn with(mut self, perm: Permission) -> Self {
        self.0 |= perm.bit();
        self
    }

    /// Removes a permission.
    #[must_use]
    pub const fn without(mut self, perm: Permission) -> Self {
        self.0 &= !perm.bit();
        self
    }

    /// Tests for a permission.
    #[must_use]
    pub const fn contains(self, perm: Permission) -> bool {
        self.0 & perm.bit() == perm.bit()
    }

    /// Whether no permission is granted.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Iterates over the granted permissions.
    pub fn iter(self) -> impl Iterator<Item = Permission> {
        Permission::ALL.into_iter().filter(move |p| self.contains(*p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_semantics() {
        let set = PermissionSet::none()
            .with(Permission::View)
            .with(Permission::Files);
        assert!(set.contains(Permission::View));
        assert!(!set.contains(Permission::Control));
        assert_eq!(set.iter().count(), 2);
        assert_eq!(
            PermissionSet::from_permissions([Permission::View, Permission::View]),
            PermissionSet::none().with(Permission::View)
        );
        assert!(PermissionSet::none().is_empty());
        assert_eq!(
            PermissionSet::all().without(Permission::Files).iter().count(),
            3
        );
    }
}
