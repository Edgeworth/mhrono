use std::io::Write;

use chrono_tz::US::Eastern;
use chrono_tz::UTC;
use eyre::Result;
use mhrono::calendars::nyse::get_nyse;
use mhrono::date::ymd;
use mhrono::iter::DateIter;
use moldenfile::Golden;

#[test]
fn nyse_spans() -> Result<()> {
    const START_YEAR: i32 = 1792;
    const END_YEAR: i32 = 2050;
    let mut g = Golden::new("tests/golden")?;
    let mut f = g.file("nyse_spans.gz")?;
    let mut t = ymd(START_YEAR, 1, 1, Eastern).time()?;
    let mut nyse = get_nyse();
    loop {
        let s = nyse.next_span(&t).unwrap();
        if s.st >= ymd(END_YEAR, 1, 1, Eastern).time()? {
            break;
        }
        writeln!(f, "{},{}", s.st.with_tz(UTC).to_iso(), s.en.with_tz(UTC).to_iso())?;
        t = s.en;
    }
    Ok(())
}

#[test]
fn nyse_holidays() -> Result<()> {
    const START_YEAR: i32 = 1792;
    const END_YEAR: i32 = 2050;
    let mut g = Golden::new("tests/golden")?;
    let mut f = g.file("nyse_holidays.gz")?;
    let mut nyse = get_nyse();

    for d in DateIter::day(ymd(START_YEAR, 1, 1, Eastern), ymd(END_YEAR, 1, 1, Eastern)) {
        let t = d.time()?;
        let s = nyse.next_span(&t).unwrap();
        if t.ymd()? != s.st.ymd()? {
            writeln!(f, "{}", &t.to_iso()[..10])?; // Holiday.
        }
    }
    Ok(())
}
