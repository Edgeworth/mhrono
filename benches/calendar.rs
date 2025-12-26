use std::hint::black_box;

use chrono_tz::US::Eastern;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use mhrono::calendars::nyse::get_nyse;
use mhrono::date::ymd;
use mhrono::iter::DateIter;

fn calendar(c: &mut Criterion) {
    let mut g = c.benchmark_group("calendar");
    for date in
        [ymd(1800, 1, 1, Eastern), ymd(1900, 1, 1, Eastern), ymd(2000, 1, 1, Eastern)].iter()
    {
        g.throughput(Throughput::Elements(1));
        let start = date.time().unwrap();
        g.bench_with_input(BenchmarkId::new("oneshot", start), &start, |b, t| {
            b.iter_batched(get_nyse, |mut nyse| nyse.next_span(t), BatchSize::SmallInput)
        });
    }

    let mut nyse = get_nyse();
    const NUM_DAYS: i32 = 10000;
    let start = ymd(1990, 1, 1, Eastern);
    let iter = DateIter::day(start, start.add_days(NUM_DAYS));
    g.throughput(Throughput::Elements(NUM_DAYS as u64));
    g.bench_with_input(BenchmarkId::new("range", iter.clone()), &iter, |b, _| {
        b.iter(|| {
            for t in iter.clone() {
                let _ = black_box(nyse.next_span(&t.time().unwrap()).unwrap());
            }
        })
    });
    g.finish();
}

criterion_group!(benches, calendar);
criterion_main!(benches);
