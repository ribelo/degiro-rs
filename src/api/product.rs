use futures_concurrency::prelude::*;
use reqwest::{header, Url};
use serde::Serialize;
use std::fmt::Debug;

use crate::client::{ApiErrorResponse, ClientError, ClientStatus, Degiro};
use crate::models::{Product, Products};
use crate::paths::{PRODUCT_INFO_PATH, PRODUCT_SEARCH_URL, REFERER};

impl Degiro {
    pub async fn product(
        &self,
        id: impl Into<String> + Send,
    ) -> Result<Option<Product>, ClientError> {
        let id = id.into();
        self.products([id.as_str()])
            .await
            .map(|products| products.0.into_iter().next())
    }

    pub async fn products<T>(&self, ids: T) -> Result<Products, ClientError>
    where
        T: Debug + Serialize + Sized + Send + Sync,
    {
        self.ensure_authorized().await?;

        // Build URL
        let url = Url::parse(PRODUCT_SEARCH_URL)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join(PRODUCT_INFO_PATH)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

        // Build request
        let req = self
            .http_client
            .post(url)
            .query(&[
                ("intAccount", self.int_account().to_string()),
                ("sessionId", self.session_id()),
            ])
            .json(&ids)
            .header(header::REFERER, REFERER);

        // Rate limit and send request
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

        let body = res.json::<serde_json::Value>().await?;

        let mut products = body
            .get("data")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ClientError::UnexpectedError("Missing data key".to_string()))?
            .iter()
            .map(|(_, v)| serde_json::from_value::<Product>(v.clone()).unwrap())
            .collect::<Vec<Product>>();

        let mut futures = Vec::with_capacity(products.len());
        for p in products.iter_mut() {
            futures.push(self.company_profile(&p.isin));
        }
        let company_profiles = futures.try_join().await;
        if let Ok(company_profiles) = company_profiles {
            for (i, cp) in company_profiles.into_iter().enumerate() {
                if let Some(profile) = cp {
                    products[i].company_profile = Some(profile);
                }
            }
        }

        Ok(Products(products))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_products_ids() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let products = client.products(["17461000"]).await.unwrap();
        dbg!(products);
    }
    #[tokio::test]
    async fn product_one_id() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let product = client.product("17461000").await.unwrap();
        dbg!(product);
    }
    // #[tokio::test]
    // async fn product_candles() {
    //     let username = std::env::args().nth(2).expect("no username given");
    //     let password = std::env::args().nth(3).expect("no password given");
    //     let mut builder = ClientBuilder::default();
    //     let client = builder
    //         .username(&username)
    //         .password(&password)
    //         .build()
    //         .unwrap();
    //     let product = client.product_by_symbol("msft").await.unwrap();
    //     let candles = product.candles(&Period::P1Y, &Period::P1M).await.unwrap();
    //     dbg!(candles);
    // }
}
