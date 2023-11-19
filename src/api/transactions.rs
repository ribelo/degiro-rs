use chrono::{DateTime, FixedOffset, NaiveDate};
use reqwest::{header, Url};
use serde::Deserialize;

use std::collections::HashMap;

use crate::client::{Client, ClientError};
use crate::util::TransactionType;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInner {
    pub auto_fx_fee_in_base_currency: f64,
    #[serde(rename = "buysell")]
    pub transaction_type: TransactionType,
    pub counter_party: Option<String>,
    pub date: DateTime<FixedOffset>,
    pub fee_in_base_currency: Option<f64>,
    pub fx_rate: f64,
    pub gross_fx_rate: f64,
    pub id: i32,
    pub nett_fx_rate: f64,
    pub order_type_id: Option<i8>,
    pub price: f64,
    pub product_id: i32,
    pub quantity: i32,
    pub total: f64,
    pub total_fees_in_base_currency: f64,
    pub total_in_base_currency: f64,
    pub total_plus_all_fees_in_base_currency: f64,
    pub total_plus_fee_in_base_currency: f64,
    pub trading_venue: Option<String>,
    pub transaction_type_id: i32,
    pub transfered: bool,
}

#[derive(Debug)]
pub struct Transaction<'a> {
    pub inner: TransactionInner,
    pub client: &'a Client,
}

impl<'a> Transaction<'a> {
    pub fn new(inner: TransactionInner, client: &'a Client) -> Self {
        Self { inner, client }
    }
}

#[derive(Debug)]
pub struct Transactions<'a> {
    pub inner: Vec<Transaction<'a>>,
}

impl<'a> Transactions<'a> {
    pub fn new(inner: Vec<Transaction<'a>>) -> Self {
        Self { inner }
    }
}

impl Client {
    pub async fn transactions(
        &self,
        from_date: impl Into<NaiveDate> + Send,
        to_date: impl Into<NaiveDate> + Send,
    ) -> Result<Transactions, ClientError> {
        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = &inner.account_config.reporting_url;
            let path_url = "v4/transactions";
            let url = Url::parse(base_url).unwrap().join(path_url).unwrap();

            inner
                .http_client
                .get(url)
                .query(&[
                    ("sessionId", &inner.session_id),
                    ("intAccount", &format!("{}", inner.int_account)),
                    ("fromDate", &from_date.into().format("%d/%m/%Y").to_string()),
                    ("toDate", &to_date.into().format("%d/%m/%Y").to_string()),
                    ("groupTransactionsByOrder", &"1".to_string()),
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
                let mut m = res
                    .json::<HashMap<String, Vec<TransactionInner>>>()
                    .await
                    .unwrap();
                let data = m.remove("data").unwrap();
                let xs: Vec<_> = {
                    data.into_iter()
                        .map(|x| Transaction::new(x, self))
                        .collect()
                };
                Ok(Transactions::new(xs))
            }
            Err(err) => match err.status().unwrap().as_u16() {
                401 => Err(ClientError::Unauthorized),
                _ => Err(ClientError::UnexpectedError {
                    source: Box::new(err),
                }),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;

    use crate::client::Client;

    #[tokio::test]
    async fn transactions() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();

        let transactions = client
            .transactions(
                NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(),
            )
            .await
            .unwrap();
        dbg!(transactions);
    }
}
