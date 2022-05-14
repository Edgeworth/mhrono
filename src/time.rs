use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::str::FromStr;

use chrono::{DateTime, Datelike, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Timelike};
use chrono_tz::{Tz, UTC};
use derive_more::Display;
use eyre::{eyre, Result};
use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};

use crate::date::{Date, Day};
use crate::duration::{Duration, SEC};
use crate::op::{TOp, TimeOp};

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Display, Ord, PartialOrd)]
#[display(fmt = "{}", t)]
pub struct Time {
    t: DateTime<Tz>,
}

/// Creation
impl Time {
    #[must_use]
    pub const fn new(t: DateTime<Tz>) -> Self {
        Self { t }
    }

    #[must_use]
    pub fn zero(tz: Tz) -> Self {
        Time::from_utc_timestamp(0, 0, tz)
    }

    pub fn from_date(d: impl Into<chrono::Date<Tz>>) -> Self {
        Self { t: d.into().and_hms(0, 0, 0) }
    }

    #[must_use]
    pub fn from_utc_timestamp(utc_secs: i64, utc_nanos: u32, tz: Tz) -> Self {
        tz.from_utc_datetime(&NaiveDateTime::from_timestamp(utc_secs, utc_nanos)).into()
    }

    #[must_use]
    pub fn from_utc_dec(utc_dec: Decimal, tz: Tz) -> Self {
        let utc_secs = utc_dec.trunc();
        let utc_nanos = ((utc_dec - utc_secs) * dec!(1000000000)).trunc();
        Self::from_utc_timestamp(utc_secs.to_i64().unwrap(), utc_nanos.to_u32().unwrap(), tz)
    }

    #[must_use]
    pub fn op(op: TOp, n: i64) -> TimeOp {
        TimeOp::new(op, n)
    }
}

/// String related functions
impl Time {
    /// From format e.g. 2020/01/30
    pub fn from_ymd_str(s: &str, tz: Tz) -> Result<Self> {
        Self::from_local_date_str(s, "%Y/%m/%d", tz)
    }

    pub fn from_local_date_str(s: &str, fmt: &str, tz: Tz) -> Result<Self> {
        let d = tz.from_local_date(&NaiveDate::parse_from_str(s, fmt)?);
        let d = d.single().ok_or_else(|| eyre!("no single representation for {}", s))?;
        Ok(Self::from_date(d))
    }

    pub fn from_local_iso(s: &str, tz: Tz) -> Result<Self> {
        let t = DateTime::parse_from_rfc3339(s)?;
        Ok(Self::from_utc_timestamp(t.timestamp(), t.timestamp_subsec_nanos(), tz))
    }

    /// Take the naive datetime assumed to be in the given timezone, and
    /// attach the timezone to it.
    pub fn from_local_datetime(d: NaiveDateTime, tz: Tz) -> Result<Self> {
        let dt = tz.from_local_datetime(&d);
        let dt = dt.single().ok_or_else(|| eyre!("no single representation for {}", d))?;
        Ok(Self::new(dt))
    }

    pub fn from_local_datetime_str(s: &str, fmt: &str, tz: Tz) -> Result<Self> {
        Self::from_local_datetime(NaiveDateTime::parse_from_str(s, fmt)?, tz)
    }

    #[must_use]
    pub fn to_iso(self) -> String {
        self.t.to_rfc3339()
    }
}

/// Accessors and conversions
impl Time {
    #[must_use]
    pub fn utc_f64(self) -> f64 {
        self.utc_dec().to_f64().unwrap()
    }

    #[must_use]
    pub fn utc_dec(&self) -> Decimal {
        let wtz = self.t.with_timezone(&UTC);
        let secs = wtz.timestamp();
        let nanos = wtz.timestamp_subsec_nanos() as i64;
        Decimal::new(secs, 0) + Decimal::new(nanos, 9)
    }

    #[must_use]
    pub fn tz(&self) -> Tz {
        self.t.timezone()
    }

    #[must_use]
    pub fn with_tz(&self, tz: Tz) -> Self {
        self.t.with_timezone(&tz).into()
    }

    #[must_use]
    pub fn ymd(&self) -> Self {
        Self::from_date(self.date())
    }

    #[must_use]
    pub fn date(&self) -> Date {
        self.t.date().into()
    }

    #[must_use]
    pub fn day(&self) -> u32 {
        self.t.day()
    }

    #[must_use]
    pub fn year(&self) -> i32 {
        self.t.year()
    }

    #[must_use]
    pub fn weekday(&self) -> Day {
        self.date().weekday()
    }

    #[must_use]
    pub fn month_name(&self) -> String {
        self.date().month_name()
    }

    #[must_use]
    pub fn month0(&self) -> u32 {
        self.t.month0()
    }

    #[must_use]
    pub fn month(&self) -> u32 {
        self.t.month()
    }
}

/// Time and date operations
impl Time {
    #[must_use]
    pub fn with_date(&self, d: impl Into<chrono::Date<Tz>>) -> Self {
        let d = d.into();
        let mut t = self.t.time();
        loop {
            let localdt = d.naive_local().and_time(t);
            let v = self.tz().from_local_datetime(&localdt);
            match v {
                LocalResult::None => {}
                LocalResult::Single(dt) => return Self::new(dt),
                LocalResult::Ambiguous(min, max) => {
                    // Preserve the offset - so the apparent time e.g. 1:30 AM
                    // always stays the same. This isn't perfect but whatever.
                    return Self::new(if min.offset() == d.offset() { min } else { max });
                }
            };
            // Add an hour until we get past the non-existent block of time.
            // This can happen when a daylight savings adjustment leaves a gap.
            t += chrono::Duration::hours(1);
        }
    }

    #[must_use]
    pub fn with_nanos(self, ns: u32) -> Self {
        self.t.with_nanosecond(ns).unwrap().into()
    }

    #[must_use]
    pub fn add_nanos(self, ns: i64) -> Self {
        (self.t + chrono::Duration::nanoseconds(ns)).into()
    }

    #[must_use]
    pub fn with_micros(self, us: u32) -> Self {
        self.t.with_nanosecond(us * 1000).unwrap().into()
    }

    #[must_use]
    pub fn add_micros(self, us: i64) -> Self {
        (self.t + chrono::Duration::microseconds(us)).into()
    }

    #[must_use]
    pub fn with_millis(self, ms: u32) -> Self {
        self.t.with_nanosecond(ms * 1000 * 1000).unwrap().into()
    }

    #[must_use]
    pub fn add_millis(self, ms: i64) -> Self {
        (self.t + chrono::Duration::milliseconds(ms)).into()
    }

    #[must_use]
    pub fn with_sec(self, s: u32) -> Self {
        self.t.with_second(s.clamp(0, 59)).unwrap().into()
    }

    #[must_use]
    pub fn add_secs(self, secs: i64) -> Self {
        (self.t + chrono::Duration::seconds(secs)).into()
    }

    #[must_use]
    pub fn with_min(self, m: u32) -> Self {
        self.t.with_minute(m.clamp(0, 59)).unwrap().into()
    }

    #[must_use]
    pub fn add_mins(self, mins: i64) -> Self {
        (self.t + chrono::Duration::minutes(mins)).into()
    }

    #[must_use]
    pub fn with_hour(self, h: u32) -> Self {
        self.t.with_hour(h.clamp(0, 23)).unwrap().into()
    }

    #[must_use]
    pub fn add_hours(self, h: i64) -> Self {
        (self.t + chrono::Duration::hours(h)).into()
    }

    #[must_use]
    pub fn with_day(self, d: u32) -> Self {
        self.with_date(self.date().with_day(d))
    }

    #[must_use]
    pub fn add_days(self, d: i32) -> Self {
        self.with_date(self.date().add_days(d))
    }

    #[must_use]
    pub fn with_month(self, m: u32) -> Self {
        self.with_date(self.date().with_month(m))
    }

    #[must_use]
    pub fn add_months(self, m: i32) -> Self {
        self.with_date(self.date().add_months(m))
    }

    #[must_use]
    pub fn with_year(self, y: i32) -> Self {
        self.with_date(self.date().with_year(y))
    }

    #[must_use]
    pub fn add_years(self, y: i32) -> Self {
        self.with_date(self.date().add_years(y))
    }
}

impl Default for Time {
    fn default() -> Self {
        Time::zero(UTC)
    }
}

impl From<DateTime<Tz>> for Time {
    fn from(v: DateTime<Tz>) -> Self {
        Self::new(v)
    }
}

impl From<Time> for DateTime<Tz> {
    fn from(v: Time) -> Self {
        v.t
    }
}

impl From<chrono::Date<Tz>> for Time {
    fn from(v: chrono::Date<Tz>) -> Self {
        Self::from_date(v)
    }
}

impl From<&chrono::Date<Tz>> for Time {
    fn from(v: &chrono::Date<Tz>) -> Self {
        Self::from_date(*v)
    }
}

impl From<Date> for Time {
    fn from(v: Date) -> Self {
        Self::from_date(v)
    }
}

impl From<&Date> for Time {
    fn from(v: &Date) -> Self {
        Self::from_date(*v)
    }
}

impl From<Time> for f64 {
    fn from(v: Time) -> Self {
        v.utc_f64()
    }
}

impl Sub<Time> for Time {
    type Output = Duration;

    fn sub(self, t: Time) -> Self::Output {
        (self.utc_dec() - t.utc_dec()) * SEC
    }
}

impl Sub<Duration> for Time {
    type Output = Time;

    fn sub(self, d: Duration) -> Self::Output {
        Self::from_utc_dec(self.utc_dec() - d.secs(), self.t.timezone())
    }
}

impl SubAssign<Duration> for Time {
    fn sub_assign(&mut self, d: Duration) {
        *self = *self - d;
    }
}

impl Add<Duration> for Time {
    type Output = Time;

    fn add(self, d: Duration) -> Self::Output {
        Self::from_utc_dec(self.utc_dec() + d.secs(), self.t.timezone())
    }
}

impl AddAssign<Duration> for Time {
    fn add_assign(&mut self, d: Duration) {
        *self = *self + d;
    }
}

impl<'a> Deserialize<'a> for Time {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        struct TimeVisitor;

        impl Visitor<'_> for TimeVisitor {
            type Value = Time;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("iso8601 string and timezone name")
            }

            fn visit_str<E>(self, v: &str) -> Result<Time, E>
            where
                E: de::Error,
            {
                let mut s = v.split_whitespace();
                let iso8601 = s
                    .next()
                    .ok_or_else(|| eyre!("missing iso8601 timestamp"))
                    .map_err(E::custom)?;
                let tz = s.next().ok_or_else(|| eyre!("missing timezone")).map_err(E::custom)?;
                let tz = Tz::from_str(tz).map_err(E::custom)?;
                let time = Time::from_local_iso(iso8601, tz).map_err(E::custom)?;
                Ok(time)
            }
        }

        d.deserialize_string(TimeVisitor)
    }
}

impl Serialize for Time {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&(self.to_iso() + " " + self.tz().name()))
    }
}

impl ToPrimitive for Time {
    fn to_i64(&self) -> Option<i64> {
        self.utc_dec().to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        self.utc_dec().to_u64()
    }

    fn to_f64(&self) -> Option<f64> {
        Some(Time::utc_f64(*self))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;
    use chrono_tz::Australia::Sydney;
    use chrono_tz::US::Eastern;

    use super::*;

    #[test]
    fn set_date_with_different_daylight_savings() {
        // On same day:
        let t = Time::new(Eastern.ymd(1994, 10, 27).and_hms(1, 44, 35));
        let d = Eastern.ymd(1994, 10, 27);
        assert_eq!(t, t.with_date(d));

        // On different days:
        let t = Time::new(Eastern.ymd(1994, 4, 30).and_hms(1, 29, 11));
        let d = Eastern.ymd(1994, 10, 30);
        assert_eq!(Time::new(UTC.ymd(1994, 10, 30).and_hms(5, 29, 11)), t.with_date(d));
    }

    #[test]
    fn nonexistent_set_date() {
        let t = Time::new(Eastern.ymd(2017, 3, 5).and_hms(2, 57, 12));
        let d = Eastern.ymd(2017, 3, 12);
        assert_eq!(Time::new(UTC.ymd(2017, 3, 12).and_hms(7, 57, 12)), t.with_date(d));
    }

    #[test]
    fn date_tz_conversion() -> Result<()> {
        let expected = Time::from_date(Sydney.ymd(2018, 1, 30));
        let d_str = "30 Jan 2018";
        assert_eq!(expected, Time::from_ymd_str("2018/1/30", Sydney)?);
        assert_eq!(expected, Time::from_local_date_str(d_str, "%d %b %Y", Sydney)?);

        Ok(())
    }

    #[test]
    fn datetime_tz_conversion() -> Result<()> {
        let expected = Time::new(Sydney.ymd(2018, 1, 30).and_hms(6, 4, 57));
        let dt_str = "30 Jan 2018 06:04:57";

        assert_eq!(
            expected,
            Time::from_local_datetime(
                NaiveDateTime::parse_from_str(dt_str, "%d %b %Y %H:%M:%S")?,
                Sydney
            )?
        );
        assert_eq!(expected, Time::from_local_datetime_str(dt_str, "%d %b %Y %H:%M:%S", Sydney)?);

        Ok(())
    }

    #[test]
    fn iso_tz_conversion() -> Result<()> {
        let expected = Time::new(Sydney.ymd(2018, 1, 30).and_hms(6, 4, 57));
        assert_eq!(expected, Time::from_local_iso("2018-01-30T06:04:57+11:00", Sydney)?);
        assert_eq!(expected, Time::from_local_iso(&expected.to_iso(), Sydney)?);
        Ok(())
    }

    #[test]
    fn tz_change() {
        let time = Time::new(Sydney.ymd(2018, 1, 30).and_hms(6, 4, 57));
        // This shouldn't change the underlying time, just the timezone it's in.
        assert_eq!(time.utc_dec(), time.with_tz(Eastern).utc_dec());
    }
}
