use chrono::{DateTime, FixedOffset, NaiveDate};
use std::collections::HashMap;

use reqwest::{header, Url};
use serde::Deserialize;

use crate::client::{Client, ClientError, ClientStatus};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub address: Address,
    pub bank_account: BankAccount,
    pub can_upgrade: bool,
    pub cellphone_number: String,
    pub client_role: String,
    pub contract_type: String,
    pub culture: String,
    pub display_language: String,
    pub display_name: String,
    pub effective_client_role: String,
    pub email: String,
    pub first_contact: FirstContact,
    pub flatex_bank_account: FlatexBankAccount,
    pub id: i32,
    pub int_account: i32,
    pub is_allocation_available: bool,
    pub is_am_client_active: bool,
    pub is_collective_portfolio: bool,
    pub is_isk_client: bool,
    pub is_withdrawal_available: bool,
    pub language: String,
    pub locale: String,
    pub logged_in_person_id: i32,
    pub member_code: String,
    pub username: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub base_currency: String,
    pub margin_type: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub city: String,
    pub country: String,
    pub street_address: String,
    pub street_address_ext: String,
    pub street_address_number: String,
    pub zip: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccount {
    pub bank_account_id: i32,
    pub bic: String,
    pub iban: String,
    pub status: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstContact {
    pub country_of_birth: String,
    pub date_of_birth: String,
    pub display_name: String,
    pub first_name: String,
    pub gender: String,
    pub last_name: String,
    pub nationality: String,
    pub place_of_birth: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize)]
pub struct FlatexBankAccount {
    pub bic: String,
    pub iban: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountConfig {
    pub allocations_url: String,
    pub beta_landing_path: String,
    pub client_id: i32,
    pub companies_service_url: String,
    pub dictionary_url: String,
    pub exante_reporting_url: String,
    pub favorites_url: String,
    pub feedback_url: String,
    pub i18n_url: String,
    pub landing_path: String,
    pub latest_searched_products_url: String,
    pub login_url: String,
    pub mobile_landing_path: String,
    pub pa_url: String,
    pub payment_service_url: String,
    pub product_notes_url: String,
    pub product_search_url: String,
    pub product_search_v2_url: String,
    pub product_types_url: String,
    pub refinitiv_agenda_url: String,
    pub refinitiv_clips_url: String,
    pub refinitiv_company_profile_url: String,
    pub refinitiv_company_ratios_url: String,
    pub refinitiv_esgs_url: String,
    pub refinitiv_estimates_url: String,
    pub refinitiv_financial_statements_url: String,
    pub refinitiv_insider_transactions_url: String,
    pub refinitiv_insiders_report_url: String,
    pub refinitiv_investor_url: String,
    pub refinitiv_news_url: String,
    pub refinitiv_shareholders_url: String,
    pub refinitiv_top_news_categories_url: String,
    pub reporting_url: String,
    pub session_id: String,
    pub settings_url: String,
    pub task_manager_url: String,
    pub trading_url: String,
    pub translations_url: String,
    pub vwd_chart_api_url: String,
    pub vwd_gossips_url: String,
    pub vwd_news_url: String,
    pub vwd_quotecast_service_url: String,
}

impl Client {
    pub async fn account_config(&self) -> Result<(), ClientError> {
        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = "https://trader.degiro.nl/";
            let path_url = "login/secure/config";
            let url = Url::parse(base_url)
                .unwrap_or_else(|_| panic!("can't parse base_url: {base_url}"))
                .join(path_url)
                .unwrap_or_else(|_| panic!("can't join path_url: {path_url}"));

            inner
                .http_client
                .get(url)
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
                let mut body = res
                    .json::<HashMap<String, AccountConfig>>()
                    .await
                    .expect("can't parse json data");
                let data = body.remove("data").expect("data key not found");
                {
                    let mut inner = self.inner.lock().unwrap();
                    inner.client_id = data.client_id;
                    inner.account_config = data;
                    inner.status = ClientStatus::Authorized;
                };
                let account_data = self.account_data().await.unwrap();
                {
                    let mut inner = self.inner.lock().unwrap();
                    inner.int_account = account_data.int_account;
                }
                Ok(())
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

impl Client {
    pub async fn account_data(&self) -> Result<AccountData, ClientError> {
        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = &inner.account_config.pa_url;
            let url = Url::parse(base_url)
                .unwrap_or_else(|_| panic!("can't parse base_url: {base_url}"))
                .join("client")
                .unwrap();

            inner
                .http_client
                .get(url)
                .query(&[("sessionId", &inner.session_id)])
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
                let mut body = res
                    .json::<HashMap<String, AccountData>>()
                    .await
                    .expect("can't parse json data");
                let account = body.remove("data").expect("data key not found");

                Ok(account)
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

impl Client {
    pub async fn account_info(&self) -> Result<AccountInfo, ClientError> {
        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = &inner.account_config.trading_url;
            let url = Url::parse(base_url)
                .unwrap()
                .join("v5/account/info/")
                .unwrap()
                .join(&format!(
                    "{};jsessionid={}",
                    &inner.int_account, &inner.session_id
                ))
                .unwrap();
            inner
                .http_client
                .get(url)
                .query(&[("sessionId", &inner.session_id)])
                .header(header::REFERER, &inner.referer)
        };

        let res = req
            .send()
            .await
            .map_err(|err| ClientError::UnexpectedError {
                source: Box::new(err),
            })?;

        match res.error_for_status() {
            Ok(res) => {
                let mut body = res
                    .json::<HashMap<String, AccountInfo>>()
                    .await
                    .expect("can't parse json data");
                let info = body.remove("data").expect("data key not found");
                Ok(info)
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

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashMovement {
    balance: Balance,
    change: f64,
    currency: String,
    date: DateTime<FixedOffset>,
    #[serde(rename = "description")]
    movement_type: CashMovementType,
    id: i32,
    order_id: Option<String>,
    product_id: Option<i32>,
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    value_date: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize)]
#[serde(from = "String")]
pub enum CashMovementType {
    Dividend(String),
    FxWithdrawal(String),
    DividentFee(String),
    FxCredit(String),
    Interest(String),
    BankWithdrawal(String),
    Deposit(String),
    TransactionFee(String),
    TransactionSell(String),
    TransactionBuy(String),
    UnknownFee(String),
    UnknownInteres(String),
    Unknown(String),
}

#[derive(Debug, Deserialize)]
pub enum TransactionType {
    #[serde(rename = "CASH_TRANSACTION")]
    Cash,
    #[serde(rename = "TRANSACTION")]
    NoCash,
    #[serde(rename = "CASH_FUND_TRANSACTION")]
    Fund,
    #[serde(rename = "PAYMENT")]
    Payment,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    cash_fund: Option<Vec<CashFund>>,
    total: f64,
    unsettled_cash: f64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CashFund {
    id: i32,
    participation: f64,
    price: f64,
}

pub struct ParseMovementTypeError;

impl From<String> for CashMovementType {
    fn from(s: String) -> Self {
        if s == "Dywidenda" {
            CashMovementType::Dividend(s)
        } else if s == "FX Withdrawal" {
            CashMovementType::FxWithdrawal(s)
        } else if s == "Podatek Dywidendowy" {
            CashMovementType::DividentFee(s)
        } else if s == "FX Credit" {
            CashMovementType::FxCredit(s)
        } else if s == "Odsetki" {
            CashMovementType::Interest(s)
        } else if s == "Wypłata" {
            CashMovementType::BankWithdrawal(s)
        } else if s == "Depozyt" {
            CashMovementType::Deposit(s)
        } else if s.to_lowercase().contains("opłata transakcyjna") {
            CashMovementType::TransactionFee(s)
        } else if s.to_lowercase().contains("sprzedaż") {
            CashMovementType::TransactionSell(s)
        } else if s.to_lowercase().contains("kupno") {
            CashMovementType::TransactionBuy(s)
        } else if s.to_lowercase().contains("fee") {
            CashMovementType::UnknownFee(s)
        } else if s.to_lowercase().contains("interest") {
            CashMovementType::UnknownInteres(s)
        } else {
            CashMovementType::Unknown(s)
        }
    }
}

#[derive(Debug)]
pub struct AccountState(Vec<CashMovement>);

impl Client {
    pub async fn account_state(
        &self,
        from_date: &NaiveDate,
        to_date: &NaiveDate,
    ) -> Result<AccountState, ClientError> {
        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = &inner.account_config.reporting_url;
            let url = Url::parse(base_url)
                .unwrap()
                .join("v6/accountoverview")
                .unwrap();
            inner
                .http_client
                .get(url)
                .query(&[
                    ("sessionId", &inner.session_id),
                    ("intAccount", &format!("{}", inner.int_account)),
                    ("fromDate", &from_date.format("%d/%m/%Y").to_string()),
                    ("toDate", &to_date.format("%d/%m/%Y").to_string()),
                ])
                .header(header::REFERER, &inner.referer)
        };

        let res = req
            .send()
            .await
            // TODO:
            .map_err(|err| ClientError::UnexpectedError {
                source: Box::new(err),
            })?;

        match res.error_for_status() {
            Ok(res) => {
                let mut body = res
                    .json::<HashMap<String, HashMap<String, Vec<CashMovement>>>>()
                    .await
                    .unwrap();
                let mut data = body.remove("data").unwrap();
                let state = data.remove("cashMovements").unwrap();
                Ok(AccountState(state))
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
    use super::*;

    #[tokio::test]
    async fn account_data() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        dbg!(&client);
        let data = client.account_data().await.unwrap();
        dbg!(data);
    }

    #[tokio::test]
    async fn account_info() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        dbg!(&client);
        let info = client.account_info().await.unwrap();
        dbg!(info);
    }

    #[tokio::test]
    async fn account_state() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        dbg!(&client);
        let state = client
            .account_state(
                &NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                &NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(),
            )
            .await
            .unwrap();
        dbg!(state);
    }
}
