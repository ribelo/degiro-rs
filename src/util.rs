use std::collections::HashSet;
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;
use strum::{self, Display, EnumString};

#[derive(Clone, Debug, Default, Deserialize, EnumString, PartialEq, Eq, Hash)]
pub enum Period {
    PT1S,
    PT1M,
    PT1H,
    P1D,
    P1W,
    P1M,
    P3M,
    P6M,
    #[default]
    P1Y,
    P3Y,
    P5Y,
    P50Y,
}

impl Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Period {
    pub fn to_ms(&self) -> u64 {
        match &self {
            Self::PT1S => 1000,
            Self::PT1M => 1000 * 60,
            Self::PT1H => 1000 * 60 * 60,
            Self::P1D => 1000 * 60 * 60 * 24,
            Self::P1W => 1000 * 60 * 60 * 24 * 7,
            Self::P1M => 1000 * 60 * 60 * 24 * 30,
            Self::P3M => 1000 * 60 * 60 * 24 * 30 * 3,
            Self::P6M => 1000 * 60 * 60 * 24 * 30 * 6,
            Self::P1Y => 1000 * 60 * 60 * 24 * 365,
            Self::P3Y => 1000 * 60 * 60 * 24 * 365 * 3,
            Self::P5Y => 1000 * 60 * 60 * 24 * 365 * 5,
            Self::P50Y => 1000 * 60 * 60 * 24 * 365 * 50,
        }
    }
    pub fn to_duration(&self) -> chrono::Duration {
        match self {
            Self::PT1S => chrono::Duration::seconds(1),
            Self::PT1M => chrono::Duration::minutes(1),
            Self::PT1H => chrono::Duration::hours(1),
            Self::P1D => chrono::Duration::days(1),
            Self::P1W => chrono::Duration::weeks(1),
            Self::P1M => chrono::Duration::weeks(4), // Approximation
            Self::P3M => chrono::Duration::weeks(4 * 3), // Approximation
            Self::P6M => chrono::Duration::weeks(4 * 6), // Approximation
            Self::P1Y => chrono::Duration::weeks(52), // Approximation
            Self::P3Y => chrono::Duration::weeks(52 * 3), // Approximation
            Self::P5Y => chrono::Duration::weeks(52 * 5), // Approximation
            Self::P50Y => chrono::Duration::weeks(52 * 50), // Approximation
        }
    }
    pub fn div(&self, other: &Period) -> usize {
        match self {
            Self::P1Y => match other {
                Self::P1M => 12,
                Self::P1D => 252,
                _ => unimplemented!(),
            },
            Self::P3Y => match other {
                Self::P1M => 36,
                Self::P1D => 756,
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}

impl std::ops::Add<&Period> for chrono::DateTime<chrono::Utc> {
    type Output = chrono::DateTime<chrono::Utc>;

    fn add(self, rhs: &Period) -> Self::Output {
        match rhs {
            Period::PT1S => self + chrono::Duration::seconds(1),
            Period::PT1M => self + chrono::Duration::minutes(1),
            Period::PT1H => self + chrono::Duration::hours(1),
            Period::P1D => self + chrono::Duration::days(1),
            Period::P1W => self + chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(self, 1),
            Period::P3M => chronoutil::delta::shift_months(self, 3),
            Period::P6M => chronoutil::delta::shift_months(self, 6),
            Period::P1Y => chronoutil::delta::shift_years(self, 1),
            Period::P3Y => chronoutil::delta::shift_years(self, 3),
            Period::P5Y => chronoutil::delta::shift_years(self, 5),
            Period::P50Y => chronoutil::delta::shift_years(self, 50),
        }
    }
}

impl std::ops::Add<Period> for chrono::DateTime<chrono::Utc> {
    type Output = chrono::DateTime<chrono::Utc>;

    fn add(self, rhs: Period) -> Self::Output {
        self + &rhs
    }
}

#[derive(
    Debug, Default, Deserialize, PartialEq, Eq, Hash, EnumString, Clone, Copy, Serialize_repr,
)]
#[strum(ascii_case_insensitive)]
#[repr(u8)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    #[default]
    Limit = 0,
    StopLimit = 1,
    Market = 2,
    StopLoss = 3,
    TrailingStop = 4,
    StandardAmount,
    StandardSize,
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize, Clone, Serialize)]
pub struct AllowedOrderTypes(HashSet<OrderType>);

impl AllowedOrderTypes {
    pub fn has(&self, x: OrderType) -> bool {
        self.0.contains(&x)
    }
}

#[derive(Clone, Debug, Deserialize, EnumString, Display, PartialEq, PartialOrd, Serialize)]
#[strum(ascii_case_insensitive)]
pub enum ProductCategory {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
}

#[derive(
    Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Hash, EnumString, Serialize_repr,
)]
#[strum(ascii_case_insensitive)]
#[repr(u8)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderTimeType {
    #[default]
    #[serde(rename(deserialize = "DAY"))]
    Day = 1,
    #[serde(rename(deserialize = "GTC"))]
    Gtc = 3,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderTimeTypes(HashSet<OrderTimeType>);

impl OrderTimeTypes {
    pub fn has(&self, x: OrderTimeType) -> bool {
        self.0.contains(&x)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProductType {
    Stock,
}

#[derive(Debug, Default, Deserialize, Clone, Copy, Serialize, PartialEq, EnumString)]
pub enum TransactionType {
    #[default]
    #[serde(rename(deserialize = "B", serialize = "BUY"))]
    Buy,
    #[serde(rename(deserialize = "S", serialize = "SELL"))]
    Sell,
}
