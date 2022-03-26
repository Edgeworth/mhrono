use std::lazy::SyncLazy;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use derive_more::Display;
use eyre::{eyre, Result};
use num_traits::ToPrimitive;
use regex::Regex;

const BASES: &[(&str, Duration)] = &[("w", WEEK), ("d", DAY), ("h", HOUR), ("m", MIN), ("s", SEC)];

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Display, Ord, PartialOrd)]
#[display(fmt = "{}", "self.human()")]
pub struct Duration {
    secs: i64,
}

impl Duration {
    #[must_use]
    pub const fn new(d: i64) -> Self {
        Self { secs: d }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self { secs: 0 }
    }

    #[must_use]
    pub const fn as_f64(self) -> f64 {
        self.secs as f64
    }

    #[must_use]
    pub const fn secs(&self) -> i64 {
        self.secs
    }

    #[must_use]
    pub fn human(&self) -> String {
        let mut rem = *self;
        let mut human = String::new();
        for (s, dur) in BASES.iter() {
            let div = (rem / *dur) as i64;
            rem -= *dur * div;
            if div > 0 {
                human += &format!("{}{}", div, s);
            }
        }
        human
    }

    pub fn from_human(s: &str) -> Result<Duration> {
        static RE: SyncLazy<Regex> = SyncLazy::new(|| Regex::new(r"(\d+)([a-z]+)").unwrap());
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

impl Mul<f64> for Duration {
    type Output = Duration;

    fn mul(self, o: f64) -> Self::Output {
        Self::new((self.as_f64() * o) as i64)
    }
}

impl Div<f64> for Duration {
    type Output = Duration;

    fn div(self, o: f64) -> Self::Output {
        Self::new((self.as_f64() / o) as i64)
    }
}

impl Div<Duration> for Duration {
    type Output = f64;

    fn div(self, o: Duration) -> Self::Output {
        self.as_f64() / o.as_f64()
    }
}

macro_rules! duration_ops {
    ($t:ty) => {
        impl MulAssign<$t> for Duration {
            fn mul_assign(&mut self, rhs: $t) {
                self.secs *= rhs as i64;
            }
        }

        impl Mul<$t> for Duration {
            type Output = Duration;

            fn mul(self, rhs: $t) -> Self::Output {
                Self::new(self.secs * rhs as i64)
            }
        }

        impl Mul<Duration> for $t {
            type Output = Duration;

            fn mul(self, rhs: Duration) -> Self::Output {
                Duration::new(self as i64 * rhs.secs)
            }
        }

        impl DivAssign<$t> for Duration {
            fn div_assign(&mut self, rhs: $t) {
                self.secs /= rhs as i64;
            }
        }

        impl Div<$t> for Duration {
            type Output = Duration;

            fn div(self, rhs: $t) -> Self::Output {
                Self::new(self.secs / rhs as i64)
            }
        }
    };
}

duration_ops!(i64);

impl ToPrimitive for Duration {
    fn to_i64(&self) -> Option<i64> {
        Some(self.secs())
    }

    fn to_u64(&self) -> Option<u64> {
        self.secs().to_u64()
    }

    fn to_f64(&self) -> Option<f64> {
        Some(Duration::as_f64(*self))
    }
}

pub const SEC: Duration = Duration::new(1);
pub const MIN: Duration = Duration::new(60);
pub const HOUR: Duration = Duration::new(60 * 60);
pub const DAY: Duration = Duration::new(24 * 60 * 60);
pub const WEEK: Duration = Duration::new(24 * 60 * 60 * 7);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human() {
        assert_eq!("1s", SEC.human());
        assert_eq!("1m", MIN.human());
        assert_eq!("1h", HOUR.human());
        assert_eq!("1d", DAY.human());
        assert_eq!("1w", WEEK.human());
        assert_eq!("5m", (5 * MIN).human());
        assert_eq!("15m", (15 * MIN).human());
        assert_eq!("15m7s", (15 * MIN + 7 * SEC).human());
    }

    #[test]
    fn from_human() -> Result<()> {
        assert_eq!(Duration::from_human("1s")?, SEC);
        assert_eq!(Duration::from_human("1m")?, MIN);
        assert_eq!(Duration::from_human("1h")?, HOUR);
        assert_eq!(Duration::from_human("1d")?, DAY);
        assert_eq!(Duration::from_human("1w")?, WEEK);
        assert_eq!(Duration::from_human("5m")?, (5 * MIN));
        assert_eq!(Duration::from_human("15m")?, (15 * MIN));
        assert_eq!(Duration::from_human("15m7s")?, (15 * MIN + 7 * SEC));
        Ok(())
    }
}
