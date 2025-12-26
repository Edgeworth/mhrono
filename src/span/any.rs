use std::fmt;
use std::ops::{
    Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive, Sub,
};

use serde::{Deserialize, Serialize};

use crate::span::endpoint::{Endpoint, EndpointConversion, EndpointSide};
use crate::span::exc::SpanExc;
use crate::span::inc::SpanInc;
use crate::span::ops::{pmax, pmin};

#[must_use]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct SpanAny<T> {
    pub st: Endpoint<T>,
    pub en: Endpoint<T>,
}

impl<T: fmt::Display> fmt::Display for SpanAny<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.st, self.en)
    }
}

impl<T: Default> SpanAny<T> {
    pub fn empty() -> Self {
        Self::exc(T::default(), T::default())
    }
}

impl<T> SpanAny<T> {
    pub const fn new(st: Endpoint<T>, en: Endpoint<T>) -> Self {
        Self { st, en }
    }

    /// Exclusive-exclusive.
    pub const fn exc_exc(st: T, en: T) -> Self {
        Self {
            st: Endpoint::Open { p: st, side: EndpointSide::Left },
            en: Endpoint::Open { p: en, side: EndpointSide::Right },
        }
    }

    /// Exclusive-inclusive.
    pub const fn exc_inc(st: T, en: T) -> Self {
        Self {
            st: Endpoint::Open { p: st, side: EndpointSide::Left },
            en: Endpoint::Closed { p: en, side: EndpointSide::Right },
        }
    }

    /// Inclusive-exclusive.
    pub const fn exc(st: T, en: T) -> Self {
        Self {
            st: Endpoint::Closed { p: st, side: EndpointSide::Left },
            en: Endpoint::Open { p: en, side: EndpointSide::Right },
        }
    }

    /// Inclusive-inclusive.
    pub const fn inc(st: T, en: T) -> Self {
        Self {
            st: Endpoint::Closed { p: st, side: EndpointSide::Left },
            en: Endpoint::Closed { p: en, side: EndpointSide::Right },
        }
    }

    pub const fn unb_exc(en: T) -> Self {
        Self {
            st: Endpoint::Unbounded { side: EndpointSide::Left },
            en: Endpoint::Open { p: en, side: EndpointSide::Right },
        }
    }

    pub const fn unb_inc(en: T) -> Self {
        Self {
            st: Endpoint::Unbounded { side: EndpointSide::Left },
            en: Endpoint::Closed { p: en, side: EndpointSide::Right },
        }
    }

    pub const fn exc_unb(st: T) -> Self {
        Self {
            st: Endpoint::Open { p: st, side: EndpointSide::Left },
            en: Endpoint::Unbounded { side: EndpointSide::Right },
        }
    }

    pub const fn inc_unb(st: T) -> Self {
        Self {
            st: Endpoint::Closed { p: st, side: EndpointSide::Left },
            en: Endpoint::Unbounded { side: EndpointSide::Right },
        }
    }

    pub const fn unb() -> Self {
        Self {
            st: Endpoint::Unbounded { side: EndpointSide::Left },
            en: Endpoint::Unbounded { side: EndpointSide::Right },
        }
    }

    #[must_use]
    pub fn to_range_full(&self) -> Option<RangeFull> {
        match (&self.st, &self.en) {
            (Endpoint::Unbounded { .. }, Endpoint::Unbounded { .. }) => Some(RangeFull),
            _ => None,
        }
    }
}

impl<T: PartialOrd> SpanAny<T> {
    #[must_use]
    pub fn contains(&self, t: &T) -> bool {
        &self.st <= t && &self.en >= t
    }

    #[must_use]
    pub fn contains_span(&self, s: &Self) -> bool {
        self.st <= s.st && self.en >= s.en
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.st > self.en
    }

    pub fn is_unb(&self) -> bool {
        if self.is_empty() {
            false
        } else {
            // If we are not empty and both endpoints are unbounded, we must be unbounded.
            matches!((&self.st, &self.en), (Endpoint::Unbounded { .. }, Endpoint::Unbounded { .. }))
        }
    }

    #[must_use]
    pub fn to_bounds_ref(&self) -> (Bound<&T>, Bound<&T>) {
        (self.st.bound(), self.en.bound())
    }
}

impl<T: EndpointConversion + Sub + Copy> SpanAny<T> {
    #[must_use]
    pub fn size(&self) -> Option<T::Output> {
        self.en.to_open().zip(self.st.to_closed()).map(|(en, st)| en - st)
    }
}

impl<T: EndpointConversion + Copy> SpanAny<T> {
    #[must_use]
    pub fn to_inc(&self) -> Option<SpanInc<T>> {
        Some(SpanInc::new(self.st.to_closed()?, self.en.to_closed()?))
    }

    #[must_use]
    pub fn to_exc(&self) -> Option<SpanExc<T>> {
        Some(SpanExc::new(self.st.to_closed()?, self.en.to_open()?))
    }

    #[must_use]
    pub fn to_range(&self) -> Option<Range<T>> {
        Some(self.st.to_closed()?..self.en.to_open()?)
    }

    #[must_use]
    pub fn to_range_inclusive(&self) -> Option<RangeInclusive<T>> {
        Some(self.st.to_closed()?..=self.en.to_closed()?)
    }

    #[must_use]
    pub fn to_range_from(&self) -> Option<RangeFrom<T>> {
        if self.en.is_unbounded() { Some(self.st.to_closed()?..) } else { None }
    }

    #[must_use]
    pub fn to_range_to(&self) -> Option<RangeTo<T>> {
        if self.st.is_unbounded() { Some(..self.en.to_open()?) } else { None }
    }

    #[must_use]
    pub fn to_range_to_inclusive(&self) -> Option<RangeToInclusive<T>> {
        if self.st.is_unbounded() { Some(..=self.en.to_closed()?) } else { None }
    }
}

impl<T: Copy> SpanAny<T> {
    #[must_use]
    pub fn to_bounds(&self) -> (Bound<T>, Bound<T>) {
        (self.st.bound().cloned(), self.en.bound().cloned())
    }

    pub fn point(p: T) -> Self {
        Self {
            st: Endpoint::Closed { p, side: EndpointSide::Left },
            en: Endpoint::Closed { p, side: EndpointSide::Right },
        }
    }
}

impl<T: PartialOrd + Copy> SpanAny<T> {
    pub fn cover(a: &Self, b: &Self) -> Self {
        if a.is_empty() {
            *b
        } else if b.is_empty() {
            *a
        } else {
            Self::new(pmin(&a.st, &b.st), pmax(&a.en, &b.en))
        }
    }

    #[must_use]
    pub fn intersect(&self, s: &Self) -> Option<Self> {
        let span = Self::new(pmax(&self.st, &s.st), pmin(&self.en, &s.en));
        if span.is_empty() { None } else { Some(span) }
    }
}

impl<T> From<(Bound<T>, Bound<T>)> for SpanAny<T> {
    fn from(v: (Bound<T>, Bound<T>)) -> Self {
        Self::new(
            Endpoint::from_bound(v.0, EndpointSide::Left),
            Endpoint::from_bound(v.1, EndpointSide::Right),
        )
    }
}

impl<T: Copy> From<(Bound<&T>, Bound<&T>)> for SpanAny<T> {
    fn from(v: (Bound<&T>, Bound<&T>)) -> Self {
        Self::new(
            Endpoint::from_bound(v.0.cloned(), EndpointSide::Left),
            Endpoint::from_bound(v.1.cloned(), EndpointSide::Right),
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn ops() {
        let exc_0_2 = SpanAny::<i64>::exc(0, 2);
        let exc_1_3 = SpanAny::<i64>::exc(1, 3);
        let exc_2_4 = SpanAny::<i64>::exc(2, 4);
        let exc_3_5 = SpanAny::<i64>::exc(3, 5);
        let inc_0_2 = SpanAny::<i64>::inc(0, 2);
        let inc_1_3 = SpanAny::<i64>::inc(1, 3);
        let inc_2_4 = SpanAny::<i64>::inc(2, 4);
        let inc_3_5 = SpanAny::<i64>::inc(3, 5);
        let unb_exc_2 = SpanAny::<i64>::unb_exc(2);
        let unb_inc_2 = SpanAny::<i64>::unb_inc(2);
        let exc_unb_2 = SpanAny::<i64>::exc_unb(2);
        let inc_unb_2 = SpanAny::<i64>::inc_unb(2);
        let unb_unb = SpanAny::<i64>::unb();
        let empty = SpanAny::<i64>::empty();

        // intersect:
        assert_eq!(exc_0_2.intersect(&exc_0_2), Some(SpanAny::exc(0, 2)));
        assert_eq!(exc_0_2.intersect(&exc_1_3), Some(SpanAny::exc(1, 2)));
        assert_eq!(exc_0_2.intersect(&exc_2_4), None);
        assert_eq!(exc_0_2.intersect(&exc_3_5), None);
        assert_eq!(exc_0_2.intersect(&inc_0_2), Some(SpanAny::exc(0, 2)));
        assert_eq!(exc_0_2.intersect(&inc_1_3), Some(SpanAny::exc(1, 2)));
        assert_eq!(exc_0_2.intersect(&inc_2_4), None);
        assert_eq!(exc_0_2.intersect(&inc_3_5), None);
        assert_eq!(exc_0_2.intersect(&unb_exc_2), Some(SpanAny::exc(0, 2)));
        assert_eq!(exc_0_2.intersect(&unb_inc_2), Some(SpanAny::exc(0, 2)));
        assert_eq!(exc_0_2.intersect(&exc_unb_2), None);
        assert_eq!(exc_0_2.intersect(&inc_unb_2), None);
        assert_eq!(exc_0_2.intersect(&unb_unb), Some(SpanAny::exc(0, 2)));
        assert_eq!(exc_0_2.intersect(&empty), None);

        assert_eq!(exc_1_3.intersect(&exc_0_2), Some(SpanAny::exc(1, 2)));
        assert_eq!(exc_1_3.intersect(&exc_1_3), Some(SpanAny::exc(1, 3)));
        assert_eq!(exc_1_3.intersect(&exc_2_4), Some(SpanAny::exc(2, 3)));
        assert_eq!(exc_1_3.intersect(&exc_3_5), None);
        assert_eq!(exc_1_3.intersect(&inc_0_2), Some(SpanAny::inc(1, 2)));
        assert_eq!(exc_1_3.intersect(&inc_1_3), Some(SpanAny::exc(1, 3)));
        assert_eq!(exc_1_3.intersect(&inc_2_4), Some(SpanAny::exc(2, 3)));
        assert_eq!(exc_1_3.intersect(&inc_3_5), None);
        assert_eq!(exc_1_3.intersect(&unb_exc_2), Some(SpanAny::exc(1, 2)));
        assert_eq!(exc_1_3.intersect(&unb_inc_2), Some(SpanAny::inc(1, 2)));
        assert_eq!(exc_1_3.intersect(&exc_unb_2), Some(SpanAny::exc_exc(2, 3)));
        assert_eq!(exc_1_3.intersect(&inc_unb_2), Some(SpanAny::exc(2, 3)));
        assert_eq!(exc_1_3.intersect(&unb_unb), Some(SpanAny::exc(1, 3)));
        assert_eq!(exc_1_3.intersect(&empty), None);

        assert_eq!(exc_2_4.intersect(&exc_0_2), None);
        assert_eq!(exc_2_4.intersect(&exc_1_3), Some(SpanAny::exc(2, 3)));
        assert_eq!(exc_2_4.intersect(&exc_2_4), Some(SpanAny::exc(2, 4)));
        assert_eq!(exc_2_4.intersect(&exc_3_5), Some(SpanAny::exc(3, 4)));
        assert_eq!(exc_2_4.intersect(&inc_0_2), Some(SpanAny::point(2)));
        assert_eq!(exc_2_4.intersect(&inc_1_3), Some(SpanAny::inc(2, 3)));
        assert_eq!(exc_2_4.intersect(&inc_2_4), Some(SpanAny::exc(2, 4)));
        assert_eq!(exc_2_4.intersect(&inc_3_5), Some(SpanAny::exc(3, 4)));
        assert_eq!(exc_2_4.intersect(&unb_exc_2), None);
        assert_eq!(exc_2_4.intersect(&unb_inc_2), Some(SpanAny::point(2)));
        assert_eq!(exc_2_4.intersect(&exc_unb_2), Some(SpanAny::exc_exc(2, 4)));
        assert_eq!(exc_2_4.intersect(&inc_unb_2), Some(SpanAny::exc(2, 4)));
        assert_eq!(exc_2_4.intersect(&unb_unb), Some(SpanAny::exc(2, 4)));
        assert_eq!(exc_2_4.intersect(&empty), None);

        assert_eq!(exc_3_5.intersect(&exc_0_2), None);
        assert_eq!(exc_3_5.intersect(&exc_1_3), None);
        assert_eq!(exc_3_5.intersect(&exc_2_4), Some(SpanAny::exc(3, 4)));
        assert_eq!(exc_3_5.intersect(&exc_3_5), Some(SpanAny::exc(3, 5)));
        assert_eq!(exc_3_5.intersect(&inc_0_2), None);
        assert_eq!(exc_3_5.intersect(&inc_1_3), Some(SpanAny::point(3)));
        assert_eq!(exc_3_5.intersect(&inc_2_4), Some(SpanAny::inc(3, 4)));
        assert_eq!(exc_3_5.intersect(&inc_3_5), Some(SpanAny::exc(3, 5)));
        assert_eq!(exc_3_5.intersect(&unb_exc_2), None);
        assert_eq!(exc_3_5.intersect(&unb_inc_2), None);
        assert_eq!(exc_3_5.intersect(&exc_unb_2), Some(SpanAny::exc(3, 5)));
        assert_eq!(exc_3_5.intersect(&inc_unb_2), Some(SpanAny::exc(3, 5)));
        assert_eq!(exc_3_5.intersect(&unb_unb), Some(SpanAny::exc(3, 5)));
        assert_eq!(exc_3_5.intersect(&empty), None);

        assert_eq!(inc_0_2.intersect(&exc_0_2), Some(SpanAny::exc(0, 2)));
        assert_eq!(inc_0_2.intersect(&exc_1_3), Some(SpanAny::inc(1, 2)));
        assert_eq!(inc_0_2.intersect(&exc_2_4), Some(SpanAny::point(2)));
        assert_eq!(inc_0_2.intersect(&exc_3_5), None);
        assert_eq!(inc_0_2.intersect(&inc_0_2), Some(SpanAny::inc(0, 2)));
        assert_eq!(inc_0_2.intersect(&inc_1_3), Some(SpanAny::inc(1, 2)));
        assert_eq!(inc_0_2.intersect(&inc_2_4), Some(SpanAny::point(2)));
        assert_eq!(inc_0_2.intersect(&inc_3_5), None);
        assert_eq!(inc_0_2.intersect(&unb_exc_2), Some(SpanAny::exc(0, 2)));
        assert_eq!(inc_0_2.intersect(&unb_inc_2), Some(SpanAny::inc(0, 2)));
        assert_eq!(inc_0_2.intersect(&exc_unb_2), None);
        assert_eq!(inc_0_2.intersect(&inc_unb_2), Some(SpanAny::point(2)));
        assert_eq!(inc_0_2.intersect(&unb_unb), Some(SpanAny::inc(0, 2)));
        assert_eq!(inc_0_2.intersect(&empty), None);

        assert_eq!(inc_1_3.intersect(&exc_0_2), Some(SpanAny::exc(1, 2)));
        assert_eq!(inc_1_3.intersect(&exc_1_3), Some(SpanAny::exc(1, 3)));
        assert_eq!(inc_1_3.intersect(&exc_2_4), Some(SpanAny::inc(2, 3)));
        assert_eq!(inc_1_3.intersect(&exc_3_5), Some(SpanAny::point(3)));
        assert_eq!(inc_1_3.intersect(&inc_0_2), Some(SpanAny::inc(1, 2)));
        assert_eq!(inc_1_3.intersect(&inc_1_3), Some(SpanAny::inc(1, 3)));
        assert_eq!(inc_1_3.intersect(&inc_2_4), Some(SpanAny::inc(2, 3)));
        assert_eq!(inc_1_3.intersect(&inc_3_5), Some(SpanAny::point(3)));
        assert_eq!(inc_1_3.intersect(&unb_exc_2), Some(SpanAny::exc(1, 2)));
        assert_eq!(inc_1_3.intersect(&unb_inc_2), Some(SpanAny::inc(1, 2)));
        assert_eq!(inc_1_3.intersect(&exc_unb_2), Some(SpanAny::exc_inc(2, 3)));
        assert_eq!(inc_1_3.intersect(&inc_unb_2), Some(SpanAny::inc(2, 3)));
        assert_eq!(inc_1_3.intersect(&unb_unb), Some(SpanAny::inc(1, 3)));
        assert_eq!(inc_1_3.intersect(&empty), None);

        assert_eq!(inc_2_4.intersect(&exc_0_2), None);
        assert_eq!(inc_2_4.intersect(&exc_1_3), Some(SpanAny::exc(2, 3)));
        assert_eq!(inc_2_4.intersect(&exc_2_4), Some(SpanAny::exc(2, 4)));
        assert_eq!(inc_2_4.intersect(&exc_3_5), Some(SpanAny::inc(3, 4)));
        assert_eq!(inc_2_4.intersect(&inc_0_2), Some(SpanAny::point(2)));
        assert_eq!(inc_2_4.intersect(&inc_1_3), Some(SpanAny::inc(2, 3)));
        assert_eq!(inc_2_4.intersect(&inc_2_4), Some(SpanAny::inc(2, 4)));
        assert_eq!(inc_2_4.intersect(&inc_3_5), Some(SpanAny::inc(3, 4)));
        assert_eq!(inc_2_4.intersect(&unb_exc_2), None);
        assert_eq!(inc_2_4.intersect(&unb_inc_2), Some(SpanAny::point(2)));
        assert_eq!(inc_2_4.intersect(&exc_unb_2), Some(SpanAny::exc_inc(2, 4)));
        assert_eq!(inc_2_4.intersect(&inc_unb_2), Some(SpanAny::inc(2, 4)));
        assert_eq!(inc_2_4.intersect(&unb_unb), Some(SpanAny::inc(2, 4)));
        assert_eq!(inc_2_4.intersect(&empty), None);

        assert_eq!(inc_3_5.intersect(&exc_0_2), None);
        assert_eq!(inc_3_5.intersect(&exc_1_3), None);
        assert_eq!(inc_3_5.intersect(&exc_2_4), Some(SpanAny::exc(3, 4)));
        assert_eq!(inc_3_5.intersect(&exc_3_5), Some(SpanAny::exc(3, 5)));
        assert_eq!(inc_3_5.intersect(&inc_0_2), None);
        assert_eq!(inc_3_5.intersect(&inc_1_3), Some(SpanAny::point(3)));
        assert_eq!(inc_3_5.intersect(&inc_2_4), Some(SpanAny::inc(3, 4)));
        assert_eq!(inc_3_5.intersect(&inc_3_5), Some(SpanAny::inc(3, 5)));
        assert_eq!(inc_3_5.intersect(&unb_exc_2), None);
        assert_eq!(inc_3_5.intersect(&unb_inc_2), None);
        assert_eq!(inc_3_5.intersect(&exc_unb_2), Some(SpanAny::inc(3, 5)));
        assert_eq!(inc_3_5.intersect(&inc_unb_2), Some(SpanAny::inc(3, 5)));
        assert_eq!(inc_3_5.intersect(&unb_unb), Some(SpanAny::inc(3, 5)));
        assert_eq!(inc_3_5.intersect(&empty), None);

        // contains:
        assert!(!exc_0_2.contains(&-1));
        assert!(exc_0_2.contains(&0));
        assert!(exc_0_2.contains(&1));
        assert!(!exc_0_2.contains(&2));
        assert!(!exc_0_2.contains(&3));
        assert!(!inc_0_2.contains(&-1));
        assert!(inc_0_2.contains(&0));
        assert!(inc_0_2.contains(&1));
        assert!(inc_0_2.contains(&2));
        assert!(!inc_0_2.contains(&3));
        assert!(unb_exc_2.contains(&-1));
        assert!(!unb_exc_2.contains(&2));
        assert!(!unb_exc_2.contains(&3));
        assert!(unb_inc_2.contains(&-1));
        assert!(unb_inc_2.contains(&2));
        assert!(!unb_inc_2.contains(&3));
        assert!(!exc_unb_2.contains(&-1));
        assert!(!exc_unb_2.contains(&2));
        assert!(exc_unb_2.contains(&3));
        assert!(!inc_unb_2.contains(&-1));
        assert!(inc_unb_2.contains(&2));
        assert!(inc_unb_2.contains(&3));
        assert!(unb_unb.contains(&-1));
        assert!(unb_unb.contains(&2));

        // contains_span:
        assert!(exc_0_2.contains_span(&exc_0_2));
        assert!(!exc_0_2.contains_span(&exc_1_3));
        assert!(inc_0_2.contains_span(&inc_0_2));
        assert!(inc_0_2.contains_span(&exc_0_2));
        assert!(!exc_0_2.contains_span(&inc_0_2));
        assert!(!exc_0_2.contains_span(&inc_1_3));
        assert!(unb_exc_2.contains_span(&exc_0_2));
        assert!(!unb_exc_2.contains_span(&exc_1_3));
        assert!(unb_inc_2.contains_span(&inc_0_2));
        assert!(unb_inc_2.contains_span(&exc_0_2));
        assert!(!exc_unb_2.contains_span(&inc_0_2));
        assert!(!exc_unb_2.contains_span(&inc_1_3));
        assert!(!inc_unb_2.contains_span(&inc_0_2));
        assert!(!inc_unb_2.contains_span(&exc_0_2));
        assert!(unb_unb.contains_span(&exc_0_2));
        assert!(unb_unb.contains_span(&inc_0_2));
        assert!(unb_unb.contains_span(&unb_exc_2));
        assert!(unb_unb.contains_span(&unb_inc_2));
        assert!(unb_unb.contains_span(&exc_unb_2));
        assert!(unb_unb.contains_span(&inc_unb_2));
        assert!(unb_unb.contains_span(&unb_unb));
        assert!(!empty.contains_span(&exc_0_2));
        assert!(!empty.contains_span(&inc_0_2));
        assert!(!empty.contains_span(&unb_exc_2));
        assert!(!empty.contains_span(&unb_inc_2));
        assert!(!empty.contains_span(&exc_unb_2));
        assert!(!empty.contains_span(&inc_unb_2));
        assert!(!empty.contains_span(&unb_unb));
        assert!(empty.contains_span(&empty));

        // cover:
        assert_eq!(SpanAny::cover(&exc_0_2, &exc_0_2), SpanAny::exc(0, 2));
        assert_eq!(SpanAny::cover(&exc_0_2, &exc_1_3), SpanAny::exc(0, 3));
        assert_eq!(SpanAny::cover(&exc_0_2, &exc_2_4), SpanAny::exc(0, 4));
        assert_eq!(SpanAny::cover(&exc_0_2, &exc_3_5), SpanAny::exc(0, 5));
        assert_eq!(SpanAny::cover(&exc_0_2, &inc_0_2), SpanAny::inc(0, 2));
        assert_eq!(SpanAny::cover(&exc_0_2, &inc_1_3), SpanAny::inc(0, 3));
        assert_eq!(SpanAny::cover(&exc_0_2, &inc_2_4), SpanAny::inc(0, 4));
        assert_eq!(SpanAny::cover(&exc_0_2, &inc_3_5), SpanAny::inc(0, 5));
        assert_eq!(SpanAny::cover(&exc_0_2, &unb_exc_2), SpanAny::unb_exc(2));
        assert_eq!(SpanAny::cover(&exc_0_2, &unb_inc_2), SpanAny::unb_inc(2));
        assert_eq!(SpanAny::cover(&exc_0_2, &exc_unb_2), SpanAny::inc_unb(0));
        assert_eq!(SpanAny::cover(&exc_0_2, &inc_unb_2), SpanAny::inc_unb(0));
        assert_eq!(SpanAny::cover(&exc_0_2, &unb_unb), SpanAny::unb());
        assert_eq!(SpanAny::cover(&exc_0_2, &empty), SpanAny::exc(0, 2));

        assert_eq!(SpanAny::cover(&exc_1_3, &exc_0_2), SpanAny::exc(0, 3));
        assert_eq!(SpanAny::cover(&exc_1_3, &exc_1_3), SpanAny::exc(1, 3));
        assert_eq!(SpanAny::cover(&exc_1_3, &exc_2_4), SpanAny::exc(1, 4));
        assert_eq!(SpanAny::cover(&exc_1_3, &exc_3_5), SpanAny::exc(1, 5));
        assert_eq!(SpanAny::cover(&exc_1_3, &inc_0_2), SpanAny::exc(0, 3));
        assert_eq!(SpanAny::cover(&exc_1_3, &inc_1_3), SpanAny::inc(1, 3));
        assert_eq!(SpanAny::cover(&exc_1_3, &inc_2_4), SpanAny::inc(1, 4));
        assert_eq!(SpanAny::cover(&exc_1_3, &inc_3_5), SpanAny::inc(1, 5));
        assert_eq!(SpanAny::cover(&exc_1_3, &unb_exc_2), SpanAny::unb_exc(3));
        assert_eq!(SpanAny::cover(&exc_1_3, &unb_inc_2), SpanAny::unb_exc(3));
        assert_eq!(SpanAny::cover(&exc_1_3, &exc_unb_2), SpanAny::inc_unb(1));
        assert_eq!(SpanAny::cover(&exc_1_3, &inc_unb_2), SpanAny::inc_unb(1));
        assert_eq!(SpanAny::cover(&exc_1_3, &unb_unb), SpanAny::unb());
        assert_eq!(SpanAny::cover(&exc_1_3, &empty), SpanAny::exc(1, 3));

        assert_eq!(SpanAny::cover(&exc_2_4, &exc_0_2), SpanAny::exc(0, 4));
        assert_eq!(SpanAny::cover(&exc_2_4, &exc_1_3), SpanAny::exc(1, 4));
        assert_eq!(SpanAny::cover(&exc_2_4, &exc_2_4), SpanAny::exc(2, 4));
        assert_eq!(SpanAny::cover(&exc_2_4, &exc_3_5), SpanAny::exc(2, 5));
        assert_eq!(SpanAny::cover(&exc_2_4, &inc_0_2), SpanAny::exc(0, 4));
        assert_eq!(SpanAny::cover(&exc_2_4, &inc_1_3), SpanAny::exc(1, 4));
        assert_eq!(SpanAny::cover(&exc_2_4, &inc_2_4), SpanAny::inc(2, 4));
        assert_eq!(SpanAny::cover(&exc_2_4, &inc_3_5), SpanAny::inc(2, 5));
        assert_eq!(SpanAny::cover(&exc_2_4, &unb_exc_2), SpanAny::unb_exc(4));
        assert_eq!(SpanAny::cover(&exc_2_4, &unb_inc_2), SpanAny::unb_exc(4));
        assert_eq!(SpanAny::cover(&exc_2_4, &exc_unb_2), SpanAny::inc_unb(2));
        assert_eq!(SpanAny::cover(&exc_2_4, &inc_unb_2), SpanAny::inc_unb(2));
        assert_eq!(SpanAny::cover(&exc_2_4, &unb_unb), SpanAny::unb());
        assert_eq!(SpanAny::cover(&exc_2_4, &empty), SpanAny::exc(2, 4));

        assert_eq!(SpanAny::cover(&exc_3_5, &exc_0_2), SpanAny::exc(0, 5));
        assert_eq!(SpanAny::cover(&exc_3_5, &exc_1_3), SpanAny::exc(1, 5));
        assert_eq!(SpanAny::cover(&exc_3_5, &exc_2_4), SpanAny::exc(2, 5));
        assert_eq!(SpanAny::cover(&exc_3_5, &exc_3_5), SpanAny::exc(3, 5));
        assert_eq!(SpanAny::cover(&exc_3_5, &inc_0_2), SpanAny::exc(0, 5));
        assert_eq!(SpanAny::cover(&exc_3_5, &inc_1_3), SpanAny::exc(1, 5));
        assert_eq!(SpanAny::cover(&exc_3_5, &inc_2_4), SpanAny::exc(2, 5));
        assert_eq!(SpanAny::cover(&exc_3_5, &inc_3_5), SpanAny::inc(3, 5));
        assert_eq!(SpanAny::cover(&exc_3_5, &unb_exc_2), SpanAny::unb_exc(5));
        assert_eq!(SpanAny::cover(&exc_3_5, &unb_inc_2), SpanAny::unb_exc(5));
        assert_eq!(SpanAny::cover(&exc_3_5, &exc_unb_2), SpanAny::exc_unb(2));
        assert_eq!(SpanAny::cover(&exc_3_5, &inc_unb_2), SpanAny::inc_unb(2));
        assert_eq!(SpanAny::cover(&exc_3_5, &unb_unb), SpanAny::unb());
        assert_eq!(SpanAny::cover(&exc_3_5, &empty), SpanAny::exc(3, 5));

        assert_eq!(SpanAny::cover(&inc_0_2, &exc_0_2), SpanAny::inc(0, 2));
        assert_eq!(SpanAny::cover(&inc_0_2, &exc_1_3), SpanAny::exc(0, 3));
        assert_eq!(SpanAny::cover(&inc_0_2, &exc_2_4), SpanAny::exc(0, 4));
        assert_eq!(SpanAny::cover(&inc_0_2, &exc_3_5), SpanAny::exc(0, 5));
        assert_eq!(SpanAny::cover(&inc_0_2, &inc_0_2), SpanAny::inc(0, 2));
        assert_eq!(SpanAny::cover(&inc_0_2, &inc_1_3), SpanAny::inc(0, 3));
        assert_eq!(SpanAny::cover(&inc_0_2, &inc_2_4), SpanAny::inc(0, 4));
        assert_eq!(SpanAny::cover(&inc_0_2, &inc_3_5), SpanAny::inc(0, 5));
        assert_eq!(SpanAny::cover(&inc_0_2, &unb_exc_2), SpanAny::unb_inc(2));
        assert_eq!(SpanAny::cover(&inc_0_2, &unb_inc_2), SpanAny::unb_inc(2));
        assert_eq!(SpanAny::cover(&inc_0_2, &exc_unb_2), SpanAny::inc_unb(0));
        assert_eq!(SpanAny::cover(&inc_0_2, &inc_unb_2), SpanAny::inc_unb(0));
        assert_eq!(SpanAny::cover(&inc_0_2, &unb_unb), SpanAny::unb());
        assert_eq!(SpanAny::cover(&inc_0_2, &empty), SpanAny::inc(0, 2));

        assert_eq!(SpanAny::cover(&inc_1_3, &exc_0_2), SpanAny::inc(0, 3));
        assert_eq!(SpanAny::cover(&inc_1_3, &exc_1_3), SpanAny::inc(1, 3));
        assert_eq!(SpanAny::cover(&inc_1_3, &exc_2_4), SpanAny::exc(1, 4));
        assert_eq!(SpanAny::cover(&inc_1_3, &exc_3_5), SpanAny::exc(1, 5));
        assert_eq!(SpanAny::cover(&inc_1_3, &inc_0_2), SpanAny::inc(0, 3));
        assert_eq!(SpanAny::cover(&inc_1_3, &inc_1_3), SpanAny::inc(1, 3));
        assert_eq!(SpanAny::cover(&inc_1_3, &inc_2_4), SpanAny::inc(1, 4));
        assert_eq!(SpanAny::cover(&inc_1_3, &inc_3_5), SpanAny::inc(1, 5));
        assert_eq!(SpanAny::cover(&inc_1_3, &unb_exc_2), SpanAny::unb_inc(3));
        assert_eq!(SpanAny::cover(&inc_1_3, &unb_inc_2), SpanAny::unb_inc(3));
        assert_eq!(SpanAny::cover(&inc_1_3, &exc_unb_2), SpanAny::inc_unb(1));
        assert_eq!(SpanAny::cover(&inc_1_3, &inc_unb_2), SpanAny::inc_unb(1));
        assert_eq!(SpanAny::cover(&inc_1_3, &unb_unb), SpanAny::unb());
        assert_eq!(SpanAny::cover(&inc_1_3, &empty), SpanAny::inc(1, 3));

        assert_eq!(SpanAny::cover(&inc_2_4, &exc_0_2), SpanAny::inc(0, 4));
        assert_eq!(SpanAny::cover(&inc_2_4, &exc_1_3), SpanAny::inc(1, 4));
        assert_eq!(SpanAny::cover(&inc_2_4, &exc_2_4), SpanAny::inc(2, 4));
        assert_eq!(SpanAny::cover(&inc_2_4, &exc_3_5), SpanAny::exc(2, 5));
        assert_eq!(SpanAny::cover(&inc_2_4, &inc_0_2), SpanAny::inc(0, 4));
        assert_eq!(SpanAny::cover(&inc_2_4, &inc_1_3), SpanAny::inc(1, 4));
        assert_eq!(SpanAny::cover(&inc_2_4, &inc_2_4), SpanAny::inc(2, 4));
        assert_eq!(SpanAny::cover(&inc_2_4, &inc_3_5), SpanAny::inc(2, 5));
        assert_eq!(SpanAny::cover(&inc_2_4, &unb_exc_2), SpanAny::unb_inc(4));
        assert_eq!(SpanAny::cover(&inc_2_4, &unb_inc_2), SpanAny::unb_inc(4));
        assert_eq!(SpanAny::cover(&inc_2_4, &exc_unb_2), SpanAny::inc_unb(2));
        assert_eq!(SpanAny::cover(&inc_2_4, &inc_unb_2), SpanAny::inc_unb(2));
        assert_eq!(SpanAny::cover(&inc_2_4, &unb_unb), SpanAny::unb());
        assert_eq!(SpanAny::cover(&inc_2_4, &empty), SpanAny::inc(2, 4));

        assert_eq!(SpanAny::cover(&inc_3_5, &exc_0_2), SpanAny::inc(0, 5));
        assert_eq!(SpanAny::cover(&inc_3_5, &exc_1_3), SpanAny::inc(1, 5));
        assert_eq!(SpanAny::cover(&inc_3_5, &exc_2_4), SpanAny::inc(2, 5));
        assert_eq!(SpanAny::cover(&inc_3_5, &exc_3_5), SpanAny::inc(3, 5));
        assert_eq!(SpanAny::cover(&inc_3_5, &inc_0_2), SpanAny::inc(0, 5));
        assert_eq!(SpanAny::cover(&inc_3_5, &inc_1_3), SpanAny::inc(1, 5));
        assert_eq!(SpanAny::cover(&inc_3_5, &inc_2_4), SpanAny::inc(2, 5));
        assert_eq!(SpanAny::cover(&inc_3_5, &inc_3_5), SpanAny::inc(3, 5));
        assert_eq!(SpanAny::cover(&inc_3_5, &unb_exc_2), SpanAny::unb_inc(5));
        assert_eq!(SpanAny::cover(&inc_3_5, &unb_inc_2), SpanAny::unb_inc(5));
        assert_eq!(SpanAny::cover(&inc_3_5, &exc_unb_2), SpanAny::exc_unb(2));
        assert_eq!(SpanAny::cover(&inc_3_5, &inc_unb_2), SpanAny::inc_unb(2));
        assert_eq!(SpanAny::cover(&inc_3_5, &unb_unb), SpanAny::unb());
        assert_eq!(SpanAny::cover(&inc_3_5, &empty), SpanAny::inc(3, 5));

        assert_eq!(SpanAny::cover(&empty, &exc_0_2), SpanAny::exc(0, 2));
        assert_eq!(SpanAny::cover(&empty, &exc_1_3), SpanAny::exc(1, 3));
        assert_eq!(SpanAny::cover(&empty, &exc_2_4), SpanAny::exc(2, 4));
        assert_eq!(SpanAny::cover(&empty, &exc_3_5), SpanAny::exc(3, 5));
        assert_eq!(SpanAny::cover(&empty, &inc_0_2), SpanAny::inc(0, 2));
        assert_eq!(SpanAny::cover(&empty, &inc_1_3), SpanAny::inc(1, 3));
        assert_eq!(SpanAny::cover(&empty, &inc_2_4), SpanAny::inc(2, 4));
        assert_eq!(SpanAny::cover(&empty, &inc_3_5), SpanAny::inc(3, 5));
        assert_eq!(SpanAny::cover(&empty, &unb_exc_2), SpanAny::unb_exc(2));
        assert_eq!(SpanAny::cover(&empty, &unb_inc_2), SpanAny::unb_inc(2));
        assert_eq!(SpanAny::cover(&empty, &exc_unb_2), SpanAny::exc_unb(2));
        assert_eq!(SpanAny::cover(&empty, &inc_unb_2), SpanAny::inc_unb(2));
        assert_eq!(SpanAny::cover(&empty, &unb_unb), SpanAny::unb());
        assert_eq!(SpanAny::cover(&empty, &empty), SpanAny::empty());

        // is_empty:
        assert!(!exc_0_2.is_empty());
        assert!(!exc_1_3.is_empty());
        assert!(!exc_2_4.is_empty());
        assert!(!exc_3_5.is_empty());
        assert!(!inc_0_2.is_empty());
        assert!(!inc_1_3.is_empty());
        assert!(!inc_2_4.is_empty());
        assert!(!inc_3_5.is_empty());
        assert!(!unb_exc_2.is_empty());
        assert!(!unb_inc_2.is_empty());
        assert!(!exc_unb_2.is_empty());
        assert!(!inc_unb_2.is_empty());
        assert!(!unb_unb.is_empty());
        assert!(empty.is_empty());

        // size:
        assert_eq!(exc_0_2.size(), Some(2));
        assert_eq!(exc_1_3.size(), Some(2));
        assert_eq!(exc_2_4.size(), Some(2));
        assert_eq!(exc_3_5.size(), Some(2));
        assert_eq!(inc_0_2.size(), Some(3));
        assert_eq!(inc_1_3.size(), Some(3));
        assert_eq!(inc_2_4.size(), Some(3));
        assert_eq!(inc_3_5.size(), Some(3));
        assert_eq!(unb_exc_2.size(), None);
        assert_eq!(unb_inc_2.size(), None);
        assert_eq!(exc_unb_2.size(), None);
        assert_eq!(inc_unb_2.size(), None);
        assert_eq!(unb_unb.size(), None);
        assert_eq!(empty.size(), Some(0));
    }
}
