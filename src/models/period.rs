use chrono::{Duration, NaiveDateTime};
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

impl std::ops::Add<Period> for chrono::DateTime<chrono::Utc> {
    type Output = chrono::DateTime<chrono::Utc>;

    fn add(self, rhs: Period) -> Self::Output {
        match rhs {
            Period::PT1S => self + chrono::Duration::seconds(1),
            Period::PT1M => self + chrono::Duration::minutes(1),
            Period::PT1H => self + chrono::Duration::hours(1),
            Period::P1D => self + chrono::Duration::days(1),
            Period::P1W => self + chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(self, 1),
            Period::P3M => chronoutil::delta::shift_months(self, 3),
            Period::P6M => chronoutil::delta::shift_months(self, 6),
            Period::P1Y => chronoutil::delta::shift_years(self, 1),
            Period::P3Y => chronoutil::delta::shift_years(self, 3),
            Period::P5Y => chronoutil::delta::shift_years(self, 5),
            Period::P50Y => chronoutil::delta::shift_years(self, 50),
        }
    }
}

impl std::ops::Add<Period> for chrono::NaiveDate {
    type Output = chrono::NaiveDate;

    fn add(self, rhs: Period) -> Self::Output {
        match rhs {
            Period::PT1S => self + chrono::Duration::seconds(1),
            Period::PT1M => self + chrono::Duration::minutes(1),
            Period::PT1H => self + chrono::Duration::hours(1),
            Period::P1D => self + chrono::Duration::days(1),
            Period::P1W => self + chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(self, 1),
            Period::P3M => chronoutil::delta::shift_months(self, 3),
            Period::P6M => chronoutil::delta::shift_months(self, 6),
            Period::P1Y => chronoutil::delta::shift_years(self, 1),
            Period::P3Y => chronoutil::delta::shift_years(self, 3),
            Period::P5Y => chronoutil::delta::shift_years(self, 5),
            Period::P50Y => chronoutil::delta::shift_years(self, 50),
        }
    }
}

impl std::ops::Sub<Period> for chrono::DateTime<chrono::Utc> {
    type Output = chrono::DateTime<chrono::Utc>;

    fn sub(self, rhs: Period) -> Self::Output {
        match rhs {
            Period::PT1S => self - chrono::Duration::seconds(1),
            Period::PT1M => self - chrono::Duration::minutes(1),
            Period::PT1H => self - chrono::Duration::hours(1),
            Period::P1D => self - chrono::Duration::days(1),
            Period::P1W => self - chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(self, -1),
            Period::P3M => chronoutil::delta::shift_months(self, -3),
            Period::P6M => chronoutil::delta::shift_months(self, -6),
            Period::P1Y => chronoutil::delta::shift_years(self, -1),
            Period::P3Y => chronoutil::delta::shift_years(self, -3),
            Period::P5Y => chronoutil::delta::shift_years(self, -5),
            Period::P50Y => chronoutil::delta::shift_years(self, -50),
        }
    }
}

impl std::ops::Sub<Period> for chrono::NaiveDate {
    type Output = chrono::NaiveDate;

    fn sub(self, rhs: Period) -> Self::Output {
        match rhs {
            Period::PT1S => self - chrono::Duration::seconds(1),
            Period::PT1M => self - chrono::Duration::minutes(1),
            Period::PT1H => self - chrono::Duration::hours(1),
            Period::P1D => self - chrono::Duration::days(1),
            Period::P1W => self - chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(self, -1),
            Period::P3M => chronoutil::delta::shift_months(self, -3),
            Period::P6M => chronoutil::delta::shift_months(self, -6),
            Period::P1Y => chronoutil::delta::shift_years(self, -1),
            Period::P3Y => chronoutil::delta::shift_years(self, -3),
            Period::P5Y => chronoutil::delta::shift_years(self, -5),
            Period::P50Y => chronoutil::delta::shift_years(self, -50),
        }
    }
}

impl std::ops::AddAssign<Period> for chrono::DateTime<chrono::Utc> {
    fn add_assign(&mut self, rhs: Period) {
        *self = match rhs {
            Period::PT1S => *self + chrono::Duration::seconds(1),
            Period::PT1M => *self + chrono::Duration::minutes(1),
            Period::PT1H => *self + chrono::Duration::hours(1),
            Period::P1D => *self + chrono::Duration::days(1),
            Period::P1W => *self + chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(*self, 1),
            Period::P3M => chronoutil::delta::shift_months(*self, 3),
            Period::P6M => chronoutil::delta::shift_months(*self, 6),
            Period::P1Y => chronoutil::delta::shift_years(*self, 1),
            Period::P3Y => chronoutil::delta::shift_years(*self, 3),
            Period::P5Y => chronoutil::delta::shift_years(*self, 5),
            Period::P50Y => chronoutil::delta::shift_years(*self, 50),
        };
    }
}

impl std::ops::SubAssign<Period> for chrono::DateTime<chrono::Utc> {
    fn sub_assign(&mut self, rhs: Period) {
        *self = match rhs {
            Period::PT1S => *self - chrono::Duration::seconds(1),
            Period::PT1M => *self - chrono::Duration::minutes(1),
            Period::PT1H => *self - chrono::Duration::hours(1),
            Period::P1D => *self - chrono::Duration::days(1),
            Period::P1W => *self - chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(*self, -1),
            Period::P3M => chronoutil::delta::shift_months(*self, -3),
            Period::P6M => chronoutil::delta::shift_months(*self, -6),
            Period::P1Y => chronoutil::delta::shift_years(*self, -1),
            Period::P3Y => chronoutil::delta::shift_years(*self, -3),
            Period::P5Y => chronoutil::delta::shift_years(*self, -5),
            Period::P50Y => chronoutil::delta::shift_years(*self, -50),
        };
    }
}

impl std::ops::Add<Period> for chrono::NaiveDateTime {
    type Output = chrono::NaiveDateTime;

    fn add(self, rhs: Period) -> Self::Output {
        match rhs {
            Period::PT1S => self + chrono::Duration::seconds(1),
            Period::PT1M => self + chrono::Duration::minutes(1),
            Period::PT1H => self + chrono::Duration::hours(1),
            Period::P1D => self + chrono::Duration::days(1),
            Period::P1W => self + chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(self, 1),
            Period::P3M => chronoutil::delta::shift_months(self, 3),
            Period::P6M => chronoutil::delta::shift_months(self, 6),
            Period::P1Y => chronoutil::delta::shift_years(self, 1),
            Period::P3Y => chronoutil::delta::shift_years(self, 3),
            Period::P5Y => chronoutil::delta::shift_years(self, 5),
            Period::P50Y => chronoutil::delta::shift_years(self, 50),
        }
    }
}

impl std::ops::AddAssign<Period> for chrono::NaiveDateTime {
    fn add_assign(&mut self, rhs: Period) {
        *self = match rhs {
            Period::PT1S => *self + chrono::Duration::seconds(1),
            Period::PT1M => *self + chrono::Duration::minutes(1),
            Period::PT1H => *self + chrono::Duration::hours(1),
            Period::P1D => *self + chrono::Duration::days(1),
            Period::P1W => *self + chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(*self, 1),
            Period::P3M => chronoutil::delta::shift_months(*self, 3),
            Period::P6M => chronoutil::delta::shift_months(*self, 6),
            Period::P1Y => chronoutil::delta::shift_years(*self, 1),
            Period::P3Y => chronoutil::delta::shift_years(*self, 3),
            Period::P5Y => chronoutil::delta::shift_years(*self, 5),
            Period::P50Y => chronoutil::delta::shift_years(*self, 50),
        };
    }
}

impl std::ops::Sub<Period> for chrono::NaiveDateTime {
    type Output = chrono::NaiveDateTime;

    fn sub(self, rhs: Period) -> Self::Output {
        match rhs {
            Period::PT1S => self - chrono::Duration::seconds(1),
            Period::PT1M => self - chrono::Duration::minutes(1),
            Period::PT1H => self - chrono::Duration::hours(1),
            Period::P1D => self - chrono::Duration::days(1),
            Period::P1W => self - chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(self, -1),
            Period::P3M => chronoutil::delta::shift_months(self, -3),
            Period::P6M => chronoutil::delta::shift_months(self, -6),
            Period::P1Y => chronoutil::delta::shift_years(self, -1),
            Period::P3Y => chronoutil::delta::shift_years(self, -3),
            Period::P5Y => chronoutil::delta::shift_years(self, -5),
            Period::P50Y => chronoutil::delta::shift_years(self, -50),
        }
    }
}

impl std::ops::SubAssign<Period> for chrono::NaiveDateTime {
    fn sub_assign(&mut self, rhs: Period) {
        *self = match rhs {
            Period::PT1S => *self - chrono::Duration::seconds(1),
            Period::PT1M => *self - chrono::Duration::minutes(1),
            Period::PT1H => *self - chrono::Duration::hours(1),
            Period::P1D => *self - chrono::Duration::days(1),
            Period::P1W => *self - chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(*self, -1),
            Period::P3M => chronoutil::delta::shift_months(*self, -3),
            Period::P6M => chronoutil::delta::shift_months(*self, -6),
            Period::P1Y => chronoutil::delta::shift_years(*self, -1),
            Period::P3Y => chronoutil::delta::shift_years(*self, -3),
            Period::P5Y => chronoutil::delta::shift_years(*self, -5),
            Period::P50Y => chronoutil::delta::shift_years(*self, -50),
        };
    }
}

impl std::ops::AddAssign<Period> for chrono::NaiveDate {
    fn add_assign(&mut self, rhs: Period) {
        *self = match rhs {
            Period::PT1S => *self + chrono::Duration::seconds(1),
            Period::PT1M => *self + chrono::Duration::minutes(1),
            Period::PT1H => *self + chrono::Duration::hours(1),
            Period::P1D => *self + chrono::Duration::days(1),
            Period::P1W => *self + chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(*self, 1),
            Period::P3M => chronoutil::delta::shift_months(*self, 3),
            Period::P6M => chronoutil::delta::shift_months(*self, 6),
            Period::P1Y => chronoutil::delta::shift_years(*self, 1),
            Period::P3Y => chronoutil::delta::shift_years(*self, 3),
            Period::P5Y => chronoutil::delta::shift_years(*self, 5),
            Period::P50Y => chronoutil::delta::shift_years(*self, 50),
        };
    }
}

impl std::ops::SubAssign<Period> for chrono::NaiveDate {
    fn sub_assign(&mut self, rhs: Period) {
        *self = match rhs {
            Period::PT1S => *self - chrono::Duration::seconds(1),
            Period::PT1M => *self - chrono::Duration::minutes(1),
            Period::PT1H => *self - chrono::Duration::hours(1),
            Period::P1D => *self - chrono::Duration::days(1),
            Period::P1W => *self - chrono::Duration::weeks(1),
            Period::P1M => chronoutil::delta::shift_months(*self, -1),
            Period::P3M => chronoutil::delta::shift_months(*self, -3),
            Period::P6M => chronoutil::delta::shift_months(*self, -6),
            Period::P1Y => chronoutil::delta::shift_years(*self, -1),
            Period::P3Y => chronoutil::delta::shift_years(*self, -3),
            Period::P5Y => chronoutil::delta::shift_years(*self, -5),
            Period::P50Y => chronoutil::delta::shift_years(*self, -50),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_add_period_to_datetime() {
        let now = Utc::now();
        let one_day_later = now + Period::P1D;
        assert_eq!(one_day_later, now + chrono::Duration::days(1));
    }

    #[test]
    fn test_sub_period_from_datetime() {
        let now = Utc::now();
        let one_hour_earlier = now - Period::PT1H;
        assert_eq!(one_hour_earlier, now - chrono::Duration::hours(1));
    }

    #[test]
    fn test_add_assign_period_to_datetime() {
        let mut now = Utc::now();
        let expected = now + Period::P1W;
        now += Period::P1W;
        assert_eq!(now, expected);
    }

    #[test]
    fn test_sub_assign_period_from_datetime() {
        let mut now = Utc::now();
        let expected = now - Period::P1M;
        now -= Period::P1M;
        assert_eq!(now, expected);
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
