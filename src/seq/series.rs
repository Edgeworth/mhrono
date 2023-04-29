use std::iter::Map;
use std::ops::RangeBounds;
use std::slice::{Iter, Windows};

use eyre::Result;

use crate::seq::inner::SeriesInner;
use crate::span::any::SpanAny;

pub type XSeries<'a, V, X> = Map<Iter<'a, V>, fn(&V) -> X>;
pub type YSeries<'a, V, Y> = Map<Iter<'a, V>, fn(&V) -> &Y>;

/// Series trait useful for making e.g. time series. Stored values can take up a
/// range of X values. Subsequence, lookup, and cloning operations are fast via
/// `SeriesInner`.
pub trait Series {
    /// X coordinate value. Must be sortable.
    type X: PartialOrd + Copy + std::fmt::Display;
    /// Value.
    type Y;
    /// Type for storage.
    type V: Clone;

    /// Returns the `SeriesInner` for this series.
    fn inner(&self) -> &SeriesInner<Self::V>;

    /// Returns a mutable reference to the `SeriesInner` for this series.
    fn inner_mut(&mut self) -> &mut SeriesInner<Self::V>;

    /// Returns a new `Series` of the underlying type from the given `SeriesInner`.
    #[must_use]
    fn make_from_inner(&self, inner: SeriesInner<Self::V>) -> Self;

    /// Returns a representative X value for the given value.
    fn x(v: &Self::V) -> Self::X;

    /// Returns the Y value for the given value.
    fn y(v: &Self::V) -> &Self::Y;

    /// Returns span of X values that the given value takes up. This span must
    /// contain its x value (or be an endpoint of the range), and must not
    /// overlap with any other value's span.
    fn span_of(v: &Self::V) -> SpanAny<Self::X>;

    /// Normalize the underlying data and perform error checking if wanted.
    /// This is called e.g. on modification.
    fn normalize(&mut self) -> Result<()>;

    /// Push the element into the backing storage. Returns true if we need
    /// to normalize the series (e.g. to maintain the sorted order).
    fn unchecked_push(&mut self, elt: Self::V) -> Result<bool>;

    #[must_use]
    fn get_y(&self, idx: usize) -> Option<&Self::Y> {
        self.get(idx).map(Self::y)
    }

    #[must_use]
    fn get(&self, idx: usize) -> Option<&Self::V> {
        self.slice().get(idx)
    }

    fn iter(&self) -> Iter<'_, Self::V> {
        self.slice().iter()
    }

    fn windows(&self, size: usize) -> Windows<'_, Self::V> {
        self.slice().windows(size)
    }

    fn first(&self) -> Option<&Self::V> {
        self.slice().first()
    }

    fn last(&self) -> Option<&Self::V> {
        self.slice().last()
    }

    fn slice(&self) -> &[Self::V] {
        self.inner().slice()
    }

    #[must_use]
    fn len(&self) -> usize {
        self.slice().len()
    }

    #[must_use]
    fn is_empty(&self) -> bool {
        self.slice().is_empty()
    }

    fn xs(&self) -> XSeries<'_, Self::V, Self::X> {
        self.iter().map(|v| Self::x(v))
    }

    fn ys(&self) -> YSeries<'_, Self::V, Self::Y> {
        self.iter().map(|v| Self::y(v))
    }

    /// Pushes a new value into the series.
    fn push(&mut self, elt: Self::V) -> Result<()> {
        if self.unchecked_push(elt)? {
            self.normalize()?;
        }
        Ok(())
    }

    /// Find the first thing greater than |x|
    fn upper_bound_idx(&self, x: Self::X) -> Option<usize> {
        let data = &self.slice();
        let idx = data.partition_point(|v| x >= Self::x(v));
        if idx < data.len() {
            Some(idx)
        } else {
            None
        }
    }

    /// Find the first thing not less than |x|
    fn lower_bound_idx(&self, x: Self::X) -> Option<usize> {
        let data = &self.slice();
        let idx = data.partition_point(|v| Self::x(v) < x);

        if idx < data.len() {
            Some(idx)
        } else {
            None
        }
    }

    /// Find the first thing not less than |x| (>= x).
    fn lower_bound(&self, x: Self::X) -> Option<&Self::V> {
        self.lower_bound_idx(x).and_then(|idx| self.get(idx))
    }

    /// Find the last element less than or equal to x
    fn lower_bound_last_idx(&self, x: Self::X) -> Option<usize> {
        let idx = self.slice().partition_point(|v| x >= Self::x(v));
        idx.checked_sub(1)
    }

    /// Find the last element less than or equal to x
    fn lower_bound_last(&self, x: Self::X) -> Option<&Self::V> {
        self.lower_bound_last_idx(x).and_then(|idx| self.get(idx))
    }

    /// Lookup the index of the record which comes before |x|.
    fn span_before_idx(&self, x: Self::X) -> Option<usize> {
        let idx = self.lower_bound_idx(x).unwrap_or(self.len() - 1);

        if Self::span_of(self.get(idx)?).en >= x {
            idx.checked_sub(1)
        } else {
            Some(idx)
        }
    }

    /// Lookup the record which comes before |x|.
    #[must_use]
    fn span_before(&self, x: Self::X) -> Option<&Self::V> {
        self.span_before_idx(x).and_then(|idx| self.get(idx))
    }

    /// Lookup the index of the record which contains |x|. If no such record
    /// exists, look up the record which is immediately before |x|,
    /// if it exists.
    fn span_at_or_before_idx(&self, x: Self::X) -> Option<usize> {
        let idx = self.upper_bound_idx(x).unwrap_or(self.len() - 1);

        if Self::span_of(self.get(idx)?).en <= x {
            Some(idx)
        } else {
            idx.checked_sub(1)
        }
    }

    /// Lookup the record which contains |x|. If no such record exists, look up
    /// the record which is immediately before |x|, if it exists.
    #[must_use]
    fn span_at_or_before(&self, x: Self::X) -> Option<&Self::V> {
        self.span_at_or_before_idx(x).and_then(|idx| self.get(idx))
    }

    /// Lookup the index of the record which contains |x|. If no such record
    /// exists, look up the record which is immediately after |x|,
    /// if it exists.
    fn span_at_or_after_idx(&self, x: Self::X) -> Option<usize> {
        let idx = self.lower_bound_idx(x)?;

        if Self::span_of(self.get(idx)?).en >= x {
            Some(idx)
        } else if idx + 1 < self.len() {
            Some(idx + 1)
        } else {
            None
        }
    }

    /// Lookup the record which contains |x|. If no such record exists, look up
    /// the record which is immediately after |x|, if it exists.
    #[must_use]
    fn span_at_or_after(&self, x: Self::X) -> Option<&Self::V> {
        self.span_at_or_after_idx(x).and_then(|idx| self.get(idx))
    }

    /// Lookup the index of the record which comes after |x|.
    fn span_after_idx(&self, x: Self::X) -> Option<usize> {
        let idx = self.upper_bound_idx(x)?;

        if Self::span_of(self.get(idx)?).en > x {
            Some(idx)
        } else if idx + 1 < self.len() {
            Some(idx + 1)
        } else {
            None
        }
    }

    /// Lookup the record which comes after |x|.
    #[must_use]
    fn span_after(&self, x: Self::X) -> Option<&Self::V> {
        self.span_after_idx(x).and_then(|idx| self.get(idx))
    }

    /// Returns (cheaply) a subsequence of the series which contains all
    /// elements fully contained within the given span.
    #[must_use]
    fn subseq(&self, s: SpanAny<Self::X>) -> &[Self::V] {
        let st = if s.st.is_left_unbounded() {
            0
        } else {
            self.slice().partition_point(|v| s.st > Self::span_of(v).st)
        };
        let en = if s.en.is_right_unbounded() {
            self.len()
        } else {
            self.slice().partition_point(|v| s.en >= Self::span_of(v).en)
        };
        &self.slice()[st..en]
    }

    /// Returns (cheaply) a subsequence of the series which contains all
    /// elements fully contained within the given span.
    #[must_use]
    fn subseq_series(&self, s: SpanAny<Self::X>) -> Self
    where
        Self: Sized,
    {
        if s.is_unb() {
            return self.make_from_inner(self.inner().clone());
        }
        let st = if s.st.is_left_unbounded() {
            0
        } else {
            self.slice().partition_point(|v| s.st > Self::span_of(v).st)
        };
        let en = if s.en.is_right_unbounded() {
            self.len()
        } else {
            self.slice().partition_point(|v| s.en >= Self::span_of(v).en)
        };
        self.make_from_inner(self.inner().subseq(st..en))
    }

    #[must_use]
    fn prefix(&self, n: usize) -> Self
    where
        Self: Sized,
    {
        self.make_from_inner(self.inner().subseq(..n))
    }

    #[must_use]
    fn suffix(&self, n: usize) -> Self
    where
        Self: Sized,
    {
        let st = self.len().saturating_sub(n);
        self.make_from_inner(self.inner().subseq(st..))
    }

    #[must_use]
    fn subseq_idx(&self, range: impl RangeBounds<usize>) -> &[Self::V] {
        &self.slice()[(range.start_bound().cloned(), range.end_bound().cloned())]
    }

    #[must_use]
    fn subseq_idx_series(&self, range: impl RangeBounds<usize>) -> Self
    where
        Self: Sized,
    {
        self.make_from_inner(self.inner().subseq(range))
    }

    #[must_use]
    fn max_of<R: PartialOrd>(&self, f: impl Fn(&Self::Y) -> R) -> Option<R> {
        self.max_by(&f).map(f)
    }

    #[must_use]
    fn max_by<R: PartialOrd>(&self, f: impl Fn(&Self::Y) -> R) -> Option<&Self::Y> {
        self.iter().max_by(|a, b| f(Self::y(a)).partial_cmp(&f(Self::y(b))).unwrap()).map(Self::y)
    }

    #[must_use]
    fn min_of<R: PartialOrd>(&self, f: impl Fn(&Self::Y) -> R) -> Option<R> {
        self.min_by(&f).map(f)
    }

    #[must_use]
    fn min_by<R: PartialOrd>(&self, f: impl Fn(&Self::Y) -> R) -> Option<&Self::Y> {
        self.iter().min_by(|a, b| f(Self::y(a)).partial_cmp(&f(Self::y(b))).unwrap()).map(Self::y)
    }

    fn span(&self) -> SpanAny<Self::X> {
        SpanAny::cover(&Self::span_of(self.first().unwrap()), &Self::span_of(self.last().unwrap()))
    }
}

#[macro_export]
macro_rules! series_ops {
    ($t:ty) => { series_ops!($t;); };
    ($t:ty; $($bounds:tt)*) => {
        impl<$($bounds)*> std::ops::Index<usize> for $t {
            type Output = <Self as Series>::V;

            fn index(&self, index: usize) -> &Self::Output {
                self.get(index).unwrap()
            }
        }

        impl<'a, $($bounds)*> IntoIterator for &'a $t {
            type Item = &'a <$t as Series>::V;
            type IntoIter = std::slice::Iter<'a, <$t as Series>::V>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use eyre::Result;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::seq::scalar_series::ScalarSeries;

    #[test]
    fn scalar_upper_bound_idx() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.upper_bound_idx(4), Some(1));
        assert_eq!(series.upper_bound_idx(5), Some(2));
        assert_eq!(series.upper_bound_idx(1), Some(0));
        assert_eq!(series.upper_bound_idx(10), None);

        Ok(())
    }

    #[test]
    fn scalar_lower_bound_idx() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.lower_bound_idx(4), Some(1));
        assert_eq!(series.lower_bound_idx(5), Some(1));
        assert_eq!(series.lower_bound_idx(1), Some(0));
        assert_eq!(series.lower_bound_idx(10), None);

        Ok(())
    }

    #[test]
    fn scalar_lower_bound() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.lower_bound(4), Some(&(5, 20)));
        assert_eq!(series.lower_bound(5), Some(&(5, 20)));
        assert_eq!(series.lower_bound(1), Some(&(2, 10)));
        assert_eq!(series.lower_bound(10), None);

        Ok(())
    }

    #[test]
    fn scalar_lower_bound_last_idx() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.lower_bound_last_idx(4), Some(0));
        assert_eq!(series.lower_bound_last_idx(5), Some(1));
        assert_eq!(series.lower_bound_last_idx(1), None);
        assert_eq!(series.lower_bound_last_idx(8), Some(2));

        Ok(())
    }

    #[test]
    fn scalar_lower_bound_last() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.lower_bound_last(4), Some(&(2, 10)));
        assert_eq!(series.lower_bound_last(5), Some(&(5, 20)));
        assert_eq!(series.lower_bound_last(1), None);
        assert_eq!(series.lower_bound_last(8), Some(&(8, 30)));

        Ok(())
    }

    #[test]
    fn scalar_span_before_idx() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.span_before_idx(0), None);
        assert_eq!(series.span_before_idx(1), None);
        assert_eq!(series.span_before_idx(2), None);
        assert_eq!(series.span_before_idx(3), Some(0));
        assert_eq!(series.span_before_idx(4), Some(0));
        assert_eq!(series.span_before_idx(5), Some(0));
        assert_eq!(series.span_before_idx(6), Some(1));
        assert_eq!(series.span_before_idx(7), Some(1));
        assert_eq!(series.span_before_idx(8), Some(1));
        assert_eq!(series.span_before_idx(9), Some(2));

        Ok(())
    }

    #[test]
    fn scalar_span_before() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.span_before(0), None);
        assert_eq!(series.span_before(1), None);
        assert_eq!(series.span_before(2), None);
        assert_eq!(series.span_before(3), Some(&(2, 10)));
        assert_eq!(series.span_before(4), Some(&(2, 10)));
        assert_eq!(series.span_before(5), Some(&(2, 10)));
        assert_eq!(series.span_before(6), Some(&(5, 20)));
        assert_eq!(series.span_before(7), Some(&(5, 20)));
        assert_eq!(series.span_before(8), Some(&(5, 20)));
        assert_eq!(series.span_before(9), Some(&(8, 30)));

        Ok(())
    }

    #[test]
    fn scalar_span_after_idx() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.span_after_idx(0), Some(0));
        assert_eq!(series.span_after_idx(1), Some(0));
        assert_eq!(series.span_after_idx(2), Some(1));
        assert_eq!(series.span_after_idx(3), Some(1));
        assert_eq!(series.span_after_idx(4), Some(1));
        assert_eq!(series.span_after_idx(5), Some(2));
        assert_eq!(series.span_after_idx(6), Some(2));
        assert_eq!(series.span_after_idx(7), Some(2));
        assert_eq!(series.span_after_idx(8), None);
        assert_eq!(series.span_after_idx(9), None);

        Ok(())
    }

    #[test]
    fn scalar_span_after() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.span_after(0), Some(&(2, 10)));
        assert_eq!(series.span_after(1), Some(&(2, 10)));
        assert_eq!(series.span_after(2), Some(&(5, 20)));
        assert_eq!(series.span_after(3), Some(&(5, 20)));
        assert_eq!(series.span_after(4), Some(&(5, 20)));
        assert_eq!(series.span_after(5), Some(&(8, 30)));
        assert_eq!(series.span_after(6), Some(&(8, 30)));
        assert_eq!(series.span_after(7), Some(&(8, 30)));
        assert_eq!(series.span_after(8), None);
        assert_eq!(series.span_after(9), None);

        Ok(())
    }

    #[test]
    fn test_scalar_span_at_or_before() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.span_at_or_before(0), None);
        assert_eq!(series.span_at_or_before(1), None);
        assert_eq!(series.span_at_or_before(2), Some(&(2, 10)));
        assert_eq!(series.span_at_or_before(3), Some(&(2, 10)));
        assert_eq!(series.span_at_or_before(5), Some(&(5, 20)));
        assert_eq!(series.span_at_or_before(7), Some(&(5, 20)));
        assert_eq!(series.span_at_or_before(8), Some(&(8, 30)));
        assert_eq!(series.span_at_or_before(9), Some(&(8, 30)));

        Ok(())
    }

    #[test]
    fn test_scalar_span_at_or_before_idx() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.span_at_or_before_idx(0), None);
        assert_eq!(series.span_at_or_before_idx(1), None);
        assert_eq!(series.span_at_or_before_idx(2), Some(0));
        assert_eq!(series.span_at_or_before_idx(3), Some(0));
        assert_eq!(series.span_at_or_before_idx(5), Some(1));
        assert_eq!(series.span_at_or_before_idx(7), Some(1));
        assert_eq!(series.span_at_or_before_idx(8), Some(2));
        assert_eq!(series.span_at_or_before_idx(9), Some(2));

        Ok(())
    }

    #[test]
    fn test_scalar_span_at_or_after() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.span_at_or_after(0), Some(&(2, 10)));
        assert_eq!(series.span_at_or_after(1), Some(&(2, 10)));
        assert_eq!(series.span_at_or_after(2), Some(&(2, 10)));
        assert_eq!(series.span_at_or_after(3), Some(&(5, 20)));
        assert_eq!(series.span_at_or_after(5), Some(&(5, 20)));
        assert_eq!(series.span_at_or_after(7), Some(&(8, 30)));
        assert_eq!(series.span_at_or_after(8), Some(&(8, 30)));
        assert_eq!(series.span_at_or_after(9), None);

        Ok(())
    }

    #[test]
    fn test_scalar_span_at_or_after_idx() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.span_at_or_after_idx(0), Some(0));
        assert_eq!(series.span_at_or_after_idx(1), Some(0));
        assert_eq!(series.span_at_or_after_idx(2), Some(0));
        assert_eq!(series.span_at_or_after_idx(3), Some(1));
        assert_eq!(series.span_at_or_after_idx(5), Some(1));
        assert_eq!(series.span_at_or_after_idx(7), Some(2));
        assert_eq!(series.span_at_or_after_idx(8), Some(2));
        assert_eq!(series.span_at_or_after_idx(9), None);

        Ok(())
    }

    #[test]
    fn scalar_upper_bound_idx_duplicates() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((5, 21))?;
        series.push((8, 30))?;

        assert_eq!(series.upper_bound_idx(4), Some(1));
        assert_eq!(series.upper_bound_idx(5), Some(3));
        assert_eq!(series.upper_bound_idx(1), Some(0));
        assert_eq!(series.upper_bound_idx(10), None);

        Ok(())
    }

    #[test]
    fn scalar_lower_bound_idx_duplicates() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((5, 21))?;
        series.push((8, 30))?;

        assert_eq!(series.lower_bound_idx(4), Some(1));
        assert_eq!(series.lower_bound_idx(5), Some(1));
        assert_eq!(series.lower_bound_idx(1), Some(0));
        assert_eq!(series.lower_bound_idx(10), None);

        Ok(())
    }

    #[test]
    fn scalar_lower_bound_last_idx_duplicates() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((5, 21))?;
        series.push((8, 30))?;

        assert_eq!(series.lower_bound_last_idx(4), Some(0));
        assert_eq!(series.lower_bound_last_idx(5), Some(2));
        assert_eq!(series.lower_bound_last_idx(1), None);
        assert_eq!(series.lower_bound_last_idx(8), Some(3));

        Ok(())
    }

    #[test]
    fn scalar_span_before_idx_duplicates() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((5, 21))?;
        series.push((8, 30))?;

        assert_eq!(series.span_before_idx(0), None);
        assert_eq!(series.span_before_idx(1), None);
        assert_eq!(series.span_before_idx(2), None);
        assert_eq!(series.span_before_idx(3), Some(0));
        assert_eq!(series.span_before_idx(4), Some(0));
        assert_eq!(series.span_before_idx(5), Some(0));
        assert_eq!(series.span_before_idx(6), Some(2));
        assert_eq!(series.span_before_idx(7), Some(2));
        assert_eq!(series.span_before_idx(8), Some(2));
        assert_eq!(series.span_before_idx(9), Some(3));

        Ok(())
    }

    #[test]
    fn scalar_span_after_idx_duplicates() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((5, 21))?;
        series.push((8, 30))?;

        assert_eq!(series.span_after_idx(0), Some(0));
        assert_eq!(series.span_after_idx(1), Some(0));
        assert_eq!(series.span_after_idx(2), Some(1));
        assert_eq!(series.span_after_idx(3), Some(1));
        assert_eq!(series.span_after_idx(4), Some(1));
        assert_eq!(series.span_after_idx(5), Some(3));
        assert_eq!(series.span_after_idx(6), Some(3));
        assert_eq!(series.span_after_idx(7), Some(3));
        assert_eq!(series.span_after_idx(8), None);
        assert_eq!(series.span_after_idx(9), None);

        Ok(())
    }

    #[test]
    fn test_scalar_span_at_or_before_idx_duplicates() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((5, 21))?;
        series.push((8, 30))?;

        assert_eq!(series.span_at_or_before_idx(0), None);
        assert_eq!(series.span_at_or_before_idx(1), None);
        assert_eq!(series.span_at_or_before_idx(2), Some(0));
        assert_eq!(series.span_at_or_before_idx(3), Some(0));
        assert_eq!(series.span_at_or_before_idx(5), Some(2));
        assert_eq!(series.span_at_or_before_idx(7), Some(2));
        assert_eq!(series.span_at_or_before_idx(8), Some(3));
        assert_eq!(series.span_at_or_before_idx(9), Some(3));

        Ok(())
    }

    #[test]
    fn test_scalar_span_at_or_after_idx_duplicates() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((5, 21))?;
        series.push((8, 30))?;

        assert_eq!(series.span_at_or_after_idx(0), Some(0));
        assert_eq!(series.span_at_or_after_idx(1), Some(0));
        assert_eq!(series.span_at_or_after_idx(2), Some(0));
        assert_eq!(series.span_at_or_after_idx(3), Some(1));
        assert_eq!(series.span_at_or_after_idx(5), Some(1));
        assert_eq!(series.span_at_or_after_idx(7), Some(3));
        assert_eq!(series.span_at_or_after_idx(8), Some(3));
        assert_eq!(series.span_at_or_after_idx(9), None);

        Ok(())
    }

    #[test]
    fn scalar_only_duplicates() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((5, 20))?;
        series.push((5, 21))?;

        // upper_bound_idx
        assert_eq!(series.upper_bound_idx(4), Some(0));
        assert_eq!(series.upper_bound_idx(5), None);
        assert_eq!(series.upper_bound_idx(6), None);

        // lower_bound_idx
        assert_eq!(series.lower_bound_idx(4), Some(0));
        assert_eq!(series.lower_bound_idx(5), Some(0));
        assert_eq!(series.lower_bound_idx(6), None);

        // lower_bound_last_idx
        assert_eq!(series.lower_bound_last_idx(4), None);
        assert_eq!(series.lower_bound_last_idx(5), Some(1));
        assert_eq!(series.lower_bound_last_idx(6), Some(1));

        // span_before_idx
        assert_eq!(series.span_before_idx(4), None);
        assert_eq!(series.span_before_idx(5), None);
        assert_eq!(series.span_before_idx(6), Some(1));

        // span_after_idx
        assert_eq!(series.span_after_idx(4), Some(0));
        assert_eq!(series.span_after_idx(5), None);
        assert_eq!(series.span_after_idx(6), None);

        // span_at_or_before_idx
        assert_eq!(series.span_at_or_before_idx(4), None);
        assert_eq!(series.span_at_or_before_idx(5), Some(1));
        assert_eq!(series.span_at_or_before_idx(6), Some(1));

        // span_at_or_after_idx
        assert_eq!(series.span_at_or_after_idx(4), Some(0));
        assert_eq!(series.span_at_or_after_idx(5), Some(0));
        assert_eq!(series.span_at_or_after_idx(6), None);

        Ok(())
    }

    #[test]
    fn scalar_push() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        assert_eq!(series.len(), 3);
        assert_eq!(series.get(0), Some(&(2, 10)));
        assert_eq!(series.get(1), Some(&(5, 20)));
        assert_eq!(series.get(2), Some(&(8, 30)));

        Ok(())
    }

    #[test]
    fn subseq_unbounded_both() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::unb();

        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(2, 10), (5, 20), (8, 30)]);
    }

    #[test]
    fn subseq_unbounded_left() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::unb_inc(5);

        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(2, 10), (5, 20)]);
    }

    #[test]
    fn scalar_subseq_unbounded_right() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::inc_unb(5);

        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(5, 20), (8, 30)]);
    }

    #[test]
    fn scalar_subseq_bounded() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::inc(5, 8);

        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(5, 20), (8, 30)]);
    }

    #[test]
    fn scalar_subseq_series_unbounded_both() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::unb();

        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 3);
        assert_eq!(subseq_series.get(0), Some(&(2, 10)));
        assert_eq!(subseq_series.get(1), Some(&(5, 20)));
        assert_eq!(subseq_series.get(2), Some(&(8, 30)));
    }

    #[test]
    fn scalar_subseq_series_unbounded_left() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::unb_inc(5);

        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(2, 10)));
        assert_eq!(subseq_series.get(1), Some(&(5, 20)));
    }

    #[test]
    fn scalar_subseq_series_unbounded_right() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::inc_unb(5);

        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(5, 20)));
        assert_eq!(subseq_series.get(1), Some(&(8, 30)));
    }

    #[test]
    fn scalar_subseq_series_bounded() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::inc(5, 8);

        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(5, 20)));
        assert_eq!(subseq_series.get(1), Some(&(8, 30)));
    }

    #[test]
    fn subseq_unbounded_left_exc() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::unb_exc(5);

        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(2, 10)]);
    }

    #[test]
    fn scalar_subseq_unbounded_right_exc() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::exc_unb(5);

        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(8, 30)]);
    }

    #[test]
    fn scalar_subseq_bounded_exc() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::exc(5, 8);

        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(5, 20)]);
    }

    #[test]
    fn scalar_subseq_bounded_exc_exc() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::exc_exc(5, 8);

        let subseq = series.subseq(span);
        assert_eq!(subseq, &[]);
    }

    #[test]
    fn scalar_subseq_series_unbounded_left_exc() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::unb_exc(5);

        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(2, 10)));
    }

    #[test]
    fn scalar_subseq_series_unbounded_right_exc() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::exc_unb(5);

        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(8, 30)));
    }

    #[test]
    fn scalar_subseq_series_bounded_exc() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::exc(5, 8);

        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(5, 20)));
    }

    #[test]
    fn scalar_subseq_series_bounded_exc_exc() {
        let mut series = ScalarSeries::new();
        series.push((2, 10)).unwrap();
        series.push((5, 20)).unwrap();
        series.push((8, 30)).unwrap();
        let span = SpanAny::exc_exc(5, 8);

        let subseq_series = series.subseq_series(span);
        assert!(subseq_series.is_empty());
    }
}
