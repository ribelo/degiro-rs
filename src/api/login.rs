use crate::{
    client::Degiro,
    error::{ClientError, AuthError},
    http::{HttpClient, HttpRequest},
    session::AuthState,
    paths::{BASE_API_URL, LOGIN_URL_PATH},
};

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
        .ok_or(AuthError::InvalidTotpSecret("Invalid base32 encoding".to_string()))?;

        let totp = totp_rs::TOTP::new(totp_rs::Algorithm::SHA1, 6, 1, 30, decoded_secret)
            .map_err(|e| AuthError::TotpGenerationFailed(e.to_string()))?;

        let token = totp
            .generate_current()
            .map_err(|e| AuthError::TotpGenerationFailed(e.to_string()))?;

        Ok(token)
    }

    pub async fn login(&self) -> Result<(), ClientError> {
        let totp = self.get_totp()?;
        
        let url = format!("{BASE_API_URL}{LOGIN_URL_PATH}totp");
        
        let login_body = json!({
            "isPassCodeReset": false,
            "isRedirectToMobile": false,
            "password": self.password,
            "username": self.username,
            "oneTimePassword": totp,
        });
        
        let body = self.request::<LoginResponse>(
            HttpRequest::post(url)
                .query("reason", "session_expired")
                .header("Content-Type", "application/json")
                .json(&login_body)?
                .no_auth()
        ).await?;

        let session_id = match body.session_id {
            Some(id) => id,
            None => return Err(ClientError::Unauthorized),
        };

        self.set_session_id(session_id);
        self.set_auth_state(AuthState::Restricted)?;
        
        // Save session after successful login
        if let Err(e) = self.save_session() {
            // Don't fail the login if we can't save the session
            tracing::warn!("Failed to save session after login: {}", e);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    #[ignore = "Integration test - hits real API"]
    async fn test_login() {
        let client = Degiro::load_from_env().expect("Failed to load Degiro client from environment variables");
        client.login().await.expect("Failed to login to Degiro");
    }

    #[test]
    fn test_totp() {
        let client = Degiro::load_from_env().expect("Failed to load Degiro client from environment variables");
        let totp = client.get_totp().expect("Failed to generate TOTP token");
        println!("TOTP: {totp}");
    }
}
