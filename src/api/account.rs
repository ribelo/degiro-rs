use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use futures_concurrency::future::Join;

use reqwest::{header, Url};

use crate::{
    client::{ApiErrorResponse, ClientError, ClientStatus, Degiro},
    models::{
        AccountConfig, AccountData, AccountInfo, AccountState, CashMovement, CashMovementType,
        Currency, Money, Period,
    },
    paths::{
        ACCOUNT_CONFIG_PATH, ACCOUNT_INFO_PATH, ACCOUNT_OVERVIEW_PATH, BASE_API_URL,
        CASH_ACCOUNT_REPORT_URL, PA_URL, REFERER, REPORTING_URL, TRADING_URL,
    },
};

impl Degiro {
    pub(crate) async fn account_config(&self) -> Result<(), ClientError> {
        if !self.is_authorized() {
            self.login().await?;
        }

        let url = Url::parse(BASE_API_URL)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join(ACCOUNT_CONFIG_PATH)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

        let req = self.http_client.get(url).header(header::REFERER, REFERER);

        self.acquire_limit().await;

        let res = req.send().await?;

        if let Err(err) = res.error_for_status_ref() {
            if err.status().unwrap().as_u16() == 401 {
                self.set_auth_state(ClientStatus::Unauthorized);
                return Err(ClientError::Unauthorized);
            }

            let error_response = res.json::<ApiErrorResponse>().await.map_err(|e| {
                ClientError::UnexpectedError(format!("Failed to parse error response: {}", e))
            })?;
            return Err(ClientError::ApiError(error_response));
        }

        let mut response_data: HashMap<String, AccountConfig> = res.json().await?;

        let account_config = response_data
            .remove("data")
            .ok_or_else(|| ClientError::UnexpectedError("Missing data key".into()))?;

        // Update client state
        self.set_client_id(account_config.client_id);
        self.set_account_config(account_config);
        self.set_auth_state(ClientStatus::Authorized);

        // Get additional account data
        let account_data = self.account_data().await?;
        self.set_int_account(account_data.int_account);

        Ok(())
    }
}

impl Degiro {
    pub async fn account_data(&self) -> Result<AccountData, ClientError> {
        if !self.is_authorized() {
            self.login().await?;
        }

        let url = Url::parse(PA_URL)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join("client")
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

        let req = self
            .http_client
            .get(url)
            .query(&[("sessionId", self.session_id())])
            .header(header::REFERER, REFERER);

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

        let response_data: HashMap<String, AccountData> = res.json().await?;

        response_data
            .get("data")
            .cloned()
            .ok_or_else(|| ClientError::UnexpectedError("Missing data key".into()))
    }
}

impl Degiro {
    pub async fn account_info(&self) -> Result<AccountInfo, ClientError> {
        self.ensure_authorized().await?;

        let url = Url::parse(TRADING_URL)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join(ACCOUNT_INFO_PATH)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join(&format!(
                "{};jsessionid={}",
                self.int_account(),
                self.session_id()
            ))
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

        let req = self
            .http_client
            .get(url)
            .query(&[("sessionId", self.session_id())])
            .header(header::REFERER, REFERER);

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

        let mut response_data: HashMap<String, AccountInfo> = res.json().await?;

        response_data
            .remove("data")
            .ok_or_else(|| ClientError::UnexpectedError("Missing data key".into()))
    }
}

impl Degiro {
    pub async fn account_state(
        &self,
        from_date: &NaiveDate,
        to_date: &NaiveDate,
    ) -> Result<AccountState, ClientError> {
        self.ensure_authorized().await?;

        let req = {
            let url = Url::parse(REPORTING_URL)
                .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
                .join(ACCOUNT_OVERVIEW_PATH)
                .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

            self.http_client
                .get(url)
                .query(&[
                    ("sessionId", self.session_id()),
                    ("intAccount", self.int_account().to_string()),
                    ("fromDate", from_date.format("%d/%m/%Y").to_string()),
                    ("toDate", to_date.format("%d/%m/%Y").to_string()),
                ])
                .header(header::REFERER, REFERER)
        };

        self.acquire_limit().await;

        let res = req.send().await?;

        if let Err(err) = res.error_for_status_ref() {
            if err.status().unwrap().as_u16() == 401 {
                self.set_auth_state(ClientStatus::Unauthorized);
                return Err(ClientError::Unauthorized);
            }

            let error_response = res.json::<ApiErrorResponse>().await.map_err(|e| {
                ClientError::UnexpectedError(format!("Failed to parse error response: {}", e))
            })?;
            return Err(ClientError::ApiError(error_response));
        }

        let mut body: HashMap<String, HashMap<String, Vec<CashMovement>>> = res.json().await?;

        let mut data = body
            .remove("data")
            .ok_or_else(|| ClientError::UnexpectedError("Missing data key".into()))?;

        let state = data
            .remove("cashMovements")
            .ok_or_else(|| ClientError::UnexpectedError("Missing cashMovements key".to_string()))?;

        Ok(AccountState(state))
    }

    pub async fn balance(&self) -> Result<Money, ClientError> {
        self.ensure_authorized().await?;

        let to_date = Utc::now().date_naive();
        let from_date = to_date - Period::P3Y;
        let (account_info, account_state, portfolio_value) = (
            self.account_info(),
            self.account_state(&from_date, &to_date),
            self.total_portfolio_value(),
        )
            .join()
            .await;
        let account_info = account_info?;
        let account_state = account_state?;
        let portfolio_value = portfolio_value?;

        let current_balance = account_state
            .iter()
            .find(|movement| {
                matches!(movement.movement_type, CashMovementType::FxCredit(..))
                    && movement.currency == account_info.base_currency
            })
            .map(|movement| movement.balance.total)
            .unwrap();

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
        self.ensure_authorized().await?;

        let url = Url::parse(CASH_ACCOUNT_REPORT_URL)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join("csv")
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

        let req = self
            .http_client
            .get(url)
            .query(&[
                ("sessionId", self.session_id()),
                ("intAccount", self.int_account().to_string()),
                ("country", "PL".to_string()),
                ("lang", "pl".to_string()),
                ("fromDate", from_date.format("%d/%m/%Y").to_string()),
                ("toDate", to_date.format("%d/%m/%Y").to_string()),
            ])
            .header(header::REFERER, REFERER);

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

        let body = res
            .text()
            .await
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

        Ok(body)
    }
}

#[cfg(test)]
mod test {
    use futures_concurrency::future::TryJoin;

    use super::*;

    #[tokio::test]
    async fn test_account_data() {
        let client = Degiro::new_from_env();
        // client.login().await.unwrap();
        client.account_config().await.unwrap();
        dbg!(&client.account_config);
        // let data = client.account_data().await.unwrap();
        // dbg!(data);
    }

    #[tokio::test]
    async fn test_account_info() {
        let client = Degiro::new_from_env();
        let info = client.account_info().await.unwrap();
        dbg!(info);
    }

    // #[tokio::test]
    // async fn test_account_info_and_data() {
    //     let client = Degiro::new_from_env();
    //     let (account_info, account_data) = match (client.account_info(), client.account_data())
    //         .try_join()
    //         .await
    //     {
    //         Ok((account_info, account_data)) => (account_info, account_data),
    //         Err(e) => {
    //             panic!("Failed to fetch account info: {e}")
    //         }
    //     };
    // }

    #[tokio::test]
    async fn test_account_state() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        // dbg!(&client);
        let state = client
            .account_state(
                &NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(),
                &NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            )
            .await
            .unwrap();
        dbg!(state);
    }

    #[tokio::test]
    async fn test_cash_report() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let report = client
            .cash_report(
                &NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
                &NaiveDate::from_ymd_opt(2024, 11, 28).unwrap(),
            )
            .await
            .unwrap();
        dbg!(report);
    }

    #[tokio::test]
    async fn test_balance() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let balance = client.balance().await.unwrap();
        dbg!(balance);
    }
}
