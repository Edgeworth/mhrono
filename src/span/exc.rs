use std::fmt;
use std::ops::{Bound, Range, RangeInclusive, Sub};

use serde::{Deserialize, Serialize};

use crate::span::any::SpanAny;
use crate::span::endpoint::EndpointConversion;
use crate::span::inc::SpanInc;
use crate::span::ops::{pmax, pmin};

#[must_use]
#[derive(
    Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize,
)]
pub struct SpanExc<T> {
    pub st: T,
    pub en: T,
}

impl<T: fmt::Display> fmt::Display for SpanExc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{},{})", self.st, self.en)
    }
}

impl<T: Default> SpanExc<T> {
    pub fn empty() -> Self {
        Self::new(T::default(), T::default())
    }
}

impl<T> SpanExc<T> {
    pub const fn new(st: T, en: T) -> Self {
        Self { st, en }
    }

    #[must_use]
    pub const fn to_bounds_ref(&self) -> (Bound<&T>, Bound<&T>) {
        (Bound::Included(&self.st), Bound::Excluded(&self.en))
    }
}

impl<T: Copy> SpanExc<T> {
    #[must_use]
    pub const fn range(&self) -> Range<T> {
        self.st..self.en
    }

    #[must_use]
    pub const fn to_bounds(&self) -> (Bound<T>, Bound<T>) {
        (Bound::Included(self.st), Bound::Excluded(self.en))
    }

    pub const fn to_any(&self) -> SpanAny<T> {
        SpanAny::exc(self.st, self.en)
    }
}

impl<T: PartialOrd> SpanExc<T> {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.st >= self.en
    }

    #[must_use]
    pub fn contains(&self, t: &T) -> bool {
        &self.st <= t && &self.en > t
    }

    #[must_use]
    pub fn contains_span(&self, s: &Self) -> bool {
        self.st <= s.st && self.en >= s.en
    }
}

impl<T: PartialOrd + Copy> SpanExc<T> {
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

impl<T: EndpointConversion + Copy> SpanExc<T> {
    #[must_use]
    pub fn inc(st: T, en: T) -> Option<Self> {
        <T as EndpointConversion>::to_open(&en, false).map(|en| Self::new(st, en))
    }

    #[must_use]
    pub fn point(p: T) -> Option<Self> {
        Self::inc(p, p)
    }

    #[must_use]
    pub fn to_inc(&self) -> Option<SpanInc<T>> {
        SpanInc::exc(self.st, self.en)
    }

    #[must_use]
    pub fn to_range_inclusive(&self) -> Option<RangeInclusive<T>> {
        self.to_inc().map(|s| s.range_inclusive())
    }
}

impl<T: Sub + Copy> SpanExc<T> {
    #[must_use]
    pub fn size(&self) -> T::Output {
        self.en - self.st
    }
}

impl<T> From<Range<T>> for SpanExc<T> {
    fn from(r: Range<T>) -> Self {
        Self::new(r.start, r.end)
    }
}

impl<T: Copy> From<SpanExc<T>> for Range<T> {
    fn from(s: SpanExc<T>) -> Self {
        s.range()
    }
}

impl<T: EndpointConversion + Copy> TryFrom<RangeInclusive<T>> for SpanExc<T> {
    type Error = ();

    fn try_from(r: RangeInclusive<T>) -> Result<Self, Self::Error> {
        Self::inc(*r.start(), *r.end()).ok_or(())
    }
}

impl<T: EndpointConversion> TryFrom<SpanExc<T>> for RangeInclusive<T> {
    type Error = ();

    fn try_from(s: SpanExc<T>) -> Result<Self, Self::Error> {
        let en = <T as EndpointConversion>::to_closed(&s.en, false).ok_or(())?;
        Ok(s.st..=en)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn ops() {
        let exc_0_2 = SpanExc::<i64>::new(0, 2);
        let exc_1_3 = SpanExc::<i64>::new(1, 3);
        let exc_2_4 = SpanExc::<i64>::new(2, 4);
        let exc_3_5 = SpanExc::<i64>::new(3, 5);
        let inc_0_2 = SpanExc::<i64>::inc(0, 2).unwrap();
        let inc_1_3 = SpanExc::<i64>::inc(1, 3).unwrap();
        let inc_2_4 = SpanExc::<i64>::inc(2, 4).unwrap();
        let inc_3_5 = SpanExc::<i64>::inc(3, 5).unwrap();
        let empty = SpanExc::<i64>::empty();

        // intersect:
        assert_eq!(exc_0_2.intersect(&exc_0_2), Some(SpanExc::new(0, 2)));
        assert_eq!(exc_0_2.intersect(&exc_1_3), Some(SpanExc::new(1, 2)));
        assert_eq!(exc_0_2.intersect(&exc_2_4), None);
        assert_eq!(exc_0_2.intersect(&exc_3_5), None);
        assert_eq!(exc_0_2.intersect(&inc_0_2), Some(SpanExc::new(0, 2)));
        assert_eq!(exc_0_2.intersect(&inc_1_3), Some(SpanExc::new(1, 2)));
        assert_eq!(exc_0_2.intersect(&inc_2_4), None);
        assert_eq!(exc_0_2.intersect(&inc_3_5), None);
        assert_eq!(exc_0_2.intersect(&empty), None);

        assert_eq!(exc_1_3.intersect(&exc_0_2), Some(SpanExc::new(1, 2)));
        assert_eq!(exc_1_3.intersect(&exc_1_3), Some(SpanExc::new(1, 3)));
        assert_eq!(exc_1_3.intersect(&exc_2_4), Some(SpanExc::new(2, 3)));
        assert_eq!(exc_1_3.intersect(&exc_3_5), None);
        assert_eq!(exc_1_3.intersect(&inc_0_2), Some(SpanExc::inc(1, 2).unwrap()));
        assert_eq!(exc_1_3.intersect(&inc_1_3), Some(SpanExc::new(1, 3)));
        assert_eq!(exc_1_3.intersect(&inc_2_4), Some(SpanExc::new(2, 3)));
        assert_eq!(exc_1_3.intersect(&inc_3_5), None);
        assert_eq!(exc_1_3.intersect(&empty), None);

        assert_eq!(exc_2_4.intersect(&exc_0_2), None);
        assert_eq!(exc_2_4.intersect(&exc_1_3), Some(SpanExc::new(2, 3)));
        assert_eq!(exc_2_4.intersect(&exc_2_4), Some(SpanExc::new(2, 4)));
        assert_eq!(exc_2_4.intersect(&exc_3_5), Some(SpanExc::new(3, 4)));
        assert_eq!(exc_2_4.intersect(&inc_0_2), Some(SpanExc::point(2).unwrap()));
        assert_eq!(exc_2_4.intersect(&inc_1_3), Some(SpanExc::inc(2, 3).unwrap()));
        assert_eq!(exc_2_4.intersect(&inc_2_4), Some(SpanExc::new(2, 4)));
        assert_eq!(exc_2_4.intersect(&inc_3_5), Some(SpanExc::new(3, 4)));
        assert_eq!(exc_2_4.intersect(&empty), None);

        assert_eq!(exc_3_5.intersect(&exc_0_2), None);
        assert_eq!(exc_3_5.intersect(&exc_1_3), None);
        assert_eq!(exc_3_5.intersect(&exc_2_4), Some(SpanExc::new(3, 4)));
        assert_eq!(exc_3_5.intersect(&exc_3_5), Some(SpanExc::new(3, 5)));
        assert_eq!(exc_3_5.intersect(&inc_0_2), None);
        assert_eq!(exc_3_5.intersect(&inc_1_3), Some(SpanExc::point(3).unwrap()));
        assert_eq!(exc_3_5.intersect(&inc_2_4), Some(SpanExc::inc(3, 4).unwrap()));
        assert_eq!(exc_3_5.intersect(&inc_3_5), Some(SpanExc::new(3, 5)));
        assert_eq!(exc_3_5.intersect(&empty), None);

        assert_eq!(inc_0_2.intersect(&exc_0_2), Some(SpanExc::new(0, 2)));
        assert_eq!(inc_0_2.intersect(&exc_1_3), Some(SpanExc::inc(1, 2).unwrap()));
        assert_eq!(inc_0_2.intersect(&exc_2_4), Some(SpanExc::point(2).unwrap()));
        assert_eq!(inc_0_2.intersect(&exc_3_5), None);
        assert_eq!(inc_0_2.intersect(&inc_0_2), Some(SpanExc::inc(0, 2).unwrap()));
        assert_eq!(inc_0_2.intersect(&inc_1_3), Some(SpanExc::inc(1, 2).unwrap()));
        assert_eq!(inc_0_2.intersect(&inc_2_4), Some(SpanExc::point(2).unwrap()));
        assert_eq!(inc_0_2.intersect(&inc_3_5), None);
        assert_eq!(inc_0_2.intersect(&empty), None);

        assert_eq!(inc_1_3.intersect(&exc_0_2), Some(SpanExc::new(1, 2)));
        assert_eq!(inc_1_3.intersect(&exc_1_3), Some(SpanExc::new(1, 3)));
        assert_eq!(inc_1_3.intersect(&exc_2_4), Some(SpanExc::inc(2, 3).unwrap()));
        assert_eq!(inc_1_3.intersect(&exc_3_5), Some(SpanExc::point(3).unwrap()));
        assert_eq!(inc_1_3.intersect(&inc_0_2), Some(SpanExc::inc(1, 2).unwrap()));
        assert_eq!(inc_1_3.intersect(&inc_1_3), Some(SpanExc::inc(1, 3).unwrap()));
        assert_eq!(inc_1_3.intersect(&inc_2_4), Some(SpanExc::inc(2, 3).unwrap()));
        assert_eq!(inc_1_3.intersect(&inc_3_5), Some(SpanExc::point(3).unwrap()));
        assert_eq!(inc_1_3.intersect(&empty), None);

        assert_eq!(inc_2_4.intersect(&exc_0_2), None);
        assert_eq!(inc_2_4.intersect(&exc_1_3), Some(SpanExc::new(2, 3)));
        assert_eq!(inc_2_4.intersect(&exc_2_4), Some(SpanExc::new(2, 4)));
        assert_eq!(inc_2_4.intersect(&exc_3_5), Some(SpanExc::inc(3, 4).unwrap()));
        assert_eq!(inc_2_4.intersect(&inc_0_2), Some(SpanExc::point(2).unwrap()));
        assert_eq!(inc_2_4.intersect(&inc_1_3), Some(SpanExc::inc(2, 3).unwrap()));
        assert_eq!(inc_2_4.intersect(&inc_2_4), Some(SpanExc::inc(2, 4).unwrap()));
        assert_eq!(inc_2_4.intersect(&inc_3_5), Some(SpanExc::inc(3, 4).unwrap()));
        assert_eq!(inc_2_4.intersect(&empty), None);

        assert_eq!(inc_3_5.intersect(&exc_0_2), None);
        assert_eq!(inc_3_5.intersect(&exc_1_3), None);
        assert_eq!(inc_3_5.intersect(&exc_2_4), Some(SpanExc::new(3, 4)));
        assert_eq!(inc_3_5.intersect(&exc_3_5), Some(SpanExc::new(3, 5)));
        assert_eq!(inc_3_5.intersect(&inc_0_2), None);
        assert_eq!(inc_3_5.intersect(&inc_1_3), Some(SpanExc::point(3).unwrap()));
        assert_eq!(inc_3_5.intersect(&inc_2_4), Some(SpanExc::inc(3, 4).unwrap()));
        assert_eq!(inc_3_5.intersect(&inc_3_5), Some(SpanExc::inc(3, 5).unwrap()));
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
        assert_eq!(SpanExc::cover(&exc_0_2, &exc_0_2), SpanExc::new(0, 2));
        assert_eq!(SpanExc::cover(&exc_0_2, &exc_1_3), SpanExc::new(0, 3));
        assert_eq!(SpanExc::cover(&exc_0_2, &exc_2_4), SpanExc::new(0, 4));
        assert_eq!(SpanExc::cover(&exc_0_2, &exc_3_5), SpanExc::new(0, 5));
        assert_eq!(SpanExc::cover(&exc_0_2, &inc_0_2), SpanExc::inc(0, 2).unwrap());
        assert_eq!(SpanExc::cover(&exc_0_2, &inc_1_3), SpanExc::inc(0, 3).unwrap());
        assert_eq!(SpanExc::cover(&exc_0_2, &inc_2_4), SpanExc::inc(0, 4).unwrap());
        assert_eq!(SpanExc::cover(&exc_0_2, &inc_3_5), SpanExc::inc(0, 5).unwrap());
        assert_eq!(SpanExc::cover(&exc_0_2, &empty), SpanExc::new(0, 2));

        assert_eq!(SpanExc::cover(&exc_1_3, &exc_0_2), SpanExc::new(0, 3));
        assert_eq!(SpanExc::cover(&exc_1_3, &exc_1_3), SpanExc::new(1, 3));
        assert_eq!(SpanExc::cover(&exc_1_3, &exc_2_4), SpanExc::new(1, 4));
        assert_eq!(SpanExc::cover(&exc_1_3, &exc_3_5), SpanExc::new(1, 5));
        assert_eq!(SpanExc::cover(&exc_1_3, &inc_0_2), SpanExc::new(0, 3));
        assert_eq!(SpanExc::cover(&exc_1_3, &inc_1_3), SpanExc::inc(1, 3).unwrap());
        assert_eq!(SpanExc::cover(&exc_1_3, &inc_2_4), SpanExc::inc(1, 4).unwrap());
        assert_eq!(SpanExc::cover(&exc_1_3, &inc_3_5), SpanExc::inc(1, 5).unwrap());
        assert_eq!(SpanExc::cover(&exc_1_3, &empty), SpanExc::new(1, 3));

        assert_eq!(SpanExc::cover(&exc_2_4, &exc_0_2), SpanExc::new(0, 4));
        assert_eq!(SpanExc::cover(&exc_2_4, &exc_1_3), SpanExc::new(1, 4));
        assert_eq!(SpanExc::cover(&exc_2_4, &exc_2_4), SpanExc::new(2, 4));
        assert_eq!(SpanExc::cover(&exc_2_4, &exc_3_5), SpanExc::new(2, 5));
        assert_eq!(SpanExc::cover(&exc_2_4, &inc_0_2), SpanExc::new(0, 4));
        assert_eq!(SpanExc::cover(&exc_2_4, &inc_1_3), SpanExc::new(1, 4));
        assert_eq!(SpanExc::cover(&exc_2_4, &inc_2_4), SpanExc::inc(2, 4).unwrap());
        assert_eq!(SpanExc::cover(&exc_2_4, &inc_3_5), SpanExc::inc(2, 5).unwrap());
        assert_eq!(SpanExc::cover(&exc_2_4, &empty), SpanExc::new(2, 4));

        assert_eq!(SpanExc::cover(&exc_3_5, &exc_0_2), SpanExc::new(0, 5));
        assert_eq!(SpanExc::cover(&exc_3_5, &exc_1_3), SpanExc::new(1, 5));
        assert_eq!(SpanExc::cover(&exc_3_5, &exc_2_4), SpanExc::new(2, 5));
        assert_eq!(SpanExc::cover(&exc_3_5, &exc_3_5), SpanExc::new(3, 5));
        assert_eq!(SpanExc::cover(&exc_3_5, &inc_0_2), SpanExc::new(0, 5));
        assert_eq!(SpanExc::cover(&exc_3_5, &inc_1_3), SpanExc::new(1, 5));
        assert_eq!(SpanExc::cover(&exc_3_5, &inc_2_4), SpanExc::new(2, 5));
        assert_eq!(SpanExc::cover(&exc_3_5, &inc_3_5), SpanExc::inc(3, 5).unwrap());
        assert_eq!(SpanExc::cover(&exc_3_5, &empty), SpanExc::new(3, 5));

        assert_eq!(SpanExc::cover(&inc_0_2, &exc_0_2), SpanExc::inc(0, 2).unwrap());
        assert_eq!(SpanExc::cover(&inc_0_2, &exc_1_3), SpanExc::new(0, 3));
        assert_eq!(SpanExc::cover(&inc_0_2, &exc_2_4), SpanExc::new(0, 4));
        assert_eq!(SpanExc::cover(&inc_0_2, &exc_3_5), SpanExc::new(0, 5));
        assert_eq!(SpanExc::cover(&inc_0_2, &inc_0_2), SpanExc::inc(0, 2).unwrap());
        assert_eq!(SpanExc::cover(&inc_0_2, &inc_1_3), SpanExc::inc(0, 3).unwrap());
        assert_eq!(SpanExc::cover(&inc_0_2, &inc_2_4), SpanExc::inc(0, 4).unwrap());
        assert_eq!(SpanExc::cover(&inc_0_2, &inc_3_5), SpanExc::inc(0, 5).unwrap());
        assert_eq!(SpanExc::cover(&inc_0_2, &empty), SpanExc::inc(0, 2).unwrap());

        assert_eq!(SpanExc::cover(&inc_1_3, &exc_0_2), SpanExc::inc(0, 3).unwrap());
        assert_eq!(SpanExc::cover(&inc_1_3, &exc_1_3), SpanExc::inc(1, 3).unwrap());
        assert_eq!(SpanExc::cover(&inc_1_3, &exc_2_4), SpanExc::new(1, 4));
        assert_eq!(SpanExc::cover(&inc_1_3, &exc_3_5), SpanExc::new(1, 5));
        assert_eq!(SpanExc::cover(&inc_1_3, &inc_0_2), SpanExc::inc(0, 3).unwrap());
        assert_eq!(SpanExc::cover(&inc_1_3, &inc_1_3), SpanExc::inc(1, 3).unwrap());
        assert_eq!(SpanExc::cover(&inc_1_3, &inc_2_4), SpanExc::inc(1, 4).unwrap());
        assert_eq!(SpanExc::cover(&inc_1_3, &inc_3_5), SpanExc::inc(1, 5).unwrap());
        assert_eq!(SpanExc::cover(&inc_1_3, &empty), SpanExc::inc(1, 3).unwrap());

        assert_eq!(SpanExc::cover(&inc_2_4, &exc_0_2), SpanExc::inc(0, 4).unwrap());
        assert_eq!(SpanExc::cover(&inc_2_4, &exc_1_3), SpanExc::inc(1, 4).unwrap());
        assert_eq!(SpanExc::cover(&inc_2_4, &exc_2_4), SpanExc::inc(2, 4).unwrap());
        assert_eq!(SpanExc::cover(&inc_2_4, &exc_3_5), SpanExc::new(2, 5));
        assert_eq!(SpanExc::cover(&inc_2_4, &inc_0_2), SpanExc::inc(0, 4).unwrap());
        assert_eq!(SpanExc::cover(&inc_2_4, &inc_1_3), SpanExc::inc(1, 4).unwrap());
        assert_eq!(SpanExc::cover(&inc_2_4, &inc_2_4), SpanExc::inc(2, 4).unwrap());
        assert_eq!(SpanExc::cover(&inc_2_4, &inc_3_5), SpanExc::inc(2, 5).unwrap());
        assert_eq!(SpanExc::cover(&inc_2_4, &empty), SpanExc::inc(2, 4).unwrap());

        assert_eq!(SpanExc::cover(&inc_3_5, &exc_0_2), SpanExc::inc(0, 5).unwrap());
        assert_eq!(SpanExc::cover(&inc_3_5, &exc_1_3), SpanExc::inc(1, 5).unwrap());
        assert_eq!(SpanExc::cover(&inc_3_5, &exc_2_4), SpanExc::inc(2, 5).unwrap());
        assert_eq!(SpanExc::cover(&inc_3_5, &exc_3_5), SpanExc::inc(3, 5).unwrap());
        assert_eq!(SpanExc::cover(&inc_3_5, &inc_0_2), SpanExc::inc(0, 5).unwrap());
        assert_eq!(SpanExc::cover(&inc_3_5, &inc_1_3), SpanExc::inc(1, 5).unwrap());
        assert_eq!(SpanExc::cover(&inc_3_5, &inc_2_4), SpanExc::inc(2, 5).unwrap());
        assert_eq!(SpanExc::cover(&inc_3_5, &inc_3_5), SpanExc::inc(3, 5).unwrap());
        assert_eq!(SpanExc::cover(&inc_3_5, &empty), SpanExc::inc(3, 5).unwrap());

        assert_eq!(SpanExc::cover(&empty, &exc_0_2), SpanExc::new(0, 2));
        assert_eq!(SpanExc::cover(&empty, &exc_1_3), SpanExc::new(1, 3));
        assert_eq!(SpanExc::cover(&empty, &exc_2_4), SpanExc::new(2, 4));
        assert_eq!(SpanExc::cover(&empty, &exc_3_5), SpanExc::new(3, 5));
        assert_eq!(SpanExc::cover(&empty, &inc_0_2), SpanExc::inc(0, 2).unwrap());
        assert_eq!(SpanExc::cover(&empty, &inc_1_3), SpanExc::inc(1, 3).unwrap());
        assert_eq!(SpanExc::cover(&empty, &inc_2_4), SpanExc::inc(2, 4).unwrap());
        assert_eq!(SpanExc::cover(&empty, &inc_3_5), SpanExc::inc(3, 5).unwrap());
        assert_eq!(SpanExc::cover(&empty, &empty), SpanExc::empty());

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
        assert_eq!(exc_0_2.size(), 2);
        assert_eq!(exc_1_3.size(), 2);
        assert_eq!(exc_2_4.size(), 2);
        assert_eq!(exc_3_5.size(), 2);
        assert_eq!(inc_0_2.size(), 3);
        assert_eq!(inc_1_3.size(), 3);
        assert_eq!(inc_2_4.size(), 3);
        assert_eq!(inc_3_5.size(), 3);
        assert_eq!(empty.size(), 0);
    }
}
