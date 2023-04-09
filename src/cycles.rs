use auto_ops::{impl_op_ex, impl_op_ex_commutative};
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

impl_op_ex!(+ |a: &Cycles, b: &Cycles| -> Cycles { Cycles { count: a.count + b.count } });
impl_op_ex!(+= |a: &mut Cycles, b: &Cycles| { *a = *a + b });

impl_op_ex!(-|a: &Cycles, b: &Cycles| -> Cycles { Cycles { count: a.count - b.count } });
impl_op_ex!(-= |a: &mut Cycles, b: &Cycles| { *a = *a - b });

impl_op_ex!(/ |a: &Cycles, b: &Cycles| -> Decimal { a.count / b.count });

// cycle / dur = freq
impl_op_ex!(/ |a: &Cycles, b: &Duration| -> Freq { Freq::new(*a, *b) });

// dur * cycles = dur
impl_op_ex_commutative!(*|a: &Cycles, b: &Duration| -> Duration { *b * a.count });

macro_rules! cycle_ops {
    ($t:ty) => {
        impl_op_ex_commutative!(* |a: & Cycles, b: &$t| -> Cycles { Cycles { count: a.count * Decimal::try_from(*b).unwrap() } });
        impl_op_ex!(*= |a: &mut Cycles, b: &$t| { a.count *= Decimal::try_from(*b).unwrap() });

        impl_op_ex_commutative!(/ |a: & Cycles, b: &$t| -> Cycles { Cycles { count: a.count / Decimal::try_from(*b).unwrap() } });
        impl_op_ex!(/= |a: &mut Cycles, b: &$t| { a.count /= Decimal::try_from(*b).unwrap() });
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
