use chrono::NaiveDate;
use reqwest::{header, Url};
use serde::{Deserialize, Serialize};

use crate::client::{Client, ClientError};

#[derive(Debug, Serialize, Default)]
pub struct FinancialReports {
    pub currency: String,
    pub annual: Reports,
    pub interim: Reports,
}

#[derive(Debug, Serialize, Default)]
pub struct Report {
    pub fiscal_year: i32,
    pub end_date: NaiveDate,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Serialize)]
pub enum Statement {
    IncomeStatement(Box<IncomeStatement>),
    BalanceSheet(Box<BalanceSheet>),
    CashFlow(Box<CashFlow>),
}

#[derive(Debug, Serialize, Default)]
pub struct Reports(Vec<Report>);

impl From<Vec<Report>> for Reports {
    fn from(reports: Vec<Report>) -> Self {
        Self(reports)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct IncomeStatement {
    pub source: String,
    pub period_type: String,
    pub period_length: i32,
    pub srev: ItemDetail, // Revenue
    pub rtlr: ItemDetail, // Total Revenue
    pub scor: ItemDetail, // Cost of Revenue, Total
    pub sgrp: ItemDetail, // Gross Profit
    pub ssga: ItemDetail, // Selling/General/Admin. Expenses, Total
    pub sdpr: ItemDetail, // Depreciation/Amortization
    pub suie: ItemDetail, // Unusual Expense (Income)
    pub etoe: ItemDetail, // Total Operating Expense
    pub sopi: ItemDetail, // Operating Income
    pub snin: ItemDetail, // Interest Inc.(Exp.),Net-Non-Op., Total
    pub sont: ItemDetail, // Other, Net
    pub eibt: ItemDetail, // Net Income Before Taxes
    pub ttax: ItemDetail, // Provision for Income Taxes
    pub tiat: ItemDetail, // Net Income After Taxes
    pub cmin: ItemDetail, // Minority Interest
    pub nibx: ItemDetail, // Net Income Before Extra. Items
    pub ninc: ItemDetail, // Net Income
    pub ciac: ItemDetail, // Income Available to Com Excl ExtraOrd
    pub xnic: ItemDetail, // Income Available to Com Incl ExtraOrd
    pub sdni: ItemDetail, // Diluted Net Income
    pub sdws: ItemDetail, // Diluted Weighted Average Shares
    pub sdbf: ItemDetail, // Diluted EPS Excluding ExtraOrd Items
    pub ddps: ItemDetail, // DPS - Common Stock Primary Issue
    pub vdes: ItemDetail, // Diluted Normalized EPS
}

impl From<&serde_json::Value> for IncomeStatement {
    fn from(value: &serde_json::Value) -> Self {
        let mut income_statement = IncomeStatement::default();
        for item in value.as_array().unwrap() {
            let code = item["code"].as_str().unwrap().to_lowercase();
            match code.as_str() {
                "srev" => income_statement.srev = item.into(),
                "rtlr" => income_statement.rtlr = item.into(),
                "scor" => income_statement.scor = item.into(),
                "sgrp" => income_statement.sgrp = item.into(),
                "ssga" => income_statement.ssga = item.into(),
                "sdpr" => income_statement.sdpr = item.into(),
                "suie" => income_statement.suie = item.into(),
                "etoe" => income_statement.etoe = item.into(),
                "sopi" => income_statement.sopi = item.into(),
                "snin" => income_statement.snin = item.into(),
                "sont" => income_statement.sont = item.into(),
                "eibt" => income_statement.eibt = item.into(),
                "ttax" => income_statement.ttax = item.into(),
                "tiat" => income_statement.tiat = item.into(),
                "cmin" => income_statement.cmin = item.into(),
                "nibx" => income_statement.nibx = item.into(),
                "ninc" => income_statement.ninc = item.into(),
                "ciac" => income_statement.ciac = item.into(),
                "xnic" => income_statement.xnic = item.into(),
                "sdni" => income_statement.sdni = item.into(),
                "sdws" => income_statement.sdws = item.into(),
                "sdbf" => income_statement.sdbf = item.into(),
                "ddps1" => income_statement.ddps = item.into(),
                "vdes" => income_statement.vdes = item.into(),
                _ => {}
            }
        }
        income_statement
    }
}

// Balance Sheet (BAL)
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub struct BalanceSheet {
    pub source: String,
    pub period_type: String,
    pub period_length: i32,
    pub acae: ItemDetail, // Cash & Equivalents
    pub scsi: ItemDetail, // Cash and Short Term Investments
    pub aacr: ItemDetail, // Accounts Receivable - Trade, Net
    pub atrc: ItemDetail, // Total Receivables, Net
    pub aitl: ItemDetail, // Total Inventory
    pub appy: ItemDetail, // Prepaid Expenses
    pub soca: ItemDetail, // Other Current Assets, Total
    pub atca: ItemDetail, // Total Current Assets
    pub aptc: ItemDetail, // Property/Plant/Equipment, Total - Gross
    pub adep: ItemDetail, // Accumulated Depreciation, Total
    pub appn: ItemDetail, // Property/Plant/Equipment, Total - Net
    pub agwi: ItemDetail, // Goodwill, Net
    pub aint: ItemDetail, // Intangibles, Net
    pub sola: ItemDetail, // Other Long Term Assets, Total
    pub atot: ItemDetail, // Total Assets
    pub lapb: ItemDetail, // Accounts Payable
    pub laex: ItemDetail, // Accrued Expenses
    pub lstd: ItemDetail, // Notes Payable/Short Term Debt
    pub lcld: ItemDetail, // Current Port. of LT Debt/Capital Leases
    pub socl: ItemDetail, // Other Current liabilities, Total
    pub ltcl: ItemDetail, // Total Current Liabilities
    pub lltd: ItemDetail, // Long Term Debt
    pub lclo: ItemDetail, // Capital Lease Obligations
    pub lttd: ItemDetail, // Total Long Term Debt
    pub stld: ItemDetail, // Total Debt
    pub sbdt: ItemDetail, // Deferred Income Tax
    pub lmin: ItemDetail, // Minority Interest
    pub sltl: ItemDetail, // Other Liabilities, Total
    pub ltll: ItemDetail, // Total Liabilities
    pub sprs: ItemDetail, // Preferred Stock - Non Redeemable, Net
    pub scms: ItemDetail, // Common Stock, Total
    pub qred: ItemDetail, // Retained Earnings (Accumulated Deficit)
    pub qtsc: ItemDetail, // Treasury Stock - Common
    pub sote: ItemDetail, // Other Equity, Total
    pub qtle: ItemDetail, // Total Equity
    pub qtel: ItemDetail, // Total Liabilities & Shareholders' Equity
    pub qtco: ItemDetail, // Total Common Shares Outstanding
    pub stbp: ItemDetail, // Tangible Book Value per Share, Common Eq
}

impl From<&serde_json::Value> for BalanceSheet {
    fn from(value: &serde_json::Value) -> Self {
        let mut balance_sheet = BalanceSheet::default();
        for item in value.as_array().unwrap() {
            let code = item["code"].as_str().unwrap().to_lowercase();
            match code.as_str() {
                "acae" => balance_sheet.acae = item.into(),
                "scsi" => balance_sheet.scsi = item.into(),
                "aacr" => balance_sheet.aacr = item.into(),
                "atrc" => balance_sheet.atrc = item.into(),
                "aitl" => balance_sheet.aitl = item.into(),
                "appy" => balance_sheet.appy = item.into(),
                "soca" => balance_sheet.soca = item.into(),
                "atca" => balance_sheet.atca = item.into(),
                "aptc" => balance_sheet.aptc = item.into(),
                "adep" => balance_sheet.adep = item.into(),
                "appn" => balance_sheet.appn = item.into(),
                "agwi" => balance_sheet.agwi = item.into(),
                "aint" => balance_sheet.aint = item.into(),
                "sola" => balance_sheet.sola = item.into(),
                "atot" => balance_sheet.atot = item.into(),
                "lapb" => balance_sheet.lapb = item.into(),
                "laex" => balance_sheet.laex = item.into(),
                "lstd" => balance_sheet.lstd = item.into(),
                "lcld" => balance_sheet.lcld = item.into(),
                "socl" => balance_sheet.socl = item.into(),
                "ltcl" => balance_sheet.ltcl = item.into(),
                "lltd" => balance_sheet.lltd = item.into(),
                "lclo" => balance_sheet.lclo = item.into(),
                "lttd" => balance_sheet.lttd = item.into(),
                "stld" => balance_sheet.stld = item.into(),
                "sbdt" => balance_sheet.sbdt = item.into(),
                "lmin" => balance_sheet.lmin = item.into(),
                "sltl" => balance_sheet.sltl = item.into(),
                "ltll" => balance_sheet.ltll = item.into(),
                "sprs" => balance_sheet.sprs = item.into(),
                "scms" => balance_sheet.scms = item.into(),
                "qred" => balance_sheet.qred = item.into(),
                "qtsc" => balance_sheet.qtsc = item.into(),
                "sote" => balance_sheet.sote = item.into(),
                "qtle" => balance_sheet.qtle = item.into(),
                "qtel" => balance_sheet.qtel = item.into(),
                "qtco" => balance_sheet.qtco = item.into(),
                "stbp" => balance_sheet.stbp = item.into(),
                _ => {}
            }
        }
        balance_sheet
    }
}

// Cash Flow (CAS)
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CashFlow {
    pub source: String,
    pub period_type: String,
    pub period_length: i32,
    pub onet: ItemDetail, // Net Income/Starting Line
    pub sded: ItemDetail, // Depreciation/Depletion
    pub obdt: ItemDetail, // Deferred Taxes
    pub snci: ItemDetail, // Non-Cash Items
    pub sctp: ItemDetail, // Cash Taxes Paid
    pub scip: ItemDetail, // Cash Interest Paid
    pub socf: ItemDetail, // Changes in Working Capital
    pub otlo: ItemDetail, // Cash from Operating Activities
    pub scex: ItemDetail, // Capital Expenditures
    pub sicf: ItemDetail, // Other Investing Cash Flow Items, Total
    pub itli: ItemDetail, // Cash from Investing Activities
    pub sfcf: ItemDetail, // Financing Cash Flow Items
    pub fcdp: ItemDetail, // Total Cash Dividends Paid
    pub fpss: ItemDetail, // Issuance (Retirement) of Stock, Net
    pub fprd: ItemDetail, // Issuance (Retirement) of Debt, Net
    pub ftlf: ItemDetail, // Cash from Financing Activities
    pub sfee: ItemDetail, // Foreign Exchange Effects
    pub sncc: ItemDetail, // Net Change in Cash
}

impl From<&serde_json::Value> for CashFlow {
    fn from(value: &serde_json::Value) -> Self {
        let mut cash_flow = CashFlow::default();
        for item in value.as_array().unwrap() {
            let code = item["code"].as_str().unwrap().to_lowercase();
            match code.as_str() {
                "onet" => cash_flow.onet = item.into(),
                "sded" => cash_flow.sded = item.into(),
                "obdt" => cash_flow.obdt = item.into(),
                "snci" => cash_flow.snci = item.into(),
                "sctp" => cash_flow.sctp = item.into(),
                "scip" => cash_flow.scip = item.into(),
                "socf" => cash_flow.socf = item.into(),
                "otlo" => cash_flow.otlo = item.into(),
                "scex" => cash_flow.scex = item.into(),
                "sicf" => cash_flow.sicf = item.into(),
                "itli" => cash_flow.itli = item.into(),
                "sfcf" => cash_flow.sfcf = item.into(),
                "fcdp" => cash_flow.fcdp = item.into(),
                "fpss" => cash_flow.fpss = item.into(),
                "fprd" => cash_flow.fprd = item.into(),
                "ftlf" => cash_flow.ftlf = item.into(),
                "sfee" => cash_flow.sfee = item.into(),
                "sncc" => cash_flow.sncc = item.into(),
                _ => {}
            }
        }
        cash_flow
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ItemDetail {
    pub meaning: String,
    pub value: f64,
}

impl From<&serde_json::Value> for ItemDetail {
    fn from(value: &serde_json::Value) -> Self {
        ItemDetail {
            meaning: value["meaning"].as_str().unwrap().to_string(),
            value: value["value"].as_f64().unwrap(),
        }
    }
}

fn process_reports(data: &serde_json::Value) -> Result<Vec<Report>, ClientError> {
    data.as_array()
        .map(|reports| {
            reports
                .iter()
                .map(|report_data| {
                    let fiscal_year = report_data["fiscalYear"]
                        .as_i64()
                        .ok_or(ClientError::ParseError("Can't get fiscalYear".to_string()))?
                        as i32;
                    let end_date = NaiveDate::parse_from_str(
                        report_data["endDate"]
                            .as_str()
                            .ok_or(ClientError::ParseError("Can't get endDate".to_string()))?,
                        "%Y-%m-%d",
                    )
                    .map_err(|err| ClientError::ParseError(err.to_string()))?;

                    let statements = report_data["statements"]
                        .as_array()
                        .ok_or(ClientError::ParseError("Can't get statements".to_string()))?
                        .iter()
                        .map(|statement| {
                            match statement["type"].as_str().ok_or(ClientError::ParseError(
                                "Can't get statement type".to_string(),
                            ))? {
                                "INC" => Ok(Statement::IncomeStatement(Box::new(
                                    (&statement["items"]).into(),
                                ))),
                                "BAL" => Ok(Statement::BalanceSheet(Box::new(
                                    (&statement["items"]).into(),
                                ))),
                                "CAS" => {
                                    Ok(Statement::CashFlow(Box::new((&statement["items"]).into())))
                                }
                                code => Err(ClientError::UnexpectedStatementType(code.to_string())),
                            }
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(Report {
                        fiscal_year,
                        end_date,
                        statements,
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or(Err(ClientError::ParseError("Can't get data".to_string())))
}

impl Client {
    pub async fn financial_statements_by_id(
        &self,
        id: &str,
    ) -> Result<FinancialReports, ClientError> {
        let isin = &self.product(id).await?.inner.isin;
        dbg!(isin);
        self.financial_statements(isin).await
    }
    pub async fn financial_statements(&self, isin: &str) -> Result<FinancialReports, ClientError> {
        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = "https://trader.degiro.nl/";
            let path_url = "dgtbxdsservice/financial-statements/";
            // https://trader.degiro.nl/dgtbxdsservice/financial-statements/US14149Y1082?intAccount=71003134&sessionId=3373A4DF798147194546B8361D4D8387.prod_b_125_2
            let url = Url::parse(base_url)
                .unwrap()
                .join(path_url)
                .unwrap()
                .join(isin)
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
                let json = res.json::<serde_json::Value>().await?;
                let data = &json["data"];

                let currency = data["currency"]
                    .as_str()
                    .ok_or(ClientError::ParseError("Can't get currency".to_string()))?
                    .to_string();
                let annual_reports = process_reports(&data["annual"])?;
                let interim_reports = process_reports(&data["interim"])?;

                let financial_report = FinancialReports {
                    currency,
                    annual: annual_reports.into(),
                    interim: interim_reports.into(),
                };

                Ok(financial_report)
            }
            Err(err) => {
                eprintln!("error: {}", err);
                Err(err.into())
            }
        }
    }
}

#[cfg(test)]
mod test {

    use crate::client::Client;

    #[tokio::test]
    async fn financial_statements() {
        let client = Client::new_from_env();
        client.login().await.unwrap();
        client.account_config().await.unwrap();

        let report = client.financial_statements_by_id("15850348").await.unwrap();
        println!("{:#?}", report);
    }
}
