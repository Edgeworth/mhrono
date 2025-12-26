use std::fmt;
use std::fmt::Write;
use std::iter::once;
use std::str::FromStr;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use derive_more::Display;
use num_traits::ToPrimitive;
use rand::distr::uniform::{SampleBorrow, SampleUniform, UniformFloat, UniformSampler};
use rand::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize, ser};

use crate::span::endpoint::{EndpointConversion, EndpointSide};
use crate::{Error, Result};

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
    pub const fn is_positive(&self) -> bool {
        self.secs.is_sign_positive() && !self.secs.is_zero()
    }

    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.secs.is_sign_negative()
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
    pub fn to_chrono(&self) -> Option<std::time::Duration> {
        let secs = self.secs.trunc();
        let nanos = ((self.secs - secs) * dec!(1000000000)).trunc();
        Some(std::time::Duration::new(secs.to_u64()?, nanos.to_u32()?))
    }

    pub fn human(&self) -> Result<String> {
        self.human_bases(Duration::BASES)
    }

    pub fn human_bases(&self, bases: &[(&str, Duration)]) -> Result<String> {
        if self.is_zero() {
            return Ok("0s".to_string());
        }
        let mut rem = *self;
        let mut human = String::new();

        if rem.is_negative() {
            rem = -rem;
            write!(human, "-").unwrap();
        }

        for &(s, dur) in bases {
            let div = (rem / dur).trunc();
            rem -= dur * div;
            if !div.is_zero() {
                write!(human, "{div}{s}").unwrap();
            }
        }
        // Some sub-attosecond duration...
        if rem.is_zero() {
            Ok(human)
        } else {
            Err(Error::DurationParse("remainder is not zero".to_string()))
        }
    }

    pub fn from_human(s: &str) -> Result<Duration> {
        let mut dur = Duration::zero();

        // First character must be a digit:
        if s.is_empty() {
            return Err(Error::DurationParse("empty duration".to_string()));
        }

        let (s, sign) = match s.chars().next().unwrap() {
            '-' => (&s[1..], -1),
            '+' => (&s[1..], 1),
            _ => (s, 1),
        };

        if !s.chars().next().unwrap().is_ascii_digit() {
            return Err(Error::DurationParse("duration must start with a digit".to_string()));
        }

        let mut cur_number = 0;
        let mut cur_ident = String::new();
        let mut is_digit = true;
        for c in s.chars().chain(once('0')) {
            if let Some(digit) = c.to_digit(10) {
                if !is_digit {
                    let base =
                        Duration::BASES.iter().find(|v| v.0 == cur_ident).ok_or_else(|| {
                            Error::DurationParse(format!("unknown duration unit {cur_ident}"))
                        })?;
                    dur += cur_number * base.1;
                    cur_number = 0;
                    cur_ident.clear();
                }
                cur_number = cur_number
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(digit as i64))
                    .ok_or_else(|| {
                    Error::DurationParse("overflow in duration number".to_string())
                })?;
                is_digit = true;
            } else {
                cur_ident.push(c);
                is_digit = false;
            }
        }
        if cur_number != 0 {
            return Err(Error::DurationParse("trailing number without unit".to_string()));
        }

        Ok(dur * sign)
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::zero()
    }
}

impl_op_ex!(-|a: &Duration| -> Duration { Duration::new(-a.secs) });

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
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_human(s)
    }
}

impl EndpointConversion for Duration {
    fn to_open(&self, side: EndpointSide) -> Option<Self> {
        self.secs.to_open(side).map(Self::new)
    }

    fn to_closed(&self, side: EndpointSide) -> Option<Self> {
        self.secs.to_closed(side).map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
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
    fn serialization() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dur = Duration::DAY;
        let se = serde_json::to_string(&dur)?;
        assert_eq!(se, "\"1d\"");
        let de: Duration = serde_json::from_str(&se)?;
        assert_eq!(de, dur);
        Ok(())
    }

    #[test]
    fn serialization_negative_round_trips() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dur = Duration::new(dec!(-61));
        let se = serde_json::to_string(&dur)?;
        assert_eq!(se, "\"-1m1s\"");
        let de: Duration = serde_json::from_str(&se)?;
        assert_eq!(de, dur);
        Ok(())
    }

    #[test]
    fn constants() {
        assert_eq!(Duration::NSEC.secs(), dec!(0.000000001));
        assert_eq!(Duration::SEC.secs(), dec!(1));
        assert_eq!(Duration::MIN.secs(), dec!(60));
        assert_eq!(Duration::DAY.secs(), dec!(86400));
    }

    #[test]
    fn new_and_zero() {
        let d = Duration::new(dec!(42.5));
        assert_eq!(d.secs(), dec!(42.5));

        let z = Duration::zero();
        assert_eq!(z.secs(), dec!(0));
        assert!(z.is_zero());

        let d = Duration::default();
        assert_eq!(d.secs(), dec!(0));
        assert!(d.is_zero());
    }

    #[test]
    fn addition() {
        let a = Duration::SEC;
        let b = Duration::MSEC;
        let c = a + b;
        assert_eq!(c.secs(), dec!(1.001));

        let mut a = Duration::SEC;
        a += Duration::MSEC;
        assert_eq!(a.secs(), dec!(1.001));
    }

    #[test]
    fn subtraction() {
        let a = Duration::SEC;
        let b = Duration::MSEC;
        let c = a - b;
        assert_eq!(c.secs(), dec!(0.999));

        let mut a = Duration::SEC;
        a -= Duration::MSEC;
        assert_eq!(a.secs(), dec!(0.999));
    }

    #[test]
    fn division() {
        let a = Duration::MIN;
        let b = Duration::SEC;
        let ratio = a / b;
        assert_eq!(ratio, dec!(60));
    }

    #[test]
    fn multiplication_with_i64() {
        let d = Duration::SEC;
        let result = d * 5_i64;
        assert_eq!(result.secs(), dec!(5));

        let result = 5_i64 * d;
        assert_eq!(result.secs(), dec!(5));
    }

    #[test]
    fn multiplication_with_decimal() {
        let d = Duration::SEC;
        let result = d * dec!(2.5);
        assert_eq!(result.secs(), dec!(2.5));

        let result = dec!(2.5) * d;
        assert_eq!(result.secs(), dec!(2.5));
    }

    #[test]
    fn division_with_decimal() {
        let d = Duration::new(dec!(10));
        let result = d / dec!(2);
        assert_eq!(result.secs(), dec!(5));
    }

    #[test]
    fn ordering() {
        let a = Duration::SEC;
        let b = Duration::MIN;
        let c = Duration::SEC;

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, c);
        assert!(a <= c);
        assert!(a >= c);
    }

    #[test]
    fn secs_f64() {
        let d = Duration::new(dec!(42.5));
        assert_relative_eq!(d.secs_f64(), 42.5);
    }

    #[test]
    fn to_chrono() {
        let d = Duration::new(dec!(1.5));
        assert_eq!(d.to_chrono(), Some(std::time::Duration::new(1, 500_000_000)));

        let d = Duration::SEC;
        assert_eq!(d.to_chrono(), Some(std::time::Duration::new(1, 0)));
    }

    #[test]
    fn to_chrono_negative_is_none() {
        let d = Duration::new(dec!(-1));
        assert_eq!(d.to_chrono(), None);
    }

    #[test]
    fn zero_duration() {
        let zero = Duration::zero();
        let one_sec = Duration::SEC;

        assert_eq!(zero + one_sec, one_sec);
        assert_eq!(one_sec + zero, one_sec);
        assert_eq!(one_sec - zero, one_sec);
        assert_eq!(zero * 100, zero);
    }

    #[test]
    fn zero_round_trip() -> Result<()> {
        let zero = Duration::zero();
        let human = zero.human()?;
        assert_eq!(human, "0s");
        let parsed = Duration::from_human(&human)?;
        assert_eq!(parsed, zero);
        Ok(())
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", Duration::SEC), "1s");
        assert_eq!(format!("{}", Duration::MIN), "1m");
        assert_eq!(format!("{}", Duration::HOUR), "1h");
        assert_eq!(format!("{}", Duration::DAY), "1d");
    }

    #[test]
    fn endpoint_conversion() {
        let d = Duration::SEC;

        // Left endpoint to open should subtract ULP
        let left_open = d.to_open(EndpointSide::Left).unwrap();
        assert!(left_open.secs() < d.secs());

        // Right endpoint to open should add ULP
        let right_open = d.to_open(EndpointSide::Right).unwrap();
        assert!(right_open.secs() > d.secs());

        // Left endpoint to closed should add ULP
        let left_closed = d.to_closed(EndpointSide::Left).unwrap();
        assert!(left_closed.secs() > d.secs());

        // Right endpoint to closed should subtract ULP
        let right_closed = d.to_closed(EndpointSide::Right).unwrap();
        assert!(right_closed.secs() < d.secs());
    }

    #[test]
    fn endpoint_conversion_uses_decimal_ulp() {
        let d = Duration::SEC;
        let ulp = Decimal::new(1, Decimal::MAX_SCALE);

        assert_eq!(d.to_open(EndpointSide::Left).unwrap(), Duration::new(dec!(1) - ulp));
        assert_eq!(d.to_open(EndpointSide::Right).unwrap(), Duration::new(dec!(1) + ulp));
        assert_eq!(d.to_closed(EndpointSide::Left).unwrap(), Duration::new(dec!(1) + ulp));
        assert_eq!(d.to_closed(EndpointSide::Right).unwrap(), Duration::new(dec!(1) - ulp));
    }

    #[test]
    fn from_human_errors() {
        assert!(Duration::from_human("").is_err());
        assert!(Duration::from_human("xyz").is_err());
        assert!(Duration::from_human("1xyz").is_err());
        assert!(Duration::from_human("1").is_err());
        assert!(Duration::from_human("1ms2").is_err());
    }

    #[test]
    fn from_human_zero_units() -> Result<()> {
        assert_eq!(Duration::from_human("0s")?, Duration::zero());
        assert_eq!(Duration::from_human("0ms")?, Duration::zero());
        assert_eq!(Duration::from_human("0m")?, Duration::zero());
        assert_eq!(Duration::from_human("0h")?, Duration::zero());
        assert_eq!(Duration::from_human("0d")?, Duration::zero());
        assert_eq!(Duration::from_human("0w")?, Duration::zero());
        assert_eq!(Duration::from_human("0as")?, Duration::zero());
        assert_eq!(Duration::from_human("0fs")?, Duration::zero());
        assert_eq!(Duration::from_human("0ps")?, Duration::zero());
        assert_eq!(Duration::from_human("0ns")?, Duration::zero());
        assert_eq!(Duration::from_human("0us")?, Duration::zero());
        // Mixed zeros should also be valid.
        assert_eq!(Duration::from_human("0s0ms")?, Duration::zero());
        Ok(())
    }

    #[test]
    fn duration_arithmetic_chain() -> Result<()> {
        let base = Duration::HOUR;
        let result = base + Duration::MIN * 30 + Duration::SEC * 45;

        assert_eq!(result.secs(), dec!(5445));
        assert_eq!(result.human()?, "1h30m45s");

        Ok(())
    }

    #[test]
    fn from_human_overflow_is_error() {
        // Too many digits to fit in i64 should result in an error.
        let big = "9999999999999999999999999999s";
        assert!(Duration::from_human(big).is_err());
    }
}
