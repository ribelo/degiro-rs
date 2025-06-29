use chrono::NaiveDate;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FinancialReports {
    pub isin: String,
    pub currency: String,
    pub annual: Reports,
    pub interim: Reports,
}

impl FinancialReports {
    pub fn get_annual(&self, fiscal_year: i32) -> Option<&Report> {
        self.annual
            .0
            .iter()
            .find(|report| report.fiscal_year == fiscal_year)
    }
    pub fn get_interim(&self, fiscal_year: i32, end_date: NaiveDate) -> Option<&Report> {
        self.interim
            .0
            .iter()
            .find(|report| report.fiscal_year == fiscal_year && report.end_date == end_date)
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
    pub value: Decimal,
}

impl From<&serde_json::Value> for ItemDetail {
    fn from(value: &serde_json::Value) -> Self {
        ItemDetail {
            meaning: value["meaning"].as_str().unwrap().to_string(),
            value: Decimal::from_f64(value["value"].as_f64().unwrap()).unwrap(),
        }
    }
}
