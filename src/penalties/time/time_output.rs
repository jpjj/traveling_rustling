use chrono::{DateTime, Utc};

use super::{
    operation_times::OperationTimes,
    time_windows::{TimeWindow, TimeWindows},
};

/// Time report module for outputs and evaluation of the time schedule.

#[derive(Debug, Clone)]
pub struct TimeReport {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub duration: chrono::Duration,
    pub lateness: chrono::Duration,
    pub working_time: chrono::Duration,
    pub waiting_time: chrono::Duration,
    pub traveling_time: chrono::Duration,
    pub schedule: Vec<Event>,
}

impl TimeReport {
    pub fn new(
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
        duration: chrono::Duration,
        lateness: chrono::Duration,
        working_time: chrono::Duration,
        waiting_time: chrono::Duration,
        traveling_time: chrono::Duration,
        schedule: Vec<Event>,
    ) -> TimeReport {
        TimeReport {
            start_time,
            end_time,
            duration,
            lateness,
            working_time,
            waiting_time,
            traveling_time,
            schedule,
        }
    }

    pub fn extend(&mut self, other: TimeReport) {
        self.end_time = other.end_time;
        self.duration += other.duration;
        self.lateness += other.lateness;
        self.working_time += other.working_time;
        self.waiting_time += other.waiting_time;
        self.traveling_time += other.traveling_time;
        self.schedule.extend(other.schedule);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    build_schedule: bool,
) -> TimeReport {
    let mut schedule: Vec<Event> = Vec::new();
    let mut job_duration_left = job_duration;
    let mut current_time = start_time;
    let mut total_waiting_time = chrono::Duration::zero();
    let mut total_lateness = chrono::Duration::zero();
    let mut total_traveling_time = chrono::Duration::zero();
    let mut total_working_time = chrono::Duration::zero();
    let mut tentative_waiting_time = chrono::Duration::zero();
    while job_duration_left > chrono::Duration::zero() {
        // we add job duration left because we want to make sure that if we have time windows (wen it is working not traveling, that we fit completely)
        let maybe_next_time_window =
            job_time_windows.and_then(|time_windows| time_windows.next_window(current_time));
        let mut ready_to_work = true;
        match maybe_next_time_window {
            Some((next_time_window, waiting_time)) => {
                if waiting_time > chrono::Duration::zero() {
                    tentative_waiting_time += waiting_time;
                    current_time += waiting_time;
                    ready_to_work = false;
                }
                let maybe_next_time_window_finish = job_time_windows.and_then(|time_windows| {
                    time_windows.next_window(current_time + job_duration_left)
                });
                match maybe_next_time_window_finish {
                    Some((next_time_window_finish, waiting_time)) => {
                        if next_time_window_finish == next_time_window {
                            // we can finish the job in the next time window
                            ready_to_work = true;
                        } else {
                            // we can not
                            tentative_waiting_time += waiting_time + job_duration_left;
                            current_time += waiting_time + job_duration_left;
                            ready_to_work = false;
                        }
                    }
                    None => {
                        // there is nothing left
                        // MAYBE I HAVE TO ADD SOMETHING HERE
                        tentative_waiting_time += waiting_time;
                        current_time += waiting_time;
                        ready_to_work = true;
                    }
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
            .map(|ot| ot.waiting_time(current_time.naive_local().time()))
            .or_else(|| Some(chrono::Duration::zero()))
            .unwrap();
        if waiting_time > chrono::Duration::zero() {
            tentative_waiting_time += waiting_time;
            current_time += waiting_time;
        } else {
            // we can finally work. If there is tentative waiting time, add event
            if tentative_waiting_time > chrono::Duration::zero() {
                if build_schedule {
                    schedule.push(Event::Wait(TimeWindow::new(
                        current_time - tentative_waiting_time,
                        current_time,
                    )));
                }
                total_waiting_time += tentative_waiting_time;
                tentative_waiting_time = chrono::Duration::zero();
            }
            // check how much time we can work
            // given the checks before, we should be able to do the whole job in the time window.
            let work_time = match operation_times {
                Some(ot) => job_duration_left.min(ot.end() - current_time.time()),
                None => job_duration_left,
            };
            if is_travel {
                if build_schedule {
                    schedule.push(Event::Travel(TimeWindow::new(
                        current_time,
                        current_time + work_time,
                    )));
                }
                total_traveling_time += work_time;
            } else {
                if build_schedule {
                    schedule.push(Event::Work(
                        TimeWindow::new(current_time, current_time + work_time),
                        location_id.unwrap(),
                    ));
                }
                total_working_time += work_time;
                if job_time_windows.is_some() && job_duration_left == work_time {
                    // only add lateness when job is done.
                    total_lateness += job_time_windows.unwrap().lateness(current_time + work_time);
                }
            }
            current_time += work_time;
            job_duration_left -= work_time;
        }
    }
    TimeReport::new(
        start_time,
        current_time,
        current_time - start_time,
        total_lateness,
        total_working_time,
        total_waiting_time,
        total_traveling_time,
        schedule,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_action_report() {
        let start = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let time_windows = TimeWindows::new(vec![]);
        let operation_times = OperationTimes::new(
            chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            chrono::NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
        );
        let report = action_report(
            chrono::Duration::minutes(30),
            Some(&time_windows),
            Some(&operation_times),
            start,
            false,
            Some(0),
            true,
        );
        assert_eq!(report.schedule.len(), 1);
        assert_eq!(
            report.schedule[0],
            Event::Work(
                TimeWindow::new(start, start + chrono::Duration::minutes(30)),
                0
            )
        );
        assert_eq!(report.lateness, chrono::Duration::zero());
        assert_eq!(report.waiting_time, chrono::Duration::zero());
        assert_eq!(report.traveling_time, chrono::Duration::zero());
        assert_eq!(report.working_time, chrono::Duration::minutes(30));
        assert_eq!(report.duration, chrono::Duration::minutes(30));
        assert_eq!(report.start_time, start);
        assert_eq!(report.end_time, start + chrono::Duration::minutes(30));
    }

    #[test]
    fn test_action_report_with_time_windows() {
        let start = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let mut time_windows = TimeWindows::new(vec![]);
        time_windows.add_window(TimeWindow::new(
            Utc.with_ymd_and_hms(2021, 1, 1, 0, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2021, 1, 1, 1, 0, 0).unwrap(),
        ));
        let operation_times = OperationTimes::new(
            chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            chrono::NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
        );
        let report = action_report(
            chrono::Duration::minutes(30),
            Some(&time_windows),
            Some(&operation_times),
            start,
            false,
            Some(0),
            true,
        );
        assert_eq!(report.schedule.len(), 2);
        assert_eq!(
            report.schedule[0],
            Event::Wait(TimeWindow::new(
                start,
                start + chrono::Duration::minutes(30)
            ))
        );
        assert_eq!(
            report.schedule[1],
            Event::Work(
                TimeWindow::new(
                    start + chrono::Duration::minutes(30),
                    start + chrono::Duration::minutes(60)
                ),
                0
            )
        );
        assert_eq!(report.lateness, chrono::Duration::zero());
        assert_eq!(report.waiting_time, chrono::Duration::minutes(30));
        assert_eq!(report.traveling_time, chrono::Duration::zero());
        assert_eq!(report.working_time, chrono::Duration::minutes(30));
        assert_eq!(report.duration, chrono::Duration::minutes(60));
        assert_eq!(report.start_time, start);
        assert_eq!(report.end_time, start + chrono::Duration::minutes(60));
    }
    #[test]
    fn test_action_report_wait_for_operation_time() {
        let start = Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap();
        let mut time_windows = TimeWindows::new(vec![]);
        time_windows.add_window(TimeWindow::new(
            Utc.with_ymd_and_hms(2021, 1, 1, 6, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2021, 1, 2, 18, 0, 0).unwrap(),
        ));
        let operation_times = OperationTimes::new(
            chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        );
        let report = action_report(
            chrono::Duration::minutes(180),
            Some(&time_windows),
            Some(&operation_times),
            start,
            false,
            Some(0),
            true,
        );
        assert_eq!(report.schedule.len(), 2);
        assert_eq!(
            report.schedule[0],
            Event::Wait(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 9, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            report.schedule[1],
            Event::Work(
                TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 1, 9, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).unwrap(),
                ),
                0
            )
        );
        assert_eq!(report.lateness, chrono::Duration::zero());
        assert_eq!(report.waiting_time, chrono::Duration::minutes(60));
        assert_eq!(report.traveling_time, chrono::Duration::zero());
        assert_eq!(report.working_time, chrono::Duration::minutes(180));
        assert_eq!(report.duration, chrono::Duration::minutes(240));
        assert_eq!(report.start_time, start);
        assert_eq!(report.end_time, start + chrono::Duration::minutes(240));
    }

    #[test]
    fn test_action_report_wait_for_time_window() {
        let start = Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap();
        let mut time_windows = TimeWindows::new(vec![]);
        time_windows.add_window(TimeWindow::new(
            Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2021, 1, 2, 18, 0, 0).unwrap(),
        ));
        let operation_times = OperationTimes::new(
            chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        );
        let report = action_report(
            chrono::Duration::minutes(180),
            Some(&time_windows),
            Some(&operation_times),
            start,
            false,
            Some(0),
            true,
        );
        assert_eq!(report.schedule.len(), 2);
        assert_eq!(
            report.schedule[0],
            Event::Wait(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            report.schedule[1],
            Event::Work(
                TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 1, 13, 0, 0).unwrap(),
                ),
                0
            )
        );
        assert_eq!(report.lateness, chrono::Duration::zero());
        assert_eq!(report.waiting_time, chrono::Duration::minutes(120));
        assert_eq!(report.traveling_time, chrono::Duration::zero());
        assert_eq!(report.working_time, chrono::Duration::minutes(180));
        assert_eq!(report.duration, chrono::Duration::minutes(300));
        assert_eq!(report.start_time, start);
        assert_eq!(report.end_time, start + chrono::Duration::minutes(300));
    }

    #[test]
    fn test_action_report_lateness() {
        let start = Utc.with_ymd_and_hms(2021, 1, 3, 8, 0, 0).unwrap();
        let mut time_windows = TimeWindows::new(vec![]);
        time_windows.add_window(TimeWindow::new(
            Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2021, 1, 2, 18, 0, 0).unwrap(),
        ));
        let operation_times = OperationTimes::new(
            chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        );
        let report = action_report(
            chrono::Duration::minutes(180),
            Some(&time_windows),
            Some(&operation_times),
            start,
            false,
            Some(0),
            true,
        );
        assert_eq!(report.schedule.len(), 2);
        assert_eq!(
            report.schedule[0],
            Event::Wait(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 3, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 3, 9, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            report.schedule[1],
            Event::Work(
                TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 3, 9, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 3, 12, 0, 0).unwrap(),
                ),
                0
            )
        );
        assert_eq!(report.lateness, chrono::Duration::hours(18));
        assert_eq!(report.waiting_time, chrono::Duration::minutes(60));
        assert_eq!(report.traveling_time, chrono::Duration::zero());
        assert_eq!(report.working_time, chrono::Duration::minutes(180));
        assert_eq!(report.duration, chrono::Duration::minutes(240));
        assert_eq!(report.start_time, start);
        assert_eq!(report.end_time, start + chrono::Duration::minutes(240));
    }

    #[test]
    fn test_action_report_tw_too_small_for_job() {
        let start = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let mut time_windows = TimeWindows::new(vec![]);
        time_windows.add_window(TimeWindow::new(
            Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap(),
        ));
        let operation_times = OperationTimes::new(
            chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        );
        let report = action_report(
            chrono::Duration::minutes(180),
            Some(&time_windows),
            Some(&operation_times),
            start,
            false,
            Some(0),
            true,
        );
        assert_eq!(report.schedule.len(), 1);
        assert_eq!(
            report.schedule[0],
            Event::Work(
                TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 1, 3, 0, 0).unwrap(),
                ),
                0
            )
        );
        assert_eq!(report.lateness, chrono::Duration::hours(1));
        assert_eq!(report.waiting_time, chrono::Duration::minutes(0));
        assert_eq!(report.traveling_time, chrono::Duration::zero());
        assert_eq!(report.working_time, chrono::Duration::minutes(180));
        assert_eq!(report.duration, chrono::Duration::minutes(180));
        assert_eq!(report.start_time, start);
        assert_eq!(report.end_time, start + chrono::Duration::minutes(180));
    }

    #[test]
    fn test_action_report_ot_too_small_for_job() {
        let start = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let mut time_windows = TimeWindows::new(vec![]);
        time_windows.add_window(TimeWindow::new(
            Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2021, 1, 1, 3, 0, 0).unwrap(),
        ));
        let operation_times = OperationTimes::new(
            chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            chrono::NaiveTime::from_hms_opt(2, 0, 0).unwrap(),
        );
        let report = action_report(
            chrono::Duration::minutes(180),
            Some(&time_windows),
            Some(&operation_times),
            start,
            false,
            Some(0),
            true,
        );
        assert_eq!(report.schedule.len(), 3);
        assert_eq!(
            report.schedule[0],
            Event::Work(
                TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap(),
                ),
                0
            )
        );
        assert_eq!(
            report.schedule[1],
            Event::Wait(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            report.schedule[2],
            Event::Work(
                TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 2, 1, 0, 0).unwrap(),
                ),
                0
            )
        );
        assert_eq!(report.lateness, chrono::Duration::hours(22));
        assert_eq!(report.waiting_time, chrono::Duration::hours(22));
        assert_eq!(report.traveling_time, chrono::Duration::zero());
        assert_eq!(report.working_time, chrono::Duration::minutes(180));
        assert_eq!(report.duration, chrono::Duration::hours(25));
        assert_eq!(report.start_time, start);
        assert_eq!(report.end_time, start + chrono::Duration::hours(25));
    }

    #[test]
    fn test_action_report_travel_many_days() {
        let start = Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap();
        let operation_times = OperationTimes::new(
            chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        );
        let report = action_report(
            chrono::Duration::days(1),
            None,
            Some(&operation_times),
            start,
            true,
            None,
            true,
        );
        assert_eq!(report.schedule.len(), 5);
        assert_eq!(
            report.schedule[0],
            Event::Travel(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 17, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            report.schedule[1],
            Event::Wait(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 17, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 2, 8, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            report.schedule[2],
            Event::Travel(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 2, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 2, 17, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            report.schedule[3],
            Event::Wait(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 2, 17, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 3, 8, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            report.schedule[4],
            Event::Travel(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 3, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 3, 14, 0, 0).unwrap(),
            ))
        );
        assert_eq!(report.lateness, chrono::Duration::zero());
        assert_eq!(report.waiting_time, chrono::Duration::hours(30));
        assert_eq!(report.traveling_time, chrono::Duration::hours(24));
        assert_eq!(report.working_time, chrono::Duration::zero());
        assert_eq!(report.duration, chrono::Duration::hours(54));
        assert_eq!(report.start_time, start);
        assert_eq!(report.end_time, start + chrono::Duration::hours(54));
    }
    #[test]
    fn test_action_report_travel_no_operation_times() {
        let start = Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap();
        let report = action_report(
            chrono::Duration::days(1),
            None,
            None,
            start,
            true,
            None,
            true,
        );
        assert_eq!(report.schedule.len(), 1);
        assert_eq!(
            report.schedule[0],
            Event::Travel(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 2, 8, 0, 0).unwrap(),
            ))
        );
        assert_eq!(report.lateness, chrono::Duration::zero());
        assert_eq!(report.waiting_time, chrono::Duration::zero());
        assert_eq!(report.traveling_time, chrono::Duration::days(1));
        assert_eq!(report.working_time, chrono::Duration::zero());
        assert_eq!(report.duration, chrono::Duration::days(1));
        assert_eq!(report.start_time, start);
        assert_eq!(report.end_time, start + chrono::Duration::days(1));
    }
    #[test]
    fn test_action_report_work_no_operation_times() {
        let start = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let mut time_windows = TimeWindows::new(vec![]);
        time_windows.add_window(TimeWindow::new(
            Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2021, 1, 2, 3, 0, 0).unwrap(),
        ));
        let report = action_report(
            chrono::Duration::hours(3),
            Some(&time_windows),
            None,
            start,
            false,
            Some(0),
            true,
        );
        assert_eq!(report.schedule.len(), 2);
        assert_eq!(
            report.schedule[0],
            Event::Wait(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            report.schedule[1],
            Event::Work(
                TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 2, 3, 0, 0).unwrap(),
                ),
                0
            )
        );
        assert_eq!(report.lateness, chrono::Duration::zero());
        assert_eq!(report.waiting_time, chrono::Duration::days(1));
        assert_eq!(report.traveling_time, chrono::Duration::zero());
        assert_eq!(report.working_time, chrono::Duration::minutes(180));
        assert_eq!(report.duration, chrono::Duration::hours(27));
        assert_eq!(report.start_time, start);
        assert_eq!(report.end_time, start + chrono::Duration::hours(27));
    }
}
