use chrono::{DateTime, NaiveDateTime, Utc};
use derivative::Derivative;
use reqwest::{header, Url};
use serde::Serialize;

use crate::{
    client::{Client, ClientError},
    money::Currency,
    util::{OrderTimeType, OrderType, TransactionType},
};

#[derive(Derivative, Clone)]
#[derivative(Debug, Default)]
pub struct Order<'a> {
    id: String,
    date: DateTime<Utc>,
    product_id: u64,
    product: String,
    contract_type: u64,
    contract_size: f64,
    currency: Currency,
    transaction_type: TransactionType,
    size: f64,
    quantity: f64,
    price: f64,
    stop_price: f64,
    total_order_value: f64,
    order_type: OrderType,
    order_type_id: u64,
    order_time_type: OrderTimeType,
    order_time_type_id: u64,
    is_modifiable: bool,
    is_deletable: bool,

    #[derivative(Debug = "ignore")]
    client: Option<&'a Client>,
}

#[allow(dead_code)]
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    product_id: String,
    #[serde(rename = "buySell")]
    transaction_type: TransactionType,
    order_type: OrderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<f64>,
    size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<f64>,
    time_type: OrderTimeType,
}

#[derive(Debug, Default)]
pub struct OrderRequestBuilder<'a> {
    product_id: Option<String>,
    transaction_type: Option<TransactionType>,
    order_type: Option<OrderType>,
    price: Option<f64>,
    size: Option<u64>,
    stop_price: Option<f64>,
    time_type: Option<OrderTimeType>,
    client: Option<&'a Client>,
}

#[derive(Debug, thiserror::Error)]
#[error("Order request builder error: {0}")]
pub struct OrderRequestBuilderError(&'static str);

impl<'a> OrderRequestBuilder<'a> {
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

    pub fn build(self) -> Result<OrderRequest, OrderRequestBuilderError> {
        let product_id = self
            .product_id
            .ok_or(OrderRequestBuilderError("Product ID is required"))?;
        let transaction_type = self
            .transaction_type
            .ok_or(OrderRequestBuilderError("Transaction type is required"))?;
        let order_type = self
            .order_type
            .ok_or(OrderRequestBuilderError("Order type is required"))?;
        let size = self
            .size
            .ok_or(OrderRequestBuilderError("Size is required"))?;
        let time_type = self
            .time_type
            .ok_or(OrderRequestBuilderError("Order time type is required"))?;

        let order_request = OrderRequest {
            product_id,
            transaction_type,
            order_type,
            price: self.price,
            size,
            stop_price: self.stop_price,
            time_type,
        };

        Ok(order_request)
    }
}

impl<'a> From<Order<'a>> for OrderRequest {
    fn from(order: Order) -> Self {
        Self {
            product_id: order.product_id.to_string(),
            transaction_type: order.transaction_type,
            order_type: order.order_type,
            price: Some(order.price),
            size: order.quantity as u64,
            stop_price: Some(order.stop_price),
            time_type: order.order_time_type,
        }
    }
}

impl<'a> From<&'a Order<'a>> for OrderRequest {
    fn from(order: &'a Order) -> Self {
        Self {
            product_id: order.product_id.to_string(),
            transaction_type: order.transaction_type,
            order_type: order.order_type,
            price: Some(order.price),
            size: order.quantity as u64,
            stop_price: Some(order.stop_price),
            time_type: order.order_time_type,
        }
    }
}

impl<'a> Order<'a> {
    pub async fn modify(
        &self,
        price: Option<f64>,
        stop_price: Option<f64>,
    ) -> Result<(), ClientError> {
        self.client
            .as_ref()
            .unwrap()
            .modify_order(&self.id, price, stop_price)
            .await
    }

    pub async fn delete(&self) -> Result<(), ClientError> {
        self.client.as_ref().unwrap().delete_order(&self.id).await
    }
}

fn parse_order_from_value<'a>(value: &serde_json::Value) -> Result<Order<'a>, ClientError> {
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

    Ok(Order {
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
        client: None,
    })
}

fn parse_orders_from_values<'a>(
    value: &[serde_json::Value],
) -> Result<Vec<Order<'a>>, ClientError> {
    value
        .iter()
        .map(parse_order_from_value)
        .collect::<Result<Vec<_>, _>>()
}

#[derive(Debug)]
pub struct Orders<'a>(Vec<Order<'a>>);

impl<'a> From<Vec<Order<'a>>> for Orders<'a> {
    fn from(orders: Vec<Order<'a>>) -> Self {
        Orders(orders)
    }
}

impl<'a> From<Vec<&'a Order<'a>>> for Orders<'a> {
    fn from(orders: Vec<&'a Order<'a>>) -> Self {
        Orders(orders.into_iter().cloned().collect::<Vec<Order<'a>>>())
    }
}

pub struct OrdersIterator<'a> {
    orders: &'a Orders<'a>,
    index: usize,
}

impl<'a> Iterator for OrdersIterator<'a> {
    type Item = &'a Order<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.orders.0.len() {
            let order = &self.orders.0[self.index];
            self.index += 1;
            return Some(order);
        }

        None
    }
}

impl<'a> From<&'a Orders<'a>> for OrdersIterator<'a> {
    fn from(orders: &'a Orders) -> Self {
        OrdersIterator { orders, index: 0 }
    }
}

impl<'a> FromIterator<Order<'a>> for Orders<'a> {
    fn from_iter<I: IntoIterator<Item = Order<'a>>>(iter: I) -> Self {
        Orders(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a Order<'a>> for Orders<'a> {
    fn from_iter<I: IntoIterator<Item = &'a Order<'a>>>(iter: I) -> Self {
        Orders(iter.into_iter().cloned().collect())
    }
}

impl<'a> Orders<'a> {
    pub fn iter(&'a self) -> OrdersIterator<'a> {
        OrdersIterator::from(self)
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
        self.iter().find(|o| o.id == id.as_ref())
    }

    pub fn filter_product_id(&self, product_id: u64) -> Orders {
        self.iter()
            .filter(|order| order.product_id == product_id)
            .collect()
    }

    pub fn filter_order_type(&self, order_type: OrderType) -> Orders {
        self.iter().filter(|o| o.order_type == order_type).collect()
    }
}

impl Client {
    pub async fn get_order<'a>(&'a self, id: &str) -> Result<Option<Order<'a>>, ClientError> {
        let orders = self.orders().await?;
        if let Some(order) = orders.iter().find(|o| o.id == id) {
            // Create a new Order instance with a reference to `self` (the Client)
            let cloned_order = Order {
                id: order.id.clone(),
                date: order.date,
                product_id: order.product_id,
                product: order.product.clone(),
                contract_type: order.contract_type,
                contract_size: order.contract_size,
                currency: order.currency,
                transaction_type: order.transaction_type,
                size: order.size,
                quantity: order.quantity,
                price: order.price,
                stop_price: order.stop_price,
                total_order_value: order.total_order_value,
                order_type: order.order_type,
                order_type_id: order.order_type_id,
                order_time_type: order.order_time_type,
                order_time_type_id: order.order_time_type_id,
                is_modifiable: order.is_modifiable,
                is_deletable: order.is_deletable,
                client: Some(self),
            };
            Ok(Some(cloned_order))
        } else {
            Ok(None)
        }
    }

    pub async fn orders(&self) -> Result<Orders, ClientError> {
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

            println!("url: {}", url);
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
                    order.client = Some(self);
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
    pub async fn create_order(&self, order_request: OrderRequest) -> Result<(), ClientError> {
        let req = {
            let inner = self.inner.lock().unwrap();
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
                .json(&order_request)
        };

        let rate_limiter = {
            let inner = self.inner.lock().unwrap();
            inner.rate_limiter.clone()
        };
        rate_limiter.acquire_one().await;

        let res = req.send().await?;

        match res.error_for_status() {
            Ok(res) => {
                dbg!(res);
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
    pub async fn delete_order(&self, id: impl Into<String>) -> Result<(), ClientError> {
        let id = id.into();
        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = &inner.account_config.trading_url;
            // https://trader.degiro.nl/trading/secure/v5/order/6126ef1a-1258-424a-b2d7-7930d44ac56a;jsessionid=1321EBE2CF052F15291645ED1965B54E.prod_b_125_2?intAccount=71003134&sessionId=1321EBE2CF052F15291645ED1965B54E.prod_b_125_2
            let path_url = "v5/order/";
            let url = Url::parse(base_url)
                .unwrap()
                .join(path_url)
                .unwrap()
                .join(&format!("{};jsessionid={}", id, inner.session_id))
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
            let inner = self.inner.lock().unwrap();
            inner.rate_limiter.clone()
        };
        rate_limiter.acquire_one().await;

        let res = req.send().await?;

        match res.error_for_status() {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn modify_order(
        &self,
        id: impl Into<String>,
        price: Option<f64>,
        stop_price: Option<f64>,
    ) -> Result<(), ClientError> {
        let id = id.into();

        let order = self.get_order(&id).await?.unwrap();

        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = &inner.account_config.trading_url;
            // https://trader.degiro.nl/trading/secure/v5/order/6126ef1a-1258-424a-b2d7-7930d44ac56a;jsessionid=1321EBE2CF052F15291645ED1965B54E.prod_b_125_2?intAccount=71003134&sessionId=1321EBE2CF052F15291645ED1965B54E.prod_b_125_2
            let path_url = "v5/order/";
            let url = Url::parse(base_url)
                .unwrap()
                .join(path_url)
                .unwrap()
                .join(&format!("{};jsessionid={}", id.as_str(), inner.session_id))
                .unwrap();

            let mut payload: OrderRequest = order.into();
            if let Some(price) = price {
                payload.price = Some(price);
            }
            if let Some(stop_price) = stop_price {
                payload.stop_price = Some(stop_price);
            }

            inner
                .http_client
                .put(url)
                .query(&[
                    ("intAccount", &inner.int_account.to_string()),
                    ("sessionId", &inner.session_id),
                ])
                .header(header::REFERER, &inner.referer)
                .json(&payload)
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
                println!("json: {:?}", json);
                Ok(())
            }
            Err(err) => {
                eprintln!("error: {}", err);
                Err(err.into())
            }
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
        let req = OrderRequestBuilder::default()
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
    async fn modify_order() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();

        let resp = client
            .modify_order("6126ef1a-1258-424a-b2d7-7930d44ac56a", Some(65.00), None)
            .await;
        dbg!(resp);
    }
    #[tokio::test]
    async fn create_order() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();

        let order_request = OrderRequestBuilder::default()
            .order_type(OrderType::Market)
            .transaction_type(TransactionType::Sell)
            .product_id(331860)
            .size(4)
            .stop_price(200.0)
            .time_type(OrderTimeType::Gtc)
            .build()
            .unwrap();

        let resp = client.create_order(order_request).await;
        dbg!(resp);
    }
}
