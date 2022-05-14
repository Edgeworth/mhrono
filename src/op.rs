use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::date::Date;
use crate::span::Span;
use crate::time::Time;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd, FromPrimitive)]
pub enum TOp {
    // Adv |n| > 0 will always move to the next/previous day, even if the current day is the one in question.
    AdvMon = 0,
    AdvTue = 1,
    AdvWed = 2,
    AdvThu = 3,
    AdvFri = 4,
    AdvSat = 5,
    AdvSun = 6,
    // Find* is like Adv* but won't advance if it's already the given day.
    FindMon = 7,
    FindTue = 8,
    FindWed = 9,
    FindThu = 10,
    FindFri = 11,
    FindSat = 12,
    FindSun = 13,
    AddYears = 14,
    AddMonths = 15,
    AddDays = 16,
    SetYear = 17,
    SetMonth = 18,
    SetDay = 19,
    Nop = 20,
    AddHours,
    AddMins,
    AddSecs,
    AddMillis,
    AddMicros,
    AddNanos,
    SetHour,
    SetMin,
    SetSec,
    SetMillis,
    SetMicros,
    SetNanos,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd)]
pub struct TimeOp {
    op: TOp,
    n: i64,
}

impl TimeOp {
    #[must_use]
    pub const fn new(op: TOp, n: i64) -> Self {
        Self { op, n }
    }

    pub fn apply(&self, t: impl Into<Time>) -> Time {
        let t = t.into();
        match self.op {
            TOp::AddHours => t.add_hours(self.n),
            TOp::AddMins => t.add_mins(self.n),
            TOp::AddSecs => t.add_secs(self.n),
            TOp::AddMillis => t.add_millis(self.n),
            TOp::AddMicros => t.add_micros(self.n),
            TOp::AddNanos => t.add_nanos(self.n),
            TOp::SetHour => t.with_hour(self.n as u32),
            TOp::SetMin => t.with_min(self.n as u32),
            TOp::SetSec => t.with_sec(self.n as u32),
            TOp::SetMillis => t.with_millis(self.n as u32),
            TOp::SetMicros => t.with_micros(self.n as u32),
            TOp::SetNanos => t.with_nanos(self.n as u32),
            _ => t.with_date(apply_dop(
                t.date(),
                FromPrimitive::from_i32(self.op as i32).unwrap(),
                self.n,
            )),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd, FromPrimitive)]
pub enum DOp {
    // Adv |n| > 0 will always move to the next/previous day, even if the current day is the one in question.
    AdvMon = 0,
    AdvTue = 1,
    AdvWed = 2,
    AdvThu = 3,
    AdvFri = 4,
    AdvSat = 5,
    AdvSun = 6,
    // Find* is like Adv* but won't advance if it's already the given day.
    FindMon = 7,
    FindTue = 8,
    FindWed = 9,
    FindThu = 10,
    FindFri = 11,
    FindSat = 12,
    FindSun = 13,
    AddYears = 14,
    AddMonths = 15,
    AddDays = 16,
    SetYear = 17,
    SetMonth = 18,
    SetDay = 19,
    Nop = 20,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd)]
pub struct DateOp {
    op: DOp,
    n: i64,
}

impl DateOp {
    #[must_use]
    pub const fn new(op: DOp, n: i64) -> Self {
        Self { op, n }
    }

    pub fn apply(&self, d: impl Into<Date>) -> Date {
        apply_dop(d.into(), self.op, self.n)
    }
}

fn apply_dop(d: Date, op: DOp, n: i64) -> Date {
    match op {
        DOp::AddYears => d.add_years(n as i32),
        DOp::AddMonths => d.add_months(n as i32),
        DOp::AddDays => d.add_days(n as i32),
        _ if (DOp::AdvMon..=DOp::AdvSun).contains(&op) => {
            let offset = (op as i32 - DOp::AdvMon as i32 - d.weekday() as i32).rem_euclid(7);
            let n = if offset != 0 && n > 0 { n - 1 } else { n };
            d.add_days(offset + 7 * n as i32)
        }
        _ if (DOp::FindMon..=DOp::FindSun).contains(&op) => {
            let offset = (op as i32 - DOp::FindMon as i32 - d.weekday() as i32).rem_euclid(7);
            let n = if n < 0 && offset != 0 { n - 1 } else { n };
            let n = n - n.signum();
            d.add_days(offset + 7 * n as i32)
        }
        DOp::SetYear => d.with_year(n as i32),
        DOp::SetMonth => d.with_month(n as u32),
        DOp::SetDay => d.with_day(n as u32),
        _ => d,
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
pub struct SpanOp {
    pub st: TimeOp,
    pub en: TimeOp, // Exclusive.
}

impl SpanOp {
    #[must_use]
    pub const fn new(st: TimeOp, en: TimeOp) -> Self {
        Self { st, en }
    }

    pub fn apply(&self, t: impl Into<Time>) -> Span<Time> {
        let t = t.into();
        Span::new(self.st.apply(t), self.en.apply(t))
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use chrono_tz::{Australia, Tz, US, UTC};

    use super::*;

    const TZ: [Tz; 3] = [US::Eastern, UTC, Australia::Eucla];

    #[test]
    fn leap_years() {
        for tz in &TZ {
            assert_eq!(
                Time::from_date(tz.ymd(2020, 2, 29)).with_year(2019),
                tz.ymd(2019, 2, 28).into()
            );
            assert_eq!(
                Time::from_date(tz.ymd(2020, 1, 29)).with_month(2),
                tz.ymd(2020, 2, 29).into()
            );
            assert_eq!(
                Time::from_date(tz.ymd(2020, 1, 30)).with_month(2),
                tz.ymd(2020, 2, 29).into()
            );
            assert_eq!(
                Time::from_date(tz.ymd(2019, 1, 29)).with_month(2),
                tz.ymd(2019, 2, 28).into()
            );
            assert_eq!(
                Time::from_date(tz.ymd(2019, 1, 30)).with_month(2),
                tz.ymd(2019, 2, 28).into()
            );
        }
    }

    #[test]
    fn time_offset() {
        for tz in &TZ {
            // Regular +1 year.
            assert_eq!(
                Time::op(TOp::AddYears, 1).apply(tz.ymd(2019, 1, 30)),
                tz.ymd(2020, 1, 30).into(),
            );
            // Leap year to non-leap year.
            assert_eq!(
                Time::op(TOp::AddYears, 1).apply(tz.ymd(2020, 2, 29)),
                tz.ymd(2021, 2, 28).into(),
            );
            // Month with more days to less days.
            assert_eq!(
                Time::op(TOp::AddMonths, 1).apply(tz.ymd(2019, 1, 30)),
                tz.ymd(2019, 2, 28).into(),
            );
            // Month with more days to less days in a leap year.
            assert_eq!(
                Time::op(TOp::AddMonths, 1).apply(tz.ymd(2020, 1, 30)),
                tz.ymd(2020, 2, 29).into(),
            );
            // 29th Feb to next year March.
            assert_eq!(
                Time::op(TOp::AddMonths, 13).apply(tz.ymd(2020, 2, 29)),
                tz.ymd(2021, 3, 29).into(),
            );
            // Leap year +1 day.
            assert_eq!(
                Time::op(TOp::AddDays, 1).apply(tz.ymd(2020, 2, 28)),
                tz.ymd(2020, 2, 29).into(),
            );
            // Non-leap year +1 day.
            assert_eq!(
                Time::op(TOp::AddDays, 1).apply(tz.ymd(2019, 2, 28)),
                tz.ymd(2019, 3, 1).into(),
            );
            // Advance Monday (+2) on a Sunday.
            assert_eq!(
                Time::op(TOp::AdvMon, 2).apply(tz.ymd(2020, 12, 6)),
                tz.ymd(2020, 12, 14).into(),
            );
            // Advance Monday (+2) on a Monday.
            assert_eq!(
                Time::op(TOp::AdvMon, 2).apply(tz.ymd(2020, 12, 7)),
                tz.ymd(2020, 12, 21).into(),
            );
            // Advance Monday (+1) on a Sunday.
            assert_eq!(
                Time::op(TOp::AdvMon, 1).apply(tz.ymd(2020, 12, 6)),
                tz.ymd(2020, 12, 7).into(),
            );
            // Advance Monday (+1) on a Monday.
            assert_eq!(
                Time::op(TOp::AdvMon, 1).apply(tz.ymd(2020, 12, 7)),
                tz.ymd(2020, 12, 14).into(),
            );
            // Advance Monday (+0) on a Sunday.
            assert_eq!(
                Time::op(TOp::AdvMon, 0).apply(tz.ymd(2020, 12, 6)),
                tz.ymd(2020, 12, 7).into(),
            );
            // Advance Monday (+0) on a Monday.
            assert_eq!(
                Time::op(TOp::AdvMon, 0).apply(tz.ymd(2020, 12, 7)),
                tz.ymd(2020, 12, 7).into(),
            );
            // Neg advance Monday (-1) on a Sunday.
            assert_eq!(
                Time::op(TOp::AdvMon, -1).apply(tz.ymd(2020, 12, 6)),
                tz.ymd(2020, 11, 30).into(),
            );
            // Neg advance Monday (-1) on a Monday.
            assert_eq!(
                Time::op(TOp::AdvMon, -1).apply(tz.ymd(2020, 12, 7)),
                tz.ymd(2020, 11, 30).into(),
            );
            // Find Monday (+2) on a Sunday.
            assert_eq!(
                Time::op(TOp::FindMon, 2).apply(tz.ymd(2020, 12, 6)),
                tz.ymd(2020, 12, 14).into(),
            );
            // Find Monday (+2) on a Monday.
            assert_eq!(
                Time::op(TOp::FindMon, 2).apply(tz.ymd(2020, 12, 7)),
                tz.ymd(2020, 12, 14).into(),
            );
            // Find Monday (+1) on a Sunday.
            assert_eq!(
                Time::op(TOp::FindMon, 1).apply(tz.ymd(2020, 12, 6)),
                tz.ymd(2020, 12, 7).into(),
            );
            // Find Monday (+1) on a Monday.
            assert_eq!(
                Time::op(TOp::FindMon, 1).apply(tz.ymd(2020, 12, 7)),
                tz.ymd(2020, 12, 7).into(),
            );
            // Find Monday (+0) on a Sunday.
            assert_eq!(
                Time::op(TOp::FindMon, 0).apply(tz.ymd(2020, 12, 6)),
                tz.ymd(2020, 12, 7).into(),
            );
            // Find Monday (+0) on a Monday.
            assert_eq!(
                Time::op(TOp::FindMon, 0).apply(tz.ymd(2020, 12, 7)),
                tz.ymd(2020, 12, 7).into(),
            );
            // Neg find Monday (-1) on a Sunday.
            assert_eq!(
                Time::op(TOp::FindMon, -1).apply(tz.ymd(2020, 12, 6)),
                tz.ymd(2020, 11, 30).into(),
            );
            // Neg find Monday (-1) on a Monday.
            assert_eq!(
                Time::op(TOp::FindMon, -1).apply(tz.ymd(2020, 12, 7)),
                tz.ymd(2020, 12, 7).into(),
            );
        }
    }
}
