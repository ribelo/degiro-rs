use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

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