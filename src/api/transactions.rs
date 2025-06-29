use chrono::{DateTime, FixedOffset, NaiveDate};
use reqwest::{header, Url};
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{
    client::{ApiErrorResponse, ClientError, ClientStatus, Degiro},
    models::TransactionType,
    paths::REPORTING_URL,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transactions(pub Vec<Transaction>);

impl Deref for Transactions {
    type Target = Vec<Transaction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Transactions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Transactions {
    type Item = Transaction;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Degiro {
    pub async fn transactions(
        &self,
        from_date: impl Into<NaiveDate> + Send,
        to_date: impl Into<NaiveDate> + Send,
    ) -> Result<Transactions, ClientError> {
        self.ensure_authorized().await?;

        let url = Url::parse(REPORTING_URL)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join(crate::paths::TRANSACTIONS_PATH)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

        let req = self
            .http_client
            .get(url)
            .query(&[
                ("sessionId", self.session_id()),
                ("intAccount", self.int_account().to_string()),
                ("fromDate", from_date.into().format("%d/%m/%Y").to_string()),
                ("toDate", to_date.into().format("%d/%m/%Y").to_string()),
                ("groupTransactionsByOrder", "1".to_string()),
            ])
            .header(header::REFERER, crate::paths::REFERER);

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

        let mut response_data = res.json::<HashMap<String, Vec<Transaction>>>().await?;

        let transactions = response_data
            .remove("data")
            .ok_or_else(|| ClientError::UnexpectedError("Missing data key".into()))?;

        Ok(Transactions(transactions))
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;

    use crate::client::Degiro;

    #[tokio::test]
    async fn transactions() {
        let client = Degiro::new_from_env();
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
