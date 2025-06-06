use std::sync::LazyLock;

use chrono_tz::US::Eastern;

use crate::calendars::calendar::DaySet;
use crate::date::{Date, Day, ymd};
use crate::iter::DateIter;
use crate::op::DateOp;

#[allow(clippy::unnecessary_wraps)]
fn sun_to_mon(d: Date) -> Option<Date> {
    Some(if d.weekday() == Day::Sun { d.add_days(1) } else { d })
}

#[allow(clippy::unnecessary_wraps)]
fn is_mon_to_thu(d: Date) -> Option<Date> {
    (Day::Mon..=Day::Thu).contains(&d.weekday()).then_some(d)
}

#[allow(clippy::unnecessary_wraps)]
fn nearest_workday(d: Date) -> Option<Date> {
    Some(match d.weekday() {
        Day::Sat => d.add_days(-1),
        Day::Sun => d.add_days(1),
        _ => d,
    })
}

fn next_tuesday_every_four_years(d: Date) -> Option<Date> {
    if d.year() % 4 == 0 { Some(DateOp::find_tue(1).apply(d)) } else { None }
}

#[allow(clippy::unnecessary_wraps)]
fn day_after_4th_thu(d: Date) -> Option<Date> {
    let d = DateOp::find_thu(4).apply(d);
    Some(DateOp::add_days(1).apply(d))
}

#[allow(clippy::many_single_char_names, clippy::unnecessary_wraps)]
fn easter(d: Date) -> Option<Date> {
    let y = d.year();
    assert!((1583..=4099).contains(&y), "easter calculation not valid in year {y}");
    let g = y % 19;
    let c = y / 100;
    let h = (c - c / 4 - (8 * c + 13) / 25 + 19 * g + 15) % 30;
    let i = h - (h / 28) * (1 - (h / 28) * (29 / (h + 1)) * ((21 - g) / 11));
    let j = (y + y / 4 + i + 2 - c + c / 4) % 7;
    let p = i - j;
    let day = 1 + (p + 27 + (p + 6) / 40) % 31;
    let m = 3 + (p + 26) / 30;
    Some(ymd(y, m as u32, day as u32, d.tz()))
}

// TODO: Add extra at http://s3.amazonaws.com/armstrongeconomics-wp/2013/07/NYSE-Closings.pdf
pub static SATURDAY: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_observance(|d: Date| (d.weekday() == Day::Sat).then_some(d))
});
pub static SUNDAY: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_observance(|d: Date| (d.weekday() == Day::Sun).then_some(d))
});
pub static GOOD_FRIDAY: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(1, 1)
        .with_observance(|d| easter(d).map(|d| DateOp::add_days(-2).apply(d)))
});
pub static US_NEW_YEARS_DAY: LazyLock<DaySet> =
    LazyLock::new(|| DaySet::new().with_md(1, 1).with_observance(sun_to_mon));
pub static US_MARTIN_LUTHER_KING_JR_AFTER1998: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(1, 1)
        .with_start(ymd(1998, 1, 1, Eastern))
        .with_observance(|d| Some(DateOp::find_mon(3).apply(d)))
});
pub static US_PRESIDENTS_DAY: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(2, 1)
        .with_start(ymd(1971, 1, 1, Eastern))
        .with_observance(|d| Some(DateOp::find_mon(3).apply(d)))
});
pub static US_LINCOLNS_BIRTH_DAY_BEFORE1954: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(2, 12)
        .with_start(ymd(1896, 1, 1, Eastern))
        .with_end(ymd(1953, 12, 31, Eastern))
        .with_observance(sun_to_mon)
});
pub static US_WASHINGTONS_BIRTH_DAY_BEFORE1964: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(2, 22)
        .with_start(ymd(1880, 1, 1, Eastern))
        .with_end(ymd(1963, 12, 31, Eastern))
        .with_observance(sun_to_mon)
});
pub static US_WASHINGTONS_BIRTH_DAY1964TO1970: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(2, 22)
        .with_start(ymd(1964, 1, 1, Eastern))
        .with_end(ymd(1970, 12, 31, Eastern))
        .with_observance(nearest_workday)
});
pub static US_MEMORIAL_DAY: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(5, 25)
        .with_start(ymd(1971, 1, 1, Eastern))
        .with_observance(|d| Some(DateOp::find_mon(1).apply(d)))
});
pub static US_MEMORIAL_DAY_BEFORE1964: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_md(5, 30).with_end(ymd(1963, 12, 31, Eastern)).with_observance(sun_to_mon)
});
pub static US_MEMORIAL_DAY1964TO1969: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(5, 30)
        .with_start(ymd(1964, 1, 1, Eastern))
        .with_end(ymd(1969, 12, 31, Eastern))
        .with_observance(nearest_workday)
});
pub static MON_TUES_THURS_BEFORE_INDEPENDENCE_DAY: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_md(7, 3).with_start(ymd(1995, 1, 1, Eastern)).with_observance(|d: Date| {
        [Day::Mon, Day::Tue, Day::Thu].contains(&d.weekday()).then_some(d)
    })
});
pub static WEDNESDAY_BEFORE_INDEPENDENCE_DAY_POST2013: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(7, 3)
        .with_start(ymd(2013, 1, 1, Eastern))
        .with_observance(|d: Date| (d.weekday() == Day::Wed).then_some(d))
});
pub static US_INDEPENDENCE_DAY_BEFORE1954: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_md(7, 4).with_end(ymd(1953, 12, 31, Eastern)).with_observance(sun_to_mon)
});
pub static US_INDEPENDENCE_DAY: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(7, 4)
        .with_start(ymd(1954, 1, 1, Eastern))
        .with_observance(nearest_workday)
});
pub static FRIDAY_AFTER_INDEPENDENCE_DAY_PRE2013: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(7, 5)
        .with_start(ymd(1995, 1, 1, Eastern))
        .with_end(ymd(2013, 1, 1, Eastern))
        .with_observance(|d: Date| (d.weekday() == Day::Fri).then_some(d))
});
pub static US_LABOR_DAY: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_md(9, 1).with_observance(|d| Some(DateOp::find_mon(1).apply(d)))
});
pub static US_COLUMBUS_DAY_BEFORE1954: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_md(10, 12).with_end(ymd(1953, 12, 31, Eastern)).with_observance(sun_to_mon)
});
pub static US_THANKSGIVING_DAY: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(11, 1)
        .with_start(ymd(1942, 1, 1, Eastern))
        .with_observance(|d| Some(DateOp::find_thu(4).apply(d)))
});
pub static US_BLACK_FRIDAY_BEFORE1993: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(11, 1)
        .with_start(ymd(1992, 1, 1, Eastern))
        .with_end(ymd(1993, 1, 1, Eastern))
        .with_observance(day_after_4th_thu)
});
pub static US_BLACK_FRIDAY_IN_OR_AFTER1993: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(11, 1)
        .with_start(ymd(1993, 1, 1, Eastern))
        .with_observance(day_after_4th_thu)
});
pub static US_ELECTION_DAY1848TO1967: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(11, 2)
        .with_start(ymd(1848, 1, 1, Eastern))
        .with_end(ymd(1967, 12, 31, Eastern))
        .with_observance(|d| Some(DateOp::find_tue(1).apply(d)))
});
pub static US_ELECTION_DAY1968TO1980: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(11, 2)
        .with_start(ymd(1968, 1, 1, Eastern))
        .with_end(ymd(1980, 12, 31, Eastern))
        .with_observance(next_tuesday_every_four_years)
});
pub static US_VETERANS_DAY1934TO1953: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(11, 11)
        .with_start(ymd(1934, 1, 1, Eastern))
        .with_end(ymd(1953, 12, 31, Eastern))
        .with_observance(sun_to_mon)
});
pub static US_THANKSGIVING_DAY_BEFORE1939: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(11, 30)
        .with_start(ymd(1864, 1, 1, Eastern))
        .with_end(ymd(1938, 12, 31, Eastern))
        .with_observance(|d| Some(DateOp::find_thu(-1).apply(d)))
});
pub static US_THANKSGIVING_DAY1939TO1941: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(11, 30)
        .with_start(ymd(1939, 1, 1, Eastern))
        .with_end(ymd(1941, 12, 31, Eastern))
        .with_observance(|d| Some(DateOp::find_thu(-2).apply(d)))
});
pub static CHRISTMAS_EVE_BEFORE1993: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_md(12, 24).with_end(ymd(1993, 1, 1, Eastern)).with_observance(is_mon_to_thu)
});
pub static CHRISTMAS_EVE_IN_OR_AFTER1993: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(12, 24)
        .with_start(ymd(1993, 1, 1, Eastern))
        .with_observance(is_mon_to_thu)
});
pub static CHRISTMAS_BEFORE1954: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_md(12, 25).with_end(ymd(1953, 12, 31, Eastern)).with_observance(sun_to_mon)
});
pub static CHRISTMAS: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new()
        .with_md(12, 25)
        .with_start(ymd(1954, 1, 1, Eastern))
        .with_observance(nearest_workday)
});
pub static BATTLE_OF_GETTYSBURG: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_adhoc(DateIter::day(ymd(1863, 7, 1, Eastern), ymd(1863, 7, 4, Eastern)))
});
pub static NOVEMBER29_BACKLOG_RELIEF: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_adhoc([ymd(1929, 11, 1, Eastern), ymd(1929, 11, 29, Eastern)])
});
pub static MARCH33_BANK_HOLIDAY: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_adhoc(DateIter::day(ymd(1933, 3, 6, Eastern), ymd(1933, 3, 15, Eastern)))
});
pub static AUGUST45_VICTORY_OVER_JAPAN: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_adhoc([ymd(1945, 8, 15, Eastern), ymd(1945, 8, 16, Eastern)])
});
pub static CHRISTMAS_EVES_ADHOC: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_adhoc([ymd(1945, 12, 24, Eastern), ymd(1956, 12, 24, Eastern)])
});
pub static DAY_AFTER_CHRISTMAS_ADHOC: LazyLock<DaySet> =
    LazyLock::new(|| DaySet::new().with_adhoc([ymd(1958, 12, 26, Eastern)]));
pub static DAY_BEFORE_DECORATION_ADHOC: LazyLock<DaySet> =
    LazyLock::new(|| DaySet::new().with_adhoc([ymd(1961, 5, 29, Eastern)]));
pub static LINCOLNS_BIRTH_DAY_ADHOC: LazyLock<DaySet> =
    LazyLock::new(|| DaySet::new().with_adhoc([ymd(1968, 2, 12, Eastern)]));
pub static PAPERWORK_CRISIS68: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_adhoc([
        ymd(1968, 6, 12, Eastern),
        ymd(1968, 6, 19, Eastern),
        ymd(1968, 6, 26, Eastern),
        ymd(1968, 7, 10, Eastern),
        ymd(1968, 7, 17, Eastern),
        ymd(1968, 7, 24, Eastern),
        ymd(1968, 7, 31, Eastern),
        ymd(1968, 8, 7, Eastern),
        ymd(1968, 8, 14, Eastern),
        ymd(1968, 8, 21, Eastern),
        ymd(1968, 8, 28, Eastern),
        ymd(1968, 9, 11, Eastern),
        ymd(1968, 9, 18, Eastern),
        ymd(1968, 9, 25, Eastern),
        ymd(1968, 10, 2, Eastern),
        ymd(1968, 10, 9, Eastern),
        ymd(1968, 10, 16, Eastern),
        ymd(1968, 10, 23, Eastern),
        ymd(1968, 10, 30, Eastern),
        ymd(1968, 11, 11, Eastern),
        ymd(1968, 11, 20, Eastern),
        ymd(1968, 12, 4, Eastern),
        ymd(1968, 12, 11, Eastern),
        ymd(1968, 12, 18, Eastern),
        ymd(1968, 12, 25, Eastern),
    ])
});
pub static DAY_AFTER_INDEPENDENCE_DAY_ADHOC: LazyLock<DaySet> =
    LazyLock::new(|| DaySet::new().with_adhoc([ymd(1968, 7, 5, Eastern)]));
pub static WEATHER_SNOW_CLOSING: LazyLock<DaySet> =
    LazyLock::new(|| DaySet::new().with_adhoc([ymd(1969, 2, 10, Eastern)]));
pub static FIRST_LUNAR_LANDING_CLOSING: LazyLock<DaySet> =
    LazyLock::new(|| DaySet::new().with_adhoc([ymd(1969, 7, 21, Eastern)]));
pub static NEW_YORK_CITY_BLACKOUT77: LazyLock<DaySet> =
    LazyLock::new(|| DaySet::new().with_adhoc([ymd(1977, 7, 14, Eastern)]));
pub static SEPTEMBER11_CLOSINGS: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_adhoc(DateIter::day(ymd(2001, 9, 11, Eastern), ymd(2001, 9, 17, Eastern)))
});
pub static HURRICANE_SANDY_CLOSINGS: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_adhoc([ymd(2012, 10, 29, Eastern), ymd(2012, 10, 30, Eastern)])
});
pub static HURRICANE_GLORIA_CLOSING: LazyLock<DaySet> =
    LazyLock::new(|| DaySet::new().with_adhoc([ymd(1985, 9, 27, Eastern)]));
pub static US_NATIONAL_DAYSOF_MOURNING: LazyLock<DaySet> = LazyLock::new(|| {
    DaySet::new().with_adhoc([
        ymd(1963, 11, 25, Eastern),
        ymd(1968, 4, 9, Eastern),
        ymd(1969, 3, 31, Eastern),
        ymd(1972, 12, 28, Eastern),
        ymd(1973, 1, 25, Eastern),
        ymd(1994, 4, 27, Eastern),
        ymd(2004, 6, 11, Eastern),
        ymd(2007, 1, 2, Eastern),
        ymd(2018, 12, 5, Eastern),
    ])
});
