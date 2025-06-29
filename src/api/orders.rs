use bon::Builder;
use chrono::{DateTime, NaiveDateTime, Utc};
use derive_more::derive::From;
use reqwest::{header, Url};
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::Value;

use crate::{
    client::{ApiErrorResponse, ClientError, ClientStatus, Degiro},
    models::{Order, OrderTimeType, OrderType, Orders, Position, TransactionType},
    paths::{REFERER, TRADING_URL, UPDATE_DATA_PATH},
};

impl Degiro {
    pub async fn get_order(&self, id: impl AsRef<str>) -> Result<Option<Order>, ClientError> {
        let id = id.as_ref();
        let orders = self.orders().await?;
        Ok(orders.iter().find(|o| o.id == id).cloned())
    }

    pub async fn orders(&self) -> Result<Orders, ClientError> {
        self.ensure_authorized().await?;

        let url = {
            let session_id = self.session_id();
            let int_account = self.int_account();

            Url::parse(TRADING_URL)
                .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
                .join(UPDATE_DATA_PATH)
                .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
                .join(&format!("{};jsessionid={}", int_account, session_id))
                .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
        };

        let req = self
            .http_client
            .get(url)
            .query(&[("orders", "0")])
            .header(header::REFERER, REFERER);

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
        let orders = json
            .get("orders")
            .and_then(|o| o.get("value").cloned())
            .ok_or_else(|| ClientError::UnexpectedError("Missing required fields".into()))
            .and_then(Orders::try_from)?;
        Ok(orders)
    }
}

#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    #[builder(into)]
    pub product_id: String,
    #[serde(rename = "buySell")]
    pub transaction_type: TransactionType,
    #[builder(into)]
    pub order_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    pub size: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Decimal>,
    #[builder(into)]
    pub time_type: u8,
}

impl From<Position> for CreateOrderRequest {
    fn from(position: Position) -> Self {
        Self {
            product_id: position.id,
            transaction_type: TransactionType::Sell,
            order_type: OrderType::Market.into(),
            price: None,
            size: position.size,
            stop_price: None,
            time_type: OrderTimeType::Gtc.into(),
        }
    }
}

impl Degiro {
    pub async fn create_order(
        &self,
        order: CreateOrderRequest,
    ) -> Result<serde_json::Value, ClientError> {
        self.ensure_authorized().await?;

        let url = {
            let session_id = self.session_id();
            let int_account = self.int_account();

            let mut url =
                Url::parse(TRADING_URL).map_err(|e| ClientError::UnexpectedError(e.to_string()))?;
            url.path_segments_mut()
                .map_err(|_| ClientError::UnexpectedError("Cannot modify URL segments".into()))?
                .push("v5")
                .push(&format!("checkOrder;jsessionid={}", session_id));

            url.query_pairs_mut()
                .append_pair("intAccount", &int_account.to_string())
                .append_pair("sessionId", &session_id);
            url
        };

        let req = self
            .http_client
            .post(url)
            .header(header::REFERER, REFERER)
            .json(&order);

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

        let json = res.json::<Value>().await?;
        let order_id = json["data"]["confirmationId"].as_str().unwrap();
        self.confirm_order(order_id, order).await
    }

    pub async fn confirm_order(
        &self,
        order_id: &str,
        order: impl Serialize,
    ) -> Result<serde_json::Value, ClientError> {
        self.ensure_authorized().await?;

        let url = {
            let session_id = self.session_id();
            let int_account = self.int_account();

            let mut url =
                Url::parse(TRADING_URL).map_err(|e| ClientError::UnexpectedError(e.to_string()))?;
            url.path_segments_mut()
                .map_err(|_| ClientError::UnexpectedError("Cannot modify URL segments".into()))?
                .push("v5")
                .push(&format!("order/{};jsessionid={}", order_id, session_id));

            url.query_pairs_mut()
                .append_pair("intAccount", &int_account.to_string())
                .append_pair("sessionId", &session_id);
            url
        };

        let req = self
            .http_client
            .post(url)
            .header(header::REFERER, REFERER)
            .json(&order);

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

        Ok(res.json().await?)
    }
}

#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct ModifyOrderRequest {
    #[builder(into)]
    pub id: String,
    #[builder(into)]
    pub product_id: String,
    #[serde(rename = "buySell")]
    pub transaction_type: TransactionType,
    pub order_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    pub size: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Decimal>,
    #[builder(into)]
    pub time_type: u8,
}

impl ModifyOrderRequest {
    pub fn stop_price(mut self, stop_price: Option<Decimal>) -> Self {
        self.stop_price = stop_price;
        self
    }
}

#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct DeleteOrderRequest {
    #[builder(into)]
    pub id: String,
}

impl From<Order> for ModifyOrderRequest {
    fn from(value: Order) -> Self {
        ModifyOrderRequest {
            id: value.id,
            product_id: value.product_id.to_string(),
            transaction_type: value.transaction_type,
            order_type: value.order_type_id,
            price: Some(value.price),
            size: value.size,
            stop_price: Some(value.stop_price),
            time_type: value.order_time_type_id,
        }
    }
}

impl From<Order> for DeleteOrderRequest {
    fn from(value: Order) -> Self {
        DeleteOrderRequest { id: value.id }
    }
}

#[derive(Debug, Clone, Serialize, From)]
#[serde(untagged)]
pub enum OrderRequest {
    Create(CreateOrderRequest),
    Modify(ModifyOrderRequest),
    Delete(DeleteOrderRequest),
}

impl OrderRequest {
    pub fn id(&self) -> &str {
        match self {
            OrderRequest::Create(req) => &req.product_id,
            OrderRequest::Modify(req) => &req.id,
            OrderRequest::Delete(req) => &req.id,
        }
    }
    pub fn size(&self) -> Option<Decimal> {
        match self {
            OrderRequest::Create(req) => Some(req.size),
            OrderRequest::Modify(req) => Some(req.size),
            OrderRequest::Delete(_) => None,
        }
    }
}

impl Degiro {
    pub async fn modify_order(
        &self,
        request: ModifyOrderRequest,
    ) -> Result<serde_json::Value, ClientError> {
        self.ensure_authorized().await?;

        let url = {
            let session_id = self.session_id();
            let int_account = self.int_account();

            let mut url =
                Url::parse(TRADING_URL).map_err(|e| ClientError::UnexpectedError(e.to_string()))?;
            url.path_segments_mut()
                .map_err(|_| ClientError::UnexpectedError("Cannot modify URL segments".into()))?
                .push("v5")
                .push("order")
                .push(&format!("{};jsessionid={}", request.id, session_id));

            url.query_pairs_mut()
                .append_pair("intAccount", &int_account.to_string())
                .append_pair("sessionId", &session_id);
            url
        };

        let req = self
            .http_client
            .put(url)
            .header(header::REFERER, REFERER)
            .json(&request);

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
        Ok(json)
    }
}

impl Degiro {
    pub async fn delete_order(
        &self,
        request: DeleteOrderRequest,
    ) -> Result<serde_json::Value, ClientError> {
        self.ensure_authorized().await?;

        let url = {
            let session_id = self.session_id();
            let int_account = self.int_account();

            let mut url =
                Url::parse(TRADING_URL).map_err(|e| ClientError::UnexpectedError(e.to_string()))?;
            url.path_segments_mut()
                .map_err(|_| ClientError::UnexpectedError("Cannot modify URL segments".into()))?
                .push("v5")
                .push("order")
                .push(&format!("{};jsessionid={}", request.id, session_id));

            url.query_pairs_mut()
                .append_pair("intAccount", &int_account.to_string())
                .append_pair("sessionId", &session_id);
            url
        };

        let req = self
            .http_client
            .delete(url)
            .header(header::REFERER, REFERER);

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

        Ok(res.json().await?)
    }
}

impl TryFrom<serde_json::Value> for Order {
    type Error = ClientError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let xs = value
            .get("value")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ClientError::UnexpectedError("Value must be an array".into()))?;

        // Create a lookup map for faster key access
        let value_map: std::collections::HashMap<_, _> = xs
            .iter()
            .filter_map(|v| Some((v["name"].as_str()?, v["value"].clone())))
            .collect();

        let get_value = |k: &str| {
            value_map
                .get(k)
                .cloned()
                .ok_or_else(|| ClientError::UnexpectedError(format!("Cannot find key: {}", k)))
        };

        let date_value = get_value("date")?;

        let date_str = date_value
            .as_str()
            .ok_or_else(|| ClientError::UnexpectedError("Invalid date".into()))?;

        let date = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S")
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .map_err(|e| ClientError::UnexpectedError(format!("Failed to parse date: {}", e)))?
            .with_timezone(&Utc);

        Ok(Order {
            id: serde_json::from_value(get_value("id")?)?,
            date,
            product: serde_json::from_value(get_value("product")?)?,
            product_id: serde_json::from_value(get_value("productId")?)?,
            contract_type: serde_json::from_value(get_value("contractType")?)?,
            contract_size: serde_json::from_value(get_value("contractSize")?)?,
            currency: serde_json::from_value(get_value("currency")?)?,
            transaction_type: serde_json::from_value(get_value("buysell")?)?,
            size: serde_json::from_value(get_value("size")?)?,
            quantity: serde_json::from_value(get_value("quantity")?)?,
            price: serde_json::from_value(get_value("price")?)?,
            stop_price: serde_json::from_value(get_value("stopPrice")?)?,
            total_order_value: serde_json::from_value(get_value("totalOrderValue")?)?,
            order_type: serde_json::from_value(get_value("orderType")?)?,
            order_type_id: serde_json::from_value(get_value("orderTypeId")?)?,
            order_time_type: serde_json::from_value(get_value("orderTimeType")?)?,
            order_time_type_id: serde_json::from_value(get_value("orderTimeTypeId")?)?,
            is_modifiable: serde_json::from_value(get_value("isModifiable")?)?,
            is_deletable: serde_json::from_value(get_value("isDeletable")?)?,
        })
    }
}

impl TryFrom<serde_json::Value> for Orders {
    type Error = ClientError;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        value
            .as_array()
            .cloned()
            .ok_or_else(|| ClientError::UnexpectedError("Expected array of orders".into()))?
            .into_iter()
            .map(Order::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(Orders)
    }
}

#[cfg(test)]
mod test {

    use rust_decimal_macros::dec;

    use crate::{
        client::Degiro,
        models::{OrderTimeType, OrderType},
    };

    use super::*;

    #[tokio::test]
    async fn test_get_all_orders() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();

        let orders = client.orders().await.unwrap();
        dbg!(orders);
    }

    #[tokio::test]
    async fn test_create_order_request() {
        let req = CreateOrderRequest::builder()
            .transaction_type(TransactionType::Buy)
            .order_type(OrderType::Market)
            .product_id("15850348")
            .size(Decimal::ONE)
            .time_type(OrderTimeType::Gtc)
            .stop_price(dec!(221.60))
            .build();

        println!("{}", serde_json::to_string_pretty(&req).unwrap());
    }
    #[tokio::test]
    async fn test_modify_order() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        // 55b9c001-be1e-4788-ace3-66876548feb2
        let order = client
            .get_order("5fa00f68-94c2-4eac-8d8d-dd872756effd")
            .await
            .unwrap()
            .expect("order must exist");
        let req = ModifyOrderRequest::from(order).stop_price(Some(dec!(397)));
        let json = serde_json::to_string_pretty(&req).unwrap();
        println!("{json}");
        // dbg!(&req);
        let res = client.modify_order(req).await.unwrap();
        // dbg!(&res);
    }

    #[tokio::test]
    async fn test_delete_order() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        // 55b9c001-be1e-4788-ace3-66876548feb2
        let order = client
            .get_order("5fa00f68-94c2-4eac-8d8d-dd872756effd")
            .await
            .unwrap()
            .unwrap();
        let delete_request = DeleteOrderRequest::from(order);
        let json = serde_json::to_string_pretty(&delete_request).unwrap();
        println!("{json}");
        let res = client.delete_order(delete_request).await.unwrap();
        dbg!(&res);
    }
    #[tokio::test]
    async fn test_send_order() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let order_request = CreateOrderRequest::builder()
            .order_type(OrderType::Market)
            .transaction_type(TransactionType::Sell)
            .product_id("332087")
            .size(dec!(6.0))
            .time_type(OrderTimeType::Gtc)
            .build();

        let resp = client.create_order(order_request).await;
        dbg!(resp);
    }
}
