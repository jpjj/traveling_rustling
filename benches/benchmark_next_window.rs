use chrono::{DateTime, Utc};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{SystemTime, UNIX_EPOCH};

struct TimeWindow {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl TimeWindow {
    fn lateness(&self, time: DateTime<Utc>) -> chrono::Duration {
        if time > self.end {
            time - self.end
        } else {
            chrono::Duration::zero()
        }
    }

    fn contains(&self, time: DateTime<Utc>) -> bool {
        self.start <= time && time <= self.end
    }

    fn waiting_time(&self, time: DateTime<Utc>) -> chrono::Duration {
        if time < self.start {
            self.start - time
        } else {
            chrono::Duration::zero()
        }
    }
}

struct TimeWindowU64 {
    start: u64,
    end: u64,
}

impl TimeWindowU64 {
    fn lateness(&self, time: u64) -> u64 {
        if time > self.end {
            time - self.end
        } else {
            0
        }
    }

    fn contains(&self, time: u64) -> bool {
        self.start <= time && time <= self.end
    }

    fn waiting_time(&self, time: u64) -> u64 {
        if time < self.start {
            self.start - time
        } else {
            0
        }
    }
}

struct TimeWindows {
    windows: Vec<TimeWindow>,
}

impl TimeWindows {
    pub fn next_window(&self, time: DateTime<Utc>) -> Option<(&TimeWindow, chrono::Duration)> {
        if self.windows.is_empty() {
            return None;
        }
        if self.windows.last().unwrap().lateness(time) > chrono::Duration::zero() {
            return None;
        }
        match self
            .windows
            .binary_search_by(|window| window.start.cmp(&time))
        {
            Ok(index) => Some((&self.windows[index], chrono::Duration::zero())),
            Err(index) => {
                if index == 0 {
                    return Some((&self.windows[0], self.windows[0].waiting_time(time)));
                }
                if self.windows[index - 1].contains(time) {
                    return Some((&self.windows[index - 1], chrono::Duration::zero()));
                }
                Some((&self.windows[index], self.windows[index].waiting_time(time)))
            }
        }
    }
}

struct TimeWindowsU64 {
    windows: Vec<TimeWindowU64>,
}

impl TimeWindowsU64 {
    pub fn next_window(&self, time: u64) -> Option<(&TimeWindowU64, u64)> {
        if self.windows.is_empty() {
            return None;
        }
        if self.windows.last().unwrap().lateness(time) > 0 {
            return None;
        }
        match self
            .windows
            .binary_search_by(|window| window.start.cmp(&time))
        {
            Ok(index) => Some((&self.windows[index], 0)),
            Err(index) => {
                if index == 0 {
                    return Some((&self.windows[0], self.windows[0].waiting_time(time)));
                }
                if self.windows[index - 1].contains(time) {
                    return Some((&self.windows[index - 1], 0));
                }
                Some((&self.windows[index], self.windows[index].waiting_time(time)))
            }
        }
    }
}

fn benchmark_next_window_datetime(c: &mut Criterion) {
    let start = Utc::now();
    let end = start + chrono::Duration::seconds(3600);
    let windows = TimeWindows {
        windows: vec![TimeWindow { start, end }],
    };
    let time = start + chrono::Duration::seconds(1800);

    c.bench_function("next_window datetime", |b| {
        b.iter(|| {
            black_box(windows.next_window(time));
        })
    });
}

fn benchmark_next_window_u64(c: &mut Criterion) {
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let end = start + 3600;
    let windows = TimeWindowsU64 {
        windows: vec![TimeWindowU64 { start, end }],
    };
    let time = start + 1800;

    c.bench_function("next_window u64", |b| {
        b.iter(|| {
            black_box(windows.next_window(time));
        })
    });
}

criterion_group!(
    benches,
    benchmark_next_window_datetime,
    benchmark_next_window_u64
);
criterion_main!(benches);
