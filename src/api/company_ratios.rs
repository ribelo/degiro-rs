use crate::{
    client::Degiro,
    error::{ClientError, DataError, ResponseError},
    http::{HttpClient, HttpRequest},
    models::{CompanyRatios, CurrentRatios},
    paths::{BASE_API_URL, COMPANY_RATIOS_PATH},
};
use reqwest::StatusCode;

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
        let url = format!("{}{}{}", BASE_API_URL, COMPANY_RATIOS_PATH, isin.as_ref());

        let json = match self
            .request_json(
                HttpRequest::get(url)
                    .query("intAccount", self.int_account().to_string())
                    .query("sessionId", self.session_id())
                    .header("Content-Type", "application/json"),
            )
            .await
        {
            Ok(json) => json,
            Err(ClientError::ResponseError(ResponseError::HttpStatus { status, .. }))
                if status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND =>
            {
                return Ok(None);
            }
            Err(err) => return Err(err),
        };
        let data = json
            .get("data")
            .ok_or_else(|| DataError::missing_field("data"))?;

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
    #[ignore = "Integration test - hits real API"]
    async fn test_company_ratios() {
        let client = Degiro::load_from_env()
            .expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client
            .account_config()
            .await
            .expect("Failed to get account configuration");

        let report = client
            .company_ratios_by_id("15850348")
            .await
            .expect("Failed to get company ratios");
        println!("{report:#?}");
    }
}
