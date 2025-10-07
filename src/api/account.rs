use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use futures_concurrency::future::TryJoin;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

use crate::{
    client::Degiro,
    error::{ClientError, DataError},
    http::{HttpClient, HttpRequest},
    models::{
        AccountConfig, AccountData, AccountInfo, AccountState, CashMovement, CashMovementType,
        Money, Period,
    },
    paths::{
        ACCOUNT_CONFIG_PATH, ACCOUNT_INFO_PATH, ACCOUNT_OVERVIEW_PATH, BASE_API_URL,
        CASH_ACCOUNT_REPORT_URL, PA_URL, REPORTING_URL,
    },
    session::AuthState,
};

impl Degiro {
    pub(crate) async fn account_config(&self) -> Result<(), ClientError> {
        let url = format!("{BASE_API_URL}{ACCOUNT_CONFIG_PATH}");

        let mut response_data: HashMap<String, AccountConfig> = self
            .request(HttpRequest::get(url).require_restricted())
            .await?;

        let account_config = response_data
            .remove("data")
            .ok_or_else(|| DataError::missing_field("data"))?;

        // Update client state
        self.set_client_id(account_config.client_id);
        self.set_account_config(account_config);

        // Get additional account data before setting Authorized state
        let account_data = self.account_data().await?;
        self.set_int_account(account_data.int_account);

        // Only set Authorized state after we've successfully gotten all data
        self.set_auth_state(AuthState::Authorized)?;

        // Save session after successful full authorization
        if let Err(e) = self.save_session() {
            // Don't fail the auth if we can't save the session
            tracing::warn!("Failed to save session after full authorization: {}", e);
        }

        Ok(())
    }
}

impl Degiro {
    pub async fn account_data(&self) -> Result<AccountData, ClientError> {
        let url = format!("{PA_URL}client");

        let response_data: HashMap<String, AccountData> = self
            .request(
                HttpRequest::get(url)
                    .require_restricted() // Only needs login, not full auth
                    .query("sessionId", self.session_id()),
            )
            .await?;

        response_data
            .get("data")
            .cloned()
            .ok_or_else(|| DataError::missing_field("data").into())
    }
}

impl Degiro {
    pub async fn account_info(&self) -> Result<AccountInfo, ClientError> {
        let url = self.build_trading_url(ACCOUNT_INFO_PATH)?;

        let mut response_data: HashMap<String, AccountInfo> = self
            .request(HttpRequest::get(url).query("sessionId", self.session_id()))
            .await?;

        let account_info = response_data
            .remove("data")
            .ok_or_else(|| DataError::missing_field("data"))?;

        // Extract and store currency rates in session
        let currency_rates: HashMap<String, Decimal> = account_info
            .currency_pairs
            .iter()
            .map(|(pair_name, currency_pair)| (pair_name.clone(), currency_pair.price))
            .collect();

        self.session.set_currency_rates(currency_rates);

        Ok(account_info)
    }
}

impl Degiro {
    pub async fn account_state(
        &self,
        from_date: &NaiveDate,
        to_date: &NaiveDate,
    ) -> Result<AccountState, ClientError> {
        let url = format!("{REPORTING_URL}{ACCOUNT_OVERVIEW_PATH}");

        let mut body: HashMap<String, HashMap<String, Vec<CashMovement>>> = self
            .request(
                HttpRequest::get(url)
                    .query("sessionId", self.session_id())
                    .query("intAccount", self.int_account().to_string())
                    .query("fromDate", from_date.format("%d/%m/%Y").to_string())
                    .query("toDate", to_date.format("%d/%m/%Y").to_string()),
            )
            .await?;

        let mut data = body
            .remove("data")
            .ok_or_else(|| DataError::missing_field("data"))?;

        let state = data
            .remove("cashMovements")
            .ok_or_else(|| DataError::missing_field("cashMovements"))?;

        Ok(AccountState::new(state))
    }

    pub async fn balance(&self) -> Result<Money, ClientError> {
        self.ensure_authorized().await?;

        let to_date = Utc::now().date_naive();
        let from_date = Period::P3Y.subtract_from_date(to_date);
        let (account_info, account_state, portfolio_value) = (
            self.account_info(),
            self.account_state(&from_date, &to_date),
            self.total_portfolio_value(),
        )
            .try_join()
            .await?;

        let current_balance = account_state
            .iter()
            .find(|movement| {
                matches!(movement.movement_type, CashMovementType::FxCredit(..))
                    && movement.currency == account_info.base_currency
            })
            .and_then(|movement| {
                movement
                    .balance
                    .as_ref()
                    .and_then(|balance| Decimal::from_f64(balance.total))
            })
            .unwrap_or(Decimal::ZERO);

        let total_value = portfolio_value + current_balance;

        Ok(Money::new(account_info.base_currency, total_value))
    }
}

impl Degiro {
    pub async fn cash_report(
        &self,
        from_date: &NaiveDate,
        to_date: &NaiveDate,
    ) -> Result<String, ClientError> {
        let url = format!("{CASH_ACCOUNT_REPORT_URL}/csv");

        self.request_text(
            HttpRequest::get(url)
                .query("sessionId", self.session_id())
                .query("intAccount", self.int_account().to_string())
                .query("country", "PL")
                .query("lang", "pl")
                .query("fromDate", from_date.format("%d/%m/%Y").to_string())
                .query("toDate", to_date.format("%d/%m/%Y").to_string()),
        )
        .await
    }
}

#[cfg(test)]
mod test {
    // use futures_concurrency::future::TryJoin;

    use super::*;

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_account_data() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");
    }

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_account_info() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        let info = client
            .account_info()
            .await
            .expect("Failed to get account info");
        dbg!(info);
    }

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_account_state() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");
        let state = client
            .account_state(
                &NaiveDate::from_ymd_opt(2024, 11, 1).expect("Failed to create start date"),
                &NaiveDate::from_ymd_opt(2024, 12, 31).expect("Failed to create end date"),
            )
            .await
            .expect("Failed to get account state");
        dbg!(state);
    }

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_cash_report() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");
        let report = client
            .cash_report(
                &NaiveDate::from_ymd_opt(2024, 10, 1).expect("Failed to create start date"),
                &NaiveDate::from_ymd_opt(2024, 11, 28).expect("Failed to create end date"),
            )
            .await
            .expect("Failed to get cash report");
        dbg!(report);
    }

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_balance() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");
        let balance = client
            .balance()
            .await
            .expect("Failed to get account balance");
        dbg!(balance);
    }
}
