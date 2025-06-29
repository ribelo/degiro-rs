use derive_more::derive::Deref;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use super::{Position, Product};

pub const CURRENCY_RISK: Decimal = dec!(0.0636);

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Display, PartialEq, Eq, Hash, PartialOrd, Serialize,
)]
#[strum(ascii_case_insensitive)]
pub enum RiskCategory {
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
    NoCategory,
}

impl RiskCategory {
    pub fn risk(&self) -> Decimal {
        match self {
            RiskCategory::A => dec!(0.6250),
            RiskCategory::B => dec!(0.8125),
            RiskCategory::C => dec!(0.9900),
            RiskCategory::D => dec!(1.0000),
            RiskCategory::E => dec!(0.0625),
            RiskCategory::F => dec!(0.1250),
            RiskCategory::G => dec!(0.1875),
            RiskCategory::H => dec!(0.2500),
            RiskCategory::I => dec!(0.3125),
            RiskCategory::J => dec!(1.0000),
            RiskCategory::NoCategory => dec!(1.0000),
        }
    }
}

#[derive(Debug, EnumString, Display)]
pub enum Profile {
    Trader,
    Active,
}

// pub struct RiskCalculator {
//     risk_table: [(RiskCategory, Decimal, Decimal); 11],
//     active_risk_table: [(RiskCategory, Decimal, Decimal); 11]
// }

// impl Default for RiskCalculator {
//     fn default() -> Self {
//         Self {
//             risk_table: [
//                 (RiskCategory::A, dec!(62.50), dec!(62.50)),
//                 (RiskCategory::B, dec!(81.25), dec!(125.00)),
//                 (RiskCategory::C, dec!(99.00), dec!(250.00)),
//                 (RiskCategory::D, dec!(100.00), dec!(375.00)),
//                 (RiskCategory::E, dec!(6.25), dec!(6.25)),
//                 (RiskCategory::F, dec!(12.50), dec!(12.50)),
//                 (RiskCategory::G, dec!(18.75), dec!(18.75)),
//                 (RiskCategory::H, dec!(25.00), dec!(25.00)),
//                 (RiskCategory::I, dec!(31.25), dec!(31.25)),
//                 (RiskCategory::J, dec!(100.00), dec!(375.00)),
//                 (RiskCategory::NoCategory, dec!(100.00), dec!(375.00))
//             ],
//             active_risk_table: [
//                 (RiskCategory::A, dec!(83.75), dec!(83.75)),
//                 (RiskCategory::B, dec!(83.75), dec!(125.00)),
//                 (RiskCategory::C, dec!(99.00), dec!(250.00)),
//                 (RiskCategory::D, dec!(100.00), dec!(375.00)),
//                 (RiskCategory::E, dec!(83.75), dec!(83.75)),
//                 (RiskCategory::F, dec!(83.75), dec!(83.75)),
//                 (RiskCategory::G, dec!(83.75), dec!(83.75)),
//                 (RiskCategory::H, dec!(83.75), dec!(83.75)),
//                 (RiskCategory::I, dec!(83.75), dec!(83.75)),
//                 (RiskCategory::J, dec!(100.00), dec!(375.00)),
//                 (RiskCategory::NoCategory, dec!(100.00), dec!(375.00))
//             ]
//         }
//     }
// }

pub trait RiskCalculator {
    fn portfolio_risk(&self) -> PortfolioRisk;
}

impl<T> From<T> for RiskData
where
    T: IntoIterator<Item = Position>,
{
    fn from(iter: T) -> Self {
        let allocations = iter
            .into_iter()
            .filter_map(|position| {
                position.product.map(|product| RiskAllocation {
                    product,
                    allocation: position.value.amount(),
                })
            })
            .collect();
        Self(allocations)
    }
}

#[derive(Debug)]
pub struct RiskAllocation {
    pub product: Product,
    pub allocation: Decimal,
}

#[derive(Debug, Deref)]
pub struct RiskData(pub Vec<RiskAllocation>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortfolioRisk {
    pub event_risk: Decimal,
    pub net_investment_risk: Decimal,
    pub sector_risk: Decimal,
    pub gross_investment_risk: Decimal,
    pub currency_risk: Decimal,
    pub liquidity_risk: Decimal,
    pub max_risk: Decimal,
}

impl RiskCalculator for RiskData {
    fn portfolio_risk(&self) -> PortfolioRisk {
        let mut event_max = Decimal::ZERO;
        let mut non_d_allocation = Decimal::ZERO;
        let mut d_allocation = Decimal::ZERO;
        let mut sector_values = std::collections::HashMap::new();
        let mut total_abs_allocation = Decimal::ZERO;
        let mut liquidity_max_risk = Decimal::ZERO;
        let mut non_d_abs_value = Decimal::ZERO;
        let mut d_abs_value = Decimal::ZERO;

        for allocation in self.iter() {
            let abs_alloc = allocation.allocation.abs();
            total_abs_allocation += abs_alloc;

            // Event risk
            let risk_ratio = allocation.product.category.risk();
            event_max = event_max.max(allocation.allocation * risk_ratio);

            // Net investment risk & Gross investment risk
            if allocation.product.category == RiskCategory::D {
                d_allocation += allocation.allocation;
                d_abs_value += abs_alloc;
            } else {
                non_d_allocation += allocation.allocation;
                non_d_abs_value += abs_alloc;

                // Sector risk
                if let Some(profile) = &allocation.product.company_profile {
                    let sector = profile.sector.to_ascii_lowercase();
                    *sector_values.entry(sector).or_insert(Decimal::ZERO) += allocation.allocation;
                }
            }

            // Liquidity risk
            if let Some(daily_volume) = allocation.product.order_book_depth {
                if daily_volume > 0 {
                    let position_ratio = abs_alloc / Decimal::from(daily_volume);

                    if allocation.allocation > Decimal::ZERO {
                        if position_ratio > dec!(0.25) {
                            liquidity_max_risk = liquidity_max_risk.max(abs_alloc * dec!(0.07));
                        } else if position_ratio > dec!(0.05) {
                            liquidity_max_risk = liquidity_max_risk.max(abs_alloc * dec!(0.05));
                        }
                    } else if position_ratio > dec!(0.125) {
                        liquidity_max_risk = liquidity_max_risk.max(abs_alloc * dec!(2.00));
                    } else if position_ratio > dec!(0.025) {
                        liquidity_max_risk = liquidity_max_risk.max(abs_alloc * dec!(1.50));
                    }
                }
            }
        }

        let currency_risk = total_abs_allocation * CURRENCY_RISK;
        let max_sector_risk = sector_values
            .values()
            .map(|v| *v * dec!(0.40))
            .max()
            .unwrap_or(Decimal::ZERO)
            + d_allocation;

        let risks = [
            event_max + liquidity_max_risk,
            (non_d_allocation * dec!(0.25) + d_allocation) + currency_risk + liquidity_max_risk,
            max_sector_risk + currency_risk + liquidity_max_risk,
            (non_d_abs_value * dec!(0.10) + d_abs_value) + currency_risk + liquidity_max_risk,
        ];

        PortfolioRisk {
            event_risk: event_max,
            net_investment_risk: non_d_allocation * dec!(0.25) + d_allocation,
            sector_risk: max_sector_risk,
            gross_investment_risk: non_d_abs_value * dec!(0.10) + d_abs_value,
            currency_risk,
            liquidity_risk: liquidity_max_risk,
            max_risk: risks.iter().max().unwrap_or(&Decimal::ZERO).clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use crate::{
        client::Degiro,
        models::risk::{RiskAllocation, RiskCalculator, RiskData},
    };

    #[tokio::test]
    async fn test_risk_calculator() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let xs = client.portfolio(false).await.unwrap();
        let xs = xs.current().0;
        let risk_data = RiskData::from(xs);
        dbg!(risk_data.portfolio_risk());
    }
    #[tokio::test]
    async fn test_allocation_risk_calculator() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();

        // Create test allocations
        let allocations = vec![
            ("1538409", dec!(0.5)),  // Duke Energy
            ("332111", dec!(0.25)),  // Microsoft
            ("1147582", dec!(0.25)), // Nvidia
        ];

        // Fetch products and create allocations
        let mut portfolio_allocations = Vec::new();
        for (id, allocation) in allocations {
            let product = client.product(id).await.unwrap().unwrap();
            portfolio_allocations.push(RiskAllocation {
                product,
                allocation,
            });
        }

        // Calculate risks
        let risk_data = RiskData(portfolio_allocations);
        // dbg!(risk_data.event_risk());
        // dbg!(risk_data.net_investment_risk());
        // dbg!(risk_data.sector_risk());
        // dbg!(risk_data.gross_investment_risk());
        // dbg!(risk_data.currency_risk());
        // dbg!(risk_data.liquidity_risk());
        dbg!(risk_data.portfolio_risk());
    }
}
