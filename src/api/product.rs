use std::{collections::HashMap, fmt::Debug, sync::Arc};

use chrono::NaiveDate;
use derivative::Derivative;
use reqwest::{header, Url};
use serde::{Deserialize, Serialize};

use crate::{
    client::{Client, ClientError},
    util::{AllowedOrderTypes, OrderTimeTypes, ProductCategory},
};

#[derive(Clone, Debug, Deserialize, Derivative, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductInner {
    pub active: bool,
    pub buy_order_types: AllowedOrderTypes,
    pub category: ProductCategory,
    pub close_price: f64,
    pub close_price_date: NaiveDate,
    pub contract_size: f64,
    pub exchange_id: String,
    pub feed_quality: Option<String>,
    pub feed_quality_secondary: Option<String>,
    pub id: String,
    pub isin: String,
    pub name: String,
    pub only_eod_prices: bool,
    pub order_book_depth: i32,
    pub order_book_depth_secondary: Option<i32>,
    pub order_time_types: OrderTimeTypes,
    pub product_bit_types: Vec<String>,
    pub product_type: String,
    pub product_type_id: i32,
    pub quality_switch_free: bool,
    pub quality_switch_free_secondary: Option<bool>,
    pub quality_switchable: bool,
    pub quality_switchable_secondary: Option<bool>,
    pub sell_order_types: AllowedOrderTypes,
    pub symbol: String,
    pub tradable: bool,
    pub vwd_id: String,
    pub vwd_id_secondary: Option<String>,
    pub vwd_identifier_type: String,
    pub vwd_identifier_type_secondary: Option<String>,
    pub vwd_module_id: i32,
    pub vwd_module_id_secondary: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct Product<'a> {
    pub inner: Arc<ProductInner>,
    pub client: &'a Client,
}

#[derive(Clone, Debug)]
pub struct Products<'a>(pub HashMap<String, Product<'a>>);

impl Client {
    pub async fn products<T>(&self, ids: T) -> Result<Products, ClientError>
    where
        T: Debug + Serialize + Sized + Send + Sync,
    {
        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = &inner.account_config.product_search_url;
            let path_url = "v5/products/info";
            let url = Url::parse(base_url)
                .unwrap_or_else(|_| panic!("can't parse base_url: {base_url}"))
                .join(path_url)
                .unwrap_or_else(|_| panic!("can't join path_url: {path_url}"));

            inner
                .http_client
                .post(url)
                .query(&[
                    ("intAccount", &inner.int_account.to_string()),
                    ("sessionId", &inner.session_id),
                ])
                .json(&ids)
                .header(header::REFERER, &inner.referer)
        };

        let rate_limiter = {
            let inner = self.inner.lock().unwrap();
            inner.rate_limiter.clone()
        };
        rate_limiter.acquire_one().await;

        let res = req.send().await?;

        match res.error_for_status() {
            Ok(res) => {
                let mut body = res
                    .json::<HashMap<String, HashMap<String, ProductInner>>>()
                    .await
                    .map_err(|_| ClientError::ProductParseError)?;
                let m = body.remove("data").unwrap();
                let mut hm = HashMap::new();
                for (k, v) in m.into_iter() {
                    let product = Product {
                        inner: Arc::new(v),
                        client: self,
                    };
                    hm.insert(k, product);
                }
                Ok(Products(hm))
            }
            Err(err) => match err.status().unwrap().as_u16() {
                401 => Err(ClientError::Unauthorized),
                _ => Err(ClientError::UnexpectedError {
                    source: Box::new(err),
                }),
            },
        }
    }
}

impl Client {
    pub async fn product(
        &self,
        id: impl Into<String> + Send + Clone,
    ) -> Result<Product, ClientError> {
        let id: String = id.into();
        match self.products(vec![id.clone()]).await {
            Ok(mut xs) => Ok(xs.0.remove(&id).unwrap()),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn products_ids() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let products = client.products(["17461000"]).await.unwrap();
        dbg!(products);
    }
    #[tokio::test]
    async fn product_one_id() {
        let client = Client::new_from_env();
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
