use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use derive_more::Display;
use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::duration::Duration;
use crate::freq::Freq;
use crate::span::endpoint::EndpointConversion;

/// Number of occurrences of something.
#[must_use]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Display, Ord, PartialOrd)]
pub struct Cycles {
    count: Decimal,
}

impl Cycles {
    pub const fn new(count: Decimal) -> Self {
        Self { count }
    }

    pub const fn zero() -> Self {
        Self { count: dec!(0) }
    }

    pub const fn one() -> Self {
        Self { count: dec!(1) }
    }

    #[must_use]
    pub const fn count(self) -> Decimal {
        self.count
    }

    pub fn from_count(count: i64) -> Self {
        Self { count: Decimal::new(count, 0) }
    }
}

impl Add<Cycles> for Cycles {
    type Output = Cycles;

    fn add(self, d: Cycles) -> Self::Output {
        Cycles { count: self.count + d.count }
    }
}

impl AddAssign<Cycles> for Cycles {
    fn add_assign(&mut self, d: Cycles) {
        self.count += d.count;
    }
}

impl Sub<Cycles> for Cycles {
    type Output = Cycles;

    fn sub(self, d: Cycles) -> Self::Output {
        Cycles { count: self.count - d.count }
    }
}

impl SubAssign<Cycles> for Cycles {
    fn sub_assign(&mut self, d: Cycles) {
        self.count -= d.count;
    }
}

impl Div<Cycles> for Cycles {
    type Output = Decimal;

    fn div(self, o: Cycles) -> Self::Output {
        self.count / o.count
    }
}

/// cycle / dur = freq
impl Div<Duration> for Cycles {
    type Output = Freq;

    fn div(self, o: Duration) -> Self::Output {
        Freq::new(self, o)
    }
}

/// dur * cycles = dur
impl Mul<Duration> for Cycles {
    type Output = Duration;

    fn mul(self, o: Duration) -> Self::Output {
        o * self.count
    }
}

/// dur * cycles = dur
impl Mul<Cycles> for Duration {
    type Output = Duration;

    fn mul(self, o: Cycles) -> Self::Output {
        o.count * self
    }
}

macro_rules! cycle_ops {
    ($t:ty) => {
        impl MulAssign<$t> for Cycles {
            fn mul_assign(&mut self, rhs: $t) {
                self.count *= Decimal::try_from(rhs).unwrap();
            }
        }

        impl Mul<$t> for Cycles {
            type Output = Cycles;

            fn mul(self, rhs: $t) -> Self::Output {
                Self { count: self.count * Decimal::try_from(rhs).unwrap() }
            }
        }

        impl Mul<Cycles> for $t {
            type Output = Cycles;

            fn mul(self, rhs: Cycles) -> Self::Output {
                Cycles { count: Decimal::try_from(self).unwrap() * rhs.count }
            }
        }

        impl DivAssign<$t> for Cycles {
            fn div_assign(&mut self, rhs: $t) {
                self.count /= Decimal::try_from(rhs).unwrap();
            }
        }

        impl Div<$t> for Cycles {
            type Output = Cycles;

            fn div(self, rhs: $t) -> Self::Output {
                Self { count: self.count / Decimal::try_from(rhs).unwrap() }
            }
        }
    };
}

cycle_ops!(i64);
cycle_ops!(Decimal);

impl ToPrimitive for Cycles {
    fn to_i64(&self) -> Option<i64> {
        self.count.to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        self.count.to_u64()
    }

    fn to_f64(&self) -> Option<f64> {
        self.count.to_f64()
    }
}

impl EndpointConversion for Cycles {
    fn to_open(p: &Self, left: bool) -> Option<Self> {
        <Decimal as EndpointConversion>::to_open(&p.count, left).map(Self::new)
    }

    fn to_closed(p: &Self, left: bool) -> Option<Self> {
        <Decimal as EndpointConversion>::to_closed(&p.count, left).map(Self::new)
    }
}
