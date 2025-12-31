use serde::{Deserialize, Deserializer, Serialize};

use super::Currency;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq, strum::Display)]
pub enum Exchange {
    NSDQ,
    NSY,
    EAM,
    XET,
    TDG,
    EPA,
    WSE,
    TSE,
    OSL,
    SWX,
    OMX,
    ATH,
    ASE,
    TSV,
    ASX,
    LSE,
    TOR,
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

impl<'de> Deserialize<'de> for Exchange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let value = match raw.as_str() {
            "663" | "NSDQ" => Exchange::NSDQ,
            "676" | "NSY" => Exchange::NSY,
            "200" | "EAM" => Exchange::EAM,
            "194" | "XET" => Exchange::XET,
            "196" | "TDG" => Exchange::TDG,
            "710" | "EPA" => Exchange::EPA,
            "801" | "WSE" => Exchange::WSE,
            "5001" | "TSE" => Exchange::TSE,
            "520" | "OSL" => Exchange::OSL,
            "947" | "SWX" => Exchange::SWX,
            "860" | "OMX" => Exchange::OMX,
            "219" | "ATH" => Exchange::ATH,
            "650" | "ASE" => Exchange::ASE,
            "893" | "TSV" => Exchange::TSV,
            "5002" | "ASX" => Exchange::ASX,
            "570" | "LSE" => Exchange::LSE,
            "892" | "TOR" => Exchange::TOR,
            "454" | "HKS" => Exchange::HKS,
            other => {
                tracing::warn!(
                    exchange_id = other,
                    "Unknown exchange id; defaulting to Unknown"
                );
                Exchange::Unknown
            }
        };
        Ok(value)
    }
}
