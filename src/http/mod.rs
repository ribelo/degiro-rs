use std::sync::Arc;
use std::time::Duration;

use leaky_bucket::RateLimiter;
use reqwest::{header, Method, StatusCode, Url};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

use crate::client::{ApiErrorResponse, ClientError};

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("API returned error: {0}")]
    ApiError(ApiErrorResponse),
    
    #[error("Unauthorized - session expired or invalid credentials")]
    Unauthorized,
    
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(#[from] serde_json::Error),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

impl From<HttpError> for ClientError {
    fn from(err: HttpError) -> Self {
        match err {
            HttpError::Unauthorized => ClientError::Unauthorized,
            HttpError::ApiError(api_err) => ClientError::ApiError(api_err),
            HttpError::RequestFailed(req_err) => ClientError::RequestError(req_err),
            HttpError::DeserializationFailed(serde_err) => ClientError::SerdeError(serde_err),
            HttpError::InvalidUrl(msg) => ClientError::UnexpectedError(msg),
        }
    }
}

/// A simple HTTP client wrapper that handles rate limiting and common headers
pub struct HttpClient {
    inner: reqwest::Client,
    rate_limiter: Arc<RateLimiter>,
    base_url: Url,
}

impl HttpClient {
    pub fn new(base_url: &str) -> Result<Self, HttpError> {
        let base_url = Url::parse(base_url)
            .map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        
        let cookie_jar = Arc::new(reqwest_cookie_store::CookieStoreMutex::default());
        let inner = reqwest::ClientBuilder::new()
            .https_only(true)
            .cookie_provider(cookie_jar)
            .timeout(Duration::from_secs(30))
            .build()?;
        
        let rate_limiter = Arc::new(
            RateLimiter::builder()
                .initial(12)
                .max(12)
                .refill(12)
                .interval(Duration::from_millis(1000))
                .build()
        );
        
        Ok(Self {
            inner,
            rate_limiter,
            base_url,
        })
    }
    
    /// Build a request for the given path
    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        RequestBuilder::new(self, method, path)
    }
    
    /// Convenience method for GET requests
    pub fn get(&self, path: &str) -> RequestBuilder {
        self.request(Method::GET, path)
    }
    
    /// Convenience method for POST requests
    pub fn post(&self, path: &str) -> RequestBuilder {
        self.request(Method::POST, path)
    }
    
    /// Execute a pre-built request
    async fn execute(&self, request: reqwest::Request) -> Result<reqwest::Response, HttpError> {
        // Rate limiting
        self.rate_limiter.acquire_one().await;
        
        // Send request
        let response = self.inner.execute(request).await?;
        
        // Check status
        match response.status() {
            StatusCode::UNAUTHORIZED => Err(HttpError::Unauthorized),
            status if status.is_client_error() || status.is_server_error() => {
                // Try to parse API error
                if let Ok(api_error) = response.json::<ApiErrorResponse>().await {
                    Err(HttpError::ApiError(api_error))
                } else {
                    Err(HttpError::RequestFailed(
                        format!("Request failed with status: {}", status).into()
                    ))
                }
            }
            _ => Ok(response)
        }
    }
}

/// A builder for HTTP requests
pub struct RequestBuilder<'a> {
    client: &'a HttpClient,
    method: Method,
    path: String,
    headers: header::HeaderMap,
    query: Vec<(String, String)>,
    body: Option<Vec<u8>>,
}

impl<'a> RequestBuilder<'a> {
    fn new(client: &'a HttpClient, method: Method, path: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::REFERER, 
            header::HeaderValue::from_static("https://trader.degiro.nl/trader/")
        );
        
        Self {
            client,
            method,
            path: path.to_string(),
            headers,
            query: Vec::new(),
            body: None,
        }
    }
    
    /// Add a header to the request
    pub fn header(mut self, key: header::HeaderName, value: &str) -> Self {
        if let Ok(header_value) = header::HeaderValue::from_str(value) {
            self.headers.insert(key, header_value);
        }
        self
    }
    
    /// Add query parameters
    pub fn query<T: Serialize>(mut self, params: &T) -> Self {
        if let Ok(query_string) = serde_urlencoded::to_string(params) {
            for pair in query_string.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    self.query.push((key.to_string(), value.to_string()));
                }
            }
        }
        self
    }
    
    /// Add a single query parameter
    pub fn query_param(mut self, key: &str, value: &str) -> Self {
        self.query.push((key.to_string(), value.to_string()));
        self
    }
    
    /// Set JSON body
    pub fn json<T: Serialize>(mut self, body: &T) -> Result<Self, HttpError> {
        self.headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json")
        );
        self.body = Some(serde_json::to_vec(body)?);
        Ok(self)
    }
    
    /// Send the request and get raw response
    pub async fn send(self) -> Result<reqwest::Response, HttpError> {
        let mut url = self.client.base_url.join(&self.path)
            .map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        
        // Add query parameters
        if !self.query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in self.query {
                pairs.append_pair(&key, &value);
            }
        }
        
        // Build request
        let mut request = self.client.inner.request(self.method, url);
        request = request.headers(self.headers);
        
        if let Some(body) = self.body {
            request = request.body(body);
        }
        
        // Execute
        self.client.execute(request.build()?).await
    }
    
    /// Send the request and deserialize JSON response
    pub async fn json<T: DeserializeOwned>(self) -> Result<T, HttpError> {
        let response = self.send().await?;
        let body = response.json::<T>().await?;
        Ok(body)
    }
    
    /// Send the request and get text response
    pub async fn text(self) -> Result<String, HttpError> {
        let response = self.send().await?;
        let text = response.text().await?;
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_creation() {
        let client = HttpClient::new("https://trader.degiro.nl/").unwrap();
        assert_eq!(client.base_url.as_str(), "https://trader.degiro.nl/");
    }
}