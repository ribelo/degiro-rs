use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    client::Degiro,
    error::{ClientError, DataError, DateTimeError, ResponseError},
    http::{HttpClient, HttpRequest},
    models::{Currency, Period, SeriesIdentifier, SeriesIdentifierKind},
    serde_utils::f64_from_string_or_number,
};

#[derive(Debug, Deserialize)]
struct Ohlc {
    n: u64,
    #[serde(deserialize_with = "f64_from_string_or_number")]
    o: f64,
    #[serde(deserialize_with = "f64_from_string_or_number")]
    h: f64,
    #[serde(deserialize_with = "f64_from_string_or_number")]
    l: f64,
    #[serde(deserialize_with = "f64_from_string_or_number")]
    c: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Candle {
    #[serde(deserialize_with = "f64_from_string_or_number")]
    pub open: f64,
    #[serde(deserialize_with = "f64_from_string_or_number")]
    pub high: f64,
    #[serde(deserialize_with = "f64_from_string_or_number")]
    pub low: f64,
    #[serde(deserialize_with = "f64_from_string_or_number")]
    pub close: f64,
    pub time: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Candles {
    pub interval: Period,
    pub data: Vec<Candle>,
}

fn parse_chart_response(raw: &str) -> Result<Value, ClientError> {
    let trimmed = raw.trim();
    match serde_json::from_str::<Value>(trimmed) {
        Ok(value) => Ok(value),
        Err(first_err) => {
            let first = trimmed.find('{');
            let last = trimmed.rfind('}');
            if let (Some(start), Some(end)) = (first, last) {
                if end >= start {
                    let candidate = &trimmed[start..=end];
                    if let Ok(value) = serde_json::from_str(candidate) {
                        return Ok(value);
                    }
                }
            }
            Err(ClientError::SerdeError(first_err))
        }
    }
}

fn parse_chart_datetime(value: &Value) -> Result<NaiveDateTime, ClientError> {
    if let Some(text) = value.as_str() {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(text) {
            return Ok(dt.naive_utc());
        }
        match NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S") {
            Ok(dt) => return Ok(dt),
            Err(err) => {
                return Err(ClientError::DateTimeError(DateTimeError::parse_error(
                    text,
                    err.to_string(),
                )));
            }
        }
    }

    serde_json::from_value::<NaiveDateTime>(value.clone()).map_err(ClientError::from)
}

impl Candles {
    /// Create a new Candles collection
    pub fn new(interval: Period, data: Vec<Candle>) -> Self {
        Self { interval, data }
    }

    /// Get a reference to the candles data
    pub fn candles(&self) -> &[Candle] {
        &self.data
    }

    /// Get a mutable reference to the candles data
    pub fn candles_mut(&mut self) -> &mut Vec<Candle> {
        &mut self.data
    }

    /// Get the number of candles
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Add a candle to the collection
    pub fn push(&mut self, candle: Candle) {
        self.data.push(candle);
    }

    /// Remove the last candle
    pub fn pop(&mut self) -> Option<Candle> {
        self.data.pop()
    }

    /// Get the last candle
    pub fn last(&self) -> Option<&Candle> {
        self.data.last()
    }

    /// Iterate over the candles
    pub fn iter(&self) -> std::slice::Iter<'_, Candle> {
        self.data.iter()
    }

    /// Iterate over the candles mutably
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Candle> {
        self.data.iter_mut()
    }

    /// Convert into the underlying Vec
    pub fn into_data(self) -> Vec<Candle> {
        self.data
    }
}

impl Candles {
    pub fn retain_incompleted(mut self) -> Self {
        if self.data.len() <= 1 {
            return self;
        }

        if let Some(last) = self.data.last() {
            if last.time
                < self
                    .interval
                    .add_to_datetime_naive(self.data[self.data.len() - 2].time)
            {
                self.data.pop();
            }
        }
        self
    }

    pub fn retain_by_min_periods(self, n: u32, period: Period) -> Self {
        if self.data.len() < 2 {
            return self;
        }

        // Check last candle against previous candle
        let last_idx = self.data.len() - 1;
        let prev_idx = last_idx - 1;

        let periods_elapsed = (0..n).fold(self.data[prev_idx].time, |acc, _| {
            period.add_to_datetime_naive(acc)
        });
        if periods_elapsed > self.data[last_idx].time {
            return Candles {
                interval: self.interval,
                data: self.data[..last_idx].to_vec(),
            };
        }

        self
    }
}

fn ohlc_vec_to_candles(
    start: NaiveDateTime,
    end: NaiveDateTime,
    interval: Period,
    ohlc: Vec<Ohlc>,
) -> Result<Candles, ClientError> {
    let data = ohlc
        .iter()
        .enumerate()
        .map(|(i, x)| -> Result<Candle, ClientError> {
            let time = if matches!(
                interval,
                Period::P1M
                    | Period::P3M
                    | Period::P6M
                    | Period::P1Y
                    | Period::P3Y
                    | Period::P5Y
                    | Period::P50Y
            ) {
                if i != ohlc.len() - 1 {
                    chronoutil::delta::with_day(
                        (0..x.n).fold(start, |acc, _| interval.add_to_datetime_naive(acc)),
                        31,
                    )
                    .ok_or_else(|| {
                        ClientError::from(DateTimeError::ParseError {
                            input: "month delta computation".to_string(),
                            reason: "Failed to compute month delta".to_string(),
                        })
                    })?
                } else {
                    end
                }
            } else {
                (0..x.n).fold(start, |acc, _| interval.add_to_datetime_naive(acc))
            };

            Ok(Candle {
                time,
                open: x.o,
                high: x.h,
                low: x.l,
                close: x.c,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Candles { interval, data })
}

impl Degiro {
    pub async fn resolve_vwd_id_by_product_id(
        &self,
        product_id: impl AsRef<str>,
    ) -> Result<Option<SeriesIdentifier>, ClientError> {
        let product_id = product_id.as_ref();
        if let Some(series) = self.session.cached_series_by_product(product_id) {
            return Ok(Some(series));
        }

        let Some(product) = self.product(product_id.to_string()).await? else {
            return Ok(None);
        };

        self.cache_product_identifiers(&product);

        Ok(product.first_series_identifier())
    }

    pub async fn resolve_vwd_id_by_isin(
        &self,
        isin: impl AsRef<str>,
    ) -> Result<Option<SeriesIdentifier>, ClientError> {
        let isin = isin.as_ref();

        if let Some(product_id) = self.session.cached_product_id_by_isin(isin) {
            if let Some(series) = self.session.cached_series_by_product(&product_id) {
                return Ok(Some(series));
            }
            if let Some(vwd_id) = self.resolve_vwd_id_by_product_id(&product_id).await? {
                return Ok(Some(vwd_id));
            }
        }

        let Some(product) = self.product(isin.to_string()).await? else {
            return Ok(None);
        };

        self.cache_product_identifiers(&product);

        Ok(product.first_series_identifier())
    }

    pub async fn quotes_by_product_id(
        &self,
        product_id: impl AsRef<str>,
        period: Period,
        interval: Period,
    ) -> Result<Option<Candles>, ClientError> {
        let product_id = product_id.as_ref();
        let Some(series) = self.resolve_vwd_id_by_product_id(product_id).await? else {
            return Ok(None);
        };
        self.quotes_with_series(&series, period, interval).await
    }

    pub async fn quotes_by_isin(
        &self,
        isin: impl AsRef<str>,
        period: Period,
        interval: Period,
    ) -> Result<Option<Candles>, ClientError> {
        let isin = isin.as_ref();
        let Some(series) = self.resolve_vwd_id_by_isin(isin).await? else {
            return Ok(None);
        };
        self.quotes_with_series(&series, period, interval).await
    }

    pub async fn quotes_fx(
        &self,
        base: Currency,
        quote: Currency,
        period: Period,
        interval: Period,
    ) -> Result<Option<Candles>, ClientError> {
        if base == quote {
            return Ok(None);
        }

        if self.session.cached_fx_pair_product(base, quote).is_none() {
            let _ = self.account_info().await?;
        }

        if let Some(product_id) = self.session.cached_fx_pair_product(base, quote) {
            return self
                .quotes_by_product_id(product_id, period, interval)
                .await;
        }

        Ok(None)
    }

    pub async fn quotes(
        &self,
        vwd_id: impl AsRef<str>,
        period: Period,
        interval: Period,
    ) -> Result<Option<Candles>, ClientError> {
        self.quotes_with_kind(
            SeriesIdentifierKind::IssueId,
            vwd_id.as_ref(),
            period,
            interval,
        )
        .await
    }

    pub async fn quotes_with_kind(
        &self,
        kind: SeriesIdentifierKind,
        identifier: impl AsRef<str>,
        period: Period,
        interval: Period,
    ) -> Result<Option<Candles>, ClientError> {
        let series = SeriesIdentifier::new(kind, identifier.as_ref());
        self.quotes_with_series(&series, period, interval).await
    }

    pub async fn quotes_with_series(
        &self,
        series: &SeriesIdentifier,
        period: Period,
        interval: Period,
    ) -> Result<Option<Candles>, ClientError> {
        let url = "https://charting.vwdservices.com/hchart/v1/deGiro/data.js";

        let request = HttpRequest::get(url)
            .require_restricted() // Quotes only need login
            .query("requestid", "1")
            .query("format", "json")
            .query("resolution", interval.to_string())
            .query("period", period.to_string())
            .query(
                "series",
                format!("ohlc:{}:{}", series.kind().as_str(), series.value()),
            )
            .query("userToken", self.client_id().to_string());

        let raw = self.request_text(request).await?;
        let mut body = parse_chart_response(&raw)?;

        let error = body
            .get("series")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.get("error"))
            .and_then(|error| error.as_str());

        if let Some(error) = error {
            return Err(ResponseError::invalid(error.to_string()).into());
        }

        let Some(start_value) = body.get("start") else {
            return Ok(None);
        };
        let start = parse_chart_datetime(start_value)?;

        let end_value = body
            .get("end")
            .ok_or_else(|| DataError::missing_field("end"))?;
        let end = parse_chart_datetime(end_value)?;

        let data = body
            .get_mut("series")
            .and_then(|s| s.as_array_mut())
            .and_then(|arr| arr.first_mut())
            .and_then(|s| s.get_mut("data"))
            .map(|d| d.take())
            .ok_or_else(|| ClientError::from(DataError::missing_field("series[0].data")))?;

        let ohlc_vec = serde_json::from_value::<Vec<Ohlc>>(data)?;
        Ok(Some(ohlc_vec_to_candles(start, end, interval, ohlc_vec)?))
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn candle_deserializes_from_string_prices() {
        let json = r#"{"interval":"P1D","data":[{"open":"1.0","high":"2.0","low":"0.5","close":"1.5","time":"2024-01-01T00:00:00"}]}"#;
        let candles: Candles =
            serde_json::from_str(json).expect("Failed to deserialize candles with string prices");
        assert_eq!(candles.data.len(), 1);
        assert!((candles.data[0].close - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn ohlc_deserializes_from_mixed_price_types() {
        let json = r#"{"n":1,"o":"1.0","h":2.0,"l":"0.5","c":1.5}"#;
        let ohlc: Ohlc =
            serde_json::from_str(json).expect("Failed to deserialize OHLC with mixed price types");
        assert!((ohlc.o - 1.0).abs() < f64::EPSILON);
        assert!((ohlc.h - 2.0).abs() < f64::EPSILON);
        assert!((ohlc.l - 0.5).abs() < f64::EPSILON);
        assert!((ohlc.c - 1.5).abs() < f64::EPSILON);
    }

    use super::*;

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_quotes() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");
        let quotes = client
            .quotes_by_product_id("332111", Period::P1Y, Period::P1M)
            .await
            .ok()
            .flatten()
            .map(|c| c.retain_by_min_periods(15, Period::P1D));
        dbg!(quotes);
    }
}
