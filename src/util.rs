use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::{self, Display, EnumString};

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Display, PartialEq, Eq, PartialOrd, Serialize,
)]
#[strum(ascii_case_insensitive)]
pub enum ProductCategory {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProductType {
    Stock,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
    Unknown(i32),
}

impl FromStr for Exchange {
    type Err = strum::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let x = s.parse::<i32>().unwrap();
        Ok(x.into())
    }
}

impl From<i32> for Exchange {
    fn from(x: i32) -> Self {
        match x {
            663 => Self::NSDQ,
            676 => Self::NSY,
            200 => Self::EAM,
            194 => Self::XET,
            196 => Self::TDG,
            710 => Self::EPA,
            801 => Self::WSE,
            5001 => Self::TSE,
            520 => Self::OSL,
            947 => Self::SWX,
            860 => Self::OMX,
            219 => Self::ATH,
            650 => Self::ASE,
            893 => Self::TSV,
            5002 => Self::ASX,
            570 => Self::LSE,
            892 => Self::TOR,
            454 => Self::HKS,
            _ => Self::Unknown(x),
        }
    }
}

impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NSDQ => write!(f, "NSDQ"),
            Self::NSY => write!(f, "NSY"),
            Self::EAM => write!(f, "EAM"),
            Self::XET => write!(f, "XET"),
            Self::TDG => write!(f, "TDG"),
            Self::EPA => write!(f, "EPA"),
            Self::WSE => write!(f, "WSE"),
            Self::TSE => write!(f, "TSE"),
            Self::OSL => write!(f, "OSL"),
            Self::SWX => write!(f, "SWX"),
            Self::OMX => write!(f, "OMX"),
            Self::ATH => write!(f, "ATH"),
            Self::ASE => write!(f, "ASE"),
            Self::TSV => write!(f, "TSV"),
            Self::ASX => write!(f, "ASX"),
            Self::LSE => write!(f, "LSE"),
            Self::TOR => write!(f, "TOR"),
            Self::HKS => write!(f, "HKS"),
            Self::Unknown(x) => write!(f, "Unknown({})", x),
        }
    }
}
