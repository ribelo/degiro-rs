use futures_concurrency::prelude::*;
use reqwest::{header, Url};
use rust_decimal::Decimal;
use serde_json::Value;

use crate::{
    client::{ApiErrorResponse, ClientError, ClientStatus, Degiro},
    models::{Portfolio, PortfolioObject, Position},
    paths::{REFERER, TRADING_URL, UPDATE_DATA_PATH},
};

impl Degiro {
    pub async fn portfolio(&self, fetch_products: bool) -> Result<Portfolio, ClientError> {
        self.ensure_authorized().await?;

        let url = {
            let url =
                Url::parse(TRADING_URL).map_err(|e| ClientError::UnexpectedError(e.to_string()))?;
            url.join(UPDATE_DATA_PATH)
                .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
                .join(&format!(
                    "{};jsessionid={}",
                    self.int_account(),
                    self.session_id()
                ))
                .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
        };

        let req = self
            .http_client
            .get(url)
            .query(&[("portfolio", 0)])
            .header(header::REFERER, REFERER);

        self.acquire_limit().await;

        let res = req.send().await?;

        if let Err(err) = res.error_for_status_ref() {
            let Some(status) = err.status() else {
                return Err(ClientError::UnexpectedError(err.to_string()));
            };

            if status.as_u16() == 401 {
                self.set_auth_state(ClientStatus::Unauthorized);
                return Err(ClientError::Unauthorized);
            }

            let error_response = res.json::<ApiErrorResponse>().await?;
            return Err(ClientError::ApiError(error_response));
        }

        let json = res.json::<Value>().await?;
        let body = json
            .get("portfolio")
            .and_then(|p| p.get("value"))
            .ok_or_else(|| ClientError::UnexpectedError("Invalid portfolio response".into()))?;
        let objs: Vec<PortfolioObject> = serde_json::from_value(body.clone())?;
        let mut positions = Vec::with_capacity(objs.len());

        let mut position_futures = Vec::new();
        for obj in objs {
            let position: Position = obj.try_into().map_err(|e| {
                ClientError::UnexpectedError(format!("Failed to parse position: {}", e))
            })?;

            if fetch_products && position.position_type == crate::models::PositionType::Product {
                position_futures.push(async {
                    let product = self.product(&position.id).await.ok().flatten();
                    (position, product)
                });
            } else {
                positions.push(position);
            }
        }

        if !position_futures.is_empty() {
            let results = position_futures.join().await;
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
    async fn test_current_portfolio() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let xs = client.portfolio(false).await.unwrap();
        let xs = xs.current().products();
        std::fs::write(
            "portfolio_tmp.json",
            serde_json::to_string_pretty(&xs).unwrap(),
        )
        .unwrap();
        // dbg!(&xs);
        // dbg!(&xs.group_by_category().keys().collect::<Vec<_>>());
        // dbg!(&xs.group_by_sector().keys().collect::<Vec<_>>());
    }
    #[tokio::test]
    async fn test_total_portfolio() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();

        let total = client.total_portfolio_value().await.unwrap();
        dbg!(&total);
        assert!(
            total >= dec!(0.0),
            "Total portfolio value should not be negative"
        );
    }
}
