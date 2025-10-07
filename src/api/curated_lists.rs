use chrono::{DateTime, Utc};

use crate::client::Degiro;
use crate::error::{ClientError, ResponseError};
use crate::http::{HttpClient, HttpRequest};
use crate::models::CuratedLists;
use crate::paths::{BASE_API_URL, CURATED_LISTS_PATH};

impl Degiro {
    pub async fn curated_lists_by_country(
        &self,
        country: impl AsRef<str>,
    ) -> Result<Option<CuratedLists>, ClientError> {
        let url = format!("{}{}{}", BASE_API_URL, CURATED_LISTS_PATH, country.as_ref());

        let json = self
            .request_json(
                HttpRequest::get(url)
                    .query("intAccount", self.int_account().to_string())
                    .query("sessionId", self.session_id())
                    .header("Content-Type", "application/json"),
            )
            .await?;
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
                        return Err(ClientError::ResponseError(ResponseError::unknown_value(
                            "list type",
                            list_type,
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
    #[ignore = "Integration test - hits real API"]
    async fn test_curated_lists_by_country() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");
        client
            .curated_lists_by_country("GB")
            .await
            .expect("Failed to get curated lists");
    }
}
