use reqwest::{header, Url};

use crate::{
    client::{ApiErrorResponse, ClientError, ClientStatus, Degiro},
    models::CompanyProfile,
    paths::{BASE_API_URL, COMPANY_PROFILE_PATH, REFERER},
};

impl Degiro {
    pub async fn company_profile_by_id(
        &self,
        id: impl AsRef<str>,
    ) -> Result<Option<CompanyProfile>, ClientError> {
        let product = self.product(id.as_ref()).await?;
        match product {
            Some(p) => self.company_profile(&p.isin).await,
            None => Ok(None),
        }
    }
    pub async fn company_profile(
        &self,
        isin: impl AsRef<str>,
    ) -> Result<Option<CompanyProfile>, ClientError> {
        self.ensure_authorized().await?;

        let url = Url::parse(BASE_API_URL)
            .unwrap()
            .join(COMPANY_PROFILE_PATH)
            .unwrap()
            .join(isin.as_ref())
            .unwrap();

        let req = self
            .http_client
            .get(url)
            .query(&[
                ("intAccount", &self.int_account().to_string()),
                ("sessionId", &self.session_id()),
            ])
            .header(header::REFERER, REFERER)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.to_string());

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

        let data = res.json::<serde_json::Value>().await?["data"].take();
        if data.is_null() {
            return Ok(None);
        }

        let mut company_profile = serde_json::from_value::<CompanyProfile>(data)?;
        company_profile.isin = isin.as_ref().to_string();

        Ok(Some(company_profile))
    }
}

#[cfg(test)]
mod tests {
    use crate::client::Degiro;

    #[tokio::test]
    async fn test_company_profile() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let profile = client.company_profile_by_id("332111").await.unwrap();
        dbg!(&profile);
    }
}
