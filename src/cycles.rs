use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use derive_more::Display;
use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::duration::Duration;
use crate::fixed_freq::FixedFreq;
use crate::span::endpoint::{EndpointConversion, EndpointSide};

/// Number of occurrences of something.
#[must_use]
#[derive(
    Debug, Eq, PartialEq, Hash, Copy, Clone, Display, Ord, PartialOrd, Serialize, Deserialize,
)]
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
impl_op_ex!(/ |a: &Cycles, b: &Duration| -> FixedFreq { FixedFreq::new(*a, *b) });

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
    fn to_open(&self, side: EndpointSide) -> Option<Self> {
        self.count.to_open(side).map(Self::new)
    }

    fn to_closed(&self, side: EndpointSide) -> Option<Self> {
        self.count.to_closed(side).map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn new_and_accessors() {
        let c = Cycles::new(dec!(5));
        assert_eq!(c.count(), dec!(5));

        let c = Cycles::zero();
        assert_eq!(c.count(), dec!(0));

        let c = Cycles::one();
        assert_eq!(c.count(), dec!(1));

        let c = Cycles::from_count(42);
        assert_eq!(c.count(), dec!(42));
    }

    #[test]
    fn addition() {
        let a = Cycles::new(dec!(3));
        let b = Cycles::new(dec!(5));
        let c = a + b;
        assert_eq!(c.count(), dec!(8));

        let mut a = Cycles::new(dec!(10));
        a += Cycles::new(dec!(5));
        assert_eq!(a.count(), dec!(15));
    }

    #[test]
    fn subtraction() {
        let a = Cycles::new(dec!(10));
        let b = Cycles::new(dec!(3));
        let c = a - b;
        assert_eq!(c.count(), dec!(7));

        let mut a = Cycles::new(dec!(10));
        a -= Cycles::new(dec!(3));
        assert_eq!(a.count(), dec!(7));
    }

    #[test]
    fn division() {
        let a = Cycles::new(dec!(10));
        let b = Cycles::new(dec!(2));
        let result = a / b;
        assert_eq!(result, dec!(5));
    }

    #[test]
    fn multiplication_with_i64() {
        let c = Cycles::new(dec!(5));
        let result = c * 3_i64;
        assert_eq!(result.count(), dec!(15));

        let result = 3_i64 * c;
        assert_eq!(result.count(), dec!(15));

        let mut c = Cycles::new(dec!(5));
        c *= 3_i64;
        assert_eq!(c.count(), dec!(15));
    }

    #[test]
    fn division_with_i64() {
        let c = Cycles::new(dec!(15));
        let result = c / 3_i64;
        assert_eq!(result.count(), dec!(5));

        let mut c = Cycles::new(dec!(15));
        c /= 3_i64;
        assert_eq!(c.count(), dec!(5));
    }

    #[test]
    fn multiplication_with_decimal() {
        let c = Cycles::new(dec!(5));
        let result = c * dec!(2.5);
        assert_eq!(result.count(), dec!(12.5));

        let result = dec!(2.5) * c;
        assert_eq!(result.count(), dec!(12.5));
    }

    #[test]
    fn cycles_duration_ops() {
        let c = Cycles::new(dec!(10));
        let d = Duration::SEC;
        let result = c * d;
        assert_eq!(result, Duration::new(dec!(10)));

        let result = d * c;
        assert_eq!(result, Duration::new(dec!(10)));
    }

    #[test]
    fn cycles_divide_duration() {
        let c = Cycles::new(dec!(10));
        let d = Duration::SEC;
        let freq = c / d;
        assert_eq!(freq.hz(), dec!(10));
    }

    #[test]
    fn ordering() {
        let a = Cycles::new(dec!(5));
        let b = Cycles::new(dec!(10));
        let c = Cycles::new(dec!(5));

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, c);
        assert!(a <= c);
        assert!(a >= c);
    }

    #[test]
    fn to_primitive() {
        let c = Cycles::new(dec!(42));
        assert_eq!(c.to_i64(), Some(42));
        assert_eq!(c.to_u64(), Some(42));
        assert_eq!(c.to_f64(), Some(42.0));

        let c = Cycles::new(dec!(42.5));
        assert_eq!(c.to_f64(), Some(42.5));
    }

    #[test]
    fn endpoint_conversion() {
        let c = Cycles::new(dec!(5));

        // Left/right endpoint conversions should adjust value appropriately
        assert!(c.to_open(EndpointSide::Left).unwrap().count() < c.count());
        assert!(c.to_open(EndpointSide::Right).unwrap().count() > c.count());
        assert!(c.to_closed(EndpointSide::Left).unwrap().count() > c.count());
        assert!(c.to_closed(EndpointSide::Right).unwrap().count() < c.count());
    }

    #[test]
    fn display() {
        let c = Cycles::new(dec!(42));
        assert_eq!(format!("{}", c), "42");

        let c = Cycles::new(dec!(3.14));
        assert_eq!(format!("{}", c), "3.14");
    }

    #[test]
    fn zero_cycles() {
        let zero = Cycles::zero();
        let non_zero = Cycles::new(dec!(5));

        assert_eq!(zero + non_zero, non_zero);
        assert_eq!(non_zero + zero, non_zero);
        assert_eq!(non_zero - zero, non_zero);
    }

    #[test]
    fn negative_cycles() {
        let pos = Cycles::new(dec!(5));
        let neg = Cycles::new(dec!(-3));

        let result = pos + neg;
        assert_eq!(result.count(), dec!(2));

        let result = neg - pos;
        assert_eq!(result.count(), dec!(-8));
    }
}
