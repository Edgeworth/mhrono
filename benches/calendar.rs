use chrono::TimeZone;
use chrono_tz::US::Eastern;
use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};
use mhrono::calendars::nyse::get_nyse;
use mhrono::date::Date;
use mhrono::iter::DateIter;
use mhrono::time::Time;

fn calendar(c: &mut Criterion) {
    let mut g = c.benchmark_group("calendar");
    for date in [Eastern.ymd(1800, 1, 1), Eastern.ymd(1900, 1, 1), Eastern.ymd(2000, 1, 1)].iter() {
        g.throughput(Throughput::Elements(1));
        let t: Time = date.into();
        g.bench_with_input(BenchmarkId::new("oneshot", t), &t, |b, &t| {
            b.iter_batched(get_nyse, |mut nyse| nyse.next_span(t), BatchSize::SmallInput)
        });
    }

    let mut nyse = get_nyse();
    const NUM_DAYS: i32 = 10000;
    let start: Date = Eastern.ymd(1990, 1, 1).into();
    let iter = DateIter::day(start, start.add_days(NUM_DAYS));
    g.throughput(Throughput::Elements(NUM_DAYS as u64));
    g.bench_with_input(BenchmarkId::new("range", iter.clone()), &iter, |b, iter| {
        b.iter(|| {
            for t in iter.clone() {
                black_box(nyse.next_span(t.into()).unwrap());
            }
        })
    });
    g.finish();
}

criterion_group!(benches, calendar);
criterion_main!(benches);
