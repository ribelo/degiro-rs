use std::{
    collections::HashMap,
    fmt::{self, Debug},
};

use chrono::NaiveDate;
use derivative::Derivative;
use reqwest::{header, Url};
use serde::{Deserialize, Serialize};

use crate::{
    client::{Client, ClientError, ClientStatus},
    util::{AllowedOrderTypes, OrderTimeTypes, ProductCategory},
};

#[derive(Clone, Debug, Deserialize, Derivative, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductDetails {
    #[serde(default)]
    pub active: bool,
    pub buy_order_types: Option<AllowedOrderTypes>,
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
    #[serde(default)]
    pub only_eod_prices: bool,
    pub order_book_depth: Option<i32>,
    pub order_book_depth_secondary: Option<i32>,
    pub order_time_types: Option<OrderTimeTypes>,
    pub product_bit_types: Option<Vec<String>>,
    pub product_type: String,
    pub product_type_id: i32,
    #[serde(default)]
    pub quality_switch_free: bool,
    #[serde(default)]
    pub quality_switch_free_secondary: bool,
    #[serde(default)]
    pub quality_switchable: bool,
    #[serde(default)]
    pub quality_switchable_secondary: bool,
    pub sell_order_types: Option<AllowedOrderTypes>,
    pub symbol: String,
    #[serde(default)]
    pub tradable: bool,
    pub vwd_id: Option<String>,
    pub vwd_id_secondary: Option<String>,
    pub vwd_identifier_type: Option<String>,
    pub vwd_identifier_type_secondary: Option<String>,
    pub vwd_module_id: Option<i32>,
    pub vwd_module_id_secondary: Option<i32>,
}

impl fmt::Display for ProductDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Product Details:")?;
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Symbol: {}", self.symbol)?;
        writeln!(f, "ISIN: {}", self.isin)?;
        writeln!(f, "Active: {}", self.active)?;
        writeln!(f, "Category: {}", self.category)?;
        writeln!(f, "Exchange ID: {}", self.exchange_id)?;
        writeln!(f, "Close Price: {}", self.close_price)?;
        writeln!(f, "Close Price Date: {}", self.close_price_date)?;
        writeln!(f, "Contract Size: {}", self.contract_size)?;
        writeln!(
            f,
            "Feed Quality: {}",
            self.feed_quality.as_deref().unwrap_or("N/A")
        )?;
        writeln!(
            f,
            "Feed Quality Secondary: {}",
            self.feed_quality_secondary.as_deref().unwrap_or("N/A")
        )?;
        writeln!(f, "ID: {}", self.id)?;
        writeln!(f, "Only EOD Prices: {}", self.only_eod_prices)?;
        writeln!(
            f,
            "Order Book Depth: {}",
            self.order_book_depth.unwrap_or(-1)
        )?;
        writeln!(
            f,
            "Order Book Depth Secondary: {}",
            self.order_book_depth_secondary
                .map_or("N/A".to_string(), |v| v.to_string())
        )?;
        writeln!(f, "Order Time Types: {:?}", self.order_time_types)?;
        writeln!(f, "Product Bit Types: {:?}", self.product_bit_types)?;
        writeln!(f, "Product Type: {}", self.product_type)?;
        writeln!(f, "Product Type ID: {}", self.product_type_id)?;
        writeln!(f, "Quality Switch Free: {}", self.quality_switch_free)?;
        writeln!(
            f,
            "Quality Switch Free Secondary: {}",
            self.quality_switch_free_secondary
        )?;
        writeln!(f, "Quality Switchable: {}", self.quality_switchable)?;
        writeln!(
            f,
            "Quality Switchable Secondary: {}",
            self.quality_switchable_secondary
        )?;
        writeln!(f, "Sell Order Types: {:?}", self.sell_order_types)?;
        writeln!(f, "Tradable: {}", self.tradable)?;
        writeln!(f, "VWD ID: {}", self.vwd_id.as_deref().unwrap_or("N/A"))?;
        writeln!(
            f,
            "VWD ID Secondary: {}",
            self.vwd_id_secondary.as_deref().unwrap_or("N/A")
        )?;
        writeln!(
            f,
            "VWD Identifier Type: {}",
            self.vwd_identifier_type.as_deref().unwrap_or("N/A")
        )?;
        writeln!(
            f,
            "VWD Identifier Type Secondary: {}",
            self.vwd_identifier_type_secondary
                .as_deref()
                .unwrap_or("N/A")
        )?;
        writeln!(
            f,
            "VWD Module ID: {}",
            self.vwd_module_id
                .map_or("N/A".to_string(), |v| v.to_string())
        )?;
        writeln!(
            f,
            "VWD Module ID Secondary: {}",
            self.vwd_module_id_secondary
                .map_or("N/A".to_string(), |v| v.to_string())
        )?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Product {
    pub inner: ProductDetails,
    pub client: Client,
}

#[derive(Clone, Debug)]
pub struct Products(pub HashMap<String, Product>);

impl Products {
    pub fn iter(&self) -> std::collections::hash_map::Iter<String, Product> {
        self.0.iter()
    }
    pub fn get(&self, id: &str) -> Option<&Product> {
        self.0.get(id)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn remove(&mut self, id: &str) -> Option<Product> {
        self.0.remove(id)
    }
    pub fn insert(&mut self, id: String, product: Product) -> Option<Product> {
        self.0.insert(id, product)
    }
}

impl IntoIterator for Products {
    type Item = (String, Product);
    type IntoIter = std::collections::hash_map::IntoIter<String, Product>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Client {
    pub async fn products<T>(&self, ids: T) -> Result<Products, ClientError>
    where
        T: Debug + Serialize + Sized + Send + Sync,
    {
        if self.inner.lock().unwrap().status != ClientStatus::Authorized {
            return Err(ClientError::Unauthorized);
        }

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
                    .json::<HashMap<String, HashMap<String, ProductDetails>>>()
                    .await
                    .map_err(ClientError::RequestError)?;
                let m = body.remove("data").unwrap();
                let mut hm = HashMap::new();
                for (k, v) in m.into_iter() {
                    let product = Product {
                        inner: v,
                        client: self.clone(),
                    };
                    hm.insert(k, product);
                }
                Ok(Products(hm))
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
