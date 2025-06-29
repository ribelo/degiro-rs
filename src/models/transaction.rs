use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Default, Deserialize, Clone, Copy, Serialize, PartialEq, EnumString, Display)]
pub enum TransactionType {
    #[default]
    #[serde(rename(deserialize = "B", serialize = "BUY"))]
    Buy,
    #[serde(rename(deserialize = "S", serialize = "SELL"))]
    Sell,
}
