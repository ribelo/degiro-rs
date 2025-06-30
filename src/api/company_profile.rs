use crate::{
    client::Degiro,
    error::ClientError,
    http::{HttpClient, HttpRequest},
    models::CompanyProfile,
    paths::{BASE_API_URL, COMPANY_PROFILE_PATH},
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
        let url = format!("{}{}{}", BASE_API_URL, COMPANY_PROFILE_PATH, isin.as_ref());
        
        let mut json = self.request_json(
            HttpRequest::get(url)
                .query("intAccount", self.int_account().to_string())
                .query("sessionId", self.session_id())
                .header("Content-Type", "application/json")
        ).await?;
        
        let data = json["data"].take();
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
    #[ignore = "Integration test - hits real API"]
    async fn test_company_profile() {
        let client = Degiro::load_from_env().expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
        client.account_config().await.expect("Failed to get account configuration");
        let profile = client.company_profile_by_id("332111").await.expect("Failed to get company profile");
        dbg!(&profile);
    }
}
