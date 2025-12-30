use chrono::{DateTime, FixedOffset, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::{
    client::Degiro,
    error::{ClientError, DataError},
    http::{HttpClient, HttpRequest},
    models::TransactionType,
    paths::{REPORTING_URL, TRANSACTIONS_PATH},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    #[serde(with = "rust_decimal::serde::float")]
    pub auto_fx_fee_in_base_currency: Decimal,
    #[serde(rename = "buysell")]
    pub transaction_type: TransactionType,
    pub counter_party: Option<String>,
    pub date: DateTime<FixedOffset>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub fee_in_base_currency: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float")]
    pub fx_rate: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gross_fx_rate: Decimal,
    pub id: i32,
    #[serde(with = "rust_decimal::serde::float")]
    pub nett_fx_rate: Decimal,
    pub order_type_id: Option<i8>,
    #[serde(with = "rust_decimal::serde::float")]
    pub price: Decimal,
    pub product_id: i32,
    /// Quantity is always reported as a positive number by Degiro; consumers should apply the
    /// correct sign based on `transaction_type` (e.g., subtract on sells).
    pub quantity: i32,
    #[serde(with = "rust_decimal::serde::float")]
    pub total: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_fees_in_base_currency: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_in_base_currency: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_plus_all_fees_in_base_currency: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_plus_fee_in_base_currency: Decimal,
    pub trading_venue: Option<String>,
    pub transaction_type_id: i32,
    pub transfered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transactions(Vec<Transaction>);

impl Transactions {
    /// Create a new Transactions collection
    pub fn new(transactions: Vec<Transaction>) -> Self {
        Self(transactions)
    }

    /// Get a reference to the transactions
    pub fn transactions(&self) -> &[Transaction] {
        &self.0
    }

    /// Get a mutable reference to the transactions
    pub fn transactions_mut(&mut self) -> &mut Vec<Transaction> {
        &mut self.0
    }

    /// Get the number of transactions
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Add a transaction to the collection
    pub fn push(&mut self, transaction: Transaction) {
        self.0.push(transaction);
    }

    /// Iterate over the transactions
    pub fn iter(&self) -> std::slice::Iter<'_, Transaction> {
        self.0.iter()
    }

    /// Iterate over the transactions mutably
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Transaction> {
        self.0.iter_mut()
    }

    /// Convert into the underlying Vec
    pub fn into_vec(self) -> Vec<Transaction> {
        self.0
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
        self.transactions_with_grouping(from_date, to_date, false)
            .await
    }

    pub async fn transactions_grouped(
        &self,
        from_date: impl Into<NaiveDate> + Send,
        to_date: impl Into<NaiveDate> + Send,
    ) -> Result<Transactions, ClientError> {
        self.transactions_with_grouping(from_date, to_date, true)
            .await
    }

    pub async fn transactions_with_grouping(
        &self,
        from_date: impl Into<NaiveDate> + Send,
        to_date: impl Into<NaiveDate> + Send,
        group_by_order: bool,
    ) -> Result<Transactions, ClientError> {
        let url = format!("{REPORTING_URL}{TRANSACTIONS_PATH}");

        let mut response_data = self
            .request::<HashMap<String, Vec<Transaction>>>(
                HttpRequest::get(url)
                    .query("sessionId", self.session_id())
                    .query("intAccount", self.int_account().to_string())
                    .query("fromDate", from_date.into().format("%d/%m/%Y").to_string())
                    .query("toDate", to_date.into().format("%d/%m/%Y").to_string())
                    .query(
                        "groupTransactionsByOrder",
                        if group_by_order { "1" } else { "0" },
                    ),
            )
            .await?;

        let transactions = response_data
            .remove("data")
            .ok_or_else(|| DataError::missing_field("data"))?;

        Ok(Transactions(transactions))
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;

    use crate::client::Degiro;

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn transactions() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");

        let transactions = client
            .transactions(
                NaiveDate::from_ymd_opt(2021, 1, 1).expect("Failed to create start date"),
                NaiveDate::from_ymd_opt(2022, 12, 31).expect("Failed to create end date"),
            )
            .await
            .expect("Failed to get transactions");
        dbg!(transactions);
    }
}
