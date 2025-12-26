use crate::calendars::calendar::Calendar;
use crate::span::exc::SpanExc;
use crate::time::Time;
use crate::{Error, Result};

// Calendar that caches the results from an inner calendar.
// Much faster than a regular calendar.
#[must_use]
#[derive(Clone)]
pub struct CachedCalendar {
    spans: Vec<SpanExc<Time>>,
    span: SpanExc<Time>,
}

impl CachedCalendar {
    pub fn new(span: SpanExc<Time>, cal: &mut Calendar) -> Self {
        let mut spans = Vec::new();

        let mut cur_t = span.st;
        while let Some(next) = cal.next_span(&cur_t)
            && next.en <= span.en
        {
            spans.push(next);
            cur_t = next.en;
        }

        Self { spans, span }
    }

    /// Finds the first span that starts strictly after the given time.
    pub fn next_span(&self, t: &Time) -> Result<Option<SpanExc<Time>>> {
        if !self.span.contains(t) {
            return Err(Error::OutOfRange(format!(
                "requested time {t} outside of cached span {}",
                self.span
            )));
        }
        let idx = self.spans.partition_point(|v| v.st <= *t);
        if idx < self.spans.len() { Ok(Some(self.spans[idx])) } else { Ok(None) }
    }

    pub fn span(&self) -> SpanExc<Time> {
        self.span
    }
}

#[cfg(test)]
mod tests {
    use chrono_tz::US::Eastern;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::date::ymd;
    use crate::op::{SpanOp, TimeOp};
    use crate::time::ymdhms;

    fn make_test_calendar() -> Calendar {
        Calendar::new("Test", Eastern).with_opens(&[
            SpanOp::new(TimeOp::add_hours(9), TimeOp::add_hours(10)),
            SpanOp::new(TimeOp::add_hours(11), TimeOp::add_hours(12)),
        ])
    }

    #[test]
    fn next_span_at_open() -> Result<()> {
        let d = ymd(2020, 1, 1, Eastern);
        let span = SpanExc::new(d.time()?, d.add_days(1).time()?);
        let mut cal = make_test_calendar();
        let cached = CachedCalendar::new(span, &mut cal);

        let at_open = ymdhms(2020, 1, 1, 9, 0, 0, Eastern);
        let got = cached.next_span(&at_open)?.unwrap();
        assert_eq!(got.st, ymdhms(2020, 1, 1, 11, 0, 0, Eastern));
        assert_eq!(got.en, ymdhms(2020, 1, 1, 12, 0, 0, Eastern));
        Ok(())
    }

    #[test]
    fn next_span_does_not_return_span_outside_cached_range() -> Result<()> {
        let d = ymd(2020, 1, 1, Eastern);
        let span = SpanExc::new(d.time()?, d.add_days(1).time()?);
        let mut cal = make_test_calendar();
        let cached = CachedCalendar::new(span, &mut cal);

        // After the last in-day span ends, the cached calendar returns the next day's open
        // even though it's outside the cached span.
        let end_of_last = ymdhms(2020, 1, 1, 12, 0, 0, Eastern);
        let got = cached.next_span(&end_of_last)?;
        assert_eq!(got, None);
        Ok(())
    }
}
