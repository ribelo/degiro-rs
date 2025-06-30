use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Source {
    RefinitivLatestNews,
    RefinitivTopNews,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct News {
    pub id: String,
    pub date: DateTime<Utc>,
    pub last_updated: Option<DateTime<Utc>>,
    pub title: String,
    pub brief: Option<String>,
    pub content: String,
    pub source: Source,
    pub language: String,
    pub category: Option<String>,
    pub isins: Vec<String>,
    pub provider: Option<String>,
    pub html_content: bool,
}

// Manual From<&serde_json::Value> implementation removed in favor of serde's automatic derivation.
// This eliminates ~80 lines of manual parsing code and relies on serde's robust error handling.
