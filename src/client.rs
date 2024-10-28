use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use derivative::Derivative;
use leaky_bucket::RateLimiter;
use thiserror::Error;

use crate::api::account::AccountConfig;

#[allow(dead_code)]
#[derive(Clone, Debug, Derivative)]
#[derivative(Default)]
pub struct Paths {
    #[derivative(Default(value = r#""https://trader.degiro.nl/trader/".to_string()"#))]
    pub(crate) referer: String,
    #[derivative(Default(value = r#""login/secure/login".to_string()"#))]
    pub(crate) login_url_path: String,
    #[derivative(Default(value = r#""v5/checkorder".to_string()"#))]
    pub(crate) create_order_path: String,
    #[derivative(Default(value = r#""secure/v5/order".to_string()"#))]
    pub(crate) order_path: String,
    #[derivative(Default(value = r#""v4/transactions".to_string()"#))]
    pub(crate) transactions_path: String,
    #[derivative(Default(value = r#""settings/user".to_string()"#))]
    pub(crate) web_user_settings_path: String,
    #[derivative(Default(value = r#""login/secure/config".to_string()"#))]
    pub(crate) account_config_path: String,
    #[derivative(Default(value = r#""document/download/".to_string()"#))]
    pub(crate) base_report_download_uri: String,
    #[derivative(Default(value = r#""https://trader.degiro.nl/".to_string()"#))]
    pub(crate) base_api_url: String,
    #[derivative(Default(value = r#""newsfeed/v2/top_news_preview".to_string()"#))]
    pub(crate) top_news_path: String,
    #[derivative(Default(value = r#""settings/web".to_string()"#))]
    pub(crate) web_settings_path: String,
    #[derivative(Default(value = r#""newsfeed/v2/latest_news".to_string()"#))]
    pub(crate) latests_news_path: String,
    #[derivative(Default(value = r#""v5/account/info/".to_string()"#))]
    pub(crate) account_info_path: String,
    #[derivative(Default(value = r#""v5/stocks".to_string()"#))]
    pub(crate) stocks_search_path: String,
    #[derivative(Default(
        value = r#""https://charting.vwdservices.com/hchart/v1/deGiro/data.js".to_string()"#
    ))]
    pub(crate) price_data_url: String,
    #[derivative(Default(value = r#""trading/secure/logout".to_string()"#))]
    pub(crate) logout_url_path: String,
    #[derivative(Default(value = r#""v5/update/".to_string()"#))]
    pub(crate) generic_data_path: String,
    #[derivative(Default(
        value = r#""https://charting.vwdservices.com/hchart/v1/deGiro/data.js".to_string()"#
    ))]
    pub(crate) chart_data_url: String,
    pub(crate) products_search_url: String,
    pub(crate) pa_url: String,
    pub(crate) trading_url: String,
    pub(crate) reporting_url: String,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("you have been logged out")]
    Unauthorized,

    #[error("login error: {source}")]
    LoginError {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("unexpected error: {source}")]
    UnexpectedError {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("can't parse product")]
    ProductParseError,

    #[error("can't find any product")]
    ProductSearchError,

    #[error("can't parse: {0}")]
    ParseError(String),

    #[error("request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("serialization/deserialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("order not found: {0}")]
    OrderNotFoundError(String),

    #[error("unexpected statement: {0}")]
    UnexpectedStatementType(String),

    #[error("No data found")]
    NoData,

    #[error("DegiroError: {0}")]
    Descripted(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientStatus {
    Unauthorized,
    Configured,
    Restricted,
    Authorized,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ClientRef {
    pub status: ClientStatus,
    pub(crate) username: String,
    pub(crate) password: String,
    pub session_id: String,
    pub(crate) client_id: i32,
    pub(crate) int_account: i32,
    pub(crate) base_api_url: String,
    pub(crate) referer: String,
    pub account_config: AccountConfig,
    pub(crate) http_client: reqwest::Client,
    pub cookie_jar: Arc<reqwest_cookie_store::CookieStoreMutex>,
    #[derivative(Debug = "ignore")]
    pub(crate) rate_limiter: Arc<RateLimiter>,
}

#[derive(Clone, Debug)]
pub struct Client {
    pub inner: Arc<Mutex<ClientRef>>,
}

#[derive(Debug, Default)]
pub struct ClientBuilder {
    pub username: Option<String>,
    pub password: Option<String>,
    pub secret_key: Option<String>,
    pub cookie_jar: Option<Arc<reqwest_cookie_store::CookieStoreMutex>>,
}

impl ClientBuilder {
    pub fn username(mut self, username: &str) -> Self {
        self.username = Some(username.to_string());

        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());

        self
    }

    pub fn cookie_jar(mut self, cookie_jar: Arc<reqwest_cookie_store::CookieStoreMutex>) -> Self {
        self.cookie_jar = Some(cookie_jar);
        self
    }

    pub fn from_env() -> Self {
        let username = std::env::var("DEGIRO_USERNAME").expect("DEGIRO_USERNAME not found");
        let password = std::env::var("DEGIRO_PASSWORD").expect("DEGIRO_PASSWORD not found");
        let secret = std::env::var("DEGIRO_SECRET").expect("DEGIRO_PASSWORD not found");

        Self {
            username: Some(username),
            password: Some(password),
            secret_key: Some(secret),
            cookie_jar: None,
        }
    }

    pub fn build(&mut self) -> Result<Client, reqwest::Error> {
        let cookie_jar = self.cookie_jar.take().unwrap_or_default();
        let http_client = reqwest::ClientBuilder::new()
            .https_only(true)
            .cookie_provider(Arc::clone(&cookie_jar))
            .build()?;

        let client = Client::new(
            self.username.as_ref().unwrap().to_string(),
            self.password.as_ref().unwrap().to_string(),
            http_client,
            cookie_jar,
        );

        Ok(client)
    }
}

impl ClientRef {
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        http_client: reqwest::Client,
        cookie_jar: Arc<reqwest_cookie_store::CookieStoreMutex>,
    ) -> Self {
        let username = username.into();
        let password = password.into();
        Self {
            status: ClientStatus::Unauthorized,
            username,
            password,
            http_client,
            cookie_jar,
            session_id: Default::default(),
            client_id: Default::default(),
            int_account: Default::default(),
            base_api_url: "https://trader.degiro.nl/".to_string(),
            referer: "https://trader.degiro.nl/trader/".to_string(),
            account_config: Default::default(),
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
}

impl Client {
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        http_client: reqwest::Client,
        cookie_jar: Arc<reqwest_cookie_store::CookieStoreMutex>,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ClientRef::new(
                username,
                password,
                http_client,
                cookie_jar,
            ))),
        }
    }
    pub fn new_from_env() -> Self {
        let username = std::env::var("DEGIRO_USERNAME").expect("DEGIRO_USERNAME not found");
        let password = std::env::var("DEGIRO_PASSWORD").expect("DEGIRO_PASSWORD not found");

        let cookie_jar = Arc::new(reqwest_cookie_store::CookieStoreMutex::default());
        let http_client = reqwest::ClientBuilder::new()
            .https_only(true)
            .cookie_provider(Arc::clone(&cookie_jar))
            .build()
            .unwrap();

        Self::new(username, password, http_client, cookie_jar)
    }
}
