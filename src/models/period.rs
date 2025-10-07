use chrono::Duration;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    EnumString,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
)]
pub enum Period {
    PT1S,
    PT1M,
    PT1H,
    P1D,
    P1W,
    P1M,
    P3M,
    P6M,
    P1Y,
    P3Y,
    P5Y,
    P50Y,
}

impl Period {
    /// Convert period to milliseconds
    pub fn to_ms(&self) -> u64 {
        match &self {
            Self::PT1S => 1000,
            Self::PT1M => 1000 * 60,
            Self::PT1H => 1000 * 60 * 60,
            Self::P1D => 1000 * 60 * 60 * 24,
            Self::P1W => 1000 * 60 * 60 * 24 * 7,
            Self::P1M => 1000 * 60 * 60 * 24 * 30,
            Self::P3M => 1000 * 60 * 60 * 24 * 30 * 3,
            Self::P6M => 1000 * 60 * 60 * 24 * 30 * 6,
            Self::P1Y => 1000 * 60 * 60 * 24 * 365,
            Self::P3Y => 1000 * 60 * 60 * 24 * 365 * 3,
            Self::P5Y => 1000 * 60 * 60 * 24 * 365 * 5,
            Self::P50Y => 1000 * 60 * 60 * 24 * 365 * 50,
        }
    }

    /// Add this period to a DateTime<Utc>
    pub fn add_to_datetime(
        &self,
        datetime: chrono::DateTime<chrono::Utc>,
    ) -> chrono::DateTime<chrono::Utc> {
        match self {
            Period::PT1S => datetime + chrono::Duration::seconds(1),
            Period::PT1M => datetime + chrono::Duration::minutes(1),
            Period::PT1H => datetime + chrono::Duration::hours(1),
            Period::P1D => datetime + chrono::Duration::days(1),
            Period::P1W => datetime + chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(datetime, 1),
            Period::P3M => chronoutil::delta::shift_months(datetime, 3),
            Period::P6M => chronoutil::delta::shift_months(datetime, 6),
            Period::P1Y => chronoutil::delta::shift_years(datetime, 1),
            Period::P3Y => chronoutil::delta::shift_years(datetime, 3),
            Period::P5Y => chronoutil::delta::shift_years(datetime, 5),
            Period::P50Y => chronoutil::delta::shift_years(datetime, 50),
        }
    }

    /// Subtract this period from a DateTime<Utc>
    pub fn subtract_from_datetime(
        &self,
        datetime: chrono::DateTime<chrono::Utc>,
    ) -> chrono::DateTime<chrono::Utc> {
        match self {
            Period::PT1S => datetime - chrono::Duration::seconds(1),
            Period::PT1M => datetime - chrono::Duration::minutes(1),
            Period::PT1H => datetime - chrono::Duration::hours(1),
            Period::P1D => datetime - chrono::Duration::days(1),
            Period::P1W => datetime - chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(datetime, -1),
            Period::P3M => chronoutil::delta::shift_months(datetime, -3),
            Period::P6M => chronoutil::delta::shift_months(datetime, -6),
            Period::P1Y => chronoutil::delta::shift_years(datetime, -1),
            Period::P3Y => chronoutil::delta::shift_years(datetime, -3),
            Period::P5Y => chronoutil::delta::shift_years(datetime, -5),
            Period::P50Y => chronoutil::delta::shift_years(datetime, -50),
        }
    }

    /// Add this period to a NaiveDate
    pub fn add_to_date(&self, date: chrono::NaiveDate) -> chrono::NaiveDate {
        match self {
            Period::PT1S => date + chrono::Duration::seconds(1),
            Period::PT1M => date + chrono::Duration::minutes(1),
            Period::PT1H => date + chrono::Duration::hours(1),
            Period::P1D => date + chrono::Duration::days(1),
            Period::P1W => date + chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(date, 1),
            Period::P3M => chronoutil::delta::shift_months(date, 3),
            Period::P6M => chronoutil::delta::shift_months(date, 6),
            Period::P1Y => chronoutil::delta::shift_years(date, 1),
            Period::P3Y => chronoutil::delta::shift_years(date, 3),
            Period::P5Y => chronoutil::delta::shift_years(date, 5),
            Period::P50Y => chronoutil::delta::shift_years(date, 50),
        }
    }

    /// Subtract this period from a NaiveDate
    pub fn subtract_from_date(&self, date: chrono::NaiveDate) -> chrono::NaiveDate {
        match self {
            Period::PT1S => date - chrono::Duration::seconds(1),
            Period::PT1M => date - chrono::Duration::minutes(1),
            Period::PT1H => date - chrono::Duration::hours(1),
            Period::P1D => date - chrono::Duration::days(1),
            Period::P1W => date - chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(date, -1),
            Period::P3M => chronoutil::delta::shift_months(date, -3),
            Period::P6M => chronoutil::delta::shift_months(date, -6),
            Period::P1Y => chronoutil::delta::shift_years(date, -1),
            Period::P3Y => chronoutil::delta::shift_years(date, -3),
            Period::P5Y => chronoutil::delta::shift_years(date, -5),
            Period::P50Y => chronoutil::delta::shift_years(date, -50),
        }
    }

    /// Add this period to a NaiveDateTime
    pub fn add_to_datetime_naive(&self, datetime: chrono::NaiveDateTime) -> chrono::NaiveDateTime {
        match self {
            Period::PT1S => datetime + chrono::Duration::seconds(1),
            Period::PT1M => datetime + chrono::Duration::minutes(1),
            Period::PT1H => datetime + chrono::Duration::hours(1),
            Period::P1D => datetime + chrono::Duration::days(1),
            Period::P1W => datetime + chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(datetime, 1),
            Period::P3M => chronoutil::delta::shift_months(datetime, 3),
            Period::P6M => chronoutil::delta::shift_months(datetime, 6),
            Period::P1Y => chronoutil::delta::shift_years(datetime, 1),
            Period::P3Y => chronoutil::delta::shift_years(datetime, 3),
            Period::P5Y => chronoutil::delta::shift_years(datetime, 5),
            Period::P50Y => chronoutil::delta::shift_years(datetime, 50),
        }
    }

    /// Subtract this period from a NaiveDateTime
    pub fn subtract_from_datetime_naive(
        &self,
        datetime: chrono::NaiveDateTime,
    ) -> chrono::NaiveDateTime {
        match self {
            Period::PT1S => datetime - chrono::Duration::seconds(1),
            Period::PT1M => datetime - chrono::Duration::minutes(1),
            Period::PT1H => datetime - chrono::Duration::hours(1),
            Period::P1D => datetime - chrono::Duration::days(1),
            Period::P1W => datetime - chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(datetime, -1),
            Period::P3M => chronoutil::delta::shift_months(datetime, -3),
            Period::P6M => chronoutil::delta::shift_months(datetime, -6),
            Period::P1Y => chronoutil::delta::shift_years(datetime, -1),
            Period::P3Y => chronoutil::delta::shift_years(datetime, -3),
            Period::P5Y => chronoutil::delta::shift_years(datetime, -5),
            Period::P50Y => chronoutil::delta::shift_years(datetime, -50),
        }
    }
}

impl From<Period> for Duration {
    fn from(period: Period) -> Self {
        match period {
            Period::PT1S => Duration::seconds(1),
            Period::PT1M => Duration::minutes(1),
            Period::PT1H => Duration::hours(1),
            Period::P1D => Duration::days(1),
            Period::P1W => Duration::weeks(1),
            Period::P1M => Duration::days(30),
            Period::P3M => Duration::days(90),
            Period::P6M => Duration::days(180),
            Period::P1Y => Duration::days(365),
            Period::P3Y => Duration::days(1095),
            Period::P5Y => Duration::days(1825),
            Period::P50Y => Duration::days(18250),
        }
    }
}

// Operator overloading implementations removed for simplicity and clarity.
// Use the explicit methods above instead:
// - date - period  =>  period.subtract_from_date(date)
// - datetime + period  =>  period.add_to_datetime(datetime)
// - etc.

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_add_period_to_datetime() {
        let now = Utc::now();
        let one_day_later = Period::P1D.add_to_datetime(now);
        assert_eq!(one_day_later, now + chrono::Duration::days(1));
    }

    #[test]
    fn test_sub_period_from_datetime() {
        let now = Utc::now();
        let one_hour_earlier = Period::PT1H.subtract_from_datetime(now);
        assert_eq!(one_hour_earlier, now - chrono::Duration::hours(1));
    }

    #[test]
    fn test_add_period_to_date() {
        let today = chrono::Utc::now().date_naive();
        let one_week_later = Period::P1W.add_to_date(today);
        assert_eq!(one_week_later, today + chrono::Duration::weeks(1));
    }

    #[test]
    fn test_sub_period_from_date() {
        let today = chrono::Utc::now().date_naive();
        let one_month_earlier = Period::P1M.subtract_from_date(today);
        assert_eq!(
            one_month_earlier,
            chronoutil::delta::shift_months(today, -1)
        );
    }

    #[test]
    fn test_period_to_duration() {
        let period = Period::P1Y;
        let duration: Duration = period.into();
        assert_eq!(duration, Duration::days(365));
    }

    #[test]
    fn test_period_to_ms() {
        let period = Period::PT1M;
        assert_eq!(period.to_ms(), 60 * 1000);
    }
}
