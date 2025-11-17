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
        if t >= self.en {
            None
        } else {
            self.t = self.op.apply(t);
            Some(t)
        }
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

#[cfg(test)]
mod tests {
    use chrono_tz::US::Eastern;
    use eyre::Result;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::date::ymd;
    use crate::time::ymdhms;

    #[test]
    fn test_time_iter_basic() -> Result<()> {
        let st = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let en = ymdhms(2020, 1, 1, 3, 0, 0, Eastern);
        let op = TimeOp::hourly();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert_eq!(times.len(), 3);
        assert_eq!(times[0], ymdhms(2020, 1, 1, 0, 0, 0, Eastern));
        assert_eq!(times[1], ymdhms(2020, 1, 1, 1, 0, 0, Eastern));
        assert_eq!(times[2], ymdhms(2020, 1, 1, 2, 0, 0, Eastern));

        Ok(())
    }

    #[test]
    fn test_time_iter_empty() -> Result<()> {
        let st = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let en = st;
        let op = TimeOp::hourly();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert_eq!(times.len(), 0);

        Ok(())
    }

    #[test]
    fn test_time_iter_minutes() -> Result<()> {
        let st = ymdhms(2020, 1, 1, 12, 0, 0, Eastern);
        let en = ymdhms(2020, 1, 1, 12, 5, 0, Eastern);
        let op = TimeOp::minutely();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert_eq!(times.len(), 5);
        for (i, time) in times.iter().enumerate() {
            assert_eq!(time.minute(), i as u32);
        }

        Ok(())
    }

    #[test]
    fn test_time_iter_seconds() -> Result<()> {
        let st = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let en = ymdhms(2020, 1, 1, 0, 0, 3, Eastern);
        let op = TimeOp::secondly();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert_eq!(times.len(), 3);
        assert_eq!(times[0].second(), 0);
        assert_eq!(times[1].second(), 1);
        assert_eq!(times[2].second(), 2);

        Ok(())
    }

    #[test]
    fn test_time_iter_days() -> Result<()> {
        let st = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let en = ymdhms(2020, 1, 5, 0, 0, 0, Eastern);
        let op = TimeOp::daily();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert_eq!(times.len(), 4);
        for (i, time) in times.iter().enumerate() {
            assert_eq!(time.day(), (i + 1) as u32);
        }

        Ok(())
    }

    #[test]
    fn test_date_iter_day() -> Result<()> {
        let st = ymd(2020, 1, 1, Eastern);
        let en = ymd(2020, 1, 5, Eastern);

        let iter = DateIter::day(st, en);
        let dates: Vec<_> = iter.collect();

        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], ymd(2020, 1, 1, Eastern));
        assert_eq!(dates[1], ymd(2020, 1, 2, Eastern));
        assert_eq!(dates[2], ymd(2020, 1, 3, Eastern));
        assert_eq!(dates[3], ymd(2020, 1, 4, Eastern));

        Ok(())
    }

    #[test]
    fn test_date_iter_empty() -> Result<()> {
        let st = ymd(2020, 1, 1, Eastern);
        let en = st;

        let iter = DateIter::day(st, en);
        let dates: Vec<_> = iter.collect();

        assert_eq!(dates.len(), 0);

        Ok(())
    }

    #[test]
    fn test_date_iter_year() -> Result<()> {
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

        Ok(())
    }

    #[test]
    fn test_date_iter_across_months() -> Result<()> {
        let st = ymd(2020, 1, 30, Eastern);
        let en = ymd(2020, 2, 3, Eastern);

        let iter = DateIter::day(st, en);
        let dates: Vec<_> = iter.collect();

        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], ymd(2020, 1, 30, Eastern));
        assert_eq!(dates[1], ymd(2020, 1, 31, Eastern));
        assert_eq!(dates[2], ymd(2020, 2, 1, Eastern));
        assert_eq!(dates[3], ymd(2020, 2, 2, Eastern));

        Ok(())
    }

    #[test]
    fn test_date_iter_leap_year() -> Result<()> {
        let st = ymd(2020, 2, 27, Eastern);
        let en = ymd(2020, 3, 2, Eastern);

        let iter = DateIter::day(st, en);
        let dates: Vec<_> = iter.collect();

        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], ymd(2020, 2, 27, Eastern));
        assert_eq!(dates[1], ymd(2020, 2, 28, Eastern));
        assert_eq!(dates[2], ymd(2020, 2, 29, Eastern));
        assert_eq!(dates[3], ymd(2020, 3, 1, Eastern));

        Ok(())
    }

    #[test]
    fn test_time_iter_single_element() -> Result<()> {
        let st = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
        let en = ymdhms(2020, 1, 1, 1, 0, 0, Eastern);
        let op = TimeOp::hourly();

        let iter = TimeIter::new(st, en, op);
        let times: Vec<_> = iter.collect();

        assert_eq!(times.len(), 1);
        assert_eq!(times[0], st);

        Ok(())
    }

    #[test]
    fn test_date_iter_single_element() -> Result<()> {
        let st = ymd(2020, 1, 1, Eastern);
        let en = ymd(2020, 1, 2, Eastern);

        let iter = DateIter::day(st, en);
        let dates: Vec<_> = iter.collect();

        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], st);

        Ok(())
    }
}
