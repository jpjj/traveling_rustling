use chrono;
use chrono::{DateTime, Utc};
use std::ops::Index;
/// A time window with a start and end time.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TimeWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeWindow {
    /// Creates a new time window with the given start and end time.
    /// Panics if the start time is after the end time.
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> TimeWindow {
        assert!(start <= end);
        TimeWindow { start, end }
    }

    fn contains(&self, time: DateTime<Utc>) -> bool {
        self.start <= time && time <= self.end
    }

    pub fn waiting_time(&self, time: DateTime<Utc>) -> chrono::Duration {
        if time < self.start {
            self.start.signed_duration_since(time)
        } else if time > self.end {
            chrono::Duration::zero()
        } else {
            chrono::Duration::zero()
        }
    }

    fn lateness(&self, time: DateTime<Utc>) -> chrono::Duration {
        if time > self.end {
            time.signed_duration_since(self.end)
        } else {
            chrono::Duration::zero()
        }
    }
}

/// A collection of time windows.
/// Time windows are stored in chronological order and do not overlap.
pub struct TimeWindows {
    pub windows: Vec<TimeWindow>,
}

impl TimeWindows {
    /// Creates a new empty collection of time windows.
    pub fn new(windows: Vec<TimeWindow>) -> TimeWindows {
        TimeWindows { windows: windows }
    }

    /// Adds a new time window to the collection.
    pub fn add_window(&mut self, time_window: TimeWindow) {
        // Always make sure that time windows exist in chronological order
        // Assume that time windows do not overlap and we have
        // self.windows[i].end < self.windows[i + 1].start
        assert!(self.windows.is_empty() || self.windows.last().unwrap().end <= time_window.start);
        self.windows.push(time_window);
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.windows.len()
    }

    pub fn next_window(&self, time: DateTime<Utc>) -> Option<(&TimeWindow, chrono::Duration)> {
        // Find the time window that contains the given time and the waiting time
        // If it does not exist, find the time window starting after the given time
        // returns None if time is after the last time window
        // binary search for time on the slice of start times
        if self.windows.is_empty() {
            return None;
        }
        if self.windows.last().unwrap().lateness(time) > chrono::Duration::zero() {
            return None;
        }
        // hence, there must be a time window that contains the time
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
                let waiting_time = self.windows[index].waiting_time(time);
                Some((&self.windows[index], waiting_time))
            }
        }
    }

    pub fn lateness(&self, time: DateTime<Utc>) -> chrono::Duration {
        if self.windows.is_empty() {
            return chrono::Duration::zero();
        }
        self.windows.last().unwrap().lateness(time)
    }
}

impl Index<usize> for TimeWindows {
    type Output = TimeWindow;

    fn index(&self, index: usize) -> &Self::Output {
        &self.windows[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_time_window() {
        let start = Utc.with_ymd_and_hms(2021, 1, 1, 1, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap();
        let time_window = TimeWindow::new(start, end);
        assert!(!time_window.contains(Utc.with_ymd_and_hms(2021, 1, 1, 0, 30, 0).unwrap()));
        assert!(time_window.contains(Utc.with_ymd_and_hms(2021, 1, 1, 1, 0, 0).unwrap()));
        assert!(time_window.contains(Utc.with_ymd_and_hms(2021, 1, 1, 1, 30, 0).unwrap()));
        assert!(time_window.contains(Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap()));
        assert!(!time_window.contains(Utc.with_ymd_and_hms(2021, 1, 1, 2, 30, 0).unwrap()));
        assert_eq!(
            time_window.waiting_time(Utc.with_ymd_and_hms(2021, 1, 1, 0, 30, 0).unwrap()),
            chrono::Duration::minutes(30)
        );
        assert_eq!(
            time_window.waiting_time(Utc.with_ymd_and_hms(2021, 1, 1, 1, 0, 0).unwrap()),
            chrono::Duration::zero()
        );
        assert_eq!(
            time_window.waiting_time(Utc.with_ymd_and_hms(2021, 1, 1, 1, 30, 0).unwrap()),
            chrono::Duration::zero()
        );
        assert_eq!(
            time_window.waiting_time(Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap()),
            chrono::Duration::zero()
        );
        assert_eq!(
            time_window.waiting_time(Utc.with_ymd_and_hms(2021, 1, 1, 2, 30, 0).unwrap()),
            chrono::Duration::zero()
        );
        assert_eq!(
            time_window.lateness(Utc.with_ymd_and_hms(2021, 1, 1, 0, 30, 0).unwrap()),
            chrono::Duration::zero()
        );
        assert_eq!(
            time_window.lateness(Utc.with_ymd_and_hms(2021, 1, 1, 1, 00, 0).unwrap()),
            chrono::Duration::zero()
        );
        assert_eq!(
            time_window.lateness(Utc.with_ymd_and_hms(2021, 1, 1, 1, 30, 0).unwrap()),
            chrono::Duration::zero()
        );
        assert_eq!(
            time_window.lateness(Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap()),
            chrono::Duration::zero()
        );
        assert_eq!(
            time_window.lateness(Utc.with_ymd_and_hms(2021, 1, 1, 2, 30, 0).unwrap()),
            chrono::Duration::minutes(30)
        );
    }

    #[test]
    fn test_time_windows() {
        let mut time_windows = TimeWindows::new(vec![]);
        let start1 = Utc.with_ymd_and_hms(2021, 1, 1, 1, 0, 0).unwrap();
        let end1 = Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap();
        let time_window1 = TimeWindow::new(start1, end1);
        let start2 = Utc.with_ymd_and_hms(2021, 1, 1, 3, 0, 0).unwrap();
        let end2 = Utc.with_ymd_and_hms(2021, 1, 1, 4, 0, 0).unwrap();
        let time_window2 = TimeWindow::new(start2, end2);
        time_windows.add_window(time_window1.clone());
        time_windows.add_window(time_window2.clone());
        assert_eq!(time_windows.len(), 2);
        assert_eq!(
            time_windows
                .next_window(Utc.with_ymd_and_hms(2021, 1, 1, 0, 30, 0).unwrap())
                .unwrap(),
            (&time_window1, chrono::Duration::minutes(30))
        );
        assert_eq!(
            time_windows
                .next_window(Utc.with_ymd_and_hms(2021, 1, 1, 1, 0, 0).unwrap())
                .unwrap(),
            (&time_window1, chrono::Duration::zero())
        );
        assert_eq!(
            time_windows
                .next_window(Utc.with_ymd_and_hms(2021, 1, 1, 1, 30, 0).unwrap())
                .unwrap(),
            (&time_window1, chrono::Duration::zero())
        );
        assert_eq!(
            time_windows
                .next_window(Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap())
                .unwrap(),
            (&time_window1, chrono::Duration::zero())
        );
        assert_eq!(
            time_windows
                .next_window(Utc.with_ymd_and_hms(2021, 1, 1, 2, 30, 0).unwrap())
                .unwrap(),
            (&time_window2, chrono::Duration::minutes(30))
        );
        assert_eq!(
            time_windows
                .next_window(Utc.with_ymd_and_hms(2021, 1, 1, 3, 0, 0).unwrap())
                .unwrap(),
            (&time_window2, chrono::Duration::zero())
        );
        assert_eq!(
            time_windows
                .next_window(Utc.with_ymd_and_hms(2021, 1, 1, 3, 30, 0).unwrap())
                .unwrap(),
            (&time_window2, chrono::Duration::zero())
        );
        assert_eq!(
            time_windows
                .next_window(Utc.with_ymd_and_hms(2021, 1, 1, 4, 0, 0).unwrap())
                .unwrap(),
            (&time_window2, chrono::Duration::zero())
        );
        assert_eq!(
            time_windows.next_window(Utc.with_ymd_and_hms(2021, 1, 1, 4, 30, 0).unwrap()),
            None
        );
    }
}
