use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

use super::Currency;

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
pub struct CurrencyPair {
    pub id: i32,
    #[serde(deserialize_with = "string_to_decimal")]
    pub price: Decimal,
}

fn string_to_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    Decimal::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub base_currency: Currency,
    pub margin_type: String,
    pub currency_pairs: HashMap<String, CurrencyPair>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub city: Option<String>,
    pub country: Option<String>,
    pub street_address: Option<String>,
    pub street_address_ext: String,
    pub street_address_number: String,
    pub zip: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccount {
    pub bank_account_id: i32,
    pub bic: String,
    pub iban: String,
    pub status: String,
}

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

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FlatexBankAccount {
    pub bic: String,
    pub iban: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct CashMovement {
    pub balance: Balance,
    pub change: Option<f64>,
    pub currency: Currency,
    pub date: DateTime<FixedOffset>,
    #[serde(rename = "description")]
    pub movement_type: CashMovementType,
    pub id: u64,
    pub order_id: Option<String>,
    pub product_id: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: AccountTransactionType,
    pub value_date: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
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
pub enum AccountTransactionType {
    #[serde(rename = "CASH_TRANSACTION")]
    Cash,
    #[serde(rename = "TRANSACTION")]
    NoCash,
    #[serde(rename = "CASH_FUND_TRANSACTION")]
    Fund,
    #[serde(rename = "PAYMENT")]
    Payment,
    #[serde(rename = "FLATEX_CASH_SWEEP")]
    FlatexCashSweep,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub cash_fund: Option<Vec<CashFund>>,
    pub total: Decimal,
    pub unsettled_cash: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CashFund {
    pub id: i32,
    pub participation: f64,
    pub price: f64,
}

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

impl AccountState {
    /// Create a new AccountState
    pub fn new(movements: Vec<CashMovement>) -> Self {
        Self(movements)
    }

    /// Get a reference to the cash movements
    pub fn movements(&self) -> &[CashMovement] {
        &self.0
    }

    /// Get a mutable reference to the cash movements
    pub fn movements_mut(&mut self) -> &mut Vec<CashMovement> {
        &mut self.0
    }

    /// Get the number of movements
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if there are no movements
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Add a movement to the account state
    pub fn push(&mut self, movement: CashMovement) {
        self.0.push(movement);
    }

    /// Iterate over the movements
    pub fn iter(&self) -> std::slice::Iter<'_, CashMovement> {
        self.0.iter()
    }

    /// Convert into the underlying Vec
    pub fn into_vec(self) -> Vec<CashMovement> {
        self.0
    }
}
