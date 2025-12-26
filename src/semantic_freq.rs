use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use rust_decimal::Decimal;
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};
use strum::{Display as StrumDisplay, EnumString};

use crate::duration::Duration;
use crate::op::{TOp, TimeOp};
use crate::time::Time;
use crate::{Error, Result};

/// Frequency. For now this should be representable as at most two lowercase
/// English letters and be case insensitive.
#[must_use]
#[derive(
    Debug,
    Eq,
    PartialEq,
    Hash,
    Copy,
    Clone,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    StrumDisplay,
    EnumString,
)]
#[strum(ascii_case_insensitive)]
pub enum SemanticFreq {
    #[strum(serialize = "ms")]
    Millisecond,
    #[strum(serialize = "s")]
    Second,
    #[strum(serialize = "m")]
    Minute,
    #[strum(serialize = "h")]
    Hour,
    #[strum(serialize = "d")]
    Day,
    #[strum(serialize = "w")]
    Week,
    #[strum(serialize = "mo")]
    Month,
    #[strum(serialize = "y")]
    Year,
}

impl SemanticFreq {
    #[must_use]
    pub const fn approx_millis(&self) -> i64 {
        match *self {
            SemanticFreq::Millisecond => 1,
            SemanticFreq::Second => 1000,
            SemanticFreq::Minute => 60 * 1000,
            SemanticFreq::Hour => 60 * 60 * 1000,
            SemanticFreq::Day => 24 * 60 * 60 * 1000,
            SemanticFreq::Week => 7 * 24 * 60 * 60 * 1000,
            SemanticFreq::Month => 30 * 24 * 60 * 60 * 1000,
            SemanticFreq::Year => 365 * 24 * 60 * 60 * 1000,
        }
    }

    pub const fn to_top(&self) -> TOp {
        match *self {
            SemanticFreq::Millisecond => TOp::AddMillis,
            SemanticFreq::Second => TOp::AddSecs,
            SemanticFreq::Minute => TOp::AddMins,
            SemanticFreq::Hour => TOp::AddHours,
            SemanticFreq::Day | SemanticFreq::Week => TOp::AddDays,
            SemanticFreq::Month => TOp::AddMonths,
            SemanticFreq::Year => TOp::AddYears,
        }
    }
}

#[must_use]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Freq {
    /// Use i16 since this type is mainly intended for simple semantic
    /// frequencies and want to keep the storage size low.
    count: i16,
    base: SemanticFreq,
}

impl fmt::Display for Freq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.count, self.base)
    }
}

impl Ord for Freq {
    fn cmp(&self, o: &Self) -> Ordering {
        let a = (self.base, self.count);
        let b = (o.base, o.count);
        b.cmp(&a)
    }
}

impl PartialOrd for Freq {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Freq {
    pub const MILLI: Freq = Freq::millis(1);
    pub const SEC: Freq = Freq::secs(1);
    pub const MIN: Freq = Freq::mins(1);
    pub const HOURLY: Freq = Freq::hours(1);
    pub const DAILY: Freq = Freq::days(1);
    pub const WEEKLY: Freq = Freq::weeks(1);
    pub const MONTHLY: Freq = Freq::months(1);
    pub const YEARLY: Freq = Freq::years(1);

    pub const fn new(count: i16, base: SemanticFreq) -> Self {
        Self { count, base }
    }

    #[must_use]
    pub const fn count(&self) -> i16 {
        self.count
    }

    pub const fn base(&self) -> SemanticFreq {
        self.base
    }

    pub const fn millis(count: i16) -> Self {
        Self::new(count, SemanticFreq::Millisecond)
    }

    pub const fn secs(count: i16) -> Self {
        Self::new(count, SemanticFreq::Second)
    }

    pub const fn mins(count: i16) -> Self {
        Self::new(count, SemanticFreq::Minute)
    }

    pub const fn hours(count: i16) -> Self {
        Self::new(count, SemanticFreq::Hour)
    }

    pub const fn days(count: i16) -> Self {
        Self::new(count, SemanticFreq::Day)
    }

    pub const fn weeks(count: i16) -> Self {
        Self::new(count, SemanticFreq::Week)
    }

    pub const fn months(count: i16) -> Self {
        Self::new(count, SemanticFreq::Month)
    }

    pub const fn years(count: i16) -> Self {
        Self::new(count, SemanticFreq::Year)
    }

    pub fn next(&self, t: &Time) -> Time {
        match self.base {
            SemanticFreq::Millisecond => t.add_millis(self.count as i64),
            SemanticFreq::Second => t.add_secs(self.count as i64),
            SemanticFreq::Minute => t.add_mins(self.count as i64),
            SemanticFreq::Hour => t.add_hours(self.count as i64),
            SemanticFreq::Day => t.add_days(self.count as i32),
            SemanticFreq::Week => t.add_days(7 * (self.count as i32)),
            SemanticFreq::Month => t.add_months(self.count as i32),
            SemanticFreq::Year => t.add_years(self.count as i32),
        }
    }

    pub fn prev(&self, t: &Time) -> Time {
        match self.base {
            SemanticFreq::Millisecond => t.add_millis(-self.count as i64),
            SemanticFreq::Second => t.add_secs(-self.count as i64),
            SemanticFreq::Minute => t.add_mins(-self.count as i64),
            SemanticFreq::Hour => t.add_hours(-self.count as i64),
            SemanticFreq::Day => t.add_days(-self.count as i32),
            SemanticFreq::Week => t.add_days(-7 * (self.count as i32)),
            SemanticFreq::Month => t.add_months(-self.count as i32),
            SemanticFreq::Year => t.add_years(-self.count as i32),
        }
    }

    pub const fn to_timeop(&self) -> TimeOp {
        let mul = if matches!(self.base, SemanticFreq::Week) { 7 } else { 1 };
        TimeOp::new(self.base.to_top(), mul * (self.count as i64))
    }

    #[must_use]
    pub const fn approx_cycle_millis(&self) -> i64 {
        self.count as i64 * self.base.approx_millis()
    }

    pub fn approx_cycle_duration(&self) -> Duration {
        Duration::new(Decimal::new(self.approx_cycle_millis(), 3))
    }
}

macro_rules! semantic_freq_ops {
    ($t:ty) => {
        impl_op_ex_commutative!(* |a: &Freq, b: &$t| -> Freq {
            let count = a.count.checked_mul((*b).try_into().unwrap()).unwrap();
            Freq { count, base: a.base
        } });
        impl_op_ex!(*= |a: &mut Freq, b: &$t| {
            a.count = a.count.checked_mul((*b).try_into().unwrap()).unwrap();
        });
   };
}

semantic_freq_ops!(i16);
semantic_freq_ops!(u16);
semantic_freq_ops!(i32);
semantic_freq_ops!(u32);
semantic_freq_ops!(i64);
semantic_freq_ops!(u64);
semantic_freq_ops!(usize);

impl<'a> Deserialize<'a> for Freq {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        struct FreqVisitor;

        impl Visitor<'_> for FreqVisitor {
            type Value = Freq;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("frequency")
            }

            fn visit_str<E>(self, v: &str) -> Result<Freq, E>
            where
                E: de::Error,
            {
                v.parse().map_err(E::custom)
            }
        }

        d.deserialize_string(FreqVisitor)
    }
}

impl Serialize for Freq {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl FromStr for Freq {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Error::FrequencyParse("empty string".to_string()));
        }

        // Assumes that the string is ascii.
        let (num_part, enum_part) = if s.len() >= 2 && !s.as_bytes()[s.len() - 2].is_ascii_digit() {
            s.split_at(s.len() - 2)
        } else {
            s.split_at(s.len() - 1)
        };

        Ok(Freq::new(num_part.parse()?, enum_part.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use chrono_tz::US::Eastern;
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;
    use crate::time::ymdhms;

    #[test]
    fn serialization() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let freq = Freq::MILLI;
        let se = serde_json::to_string(&freq)?;
        assert_eq!(se, "\"1ms\"");
        let de: Freq = serde_json::from_str(&se)?;
        assert_eq!(de, freq);

        let freq = Freq::SEC;
        let se = serde_json::to_string(&freq)?;
        assert_eq!(se, "\"1s\"");
        let de: Freq = serde_json::from_str(&se)?;
        assert_eq!(de, freq);

        let freq = Freq::MIN;
        let se = serde_json::to_string(&freq)?;
        assert_eq!(se, "\"1m\"");
        let de: Freq = serde_json::from_str(&se)?;
        assert_eq!(de, freq);

        let freq = Freq::HOURLY;
        let se = serde_json::to_string(&freq)?;
        assert_eq!(se, "\"1h\"");
        let de: Freq = serde_json::from_str(&se)?;
        assert_eq!(de, freq);

        let freq = Freq::DAILY;
        let se = serde_json::to_string(&freq)?;
        assert_eq!(se, "\"1d\"");
        let de: Freq = serde_json::from_str(&se)?;
        assert_eq!(de, freq);

        let freq = Freq::WEEKLY;
        let se = serde_json::to_string(&freq)?;
        assert_eq!(se, "\"1w\"");
        let de: Freq = serde_json::from_str(&se)?;
        assert_eq!(de, freq);

        let freq = Freq::MONTHLY;
        let se = serde_json::to_string(&freq)?;
        assert_eq!(se, "\"1mo\"");
        let de: Freq = serde_json::from_str(&se)?;
        assert_eq!(de, freq);

        let freq = Freq::YEARLY;
        let se = serde_json::to_string(&freq)?;
        assert_eq!(se, "\"1y\"");
        let de: Freq = serde_json::from_str(&se)?;
        assert_eq!(de, freq);

        let freq = Freq::days(4);
        let se = serde_json::to_string(&freq)?;
        assert_eq!(se, "\"4d\"");
        let de: Freq = serde_json::from_str(&se)?;
        assert_eq!(de, freq);

        Ok(())
    }

    #[test]
    fn deserialization_invalid_input() {
        let data = "1zz"; // Invalid frequency
        let result: Result<Freq, _> = serde_json::from_str(data);
        assert!(result.is_err());

        let data = "zz"; // Invalid format
        let result: Result<Freq, _> = serde_json::from_str(data);
        assert!(result.is_err());
    }

    #[test]
    fn deserialization_invalid_input_as_json_string() {
        let data = "\"1zz\""; // Invalid frequency
        let result: Result<Freq, _> = serde_json::from_str(data);
        assert!(result.is_err());

        let data = "\"zz\""; // Invalid format
        let result: Result<Freq, _> = serde_json::from_str(data);
        assert!(result.is_err());
    }

    #[test]
    fn freq_parsing_is_case_insensitive() -> std::result::Result<(), Box<dyn std::error::Error>> {
        assert_eq!("1D".parse::<Freq>()?, Freq::DAILY);
        let result: Freq = serde_json::from_str("\"1D\"")?;
        assert_eq!(result, Freq::DAILY);
        Ok(())
    }

    #[test]
    fn freq_eq() {
        assert_eq!(Freq::HOURLY, Freq::HOURLY);
        assert_ne!(Freq::HOURLY, Freq::WEEKLY);
    }

    #[test]
    fn freq_ord() {
        assert!(Freq::HOURLY > Freq::DAILY);
        assert!(Freq::SEC > Freq::DAILY);
        assert!(Freq::MONTHLY < Freq::DAILY);
        assert!(Freq::weeks(2) < Freq::WEEKLY);
    }

    #[test]
    fn semantic_freq_ops() {
        let mut freq = Freq::new(1, SemanticFreq::Millisecond);
        freq *= 2;
        assert_eq!(freq, Freq::new(2, SemanticFreq::Millisecond));

        let freq = Freq::new(2, SemanticFreq::Millisecond) * 3;
        assert_eq!(freq, Freq::new(6, SemanticFreq::Millisecond));
    }

    #[test]
    fn next() {
        let t = ymdhms(2017, 3, 5, 2, 57, 12, Eastern);

        assert_eq!(Freq::SEC.next(&t), ymdhms(2017, 3, 5, 2, 57, 13, Eastern));
        assert_eq!(Freq::MIN.next(&t), ymdhms(2017, 3, 5, 2, 58, 12, Eastern));
        assert_eq!(Freq::HOURLY.next(&t), ymdhms(2017, 3, 5, 3, 57, 12, Eastern));
        assert_eq!(Freq::DAILY.next(&t), ymdhms(2017, 3, 6, 2, 57, 12, Eastern));
        // This time non-existent so skip forward to next possible time.
        assert_eq!(Freq::WEEKLY.next(&t), ymdhms(2017, 3, 12, 3, 0, 0, Eastern));
        assert_eq!(Freq::MONTHLY.next(&t), ymdhms(2017, 4, 5, 2, 57, 12, Eastern));
        assert_eq!(Freq::YEARLY.next(&t), ymdhms(2018, 3, 5, 2, 57, 12, Eastern));
    }
    #[test]
    fn prev() {
        let t = ymdhms(2017, 3, 5, 2, 57, 12, Eastern);

        assert_eq!(Freq::SEC.prev(&t), ymdhms(2017, 3, 5, 2, 57, 11, Eastern));
        assert_eq!(Freq::MIN.prev(&t), ymdhms(2017, 3, 5, 2, 56, 12, Eastern));
        assert_eq!(Freq::HOURLY.prev(&t), ymdhms(2017, 3, 5, 1, 57, 12, Eastern));
        assert_eq!(Freq::DAILY.prev(&t), ymdhms(2017, 3, 4, 2, 57, 12, Eastern));
        assert_eq!(Freq::WEEKLY.prev(&t), ymdhms(2017, 2, 26, 2, 57, 12, Eastern));
        assert_eq!(Freq::MONTHLY.prev(&t), ymdhms(2017, 2, 5, 2, 57, 12, Eastern));
        assert_eq!(Freq::YEARLY.prev(&t), ymdhms(2016, 3, 5, 2, 57, 12, Eastern));
    }

    #[test]
    fn approx_cycle_millis() {
        assert_eq!(Freq::millis(2).approx_cycle_millis(), 2);
        assert_eq!(Freq::secs(2).approx_cycle_millis(), 2 * 1000);
        assert_eq!(Freq::mins(2).approx_cycle_millis(), 2 * 60 * 1000);
        assert_eq!(Freq::hours(2).approx_cycle_millis(), 2 * 60 * 60 * 1000);
        assert_eq!(Freq::days(2).approx_cycle_millis(), 2 * 24 * 60 * 60 * 1000);
        assert_eq!(Freq::weeks(2).approx_cycle_millis(), 2 * 7 * 24 * 60 * 60 * 1000);
        assert_eq!(Freq::months(2).approx_cycle_millis(), 2 * 30 * 24 * 60 * 60 * 1000);
        assert_eq!(Freq::years(2).approx_cycle_millis(), 2 * 365 * 24 * 60 * 60 * 1000);
    }

    #[test]
    fn approx_cycle_duration() {
        assert_eq!(Freq::millis(2).approx_cycle_duration(), 2 * Duration::MSEC);
        assert_eq!(Freq::secs(2).approx_cycle_duration(), 2 * Duration::SEC);
        assert_eq!(Freq::mins(2).approx_cycle_duration(), 2 * Duration::MIN);
        assert_eq!(Freq::hours(2).approx_cycle_duration(), 2 * Duration::HOUR);
        assert_eq!(Freq::days(2).approx_cycle_duration(), 2 * Duration::DAY);
        assert_eq!(Freq::weeks(2).approx_cycle_duration(), 2 * Duration::WEEK);
        assert_eq!(Freq::months(2).approx_cycle_duration(), 2 * 30 * Duration::DAY);
        assert_eq!(Freq::years(2).approx_cycle_duration(), 2 * 365 * Duration::DAY);
    }
}
