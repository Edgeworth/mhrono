use std::fmt;
use std::fmt::Write;
use std::iter::once;
use std::str::FromStr;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use derive_more::Display;
use eyre::{Result, eyre};
use num_traits::ToPrimitive;
use rand::distr::uniform::{SampleBorrow, SampleUniform, UniformFloat, UniformSampler};
use rand::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize, ser};

use crate::span::endpoint::EndpointConversion;

#[must_use]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Display, Ord, PartialOrd)]
#[display("{}", self.human().unwrap_or_else(|_| self.secs.to_string()))]
pub struct Duration {
    secs: Decimal,
}

impl Duration {
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

    pub const BASES: &'static [(&'static str, Duration)] = &[
        ("w", Duration::WEEK),
        ("d", Duration::DAY),
        ("h", Duration::HOUR),
        ("m", Duration::MIN),
        ("s", Duration::SEC),
        ("ms", Duration::MSEC),
        ("us", Duration::USEC),
        ("ns", Duration::NSEC),
        ("ps", Duration::PSEC),
        ("fs", Duration::FSEC),
        ("as", Duration::ASEC),
    ];

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
        self.human_bases(Duration::BASES)
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
        if rem.is_zero() { Ok(human) } else { Err(eyre!("remainder is not zero")) }
    }

    pub fn from_human(s: &str) -> Result<Duration> {
        let mut dur = Duration::zero();

        // First character must be a digit:
        if s.is_empty() {
            return Err(eyre!("empty duration"));
        }
        if !s.chars().next().unwrap().is_ascii_digit() {
            return Err(eyre!("duration must start with a digit"));
        }

        let mut cur_number = 0;
        let mut cur_ident = String::new();
        let mut is_digit = true;
        for c in s.chars().chain(once('0')) {
            if let Some(digit) = c.to_digit(10) {
                if !is_digit {
                    let base = Duration::BASES
                        .iter()
                        .find(|v| v.0 == cur_ident)
                        .ok_or_else(|| eyre!("unknown duration"))?;
                    dur += cur_number * base.1;
                    cur_number = 0;
                    cur_ident.clear();
                }
                cur_number = cur_number * 10 + digit as i64;
                is_digit = true;
            } else {
                cur_ident.push(c);
                is_digit = false;
            }
        }

        Ok(dur)
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::zero()
    }
}

impl_op_ex!(+ |a: &Duration, b: &Duration| -> Duration {Duration::new(a.secs + b.secs) });
impl_op_ex!(+= |a: &mut Duration, b: &Duration| { a.secs += b.secs });

impl_op_ex!(-|a: &Duration, b: &Duration| -> Duration { Duration::new(a.secs - b.secs) });
impl_op_ex!(-= |a: &mut Duration, b: &Duration| { a.secs -= b.secs });

impl_op_ex!(/ |a: &Duration, b: &Duration| -> Decimal { a.secs / b.secs });

macro_rules! duration_ops {
    ($t:ty) => {
        impl_op_ex_commutative!(* |a: &Duration, b: &$t| -> Duration { Duration::new(a.secs * Decimal::try_from(*b).unwrap()) });
        impl_op_ex!(*= |a: &mut Duration, b: &$t| { a.secs *= Decimal::try_from(*b).unwrap() });

        impl_op_ex!(/ |a: &Duration, b: &$t| -> Duration { Duration::new(a.secs / Decimal::try_from(*b).unwrap()) });
        impl_op_ex!(/= |a: &mut Duration, b: &$t| { a.secs /= Decimal::try_from(*b).unwrap() });

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

    fn new<B1, B2>(low: B1, high: B2) -> Result<Self, rand::distr::uniform::Error>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Ok(UniformDuration(UniformFloat::<f64>::new(
            low.borrow().secs_f64(),
            high.borrow().secs_f64(),
        )?))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Result<Self, rand::distr::uniform::Error>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Ok(UniformDuration(UniformFloat::<f64>::new_inclusive(
            low.borrow().secs_f64(),
            high.borrow().secs_f64(),
        )?))
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
    fn to_open(&self, left: bool) -> Option<Self> {
        self.secs.to_open(left).map(Self::new)
    }

    fn to_closed(&self, left: bool) -> Option<Self> {
        self.secs.to_closed(left).map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn human() -> Result<()> {
        assert!(Duration::new(Decimal::new(1, 26)).human().is_err());
        assert_eq!("1as", Duration::ASEC.human()?);
        assert_eq!("1fs", Duration::FSEC.human()?);
        assert_eq!("1ps", Duration::PSEC.human()?);
        assert_eq!("1ns", Duration::NSEC.human()?);
        assert_eq!("1us", Duration::USEC.human()?);
        assert_eq!("1ms", Duration::MSEC.human()?);
        assert_eq!("1s", Duration::SEC.human()?);
        assert_eq!("1s1ms", (Duration::SEC + Duration::MSEC).human()?);
        assert_eq!("1s1ms1us", (Duration::SEC + Duration::MSEC + Duration::USEC).human()?);
        assert_eq!(
            "1s1ms1us1ns",
            (Duration::SEC + Duration::MSEC + Duration::USEC + Duration::NSEC).human()?
        );
        assert_eq!("1m", Duration::MIN.human()?);
        assert_eq!("1h", Duration::HOUR.human()?);
        assert_eq!("1d", Duration::DAY.human()?);
        assert_eq!("1w", Duration::WEEK.human()?);
        assert_eq!("5m", (5 * Duration::MIN).human()?);
        assert_eq!("15m", (15 * Duration::MIN).human()?);
        assert_eq!("15m7s", (15 * Duration::MIN + 7 * Duration::SEC).human()?);
        Ok(())
    }

    #[test]
    fn from_human() -> Result<()> {
        assert_eq!(Duration::from_human("1as")?, Duration::ASEC);
        assert_eq!(Duration::from_human("1fs")?, Duration::FSEC);
        assert_eq!(Duration::from_human("1ps")?, Duration::PSEC);
        assert_eq!(Duration::from_human("1ns")?, Duration::NSEC);
        assert_eq!(Duration::from_human("1us")?, Duration::USEC);
        assert_eq!(Duration::from_human("1ms")?, Duration::MSEC);
        assert_eq!(Duration::from_human("1s")?, Duration::SEC);
        assert_eq!(Duration::from_human("1s1ms")?, (Duration::SEC + Duration::MSEC));
        assert_eq!(
            Duration::from_human("1s1ms1us")?,
            (Duration::SEC + Duration::MSEC + Duration::USEC)
        );
        assert_eq!(
            Duration::from_human("1s1ms1us1ns")?,
            (Duration::SEC + Duration::MSEC + Duration::USEC + Duration::NSEC)
        );
        assert_eq!(Duration::from_human("1m")?, Duration::MIN);
        assert_eq!(Duration::from_human("1h")?, Duration::HOUR);
        assert_eq!(Duration::from_human("1d")?, Duration::DAY);
        assert_eq!(Duration::from_human("1w")?, Duration::WEEK);
        assert_eq!(Duration::from_human("5m")?, (5 * Duration::MIN));
        assert_eq!(Duration::from_human("15m")?, (15 * Duration::MIN));
        assert_eq!(Duration::from_human("15m7s")?, (15 * Duration::MIN + 7 * Duration::SEC));
        Ok(())
    }

    #[test]
    fn serialization() -> Result<()> {
        let dur = Duration::DAY;
        let se = serde_json::to_string(&dur)?;
        assert_eq!(se, "\"1d\"");
        let de: Duration = serde_json::from_str(&se)?;
        assert_eq!(de, dur);
        Ok(())
    }
}
