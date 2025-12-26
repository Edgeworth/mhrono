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
    /// |side| is whether this endpoint ends on the left or the right side.
    Open {
        p: T,
        side: EndpointSide,
    },
    Closed {
        p: T,
        side: EndpointSide,
    },
    Unbounded {
        side: EndpointSide,
    },
}

impl<T> Endpoint<T> {
    pub fn from_bound(bound: Bound<T>, side: EndpointSide) -> Self {
        match bound {
            Bound::Included(p) => Self::Closed { p, side },
            Bound::Excluded(p) => Self::Open { p, side },
            Bound::Unbounded => Self::Unbounded { side },
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
            Endpoint::Open { side, .. }
            | Endpoint::Closed { side, .. }
            | Endpoint::Unbounded { side } => matches!(side, EndpointSide::Left),
        }
    }

    pub const fn is_right(&self) -> bool {
        !self.is_left()
    }

    pub const fn is_left_unbounded(&self) -> bool {
        match self {
            Endpoint::Unbounded { side } => matches!(side, EndpointSide::Left),
            _ => false,
        }
    }

    pub const fn is_right_unbounded(&self) -> bool {
        match self {
            Endpoint::Unbounded { side } => matches!(side, EndpointSide::Right),
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
            Endpoint::Open { p, side } => match side {
                EndpointSide::Left => write!(f, "({p}"),
                EndpointSide::Right => write!(f, "{p})"),
            },
            Endpoint::Closed { p, side } => match side {
                EndpointSide::Left => write!(f, "[{p}"),
                EndpointSide::Right => write!(f, "{p}]"),
            },
            Endpoint::Unbounded { side } => match side {
                EndpointSide::Left => write!(f, "(-inf"),
                EndpointSide::Right => write!(f, "+inf)"),
            },
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
            Endpoint::Open { p, side } => match p.partial_cmp(other) {
                Some(Ordering::Equal) => Some(match side {
                    EndpointSide::Left => Ordering::Greater,
                    EndpointSide::Right => Ordering::Less,
                }),
                x => x,
            },
            Endpoint::Closed { p, .. } => p.partial_cmp(other),
            Endpoint::Unbounded { side } => Some(match side {
                EndpointSide::Left => Ordering::Less,
                EndpointSide::Right => Ordering::Greater,
            }),
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
        use EndpointSide::{Left, Right};

        match (self, other) {
            (Endpoint::Open { p: p1, side: s1 }, Endpoint::Open { p: p2, side: s2 }) => {
                match p1.partial_cmp(p2) {
                    Some(Ordering::Equal) => Some(match (s1, s2) {
                        (Left, Left) | (Right, Right) => Ordering::Equal,
                        (Left, Right) => Ordering::Greater,
                        (Right, Left) => Ordering::Less,
                    }),
                    x => x,
                }
            }
            (Endpoint::Open { p: p1, side }, Endpoint::Closed { p: p2, .. }) => {
                match p1.partial_cmp(p2) {
                    Some(Ordering::Equal) => Some(match side {
                        Left => Ordering::Greater,
                        Right => Ordering::Less,
                    }),
                    x => x,
                }
            }
            (Endpoint::Open { .. } | Endpoint::Closed { .. }, Endpoint::Unbounded { side }) => {
                Some(match side {
                    Left => Ordering::Greater,
                    Right => Ordering::Less,
                })
            }
            (Endpoint::Closed { p: p1, .. }, Endpoint::Open { p: p2, side }) => {
                match p1.partial_cmp(p2) {
                    Some(Ordering::Equal) => Some(match side {
                        Left => Ordering::Less,
                        Right => Ordering::Greater,
                    }),
                    x => x,
                }
            }
            (Endpoint::Closed { p: p1, .. }, Endpoint::Closed { p: p2, .. }) => p1.partial_cmp(p2),
            (Endpoint::Unbounded { side }, Endpoint::Open { .. } | Endpoint::Closed { .. }) => {
                Some(match side {
                    Left => Ordering::Less,
                    Right => Ordering::Greater,
                })
            }
            (Endpoint::Unbounded { side: s1 }, Endpoint::Unbounded { side: s2 }) => {
                Some(match (s1, s2) {
                    (Left, Left) | (Right, Right) => Ordering::Equal,
                    (Left, Right) => Ordering::Less,
                    (Right, Left) => Ordering::Greater,
                })
            }
        }
    }
}

impl<U, T: Add<U, Output = T>> Add<U> for Endpoint<T> {
    type Output = Endpoint<T>;

    fn add(self, other: U) -> Self::Output {
        match self {
            Endpoint::Open { p, side } => Endpoint::Open { p: p + other, side },
            Endpoint::Closed { p, side } => Endpoint::Closed { p: p + other, side },
            Endpoint::Unbounded { side } => Endpoint::Unbounded { side },
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
            Endpoint::Open { p, side } => Endpoint::Open { p: p - other, side },
            Endpoint::Closed { p, side } => Endpoint::Closed { p: p - other, side },
            Endpoint::Unbounded { side } => Endpoint::Unbounded { side },
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

#[must_use]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum EndpointSide {
    Left,
    Right,
}

pub trait EndpointConversion {
    fn to_open(&self, side: EndpointSide) -> Option<Self>
    where
        Self: Sized;
    fn to_closed(&self, side: EndpointSide) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! endpoint_int_ops {
    ($($t:ty),*) => ($(
        impl EndpointConversion for $t {
            fn to_open(&self, side: EndpointSide) -> Option<Self> {
                match side {
                    EndpointSide::Left => self.checked_sub(1),
                    EndpointSide::Right => self.checked_add(1),
                }
            }

            fn to_closed(&self, side: EndpointSide) -> Option<Self> {
                match side {
                    EndpointSide::Left => self.checked_add(1),
                    EndpointSide::Right => self.checked_sub(1),
                }
            }
        }
    )*)
}

endpoint_int_ops!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

const ULP: Decimal = Decimal::from_parts(1, 0, 0, false, Decimal::MAX_SCALE);

impl EndpointConversion for Decimal {
    fn to_open(&self, side: EndpointSide) -> Option<Self> {
        match side {
            EndpointSide::Left => self.checked_sub(ULP),
            EndpointSide::Right => self.checked_add(ULP),
        }
    }

    fn to_closed(&self, side: EndpointSide) -> Option<Self> {
        match side {
            EndpointSide::Left => self.checked_add(ULP),
            EndpointSide::Right => self.checked_sub(ULP),
        }
    }
}

impl<T: EndpointConversion + Copy> Endpoint<T> {
    #[must_use]
    pub fn to_open(&self) -> Option<T> {
        match self {
            Endpoint::Open { p, .. } => Some(*p),
            Endpoint::Closed { p, side } => p.to_open(*side),
            Endpoint::Unbounded { .. } => None,
        }
    }

    #[must_use]
    pub fn to_closed(&self) -> Option<T> {
        match self {
            Endpoint::Open { p, side } => p.to_closed(*side),
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
        let left_closed_1 = Endpoint::Closed { p: 1, side: EndpointSide::Left };
        let left_open_1 = Endpoint::Open { p: 1, side: EndpointSide::Left };
        let right_closed_1 = Endpoint::Closed { p: 1, side: EndpointSide::Right };
        let right_open_1 = Endpoint::Open { p: 1, side: EndpointSide::Right };
        let left_closed_2 = Endpoint::Closed { p: 2, side: EndpointSide::Left };
        let left_open_2 = Endpoint::Open { p: 2, side: EndpointSide::Left };
        let right_closed_2 = Endpoint::Closed { p: 2, side: EndpointSide::Right };
        let right_open_2 = Endpoint::Open { p: 2, side: EndpointSide::Right };
        let left_unbounded = Endpoint::Unbounded { side: EndpointSide::Left };
        let right_unbounded = Endpoint::Unbounded { side: EndpointSide::Right };

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

    #[test]
    fn decimal_endpoint_conversion_uses_min_decimal_ulp() {
        let z = Decimal::new(0, 0);
        let ulp = Decimal::new(1, Decimal::MAX_SCALE);

        assert_eq!(z.to_open(EndpointSide::Left).unwrap(), Decimal::new(-1, 28));
        assert_eq!(z.to_open(EndpointSide::Right).unwrap(), ulp);
        assert_eq!(z.to_closed(EndpointSide::Left).unwrap(), ulp);
        assert_eq!(z.to_closed(EndpointSide::Right).unwrap(), Decimal::new(-1, 28));
    }
}
