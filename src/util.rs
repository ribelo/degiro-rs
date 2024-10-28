use std::fmt::Display;
use std::str::FromStr;
use std::{collections::HashSet, fmt};

use serde::{Deserialize, Serialize};
use strum::{self, Display, EnumString};

#[derive(
    Clone, Copy, Debug, Default, Serialize, Deserialize, EnumString, PartialEq, Eq, Hash, Display,
)]
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
    pub fn div(&self, other: Period) -> usize {
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

impl std::ops::Add<Period> for chrono::DateTime<chrono::Utc> {
    type Output = chrono::DateTime<chrono::Utc>;

    fn add(self, rhs: Period) -> Self::Output {
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

impl std::ops::Add<Period> for chrono::NaiveDate {
    type Output = chrono::NaiveDate;

    fn add(self, rhs: Period) -> Self::Output {
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
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AllowedOrderTypes(HashSet<OrderType>);

impl fmt::Display for AllowedOrderTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in &self.0 {
            write!(f, "{}, ", x)?;
        }
        Ok(())
    }
}

impl AllowedOrderTypes {
    pub fn has(&self, x: OrderType) -> bool {
        self.0.contains(&x)
    }
}

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Display, PartialEq, PartialOrd, Serialize,
)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderTimeTypes(HashSet<OrderTimeType>);

impl fmt::Display for OrderTimeTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in &self.0 {
            write!(f, "{}, ", x)?;
        }
        Ok(())
    }
}

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

#[derive(
    Debug, Default, Deserialize, Clone, Copy, Serialize, PartialEq, EnumString, strum::Display,
)]
pub enum TransactionType {
    #[default]
    #[serde(rename(deserialize = "B", serialize = "BUY"))]
    Buy,
    #[serde(rename(deserialize = "S", serialize = "SELL"))]
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Exchange {
    NSDQ,
    NSY,
    EAM,
    XET,
    TDG,
    EPA,
    WSE,
    TSE,
    OSL,
    SWX,
    OMX,
    ATH,
    ASE,
    TSV,
    ASX,
    LSE,
    TOR,
    HKS,
    Unknown(i32),
}

impl FromStr for Exchange {
    type Err = strum::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let x = s.parse::<i32>().unwrap();
        Ok(x.into())
    }
}

impl From<i32> for Exchange {
    fn from(x: i32) -> Self {
        match x {
            663 => Self::NSDQ,
            676 => Self::NSY,
            200 => Self::EAM,
            194 => Self::XET,
            196 => Self::TDG,
            710 => Self::EPA,
            801 => Self::WSE,
            5001 => Self::TSE,
            520 => Self::OSL,
            947 => Self::SWX,
            860 => Self::OMX,
            219 => Self::ATH,
            650 => Self::ASE,
            893 => Self::TSV,
            5002 => Self::ASX,
            570 => Self::LSE,
            892 => Self::TOR,
            454 => Self::HKS,
            _ => Self::Unknown(x),
        }
    }
}

impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NSDQ => write!(f, "NSDQ"),
            Self::NSY => write!(f, "NSY"),
            Self::EAM => write!(f, "EAM"),
            Self::XET => write!(f, "XET"),
            Self::TDG => write!(f, "TDG"),
            Self::EPA => write!(f, "EPA"),
            Self::WSE => write!(f, "WSE"),
            Self::TSE => write!(f, "TSE"),
            Self::OSL => write!(f, "OSL"),
            Self::SWX => write!(f, "SWX"),
            Self::OMX => write!(f, "OMX"),
            Self::ATH => write!(f, "ATH"),
            Self::ASE => write!(f, "ASE"),
            Self::TSV => write!(f, "TSV"),
            Self::ASX => write!(f, "ASX"),
            Self::LSE => write!(f, "LSE"),
            Self::TOR => write!(f, "TOR"),
            Self::HKS => write!(f, "HKS"),
            Self::Unknown(x) => write!(f, "Unknown({})", x),
        }
    }
}
