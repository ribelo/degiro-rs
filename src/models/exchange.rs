use serde::{Deserialize, Serialize};

use super::Currency;

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