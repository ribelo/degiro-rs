use reqwest::{header, Url};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, convert::TryInto};
use strum::EnumString;
use thiserror::Error;

use crate::{
    client::{Client, ClientError, ClientStatus},
    money::{Currency, Money},
};

use super::product::Product;

#[derive(Debug, Deserialize)]
struct PortfolioObject {
    value: Vec<ValueObject>,
}

#[derive(Debug, Deserialize)]
struct ValueObject {
    #[serde(rename = "name")]
    elem_type: ElemType,
    value: Option<Value>,
}

#[derive(Debug, Deserialize, EnumString)]
#[serde(rename_all = "camelCase")]
enum ElemType {
    Id,
    PositionType,
    Size,
    Price,
    Value,
    AccruedInterest,
    PlBase,
    TodayPlBase,
    PortfolioValueCorrection,
    BreakEvenPrice,
    AverageFxRate,
    RealizedProductPl,
    RealizedFxPl,
    TodayRealizedProductPl,
    TodayRealizedFxPl,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct PositionDetails {
    pub id: String,
    pub position_type: PositionType,
    pub size: f64,
    pub price: f64,
    pub currency: Currency,
    pub value: Money,
    pub accrued_interest: Option<f64>,
    pub base_value: Money,
    pub today_value: Money,
    pub portfolio_value_correction: f64,
    pub break_even_price: f64,
    pub average_fx_rate: f64,
    pub realized_product_profit: Money,
    pub realized_fx_profit: Money,
    pub today_realized_product_pl: Money,
    pub today_realized_fx_pl: Money,
    pub total_profit: Money,
    pub product_profit: Money,
    pub fx_profit: Money,
}

#[derive(Clone, Debug)]
pub struct Position {
    pub inner: PositionDetails,
    pub client: Client,
}

impl Position {
    pub fn new(inner: PositionDetails, client: Client) -> Self {
        Self { inner, client }
    }
    pub async fn product(&self) -> Result<Product, ClientError> {
        self.client.product(&self.inner.id).await
    }
}

#[derive(Clone, Debug, Default)]
pub struct Portfolio(pub Vec<Position>);

impl Portfolio {
    pub fn new(xs: impl Into<Vec<Position>>) -> Self {
        Self(xs.into())
    }
    pub fn iter(&self) -> std::slice::Iter<Position> {
        self.0.iter()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn first(&self) -> Option<&Position> {
        self.0.first()
    }
    pub fn last(&self) -> Option<&Position> {
        self.0.last()
    }
    pub fn get(&self, index: usize) -> Option<&Position> {
        self.0.get(index)
    }
    pub fn into_inner(self) -> Vec<Position> {
        self.0
    }
    pub fn into_details(self) -> Vec<PositionDetails> {
        self.0.into_iter().map(|x| x.inner).collect()
    }
    pub fn as_slice(&self) -> &[Position] {
        self.0.as_slice()
    }
    pub fn as_mut_slice(&mut self) -> &mut [Position] {
        self.0.as_mut_slice()
    }
}

impl Portfolio {
    pub fn value(&self) -> HashMap<Currency, f64> {
        let mut m = HashMap::default();
        for p in &self.0 {
            let money = &p.inner.value;
            let x = m.entry(money.currency).or_insert(0.0);
            *x += money.amount;
        }
        m
    }

    pub fn base_value(&self) -> HashMap<Currency, f64> {
        let mut m = HashMap::default();
        for p in &self.0 {
            let money = &p.inner.base_value;
            let x = m.entry(money.currency).or_insert(0.0);
            *x += money.amount;
        }
        m
    }

    pub fn current(self) -> Self {
        let xs = self
            .0
            .into_iter()
            .filter(|p| p.inner.size > 0.0)
            .collect::<Vec<_>>();

        Portfolio::new(xs)
    }

    pub fn products(self) -> Self {
        let xs = self
            .0
            .into_iter()
            .filter(|p| p.inner.position_type == PositionType::Product)
            .collect::<Vec<_>>();

        Portfolio::new(xs)
    }

    pub fn cash(self) -> Self {
        let xs = self
            .0
            .into_iter()
            .filter(|p| p.inner.position_type == PositionType::Cash)
            .collect::<Vec<_>>();

        Portfolio::new(xs)
    }

    pub fn only_id(self, id: &str) -> Self {
        let xs = self
            .0
            .into_iter()
            .filter(|p| p.inner.id == id)
            .collect::<Vec<_>>();

        Portfolio::new(xs)
    }
}

#[derive(Clone, Debug, Default, EnumString, PartialEq)]
#[strum(ascii_case_insensitive)]
pub enum PositionType {
    Cash,
    #[default]
    Product,
}

#[derive(Debug, Error)]
#[error("can't parse object {:#?}", 0)]
pub struct ParsePositionError(PortfolioObject);

impl TryFrom<PortfolioObject> for PositionDetails {
    type Error = ParsePositionError;

    fn try_from(obj: PortfolioObject) -> Result<Self, Self::Error> {
        let mut position = PositionDetails::default();
        let mut value = 0.0;
        for row in &obj.value {
            match row.elem_type {
                ElemType::Id => {
                    position.id = row.value.as_ref().unwrap().as_str().unwrap().to_string();
                }
                ElemType::PositionType => {
                    match row.value.as_ref().unwrap().as_str().unwrap().parse() {
                        Ok(val) => position.position_type = val,
                        Err(_) => return Err(ParsePositionError(obj)),
                    };
                }
                ElemType::Size => {
                    let val = row.value.as_ref().unwrap().as_f64().unwrap();
                    position.size = val;
                }
                ElemType::Price => {
                    position.price = row.value.as_ref().unwrap().as_f64().unwrap();
                }
                ElemType::Value => {
                    value = row.value.as_ref().unwrap().as_f64().unwrap();
                }
                ElemType::AccruedInterest => {
                    if let Some(s) = &row.value {
                        let val = s.as_f64().unwrap();
                        if val > 0.0 {
                            position.accrued_interest = Some(val);
                        }
                    }
                }
                ElemType::PlBase => {
                    match serde_json::from_value::<HashMap<String, f64>>(
                        row.value.as_ref().unwrap().clone(),
                    ) {
                        Ok(m) => match TryInto::<Money>::try_into(m) {
                            Ok(val) => {
                                position.currency = val.currency;
                                position.base_value = -val;
                            }
                            Err(_) => return Err(ParsePositionError(obj)),
                        },
                        Err(_) => return Err(ParsePositionError(obj)),
                    }
                }
                ElemType::TodayPlBase => {
                    match serde_json::from_value::<HashMap<String, f64>>(
                        row.value.as_ref().unwrap().clone(),
                    ) {
                        Ok(m) => match m.try_into() {
                            Ok(val) => position.today_value = val,
                            Err(_) => return Err(ParsePositionError(obj)),
                        },
                        Err(_) => return Err(ParsePositionError(obj)),
                    }
                }
                ElemType::PortfolioValueCorrection => {
                    if let Some(s) = &row.value {
                        let val = s.as_f64().unwrap();
                        position.portfolio_value_correction = val;
                    }
                }
                ElemType::BreakEvenPrice => {
                    if let Some(s) = &row.value {
                        let val = s.as_f64().unwrap();
                        position.break_even_price = val;
                    }
                }
                ElemType::AverageFxRate => {
                    if let Some(s) = &row.value {
                        let val = s.as_f64().unwrap();
                        position.average_fx_rate = val;
                    }
                }
                ElemType::RealizedProductPl => {
                    if let Some(s) = &row.value {
                        let val = s.as_f64().unwrap();
                        position.realized_product_profit = Money::new(position.currency, val);
                    }
                }
                ElemType::RealizedFxPl => {
                    if let Some(s) = &row.value {
                        let val = s.as_f64().unwrap();
                        position.realized_fx_profit = Money::new(position.currency, val);
                    }
                }
                ElemType::TodayRealizedProductPl => {
                    if let Some(s) = &row.value {
                        let val = s.as_f64().unwrap();
                        position.today_realized_product_pl = Money::new(position.currency, val);
                    }
                }
                ElemType::TodayRealizedFxPl => {
                    if let Some(s) = &row.value {
                        let val = s.as_f64().unwrap();
                        position.today_realized_fx_pl = Money::new(position.currency, val);
                    }
                }
            }
        }
        let currency = position.total_profit.currency;
        position.total_profit = Money::new(
            currency,
            (position.price * position.size - position.break_even_price * position.size)
                * position.average_fx_rate,
        );
        let profit = (position.price * position.size)
            - (position.break_even_price * position.size) / position.average_fx_rate;
        position.product_profit = Money::new(currency, profit);
        position.value = Money::new(currency, value);
        position.fx_profit = ((position.total_profit.clone() - position.product_profit.clone())
            .unwrap()
            - position.realized_fx_profit.clone())
        .unwrap();
        Ok(position)
    }
}

impl Client {
    pub async fn portfolio(&self) -> Result<Portfolio, ClientError> {
        if self.inner.lock().unwrap().status != ClientStatus::Authorized {
            return Err(ClientError::Unauthorized);
        }

        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = &inner.account_config.trading_url;
            let path_url = "v5/update/";
            let url = Url::parse(base_url)
                .unwrap()
                .join(path_url)
                .unwrap()
                .join(&format!(
                    "{};jsessionid={}",
                    inner.int_account, inner.session_id
                ))
                .unwrap();

            inner
                .http_client
                .get(url)
                .query(&[("portfolio", 0)])
                .header(header::REFERER, &inner.referer)
        };

        let rate_limiter = {
            let inner = self.inner.lock().unwrap();
            inner.rate_limiter.clone()
        };
        rate_limiter.acquire_one().await;

        let res = req.send().await?;

        match res.error_for_status() {
            Ok(res) => {
                let json = res.json::<Value>().await.unwrap();
                let body = json.get("portfolio").unwrap().get("value").unwrap();
                let objs: Vec<PortfolioObject> = serde_json::from_value(body.clone()).unwrap();
                let mut xs: Vec<_> = Vec::new();
                for obj in objs {
                    let p: PositionDetails = obj.try_into().unwrap();
                    let position = Position::new(p, self.clone());
                    xs.push(position)
                }
                Ok(Portfolio::new(xs))
            }
            Err(err) => match err.status().unwrap().as_u16() {
                401 => {
                    self.inner.lock().unwrap().status = ClientStatus::Unauthorized;
                    Err(ClientError::Unauthorized)
                }
                _ => Err(ClientError::UnexpectedError {
                    source: Box::new(err),
                }),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::client::Client;

    #[tokio::test]
    async fn current_portfolio() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let xs = client.portfolio().await.unwrap();
        dbg!(&xs);
        dbg!(&xs.value());
        dbg!(&xs.base_value());
    }
}
