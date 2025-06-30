use bon::Builder;
use chrono::{DateTime, NaiveDateTime, Utc};
use derive_more::derive::From;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::{
    client::Degiro,
    error::{ClientError, DataError, ResponseError},
    http::{HttpClient, HttpRequest},
    models::{Order, OrderTimeType, OrderType, Orders, Position, TransactionType},
    paths::{UPDATE_DATA_PATH, TRADING_URL},
};

impl Degiro {
    pub async fn get_order(&self, id: impl AsRef<str>) -> Result<Option<Order>, ClientError> {
        let id = id.as_ref();
        let orders = self.orders().await?;
        Ok(orders.iter().find(|o| o.id == id).cloned())
    }

    pub async fn orders(&self) -> Result<Orders, ClientError> {
        let url = self.build_trading_url(UPDATE_DATA_PATH)?;

        let json = self.request_json(
            HttpRequest::get(url)
                .query("orders", "0")
        ).await?;

        let orders = json
            .get("orders")
            .and_then(|o| o.get("value").cloned())
            .ok_or_else(|| DataError::missing_field("orders.value").into())
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
        let session_id = self.session_id();
        let int_account = self.int_account();

        let url = format!("{TRADING_URL}/v5/checkOrder;jsessionid={session_id}");

        let json = self.request_json(
            HttpRequest::post(url)
                .query("intAccount", int_account.to_string())
                .query("sessionId", &session_id)
                .json(&order)?
        ).await?;
        let order_id = json["data"]["confirmationId"]
            .as_str()
            .ok_or_else(|| DataError::missing_field("data.confirmationId"))?;
        self.confirm_order(order_id, order).await
    }

    pub async fn confirm_order(
        &self,
        order_id: &str,
        order: impl Serialize,
    ) -> Result<serde_json::Value, ClientError> {
        let session_id = self.session_id();
        let int_account = self.int_account();

        let url = format!(
            "{TRADING_URL}v5/order/{order_id};jsessionid={session_id}"
        );

        self.request_json(
            HttpRequest::post(url)
                .query("intAccount", int_account.to_string())
                .query("sessionId", &session_id)
                .json(&order)?
        ).await
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
        let session_id = self.session_id();
        let int_account = self.int_account();

        let url = format!(
            "{}v5/order/{};jsessionid={}",
            TRADING_URL, request.id, session_id
        );

        self.request_json(
            HttpRequest::put(url)
                .query("intAccount", int_account.to_string())
                .query("sessionId", &session_id)
                .json(&request)?
        ).await
    }
}

impl Degiro {
    pub async fn delete_order(
        &self,
        request: DeleteOrderRequest,
    ) -> Result<serde_json::Value, ClientError> {
        let session_id = self.session_id();
        let int_account = self.int_account();

        let url = format!(
            "{}v5/order/{};jsessionid={}",
            TRADING_URL, request.id, session_id
        );

        self.request_json(
            HttpRequest::delete(url)
                .query("intAccount", int_account.to_string())
                .query("sessionId", &session_id)
        ).await
    }
}

impl TryFrom<serde_json::Value> for Order {
    type Error = ClientError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let xs = value
            .get("value")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ResponseError::unexpected_structure("Order value must be an array"))?;

        // Create a lookup map for faster key access
        let value_map: std::collections::HashMap<_, _> = xs
            .iter()
            .filter_map(|v| Some((v["name"].as_str()?, v["value"].clone())))
            .collect();

        let get_value = |k: &str| -> Result<serde_json::Value, ClientError> {
            value_map
                .get(k)
                .cloned()
                .ok_or_else(|| ClientError::from(DataError::missing_field(k)))
        };

        let date_value = get_value("date")?;

        let date_str = date_value
            .as_str()
            .ok_or_else(|| DataError::invalid_type("date", "string"))?;

        let date = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S")
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .map_err(|e| ClientError::from(DataError::parse_error("order date", e.to_string())))?
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
            .ok_or_else(|| ClientError::from(ResponseError::unexpected_structure("Expected array of orders")))?
            .into_iter()
            .map(Order::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(Orders)
    }
}

#[cfg(test)]
mod test {
    //! ⚠️  WARNING: This module contains DANGEROUS tests that can create, modify, 
    //! or delete real orders on real trading accounts. Most dangerous tests are 
    //! commented out for safety. Only uncomment them in a controlled test environment
    //! with proper safeguards in place.

    use rust_decimal_macros::dec;

    use crate::{
        client::Degiro,
        models::{OrderTimeType, OrderType},
    };

    use super::*;

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_get_all_orders() {
        let client = Degiro::load_from_env().expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client.account_config().await.expect("Failed to get account configuration");

        let orders = client.orders().await.expect("Failed to get orders");
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

        println!("{}", serde_json::to_string_pretty(&req).expect("Failed to serialize order request to JSON"));
    }
    // DANGEROUS TEST COMMENTED OUT - MODIFIES REAL ORDERS
    // #[tokio::test]
    // #[ignore = "Unsafe: modifies real orders on real account"]
    // async fn test_modify_order() {
    //     let client = Degiro::load_from_env().unwrap();
    //     client.login().await.unwrap();
    //     client.account_config().await.unwrap();
    //     // 55b9c001-be1e-4788-ace3-66876548feb2
    //     let order = client
    //         .get_order("5fa00f68-94c2-4eac-8d8d-dd872756effd")
    //         .await
    //         .unwrap()
    //         .expect("order must exist");
    //     let req = ModifyOrderRequest::from(order).stop_price(Some(dec!(397)));
    //     let json = serde_json::to_string_pretty(&req).unwrap();
    //     println!("{json}");
    //     // dbg!(&req);
    //     let _res = client.modify_order(req).await.unwrap();
    //     // dbg!(&res);
    // }

    // DANGEROUS TEST COMMENTED OUT - DELETES REAL ORDERS
    // #[tokio::test]
    // #[ignore = "Unsafe: deletes real orders on real account"]
    // async fn test_delete_order() {
    //     let client = Degiro::load_from_env().unwrap();
    //     client.login().await.unwrap();
    //     client.account_config().await.unwrap();
    //     // 55b9c001-be1e-4788-ace3-66876548feb2
    //     let order = client
    //         .get_order("5fa00f68-94c2-4eac-8d8d-dd872756effd")
    //         .await
    //         .unwrap()
    //         .unwrap();
    //     let delete_request = DeleteOrderRequest::from(order);
    //     let json = serde_json::to_string_pretty(&delete_request).unwrap();
    //     println!("{json}");
    //     let res = client.delete_order(delete_request).await.unwrap();
    //     dbg!(&res);
    // }
    // DANGEROUS TEST COMMENTED OUT - CREATES REAL ORDERS
    // #[tokio::test]
    // #[ignore = "Unsafe: creates real orders on real account"]
    // async fn test_send_order() {
    //     let client = Degiro::load_from_env().unwrap();
    //     client.login().await.unwrap();
    //     client.account_config().await.unwrap();
    //     let order_request = CreateOrderRequest::builder()
    //         .order_type(OrderType::Market)
    //         .transaction_type(TransactionType::Sell)
    //         .product_id("332087")
    //         .size(dec!(6.0))
    //         .time_type(OrderTimeType::Gtc)
    //         .build();
    //
    //     let resp = client.create_order(order_request).await;
    //     dbg!(resp).ok();
    // }
}
