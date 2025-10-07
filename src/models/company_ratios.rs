use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyRatios {
    pub isin: String,
    pub current_ratios: CurrentRatios,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentRatios {
    pub currency: String,
    pub price_currency: String,
    /// Price - closing or last bid
    pub current_price: ItemDetail<f64>,
    /// Price - 12 month high
    pub high_12m: ItemDetail<f64>,
    /// Price - 12 month low
    pub low_12m: ItemDetail<f64>,
    /// Pricing date
    pub pricing_date: ItemDetail<NaiveDateTime>,
    /// Volume - avg. trading volume for the last ten days
    pub volume_avg_10d: ItemDetail<f64>,
    /// Market capitalization
    pub market_cap: ItemDetail<f64>,
    /// 12 Month High price date
    pub high_date_12m: ItemDetail<NaiveDateTime>,
    /// 12 Month Low price date
    pub low_date_12m: ItemDetail<NaiveDateTime>,
    /// Volume - avg. trading volume for the last 3 months
    pub volume_avg_3m: ItemDetail<f64>,
    /// Beta
    pub beta: ItemDetail<f64>,
    /// Price - 1 Day % Change
    pub price_change_1d: ItemDetail<f64>,
    /// Price - 13 week price percent change
    pub price_change_13w: ItemDetail<f64>,
    /// Price - 26 week price percent change
    pub price_change_26w: ItemDetail<f64>,
    /// Price - 5 Day % Change
    pub price_change_5d: ItemDetail<f64>,
    /// Price - 52 week price percent change
    pub price_change_52w: ItemDetail<f64>,
    /// Price - YTD price percent change
    pub price_change_ytd: ItemDetail<f64>,
    /// Price % Change Month To Date
    pub price_change_mtd: ItemDetail<f64>,
    /// Relative (S&P500) price percent change - 04 week
    pub relative_price_change_4w: ItemDetail<f64>,
    /// Relative (S&P500) price percent change - 13 week
    pub relative_price_change_13w: ItemDetail<f64>,
    /// Relative (S&P500) price percent change - 26 week
    pub relative_price_change_26w: ItemDetail<f64>,
    /// Relative (S&P500) price percent change - 52 week
    pub relative_price_change_52w: ItemDetail<f64>,
    /// Relative (S&P500) price percent change - Year to Date
    pub relative_price_change_ytd: ItemDetail<f64>,
    /// EPS excluding extraordinary items - most recent fiscal year
    pub eps_excluding_extraordinary_items_annual: ItemDetail<f64>,
    /// EPS excluding extraordinary items - trailing 12 month
    pub eps_excluding_extraordinary_items_ttm: ItemDetail<f64>,
    /// EPS Normalized - most recent fiscal year
    pub eps_normalized_annual: ItemDetail<f64>,
    /// Revenue/share - most recent fiscal year
    pub revenue_per_share_annual: ItemDetail<f64>,
    /// Revenue/share - trailing 12 month
    pub revenue_per_share_ttm: ItemDetail<f64>,
    /// Book value (Total Equity) per share - most recent fiscal year
    pub book_value_per_share_annual: ItemDetail<f64>,
    /// Book value (Total Equity) per share - most recent quarter
    pub book_value_per_share_quarterly: ItemDetail<f64>,
    /// Book value (tangible) per share - most recent fiscal year
    pub tangible_book_value_per_share_annual: ItemDetail<f64>,
    /// Book value (tangible) per share - most recent quarter
    pub tangible_book_value_per_share_quarterly: ItemDetail<f64>,
    /// Cash per share - most recent fiscal year
    pub cash_per_share_annual: ItemDetail<f64>,
    /// Cash per share - most recent quarter
    pub cash_per_share_quarterly: ItemDetail<f64>,
    /// Cash Flow per share - most recent fiscal year
    pub cash_flow_per_share_annual: ItemDetail<f64>,
    /// Cash Flow per share - trailing 12 month
    pub cash_flow_per_share_ttm: ItemDetail<f64>,
    /// Dividend per share - most recent fiscal year
    pub dividend_per_share_annual: ItemDetail<f64>,
    /// Dividends per share - trailing 12 month
    pub dividend_per_share_ttm: ItemDetail<f64>,
    /// EBITD per share - trailing 12 month
    pub ebitd_per_share_ttm: ItemDetail<f64>,
    /// ABEPSXCLXO
    pub abepsxclxo: ItemDetail<f64>,
    /// EPS Basic excluding extraordinary items - trailing 12 month
    pub eps_basic_excluding_extraordinary_items_ttm: ItemDetail<f64>,
    /// EPS including extraordinary items - most recent fiscal year
    pub eps_including_extraordinary_items_annual: ItemDetail<f64>,
    /// EPS including extraordinary items - trailing 12 month
    pub eps_including_extraordinary_items_ttm: ItemDetail<f64>,
    /// Free Cash Flow per share - trailing 12 month
    pub free_cash_flow_per_share_ttm: ItemDetail<f64>,
    /// Dividend Per Share - 5 year average
    pub dividend_per_share_5yr_avg: ItemDetail<f64>,
    /// P/E excluding extraordinary items, most recent fiscal year
    pub pe_excluding_extraordinary_items_annual: ItemDetail<f64>,
    /// P/E excluding extraordinary items - TTM
    pub pe_excluding_extraordinary_items_ttm: ItemDetail<f64>,
    /// P/E Normalized, most recent fiscal year
    pub pe_normalized_annual: ItemDetail<f64>,
    /// Price to sales - most recent fiscal year
    pub price_to_sales_annual: ItemDetail<f64>,
    /// Price to sales - trailing 12 month
    pub price_to_sales_ttm: ItemDetail<f64>,
    /// Price to Tangible Book - most fiscal year
    pub price_to_tangible_book_annual: ItemDetail<f64>,
    /// Price to Tangible Book - most recent quarter
    pub price_to_tangible_book_quarterly: ItemDetail<f64>,
    /// Price to Free Cash Flow per Share - most recent fiscal year
    pub price_to_free_cash_flow_per_share_annual: ItemDetail<f64>,
    /// Price to Cash Flow per share - trailing 12 month
    pub price_to_cash_flow_per_share_ttm: ItemDetail<f64>,
    /// Price to Free Cash Flow per Share - trailing 12 months
    pub price_to_free_cash_flow_per_share_ttm: ItemDetail<f64>,
    /// Price to Book - most recent fiscal year
    pub price_to_book_annual: ItemDetail<f64>,
    /// Price to Book - most recent quarter
    pub price_to_book_quarterly: ItemDetail<f64>,
    /// P/E Basic excluding extraordinary items - TTM
    pub pe_basic_excluding_extraordinary_items_ttm: ItemDetail<f64>,
    /// P/E excluding extraordinary items high, trailing 12 months
    pub pe_high_excluding_extraordinary_items_ttm: ItemDetail<f64>,
    /// P/E excluding extraordinary items low, trailing 12 months
    pub pe_low_excluding_extraordinary_items_ttm: ItemDetail<f64>,
    /// P/E including extraordinary items - TTM
    pub pe_including_extraordinary_items_ttm: ItemDetail<f64>,
    /// Net Debt, LFI
    pub net_debt_lfi: ItemDetail<f64>,
    /// Net Debt, LFY
    pub net_debt_lfy: ItemDetail<f64>,
    /// Dividend Yield - 5 Year Average
    pub dividend_yield_5yr_avg: ItemDetail<f64>,
    /// Dividend Yield - indicated annual dividend divided by closing price
    pub dividend_yield: ItemDetail<f64>,
    /// Current Dividend Yield - Common Stock Primary Issue, LTM
    pub current_dividend_yield_ttm: ItemDetail<f64>,
    /// Current ratio - most recent fiscal year
    pub current_ratio_annual: ItemDetail<f64>,
    /// Current ratio - most recent quarter
    pub current_ratio_quarterly: ItemDetail<f64>,
    /// Quick ratio - most recent fiscal year
    pub quick_ratio_annual: ItemDetail<f64>,
    /// Quick ratio - most recent quarter
    pub quick_ratio_quarterly: ItemDetail<f64>,
    /// LT debt/equity - most recent fiscal year
    pub long_term_debt_to_equity_annual: ItemDetail<f64>,
    /// LT debt/equity - most recent quarter
    pub long_term_debt_to_equity_quarterly: ItemDetail<f64>,
    /// Total debt/total equity - most recent fiscal year
    pub total_debt_to_equity_annual: ItemDetail<f64>,
    /// Total debt/total equity - most recent quarter
    pub total_debt_to_equity_quarterly: ItemDetail<f64>,
    /// Payout ratio - most recent fiscal year
    pub payout_ratio_annual: ItemDetail<f64>,
    /// Payout ratio - trailing 12 month
    pub payout_ratio_ttm: ItemDetail<f64>,
    /// Current EV/Free Cash Flow, LFY
    pub ev_to_free_cash_flow_current_annual: ItemDetail<f64>,
    /// Current EV/Free Cash Flow, LTM
    pub ev_to_free_cash_flow_current_ttm: ItemDetail<f64>,
    /// Interest coverage - most recent fiscal year
    pub interest_coverage_annual: ItemDetail<f64>,
    /// Interest coverage - trailing 12 month
    pub interest_coverage_ttm: ItemDetail<f64>,
    /// Free Cash Flow - 1st historical fiscal year
    pub free_cash_flow_historical_annual: ItemDetail<f64>,
    /// Free Cash Flow - trailing 12 month
    pub free_cash_flow_ttm: ItemDetail<f64>,
    /// Revenue - most recent fiscal year
    pub revenue_annual: ItemDetail<f64>,
    /// Revenue - trailing 12 month
    pub revenue_ttm: ItemDetail<f64>,
    /// EBITD - most recent fiscal year
    pub ebitd_annual: ItemDetail<f64>,
    /// EBITD - trailing 12 month
    pub ebitd_ttm: ItemDetail<f64>,
    /// Earnings before taxes - most recent fiscal year
    pub earnings_before_taxes_annual: ItemDetail<f64>,
    /// Earnings before taxes - trailing 12 month
    pub earnings_before_taxes_ttm: ItemDetail<f64>,
    /// Net Income available to common - most recent fiscal year
    pub net_income_to_common_annual: ItemDetail<f64>,
    /// Net Income available to common - trailing 12 months
    pub net_income_to_common_ttm: ItemDetail<f64>,
    /// Earnings before taxes Normalized - most recent fiscal year
    pub normalized_earnings_before_taxes_annual: ItemDetail<f64>,
    /// Net Income Available to Common, Normalized - most recent fiscal year
    pub normalized_net_income_to_common_annual: ItemDetail<f64>,
    /// Earnings per Share, Normalized, Excluding Extraordinary Items, Avg. Diluted Shares Outstanding, TTM
    pub normalized_eps_excluding_extraordinary_ttm: ItemDetail<f64>,
    /// Gross Margin - 1st historical fiscal year
    pub gross_margin_first_historical_year: ItemDetail<f64>,
    /// Gross Margin - trailing 12 month
    pub gross_margin_ttm: ItemDetail<f64>,
    /// Net Profit Margin % - 1st historical fiscal year
    pub net_profit_margin_first_historical_year: ItemDetail<f64>,
    /// Net Profit Margin % - trailing 12 month
    pub net_profit_margin_ttm: ItemDetail<f64>,
    /// Operating margin - 1st historical fiscal year
    pub operating_margin_first_historical_year: ItemDetail<f64>,
    /// Operating margin - trailing 12 month
    pub operating_margin_ttm: ItemDetail<f64>,
    /// Pretax margin - trailing 12 month
    pub pretax_margin_ttm: ItemDetail<f64>,
    /// Pretax margin - 1st historical fiscal year
    pub pretax_margin_first_historical_year: ItemDetail<f64>,
    /// Operating Margin - 5 year average
    pub operating_margin_5yr_avg: ItemDetail<f64>,
    /// Pretax Margin - 5 year average
    pub pretax_margin_5yr_avg: ItemDetail<f64>,
    /// Free Operating Cash Flow/Revenue, 5 Year Average
    pub free_operating_cash_flow_to_revenue_5yr_avg: ItemDetail<f64>,
    /// Free Operating Cash Flow/Revenue, TTM
    pub free_operating_cash_flow_to_revenue_ttm: ItemDetail<f64>,
    /// Gross Margin - 5 year average
    pub gross_margin_5yr_avg: ItemDetail<f64>,
    /// Net Profit Margin - 5 year average
    pub net_profit_margin_5yr_avg: ItemDetail<f64>,
    /// Return on average assets - most recent fiscal year
    pub return_on_average_assets_annual: ItemDetail<f64>,
    /// Return on average assets - trailing 12 month
    pub return_on_average_assets_ttm: ItemDetail<f64>,
    /// Return on average equity - most recent fiscal year
    pub return_on_average_equity_annual: ItemDetail<f64>,
    /// Return on average equity - trailing 12 month
    pub return_on_average_equity_ttm: ItemDetail<f64>,
    /// Return on investment - most recent fiscal year
    pub return_on_investment_annual: ItemDetail<f64>,
    /// Return on investment - trailing 12 month
    pub return_on_investment_ttm: ItemDetail<f64>,
    /// Return on average assets - 5 year average
    pub return_on_average_assets_5yr_avg: ItemDetail<f64>,
    /// Return on average equity - 5 year average
    pub return_on_average_equity_5yr_avg: ItemDetail<f64>,
    /// Return on investment - 5 year average
    pub return_on_investment_5yr_avg: ItemDetail<f64>,
    /// Asset turnover - most recent fiscal year
    pub asset_turnover_annual: ItemDetail<f64>,
    /// Asset turnover - trailing 12 month
    pub asset_turnover_ttm: ItemDetail<f64>,
    /// Inventory turnover - most recent fiscal year
    pub inventory_turnover_annual: ItemDetail<f64>,
    /// Inventory turnover - trailing 12 month
    pub inventory_turnover_ttm: ItemDetail<f64>,
    /// Net Income per employee - most recent fiscal year
    pub net_income_per_employee_annual: ItemDetail<f64>,
    /// Net Income per employee - trailing 12 month
    pub net_income_per_employee_ttm: ItemDetail<f64>,
    /// Receivables turnover - most recent fiscal year
    pub receivables_turnover_annual: ItemDetail<f64>,
    /// Receivables turnover - trailing 12 month
    pub receivables_turnover_ttm: ItemDetail<f64>,
    /// Revenue per employee - most recent fiscal year
    pub revenue_per_employee_annual: ItemDetail<f64>,
    /// Revenue per employee - trailing 12 month
    pub revenue_per_employee_ttm: ItemDetail<f64>,
    /// Revenue Change % - most recent quarter 1 year ago
    pub revenue_change_percent_last_quarter_year_ago: ItemDetail<f64>,
    /// Revenue growth rate, 5 year
    pub revenue_growth_rate_5_year: ItemDetail<f64>,
    /// EPS Change % - most recent quarter 1 year ago
    pub eps_change_percent_last_quarter_year_ago: ItemDetail<f64>,
    /// EPS Change %, TTM over TTM
    pub eps_change_percent_ttm_over_ttm: ItemDetail<f64>,
    /// EPS growth rate, 5 year
    pub eps_growth_rate_5_year: ItemDetail<f64>,
    /// Growth rate% - dividend, 3 year
    pub dividend_growth_rate_3_year: ItemDetail<f64>,
    /// Revenue Change %, TTM over TTM
    pub revenue_change_percent_ttm_over_ttm: ItemDetail<f64>,
    /// Revenue/share (5 yr growth)
    pub revenue_per_share_5_year_growth: ItemDetail<f64>,
    /// Growth rate% - Revenue, 3 year
    pub revenue_growth_rate_3_year: ItemDetail<f64>,
    /// Growth rate% - EPS, 3 year
    pub eps_growth_rate_3_year: ItemDetail<f64>,
    /// Book value per share growth rate, 5 year
    pub book_value_per_share_growth_rate_5_year: ItemDetail<f64>,
    /// Tangible Book Value, Total Equity, 5 Year CAGR
    pub tangible_book_value_total_equity_5yr_cagr: ItemDetail<f64>,
    /// Capital Spending growth rate, 5 year
    pub capital_spending_growth_rate_5_year: ItemDetail<f64>,
    /// Earnings Before Interest, Taxes, Depreciation & Amortization, 5 Year CAGR
    pub ebitda_5_year_cagr: ItemDetail<f64>,
    /// Earnings Before Interest, Taxes, Depreciation & Amortization, 5 Year Interim CAGR
    pub ebitda_5_year_interim_cagr: ItemDetail<f64>,
    /// Free Operating Cash Flow, 5 Year CAGR
    pub free_operating_cash_flow_5_year_cagr: ItemDetail<f64>,
    /// Total Debt, 5 Year CAGR
    pub total_debt_5_year_cagr: ItemDetail<f64>,
    /// Net Profit Margin growth rate, 5 year
    pub net_profit_margin_growth_rate_5_year: ItemDetail<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemDetail<T> {
    pub meaning: String,
    pub value: Option<T>,
}

impl<T> From<&Value> for ItemDetail<T>
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    fn from(item: &Value) -> Self {
        let meaning = item["name"].as_str().unwrap().to_string();
        let value = item
            .get("value")
            .map(|v| v.as_str().unwrap().parse::<T>().unwrap());
        Self { meaning, value }
    }
}

fn fill_ratio(current_ratios: &mut CurrentRatios, item: &Value) {
    match item["id"].as_str().unwrap() {
        "NPRICE" => current_ratios.current_price = item.into(),
        "NHIG" => current_ratios.high_12m = item.into(),
        "NLOW" => current_ratios.low_12m = item.into(),
        "PDATE" => current_ratios.pricing_date = item.into(),
        "VOL10DAVG" => current_ratios.volume_avg_10d = item.into(),
        "MKTCAP" => current_ratios.market_cap = item.into(),
        "NHIGDATE" => current_ratios.high_date_12m = item.into(),
        "NLOWDATE" => current_ratios.low_date_12m = item.into(),
        "VOL3MAVG" => current_ratios.volume_avg_3m = item.into(),
        "BETA" => current_ratios.beta = item.into(),
        "PR1DAYPRC" => current_ratios.price_change_1d = item.into(),
        "PR13WKPCT" => current_ratios.price_change_13w = item.into(),
        "PR26WKPCT" => current_ratios.price_change_26w = item.into(),
        "PR5DAYPRC" => current_ratios.price_change_5d = item.into(),
        "PR52WKPCT" => current_ratios.price_change_52w = item.into(),
        "PRYTDPCT" => current_ratios.price_change_ytd = item.into(),
        "ChPctPriceMTD" => current_ratios.price_change_mtd = item.into(),
        "PR04WKPCTR" => current_ratios.relative_price_change_4w = item.into(),
        "PR13WKPCTR" => current_ratios.relative_price_change_13w = item.into(),
        "PR26WKPCTR" => current_ratios.relative_price_change_26w = item.into(),
        "PR52WKPCTR" => current_ratios.relative_price_change_52w = item.into(),
        "PRYTDPCTR" => current_ratios.relative_price_change_ytd = item.into(),
        "AEPSXCLXOR" => current_ratios.eps_excluding_extraordinary_items_annual = item.into(),
        "TTMEPSXCLX" => current_ratios.eps_excluding_extraordinary_items_ttm = item.into(),
        "AEPSNORM" => current_ratios.eps_normalized_annual = item.into(),
        "AREVPS" => current_ratios.revenue_per_share_annual = item.into(),
        "TTMREVPS" => current_ratios.revenue_per_share_ttm = item.into(),
        "ABVPS" => current_ratios.book_value_per_share_annual = item.into(),
        "QBVPS" => current_ratios.book_value_per_share_quarterly = item.into(),
        "ATANBVPS" => current_ratios.tangible_book_value_per_share_annual = item.into(),
        "QTANBVPS" => current_ratios.tangible_book_value_per_share_quarterly = item.into(),
        "ACSHPS" => current_ratios.cash_per_share_annual = item.into(),
        "QCSHPS" => current_ratios.cash_per_share_quarterly = item.into(),
        "ACFSHR" => current_ratios.cash_flow_per_share_annual = item.into(),
        "TTMCFSHR" => current_ratios.cash_flow_per_share_ttm = item.into(),
        "ADIVSHR" => current_ratios.dividend_per_share_annual = item.into(),
        "TTMDIVSHR" => current_ratios.dividend_per_share_ttm = item.into(),
        "TTMEBITDPS" => current_ratios.ebitd_per_share_ttm = item.into(),
        "ABEPSXCLXO" => current_ratios.abepsxclxo = item.into(),
        "TTMBEPSXCL" => current_ratios.eps_basic_excluding_extraordinary_items_ttm = item.into(),
        "AEPSINCLXO" => current_ratios.eps_including_extraordinary_items_annual = item.into(),
        "TTMEPSINCX" => current_ratios.eps_including_extraordinary_items_ttm = item.into(),
        "TTMFCFSHR" => current_ratios.free_cash_flow_per_share_ttm = item.into(),
        "ADIV5YAVG" => current_ratios.dividend_per_share_5yr_avg = item.into(),
        "APEEXCLXOR" => current_ratios.pe_excluding_extraordinary_items_annual = item.into(),
        "PEEXCLXOR" => current_ratios.pe_excluding_extraordinary_items_ttm = item.into(),
        "APENORM" => current_ratios.pe_normalized_annual = item.into(),
        "APR2REV" => current_ratios.price_to_sales_annual = item.into(),
        "TTMPR2REV" => current_ratios.price_to_sales_ttm = item.into(),
        "APR2TANBK" => current_ratios.price_to_tangible_book_annual = item.into(),
        "PR2TANBK" => current_ratios.price_to_tangible_book_quarterly = item.into(),
        "APRFCFPS" => current_ratios.price_to_free_cash_flow_per_share_annual = item.into(),
        "TTMPRCFPS" => current_ratios.price_to_cash_flow_per_share_ttm = item.into(),
        "TTMPRFCFPS" => current_ratios.price_to_free_cash_flow_per_share_ttm = item.into(),
        "APRICE2BK" => current_ratios.price_to_book_annual = item.into(),
        "PRICE2BK" => current_ratios.price_to_book_quarterly = item.into(),
        "PEBEXCLXOR" => current_ratios.pe_basic_excluding_extraordinary_items_ttm = item.into(),
        "TTMPEHIGH" => current_ratios.pe_high_excluding_extraordinary_items_ttm = item.into(),
        "TTMPELOW" => current_ratios.pe_low_excluding_extraordinary_items_ttm = item.into(),
        "PEINCLXOR" => current_ratios.pe_including_extraordinary_items_ttm = item.into(),
        "NetDebt_I" => current_ratios.net_debt_lfi = item.into(),
        "NetDebt_A" => current_ratios.net_debt_lfy = item.into(),
        "YLD5YAVG" => current_ratios.dividend_yield_5yr_avg = item.into(),
        "YIELD" => current_ratios.dividend_yield = item.into(),
        "DivYield_CurTTM" => current_ratios.current_dividend_yield_ttm = item.into(),
        "ACURRATIO" => current_ratios.current_ratio_annual = item.into(),
        "QCURRATIO" => current_ratios.current_ratio_quarterly = item.into(),
        "AQUICKRATI" => current_ratios.quick_ratio_annual = item.into(),
        "QQUICKRATI" => current_ratios.quick_ratio_quarterly = item.into(),
        "ALTD2EQ" => current_ratios.long_term_debt_to_equity_annual = item.into(),
        "QLTD2EQ" => current_ratios.long_term_debt_to_equity_quarterly = item.into(),
        "ATOTD2EQ" => current_ratios.total_debt_to_equity_annual = item.into(),
        "QTOTD2EQ" => current_ratios.total_debt_to_equity_quarterly = item.into(),
        "APAYRATIO" => current_ratios.payout_ratio_annual = item.into(),
        "TTMPAYRAT" => current_ratios.payout_ratio_ttm = item.into(),
        "EV2FCF_CurA" => current_ratios.ev_to_free_cash_flow_current_annual = item.into(),
        "EV2FCF_CurTTM" => current_ratios.ev_to_free_cash_flow_current_ttm = item.into(),
        "AINTCOV" => current_ratios.interest_coverage_annual = item.into(),
        "TTMINTCOV" => current_ratios.interest_coverage_ttm = item.into(),
        "A1FCF" => current_ratios.free_cash_flow_historical_annual = item.into(),
        "TTMFCF" => current_ratios.free_cash_flow_ttm = item.into(),
        "AREV" => current_ratios.revenue_annual = item.into(),
        "TTMREV" => current_ratios.revenue_ttm = item.into(),
        "AEBITD" => current_ratios.ebitd_annual = item.into(),
        "TTMEBITD" => current_ratios.ebitd_ttm = item.into(),
        "AEBT" => current_ratios.earnings_before_taxes_annual = item.into(),
        "TTMEBT" => current_ratios.earnings_before_taxes_ttm = item.into(),
        "ANIAC" => current_ratios.net_income_to_common_annual = item.into(),
        "TTMNIAC" => current_ratios.net_income_to_common_ttm = item.into(),
        "AEBTNORM" => current_ratios.normalized_earnings_before_taxes_annual = item.into(),
        "ANIACNORM" => current_ratios.normalized_net_income_to_common_annual = item.into(),
        "VDES_TTM" => current_ratios.normalized_eps_excluding_extraordinary_ttm = item.into(),
        "AGROSMGN" => current_ratios.gross_margin_first_historical_year = item.into(),
        "TTMGROSMGN" => current_ratios.gross_margin_ttm = item.into(),
        "ANPMGNPCT" => current_ratios.net_profit_margin_first_historical_year = item.into(),
        "TTMNPMGN" => current_ratios.net_profit_margin_ttm = item.into(),
        "AOPMGNPCT" => current_ratios.operating_margin_first_historical_year = item.into(),
        "TTMOPMGN" => current_ratios.operating_margin_ttm = item.into(),
        "TTMPTMGN" => current_ratios.pretax_margin_ttm = item.into(),
        "APTMGNPCT" => current_ratios.pretax_margin_first_historical_year = item.into(),
        "OPMGN5YR" => current_ratios.operating_margin_5yr_avg = item.into(),
        "PTMGN5YR" => current_ratios.pretax_margin_5yr_avg = item.into(),
        "Focf2Rev_AAvg5" => {
            current_ratios.free_operating_cash_flow_to_revenue_5yr_avg = item.into()
        }
        "Focf2Rev_TTM" => current_ratios.free_operating_cash_flow_to_revenue_ttm = item.into(),
        "GROSMGN5YR" => current_ratios.gross_margin_5yr_avg = item.into(),
        "MARGIN5YR" => current_ratios.net_profit_margin_5yr_avg = item.into(),
        "AROAPCT" => current_ratios.return_on_average_assets_annual = item.into(),
        "TTMROAPCT" => current_ratios.return_on_average_assets_ttm = item.into(),
        "AROEPCT" => current_ratios.return_on_average_equity_annual = item.into(),
        "TTMROEPCT" => current_ratios.return_on_average_equity_ttm = item.into(),
        "AROIPCT" => current_ratios.return_on_investment_annual = item.into(),
        "TTMROIPCT" => current_ratios.return_on_investment_ttm = item.into(),
        "AROA5YAVG" => current_ratios.return_on_average_assets_5yr_avg = item.into(),
        "AROE5YAVG" => current_ratios.return_on_average_equity_5yr_avg = item.into(),
        "AROI5YRAVG" => current_ratios.return_on_investment_5yr_avg = item.into(),
        "AASTTURN" => current_ratios.asset_turnover_annual = item.into(),
        "TTMASTTURN" => current_ratios.asset_turnover_ttm = item.into(),
        "AINVTURN" => current_ratios.inventory_turnover_annual = item.into(),
        "TTMINVTURN" => current_ratios.inventory_turnover_ttm = item.into(),
        "ANIPEREMP" => current_ratios.net_income_per_employee_annual = item.into(),
        "TTMNIPEREM" => current_ratios.net_income_per_employee_ttm = item.into(),
        "ARECTURN" => current_ratios.receivables_turnover_annual = item.into(),
        "TTMRECTURN" => current_ratios.receivables_turnover_ttm = item.into(),
        "AREVPEREMP" => current_ratios.revenue_per_employee_annual = item.into(),
        "TTMREVPERE" => current_ratios.revenue_per_employee_ttm = item.into(),
        "REVCHNGYR" => current_ratios.revenue_change_percent_last_quarter_year_ago = item.into(),
        "REVTRENDGR" => current_ratios.revenue_growth_rate_5_year = item.into(),
        "EPSCHNGYR" => current_ratios.eps_change_percent_last_quarter_year_ago = item.into(),
        "TTMEPSCHG" => current_ratios.eps_change_percent_ttm_over_ttm = item.into(),
        "EPSTRENDGR" => current_ratios.eps_growth_rate_5_year = item.into(),
        "DIVGRPCT" => current_ratios.dividend_growth_rate_3_year = item.into(),
        "TTMREVCHG" => current_ratios.revenue_change_percent_ttm_over_ttm = item.into(),
        "REVPS5YGR" => current_ratios.revenue_per_share_5_year_growth = item.into(),
        "REVGRPCT" => current_ratios.revenue_growth_rate_3_year = item.into(),
        "EPSGRPCT" => current_ratios.eps_growth_rate_3_year = item.into(),
        "BVTRENDGR" => current_ratios.book_value_per_share_growth_rate_5_year = item.into(),
        "TanBV_AYr5CAGR" => current_ratios.tangible_book_value_total_equity_5yr_cagr = item.into(),
        "CSPTRENDGR" => current_ratios.capital_spending_growth_rate_5_year = item.into(),
        "Ebitda_AYr5CAGR" => current_ratios.ebitda_5_year_cagr = item.into(),
        "Ebitda_TTMY5CAGR" => current_ratios.ebitda_5_year_interim_cagr = item.into(),
        "FOCF_AYr5CAGR" => current_ratios.free_operating_cash_flow_5_year_cagr = item.into(),
        "STLD_AYr5CAGR" => current_ratios.total_debt_5_year_cagr = item.into(),
        "NPMTRENDGR" => current_ratios.net_profit_margin_growth_rate_5_year = item.into(),
        _ => {
            panic!("Unknown item id: {}", item["id"].as_str().unwrap())
        }
    }
}

impl From<Value> for CurrentRatios {
    fn from(value: Value) -> Self {
        let mut current_ratios = Self {
            currency: value["currency"].as_str().unwrap().to_string(),
            ..Self::default()
        };
        current_ratios.currency = value["currency"].as_str().unwrap().to_string();
        current_ratios.price_currency = value["priceCurrency"].as_str().unwrap().to_string();
        for group in value["ratiosGroups"].as_array().unwrap() {
            for item in group["items"].as_array().unwrap() {
                fill_ratio(&mut current_ratios, item);
            }
        }
        current_ratios
    }
}

//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct RatiosGroup {
//     pub items: Vec<Item>,
//     pub name: String,
// }
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Item {
//     pub id: String,
//     pub name: String,
//     #[serde(rename = "type")]
//     pub type_field: String,
//     pub value: Option<String>,
// }
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ForecastData {
//     pub consensus_type: String,
//     pub earnings_basis: String,
//     pub end_month: String,
//     pub fiscal_year: String,
//     pub interim_end_cal_month: String,
//     pub interim_end_cal_year: String,
//     pub ratios: Vec<Ratio>,
// }
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Ratio {
//     pub id: String,
//     pub name: String,
//     #[serde(rename = "type")]
//     pub type_field: String,
//     pub value: Option<String>,
// }
