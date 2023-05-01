use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Bound, Sub, SubAssign};

use derive_more::IsVariant;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents an endpoint of a span. For comparison, endpoints behave as closed
/// points - that is, an open endpoint should be compared with >= and <=.
#[must_use]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize, IsVariant)]
pub enum Endpoint<T> {
    /// |left| is whether this endpoint ends on the left or the right side. If true,
    /// imagine the span extending off from the left to the right.
    Open {
        p: T,
        left: bool,
    },
    Closed {
        p: T,
        left: bool,
    },
    Unbounded {
        left: bool,
    },
}

impl<T> Endpoint<T> {
    pub fn from_bound(bound: Bound<T>, left: bool) -> Self {
        match bound {
            Bound::Included(p) => Self::Closed { p, left },
            Bound::Excluded(p) => Self::Open { p, left },
            Bound::Unbounded => Self::Unbounded { left },
        }
    }

    pub const fn bound(&self) -> Bound<&T> {
        match self {
            Endpoint::Open { p, .. } => Bound::Excluded(p),
            Endpoint::Closed { p, .. } => Bound::Included(p),
            Endpoint::Unbounded { .. } => Bound::Unbounded,
        }
    }

    pub const fn value(&self) -> Option<&T> {
        match self {
            Endpoint::Closed { p, .. } | Endpoint::Open { p, .. } => Some(p),
            Endpoint::Unbounded { .. } => None,
        }
    }

    pub const fn is_left(&self) -> bool {
        match self {
            Endpoint::Open { left, .. }
            | Endpoint::Closed { left, .. }
            | Endpoint::Unbounded { left } => *left,
        }
    }

    pub const fn is_right(&self) -> bool {
        !self.is_left()
    }

    pub const fn is_left_unbounded(&self) -> bool {
        match self {
            Endpoint::Unbounded { left } => *left,
            _ => false,
        }
    }

    pub const fn is_right_unbounded(&self) -> bool {
        match self {
            Endpoint::Unbounded { left } => !*left,
            _ => false,
        }
    }
}

impl<T: Clone> From<Endpoint<T>> for Bound<T> {
    fn from(value: Endpoint<T>) -> Self {
        value.bound().cloned()
    }
}

impl<T: fmt::Display> fmt::Display for Endpoint<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Endpoint::Open { p, left } => {
                if *left {
                    write!(f, "({p}")
                } else {
                    write!(f, "{p})")
                }
            }
            Endpoint::Closed { p, left } => {
                if *left {
                    write!(f, "[{p}")
                } else {
                    write!(f, "{p}]")
                }
            }
            Endpoint::Unbounded { left } => {
                if *left {
                    write!(f, "(-inf")
                } else {
                    write!(f, "+inf)")
                }
            }
        }
    }
}

impl<T: PartialEq> PartialEq<T> for Endpoint<T> {
    fn eq(&self, other: &T) -> bool {
        if let Endpoint::Closed { p, .. } = self {
            p == other
        } else {
            // If open, can't be equal to any closed point.
            false
        }
    }
}

impl<T: PartialOrd> PartialOrd<T> for Endpoint<T> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        match self {
            Endpoint::Open { p, left } => match p.partial_cmp(other) {
                Some(Ordering::Equal) => {
                    if *left {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Less)
                    }
                }
                x => x,
            },
            Endpoint::Closed { p, .. } => p.partial_cmp(other),
            Endpoint::Unbounded { left } => {
                if *left {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
        }
    }
}

impl<T: Ord> Ord for Endpoint<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<T: PartialOrd> PartialOrd for Endpoint<T> {
    fn partial_cmp(&self, other: &Endpoint<T>) -> Option<Ordering> {
        match (self, other) {
            (Endpoint::Open { p: p1, left: left1 }, Endpoint::Open { p: p2, left: left2 }) => {
                match p1.partial_cmp(p2) {
                    Some(Ordering::Equal) => match (left1, left2) {
                        (false, false) | (true, true) => Some(Ordering::Equal),
                        (true, false) => Some(Ordering::Greater),
                        (false, true) => Some(Ordering::Less),
                    },
                    x => x,
                }
            }
            (Endpoint::Open { p: p1, left: left1 }, Endpoint::Closed { p: p2, left: left2 }) => {
                match p1.partial_cmp(p2) {
                    Some(Ordering::Equal) => match (left1, left2) {
                        (true, false | true) => Some(Ordering::Greater),
                        (false, true | false) => Some(Ordering::Less),
                    },
                    x => x,
                }
            }
            (Endpoint::Open { .. } | Endpoint::Closed { .. }, Endpoint::Unbounded { left }) => {
                if *left {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            (Endpoint::Closed { p: p1, left: left1 }, Endpoint::Open { p: p2, left: left2 }) => {
                match p1.partial_cmp(p2) {
                    Some(Ordering::Equal) => match (left1, left2) {
                        (false | true, true) => Some(Ordering::Less),
                        (true | false, false) => Some(Ordering::Greater),
                    },
                    x => x,
                }
            }
            (Endpoint::Closed { p: p1, .. }, Endpoint::Closed { p: p2, .. }) => p1.partial_cmp(p2),
            (Endpoint::Unbounded { left }, Endpoint::Open { .. } | Endpoint::Closed { .. }) => {
                if *left {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
            (Endpoint::Unbounded { left: left1 }, Endpoint::Unbounded { left: left2 }) => {
                left2.partial_cmp(left1)
            }
        }
    }
}

impl<U, T: Add<U, Output = T>> Add<U> for Endpoint<T> {
    type Output = Endpoint<T>;

    fn add(self, other: U) -> Self::Output {
        match self {
            Endpoint::Open { p, left } => Endpoint::Open { p: p + other, left },
            Endpoint::Closed { p, left } => Endpoint::Closed { p: p + other, left },
            Endpoint::Unbounded { left } => Endpoint::Unbounded { left },
        }
    }
}

impl<U, T: AddAssign<U>> AddAssign<U> for Endpoint<T> {
    fn add_assign(&mut self, other: U) {
        match self {
            Endpoint::Closed { p, .. } | Endpoint::Open { p, .. } => *p += other,
            Endpoint::Unbounded { .. } => {}
        }
    }
}

impl<U, T: Sub<U, Output = T>> Sub<U> for Endpoint<T> {
    type Output = Endpoint<T>;

    fn sub(self, other: U) -> Self::Output {
        match self {
            Endpoint::Open { p, left } => Endpoint::Open { p: p - other, left },
            Endpoint::Closed { p, left } => Endpoint::Closed { p: p - other, left },
            Endpoint::Unbounded { left } => Endpoint::Unbounded { left },
        }
    }
}

impl<U, T: SubAssign<U>> SubAssign<U> for Endpoint<T> {
    fn sub_assign(&mut self, other: U) {
        match self {
            Endpoint::Closed { p, .. } | Endpoint::Open { p, .. } => *p -= other,
            Endpoint::Unbounded { .. } => {}
        }
    }
}

pub trait EndpointConversion {
    fn to_open(&self, left: bool) -> Option<Self>
    where
        Self: Sized;
    fn to_closed(&self, left: bool) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! endpoint_int_ops {
    ($($t:ty),*) => ($(
        impl EndpointConversion for $t {
            fn to_open(&self, left: bool) -> Option<Self> {
                if left {
                    self.checked_sub(1)
                } else {
                    self.checked_add(1)
                }
            }

            fn to_closed(&self, left: bool) -> Option<Self> {
                if left {
                    self.checked_add(1)
                } else {
                    self.checked_sub(1)
                }
            }
        }
    )*)
}

endpoint_int_ops!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

const ULP: Decimal = Decimal::from_parts(1, 0, 0, false, 0);

impl EndpointConversion for Decimal {
    fn to_open(&self, left: bool) -> Option<Self> {
        if left {
            self.checked_sub(ULP)
        } else {
            self.checked_add(ULP)
        }
    }

    fn to_closed(&self, left: bool) -> Option<Self> {
        if left {
            self.checked_add(ULP)
        } else {
            self.checked_sub(ULP)
        }
    }
}

impl<T: EndpointConversion + Copy> Endpoint<T> {
    #[must_use]
    pub fn to_open(&self) -> Option<T> {
        match self {
            Endpoint::Open { p, .. } => Some(*p),
            Endpoint::Closed { p, left } => T::to_open(p, *left),
            Endpoint::Unbounded { .. } => None,
        }
    }

    #[must_use]
    pub fn to_closed(&self) -> Option<T> {
        match self {
            Endpoint::Open { p, left } => T::to_closed(p, *left),
            Endpoint::Closed { p, .. } => Some(*p),
            Endpoint::Unbounded { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    #[test]
    fn endpoints() {
        let left_closed_1 = Endpoint::Closed { p: 1, left: true };
        let left_open_1 = Endpoint::Open { p: 1, left: true };
        let right_closed_1 = Endpoint::Closed { p: 1, left: false };
        let right_open_1 = Endpoint::Open { p: 1, left: false };
        let left_closed_2 = Endpoint::Closed { p: 2, left: true };
        let left_open_2 = Endpoint::Open { p: 2, left: true };
        let right_closed_2 = Endpoint::Closed { p: 2, left: false };
        let right_open_2 = Endpoint::Open { p: 2, left: false };
        let left_unbounded = Endpoint::Unbounded { left: true };
        let right_unbounded = Endpoint::Unbounded { left: false };

        // Comparison to values:
        assert_eq!(left_closed_1, 1);
        assert_ne!(left_open_1, 1);
        assert_eq!(right_closed_1, 1);
        assert_ne!(right_open_1, 1);
        assert_ne!(left_closed_1, 2);
        assert_ne!(left_open_1, 2);
        assert_ne!(right_closed_1, 2);
        assert_ne!(right_open_1, 2);

        assert!(left_closed_1 < 2);
        assert!(left_open_1 < 2);
        assert!(right_closed_1 < 2);
        assert!(right_open_1 < 2);
        assert!(left_closed_1 <= 2);
        assert!(left_open_1 <= 2);
        assert!(right_closed_1 <= 2);
        assert!(right_open_1 <= 2);

        assert!(left_closed_1 > 0);
        assert!(left_open_1 > 0);
        assert!(right_closed_1 > 0);
        assert!(right_open_1 > 0);
        assert!(left_closed_1 >= 0);
        assert!(left_open_1 >= 0);
        assert!(right_closed_1 >= 0);
        assert!(right_open_1 >= 0);

        assert!(left_open_1 > 1);
        assert!(right_open_1 < 1);
        assert!(left_open_1 >= 1);
        assert!(right_open_1 <= 1);

        assert!(left_unbounded < 1);
        assert!(left_unbounded <= 1);
        assert!(right_unbounded > 1);
        assert!(right_unbounded >= 1);

        // Comparison to other endpoints:
        assert_eq!(left_closed_1.cmp(&left_closed_1), Ordering::Equal);
        assert_eq!(left_closed_1.cmp(&left_open_1), Ordering::Less);
        assert_eq!(left_closed_1.cmp(&right_closed_1), Ordering::Equal);
        assert_eq!(left_closed_1.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(left_closed_1.cmp(&left_closed_2), Ordering::Less);
        assert_eq!(left_closed_1.cmp(&left_open_2), Ordering::Less);
        assert_eq!(left_closed_1.cmp(&right_closed_2), Ordering::Less);
        assert_eq!(left_closed_1.cmp(&right_open_2), Ordering::Less);
        assert_eq!(left_closed_1.cmp(&left_unbounded), Ordering::Greater);
        assert_eq!(left_closed_1.cmp(&right_unbounded), Ordering::Less);
        assert_eq!(left_open_1.cmp(&left_closed_1), Ordering::Greater);
        assert_eq!(left_open_1.cmp(&left_open_1), Ordering::Equal);
        assert_eq!(left_open_1.cmp(&right_closed_1), Ordering::Greater);
        assert_eq!(left_open_1.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(left_open_1.cmp(&left_closed_2), Ordering::Less);
        assert_eq!(left_open_1.cmp(&left_open_2), Ordering::Less);
        assert_eq!(left_open_1.cmp(&right_closed_2), Ordering::Less);
        assert_eq!(left_open_1.cmp(&right_open_2), Ordering::Less);
        assert_eq!(left_open_1.cmp(&left_unbounded), Ordering::Greater);
        assert_eq!(left_open_1.cmp(&right_unbounded), Ordering::Less);
        assert_eq!(right_closed_1.cmp(&left_closed_1), Ordering::Equal);
        assert_eq!(right_closed_1.cmp(&left_open_1), Ordering::Less);
        assert_eq!(right_closed_1.cmp(&right_closed_1), Ordering::Equal);
        assert_eq!(right_closed_1.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(right_closed_1.cmp(&left_closed_2), Ordering::Less);
        assert_eq!(right_closed_1.cmp(&left_open_2), Ordering::Less);
        assert_eq!(right_closed_1.cmp(&right_closed_2), Ordering::Less);
        assert_eq!(right_closed_1.cmp(&right_open_2), Ordering::Less);
        assert_eq!(right_closed_1.cmp(&left_unbounded), Ordering::Greater);
        assert_eq!(right_closed_1.cmp(&right_unbounded), Ordering::Less);
        assert_eq!(right_open_1.cmp(&left_closed_1), Ordering::Less);
        assert_eq!(right_open_1.cmp(&left_open_1), Ordering::Less);
        assert_eq!(right_open_1.cmp(&right_closed_1), Ordering::Less);
        assert_eq!(right_open_1.cmp(&right_open_1), Ordering::Equal);
        assert_eq!(right_open_1.cmp(&left_closed_2), Ordering::Less);
        assert_eq!(right_open_1.cmp(&left_open_2), Ordering::Less);
        assert_eq!(right_open_1.cmp(&right_closed_2), Ordering::Less);
        assert_eq!(right_open_1.cmp(&right_open_2), Ordering::Less);
        assert_eq!(right_open_1.cmp(&left_unbounded), Ordering::Greater);
        assert_eq!(right_open_1.cmp(&right_unbounded), Ordering::Less);
        assert_eq!(left_closed_2.cmp(&left_closed_1), Ordering::Greater);
        assert_eq!(left_closed_2.cmp(&left_open_1), Ordering::Greater);
        assert_eq!(left_closed_2.cmp(&right_closed_1), Ordering::Greater);
        assert_eq!(left_closed_2.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(left_closed_2.cmp(&left_closed_2), Ordering::Equal);
        assert_eq!(left_closed_2.cmp(&left_open_2), Ordering::Less);
        assert_eq!(left_closed_2.cmp(&right_closed_2), Ordering::Equal);
        assert_eq!(left_closed_2.cmp(&right_open_2), Ordering::Greater);
        assert_eq!(left_closed_2.cmp(&left_unbounded), Ordering::Greater);
        assert_eq!(left_closed_2.cmp(&right_unbounded), Ordering::Less);
        assert_eq!(left_open_2.cmp(&left_closed_1), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&left_open_1), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&right_closed_1), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&left_closed_2), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&left_open_2), Ordering::Equal);
        assert_eq!(left_open_2.cmp(&right_closed_2), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&right_open_2), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&left_unbounded), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&right_unbounded), Ordering::Less);
        assert_eq!(right_closed_2.cmp(&left_closed_1), Ordering::Greater);
        assert_eq!(right_closed_2.cmp(&left_open_1), Ordering::Greater);
        assert_eq!(right_closed_2.cmp(&right_closed_1), Ordering::Greater);
        assert_eq!(right_closed_2.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(right_closed_2.cmp(&left_closed_2), Ordering::Equal);
        assert_eq!(right_closed_2.cmp(&left_open_2), Ordering::Less);
        assert_eq!(right_closed_2.cmp(&right_closed_2), Ordering::Equal);
        assert_eq!(right_closed_2.cmp(&right_open_2), Ordering::Greater);
        assert_eq!(right_closed_2.cmp(&left_unbounded), Ordering::Greater);
        assert_eq!(right_closed_2.cmp(&right_unbounded), Ordering::Less);
        assert_eq!(right_open_2.cmp(&left_closed_1), Ordering::Greater);
        assert_eq!(right_open_2.cmp(&left_open_1), Ordering::Greater);
        assert_eq!(right_open_2.cmp(&right_closed_1), Ordering::Greater);
        assert_eq!(right_open_2.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(right_open_2.cmp(&left_closed_2), Ordering::Less);
        assert_eq!(right_open_2.cmp(&left_open_2), Ordering::Less);
        assert_eq!(right_open_2.cmp(&right_closed_2), Ordering::Less);
        assert_eq!(right_open_2.cmp(&right_open_2), Ordering::Equal);
        assert_eq!(right_open_2.cmp(&left_unbounded), Ordering::Greater);
        assert_eq!(right_open_2.cmp(&right_unbounded), Ordering::Less);
        assert_eq!(left_unbounded.cmp(&left_closed_1), Ordering::Less);
        assert_eq!(left_unbounded.cmp(&left_open_1), Ordering::Less);
        assert_eq!(left_unbounded.cmp(&right_closed_1), Ordering::Less);
        assert_eq!(left_unbounded.cmp(&right_open_1), Ordering::Less);
        assert_eq!(left_unbounded.cmp(&left_closed_2), Ordering::Less);
        assert_eq!(left_unbounded.cmp(&left_open_2), Ordering::Less);
        assert_eq!(left_unbounded.cmp(&right_closed_2), Ordering::Less);
        assert_eq!(left_unbounded.cmp(&right_open_2), Ordering::Less);
        assert_eq!(left_unbounded.cmp(&left_unbounded), Ordering::Equal);
        assert_eq!(left_unbounded.cmp(&right_unbounded), Ordering::Less);
        assert_eq!(right_unbounded.cmp(&left_closed_1), Ordering::Greater);
        assert_eq!(right_unbounded.cmp(&left_open_1), Ordering::Greater);
        assert_eq!(right_unbounded.cmp(&right_closed_1), Ordering::Greater);
        assert_eq!(right_unbounded.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(right_unbounded.cmp(&left_closed_2), Ordering::Greater);
        assert_eq!(right_unbounded.cmp(&left_open_2), Ordering::Greater);
        assert_eq!(right_unbounded.cmp(&right_closed_2), Ordering::Greater);
        assert_eq!(right_unbounded.cmp(&right_open_2), Ordering::Greater);
        assert_eq!(right_unbounded.cmp(&left_unbounded), Ordering::Greater);
        assert_eq!(right_unbounded.cmp(&right_unbounded), Ordering::Equal);
    }
}
