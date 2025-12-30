//! Optional session persistence utilities.
//!
//! This module provides encryption/decryption helpers for [`SessionState`] that consumers
//! can use to persist sessions. The library no longer mandates where sessions are stored.
//!
//! # Example
//!
//! ```ignore
//! use degiro_ox::storage;
//! use std::fs;
//!
//! // After login, encrypt and save session
//! let encrypted = storage::encrypt_session(&session_state, "username", "password")?;
//! fs::write("/my/custom/path/session.enc", encrypted)?;
//!
//! // Later, load and decrypt
//! let data = fs::read("/my/custom/path/session.enc")?;
//! let restored = storage::decrypt_session(&data, "username", "password")?;
//! ```

use aes_gcm::{
    aead::{Aead, AeadCore},
    Aes256Gcm, Key, KeyInit,
};
use argon2::Argon2;
use base64::{engine::general_purpose, Engine as _};
use rand::rngs::OsRng;
use rand::RngCore;
use thiserror::Error;

use crate::session::SessionState;

/// Errors that can occur during session storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Failed to serialize session: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("Encryption failed: {0}")]
    Encryption(String),

    #[error("Decryption failed: {0}")]
    Decryption(String),

    #[error("Invalid data format: {0}")]
    InvalidFormat(String),

    #[error("Session has expired")]
    Expired,
}

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

/// Derive an encryption key from password and salt using Argon2.
fn derive_key(password: &str, salt: &[u8]) -> Key<Aes256Gcm> {
    let argon2 = Argon2::default();

    let mut output_key_material = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output_key_material)
        .expect("Failed to derive key");

    *Key::<Aes256Gcm>::from_slice(&output_key_material)
}

/// Encrypt a [`SessionState`] into bytes suitable for storage.
///
/// Uses AES-256-GCM with a key derived from password via Argon2 with a random salt.
/// The returned bytes include the salt and nonce as prefixes and can be safely written to disk.
///
/// Format: `[salt: 16 bytes][nonce: 12 bytes][ciphertext: variable]`
pub fn encrypt_session(
    state: &SessionState,
    _username: &str,
    password: &str,
) -> Result<Vec<u8>, StorageError> {
    let json = serde_json::to_string(state)?;

    // Generate random salt for key derivation
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let encrypted = cipher
        .encrypt(&nonce, json.as_bytes())
        .map_err(|e| StorageError::Encryption(e.to_string()))?;

    // Combine: salt + nonce + ciphertext
    let mut combined = Vec::with_capacity(SALT_LEN + NONCE_LEN + encrypted.len());
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&encrypted);

    Ok(combined)
}

/// Encrypt a [`SessionState`] into a base64-encoded string for text storage.
pub fn encrypt_session_base64(
    state: &SessionState,
    username: &str,
    password: &str,
) -> Result<String, StorageError> {
    let bytes = encrypt_session(state, username, password)?;
    Ok(general_purpose::STANDARD.encode(bytes))
}

/// Decrypt a [`SessionState`] from bytes.
///
/// The input should be the raw bytes produced by [`encrypt_session`].
/// Format expected: `[salt: 16 bytes][nonce: 12 bytes][ciphertext: variable]`
pub fn decrypt_session(
    data: &[u8],
    _username: &str,
    password: &str,
) -> Result<SessionState, StorageError> {
    let min_len = SALT_LEN + NONCE_LEN;
    if data.len() < min_len {
        return Err(StorageError::InvalidFormat(
            "Data too short to contain salt and nonce".to_string(),
        ));
    }

    let (salt, rest) = data.split_at(SALT_LEN);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);
    let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);

    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new(&key);

    let decrypted = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| StorageError::Decryption(e.to_string()))?;

    let state: SessionState = serde_json::from_slice(&decrypted)?;

    // Check expiry
    if !state.is_valid() {
        return Err(StorageError::Expired);
    }

    Ok(state)
}

/// Decrypt a [`SessionState`] from a base64-encoded string.
pub fn decrypt_session_base64(
    encoded: &str,
    username: &str,
    password: &str,
) -> Result<SessionState, StorageError> {
    let data = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| StorageError::InvalidFormat(e.to_string()))?;
    decrypt_session(&data, username, password)
}

/// Get the legacy session file path for a user.
///
/// This returns the path that was used by the deprecated implicit session storage.
/// Provided for migration purposes.
#[deprecated(note = "Use your own storage path. This is for migration only.")]
pub fn legacy_session_path(username: &str) -> Option<std::path::PathBuf> {
    let config_dir = dirs::config_dir()?;
    let degiro_dir = config_dir.join(".degiro");

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    username.hash(&mut hasher);
    let user_hash = hasher.finish();

    Some(degiro_dir.join(format!("session_{user_hash:x}.enc")))
}
