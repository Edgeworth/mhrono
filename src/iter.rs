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
        if self.t >= self.en {
            return None;
        }
        let t = self.t;
        self.t = self.op.apply(t);

        // Prevent infinite loops for no-op TimeOps.
        if t == self.t {
            self.t = self.en;
        }
        Some(t)
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
    pub fn new<A: Into<Date>>(st: A, en: A, op: DateOp) -> Self {
        Self { d: st.into(), en: en.into(), op }
    }

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
        if self.d >= self.en {
            return None;
        }
        let d = self.d;
        self.d = self.op.apply(d);

        // Prevent infinite loops for no-op DateOps.
        if d == self.d {
            self.d = self.en;
        }
        Some(d)
    }
}

#[cfg(test)]
mod tests {
    use chrono_tz::US::Eastern;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::date::{Day, ymd};
    use crate::duration::Duration;
    use crate::time::ymdhms;

    #[test]
    fn time_iter_basic() {
        let st = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let en = ymdhms(2020, 1, 1, 3, 0, 0, Eastern);
        let op = TimeOp::hourly();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert_eq!(times.len(), 3);
        assert_eq!(times[0], ymdhms(2020, 1, 1, 0, 0, 0, Eastern));
        assert_eq!(times[1], ymdhms(2020, 1, 1, 1, 0, 0, Eastern));
        assert_eq!(times[2], ymdhms(2020, 1, 1, 2, 0, 0, Eastern));
    }

    #[test]
    fn time_iter_empty() {
        let st = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let en = st;
        let op = TimeOp::hourly();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert_eq!(times.len(), 0);
    }

    #[test]
    fn time_iter_minutes() {
        let st = ymdhms(2020, 1, 1, 12, 0, 0, Eastern);
        let en = ymdhms(2020, 1, 1, 12, 5, 0, Eastern);
        let op = TimeOp::minutely();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert_eq!(times.len(), 5);
        for (i, time) in times.iter().enumerate() {
            assert_eq!(time.minute(), i as u32);
        }
    }

    #[test]
    fn time_iter_seconds() {
        let st = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let en = ymdhms(2020, 1, 1, 0, 0, 3, Eastern);
        let op = TimeOp::secondly();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert_eq!(times.len(), 3);
        assert_eq!(times[0].second(), 0);
        assert_eq!(times[1].second(), 1);
        assert_eq!(times[2].second(), 2);
    }

    #[test]
    fn time_iter_days() {
        let st = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let en = ymdhms(2020, 1, 5, 0, 0, 0, Eastern);
        let op = TimeOp::daily();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert_eq!(times.len(), 4);
        for (i, time) in times.iter().enumerate() {
            assert_eq!(time.day(), (i + 1) as u32);
        }
    }

    #[test]
    fn date_iter_day() {
        let st = ymd(2020, 1, 1, Eastern);
        let en = ymd(2020, 1, 5, Eastern);

        let iter = DateIter::day(st, en);
        let dates: Vec<_> = iter.collect();

        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], ymd(2020, 1, 1, Eastern));
        assert_eq!(dates[1], ymd(2020, 1, 2, Eastern));
        assert_eq!(dates[2], ymd(2020, 1, 3, Eastern));
        assert_eq!(dates[3], ymd(2020, 1, 4, Eastern));
    }

    #[test]
    fn date_iter_empty() {
        let st = ymd(2020, 1, 1, Eastern);
        let en = st;

        let iter = DateIter::day(st, en);
        let dates: Vec<_> = iter.collect();

        assert_eq!(dates.len(), 0);
    }

    #[test]
    fn date_iter_year() {
        let st = ymd(2020, 1, 1, Eastern);
        let en = ymd(2025, 1, 1, Eastern);

        let iter = DateIter::year(st, en);
        let dates: Vec<_> = iter.collect();

        assert_eq!(dates.len(), 5);
        assert_eq!(dates[0], ymd(2020, 1, 1, Eastern));
        assert_eq!(dates[1], ymd(2021, 1, 1, Eastern));
        assert_eq!(dates[2], ymd(2022, 1, 1, Eastern));
        assert_eq!(dates[3], ymd(2023, 1, 1, Eastern));
        assert_eq!(dates[4], ymd(2024, 1, 1, Eastern));
    }

    #[test]
    fn date_iter_across_months() {
        let st = ymd(2020, 1, 30, Eastern);
        let en = ymd(2020, 2, 3, Eastern);

        let iter = DateIter::day(st, en);
        let dates: Vec<_> = iter.collect();

        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], ymd(2020, 1, 30, Eastern));
        assert_eq!(dates[1], ymd(2020, 1, 31, Eastern));
        assert_eq!(dates[2], ymd(2020, 2, 1, Eastern));
        assert_eq!(dates[3], ymd(2020, 2, 2, Eastern));
    }

    #[test]
    fn date_iter_leap_year() {
        let st = ymd(2020, 2, 27, Eastern);
        let en = ymd(2020, 3, 2, Eastern);

        let iter = DateIter::day(st, en);
        let dates: Vec<_> = iter.collect();

        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], ymd(2020, 2, 27, Eastern));
        assert_eq!(dates[1], ymd(2020, 2, 28, Eastern));
        assert_eq!(dates[2], ymd(2020, 2, 29, Eastern));
        assert_eq!(dates[3], ymd(2020, 3, 1, Eastern));
    }

    #[test]
    fn time_iter_single_element() {
        let st = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let en = ymdhms(2020, 1, 1, 1, 0, 0, Eastern);
        let op = TimeOp::hourly();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert_eq!(times.len(), 1);
        assert_eq!(times[0], st);
    }

    #[test]
    fn date_iter_single_element() {
        let st = ymd(2020, 1, 1, Eastern);
        let en = ymd(2020, 1, 2, Eastern);

        let iter = DateIter::day(st, en);
        let dates: Vec<_> = iter.collect();

        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], st);
    }

    #[test]
    fn time_iter_start_after_end_is_empty() {
        let st = ymdhms(2020, 1, 2, 0, 0, 0, Eastern);
        let en = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let op = TimeOp::hourly();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert!(times.is_empty());
    }

    #[test]
    fn date_iter_start_after_end_is_empty() {
        let st = ymd(2020, 1, 2, Eastern);
        let en = ymd(2020, 1, 1, Eastern);

        let iter = DateIter::day(st, en);
        let dates: Vec<_> = iter.collect();

        assert!(dates.is_empty());
    }

    #[test]
    fn date_iteration_with_weekday_filtering() {
        let start = ymd(2020, 3, 9, Eastern); // Monday
        let end = ymd(2020, 3, 16, Eastern);

        let dates: Vec<_> = DateIter::day(start, end).collect();
        let weekdays: Vec<_> = dates.iter().map(Date::weekday).collect();

        assert_eq!(dates.len(), 7);
        assert_eq!(
            weekdays,
            vec![Day::Mon, Day::Tue, Day::Wed, Day::Thu, Day::Fri, Day::Sat, Day::Sun]
        );

        let weekdays_only: Vec<_> = dates
            .iter()
            .filter(|d| matches!(d.weekday(), Day::Mon | Day::Tue | Day::Wed | Day::Thu | Day::Fri))
            .collect();
        assert_eq!(weekdays_only.len(), 5);
    }

    #[test]
    fn time_iteration_with_duration() {
        let start = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let end = ymdhms(2020, 1, 1, 3, 0, 0, Eastern);
        let op = TimeOp::hourly();

        let times: Vec<_> = TimeIter::new(start, end, op).collect();

        for pair in times.windows(2) {
            assert_eq!(pair[1] - pair[0], Duration::HOUR);
        }
    }

    #[test]
    fn time_operations_across_midnight() {
        let late = ymdhms(2020, 1, 1, 23, 0, 0, Eastern);
        let early = ymdhms(2020, 1, 2, 2, 0, 0, Eastern);

        let times: Vec<_> = TimeIter::new(late, early, TimeOp::hourly()).collect();

        assert_eq!(times.len(), 3);
        assert_eq!(times[0].hour(), 23);
        assert_eq!(times[0].day(), 1);
        assert_eq!(times[1].hour(), 0);
        assert_eq!(times[1].day(), 2);
        assert_eq!(times[2].hour(), 1);
        assert_eq!(times[2].day(), 2);
    }

    #[test]
    fn time_iter_nop_stops_to_avoid_infinite_loops() {
        let start = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let end = ymdhms(2020, 1, 1, 1, 0, 0, Eastern);

        let mut iter = TimeIter::new(start, end, TimeOp::nop());
        assert_eq!(iter.next(), Some(start));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn date_iter_nop_stops_to_avoid_infinite_loops() {
        let start = ymd(2020, 1, 1, Eastern);
        let end = ymd(2020, 1, 2, Eastern);

        let mut iter = DateIter::new(start, end, DateOp::nop());
        assert_eq!(iter.next(), Some(start));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn time_iter_across_dst_spring_forward() {
        // March 8, 2020: DST starts at 2 AM EST, clocks move forward to 3 AM EDT
        let start = ymdhms(2020, 3, 8, 0, 0, 0, Eastern);
        let end = ymdhms(2020, 3, 8, 5, 0, 0, Eastern);
        let times: Vec<_> = TimeIter::new(start, end, TimeOp::hourly()).collect();

        assert_eq!(times.len(), 4);
        assert_eq!(times[0].hour(), 0);
        assert_eq!(times[1].hour(), 1);
        assert_eq!(times[2].hour(), 3); // 2 AM doesn't exist, jumps to 3 AM
        assert_eq!(times[3].hour(), 4);

        // Each step should still be exactly 1 hour in real time
        for pair in times.windows(2) {
            assert_eq!(pair[1] - pair[0], Duration::HOUR);
        }
    }

    #[test]
    fn time_iter_across_dst_fall_back() {
        // November 1, 2020: DST ends at 2 AM EDT, clocks move back to 1 AM EST
        // Since TimeOp::hourly() adds real time (Duration::HOUR), and there are
        // 5 real hours between 0:00 and 4:00 on fall-back day, we get 5 iterations.
        let start = ymdhms(2020, 11, 1, 0, 0, 0, Eastern);
        let end = ymdhms(2020, 11, 1, 4, 0, 0, Eastern);
        let times: Vec<_> = TimeIter::new(start, end, TimeOp::hourly()).collect();

        // 5 iterations because 1 AM occurs twice (once in EDT, once in EST)
        assert_eq!(times.len(), 5);

        // Each step should be exactly 1 hour in real time
        for pair in times.windows(2) {
            assert_eq!(pair[1] - pair[0], Duration::HOUR);
        }
    }

    #[test]
    fn time_iter_minutely_across_dst_spring_forward() {
        // Iterate by minutes across the DST gap
        let start = ymdhms(2020, 3, 8, 1, 58, 0, Eastern);
        let end = ymdhms(2020, 3, 8, 3, 2, 0, Eastern);
        let times: Vec<_> = TimeIter::new(start, end, TimeOp::minutely()).collect();

        // 1:58, 1:59, 3:00, 3:01 - 4 minutes total (2:00-2:59 don't exist)
        assert_eq!(times.len(), 4);
        assert_eq!(times[0].hour(), 1);
        assert_eq!(times[0].minute(), 58);
        assert_eq!(times[1].hour(), 1);
        assert_eq!(times[1].minute(), 59);
        assert_eq!(times[2].hour(), 3);
        assert_eq!(times[2].minute(), 0);
        assert_eq!(times[3].hour(), 3);
        assert_eq!(times[3].minute(), 1);
    }
}
