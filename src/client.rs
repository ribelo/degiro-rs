use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicI32, AtomicU8},
        Arc, RwLock,
    },
    time::Duration,
};

use bon::Builder;
use leaky_bucket::RateLimiter;
use thiserror::Error;
use tokio::sync::Semaphore;

use crate::models::AccountConfig;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub text: String,
    #[serde(default)]
    pub additional_info: Option<String>,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(info) = &self.additional_info {
            write!(f, "{}: {}", self.text, info)
        } else {
            write!(f, "{}", self.text)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub errors: Vec<ApiError>,
}

impl Display for ApiErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let errors = self
            .errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{}", errors)
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Session expired or invalid credentials")]
    Unauthorized,

    #[error("Client is missing required configuration")]
    Unconfigured,

    #[error("API error: {0}")]
    ApiError(ApiErrorResponse),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientStatus {
    Unauthorized,
    Restricted,
    Authorized,
}

#[derive(Clone, Builder)]
pub struct Degiro {
    #[builder(skip = Arc::new(AtomicU8::new(ClientStatus::Unauthorized as u8)))]
    pub auth_state: Arc<AtomicU8>,
    #[builder(skip = Arc::new(Semaphore::new(1)))]
    auth_semaphore: Arc<Semaphore>,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) totp_secret: String,
    #[builder(skip)]
    pub session_id: Arc<RwLock<String>>,
    #[builder(skip)]
    pub(crate) client_id: Arc<AtomicI32>,
    #[builder(skip)]
    pub(crate) int_account: Arc<AtomicI32>,
    #[builder(skip)]
    pub account_config: Arc<RwLock<Option<AccountConfig>>>,
    #[builder(skip)]
    pub cookie_jar: Arc<reqwest_cookie_store::CookieStoreMutex>,
    #[builder(default = reqwest::ClientBuilder::new().https_only(true).cookie_provider(Arc::clone(&cookie_jar)).build().unwrap())]
    pub(crate) http_client: reqwest::Client,
    #[builder(default = Arc::new(RateLimiter::builder().initial(12).max(12).refill(12).interval(Duration::from_millis(1000)).build()))]
    pub(crate) rate_limiter: Arc<RateLimiter>,
}

impl Degiro {
    pub fn username(mut self, username: &str) -> Self {
        self.username = username.to_string();

        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = password.to_string();

        self
    }

    pub fn auth_state(&self) -> ClientStatus {
        match self.auth_state.load(std::sync::atomic::Ordering::Relaxed) {
            0 => ClientStatus::Unauthorized,
            1 => ClientStatus::Restricted,
            _ => ClientStatus::Authorized,
        }
    }

    pub(crate) fn set_auth_state(&self, state: ClientStatus) {
        self.auth_state
            .store(state as u8, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn int_account(&self) -> i32 {
        self.int_account.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_int_account(&self, int_account: i32) {
        self.int_account
            .store(int_account, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn client_id(&self) -> i32 {
        self.client_id.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_client_id(&self, client_id: i32) {
        self.client_id
            .store(client_id, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn session_id(&self) -> String {
        self.session_id.read().unwrap().clone()
    }

    pub fn set_session_id(&self, session_id: String) {
        *self.session_id.write().unwrap() = session_id;
    }

    pub fn account_id(&self) -> i32 {
        self.client_id.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_account_id(&self, account_id: i32) {
        self.client_id
            .store(account_id, std::sync::atomic::Ordering::Relaxed)
    }

    // pub(crate) fn account_config(&self) -> parking_lot::RwLockReadGuard<Option<AccountConfig>> {
    //     self.account_config.read()
    // }

    pub(crate) fn set_account_config(&self, config: AccountConfig) {
        *self.account_config.write().unwrap() = Some(config);
    }

    pub fn new_from_env() -> Self {
        let username = std::env::var("DEGIRO_USERNAME").expect("DEGIRO_USERNAME not found");
        let password = std::env::var("DEGIRO_PASSWORD").expect("DEGIRO_PASSWORD not found");
        let secret = std::env::var("DEGIRO_TOTP_SECRET").expect("DEGIRO_PASSWORD not found");

        let cookie_jar = Arc::new(reqwest_cookie_store::CookieStoreMutex::default());
        let http_client = reqwest::ClientBuilder::new()
            .https_only(true)
            .cookie_provider(Arc::clone(&cookie_jar))
            .build()
            .unwrap();

        Self {
            auth_state: Arc::new(AtomicU8::new(ClientStatus::Unauthorized as u8)),
            auth_semaphore: Arc::new(Semaphore::new(1)),
            username,
            password,
            totp_secret: secret,
            session_id: Arc::new(RwLock::new(String::default())),
            client_id: Arc::new(AtomicI32::new(0)),
            int_account: Arc::new(AtomicI32::new(0)),
            account_config: Arc::new(RwLock::new(None)),
            http_client,
            cookie_jar,
            rate_limiter: Arc::new(
                RateLimiter::builder()
                    .initial(12)
                    .max(12)
                    .refill(12)
                    .interval(Duration::from_millis(1000))
                    .build(),
            ),
        }
    }

    pub(crate) async fn acquire_limit(&self) {
        self.rate_limiter.acquire_one().await
    }

    pub fn is_authorized(&self) -> bool {
        self.auth_state.load(std::sync::atomic::Ordering::Relaxed) == ClientStatus::Authorized as u8
    }

    pub async fn ensure_authorized(&self) -> Result<(), ClientError> {
        if self.is_authorized() {
            return Ok(());
        }

        let _permit = self.auth_semaphore.acquire().await.unwrap();
        if self.is_authorized() {
            return Ok(());
        }

        self.authorize().await
    }
}
