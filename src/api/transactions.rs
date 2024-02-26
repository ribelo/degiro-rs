use chrono::{DateTime, FixedOffset, NaiveDate};
use reqwest::{header, Url};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::client::{Client, ClientError, ClientStatus};
use crate::util::TransactionType;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDetails {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub inner: TransactionDetails,
    #[serde(skip)]
    pub client: Option<Client>,
}

impl Transaction {
    pub fn new(details: TransactionDetails, client: Client) -> Self {
        Self {
            inner: details,
            client: Some(client),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transactions(pub Vec<Transaction>);

impl Transactions {
    pub fn new(inner: Vec<Transaction>) -> Self {
        Self(inner)
    }
    pub fn iter(&self) -> std::slice::Iter<Transaction> {
        self.0.iter()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn first(&self) -> Option<&Transaction> {
        self.0.first()
    }
    pub fn last(&self) -> Option<&Transaction> {
        self.0.last()
    }
    pub fn get(&self, index: usize) -> Option<&Transaction> {
        self.0.get(index)
    }
    pub fn into_inner(self) -> Vec<Transaction> {
        self.0
    }
    pub fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0);
    }
    pub fn push(&mut self, other: Transaction) {
        self.0.push(other);
    }
    pub fn pop(&mut self) -> Option<Transaction> {
        self.0.pop()
    }
    pub fn remove(&mut self, index: usize) -> Transaction {
        self.0.remove(index)
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub fn as_slice(&self) -> &[Transaction] {
        self.0.as_slice()
    }
    pub fn as_mut_slice(&mut self) -> &mut [Transaction] {
        self.0.as_mut_slice()
    }
    pub fn into_details(self) -> Vec<TransactionDetails> {
        self.0.into_iter().map(|x| x.inner).collect()
    }
}

impl IntoIterator for Transactions {
    type Item = Transaction;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Client {
    pub async fn transactions(
        &self,
        from_date: impl Into<NaiveDate> + Send,
        to_date: impl Into<NaiveDate> + Send,
    ) -> Result<Transactions, ClientError> {
        if self.inner.lock().unwrap().status != ClientStatus::Authorized {
            return Err(ClientError::Unauthorized);
        }
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
                    .json::<HashMap<String, Vec<TransactionDetails>>>()
                    .await
                    .unwrap();
                let data = m.remove("data").unwrap();
                let xs: Vec<_> = {
                    data.into_iter()
                        .map(|x| Transaction::new(x, self.clone()))
                        .collect()
                };
                Ok(Transactions::new(xs))
            }
            Err(err) => match err.status().unwrap().as_u16() {
                401 => {
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
