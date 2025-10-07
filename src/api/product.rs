use serde::Serialize;
use std::fmt::Debug;
use tracing::warn;

use crate::client::Degiro;
use crate::error::{ClientError, DataError};
use crate::http::{HttpClient, HttpRequest};
use crate::models::{Product, Products};
use crate::paths::{PRODUCT_INFO_PATH, PRODUCT_SEARCH_URL};

impl Degiro {
    pub async fn product(
        &self,
        id: impl Into<String> + Send,
    ) -> Result<Option<Product>, ClientError> {
        let id = id.into();
        self.products([id.as_str()])
            .await
            .map(|products| products.into_vec().into_iter().next())
    }

    pub async fn products<T>(&self, ids: T) -> Result<Products, ClientError>
    where
        T: Debug + Serialize + Sized + Send + Sync,
    {
        let url = format!("{PRODUCT_SEARCH_URL}{PRODUCT_INFO_PATH}");

        let body = self
            .request_json(
                HttpRequest::post(url)
                    .query("intAccount", self.int_account().to_string())
                    .query("sessionId", self.session_id())
                    .json(&ids)?,
            )
            .await?;

        let mut products = body
            .get("data")
            .and_then(|v| v.as_object())
            .ok_or_else(|| DataError::missing_field("data"))?
            .iter()
            .map(|(_, v)| serde_json::from_value::<Product>(v.clone()))
            .collect::<Result<Vec<Product>, _>>()?;

        for product in products.iter_mut() {
            match self.company_profile(&product.isin).await {
                Ok(Some(profile)) => {
                    product.company_profile = Some(profile);
                }
                Ok(None) => {}
                Err(err)
                    if matches!(
                        err,
                        ClientError::ResponseError(_) | ClientError::ApiError(_)
                    ) =>
                {
                    warn!(
                        isin = %product.isin,
                        "Skipping company profile – optional data fetch failed: {err}"
                    );
                }
                Err(err) => return Err(err),
            }
        }

        Ok(Products::new(products))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_products_ids() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");
        let products = client
            .products(["17461000"])
            .await
            .expect("Failed to get products");
        dbg!(products);
    }
    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn product_one_id() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");
        let product = client
            .product("17461000")
            .await
            .expect("Failed to get product");
        dbg!(product);
    }
}
