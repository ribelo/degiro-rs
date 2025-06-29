use std::{
    fmt::{self, Debug},
    ops::{Deref, DerefMut},
};

use chrono::NaiveDate;
use derivative::Derivative;
use serde::{Deserialize, Serialize};

use rust_decimal::Decimal;

use crate::models::{AllowedOrderTypes, OrderTimeTypes};

use super::{risk::RiskCategory, CompanyProfile, Currency};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, strum::Display)]
pub enum Exchange {
    #[serde(rename = "663")]
    NSDQ,
    #[serde(rename = "676")]
    NSY,
    #[serde(rename = "200")]
    EAM,
    #[serde(rename = "194")]
    XET,
    #[serde(rename = "196")]
    TDG,
    #[serde(rename = "710")]
    EPA,
    #[serde(rename = "801")]
    WSE,
    #[serde(rename = "5001")]
    TSE,
    #[serde(rename = "520")]
    OSL,
    #[serde(rename = "947")]
    SWX,
    #[serde(rename = "860")]
    OMX,
    #[serde(rename = "219")]
    ATH,
    #[serde(rename = "650")]
    ASE,
    #[serde(rename = "893")]
    TSV,
    #[serde(rename = "5002")]
    ASX,
    #[serde(rename = "570")]
    LSE,
    #[serde(rename = "892")]
    TOR,
    #[serde(rename = "454")]
    HKS,
    Unknown,
}

impl From<Exchange> for Currency {
    fn from(exchange: Exchange) -> Self {
        match exchange {
            Exchange::NSY | Exchange::NSDQ => Currency::USD,
            Exchange::XET | Exchange::TDG | Exchange::EAM => Currency::EUR,
            Exchange::SWX => Currency::CHF,
            Exchange::TSE => Currency::JPY,
            Exchange::WSE => Currency::PLN,
            Exchange::LSE => Currency::GBP,
            _ => Currency::EUR,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Derivative, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    #[serde(default)]
    pub active: bool,
    pub buy_order_types: Option<AllowedOrderTypes>,
    pub category: RiskCategory,
    pub close_price: Decimal,
    pub close_price_date: NaiveDate,
    pub contract_size: Decimal,
    #[serde(rename = "exchangeId")]
    pub exchange: Exchange,
    pub feed_quality: Option<String>,
    pub feed_quality_secondary: Option<String>,
    pub id: String,
    pub isin: String,
    pub name: String,
    #[serde(default)]
    pub only_eod_prices: bool,
    pub order_book_depth: Option<i32>,
    pub order_book_depth_secondary: Option<i32>,
    pub order_time_types: Option<OrderTimeTypes>,
    pub product_bit_types: Option<Vec<String>>,
    pub product_type: String,
    pub product_type_id: i32,
    #[serde(default)]
    pub quality_switch_free: bool,
    #[serde(default)]
    pub quality_switch_free_secondary: bool,
    #[serde(default)]
    pub quality_switchable: bool,
    #[serde(default)]
    pub quality_switchable_secondary: bool,
    pub sell_order_types: Option<AllowedOrderTypes>,
    pub symbol: String,
    #[serde(default)]
    pub tradable: bool,
    pub vwd_id: Option<String>,
    pub vwd_id_secondary: Option<String>,
    pub vwd_identifier_type: Option<String>,
    pub vwd_identifier_type_secondary: Option<String>,
    pub vwd_module_id: Option<i32>,
    pub vwd_module_id_secondary: Option<i32>,
    pub company_profile: Option<CompanyProfile>,
}

impl Product {
    pub fn is_tradable(&self) -> bool {
        self.tradable && self.active
    }

    pub fn has_order_book(&self) -> bool {
        self.order_book_depth.is_some()
    }

    pub fn is_quality_switchable(&self) -> bool {
        self.quality_switchable || self.quality_switchable_secondary
    }

    pub fn is_eod_only(&self) -> bool {
        self.only_eod_prices
    }

    pub fn can_buy(&self) -> bool {
        self.buy_order_types.is_some()
    }

    pub fn can_sell(&self) -> bool {
        self.sell_order_types.is_some()
    }

    pub fn has_secondary_feed(&self) -> bool {
        self.feed_quality_secondary.is_some()
    }

    pub fn full_name(&self) -> String {
        format!("{} ({})", self.name, self.symbol)
    }
}

#[derive(Debug, Default)]
pub struct Products(pub Vec<Product>);

impl Deref for Products {
    type Target = Vec<Product>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Products {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Product {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Helper functions to reduce duplication
        let format_opt_str = |val: &Option<String>| val.as_deref().unwrap_or("N/A").to_string();
        let format_opt_num = |val: &Option<i32>| val.map_or("N/A".to_string(), |v| v.to_string());

        writeln!(f, "Product: {} ({})", self.name, self.symbol)?;
        writeln!(f, "ID: {} | ISIN: {}", self.id, self.isin)?;
        writeln!(f, "Active: {} | Category: {}", self.active, self.category)?;
        writeln!(f, "Exchange ID: {}", self.exchange)?;
        writeln!(
            f,
            "Prices: {:.4} ({})",
            self.close_price, self.close_price_date
        )?;
        writeln!(f, "Contract Size: {:.4}", self.contract_size)?;
        writeln!(
            f,
            "Feed Quality: {} | Secondary: {}",
            format_opt_str(&self.feed_quality),
            format_opt_str(&self.feed_quality_secondary)
        )?;
        writeln!(f, "Only EOD Prices: {}", self.only_eod_prices)?;
        writeln!(
            f,
            "Order Book Depth: {} | Secondary: {}",
            self.order_book_depth.unwrap_or(-1),
            format_opt_num(&self.order_book_depth_secondary)
        )?;
        writeln!(f, "Order Time Types: {:?}", self.order_time_types)?;
        writeln!(
            f,
            "Product: {} (Type ID: {})",
            self.product_type, self.product_type_id
        )?;
        writeln!(f, "Product Bit Types: {:?}", self.product_bit_types)?;
        writeln!(f, "Quality Settings:")?;
        writeln!(
            f,
            "  Switch Free: {} | Secondary: {}",
            self.quality_switch_free, self.quality_switch_free_secondary
        )?;
        writeln!(
            f,
            "  Switchable: {} | Secondary: {}",
            self.quality_switchable, self.quality_switchable_secondary
        )?;
        writeln!(
            f,
            "Order Types: Buy={:?}, Sell={:?}",
            self.buy_order_types, self.sell_order_types
        )?;
        writeln!(f, "Tradable: {}", self.tradable)?;
        writeln!(f, "VWD Details:")?;
        writeln!(
            f,
            "  ID: {} | Secondary: {}",
            format_opt_str(&self.vwd_id),
            format_opt_str(&self.vwd_id_secondary)
        )?;
        writeln!(
            f,
            "  Identifier: {} | Secondary: {}",
            format_opt_str(&self.vwd_identifier_type),
            format_opt_str(&self.vwd_identifier_type_secondary)
        )?;
        writeln!(
            f,
            "  Module ID: {} | Secondary: {}",
            format_opt_num(&self.vwd_module_id),
            format_opt_num(&self.vwd_module_id_secondary)
        )?;
        Ok(())
    }
}
