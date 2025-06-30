use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thiserror::Error;

use crate::{
    client::Degiro,
    error::{ClientError, DataError, DateTimeError, ResponseError},
    http::{HttpClient, HttpRequest},
    models::{BalanceSheetReport, CashFlowReport, FinancialReports, IncomeStatementReport, Report},
    paths::{BASE_API_URL, FINANCIAL_STATEMENTS_PATH},
};

fn process_reports(data: &serde_json::Value) -> Result<Vec<Report>, ClientError> {
    let reports = data
        .as_array()
        .ok_or_else(|| ResponseError::unexpected_structure("Financial data must be an array"))?;

    let mut results = Vec::with_capacity(reports.len());

    for report_data in reports {
        let fiscal_year = report_data["fiscalYear"]
            .as_i64()
            .ok_or_else(|| DataError::missing_field("fiscalYear"))?
            as i32;

        let end_date = NaiveDate::parse_from_str(
            report_data["endDate"]
                .as_str()
                .ok_or_else(|| DataError::missing_field("endDate"))?,
            "%Y-%m-%d",
        )
        .map_err(|err| ClientError::from(DateTimeError::ParseError {
            input: report_data["endDate"].as_str().unwrap_or("unknown").to_string(),
            reason: err.to_string(),
        }))?;

        let mut report = Report {
            fiscal_year,
            end_date,
            ..Default::default()
        };

        let statements = report_data["statements"]
            .as_array()
            .ok_or_else(|| DataError::missing_field("statements"))?;

        for statement in statements {
            let statement_type = statement["type"]
                .as_str()
                .ok_or_else(|| DataError::missing_field("statement.type"))?;

            match statement_type {
                "INC" => {
                    report.income_report = IncomeStatementReport {
                        source: statement["source"]
                            .as_str()
                            .ok_or_else(|| DataError::missing_field("source"))?
                            .to_string(),
                        period_type: statement["periodType"]
                            .as_str()
                            .ok_or_else(|| {
                                DataError::missing_field("periodType")
                            })?
                            .to_string(),
                        period_length: statement["periodLength"].as_i64().ok_or_else(|| {
                            DataError::missing_field("periodLength")
                        })? as i32,
                        statement: Box::new((&statement["items"]).into()),
                    };
                }
                "BAL" => {
                    report.balance_sheet = BalanceSheetReport {
                        source: statement["source"]
                            .as_str()
                            .ok_or_else(|| DataError::missing_field("source"))?
                            .to_string(),
                        statement: Box::new((&statement["items"]).into()),
                    };
                }
                "CAS" => {
                    report.cash_flow = CashFlowReport {
                        source: statement["source"]
                            .as_str()
                            .ok_or_else(|| DataError::missing_field("source"))?
                            .to_string(),
                        period_type: statement["periodType"]
                            .as_str()
                            .ok_or_else(|| {
                                DataError::missing_field("periodType")
                            })?
                            .to_string(),
                        period_length: statement["periodLength"].as_i64().ok_or_else(|| {
                            DataError::missing_field("periodLength")
                        })? as i32,
                        statement: Box::new((&statement["items"]).into()),
                    };
                }
                code => {
                    return Err(ResponseError::unknown_value("statement type", code).into())
                }
            }
        }

        results.push(report);
    }

    Ok(results)
}

impl Degiro {
    pub async fn financial_statements_by_id(
        &self,
        id: impl AsRef<str>,
    ) -> Result<Option<FinancialReports>, ClientError> {
        let id = id.as_ref();
        let product = &self.product(id).await?;
        match product {
            Some(p) => self.financial_statements(&p.isin).await,
            None => Ok(None),
        }
    }

    pub async fn financial_statements(
        &self,
        isin: impl AsRef<str>,
    ) -> Result<Option<FinancialReports>, ClientError> {
        let url = format!("{}{}{}", BASE_API_URL, FINANCIAL_STATEMENTS_PATH, isin.as_ref());
        
        let json = self.request_json(
            HttpRequest::get(url)
                .query("intAccount", self.int_account().to_string())
                .query("sessionId", self.session_id())
                .header("Content-Type", "application/json")
        ).await?;
        let data = match json.get("data") {
            Some(data) => data,
            None => return Ok(None),
        };

        if data.is_null() {
            return Ok(None);
        }

        let currency = data
            .get("currency")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DataError::missing_field("currency"))?
            .to_string();

        let annual_reports = process_reports(&data["annual"])?;
        let interim_reports = process_reports(&data["interim"])?;

        Ok(Some(FinancialReports {
            isin: isin.as_ref().to_string(),
            currency,
            annual: annual_reports.into(),
            interim: interim_reports.into(),
        }))
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
    pub fn revenue(&self) -> Decimal {
        self.income_report.statement.srev.value
    }
    pub fn total_revenue(&self) -> Decimal {
        self.income_report.statement.rtlr.value
    }
    pub fn cost_of_revenue(&self) -> Decimal {
        self.income_report.statement.scor.value
    }
    pub fn gross_profit(&self) -> Decimal {
        self.income_report.statement.sgrp.value
    }
    pub fn general_expenses(&self) -> Decimal {
        self.income_report.statement.ssga.value
    }
    pub fn depreciation(&self) -> Decimal {
        self.income_report.statement.sdpr.value
    }
    pub fn unusual_expenses(&self) -> Decimal {
        self.income_report.statement.suie.value
    }
    pub fn operating_expenses(&self) -> Decimal {
        self.income_report.statement.etoe.value
    }
    pub fn operating_income(&self) -> Decimal {
        self.income_report.statement.sopi.value
    }
    pub fn interest_income(&self) -> Decimal {
        self.income_report.statement.snin.value
    }
    pub fn other_income(&self) -> Decimal {
        self.income_report.statement.sont.value
    }
    pub fn ebit(&self) -> Decimal {
        self.income_report.statement.eibt.value
    }
    pub fn taxes(&self) -> Decimal {
        self.income_report.statement.ttax.value
    }
    pub fn net_income_after_taxes(&self) -> Decimal {
        self.income_report.statement.tiat.value
    }
    pub fn minority_interest(&self) -> Decimal {
        self.income_report.statement.cmin.value
    }
    pub fn net_income_before_extra_items(&self) -> Decimal {
        self.income_report.statement.nibx.value
    }
    pub fn net_income(&self) -> Decimal {
        self.income_report.statement.ninc.value
    }
    pub fn income_available_to_common_excluding_extra_items(&self) -> Decimal {
        self.income_report.statement.ciac.value
    }
    pub fn income_available_to_common_including_extra_items(&self) -> Decimal {
        self.income_report.statement.xnic.value
    }
    pub fn diluted_net_income(&self) -> Decimal {
        self.income_report.statement.sdni.value
    }
    pub fn diluted_weighted_average_shares(&self) -> Decimal {
        self.income_report.statement.sdws.value
    }
    pub fn diluted_eps_excluding_extra_items(&self) -> Decimal {
        self.income_report.statement.sdbf.value
    }
    pub fn dps_common_stock_primary_issue(&self) -> Decimal {
        self.income_report.statement.ddps.value
    }
    pub fn diluted_normalized_eps(&self) -> Decimal {
        self.income_report.statement.vdes.value
    }
    pub fn cash_and_equivalents(&self) -> Decimal {
        self.balance_sheet.statement.acae.value
    }
    pub fn cash_and_short_term_investments(&self) -> Decimal {
        self.balance_sheet.statement.scsi.value
    }
    pub fn accounts_receivable_trade_net(&self) -> Decimal {
        self.balance_sheet.statement.aacr.value
    }
    pub fn total_receivables_net(&self) -> Decimal {
        self.balance_sheet.statement.atrc.value
    }
    pub fn total_inventory(&self) -> Decimal {
        self.balance_sheet.statement.aitl.value
    }
    pub fn prepaid_expenses(&self) -> Decimal {
        self.balance_sheet.statement.appy.value
    }
    pub fn other_current_assets(&self) -> Decimal {
        self.balance_sheet.statement.soca.value
    }
    pub fn total_current_assets(&self) -> Decimal {
        self.balance_sheet.statement.atca.value
    }
    pub fn property_plant_equipment_total_gross(&self) -> Decimal {
        self.balance_sheet.statement.aptc.value
    }
    pub fn accumulated_depreciation_total(&self) -> Decimal {
        self.balance_sheet.statement.adep.value
    }
    pub fn property_plant_equipment_total_net(&self) -> Decimal {
        self.balance_sheet.statement.appn.value
    }
    pub fn goodwill_net(&self) -> Decimal {
        self.balance_sheet.statement.agwi.value
    }
    pub fn intangibles_net(&self) -> Decimal {
        self.balance_sheet.statement.aint.value
    }
    pub fn other_long_term_assets_total(&self) -> Decimal {
        self.balance_sheet.statement.sola.value
    }
    pub fn total_assets(&self) -> Decimal {
        self.balance_sheet.statement.atot.value
    }
    pub fn accounts_payable(&self) -> Decimal {
        self.balance_sheet.statement.lapb.value
    }
    pub fn accrued_expenses(&self) -> Decimal {
        self.balance_sheet.statement.laex.value
    }
    pub fn notes_payable_short_term_debt(&self) -> Decimal {
        self.balance_sheet.statement.lstd.value
    }
    pub fn current_port_of_lt_debt_capital_leases(&self) -> Decimal {
        self.balance_sheet.statement.lcld.value
    }
    pub fn other_current_liabilities_total(&self) -> Decimal {
        self.balance_sheet.statement.socl.value
    }
    pub fn total_current_liabilities(&self) -> Decimal {
        self.balance_sheet.statement.ltcl.value
    }
    pub fn long_term_debt(&self) -> Decimal {
        self.balance_sheet.statement.lltd.value
    }
    pub fn capital_lease_obligations(&self) -> Decimal {
        self.balance_sheet.statement.lclo.value
    }
    pub fn total_long_term_debt(&self) -> Decimal {
        self.balance_sheet.statement.lttd.value
    }
    pub fn total_debt(&self) -> Decimal {
        self.balance_sheet.statement.stld.value
    }
    pub fn deferred_income_tax(&self) -> Decimal {
        self.balance_sheet.statement.sbdt.value
    }
    pub fn other_liabilities_total(&self) -> Decimal {
        self.balance_sheet.statement.sltl.value
    }
    pub fn total_liabilities(&self) -> Decimal {
        self.balance_sheet.statement.ltll.value
    }
    pub fn preferred_stock_non_redeemable_net(&self) -> Decimal {
        self.balance_sheet.statement.sprs.value
    }
    pub fn common_stock_total(&self) -> Decimal {
        self.balance_sheet.statement.scms.value
    }
    pub fn retained_earnings_accumulated_deficit(&self) -> Decimal {
        self.balance_sheet.statement.qred.value
    }
    pub fn treasury_stock_common(&self) -> Decimal {
        self.balance_sheet.statement.qtsc.value
    }
    pub fn other_equity_total(&self) -> Decimal {
        self.balance_sheet.statement.sote.value
    }
    pub fn total_equity(&self) -> Decimal {
        self.balance_sheet.statement.qtle.value
    }
    pub fn total_liabilities_shareholders_equity(&self) -> Decimal {
        self.balance_sheet.statement.qtel.value
    }
    pub fn total_common_shares_outstanding(&self) -> Decimal {
        self.balance_sheet.statement.qtco.value
    }
    pub fn tangible_book_value_per_share_common_eq(&self) -> Decimal {
        self.balance_sheet.statement.stbp.value
    }
    pub fn net_income_starting_line(&self) -> Decimal {
        self.cash_flow.statement.onet.value
    }
    pub fn depreciation_depletion(&self) -> Decimal {
        self.cash_flow.statement.sded.value
    }
    pub fn deferred_taxes(&self) -> Decimal {
        self.cash_flow.statement.obdt.value
    }
    pub fn non_cash_items(&self) -> Decimal {
        self.cash_flow.statement.snci.value
    }
    pub fn cash_taxes_paid(&self) -> Decimal {
        self.cash_flow.statement.sctp.value
    }
    pub fn cash_interest_paid(&self) -> Decimal {
        self.cash_flow.statement.scip.value
    }
    pub fn changes_in_working_capital(&self) -> Decimal {
        self.cash_flow.statement.socf.value
    }
    pub fn cash_from_operating_activities(&self) -> Decimal {
        self.cash_flow.statement.otlo.value
    }
    pub fn capital_expenditures(&self) -> Decimal {
        self.cash_flow.statement.scex.value
    }
    pub fn other_investing_cash_flow_items_total(&self) -> Decimal {
        self.cash_flow.statement.sicf.value
    }
    pub fn cash_from_investing_activities(&self) -> Decimal {
        self.cash_flow.statement.itli.value
    }
    pub fn financing_cash_flow_items(&self) -> Decimal {
        self.cash_flow.statement.sfcf.value
    }
    pub fn total_cash_dividends_paid(&self) -> Decimal {
        self.cash_flow.statement.fcdp.value
    }
    pub fn issuance_retirement_of_stock_net(&self) -> Decimal {
        self.cash_flow.statement.fpss.value
    }
    pub fn issuance_retirement_of_debt_net(&self) -> Decimal {
        self.cash_flow.statement.fprd.value
    }
    pub fn cash_from_financing_activities(&self) -> Decimal {
        self.cash_flow.statement.ftlf.value
    }
    pub fn foreign_exchange_effects(&self) -> Decimal {
        self.cash_flow.statement.sfee.value
    }
    pub fn net_change_in_cash(&self) -> Decimal {
        self.cash_flow.statement.sncc.value
    }
    pub fn ebitda(&self) -> Decimal {
        self.ebit() + self.depreciation()
    }
    pub fn ebitda_margin(&self) -> Decimal {
        let revenue = self.revenue();
        if revenue.is_zero() {
            dec!(0)
        } else {
            self.ebitda() / revenue
        }
    }
    pub fn ebit_margin(&self) -> Decimal {
        let revenue = self.revenue();
        if revenue.is_zero() {
            dec!(0)
        } else {
            self.ebit() / revenue
        }
    }
    pub fn net_margin(&self) -> Decimal {
        let revenue = self.revenue();
        if revenue.is_zero() {
            dec!(0)
        } else {
            self.net_income() / revenue
        }
    }
    pub fn gross_margin(&self) -> Decimal {
        let revenue = self.revenue();
        if revenue.is_zero() {
            dec!(0)
        } else {
            self.gross_profit() / revenue
        }
    }
    pub fn operating_margin(&self) -> Decimal {
        let revenue = self.revenue();
        if revenue.is_zero() {
            dec!(0)
        } else {
            self.operating_income() / revenue
        }
    }
    pub fn roe(&self) -> Decimal {
        let total_equity = self.total_equity();
        if total_equity.is_zero() {
            dec!(0)
        } else {
            self.net_income() / total_equity
        }
    }
    pub fn roa(&self) -> Decimal {
        let total_assets = self.total_assets();
        if total_assets.is_zero() {
            dec!(0)
        } else {
            self.net_income() / total_assets
        }
    }
    pub fn roce(&self) -> Decimal {
        let total_assets = self.total_assets();
        if total_assets.is_zero() {
            dec!(0)
        } else {
            self.ebit() / total_assets
        }
    }
    pub fn nopat(&self) -> Decimal {
        self.income_report.statement.sopi.value - self.income_report.statement.ttax.value
    }
    pub fn invested_capital(&self) -> Decimal {
        (self.balance_sheet.statement.stld.value + self.balance_sheet.statement.qtle.value)
            - self.balance_sheet.statement.acae.value
    }
    pub fn roic(&self) -> Decimal {
        self.nopat() / self.invested_capital()
    }
    pub fn wacc(&self, equity_cost: Decimal) -> Decimal {
        let debt = self.balance_sheet.statement.stld.value;
        let equity = self.balance_sheet.statement.qtle.value;
        let total = debt + equity;
        let debt_rate = debt / total;
        let equity_rate = equity / total;
        debt_rate * self.debt_cost() + equity_rate * equity_cost
    }
    pub fn tax_rate(&self) -> Decimal {
        let ebit = self.ebit();
        let taxes = self.taxes();
        if ebit.is_zero() {
            dec!(0)
        } else {
            taxes / ebit
        }
    }
    pub fn debt_cost(&self) -> Decimal {
        let debt = self.total_debt();
        let interest = self.cash_interest_paid();
        if debt.is_zero() {
            dec!(0)
        } else {
            (interest / debt) * (dec!(1) - self.tax_rate())
        }
    }

    // capm - Capital Asset Pricing Model equity cost
    pub fn capm_equity_cost(
        &self,
        market_return: Decimal,
        risk_free_rate: Decimal,
        beta: Decimal,
    ) -> Decimal {
        risk_free_rate + beta * (market_return - risk_free_rate)
    }
    pub fn price_earnings_ratio(&self, current_price: Decimal) -> Decimal {
        let eps = self.diluted_normalized_eps();
        if eps.is_zero() {
            dec!(0)
        } else {
            current_price / eps
        }
    }

    pub fn free_cash_flow(&self) -> Decimal {
        self.cash_flow.statement.otlo.value - self.cash_flow.statement.scex.value
    }

    pub fn cash_conversion_cycle(&self) -> Decimal {
        let inventory = self.total_inventory();
        let accounts_payable = self.accounts_payable();
        let revenue = self.revenue();
        if revenue.is_zero() {
            dec!(0)
        } else {
            (inventory + accounts_payable) / revenue
        }
    }

    pub fn debt_to_equity(&self) -> Decimal {
        let total_debt = self.total_debt();
        let total_equity = self.total_equity();
        if total_equity.is_zero() {
            dec!(0)
        } else {
            total_debt / total_equity
        }
    }

    pub fn net_debt_to_ebitda(&self) -> Decimal {
        let net_debt = self.total_debt() - self.cash_and_short_term_investments();
        let ebitda = self.ebitda();
        if ebitda.is_zero() {
            dec!(0)
        } else {
            net_debt / ebitda
        }
    }

    pub fn days_inventory_outstanding(&self) -> Decimal {
        let inventory = self.total_inventory();
        let cost_of_revenue = self.cost_of_revenue();
        if cost_of_revenue.is_zero() {
            dec!(0)
        } else {
            inventory / (cost_of_revenue / dec!(365))
        }
    }

    pub fn days_sales_outstanding(&self) -> Decimal {
        let accounts_receivable = self.accounts_receivable_trade_net();
        let revenue = self.revenue();
        if revenue.is_zero() {
            dec!(0)
        } else {
            accounts_receivable / (revenue / dec!(365))
        }
    }

    pub fn days_payables_outstanding(&self) -> Decimal {
        let accounts_payable = self.accounts_payable();
        let cost_of_revenue = self.cost_of_revenue();
        if cost_of_revenue.is_zero() {
            dec!(0)
        } else {
            accounts_payable / (cost_of_revenue / dec!(365))
        }
    }
    pub fn current_ratio(&self) -> Decimal {
        let current_liabilities = self.total_current_liabilities();
        if current_liabilities.is_zero() {
            dec!(0)
        } else {
            self.total_current_assets() / current_liabilities
        }
    }

    pub fn quick_ratio(&self) -> Decimal {
        let current_liabilities = self.total_current_liabilities();
        if current_liabilities.is_zero() {
            dec!(0)
        } else {
            (self.total_current_assets() - self.total_inventory()) / current_liabilities
        }
    }

    pub fn asset_turnover(&self) -> Decimal {
        let total_assets = self.total_assets();
        if total_assets.is_zero() {
            dec!(0)
        } else {
            self.revenue() / total_assets
        }
    }

    pub fn equity_multiplier(&self) -> Decimal {
        let total_equity = self.total_equity();
        if total_equity.is_zero() {
            dec!(0)
        } else {
            self.total_assets() / total_equity
        }
    }

    pub fn interest_coverage(&self) -> Decimal {
        let interest_paid = self.cash_interest_paid();
        if interest_paid.is_zero() {
            dec!(0)
        } else {
            self.ebit() / interest_paid
        }
    }

    pub fn fixed_asset_turnover(&self) -> Decimal {
        let fixed_assets = self.property_plant_equipment_total_net();
        if fixed_assets.is_zero() {
            dec!(0)
        } else {
            self.revenue() / fixed_assets
        }
    }

    pub fn working_capital(&self) -> Decimal {
        self.total_current_assets() - self.total_current_liabilities()
    }

    pub fn net_working_capital(&self) -> Decimal {
        self.working_capital() - self.cash_and_equivalents()
    }

    pub fn debt_service_coverage_ratio(&self) -> Decimal {
        let debt_service =
            self.cash_interest_paid() + self.current_port_of_lt_debt_capital_leases();
        if debt_service.is_zero() {
            dec!(0)
        } else {
            self.operating_income() / debt_service
        }
    }

    pub fn gross_fixed_assets(&self) -> Decimal {
        self.property_plant_equipment_total_gross()
    }

    pub fn net_fixed_assets(&self) -> Decimal {
        self.property_plant_equipment_total_net()
    }

    pub fn inventory_turnover(&self) -> Decimal {
        let avg_inventory = self.total_inventory();
        if avg_inventory.is_zero() {
            dec!(0)
        } else {
            self.cost_of_revenue() / avg_inventory
        }
    }

    pub fn operating_cash_flow_ratio(&self) -> Decimal {
        let current_liabilities = self.total_current_liabilities();
        if current_liabilities.is_zero() {
            dec!(0)
        } else {
            self.cash_from_operating_activities() / current_liabilities
        }
    }

    pub fn dividend_payout_ratio(&self) -> Decimal {
        let net_income = self.net_income();
        if net_income.is_zero() {
            dec!(0)
        } else {
            self.total_cash_dividends_paid() / net_income
        }
    }

    pub fn operating_cycle(&self) -> Decimal {
        self.days_inventory_outstanding() + self.days_sales_outstanding()
    }

    pub fn cash_ratio(&self) -> Decimal {
        let current_liabilities = self.total_current_liabilities();
        if current_liabilities.is_zero() {
            dec!(0)
        } else {
            self.cash_and_short_term_investments() / current_liabilities
        }
    }
}
