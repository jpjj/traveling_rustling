use chrono::{DateTime, Utc};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct TimeWindow {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl TimeWindow {
    fn contains(&self, time: DateTime<Utc>) -> bool {
        self.start <= time && time <= self.end
    }
}

struct TimeWindowU64 {
    start: u64,
    end: u64,
}

impl TimeWindowU64 {
    fn contains(&self, time: u64) -> bool {
        self.start <= time && time <= self.end
    }
}

fn benchmark_datetime(c: &mut Criterion) {
    let start = Utc::now();
    let end = start + chrono::Duration::seconds(3600);
    let time_window = TimeWindow { start, end };
    let time = start + chrono::Duration::seconds(1800);

    c.bench_function("datetime contains", |b| {
        b.iter(|| {
            black_box(time_window.contains(time));
        })
    });
}

fn benchmark_u64(c: &mut Criterion) {
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let end = start + 3600;
    let time_window = TimeWindowU64 { start, end };
    let time = start + 1800;

    c.bench_function("u64 contains", |b| {
        b.iter(|| {
            black_box(time_window.contains(time));
        })
    });
}

criterion_group!(benches, benchmark_datetime, benchmark_u64);
criterion_main!(benches);
