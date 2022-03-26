use std::lazy::SyncLazy;

use chrono::TimeZone;
use chrono_tz::US::Eastern;

use crate::calendars::calendar::{Calendar, DaySet};
use crate::calendars::us_holidays::{
    AUGUST45_VICTORY_OVER_JAPAN, CHRISTMAS, CHRISTMAS_BEFORE1954, CHRISTMAS_EVES_ADHOC,
    CHRISTMAS_EVE_BEFORE1993, CHRISTMAS_EVE_IN_OR_AFTER1993, DAY_AFTER_CHRISTMAS_ADHOC,
    DAY_AFTER_INDEPENDENCE_DAY_ADHOC, DAY_BEFORE_DECORATION_ADHOC, FIRST_LUNAR_LANDING_CLOSING,
    FRIDAY_AFTER_INDEPENDENCE_DAY_PRE2013, GOOD_FRIDAY, HURRICANE_GLORIA_CLOSING,
    HURRICANE_SANDY_CLOSINGS, LINCOLNS_BIRTH_DAY_ADHOC, MARCH33_BANK_HOLIDAY,
    MON_TUES_THURS_BEFORE_INDEPENDENCE_DAY, NEW_YORK_CITY_BLACKOUT77, NOVEMBER29_BACKLOG_RELIEF,
    PAPERWORK_CRISIS68, SATURDAY, SEPTEMBER11_CLOSINGS, SUNDAY, US_BLACK_FRIDAY_BEFORE1993,
    US_BLACK_FRIDAY_IN_OR_AFTER1993, US_COLUMBUS_DAY_BEFORE1954, US_ELECTION_DAY1848TO1967,
    US_ELECTION_DAY1968TO1980, US_INDEPENDENCE_DAY, US_INDEPENDENCE_DAY_BEFORE1954, US_LABOR_DAY,
    US_LINCOLNS_BIRTH_DAY_BEFORE1954, US_MARTIN_LUTHER_KING_JR_AFTER1998, US_MEMORIAL_DAY,
    US_MEMORIAL_DAY1964TO1969, US_MEMORIAL_DAY_BEFORE1964, US_NATIONAL_DAYSOF_MOURNING,
    US_NEW_YEARS_DAY, US_PRESIDENTS_DAY, US_THANKSGIVING_DAY, US_THANKSGIVING_DAY1939TO1941,
    US_THANKSGIVING_DAY_BEFORE1939, US_VETERANS_DAY1934TO1953, US_WASHINGTONS_BIRTH_DAY1964TO1970,
    US_WASHINGTONS_BIRTH_DAY_BEFORE1964, WEATHER_SNOW_CLOSING,
    WEDNESDAY_BEFORE_INDEPENDENCE_DAY_POST2013,
};
use crate::op::{SpanOp, TOp};
use crate::time::Time;

// Exchange calendar for NYSE
//
// Open Time: 9:30 AM, US/Eastern
// Close Time: 4:00 PM, US/Eastern
//
// Regularly-Observed SpanSets:
// - New Years Day (observed on monday when Jan 1 is a Sunday)
// - Martin Luther King Jr. Day (3rd Monday in January, only after 1998)
// - Lincoln's Birthday (February 12th, only before 1954)
// - Washington's Birthday (February 22nd, before 1971 with rule change in
//   1964)
// - Washington's Birthday (aka President's Day, 3rd Monday in February,
//   after 1970)
// - Good Friday (two days before Easter Sunday)
// - Memorial Day (May 30th, before 1970, with rule change in 1964)
// - Memorial Day (last Monday in May, after 1970)
// - Independence Day (July 4th Sunday to Monday, before 1954)
// - Independence Day (observed on the nearest weekday to July 4th, after
//   1953)
// - Election Day (First Tuesday starting on November 2nd, between 1848 and
//   1967)
// - Election Day (Every four years, first Tuesday starting on November 2nd,
//   between 1968 and 1980)
// - Veterans Day (November 11th, between 1934 and 1953)
// - Columbus Day (October 12th, before 1954)
// - Labor Day (first Monday in September)
// - Thanksgiving (last Thursday in November, before 1939)
// - Thanksgiving (second to last Thursday in November, between 1939 and 1941)
// - Thanksgiving (fourth Thursday in November, after 1941)
// - Christmas (December 25th, Sunday to Monday, before 1954)
// - Christmas (observed on nearest weekday to December 25, after 1953)
//
// NOTE: The NYSE does not observe the following US Federal SpanSets:
// - Columbus Day (after 1953)
// - Veterans Day (after 1953)
//
// Regularly-Observed Early Closes:
// - July 3rd (Mondays, Tuesdays, and Thursdays, 1995 onward)
// - July 5th (Fridays, 1995 onward, except 2013)
// - Christmas Eve (except on Fridays, when the exchange is closed entirely)
// - Day After Thanksgiving (aka Black Friday, observed from 1992 onward)
//
// NOTE: Until 1993, the standard early close time for the NYSE was 2:00 PM.
// From 1993 onward, it has been 1:00 PM.
//
// Additional Irregularities:
// - Closed on 11/1/1929 and 11/29/1929 for backlog relief.
// - Closed between 3/6/1933 and 3/14/1933 due to bank holiday.
// - Closed on 8/15/1945 and 8/16/1945 following victory over Japan.
// - Closed on Christmas Eve in 1945 and 1946.
// - Closed on December 26th in 1958.
// - Closed the day before Memorial Day in 1961.
// - Closed on 11/25/1963 due to John F. Kennedy's death.
// - Closed for Lincoln's Birthday in 1968.
// - Closed a number of days between June 12th and  December 24th in 1968
//   due to paperwork crisis.
// - Closed on 4/9/1968 due to Martin Luther King's death.
// - Closed the day after Independence Day in 1968.
// - Closed on 2/10/1969 due to weather (snow).
// - Closed on 3/31/1969 due to Dwight D. Eisenhower's death.
// - Closed on 7/21/1969 following the first lunar landing.
// - Closed on 12/28/1972 due to Harry S. Truman's death.
// - Closed on 1/25/1973 due to Lyndon B. Johnson's death.
// - Closed on 7/14/1977 due to New York City outage.
// - Closed on 9/27/1985 due to Hurricane Gloria.
// - Closed on 4/27/1994 due to Richard Nixon's death.
// - Closed from 9/11/2001 to 9/16/2001 due to terrorist attacks in NYC.
// - Closed on 6/11/2004 due to Ronald Reagan's death.
// - Closed on 1/2/2007 due to Gerald Ford's death.
// - Closed on 10/29/2012 and 10/30/2012 due to Hurricane Sandy.
// - Closed on 12/5/2018 due to George H.W. Bush's death.
// - Closed at 1:00 PM on Wednesday, July 3rd, 2013
// - Closed at 1:00 PM on Friday, December 31, 1999
// - Closed at 1:00 PM on Friday, December 26, 1997
// - Closed at 1:00 PM on Friday, December 26, 2003
//
// NOTE: The exchange was **not** closed early on Friday December 26, 2008,
// nor was it closed on Friday December 26, 2014. The next Thursday Christmas
// will be in 2025.

static NYSE_SPECIAL: SyncLazy<DaySet> = SyncLazy::new(|| {
    DaySet::new().with_adhoc(&[
        Eastern.ymd(1997, 12, 26),
        Eastern.ymd(1999, 12, 31),
        Eastern.ymd(2003, 12, 26),
        Eastern.ymd(2013, 7, 3),
    ])
});

#[must_use]
pub fn get_nyse() -> Calendar {
    Calendar::new(
        "NYSE",
        Eastern,
        &[SpanOp::new(Time::op(TOp::AddMins, 570), Time::op(TOp::AddHours, 16))],
        &[
            &SATURDAY,
            &SUNDAY,
            &US_NEW_YEARS_DAY,
            &US_MARTIN_LUTHER_KING_JR_AFTER1998,
            &US_LINCOLNS_BIRTH_DAY_BEFORE1954,
            &US_WASHINGTONS_BIRTH_DAY_BEFORE1964,
            &US_WASHINGTONS_BIRTH_DAY1964TO1970,
            &US_PRESIDENTS_DAY,
            &GOOD_FRIDAY,
            &US_MEMORIAL_DAY_BEFORE1964,
            &US_MEMORIAL_DAY1964TO1969,
            &US_MEMORIAL_DAY,
            &US_INDEPENDENCE_DAY_BEFORE1954,
            &US_INDEPENDENCE_DAY,
            &US_LABOR_DAY,
            &US_THANKSGIVING_DAY_BEFORE1939,
            &US_THANKSGIVING_DAY1939TO1941,
            &US_THANKSGIVING_DAY,
            &US_ELECTION_DAY1848TO1967,
            &US_ELECTION_DAY1968TO1980,
            &US_VETERANS_DAY1934TO1953,
            &US_COLUMBUS_DAY_BEFORE1954,
            &CHRISTMAS_BEFORE1954,
            &CHRISTMAS,
            &NOVEMBER29_BACKLOG_RELIEF,
            &MARCH33_BANK_HOLIDAY,
            &AUGUST45_VICTORY_OVER_JAPAN,
            &CHRISTMAS_EVES_ADHOC,
            &DAY_AFTER_CHRISTMAS_ADHOC,
            &DAY_BEFORE_DECORATION_ADHOC,
            &LINCOLNS_BIRTH_DAY_ADHOC,
            &PAPERWORK_CRISIS68,
            &DAY_AFTER_INDEPENDENCE_DAY_ADHOC,
            &WEATHER_SNOW_CLOSING,
            &FIRST_LUNAR_LANDING_CLOSING,
            &SEPTEMBER11_CLOSINGS,
            &NEW_YORK_CITY_BLACKOUT77,
            &HURRICANE_GLORIA_CLOSING,
            &HURRICANE_SANDY_CLOSINGS,
            &US_NATIONAL_DAYSOF_MOURNING,
        ],
        &[
            (
                &[SpanOp::new(Time::op(TOp::AddMins, 570), Time::op(TOp::AddHours, 13))],
                &[
                    &MON_TUES_THURS_BEFORE_INDEPENDENCE_DAY,
                    &FRIDAY_AFTER_INDEPENDENCE_DAY_PRE2013,
                    &WEDNESDAY_BEFORE_INDEPENDENCE_DAY_POST2013,
                    &US_BLACK_FRIDAY_IN_OR_AFTER1993,
                    &CHRISTMAS_EVE_IN_OR_AFTER1993,
                    &NYSE_SPECIAL,
                ],
            ),
            (
                &[SpanOp::new(Time::op(TOp::AddMins, 570), Time::op(TOp::AddHours, 14))],
                &[&CHRISTMAS_EVE_BEFORE1993, &US_BLACK_FRIDAY_BEFORE1993],
            ),
        ],
    )
}
