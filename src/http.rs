use backon::{ExponentialBuilder, Retryable};
use reqwest::{header, Method, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};

use crate::{
    client::Degiro,
    error::{ApiErrorResponse, ClientError, ResponseError},
    paths::REFERER,
    session::{AuthLevel, AuthState},
};

pub struct HttpRequest {
    method: Method,
    path: String,
    query_params: Vec<(String, String)>,
    body: Option<serde_json::Value>,
    auth_level: AuthLevel,
    custom_headers: HashMap<String, String>,
}

impl HttpRequest {
    pub fn get(path: impl Into<String>) -> Self {
        Self {
            method: Method::GET,
            path: path.into(),
            query_params: Vec::new(),
            body: None,
            auth_level: AuthLevel::Authorized, // Default to highest security
            custom_headers: HashMap::new(),
        }
    }

    pub fn post(path: impl Into<String>) -> Self {
        Self {
            method: Method::POST,
            path: path.into(),
            query_params: Vec::new(),
            body: None,
            auth_level: AuthLevel::Authorized, // Default to highest security
            custom_headers: HashMap::new(),
        }
    }

    pub fn put(path: impl Into<String>) -> Self {
        Self {
            method: Method::PUT,
            path: path.into(),
            query_params: Vec::new(),
            body: None,
            auth_level: AuthLevel::Authorized, // Default to highest security
            custom_headers: HashMap::new(),
        }
    }

    pub fn delete(path: impl Into<String>) -> Self {
        Self {
            method: Method::DELETE,
            path: path.into(),
            query_params: Vec::new(),
            body: None,
            auth_level: AuthLevel::Authorized, // Default to highest security
            custom_headers: HashMap::new(),
        }
    }

    #[must_use]
    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.push((key.into(), value.into()));
        self
    }

    #[must_use]
    pub fn queries<I, K, V>(mut self, params: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (key, value) in params {
            self.query_params.push((key.into(), value.into()));
        }
        self
    }

    pub fn json<T: Serialize>(mut self, body: &T) -> Result<Self, ClientError> {
        self.body = Some(serde_json::to_value(body)?);
        Ok(self)
    }

    #[must_use]
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_headers.insert(key.into(), value.into());
        self
    }

    #[must_use]
    pub fn no_auth(mut self) -> Self {
        self.auth_level = AuthLevel::None;
        self
    }

    #[must_use]
    pub fn require_restricted(mut self) -> Self {
        self.auth_level = AuthLevel::Restricted;
        self
    }

    #[must_use]
    pub fn require_authorized(mut self) -> Self {
        self.auth_level = AuthLevel::Authorized;
        self
    }
}

// Extension trait to add HTTP methods to Degiro
#[async_trait::async_trait]
pub trait HttpClient {
    async fn request<T: DeserializeOwned>(&self, req: HttpRequest) -> Result<T, ClientError>;
    async fn request_json(&self, req: HttpRequest) -> Result<serde_json::Value, ClientError>;
    async fn request_empty(&self, req: HttpRequest) -> Result<(), ClientError>;
    async fn request_text(&self, req: HttpRequest) -> Result<String, ClientError>;
}

#[async_trait::async_trait]
impl HttpClient for Degiro {
    async fn request<T: DeserializeOwned>(&self, req: HttpRequest) -> Result<T, ClientError> {
        let res = self.execute_request(req).await?;
        Ok(res.json::<T>().await?)
    }

    async fn request_json(&self, req: HttpRequest) -> Result<serde_json::Value, ClientError> {
        let res = self.execute_request(req).await?;
        Ok(res.json::<serde_json::Value>().await?)
    }

    async fn request_empty(&self, req: HttpRequest) -> Result<(), ClientError> {
        self.execute_request(req).await?;
        Ok(())
    }

    async fn request_text(&self, req: HttpRequest) -> Result<String, ClientError> {
        let res = self.execute_request(req).await?;
        Ok(res.text().await?)
    }
}

impl Degiro {
    #[instrument(skip(self, req), fields(method = %req.method, path = %req.path, auth_level = ?req.auth_level))]
    async fn execute_request(&self, req: HttpRequest) -> Result<Response, ClientError> {
        // Auto-authenticate to required level (outside retry logic)
        debug!("Ensuring authentication level: {:?}", req.auth_level);
        self.ensure_auth_level(req.auth_level).await?;

        // Create the retry policy
        let retry_cfg = self.retry_policy();
        let retry_policy = ExponentialBuilder::default()
            .with_min_delay(retry_cfg.min_delay)
            .with_max_delay(retry_cfg.max_delay)
            .with_max_times(retry_cfg.max_retries);

        info!("Executing HTTP request with retry policy");

        // Execute request with retry logic (auth check already done)
        match (|| self.execute_single_request(&req))
            .retry(&retry_policy)
            .when(|e| {
                // Only retry on transient errors, not auth errors
                matches!(e,
                    ClientError::RequestError(reqwest_err)
                        if reqwest_err.is_timeout() || reqwest_err.is_connect() ||
                           reqwest_err.status().is_some_and(|s|
                               matches!(s.as_u16(), 500 | 502 | 503 | 504 | 429))
                )
            })
            .await
        {
            Ok(response) => {
                info!("HTTP request completed successfully");
                self.record_success();
                Ok(response)
            }
            Err(e) => {
                error!("HTTP request failed after retries: {}", e);
                self.record_failure(&e.to_string());
                Err(e)
            }
        }
    }

    async fn execute_single_request(&self, req: &HttpRequest) -> Result<Response, ClientError> {
        // Build URL with query parameters
        let url = if req.query_params.is_empty() {
            req.path.clone()
        } else {
            let params: Vec<String> = req
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect();
            format!("{}?{}", req.path, params.join("&"))
        };

        debug!("Built request URL: {}", url);

        // Build request
        let mut request_builder = match req.method {
            Method::GET => self.http_client.get(&url),
            Method::POST => self.http_client.post(&url),
            Method::PUT => self.http_client.put(&url),
            Method::DELETE => self.http_client.delete(&url),
            _ => {
                return Err(ClientError::InvalidRequest(format!(
                    "Unsupported HTTP method: {:?}",
                    req.method
                )))
            }
        };

        // Add default headers
        request_builder = request_builder.header(header::REFERER, REFERER);

        // Add custom headers
        for (key, value) in &req.custom_headers {
            request_builder = request_builder.header(key, value);
        }

        // Add body if present
        if let Some(body) = &req.body {
            debug!("Adding JSON body to request");
            request_builder = request_builder.json(body);
        }

        // Apply rate limiting
        debug!("Acquiring rate limit");
        self.acquire_limit().await;

        // Send request
        debug!("Sending HTTP request");
        let res = request_builder.send().await.map_err(|e| {
            warn!("Network error occurred: {}", e);
            // All network errors should be retried
            ClientError::RequestError(e)
        })?;

        // Handle HTTP status errors
        if let Err(err) = res.error_for_status_ref() {
            let Some(status) = err.status() else {
                error!("HTTP error without status code: {}", err);
                return Err(ResponseError::invalid(err.to_string()).into());
            };

            match status {
                StatusCode::UNAUTHORIZED => {
                    warn!("Received 401 Unauthorized, clearing auth state");
                    let _ = self.set_auth_state(AuthState::Unauthorized);
                    return Err(ClientError::Unauthorized);
                }
                // Retry server errors and rate limits
                StatusCode::INTERNAL_SERVER_ERROR
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
                | StatusCode::TOO_MANY_REQUESTS => {
                    warn!(
                        "Received retryable HTTP error: {} {}",
                        status.as_u16(),
                        status.canonical_reason().unwrap_or("")
                    );

                    return Err(ClientError::RequestError(err));
                }
                // Don't retry client errors (4xx except 429)
                _ => {
                    error!(
                        "Received non-retryable HTTP error: {} {}",
                        status.as_u16(),
                        status.canonical_reason().unwrap_or("")
                    );

                    let body_text = res.text().await.unwrap_or_default();
                    if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body_text)
                    {
                        error!("API error response: {:?}", error_response);
                        return Err(ClientError::ApiError(error_response));
                    }

                    return Err(ResponseError::http_status(status, body_text).into());
                }
            }
        }

        debug!("HTTP request completed with status: {}", res.status());

        Ok(res)
    }
}
