use chrono::NaiveDate;
use derivative::Derivative;
use reqwest::{header, Url};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    client::{Client, ClientError, ClientStatus},
    util::{AllowedOrderTypes, OrderTimeTypes, ProductCategory},
};

use super::product::Product;

#[allow(dead_code)]
#[derive(Debug)]
pub struct QueryBuilder {
    query: String,
    symbol: Option<String>,
    limit: u32,
    offset: u32,
    client: Client,
}

#[derive(Deserialize, Derivative, Clone)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct QueryProductDetails {
    pub active: bool,
    pub buy_order_types: AllowedOrderTypes,
    pub category: ProductCategory,
    pub close_price: Option<f64>,
    pub close_price_date: Option<NaiveDate>,
    pub contract_size: f64,
    pub exchange_id: String,
    pub feed_quality: Option<String>,
    pub feed_quality_secondary: Option<String>,
    pub id: String,
    pub isin: String,
    pub name: String,
    pub only_eod_prices: bool,
    pub order_book_depth: Option<i32>,
    pub order_book_depth_secondary: Option<i32>,
    pub order_time_types: OrderTimeTypes,
    pub product_bit_types: Vec<String>,
    pub product_type: String,
    pub product_type_id: i32,
    pub quality_switch_free: Option<bool>,
    pub quality_switch_free_secondary: Option<bool>,
    pub quality_switchable: Option<bool>,
    pub quality_switchable_secondary: Option<bool>,
    pub sell_order_types: AllowedOrderTypes,
    pub symbol: String,
    pub tradable: bool,
}

#[derive(Clone, Debug)]
pub struct QueryProduct {
    pub inner: QueryProductDetails,
    pub client: Client,
}

impl QueryBuilder {
    pub fn query(mut self, query: &str) -> Self {
        self.query = query.to_uppercase();
        self
    }
    pub fn symbol(mut self, symbol: &str) -> Self {
        self.symbol = Some(symbol.to_uppercase());
        self
    }
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    pub async fn send(&self) -> Result<Vec<QueryProduct>, ClientError> {
        if self.client.inner.lock().unwrap().status != ClientStatus::Authorized {
            return Err(ClientError::Unauthorized);
        }
        let req = {
            let inner = self.client.inner.try_lock().unwrap();
            let base_url = &inner.account_config.product_search_url;
            let url = Url::parse(base_url)
                .unwrap()
                .join("v5/products/lookup")
                .unwrap();

            inner
                .http_client
                .get(url)
                .query(&[
                    ("intAccount", &inner.int_account.to_string()),
                    ("sessionId", &inner.session_id),
                    ("searchText", &self.query),
                    ("limit", &self.limit.to_string()),
                    ("offset", &self.offset.to_string()),
                ])
                .header(header::REFERER, &inner.referer)
        };

        let res = req.send().await.unwrap();
        match res.error_for_status() {
            Ok(res) => {
                let mut body = res.json::<Value>().await.unwrap();
                if let Some(products) = body.get_mut("products") {
                    let products_inner =
                        serde_json::from_value::<Vec<QueryProductDetails>>(products.take())
                            .unwrap();
                    let mut products = Vec::new();
                    for p in products_inner {
                        products.push(QueryProduct {
                            inner: p,
                            client: self.client.clone(),
                        })
                    }
                    if let Some(symbol) = &self.symbol {
                        Ok(products
                            .into_iter()
                            .filter(|p| p.inner.symbol == symbol.to_uppercase())
                            .collect())
                    } else {
                        Ok(products)
                    }
                } else {
                    Err(ClientError::ProductSearchError)
                }
            }
            Err(err) => match err.status().unwrap().as_u16() {
                401 => {
                    self.client.inner.lock().unwrap().status = ClientStatus::Unauthorized;
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
    pub fn search(&self) -> QueryBuilder {
        QueryBuilder {
            query: Default::default(),
            symbol: None,
            limit: 1,
            offset: 0,
            client: self.clone(),
        }
    }
}

impl QueryProduct {
    pub async fn product(&self) -> Result<Product, ClientError> {
        self.client.product(&self.inner.id).await
    }
}

#[cfg(test)]
mod test {
    use crate::client::Client;

    #[tokio::test]
    async fn search() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let mut query = client.search();
        let products = query
            .query("CA8849037095")
            .limit(10)
            .symbol("TRI")
            .send()
            .await
            .unwrap();
        dbg!(products.first().unwrap());
    }
}
