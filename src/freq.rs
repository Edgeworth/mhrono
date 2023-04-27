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
use crate::duration::{
    Duration, ASEC, BASES, DAY, FSEC, HOUR, MIN, MSEC, NSEC, PSEC, SEC, USEC, WEEK,
};

/// Number of times something happens in a second. Hertz.
#[must_use]
#[derive(Debug, Eq, Copy, Clone, Display)]
#[display(fmt = "{}", "self.human().unwrap_or_else(|_| format!(\"{}:{}\", self.num, self.denom))")]
pub struct Freq {
    /// Rational-like representation so that we can accurately represent ratios.
    /// Represents number of cycles per duration.
    num: Decimal,
    denom: Decimal,
}

impl Hash for Freq {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // This will have some collisions, but it will definitely be the same
        // for the same ratios.
        (self.num / self.denom).hash(state);
    }
}

impl Ord for Freq {
    fn cmp(&self, o: &Self) -> Ordering {
        let a = self.num * o.denom;
        let b = o.num * self.denom;
        // Reverse since a smaller period corresponds to a higher frequency.
        b.cmp(&a)
    }
}

impl PartialOrd for Freq {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl PartialEq for Freq {
    fn eq(&self, o: &Self) -> bool {
        self.num * o.denom == o.num * self.denom
    }
}

impl Freq {
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
        self.human_bases(BASES)
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

impl_op_ex!(/ |a: &Freq, b: &Freq| -> Decimal { (a.num * b.denom) / (b.num * a.denom) });

// cycle / freq = dur
impl_op_ex!(/ |a: &Cycles, b: &Freq| -> Duration { Duration::new(a.count() * b.denom / b.num) });

// dur * freq = cycles
impl_op_ex_commutative!(*|a: &Freq, b: &Duration| -> Cycles {
    Cycles::new(b.secs() * a.num / a.denom)
});

macro_rules! freq_ops {
    ($t:ty) => {

        impl_op_ex_commutative!(* |a: &Freq, b: &$t| -> Freq { Freq { num: a.num * Decimal::try_from(*b).unwrap(), denom: a.denom } });
        impl_op_ex!(*= |a: &mut Freq, b: &$t| { a.num *= Decimal::try_from(*b).unwrap() });

        impl_op_ex!(/ |a: &Freq, b: &$t| -> Freq { Freq { num: a.num, denom: a.denom * Decimal::try_from(*b).unwrap() } });
        impl_op_ex!(/= |a: &mut Freq, b: &$t| { a.denom *= Decimal::try_from(*b).unwrap() });
    };
}

freq_ops!(i64);
freq_ops!(Decimal);

impl ToPrimitive for Freq {
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
                Freq::from_human(v).map_err(E::custom)
            }
        }

        d.deserialize_string(FreqVisitor)
    }
}

impl Serialize for Freq {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.human().map_err(ser::Error::custom)?)
    }
}

impl FromStr for Freq {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_human(s)
    }
}

pub const ASECLY: Freq = Freq::new(Cycles::one(), ASEC);
pub const FSECLY: Freq = Freq::new(Cycles::one(), FSEC);
pub const PSECLY: Freq = Freq::new(Cycles::one(), PSEC);
pub const NSECLY: Freq = Freq::new(Cycles::one(), NSEC);
pub const USECLY: Freq = Freq::new(Cycles::one(), USEC);
pub const MSECLY: Freq = Freq::new(Cycles::one(), MSEC);
pub const SECLY: Freq = Freq::new(Cycles::one(), SEC);
pub const MINLY: Freq = Freq::new(Cycles::one(), MIN);
pub const HOURLY: Freq = Freq::new(Cycles::one(), HOUR);
pub const DAILY: Freq = Freq::new(Cycles::one(), DAY);
pub const WEEKLY: Freq = Freq::new(Cycles::one(), WEEK);

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn serialization() -> Result<()> {
        let freq = DAILY;
        let se = serde_json::to_string(&freq)?;
        assert_eq!(se, "\"1d\"");
        let de: Freq = serde_json::from_str(&se)?;
        assert_eq!(de, freq);
        Ok(())
    }
}
