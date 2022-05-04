use eyre::{eyre, Result};

use crate::calendars::calendar::Calendar;
use crate::span::Span;
use crate::time::Time;

// Calendar that caches the results from an inner calendar.
// Much faster than a regular calendar.
#[derive(Clone)]
pub struct CachedCalendar {
    spans: Vec<Span<Time>>,
    span: Span<Time>,
}

impl CachedCalendar {
    pub fn new(span: Span<Time>, cal: &mut Calendar) -> Self {
        let mut spans = Vec::new();

        let mut cur_t = span.st;
        while cur_t < span.en {
            if let Some(next) = cal.next_span(cur_t) {
                spans.push(next);
                cur_t = next.en;
            } else {
                break;
            }
        }

        Self { spans, span }
    }

    pub fn next_span(&self, t: Time) -> Result<Option<Span<Time>>> {
        if !self.span.contains(t) {
            return Err(eyre!("requested time {} outside of cached span {}", t, self.span));
        }
        let idx = self.spans.partition_point(|v| v.st <= t);
        if idx < self.spans.len() {
            Ok(Some(self.spans[idx]))
        } else {
            Ok(None)
        }
    }

    #[must_use]
    pub fn span(&self) -> Span<Time> {
        self.span
    }
}
