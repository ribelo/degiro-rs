use crate::client::{Client, ClientError};

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

impl Client {
    pub async fn login(&self) -> Result<(), ClientError> {
        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = &inner.base_api_url;
            let path_url = "login/secure/login";

            let url = Url::parse(base_url)
                .unwrap_or_else(|_| panic!("can't parse base_url: {base_url}"))
                .join(path_url)
                .unwrap_or_else(|_| panic!("can't join path_url: {path_url}"));
            let body = json!({
                "isPassCodeReset": false,
                "isRedirectToMobile": false,
                "password": inner.password,
                "username": inner.username,
            });

            inner
                .http_client
                .post(url)
                .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.to_string())
                .header(header::REFERER, &inner.referer)
                .json(&body)
                .query(&[("reason", "session_expired")])
        };

        let rate_limiter = {
            let inner = self.inner.lock().unwrap();
            inner.rate_limiter.clone()
        };
        rate_limiter.acquire_one().await;

        let res = req.send().await?;

        match res.error_for_status() {
            Ok(res) => {
                let body = res.json::<LoginResponse>().await.expect("can't parse json");

                {
                    let mut inner = self.inner.lock().unwrap();
                    inner.session_id = body.session_id.unwrap();
                };

                Ok(())
            }
            Err(err) => Err(ClientError::LoginError {
                source: Box::new(err),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn login() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        dbg!(&client);
    }
}
