use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use itertools::Itertools;

use std::{collections::HashMap, convert::TryInto};
use strum::EnumString;
use thiserror::Error;

use crate::models::{Currency, Money};

use derive_more::{
    derive::{Deref, DerefMut},
    From, Into, Sub,
};

use super::{risk::RiskCategory, Product};

/// A portfolio object received from the Degiro API.
///
/// Contains a collection of individual positions represented as `ValueObject`s.
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct PortfolioObject {
    value: Vec<ValueObject>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ValueObject {
    #[serde(rename = "name")]
    elem_type: ElemType,
    value: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, EnumString, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ElemType {
    Id,
    PositionType,
    Size,
    Price,
    Value,
    AccruedInterest,
    PlBase,
    TodayPlBase,
    PortfolioValueCorrection,
    BreakEvenPrice,
    AverageFxRate,
    RealizedProductPl,
    RealizedFxPl,
    TodayRealizedProductPl,
    TodayRealizedFxPl,
}

#[allow(dead_code)]
/// Represents the detailed information about a position in a portfolio.
///
/// This includes financial metrics like price, size, value, and various profit calculations
/// for both product and FX movements.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Position {
    pub id: String,
    pub position_type: PositionType,
    pub size: Decimal,
    pub price: Decimal,
    pub currency: Currency,
    pub value: Money,
    pub accrued_interest: Option<Decimal>,
    pub base_value: Money,
    pub today_value: Money,
    pub portfolio_value_correction: Decimal,
    pub break_even_price: Decimal,
    pub average_fx_rate: Decimal,
    pub realized_product_profit: Money,
    pub realized_fx_profit: Money,
    pub today_realized_product_pl: Money,
    pub today_realized_fx_pl: Money,
    pub total_profit: Money,
    pub product_profit: Money,
    pub fx_profit: Money,
    pub product: Option<Product>,
}

#[derive(Clone, Debug, Default, From, Deref, DerefMut, Into, Serialize)]
pub struct Portfolio(pub Vec<Position>);

// impl FromIterator<Position> for Portfolio {
//     fn from_iter<I: IntoIterator<Item = Position>>(iter: I) -> Self {
//         Self(iter.into_iter().collect())
//     }
// }

impl IntoIterator for Portfolio {
    type Item = Position;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Portfolio {
    type Item = &'a Position;
    type IntoIter = std::slice::Iter<'a, Position>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Portfolio {
    type Item = &'a mut Position;
    type IntoIter = std::slice::IterMut<'a, Position>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl FromIterator<Position> for Portfolio {
    fn from_iter<I: IntoIterator<Item = Position>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// Portfolio represents a collection of trading positions
impl Portfolio {
    // Constructor
    pub fn new(xs: impl Into<Vec<Position>>) -> Self {
        Self(xs.into())
    }

    // Private helpers
    fn filter(self, pred: impl Fn(&Position) -> bool) -> Self {
        let xs = self.0.into_iter().filter(pred).collect();
        Self(xs)
    }

    // Public filters
    pub fn cash(self) -> Self {
        self.filter(|p| p.position_type == PositionType::Cash)
    }

    pub fn current(self) -> Self {
        self.filter(|p| p.size > dec!(0.0))
    }

    pub fn only_id(self, id: &str) -> Self {
        self.filter(|p| p.id == id)
    }

    pub fn products(self) -> Self {
        self.filter(|p| p.position_type == PositionType::Product)
    }

    pub fn active(self) -> Self {
        self.filter(|p| p.size > dec!(0.0))
    }

    /// Groups positions by product category
    pub fn group_by_category(&self) -> HashMap<RiskCategory, Vec<&Position>> {
        self.0
            .iter()
            .filter_map(|p| p.product.as_ref().map(|prod| (prod.category, p)))
            .into_group_map()
    }

    /// Groups positions by sector based on product information
    pub fn group_by_sector(&self) -> HashMap<String, Vec<&Position>> {
        self.0
            .iter()
            .filter_map(|p| {
                p.product.as_ref().map(|prod| {
                    (
                        prod.company_profile
                            .as_ref()
                            .map(|cp| cp.sector.clone())
                            .unwrap_or_else(|| "Unknown".to_string()),
                        p,
                    )
                })
            })
            .into_group_map()
    }

    // Value calculations
    pub fn base_value(&self) -> HashMap<Currency, Decimal> {
        let mut m = HashMap::default();
        for p in &self.0 {
            let money = &p.base_value;
            let x = m.entry(money.currency()).or_insert(Decimal::ZERO);
            *x += money.amount();
        }
        m
    }

    pub fn value(&self) -> HashMap<Currency, Decimal> {
        let mut m = HashMap::default();
        for p in &self.0 {
            let money = &p.value;
            let x = m.entry(money.currency()).or_insert(Decimal::ZERO);
            *x += money.amount();
        }
        m
    }

    /// Calculates the Value at Risk (VaR) for the portfolio
    /// Returns a HashMap mapping each currency to its VaR value
    pub fn value_at_risk(&self, confidence_level: f64) -> HashMap<Currency, Decimal> {
        let mut var_by_currency = HashMap::new();

        // Group positions by currency
        let positions_by_currency = self.0.iter().group_by(|p| p.currency);

        for (currency, positions) in &positions_by_currency {
            let total_value: Decimal = positions.map(|p| p.value.amount().abs()).sum();

            // Simple VaR calculation using total value * confidence level
            let var =
                total_value * Decimal::from_f64(1.0 - confidence_level).unwrap_or(Decimal::ZERO);

            var_by_currency.insert(currency, var);
        }

        var_by_currency
    }

    /// Calculates portfolio concentration by currency
    /// Returns percentage of total portfolio value in each currency
    pub fn currency_concentration(&self) -> HashMap<Currency, Decimal> {
        let total_value: Decimal = self.0.iter().map(|p| p.value.amount().abs()).sum();

        if total_value == Decimal::ZERO {
            return HashMap::new();
        }

        self.0
            .iter()
            .group_by(|p| p.currency)
            .into_iter()
            .map(|(currency, positions)| {
                let currency_value: Decimal = positions.map(|p| p.value.amount().abs()).sum();
                (currency, currency_value / total_value)
            })
            .collect()
    }

    /// Calculates the maximum drawdown across all positions
    pub fn max_drawdown(&self) -> HashMap<Currency, Decimal> {
        let mut drawdowns = HashMap::new();

        for position in &self.0 {
            let drawdown = if position.break_even_price > position.price {
                (position.break_even_price - position.price) / position.break_even_price
            } else {
                Decimal::ZERO
            };

            let entry = drawdowns.entry(position.currency).or_insert(Decimal::ZERO);
            *entry = (*entry).max(drawdown);
        }

        drawdowns
    }
}

#[derive(Clone, Debug, Default, EnumString, PartialEq, Eq, Hash, Serialize)]
#[strum(ascii_case_insensitive)]
pub enum PositionType {
    Cash,
    #[default]
    Product,
}

#[derive(Debug, Error)]
pub enum ParsePositionError {
    #[error("Missing required field {0}")]
    MissingField(&'static str),
    #[error("Failed to parse value for field {0}")]
    InvalidValue(&'static str),
    #[error("Failed to parse position type")]
    InvalidPositionType,
    #[error("Failed to parse currency information")]
    InvalidCurrency,
    #[error("Failed to deserialize JSON value")]
    JsonDeserialize(#[from] serde_json::Error),
    #[error("Value conversion failed")]
    ValueConversion,
    #[error("Invalid portfolio data: {0}")]
    InvalidData(String),
}

impl TryFrom<PortfolioObject> for Position {
    type Error = ParsePositionError;

    fn try_from(obj: PortfolioObject) -> Result<Self, Self::Error> {
        let mut position = Position::default();
        let mut value = 0.0;
        for row in &obj.value {
            match row.elem_type {
                ElemType::Id => {
                    position.id = row
                        .value
                        .as_ref()
                        .and_then(|v| v.as_str())
                        .ok_or(ParsePositionError::MissingField("id"))?
                        .to_string();
                }
                ElemType::PositionType => {
                    position.position_type = row
                        .value
                        .as_ref()
                        .and_then(|v| v.as_str())
                        .ok_or(ParsePositionError::MissingField("position_type"))?
                        .parse()
                        .map_err(|_| ParsePositionError::InvalidPositionType)?;
                }
                ElemType::Size => {
                    position.size = row
                        .value
                        .as_ref()
                        .and_then(|v| v.as_f64())
                        .and_then(Decimal::from_f64)
                        .ok_or(ParsePositionError::MissingField("size"))?;
                }
                ElemType::Price => {
                    position.price = row
                        .value
                        .as_ref()
                        .and_then(|v| v.as_f64())
                        .and_then(Decimal::from_f64)
                        .ok_or(ParsePositionError::InvalidValue("price"))?;
                }
                ElemType::Value => {
                    value = row
                        .value
                        .as_ref()
                        .and_then(|v| v.as_f64())
                        .ok_or(ParsePositionError::MissingField("value"))?;
                }
                ElemType::AccruedInterest => {
                    if let Some(s) = &row.value {
                        if let Some(val) = s.as_f64().and_then(Decimal::from_f64) {
                            if val > Decimal::ZERO {
                                position.accrued_interest = Some(val);
                            }
                        }
                    }
                }
                ElemType::PlBase => {
                    let map = row
                        .value
                        .as_ref()
                        .ok_or(ParsePositionError::MissingField("pl_base"))?;
                    let money_map = serde_json::from_value::<HashMap<String, Decimal>>(map.clone())
                        .map_err(|_| {
                            ParsePositionError::InvalidData(
                                "Failed to parse PlBase value".to_string(),
                            )
                        })?;
                    let money = TryInto::<Money>::try_into(money_map).map_err(|_| {
                        ParsePositionError::InvalidData("Invalid PlBase value".to_string())
                    })?;
                    position.currency = money.currency();
                    position.base_value = -money;
                }
                ElemType::TodayPlBase => {
                    let map = row
                        .value
                        .as_ref()
                        .ok_or(ParsePositionError::MissingField("today_pl_base"))?;
                    let money_map = serde_json::from_value::<HashMap<String, Decimal>>(map.clone())
                        .map_err(|_| {
                            ParsePositionError::InvalidData(
                                "Failed to parse TodayPlBase value".to_string(),
                            )
                        })?;
                    position.today_value = TryInto::<Money>::try_into(money_map).map_err(|_| {
                        ParsePositionError::InvalidData("Invalid TodayPlBase value".to_string())
                    })?;
                }
                ElemType::PortfolioValueCorrection => {
                    position.portfolio_value_correction = row
                        .value
                        .as_ref()
                        .and_then(|v| v.as_f64())
                        .and_then(Decimal::from_f64)
                        .ok_or(ParsePositionError::InvalidValue(
                            "portfolio_value_correction",
                        ))?;
                }
                ElemType::BreakEvenPrice => {
                    position.break_even_price = row
                        .value
                        .as_ref()
                        .and_then(|v| v.as_f64())
                        .and_then(Decimal::from_f64)
                        .ok_or(ParsePositionError::InvalidValue("break_even_price"))?;
                }
                ElemType::AverageFxRate => {
                    position.average_fx_rate = row
                        .value
                        .as_ref()
                        .and_then(|v| v.as_f64())
                        .and_then(Decimal::from_f64)
                        .ok_or(ParsePositionError::InvalidValue("average_fx_rate"))?;
                }
                ElemType::RealizedProductPl => {
                    let val = row
                        .value
                        .as_ref()
                        .and_then(|v| v.as_f64())
                        .and_then(Decimal::from_f64)
                        .ok_or(ParsePositionError::InvalidValue("realized_product_pl"))?;
                    position.realized_product_profit = Money::new(position.currency, val);
                }
                ElemType::RealizedFxPl => {
                    let val = row
                        .value
                        .as_ref()
                        .and_then(|v| v.as_f64())
                        .and_then(Decimal::from_f64)
                        .ok_or(ParsePositionError::InvalidValue("realized_fx_pl"))?;
                    position.realized_fx_profit = Money::new(position.currency, val);
                }
                ElemType::TodayRealizedProductPl => {
                    let val = row
                        .value
                        .as_ref()
                        .and_then(|v| v.as_f64())
                        .and_then(Decimal::from_f64)
                        .ok_or(ParsePositionError::InvalidValue(
                            "today_realized_product_pl",
                        ))?;
                    position.today_realized_product_pl = Money::new(position.currency, val);
                }
                ElemType::TodayRealizedFxPl => {
                    let val = row
                        .value
                        .as_ref()
                        .and_then(|v| v.as_f64())
                        .and_then(Decimal::from_f64)
                        .ok_or(ParsePositionError::InvalidValue("today_realized_fx_pl"))?;
                    position.today_realized_fx_pl = Money::new(position.currency, val);
                }
            }
        }
        let currency = position.total_profit.currency();
        position.total_profit = Money::new(
            currency,
            (position.price * position.size - position.break_even_price * position.size)
                * position.average_fx_rate,
        );
        let profit = if position.average_fx_rate == Decimal::ZERO {
            Decimal::ZERO
        } else {
            (position.price * position.size)
                - (position.break_even_price * position.size) / position.average_fx_rate
        };
        position.product_profit = Money::new(currency, profit);
        position.value = Money::new(
            currency,
            Decimal::try_from(value).map_err(|_| ParsePositionError::ValueConversion)?,
        );
        position.fx_profit = match position.total_profit.sub(position.product_profit) {
            Ok(total_minus_product) => match total_minus_product.sub(position.realized_fx_profit) {
                Ok(result) => result,
                Err(_) => return Err(ParsePositionError::ValueConversion),
            },
            Err(_) => return Err(ParsePositionError::ValueConversion),
        };
        Ok(position)
    }
}
