use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use derive_more::Display;
use eyre::Result;
use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::de::{self, Visitor};
use serde::{ser, Deserialize, Serialize};

use crate::cycles::Cycles;
use crate::duration::Duration;

/// Number of times something happens in a second. Hertz.
#[must_use]
#[derive(Debug, Eq, Copy, Clone, Display)]
#[display(fmt = "{}", "self.human().unwrap_or_else(|_| format!(\"{}:{}\", self.num, self.denom))")]
pub struct FixedFreq {
    /// Rational-like representation so that we can accurately represent ratios.
    /// Represents number of cycles per duration.
    num: Decimal,
    denom: Decimal,
}

impl Hash for FixedFreq {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // This will have some collisions, but it will definitely be the same
        // for the same ratios.
        (self.num / self.denom).hash(state);
    }
}

impl Ord for FixedFreq {
    fn cmp(&self, o: &Self) -> Ordering {
        let a = self.num * o.denom;
        let b = o.num * self.denom;
        a.cmp(&b)
    }
}

impl PartialOrd for FixedFreq {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl PartialEq for FixedFreq {
    fn eq(&self, o: &Self) -> bool {
        self.num * o.denom == o.num * self.denom
    }
}

impl FixedFreq {
    pub const ATTO: FixedFreq = FixedFreq::new(Cycles::one(), Duration::ASEC);
    pub const FEMTO: FixedFreq = FixedFreq::new(Cycles::one(), Duration::FSEC);
    pub const PICO: FixedFreq = FixedFreq::new(Cycles::one(), Duration::PSEC);
    pub const NANO: FixedFreq = FixedFreq::new(Cycles::one(), Duration::NSEC);
    pub const MICRO: FixedFreq = FixedFreq::new(Cycles::one(), Duration::USEC);
    pub const MILLI: FixedFreq = FixedFreq::new(Cycles::one(), Duration::MSEC);
    pub const SEC: FixedFreq = FixedFreq::new(Cycles::one(), Duration::SEC);
    pub const MIN: FixedFreq = FixedFreq::new(Cycles::one(), Duration::MIN);
    pub const HOURLY: FixedFreq = FixedFreq::new(Cycles::one(), Duration::HOUR);
    pub const DAILY: FixedFreq = FixedFreq::new(Cycles::one(), Duration::DAY);
    pub const WEEKLY: FixedFreq = FixedFreq::new(Cycles::one(), Duration::WEEK);

    pub const fn from_hz(hz: Decimal) -> Self {
        Self { num: hz, denom: dec!(1) }
    }

    /// |cycles| cycles per given duration.
    pub const fn new(cyc: Cycles, dur: Duration) -> Self {
        Self { num: cyc.count(), denom: dur.secs() }
    }

    /// |cycles| cycles per second.
    #[must_use]
    pub const fn num(&self) -> Decimal {
        self.num
    }

    /// Duration of a single cycle.
    #[must_use]
    pub const fn denom(&self) -> Decimal {
        self.denom
    }

    pub fn cycle_duration(&self) -> Duration {
        Duration::new(self.denom / self.num)
    }

    #[must_use]
    pub fn hz(&self) -> Decimal {
        self.num / self.denom
    }

    pub fn human(&self) -> Result<String> {
        self.human_bases(Duration::BASES)
    }

    pub fn human_bases(&self, bases: &[(&str, Duration)]) -> Result<String> {
        let dur_human = Duration::new(self.denom).human_bases(bases)?;
        if self.num == dec!(1) {
            Ok(dur_human)
        } else {
            Ok(format!("{}:{dur_human}", self.num))
        }
    }

    pub fn from_human(human: &str) -> Result<Self> {
        Ok(Self::new(Cycles::one(), Duration::from_human(human)?))
    }
}

impl_op_ex!(/ |a: &FixedFreq, b: &FixedFreq| -> Decimal { (a.num * b.denom) / (b.num * a.denom) });

// cycle / freq = dur
impl_op_ex!(/ |a: &Cycles, b: &FixedFreq| -> Duration { Duration::new(a.count() * b.denom / b.num) });

// dur * freq = cycles
impl_op_ex_commutative!(*|a: &FixedFreq, b: &Duration| -> Cycles {
    Cycles::new(b.secs() * a.num / a.denom)
});

// freq * cycles = dur
impl_op_ex_commutative!(*|a: &FixedFreq, b: &Cycles| -> Duration {
    Duration::new(b.count() * a.denom / a.num)
});

macro_rules! fixed_freq_ops {
    ($t:ty) => {
        impl_op_ex_commutative!(* |a: &FixedFreq, b: &$t| -> FixedFreq { FixedFreq { num: a.num * Decimal::try_from(*b).unwrap(), denom: a.denom } });
        impl_op_ex!(*= |a: &mut FixedFreq, b: &$t| { a.num *= Decimal::try_from(*b).unwrap() });

        impl_op_ex!(/ |a: &FixedFreq, b: &$t| -> FixedFreq { FixedFreq { num: a.num, denom: a.denom * Decimal::try_from(*b).unwrap() } });
        impl_op_ex!(/= |a: &mut FixedFreq, b: &$t| { a.denom *= Decimal::try_from(*b).unwrap() });
    };
}

fixed_freq_ops!(i16);
fixed_freq_ops!(u16);
fixed_freq_ops!(i32);
fixed_freq_ops!(u32);
fixed_freq_ops!(i64);
fixed_freq_ops!(u64);
fixed_freq_ops!(usize);
fixed_freq_ops!(Decimal);

impl ToPrimitive for FixedFreq {
    fn to_i64(&self) -> Option<i64> {
        self.hz().to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        self.hz().to_u64()
    }

    fn to_f64(&self) -> Option<f64> {
        self.hz().to_f64()
    }
}

impl<'a> Deserialize<'a> for FixedFreq {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        struct FreqVisitor;

        impl Visitor<'_> for FreqVisitor {
            type Value = FixedFreq;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("frequency")
            }

            fn visit_str<E>(self, v: &str) -> Result<FixedFreq, E>
            where
                E: de::Error,
            {
                FixedFreq::from_human(v).map_err(E::custom)
            }
        }

        d.deserialize_string(FreqVisitor)
    }
}

impl Serialize for FixedFreq {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.human().map_err(ser::Error::custom)?)
    }
}

impl FromStr for FixedFreq {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_human(s)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn serialization() -> Result<()> {
        let freq = FixedFreq::DAILY;
        let se = serde_json::to_string(&freq)?;
        assert_eq!(se, "\"1d\"");
        let de: FixedFreq = serde_json::from_str(&se)?;
        assert_eq!(de, freq);
        Ok(())
    }

    #[test]
    fn test_freq_from_hz() {
        let freq = FixedFreq::from_hz(dec!(60));
        assert_eq!(freq.num, dec!(60));
        assert_eq!(freq.denom, dec!(1));
    }

    #[test]
    fn test_freq_new() {
        let cyc = Cycles::new(dec!(5));
        let dur = Duration::new(dec!(2));
        let freq = FixedFreq::new(cyc, dur);
        assert_eq!(freq.num, dec!(5));
        assert_eq!(freq.denom, dec!(2));
    }

    #[test]
    fn test_freq_cycle_duration() {
        let cyc = Cycles::new(dec!(5));
        let dur = Duration::new(dec!(2));
        let freq = FixedFreq::new(cyc, dur);
        assert_eq!(freq.cycle_duration(), Duration::new(dec!(2) / dec!(5)));
    }

    #[test]
    fn test_freq_hz() {
        let cyc = Cycles::new(dec!(5));
        let dur = Duration::new(dec!(2));
        let freq = FixedFreq::new(cyc, dur);
        assert_eq!(freq.hz(), dec!(5) / dec!(2));
    }

    #[test]
    fn test_freq_eq() {
        let freq1 = FixedFreq::from_hz(dec!(60));
        let freq2 = FixedFreq::new(Cycles::new(dec!(120)), Duration::new(dec!(2)));
        assert_eq!(freq1, freq2);
    }

    #[test]
    fn test_freq_ord() {
        let freq1 = FixedFreq::from_hz(dec!(60));
        let freq2 = FixedFreq::from_hz(dec!(120));
        assert!(freq1 < freq2);
        assert!(FixedFreq::HOURLY > FixedFreq::DAILY);
    }

    #[test]
    fn test_freq_multiplication_with_cycles() {
        let freq = FixedFreq::from_hz(dec!(60));
        let cycles = Cycles::new(dec!(2));
        let result = freq * cycles;
        assert_eq!(result, Duration::new(dec!(1) / dec!(30)));
    }

    #[test]
    fn test_freq_multiplication_with_duration() {
        let freq = FixedFreq::from_hz(dec!(60));
        let duration = Duration::new(dec!(2));
        let result = freq * duration;
        assert_eq!(result, Cycles::new(dec!(120)));
    }

    #[test]
    fn test_freq_division_with_cycles() {
        let cycles = Cycles::new(dec!(120));
        let freq = FixedFreq::from_hz(dec!(60));
        let result = cycles / freq;
        assert_eq!(result, Duration::new(dec!(2)));
    }

    #[test]
    fn test_freq_division_with_duration() {
        let freq1 = FixedFreq::from_hz(dec!(120));
        let freq2 = FixedFreq::from_hz(dec!(60));
        let result = freq1 / freq2;
        assert_eq!(result, dec!(2));
    }
}
