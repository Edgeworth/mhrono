use std::ops::{Bound, Deref, DerefMut, Index, RangeBounds};
use std::sync::Arc;

/// Prevents others from modifying `SeriesInner` while we allow outside operations
/// on the underlying data.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SeriesGuard<'a, V: Clone> {
    inner: &'a mut SeriesInner<V>,
}

impl<V: Clone> SeriesGuard<'_, V> {
    #[must_use]
    pub fn vec(&self) -> &Vec<V> {
        &self.inner.data
    }

    pub fn vec_mut(&mut self) -> &mut Vec<V> {
        Arc::make_mut(&mut self.inner.data)
    }
}

impl<V: Clone> Deref for SeriesGuard<'_, V> {
    type Target = Vec<V>;

    fn deref(&self) -> &Self::Target {
        self.vec()
    }
}

impl<V: Clone> DerefMut for SeriesGuard<'_, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.vec_mut()
    }
}

impl<V: Clone> Drop for SeriesGuard<'_, V> {
    fn drop(&mut self) {
        self.inner.st = 0;
        self.inner.en = self.inner.data.len();
    }
}

#[must_use]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub struct SeriesInner<V: Clone> {
    data: Arc<Vec<V>>,
    st: usize,
    en: usize, // exclusive
}

impl<V: Clone> Default for SeriesInner<V> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<V: Clone> SeriesInner<V> {
    pub fn new(data: impl Into<Arc<Vec<V>>>) -> Self {
        let data = data.into();
        let en = data.len();
        Self { data, st: 0, en }
    }

    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.en - self.st
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn slice(&self) -> &[V] {
        &self.data[self.st..self.en]
    }

    #[must_use]
    pub fn first(&self) -> Option<&V> {
        self.data.get(self.st)
    }

    #[must_use]
    pub fn last(&self) -> Option<&V> {
        if self.en == 0 { None } else { self.data.get(self.en - 1) }
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&V> {
        self.data.get(self.st + index)
    }

    pub fn data_mut(&mut self) -> SeriesGuard<'_, V> {
        // Have to move to new backing storage if we have a subsequence set.
        if self.st != 0 || self.en != self.data.len() {
            self.data = Arc::new(self.slice().to_vec());
            self.st = 0;
            self.en = self.data.len();
        }

        SeriesGuard { inner: self }
    }

    pub fn push(&mut self, elt: V) {
        if self.en == self.data.len() {
            // Can (potentially) avoid cloning if the range goes to the end.
            Arc::make_mut(&mut self.data).push(elt);
            self.en += 1;
        } else {
            self.data_mut().push(elt);
        }
    }

    pub fn pop(&mut self) -> Option<V> {
        if self.is_empty() {
            return None;
        }
        if self.en == self.data.len() {
            // Can (potentially) avoid cloning if the range goes to the end.
            self.en -= 1;
            Arc::make_mut(&mut self.data).pop()
        } else {
            self.data_mut().pop()
        }
    }

    pub fn subseq(&self, range: impl RangeBounds<usize>) -> Self {
        let st = match range.start_bound() {
            Bound::Included(&st) => self.st + st,
            Bound::Excluded(&st) => self.st + st + 1,
            Bound::Unbounded => self.st,
        };
        let en = match range.end_bound() {
            Bound::Included(&en) => self.st + en + 1,
            Bound::Excluded(&en) => self.st + en,
            Bound::Unbounded => self.en,
        };
        assert!(
            !(st > en || st > self.data.len() || en > self.data.len()),
            "Invalid range: {st} > {en}"
        );

        Self { data: Arc::clone(&self.data), st, en }
    }
}

impl<V: Clone> Index<usize> for SeriesInner<V> {
    type Output = V;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[self.st + index]
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_new() {
        let data = vec![1, 2, 3];
        let series = SeriesInner::new(data.clone());
        assert_eq!(series.data.as_ref(), &data);
    }

    #[test]
    fn test_empty() {
        let empty_series: SeriesInner<i32> = SeriesInner::empty();
        assert_eq!(empty_series.data.as_ref(), &Vec::<i32>::new());
    }

    #[test]
    fn test_len() {
        let series = SeriesInner::new(vec![1, 2, 3]);
        assert_eq!(series.len(), 3);
    }

    #[test]
    fn test_is_empty() {
        let empty_series: SeriesInner<i32> = SeriesInner::empty();
        assert!(empty_series.is_empty());
    }

    #[test]
    fn test_slice() {
        let series = SeriesInner::new(vec![1, 2, 3]);
        assert_eq!(series.slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_data_mut() {
        let mut series = SeriesInner::new(vec![1, 2, 3]);
        series.st = 1;
        let data_mut = series.data_mut();
        assert_eq!(data_mut.vec(), &vec![2, 3]);
    }

    #[test]
    fn test_push() {
        let mut series = SeriesInner::new(vec![1, 2, 3]);
        series.push(4);
        assert_eq!(series.data.as_ref(), &vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_pop() {
        let mut series = SeriesInner::new(vec![1, 2, 3]);
        let popped_value = series.pop();
        assert_eq!(popped_value, Some(3));
        assert_eq!(series.data.as_ref(), &vec![1, 2]);
    }

    #[test]
    fn test_subseq() {
        let series = SeriesInner::new(vec![1, 2, 3, 4, 5]);
        let subseries = series.subseq(1..4);
        assert_eq!(subseries.data.as_ref(), series.data.as_ref());
        assert_eq!(subseries.st, 1);
        assert_eq!(subseries.en, 4);
        assert_eq!(subseries.slice(), &[2, 3, 4]);
    }

    #[test]
    fn test_push_subseq() {
        let series = SeriesInner::new(vec![1, 2, 3, 4, 5]);
        let mut subseries = series.subseq(1..3);
        assert_eq!(subseries.data.as_ref(), &vec![1, 2, 3, 4, 5]);
        assert_eq!(subseries.slice(), &[2, 3]);
        assert_eq!(subseries.st, 1);
        assert_eq!(subseries.en, 3);

        subseries.push(6);
        assert_eq!(subseries.data.as_ref(), &vec![2, 3, 6]);
        assert_eq!(subseries.slice(), &[2, 3, 6]);
        assert_eq!(subseries.st, 0);
        assert_eq!(subseries.en, 3);

        let mut subseries_one_sided_left = series.subseq(0..3);
        assert_eq!(subseries_one_sided_left.data.as_ref(), &vec![1, 2, 3, 4, 5]);
        assert_eq!(subseries_one_sided_left.slice(), &[1, 2, 3]);
        assert_eq!(subseries_one_sided_left.st, 0);
        assert_eq!(subseries_one_sided_left.en, 3);

        subseries_one_sided_left.push(7);
        assert_eq!(subseries_one_sided_left.data.as_ref(), &vec![1, 2, 3, 7]);
        assert_eq!(subseries_one_sided_left.slice(), &[1, 2, 3, 7]);
        assert_eq!(subseries_one_sided_left.st, 0);
        assert_eq!(subseries_one_sided_left.en, 4);

        let mut subseries_one_sided_right = series.subseq(2..);
        assert_eq!(subseries_one_sided_right.data.as_ref(), &vec![1, 2, 3, 4, 5]);
        assert_eq!(subseries_one_sided_right.slice(), &[3, 4, 5]);
        assert_eq!(subseries_one_sided_right.st, 2);
        assert_eq!(subseries_one_sided_right.en, 5);

        subseries_one_sided_right.push(8);
        assert_eq!(subseries_one_sided_right.data.as_ref(), &vec![1, 2, 3, 4, 5, 8]);
        assert_eq!(subseries_one_sided_right.slice(), &[3, 4, 5, 8]);
        assert_eq!(subseries_one_sided_right.st, 2);
        assert_eq!(subseries_one_sided_right.en, 6);
    }

    #[test]
    fn test_pop_subseq() {
        let series = SeriesInner::new(vec![1, 2, 3, 4, 5]);
        let mut subseries = series.subseq(1..3);
        assert_eq!(subseries.data.as_ref(), &vec![1, 2, 3, 4, 5]);
        assert_eq!(subseries.slice(), &[2, 3]);
        assert_eq!(subseries.st, 1);
        assert_eq!(subseries.en, 3);

        let popped_value = subseries.pop();
        assert_eq!(popped_value, Some(3));
        assert_eq!(subseries.data.as_ref(), &vec![2]);
        assert_eq!(subseries.slice(), &[2]);
        assert_eq!(subseries.st, 0);
        assert_eq!(subseries.en, 1);

        let mut subseries_one_sided_left = series.subseq(0..3);
        assert_eq!(subseries_one_sided_left.data.as_ref(), &vec![1, 2, 3, 4, 5]);
        assert_eq!(subseries_one_sided_left.slice(), &[1, 2, 3]);
        assert_eq!(subseries_one_sided_left.st, 0);
        assert_eq!(subseries_one_sided_left.en, 3);

        let popped_value_one_sided_left = subseries_one_sided_left.pop();
        assert_eq!(popped_value_one_sided_left, Some(3));
        assert_eq!(subseries_one_sided_left.data.as_ref(), &vec![1, 2]);
        assert_eq!(subseries_one_sided_left.slice(), &[1, 2]);
        assert_eq!(subseries_one_sided_left.st, 0);
        assert_eq!(subseries_one_sided_left.en, 2);

        let mut subseries_one_sided_right = series.subseq(2..);
        assert_eq!(subseries_one_sided_right.data.as_ref(), &vec![1, 2, 3, 4, 5]);
        assert_eq!(subseries_one_sided_right.slice(), &[3, 4, 5]);
        assert_eq!(subseries_one_sided_right.st, 2);
        assert_eq!(subseries_one_sided_right.en, 5);

        let popped_value_one_sided_right = subseries_one_sided_right.pop();
        assert_eq!(popped_value_one_sided_right, Some(5));
        assert_eq!(subseries_one_sided_right.data.as_ref(), &vec![1, 2, 3, 4]);
        assert_eq!(subseries_one_sided_right.slice(), &[3, 4]);
        assert_eq!(subseries_one_sided_right.st, 2);
        assert_eq!(subseries_one_sided_right.en, 4);
    }

    #[test]
    fn test_data_mut_modify_vec_push() {
        let mut series = SeriesInner::new(vec![1, 2, 3]);

        {
            let mut data_mut = series.data_mut();
            data_mut.push(4);
            data_mut.push(5);
            data_mut.push(6);
        }

        assert_eq!(series.len(), 6);
        assert_eq!(series.slice(), &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_data_mut_modify_vec_pop() {
        let mut series = SeriesInner::new(vec![1, 2, 3]);

        {
            let mut data_mut = series.data_mut();
            assert_eq!(data_mut.pop(), Some(3));
        }

        assert_eq!(series.len(), 2);
        assert_eq!(series.slice(), &[1, 2]);
    }
}
