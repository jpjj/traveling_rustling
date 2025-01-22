use super::{
    operation_times::OperationTimes,
    time_windows::{TimeWindow, TimeWindows},
};
/// input for time window constraints

pub struct TimeInput {
    pub duration_matrix: Vec<Vec<chrono::Duration>>,
    pub job_durations: Vec<chrono::Duration>,
    pub time_windows: Vec<TimeWindows>,
    pub operation_times: Option<OperationTimes>,
    pub travel_duration_until_break: Option<u64>,
    pub break_duration: Option<u64>,
}

impl TimeInput {
    // pub fn new(duration_matrix: Vec<Vec<chrono::Duration>>, job_durations: Vec<chrono::Duration>, time_windows: Vec<TimeWindows>, operation_times: OperationTimes) -> TimeInput{
    //     TimeInput{
    //         duration_matrix,
    //         job_durations,
    //         time_windows,
    //         operation_times
    //     }
    // }

    pub fn travel_time(&self, from: usize, to: usize) -> chrono::Duration {
        self.duration_matrix[from][to]
    }
}

pub fn transform(
    duration_matrix: Option<Vec<Vec<u64>>>,
    job_durations: Option<Vec<u64>>,
    time_windows: Option<Vec<Vec<(u64, u64)>>>,
    operation_times: Option<(u64, u64)>,
    working_days: Option<Vec<bool>>,
    travel_duration_until_break: Option<u64>,
    break_duration: Option<u64>,
) -> Option<TimeInput> {
    let duration_matrix = match duration_matrix {
        Some(matrix) => Some(
            matrix
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|&x| chrono::Duration::seconds(x as i64))
                        .collect::<Vec<chrono::Duration>>()
                })
                .collect::<Vec<Vec<chrono::Duration>>>(),
        ),
        None => None,
    };
    let job_durations = match job_durations {
        Some(durations) => Some(
            durations
                .iter()
                .map(|&x| chrono::Duration::seconds(x as i64))
                .collect::<Vec<chrono::Duration>>(),
        ),
        None => None,
    };
    let time_windows = match time_windows {
        Some(windows) => Some(
            windows
                .iter()
                .map(|window| {
                    TimeWindows::new(
                        window
                            .iter()
                            .map(|&(start, end)| {
                                TimeWindow::new(
                                    chrono::DateTime::from_timestamp(start as i64, 0).unwrap(),
                                    chrono::DateTime::from_timestamp(end as i64, 0).unwrap(),
                                )
                            })
                            .collect::<Vec<TimeWindow>>(),
                    )
                })
                .collect::<Vec<TimeWindows>>(),
        ),
        None => None,
    };
    let operation_times = match operation_times {
        Some((start, end)) => {
            // if they are 24 hours, we can ignore operating times
            if end - start == 24 * 3600 || start == end {
                None
            } else {
                Some(OperationTimes::new(
                    chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                        + chrono::Duration::seconds(start as i64),
                    chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                        + chrono::Duration::seconds(end as i64),
                    match working_days {
                        None => None,
                        Some(days) => Some(
                            days.iter()
                                .enumerate()
                                .filter_map(|(i, &x)| if x { Some(i) } else { None })
                                .map(|x| match x {
                                    0 => chrono::Weekday::Mon,
                                    1 => chrono::Weekday::Tue,
                                    2 => chrono::Weekday::Wed,
                                    3 => chrono::Weekday::Thu,
                                    4 => chrono::Weekday::Fri,
                                    5 => chrono::Weekday::Sat,
                                    6 => chrono::Weekday::Sun,
                                    _ => panic!("Invalid day"),
                                })
                                .collect::<Vec<chrono::Weekday>>(),
                        ),
                    },
                ))
            }
        }
        None => None,
    };
    // Here we could do even more matches like if duration matrix is None, we
    // will not calculate any travel time, in the calculation, same for job durations.
    match (
        duration_matrix,
        job_durations,
        time_windows,
        operation_times,
    ) {
        (Some(duration_matrix), Some(job_durations), Some(time_windows), operation_times) => {
            Some(TimeInput {
                duration_matrix,
                job_durations,
                time_windows,
                operation_times,
                travel_duration_until_break,
                break_duration,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn test_transform() {
        let time_input = transform(
            Some(vec![vec![0, 1, 2], vec![1, 0, 3], vec![2, 3, 0]]),
            Some(vec![3, 3, 3]),
            Some(vec![
                vec![(1, 2), (3, 4)],
                vec![(5, 6), (7, 8)],
                vec![(9, 10), (11, 12)],
            ]),
            Some((8, 16)),
            Some(vec![true, true, true, false, false, true, false]),
            None,
            None,
        );
        assert!(time_input.is_some());
        let time_input = time_input.unwrap();
        assert_eq!(time_input.travel_time(0, 1), chrono::Duration::seconds(1));
        assert_eq!(time_input.travel_time(1, 2), chrono::Duration::seconds(3));
        assert_eq!(time_input.travel_time(2, 0), chrono::Duration::seconds(2));
        assert_eq!(
            time_input.job_durations,
            vec![chrono::Duration::seconds(3); 3]
        );
        assert_eq!(time_input.time_windows.len(), 3);
        assert_eq!(time_input.time_windows[0].windows.len(), 2);
        assert_eq!(time_input.time_windows[1].windows.len(), 2);
        assert_eq!(time_input.time_windows[2].windows.len(), 2);
        assert_eq!(
            time_input.time_windows[0].windows[0].start,
            chrono::DateTime::from_timestamp(1, 0).unwrap()
        );
        assert_eq!(
            time_input.time_windows[0].windows[0].end,
            chrono::DateTime::from_timestamp(2, 0).unwrap()
        );
        assert_eq!(
            time_input.time_windows[0].windows[1].start,
            chrono::DateTime::from_timestamp(3, 0).unwrap()
        );
        assert_eq!(
            time_input.time_windows[0].windows[1].end,
            chrono::DateTime::from_timestamp(4, 0).unwrap()
        );
        assert_eq!(
            time_input.time_windows[1].windows[0].start,
            chrono::DateTime::from_timestamp(5, 0).unwrap()
        );
        assert_eq!(
            time_input.time_windows[1].windows[0].end,
            chrono::DateTime::from_timestamp(6, 0).unwrap()
        );
        assert_eq!(
            time_input.time_windows[1].windows[1].start,
            chrono::DateTime::from_timestamp(7, 0).unwrap()
        );
        assert_eq!(
            time_input.time_windows[1].windows[1].end,
            chrono::DateTime::from_timestamp(8, 0).unwrap()
        );
        assert_eq!(
            time_input.time_windows[2].windows[0].start,
            chrono::DateTime::from_timestamp(9, 0).unwrap()
        );
        assert_eq!(
            time_input.time_windows[2].windows[0].end,
            chrono::DateTime::from_timestamp(10, 0).unwrap()
        );
        assert_eq!(
            time_input.time_windows[2].windows[1].start,
            chrono::DateTime::from_timestamp(11, 0).unwrap()
        );
        assert_eq!(
            time_input.time_windows[2].windows[1].end,
            chrono::DateTime::from_timestamp(12, 0).unwrap()
        );
        let operation_times = time_input.operation_times.unwrap();
        assert_eq!(
            operation_times.start(),
            chrono::NaiveTime::from_hms_opt(0, 0, 8).unwrap()
        );
        assert_eq!(
            operation_times.end(),
            chrono::NaiveTime::from_hms_opt(0, 0, 16).unwrap()
        );
        // Working days are Mon, Tue, Wed and Sat
        // Mon -> Tue
        assert_eq!(
            operation_times.next_day(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()
        );
        // Tue -> Wed
        assert_eq!(
            operation_times.next_day(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap()),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 3).unwrap()
        );
        // Wed -> Sat
        assert_eq!(
            operation_times.next_day(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap()),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 6).unwrap()
        );
        // Thu -> Sat
        assert_eq!(
            operation_times.next_day(Utc.with_ymd_and_hms(2024, 1, 4, 0, 0, 0).unwrap()),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 6).unwrap()
        );
        // Fri -> Sat
        assert_eq!(
            operation_times.next_day(Utc.with_ymd_and_hms(2024, 1, 5, 0, 0, 0).unwrap()),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 6).unwrap()
        );
        // Sat -> Mon
        assert_eq!(
            operation_times.next_day(Utc.with_ymd_and_hms(2024, 1, 6, 0, 0, 0).unwrap()),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 8).unwrap()
        );
        assert_eq!(
            operation_times.next_day(Utc.with_ymd_and_hms(2024, 1, 7, 0, 0, 0).unwrap()),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 8).unwrap()
        );
    }
}
