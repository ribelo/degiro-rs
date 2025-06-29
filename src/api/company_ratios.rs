use reqwest::{header, Url};

use crate::{
    client::{ApiErrorResponse, ClientError, ClientStatus, Degiro},
    models::{CompanyRatios, CurrentRatios},
    paths::{BASE_API_URL, REFERER},
};

impl Degiro {
    pub async fn company_ratios_by_id(
        &self,
        id: impl AsRef<str>,
    ) -> Result<Option<CompanyRatios>, ClientError> {
        let product = self.product(id.as_ref()).await?;
        match &product {
            Some(p) => self.company_ratios(&p.isin).await,
            None => Ok(None),
        }
    }

    pub async fn company_ratios(
        &self,
        isin: impl AsRef<str>,
    ) -> Result<Option<CompanyRatios>, ClientError> {
        self.ensure_authorized().await?;

        let url = Url::parse(BASE_API_URL)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join(crate::paths::COMPANY_RATIOS_PATH)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join(isin.as_ref())
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

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

        let json = res.json::<serde_json::Value>().await?;
        let data = json
            .get("data")
            .ok_or_else(|| ClientError::UnexpectedError("Missing data key".to_string()))?;

        if data.is_null() {
            return Ok(None);
        }

        Ok(Some(CompanyRatios {
            isin: isin.as_ref().to_string(),
            current_ratios: CurrentRatios::from(data["currentRatios"].clone()),
        }))
    }
}

#[cfg(test)]
mod test {

    use crate::client::Degiro;

    #[tokio::test]
    async fn test_company_ratios() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();

        let report = client.company_ratios_by_id("15850348").await.unwrap();
        println!("{:#?}", report);
    }
}
