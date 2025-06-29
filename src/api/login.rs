use crate::{
    client::{ApiErrorResponse, ClientError, ClientStatus, Degiro},
    paths::{LOGIN_URL_PATH, REFERER},
};

use mime;
use reqwest::{header, Url};
use serde::Deserialize;
use serde_json::json;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    locale: Option<String>,
    session_id: Option<String>,
    status: i32,
    status_text: String,
}

impl Degiro {
    pub async fn authorize(&self) -> Result<(), ClientError> {
        self.login().await?;
        self.account_config().await?;
        Ok(())
    }

    pub fn get_totp(&self) -> Result<String, ClientError> {
        let decoded_secret = base32::decode(
            base32::Alphabet::Rfc4648 { padding: false },
            &self.totp_secret,
        )
        .ok_or(ClientError::UnexpectedError("Invalid base32".to_string()))?;

        let totp = totp_rs::TOTP::new(totp_rs::Algorithm::SHA1, 6, 1, 30, decoded_secret)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

        let token = totp
            .generate_current()
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

        Ok(token)
    }

    pub async fn login(&self) -> Result<(), ClientError> {
        let totp = self.get_totp()?;

        // Build login URL
        let url = Url::parse(crate::paths::BASE_API_URL)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join(LOGIN_URL_PATH)
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?
            .join("totp")
            .map_err(|e| ClientError::UnexpectedError(e.to_string()))?;

        let body = json!({
            "isPassCodeReset": false,
            "isRedirectToMobile": false,
            "password": self.password,
            "username": self.username,
            "oneTimePassword": totp,
        });

        self.acquire_limit().await;

        let res = self
            .http_client
            .post(url)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.to_string())
            .header(header::REFERER, REFERER)
            .json(&body)
            .query(&[("reason", "session_expired")])
            .send()
            .await?;

        if res.error_for_status_ref().is_err() {
            let text = res.text().await?;
            let error_response: ApiErrorResponse = serde_json::from_str(&text)?;
            return Err(ClientError::ApiError(error_response));
        }

        let body = res.json::<LoginResponse>().await?;

        let session_id = match body.session_id {
            Some(id) => id,
            None => return Err(ClientError::Unauthorized),
        };

        self.set_session_id(session_id);
        self.set_auth_state(ClientStatus::Restricted);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_login() {
        let client = Degiro::new_from_env();
        client.login().await.unwrap();
    }

    #[test]
    fn test_totp() {
        let client = Degiro::new_from_env();
        let totp = client.get_totp().unwrap();
        println!("TOTP: {}", totp);
    }
}
