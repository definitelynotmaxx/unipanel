//! Password hashing and policy.

use argon2::password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString};
use argon2::{Argon2, Algorithm, Params, Version};
use rand::rngs::OsRng;
use unipanel_core::ErrorKey;

/// Minimum accepted password length.
pub const MIN_PASSWORD_LEN: usize = 10;
/// Maximum accepted password length, to keep Argon2 work bounded.
pub const MAX_PASSWORD_LEN: usize = 200;

/// Passwords that are common enough to reject outright.
///
/// This is a small, curated list rather than a full dictionary: the goal is to
/// stop the obvious choices, not to replace a breach corpus.
const COMMON_PASSWORDS: &[&str] = &[
    "1234567890",
    "password123",
    "password1234",
    "qwertzuiop",
    "qwertyuiop",
    "iloveyou123",
    "adminadmin",
    "administrator",
    "letmein1234",
    "welcome12345",
    "changeme123",
    "unipanel123",
];

/// A password policy failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PasswordError {
    /// Shorter than [`MIN_PASSWORD_LEN`].
    #[error("password is too short")]
    TooShort,
    /// Longer than [`MAX_PASSWORD_LEN`].
    #[error("password is too long")]
    TooLong,
    /// Present in the common-password list.
    #[error("password is too common")]
    TooCommon,
    /// Equal to the username or display name.
    #[error("password must not contain the account name")]
    ContainsAccountName,
    /// Input could not be hashed (internal error).
    #[error("could not hash password")]
    Hashing,
}

impl PasswordError {
    /// The message key returned to the client.
    #[must_use]
    pub const fn error_key(self) -> ErrorKey {
        match self {
            Self::TooShort | Self::TooLong => ErrorKey::PasswordTooShort,
            Self::TooCommon | Self::ContainsAccountName => ErrorKey::PasswordTooCommon,
            Self::Hashing => ErrorKey::Internal,
        }
    }
}

/// Validation rules for local passwords.
#[derive(Debug, Clone, Copy)]
pub struct PasswordPolicy {
    /// Minimum length in characters.
    pub min_len: usize,
    /// Maximum length in characters.
    pub max_len: usize,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_len: MIN_PASSWORD_LEN,
            max_len: MAX_PASSWORD_LEN,
        }
    }
}

impl PasswordPolicy {
    /// Checks length, the common-password list and account-name reuse.
    ///
    /// # Errors
    /// Returns the specific [`PasswordError`] describing the first violation.
    pub fn validate(&self, password: &str, account_names: &[&str]) -> Result<(), PasswordError> {
        let len = password.chars().count();
        if len < self.min_len {
            return Err(PasswordError::TooShort);
        }
        if password.len() > self.max_len {
            return Err(PasswordError::TooLong);
        }
        let lowered = password.to_lowercase();
        if COMMON_PASSWORDS.iter().any(|common| lowered == *common) {
            return Err(PasswordError::TooCommon);
        }
        for name in account_names {
            let name = name.trim().to_lowercase();
            if name.len() >= 3 && lowered.contains(&name) {
                return Err(PasswordError::ContainsAccountName);
            }
        }
        Ok(())
    }
}

/// Argon2id parameters used for new hashes.
///
/// 19 MiB / 2 iterations / 1 lane is the OWASP baseline and takes roughly 40 ms
/// on a small VPS, which is acceptable for interactive logins.
fn argon2() -> Argon2<'static> {
    let params = Params::new(19_456, 2, 1, None).unwrap_or_default();
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// Hashes a password with Argon2id and a random salt.
///
/// # Errors
/// Returns [`PasswordError::Hashing`] when the hasher rejects the input.
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    argon2()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| PasswordError::Hashing)
}

/// Verifies a password against a stored PHC string.
///
/// Returns `false` for malformed hashes instead of failing, so a corrupt row
/// cannot panic the login route. Callers must still add a constant delay to
/// avoid trivially distinguishing unknown users.
#[must_use]
pub fn verify_password(stored: &str, password: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(stored) else {
        return false;
    };
    argon2().verify_password(password.as_bytes(), &parsed).is_ok()
}

/// A fixed hash used to spend the same CPU time when the username is unknown.
/// Generated lazily and cached for the process lifetime.
#[must_use]
pub fn dummy_hash() -> &'static str {
    use std::sync::OnceLock;
    static DUMMY: OnceLock<String> = OnceLock::new();
    DUMMY.get_or_init(|| {
        hash_password("unipanel-dummy-password-for-timing")
            .unwrap_or_else(|_| String::from("$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHQ$aGFzaGhhc2g"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_and_verifies() {
        let hash = hash_password("correct horse battery staple").expect("hash");
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password(&hash, "correct horse battery staple"));
        assert!(!verify_password(&hash, "wrong"));
    }

    #[test]
    fn hashes_are_salted() {
        let a = hash_password("same password twice").expect("a");
        let b = hash_password("same password twice").expect("b");
        assert_ne!(a, b);
    }

    #[test]
    fn malformed_hash_never_panics() {
        assert!(!verify_password("not-a-hash", "x"));
        assert!(!verify_password("", "x"));
    }

    #[test]
    fn policy_rejects_short_common_and_personal_passwords() {
        let policy = PasswordPolicy::default();
        assert_eq!(policy.validate("short", &[]), Err(PasswordError::TooShort));
        assert_eq!(
            policy.validate("password123", &[]),
            Err(PasswordError::TooCommon)
        );
        assert_eq!(
            policy.validate("my-admin-secret", &["admin"]),
            Err(PasswordError::ContainsAccountName)
        );
        assert!(policy.validate("a-long-and-unusual-passphrase", &["admin"]).is_ok());
    }

    #[test]
    fn dummy_hash_is_stable_and_valid_argon2() {
        assert_eq!(dummy_hash(), dummy_hash());
        assert!(dummy_hash().starts_with("$argon2id$"));
    }
}
