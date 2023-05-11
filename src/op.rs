use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::span::exc::SpanExc;
use crate::time::Time;

#[must_use]
#[derive(
    Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd, FromPrimitive, Serialize, Deserialize,
)]
pub enum TOp {
    // Adv |n| > 0 will always move to the next/previous day, even if the current day is the one in question.
    AdvMon = 0,
    AdvTue = 1,
    AdvWed = 2,
    AdvThu = 3,
    AdvFri = 4,
    AdvSat = 5,
    AdvSun = 6,
    AdvDay = 7,
    AdvMonth = 8,
    // Find* is like Adv* but won't advance if it's already the given day.
    FindMon = 9,
    FindTue = 10,
    FindWed = 11,
    FindThu = 12,
    FindFri = 13,
    FindSat = 14,
    FindSun = 15,
    FindDay = 16,
    FindMonth = 17,
    AddYears = 18,
    AddMonths = 19,
    AddDays = 20,
    SetYear = 21,
    SetMonth = 22,
    SetDay = 23,
    Nop = 24,
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

#[must_use]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TimeOp {
    op: TOp,
    n: i64,
}

impl TimeOp {
    pub const fn new(op: TOp, n: i64) -> Self {
        Self { op, n }
    }

    pub const fn advance_mon(n: i64) -> Self {
        Self::new(TOp::AdvMon, n)
    }

    pub const fn advance_tue(n: i64) -> Self {
        Self::new(TOp::AdvTue, n)
    }

    pub const fn advance_wed(n: i64) -> Self {
        Self::new(TOp::AdvWed, n)
    }

    pub const fn advance_thu(n: i64) -> Self {
        Self::new(TOp::AdvThu, n)
    }

    pub const fn advance_fri(n: i64) -> Self {
        Self::new(TOp::AdvFri, n)
    }

    pub const fn advance_sat(n: i64) -> Self {
        Self::new(TOp::AdvSat, n)
    }

    pub const fn advance_sun(n: i64) -> Self {
        Self::new(TOp::AdvSun, n)
    }

    pub const fn advance_day(n: i64) -> Self {
        Self::new(TOp::AdvDay, n)
    }

    pub const fn advance_month(n: i64) -> Self {
        Self::new(TOp::AdvMonth, n)
    }

    pub const fn find_mon(n: i64) -> Self {
        Self::new(TOp::FindMon, n)
    }

    pub const fn find_tue(n: i64) -> Self {
        Self::new(TOp::FindTue, n)
    }

    pub const fn find_wed(n: i64) -> Self {
        Self::new(TOp::FindWed, n)
    }

    pub const fn find_thu(n: i64) -> Self {
        Self::new(TOp::FindThu, n)
    }

    pub const fn find_fri(n: i64) -> Self {
        Self::new(TOp::FindFri, n)
    }

    pub const fn find_sat(n: i64) -> Self {
        Self::new(TOp::FindSat, n)
    }

    pub const fn find_sun(n: i64) -> Self {
        Self::new(TOp::FindSun, n)
    }

    pub const fn find_day(n: i64) -> Self {
        Self::new(TOp::FindDay, n)
    }

    pub const fn find_month(n: i64) -> Self {
        Self::new(TOp::FindMonth, n)
    }

    pub const fn add_years(n: i64) -> Self {
        Self::new(TOp::AddYears, n)
    }

    pub const fn yearly() -> Self {
        Self::add_years(1)
    }

    pub const fn add_months(n: i64) -> Self {
        Self::new(TOp::AddMonths, n)
    }

    pub const fn monthly() -> Self {
        Self::add_months(1)
    }

    pub const fn add_days(n: i64) -> Self {
        Self::new(TOp::AddDays, n)
    }

    pub const fn daily() -> Self {
        Self::add_days(1)
    }

    pub const fn set_year(n: i64) -> Self {
        Self::new(TOp::SetYear, n)
    }

    pub const fn set_month(n: i64) -> Self {
        Self::new(TOp::SetMonth, n)
    }

    pub const fn set_day(n: i64) -> Self {
        Self::new(TOp::SetDay, n)
    }

    pub const fn nop() -> Self {
        Self::new(TOp::Nop, 0)
    }

    pub const fn add_hours(n: i64) -> Self {
        Self::new(TOp::AddHours, n)
    }

    pub const fn hourly() -> Self {
        Self::add_hours(1)
    }

    pub const fn add_mins(n: i64) -> Self {
        Self::new(TOp::AddMins, n)
    }

    pub const fn minutely() -> Self {
        Self::add_mins(1)
    }

    pub const fn add_secs(n: i64) -> Self {
        Self::new(TOp::AddSecs, n)
    }

    pub const fn secondly() -> Self {
        Self::add_secs(1)
    }

    pub const fn add_millis(n: i64) -> Self {
        Self::new(TOp::AddMillis, n)
    }

    pub const fn add_micros(n: i64) -> Self {
        Self::new(TOp::AddMicros, n)
    }

    pub const fn add_nanos(n: i64) -> Self {
        Self::new(TOp::AddNanos, n)
    }

    pub const fn set_hour(n: i64) -> Self {
        Self::new(TOp::SetHour, n)
    }

    pub const fn set_min(n: i64) -> Self {
        Self::new(TOp::SetMin, n)
    }

    pub const fn set_sec(n: i64) -> Self {
        Self::new(TOp::SetSec, n)
    }

    pub const fn set_millis(n: i64) -> Self {
        Self::new(TOp::SetMillis, n)
    }

    pub const fn set_micros(n: i64) -> Self {
        Self::new(TOp::SetMicros, n)
    }

    pub const fn set_nanos(n: i64) -> Self {
        Self::new(TOp::SetNanos, n)
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

#[must_use]
#[derive(
    Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd, FromPrimitive, Serialize, Deserialize,
)]
pub enum DOp {
    // Adv |n| > 0 will always move to the next/previous day, even if the current day is the one in question.
    AdvMon = 0,
    AdvTue = 1,
    AdvWed = 2,
    AdvThu = 3,
    AdvFri = 4,
    AdvSat = 5,
    AdvSun = 6,
    AdvDay = 7,
    AdvMonth = 8,
    // Find* is like Adv* but won't advance if it's already the given day.
    FindMon = 9,
    FindTue = 10,
    FindWed = 11,
    FindThu = 12,
    FindFri = 13,
    FindSat = 14,
    FindSun = 15,
    FindDay = 16,
    FindMonth = 17,
    AddYears = 18,
    AddMonths = 19,
    AddDays = 20,
    SetYear = 21,
    SetMonth = 22,
    SetDay = 23,
    Nop = 24,
}

#[must_use]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct DateOp {
    op: DOp,
    n: i64,
}

impl DateOp {
    pub const fn new(op: DOp, n: i64) -> Self {
        Self { op, n }
    }

    pub const fn advance_mon(n: i64) -> Self {
        Self::new(DOp::AdvMon, n)
    }

    pub const fn advance_tue(n: i64) -> Self {
        Self::new(DOp::AdvTue, n)
    }

    pub const fn advance_wed(n: i64) -> Self {
        Self::new(DOp::AdvWed, n)
    }

    pub const fn advance_thu(n: i64) -> Self {
        Self::new(DOp::AdvThu, n)
    }

    pub const fn advance_fri(n: i64) -> Self {
        Self::new(DOp::AdvFri, n)
    }

    pub const fn advance_sat(n: i64) -> Self {
        Self::new(DOp::AdvSat, n)
    }

    pub const fn advance_sun(n: i64) -> Self {
        Self::new(DOp::AdvSun, n)
    }

    pub const fn advance_day(n: i64) -> Self {
        Self::new(DOp::AdvDay, n)
    }

    pub const fn advance_month(n: i64) -> Self {
        Self::new(DOp::AdvMonth, n)
    }

    pub const fn find_mon(n: i64) -> Self {
        Self::new(DOp::FindMon, n)
    }

    pub const fn find_tue(n: i64) -> Self {
        Self::new(DOp::FindTue, n)
    }

    pub const fn find_wed(n: i64) -> Self {
        Self::new(DOp::FindWed, n)
    }

    pub const fn find_thu(n: i64) -> Self {
        Self::new(DOp::FindThu, n)
    }

    pub const fn find_fri(n: i64) -> Self {
        Self::new(DOp::FindFri, n)
    }

    pub const fn find_sat(n: i64) -> Self {
        Self::new(DOp::FindSat, n)
    }

    pub const fn find_sun(n: i64) -> Self {
        Self::new(DOp::FindSun, n)
    }

    pub const fn find_day(n: i64) -> Self {
        Self::new(DOp::FindDay, n)
    }

    pub const fn find_month(n: i64) -> Self {
        Self::new(DOp::FindMonth, n)
    }

    pub const fn add_years(n: i64) -> Self {
        Self::new(DOp::AddYears, n)
    }

    pub const fn yearly() -> Self {
        Self::add_years(1)
    }

    pub const fn add_months(n: i64) -> Self {
        Self::new(DOp::AddMonths, n)
    }

    pub const fn monthly() -> Self {
        Self::add_months(1)
    }

    pub const fn add_days(n: i64) -> Self {
        Self::new(DOp::AddDays, n)
    }

    pub const fn daily() -> Self {
        Self::add_days(1)
    }

    pub const fn set_year(n: i64) -> Self {
        Self::new(DOp::SetYear, n)
    }

    pub const fn set_month(n: i64) -> Self {
        Self::new(DOp::SetMonth, n)
    }

    pub const fn set_day(n: i64) -> Self {
        Self::new(DOp::SetDay, n)
    }

    pub const fn nop() -> Self {
        Self::new(DOp::Nop, 0)
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
        DOp::AdvDay => {
            let n = d.with_day(n as u32);
            if n <= d {
                n.add_months(1)
            } else {
                n
            }
        }
        DOp::AdvMonth => {
            let n = d.with_month(n as u32);
            if n <= d {
                n.add_years(1)
            } else {
                n
            }
        }
        DOp::FindDay => {
            let n = d.with_day(n as u32);
            if n < d {
                n.add_months(1)
            } else {
                n
            }
        }
        DOp::FindMonth => {
            let n = d.with_month(n as u32);
            if n < d {
                n.add_years(1)
            } else {
                n
            }
        }
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

#[must_use]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct SpanOp {
    pub st: TimeOp,
    pub en: TimeOp, // Exclusive.
}

impl SpanOp {
    pub const fn new(st: TimeOp, en: TimeOp) -> Self {
        Self { st, en }
    }

    pub fn apply(&self, t: impl Into<Time>) -> SpanExc<Time> {
        let t = t.into();
        SpanExc::new(self.st.apply(t), self.en.apply(t))
    }
}

#[cfg(test)]
mod tests {
    use chrono_tz::{Australia, Tz, US, UTC};
    use eyre::Result;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::date::ymd;

    const TZ: [Tz; 3] = [US::Eastern, UTC, Australia::Eucla];

    #[test]
    fn leap_years() -> Result<()> {
        for tz in &TZ {
            assert_eq!(ymd(2020, 2, 29, tz).time()?.with_year(2019), ymd(2019, 2, 28, tz).time()?);
            assert_eq!(ymd(2020, 1, 29, tz).time()?.with_month(2), ymd(2020, 2, 29, tz).time()?);
            assert_eq!(ymd(2020, 1, 30, tz).time()?.with_month(2), ymd(2020, 2, 29, tz).time()?);
            assert_eq!(ymd(2019, 1, 29, tz).time()?.with_month(2), ymd(2019, 2, 28, tz).time()?);
            assert_eq!(ymd(2019, 1, 30, tz).time()?.with_month(2), ymd(2019, 2, 28, tz).time()?);
        }
        Ok(())
    }

    #[test]
    fn time_offset() -> Result<()> {
        for tz in &TZ {
            // Regular +1 year.
            assert_eq!(
                TimeOp::yearly().apply(ymd(2019, 1, 30, tz).time()?),
                ymd(2020, 1, 30, tz).time()?,
            );
            // Leap year to non-leap year.
            assert_eq!(
                TimeOp::yearly().apply(ymd(2020, 2, 29, tz).time()?),
                ymd(2021, 2, 28, tz).time()?,
            );
            // Month with more days to less days.
            assert_eq!(
                TimeOp::monthly().apply(ymd(2019, 1, 30, tz).time()?),
                ymd(2019, 2, 28, tz).time()?,
            );
            // Month with more days to less days in a leap year.
            assert_eq!(
                TimeOp::monthly().apply(ymd(2020, 1, 30, tz).time()?),
                ymd(2020, 2, 29, tz).time()?,
            );
            // 29th Feb to next year March.
            assert_eq!(
                TimeOp::add_months(13).apply(ymd(2020, 2, 29, tz).time()?),
                ymd(2021, 3, 29, tz).time()?,
            );
            // Leap year +1 day.
            assert_eq!(
                TimeOp::daily().apply(ymd(2020, 2, 28, tz).time()?),
                ymd(2020, 2, 29, tz).time()?,
            );
            // Non-leap year +1 day.
            assert_eq!(
                TimeOp::daily().apply(ymd(2019, 2, 28, tz).time()?),
                ymd(2019, 3, 1, tz).time()?,
            );
            // Advance Monday (+2) on a Sunday.
            assert_eq!(
                TimeOp::advance_mon(2).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2020, 12, 14, tz).time()?,
            );
            // Advance Monday (+2) on a Monday.
            assert_eq!(
                TimeOp::advance_mon(2).apply(ymd(2020, 12, 7, tz).time()?),
                ymd(2020, 12, 21, tz).time()?,
            );
            // Advance Monday (+1) on a Sunday.
            assert_eq!(
                TimeOp::advance_mon(1).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2020, 12, 7, tz).time()?,
            );
            // Advance Monday (+1) on a Monday.
            assert_eq!(
                TimeOp::advance_mon(1).apply(ymd(2020, 12, 7, tz).time()?),
                ymd(2020, 12, 14, tz).time()?,
            );
            // Advance Monday (+0) on a Sunday.
            assert_eq!(
                TimeOp::advance_mon(0).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2020, 12, 7, tz).time()?,
            );
            // Advance Monday (+0) on a Monday.
            assert_eq!(
                TimeOp::advance_mon(0).apply(ymd(2020, 12, 7, tz).time()?),
                ymd(2020, 12, 7, tz).time()?,
            );
            // Neg advance Monday (-1) on a Sunday.
            assert_eq!(
                TimeOp::advance_mon(-1).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2020, 11, 30, tz).time()?,
            );
            // Neg advance Monday (-1) on a Monday.
            assert_eq!(
                TimeOp::advance_mon(-1).apply(ymd(2020, 12, 7, tz).time()?),
                ymd(2020, 11, 30, tz).time()?,
            );
            // Find Monday (+2) on a Sunday.
            assert_eq!(
                TimeOp::find_mon(2).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2020, 12, 14, tz).time()?,
            );
            // Find Monday (+2) on a Monday.
            assert_eq!(
                TimeOp::find_mon(2).apply(ymd(2020, 12, 7, tz).time()?),
                ymd(2020, 12, 14, tz).time()?,
            );
            // Find Monday (+1) on a Sunday.
            assert_eq!(
                TimeOp::find_mon(1).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2020, 12, 7, tz).time()?,
            );
            // Find Monday (+1) on a Monday.
            assert_eq!(
                TimeOp::find_mon(1).apply(ymd(2020, 12, 7, tz).time()?),
                ymd(2020, 12, 7, tz).time()?,
            );
            // Find Monday (+0) on a Sunday.
            assert_eq!(
                TimeOp::find_mon(0).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2020, 12, 7, tz).time()?,
            );
            // Find Monday (+0) on a Monday.
            assert_eq!(
                TimeOp::find_mon(0).apply(ymd(2020, 12, 7, tz).time()?),
                ymd(2020, 12, 7, tz).time()?,
            );
            // Neg find Monday (-1) on a Sunday.
            assert_eq!(
                TimeOp::find_mon(-1).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2020, 11, 30, tz).time()?,
            );
            // Neg find Monday (-1) on a Monday.
            assert_eq!(
                TimeOp::find_mon(-1).apply(ymd(2020, 12, 7, tz).time()?),
                ymd(2020, 12, 7, tz).time()?,
            );

            // AdvDay to the next day with the same day number
            assert_eq!(
                TimeOp::advance_day(6).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2021, 1, 6, tz).time()?,
            );

            assert_eq!(
                TimeOp::advance_day(6).apply(ymd(2021, 1, 5, tz).time()?),
                ymd(2021, 1, 6, tz).time()?,
            );

            // AdvMonth to the next month with the same month number
            assert_eq!(
                TimeOp::advance_month(12).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2021, 12, 6, tz).time()?,
            );

            assert_eq!(
                TimeOp::advance_month(12).apply(ymd(2021, 11, 6, tz).time()?),
                ymd(2021, 12, 6, tz).time()?,
            );

            // FindDay for the current day number
            assert_eq!(
                TimeOp::find_day(6).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2020, 12, 6, tz).time()?,
            );

            // FindDay for the next day number
            assert_eq!(
                TimeOp::find_day(7).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2020, 12, 7, tz).time()?,
            );

            // FindMonth for the current month number
            assert_eq!(
                TimeOp::find_month(12).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2020, 12, 6, tz).time()?,
            );

            // FindMonth for the next month number
            assert_eq!(
                TimeOp::find_month(1).apply(ymd(2020, 12, 6, tz).time()?),
                ymd(2021, 1, 6, tz).time()?,
            );
        }
        Ok(())
    }
}
