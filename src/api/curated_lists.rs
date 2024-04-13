use core::fmt;

use chrono::{DateTime, Utc};
use reqwest::{header, Url};
use serde::{Deserialize, Serialize};

use crate::client::{Client, ClientError, ClientStatus};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CuratedLists {
    pub last_updated: DateTime<Utc>,
    pub most_traded_weekly: Vec<u64>,
    pub most_traded_daily: Vec<u64>,
    pub largest_world_etfs: Vec<u64>,
    pub most_held: Vec<u64>,
}

impl Client {
    pub async fn curated_lists_by_country<T>(&self, country: T) -> Result<CuratedLists, ClientError>
    where
        T: AsRef<str> + fmt::Display,
    {
        if self.inner.lock().unwrap().status != ClientStatus::Authorized {
            return Err(ClientError::Unauthorized);
        }

        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = "https://trader.degiro.nl/curated-lists/api/secure/v1/internal/";
            let url = Url::parse(base_url)
                .unwrap_or_else(|_| panic!("can't parse base_url: {base_url}"))
                .join(country.as_ref())
                .unwrap_or_else(|_| panic!("can't join country: {country}"));

            inner
                .http_client
                .get(url)
                .query(&[
                    ("intAccount", &inner.int_account.to_string()),
                    ("sessionId", &inner.session_id),
                ])
                .header(header::REFERER, &inner.referer)
                .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        };

        let rate_limiter = {
            let inner = self.inner.lock().unwrap();
            inner.rate_limiter.clone()
        };

        rate_limiter.acquire_one().await;

        let res = req.send().await?;

        match res.error_for_status() {
            Ok(res) => {
                let json = res.json::<serde_json::Value>().await?;
                let mut list = CuratedLists::default();

                if let Some(first_obj) = json.as_array().and_then(|arr| arr.first()) {
                    if let Some(last_updated_str) = first_obj["lastUpdated"].as_str() {
                        if let Ok(last_updated) = DateTime::parse_from_rfc3339(last_updated_str) {
                            list.last_updated = last_updated.with_timezone(&Utc);
                        }
                    }
                }

                for obj in json.as_array().unwrap_or(&Vec::new()) {
                    if let Some(product_ids) = obj["productIds"].as_array() {
                        let ids: Vec<u64> =
                            product_ids.iter().filter_map(|id| id.as_u64()).collect();

                        match obj["type"].as_str() {
                            Some("MOST_TRADED_DAILY") => list.most_traded_daily = ids,
                            Some("MOST_TRADED_WEEKLY") => list.most_traded_weekly = ids,
                            Some("LARGEST_WORLD_ETFS") => list.largest_world_etfs = ids,
                            Some("MOST_HELD") => list.most_held = ids,
                            _ => panic!("Unknown list type: {obj:#?}"),
                        }
                    }
                }

                Ok(list)
            }
            Err(err) => match err.status().unwrap().as_u16() {
                401 => {
                    self.inner.lock().unwrap().status = ClientStatus::Unauthorized;
                    Err(ClientError::Unauthorized)
                }
                _ => Err(ClientError::UnexpectedError {
                    source: Box::new(err),
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Client;

    #[tokio::test]
    async fn test_curated_lists_by_country_success() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        client.curated_lists_by_country("GB").await.unwrap();
    }
}
