use std::fmt;
use std::fmt::Write;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::str::FromStr;
use std::sync::LazyLock;

use derive_more::Display;
use eyre::{eyre, Result};
use num_traits::ToPrimitive;
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformFloat, UniformSampler};
use rand::prelude::*;
use regex::Regex;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::de::{self, Visitor};
use serde::{ser, Deserialize, Serialize};

use crate::span::endpoint::EndpointConversion;

pub const BASES: &[(&str, Duration)] = &[
    ("w", WEEK),
    ("d", DAY),
    ("h", HOUR),
    ("m", MIN),
    ("s", SEC),
    ("ms", MSEC),
    ("us", USEC),
    ("ns", NSEC),
    ("ps", PSEC),
    ("fs", FSEC),
    ("as", ASEC),
];

#[must_use]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Display, Ord, PartialOrd)]
#[display(fmt = "{}", "self.human().unwrap_or_else(|_| self.secs.to_string())")]
pub struct Duration {
    secs: Decimal,
}

impl Duration {
    pub const fn new(secs: Decimal) -> Self {
        Self { secs }
    }

    pub const fn zero() -> Self {
        Self { secs: dec!(0) }
    }

    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.secs.is_zero()
    }

    #[must_use]
    pub fn secs_f64(self) -> f64 {
        self.secs.to_f64().unwrap()
    }

    #[must_use]
    pub const fn secs(&self) -> Decimal {
        self.secs
    }

    #[must_use]
    pub fn to_chrono(&self) -> std::time::Duration {
        let secs = self.secs.trunc();
        let nanos = ((self.secs - secs) * dec!(1000000000)).trunc();
        std::time::Duration::new(secs.to_u64().unwrap(), nanos.to_u64().unwrap() as u32)
    }

    pub fn human(&self) -> Result<String> {
        self.human_bases(BASES)
    }

    pub fn human_bases(&self, bases: &[(&str, Duration)]) -> Result<String> {
        let mut rem = *self;
        let mut human = String::new();
        for &(s, dur) in bases {
            let div = (rem / dur).trunc();
            rem -= dur * div;
            if !div.is_zero() {
                let _ = write!(human, "{div}{s}");
            }
        }
        // Some sub-attosecond duration...
        if rem.is_zero() {
            Ok(human)
        } else {
            Err(eyre!("remainder is not zero"))
        }
    }

    pub fn from_human(s: &str) -> Result<Duration> {
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)([a-z]+)").unwrap());
        let mut dur = Duration::zero();
        for caps in RE.captures_iter(s) {
            let caps: Vec<_> = caps.iter().collect();
            let count =
                caps.get(1).unwrap().ok_or_else(|| eyre!("invalid"))?.as_str().parse::<i64>()?;
            let ident = caps.get(2).unwrap().ok_or_else(|| eyre!("invalid"))?.as_str();
            let base =
                BASES.iter().find(|v| v.0 == ident).ok_or_else(|| eyre!("unknown duration"))?;
            dur += base.1 * count;
        }

        Ok(dur)
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::zero()
    }
}

impl Add<Duration> for Duration {
    type Output = Duration;

    fn add(self, d: Duration) -> Self::Output {
        Duration::new(self.secs + d.secs)
    }
}

impl AddAssign<Duration> for Duration {
    fn add_assign(&mut self, d: Duration) {
        self.secs += d.secs;
    }
}

impl Sub<Duration> for Duration {
    type Output = Duration;

    fn sub(self, d: Duration) -> Self::Output {
        Duration::new(self.secs - d.secs)
    }
}

impl SubAssign<Duration> for Duration {
    fn sub_assign(&mut self, d: Duration) {
        self.secs -= d.secs;
    }
}

impl Div<Duration> for Duration {
    type Output = Decimal;

    fn div(self, o: Duration) -> Self::Output {
        self.secs / o.secs
    }
}

macro_rules! duration_ops {
    ($t:ty) => {
        impl MulAssign<$t> for Duration {
            fn mul_assign(&mut self, rhs: $t) {
                self.secs *= Decimal::try_from(rhs).unwrap();
            }
        }

        impl Mul<$t> for Duration {
            type Output = Duration;

            fn mul(self, rhs: $t) -> Self::Output {
                Self::new(self.secs * Decimal::try_from(rhs).unwrap())
            }
        }

        impl Mul<Duration> for $t {
            type Output = Duration;

            fn mul(self, rhs: Duration) -> Self::Output {
                Duration::new(Decimal::try_from(self).unwrap() * rhs.secs)
            }
        }

        impl DivAssign<$t> for Duration {
            fn div_assign(&mut self, rhs: $t) {
                self.secs /= Decimal::try_from(rhs).unwrap();
            }
        }

        impl Div<$t> for Duration {
            type Output = Duration;

            fn div(self, rhs: $t) -> Self::Output {
                Self::new(self.secs / Decimal::try_from(rhs).unwrap())
            }
        }
    };
}

duration_ops!(i64);
duration_ops!(Decimal);

impl ToPrimitive for Duration {
    fn to_i64(&self) -> Option<i64> {
        self.secs().to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        self.secs().to_u64()
    }

    fn to_f64(&self) -> Option<f64> {
        Some(self.secs_f64())
    }
}

#[must_use]
pub struct UniformDuration(UniformFloat<f64>);

impl UniformSampler for UniformDuration {
    type X = Duration;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformDuration(UniformFloat::<f64>::new(low.borrow().secs_f64(), high.borrow().secs_f64()))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformDuration(UniformFloat::<f64>::new_inclusive(
            low.borrow().secs_f64(),
            high.borrow().secs_f64(),
        ))
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Duration::new(self.0.sample(rng).try_into().unwrap())
    }
}

impl SampleUniform for Duration {
    type Sampler = UniformDuration;
}

impl<'a> Deserialize<'a> for Duration {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        struct DurationVisitor;

        impl Visitor<'_> for DurationVisitor {
            type Value = Duration;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("frequency")
            }

            fn visit_str<E>(self, v: &str) -> Result<Duration, E>
            where
                E: de::Error,
            {
                Duration::from_human(v).map_err(E::custom)
            }
        }

        d.deserialize_string(DurationVisitor)
    }
}

impl Serialize for Duration {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.human().map_err(ser::Error::custom)?)
    }
}

impl FromStr for Duration {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_human(s)
    }
}

impl EndpointConversion for Duration {
    fn to_open(p: &Self, left: bool) -> Option<Self> {
        <Decimal as EndpointConversion>::to_open(&p.secs, left).map(Self::new)
    }

    fn to_closed(p: &Self, left: bool) -> Option<Self> {
        <Decimal as EndpointConversion>::to_closed(&p.secs, left).map(Self::new)
    }
}

pub const ASEC: Duration = Duration::new(dec!(0.000000000000000001));
pub const FSEC: Duration = Duration::new(dec!(0.000000000000001));
pub const PSEC: Duration = Duration::new(dec!(0.000000000001));
pub const NSEC: Duration = Duration::new(dec!(0.000000001));
pub const USEC: Duration = Duration::new(dec!(0.000001));
pub const MSEC: Duration = Duration::new(dec!(0.001));
pub const SEC: Duration = Duration::new(dec!(1));
pub const MIN: Duration = Duration::new(dec!(60));
pub const HOUR: Duration = Duration::new(dec!(3600));
pub const DAY: Duration = Duration::new(dec!(86400));
pub const WEEK: Duration = Duration::new(dec!(604800));

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn human() -> Result<()> {
        assert!(Duration::new(Decimal::new(1, 26)).human().is_err());
        assert_eq!("1as", ASEC.human()?);
        assert_eq!("1fs", FSEC.human()?);
        assert_eq!("1ps", PSEC.human()?);
        assert_eq!("1ns", NSEC.human()?);
        assert_eq!("1us", USEC.human()?);
        assert_eq!("1ms", MSEC.human()?);
        assert_eq!("1s", SEC.human()?);
        assert_eq!("1s1ms", (SEC + MSEC).human()?);
        assert_eq!("1s1ms1us", (SEC + MSEC + USEC).human()?);
        assert_eq!("1s1ms1us1ns", (SEC + MSEC + USEC + NSEC).human()?);
        assert_eq!("1m", MIN.human()?);
        assert_eq!("1h", HOUR.human()?);
        assert_eq!("1d", DAY.human()?);
        assert_eq!("1w", WEEK.human()?);
        assert_eq!("5m", (5 * MIN).human()?);
        assert_eq!("15m", (15 * MIN).human()?);
        assert_eq!("15m7s", (15 * MIN + 7 * SEC).human()?);
        Ok(())
    }

    #[test]
    fn from_human() -> Result<()> {
        assert_eq!(Duration::from_human("1as")?, ASEC);
        assert_eq!(Duration::from_human("1fs")?, FSEC);
        assert_eq!(Duration::from_human("1ps")?, PSEC);
        assert_eq!(Duration::from_human("1ns")?, NSEC);
        assert_eq!(Duration::from_human("1us")?, USEC);
        assert_eq!(Duration::from_human("1ms")?, MSEC);
        assert_eq!(Duration::from_human("1s")?, SEC);
        assert_eq!(Duration::from_human("1s1ms")?, (SEC + MSEC));
        assert_eq!(Duration::from_human("1s1ms1us")?, (SEC + MSEC + USEC));
        assert_eq!(Duration::from_human("1s1ms1us1ns")?, (SEC + MSEC + USEC + NSEC));
        assert_eq!(Duration::from_human("1m")?, MIN);
        assert_eq!(Duration::from_human("1h")?, HOUR);
        assert_eq!(Duration::from_human("1d")?, DAY);
        assert_eq!(Duration::from_human("1w")?, WEEK);
        assert_eq!(Duration::from_human("5m")?, (5 * MIN));
        assert_eq!(Duration::from_human("15m")?, (15 * MIN));
        assert_eq!(Duration::from_human("15m7s")?, (15 * MIN + 7 * SEC));
        Ok(())
    }

    #[test]
    fn serialization() -> Result<()> {
        let dur = DAY;
        let se = serde_json::to_string(&dur)?;
        assert_eq!(se, "\"1d\"");
        let de: Duration = serde_json::from_str(&se)?;
        assert_eq!(de, dur);
        Ok(())
    }
}
