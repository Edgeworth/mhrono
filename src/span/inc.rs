use std::fmt;
use std::ops::{Bound, Range, RangeInclusive, Sub};

use serde::{Deserialize, Serialize};

use crate::span::any::SpanAny;
use crate::span::endpoint::EndpointConversion;
use crate::span::exc::SpanExc;
use crate::span::ops::{pmax, pmin};

#[must_use]
#[derive(
    Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize,
)]
pub struct SpanInc<T> {
    pub st: T,
    pub en: T,
}

impl<T: fmt::Display> fmt::Display for SpanInc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{},{}]", self.st, self.en)
    }
}

impl<T: Default + EndpointConversion> SpanInc<T> {
    #[must_use]
    pub fn empty() -> Option<Self> {
        Self::exc(T::default(), T::default())
    }
}

impl<T> SpanInc<T> {
    pub const fn new(st: T, en: T) -> Self {
        Self { st, en }
    }

    #[must_use]
    pub const fn to_bounds_ref(&self) -> (Bound<&T>, Bound<&T>) {
        (Bound::Included(&self.st), Bound::Included(&self.en))
    }
}

impl<T: Copy> SpanInc<T> {
    pub const fn point(p: T) -> Self {
        Self { st: p, en: p }
    }

    #[must_use]
    pub const fn range_inclusive(&self) -> RangeInclusive<T> {
        self.st..=self.en
    }

    #[must_use]
    pub const fn to_bounds(&self) -> (Bound<T>, Bound<T>) {
        (Bound::Included(self.st), Bound::Included(self.en))
    }

    pub const fn to_any(&self) -> SpanAny<T> {
        SpanAny::inc(self.st, self.en)
    }
}

impl<T: PartialOrd> SpanInc<T> {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.st > self.en
    }

    #[must_use]
    pub fn contains(&self, t: &T) -> bool {
        &self.st <= t && &self.en >= t
    }

    #[must_use]
    pub fn contains_span(&self, s: &Self) -> bool {
        self.st <= s.st && self.en >= s.en
    }
}

impl<T: PartialOrd + Copy> SpanInc<T> {
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
        if span.is_empty() {
            None
        } else {
            Some(span)
        }
    }
}

impl<T: EndpointConversion> SpanInc<T> {
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn exc(st: T, en: T) -> Option<Self> {
        <T as EndpointConversion>::to_closed(&en, false).map(|en| Self::new(st, en))
    }
}

impl<T: EndpointConversion + Copy> SpanInc<T> {
    #[must_use]
    pub fn to_exc(&self) -> Option<SpanExc<T>> {
        SpanExc::inc(self.st, self.en)
    }

    #[must_use]
    pub fn to_range(&self) -> Option<Range<T>> {
        SpanExc::inc(self.st, self.en).map(|s| s.range())
    }
}

impl<T: Sub + EndpointConversion + Copy> SpanInc<T> {
    #[must_use]
    pub fn size(&self) -> Option<T::Output> {
        <T as EndpointConversion>::to_open(&self.en, false).map(|v| v - self.st)
    }
}

impl<T: Copy> From<RangeInclusive<T>> for SpanInc<T> {
    fn from(r: RangeInclusive<T>) -> Self {
        Self::new(*r.start(), *r.end())
    }
}

impl<T: Copy> From<SpanInc<T>> for RangeInclusive<T> {
    fn from(s: SpanInc<T>) -> Self {
        s.range_inclusive()
    }
}

impl<T: EndpointConversion> TryFrom<Range<T>> for SpanInc<T> {
    type Error = ();

    fn try_from(r: Range<T>) -> Result<Self, Self::Error> {
        Self::exc(r.start, r.end).ok_or(())
    }
}

impl<T: EndpointConversion> TryFrom<SpanInc<T>> for Range<T> {
    type Error = ();

    fn try_from(s: SpanInc<T>) -> Result<Self, Self::Error> {
        let en = <T as EndpointConversion>::to_open(&s.en, false).ok_or(())?;
        Ok(s.st..en)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn ops() {
        let exc_0_2 = SpanInc::<i64>::exc(0, 2).unwrap();
        let exc_1_3 = SpanInc::<i64>::exc(1, 3).unwrap();
        let exc_2_4 = SpanInc::<i64>::exc(2, 4).unwrap();
        let exc_3_5 = SpanInc::<i64>::exc(3, 5).unwrap();
        let inc_0_2 = SpanInc::<i64>::new(0, 2);
        let inc_1_3 = SpanInc::<i64>::new(1, 3);
        let inc_2_4 = SpanInc::<i64>::new(2, 4);
        let inc_3_5 = SpanInc::<i64>::new(3, 5);
        let empty = SpanInc::<i64>::empty().unwrap();

        // intersect:
        assert_eq!(exc_0_2.intersect(&exc_0_2), Some(SpanInc::exc(0, 2).unwrap()));
        assert_eq!(exc_0_2.intersect(&exc_1_3), Some(SpanInc::exc(1, 2).unwrap()));
        assert_eq!(exc_0_2.intersect(&exc_2_4), None);
        assert_eq!(exc_0_2.intersect(&exc_3_5), None);
        assert_eq!(exc_0_2.intersect(&inc_0_2), Some(SpanInc::exc(0, 2).unwrap()));
        assert_eq!(exc_0_2.intersect(&inc_1_3), Some(SpanInc::exc(1, 2).unwrap()));
        assert_eq!(exc_0_2.intersect(&inc_2_4), None);
        assert_eq!(exc_0_2.intersect(&inc_3_5), None);
        assert_eq!(exc_0_2.intersect(&empty), None);

        assert_eq!(exc_1_3.intersect(&exc_0_2), Some(SpanInc::exc(1, 2).unwrap()));
        assert_eq!(exc_1_3.intersect(&exc_1_3), Some(SpanInc::exc(1, 3).unwrap()));
        assert_eq!(exc_1_3.intersect(&exc_2_4), Some(SpanInc::exc(2, 3).unwrap()));
        assert_eq!(exc_1_3.intersect(&exc_3_5), None);
        assert_eq!(exc_1_3.intersect(&inc_0_2), Some(SpanInc::new(1, 2)));
        assert_eq!(exc_1_3.intersect(&inc_1_3), Some(SpanInc::exc(1, 3).unwrap()));
        assert_eq!(exc_1_3.intersect(&inc_2_4), Some(SpanInc::exc(2, 3).unwrap()));
        assert_eq!(exc_1_3.intersect(&inc_3_5), None);
        assert_eq!(exc_1_3.intersect(&empty), None);

        assert_eq!(exc_2_4.intersect(&exc_0_2), None);
        assert_eq!(exc_2_4.intersect(&exc_1_3), Some(SpanInc::exc(2, 3).unwrap()));
        assert_eq!(exc_2_4.intersect(&exc_2_4), Some(SpanInc::exc(2, 4).unwrap()));
        assert_eq!(exc_2_4.intersect(&exc_3_5), Some(SpanInc::exc(3, 4).unwrap()));
        assert_eq!(exc_2_4.intersect(&inc_0_2), Some(SpanInc::point(2)));
        assert_eq!(exc_2_4.intersect(&inc_1_3), Some(SpanInc::new(2, 3)));
        assert_eq!(exc_2_4.intersect(&inc_2_4), Some(SpanInc::exc(2, 4).unwrap()));
        assert_eq!(exc_2_4.intersect(&inc_3_5), Some(SpanInc::exc(3, 4).unwrap()));
        assert_eq!(exc_2_4.intersect(&empty), None);

        assert_eq!(exc_3_5.intersect(&exc_0_2), None);
        assert_eq!(exc_3_5.intersect(&exc_1_3), None);
        assert_eq!(exc_3_5.intersect(&exc_2_4), Some(SpanInc::exc(3, 4).unwrap()));
        assert_eq!(exc_3_5.intersect(&exc_3_5), Some(SpanInc::exc(3, 5).unwrap()));
        assert_eq!(exc_3_5.intersect(&inc_0_2), None);
        assert_eq!(exc_3_5.intersect(&inc_1_3), Some(SpanInc::point(3)));
        assert_eq!(exc_3_5.intersect(&inc_2_4), Some(SpanInc::new(3, 4)));
        assert_eq!(exc_3_5.intersect(&inc_3_5), Some(SpanInc::exc(3, 5).unwrap()));
        assert_eq!(exc_3_5.intersect(&empty), None);

        assert_eq!(inc_0_2.intersect(&exc_0_2), Some(SpanInc::exc(0, 2).unwrap()));
        assert_eq!(inc_0_2.intersect(&exc_1_3), Some(SpanInc::new(1, 2)));
        assert_eq!(inc_0_2.intersect(&exc_2_4), Some(SpanInc::point(2)));
        assert_eq!(inc_0_2.intersect(&exc_3_5), None);
        assert_eq!(inc_0_2.intersect(&inc_0_2), Some(SpanInc::new(0, 2)));
        assert_eq!(inc_0_2.intersect(&inc_1_3), Some(SpanInc::new(1, 2)));
        assert_eq!(inc_0_2.intersect(&inc_2_4), Some(SpanInc::point(2)));
        assert_eq!(inc_0_2.intersect(&inc_3_5), None);
        assert_eq!(inc_0_2.intersect(&empty), None);

        assert_eq!(inc_1_3.intersect(&exc_0_2), Some(SpanInc::exc(1, 2).unwrap()));
        assert_eq!(inc_1_3.intersect(&exc_1_3), Some(SpanInc::exc(1, 3).unwrap()));
        assert_eq!(inc_1_3.intersect(&exc_2_4), Some(SpanInc::new(2, 3)));
        assert_eq!(inc_1_3.intersect(&exc_3_5), Some(SpanInc::point(3)));
        assert_eq!(inc_1_3.intersect(&inc_0_2), Some(SpanInc::new(1, 2)));
        assert_eq!(inc_1_3.intersect(&inc_1_3), Some(SpanInc::new(1, 3)));
        assert_eq!(inc_1_3.intersect(&inc_2_4), Some(SpanInc::new(2, 3)));
        assert_eq!(inc_1_3.intersect(&inc_3_5), Some(SpanInc::point(3)));
        assert_eq!(inc_1_3.intersect(&empty), None);

        assert_eq!(inc_2_4.intersect(&exc_0_2), None);
        assert_eq!(inc_2_4.intersect(&exc_1_3), Some(SpanInc::exc(2, 3).unwrap()));
        assert_eq!(inc_2_4.intersect(&exc_2_4), Some(SpanInc::exc(2, 4).unwrap()));
        assert_eq!(inc_2_4.intersect(&exc_3_5), Some(SpanInc::new(3, 4)));
        assert_eq!(inc_2_4.intersect(&inc_0_2), Some(SpanInc::point(2)));
        assert_eq!(inc_2_4.intersect(&inc_1_3), Some(SpanInc::new(2, 3)));
        assert_eq!(inc_2_4.intersect(&inc_2_4), Some(SpanInc::new(2, 4)));
        assert_eq!(inc_2_4.intersect(&inc_3_5), Some(SpanInc::new(3, 4)));
        assert_eq!(inc_2_4.intersect(&empty), None);

        assert_eq!(inc_3_5.intersect(&exc_0_2), None);
        assert_eq!(inc_3_5.intersect(&exc_1_3), None);
        assert_eq!(inc_3_5.intersect(&exc_2_4), Some(SpanInc::exc(3, 4).unwrap()));
        assert_eq!(inc_3_5.intersect(&exc_3_5), Some(SpanInc::exc(3, 5).unwrap()));
        assert_eq!(inc_3_5.intersect(&inc_0_2), None);
        assert_eq!(inc_3_5.intersect(&inc_1_3), Some(SpanInc::point(3)));
        assert_eq!(inc_3_5.intersect(&inc_2_4), Some(SpanInc::new(3, 4)));
        assert_eq!(inc_3_5.intersect(&inc_3_5), Some(SpanInc::new(3, 5)));
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

        // contains_span:
        assert!(exc_0_2.contains_span(&exc_0_2));
        assert!(!exc_0_2.contains_span(&exc_1_3));
        assert!(inc_0_2.contains_span(&inc_0_2));
        assert!(inc_0_2.contains_span(&exc_0_2));
        assert!(!exc_0_2.contains_span(&inc_0_2));
        assert!(!exc_0_2.contains_span(&inc_1_3));
        assert!(!empty.contains_span(&exc_0_2));
        assert!(!empty.contains_span(&inc_0_2));
        assert!(empty.contains_span(&empty));

        // cover:
        assert_eq!(SpanInc::cover(&exc_0_2, &exc_0_2), SpanInc::exc(0, 2).unwrap());
        assert_eq!(SpanInc::cover(&exc_0_2, &exc_1_3), SpanInc::exc(0, 3).unwrap());
        assert_eq!(SpanInc::cover(&exc_0_2, &exc_2_4), SpanInc::exc(0, 4).unwrap());
        assert_eq!(SpanInc::cover(&exc_0_2, &exc_3_5), SpanInc::exc(0, 5).unwrap());
        assert_eq!(SpanInc::cover(&exc_0_2, &inc_0_2), SpanInc::new(0, 2));
        assert_eq!(SpanInc::cover(&exc_0_2, &inc_1_3), SpanInc::new(0, 3));
        assert_eq!(SpanInc::cover(&exc_0_2, &inc_2_4), SpanInc::new(0, 4));
        assert_eq!(SpanInc::cover(&exc_0_2, &inc_3_5), SpanInc::new(0, 5));
        assert_eq!(SpanInc::cover(&exc_0_2, &empty), SpanInc::exc(0, 2).unwrap());

        assert_eq!(SpanInc::cover(&exc_1_3, &exc_0_2), SpanInc::exc(0, 3).unwrap());
        assert_eq!(SpanInc::cover(&exc_1_3, &exc_1_3), SpanInc::exc(1, 3).unwrap());
        assert_eq!(SpanInc::cover(&exc_1_3, &exc_2_4), SpanInc::exc(1, 4).unwrap());
        assert_eq!(SpanInc::cover(&exc_1_3, &exc_3_5), SpanInc::exc(1, 5).unwrap());
        assert_eq!(SpanInc::cover(&exc_1_3, &inc_0_2), SpanInc::exc(0, 3).unwrap());
        assert_eq!(SpanInc::cover(&exc_1_3, &inc_1_3), SpanInc::new(1, 3));
        assert_eq!(SpanInc::cover(&exc_1_3, &inc_2_4), SpanInc::new(1, 4));
        assert_eq!(SpanInc::cover(&exc_1_3, &inc_3_5), SpanInc::new(1, 5));
        assert_eq!(SpanInc::cover(&exc_1_3, &empty), SpanInc::exc(1, 3).unwrap());

        assert_eq!(SpanInc::cover(&exc_2_4, &exc_0_2), SpanInc::exc(0, 4).unwrap());
        assert_eq!(SpanInc::cover(&exc_2_4, &exc_1_3), SpanInc::exc(1, 4).unwrap());
        assert_eq!(SpanInc::cover(&exc_2_4, &exc_2_4), SpanInc::exc(2, 4).unwrap());
        assert_eq!(SpanInc::cover(&exc_2_4, &exc_3_5), SpanInc::exc(2, 5).unwrap());
        assert_eq!(SpanInc::cover(&exc_2_4, &inc_0_2), SpanInc::exc(0, 4).unwrap());
        assert_eq!(SpanInc::cover(&exc_2_4, &inc_1_3), SpanInc::exc(1, 4).unwrap());
        assert_eq!(SpanInc::cover(&exc_2_4, &inc_2_4), SpanInc::new(2, 4));
        assert_eq!(SpanInc::cover(&exc_2_4, &inc_3_5), SpanInc::new(2, 5));
        assert_eq!(SpanInc::cover(&exc_2_4, &empty), SpanInc::exc(2, 4).unwrap());

        assert_eq!(SpanInc::cover(&exc_3_5, &exc_0_2), SpanInc::exc(0, 5).unwrap());
        assert_eq!(SpanInc::cover(&exc_3_5, &exc_1_3), SpanInc::exc(1, 5).unwrap());
        assert_eq!(SpanInc::cover(&exc_3_5, &exc_2_4), SpanInc::exc(2, 5).unwrap());
        assert_eq!(SpanInc::cover(&exc_3_5, &exc_3_5), SpanInc::exc(3, 5).unwrap());
        assert_eq!(SpanInc::cover(&exc_3_5, &inc_0_2), SpanInc::exc(0, 5).unwrap());
        assert_eq!(SpanInc::cover(&exc_3_5, &inc_1_3), SpanInc::exc(1, 5).unwrap());
        assert_eq!(SpanInc::cover(&exc_3_5, &inc_2_4), SpanInc::exc(2, 5).unwrap());
        assert_eq!(SpanInc::cover(&exc_3_5, &inc_3_5), SpanInc::new(3, 5));
        assert_eq!(SpanInc::cover(&exc_3_5, &empty), SpanInc::exc(3, 5).unwrap());

        assert_eq!(SpanInc::cover(&inc_0_2, &exc_0_2), SpanInc::new(0, 2));
        assert_eq!(SpanInc::cover(&inc_0_2, &exc_1_3), SpanInc::exc(0, 3).unwrap());
        assert_eq!(SpanInc::cover(&inc_0_2, &exc_2_4), SpanInc::exc(0, 4).unwrap());
        assert_eq!(SpanInc::cover(&inc_0_2, &exc_3_5), SpanInc::exc(0, 5).unwrap());
        assert_eq!(SpanInc::cover(&inc_0_2, &inc_0_2), SpanInc::new(0, 2));
        assert_eq!(SpanInc::cover(&inc_0_2, &inc_1_3), SpanInc::new(0, 3));
        assert_eq!(SpanInc::cover(&inc_0_2, &inc_2_4), SpanInc::new(0, 4));
        assert_eq!(SpanInc::cover(&inc_0_2, &inc_3_5), SpanInc::new(0, 5));
        assert_eq!(SpanInc::cover(&inc_0_2, &empty), SpanInc::new(0, 2));

        assert_eq!(SpanInc::cover(&inc_1_3, &exc_0_2), SpanInc::new(0, 3));
        assert_eq!(SpanInc::cover(&inc_1_3, &exc_1_3), SpanInc::new(1, 3));
        assert_eq!(SpanInc::cover(&inc_1_3, &exc_2_4), SpanInc::exc(1, 4).unwrap());
        assert_eq!(SpanInc::cover(&inc_1_3, &exc_3_5), SpanInc::exc(1, 5).unwrap());
        assert_eq!(SpanInc::cover(&inc_1_3, &inc_0_2), SpanInc::new(0, 3));
        assert_eq!(SpanInc::cover(&inc_1_3, &inc_1_3), SpanInc::new(1, 3));
        assert_eq!(SpanInc::cover(&inc_1_3, &inc_2_4), SpanInc::new(1, 4));
        assert_eq!(SpanInc::cover(&inc_1_3, &inc_3_5), SpanInc::new(1, 5));
        assert_eq!(SpanInc::cover(&inc_1_3, &empty), SpanInc::new(1, 3));

        assert_eq!(SpanInc::cover(&inc_2_4, &exc_0_2), SpanInc::new(0, 4));
        assert_eq!(SpanInc::cover(&inc_2_4, &exc_1_3), SpanInc::new(1, 4));
        assert_eq!(SpanInc::cover(&inc_2_4, &exc_2_4), SpanInc::new(2, 4));
        assert_eq!(SpanInc::cover(&inc_2_4, &exc_3_5), SpanInc::exc(2, 5).unwrap());
        assert_eq!(SpanInc::cover(&inc_2_4, &inc_0_2), SpanInc::new(0, 4));
        assert_eq!(SpanInc::cover(&inc_2_4, &inc_1_3), SpanInc::new(1, 4));
        assert_eq!(SpanInc::cover(&inc_2_4, &inc_2_4), SpanInc::new(2, 4));
        assert_eq!(SpanInc::cover(&inc_2_4, &inc_3_5), SpanInc::new(2, 5));
        assert_eq!(SpanInc::cover(&inc_2_4, &empty), SpanInc::new(2, 4));

        assert_eq!(SpanInc::cover(&inc_3_5, &exc_0_2), SpanInc::new(0, 5));
        assert_eq!(SpanInc::cover(&inc_3_5, &exc_1_3), SpanInc::new(1, 5));
        assert_eq!(SpanInc::cover(&inc_3_5, &exc_2_4), SpanInc::new(2, 5));
        assert_eq!(SpanInc::cover(&inc_3_5, &exc_3_5), SpanInc::new(3, 5));
        assert_eq!(SpanInc::cover(&inc_3_5, &inc_0_2), SpanInc::new(0, 5));
        assert_eq!(SpanInc::cover(&inc_3_5, &inc_1_3), SpanInc::new(1, 5));
        assert_eq!(SpanInc::cover(&inc_3_5, &inc_2_4), SpanInc::new(2, 5));
        assert_eq!(SpanInc::cover(&inc_3_5, &inc_3_5), SpanInc::new(3, 5));
        assert_eq!(SpanInc::cover(&inc_3_5, &empty), SpanInc::new(3, 5));

        assert_eq!(SpanInc::cover(&empty, &exc_0_2), SpanInc::exc(0, 2).unwrap());
        assert_eq!(SpanInc::cover(&empty, &exc_1_3), SpanInc::exc(1, 3).unwrap());
        assert_eq!(SpanInc::cover(&empty, &exc_2_4), SpanInc::exc(2, 4).unwrap());
        assert_eq!(SpanInc::cover(&empty, &exc_3_5), SpanInc::exc(3, 5).unwrap());
        assert_eq!(SpanInc::cover(&empty, &inc_0_2), SpanInc::new(0, 2));
        assert_eq!(SpanInc::cover(&empty, &inc_1_3), SpanInc::new(1, 3));
        assert_eq!(SpanInc::cover(&empty, &inc_2_4), SpanInc::new(2, 4));
        assert_eq!(SpanInc::cover(&empty, &inc_3_5), SpanInc::new(3, 5));
        assert_eq!(SpanInc::cover(&empty, &empty), SpanInc::empty().unwrap());

        // is_empty:
        assert!(!exc_0_2.is_empty());
        assert!(!exc_1_3.is_empty());
        assert!(!exc_2_4.is_empty());
        assert!(!exc_3_5.is_empty());
        assert!(!inc_0_2.is_empty());
        assert!(!inc_1_3.is_empty());
        assert!(!inc_2_4.is_empty());
        assert!(!inc_3_5.is_empty());
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
        assert_eq!(empty.size(), Some(0));
    }
}
