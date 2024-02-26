use chrono::{DateTime, NaiveDateTime, Utc};
#[cfg(feature = "erfurt")]
use erfurt::candle::{Candle, Candles, CandlesExt};
use reqwest::{header, Url};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    client::{Client, ClientError, ClientStatus},
    util::Period,
};

use super::product::Product;

#[derive(Debug, Deserialize)]
struct CandlesData(Vec<Ohlc>);

#[derive(Debug, Deserialize)]
struct Ohlc {
    n: u64,
    o: f64,
    h: f64,
    l: f64,
    c: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Quotes {
    pub id: String,
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
    pub volume: Option<Vec<f64>>,
    pub time: Vec<DateTime<Utc>>,
}

#[cfg(feature = "erfurt")]
impl CandlesExt for Quotes {
    fn get(&self, index: usize) -> Option<erfurt::candle::Candle> {
        if index < self.time.len() {
            let symbol = &self.id;
            let open = self.open[index];
            let high = self.high[index];
            let low = self.low[index];
            let close = self.close[index];
            let volume = self.volume.as_ref().map(|x| x[index]);
            let time = self.time[index];

            Some(Candle {
                id: symbol.clone(),
                open,
                high,
                low,
                close,
                volume,
                time,
            })
        } else {
            None
        }
    }

    fn open(&self) -> &Vec<f64> {
        &self.open
    }

    fn high(&self) -> &Vec<f64> {
        &self.high
    }

    fn low(&self) -> &Vec<f64> {
        &self.low
    }

    fn close(&self) -> &Vec<f64> {
        &self.close
    }

    fn volume(&self) -> &Option<Vec<f64>> {
        &self.volume
    }

    fn time(&self) -> &Vec<DateTime<Utc>> {
        &self.time
    }

    fn last(&self) -> Option<erfurt::candle::Candle> {
        self.time.last().map(|time| Candle {
            id: self.id.clone(),
            open: *self.open.last().unwrap(),
            high: *self.high.last().unwrap(),
            low: *self.low.last().unwrap(),
            close: *self.close.last().unwrap(),
            volume: self.volume.as_ref().map(|xs| *xs.last().unwrap()),
            time: *time,
        })
    }

    fn take_last(&self, n: usize) -> Option<Candles> {
        let len = self.time.len();
        if len < n {
            None
        } else {
            let id = self.id.clone();
            let open = self.open[len - n..].to_vec();
            let high = self.high[len - n..].to_vec();
            let low = self.low[len - n..].to_vec();
            let close = self.close[len - n..].to_vec();
            let volume = self
                .volume
                .as_ref()
                .map(|xs| xs[len - n..].to_vec())
                .filter(|xs| xs.len() == n);
            let time = self.time[len - n..].to_vec();
            Some(Candles {
                id,
                open,
                high,
                low,
                close,
                volume,
                time,
            })
        }
    }
}

impl CandlesData {
    pub fn as_quotes(
        &self,
        id: impl Into<String>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        interval: Period,
    ) -> Quotes {
        let mut quotes = Quotes {
            id: id.into().to_uppercase(),
            ..Default::default()
        };
        for (i, x) in self.0.iter().enumerate() {
            let mut dt = (0..x.n).fold(start, |acc, _| acc + interval);
            match interval {
                Period::P1M
                | Period::P3M
                | Period::P6M
                | Period::P1Y
                | Period::P3Y
                | Period::P5Y
                | Period::P50Y => {
                    if i != self.0.len() - 1 {
                        dt = chronoutil::delta::with_day(dt, 31).unwrap();
                    } else {
                        dt = end;
                    }
                }
                _ => (),
            }
            quotes.time.push(dt);
            quotes.open.push(x.o);
            quotes.high.push(x.h);
            quotes.low.push(x.l);
            quotes.close.push(x.c);
        }
        quotes
    }
}

#[cfg(feature = "erfurt")]
impl From<Quotes> for Candles {
    fn from(quotes: Quotes) -> Self {
        Candles {
            id: quotes.id,
            open: quotes.open,
            high: quotes.high,
            low: quotes.low,
            close: quotes.close,
            volume: quotes.volume,
            time: quotes.time,
        }
    }
}

impl Client {
    pub async fn quotes(
        &self,
        id: &str,
        period: Period,
        interval: Period,
    ) -> Result<Quotes, ClientError> {
        if self.inner.lock().unwrap().status != ClientStatus::Authorized {
            return Err(ClientError::Unauthorized);
        }

        let product = self.product(id).await?;

        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = "https://charting.vwdservices.com/hchart/v1/deGiro/data.js";
            let url = Url::parse(base_url).unwrap();

            inner
                .http_client
                .get(url)
                .query(&[
                    ("requestid", 1.to_string()),
                    ("format", "json".to_string()),
                    ("resolution", interval.to_string()),
                    ("period", period.to_string()),
                    ("series", format!("ohlc:issueid:{}", &product.inner.vwd_id)),
                    ("userToken", inner.client_id.to_string()),
                ])
                .header(header::REFERER, &inner.referer)
        };

        let rate_limiter = {
            let inner = self.inner.lock().unwrap();
            inner.rate_limiter.clone()
        };
        rate_limiter.acquire_one().await;

        let res = req.send().await?;

        match res.error_for_status() {
            Ok(res) => {
                let body = res.json::<Value>().await?;
                let error = body
                    .get("series")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|obj| obj.get("error"))
                    .and_then(|error| error.as_str());

                if let Some(error) = error {
                    return Err(ClientError::Descripted(error.to_string()));
                }

                let start = serde_json::from_value::<NaiveDateTime>(body["start"].clone())?;
                let start: DateTime<Utc> = DateTime::from_naive_utc_and_offset(start, Utc);
                let end = serde_json::from_value::<NaiveDateTime>(body["end"].clone())?;
                let end: DateTime<Utc> = DateTime::from_naive_utc_and_offset(end, Utc);
                let series = body["series"].as_array().unwrap();
                let data = series.first().unwrap()["data"].clone();
                let candles = serde_json::from_value::<CandlesData>(data)?;
                let quotes = candles.as_quotes(&product.inner.id, start, end, interval);
                Ok(quotes)
            }
            Err(err) => match err.status() {
                Some(status) if status.as_u16() == 401 => {
                    self.inner.lock().unwrap().status = ClientStatus::Unauthorized;
                    Err(ClientError::Unauthorized)
                }
                _ => Err(ClientError::UnexpectedError {
                    source: Box::new(err),
                }),
            },
        }
    }
}

impl Product {
    pub async fn quotes(&self, period: Period, interval: Period) -> Result<Quotes, ClientError> {
        self.client.quotes(&self.inner.id, period, interval).await
    }
}

#[cfg(test)]
mod test {
    use crate::{client::Client, util::Period};

    #[tokio::test]
    async fn quotes() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let product = client.product("17461000").await.unwrap();
        let quotes = product.quotes(Period::P1Y, Period::P1D).await.unwrap();
        dbg!(quotes);
    }
}
