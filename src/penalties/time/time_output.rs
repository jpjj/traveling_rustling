use chrono::{DateTime, Utc};

use super::{
    operation_times::OperationTimes,
    time_windows::{TimeWindow, TimeWindows},
};

/// Time report module for outputs and evaluation of the time schedule.

#[derive(Debug, Clone)]
pub struct TimeReport {
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
    duration: chrono::Duration,
    lateness: chrono::Duration,
    waiting_time: chrono::Duration,
    schedule: Vec<Event>,
}

impl TimeReport {
    pub fn new(
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
        duration: chrono::Duration,
        lateness: chrono::Duration,
        waiting_time: chrono::Duration,
        schedule: Vec<Event>,
    ) -> TimeReport {
        TimeReport {
            start_time,
            end_time,
            duration,
            lateness,
            waiting_time,
            schedule,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Work(TimeWindow, usize),
    Travel(TimeWindow),
    Wait(TimeWindow),
}

pub fn action_report(
    job_duration: chrono::Duration,
    job_time_windows: Option<&TimeWindows>,
    operation_times: Option<&OperationTimes>,
    start_time: DateTime<Utc>,
    is_travel: bool,
    location_id: Option<usize>,
) -> TimeReport {
    let mut schedule: Vec<Event> = Vec::new();
    let job_duration_left = job_duration;
    let mut current_time = start_time;
    let mut total_waiting_time = chrono::Duration::zero();
    let mut total_lateness = chrono::Duration::zero();
    let mut total_travel_time = chrono::Duration::zero();
    let mut tentative_waiting_time = chrono::Duration::zero();
    while job_duration_left > chrono::Duration::zero() {
        let maybe_next_time_window =
            job_time_windows.and_then(|time_windows| time_windows.next_window(current_time));
        let mut ready_to_work = false;
        match maybe_next_time_window {
            Some((next_time_window, waiting_time)) => {
                if waiting_time > chrono::Duration::zero() {
                    tentative_waiting_time += waiting_time;
                    current_time += waiting_time;
                    ready_to_work = true;
                }
                // job does not fit in time windows
                if next_time_window.end - current_time < job_duration_left {
                    ready_to_work = false;
                    let more_waiting_time =
                        next_time_window.end + chrono::Duration::seconds(1) - current_time;
                    tentative_waiting_time += more_waiting_time;
                    current_time += more_waiting_time;
                }
            }
            None => {
                // no more time windows
                ready_to_work = true;
            }
        }
        // we reached a time where either all time windows are over or we are inside one.
        if !ready_to_work {
            continue;
        }
        let waiting_time = operation_times
            .map(|ot| ot.waiting_time(current_time.time()))
            .or_else(|| Some(chrono::Duration::zero()))
            .unwrap();
        if waiting_time > chrono::Duration::zero() {
            tentative_waiting_time += waiting_time;
            current_time += waiting_time;
        } else {
            // we can finally work. If there is tentative waiting time, add event
            if tentative_waiting_time > chrono::Duration::zero() {
                schedule.push(Event::Wait(TimeWindow::new(
                    current_time - tentative_waiting_time,
                    current_time,
                )));
                total_waiting_time += tentative_waiting_time;
                tentative_waiting_time = chrono::Duration::zero();
            }
            // check how much time we can work
            // given the checks before, we should be able to do the whole job in the time window.
            let work_time =
                job_duration_left.min(operation_times.unwrap().end() - current_time.time());
            if is_travel {
                schedule.push(Event::Travel(TimeWindow::new(
                    current_time,
                    current_time + work_time,
                )));
                total_travel_time += work_time;
            } else {
                schedule.push(Event::Work(
                    TimeWindow::new(current_time, current_time + work_time),
                    location_id.unwrap(),
                ));
                if job_time_windows.is_some() {
                    total_lateness += job_time_windows.unwrap().lateness(current_time + work_time);
                }
            }
            current_time += work_time;
        }
    }
    TimeReport::new(
        start_time,
        current_time,
        current_time - start_time,
        total_lateness,
        total_waiting_time,
        schedule,
    )
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use chrono::TimeZone;

//     #[test]
//     fn test_action_report() {
//         let start = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);
//         let end = Utc.ymd(2021, 1, 1).and_hms(1, 0, 0);
//         let time_windows = TimeWindows::new();
//         let operation_times = OperationTimes::new(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(), chrono::NaiveTime::from_hms_opt(1, 0, 0).unwrap());
//         let report = action_report(chrono::Duration::minutes(30), Some(&time_windows), Some(&operation_times), start, false, Some(0));
//         assert_eq!(report.schedule.len(), 1);
//         assert_eq!(report.schedule[0], Event::Work(TimeWindow::new(start, start + chrono::Duration::minutes(30)), 0));
//         assert_eq!(report.lateness, chrono::Duration::zero());
//         assert_eq!(report.waiting_time, chrono::Duration::zero());
//         assert_eq!(report.duration, chrono::Duration::minutes(30));
//         assert_eq!(report.start_time, start);
//         assert_eq!(report.end_time, start + chrono::Duration::minutes(30));
//     }

//     #[test]
//     fn test_action_report_with_time_windows() {
//         let start = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);
//         let end = Utc.ymd(2021, 1, 1).and_hms(1, 0, 0);
//         let time_windows = TimeWindows::new();
//         time_windows.push(TimeWindow::new(Utc.ymd(2021, 1, 1).and_hms(0, 0, 0), Utc.ymd(2021, 1, 1).and_hms(0, 30, 0)));
//         let operation_times = operation_times::OperationTimes::new(Utc.ymd(2021, 1, 1).and_hms(0, 0, 0),
//         Utc.ymd(2021, 1, 1).and_hms(1, 0, 0));
//         let report = action_report(chrono::Duration::minutes(30), Some(&time_windows), Some(&operation_times), start, false, Some(0));
//         assert_eq!(report.schedule.len(), 2);
//         assert_eq!(report.schedule[0], Event::Wait(TimeWindow::new(start, start + chrono::Duration::minutes(30))));
//         assert_eq!(report.schedule[1], Event::Work(TimeWindow::new(start + chrono::Duration::minutes(30), start + chrono::Duration::minutes(30) + chrono::Duration::zero()), 0));
//         assert_eq!(report.lateness, chrono::Duration::zero());
//         assert_eq!(report.waiting_time, chrono::Duration::minutes(30));
//         assert_eq!(report.duration, chrono::Duration::minutes(30));
//         assert_eq!(report.start_time, start);
//         assert_eq!(report.end_time, start + chrono::Duration::minutes(30));
//     }
// }
