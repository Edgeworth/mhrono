use eyre::Result;

use crate::seq::inner::SeriesInner;
use crate::seq::series::Series;
use crate::series_ops;
use crate::span::any::SpanAny;
use crate::time::Time;

pub type F64Series = TimeSeries<f64>;

// TimeSeries is generic and allowed to contain duplicate values.
#[must_use]
#[derive(Debug, Eq, Default, PartialEq, PartialOrd, Hash, Clone)]
pub struct TimeSeries<Y: Clone> {
    inner: SeriesInner<(Time, Y)>,
}

impl<Y: Clone> TimeSeries<Y> {
    pub fn new() -> Self {
        Self { inner: SeriesInner::empty() }
    }
}

impl<Y: Clone> Series for TimeSeries<Y> {
    type X = Time;
    type Y = Y;
    type V = (Time, Y);

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

series_ops!(TimeSeries<Y>, <Y: Clone>);
