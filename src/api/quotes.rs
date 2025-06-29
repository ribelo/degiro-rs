use std::ops::{Deref, DerefMut};

use chrono::NaiveDateTime;
use reqwest::{header, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    client::{ApiErrorResponse, ClientError, ClientStatus, Degiro},
    models::Period,
    paths::REFERER,
};

#[derive(Debug, Deserialize)]
struct Ohlc {
    n: u64,
    o: f64,
    h: f64,
    l: f64,
    c: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub time: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Candles {
    pub interval: Period,
    pub data: Vec<Candle>,
}

impl Deref for Candles {
    type Target = Vec<Candle>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Candles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Candles {
    pub fn retain_incompleted(mut self) -> Self {
        if self.data.len() <= 1 {
            return self;
        }

        if let Some(last) = self.data.last() {
            if last.time < self.data[self.data.len() - 2].time + self.interval {
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

        let periods_elapsed = (0..n).fold(self.data[prev_idx].time, |acc, _| acc + period);
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
                    chronoutil::delta::with_day((0..x.n).fold(start, |acc, _| acc + interval), 31)
                        .ok_or_else(|| {
                        ClientError::UnexpectedError("Failed to compute month delta".into())
                    })?
                } else {
                    end
                }
            } else {
                (0..x.n).fold(start, |acc, _| acc + interval)
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
    pub async fn quotes_by_id(
        &self,
        isin: impl AsRef<str>,
        period: Period,
        interval: Period,
    ) -> Result<Option<Candles>, ClientError> {
        let Some(product) = self.product(isin.as_ref()).await? else {
            return Ok(None);
        };
        let Some(vwd_id) = product.vwd_id else {
            return Ok(None);
        };
        self.quotes(vwd_id.as_str(), period, interval).await
    }
    pub async fn quotes(
        &self,
        vwd_id: impl AsRef<str>,
        period: Period,
        interval: Period,
    ) -> Result<Option<Candles>, ClientError> {
        let vwd_id = vwd_id.as_ref();
        self.ensure_authorized().await?;

        let req = {
            let base_url = "https://charting.vwdservices.com/hchart/v1/deGiro/data.js";
            let url =
                Url::parse(base_url).map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

            self.http_client
                .get(url)
                .query(&[
                    ("requestid", "1"),
                    ("format", "json"),
                    ("resolution", &interval.to_string()),
                    ("period", &period.to_string()),
                    ("series", &format!("ohlc:issueid:{}", vwd_id)),
                    ("userToken", &self.client_id().to_string()),
                ])
                .header(header::REFERER, REFERER)
        };

        self.acquire_limit().await;

        let res = req.send().await?;

        if let Err(err) = res.error_for_status_ref() {
            let Some(status) = err.status() else {
                return Err(ClientError::UnexpectedError(err.to_string()));
            };

            if status.as_u16() == 401 {
                self.set_auth_state(ClientStatus::Unauthorized);
                return Err(ClientError::Unauthorized);
            }

            let error_response = res.json::<ApiErrorResponse>().await?;
            return Err(ClientError::ApiError(error_response));
        }

        let mut body = res.json::<Value>().await?;

        let error = body
            .get("series")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.get("error"))
            .and_then(|error| error.as_str());

        if let Some(error) = error {
            return Err(ClientError::UnexpectedError(error.to_string()));
        }

        let start = body
            .get("start")
            .ok_or_else(|| ClientError::UnexpectedError("Missing start timestamp".into()))?;
        let start = serde_json::from_value::<NaiveDateTime>(start.clone())?;

        let end = body
            .get("end")
            .ok_or_else(|| ClientError::UnexpectedError("Missing end timestamp".into()))?;
        let end = serde_json::from_value::<NaiveDateTime>(end.clone())?;

        let data = body
            .get_mut("series")
            .and_then(|s| s.as_array_mut())
            .and_then(|arr| arr.first_mut())
            .and_then(|s| s.get_mut("data"))
            .map(|d| d.take())
            .ok_or_else(|| {
                ClientError::UnexpectedError("Missing or invalid data in series".to_string())
            })?;

        let ohlc_vec = serde_json::from_value::<Vec<Ohlc>>(data)?;
        Ok(Some(ohlc_vec_to_candles(start, end, interval, ohlc_vec)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_quotes() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let quotes = client
            .quotes_by_id("332111", Period::P1Y, Period::P1M)
            .await
            .ok()
            .flatten()
            .map(|c| c.retain_by_min_periods(15, Period::P1D));
        dbg!(quotes);
    }
}
