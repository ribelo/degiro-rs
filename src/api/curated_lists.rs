use chrono::{DateTime, Utc};
use reqwest::{header, Url};

use crate::models::CuratedLists;
use crate::paths::{BASE_API_URL, CURATED_LISTS_PATH, REFERER};

use crate::client::{ApiErrorResponse, ClientError, ClientStatus, Degiro};

impl Degiro {
    pub async fn curated_lists_by_country(
        &self,
        country: impl AsRef<str>,
    ) -> Result<Option<CuratedLists>, ClientError> {
        self.ensure_authorized().await?;

        let url = Url::parse(BASE_API_URL)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join(CURATED_LISTS_PATH)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join(country.as_ref())
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

        let req = self
            .http_client
            .get(url)
            .query(&[
                ("intAccount", &self.int_account().to_string()),
                ("sessionId", &self.session_id()),
            ])
            .header(header::REFERER, REFERER)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref());

        self.acquire_limit().await;

        let res = req.send().await?;

        if let Err(err) = res.error_for_status_ref() {
            let Some(status) = err.status() else {
                return Err(ClientError::UnexpectedError(err.to_string()));
            };

            if status.as_u16() == 401 {
                self.set_auth_state(ClientStatus::Unauthorized);
                return Err(ClientError::Unauthorized);
            }

            let error_response = res.json::<ApiErrorResponse>().await?;
            return Err(ClientError::ApiError(error_response));
        }

        let json = res.json::<serde_json::Value>().await?;
        dbg!(&json);
        let mut list = CuratedLists::default();

        let arr = match json.as_array() {
            Some(a) => a,
            None => return Ok(None),
        };

        if let Some(first_obj) = arr.first() {
            if let Some(last_updated_str) = first_obj.get("lastUpdated").and_then(|v| v.as_str()) {
                if let Ok(last_updated) = DateTime::parse_from_rfc3339(last_updated_str) {
                    list.last_updated = last_updated.with_timezone(&Utc);
                }
            }
        }

        for obj in arr {
            let product_ids = match obj.get("productIds").and_then(|v| v.as_array()) {
                Some(arr) => arr,
                None => continue,
            };

            let ids: Vec<u64> = product_ids.iter().filter_map(|id| id.as_u64()).collect();

            if let Some(list_type) = obj.get("type").and_then(|t| t.as_str()) {
                match list_type {
                    "MOST_TRADED_DAILY" => list.most_traded_daily = ids,
                    "MOST_TRADED_WEEKLY" => list.most_traded_weekly = ids,
                    "LARGEST_WORLD_ETFS" => list.largest_world_etfs = ids,
                    "MOST_HELD" => list.most_held = ids,
                    _ => {
                        return Err(ClientError::UnexpectedError(format!(
                            "Unknown list type: {}",
                            list_type
                        )))
                    }
                }
            }
        }

        Ok(Some(list))
    }
}

#[cfg(test)]
mod tests {
    use crate::client::Degiro;

    #[tokio::test]
    async fn test_curated_lists_by_country() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        client.curated_lists_by_country("GB").await.unwrap();
    }
}
