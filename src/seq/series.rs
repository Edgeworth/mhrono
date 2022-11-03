use std::iter::Map;
use std::ops::RangeBounds;
use std::slice::{Iter, Windows};

use eyre::Result;

use crate::seq::inner::SeriesInner;
use crate::span::any::SpanAny;

pub type XSeries<'a, V, X> = Map<Iter<'a, V>, fn(&V) -> X>;
pub type YSeries<'a, V, Y> = Map<Iter<'a, V>, fn(&V) -> &Y>;

/// Series trait useful for making time series. Stored values can take up a
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
    fn checked_push(&mut self, elt: Self::V) -> Result<bool>;

    #[must_use]
    fn get_y(&self, idx: usize) -> Option<&Self::Y> {
        self.get(idx).map(Self::y)
    }

    #[must_use]
    fn get(&self, idx: usize) -> Option<&Self::V> {
        self.slice().get(idx)
    }

    /// Note: may be expensive if there are other owners.
    #[must_use]
    fn get_mut(&mut self, idx: usize) -> Option<&mut Self::V> {
        self.inner_mut().data_mut().get_mut(idx)
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
        if self.checked_push(elt)? {
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

    /// Find the first thing not less than |x|
    fn lower_bound(&self, x: Self::X) -> Option<&Self::V> {
        self.lower_bound_idx(x).and_then(|idx| self.get(idx))
    }

    /// Find the last element less than or equal to x
    fn lower_bound_last_idx(&self, x: Self::X) -> Option<usize> {
        let idx = self.slice().partition_point(|v| x >= Self::x(v));
        if idx > 0 {
            Some(idx - 1)
        } else {
            None
        }
    }

    /// Find the last element less than or equal to x
    fn lower_bound_last(&self, x: Self::X) -> Option<&Self::V> {
        self.lower_bound_last_idx(x).and_then(|idx| self.get(idx))
    }

    /// Lookup the index of the record which contains |x|. If no such record
    /// exists, look up the record which is immediately before |x|,
    /// if it exists.
    fn lookup_before_idx(&self, x: Self::X) -> Option<usize> {
        let idx = self.upper_bound_idx(x)?;

        if Self::span_of(self.get(idx)?).contains(&x) {
            Some(idx)
        } else if idx > 0 {
            Some(idx - 1)
        } else {
            None
        }
    }

    /// Lookup the record which contains |x|. If no such record exists, look up
    /// the record which is immediately before |x|, if it exists.
    #[must_use]
    fn lookup_before(&self, x: Self::X) -> Option<&Self::V> {
        self.lookup_before_idx(x).and_then(|idx| self.get(idx))
    }

    /// Lookup the index of the record which contains |x|. If no such record
    /// exists, look up the record which is immediately after |x|,
    /// if it exists.
    fn lookup_after_idx(&self, x: Self::X) -> Option<usize> {
        let idx = self.lower_bound_idx(x)?;

        if Self::span_of(self.get(idx)?).contains(&x) {
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
    fn lookup_after(&self, x: Self::X) -> Option<&Self::V> {
        self.lookup_after_idx(x).and_then(|idx| self.get(idx))
    }

    /// Returns (cheaply) a subsequence of the series which contains all
    /// elements fully contained within the given span.
    #[must_use]
    fn subseq(&self, s: SpanAny<Self::X>) -> &[Self::V] {
        let st = self.slice().partition_point(|v| s.st >= Self::span_of(v).st);
        let en = self.slice().partition_point(|v| s.en >= Self::span_of(v).en);
        &self.slice()[st..en]
    }

    /// Returns (cheaply) a subsequence of the series which contains all
    /// elements fully contained within the given span.
    #[must_use]
    fn subseq_series(&self, s: SpanAny<Self::X>) -> Self
    where
        Self: Sized,
    {
        let st = self.slice().partition_point(|v| s.st >= Self::span_of(v).st);
        let en = self.slice().partition_point(|v| s.en >= Self::span_of(v).en);
        self.make_from_inner(self.inner().subseq(st..en))
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
    ($t:ty) => {series_ops!($t, <>);};
    ($t:ty, $($bounds:tt)*) => {
        impl$($bounds)* std::ops::Index<usize> for $t {
            type Output = <Self as Series>::V;

            fn index(&self, index: usize) -> &Self::Output {
                self.get(index).unwrap()
            }
        }

        impl$($bounds)* std::ops::IndexMut<usize> for $t {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                self.get_mut(index).unwrap()
            }
        }
    };
}
