use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Bound, Sub, SubAssign};

use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Represents an endpoint of a span. For comparison, endpoints behave as closed
/// points - that is, an open endpoint should be compared with >= and <=.
#[must_use]
#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct Endpoint<T> {
    pub p: T,
    /// Whether this endpoint ends on the left or the right side. If true,
    /// imagine the span extending off from the left to the right.
    pub left: bool,
    /// Whether this endpoint is open or closed.
    pub closed: bool,
}

impl<T> Endpoint<T> {
    pub fn bound(&self) -> Bound<&T> {
        if self.closed {
            Bound::Included(&self.p)
        } else {
            Bound::Excluded(&self.p)
        }
    }
}

impl<T: fmt::Display> fmt::Display for Endpoint<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.left {
            if self.closed {
                write!(f, "[")?;
            } else {
                write!(f, "(")?;
            }
        }
        write!(f, "{}", self.p)?;
        if !self.left {
            if self.closed {
                write!(f, "]")?;
            } else {
                write!(f, ")")?;
            }
        }
        Ok(())
    }
}

impl<T: PartialEq> PartialEq<T> for Endpoint<T> {
    fn eq(&self, other: &T) -> bool {
        if self.closed {
            self.p == *other
        } else {
            // If open, can't be equal to any closed point.
            false
        }
    }
}

impl<T: PartialOrd> PartialOrd<T> for Endpoint<T> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        match self.p.partial_cmp(other) {
            Some(Ordering::Equal) => {
                if self.closed {
                    Some(Ordering::Equal)
                } else if self.left {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            x => x,
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
        match self.p.partial_cmp(&other.p) {
            Some(Ordering::Equal) => match (self.left, other.left) {
                (true, true) => match (self.closed, other.closed) {
                    (true, true) | (false, false) => Some(Ordering::Equal),
                    (true, false) => Some(Ordering::Less),
                    (false, true) => Some(Ordering::Greater),
                },
                (true, false) => match (self.closed, other.closed) {
                    (true, true) => Some(Ordering::Equal),
                    _ => Some(Ordering::Greater),
                },
                (false, true) => match (self.closed, other.closed) {
                    (true, true) => Some(Ordering::Equal),
                    _ => Some(Ordering::Less),
                },
                (false, false) => match (self.closed, other.closed) {
                    (true, true) | (false, false) => Some(Ordering::Equal),
                    (true, false) => Some(Ordering::Greater),
                    (false, true) => Some(Ordering::Less),
                },
            },
            x => x,
        }
    }
}

impl<U, T: Add<U, Output = T>> Add<U> for Endpoint<T> {
    type Output = Endpoint<T>;

    fn add(self, other: U) -> Self::Output {
        Endpoint { p: self.p + other, left: self.left, closed: self.closed }
    }
}

impl<U, T: AddAssign<U>> AddAssign<U> for Endpoint<T> {
    fn add_assign(&mut self, other: U) {
        self.p += other;
    }
}

impl<U, T: Sub<U, Output = T>> Sub<U> for Endpoint<T> {
    type Output = Endpoint<T>;

    fn sub(self, other: U) -> Self::Output {
        Endpoint { p: self.p - other, left: self.left, closed: self.closed }
    }
}

impl<U, T: SubAssign<U>> SubAssign<U> for Endpoint<T> {
    fn sub_assign(&mut self, other: U) {
        self.p -= other;
    }
}

#[must_use]
#[derive(
    Debug,
    Default,
    Display,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Copy,
    Clone,
    Serialize,
    Deserialize,
)]
#[display(fmt = "{st},{en}")]
pub struct Span<T: std::fmt::Display> {
    pub st: Endpoint<T>,
    pub en: Endpoint<T>,
}

/// Returns |a| if |b| is not comparable.
pub fn pmin<X: PartialOrd + Copy>(a: X, b: X) -> X {
    if b < a {
        b
    } else {
        a
    }
}

/// Returns |a| if |b| is not comparable.
pub fn pmax<X: PartialOrd + Copy + fmt::Display>(a: X, b: X) -> X {
    if b > a {
        b
    } else {
        a
    }
}

impl<T: PartialOrd + Copy + fmt::Display> Span<T> {
    pub const fn new(st: Endpoint<T>, en: Endpoint<T>) -> Span<T> {
        Span { st, en }
    }

    /// Exclusive-exclusive.
    pub fn exc_exc(st: impl Into<T>, en: impl Into<T>) -> Self {
        Self {
            st: Endpoint { p: st.into(), left: true, closed: false },
            en: Endpoint { p: en.into(), left: false, closed: false },
        }
    }

    /// Inclusive-exclusive.
    pub fn exc(st: impl Into<T>, en: impl Into<T>) -> Self {
        Self {
            st: Endpoint { p: st.into(), left: true, closed: true },
            en: Endpoint { p: en.into(), left: false, closed: false },
        }
    }

    /// Inclusive-inclusive.
    pub fn inc(st: impl Into<T>, en: impl Into<T>) -> Self {
        Self {
            st: Endpoint { p: st.into(), left: true, closed: true },
            en: Endpoint { p: en.into(), left: false, closed: true },
        }
    }

    pub fn point(p: impl Into<T>) -> Self {
        let p = p.into();
        Self {
            st: Endpoint { p, left: true, closed: true },
            en: Endpoint { p, left: false, closed: true },
        }
    }

    pub fn cover(a: &Self, b: &Self) -> Self {
        if a.is_empty() {
            *b
        } else if b.is_empty() {
            *a
        } else {
            Span::new(pmin(a.st, b.st), pmax(a.en, b.en))
        }
    }

    pub fn contains(&self, t: T) -> bool {
        self.st <= t && self.en >= t
    }

    pub fn contains_span(&self, s: &Self) -> bool {
        self.st <= s.st && self.en >= s.en
    }

    pub fn is_empty(&self) -> bool {
        // [x, x) treated as empty.
        self.st.p > self.en.p || (self.st.p == self.en.p && (!self.st.closed || !self.en.closed))
    }

    pub fn intersect(&self, s: &Self) -> Option<Self> {
        let span = Span::new(pmax(self.st, s.st), pmin(self.en, s.en));
        if span.is_empty() {
            None
        } else {
            Some(span)
        }
    }

    pub fn range_ref(&self) -> (Bound<&T>, Bound<&T>) {
        (self.st.bound(), self.en.bound())
    }

    pub fn range(&self) -> (Bound<T>, Bound<T>) {
        (self.st.bound().cloned(), self.en.bound().cloned())
    }
}

impl<T: PartialOrd + Copy + Default + fmt::Display> Span<T> {
    pub fn empty() -> Self {
        Self::exc_exc(T::default(), T::default())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    #[test]
    fn endpoints() {
        let left_closed_1 = Endpoint { p: 1, left: true, closed: true };
        let left_open_1 = Endpoint { p: 1, left: true, closed: false };
        let right_closed_1 = Endpoint { p: 1, left: false, closed: true };
        let right_open_1 = Endpoint { p: 1, left: false, closed: false };
        let left_closed_2 = Endpoint { p: 2, left: true, closed: true };
        let left_open_2 = Endpoint { p: 2, left: true, closed: false };
        let right_closed_2 = Endpoint { p: 2, left: false, closed: true };
        let right_open_2 = Endpoint { p: 2, left: false, closed: false };

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

        // Comparison to other endpoints:
        assert_eq!(left_closed_1.cmp(&left_closed_1), Ordering::Equal);
        assert_eq!(left_closed_1.cmp(&left_open_1), Ordering::Less);
        assert_eq!(left_closed_1.cmp(&right_closed_1), Ordering::Equal);
        assert_eq!(left_closed_1.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(left_closed_1.cmp(&left_closed_2), Ordering::Less);
        assert_eq!(left_closed_1.cmp(&left_open_2), Ordering::Less);
        assert_eq!(left_closed_1.cmp(&right_closed_2), Ordering::Less);
        assert_eq!(left_closed_1.cmp(&right_open_2), Ordering::Less);
        assert_eq!(left_open_1.cmp(&left_closed_1), Ordering::Greater);
        assert_eq!(left_open_1.cmp(&left_open_1), Ordering::Equal);
        assert_eq!(left_open_1.cmp(&right_closed_1), Ordering::Greater);
        assert_eq!(left_open_1.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(left_open_1.cmp(&left_closed_2), Ordering::Less);
        assert_eq!(left_open_1.cmp(&left_open_2), Ordering::Less);
        assert_eq!(left_open_1.cmp(&right_closed_2), Ordering::Less);
        assert_eq!(left_open_1.cmp(&right_open_2), Ordering::Less);
        assert_eq!(right_closed_1.cmp(&left_closed_1), Ordering::Equal);
        assert_eq!(right_closed_1.cmp(&left_open_1), Ordering::Less);
        assert_eq!(right_closed_1.cmp(&right_closed_1), Ordering::Equal);
        assert_eq!(right_closed_1.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(right_closed_1.cmp(&left_closed_2), Ordering::Less);
        assert_eq!(right_closed_1.cmp(&left_open_2), Ordering::Less);
        assert_eq!(right_closed_1.cmp(&right_closed_2), Ordering::Less);
        assert_eq!(right_closed_1.cmp(&right_open_2), Ordering::Less);
        assert_eq!(right_open_1.cmp(&left_closed_1), Ordering::Less);
        assert_eq!(right_open_1.cmp(&left_open_1), Ordering::Less);
        assert_eq!(right_open_1.cmp(&right_closed_1), Ordering::Less);
        assert_eq!(right_open_1.cmp(&right_open_1), Ordering::Equal);
        assert_eq!(right_open_1.cmp(&left_closed_2), Ordering::Less);
        assert_eq!(right_open_1.cmp(&left_open_2), Ordering::Less);
        assert_eq!(right_open_1.cmp(&right_closed_2), Ordering::Less);
        assert_eq!(right_open_1.cmp(&right_open_2), Ordering::Less);
        assert_eq!(left_closed_2.cmp(&left_closed_1), Ordering::Greater);
        assert_eq!(left_closed_2.cmp(&left_open_1), Ordering::Greater);
        assert_eq!(left_closed_2.cmp(&right_closed_1), Ordering::Greater);
        assert_eq!(left_closed_2.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(left_closed_2.cmp(&left_closed_2), Ordering::Equal);
        assert_eq!(left_closed_2.cmp(&left_open_2), Ordering::Less);
        assert_eq!(left_closed_2.cmp(&right_closed_2), Ordering::Equal);
        assert_eq!(left_closed_2.cmp(&right_open_2), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&left_closed_1), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&left_open_1), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&right_closed_1), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&left_closed_2), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&left_open_2), Ordering::Equal);
        assert_eq!(left_open_2.cmp(&right_closed_2), Ordering::Greater);
        assert_eq!(left_open_2.cmp(&right_open_2), Ordering::Greater);
        assert_eq!(right_closed_2.cmp(&left_closed_1), Ordering::Greater);
        assert_eq!(right_closed_2.cmp(&left_open_1), Ordering::Greater);
        assert_eq!(right_closed_2.cmp(&right_closed_1), Ordering::Greater);
        assert_eq!(right_closed_2.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(right_closed_2.cmp(&left_closed_2), Ordering::Equal);
        assert_eq!(right_closed_2.cmp(&left_open_2), Ordering::Less);
        assert_eq!(right_closed_2.cmp(&right_closed_2), Ordering::Equal);
        assert_eq!(right_closed_2.cmp(&right_open_2), Ordering::Greater);
        assert_eq!(right_open_2.cmp(&left_closed_1), Ordering::Greater);
        assert_eq!(right_open_2.cmp(&left_open_1), Ordering::Greater);
        assert_eq!(right_open_2.cmp(&right_closed_1), Ordering::Greater);
        assert_eq!(right_open_2.cmp(&right_open_1), Ordering::Greater);
        assert_eq!(right_open_2.cmp(&left_closed_2), Ordering::Less);
        assert_eq!(right_open_2.cmp(&left_open_2), Ordering::Less);
        assert_eq!(right_open_2.cmp(&right_closed_2), Ordering::Less);
        assert_eq!(right_open_2.cmp(&right_open_2), Ordering::Equal);
    }

    #[test]
    fn ops() {
        let exc_0_2 = Span::<i64>::exc(0, 2);
        let exc_1_3 = Span::<i64>::exc(1, 3);
        let exc_2_4 = Span::<i64>::exc(2, 4);
        let exc_3_5 = Span::<i64>::exc(3, 5);
        let inc_0_2 = Span::<i64>::inc(0, 2);
        let inc_1_3 = Span::<i64>::inc(1, 3);
        let inc_2_4 = Span::<i64>::inc(2, 4);
        let inc_3_5 = Span::<i64>::inc(3, 5);
        let empty = Span::<i64>::empty();

        // intersect:
        assert_eq!(exc_0_2.intersect(&exc_0_2), Some(Span::exc(0, 2)));
        assert_eq!(exc_0_2.intersect(&exc_1_3), Some(Span::exc(1, 2)));
        assert_eq!(exc_0_2.intersect(&exc_2_4), None);
        assert_eq!(exc_0_2.intersect(&exc_3_5), None);
        assert_eq!(exc_0_2.intersect(&inc_0_2), Some(Span::exc(0, 2)));
        assert_eq!(exc_0_2.intersect(&inc_1_3), Some(Span::exc(1, 2)));
        assert_eq!(exc_0_2.intersect(&inc_2_4), None);
        assert_eq!(exc_0_2.intersect(&inc_3_5), None);
        assert_eq!(exc_0_2.intersect(&empty), None);
        assert_eq!(exc_1_3.intersect(&exc_0_2), Some(Span::exc(1, 2)));
        assert_eq!(exc_1_3.intersect(&exc_1_3), Some(Span::exc(1, 3)));
        assert_eq!(exc_1_3.intersect(&exc_2_4), Some(Span::exc(2, 3)));
        assert_eq!(exc_1_3.intersect(&exc_3_5), None);
        assert_eq!(exc_1_3.intersect(&inc_0_2), Some(Span::inc(1, 2)));
        assert_eq!(exc_1_3.intersect(&inc_1_3), Some(Span::exc(1, 3)));
        assert_eq!(exc_1_3.intersect(&inc_2_4), Some(Span::exc(2, 3)));
        assert_eq!(exc_1_3.intersect(&inc_3_5), None);
        assert_eq!(exc_1_3.intersect(&empty), None);
        assert_eq!(exc_2_4.intersect(&exc_0_2), None);
        assert_eq!(exc_2_4.intersect(&exc_1_3), Some(Span::exc(2, 3)));
        assert_eq!(exc_2_4.intersect(&exc_2_4), Some(Span::exc(2, 4)));
        assert_eq!(exc_2_4.intersect(&exc_3_5), Some(Span::exc(3, 4)));
        assert_eq!(exc_2_4.intersect(&inc_0_2), Some(Span::point(2)));
        assert_eq!(exc_2_4.intersect(&inc_1_3), Some(Span::inc(2, 3)));
        assert_eq!(exc_2_4.intersect(&inc_2_4), Some(Span::exc(2, 4)));
        assert_eq!(exc_2_4.intersect(&inc_3_5), Some(Span::exc(3, 4)));
        assert_eq!(exc_2_4.intersect(&empty), None);
        assert_eq!(exc_3_5.intersect(&exc_0_2), None);
        assert_eq!(exc_3_5.intersect(&exc_1_3), None);
        assert_eq!(exc_3_5.intersect(&exc_2_4), Some(Span::exc(3, 4)));
        assert_eq!(exc_3_5.intersect(&exc_3_5), Some(Span::exc(3, 5)));
        assert_eq!(exc_3_5.intersect(&inc_0_2), None);
        assert_eq!(exc_3_5.intersect(&inc_1_3), Some(Span::point(3)));
        assert_eq!(exc_3_5.intersect(&inc_2_4), Some(Span::inc(3, 4)));
        assert_eq!(exc_3_5.intersect(&inc_3_5), Some(Span::exc(3, 5)));
        assert_eq!(exc_3_5.intersect(&empty), None);
        assert_eq!(inc_0_2.intersect(&exc_0_2), Some(Span::exc(0, 2)));
        assert_eq!(inc_0_2.intersect(&exc_1_3), Some(Span::inc(1, 2)));
        assert_eq!(inc_0_2.intersect(&exc_2_4), Some(Span::point(2)));
        assert_eq!(inc_0_2.intersect(&exc_3_5), None);
        assert_eq!(inc_0_2.intersect(&inc_0_2), Some(Span::inc(0, 2)));
        assert_eq!(inc_0_2.intersect(&inc_1_3), Some(Span::inc(1, 2)));
        assert_eq!(inc_0_2.intersect(&inc_2_4), Some(Span::point(2)));
        assert_eq!(inc_0_2.intersect(&inc_3_5), None);
        assert_eq!(inc_0_2.intersect(&empty), None);
        assert_eq!(inc_1_3.intersect(&exc_0_2), Some(Span::exc(1, 2)));
        assert_eq!(inc_1_3.intersect(&exc_1_3), Some(Span::exc(1, 3)));
        assert_eq!(inc_1_3.intersect(&exc_2_4), Some(Span::inc(2, 3)));
        assert_eq!(inc_1_3.intersect(&exc_3_5), Some(Span::point(3)));
        assert_eq!(inc_1_3.intersect(&inc_0_2), Some(Span::inc(1, 2)));
        assert_eq!(inc_1_3.intersect(&inc_1_3), Some(Span::inc(1, 3)));
        assert_eq!(inc_1_3.intersect(&inc_2_4), Some(Span::inc(2, 3)));
        assert_eq!(inc_1_3.intersect(&inc_3_5), Some(Span::point(3)));
        assert_eq!(inc_1_3.intersect(&empty), None);
        assert_eq!(inc_2_4.intersect(&exc_0_2), None);
        assert_eq!(inc_2_4.intersect(&exc_1_3), Some(Span::exc(2, 3)));
        assert_eq!(inc_2_4.intersect(&exc_2_4), Some(Span::exc(2, 4)));
        assert_eq!(inc_2_4.intersect(&exc_3_5), Some(Span::inc(3, 4)));
        assert_eq!(inc_2_4.intersect(&inc_0_2), Some(Span::point(2)));
        assert_eq!(inc_2_4.intersect(&inc_1_3), Some(Span::inc(2, 3)));
        assert_eq!(inc_2_4.intersect(&inc_2_4), Some(Span::inc(2, 4)));
        assert_eq!(inc_2_4.intersect(&inc_3_5), Some(Span::inc(3, 4)));
        assert_eq!(inc_2_4.intersect(&empty), None);
        assert_eq!(inc_3_5.intersect(&exc_0_2), None);
        assert_eq!(inc_3_5.intersect(&exc_1_3), None);
        assert_eq!(inc_3_5.intersect(&exc_2_4), Some(Span::exc(3, 4)));
        assert_eq!(inc_3_5.intersect(&exc_3_5), Some(Span::exc(3, 5)));
        assert_eq!(inc_3_5.intersect(&inc_0_2), None);
        assert_eq!(inc_3_5.intersect(&inc_1_3), Some(Span::point(3)));
        assert_eq!(inc_3_5.intersect(&inc_2_4), Some(Span::inc(3, 4)));
        assert_eq!(inc_3_5.intersect(&inc_3_5), Some(Span::inc(3, 5)));
        assert_eq!(inc_3_5.intersect(&empty), None);

        // contains:
        assert!(!exc_0_2.contains(-1));
        assert!(exc_0_2.contains(0));
        assert!(exc_0_2.contains(1));
        assert!(!exc_0_2.contains(2));
        assert!(!exc_0_2.contains(3));
        assert!(!inc_0_2.contains(-1));
        assert!(inc_0_2.contains(0));
        assert!(inc_0_2.contains(1));
        assert!(inc_0_2.contains(2));
        assert!(!inc_0_2.contains(3));

        // contains_span:
        assert!(exc_0_2.contains_span(&exc_0_2));
        assert!(!exc_0_2.contains_span(&exc_1_3));
        assert!(inc_0_2.contains_span(&inc_0_2));
        assert!(inc_0_2.contains_span(&exc_0_2));
        assert!(!exc_0_2.contains_span(&inc_0_2));

        // cover:
        assert_eq!(Span::cover(&exc_0_2, &exc_0_2), Span::exc(0, 2));
        assert_eq!(Span::cover(&exc_0_2, &exc_1_3), Span::exc(0, 3));
        assert_eq!(Span::cover(&exc_0_2, &exc_2_4), Span::exc(0, 4));
        assert_eq!(Span::cover(&exc_0_2, &exc_3_5), Span::exc(0, 5));
        assert_eq!(Span::cover(&exc_0_2, &inc_0_2), Span::inc(0, 2));
        assert_eq!(Span::cover(&exc_0_2, &inc_1_3), Span::inc(0, 3));
        assert_eq!(Span::cover(&exc_0_2, &inc_2_4), Span::inc(0, 4));
        assert_eq!(Span::cover(&exc_0_2, &inc_3_5), Span::inc(0, 5));
        assert_eq!(Span::cover(&exc_0_2, &empty), Span::exc(0, 2));

        assert_eq!(Span::cover(&exc_1_3, &exc_0_2), Span::exc(0, 3));
        assert_eq!(Span::cover(&exc_1_3, &exc_1_3), Span::exc(1, 3));
        assert_eq!(Span::cover(&exc_1_3, &exc_2_4), Span::exc(1, 4));
        assert_eq!(Span::cover(&exc_1_3, &exc_3_5), Span::exc(1, 5));
        assert_eq!(Span::cover(&exc_1_3, &inc_0_2), Span::exc(0, 3));
        assert_eq!(Span::cover(&exc_1_3, &inc_1_3), Span::inc(1, 3));
        assert_eq!(Span::cover(&exc_1_3, &inc_2_4), Span::inc(1, 4));
        assert_eq!(Span::cover(&exc_1_3, &inc_3_5), Span::inc(1, 5));
        assert_eq!(Span::cover(&exc_1_3, &empty), Span::exc(1, 3));

        assert_eq!(Span::cover(&exc_2_4, &exc_0_2), Span::exc(0, 4));
        assert_eq!(Span::cover(&exc_2_4, &exc_1_3), Span::exc(1, 4));
        assert_eq!(Span::cover(&exc_2_4, &exc_2_4), Span::exc(2, 4));
        assert_eq!(Span::cover(&exc_2_4, &exc_3_5), Span::exc(2, 5));
        assert_eq!(Span::cover(&exc_2_4, &inc_0_2), Span::exc(0, 4));
        assert_eq!(Span::cover(&exc_2_4, &inc_1_3), Span::exc(1, 4));
        assert_eq!(Span::cover(&exc_2_4, &inc_2_4), Span::inc(2, 4));
        assert_eq!(Span::cover(&exc_2_4, &inc_3_5), Span::inc(2, 5));
        assert_eq!(Span::cover(&exc_2_4, &empty), Span::exc(2, 4));

        assert_eq!(Span::cover(&exc_3_5, &exc_0_2), Span::exc(0, 5));
        assert_eq!(Span::cover(&exc_3_5, &exc_1_3), Span::exc(1, 5));
        assert_eq!(Span::cover(&exc_3_5, &exc_2_4), Span::exc(2, 5));
        assert_eq!(Span::cover(&exc_3_5, &exc_3_5), Span::exc(3, 5));
        assert_eq!(Span::cover(&exc_3_5, &inc_0_2), Span::exc(0, 5));
        assert_eq!(Span::cover(&exc_3_5, &inc_1_3), Span::exc(1, 5));
        assert_eq!(Span::cover(&exc_3_5, &inc_2_4), Span::exc(2, 5));
        assert_eq!(Span::cover(&exc_3_5, &inc_3_5), Span::inc(3, 5));
        assert_eq!(Span::cover(&exc_3_5, &empty), Span::exc(3, 5));

        assert_eq!(Span::cover(&inc_0_2, &exc_0_2), Span::inc(0, 2));
        assert_eq!(Span::cover(&inc_0_2, &exc_1_3), Span::exc(0, 3));
        assert_eq!(Span::cover(&inc_0_2, &exc_2_4), Span::exc(0, 4));
        assert_eq!(Span::cover(&inc_0_2, &exc_3_5), Span::exc(0, 5));
        assert_eq!(Span::cover(&inc_0_2, &inc_0_2), Span::inc(0, 2));
        assert_eq!(Span::cover(&inc_0_2, &inc_1_3), Span::inc(0, 3));
        assert_eq!(Span::cover(&inc_0_2, &inc_2_4), Span::inc(0, 4));
        assert_eq!(Span::cover(&inc_0_2, &inc_3_5), Span::inc(0, 5));
        assert_eq!(Span::cover(&inc_0_2, &empty), Span::inc(0, 2));

        assert_eq!(Span::cover(&inc_1_3, &exc_0_2), Span::inc(0, 3));
        assert_eq!(Span::cover(&inc_1_3, &exc_1_3), Span::inc(1, 3));
        assert_eq!(Span::cover(&inc_1_3, &exc_2_4), Span::exc(1, 4));
        assert_eq!(Span::cover(&inc_1_3, &exc_3_5), Span::exc(1, 5));
        assert_eq!(Span::cover(&inc_1_3, &inc_0_2), Span::inc(0, 3));
        assert_eq!(Span::cover(&inc_1_3, &inc_1_3), Span::inc(1, 3));
        assert_eq!(Span::cover(&inc_1_3, &inc_2_4), Span::inc(1, 4));
        assert_eq!(Span::cover(&inc_1_3, &inc_3_5), Span::inc(1, 5));
        assert_eq!(Span::cover(&inc_1_3, &empty), Span::inc(1, 3));

        assert_eq!(Span::cover(&inc_2_4, &exc_0_2), Span::inc(0, 4));
        assert_eq!(Span::cover(&inc_2_4, &exc_1_3), Span::inc(1, 4));
        assert_eq!(Span::cover(&inc_2_4, &exc_2_4), Span::inc(2, 4));
        assert_eq!(Span::cover(&inc_2_4, &exc_3_5), Span::exc(2, 5));
        assert_eq!(Span::cover(&inc_2_4, &inc_0_2), Span::inc(0, 4));
        assert_eq!(Span::cover(&inc_2_4, &inc_1_3), Span::inc(1, 4));
        assert_eq!(Span::cover(&inc_2_4, &inc_2_4), Span::inc(2, 4));
        assert_eq!(Span::cover(&inc_2_4, &inc_3_5), Span::inc(2, 5));
        assert_eq!(Span::cover(&inc_2_4, &empty), Span::inc(2, 4));

        assert_eq!(Span::cover(&inc_3_5, &exc_0_2), Span::inc(0, 5));
        assert_eq!(Span::cover(&inc_3_5, &exc_1_3), Span::inc(1, 5));
        assert_eq!(Span::cover(&inc_3_5, &exc_2_4), Span::inc(2, 5));
        assert_eq!(Span::cover(&inc_3_5, &exc_3_5), Span::inc(3, 5));
        assert_eq!(Span::cover(&inc_3_5, &inc_0_2), Span::inc(0, 5));
        assert_eq!(Span::cover(&inc_3_5, &inc_1_3), Span::inc(1, 5));
        assert_eq!(Span::cover(&inc_3_5, &inc_2_4), Span::inc(2, 5));
        assert_eq!(Span::cover(&inc_3_5, &inc_3_5), Span::inc(3, 5));
        assert_eq!(Span::cover(&inc_3_5, &empty), Span::inc(3, 5));

        assert_eq!(Span::cover(&empty, &exc_0_2), Span::exc(0, 2));
        assert_eq!(Span::cover(&empty, &exc_1_3), Span::exc(1, 3));
        assert_eq!(Span::cover(&empty, &exc_2_4), Span::exc(2, 4));
        assert_eq!(Span::cover(&empty, &exc_3_5), Span::exc(3, 5));
        assert_eq!(Span::cover(&empty, &inc_0_2), Span::inc(0, 2));
        assert_eq!(Span::cover(&empty, &inc_1_3), Span::inc(1, 3));
        assert_eq!(Span::cover(&empty, &inc_2_4), Span::inc(2, 4));
        assert_eq!(Span::cover(&empty, &inc_3_5), Span::inc(3, 5));
        assert_eq!(Span::cover(&empty, &empty), Span::empty());
    }
}
