use derive_more::derive::Deref;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use super::{Position, Product};

pub const CURRENCY_RISK: f64 = 0.0636;

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
    pub fn risk(&self) -> f64 {
        match self {
            RiskCategory::A => 0.6250,
            RiskCategory::B => 0.8125,
            RiskCategory::C => 0.9900,
            RiskCategory::D => 1.0000,
            RiskCategory::E => 0.0625,
            RiskCategory::F => 0.1250,
            RiskCategory::G => 0.1875,
            RiskCategory::H => 0.2500,
            RiskCategory::I => 0.3125,
            RiskCategory::J => 1.0000,
            RiskCategory::NoCategory => 1.0000,
        }
    }
}

#[derive(Debug, EnumString, Display)]
pub enum Profile {
    Trader,
    Active,
}

// pub struct RiskCalculator {
//     risk_table: [(RiskCategory, f64, f64); 11],
//     active_risk_table: [(RiskCategory, f64, f64); 11]
// }

// impl Default for RiskCalculator {
//     fn default() -> Self {
//         Self {
//             risk_table: [
//                 (RiskCategory::A, (62.50), (62.50)),
//                 (RiskCategory::B, (81.25), (125.00)),
//                 (RiskCategory::C, (99.00), (250.00)),
//                 (RiskCategory::D, (100.00), (375.00)),
//                 (RiskCategory::E, (6.25), (6.25)),
//                 (RiskCategory::F, (12.50), (12.50)),
//                 (RiskCategory::G, (18.75), (18.75)),
//                 (RiskCategory::H, (25.00), (25.00)),
//                 (RiskCategory::I, (31.25), (31.25)),
//                 (RiskCategory::J, (100.00), (375.00)),
//                 (RiskCategory::NoCategory, (100.00), (375.00))
//             ],
//             active_risk_table: [
//                 (RiskCategory::A, (83.75), (83.75)),
//                 (RiskCategory::B, (83.75), (125.00)),
//                 (RiskCategory::C, (99.00), (250.00)),
//                 (RiskCategory::D, (100.00), (375.00)),
//                 (RiskCategory::E, (83.75), (83.75)),
//                 (RiskCategory::F, (83.75), (83.75)),
//                 (RiskCategory::G, (83.75), (83.75)),
//                 (RiskCategory::H, (83.75), (83.75)),
//                 (RiskCategory::I, (83.75), (83.75)),
//                 (RiskCategory::J, (100.00), (375.00)),
//                 (RiskCategory::NoCategory, (100.00), (375.00))
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
                    allocation: position.value.amount().to_f64().unwrap_or(0.0),
                })
            })
            .collect();
        Self(allocations)
    }
}

#[derive(Debug)]
pub struct RiskAllocation {
    pub product: Product,
    pub allocation: f64,
}

#[derive(Debug, Deref)]
pub struct RiskData(pub Vec<RiskAllocation>);

#[derive(Debug, Clone, PartialEq)]
pub struct PortfolioRisk {
    pub event_risk: f64,
    pub net_investment_risk: f64,
    pub sector_risk: f64,
    pub gross_investment_risk: f64,
    pub currency_risk: f64,
    pub liquidity_risk: f64,
    pub max_risk: f64,
}

impl RiskCalculator for RiskData {
    fn portfolio_risk(&self) -> PortfolioRisk {
        let mut event_max = 0.0;
        let mut non_d_allocation = 0.0;
        let mut d_allocation = 0.0;
        let mut sector_values = std::collections::HashMap::new();
        let mut total_abs_allocation = 0.0;
        let mut liquidity_max_risk = 0.0;
        let mut non_d_abs_value = 0.0;
        let mut d_abs_value = 0.0;

        for allocation in self.iter() {
            let abs_alloc = allocation.allocation.abs();
            total_abs_allocation += abs_alloc;

            // Event risk
            let risk_ratio = allocation.product.category.risk();
            event_max = f64::max(event_max, allocation.allocation * risk_ratio);

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
                    *sector_values.entry(sector).or_insert(0.0) += allocation.allocation;
                }
            }

            // Liquidity risk
            if let Some(daily_volume) = allocation.product.order_book_depth {
                if daily_volume > 0 {
                    let position_ratio = abs_alloc / daily_volume as f64;

                    if allocation.allocation > 0.0 {
                        if position_ratio > 0.25 {
                            liquidity_max_risk = f64::max(liquidity_max_risk, abs_alloc * 0.07);
                        } else if position_ratio > 0.05 {
                            liquidity_max_risk = f64::max(liquidity_max_risk, abs_alloc * 0.05);
                        }
                    } else if position_ratio > 0.125 {
                        liquidity_max_risk = f64::max(liquidity_max_risk, abs_alloc * 2.00);
                    } else if position_ratio > 0.025 {
                        liquidity_max_risk = f64::max(liquidity_max_risk, abs_alloc * 1.50);
                    }
                }
            }
        }

        let currency_risk = total_abs_allocation * CURRENCY_RISK;
        let max_sector_risk = sector_values
            .values()
            .map(|v| *v * 0.40)
            .fold(0.0, f64::max)
            + d_allocation;

        let risks = [
            event_max + liquidity_max_risk,
            (non_d_allocation * 0.25 + d_allocation) + currency_risk + liquidity_max_risk,
            max_sector_risk + currency_risk + liquidity_max_risk,
            (non_d_abs_value * 0.10 + d_abs_value) + currency_risk + liquidity_max_risk,
        ];

        PortfolioRisk {
            event_risk: event_max,
            net_investment_risk: non_d_allocation * 0.25 + d_allocation,
            sector_risk: max_sector_risk,
            gross_investment_risk: non_d_abs_value * 0.10 + d_abs_value,
            currency_risk,
            liquidity_risk: liquidity_max_risk,
            max_risk: risks.iter().fold(0.0, |a, &b| a.max(b)),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        client::Degiro,
        models::risk::{RiskAllocation, RiskCalculator, RiskData},
    };

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_risk_calculator() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");
        let xs = client
            .portfolio(false)
            .await
            .expect("Failed to get portfolio");
        let xs = xs.current().0;
        let risk_data = RiskData::from(xs);
        dbg!(risk_data.portfolio_risk());
    }
    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_allocation_risk_calculator() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");

        // Create test allocations
        let allocations = vec![
            ("1538409", 0.5),  // Duke Energy
            ("332111", 0.25),  // Microsoft
            ("1147582", 0.25), // Nvidia
        ];

        // Fetch products and create allocations
        let mut portfolio_allocations = Vec::new();
        for (id, allocation) in allocations {
            let product = client
                .product(id)
                .await
                .expect("Failed to get product")
                .expect("Product not found");
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
