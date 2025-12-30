use aes_gcm::{
    aead::{Aead, AeadCore},
    Aes256Gcm, Key, KeyInit, Nonce,
};
use argon2::Argon2;
use base64::{engine::general_purpose, Engine as _};
use rand::rngs::OsRng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

use crate::models::{AccountConfig, Currency, SeriesIdentifier};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AuthState {
    #[default]
    Unauthorized,
    Restricted,
    Authorized,
}

impl AuthState {
    /// Check if the current state allows the given operation
    pub fn can_perform(&self, required: AuthLevel) -> bool {
        matches!(
            (self, required),
            (_, AuthLevel::None)
                | (
                    AuthState::Restricted | AuthState::Authorized,
                    AuthLevel::Restricted
                )
                | (AuthState::Authorized, AuthLevel::Authorized)
        )
    }

    /// Get the next valid state transition
    pub fn next_state(&self) -> Option<AuthState> {
        match self {
            AuthState::Unauthorized => Some(AuthState::Restricted),
            AuthState::Restricted => Some(AuthState::Authorized),
            AuthState::Authorized => None, // Already at highest level
        }
    }

    /// Check if transition to target state is valid
    pub fn can_transition_to(&self, target: AuthState) -> bool {
        match (self, target) {
            // Can always go to unauthorized (logout)
            (_, AuthState::Unauthorized) => true,
            // Allow idempotent transitions
            (current, target) if *current == target => true,
            // Can only go to restricted from unauthorized
            (AuthState::Unauthorized, AuthState::Restricted) => true,
            // Can move from authorized back to restricted when re-authenticating
            (AuthState::Authorized, AuthState::Restricted) => true,
            // Can only go to authorized from restricted
            (AuthState::Restricted, AuthState::Authorized) => true,
            // Can't skip other transitions
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthLevel {
    None,       // No auth required
    Restricted, // Requires login (session_id)
    Authorized, // Requires full auth (account config)
}

impl From<AuthState> for u8 {
    fn from(state: AuthState) -> Self {
        match state {
            AuthState::Unauthorized => 0,
            AuthState::Restricted => 1,
            AuthState::Authorized => 2,
        }
    }
}

impl From<u8> for AuthState {
    fn from(value: u8) -> Self {
        match value {
            0 => AuthState::Unauthorized,
            1 => AuthState::Restricted,
            _ => AuthState::Authorized,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionState {
    pub auth_state: AuthState,
    pub session_id: String,
    pub client_id: i32,
    pub int_account: i32,
    pub account_config: Option<AccountConfig>,
    pub expires_at: Option<u64>, // Unix timestamp
    pub currency_rates: HashMap<String, Decimal>,
}

#[derive(Debug, Default)]
struct SessionCaches {
    id_to_series: HashMap<String, SeriesIdentifier>,
    isin_to_id: HashMap<String, String>,
    fx_pair_to_product_id: HashMap<(Currency, Currency), String>,
}

impl SessionCaches {
    fn clear(&mut self) {
        self.id_to_series.clear();
        self.isin_to_id.clear();
        self.fx_pair_to_product_id.clear();
    }
}

impl SessionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_authorized(&self) -> bool {
        matches!(self.auth_state, AuthState::Authorized)
    }

    pub fn is_restricted(&self) -> bool {
        matches!(self.auth_state, AuthState::Restricted)
    }

    pub fn can_perform(&self, required: AuthLevel) -> bool {
        self.auth_state.can_perform(required)
    }

    /// Validate that we have the required data for the current auth state
    pub fn is_valid(&self) -> bool {
        // Check if session has expired
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now >= expires_at {
                return false;
            }
        }

        match self.auth_state {
            AuthState::Unauthorized => true,
            AuthState::Restricted => !self.session_id.is_empty(),
            AuthState::Authorized => {
                !self.session_id.is_empty()
                    && self.client_id != 0
                    && self.int_account != 0
                    && self.account_config.is_some()
            }
        }
    }

    /// Set session expiry (default 24 hours from now)
    pub fn set_expiry(&mut self, hours: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.expires_at = Some(now + (hours * 3600));
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now >= expires_at
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    state: Arc<RwLock<SessionState>>,
    caches: Arc<RwLock<SessionCaches>>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(SessionState::default())),
            caches: Arc::new(RwLock::new(SessionCaches::default())),
        }
    }

    pub fn get_state(&self) -> SessionState {
        self.state
            .read()
            .unwrap_or_else(|e| {
                warn!("RwLock poisoned while reading session state, recovering");
                e.into_inner()
            })
            .clone()
    }

    pub fn update_state<F>(&self, f: F)
    where
        F: FnOnce(&mut SessionState),
    {
        let mut state = self.state.write().unwrap_or_else(|e| {
            warn!("RwLock poisoned while updating session state, recovering");
           e.into_inner()
       });
       f(&mut state);
   }

    /// Restore session from a previously saved [`SessionState`].
    ///
    /// This allows consumers to restore sessions from their own storage backends
    /// without using the deprecated file-based persistence.
    pub fn restore_state(&self, restored: SessionState) {
        let mut state = self.state.write().unwrap_or_else(|e| {
            warn!("RwLock poisoned while restoring session state, recovering");
            e.into_inner()
        });
        *state = restored;
    }

   pub fn auth_state(&self) -> AuthState {
       self.state
            .read()
            .unwrap_or_else(|e| {
                warn!("RwLock poisoned while reading auth state, recovering");
                e.into_inner()
            })
            .auth_state
    }

    pub fn set_auth_state(&self, auth_state: AuthState) -> Result<(), crate::error::AuthError> {
        let mut state = self.state.write().unwrap_or_else(|e| {
            warn!("RwLock poisoned while setting auth state, recovering");
            e.into_inner()
        });

        // Validate transition
        if !state.auth_state.can_transition_to(auth_state) {
            warn!(
                "Invalid auth state transition from {:?} to {:?}",
                state.auth_state, auth_state
            );
            return Err(crate::error::AuthError::LoginFailed(format!(
                "Invalid auth state transition from {:?} to {:?}",
                state.auth_state, auth_state
            )));
        }

        info!(
            "Auth state changed: {:?} → {:?}",
            state.auth_state, auth_state
        );
        state.auth_state = auth_state;
        Ok(())
    }

    /// Try to transition to the next auth state if possible
    pub fn advance_auth_state(&self) -> Result<bool, crate::error::AuthError> {
        let mut state = self.state.write().unwrap_or_else(|e| e.into_inner());

        if let Some(next_state) = state.auth_state.next_state() {
            if !state.is_valid() {
                return Err(crate::error::AuthError::SessionNotConfigured);
            }
            state.auth_state = next_state;
            Ok(true)
        } else {
            Ok(false) // Already at highest state
        }
    }

    pub fn session_id(&self) -> String {
        self.state
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .session_id
            .clone()
    }

    pub fn set_session_id(&self, session_id: String) {
        self.state
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .session_id = session_id;
    }

    pub fn client_id(&self) -> i32 {
        self.state
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .client_id
    }

    pub fn set_client_id(&self, client_id: i32) {
        self.state
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .client_id = client_id;
    }

    pub fn int_account(&self) -> i32 {
        self.state
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .int_account
    }

    pub fn set_int_account(&self, int_account: i32) {
        self.state
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .int_account = int_account;
    }

    pub fn account_config(&self) -> Option<AccountConfig> {
        self.state
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .account_config
            .clone()
    }

    pub fn set_account_config(&self, config: AccountConfig) {
        self.state
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .account_config = Some(config);
    }

    pub fn is_authorized(&self) -> bool {
        self.state
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .is_authorized()
    }

    pub fn is_restricted(&self) -> bool {
        self.state
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .is_restricted()
    }

    pub fn clear(&self) {
        self.state
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        self.caches
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }

    pub fn currency_rates(&self) -> HashMap<String, Decimal> {
        self.state
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .currency_rates
            .clone()
    }

    pub fn set_currency_rates(&self, rates: HashMap<String, Decimal>) {
        self.state
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .currency_rates = rates;
    }

    pub fn cache_product_identifiers(
        &self,
        product_id: &str,
        isin: Option<&str>,
        series: Option<&SeriesIdentifier>,
    ) {
        let mut caches = self.caches.write().unwrap_or_else(|e| e.into_inner());
        if let Some(series) = series {
            caches
                .id_to_series
                .insert(product_id.to_string(), series.clone());
        }
        if let Some(isin) = isin {
            let trimmed = isin.trim();
            if !trimmed.is_empty() {
                caches
                    .isin_to_id
                    .insert(trimmed.to_string(), product_id.to_string());
            }
        }
    }

    pub fn cached_series_by_product(&self, product_id: &str) -> Option<SeriesIdentifier> {
        self.caches
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .id_to_series
            .get(product_id)
            .cloned()
    }

    pub fn cached_product_id_by_isin(&self, isin: &str) -> Option<String> {
        self.caches
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .isin_to_id
            .get(isin)
            .cloned()
    }

    pub fn cache_fx_pair_product(&self, from: Currency, to: Currency, product_id: String) {
        self.caches
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .fx_pair_to_product_id
            .insert((from, to), product_id);
    }

    pub fn cached_fx_pair_product(&self, from: Currency, to: Currency) -> Option<String> {
        self.caches
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .fx_pair_to_product_id
            .get(&(from, to))
            .cloned()
    }

    pub fn set_fx_pair_products<I>(&self, pairs: I)
    where
        I: IntoIterator<Item = ((Currency, Currency), String)>,
    {
        let mut caches = self.caches.write().unwrap_or_else(|e| e.into_inner());
        caches.fx_pair_to_product_id.clear();
        caches.fx_pair_to_product_id.extend(pairs.into_iter());
    }

    /// Check if current session can perform an operation requiring given auth level
    pub fn can_perform(&self, required: AuthLevel) -> bool {
        let state = self.state.read().unwrap_or_else(|e| e.into_inner());
        state.can_perform(required) && state.is_valid()
    }

    /// Ensure the session meets the required auth level
    pub fn require_auth(&self, required: AuthLevel) -> Result<(), crate::error::AuthError> {
        if self.can_perform(required) {
            Ok(())
        } else {
            let current_state = self.auth_state();
            Err(crate::error::AuthError::LoginFailed(format!(
                "Operation requires {required:?} auth level, but current state is {current_state:?}"
            )))
        }
    }

    /// Get the minimum auth level required to reach target state
    pub fn auth_level_for_state(state: AuthState) -> AuthLevel {
        match state {
            AuthState::Unauthorized => AuthLevel::None,
            AuthState::Restricted => AuthLevel::Restricted,
            AuthState::Authorized => AuthLevel::Authorized,
        }
    }

    /// Generate encryption key from username and password using Argon2
    fn derive_key(username: &str, password: &str) -> Key<Aes256Gcm> {
        // Use Argon2 for secure key derivation
        let argon2 = Argon2::default();

        // Create a deterministic salt from username to ensure consistent key derivation
        // This allows the same username/password to decrypt saved sessions
        let mut salt_input = [0u8; 16];
        let username_bytes = username.as_bytes();
        let len = username_bytes.len().min(16);
        salt_input[..len].copy_from_slice(&username_bytes[..len]);

        // Use Argon2's output directly as key material
        let mut output_key_material = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), &salt_input, &mut output_key_material)
            .expect("Failed to derive key");

        *Key::<Aes256Gcm>::from_slice(&output_key_material)
    }

    /// Get the session file path for a specific user
    fn session_file_path_for_user(username: &str) -> Result<PathBuf, crate::error::AuthError> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            crate::error::AuthError::LoginFailed("Cannot determine config directory".to_string())
        })?;

        let degiro_dir = config_dir.join(".degiro");

        // Create directory if it doesn't exist
        if !degiro_dir.exists() {
            fs::create_dir_all(&degiro_dir).map_err(|e| {
                crate::error::AuthError::LoginFailed(format!("Cannot create config directory: {e}"))
            })?;
        }

        // Include username hash in filename for multi-user support
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        username.hash(&mut hasher);
        let user_hash = hasher.finish();

        Ok(degiro_dir.join(format!("session_{user_hash:x}.enc")))
    }

    /// Get the session file path (legacy, for backward compatibility)
    fn session_file_path() -> Result<PathBuf, crate::error::AuthError> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            crate::error::AuthError::LoginFailed("Cannot determine config directory".to_string())
        })?;

        let degiro_dir = config_dir.join(".degiro");

        // Create directory if it doesn't exist
        if !degiro_dir.exists() {
            fs::create_dir_all(&degiro_dir).map_err(|e| {
                crate::error::AuthError::LoginFailed(format!("Cannot create config directory: {e}"))
            })?;
        }

        Ok(degiro_dir.join("session.enc"))
   }

   /// Save session state to encrypted file
    #[deprecated(
        since = "0.2.0",
        note = "Use degiro_ox::storage::encrypt_session and handle I/O yourself"
    )]
   pub fn save_session(
       &self,
       username: &str,
       password: &str,
    ) -> Result<(), crate::error::AuthError> {
        let file_path = Self::session_file_path_for_user(username)?;
        let state = self.get_state();

        // Don't save if session is not meaningful
        if matches!(state.auth_state, AuthState::Unauthorized) || state.session_id.is_empty() {
            debug!("Skipping session save - no meaningful session to save");
            return Ok(());
        }

        // Set expiry if not already set (24 hours default)
        let mut state_to_save = state;
        if state_to_save.expires_at.is_none() {
            state_to_save.set_expiry(24);
        }

        // Serialize state
        let json = serde_json::to_string(&state_to_save).map_err(|e| {
            crate::error::AuthError::LoginFailed(format!("Failed to serialize session: {e}"))
        })?;

        // Encrypt
        let key = Self::derive_key(username, password);
        let cipher = Aes256Gcm::new(&key);

        // Generate a random nonce for each encryption
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let encrypted = cipher.encrypt(&nonce, json.as_bytes()).map_err(|e| {
            crate::error::AuthError::LoginFailed(format!("Failed to encrypt session: {e}"))
        })?;

        // Combine nonce and ciphertext for storage
        let mut combined = Vec::new();
        combined.extend_from_slice(&nonce);
        combined.extend_from_slice(&encrypted);

        // Encode and save
        let encoded = general_purpose::STANDARD.encode(combined);

        fs::write(&file_path, encoded).map_err(|e| {
            crate::error::AuthError::LoginFailed(format!("Failed to save session file: {e}"))
        })?;

        // Set restrictive permissions (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&file_path)
                .map_err(|e| {
                    crate::error::AuthError::LoginFailed(format!(
                        "Failed to get file metadata: {e}"
                    ))
                })?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&file_path, perms).map_err(|e| {
                crate::error::AuthError::LoginFailed(format!("Failed to set file permissions: {e}"))
            })?;
        }

        info!("Session saved to {:?}", file_path);
       Ok(())
   }

   /// Load session state from encrypted file
    #[deprecated(
        since = "0.2.0",
        note = "Use degiro_ox::storage::decrypt_session and handle I/O yourself"
    )]
   pub fn load_session(
       &self,
       username: &str,
       password: &str,
    ) -> Result<bool, crate::error::AuthError> {
        let file_path = Self::session_file_path_for_user(username)?;

        if !file_path.exists() {
            debug!("No session file found at {:?}", file_path);
            return Ok(false);
        }

        // Read and decode
        let encoded = fs::read_to_string(&file_path).map_err(|e| {
            crate::error::AuthError::LoginFailed(format!("Failed to read session file: {e}"))
        })?;

        let combined = general_purpose::STANDARD
            .decode(encoded.trim())
            .map_err(|e| {
                crate::error::AuthError::LoginFailed(format!("Failed to decode session: {e}"))
            })?;

        // Extract nonce and ciphertext
        if combined.len() < 12 {
            return Err(crate::error::AuthError::LoginFailed(
                "Invalid session file format".to_string(),
            ));
        }

        let (nonce_bytes, encrypted) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        let key = Self::derive_key(username, password);
        let cipher = Aes256Gcm::new(&key);

        let decrypted = cipher.decrypt(nonce, encrypted).map_err(|e| {
            warn!("Failed to decrypt session - credentials might have changed");
            crate::error::AuthError::LoginFailed(format!("Failed to decrypt session: {e}"))
        })?;

        // Deserialize
        let json = String::from_utf8(decrypted).map_err(|e| {
            crate::error::AuthError::LoginFailed(format!("Invalid session data: {e}"))
        })?;

        let loaded_state: SessionState = serde_json::from_str(&json).map_err(|e| {
            crate::error::AuthError::LoginFailed(format!("Failed to deserialize session: {e}"))
        })?;

        // Check if session is valid and not expired
        if !loaded_state.is_valid() {
            info!("Loaded session is invalid or expired, discarding");
            self.clear_saved_session()?;
            return Ok(false);
        }

        // Update current session
        self.update_state(|state| *state = loaded_state);

        info!("Session loaded successfully from {:?}", file_path);
       Ok(true)
   }

   /// Clear saved session file for a specific user
    #[deprecated(
        since = "0.2.0",
        note = "Handle session file deletion yourself"
    )]
   pub fn clear_saved_session_for_user(
       &self,
       username: &str,
    ) -> Result<(), crate::error::AuthError> {
        let file_path = Self::session_file_path_for_user(username)?;

        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| {
                crate::error::AuthError::LoginFailed(format!("Failed to remove session file: {e}"))
            })?;
            info!("Saved session file removed for user");
        }

       Ok(())
   }

   /// Clear saved session file (legacy)
    #[deprecated(
        since = "0.2.0",
        note = "Handle session file deletion yourself"
    )]
   pub fn clear_saved_session(&self) -> Result<(), crate::error::AuthError> {
       let file_path = Self::session_file_path()?;

        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| {
                crate::error::AuthError::LoginFailed(format!("Failed to remove session file: {e}"))
            })?;
            info!("Saved session file removed");
        }

        Ok(())
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Currency;

    #[test]
    fn test_auth_state_transitions() {
        let session = Session::new();

        // Start unauthorized
        assert_eq!(session.auth_state(), AuthState::Unauthorized);
        assert!(!session.can_perform(AuthLevel::Restricted));
        assert!(!session.can_perform(AuthLevel::Authorized));

        // Can transition to restricted
        session.set_session_id("test_session".to_string());
        session
            .set_auth_state(AuthState::Restricted)
            .expect("Failed to set auth state to Restricted");
        assert_eq!(session.auth_state(), AuthState::Restricted);
        assert!(session.can_perform(AuthLevel::Restricted));
        assert!(!session.can_perform(AuthLevel::Authorized));

        // Set required data and transition to authorized
        session.set_client_id(123);
        session.set_int_account(456);
        session.set_account_config(crate::models::AccountConfig {
            client_id: 123,
            ..Default::default()
        });
        session
            .set_auth_state(AuthState::Authorized)
            .expect("Failed to set auth state to Authorized");
        assert_eq!(session.auth_state(), AuthState::Authorized);
        assert!(session.can_perform(AuthLevel::Authorized));
    }

    #[test]
    fn test_invalid_transitions() {
        let session = Session::new();

        // Can't skip states
        let result = session.set_auth_state(AuthState::Authorized);
        assert!(result.is_err());

        // Can go back to unauthorized
        session
            .set_auth_state(AuthState::Restricted)
            .expect("Failed to set auth state to Restricted");
        session
            .set_auth_state(AuthState::Unauthorized)
            .expect("Failed to set auth state to Unauthorized");
        assert_eq!(session.auth_state(), AuthState::Unauthorized);
    }

    #[test]
    fn test_auth_level_requirements() {
        assert!(AuthState::Unauthorized.can_perform(AuthLevel::None));
        assert!(!AuthState::Unauthorized.can_perform(AuthLevel::Restricted));
        assert!(!AuthState::Unauthorized.can_perform(AuthLevel::Authorized));

        assert!(AuthState::Restricted.can_perform(AuthLevel::None));
        assert!(AuthState::Restricted.can_perform(AuthLevel::Restricted));
        assert!(!AuthState::Restricted.can_perform(AuthLevel::Authorized));

        assert!(AuthState::Authorized.can_perform(AuthLevel::None));
        assert!(AuthState::Authorized.can_perform(AuthLevel::Restricted));
        assert!(AuthState::Authorized.can_perform(AuthLevel::Authorized));
    }

    #[test]
    fn caches_store_product_identifiers() {
        let session = Session::new();

        session.cache_product_identifiers(
            "123",
            Some("US1234567"),
            Some(&SeriesIdentifier::issue_id("vwd123")),
        );

        assert_eq!(
            session.cached_series_by_product("123").unwrap().value(),
            "vwd123"
        );
        assert_eq!(
            session.cached_product_id_by_isin("US1234567").as_deref(),
            Some("123")
        );

        session.clear();

        assert!(session.cached_series_by_product("123").is_none());
        assert!(session.cached_product_id_by_isin("US1234567").is_none());
    }

    #[test]
    fn caches_store_fx_pairs() {
        let session = Session::new();

        session.cache_fx_pair_product(Currency::EUR, Currency::USD, "42".to_string());
        assert_eq!(
            session
                .cached_fx_pair_product(Currency::EUR, Currency::USD)
                .as_deref(),
            Some("42")
        );

        session.set_fx_pair_products([((Currency::GBP, Currency::CHF), "7".to_string())]);

        assert!(session
            .cached_fx_pair_product(Currency::EUR, Currency::USD)
            .is_none());
        assert_eq!(
            session
                .cached_fx_pair_product(Currency::GBP, Currency::CHF)
                .as_deref(),
            Some("7")
        );
    }
}
