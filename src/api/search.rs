use bon::Builder;
use chrono::NaiveDate;
use serde::Deserialize;

use crate::{
    client::Degiro,
    error::ClientError,
    http::{HttpClient, HttpRequest},
    models::{AllowedOrderTypes, Exchange, OrderTimeTypes, ProductCategory},
    paths::{PRODUCT_LOOKUP_PATH, PRODUCT_SEARCH_URL},
};

#[derive(Debug, Builder)]
pub struct Query {
    #[builder(into)]
    query: String,
    #[builder(into)]
    symbol: Option<String>,
    #[builder(default = 1)]
    limit: u32,
    #[builder(default)]
    offset: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryProduct {
    pub active: bool,
    pub buy_order_types: AllowedOrderTypes,
    pub category: ProductCategory,
    pub close_price: Option<f64>,
    pub close_price_date: Option<NaiveDate>,
    pub contract_size: f64,
    #[serde(rename = "exchangeId")]
    pub exchange: Exchange,
    pub feed_quality: Option<String>,
    pub feed_quality_secondary: Option<String>,
    pub id: String,
    pub isin: String,
    pub name: String,
    pub only_eod_prices: bool,
    pub order_book_depth: Option<i32>,
    pub order_book_depth_secondary: Option<i32>,
    pub order_time_types: OrderTimeTypes,
    pub product_bit_types: Option<Vec<String>>,
    pub product_type: String,
    pub product_type_id: i32,
    pub quality_switch_free: Option<bool>,
    pub quality_switch_free_secondary: Option<bool>,
    pub quality_switchable: Option<bool>,
    pub quality_switchable_secondary: Option<bool>,
    pub sell_order_types: AllowedOrderTypes,
    pub symbol: Option<String>,
    pub tradable: bool,
}

impl Degiro {
    pub async fn search(&self, query: Query) -> Result<Option<Vec<QueryProduct>>, ClientError> {
        let url = format!("{PRODUCT_SEARCH_URL}{PRODUCT_LOOKUP_PATH}");

        let mut body = self
            .request_json(
                HttpRequest::get(url)
                    .query("intAccount", self.int_account().to_string())
                    .query("sessionId", self.session_id())
                    .query("searchText", &query.query)
                    .query("limit", query.limit.to_string())
                    .query("offset", query.offset.to_string()),
            )
            .await?;

        if let Some(products) = body.get_mut("products") {
            let products: Vec<QueryProduct> = serde_json::from_value(products.take())?;

            if let Some(symbol) = &query.symbol {
                Ok(Some(
                    products
                        .into_iter()
                        .filter(|p| {
                            p.symbol
                                .as_ref()
                                .map(|s| s == &symbol.to_uppercase())
                                .unwrap_or(false)
                        })
                        .collect(),
                ))
            } else {
                Ok(Some(products))
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{api::search::Query, client::Degiro};

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_search() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");
        let query = Query::builder().query("microsoft").limit(10).build();
        let products = client
            .search(query)
            .await
            .expect("Failed to search for products");
        dbg!(products
            .expect("Search returned None")
            .first()
            .expect("No products found in search results"));
    }
}
