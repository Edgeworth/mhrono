use chrono_tz::US::Eastern;
use eyre::Result;
use mhrono::cycles::Cycles;
use mhrono::date::{ymd, Day};
use mhrono::duration::Duration;
use mhrono::iter::{DateIter, TimeIter};
use mhrono::op::{DateOp, TimeOp};
use mhrono::span::exc::SpanExc;
use mhrono::span::inc::SpanInc;
use mhrono::time::ymdhms;
use rust_decimal_macros::dec;

#[test]
fn test_date_iteration_with_weekday_filtering() -> Result<()> {
    // Test iterating over dates and filtering by weekday
    let start = ymd(2020, 3, 9, Eastern); // Monday
    let end = ymd(2020, 3, 16, Eastern);

    let dates: Vec<_> = DateIter::day(start, end).collect();
    let weekdays: Vec<_> = dates.iter().map(|d| d.weekday()).collect();

    assert_eq!(dates.len(), 7);
    assert_eq!(weekdays[0], Day::Mon);
    assert_eq!(weekdays[1], Day::Tue);
    assert_eq!(weekdays[2], Day::Wed);
    assert_eq!(weekdays[3], Day::Thu);
    assert_eq!(weekdays[4], Day::Fri);
    assert_eq!(weekdays[5], Day::Sat);
    assert_eq!(weekdays[6], Day::Sun);

    // Filter for weekdays only (Mon-Fri)
    let weekdays_only: Vec<_> = dates
        .iter()
        .filter(|d| {
            matches!(
                d.weekday(),
                Day::Mon | Day::Tue | Day::Wed | Day::Thu | Day::Fri
            )
        })
        .collect();
    assert_eq!(weekdays_only.len(), 5);

    Ok(())
}

#[test]
fn test_time_iteration_with_duration() -> Result<()> {
    // Test that TimeIter produces times separated by the correct duration
    let start = ymdhms(2020, 1, 1, 0, 0, 0, Eastern);
    let end = ymdhms(2020, 1, 1, 3, 0, 0, Eastern);
    let op = TimeOp::hourly();

    let times: Vec<_> = TimeIter::new(start, end, op).collect();

    // Verify times are 1 hour apart
    for i in 0..times.len() - 1 {
        let diff = times[i + 1] - times[i];
        assert_eq!(diff, Duration::HOUR);
    }

    Ok(())
}

#[test]
fn test_date_operations_across_months() -> Result<()> {
    // Test date operations that cross month boundaries
    let jan31 = ymd(2020, 1, 31, Eastern);

    // Adding months should handle day clamping correctly
    let feb = jan31.add_months(1);
    assert_eq!(feb.month(), 2);
    assert_eq!(feb.day(), 29); // 2020 is a leap year

    let mar = feb.add_months(1);
    assert_eq!(mar.month(), 3);
    assert_eq!(mar.day(), 29); // Should maintain day 29

    // Test iteration across month boundary
    let start = ymd(2020, 1, 30, Eastern);
    let end = ymd(2020, 2, 3, Eastern);
    let dates: Vec<_> = DateIter::day(start, end).collect();

    assert_eq!(dates.len(), 4);
    assert_eq!(dates[0].month(), 1);
    assert_eq!(dates[1].month(), 1);
    assert_eq!(dates[2].month(), 2);
    assert_eq!(dates[3].month(), 2);

    Ok(())
}

#[test]
fn test_span_operations_with_dates() -> Result<()> {
    // Test span operations with date types
    let d1 = ymd(2020, 1, 1, Eastern);
    let d2 = ymd(2020, 1, 10, Eastern);
    let d3 = ymd(2020, 1, 5, Eastern);
    let d4 = ymd(2020, 1, 15, Eastern);

    let span1 = SpanExc::new(d1, d2);
    let span2 = SpanExc::new(d3, d4);

    // Test containment
    assert!(span1.contains(&d1));
    assert!(span1.contains(&ymd(2020, 1, 5, Eastern)));
    assert!(!span1.contains(&d2)); // Exclusive end

    // Test intersection
    let intersection = span1.intersect(&span2);
    assert!(intersection.is_some());
    let inter = intersection.unwrap();
    assert_eq!(inter.st, d3);
    assert_eq!(inter.en, d2);

    Ok(())
}

#[test]
fn test_inclusive_span_operations() -> Result<()> {
    let d1 = ymd(2020, 1, 1, Eastern);
    let d2 = ymd(2020, 1, 10, Eastern);

    let span = SpanInc::new(d1, d2);

    // Inclusive spans should contain both endpoints
    assert!(span.contains(&d1));
    assert!(span.contains(&d2));
    assert!(span.contains(&ymd(2020, 1, 5, Eastern)));
    assert!(!span.contains(&ymd(2019, 12, 31, Eastern)));
    assert!(!span.contains(&ymd(2020, 1, 11, Eastern)));

    Ok(())
}

#[test]
fn test_duration_with_cycles() -> Result<()> {
    // Test that duration and cycles interact correctly
    let dur = Duration::SEC;
    let cycles = Cycles::from_count(10);

    // cycles * duration = duration
    let total_duration = cycles * dur;
    assert_eq!(total_duration, Duration::new(dec!(10)));

    // cycles / duration = frequency
    let freq = cycles / dur;
    assert_eq!(freq.hz(), dec!(10));

    Ok(())
}

#[test]
fn test_time_with_timezone_conversion() -> Result<()> {
    let t = ymdhms(2020, 3, 15, 12, 0, 0, Eastern);

    // Extract date and verify it preserves timezone
    let d = t.date();
    assert_eq!(d.tz(), Eastern);
    assert_eq!(d.year(), 2020);
    assert_eq!(d.month(), 3);
    assert_eq!(d.day(), 15);

    // Convert back to time and verify
    let t2 = d.and_hms(12, 0, 0)?;
    assert_eq!(t2, t);

    Ok(())
}

#[test]
fn test_complex_date_iteration() -> Result<()> {
    // Test iteration with non-trivial date operation
    let start = ymd(2020, 1, 1, Eastern);
    let end = ymd(2023, 1, 1, Eastern);

    let dates: Vec<_> = DateIter::year(start, end).collect();

    assert_eq!(dates.len(), 3);
    assert_eq!(dates[0].year(), 2020);
    assert_eq!(dates[1].year(), 2021);
    assert_eq!(dates[2].year(), 2022);

    // All dates should be January 1st
    for date in &dates {
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 1);
    }

    Ok(())
}

#[test]
fn test_span_covering() -> Result<()> {
    let d1 = ymd(2020, 1, 1, Eastern);
    let d2 = ymd(2020, 1, 5, Eastern);
    let d3 = ymd(2020, 1, 10, Eastern);
    let d4 = ymd(2020, 1, 15, Eastern);

    let span1 = SpanExc::new(d1, d2);
    let span2 = SpanExc::new(d3, d4);

    // Cover should create a span that contains both
    let cover = SpanExc::cover(&span1, &span2);
    assert_eq!(cover.st, d1);
    assert_eq!(cover.en, d4);

    // Cover with empty span should return the non-empty span
    let empty = SpanExc::new(d1, d1);
    let cover2 = SpanExc::cover(&span1, &empty);
    assert_eq!(cover2, span1);

    Ok(())
}

#[test]
fn test_duration_arithmetic_chain() -> Result<()> {
    // Test chaining multiple duration operations
    let base = Duration::HOUR;

    let result = base + Duration::MIN * 30 + Duration::SEC * 45;

    // 1 hour + 30 minutes + 45 seconds = 3600 + 1800 + 45 = 5445 seconds
    assert_eq!(result.secs(), dec!(5445));

    // Test the human-readable output
    let human = result.human()?;
    assert_eq!(human, "1h30m45s");

    Ok(())
}

#[test]
fn test_date_month_operations_with_iteration() -> Result<()> {
    // Test monthly iteration and operations
    let start = ymd(2020, 1, 15, Eastern);

    let mut dates = Vec::new();
    let mut current = start;
    for _ in 0..12 {
        dates.push(current);
        current = current.add_months(1);
    }

    assert_eq!(dates.len(), 12);

    // All should be on the 15th
    for date in &dates {
        assert_eq!(date.day(), 15);
    }

    // Should cover all 12 months of 2020
    for (i, date) in dates.iter().enumerate() {
        assert_eq!(date.month(), (i as u32 + 1));
        assert_eq!(date.year(), 2020);
    }

    Ok(())
}

#[test]
fn test_time_operations_across_midnight() -> Result<()> {
    // Test time iteration that crosses midnight
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

    Ok(())
}
