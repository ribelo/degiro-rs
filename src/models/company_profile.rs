use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyProfile {
    #[serde(default)]
    pub isin: String,
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
            writeln!(f, "{mgmt}")?;
            writeln!(f)?;
        }

        writeln!(f, "\nIssues:")?;
        for issue in &self.issues {
            writeln!(f, "{issue}")?;
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

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Contacts {
    pub name: String,
    pub address: Option<String>,
    pub postcode: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub telephone: Option<String>,
    pub fax: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub stateregion: Option<String>,
}

impl fmt::Display for Contacts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        if let Some(address) = &self.address {
            writeln!(f, "Address: {address}")?;
        }
        if let Some(postcode) = &self.postcode {
            writeln!(f, "Postcode: {postcode}")?;
        }
        if let Some(city) = &self.city {
            writeln!(f, "City: {city}")?;
        }
        if let Some(country) = &self.country {
            writeln!(f, "Country: {country}")?;
        }
        if let Some(telephone) = &self.telephone {
            writeln!(f, "Telephone: {telephone}")?;
        };
        if let Some(fax) = &self.fax {
            writeln!(f, "Fax: {fax}")?;
        };
        if let Some(email) = &self.email {
            writeln!(f, "Email: {email}")?;
        };
        if let Some(website) = &self.website {
            writeln!(f, "Website: {website}")?;
        };
        if let Some(stateregion) = &self.stateregion {
            write!(f, "State/Region: {stateregion}")?;
        };
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            writeln!(f, "Age: {age}")?;
        }
        if let Some(since) = &self.since {
            writeln!(f, "Since: {since}")?;
        }
        if let Some(title_start) = &self.title_start {
            writeln!(f, "Title Start: {title_start}")?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            writeln!(f, "Name: {name}")?;
        }
        if let Some(ticker) = &self.ticker {
            writeln!(f, "Ticker: {ticker}")?;
        }
        if let Some(exchange) = &self.exchange {
            writeln!(f, "Exchange: {exchange}")?;
        }
        if let Some(description) = &self.description {
            writeln!(f, "Description: {description}")?;
        }
        if let Some(most_recent_split_value) = &self.most_recent_split_value {
            writeln!(f, "Most Recent Split Value: {most_recent_split_value}")?;
        }
        if let Some(most_recent_split_date) = &self.most_recent_split_date {
            writeln!(f, "Most Recent Split Date: {most_recent_split_date}")?;
        }
        Ok(())
    }
}
