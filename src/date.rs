use chrono::{Datelike, Month, TimeZone};
use chrono_tz::{Tz, UTC};
use derive_more::Display;
use num_traits::FromPrimitive;

use crate::op::{DOp, DateOp};

#[must_use]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd)]
pub enum Day {
    Mon = 0,
    Tue = 1,
    Wed = 2,
    Thu = 3,
    Fri = 4,
    Sat = 5,
    Sun = 6,
}

#[must_use]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Display, Ord, PartialOrd)]
#[display(fmt = "{d}")]
pub struct Date {
    d: chrono::Date<Tz>,
}

impl Default for Date {
    fn default() -> Self {
        Self::new(UTC.ymd(1970, 1, 1))
    }
}

impl From<chrono::Date<Tz>> for Date {
    fn from(v: chrono::Date<Tz>) -> Self {
        Self::new(v)
    }
}

impl From<&chrono::Date<Tz>> for Date {
    fn from(v: &chrono::Date<Tz>) -> Self {
        Self::new(*v)
    }
}

impl From<Date> for chrono::Date<Tz> {
    fn from(v: Date) -> Self {
        v.inner()
    }
}

impl Date {
    pub fn new(d: chrono::Date<Tz>) -> Self {
        Self { d }
    }

    #[must_use]
    pub fn format(&self, f: &str) -> String {
        self.d.format(f).to_string()
    }

    #[must_use]
    pub fn inner(&self) -> chrono::Date<Tz> {
        self.d
    }

    pub fn op(op: DOp, n: i64) -> DateOp {
        DateOp::new(op, n)
    }

    #[must_use]
    pub fn tz(&self) -> Tz {
        self.d.timezone()
    }

    #[must_use]
    pub fn day(&self) -> u32 {
        self.d.day()
    }

    pub fn with_day(self, d: u32) -> Self {
        for max in (28..=31).rev() {
            if let Some(res) = self.d.with_day(d.clamp(1, max)) {
                return res.into();
            }
        }
        panic!("bug: invalid day {d}");
    }

    pub fn add_days(self, d: i32) -> Self {
        (self.d + chrono::Duration::days(i64::from(d))).into()
    }

    pub fn weekday(&self) -> Day {
        match self.d.weekday() {
            chrono::Weekday::Mon => Day::Mon,
            chrono::Weekday::Tue => Day::Tue,
            chrono::Weekday::Wed => Day::Wed,
            chrono::Weekday::Thu => Day::Thu,
            chrono::Weekday::Fri => Day::Fri,
            chrono::Weekday::Sat => Day::Sat,
            chrono::Weekday::Sun => Day::Sun,
        }
    }

    #[must_use]
    pub fn month_name(&self) -> String {
        Month::from_u32(self.month()).unwrap().name().to_owned()
    }

    #[must_use]
    pub fn month0(&self) -> u32 {
        self.d.month0()
    }

    #[must_use]
    pub fn month(&self) -> u32 {
        self.d.month()
    }

    pub fn with_month(self, m: u32) -> Self {
        let d = self.day();
        Self::new(self.with_day(1).d.with_month(m).unwrap()).with_day(d)
    }

    pub fn add_months(self, add_m: i32) -> Self {
        let d = self.day();
        let total_m = self.month0() as i32 + add_m;
        let y = total_m.div_euclid(12) + self.year();
        let m = total_m.rem_euclid(12) as u32 + 1;
        Self::new(self.tz().ymd(y, m, 1)).with_day(d)
    }

    #[must_use]
    pub fn year(&self) -> i32 {
        self.d.year()
    }

    pub fn with_year(self, y: i32) -> Self {
        let d = self.day();
        Self::new(self.with_day(1).d.with_year(y).unwrap()).with_day(d)
    }

    pub fn add_years(self, y: i32) -> Self {
        self.with_year(self.year() + y)
    }
}
