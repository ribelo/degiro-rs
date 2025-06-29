use std::{collections::HashMap, fmt::Display};

use rust_decimal::Decimal;

use serde::{Deserialize, Serialize};
use strum::EnumString;
use thiserror::Error;

#[derive(
    Debug,
    Default,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    Eq,
    PartialEq,
    EnumString,
    Hash,
    strum::Display,
)]

pub enum Currency {
    USD,
    #[default]
    EUR,
    CHF,
    JPY,
    PLN,
    GBP,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Money {
    currency: Currency,
    amount: Decimal,
}

impl Money {
    pub fn new(currency: Currency, amount: Decimal) -> Self {
        Self { currency, amount }
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn amount_mut(&mut self) -> &mut Decimal {
        &mut self.amount
    }

    pub fn abs(&self) -> Self {
        Self {
            amount: self.amount.abs(),
            currency: self.currency,
        }
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sign_plus = f.sign_plus();
        let precision = f.precision().unwrap_or(2);

        let amount_str = if sign_plus {
            format!("{:+.*}", precision, self.amount)
        } else {
            format!("{:.*}", precision, self.amount)
        };

        write!(f, "{} {}", amount_str, self.currency)
    }
}

#[derive(Debug, Error)]
pub enum MoneyError {
    #[error("Failed to parse money value")]
    Parse,
    #[error("Cannot add money with different currencies: {0} and {1}")]
    Add(Currency, Currency),
    #[error("Cannot subtract money with different currencies: {0} and {1}")]
    Sub(Currency, Currency),
    #[error("Cannot multiply money with different currencies: {0} and {1}")]
    Mul(Currency, Currency),

    #[error("Arithmetic operation failed")]
    ArithmeticError,
    #[error("Division by zero")]
    DivisionByZero,

    #[error("Cannot divide money with different currencies: {0} and {1}")]
    Div(Currency, Currency),
}

impl std::ops::Add<Money> for Money {
    type Output = Result<Self, MoneyError>;

    fn add(self, rhs: Self) -> Self::Output {
        if self.currency != rhs.currency {
            return Err(MoneyError::Add(self.currency, rhs.currency));
        }
        Ok(Self::new(self.currency, self.amount + rhs.amount))
    }
}

impl std::ops::Add<Decimal> for Money {
    type Output = Self;

    fn add(self, rhs: Decimal) -> Self::Output {
        Self::new(self.currency, self.amount + rhs)
    }
}

impl std::ops::AddAssign<Decimal> for Money {
    fn add_assign(&mut self, rhs: Decimal) {
        self.amount = self.amount + rhs;
    }
}

impl std::ops::Sub for Money {
    type Output = Result<Self, MoneyError>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.currency != rhs.currency {
            return Err(MoneyError::Sub(self.currency, rhs.currency));
        }
        Ok(Self::new(self.currency, self.amount - rhs.amount))
    }
}

impl std::ops::Sub<Decimal> for Money {
    type Output = Self;

    fn sub(self, rhs: Decimal) -> Self::Output {
        Self::new(self.currency, self.amount - rhs)
    }
}

impl std::ops::SubAssign<Decimal> for Money {
    fn sub_assign(&mut self, rhs: Decimal) {
        self.amount = self.amount - rhs;
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
        if self.currency != rhs.currency {
            return Err(MoneyError::Mul(self.currency, rhs.currency));
        }
        Ok(Self::new(self.currency, self.amount * rhs.amount))
    }
}

impl std::ops::Mul<Decimal> for Money {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self::Output {
        Self::new(self.currency, self.amount * rhs)
    }
}

impl std::ops::MulAssign<Decimal> for Money {
    fn mul_assign(&mut self, rhs: Decimal) {
        self.amount = self.amount * rhs;
    }
}

impl std::ops::Div for Money {
    type Output = Result<Self, MoneyError>;

    fn div(self, rhs: Self) -> Self::Output {
        if self.currency != rhs.currency {
            return Err(MoneyError::Div(self.currency, rhs.currency));
        }
        Ok(Self::new(self.currency, self.amount / rhs.amount))
    }
}

impl std::ops::Div<Decimal> for Money {
    type Output = Self;

    fn div(self, rhs: Decimal) -> Self::Output {
        Self::new(self.currency, self.amount / rhs)
    }
}

impl std::ops::DivAssign<Decimal> for Money {
    fn div_assign(&mut self, rhs: Decimal) {
        self.amount = self.amount / rhs;
    }
}

impl std::iter::Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| (a + b).unwrap()).unwrap_or_default()
    }
}

impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.currency == other.currency {
            self.amount.partial_cmp(&other.amount)
        } else {
            None
        }
    }
}

impl Ord for Money {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl TryFrom<HashMap<String, Decimal>> for Money {
    type Error = MoneyError;

    fn try_from(map: HashMap<String, Decimal>) -> Result<Self, Self::Error> {
        let (currency_str, amount) = map.into_iter().next().ok_or(MoneyError::Parse)?;
        let currency = Currency::try_from(currency_str.as_str()).map_err(|_| MoneyError::Parse)?;
        Ok(Self::new(currency, amount))
    }
}

impl TryFrom<(String, Decimal)> for Money {
    type Error = MoneyError;

    fn try_from((currency_str, amount): (String, Decimal)) -> Result<Self, Self::Error> {
        let currency = Currency::try_from(currency_str.as_str()).map_err(|_| MoneyError::Parse)?;
        Ok(Self::new(currency, amount))
    }
}

impl From<(Currency, Decimal)> for Money {
    fn from((currency, amount): (Currency, Decimal)) -> Self {
        Self::new(currency, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_money_creation() {
        let money = Money::new(Currency::EUR, dec!(10.5));
        assert_eq!(money.currency(), Currency::EUR);
        assert_eq!(money.amount(), dec!(10.5));
    }

    #[test]
    fn test_money_addition() {
        let m1 = Money::new(Currency::EUR, dec!(10));
        let m2 = Money::new(Currency::EUR, dec!(20));
        let m3 = Money::new(Currency::USD, dec!(20));

        assert_eq!((m1 + m2).unwrap(), Money::new(Currency::EUR, dec!(30)));
        assert!(matches!(m1 + m3, Err(MoneyError::Add(_, _))));
    }

    #[test]
    fn test_money_decimal_ops() {
        let mut money = Money::new(Currency::EUR, dec!(10));

        assert_eq!(money + dec!(5), Money::new(Currency::EUR, dec!(15)));
        assert_eq!(money - dec!(5), Money::new(Currency::EUR, dec!(5)));
        assert_eq!(money * dec!(2), Money::new(Currency::EUR, dec!(20)));

        money += dec!(5);
        assert_eq!(money, Money::new(Currency::EUR, dec!(15)));

        money -= dec!(3);
        assert_eq!(money, Money::new(Currency::EUR, dec!(12)));

        money *= dec!(2);
        assert_eq!(money, Money::new(Currency::EUR, dec!(24)));
    }

    #[test]
    fn test_money_conversion() {
        let mut map = HashMap::new();
        map.insert("EUR".to_string(), dec!(10));

        let money = Money::try_from(map).unwrap();
        assert_eq!(money, Money::new(Currency::EUR, dec!(10)));

        let money = Money::try_from(("EUR".to_string(), dec!(10))).unwrap();
        assert_eq!(money, Money::new(Currency::EUR, dec!(10)));

        let money = Money::from((Currency::EUR, dec!(10)));
        assert_eq!(money, Money::new(Currency::EUR, dec!(10)));
    }

    #[test]
    fn test_money_display() {
        let money = Money::new(Currency::EUR, dec!(10.5));
        assert_eq!(money.to_string(), "10.50 EUR");
    }
}
