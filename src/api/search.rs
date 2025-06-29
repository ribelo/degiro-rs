use bon::Builder;
use chrono::NaiveDate;
use reqwest::{header, Url};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    client::{ApiErrorResponse, ClientError, ClientStatus, Degiro},
    models::{AllowedOrderTypes, Exchange, OrderTimeTypes},
    paths::{PRODUCT_LOOKUP_PATH, PRODUCT_SEARCH_URL},
    util::ProductCategory,
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
        self.ensure_authorized().await?;

        let url = Url::parse(PRODUCT_SEARCH_URL)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join(PRODUCT_LOOKUP_PATH)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

        let req = self
            .http_client
            .get(url)
            .query(&[
                ("intAccount", &self.int_account().to_string()),
                ("sessionId", &self.session_id()),
                ("searchText", &query.query),
                ("limit", &query.limit.to_string()),
                ("offset", &query.offset.to_string()),
            ])
            .header(header::REFERER, crate::paths::REFERER);

        self.acquire_limit().await;

        let res = req.send().await?;

        if let Err(err) = res.error_for_status_ref() {
            dbg!(&err);
            let Some(status) = err.status() else {
                return Err(ClientError::UnexpectedError(err.to_string()));
            };

            if status.as_u16() == 401 {
                self.set_auth_state(ClientStatus::Unauthorized);
                return Err(ClientError::Unauthorized);
            }

            let error_response = res.json::<ApiErrorResponse>().await?;
            dbg!(&error_response);
            return Err(ClientError::ApiError(error_response));
        }

        let mut body = res.json::<Value>().await?;

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
    async fn test_search() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let query = Query::builder().query("microsoft").limit(10).build();
        let products = client.search(query).await.unwrap();
        dbg!(products.unwrap().first().unwrap());
    }
}
