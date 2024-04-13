use chrono::{DateTime, NaiveDateTime, Utc};
use derivative::Derivative;
use reqwest::{header, Url};
use serde::{Deserialize, Serialize};

use crate::{
    client::{Client, ClientError, ClientStatus},
    money::Currency,
    util::{OrderTimeType, OrderType, TransactionType},
};
#[derive(Derivative, Clone, Deserialize)]
#[derivative(Debug, Default)]
pub struct OrderDetails {
    pub id: String,
    pub date: DateTime<Utc>,
    pub product_id: u64,
    pub product: String,
    pub contract_type: u64,
    pub contract_size: f64,
    pub currency: Currency,
    pub transaction_type: TransactionType,
    pub size: f64,
    pub quantity: f64,
    pub price: f64,
    pub stop_price: f64,
    pub total_order_value: f64,
    pub order_type: OrderType,
    pub order_type_id: u64,
    pub order_time_type: OrderTimeType,
    pub order_time_type_id: u64,
    pub is_modifiable: bool,
    pub is_deletable: bool,
}

#[derive(Derivative, Clone)]
#[derivative(Debug, Default)]
pub struct Order {
    pub inner: OrderDetails,
    client: Option<Client>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    product_id: String,
    #[serde(rename = "buySell")]
    transaction_type: TransactionType,
    order_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<f64>,
    size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<f64>,
    time_type: u8,
    #[serde(skip)]
    client: Client,
}

#[derive(Debug, Default)]
pub struct CreateOrderRequestBuilder {
    pub product_id: Option<String>,
    pub transaction_type: Option<TransactionType>,
    pub order_type: Option<OrderType>,
    pub price: Option<f64>,
    pub size: Option<u64>,
    pub stop_price: Option<f64>,
    pub time_type: Option<OrderTimeType>,
    pub client: Option<Client>,
}

#[derive(Debug, thiserror::Error)]
pub enum OrderRequestBuilderError {
    #[error("Order ID is required")]
    IdNotSet,
    #[error("Product ID is required")]
    ProductIdNotSet,
    #[error("Transaction type is required")]
    TransactionTypeNotSet,
    #[error("Order type is required")]
    OrderTypeNotSet,
    #[error("Price is required")]
    PriceNotSet,
    #[error("Size is required")]
    SizeNotSet,
    #[error("TimeType is required")]
    TimeTypeNotSet,
    #[error("Client is required")]
    ClientNotSet,
}

impl CreateOrderRequestBuilder {
    pub fn product_id(mut self, product_id: impl ToString) -> Self {
        self.product_id = Some(product_id.to_string());
        self
    }

    pub fn transaction_type(mut self, transaction_type: TransactionType) -> Self {
        self.transaction_type = Some(transaction_type);
        self
    }

    pub fn order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = Some(order_type);
        self
    }

    pub fn price(mut self, price: f64) -> Self {
        self.price = Some(price);
        self
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn stop_price(mut self, stop_price: f64) -> Self {
        self.stop_price = Some(stop_price);
        self
    }

    pub fn time_type(mut self, time_type: OrderTimeType) -> Self {
        self.time_type = Some(time_type);
        self
    }

    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn build(self) -> Result<CreateOrderRequest, OrderRequestBuilderError> {
        let product_id = self
            .product_id
            .ok_or(OrderRequestBuilderError::ProductIdNotSet)?;
        let transaction_type = self
            .transaction_type
            .ok_or(OrderRequestBuilderError::TransactionTypeNotSet)?;
        let order_type = self
            .order_type
            .ok_or(OrderRequestBuilderError::OrderTypeNotSet)?;
        let size = self.size.ok_or(OrderRequestBuilderError::SizeNotSet)?;
        let time_type = self
            .time_type
            .ok_or(OrderRequestBuilderError::TransactionTypeNotSet)?;
        let client = self.client.ok_or(OrderRequestBuilderError::ClientNotSet)?;

        let order_request = CreateOrderRequest {
            product_id,
            transaction_type,
            order_type: order_type.into(),
            price: self.price,
            size,
            stop_price: self.stop_price,
            time_type: time_type.into(),
            client,
        };

        Ok(order_request)
    }
}

impl CreateOrderRequest {
    pub async fn send(&self) -> Result<serde_json::Value, ClientError> {
        let req = {
            let inner = self.client.inner.lock().unwrap();
            let base_url = &inner.account_config.trading_url;
            // https://trader.degiro.nl/trading/secure/v5/checkOrder;jsessionid=44EA8AC91C97B26F4CB2CD3ECBD37F9D.prod_b_125_2?intAccount=71003134&sessionId=44EA8AC91C97B26F4CB2CD3ECBD37F9D.prod_b_125_2
            let path_url = "v5/checkOrder";
            let url = Url::parse(base_url)
                .unwrap()
                .join(&format!("{};jsessionid={}", path_url, inner.session_id))
                .unwrap();

            inner
                .http_client
                .post(url)
                .query(&[
                    ("intAccount", &inner.int_account.to_string()),
                    ("sessionId", &inner.session_id),
                ])
                .header(header::REFERER, &inner.referer)
                .json(&self)
        };

        let rate_limiter = {
            let inner = self.client.inner.lock().unwrap();
            inner.rate_limiter.clone()
        };
        rate_limiter.acquire_one().await;

        let res = req.send().await?;

        match res.error_for_status() {
            Ok(res) => {
                let json = res.json::<serde_json::Value>().await?;
                Ok(json)
            }
            Err(err) => Err(err.into()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyOrderRequest {
    #[serde(skip)]
    pub id: String,
    pub product_id: String,
    #[serde(rename = "buySell")]
    pub transaction_type: TransactionType,
    pub order_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    pub time_type: u8,
    #[serde(skip)]
    pub client: Client,
}

#[derive(Debug, Default)]
pub struct ModifyOrderRequestBuilder {
    pub id: Option<String>,
    pub product_id: Option<String>,
    pub transaction_type: Option<TransactionType>,
    pub order_type: Option<OrderType>,
    pub price: Option<f64>,
    pub size: Option<u64>,
    pub stop_price: Option<f64>,
    pub time_type: Option<OrderTimeType>,
    pub client: Option<Client>,
}

#[derive(Debug, thiserror::Error)]
pub enum ModifyOrderRequestBuilderError {
    #[error("ID is required")]
    IdNotSet,
    #[error("Price or stop price is required")]
    PriceOrStopPriceNotSet,
    #[error("Client is required")]
    ClientNotSet,
}

impl ModifyOrderRequestBuilder {
    pub fn id<T: AsRef<str>>(mut self, id: T) -> Self {
        self.id = Some(id.as_ref().to_string());
        self
    }

    pub fn product_id(mut self, product_id: impl ToString) -> Self {
        self.product_id = Some(product_id.to_string());
        self
    }

    pub fn transaction_type(mut self, transaction_type: TransactionType) -> Self {
        self.transaction_type = Some(transaction_type);
        self
    }

    pub fn order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = Some(order_type);
        self
    }

    pub fn price(mut self, price: f64) -> Self {
        self.price = Some(price);
        self
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn stop_price(mut self, stop_price: f64) -> Self {
        self.stop_price = Some(stop_price);
        self
    }

    pub fn time_type(mut self, time_type: OrderTimeType) -> Self {
        self.time_type = Some(time_type);
        self
    }

    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn build(self) -> Result<ModifyOrderRequest, OrderRequestBuilderError> {
        let id = self.id.ok_or(OrderRequestBuilderError::IdNotSet)?;
        let product_id = self
            .product_id
            .ok_or(OrderRequestBuilderError::ProductIdNotSet)?;
        let transaction_type = self
            .transaction_type
            .ok_or(OrderRequestBuilderError::TransactionTypeNotSet)?;
        let order_type = self
            .order_type
            .ok_or(OrderRequestBuilderError::OrderTypeNotSet)?;
        let size = self.size.ok_or(OrderRequestBuilderError::SizeNotSet)?;
        let time_type = self
            .time_type
            .ok_or(OrderRequestBuilderError::TransactionTypeNotSet)?;
        let client = self.client.ok_or(OrderRequestBuilderError::ClientNotSet)?;

        let modify_request = ModifyOrderRequest {
            id,
            product_id,
            transaction_type,
            order_type: order_type.into(),
            price: self.price,
            size,
            stop_price: self.stop_price,
            time_type: time_type.into(),
            client,
        };

        Ok(modify_request)
    }
}

impl From<&Order> for ModifyOrderRequestBuilder {
    fn from(value: &Order) -> Self {
        ModifyOrderRequestBuilder {
            id: Some(value.inner.id.clone()),
            product_id: Some(value.inner.product_id.to_string()),
            transaction_type: Some(value.inner.transaction_type),
            order_type: Some(value.inner.order_type),
            price: Some(value.inner.price),
            size: Some(value.inner.size as u64),
            stop_price: Some(value.inner.stop_price),
            time_type: Some(value.inner.order_time_type),
            client: value.client.clone(),
        }
    }
}

impl ModifyOrderRequest {
    pub async fn send(&self) -> Result<serde_json::Value, ClientError> {
        let req = {
            let inner = self.client.inner.lock().unwrap();
            let base_url = &inner.account_config.trading_url;
            // https://trader.degiro.nl/trading/secure/v5/order/6126ef1a-1258-424a-b2d7-7930d44ac56a;jsessionid=1321EBE2CF052F15291645ED1965B54E.prod_b_125_2?intAccount=71003134&sessionId=1321EBE2CF052F15291645ED1965B54E.prod_b_125_2
            let path_url = "v5/order/";
            let url = Url::parse(base_url)
                .unwrap()
                .join(path_url)
                .unwrap()
                .join(&format!(
                    "{};jsessionid={}",
                    self.id.as_str(),
                    inner.session_id
                ))
                .unwrap();
            dbg!(&url);

            inner
                .http_client
                .put(url)
                .query(&[
                    ("intAccount", &inner.int_account.to_string()),
                    ("sessionId", &inner.session_id),
                ])
                .header(header::REFERER, &inner.referer)
                .json(&self)
        };

        let rate_limiter = {
            let inner = self.client.inner.lock().unwrap();
            inner.rate_limiter.clone()
        };
        rate_limiter.acquire_one().await;

        let res = req.send().await?;

        match res.error_for_status() {
            Ok(res) => {
                let json = res.json::<serde_json::Value>().await?;
                Ok(json)
            }
            Err(err) => Err(err.into()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteOrderRequest {
    pub id: String,
    #[serde(skip)]
    pub client: Client,
}

#[derive(Debug, Default)]
pub struct DeleteOrderRequestBuilder {
    pub id: Option<String>,
    pub client: Option<Client>,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteRequestBuilderError {
    #[error("ID is required")]
    IdNotSet,
    #[error("Client is required")]
    ClientNotSet,
}

impl DeleteOrderRequestBuilder {
    pub fn id<T: AsRef<str>>(mut self, id: T) -> Self {
        self.id = Some(id.as_ref().to_string());
        self
    }
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }
    pub fn build(self) -> Result<DeleteOrderRequest, DeleteRequestBuilderError> {
        let id = self.id.ok_or(DeleteRequestBuilderError::IdNotSet)?;
        let client = self.client.ok_or(DeleteRequestBuilderError::ClientNotSet)?;

        Ok(DeleteOrderRequest { id, client })
    }
}

impl From<&Order> for DeleteOrderRequestBuilder {
    fn from(value: &Order) -> Self {
        DeleteOrderRequestBuilder {
            id: Some(value.inner.id.clone()),
            client: value.client.clone(),
        }
    }
}

impl DeleteOrderRequest {
    pub async fn send(&self) -> Result<serde_json::Value, ClientError> {
        let req = {
            let inner = self.client.inner.lock().unwrap();
            let base_url = &inner.account_config.trading_url;
            // https://trader.degiro.nl/trading/secure/v5/order/6126ef1a-1258-424a-b2d7-7930d44ac56a;jsessionid=1321EBE2CF052F15291645ED1965B54E.prod_b_125_2?intAccount=71003134&sessionId=1321EBE2CF052F15291645ED1965B54E.prod_b_125_2
            let path_url = "v5/order/";
            let url = Url::parse(base_url)
                .unwrap()
                .join(path_url)
                .unwrap()
                .join(&format!("{};jsessionid={}", self.id, inner.session_id))
                .unwrap();

            inner
                .http_client
                .delete(url)
                .query(&[
                    ("intAccount", &inner.int_account.to_string()),
                    ("sessionId", &inner.session_id),
                ])
                .header(header::REFERER, &inner.referer)
                .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.to_string())
        };

        let rate_limiter = {
            let inner = self.client.inner.lock().unwrap();
            inner.rate_limiter.clone()
        };
        rate_limiter.acquire_one().await;

        let res = req.send().await?;

        match res.error_for_status() {
            Ok(res) => {
                let json = res.json::<serde_json::Value>().await?;
                Ok(json)
            }
            Err(err) => Err(err.into()),
        }
    }
}

impl Order {
    pub async fn modify(&self) -> ModifyOrderRequestBuilder {
        self.into()
    }

    pub async fn delete(&self) -> DeleteOrderRequestBuilder {
        self.into()
    }
}

fn parse_order_from_value(value: &serde_json::Value) -> Result<Order, ClientError> {
    let xs = value["value"].as_array().ok_or(ClientError::ParseError(
        "Value must be an array".to_string(),
    ))?;

    let find_key = |k: &str| {
        xs.iter()
            .find(|v| v["name"] == k)
            .map(|v| v["value"].clone())
            .ok_or_else(|| ClientError::ParseError(format!("Cannot find key: {}", k)))
    };

    let date_str = find_key("date")?
        .as_str()
        .ok_or(ClientError::ParseError("Invalid date".to_string()))?
        .to_owned();

    let date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S")
        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .map_err(|e| ClientError::ParseError(format!("Failed to parse date: {}", e)))?
        .with_timezone(&Utc);

    let details = OrderDetails {
        id: serde_json::from_value(find_key("id")?)?,
        date,
        product: serde_json::from_value(find_key("product")?)?,
        product_id: serde_json::from_value(find_key("productId")?)?,
        contract_type: serde_json::from_value(find_key("contractType")?)?,
        contract_size: serde_json::from_value(find_key("contractSize")?)?,
        currency: serde_json::from_value(find_key("currency")?)?,
        transaction_type: serde_json::from_value(find_key("buysell")?)?,
        size: serde_json::from_value(find_key("size")?)?,
        quantity: serde_json::from_value(find_key("quantity")?)?,
        price: serde_json::from_value(find_key("price")?)?,
        stop_price: serde_json::from_value(find_key("stopPrice")?)?,
        total_order_value: serde_json::from_value(find_key("totalOrderValue")?)?,
        order_type: serde_json::from_value(find_key("orderType")?)?,
        order_type_id: serde_json::from_value(find_key("orderTypeId")?)?,
        order_time_type: serde_json::from_value(find_key("orderTimeType")?)?,
        order_time_type_id: serde_json::from_value(find_key("orderTimeTypeId")?)?,
        is_modifiable: serde_json::from_value(find_key("isModifiable")?)?,
        is_deletable: serde_json::from_value(find_key("isDeletable")?)?,
    };

    Ok(Order {
        inner: details,
        client: None,
    })
}

fn parse_orders_from_values(value: &[serde_json::Value]) -> Result<Vec<Order>, ClientError> {
    value
        .iter()
        .map(parse_order_from_value)
        .collect::<Result<Vec<_>, _>>()
}

#[derive(Debug, Clone)]
pub struct Orders(pub Vec<Order>);

impl From<Vec<Order>> for Orders {
    fn from(orders: Vec<Order>) -> Self {
        Orders(orders)
    }
}

impl FromIterator<Order> for Orders {
    fn from_iter<T: IntoIterator<Item = Order>>(iter: T) -> Self {
        Orders(iter.into_iter().collect())
    }
}

impl Orders {
    pub fn iter(&self) -> std::slice::Iter<Order> {
        self.0.iter()
    }

    pub fn first(&self) -> Option<&Order> {
        self.0.first()
    }

    pub fn last(&self) -> Option<&Order> {
        self.0.last()
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_order(&self, id: impl AsRef<str>) -> Option<&Order> {
        self.iter().find(|o| o.inner.id == id.as_ref())
    }

    pub fn filter_product_id(&self, product_id: u64) -> Orders {
        self.iter()
            .filter(|o| o.inner.product_id == product_id)
            .cloned()
            .collect()
    }

    pub fn filter_order_type(&self, order_type: OrderType) -> Orders {
        self.iter()
            .filter(|o| o.inner.order_type == order_type)
            .cloned()
            .collect()
    }
}

impl Client {
    pub async fn get_order(&self, id: &str) -> Result<Option<Order>, ClientError> {
        let orders = self.orders().await?;
        if let Some(order) = orders.iter().find(|o| o.inner.id == id) {
            // Create a new Order instance with a reference to `self` (the Client)
            let cloned_order = order.inner.clone();
            Ok(Some(Order {
                inner: cloned_order,
                client: Some(self.clone()),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn orders(&self) -> Result<Orders, ClientError> {
        if self.inner.lock().unwrap().status != ClientStatus::Authorized {
            return Err(ClientError::Unauthorized);
        }

        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = &inner.account_config.trading_url;
            let path_url = "v5/update/";
            let url = Url::parse(base_url)
                .unwrap()
                .join(path_url)
                .unwrap()
                .join(&format!(
                    "{};jsessionid={}",
                    inner.int_account, inner.session_id
                ))
                .unwrap();

            inner
                .http_client
                .get(url)
                .query(&[("orders", "0")])
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
                let json = res.json::<serde_json::Value>().await?;
                let raw_orders = json["orders"]["value"].as_array().unwrap().as_slice();
                let mut orders = parse_orders_from_values(raw_orders)?;

                orders.iter_mut().for_each(|order| {
                    order.client = Some(self.clone());
                });

                Ok(orders.into())
            }
            Err(err) => {
                eprintln!("error: {}", err);
                Err(err.into())
            }
        }
    }
}

impl Client {
    pub fn create_order(&self) -> CreateOrderRequestBuilder {
        CreateOrderRequestBuilder {
            client: Some(self.clone()),
            ..Default::default()
        }
    }
    pub fn delete_order(&self) -> DeleteOrderRequestBuilder {
        DeleteOrderRequestBuilder {
            client: Some(self.clone()),
            ..Default::default()
        }
    }

    pub fn modify_order(&self) -> ModifyOrderRequestBuilder {
        ModifyOrderRequestBuilder {
            client: Some(self.clone()),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {

    use crate::client::Client;

    use super::*;

    #[tokio::test]
    async fn orders() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();

        let orders = client.orders().await.unwrap();
        dbg!(orders);
    }

    #[tokio::test]
    async fn request_builder() {
        let req = CreateOrderRequestBuilder::default()
            .transaction_type(TransactionType::Buy)
            .order_type(OrderType::Market)
            .product_id(15850348)
            .size(1)
            .time_type(OrderTimeType::Gtc)
            .stop_price(221.60)
            .build()
            .unwrap();

        println!("{}", serde_json::to_string_pretty(&req).unwrap());
    }
    #[tokio::test]
    async fn test_modify_order() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        // 55b9c001-be1e-4788-ace3-66876548feb2
        let order = client
            .get_order("55b9c001-be1e-4788-ace3-66876548feb2")
            .await
            .unwrap()
            .unwrap();
        let req = ModifyOrderRequestBuilder::from(&order)
            .stop_price(52.0)
            .build()
            .unwrap();
        let json = serde_json::to_string_pretty(&req).unwrap();
        println!("{json}");
        // dbg!(&req);
        let res = req.send().await.unwrap();
        // dbg!(&res);
    }

    #[tokio::test]
    async fn test_delete_order() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        // 55b9c001-be1e-4788-ace3-66876548feb2
        let order = client
            .get_order("55b9c001-be1e-4788-ace3-66876548feb2")
            .await
            .unwrap()
            .unwrap();
        let req = DeleteOrderRequestBuilder::from(&order).build().unwrap();
        let json = serde_json::to_string_pretty(&req).unwrap();
        println!("{json}");
        let res = req.send().await.unwrap();
        dbg!(&res);
    }
    // #[tokio::test]
    // async fn create_order() {
    //     let client = Client::new_from_env();
    //     client.login().await.unwrap();
    //     client.account_config().await.unwrap();
    //
    //     let order_request = CreateOrderRequestBuilder::default()
    //         .order_type(OrderType::Market)
    //         .transaction_type(TransactionType::Sell)
    //         .product_id(15850348)
    //         .size(4)
    //         .stop_price(200.0)
    //         .time_type(OrderTimeType::Gtc)
    //         .build()
    //         .unwrap();
    //
    //     let resp = client.create_order(order_request).await;
    //     dbg!(resp);
    // }
}
