use std::ops::{Bound, RangeBounds};
use std::sync::Arc;

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

    pub fn data_mut(&mut self) -> &mut Vec<V> {
        // Have to move to new backing storage if we have a subsequence set.
        if self.st != 0 || self.en != self.data.len() {
            self.data = Arc::new(self.slice().to_vec());
        }
        Arc::make_mut(&mut self.data)
    }

    pub fn push(&mut self, elt: V) {
        if self.en == self.data.len() {
            // Can avoid cloning if the range goes to the end.
            Arc::make_mut(&mut self.data).push(elt);
        } else {
            self.data_mut().push(elt);
        }
        self.en += 1;
    }

    pub fn pop(&mut self) -> Option<V> {
        if self.is_empty() {
            return None;
        }
        let ret = if self.en == self.data.len() {
            // Can avoid cloning if the range goes to the end.
            Arc::make_mut(&mut self.data).pop()
        } else {
            self.data_mut().pop()
        };
        self.en -= 1;
        ret
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
