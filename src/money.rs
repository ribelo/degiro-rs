use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};
use strum::EnumString;
use thiserror::Error;

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, Eq, PartialEq, EnumString, Hash)]
pub enum Currency {
    USD,
    #[default]
    EUR,
    CHF,
    JPY,
    PLN,
    GBP,
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
pub struct Money {
    pub currency: Currency,
    pub amount: f64,
}

impl Money {
    pub fn new(currency: Currency, amount: f64) -> Self {
        Self { currency, amount }
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:#?}, {:#?}]", self.currency, self.amount)
    }
}

#[derive(Debug, Error)]
pub enum MoneyError {
    #[error("can't parse error")]
    ParseError,
    #[error("can't add {0}, {1}")]
    AddError(Money, Money),
    #[error("can't sub {0}, {1}")]
    SubError(Money, Money),
    #[error("can't mul {0}, {1}")]
    MulError(Money, Money),
    #[error("can't div {0}, {1}")]
    DivError(Money, Money),
}

impl std::ops::Add for Money {
    type Output = Result<Self, MoneyError>;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (
                Money {
                    currency: cx,
                    amount: ax,
                },
                Money {
                    currency: cy,
                    amount: ay,
                },
            ) if cx == cy => Ok(Money::new(*cx, ax + ay)),
            _ => Err(MoneyError::AddError(self, rhs)),
        }
    }
}

impl std::ops::Sub for Money {
    type Output = Result<Self, MoneyError>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (
                Money {
                    currency: cx,
                    amount: ax,
                },
                Money {
                    currency: cy,
                    amount: ay,
                },
            ) if cx == cy => Ok(Money::new(*cx, ax - ay)),
            _ => Err(MoneyError::SubError(self, rhs)),
        }
    }
}

impl std::ops::Neg for Money {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Money::new(self.currency, -self.amount)
    }
}

impl std::ops::Mul for Money {
    type Output = Result<Self, MoneyError>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (
                Money {
                    currency: cx,
                    amount: ax,
                },
                Money {
                    currency: cy,
                    amount: ay,
                },
            ) if cx == cy => Ok(Money::new(*cx, ax * ay)),
            _ => Err(MoneyError::MulError(self, rhs)),
        }
    }
}

impl std::ops::Div for Money {
    type Output = Result<Self, MoneyError>;

    fn div(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (
                Money {
                    currency: cx,
                    amount: ax,
                },
                Money {
                    currency: cy,
                    amount: ay,
                },
            ) if cx == cy => Ok(Money::new(*cx, ax / ay)),
            _ => Err(MoneyError::DivError(self, rhs)),
        }
    }
}

impl TryFrom<HashMap<String, f64>> for Money {
    type Error = MoneyError;

    fn try_from(m: HashMap<String, f64>) -> Result<Self, Self::Error> {
        if !m.is_empty() {
            let mut money = Money::new(Currency::USD, 0.0);
            if let Some((k, &v)) = m.iter().next() {
                let curr: Currency = k.parse().map_err(|_| MoneyError::ParseError)?;
                money.currency = curr;
                money.amount = v;
            }
            Ok(money)
        } else {
            Err(MoneyError::ParseError)
        }
    }
}
