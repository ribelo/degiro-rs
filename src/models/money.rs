use std::collections::HashMap;

use rust_decimal::prelude::FromPrimitive;
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
    Ord,
    PartialOrd,
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

    pub fn try_new_from_f64(currency: Currency, amount: f64) -> Result<Self, MoneyError> {
        Decimal::from_f64(amount)
            .map(|decimal| Self::new(currency, decimal))
            .ok_or(MoneyError::Parse)
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
        Self::new(self.currency, self.amount.abs())
    }

    pub fn convert_to(
        &self,
        to: Currency,
        client: &crate::client::Degiro,
    ) -> Result<Money, crate::error::ClientError> {
        if self.currency == to {
            return Ok(*self);
        }

        let rate = client.get_rate(self.currency, to)?;
        Ok(Money::new(to, self.amount * rate))
    }

    pub fn try_add(
        &self,
        other: Self,
        client: &crate::client::Degiro,
    ) -> Result<Money, crate::error::ClientError> {
        if self.currency == other.currency {
            Ok(Money::new(self.currency, self.amount + other.amount))
        } else {
            let converted_other = other.convert_to(self.currency, client)?;
            Ok(Money::new(
                self.currency,
                self.amount + converted_other.amount,
            ))
        }
    }

    pub fn try_sub(
        &self,
        other: Self,
        client: &crate::client::Degiro,
    ) -> Result<Money, crate::error::ClientError> {
        if self.currency == other.currency {
            Ok(Money::new(self.currency, self.amount - other.amount))
        } else {
            let converted_other = other.convert_to(self.currency, client)?;
            Ok(Money::new(
                self.currency,
                self.amount - converted_other.amount,
            ))
        }
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let precision = f.precision().unwrap_or(2);

        if f.sign_plus() {
            write!(f, "{:+.*} {}", precision, self.amount, self.currency)
        } else {
            write!(f, "{:.*} {}", precision, self.amount, self.currency)
        }
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
        self.amount += rhs;
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
        self.amount -= rhs;
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
        self.amount *= rhs;
    }
}

impl std::ops::Div for Money {
    type Output = Result<Self, MoneyError>;

    fn div(self, rhs: Self) -> Self::Output {
        if self.currency != rhs.currency {
            return Err(MoneyError::Div(self.currency, rhs.currency));
        }
        if rhs.amount.is_zero() {
            return Err(MoneyError::DivisionByZero);
        }
        Ok(Self::new(self.currency, self.amount / rhs.amount))
    }
}

impl std::ops::Div<Decimal> for Money {
    type Output = Self;

    fn div(self, rhs: Decimal) -> Self::Output {
        if rhs.is_zero() {
            panic!("Division by zero");
        }
        Self::new(self.currency, self.amount / rhs)
    }
}

impl std::ops::DivAssign<Decimal> for Money {
    fn div_assign(&mut self, rhs: Decimal) {
        if rhs.is_zero() {
            panic!("Division by zero");
        }
        self.amount /= rhs;
    }
}

impl std::iter::Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| (a + b).unwrap_or_else(|_| Money::default()))
            .unwrap_or_default()
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
    use std::collections::HashMap;

    fn create_test_client_with_rates() -> crate::client::Degiro {
        use crate::client::Degiro;

        let client = Degiro::builder()
            .username("test".to_string())
            .password("test".to_string())
            .totp_secret("test".to_string())
            .build();

        let mut rates = HashMap::new();
        rates.insert("EUR/USD".to_string(), dec!(1.10));
        rates.insert("USD/GBP".to_string(), dec!(0.75));
        rates.insert("EUR/GBP".to_string(), dec!(0.825));

        client.session.set_currency_rates(rates);
        client
    }

    #[test]
    fn test_money_creation() {
        let money = Money::new(Currency::EUR, dec!(10.5));
        assert_eq!(money.currency(), Currency::EUR);
        assert_eq!(money.amount(), dec!(10.5));
    }

    #[test]
    fn test_money_addition() {
        let m1 = Money::new(Currency::EUR, dec!(10.0));
        let m2 = Money::new(Currency::EUR, dec!(20.0));
        let m3 = Money::new(Currency::USD, dec!(20.0));

        assert_eq!(
            (m1 + m2).expect("Failed to add money with same currency"),
            Money::new(Currency::EUR, dec!(30.0))
        );
        assert!(matches!(m1 + m3, Err(MoneyError::Add(_, _))));
    }

    #[test]
    fn test_money_decimal_ops() {
        let mut money = Money::new(Currency::EUR, dec!(10.0));

        assert_eq!(money + dec!(5.0), Money::new(Currency::EUR, dec!(15.0)));
        assert_eq!(money - dec!(5.0), Money::new(Currency::EUR, dec!(5.0)));
        assert_eq!(money * dec!(2.0), Money::new(Currency::EUR, dec!(20.0)));

        money += dec!(5.0);
        assert_eq!(money, Money::new(Currency::EUR, dec!(15.0)));

        money -= dec!(3.0);
        assert_eq!(money, Money::new(Currency::EUR, dec!(12.0)));

        money *= dec!(2.0);
        assert_eq!(money, Money::new(Currency::EUR, dec!(24.0)));
    }

    #[test]
    fn test_money_conversion() {
        let mut map = HashMap::new();
        map.insert("EUR".to_string(), dec!(10.0));

        let money = Money::try_from(map).expect("Failed to create Money from HashMap");
        assert_eq!(money, Money::new(Currency::EUR, dec!(10.0)));

        let money = Money::try_from(("EUR".to_string(), dec!(10.0)))
            .expect("Failed to create Money from tuple");
        assert_eq!(money, Money::new(Currency::EUR, dec!(10.0)));

        let money = Money::from((Currency::EUR, dec!(10.0)));
        assert_eq!(money, Money::new(Currency::EUR, dec!(10.0)));
    }

    #[test]
    fn test_money_display() {
        let money = Money::new(Currency::EUR, dec!(10.5));
        assert_eq!(money.to_string(), "10.50 EUR");
    }

    #[test]
    fn test_currency_conversion_same_currency() {
        let client = create_test_client_with_rates();
        let eur_money = Money::new(Currency::EUR, dec!(100.0));

        let result = eur_money
            .convert_to(Currency::EUR, &client)
            .expect("Failed to convert to same currency");
        assert_eq!(result, eur_money);
    }

    #[test]
    fn test_currency_conversion_direct_rate() {
        let client = create_test_client_with_rates();
        let eur_money = Money::new(Currency::EUR, dec!(100.0));

        let usd_result = eur_money
            .convert_to(Currency::USD, &client)
            .expect("Failed to convert EUR to USD");
        assert_eq!(usd_result.amount(), dec!(110.0));
    }

    #[test]
    fn test_currency_conversion_inverse_rate() {
        let client = create_test_client_with_rates();
        let usd_money = Money::new(Currency::USD, dec!(110.0));

        let eur_result = usd_money
            .convert_to(Currency::EUR, &client)
            .expect("Failed to convert USD to EUR");
        assert_eq!(eur_result.currency(), Currency::EUR);
        assert!(eur_result.amount().round_dp(2) == dec!(100.00));
    }

    #[test]
    fn test_currency_conversion_missing_rate() {
        let client = create_test_client_with_rates();
        let chf_money = Money::new(Currency::CHF, dec!(100.0));

        let result = chf_money.convert_to(Currency::JPY, &client);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_add_same_currency() {
        let client = create_test_client_with_rates();
        let m1 = Money::new(Currency::EUR, dec!(100.0));
        let m2 = Money::new(Currency::EUR, dec!(50.0));

        let result = m1
            .try_add(m2, &client)
            .expect("Failed to add money with same currency");
        assert_eq!(result, Money::new(Currency::EUR, dec!(150.0)));
    }

    #[test]
    fn test_try_add_different_currencies() {
        let client = create_test_client_with_rates();
        let eur_money = Money::new(Currency::EUR, dec!(100.0));
        let usd_money = Money::new(Currency::USD, dec!(110.0));

        let result = eur_money
            .try_add(usd_money, &client)
            .expect("Failed to add money with different currencies");
        assert_eq!(result.currency(), Currency::EUR);
        assert!(result.amount().round_dp(2) == dec!(200.00));
    }

    #[test]
    fn test_try_sub_same_currency() {
        let client = create_test_client_with_rates();
        let m1 = Money::new(Currency::EUR, dec!(100.0));
        let m2 = Money::new(Currency::EUR, dec!(30.0));

        let result = m1
            .try_sub(m2, &client)
            .expect("Failed to subtract money with same currency");
        assert_eq!(result, Money::new(Currency::EUR, dec!(70.0)));
    }

    #[test]
    fn test_try_sub_different_currencies() {
        let client = create_test_client_with_rates();
        let eur_money = Money::new(Currency::EUR, dec!(200.0));
        let usd_money = Money::new(Currency::USD, dec!(110.0));

        let result = eur_money
            .try_sub(usd_money, &client)
            .expect("Failed to subtract money with different currencies");
        assert_eq!(result.currency(), Currency::EUR);
        assert!(result.amount().round_dp(2) == dec!(100.00));
    }
}
