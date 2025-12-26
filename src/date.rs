use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use chrono::{Datelike, Month, NaiveDate, TimeZone};
use chrono_tz::{Tz, UTC};
use num_traits::FromPrimitive;
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};

use crate::op::{DOp, DateOp};
use crate::span::endpoint::{EndpointConversion, EndpointSide};
use crate::time::Time;
use crate::{Error, Result};

pub fn ymd<T: Borrow<Tz>>(y: i32, m: u32, d: u32, tz: T) -> Date {
    Date::new(NaiveDate::from_ymd_opt(y, m, d).unwrap(), *tz.borrow())
}

#[must_use]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Day {
    // This is the same as python datetime.weekday() and pandas.
    Mon = 0,
    Tue = 1,
    Wed = 2,
    Thu = 3,
    Fri = 4,
    Sat = 5,
    Sun = 6,
}

#[must_use]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Date {
    d: NaiveDate,
    tz: Tz,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fmt("%Y-%m-%d"))
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.d.cmp(&other.d) {
            Ordering::Equal => self.tz.name().cmp(other.tz.name()),
            ord => ord,
        }
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for Date {
    fn default() -> Self {
        Self::new(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(), UTC)
    }
}

impl From<Date> for NaiveDate {
    fn from(v: Date) -> Self {
        v.inner()
    }
}

impl Date {
    pub const fn new(d: NaiveDate, tz: Tz) -> Self {
        Self { d, tz }
    }

    #[must_use]
    pub fn fmt(&self, f: &str) -> String {
        self.d.format(f).to_string()
    }

    pub fn from_ymd(s: &str, tz: Tz) -> Result<Self> {
        Self::from_fmt(s, "%Y-%m-%d", tz)
    }

    pub fn from_fmt(s: &str, fmt: &str, tz: Tz) -> Result<Self> {
        Ok(Self { d: NaiveDate::parse_from_str(s, fmt)?, tz })
    }

    #[must_use]
    pub fn inner(&self) -> NaiveDate {
        self.d
    }

    pub fn op(op: DOp, n: i64) -> DateOp {
        DateOp::new(op, n)
    }

    pub fn apply(&self, op: DateOp) -> Self {
        op.apply(*self)
    }

    #[must_use]
    pub fn tz(&self) -> Tz {
        self.tz
    }

    pub fn and_hms(&self, hour: u32, min: u32, sec: u32) -> Result<Time> {
        let dt = self.d.and_hms_opt(hour, min, sec).ok_or(Error::InvalidTimeComponents)?;
        let dt = self
            .tz()
            .from_local_datetime(&dt)
            .single()
            .ok_or_else(|| Error::InvalidLocalDateTime(dt.to_string()))?;
        Ok(Time::new(dt))
    }

    pub fn time(&self) -> Result<Time> {
        self.and_hms(0, 0, 0)
    }

    #[must_use]
    pub fn day(&self) -> u32 {
        self.d.day()
    }

    pub fn with_day(&self, d: u32) -> Self {
        for max in (28..=31).rev() {
            if let Some(res) = self.d.with_day(d.clamp(1, max)) {
                return Self::new(res, self.tz());
            }
        }
        unreachable!()
    }

    pub fn add_days(&self, d: i32) -> Self {
        Self::new(self.d + chrono::Duration::try_days(i64::from(d)).unwrap(), self.tz())
    }

    pub fn weekday(&self) -> Day {
        match self.d.weekday() {
            chrono::Weekday::Mon => Day::Mon,
            chrono::Weekday::Tue => Day::Tue,
            chrono::Weekday::Wed => Day::Wed,
            chrono::Weekday::Thu => Day::Thu,
            chrono::Weekday::Fri => Day::Fri,
            chrono::Weekday::Sat => Day::Sat,
            chrono::Weekday::Sun => Day::Sun,
        }
    }

    #[must_use]
    pub fn month_name(&self) -> String {
        Month::from_u32(self.month()).unwrap().name().to_owned()
    }

    #[must_use]
    pub fn month0(&self) -> u32 {
        self.d.month0()
    }

    #[must_use]
    pub fn month(&self) -> u32 {
        self.d.month()
    }

    pub fn with_month(&self, m: u32) -> Self {
        let d = self.day();
        Self::new(self.with_day(1).d.with_month(m).unwrap(), self.tz()).with_day(d)
    }

    pub fn add_months(&self, add_m: i32) -> Self {
        let d = self.day();
        let total_m = self.month0() as i32 + add_m;
        let y = total_m.div_euclid(12) + self.year();
        let m = total_m.rem_euclid(12) as u32 + 1;
        ymd(y, m, 1, self.tz()).with_day(d)
    }

    #[must_use]
    pub fn year(&self) -> i32 {
        self.d.year()
    }

    pub fn with_year(&self, y: i32) -> Self {
        let d = self.day();
        Self::new(self.with_day(1).d.with_year(y).unwrap(), self.tz()).with_day(d)
    }

    pub fn add_years(&self, y: i32) -> Self {
        self.with_year(self.year() + y)
    }
}

impl EndpointConversion for Date {
    fn to_open(&self, side: EndpointSide) -> Option<Self> {
        let d = match side {
            EndpointSide::Left => self.d.pred_opt(),
            EndpointSide::Right => self.d.succ_opt(),
        };
        d.map(|d| Self::new(d, self.tz()))
    }

    fn to_closed(&self, side: EndpointSide) -> Option<Self> {
        let d = match side {
            EndpointSide::Left => self.d.succ_opt(),
            EndpointSide::Right => self.d.pred_opt(),
        };
        d.map(|d| Self::new(d, self.tz()))
    }
}

impl<'a> Deserialize<'a> for Date {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        struct DateVisitor;

        impl Visitor<'_> for DateVisitor {
            type Value = Date;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("date string %Y-%m-%d and timezone name")
            }

            fn visit_str<E>(self, v: &str) -> Result<Date, E>
            where
                E: de::Error,
            {
                let mut s = v.split_whitespace();
                let local = s.next().ok_or_else(|| E::custom("missing timestamp"))?;
                let tz = s.next().ok_or_else(|| E::custom("missing timezone"))?;
                let tz = Tz::from_str(tz).map_err(E::custom)?;
                let time = Date::from_ymd(local, tz).map_err(E::custom)?;
                Ok(time)
            }
        }

        d.deserialize_string(DateVisitor)
    }
}

impl Serialize for Date {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&(self.fmt("%Y-%m-%d") + " " + self.tz().name()))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use chrono_tz::US::Eastern;
    use chrono_tz::UTC;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn date_new() {
        let naive = NaiveDate::from_ymd_opt(2020, 3, 15).unwrap();
        let d = Date::new(naive, Eastern);
        assert_eq!(d.year(), 2020);
        assert_eq!(d.month(), 3);
        assert_eq!(d.day(), 15);
    }

    #[test]
    fn date_fmt() {
        let d = ymd(2020, 3, 15, Eastern);
        assert_eq!(d.fmt("%Y-%m-%d"), "2020-03-15");
        assert_eq!(d.fmt("%m/%d/%Y"), "03/15/2020");
        assert_eq!(d.fmt("%d %b %Y"), "15 Mar 2020");
    }

    #[test]
    fn date_from_ymd() -> Result<()> {
        let d = Date::from_ymd("2020-03-15", Eastern)?;
        assert_eq!(d.year(), 2020);
        assert_eq!(d.month(), 3);
        assert_eq!(d.day(), 15);
        Ok(())
    }

    #[test]
    fn date_from_fmt() -> Result<()> {
        let d = Date::from_fmt("03/15/2020", "%m/%d/%Y", Eastern)?;
        assert_eq!(d.year(), 2020);
        assert_eq!(d.month(), 3);
        assert_eq!(d.day(), 15);
        Ok(())
    }

    #[test]
    fn weekday() {
        assert_eq!(ymd(2020, 3, 16, Eastern).weekday(), Day::Mon);
        assert_eq!(ymd(2020, 3, 20, Eastern).weekday(), Day::Fri);
        assert_eq!(ymd(2020, 3, 15, Eastern).weekday(), Day::Sun);
    }

    #[test]
    fn month_name() {
        assert_eq!(ymd(2020, 1, 1, Eastern).month_name(), "January");
        assert_eq!(ymd(2020, 12, 1, Eastern).month_name(), "December");
    }

    #[test]
    fn month0() {
        let d = ymd(2020, 1, 15, Eastern);
        assert_eq!(d.month0(), 0);

        let d = ymd(2020, 12, 15, Eastern);
        assert_eq!(d.month0(), 11);
    }

    #[test]
    fn with_day() {
        let d = ymd(2020, 3, 15, Eastern);
        let d2 = d.with_day(20);
        assert_eq!(d2.day(), 20);
        assert_eq!(d2.month(), 3);
        assert_eq!(d2.year(), 2020);
    }

    #[test]
    fn with_day_clamp() {
        // February 2020 is a leap year
        let d = ymd(2020, 1, 31, Eastern);
        let d2 = d.with_day(31).with_month(2);
        assert_eq!(d2.day(), 29); // Clamped to Feb 29

        // February 2019 is not a leap year
        let d = ymd(2019, 1, 31, Eastern);
        let d2 = d.with_day(31).with_month(2);
        assert_eq!(d2.day(), 28); // Clamped to Feb 28
    }

    #[test]
    fn add_days() {
        let d = ymd(2020, 3, 15, Eastern);
        let d2 = d.add_days(5);
        assert_eq!(d2.day(), 20);

        let d3 = d.add_days(-5);
        assert_eq!(d3.day(), 10);
    }

    #[test]
    fn add_days_across_month() {
        let d = ymd(2020, 1, 30, Eastern);
        let d2 = d.add_days(5);
        assert_eq!(d2.month(), 2);
        assert_eq!(d2.day(), 4);
    }

    #[test]
    fn with_month() {
        let d = ymd(2020, 3, 15, Eastern);
        let d2 = d.with_month(7);
        assert_eq!(d2.month(), 7);
        assert_eq!(d2.day(), 15);
        assert_eq!(d2.year(), 2020);
    }

    #[test]
    fn add_months() {
        let d = ymd(2020, 1, 15, Eastern);
        let d2 = d.add_months(3);
        assert_eq!(d2.month(), 4);
        assert_eq!(d2.year(), 2020);

        let d3 = d.add_months(-3);
        assert_eq!(d3.month(), 10);
        assert_eq!(d3.year(), 2019);
    }

    #[test]
    fn add_months_iteration_over_year() {
        let start = ymd(2020, 1, 15, Eastern);

        let mut dates = Vec::new();
        let mut current = start;
        for _ in 0..12 {
            dates.push(current);
            current = current.add_months(1);
        }

        assert_eq!(dates.len(), 12);

        for (i, date) in dates.iter().enumerate() {
            assert_eq!(date.day(), 15);
            assert_eq!(date.month(), i as u32 + 1);
            assert_eq!(date.year(), 2020);
        }
    }

    #[test]
    fn add_months_across_year() {
        let d = ymd(2020, 11, 15, Eastern);
        let d2 = d.add_months(3);
        assert_eq!(d2.month(), 2);
        assert_eq!(d2.year(), 2021);

        let d3 = d.add_months(-13);
        assert_eq!(d3.month(), 10);
        assert_eq!(d3.year(), 2019);
    }

    #[test]
    fn add_months_day_clamp() {
        // Jan 31 + 1 month = Feb 29 (in 2020, leap year)
        let d = ymd(2020, 1, 31, Eastern);
        let d2 = d.add_months(1);
        assert_eq!(d2.month(), 2);
        assert_eq!(d2.day(), 29);
    }

    #[test]
    fn with_year() {
        let d = ymd(2020, 3, 15, Eastern);
        let d2 = d.with_year(2025);
        assert_eq!(d2.year(), 2025);
        assert_eq!(d2.month(), 3);
        assert_eq!(d2.day(), 15);
    }

    #[test]
    fn add_years() {
        let d = ymd(2020, 3, 15, Eastern);
        let d2 = d.add_years(5);
        assert_eq!(d2.year(), 2025);

        let d3 = d.add_years(-5);
        assert_eq!(d3.year(), 2015);
    }

    #[test]
    fn add_years_leap_day() {
        // Feb 29, 2020 + 1 year = Feb 28, 2021
        let d = ymd(2020, 2, 29, Eastern);
        let d2 = d.add_years(1);
        assert_eq!(d2.year(), 2021);
        assert_eq!(d2.month(), 2);
        assert_eq!(d2.day(), 28);
    }

    #[test]
    fn and_hms() -> Result<()> {
        let d = ymd(2020, 3, 15, Eastern);
        let t = d.and_hms(14, 30, 45)?;
        assert_eq!(t.hour(), 14);
        assert_eq!(t.minute(), 30);
        assert_eq!(t.second(), 45);
        assert_eq!(t.date(), d);
        Ok(())
    }

    #[test]
    fn time() -> Result<()> {
        let d = ymd(2020, 3, 15, Eastern);
        let t = d.time()?;
        assert_eq!(t.hour(), 0);
        assert_eq!(t.minute(), 0);
        assert_eq!(t.second(), 0);
        assert_eq!(t.date(), d);
        Ok(())
    }

    #[test]
    fn ordering() {
        let d1 = ymd(2020, 3, 15, Eastern);
        let d2 = ymd(2020, 3, 16, Eastern);
        let d3 = ymd(2020, 3, 15, Eastern);

        assert!(d1 < d2);
        assert!(d2 > d1);
        assert_eq!(d1, d3);
        assert_eq!(d1.cmp(&d3), Ordering::Equal);

        // Dates with differing timezones should not compare as equal for ordering.
        let d4 = ymd(2020, 3, 15, UTC);
        assert_ne!(d1, d4);
        assert_ne!(d1.cmp(&d4), Ordering::Equal);
    }

    #[test]
    fn btreeset_can_hold_distinct_timezones_for_same_naive_date() {
        use std::collections::BTreeSet;

        let d_eastern = ymd(2020, 3, 15, Eastern);
        let d_utc = ymd(2020, 3, 15, UTC);
        assert_ne!(d_eastern, d_utc);
        assert_ne!(d_eastern.cmp(&d_utc), Ordering::Equal);

        let mut set = BTreeSet::new();
        assert!(set.insert(d_eastern));
        assert!(set.insert(d_utc));
        assert_eq!(set.len(), 2);
        assert!(set.remove(&d_utc));
        assert_eq!(set.len(), 1);
        assert!(set.contains(&d_eastern));
    }

    #[test]
    fn default() {
        let d = Date::default();
        assert_eq!(d.year(), 1970);
        assert_eq!(d.month(), 1);
        assert_eq!(d.day(), 1);
        assert_eq!(d.tz(), UTC);
    }

    #[test]
    fn endpoint_conversion() {
        let d = ymd(2020, 3, 15, Eastern);

        // to_open on left should give previous day
        let left_open = d.to_open(EndpointSide::Left).unwrap();
        assert_eq!(left_open, ymd(2020, 3, 14, Eastern));

        // to_open on right should give next day
        let right_open = d.to_open(EndpointSide::Right).unwrap();
        assert_eq!(right_open, ymd(2020, 3, 16, Eastern));

        // to_closed on left should give next day
        let left_closed = d.to_closed(EndpointSide::Left).unwrap();
        assert_eq!(left_closed, ymd(2020, 3, 16, Eastern));

        // to_closed on right should give previous day
        let right_closed = d.to_closed(EndpointSide::Right).unwrap();
        assert_eq!(right_closed, ymd(2020, 3, 14, Eastern));
    }

    #[test]
    fn serialization() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let d = ymd(2020, 3, 15, Eastern);
        let se = serde_json::to_string(&d)?;
        let de: Date = serde_json::from_str(&se)?;
        assert_eq!(de.year(), d.year());
        assert_eq!(de.month(), d.month());
        assert_eq!(de.day(), d.day());
        assert_eq!(de.tz(), d.tz());
        Ok(())
    }

    #[test]
    fn display() {
        let d = ymd(2020, 3, 15, Eastern);
        assert_eq!(format!("{}", d), "2020-03-15");
    }

    #[test]
    fn from_into_naive_date() {
        let d = ymd(2020, 3, 15, Eastern);
        let naive: NaiveDate = d.into();
        assert_eq!(naive.year(), 2020);
        assert_eq!(naive.month(), 3);
        assert_eq!(naive.day(), 15);
    }

    #[test]
    fn edge_case_dates() {
        // Test end of month boundaries
        let d = ymd(2020, 1, 31, Eastern);
        assert_eq!(d.add_days(1).month(), 2);

        let d = ymd(2020, 2, 29, Eastern); // Leap year
        assert_eq!(d.add_days(1).month(), 3);

        let d = ymd(2019, 2, 28, Eastern); // Non-leap year
        assert_eq!(d.add_days(1).month(), 3);
    }
}
