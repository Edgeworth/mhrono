use crate::seq::inner::SeriesInner;
use crate::seq::series::Series;
use crate::span::any::SpanAny;
use crate::time::Time;
use crate::{Result, series_ops};

pub type TimeSeries<Y> = ScalarSeries<Time, Y>;

// ScalarSeries is generic and allowed to contain duplicate values.
#[must_use]
#[derive(Debug, Eq, Default, PartialEq, PartialOrd, Hash, Clone)]
pub struct ScalarSeries<X: PartialOrd + Copy + std::fmt::Display, Y: Clone> {
    inner: SeriesInner<(X, Y)>,
}

impl<X: PartialOrd + Copy + std::fmt::Display, Y: Clone> ScalarSeries<X, Y> {
    pub fn new() -> Self {
        Self { inner: SeriesInner::empty() }
    }
}

impl<X: PartialOrd + Copy + std::fmt::Display, Y: Clone> Series for ScalarSeries<X, Y> {
    type X = X;
    type Y = Y;
    type V = (X, Y);

    fn inner(&self) -> &SeriesInner<Self::V> {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut SeriesInner<Self::V> {
        &mut self.inner
    }

    fn make_from_inner(&self, inner: SeriesInner<Self::V>) -> Self {
        Self { inner }
    }

    fn x(v: &Self::V) -> Self::X {
        v.0
    }

    fn y(v: &Self::V) -> &Self::Y {
        &v.1
    }

    fn span_of(v: &Self::V) -> SpanAny<Self::X> {
        SpanAny::point(v.0)
    }

    fn normalize(&mut self) -> Result<()> {
        // Stable sort since may contain duplicate x values.
        self.inner.data_mut().sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        Ok(())
    }

    fn unchecked_push(&mut self, elt: Self::V) -> Result<bool> {
        let needs_sort = if let Some(last) = self.last() { last.0 > Self::x(&elt) } else { false };
        self.inner.push(elt);
        Ok(needs_sort)
    }
}

series_ops!(ScalarSeries<X, Y>; X: PartialOrd + Copy + std::fmt::Display, Y: Clone);

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

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
    fn scalar_span_at_or_before() -> Result<()> {
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
    fn scalar_span_at_or_before_idx() -> Result<()> {
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
    fn scalar_span_at_or_after() -> Result<()> {
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
    fn scalar_span_at_or_after_idx() -> Result<()> {
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
    fn scalar_span_at_or_before_idx_duplicates() -> Result<()> {
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
    fn scalar_span_at_or_after_idx_duplicates() -> Result<()> {
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
    fn scalar_subseq_unbounded_both() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::unb();
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(2, 10), (5, 20), (8, 30)]);
        Ok(())
    }

    #[test]
    fn scalar_subseq_unbounded_left() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::unb_inc(5);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(2, 10), (5, 20)]);
        Ok(())
    }

    #[test]
    fn scalar_subseq_unbounded_right() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::inc_unb(5);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(5, 20), (8, 30)]);
        Ok(())
    }

    #[test]
    fn scalar_subseq_bounded() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::inc(5, 8);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(5, 20), (8, 30)]);
        Ok(())
    }

    #[test]
    fn scalar_subseq_series_unbounded_both() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::unb();
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 3);
        assert_eq!(subseq_series.get(0), Some(&(2, 10)));
        assert_eq!(subseq_series.get(1), Some(&(5, 20)));
        assert_eq!(subseq_series.get(2), Some(&(8, 30)));
        Ok(())
    }

    #[test]
    fn scalar_subseq_series_unbounded_left() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::unb_inc(5);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(2, 10)));
        assert_eq!(subseq_series.get(1), Some(&(5, 20)));
        Ok(())
    }

    #[test]
    fn scalar_subseq_series_unbounded_right() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::inc_unb(5);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(5, 20)));
        assert_eq!(subseq_series.get(1), Some(&(8, 30)));
        Ok(())
    }

    #[test]
    fn scalar_subseq_series_bounded() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::inc(5, 8);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(5, 20)));
        assert_eq!(subseq_series.get(1), Some(&(8, 30)));
        Ok(())
    }

    #[test]
    fn scalar_subseq_unbounded_left_exc() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::unb_exc(5);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(2, 10)]);
        Ok(())
    }

    #[test]
    fn scalar_subseq_unbounded_right_exc() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::exc_unb(5);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(8, 30)]);
        Ok(())
    }

    #[test]
    fn scalar_subseq_bounded_exc() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::exc(5, 8);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(5, 20)]);
        Ok(())
    }

    #[test]
    fn scalar_subseq_bounded_exc_exc() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::exc_exc(5, 8);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[]);
        Ok(())
    }

    #[test]
    fn scalar_subseq_series_unbounded_left_exc() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::unb_exc(5);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(2, 10)));
        Ok(())
    }

    #[test]
    fn scalar_subseq_series_unbounded_right_exc() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::exc_unb(5);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(8, 30)));
        Ok(())
    }

    #[test]
    fn scalar_subseq_series_bounded_exc() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::exc(5, 8);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(5, 20)));
        Ok(())
    }

    #[test]
    fn scalar_subseq_series_bounded_exc_exc() -> Result<()> {
        let mut series = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;
        series.push((8, 30))?;

        let span = SpanAny::exc_exc(5, 8);
        let subseq_series = series.subseq_series(span);
        assert!(subseq_series.is_empty());
        Ok(())
    }

    #[test]
    fn scalar_span_before_idx_empty_is_none() {
        let series: ScalarSeries<i64, i64> = ScalarSeries::new();
        assert_eq!(series.span_before_idx(0), None);
    }

    #[test]
    fn scalar_span_at_or_before_idx_empty_is_none() {
        let series: ScalarSeries<i64, i64> = ScalarSeries::new();
        assert_eq!(series.span_at_or_before_idx(0), None);
    }

    #[test]
    fn scalar_suffix_zero_is_empty() -> Result<()> {
        let mut series: ScalarSeries<i64, i64> = ScalarSeries::new();
        series.push((2, 10))?;
        series.push((5, 20))?;

        let suffix = series.suffix(0);
        assert!(suffix.is_empty());
        Ok(())
    }
}
