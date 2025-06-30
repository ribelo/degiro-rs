use futures_concurrency::prelude::*;
use rust_decimal::Decimal;

use crate::{
    client::Degiro,
    error::{ClientError, DataError, ResponseError},
    http::{HttpClient, HttpRequest},
    models::{Portfolio, PortfolioObject, Position},
    paths::{UPDATE_DATA_PATH},
};

impl Degiro {
    pub async fn portfolio(&self, fetch_products: bool) -> Result<Portfolio, ClientError> {
        let url = self.build_trading_url(UPDATE_DATA_PATH)?;
        
        let res = self.request_json(
            HttpRequest::get(url)
                .query("portfolio", "0")
        ).await?;
        
        let body = res
            .get("portfolio")
            .and_then(|p| p.get("value"))
            .ok_or_else(|| ClientError::ResponseError(ResponseError::invalid("Missing portfolio.value in response")))?;
        let objs: Vec<PortfolioObject> = serde_json::from_value(body.clone())?;
        let mut positions = Vec::with_capacity(objs.len());

        let mut position_futures = Vec::new();
        for obj in objs {
            let position: Position = obj.try_into().map_err(|e| {
                ClientError::DataError(DataError::parse_error("position", format!("Failed to parse position: {e}")))
            })?;

            if fetch_products && position.position_type == crate::models::PositionType::Product {
                position_futures.push(async move {
                    let product = self.product(&position.id).await?;
                    Ok::<(Position, Option<crate::models::Product>), ClientError>((position, product))
                });
            } else {
                positions.push(position);
            }
        }

        if !position_futures.is_empty() {
            let results = position_futures.try_join().await?;
            for (mut position, product) in results {
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
    use rust_decimal_macros::dec;

    use crate::client::Degiro;

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_current_portfolio() {
        let client = Degiro::load_from_env().expect("Failed to load client from env");
        client.login().await.expect("Failed to login");
        client.account_config().await.expect("Failed to get account config");
        let xs = client.portfolio(false).await.expect("Failed to get portfolio");
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
        client.account_config().await.expect("Failed to get account config");

        let total = client.total_portfolio_value().await.expect("Failed to get total portfolio value");
        dbg!(&total);
        assert!(
            total >= dec!(0.0),
            "Total portfolio value should not be negative"
        );
    }
}
