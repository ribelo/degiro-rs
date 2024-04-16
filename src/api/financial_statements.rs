use chrono::NaiveDate;
use reqwest::{header, Url};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::client::{Client, ClientError, ClientStatus};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FinancialReports {
    pub id: String,
    pub currency: String,
    pub annual: Reports,
    pub interim: Reports,
}

impl FinancialReports {
    pub fn get_annual(&self, fiscal_year: i32) -> Option<&Report> {
        self.annual
            .get(fiscal_year)
            .or_else(|| self.interim.get(fiscal_year))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub fiscal_year: i32,
    pub end_date: NaiveDate,
    pub income_report: IncomeStatementReport,
    pub balance_sheet: BalanceSheetReport,
    pub cash_flow: CashFlowReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IncomeStatementReport {
    pub source: String,
    pub period_type: String,
    pub period_length: i32,
    pub statement: Box<IncomeStatement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BalanceSheetReport {
    pub source: String,
    pub statement: Box<BalanceSheet>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CashFlowReport {
    pub source: String,
    pub period_type: String,
    pub period_length: i32,
    pub statement: Box<CashFlow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Reports(Vec<Report>);

impl From<Vec<Report>> for Reports {
    fn from(reports: Vec<Report>) -> Self {
        Self(reports)
    }
}

impl Reports {
    pub fn get(&self, fiscal_year: i32) -> Option<&Report> {
        self.0
            .iter()
            .find(|report| report.fiscal_year == fiscal_year)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IncomeStatement {
    /// Revenue
    pub srev: ItemDetail,
    /// Total Revenue
    pub rtlr: ItemDetail,
    /// Cost of Revenue, Total
    pub scor: ItemDetail,
    /// Gross Profit
    pub sgrp: ItemDetail,
    /// Selling/General/Admin. Expenses, Total
    pub ssga: ItemDetail,
    /// Depreciation/Amortization
    pub sdpr: ItemDetail,
    /// Unusual Expense (Income)
    pub suie: ItemDetail,
    /// Total Operating Expense
    pub etoe: ItemDetail,
    /// Operating Income
    pub sopi: ItemDetail,
    /// Interest Inc.(Exp.),Net-Non-Op., Total
    pub snin: ItemDetail,
    /// Other, Net
    pub sont: ItemDetail,
    /// Net Income Before Taxes
    pub eibt: ItemDetail,
    /// Provision for Income Taxes
    pub ttax: ItemDetail,
    /// Net Income After Taxes
    pub tiat: ItemDetail,
    /// Minority Interest
    pub cmin: ItemDetail,
    /// Net Income Before Extra. Items
    pub nibx: ItemDetail,
    /// Net Income
    pub ninc: ItemDetail,
    /// Income Available to Com Excl ExtraOrd
    pub ciac: ItemDetail,
    /// Income Available to Com Incl ExtraOrd
    pub xnic: ItemDetail,
    /// Diluted Net Income
    pub sdni: ItemDetail,
    /// Diluted Weighted Average Shares
    pub sdws: ItemDetail,
    /// Diluted EPS Excluding ExtraOrd Items
    pub sdbf: ItemDetail,
    /// DPS - Common Stock Primary Issue
    pub ddps: ItemDetail,
    /// Diluted Normalized EPS
    pub vdes: ItemDetail,
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub struct BalanceSheet {
    pub source: String,
    pub period_type: String,
    pub period_length: i32,

    /// Cash & Equivalents
    pub acae: ItemDetail,
    /// Cash and Short Term Investments
    pub scsi: ItemDetail,
    /// Accounts Receivable - Trade, Net
    pub aacr: ItemDetail,
    /// Total Receivables, Net
    pub atrc: ItemDetail,
    /// Total Inventory
    pub aitl: ItemDetail,
    /// Prepaid Expenses
    pub appy: ItemDetail,
    /// Other Current Assets, Total
    pub soca: ItemDetail,
    /// Total Current Assets
    pub atca: ItemDetail,
    /// Property/Plant/Equipment, Total - Gross
    pub aptc: ItemDetail,
    /// Accumulated Depreciation, Total
    pub adep: ItemDetail,
    /// Property/Plant/Equipment, Total - Net
    pub appn: ItemDetail,
    /// Goodwill, Net
    pub agwi: ItemDetail,
    /// Intangibles, Net
    pub aint: ItemDetail,
    /// Other Long Term Assets, Total
    pub sola: ItemDetail,
    /// Total Assets
    pub atot: ItemDetail,
    /// Accounts Payable
    pub lapb: ItemDetail,
    /// Accrued Expenses
    pub laex: ItemDetail,
    /// Notes Payable/Short Term Debt
    pub lstd: ItemDetail,
    /// Current Port. of LT Debt/Capital Leases
    pub lcld: ItemDetail,
    /// Other Current liabilities, Total
    pub socl: ItemDetail,
    /// Total Current Liabilities
    pub ltcl: ItemDetail,
    /// Long Term Debt
    pub lltd: ItemDetail,
    /// Capital Lease Obligations
    pub lclo: ItemDetail,
    /// Total Long Term Debt
    pub lttd: ItemDetail,
    /// Total Debt
    pub stld: ItemDetail,
    /// Deferred Income Tax
    pub sbdt: ItemDetail,
    /// Minority Interest
    pub lmin: ItemDetail,
    /// Other Liabilities, Total
    pub sltl: ItemDetail,
    /// Total Liabilities
    pub ltll: ItemDetail,
    /// Preferred Stock - Non Redeemable, Net
    pub sprs: ItemDetail,
    /// Common Stock, Total
    pub scms: ItemDetail,
    /// Retained Earnings (Accumulated Deficit)
    pub qred: ItemDetail,
    /// Treasury Stock - Common
    pub qtsc: ItemDetail,
    /// Other Equity, Total
    pub sote: ItemDetail,
    /// Total Equity
    pub qtle: ItemDetail,
    /// Total Liabilities & Shareholders' Equity
    pub qtel: ItemDetail,
    /// Total Common Shares Outstanding
    pub qtco: ItemDetail,
    /// Tangible Book Value per Share, Common Eq
    pub stbp: ItemDetail,
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CashFlow {
    pub source: String,
    pub period_type: String,
    pub period_length: i32,
    /// Net Income/Starting Line
    pub onet: ItemDetail,
    /// Depreciation/Depletion
    pub sded: ItemDetail,
    /// Deferred Taxes
    pub obdt: ItemDetail,
    /// Non-Cash Items
    pub snci: ItemDetail,
    /// Cash Taxes Paid
    pub sctp: ItemDetail,
    /// Cash Interest Paid
    pub scip: ItemDetail,
    /// Changes in Working Capital
    pub socf: ItemDetail,
    /// Cash from Operating Activities
    pub otlo: ItemDetail,
    /// Capital Expenditures
    pub scex: ItemDetail,
    /// Other Investing Cash Flow Items, Total
    pub sicf: ItemDetail,
    /// Cash from Investing Activities
    pub itli: ItemDetail,
    /// Financing Cash Flow Items
    pub sfcf: ItemDetail,
    /// Total Cash Dividends Paid
    pub fcdp: ItemDetail,
    /// Issuance (Retirement) of Stock, Net
    pub fpss: ItemDetail,
    /// Issuance (Retirement) of Debt, Net
    pub fprd: ItemDetail,
    /// Cash from Financing Activities
    pub ftlf: ItemDetail,
    /// Foreign Exchange Effects
    pub sfee: ItemDetail,
    /// Net Change in Cash
    pub sncc: ItemDetail,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

                    let mut report = Report {
                        fiscal_year,
                        end_date,
                        ..Default::default()
                    };

                    for statement in report_data["statements"]
                        .as_array()
                        .ok_or(ClientError::ParseError("Can't get statements".to_string()))?
                        .iter()
                    {
                        match statement["type"].as_str().ok_or(ClientError::ParseError(
                            "Can't get statement type".to_string(),
                        ))? {
                            "INC" => {
                                let income_report = IncomeStatementReport {
                                    source: statement["source"].as_str().unwrap().to_string(),
                                    period_type: statement["periodType"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    period_length: statement["periodLength"].as_i64().unwrap()
                                        as i32,
                                    statement: Box::new((&statement["items"]).into()),
                                };
                                report.income_report = income_report;
                            }
                            "BAL" => {
                                let balance_report = BalanceSheetReport {
                                    source: statement["source"].as_str().unwrap().to_string(),
                                    statement: Box::new((&statement["items"]).into()),
                                };
                                report.balance_sheet = balance_report;
                            }
                            "CAS" => {
                                let cash_flow_report = CashFlowReport {
                                    source: statement["source"].as_str().unwrap().to_string(),
                                    period_type: statement["periodType"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    period_length: statement["periodLength"].as_i64().unwrap()
                                        as i32,
                                    statement: Box::new((&statement["items"]).into()),
                                };
                                report.cash_flow = cash_flow_report;
                            }
                            code => Err(ClientError::UnexpectedStatementType(code.to_string()))?,
                        }
                    }

                    Ok(report)
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or(Err(ClientError::ParseError("Can't get data".to_string())))
}

impl Client {
    pub async fn financial_statements_by_id(
        &self,
        id: impl AsRef<str>,
    ) -> Result<FinancialReports, ClientError> {
        let isin = &self.product(id.as_ref()).await?.inner.isin;
        self.financial_statements(id, isin).await
    }
    pub async fn financial_statements(
        &self,
        id: impl AsRef<str>,
        isin: impl AsRef<str>,
    ) -> Result<FinancialReports, ClientError> {
        if self.inner.lock().unwrap().status != ClientStatus::Authorized {
            return Err(ClientError::Unauthorized);
        }
        let req = {
            let inner = self.inner.lock().unwrap();
            let base_url = "https://trader.degiro.nl/";
            let path_url = "dgtbxdsservice/financial-statements/";
            // https://trader.degiro.nl/dgtbxdsservice/financial-statements/US14149Y1082?intAccount=71003134&sessionId=3373A4DF798147194546B8361D4D8387.prod_b_125_2
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
                let json = res.json::<serde_json::Value>().await?;
                let data = &json["data"];

                if data.is_null() {
                    return Err(ClientError::NoData);
                };

                let currency = data["currency"]
                    .as_str()
                    .ok_or(ClientError::ParseError("Can't get currency".to_string()))?
                    .to_string();

                let annual_reports = process_reports(&data["annual"])?;
                let interim_reports = process_reports(&data["interim"])?;

                let financial_report = FinancialReports {
                    id: id.as_ref().to_string(),
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

#[derive(Debug, Error)]
pub enum FinancialError {
    #[error("Revenue error: {0}")]
    Revenue(String),
    #[error("Total Revenue error: {0}")]
    TotalEquity(String),
    #[error("Total Assets error: {0}")]
    TotalAssets(String),
}

impl Report {
    pub fn revenue(&self) -> f64 {
        self.income_report.statement.srev.value
    }
    pub fn total_revenue(&self) -> f64 {
        self.income_report.statement.rtlr.value
    }
    pub fn cost_of_revenue(&self) -> f64 {
        self.income_report.statement.scor.value
    }
    pub fn gross_profit(&self) -> f64 {
        self.income_report.statement.sgrp.value
    }
    pub fn general_expenses(&self) -> f64 {
        self.income_report.statement.ssga.value
    }
    pub fn depreciation(&self) -> f64 {
        self.income_report.statement.sdpr.value
    }
    pub fn unusual_expenses(&self) -> f64 {
        self.income_report.statement.suie.value
    }
    pub fn operating_expenses(&self) -> f64 {
        self.income_report.statement.etoe.value
    }
    pub fn operating_income(&self) -> f64 {
        self.income_report.statement.sopi.value
    }
    pub fn interest_income(&self) -> f64 {
        self.income_report.statement.snin.value
    }
    pub fn other_income(&self) -> f64 {
        self.income_report.statement.sont.value
    }
    pub fn ebit(&self) -> f64 {
        self.income_report.statement.eibt.value
    }
    pub fn taxes(&self) -> f64 {
        self.income_report.statement.ttax.value
    }
    pub fn net_income_after_taxes(&self) -> f64 {
        self.income_report.statement.tiat.value
    }
    pub fn minority_interest(&self) -> f64 {
        self.income_report.statement.cmin.value
    }
    pub fn net_income_before_extra_items(&self) -> f64 {
        self.income_report.statement.nibx.value
    }
    pub fn net_income(&self) -> f64 {
        self.income_report.statement.ninc.value
    }
    pub fn income_available_to_common_excluding_extra_items(&self) -> f64 {
        self.income_report.statement.ciac.value
    }
    pub fn income_available_to_common_including_extra_items(&self) -> f64 {
        self.income_report.statement.xnic.value
    }
    pub fn diluted_net_income(&self) -> f64 {
        self.income_report.statement.sdni.value
    }
    pub fn diluted_weighted_average_shares(&self) -> f64 {
        self.income_report.statement.sdws.value
    }
    pub fn diluted_eps_excluding_extra_items(&self) -> f64 {
        self.income_report.statement.sdbf.value
    }
    pub fn dps_common_stock_primary_issue(&self) -> f64 {
        self.income_report.statement.ddps.value
    }
    pub fn diluted_normalized_eps(&self) -> f64 {
        self.income_report.statement.vdes.value
    }
    pub fn cash_and_equivalents(&self) -> f64 {
        self.balance_sheet.statement.acae.value
    }
    pub fn cash_and_short_term_investments(&self) -> f64 {
        self.balance_sheet.statement.scsi.value
    }
    pub fn accounts_receivable_trade_net(&self) -> f64 {
        self.balance_sheet.statement.aacr.value
    }
    pub fn total_receivables_net(&self) -> f64 {
        self.balance_sheet.statement.atrc.value
    }
    pub fn total_inventory(&self) -> f64 {
        self.balance_sheet.statement.aitl.value
    }
    pub fn prepaid_expenses(&self) -> f64 {
        self.balance_sheet.statement.appy.value
    }
    pub fn other_current_assets(&self) -> f64 {
        self.balance_sheet.statement.soca.value
    }
    pub fn total_current_assets(&self) -> f64 {
        self.balance_sheet.statement.atca.value
    }
    pub fn property_plant_equipment_total_gross(&self) -> f64 {
        self.balance_sheet.statement.aptc.value
    }
    pub fn accumulated_depreciation_total(&self) -> f64 {
        self.balance_sheet.statement.adep.value
    }
    pub fn property_plant_equipment_total_net(&self) -> f64 {
        self.balance_sheet.statement.appn.value
    }
    pub fn goodwill_net(&self) -> f64 {
        self.balance_sheet.statement.agwi.value
    }
    pub fn intangibles_net(&self) -> f64 {
        self.balance_sheet.statement.aint.value
    }
    pub fn other_long_term_assets_total(&self) -> f64 {
        self.balance_sheet.statement.sola.value
    }
    pub fn total_assets(&self) -> f64 {
        self.balance_sheet.statement.atot.value
    }
    pub fn accounts_payable(&self) -> f64 {
        self.balance_sheet.statement.lapb.value
    }
    pub fn accrued_expenses(&self) -> f64 {
        self.balance_sheet.statement.laex.value
    }
    pub fn notes_payable_short_term_debt(&self) -> f64 {
        self.balance_sheet.statement.lstd.value
    }
    pub fn current_port_of_lt_debt_capital_leases(&self) -> f64 {
        self.balance_sheet.statement.lcld.value
    }
    pub fn other_current_liabilities_total(&self) -> f64 {
        self.balance_sheet.statement.socl.value
    }
    pub fn total_current_liabilities(&self) -> f64 {
        self.balance_sheet.statement.ltcl.value
    }
    pub fn long_term_debt(&self) -> f64 {
        self.balance_sheet.statement.lltd.value
    }
    pub fn capital_lease_obligations(&self) -> f64 {
        self.balance_sheet.statement.lclo.value
    }
    pub fn total_long_term_debt(&self) -> f64 {
        self.balance_sheet.statement.lttd.value
    }
    pub fn total_debt(&self) -> f64 {
        self.balance_sheet.statement.stld.value
    }
    pub fn deferred_income_tax(&self) -> f64 {
        self.balance_sheet.statement.sbdt.value
    }
    // pub fn minority_interest(&self) -> f64 {
    //     self.balance_sheet.statement.lmin.value
    // }
    pub fn other_liabilities_total(&self) -> f64 {
        self.balance_sheet.statement.sltl.value
    }
    pub fn total_liabilities(&self) -> f64 {
        self.balance_sheet.statement.ltll.value
    }
    pub fn preferred_stock_non_redeemable_net(&self) -> f64 {
        self.balance_sheet.statement.sprs.value
    }
    pub fn common_stock_total(&self) -> f64 {
        self.balance_sheet.statement.scms.value
    }
    pub fn retained_earnings_accumulated_deficit(&self) -> f64 {
        self.balance_sheet.statement.qred.value
    }
    pub fn treasury_stock_common(&self) -> f64 {
        self.balance_sheet.statement.qtsc.value
    }
    pub fn other_equity_total(&self) -> f64 {
        self.balance_sheet.statement.sote.value
    }
    pub fn total_equity(&self) -> f64 {
        self.balance_sheet.statement.qtle.value
    }
    pub fn total_liabilities_shareholders_equity(&self) -> f64 {
        self.balance_sheet.statement.qtel.value
    }
    pub fn total_common_shares_outstanding(&self) -> f64 {
        self.balance_sheet.statement.qtco.value
    }
    pub fn tangible_book_value_per_share_common_eq(&self) -> f64 {
        self.balance_sheet.statement.stbp.value
    }
    pub fn net_income_starting_line(&self) -> f64 {
        self.cash_flow.statement.onet.value
    }
    pub fn depreciation_depletion(&self) -> f64 {
        self.cash_flow.statement.sded.value
    }
    pub fn deferred_taxes(&self) -> f64 {
        self.cash_flow.statement.obdt.value
    }
    pub fn non_cash_items(&self) -> f64 {
        self.cash_flow.statement.snci.value
    }
    pub fn cash_taxes_paid(&self) -> f64 {
        self.cash_flow.statement.sctp.value
    }
    pub fn cash_interest_paid(&self) -> f64 {
        self.cash_flow.statement.scip.value
    }
    pub fn changes_in_working_capital(&self) -> f64 {
        self.cash_flow.statement.socf.value
    }
    pub fn cash_from_operating_activities(&self) -> f64 {
        self.cash_flow.statement.otlo.value
    }
    pub fn capital_expenditures(&self) -> f64 {
        self.cash_flow.statement.scex.value
    }
    pub fn other_investing_cash_flow_items_total(&self) -> f64 {
        self.cash_flow.statement.sicf.value
    }
    pub fn cash_from_investing_activities(&self) -> f64 {
        self.cash_flow.statement.itli.value
    }
    pub fn financing_cash_flow_items(&self) -> f64 {
        self.cash_flow.statement.sfcf.value
    }
    pub fn total_cash_dividends_paid(&self) -> f64 {
        self.cash_flow.statement.fcdp.value
    }
    pub fn issuance_retirement_of_stock_net(&self) -> f64 {
        self.cash_flow.statement.fpss.value
    }
    pub fn issuance_retirement_of_debt_net(&self) -> f64 {
        self.cash_flow.statement.fprd.value
    }
    pub fn cash_from_financing_activities(&self) -> f64 {
        self.cash_flow.statement.ftlf.value
    }
    pub fn foreign_exchange_effects(&self) -> f64 {
        self.cash_flow.statement.sfee.value
    }
    pub fn net_change_in_cash(&self) -> f64 {
        self.cash_flow.statement.sncc.value
    }
    pub fn ebitda(&self) -> f64 {
        self.ebit() + self.depreciation()
    }
    pub fn ebitda_margin(&self) -> f64 {
        let revenue = self.revenue();
        if revenue == 0.0 {
            0.0
        } else {
            self.ebitda() / revenue
        }
    }
    pub fn ebit_margin(&self) -> f64 {
        let revenue = self.revenue();
        if revenue == 0.0 {
            0.0
        } else {
            self.ebit() / revenue
        }
    }
    pub fn net_margin(&self) -> f64 {
        let revenue = self.revenue();
        if revenue == 0.0 {
            0.0
        } else {
            self.net_income() / revenue
        }
    }
    pub fn gross_margin(&self) -> f64 {
        let revenue = self.revenue();
        if revenue == 0.0 {
            0.0
        } else {
            self.gross_profit() / revenue
        }
    }
    pub fn operating_margin(&self) -> f64 {
        let revenue = self.revenue();
        if revenue == 0.0 {
            0.0
        } else {
            self.operating_income() / revenue
        }
    }
    pub fn roe(&self) -> f64 {
        let total_equity = self.total_equity();
        if total_equity == 0.0 {
            0.0
        } else {
            self.net_income() / total_equity
        }
    }
    pub fn roa(&self) -> f64 {
        let total_assets = self.total_assets();
        if total_assets == 0.0 {
            0.0
        } else {
            self.net_income() / total_assets
        }
    }
    pub fn roce(&self) -> f64 {
        let total_assets = self.total_assets();
        if total_assets == 0.0 {
            0.0
        } else {
            self.ebit() / total_assets
        }
    }
    pub fn nopat(&self) -> f64 {
        self.income_report.statement.sopi.value - self.income_report.statement.ttax.value
    }
    pub fn invested_capital(&self) -> f64 {
        (self.balance_sheet.statement.stld.value + self.balance_sheet.statement.qtle.value)
            - self.balance_sheet.statement.acae.value
    }
    pub fn roic(&self) -> f64 {
        self.nopat() / self.invested_capital()
    }
    pub fn wacc(&self, equity_cost: f64) -> f64 {
        let debt = self.balance_sheet.statement.stld.value;
        let equity = self.balance_sheet.statement.qtle.value;
        let total = debt + equity;
        let debt_rate = debt / total;
        let equity_rate = equity / total;
        debt_rate * self.debt_cost() + equity_rate * equity_cost
    }
    pub fn tax_rate(&self) -> f64 {
        let ebit = self.ebit();
        let taxes = self.taxes();
        if ebit == 0.0 {
            0.0
        } else {
            taxes / ebit
        }
    }
    pub fn debt_cost(&self) -> f64 {
        let debt = self.total_debt();
        let interest = self.cash_interest_paid();
        if debt == 0.0 {
            0.0
        } else {
            (interest / debt) * (1.0 - self.tax_rate())
        }
    }

    // capm - Capital Asset Pricing Model equity cost
    pub fn capm_equity_cost(&self, market_return: f64, risk_free_rate: f64, beta: f64) -> f64 {
        risk_free_rate + beta * (market_return - risk_free_rate)
    }
    pub fn price_earnings_ratio(&self, current_price: f64) -> f64 {
        let eps = self.diluted_normalized_eps();
        if eps == 0.0 {
            0.0
        } else {
            current_price / eps
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
