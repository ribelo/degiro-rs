use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Source {
    RefinitivLatestNews,
    RefinitivTopNews,
    Unknown(String),
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

impl From<&serde_json::Value> for News {
    fn from(value: &serde_json::Value) -> Self {
        // Get source field first to avoid later lookups
        let source = if let Some(src) = value.get("source").and_then(|v| v.as_str()) {
            match src {
                "REFINITIV_LATEST_NEWS" => Source::RefinitivLatestNews,
                "REFINITIV_TOP_NEWS" => Source::RefinitivTopNews,
                other => Source::Unknown(other.to_owned()),
            }
        } else {
            Source::Unknown(String::new())
        };

        // Use get() instead of [] to avoid potential panics
        let isins = value
            .get("isins")
            .and_then(|v| v.as_array())
            .map(|arr| {
                // Pre-allocate vector capacity
                let mut vec = Vec::with_capacity(arr.len());
                for isin in arr {
                    if let Some(s) = isin.as_str() {
                        vec.push(s.to_owned());
                    }
                }
                vec
            })
            .unwrap_or_default();

        Self {
            id: value
                .get("id")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default(),
            date: value
                .get("date")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(Utc::now),
            last_updated: value
                .get("lastUpdated")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok()),
            title: value
                .get("title")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default(),
            brief: value
                .get("brief")
                .and_then(|v| v.as_str())
                .map(String::from),
            content: value
                .get("content")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default(),
            source,
            language: value
                .get("language")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default(),
            category: value
                .get("category")
                .and_then(|v| v.as_str())
                .map(String::from),
            isins,
            provider: value
                .get("provider")
                .and_then(|v| v.as_str())
                .map(String::from),
            html_content: value
                .get("htmlContent")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }
    }
}
