use reqwest::StatusCode;
use serde::Deserialize;
use std::fmt::Display;
use thiserror::Error;

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
        write!(f, "{errors}")
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

    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Data error: {0}")]
    DataError(#[from] DataError),

    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),

    #[error("Response format error: {0}")]
    ResponseError(#[from] ResponseError),

    #[error("Date/time error: {0}")]
    DateTimeError(#[from] DateTimeError),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Missing credentials: {0}")]
    MissingCredentials(String),
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Missing required field '{field}'")]
    MissingField { field: String },

    #[error("Invalid type for field '{field}': expected {expected}")]
    InvalidType {
        field: &'static str,
        expected: &'static str,
    },

    #[error("Failed to parse {entity}: {reason}")]
    ParseError {
        entity: &'static str,
        reason: String,
    },

    #[error("Invalid value for {field}: {value}")]
    InvalidValue { field: &'static str, value: String },
}

impl DataError {
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    pub fn invalid_type(field: &'static str, expected: &'static str) -> Self {
        Self::InvalidType { field, expected }
    }

    pub fn parse_error(entity: &'static str, reason: impl Into<String>) -> Self {
        Self::ParseError {
            entity,
            reason: reason.into(),
        }
    }

    pub fn invalid_value(field: &'static str, value: impl Into<String>) -> Self {
        Self::InvalidValue {
            field,
            value: value.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid TOTP secret: {0}")]
    InvalidTotpSecret(String),

    #[error("TOTP generation failed: {0}")]
    TotpGenerationFailed(String),

    #[error("Session not configured")]
    SessionNotConfigured,

    #[error("Login failed: {0}")]
    LoginFailed(String),
}

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("Unexpected response structure: {0}")]
    UnexpectedStructure(String),

    #[error("Unknown {entity_type}: '{value}'")]
    UnknownValue {
        entity_type: &'static str,
        value: String,
    },

    #[error("Empty response when data was expected")]
    EmptyResponse,

    #[error("Invalid response: {0}")]
    Invalid(String),

    #[error("HTTP {status}: {body}")]
    HttpStatus { status: StatusCode, body: String },
}

impl ResponseError {
    pub fn unexpected_structure(description: impl Into<String>) -> Self {
        Self::UnexpectedStructure(description.into())
    }

    pub fn unknown_value(entity_type: &'static str, value: impl Into<String>) -> Self {
        Self::UnknownValue {
            entity_type,
            value: value.into(),
        }
    }

    pub fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }

    pub fn network(reason: impl Into<String>) -> Self {
        Self::Invalid(format!("Network error: {}", reason.into()))
    }

    pub fn http_status(status: StatusCode, body: impl Into<String>) -> Self {
        Self::HttpStatus {
            status,
            body: body.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum DateTimeError {
    #[error("Failed to parse date '{input}': {reason}")]
    ParseError { input: String, reason: String },

    #[error("Invalid date calculation: {0}")]
    InvalidCalculation(String),

    #[error("Chrono error: {0}")]
    ChronoError(#[from] chrono::ParseError),
}

impl DateTimeError {
    pub fn parse_error(input: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ParseError {
            input: input.into(),
            reason: reason.into(),
        }
    }
}
