use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use chrono::{Datelike, Month, NaiveDate, TimeZone};
use chrono_tz::{Tz, UTC};
use eyre::{Result, eyre};
use num_traits::FromPrimitive;
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};

use crate::op::{DOp, DateOp};
use crate::span::endpoint::EndpointConversion;
use crate::time::Time;

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
        self.d.cmp(&other.d)
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
        let dt = self.d.and_hms_opt(hour, min, sec).ok_or_else(|| eyre!("invalid hms"))?;
        let dt = self
            .tz()
            .from_local_datetime(&dt)
            .single()
            .ok_or_else(|| eyre!("no single representation for {}", dt))?;
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
        panic!("bug: invalid day {d}");
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
    fn to_open(&self, left: bool) -> Option<Self> {
        let d = if left { self.d.pred_opt() } else { self.d.succ_opt() };
        d.map(|d| Self::new(d, self.tz()))
    }

    fn to_closed(&self, left: bool) -> Option<Self> {
        let d = if left { self.d.succ_opt() } else { self.d.pred_opt() };
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
                let local =
                    s.next().ok_or_else(|| eyre!("missing timestamp")).map_err(E::custom)?;
                let tz = s.next().ok_or_else(|| eyre!("missing timezone")).map_err(E::custom)?;
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
