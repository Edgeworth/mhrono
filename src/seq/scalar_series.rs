use eyre::Result;

use crate::seq::inner::SeriesInner;
use crate::seq::series::Series;
use crate::series_ops;
use crate::span::any::SpanAny;
use crate::time::Time;

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
