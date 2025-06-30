use std::{collections::HashSet, fmt};

use chrono::{DateTime, Utc};
use derivative::Derivative;
use derive_more::derive::{Deref, DerefMut};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::models::Currency;

use super::TransactionType;

#[derive(Derivative, Clone, Deserialize)]
#[derivative(Debug, Default)]
pub struct Order {
    pub id: String,
    pub date: DateTime<Utc>,
    pub product_id: u64,
    pub product: String,
    pub contract_type: u64,
    pub contract_size: Decimal,
    pub currency: Currency,
    pub transaction_type: TransactionType,
    pub size: Decimal,
    pub quantity: Decimal,
    pub price: Decimal,
    pub stop_price: Decimal,
    pub total_order_value: Decimal,
    pub order_type: OrderType,
    pub order_type_id: u8,
    pub order_time_type: OrderTimeType,
    pub order_time_type_id: u8,
    pub is_modifiable: bool,
    pub is_deletable: bool,
}

#[derive(Debug, Clone, Deref, DerefMut)]
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
    pub fn get_order(&self, id: impl AsRef<str>) -> Option<&Order> {
        self.iter().find(|o| o.id == id.as_ref())
    }

    pub fn filter_product_id(&self, product_id: u64) -> Orders {
        self.iter()
            .filter(|o| o.product_id == product_id)
            .cloned()
            .collect()
    }

    pub fn filter_order_type(&self, order_type: OrderType) -> Orders {
        self.iter()
            .filter(|o| o.order_type == order_type)
            .cloned()
            .collect()
    }

    pub fn filter_order_time_type(&self, order_time_type: OrderTimeType) -> Orders {
        self.iter()
            .filter(|o| o.order_time_type == order_time_type)
            .cloned()
            .collect()
    }

    pub fn filter_transaction_type(&self, transaction_type: TransactionType) -> Orders {
        self.iter()
            .filter(|o| o.transaction_type == transaction_type)
            .cloned()
            .collect()
    }

    pub fn sort_by_date(&self) -> Orders {
        let mut orders = self.0.clone();
        orders.sort_by(|a, b| b.date.cmp(&a.date));
        Orders(orders)
    }

    pub fn total_value(&self) -> Decimal {
        self.iter().map(|o| o.total_order_value).sum()
    }

    pub fn get_unique_products(&self) -> HashSet<u64> {
        self.iter().map(|o| o.product_id).collect()
    }

    pub fn filter_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Orders {
        self.iter()
            .filter(|o| o.date >= start && o.date <= end)
            .cloned()
            .collect()
    }
}

#[derive(
    Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Hash, EnumString, Serialize, Display,
)]
#[strum(ascii_case_insensitive)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderTimeType {
    #[default]
    #[serde(rename(deserialize = "DAY"))]
    Day,
    #[serde(rename(deserialize = "GTC"))]
    Gtc,
}

impl From<OrderTimeType> for u8 {
    fn from(value: OrderTimeType) -> Self {
        match value {
            OrderTimeType::Day => 1,
            OrderTimeType::Gtc => 3,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct OrderTimeTypes(HashSet<OrderTimeType>);

impl fmt::Display for OrderTimeTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in &self.0 {
            write!(f, "{x}, ")?;
        }
        Ok(())
    }
}

impl OrderTimeTypes {
    pub fn has(&self, x: OrderTimeType) -> bool {
        self.0.contains(&x)
    }
}

#[derive(
    Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Clone, Copy, Display,
)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    #[default]
    Limit,
    StopLimit,
    Market,
    StopLoss,
    TrailingStop,
    StandardAmount,
    StandardSize,
}

impl From<OrderType> for u8 {
    fn from(value: OrderType) -> Self {
        match value {
            OrderType::Limit => 0,
            OrderType::StopLimit => 1,
            OrderType::Market => 2,
            OrderType::StopLoss => 3,
            _ => unimplemented!(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllowedOrderTypes(HashSet<OrderType>);

impl fmt::Display for AllowedOrderTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in &self.0 {
            write!(f, "{x}, ")?;
        }
        Ok(())
    }
}

impl AllowedOrderTypes {
    pub fn has(&self, x: OrderType) -> bool {
        self.0.contains(&x)
    }
}
