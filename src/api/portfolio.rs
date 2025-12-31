use futures_concurrency::prelude::*;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

use crate::{
    client::Degiro,
    error::{ClientError, DataError, ResponseError},
    http::{HttpClient, HttpRequest},
    models::{Currency, Portfolio, PortfolioObject, Position, PositionType, Product},
    paths::UPDATE_DATA_PATH,
};

fn parse_currency_code(raw: &str) -> Option<Currency> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed
        .parse()
        .ok()
        .or_else(|| trimmed.to_ascii_uppercase().parse().ok())
}

fn profile_instrument_currency(product: &Product) -> Option<Currency> {
    product
        .company_profile
        .as_ref()
        .and_then(|profile| parse_currency_code(&profile.currency))
}

fn inferred_instrument_currency(product: &Product) -> Currency {
    profile_instrument_currency(product).unwrap_or_else(|| Currency::from(product.exchange))
}

fn should_refine_instrument_currency(position: &Position) -> bool {
    position.position_type == PositionType::Product
        && position.instrument_currency == position.base_currency
        && position.value.currency() == position.base_currency
}

fn refine_instrument_currency_from_product(position: &mut Position, product: &Product) {
    debug_assert!(
        position.position_type != PositionType::Cash,
        "Currency refinement should not target cash positions"
    );
    debug_assert!(
        position.value.currency() == position.base_currency
            || position.instrument_currency != position.base_currency,
        "Positions with explicit value currency should already have instrument currency set"
    );
    if should_refine_instrument_currency(position) {
        position.instrument_currency = inferred_instrument_currency(product);
    }
}

impl Degiro {
    pub async fn portfolio(&self, fetch_products: bool) -> Result<Portfolio, ClientError> {
        let url = self.build_trading_url(UPDATE_DATA_PATH)?;

        let res = self
            .request_json(HttpRequest::get(url).query("portfolio", "0"))
            .await?;

        let body = res
            .get("portfolio")
            .and_then(|p| p.get("value"))
            .ok_or_else(|| {
                ClientError::ResponseError(ResponseError::invalid(
                    "Missing portfolio.value in response",
                ))
            })?;
        let objs: Vec<PortfolioObject> = serde_json::from_value(body.clone())?;
        let mut positions = Vec::with_capacity(objs.len());

        let mut position_futures = Vec::new();
        let min_position_size = Decimal::from_f64(1e-6).unwrap_or(Decimal::ZERO);
        for obj in objs {
            let position: Position = obj.try_into().map_err(|e| {
                ClientError::DataError(DataError::parse_error(
                    "position",
                    format!("Failed to parse position: {e}"),
                ))
            })?;

            if position.size.abs() < min_position_size {
                continue;
            }

            if fetch_products && position.position_type == PositionType::Product {
                let id = position.id.clone();
                position_futures.push(async move {
                    let product = self.product(&id).await?;
                    Ok::<(Position, Option<crate::models::Product>), ClientError>((
                        position, product,
                    ))
                });
            } else {
                positions.push(position);
            }
        }

        if !position_futures.is_empty() {
            let results = position_futures.try_join().await?;
            for (mut position, product) in results {
                if let Some(ref prod) = product {
                    refine_instrument_currency_from_product(&mut position, prod);
                }
                position.product = product;
                positions.push(position);
            }
        }

        Ok(Portfolio::new(positions))
    }

    pub async fn total_portfolio_value(&self) -> Result<Decimal, ClientError> {
        let portfolio = self.portfolio(false).await?.products();
        let mut total = Decimal::ZERO;

        for position in portfolio.0 {
            total += position.value.amount();
        }

        Ok(total)
    }
}

#[cfg(test)]
mod test {
    use super::{refine_instrument_currency_from_product, should_refine_instrument_currency};

    use crate::{
        client::Degiro,
        http::{HttpClient, HttpRequest},
        models::{
            risk::RiskCategory, CompanyProfile, Currency, Exchange, Money, Position, PositionType,
            Product,
        },
        paths::UPDATE_DATA_PATH,
    };
    use rust_decimal::Decimal;

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_current_portfolio() {
        let client = Degiro::load_from_env().expect("Failed to load client from env");
        client.login().await.expect("Failed to login");
        client
            .account_config()
            .await
            .expect("Failed to get account config");
        let url = client
            .build_trading_url(UPDATE_DATA_PATH)
            .expect("build trading url");
        let raw = client
            .request_json(HttpRequest::get(url).query("portfolio", "0"))
            .await
            .expect("fetch raw portfolio");
        std::fs::write(
            "portfolio_raw_tmp.json",
            serde_json::to_string_pretty(&raw).expect("serialize raw portfolio"),
        )
        .expect("write raw portfolio");
        let xs = client
            .portfolio(false)
            .await
            .expect("Failed to get portfolio");
        let xs = xs.current().products();
        std::fs::write(
            "portfolio_tmp.json",
            serde_json::to_string_pretty(&xs).expect("Failed to serialize portfolio"),
        )
        .expect("Failed to write portfolio file");
        // dbg!(&xs);
        // dbg!(&xs.group_by_category().keys().collect::<Vec<_>>());
        // dbg!(&xs.group_by_sector().keys().collect::<Vec<_>>());
    }
    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_total_portfolio() {
        let client = Degiro::load_from_env().expect("Failed to load client from env");
        client.login().await.expect("Failed to login");
        client
            .account_config()
            .await
            .expect("Failed to get account config");

        let total = client
            .total_portfolio_value()
            .await
            .expect("Failed to get total portfolio value");
        dbg!(&total);
        assert!(
            total >= Decimal::ZERO,
            "Total portfolio value should not be negative"
        );
    }

    fn make_product(exchange: Exchange, profile_currency: Option<&str>) -> Product {
        let profile = profile_currency.map(|ccy| {
            let mut profile = CompanyProfile::default();
            profile.currency = ccy.to_string();
            profile
        });
        Product {
            active: true,
            buy_order_types: None,
            category: RiskCategory::A,
            close_price: 0.0,
            close_price_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                .expect("valid close price date"),
            contract_size: 1.0,
            exchange,
            feed_quality: None,
            feed_quality_secondary: None,
            id: "TEST".to_string(),
            isin: String::new(),
            name: "Test Instrument".to_string(),
            only_eod_prices: false,
            order_book_depth: None,
            order_book_depth_secondary: None,
            order_time_types: None,
            product_bit_types: None,
            product_type: "STOCK".to_string(),
            product_type_id: 0,
            quality_switch_free: false,
            quality_switch_free_secondary: false,
            quality_switchable: false,
            quality_switchable_secondary: false,
            sell_order_types: None,
            symbol: "TST".to_string(),
            tradable: true,
            vwd_id: None,
            vwd_id_secondary: None,
            vwd_identifier_type: None,
            vwd_identifier_type_secondary: None,
            vwd_module_id: None,
            vwd_module_id_secondary: None,
            company_profile: profile,
        }
    }

    #[test]
    fn refine_instrument_currency_prefers_profile_currency() {
        let mut position = Position::default();
        position.position_type = PositionType::Product;
        position.base_currency = Currency::EUR;
        position.instrument_currency = Currency::EUR;
        position.value = Money::new(Currency::EUR, Decimal::ZERO);
        let product = make_product(Exchange::NSDQ, Some("usd"));

        refine_instrument_currency_from_product(&mut position, &product);

        assert_eq!(position.instrument_currency, Currency::USD);
        assert!(!should_refine_instrument_currency(&position));
    }

    #[test]
    fn refine_instrument_currency_falls_back_to_exchange_mapping() {
        let mut position = Position::default();
        position.position_type = PositionType::Product;
        position.base_currency = Currency::EUR;
        position.instrument_currency = Currency::EUR;
        position.value = Money::new(Currency::EUR, Decimal::ZERO);
        let product = make_product(Exchange::SWX, None);

        refine_instrument_currency_from_product(&mut position, &product);

        assert_eq!(position.instrument_currency, Currency::CHF);
        assert_eq!(position.value.currency(), Currency::EUR);
    }

    #[test]
    fn refine_instrument_currency_respects_explicit_value_currency() {
        let mut position = Position::default();
        position.position_type = PositionType::Product;
        position.base_currency = Currency::EUR;
        position.instrument_currency = Currency::USD;
        position.value = Money::new(Currency::USD, Decimal::ZERO);
        let product = make_product(Exchange::LSE, Some("GBP"));

        refine_instrument_currency_from_product(&mut position, &product);

        assert_eq!(position.instrument_currency, Currency::USD);
        assert_eq!(position.value.currency(), Currency::USD);
    }
}
