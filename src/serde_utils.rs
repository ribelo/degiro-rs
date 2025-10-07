use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;
use std::str::FromStr;

/// Accepts either JSON numbers or stringified numbers and converts to f64.
pub(crate) fn f64_from_string_or_number<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as DeError;

    match Value::deserialize(deserializer)? {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| DeError::custom("Failed to convert number to f64")),
        Value::String(s) => s.parse::<f64>().map_err(DeError::custom),
        Value::Null => Ok(0.0),
        other => Err(DeError::custom(format!(
            "Expected number or string for float field, received {other:?}"
        ))),
    }
}

/// Accepts either JSON numbers or stringified numbers and converts to Decimal.
pub(crate) fn decimal_from_string_or_number<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as DeError;

    match Value::deserialize(deserializer)? {
        Value::Number(n) => n
            .as_f64()
            .and_then(Decimal::from_f64)
            .ok_or_else(|| DeError::custom("Failed to convert number to Decimal")),
        Value::String(s) => {
            Decimal::from_str(&s).map_err(|_| DeError::custom("Invalid decimal string"))
        }
        Value::Null => Ok(Decimal::ZERO),
        other => Err(DeError::custom(format!(
            "Expected number or string for decimal field, received {other:?}"
        ))),
    }
}
