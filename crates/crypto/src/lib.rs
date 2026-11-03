//! Symmetric encryption for secrets at rest.
//!
//! Connector configuration blobs (API keys, OIDC client secrets) are sealed
//! with ChaCha20-Poly1305 under a single master key. The master key comes from
//! the environment or a file, never from the database, so a stolen SQLite file
//! alone is useless.
//!
//! Wire format of a sealed blob:
//!
//! ```text
//! [ version:u8 ][ nonce:12 ][ ciphertext + tag ]
//! ```
//!
//! Blobs are returned as raw bytes and stored base64-encoded by the storage
//! layer. The version byte reserves room for key rotation in a later release.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::RngCore as _;
use rand::rngs::OsRng;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Current blob format version.
const VERSION: u8 = 1;
/// Nonce length used by ChaCha20-Poly1305.
const NONCE_LEN: usize = 12;
/// Master key length.
pub const KEY_LEN: usize = 32;

/// A 32-byte master key. Wiped from memory when dropped.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MasterKey([u8; KEY_LEN]);

impl std::fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MasterKey(<redacted>)")
    }
}

impl MasterKey {
    /// Generates a new random master key from the OS CSPRNG.
    #[must_use]
    pub fn generate() -> Self {
        let mut bytes = [0_u8; KEY_LEN];
        OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Parses a base64 (standard alphabet) encoded key.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidKey`] when the input is not base64 or not
    /// exactly 32 bytes.
    pub fn from_base64(encoded: &str) -> Result<Self, CryptoError> {
        let raw = BASE64
            .decode(encoded.trim())
            .map_err(|_| CryptoError::InvalidKey)?;
        Self::from_slice(&raw)
    }

    /// Copies a key from a byte slice.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidKey`] for any length other than 32 bytes.
    pub fn from_slice(raw: &[u8]) -> Result<Self, CryptoError> {
        let bytes: [u8; KEY_LEN] = raw.try_into().map_err(|_| CryptoError::InvalidKey)?;
        Ok(Self(bytes))
    }

    /// Exposes the key material for tests and for the persisting side of a
    /// freshly generated key.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }

    /// Base64 representation, used when writing a generated key to disk.
    #[must_use]
    pub fn to_base64(&self) -> String {
        BASE64.encode(self.0)
    }
}

/// Errors produced by the crypto layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CryptoError {
    /// The key is malformed.
    #[error("invalid master key: expected 32 bytes")]
    InvalidKey,
    /// The blob is too short or uses an unknown format version.
    #[error("invalid sealed blob")]
    InvalidBlob,
    /// Authentication failed: wrong key or tampered data.
    #[error("could not decrypt sealed blob")]
    Decrypt,
    /// Encryption failed (should not happen for in-memory buffers).
    #[error("could not encrypt secret")]
    Encrypt,
}

/// Seals and opens secrets under a [`MasterKey`].
#[derive(Clone)]
pub struct SecretBox {
    cipher: ChaCha20Poly1305,
}

impl std::fmt::Debug for SecretBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretBox(..)")
    }
}

impl SecretBox {
    /// Creates a box from a master key.
    #[must_use]
    pub fn new(key: &MasterKey) -> Self {
        Self {
            cipher: ChaCha20Poly1305::new(Key::from_slice(key.as_bytes())),
        }
    }

    /// Encrypts `plaintext` and binds it to `aad` (for example the connector id
    /// or the settings key, which prevents blob swapping between rows).
    ///
    /// # Errors
    /// Returns [`CryptoError::Encrypt`] if the AEAD rejects the input.
    pub fn seal(&self, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut nonce = [0_u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);
        let ciphertext = self
            .cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|_| CryptoError::Encrypt)?;
        let mut out = Vec::with_capacity(1 + NONCE_LEN + ciphertext.len());
        out.push(VERSION);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    /// Decrypts a blob produced by [`SecretBox::seal`].
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidBlob`] for a malformed header and
    /// [`CryptoError::Decrypt`] when the key or AAD do not match.
    pub fn open(&self, blob: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if blob.len() <= 1 + NONCE_LEN || blob[0] != VERSION {
            return Err(CryptoError::InvalidBlob);
        }
        let nonce = &blob[1..1 + NONCE_LEN];
        let ciphertext = &blob[1 + NONCE_LEN..];
        self.cipher
            .decrypt(
                Nonce::from_slice(nonce),
                Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|_| CryptoError::Decrypt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> MasterKey {
        MasterKey::from_slice(&[7_u8; KEY_LEN]).expect("key")
    }

    #[test]
    fn round_trips() {
        let secret = SecretBox::new(&key());
        let blob = secret.seal(b"hunter2", b"connector:abc").expect("seal");
        assert_eq!(
            secret.open(&blob, b"connector:abc").expect("open"),
            b"hunter2"
        );
    }

    #[test]
    fn nonces_are_unique() {
        let secret = SecretBox::new(&key());
        let a = secret.seal(b"same", b"").expect("seal");
        let b = secret.seal(b"same", b"").expect("seal");
        assert_ne!(a, b);
    }

    #[test]
    fn aad_mismatch_fails() {
        let secret = SecretBox::new(&key());
        let blob = secret.seal(b"secret", b"row-a").expect("seal");
        assert_eq!(secret.open(&blob, b"row-b"), Err(CryptoError::Decrypt));
    }

    #[test]
    fn tampering_fails() {
        let secret = SecretBox::new(&key());
        let mut blob = secret.seal(b"secret", b"").expect("seal");
        let last = blob.len() - 1;
        blob[last] ^= 0xFF;
        assert_eq!(secret.open(&blob, b""), Err(CryptoError::Decrypt));
    }

    #[test]
    fn wrong_key_fails() {
        let a = SecretBox::new(&MasterKey::from_slice(&[1; 32]).expect("key"));
        let b = SecretBox::new(&MasterKey::from_slice(&[2; 32]).expect("key"));
        let blob = a.seal(b"x", b"").expect("seal");
        assert_eq!(b.open(&blob, b""), Err(CryptoError::Decrypt));
    }

    #[test]
    fn key_encoding_round_trips() {
        let key = MasterKey::generate();
        let encoded = key.to_base64();
        let parsed = MasterKey::from_base64(&encoded).expect("parse");
        assert_eq!(parsed.as_bytes(), key.as_bytes());
    }

    #[test]
    fn rejects_bad_keys() {
        assert_eq!(MasterKey::from_base64("nope").unwrap_err(), CryptoError::InvalidKey);
        assert_eq!(
            MasterKey::from_slice(&[0_u8; 31]).unwrap_err(),
            CryptoError::InvalidKey
        );
    }

    #[test]
    fn debug_never_leaks_key_material() {
        let key = MasterKey::generate();
        let debug = format!("{key:?}");
        assert!(!debug.contains(&key.to_base64()));
        assert_eq!(debug, "MasterKey(<redacted>)");
    }
}
