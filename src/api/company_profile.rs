use core::fmt;

use reqwest::{header, Url};
use serde::{Deserialize, Serialize};

use crate::client::{Client, ClientError, ClientStatus};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyProfile {
    pub contacts: Contacts,
    pub management: Vec<Management>,
    pub issues: Vec<Issue>,
    pub sector: String,
    pub industry: String,
    pub employees: i64,
    pub business_summary: String,
    pub financial_summary: String,
    pub business_summary_last_modified: String,
    pub financial_summary_last_modified: String,
    pub shr_floating: String,
    pub shr_outstanding: String,
    pub la_interim_data: String,
    pub la_annual_data: String,
    pub lu_employees: String,
    pub lu_shares: String,
    pub last_updated: String,
    pub us_irs_no: Option<String>,
    pub us_cik_no: Option<String>,
    pub currency: String,
}

impl fmt::Display for CompanyProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Contacts:")?;
        writeln!(f, "{}", self.contacts)?;

        writeln!(f, "\nManagement:")?;
        for mgmt in &self.management {
            writeln!(f, "{}", mgmt)?;
            writeln!(f)?;
        }

        writeln!(f, "\nIssues:")?;
        for issue in &self.issues {
            writeln!(f, "{}", issue)?;
            writeln!(f)?;
        }

        writeln!(f, "\nSector: {}", self.sector)?;
        writeln!(f, "Industry: {}", self.industry)?;
        writeln!(f, "Employees: {}", self.employees)?;

        writeln!(f, "\nBusiness Summary:")?;
        writeln!(f, "{}", self.business_summary)?;

        writeln!(f, "\nFinancial Summary:")?;
        writeln!(f, "{}", self.financial_summary)?;

        writeln!(
            f,
            "\nBusiness Summary Last Modified: {}",
            self.business_summary_last_modified
        )?;
        writeln!(
            f,
            "Financial Summary Last Modified: {}",
            self.financial_summary_last_modified
        )?;

        writeln!(f, "\nShares Floating: {}", self.shr_floating)?;
        writeln!(f, "Shares Outstanding: {}", self.shr_outstanding)?;

        writeln!(f, "\nLatest Interim Data: {}", self.la_interim_data)?;
        writeln!(f, "Latest Annual Data: {}", self.la_annual_data)?;
        writeln!(f, "Latest Employees Update: {}", self.lu_employees)?;
        writeln!(f, "Latest Shares Update: {}", self.lu_shares)?;

        writeln!(f, "\nLast Updated: {}", self.last_updated)?;

        writeln!(
            f,
            "\nUS IRS Number: {}",
            self.us_irs_no.as_deref().unwrap_or("Not Available")
        )?;
        writeln!(
            f,
            "US CIK Number: {}",
            self.us_cik_no.as_deref().unwrap_or("Not Available")
        )?;

        write!(f, "\nCurrency: {}", self.currency)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Contacts {
    pub name: String,
    pub address: String,
    pub postcode: String,
    pub city: String,
    pub country: String,
    pub telephone: Option<String>,
    pub fax: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub stateregion: Option<String>,
}

impl fmt::Display for Contacts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Address: {}", self.address)?;
        writeln!(f, "Postcode: {}", self.postcode)?;
        writeln!(f, "City: {}", self.city)?;
        writeln!(f, "Country: {}", self.country)?;
        if let Some(telephone) = &self.telephone {
            writeln!(f, "Telephone: {}", telephone)?;
        };
        if let Some(fax) = &self.fax {
            writeln!(f, "Fax: {}", fax)?;
        };
        if let Some(email) = &self.email {
            writeln!(f, "Email: {}", email)?;
        };
        if let Some(website) = &self.website {
            writeln!(f, "Website: {}", website)?;
        };
        if let Some(stateregion) = &self.stateregion {
            write!(f, "State/Region: {}", stateregion)?;
        };
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Management {
    pub name: String,
    pub function: String,
    pub long_function: String,
    pub age: Option<i64>,
    pub since: Option<String>,
    pub title_start: Option<String>,
}

impl fmt::Display for Management {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Function: {}", self.function)?;
        writeln!(f, "Long Function: {}", self.long_function)?;
        if let Some(age) = &self.age {
            writeln!(f, "Age: {}", age)?;
        }
        if let Some(since) = &self.since {
            writeln!(f, "Since: {}", since)?;
        }
        if let Some(title_start) = &self.title_start {
            writeln!(f, "Title Start: {}", title_start)?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: i64,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub exchange: Option<String>,
    pub description: Option<String>,
    pub most_recent_split_value: Option<String>,
    pub most_recent_split_date: Option<String>,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ID: {}", self.id)?;
        if let Some(name) = &self.name {
            writeln!(f, "Name: {}", name)?;
        }
        if let Some(ticker) = &self.ticker {
            writeln!(f, "Ticker: {}", ticker)?;
        }
        if let Some(exchange) = &self.exchange {
            writeln!(f, "Exchange: {}", exchange)?;
        }
        if let Some(description) = &self.description {
            writeln!(f, "Description: {}", description)?;
        }
        if let Some(most_recent_split_value) = &self.most_recent_split_value {
            writeln!(f, "Most Recent Split Value: {}", most_recent_split_value)?;
        }
        if let Some(most_recent_split_date) = &self.most_recent_split_date {
            writeln!(f, "Most Recent Split Date: {}", most_recent_split_date)?;
        }
        Ok(())
    }
}

impl Client {
    pub async fn company_profile_by_id<T: AsRef<str>>(
        &self,
        id: T,
    ) -> Result<CompanyProfile, ClientError> {
        let isin = &self.product(id.as_ref()).await?.inner.isin;
        self.company_profile(isin).await
    }
    pub async fn company_profile(
        &self,
        isin: impl AsRef<str>,
    ) -> Result<CompanyProfile, ClientError> {
        if self.inner.lock().unwrap().status != ClientStatus::Authorized {
            return Err(ClientError::Unauthorized);
        }
        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = "https://trader.degiro.nl/";
            let path_url = "dgtbxdsservice/company-profile/v2/";
            let url = Url::parse(base_url)
                .unwrap()
                .join(path_url)
                .unwrap()
                .join(isin.as_ref())
                .unwrap();

            inner
                .http_client
                .get(url)
                .query(&[
                    ("intAccount", &inner.int_account.to_string()),
                    ("sessionId", &inner.session_id),
                ])
                .header(header::REFERER, &inner.referer)
                .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.to_string())
        };

        let rate_limiter = {
            let inner = self.inner.lock().unwrap();
            inner.rate_limiter.clone()
        };
        rate_limiter.acquire_one().await;

        let res = req.send().await?;

        match res.error_for_status() {
            Ok(res) => {
                let mut json = res.json::<serde_json::Value>().await?;
                let mut data = json["data"].take();
                if data.is_null() {
                    return Err(ClientError::NoData);
                }

                let company_profile = serde_json::from_value::<CompanyProfile>(data.take())?;

                Ok(company_profile)
            }
            Err(err) => {
                eprintln!("error: {}", err);
                Err(err.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::Client;

    #[tokio::test]
    async fn test_company_profile_success() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();
        let profile = client.company_profile_by_id("332111").await.unwrap();
        println!("{profile}");
    }
}
