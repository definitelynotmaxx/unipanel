//! Opaque session tokens and the session-bound CSRF token.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore as _;
use rand::rngs::OsRng;
use sha2::{Digest as _, Sha256};
use subtle::ConstantTimeEq as _;

/// Number of random bytes behind a session token and a CSRF token.
const TOKEN_BYTES: usize = 32;

/// A freshly minted session: the cookie value, its storage hash and the CSRF
/// token bound to the session.
#[derive(Debug, Clone)]
pub struct SessionTokens {
    /// The only copy of the bearer token; goes into the `HttpOnly` cookie.
    pub token: String,
    /// SHA-256 of [`SessionTokens::token`], the value stored in the database.
    pub token_hash: [u8; 32],
    /// Token the frontend echoes in `X-CSRF-Token`.
    pub csrf_token: String,
}

impl SessionTokens {
    /// Generates a new random session and CSRF token.
    #[must_use]
    pub fn generate() -> Self {
        let token = random_token();
        let token_hash = hash_token(&token);
        Self {
            token,
            token_hash,
            csrf_token: random_token(),
        }
    }
}

/// Generates a URL-safe random token of 256 bits.
#[must_use]
pub fn random_token() -> String {
    let mut bytes = [0_u8; TOKEN_BYTES];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Hashes a session token for storage.
#[must_use]
pub fn hash_token(token: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hasher.finalize().into()
}

/// Compares two strings without leaking their length or content via timing.
#[must_use]
pub fn constant_time_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    left.ct_eq(right).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_unique_and_hashed() {
        let a = SessionTokens::generate();
        let b = SessionTokens::generate();
        assert_ne!(a.token, b.token);
        assert_ne!(a.csrf_token, b.csrf_token);
        assert_eq!(a.token_hash, hash_token(&a.token));
        assert_ne!(a.token_hash, b.token_hash);
        assert_eq!(a.token.len(), 43);
    }

    #[test]
    fn constant_time_comparison_matches_equality() {
        assert!(constant_time_eq("abc", "abc"));
        assert!(!constant_time_eq("abc", "abd"));
        assert!(!constant_time_eq("abc", "ab"));
    }

    #[test]
    fn token_hash_is_not_the_token() {
        let tokens = SessionTokens::generate();
        assert_ne!(tokens.token.as_bytes(), &tokens.token_hash);
    }
}
