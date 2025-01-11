use chrono;
/// A time window with a start and end time.
struct TimeWindow {
    start: chrono::NaiveDateTime,
    end: chrono::NaiveDateTime,
}

impl TimeWindow {
    /// Creates a new time window with the given start and end time.
    /// Panics if the start time is after the end time.
    fn new(start: chrono::NaiveDateTime, end: chrono::NaiveDateTime) -> TimeWindow {
        assert!(start <= end);
        TimeWindow {
            start,
            end,
        }
    }
}

/// A collection of time windows.
/// Time windows are stored in chronological order and do not overlap.
struct TimeWindows {
    windows: Vec<TimeWindow>,
}

impl TimeWindows {
    /// Creates a new empty collection of time windows.
    fn new() -> TimeWindows {
        TimeWindows {
            windows: Vec::new(),
        }
    }

    /// Adds a new time window to the collection.
    fn add_window(&mut self, start: chrono::NaiveDateTime, end: chrono::NaiveDateTime) {
        // Always make sure that time windows exist in chronological order
        // Assume that time windows do not overlap and we have
        // self.windows[i].end < self.windows[i + 1].start
        // now, we add a new window while maintaining these two conditions
        // binary search
        self.windows.push(TimeWindow { start, end });
    }

    fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    fn len(&self) -> usize {
        self.windows.len()
    }

    fn find_next_window(&self, time: chrono::NaiveDateTime) -> Option<&TimeWindow> {
        // Find the time window that contains the given time
        // If it does not exist, find the time window starting after the given time
        self.windows.iter().find(|window| window.start <= time && time <= window.end)
    }
}