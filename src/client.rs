use arc_swap::ArcSwap;
use async_trait::async_trait;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovRateLimiter,
};
use std::{
    num::NonZeroU32,
    sync::atomic::{AtomicU64, Ordering},
    sync::Arc,
    time::{Duration, Instant},
};

use bon::Builder;
use rust_decimal::Decimal;
use tokio::sync::Semaphore;
use tracing::{debug, info, instrument};

use crate::{
    error::{ClientError, DataError, ResponseError},
    models::{AccountConfig, Currency},
    session::{AuthState, Session},
};

type DegiroRateLimiter = GovRateLimiter<NotKeyed, InMemoryState, DefaultClock>;

fn build_limiter(policy: &RatePolicy) -> DegiroRateLimiter {
    DegiroRateLimiter::direct(policy.quota())
}

fn default_rate_limiter_swap() -> Arc<ArcSwap<DegiroRateLimiter>> {
    Arc::new(ArcSwap::from_pointee(build_limiter(&RatePolicy::default())))
}

fn default_rate_policy_lock() -> Arc<parking_lot::RwLock<RatePolicy>> {
    Arc::new(parking_lot::RwLock::new(RatePolicy::default()))
}

fn default_retry_policy_lock() -> Arc<parking_lot::RwLock<RetryPolicy>> {
    Arc::new(parking_lot::RwLock::new(RetryPolicy::default()))
}

#[derive(Debug, Clone)]
pub struct RatePolicy {
    pub requests_per_second: NonZeroU32,
    pub burst: NonZeroU32,
}

impl RatePolicy {
    #[must_use]
    pub fn new(requests_per_second: NonZeroU32, burst: NonZeroU32) -> Self {
        Self {
            requests_per_second,
            burst,
        }
    }

    pub fn per_second(requests_per_second: u32, burst: u32) -> Result<Self, ClientError> {
        let requests_per_second = NonZeroU32::new(requests_per_second).ok_or_else(|| {
            ClientError::InvalidRequest(
                "rate policy requires requests_per_second greater than zero".to_string(),
            )
        })?;
        let burst = NonZeroU32::new(burst).ok_or_else(|| {
            ClientError::InvalidRequest("rate policy requires burst greater than zero".to_string())
        })?;
        Ok(Self::new(requests_per_second, burst))
    }

    fn quota(&self) -> Quota {
        Quota::per_second(self.requests_per_second).allow_burst(self.burst)
    }
}

impl Default for RatePolicy {
    fn default() -> Self {
        Self::new(
            NonZeroU32::new(8).expect("default requests_per_second"),
            NonZeroU32::new(16).expect("default burst"),
        )
    }
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub min_delay: Duration,
    pub max_delay: Duration,
    pub max_retries: usize,
}

impl RetryPolicy {
    pub fn new(
        min_delay: Duration,
        max_delay: Duration,
        max_retries: usize,
    ) -> Result<Self, ClientError> {
        if max_delay < min_delay {
            return Err(ClientError::InvalidRequest(
                "retry policy requires max_delay to be greater than or equal to min_delay"
                    .to_string(),
            ));
        }
        Ok(Self {
            min_delay,
            max_delay,
            max_retries,
        })
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            min_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            max_retries: 5,
        }
    }
}

#[async_trait]
pub trait CompanyProfileCache: Send + Sync {
    async fn should_skip(&self, product_id: &str) -> bool;
    async fn record_failure(&self, product_id: &str, error: &ClientError);
    async fn record_success(&self, product_id: &str);
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub session_valid: bool,
    pub auth_state: AuthState,
    pub last_successful_request: Option<Instant>,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub rate_limit_remaining: u32,
    pub last_error: Option<(Instant, String)>,
}

#[derive(Clone, Builder)]
pub struct Degiro {
    #[builder(skip = Session::new())]
    pub session: Session,
    #[builder(skip = Arc::new(Semaphore::new(1)))]
    auth_semaphore: Arc<Semaphore>,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) totp_secret: String,
    #[builder(skip)]
    pub cookie_jar: Arc<reqwest_cookie_store::CookieStoreMutex>,
    #[builder(default = reqwest::ClientBuilder::new().https_only(true).cookie_provider(Arc::clone(&cookie_jar)).build().expect("Failed to build HTTP client"))]
    pub(crate) http_client: reqwest::Client,
    #[builder(skip = default_rate_limiter_swap())]
    pub(crate) rate_limiter: Arc<ArcSwap<DegiroRateLimiter>>,
    #[builder(skip = default_rate_policy_lock())]
    rate_policy: Arc<parking_lot::RwLock<RatePolicy>>,
    #[builder(skip = default_retry_policy_lock())]
    retry_policy: Arc<parking_lot::RwLock<RetryPolicy>>,
    #[builder(skip = Arc::new(AtomicU64::new(0)))]
    total_requests: Arc<AtomicU64>,
    #[builder(skip = Arc::new(AtomicU64::new(0)))]
    failed_requests: Arc<AtomicU64>,
    #[builder(skip = Arc::new(parking_lot::RwLock::new(None)))]
    last_successful_request: Arc<parking_lot::RwLock<Option<Instant>>>,
    #[builder(skip = Arc::new(parking_lot::RwLock::new(None)))]
    last_error: Arc<parking_lot::RwLock<Option<(Instant, String)>>>,
    #[builder(skip = Arc::new(parking_lot::RwLock::new(None)))]
    company_profile_cache:
        Arc<parking_lot::RwLock<Option<Arc<dyn CompanyProfileCache + Send + Sync + 'static>>>>,
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

    pub fn auth_state(&self) -> AuthState {
        self.session.auth_state()
    }

    pub(crate) fn set_auth_state(&self, state: AuthState) -> Result<(), crate::error::AuthError> {
        self.session.set_auth_state(state)
    }

    pub fn int_account(&self) -> i32 {
        self.session.int_account()
    }

    pub fn set_int_account(&self, int_account: i32) {
        self.session.set_int_account(int_account);
    }

    pub fn client_id(&self) -> i32 {
        self.session.client_id()
    }

    pub fn set_client_id(&self, client_id: i32) {
        self.session.set_client_id(client_id);
    }

    pub fn session_id(&self) -> String {
        self.session.session_id()
    }

    pub fn set_session_id(&self, session_id: String) {
        self.session.set_session_id(session_id);
    }

    pub fn account_id(&self) -> i32 {
        self.session.client_id()
    }

    pub fn set_account_id(&self, account_id: i32) {
        self.session.set_client_id(account_id);
    }

    pub fn fx_pair_product_id(&self, base: Currency, quote: Currency) -> Option<String> {
        self.session.cached_fx_pair_product(base, quote)
    }

    // pub(crate) fn account_config(&self) -> parking_lot::RwLockReadGuard<Option<AccountConfig>> {
    //     self.account_config.read()
    // }

    pub(crate) fn set_account_config(&self, config: AccountConfig) {
        self.session.set_account_config(config);
    }

    pub fn set_company_profile_cache(
        &self,
        cache: Option<Arc<dyn CompanyProfileCache + Send + Sync + 'static>>,
    ) {
        *self.company_profile_cache.write() = cache;
    }

    pub fn company_profile_cache(
        &self,
    ) -> Option<Arc<dyn CompanyProfileCache + Send + Sync + 'static>> {
        self.company_profile_cache.read().clone()
    }

    pub fn load_from_env() -> Result<Self, ClientError> {
        let username = std::env::var("DEGIRO_USERNAME").map_err(|_| {
            ClientError::MissingCredentials(
                "DEGIRO_USERNAME environment variable not set".to_string(),
            )
        })?;
        let password = std::env::var("DEGIRO_PASSWORD").map_err(|_| {
            ClientError::MissingCredentials(
                "DEGIRO_PASSWORD environment variable not set".to_string(),
            )
        })?;
        let secret = std::env::var("DEGIRO_TOTP_SECRET").map_err(|_| {
            ClientError::MissingCredentials(
                "DEGIRO_TOTP_SECRET environment variable not set".to_string(),
            )
        })?;

        let cookie_jar = Arc::new(reqwest_cookie_store::CookieStoreMutex::default());
        let http_client = reqwest::ClientBuilder::new()
            .https_only(true)
            .cookie_provider(Arc::clone(&cookie_jar))
            .build()
            .map_err(|e| ClientError::ResponseError(ResponseError::network(e.to_string())))?;

        let client = Self {
            session: Session::new(),
            auth_semaphore: Arc::new(Semaphore::new(1)),
            username,
            password,
            totp_secret: secret,
            http_client,
            cookie_jar,
            rate_limiter: default_rate_limiter_swap(),
            rate_policy: default_rate_policy_lock(),
            retry_policy: default_retry_policy_lock(),
            total_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            last_successful_request: Arc::new(parking_lot::RwLock::new(None)),
            last_error: Arc::new(parking_lot::RwLock::new(None)),
            company_profile_cache: Arc::new(parking_lot::RwLock::new(None)),
        };

        // Try to load saved session (skip in test mode)
        #[cfg(not(test))]
        if let Err(e) = client.load_session() {
            debug!("Failed to load saved session: {}", e);
        }

       Ok(client)
   }

   /// Load session from disk if available
    #[deprecated(
        since = "0.2.0",
        note = "Use degiro_ox::storage::decrypt_session and handle I/O yourself"
    )]
   pub fn load_session(&self) -> Result<bool, ClientError> {
       self.session
           .load_session(&self.username, &self.password)
           .map_err(ClientError::AuthError)
   }

   /// Save current session to disk
    #[deprecated(
        since = "0.2.0",
        note = "Use degiro_ox::storage::encrypt_session and handle I/O yourself"
    )]
   pub fn save_session(&self) -> Result<(), ClientError> {
       self.session
           .save_session(&self.username, &self.password)
           .map_err(ClientError::AuthError)
   }

   /// Clear saved session from disk
    #[deprecated(
        since = "0.2.0",
        note = "Handle session file deletion yourself"
    )]
   pub fn clear_saved_session(&self) -> Result<(), ClientError> {
       self.session
           .clear_saved_session()
            .map_err(ClientError::AuthError)
    }

    /// Logout and clear session
    pub fn logout(&self) -> Result<(), ClientError> {
        // Clear in-memory session
        self.session.clear();

        // Clear saved session from disk
        self.clear_saved_session()?;

        info!("Logged out successfully");
        Ok(())
    }

    /// Record a successful request
    pub(crate) fn record_success(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        *self.last_successful_request.write() = Some(Instant::now());
    }

    /// Record a failed request
    pub(crate) fn record_failure(&self, error: &str) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
        *self.last_error.write() = Some((Instant::now(), error.to_string()));
    }

    /// Get current health status
    pub fn health_status(&self) -> HealthStatus {
        let session_state = self.session.get_state();
        let rate_limit_remaining = self.rate_policy.read().burst.get();

        HealthStatus {
            session_valid: session_state.is_valid(),
            auth_state: session_state.auth_state,
            last_successful_request: *self.last_successful_request.read(),
            total_requests: self.total_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            rate_limit_remaining,
            last_error: self.last_error.read().clone(),
        }
    }

    /// Reset health metrics
    pub fn reset_health_metrics(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        *self.last_successful_request.write() = None;
        *self.last_error.write() = None;
    }

    pub(crate) async fn acquire_limit(&self) {
        self.rate_limiter.load().until_ready().await;
    }

    pub fn rate_policy(&self) -> RatePolicy {
        self.rate_policy.read().clone()
    }

    pub fn retry_policy(&self) -> RetryPolicy {
        self.retry_policy.read().clone()
    }

    pub fn set_rate_policy(&self, policy: RatePolicy) -> Result<(), ClientError> {
        let limiter = Arc::new(build_limiter(&policy));
        self.rate_limiter.store(limiter);
        *self.rate_policy.write() = policy;
        Ok(())
    }

    pub fn set_retry_policy(&self, policy: RetryPolicy) -> Result<(), ClientError> {
        if policy.max_delay < policy.min_delay {
            return Err(ClientError::InvalidRequest(
                "retry policy requires max_delay to be greater than or equal to min_delay"
                    .to_string(),
            ));
        }
        *self.retry_policy.write() = policy;
        Ok(())
    }

    pub fn with_rate_policy(self, policy: RatePolicy) -> Result<Self, ClientError> {
        self.set_rate_policy(policy)?;
        Ok(self)
    }

    pub fn with_retry_policy(self, policy: RetryPolicy) -> Result<Self, ClientError> {
        self.set_retry_policy(policy)?;
        Ok(self)
    }

    pub fn is_authorized(&self) -> bool {
        self.session.is_authorized()
    }

    /// Automatically authenticate to the required level
    #[instrument(skip(self), fields(required = ?required, current_state = ?self.session.auth_state()))]
    pub async fn ensure_auth_level(
        &self,
        required: crate::session::AuthLevel,
    ) -> Result<(), ClientError> {
        if self.session.can_perform(required) {
            debug!("Already have required auth level");
            return Ok(());
        }

        info!("Auth level upgrade required, acquiring auth semaphore");
        let _permit = self.auth_semaphore.acquire().await.map_err(|_| {
            ClientError::ResponseError(ResponseError::network(
                "Failed to acquire auth semaphore".to_string(),
            ))
        })?;

        // Double-check after acquiring semaphore
        if self.session.can_perform(required) {
            debug!("Auth level was upgraded by another task");
            return Ok(());
        }

        // Auto-authenticate to required level
        match required {
            crate::session::AuthLevel::None => {
                debug!("No authentication required");
                Ok(())
            }
            crate::session::AuthLevel::Restricted => {
                if !self
                    .session
                    .can_perform(crate::session::AuthLevel::Restricted)
                {
                    info!("Performing login to reach Restricted auth level");
                    self.login().await
                } else {
                    Ok(())
                }
            }
            crate::session::AuthLevel::Authorized => {
                if !self
                    .session
                    .can_perform(crate::session::AuthLevel::Restricted)
                {
                    info!("Performing login to reach Restricted auth level first");
                    self.login().await?;
                }
                if !self
                    .session
                    .can_perform(crate::session::AuthLevel::Authorized)
                {
                    info!("Fetching account config to reach Authorized auth level");
                    self.account_config().await
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Legacy method for backward compatibility
    pub async fn ensure_authorized(&self) -> Result<(), ClientError> {
        self.ensure_auth_level(crate::session::AuthLevel::Authorized)
            .await
    }

    pub(crate) fn build_trading_url(&self, path: &str) -> Result<String, ClientError> {
        use reqwest::Url;

        let session_id = self.session_id();
        let int_account = self.int_account();

        let url = Url::parse(crate::paths::TRADING_URL)
            .map_err(|e| ResponseError::invalid(format!("Invalid TRADING_URL: {e}")))?
            .join(path)
            .map_err(|e| ResponseError::invalid(format!("Invalid path '{path}': {e}")))?
            .join(&format!("{int_account};jsessionid={session_id}"))
            .map_err(|e| ResponseError::invalid(format!("Failed to build URL: {e}")))?;

        Ok(url.to_string())
    }

    pub(crate) fn get_rate(&self, from: Currency, to: Currency) -> Result<Decimal, ClientError> {
        if from == to {
            return Ok(Decimal::ONE);
        }

        let currency_rates = self.session.currency_rates();

        let direct_key = format!("{from}/{to}");
        if let Some(rate) = currency_rates.get(&direct_key) {
            return Ok(*rate);
        }

        let inverse_key = format!("{to}/{from}");
        if let Some(rate) = currency_rates.get(&inverse_key) {
            if rate.is_zero() {
                return Err(ClientError::DataError(DataError::invalid_value(
                    "exchange_rate",
                    format!("Zero exchange rate for {inverse_key}"),
                )));
            }
            return Ok(Decimal::ONE / *rate);
        }

        Err(ClientError::DataError(DataError::parse_error(
            "exchange_rate",
            format!("Exchange rate not available for {from} to {to}"),
        )))
    }
}
