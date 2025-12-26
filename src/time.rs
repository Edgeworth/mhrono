use std::borrow::Borrow;
use std::fmt;
use std::str::FromStr;

use auto_ops::impl_op_ex;
use chrono::{DateTime, Datelike, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};
use chrono_tz::{Tz, UTC};
use derive_more::Display;
use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};

use crate::date::{Date, Day};
use crate::duration::Duration;
use crate::op::{TOp, TimeOp};
use crate::span::endpoint::{EndpointConversion, EndpointSide};
use crate::{Error, Result};

pub fn ymdhms<T: Borrow<Tz>>(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
    tz: T,
) -> Time {
    Time::new(tz.borrow().with_ymd_and_hms(year, month, day, hour, min, sec).unwrap())
}

// Chrono allows nanoseconds up to 2 seconds to represent leap second fractions.
const MAX_NANOSECOND: u32 = 1_999_999_999;

/// Note that hashses and comparisons are based on the underlying time, so two
/// times with different timezones can compare as the same.
#[must_use]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Display, Ord, PartialOrd)]
#[display("{t}")]
pub struct Time {
    t: DateTime<Tz>,
}

/// Creation
impl Time {
    pub const LOCAL_FMT: &'static str = "%Y-%m-%dT%H:%M:%S%.f";

    pub const fn new(t: DateTime<Tz>) -> Self {
        Self { t }
    }

    pub fn now_utc() -> Self {
        Self::new(UTC.from_utc_datetime(&Utc::now().naive_utc()))
    }

    pub fn zero(tz: Tz) -> Self {
        Time::from_utc_timestamp(0, 0, tz)
    }

    pub fn from_naive_date(d: NaiveDate, tz: Tz) -> Result<Self> {
        Date::new(d, tz).time()
    }

    pub fn from_utc_timestamp(utc_secs: i64, utc_nanos: u32, tz: Tz) -> Self {
        tz.from_utc_datetime(&DateTime::from_timestamp(utc_secs, utc_nanos).unwrap().naive_utc())
            .into()
    }

    pub fn from_utc_dec(utc_dec: Decimal, tz: Tz) -> Self {
        let utc_secs = utc_dec.floor();
        let utc_nanos = ((utc_dec - utc_secs) * dec!(1000000000)).trunc();
        Self::from_utc_timestamp(utc_secs.to_i64().unwrap(), utc_nanos.to_u32().unwrap(), tz)
    }

    pub fn from_utc_f64(utc_f64: f64, tz: Tz) -> Self {
        Self::from_utc_dec(utc_f64.try_into().unwrap(), tz)
    }

    pub fn op(op: TOp, n: i64) -> TimeOp {
        TimeOp::new(op, n)
    }

    pub fn apply(&self, op: TimeOp) -> Self {
        op.apply(*self)
    }
}

/// String related functions
impl Time {
    /// From format e.g. 2020/01/30
    pub fn from_ymd(s: &str, tz: Tz) -> Result<Self> {
        Self::from_local_date_fmt(s, "%Y-%m-%d", tz)
            .or_else(|_| Self::from_local_date_fmt(s, "%Y/%m/%d", tz))
    }

    pub fn from_ymd_fmt(s: &str, fmt: &str, tz: Tz) -> Result<Self> {
        Self::from_local_date_fmt(s, fmt, tz)
    }

    pub fn from_local_date_fmt(s: &str, fmt: &str, tz: Tz) -> Result<Self> {
        Self::from_naive_date(NaiveDate::parse_from_str(s, fmt)?, tz)
    }

    /// From a local time in ISO RFC3339 format.
    pub fn from_local_iso(s: &str, tz: Tz) -> Result<Self> {
        let t = DateTime::parse_from_rfc3339(s)?;
        Ok(Self::from_utc_timestamp(t.timestamp(), t.timestamp_subsec_nanos(), tz))
    }

    /// From a local time.
    pub fn from_local(s: &str, tz: Tz) -> Result<Self> {
        Self::from_local_datetime_fmt(s, Self::LOCAL_FMT, tz)
    }

    /// Take the naive datetime assumed to be in the given timezone, and
    /// attach the timezone to it.
    pub fn from_local_datetime(d: NaiveDateTime, tz: Tz) -> Result<Self> {
        let dt = tz.from_local_datetime(&d);
        let dt = dt.single().ok_or_else(|| Error::InvalidLocalDateTime(d.to_string()))?;
        Ok(Self::new(dt))
    }

    pub fn from_local_datetime_fmt(s: &str, fmt: &str, tz: Tz) -> Result<Self> {
        Self::from_local_datetime(NaiveDateTime::parse_from_str(s, fmt)?, tz)
    }

    #[must_use]
    pub fn to_iso(&self) -> String {
        self.t.to_rfc3339()
    }

    #[must_use]
    pub fn to_local(&self) -> String {
        self.t.format(Self::LOCAL_FMT).to_string()
    }

    #[must_use]
    pub fn format(&self, f: &str) -> String {
        self.t.format(f).to_string()
    }
}

/// Accessors and conversions
impl Time {
    #[must_use]
    pub fn utc_f64(self) -> f64 {
        self.utc_dec().to_f64().unwrap()
    }

    #[must_use]
    pub fn utc_timestamp(&self) -> (i64, u32) {
        let wtz = self.t.with_timezone(&UTC);
        (wtz.timestamp(), wtz.timestamp_subsec_nanos())
    }

    #[must_use]
    pub fn utc_dec(&self) -> Decimal {
        let (secs, nanos) = self.utc_timestamp();
        Decimal::new(secs, 0) + Decimal::new(nanos as i64, 9)
    }

    #[must_use]
    pub fn tz(&self) -> Tz {
        self.t.timezone()
    }

    pub fn with_tz(&self, tz: Tz) -> Self {
        self.t.with_timezone(&tz).into()
    }

    pub fn ymd(&self) -> Result<Self> {
        self.date().time()
    }

    pub fn date(&self) -> Date {
        Date::new(self.t.date_naive(), self.tz())
    }

    #[must_use]
    pub fn nanosecond(&self) -> u32 {
        self.t.nanosecond()
    }

    #[must_use]
    pub fn microsecond(&self) -> u32 {
        self.t.nanosecond() / 1000
    }

    #[must_use]
    pub fn second(&self) -> u32 {
        self.t.second()
    }

    #[must_use]
    pub fn minute(&self) -> u32 {
        self.t.minute()
    }

    #[must_use]
    pub fn hour(&self) -> u32 {
        self.t.hour()
    }

    #[must_use]
    pub fn day(&self) -> u32 {
        self.t.day()
    }

    #[must_use]
    pub fn year(&self) -> i32 {
        self.t.year()
    }

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
    /// Returns a time with the given date. If the time of day doesn't
    /// exist on that date (because of daylight savings etc), returns
    /// the next time that does exist.
    pub fn with_date(&self, d: impl Into<NaiveDate>) -> Self {
        let d = d.into();
        let mut localdt = d.and_time(self.t.time());
        loop {
            let v = self.tz().from_local_datetime(&localdt);
            match v {
                LocalResult::None => {}
                LocalResult::Single(dt) => return Self::new(dt),
                LocalResult::Ambiguous(min, max) => {
                    // Preserve the offset - so the apparent time e.g. 1:30 AM
                    // always stays the same. This isn't perfect but whatever.
                    return Self::new(if min.offset() == self.t.offset() { min } else { max });
                }
            }
            // Add a minute until we get past the non-existent block of time.
            // This can happen when a daylight savings adjustment leaves a gap, or when an entire
            // local date is skipped (e.g. dateline changes).
            localdt += chrono::Duration::minutes(1);
            localdt = localdt.with_second(0).unwrap().with_nanosecond(0).unwrap();
        }
    }

    pub fn with_nanos(&self, ns: u32) -> Self {
        self.t.with_nanosecond(ns.min(MAX_NANOSECOND)).unwrap().into()
    }

    pub fn add_nanos(&self, ns: i64) -> Self {
        (self.t + chrono::Duration::nanoseconds(ns)).into()
    }

    pub fn with_micros(&self, us: u32) -> Self {
        let ns = u64::from(us) * 1000;
        self.with_nanos(ns.min(u64::from(MAX_NANOSECOND)) as u32)
    }

    pub fn add_micros(&self, us: i64) -> Self {
        (self.t + chrono::Duration::microseconds(us)).into()
    }

    pub fn with_millis(&self, ms: u32) -> Self {
        let ns = u64::from(ms) * 1000 * 1000;
        self.with_nanos(ns.min(u64::from(MAX_NANOSECOND)) as u32)
    }

    pub fn add_millis(&self, ms: i64) -> Self {
        (self.t + chrono::Duration::try_milliseconds(ms).unwrap()).into()
    }

    pub fn with_sec(&self, s: u32) -> Self {
        self.t.with_second(s.clamp(0, 59)).unwrap().into()
    }

    pub fn add_secs(&self, secs: i64) -> Self {
        (self.t + chrono::Duration::try_seconds(secs).unwrap()).into()
    }

    pub fn with_min(&self, m: u32) -> Self {
        self.t.with_minute(m.clamp(0, 59)).unwrap().into()
    }

    pub fn add_mins(&self, mins: i64) -> Self {
        (self.t + chrono::Duration::try_minutes(mins).unwrap()).into()
    }

    pub fn with_hour(&self, h: u32) -> Self {
        self.t.with_hour(h.clamp(0, 23)).unwrap().into()
    }

    pub fn add_hours(&self, h: i64) -> Self {
        (self.t + chrono::Duration::try_hours(h).unwrap()).into()
    }

    pub fn with_day(&self, d: u32) -> Self {
        self.with_date(self.date().with_day(d))
    }

    pub fn add_days(&self, d: i32) -> Self {
        self.with_date(self.date().add_days(d))
    }

    pub fn with_month(&self, m: u32) -> Self {
        self.with_date(self.date().with_month(m))
    }

    pub fn add_months(&self, m: i32) -> Self {
        self.with_date(self.date().add_months(m))
    }

    pub fn with_year(&self, y: i32) -> Self {
        self.with_date(self.date().with_year(y))
    }

    pub fn add_years(&self, y: i32) -> Self {
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

impl From<Time> for f64 {
    fn from(v: Time) -> Self {
        v.utc_f64()
    }
}

impl_op_ex!(-|a: &Time, b: &Time| -> Duration { (a.utc_dec() - b.utc_dec()) * Duration::SEC });

impl_op_ex!(-|a: &Time, b: &Duration| -> Time {
    Time::from_utc_dec(a.utc_dec() - b.secs(), a.t.timezone())
});
impl_op_ex!(-= |a: &mut Time, b: &Duration| { *a = *a - b });

impl_op_ex!(+ |a: &Time, b: &Duration| -> Time { Time::from_utc_dec(a.utc_dec() + b.secs(), a.t.timezone()) });
impl_op_ex!(+= |a: &mut Time, b: &Duration| { *a = *a + b });

impl<'a> Deserialize<'a> for Time {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        struct TimeVisitor;

        impl Visitor<'_> for TimeVisitor {
            type Value = Time;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("rfc3339 string and timezone name")
            }

            fn visit_str<E>(self, v: &str) -> Result<Time, E>
            where
                E: de::Error,
            {
                let mut s = v.split_whitespace();
                let iso = s.next().ok_or_else(|| E::custom("missing timestamp"))?;
                let tz = s.next().ok_or_else(|| E::custom("missing timezone"))?;
                let tz = Tz::from_str(tz).map_err(E::custom)?;
                let time = Time::from_local_iso(iso, tz).map_err(E::custom)?;
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

impl EndpointConversion for Time {
    fn to_open(&self, side: EndpointSide) -> Option<Self> {
        let ulp = chrono::Duration::nanoseconds(1);
        let d = match side {
            EndpointSide::Left => self.t.checked_sub_signed(ulp),
            EndpointSide::Right => self.t.checked_add_signed(ulp),
        };
        d.map(Self::new)
    }

    fn to_closed(&self, side: EndpointSide) -> Option<Self> {
        let ulp = chrono::Duration::nanoseconds(1);
        let d = match side {
            EndpointSide::Left => self.t.checked_add_signed(ulp),
            EndpointSide::Right => self.t.checked_sub_signed(ulp),
        };
        d.map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    use chrono_tz::Australia::Sydney;
    use chrono_tz::US::Eastern;
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;
    use crate::date::ymd;

    #[test]
    fn set_date_with_different_daylight_savings() {
        // On same day:
        let t = ymdhms(1994, 10, 27, 1, 44, 35, Eastern);
        let d = ymd(1994, 10, 27, Eastern);
        assert_eq!(t, t.with_date(d));

        // On different days:
        let t = ymdhms(1994, 4, 30, 1, 29, 11, Eastern);
        let d = ymd(1994, 10, 30, Eastern);
        assert_eq!(ymdhms(1994, 10, 30, 5, 29, 11, UTC), t.with_date(d));
    }

    #[test]
    fn nonexistent_set_date() {
        let t = ymdhms(2017, 3, 5, 2, 57, 12, Eastern);
        let d = ymd(2017, 3, 12, Eastern);
        assert_eq!(ymdhms(2017, 3, 12, 7, 0, 0, UTC), t.with_date(d));
    }

    #[test]
    fn date_tz_conversion() -> Result<()> {
        let expected = ymd(2018, 1, 30, Sydney).time()?;
        let d_str = "30 Jan 2018";
        assert_eq!(expected, Time::from_ymd("2018/1/30", Sydney)?);
        assert_eq!(expected, Time::from_local_date_fmt(d_str, "%d %b %Y", Sydney)?);

        Ok(())
    }

    #[test]
    fn datetime_tz_conversion() -> Result<()> {
        let expected = ymdhms(2018, 1, 30, 6, 4, 57, Sydney);
        let dt_str = "30 Jan 2018 06:04:57";

        assert_eq!(
            expected,
            Time::from_local_datetime(
                NaiveDateTime::parse_from_str(dt_str, "%d %b %Y %H:%M:%S")?,
                Sydney
            )?
        );
        assert_eq!(expected, Time::from_local_datetime_fmt(dt_str, "%d %b %Y %H:%M:%S", Sydney)?);

        Ok(())
    }

    #[test]
    fn tz_conversion() -> Result<()> {
        let dt = ymdhms(2018, 1, 30, 6, 4, 57, Sydney);
        assert_eq!(dt, Time::from_local_iso("2018-01-30T06:04:57+11:00", Sydney)?);
        assert_eq!(dt, Time::from_local("2018-01-30T06:04:57", Sydney)?);
        assert_eq!("2018-01-30T06:04:57+11:00", dt.to_iso());
        assert_eq!("2018-01-30T06:04:57", dt.to_local());
        assert_eq!(dt, Time::from_local_iso(&dt.to_iso(), Sydney)?);
        assert_eq!(dt, Time::from_local(&dt.to_local(), Sydney)?);
        Ok(())
    }

    #[test]
    fn serialization() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dt = ymdhms(2018, 1, 30, 6, 4, 57, Sydney);
        let se = serde_json::to_string(&dt)?;
        assert_eq!(se, "\"2018-01-30T06:04:57+11:00 Australia/Sydney\"");
        let de: Time = serde_json::from_str(&se)?;
        assert_eq!(de, dt);
        Ok(())
    }

    #[test]
    fn serialization_round_trips_ambiguous_local_time()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let a = Time::from_local_iso("2017-11-05T01:30:00-04:00", Eastern)?;
        let b = Time::from_local_iso("2017-11-05T01:30:00-05:00", Eastern)?;
        assert_ne!(a.utc_dec(), b.utc_dec());
        assert_eq!(a.to_local(), b.to_local());

        let se_a = serde_json::to_string(&a)?;
        let se_b = serde_json::to_string(&b)?;
        assert_ne!(se_a, se_b);

        let de_a: Time = serde_json::from_str(&se_a)?;
        let de_b: Time = serde_json::from_str(&se_b)?;
        assert_eq!(de_a, a);
        assert_eq!(de_b, b);

        Ok(())
    }

    #[test]
    fn tz_change() {
        let time = ymdhms(2018, 1, 30, 6, 4, 57, Sydney);
        // This shouldn't change the underlying time, just the timezone it's in.
        assert_eq!(time.utc_dec(), time.with_tz(Eastern).utc_dec());
    }

    #[test]
    fn time_with_timezone_conversion() -> Result<()> {
        let t = ymdhms(2020, 3, 15, 12, 0, 0, Eastern);

        let d = t.date();
        assert_eq!(d.tz(), Eastern);
        assert_eq!(d.year(), 2020);
        assert_eq!(d.month(), 3);
        assert_eq!(d.day(), 15);

        let t2 = d.and_hms(12, 0, 0)?;
        assert_eq!(t2, t);

        Ok(())
    }

    #[test]
    fn with_nanos_allows_leap_second_fraction() {
        let t = Time::zero(UTC);
        assert_eq!(t.with_nanos(1_000_000_000).utc_timestamp(), (0, 1_000_000_000));
    }

    #[test]
    fn with_micros_allows_leap_second_fraction() {
        let t = Time::zero(UTC);
        assert_eq!(t.with_micros(1_000_000).utc_timestamp(), (0, 1_000_000_000));
    }

    #[test]
    fn with_millis_allows_leap_second_fraction() {
        let t = Time::zero(UTC);
        assert_eq!(t.with_millis(1000).utc_timestamp(), (0, 1_000_000_000));
    }
}
