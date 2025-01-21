use chrono;
use chrono::{DateTime, Utc};
use std::cmp::{max, min};
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

    pub fn duration(&self) -> chrono::Duration {
        self.end.signed_duration_since(self.start)
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

    pub fn find_next_fitting_time(
        &self,
        current_time: DateTime<Utc>,
        job_duration: chrono::Duration,
        must_fit: bool,
    ) -> Option<TimeWindow> {
        // find the earliest time window that
        // 1. is within time_windows
        // and
        // 2. if must_fit is true fits the complete job_duration
        //    or if false, fits as much as possible
        // return None if no such time window exists

        // if no windows, return None
        if self.windows.is_empty() {
            return None;
        }
        // binary search: find the index of the first window that starts after current_time or contains it
        // comparing with the end of the windows,
        let index = match self
            .windows
            .binary_search_by(|window| window.end.cmp(&current_time))
        {
            // technically, the end of a time window is not part of the time window,
            // hence, when Ok(index) is returned, the next time window is the one we are looking for (see doc binary_search_by)
            Ok(index) => index + 1,
            // if Err is returned, index would be the place where we would insert current time to maintain the order
            // Hence, time_windows[index - 1].end < current_time < time_windows[index].end
            // so we return index
            Err(index) => index,
        };
        // if index is the length of the windows, there is no fitting time window anymore
        if index == self.windows.len() {
            return None;
        }
        match must_fit {
            false => {
                // we can use the first time window, just check whether window's duration is larger or job's duration
                let start = max(self.windows[index].start, current_time);
                let end = start + min(self.windows[index].duration(), job_duration);
                return Some(TimeWindow::new(start, end));
            }
            true => {
                // Here, we have to iterate over the remaining time windows until we find on that fits the job duration
                return self
                    .windows
                    .iter()
                    .skip(index)
                    .find(|window| window.duration() >= job_duration) // TODO is there a better way to do this? Maybe, there is a leetcode problem for this
                    .map(|window| {
                        TimeWindow::new(
                            max(window.start, current_time),
                            max(window.start, current_time) + job_duration,
                        )
                    });
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

    // test basic functionality of TimeWindow
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
        let end2 = Utc.with_ymd_and_hms(2021, 1, 1, 5, 0, 0).unwrap();
        let time_window2 = TimeWindow::new(start2, end2);
        time_windows.add_window(time_window1);
        time_windows.add_window(time_window2);
        assert_eq!(time_windows.len(), 2);

        let result = time_windows.find_next_fitting_time(
            Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            chrono::Duration::hours(1),
            true,
        );
        assert_eq!(
            result,
            Some(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 1, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap()
            ))
        );
        let result = time_windows.find_next_fitting_time(
            Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            chrono::Duration::hours(2),
            true,
        );
        assert_eq!(
            result,
            Some(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 3, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 5, 0, 0).unwrap()
            ))
        );
        let result = time_windows.find_next_fitting_time(
            Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            chrono::Duration::hours(3),
            true,
        );
        assert_eq!(result, None);

        let result = time_windows.find_next_fitting_time(
            Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            chrono::Duration::hours(2),
            false,
        );
        assert_eq!(
            result,
            Some(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 1, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap()
            ))
        );
        let result = time_windows.find_next_fitting_time(
            Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap(),
            chrono::Duration::hours(3),
            false,
        );
        assert_eq!(
            result,
            Some(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 3, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 5, 0, 0).unwrap()
            ))
        );
        let result = time_windows.find_next_fitting_time(
            Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap(),
            chrono::Duration::hours(3),
            false,
        );
        assert_eq!(result, None);
    }
}
