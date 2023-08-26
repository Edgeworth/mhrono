use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::op::{DateOp, TimeOp};
use crate::time::Time;

#[must_use]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Display, Serialize, Deserialize)]
#[display("[{t}, {en:?})")]
pub struct TimeIter {
    t: Time,
    en: Time,
    op: TimeOp,
}

impl TimeIter {
    pub fn new<T: Into<Time>>(st: T, en: T, op: TimeOp) -> Self {
        Self { t: st.into(), en: en.into(), op }
    }
}

impl Iterator for TimeIter {
    type Item = Time;

    fn next(&mut self) -> Option<Self::Item> {
        let t = self.t;
        self.t = self.op.apply(t);
        if self.t >= self.en { None } else { Some(self.t) }
    }
}

// Date iterator that is exclusive (doesn't include the endpoint).
#[must_use]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Display, Serialize, Deserialize)]
#[display("[{d}, {en:?})")]
pub struct DateIter {
    d: Date,
    en: Date,
    op: DateOp,
}

impl DateIter {
    pub fn day<A: Into<Date>>(st: A, en: A) -> Self {
        Self { d: st.into(), en: en.into(), op: DateOp::add_days(1) }
    }

    pub fn year<A: Into<Date>>(st: A, en: A) -> Self {
        Self { d: st.into(), en: en.into(), op: DateOp::add_years(1) }
    }
}

impl Iterator for DateIter {
    type Item = Date;

    fn next(&mut self) -> Option<Self::Item> {
        let d = self.d;
        self.d = self.op.apply(d);
        if d >= self.en { None } else { Some(d) }
    }
}
