use std::cmp::{Eq, PartialEq, PartialOrd};
use std::hash::Hash;

use eyre::Result;

use crate::seq::inner::SeriesInner;
use crate::seq::series::Series;
use crate::span::any::SpanAny;
use crate::span::endpoint::EndpointConversion;
use crate::span::exc::SpanExc;

#[must_use]
#[derive(Debug, Eq, Default, PartialEq, PartialOrd, Hash, Clone)]
pub struct SpanExcSeries<X: PartialOrd + Copy + std::fmt::Display, Y: Clone> {
    inner: SeriesInner<(SpanExc<X>, Y)>,
}

impl<X: PartialOrd + Copy + std::fmt::Display, Y: Clone> SpanExcSeries<X, Y> {
    pub fn new() -> Self {
        Self { inner: SeriesInner::empty() }
    }
}

impl<X: PartialOrd + Copy + std::fmt::Display, Y: Clone> Series for SpanExcSeries<X, Y> {
    type X = X;
    type Y = Y;
    type V = (SpanExc<X>, Y);

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
        v.0.st
    }

    fn y(v: &Self::V) -> &Self::Y {
        &v.1
    }

    fn span_of(v: &Self::V) -> SpanAny<Self::X> {
        v.0.to_any()
    }

    fn normalize(&mut self) -> Result<()> {
        // Stable sort since may contain duplicate x values.
        self.inner.data_mut().sort_by(|a, b| Self::x(a).partial_cmp(&Self::x(b)).unwrap());
        Ok(())
    }

    fn unchecked_push(&mut self, elt: Self::V) -> Result<bool> {
        let needs_sort =
            if let Some(last) = self.last() { Self::x(last) >= Self::x(&elt) } else { false };
        self.inner.push(elt);
        Ok(needs_sort)
    }
}

#[must_use]
#[derive(Debug, Eq, Default, PartialEq, PartialOrd, Hash, Clone)]
pub struct SpanExcSeriesRight<X: PartialOrd + Copy + std::fmt::Display, Y: Clone> {
    inner: SeriesInner<(SpanExc<X>, Y)>,
}

impl<X: PartialOrd + Copy + std::fmt::Display, Y: Clone> SpanExcSeriesRight<X, Y> {
    pub fn new() -> Self {
        Self { inner: SeriesInner::empty() }
    }
}

impl<X: PartialOrd + Copy + std::fmt::Display + EndpointConversion, Y: Clone> Series
    for SpanExcSeriesRight<X, Y>
{
    type X = X;
    type Y = Y;
    type V = (SpanExc<X>, Y);

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
        v.0.to_inc().unwrap().en
    }

    fn y(v: &Self::V) -> &Self::Y {
        &v.1
    }

    fn span_of(v: &Self::V) -> SpanAny<Self::X> {
        v.0.to_any()
    }

    fn normalize(&mut self) -> Result<()> {
        // Stable sort since may contain duplicate x values.
        self.inner.data_mut().sort_by(|a, b| Self::x(a).partial_cmp(&Self::x(b)).unwrap());
        Ok(())
    }

    fn unchecked_push(&mut self, elt: Self::V) -> Result<bool> {
        let needs_sort =
            if let Some(last) = self.last() { Self::x(last) >= Self::x(&elt) } else { false };
        self.inner.push(elt);
        Ok(needs_sort)
    }
}

#[cfg(test)]
mod tests {
    use eyre::Result;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn span_exc_upper_bound_idx() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.upper_bound_idx(4), Some(1));
        assert_eq!(series.upper_bound_idx(5), Some(2));
        assert_eq!(series.upper_bound_idx(1), Some(0));
        assert_eq!(series.upper_bound_idx(10), None);

        Ok(())
    }

    #[test]
    fn span_exc_lower_bound_idx() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.lower_bound_idx(4), Some(1));
        assert_eq!(series.lower_bound_idx(5), Some(1));
        assert_eq!(series.lower_bound_idx(1), Some(0));
        assert_eq!(series.lower_bound_idx(10), None);

        Ok(())
    }

    #[test]
    fn span_exc_lower_bound_last_idx() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.lower_bound_last_idx(4), Some(0));
        assert_eq!(series.lower_bound_last_idx(5), Some(1));
        assert_eq!(series.lower_bound_last_idx(1), None);
        assert_eq!(series.lower_bound_last_idx(8), Some(2));

        Ok(())
    }

    #[test]
    fn span_exc_span_before_idx() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

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
    fn span_exc_span_after_idx() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

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
    fn span_exc_span_at_or_before_idx() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.span_at_or_before_idx(0), None);
        assert_eq!(series.span_at_or_before_idx(1), None);
        assert_eq!(series.span_at_or_before_idx(2), Some(0));
        assert_eq!(series.span_at_or_before_idx(3), Some(0));
        assert_eq!(series.span_at_or_before_idx(5), Some(1));
        assert_eq!(series.span_at_or_before_idx(7), Some(1));
        assert_eq!(series.span_at_or_before_idx(8), Some(2));
        assert_eq!(series.span_at_or_before_idx(9), Some(2));
        assert_eq!(series.span_at_or_before_idx(10), Some(2));

        Ok(())
    }

    #[test]
    fn span_exc_span_at_or_after_idx() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.span_at_or_after_idx(0), Some(0));
        assert_eq!(series.span_at_or_after_idx(1), Some(0));
        assert_eq!(series.span_at_or_after_idx(2), Some(0));
        assert_eq!(series.span_at_or_after_idx(3), Some(1));
        assert_eq!(series.span_at_or_after_idx(4), Some(1));
        assert_eq!(series.span_at_or_after_idx(5), Some(1));
        assert_eq!(series.span_at_or_after_idx(6), Some(2));
        assert_eq!(series.span_at_or_after_idx(7), Some(2));
        assert_eq!(series.span_at_or_after_idx(8), Some(2));
        assert_eq!(series.span_at_or_after_idx(9), None);

        Ok(())
    }

    #[test]
    fn span_exc_upper_bound_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.upper_bound_idx(4), Some(1));
        assert_eq!(series.upper_bound_idx(5), Some(3));
        assert_eq!(series.upper_bound_idx(1), Some(0));
        assert_eq!(series.upper_bound_idx(10), None);

        Ok(())
    }

    #[test]
    fn span_exc_lower_bound_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.lower_bound_idx(4), Some(1));
        assert_eq!(series.lower_bound_idx(5), Some(1));
        assert_eq!(series.lower_bound_idx(1), Some(0));
        assert_eq!(series.lower_bound_idx(10), None);

        Ok(())
    }

    #[test]
    fn span_exc_lower_bound_last_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.lower_bound_last_idx(4), Some(0));
        assert_eq!(series.lower_bound_last_idx(5), Some(2));
        assert_eq!(series.lower_bound_last_idx(1), None);
        assert_eq!(series.lower_bound_last_idx(8), Some(3));

        Ok(())
    }

    #[test]
    fn span_exc_span_before_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

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
    fn span_exc_span_after_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

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
    fn span_exc_span_at_or_before_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.span_at_or_before_idx(0), None);
        assert_eq!(series.span_at_or_before_idx(1), None);
        assert_eq!(series.span_at_or_before_idx(2), Some(0));
        assert_eq!(series.span_at_or_before_idx(3), Some(0));
        assert_eq!(series.span_at_or_before_idx(4), Some(0));
        assert_eq!(series.span_at_or_before_idx(5), Some(2));
        assert_eq!(series.span_at_or_before_idx(6), Some(2));
        assert_eq!(series.span_at_or_before_idx(7), Some(2));
        assert_eq!(series.span_at_or_before_idx(8), Some(3));
        assert_eq!(series.span_at_or_before_idx(9), Some(3));

        Ok(())
    }

    #[test]
    fn span_exc_span_at_or_after_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.span_at_or_after_idx(0), Some(0));
        assert_eq!(series.span_at_or_after_idx(1), Some(0));
        assert_eq!(series.span_at_or_after_idx(2), Some(0));
        assert_eq!(series.span_at_or_after_idx(3), Some(1));
        assert_eq!(series.span_at_or_after_idx(4), Some(1));
        assert_eq!(series.span_at_or_after_idx(5), Some(1));
        assert_eq!(series.span_at_or_after_idx(6), Some(3));
        assert_eq!(series.span_at_or_after_idx(7), Some(3));
        assert_eq!(series.span_at_or_after_idx(8), Some(3));
        assert_eq!(series.span_at_or_after_idx(9), None);

        Ok(())
    }

    #[test]
    fn span_exc_only_duplicates() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;

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
    fn span_exc_push() -> Result<()> {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.len(), 3);
        assert_eq!(series.get(0), Some(&(SpanExc::new(2, 3), 10)));
        assert_eq!(series.get(1), Some(&(SpanExc::new(5, 6), 20)));
        assert_eq!(series.get(2), Some(&(SpanExc::new(8, 9), 30)));

        Ok(())
    }

    #[test]
    fn span_exc_subseq_unbounded_both() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();
        let span = SpanAny::unb();

        let subseq = series.subseq(span);
        assert_eq!(
            subseq,
            &[(SpanExc::new(2, 3), 10), (SpanExc::new(5, 6), 20), (SpanExc::new(8, 9), 30)]
        );
    }

    #[test]
    fn span_exc_subseq_unbounded_left() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::unb_inc(5);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(2, 3), 10)]);

        let span = SpanAny::unb_inc(6);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(2, 3), 10), (SpanExc::new(5, 6), 20)]);
    }

    #[test]
    fn span_exc_subseq_unbounded_right() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::inc_unb(5);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(5, 6), 20), (SpanExc::new(8, 9), 30)]);

        let span = SpanAny::inc_unb(6);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(8, 9), 30)]);
    }

    #[test]
    fn span_exc_subseq_bounded() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::inc(5, 9);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(5, 6), 20), (SpanExc::new(8, 9), 30)]);

        let span = SpanAny::inc(6, 8);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[]);
    }

    #[test]
    fn span_exc_subseq_series_unbounded_both() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();
        let span = SpanAny::unb();

        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 3);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(2, 3), 10)));
        assert_eq!(subseq_series.get(1), Some(&(SpanExc::new(5, 6), 20)));
        assert_eq!(subseq_series.get(2), Some(&(SpanExc::new(8, 9), 30)));
    }

    #[test]
    fn span_exc_subseq_series_unbounded_left() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::unb_inc(5);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(2, 3), 10)));

        let span = SpanAny::unb_inc(6);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(2, 3), 10)));
        assert_eq!(subseq_series.get(1), Some(&(SpanExc::new(5, 6), 20)));
    }

    #[test]
    fn span_exc_subseq_series_unbounded_right() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::inc_unb(5);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(5, 6), 20)));
        assert_eq!(subseq_series.get(1), Some(&(SpanExc::new(8, 9), 30)));

        let span = SpanAny::inc_unb(6);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(8, 9), 30)));
    }

    #[test]
    fn span_exc_subseq_series_bounded() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::inc(5, 9);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(5, 6), 20)));
        assert_eq!(subseq_series.get(1), Some(&(SpanExc::new(8, 9), 30)));

        let span = SpanAny::inc(6, 8);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 0);
    }

    #[test]
    fn span_exc_subseq_unbounded_left_exc() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::unb_exc(5);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(2, 3), 10)]);

        let span = SpanAny::unb_exc(6);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(2, 3), 10), (SpanExc::new(5, 6), 20)]);
    }

    #[test]
    fn span_exc_subseq_unbounded_right_exc() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::exc_unb(5);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(8, 9), 30)]);

        let span = SpanAny::exc_unb(6);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(8, 9), 30)]);
    }

    #[test]
    fn span_exc_subseq_bounded_exc() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::exc(5, 8);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(5, 6), 20)]);

        let span = SpanAny::exc(6, 9);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(8, 9), 30)]);
    }

    #[test]
    fn span_exc_subseq_bounded_exc_exc() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();
        let span = SpanAny::exc_exc(5, 8);

        let subseq = series.subseq(span);
        assert_eq!(subseq, &[]);

        let span = SpanAny::exc_exc(6, 9);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(8, 9), 30)]);
    }

    #[test]
    fn span_exc_subseq_series_unbounded_left_exc() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::unb_exc(5);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(2, 3), 10)));

        let span = SpanAny::unb_exc(6);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(2, 3), 10)));
        assert_eq!(subseq_series.get(1), Some(&(SpanExc::new(5, 6), 20)));
    }

    #[test]
    fn span_exc_subseq_series_unbounded_right_exc() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::exc_unb(5);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(8, 9), 30)));

        let span = SpanAny::exc_unb(6);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(8, 9), 30)));
    }

    #[test]
    fn span_exc_subseq_series_bounded_exc() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::exc(5, 8);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(5, 6), 20)));

        let span = SpanAny::exc(6, 9);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(8, 9), 30)));
    }

    #[test]
    fn span_exc_subseq_series_bounded_exc_exc() {
        let mut series = SpanExcSeries::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::exc_exc(5, 8);
        let subseq_series = series.subseq_series(span);
        assert!(subseq_series.is_empty());

        let span = SpanAny::exc_exc(6, 9);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(8, 9), 30)));
    }

    #[test]
    fn span_exc_right_upper_bound_idx() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.upper_bound_idx(3), Some(1));
        assert_eq!(series.upper_bound_idx(6), Some(2));
        assert_eq!(series.upper_bound_idx(1), Some(0));
        assert_eq!(series.upper_bound_idx(9), None);

        Ok(())
    }

    #[test]
    fn span_exc_right_lower_bound_idx() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.lower_bound_idx(4), Some(1));
        assert_eq!(series.lower_bound_idx(6), Some(2));
        assert_eq!(series.lower_bound_idx(1), Some(0));
        assert_eq!(series.lower_bound_idx(9), None);

        Ok(())
    }

    #[test]
    fn span_exc_right_lower_bound_last_idx() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.lower_bound_last_idx(4), Some(0));
        assert_eq!(series.lower_bound_last_idx(6), Some(1));
        assert_eq!(series.lower_bound_last_idx(1), None);
        assert_eq!(series.lower_bound_last_idx(9), Some(2));

        Ok(())
    }

    #[test]
    fn span_exc_right_span_before_idx() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

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
    fn span_exc_right_span_after_idx() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

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
    fn span_exc_right_span_at_or_before_idx() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.span_at_or_before_idx(0), None);
        assert_eq!(series.span_at_or_before_idx(1), None);
        assert_eq!(series.span_at_or_before_idx(2), Some(0));
        assert_eq!(series.span_at_or_before_idx(3), Some(0));
        assert_eq!(series.span_at_or_before_idx(5), Some(1));
        assert_eq!(series.span_at_or_before_idx(7), Some(1));
        assert_eq!(series.span_at_or_before_idx(8), Some(2));
        assert_eq!(series.span_at_or_before_idx(9), Some(2));
        assert_eq!(series.span_at_or_before_idx(10), Some(2));

        Ok(())
    }

    #[test]
    fn span_exc_right_span_at_or_after_idx() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.span_at_or_after_idx(0), Some(0));
        assert_eq!(series.span_at_or_after_idx(1), Some(0));
        assert_eq!(series.span_at_or_after_idx(2), Some(0));
        assert_eq!(series.span_at_or_after_idx(3), Some(1));
        assert_eq!(series.span_at_or_after_idx(4), Some(1));
        assert_eq!(series.span_at_or_after_idx(5), Some(1));
        assert_eq!(series.span_at_or_after_idx(6), Some(2));
        assert_eq!(series.span_at_or_after_idx(7), Some(2));
        assert_eq!(series.span_at_or_after_idx(8), Some(2));
        assert_eq!(series.span_at_or_after_idx(9), None);

        Ok(())
    }

    #[test]
    fn span_exc_right_upper_bound_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.upper_bound_idx(4), Some(1));
        assert_eq!(series.upper_bound_idx(5), Some(3));
        assert_eq!(series.upper_bound_idx(6), Some(3));
        assert_eq!(series.upper_bound_idx(1), Some(0));
        assert_eq!(series.upper_bound_idx(10), None);

        Ok(())
    }

    #[test]
    fn span_exc_right_lower_bound_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.lower_bound_idx(4), Some(1));
        assert_eq!(series.lower_bound_idx(5), Some(1));
        assert_eq!(series.lower_bound_idx(6), Some(3));
        assert_eq!(series.lower_bound_idx(1), Some(0));
        assert_eq!(series.lower_bound_idx(10), None);

        Ok(())
    }

    #[test]
    fn span_exc_right_lower_bound_last_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.lower_bound_last_idx(4), Some(0));
        assert_eq!(series.lower_bound_last_idx(5), Some(2));
        assert_eq!(series.lower_bound_last_idx(6), Some(2));
        assert_eq!(series.lower_bound_last_idx(1), None);
        assert_eq!(series.lower_bound_last_idx(8), Some(3));
        assert_eq!(series.lower_bound_last_idx(9), Some(3));

        Ok(())
    }

    #[test]
    fn span_exc_right_span_before_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

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
    fn span_exc_right_span_after_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

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
    fn span_exc_right_span_at_or_before_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.span_at_or_before_idx(0), None);
        assert_eq!(series.span_at_or_before_idx(1), None);
        assert_eq!(series.span_at_or_before_idx(2), Some(0));
        assert_eq!(series.span_at_or_before_idx(3), Some(0));
        assert_eq!(series.span_at_or_before_idx(4), Some(0));
        assert_eq!(series.span_at_or_before_idx(5), Some(2));
        assert_eq!(series.span_at_or_before_idx(6), Some(2));
        assert_eq!(series.span_at_or_before_idx(7), Some(2));
        assert_eq!(series.span_at_or_before_idx(8), Some(3));
        assert_eq!(series.span_at_or_before_idx(9), Some(3));

        Ok(())
    }

    #[test]
    fn span_exc_right_span_at_or_after_idx_duplicates() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.span_at_or_after_idx(0), Some(0));
        assert_eq!(series.span_at_or_after_idx(1), Some(0));
        assert_eq!(series.span_at_or_after_idx(2), Some(0));
        assert_eq!(series.span_at_or_after_idx(3), Some(1));
        assert_eq!(series.span_at_or_after_idx(4), Some(1));
        assert_eq!(series.span_at_or_after_idx(5), Some(1));
        assert_eq!(series.span_at_or_after_idx(6), Some(3));
        assert_eq!(series.span_at_or_after_idx(7), Some(3));
        assert_eq!(series.span_at_or_after_idx(8), Some(3));
        assert_eq!(series.span_at_or_after_idx(9), None);

        Ok(())
    }

    #[test]
    fn span_exc_right_only_duplicates() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(5, 6), 21))?;

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
    fn span_exc_right_push() -> Result<()> {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10))?;
        series.push((SpanExc::new(5, 6), 20))?;
        series.push((SpanExc::new(8, 9), 30))?;

        assert_eq!(series.len(), 3);
        assert_eq!(series.get(0), Some(&(SpanExc::new(2, 3), 10)));
        assert_eq!(series.get(1), Some(&(SpanExc::new(5, 6), 20)));
        assert_eq!(series.get(2), Some(&(SpanExc::new(8, 9), 30)));

        Ok(())
    }

    #[test]
    fn span_exc_right_subseq_unbounded_both() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();
        let span = SpanAny::unb();

        let subseq = series.subseq(span);
        assert_eq!(
            subseq,
            &[(SpanExc::new(2, 3), 10), (SpanExc::new(5, 6), 20), (SpanExc::new(8, 9), 30)]
        );
    }

    #[test]
    fn span_exc_right_subseq_unbounded_left() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::unb_inc(5);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(2, 3), 10)]);

        let span = SpanAny::unb_inc(6);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(2, 3), 10), (SpanExc::new(5, 6), 20)]);
    }

    #[test]
    fn span_exc_right_subseq_unbounded_right() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::inc_unb(5);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(5, 6), 20), (SpanExc::new(8, 9), 30)]);

        let span = SpanAny::inc_unb(6);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(8, 9), 30)]);
    }

    #[test]
    fn span_exc_right_subseq_bounded() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::inc(5, 9);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(5, 6), 20), (SpanExc::new(8, 9), 30)]);

        let span = SpanAny::inc(6, 8);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[]);
    }

    #[test]
    fn span_exc_right_subseq_series_unbounded_both() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();
        let span = SpanAny::unb();

        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 3);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(2, 3), 10)));
        assert_eq!(subseq_series.get(1), Some(&(SpanExc::new(5, 6), 20)));
        assert_eq!(subseq_series.get(2), Some(&(SpanExc::new(8, 9), 30)));
    }

    #[test]
    fn span_exc_right_subseq_series_unbounded_left() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::unb_inc(5);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(2, 3), 10)));

        let span = SpanAny::unb_inc(6);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(2, 3), 10)));
        assert_eq!(subseq_series.get(1), Some(&(SpanExc::new(5, 6), 20)));
    }

    #[test]
    fn span_exc_right_subseq_series_unbounded_right() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::inc_unb(5);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(5, 6), 20)));
        assert_eq!(subseq_series.get(1), Some(&(SpanExc::new(8, 9), 30)));

        let span = SpanAny::inc_unb(6);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(8, 9), 30)));
    }

    #[test]
    fn span_exc_right_subseq_series_bounded() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::inc(5, 9);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(5, 6), 20)));
        assert_eq!(subseq_series.get(1), Some(&(SpanExc::new(8, 9), 30)));

        let span = SpanAny::inc(6, 8);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 0);
    }

    #[test]
    fn span_exc_right_subseq_unbounded_left_exc() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::unb_exc(5);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(2, 3), 10)]);

        let span = SpanAny::unb_exc(6);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(2, 3), 10), (SpanExc::new(5, 6), 20)]);
    }

    #[test]
    fn span_exc_right_subseq_unbounded_right_exc() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::exc_unb(5);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(8, 9), 30)]);

        let span = SpanAny::exc_unb(6);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(8, 9), 30)]);
    }

    #[test]
    fn span_exc_right_subseq_bounded_exc() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::exc(5, 8);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(5, 6), 20)]);

        let span = SpanAny::exc(6, 9);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(8, 9), 30)]);
    }

    #[test]
    fn span_exc_right_subseq_bounded_exc_exc() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();
        let span = SpanAny::exc_exc(5, 8);

        let subseq = series.subseq(span);
        assert_eq!(subseq, &[]);

        let span = SpanAny::exc_exc(6, 9);
        let subseq = series.subseq(span);
        assert_eq!(subseq, &[(SpanExc::new(8, 9), 30)]);
    }

    #[test]
    fn span_exc_right_subseq_series_unbounded_left_exc() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::unb_exc(5);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(2, 3), 10)));

        let span = SpanAny::unb_exc(6);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 2);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(2, 3), 10)));
        assert_eq!(subseq_series.get(1), Some(&(SpanExc::new(5, 6), 20)));
    }

    #[test]
    fn span_exc_right_subseq_series_unbounded_right_exc() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::exc_unb(5);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(8, 9), 30)));

        let span = SpanAny::exc_unb(6);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(8, 9), 30)));
    }

    #[test]
    fn span_exc_right_subseq_series_bounded_exc() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::exc(5, 8);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(5, 6), 20)));

        let span = SpanAny::exc(6, 9);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(8, 9), 30)));
    }

    #[test]
    fn span_exc_right_subseq_series_bounded_exc_exc() {
        let mut series = SpanExcSeriesRight::new();
        series.push((SpanExc::new(2, 3), 10)).unwrap();
        series.push((SpanExc::new(5, 6), 20)).unwrap();
        series.push((SpanExc::new(8, 9), 30)).unwrap();

        let span = SpanAny::exc_exc(5, 8);
        let subseq_series = series.subseq_series(span);
        assert!(subseq_series.is_empty());

        let span = SpanAny::exc_exc(6, 9);
        let subseq_series = series.subseq_series(span);
        assert_eq!(subseq_series.len(), 1);
        assert_eq!(subseq_series.get(0), Some(&(SpanExc::new(8, 9), 30)));
    }
}
