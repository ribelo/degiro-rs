use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Default, Deserialize, Clone, Copy, Serialize, PartialEq, EnumString, Display)]
pub enum TransactionType {
    #[default]
    #[serde(rename = "B", alias = "BUY")]
    Buy,
    #[serde(rename = "S", alias = "SELL")]
    Sell,
}
