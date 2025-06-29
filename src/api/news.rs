use reqwest::header;

use crate::{
    client::{ApiErrorResponse, ClientError, ClientStatus, Degiro},
    models::News,
};

impl Degiro {
    pub async fn company_news_by_id<T: AsRef<str>>(
        &self,
        id: T,
    ) -> Result<Option<Vec<News>>, ClientError> {
        let product = &self.product(id.as_ref()).await?;
        match product {
            Some(p) => self.company_news(&p.isin).await,
            None => Ok(None),
        }
    }

    pub async fn company_news<T: AsRef<str>>(
        &self,
        isin: T,
    ) -> Result<Option<Vec<News>>, ClientError> {
        self.ensure_authorized().await?;

        let url = crate::paths::BASE_API_URL.to_owned() + crate::paths::COMPANY_NEWS_PATH;

        self.acquire_limit().await;

        let req = self
            .http_client
            .get(url)
            .query(&[
                ("isin", isin.as_ref()),
                ("intAccount", &self.int_account().to_string()),
                ("sessionId", &self.session_id()),
                ("limit", "10"),
                ("offset", "0"),
                ("languages", "en,pl"),
            ])
            .header(header::REFERER, crate::paths::REFERER)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.to_string());

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

        let json = res.json::<serde_json::Value>().await?;
        let data = match json.get("data") {
            None => return Ok(None),
            Some(d) if d.is_null() => return Ok(None),
            Some(d) => d,
        };

        let news = data
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or(ClientError::UnexpectedError("Missing items field".into()))?
            .iter()
            .map(News::from)
            .collect();

        Ok(Some(news))
    }
}

#[cfg(test)]
mod tests {
    use crate::client::Degiro;
    #[tokio::test]
    async fn test_news_by_company() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let news = client.company_news("US7433151039").await.unwrap();
        for x in &news {
            println!("{}", serde_json::to_string_pretty(x).unwrap());
        }
    }
}
