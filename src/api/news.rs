use chrono::{DateTime, Utc};
use reqwest::{header, Url};
use serde::{Deserialize, Serialize};

use crate::client::{Client, ClientError, ClientStatus};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct News {
    pub id: String,
    pub date: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub title: String,
    pub brief: Option<String>,
    pub content: String,
    pub source: Source,
    pub language: String,
    pub category: Option<String>,
    pub isins: Vec<String>,
    pub provider: String,
    pub html_content: bool,
}

impl News {
    pub fn new(item: &serde_json::Value) -> Self {
        let source = match item["source"].as_str().unwrap() {
            "REFINITIV_LATEST_NEWS" => Source::RefinitivLatestNews,
            "REFINITIV_TOP_NEWS" => Source::RefinitivTopNews,
            other => Source::Unknown(other.to_string()),
        };
        Self {
            id: item["id"].as_str().unwrap().to_string(),
            date: item["date"].as_str().unwrap().parse().unwrap(),
            last_updated: item["lastUpdated"].as_str().unwrap().parse().unwrap(),
            title: item["title"].as_str().unwrap().to_string(),
            brief: item["brief"].as_str().map(|s| s.to_string()),
            content: item["content"].as_str().unwrap().to_string(),
            source,
            language: item["language"].as_str().unwrap().to_string(),
            category: item["category"].as_str().map(|s| s.to_string()),
            isins: item["isins"]
                .as_array()
                .unwrap()
                .iter()
                .map(|isin| isin.as_str().unwrap().to_string())
                .collect(),
            provider: item["provider"].as_str().unwrap().to_string(),
            html_content: item["htmlContent"].as_bool().unwrap(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Source {
    RefinitivLatestNews,
    RefinitivTopNews,
    Unknown(String),
}

impl Client {
    pub async fn company_news<T: AsRef<str>>(&self, isin: T) -> Result<Vec<News>, ClientError> {
        if self.inner.lock().unwrap().status != ClientStatus::Authorized {
            return Err(ClientError::Unauthorized);
        }
        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = "https://trader.degiro.nl/";
            let path_url = "/dgtbxdsservice/newsfeed/v2/news-by-company/";
            let url = Url::parse(base_url).unwrap().join(path_url).unwrap();

            inner
                .http_client
                .get(url)
                .query(&[
                    ("isin", isin.as_ref()),
                    ("intAccount", &inner.int_account.to_string()),
                    ("sessionId", &inner.session_id),
                    ("limit", "10"),
                    ("offset", "0"),
                    ("languages", "en,pl"),
                ])
                .header(header::REFERER, &inner.referer)
                .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.to_string())
        };

        let rate_limiter = {
            let inner = self.inner.lock().unwrap();
            inner.rate_limiter.clone()
        };
        rate_limiter.acquire_one().await;

        let res = req.send().await?;

        match res.error_for_status() {
            Ok(res) => {
                let mut json = res.json::<serde_json::Value>().await?;
                let data = json["data"].take();
                if data.is_null() {
                    return Err(ClientError::NoData);
                }
                let items = data["items"]
                    .as_array()
                    .ok_or(ClientError::NoData)?
                    .iter()
                    .map(News::new)
                    .collect();
                Ok(items)
            }
            Err(err) => {
                eprintln!("error: {}", err);
                Err(err.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Client;
    #[tokio::test]
    async fn test_news_by_company_success() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let news = client.company_news("US7433151039").await.unwrap();
        for x in &news {
            println!("{}", serde_json::to_string_pretty(x).unwrap());
        }
    }
}
