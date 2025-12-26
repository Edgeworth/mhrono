use std::collections::BTreeSet;
use std::collections::btree_set::Range;
use std::sync::Arc;

use chrono_tz::Tz;

use crate::date::{Date, ymd};
use crate::iter::DateIter;
use crate::op::SpanOp;
use crate::span::exc::SpanExc;
use crate::time::Time;

#[must_use]
#[derive(Debug, Clone, Default)]
struct RangeCache {
    cache: BTreeSet<Date>,
    computed: SpanExc<Date>,
}

impl RangeCache {
    fn new() -> Self {
        Self { cache: BTreeSet::new(), computed: SpanExc::default() }
    }

    fn contains(&mut self, d: Date, r: &mut impl Ranger) -> bool {
        self.ensure_range(SpanExc::point(d).unwrap(), r);
        self.cache.contains(&d)
    }

    fn get_range(&mut self, s: SpanExc<Date>, r: &mut impl Ranger) -> Range<'_, Date> {
        self.ensure_range(s, r);
        self.cache.range(s.range())
    }

    fn ensure_range(&mut self, s: SpanExc<Date>, r: &mut impl Ranger) {
        if !self.computed.contains_span(&s) {
            self.computed = SpanExc::cover(&self.computed, &s);
            let years = 20.max(self.computed.en.year() - self.computed.st.year() + 1);
            // Expand on left or right if we bumped up against it.
            if s.st == self.computed.st {
                self.computed.st = self.computed.st.add_years(-years);
            }
            if s.en == self.computed.en {
                self.computed.en = self.computed.en.add_years(years);
            }
            // TODO: only call append_range for changed bits
            r.append_range(self.computed, &mut self.cache);
        }
    }
}

pub trait Ranger {
    fn append_range<T: Into<SpanExc<Date>>>(&mut self, s: T, v: &mut BTreeSet<Date>);
}

#[must_use]
struct RangerUnion<'a, R: Ranger> {
    rs: &'a mut [R],
}

impl<'a, R: Ranger> RangerUnion<'a, R> {
    fn new(rs: &'a mut [R]) -> Self {
        Self { rs }
    }
}

impl<R: Ranger> Ranger for RangerUnion<'_, R> {
    fn append_range<T: Into<SpanExc<Date>>>(&mut self, s: T, v: &mut BTreeSet<Date>) {
        let s = s.into();
        for r in &mut *self.rs {
            r.append_range(s, v);
        }
    }
}

// TODO(1): handle early closes
#[must_use]
#[derive(Clone)]
pub struct Calendar {
    pub name: String,
    pub tz: Tz,
    opens: Vec<SpanOp>,
    hols: Vec<DaySet>,
    cache: RangeCache,
    overrides: Vec<(Vec<SpanOp>, Vec<DaySet>, RangeCache)>,
}

impl Calendar {
    pub fn new(name: &str, tz: Tz) -> Self {
        Self {
            name: name.to_owned(),
            tz,
            opens: Vec::new(),
            hols: Vec::new(),
            cache: RangeCache::new(),
            overrides: Vec::new(),
        }
    }

    /// Note: `SpanOp`s must be in chronological order.
    pub fn with_opens(mut self, opens: &[SpanOp]) -> Self {
        self.opens = opens.to_vec();
        self
    }

    pub fn with_holidays(mut self, hols: &[&'static DaySet]) -> Self {
        self.hols = hols.iter().map(|&v| v.clone()).collect();
        self
    }

    /// If a holiday affects a day, spans will be chosen from the list of span ops that the
    /// holiday list is associated with. If multiple overrides match, the first one wins.
    pub fn with_overrides(mut self, v: &[(&[SpanOp], &[&'static DaySet])]) -> Self {
        self.overrides = v
            .iter()
            .map(|(opens, hols)| {
                (opens.to_vec(), hols.iter().map(|&v| v.clone()).collect(), RangeCache::new())
            })
            .collect();
        self
    }

    pub fn with_override(mut self, opens: &[SpanOp], hols: &[&'static DaySet]) -> Self {
        self.overrides.push((
            opens.to_vec(),
            hols.iter().map(|&v| v.clone()).collect(),
            RangeCache::new(),
        ));
        self
    }

    /// Finds the first span that starts at or after the given time.
    pub fn next_span(&mut self, t: &Time) -> Option<SpanExc<Time>> {
        if self.opens.is_empty() && self.overrides.iter().all(|(opens, _, _)| opens.is_empty()) {
            return None;
        }
        let mut t = t.with_tz(self.tz);
        loop {
            let d = t.date();
            if !self.cache.contains(d, &mut RangerUnion::new(&mut self.hols))
                && let Some(s) = self.next_span_in_day(d, &t)
            {
                return Some(s);
            }
            // Use given time of day on the first iteration, but start from
            // midnight on subsequent iterations.
            t = d.add_days(1).time().unwrap();
        }
    }

    fn next_span_in_day(&mut self, d: Date, t: &Time) -> Option<SpanExc<Time>> {
        // Check overrides.
        for (opens, daysets, cache) in &mut self.overrides {
            // If there's an override span today, then process the opens for this override.
            if cache.contains(d, &mut RangerUnion::new(daysets)) {
                return Self::find_next_span_in_opens(d, t, opens);
            }
        }

        // Otherwise, return the regular span.
        Self::find_next_span_in_opens(d, t, &self.opens)
    }

    fn find_next_span_in_opens(d: Date, t: &Time, opens: &[SpanOp]) -> Option<SpanExc<Time>> {
        // Find first non-zero span starting >= t.
        // SpanOps from midnight.
        let base_t: Time = d.time().unwrap();
        for open in opens {
            let s = open.apply(base_t);
            if s.st >= *t {
                return Some(s);
            }
        }
        None
    }
}

pub trait Observance = Fn(Date) -> Option<Date> + Sync + Send;

#[must_use]
#[derive(Clone, Default)]
pub struct DaySet {
    uncached: UncachedDaySet,
    cache: RangeCache,
    adhoc: Vec<Date>,
}

impl DaySet {
    pub fn new() -> Self {
        Self { uncached: UncachedDaySet::new(), cache: RangeCache::new(), adhoc: Vec::new() }
    }

    pub fn with_md(self, m: u32, d: u32) -> Self {
        Self { uncached: UncachedDaySet { md: Some((m, d)), ..self.uncached }, ..self }
    }

    pub fn with_start(self, d: impl Into<Date>) -> Self {
        Self { uncached: UncachedDaySet { st: Some(d.into()), ..self.uncached }, ..self }
    }

    pub fn with_end(self, d: impl Into<Date>) -> Self {
        Self { uncached: UncachedDaySet { en: Some(d.into()), ..self.uncached }, ..self }
    }

    pub fn with_observance(self, o: impl 'static + Observance) -> Self {
        Self { uncached: UncachedDaySet { observance: Some(Arc::new(o)), ..self.uncached }, ..self }
    }

    pub fn with_adhoc<T>(mut self, adhoc: T) -> Self
    where
        T::Item: Into<Date>,
        T: IntoIterator,
    {
        self.adhoc.extend(adhoc.into_iter().map(Into::into));
        self.adhoc.sort_unstable();
        self
    }
}

impl Ranger for DaySet {
    fn append_range<T: Into<SpanExc<Date>>>(&mut self, s: T, v: &mut BTreeSet<Date>) {
        if self.adhoc.is_empty() {
            for d in self.cache.get_range(s.into(), &mut self.uncached) {
                v.insert(*d);
            }
        } else {
            for d in &self.adhoc {
                v.insert(*d);
            }
        }
    }
}

#[must_use]
#[derive(Clone, Default)]
struct UncachedDaySet {
    md: Option<(u32, u32)>,
    st: Option<Date>,
    en: Option<Date>,
    observance: Option<Arc<dyn Observance>>, // Adjusts the holiday date.
}

impl UncachedDaySet {
    fn new() -> Self {
        Self { md: None, st: None, en: None, observance: None }
    }

    fn iter_span(&mut self, s: SpanExc<Date>, iter: DateIter, v: &mut BTreeSet<Date>) {
        for cursor in iter {
            let d = self.observance.as_ref().map_or(Some(cursor), |f| f(cursor));
            if let Some(d) = d
                && s.contains(&d)
            {
                v.insert(d);
            }
        }
    }
}

impl Ranger for UncachedDaySet {
    fn append_range<T: Into<SpanExc<Date>>>(&mut self, s: T, v: &mut BTreeSet<Date>) {
        let s = s.into();
        let (st, en) = (s.st, s.en);
        // Account for observances going 1 year into the past or future. This keeps month and day stable too.
        let sty = self.st.map_or(st.year(), |v| v.year().max(st.year())) - 1;
        let iter_en = en.with_year(self.en.map_or(en.year(), |v| v.year().min(en.year())) + 1);
        let s = SpanExc::new(self.st.map_or(st, |v| v.max(st)), self.en.map_or(en, |v| v.min(en)));
        if let Some((m, d)) = self.md {
            let iter_st = ymd(sty, m, d, st.tz());
            self.iter_span(s, DateIter::year(iter_st, iter_en), v);
        } else {
            self.iter_span(s, DateIter::day(st.with_year(sty), iter_en), v);
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono_tz::US::Eastern;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn next_span_in_day_with_no_opens_is_none() -> crate::Result<()> {
        let mut cal = Calendar::new("Empty", Eastern);
        let d = ymd(2020, 1, 1, Eastern);
        let t = d.time()?;
        assert_eq!(cal.next_span_in_day(d, &t), None);
        Ok(())
    }

    #[test]
    fn next_span_with_no_opens_returns_none() -> crate::Result<()> {
        let mut cal = Calendar::new("Empty", Eastern);
        let t = ymd(2020, 1, 1, Eastern).time()?;
        assert_eq!(cal.next_span(&t), None);
        Ok(())
    }
}
