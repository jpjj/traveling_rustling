use super::time_windows::{TimeWindow, TimeWindows};

/// Time report module for outputs and evaluation of the time schedule.

#[derive(Debug, Clone)]
pub struct TimeOutput<S: CompletionState> {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub duration: chrono::Duration,
    pub lateness: chrono::Duration,
    pub working_time: chrono::Duration,
    pub waiting_time: chrono::Duration,
    pub traveling_time: chrono::Duration,
    pub job_splits: u32,
    pub schedule: Vec<Event>,
    phantom: std::marker::PhantomData<S>,
}

pub enum Incomplete {}
#[derive(Debug, Clone)]
pub enum Complete {}

trait CompletionState {}
impl CompletionState for Incomplete {}
impl CompletionState for Complete {}

impl TimeOutput<Incomplete> {
    pub fn new(start_time: chrono::DateTime<chrono::Utc>) -> TimeOutput<Incomplete> {
        TimeOutput {
            start_time,
            end_time: start_time,
            duration: chrono::Duration::zero(),
            lateness: chrono::Duration::zero(),
            working_time: chrono::Duration::zero(),
            waiting_time: chrono::Duration::zero(),
            traveling_time: chrono::Duration::zero(),
            job_splits: 0,
            schedule: vec![],
            phantom: std::marker::PhantomData,
        }
    }
    pub fn add_waiting(&mut self, time_window: TimeWindow, build_schedule: bool) {
        let duration = time_window.duration();
        self.waiting_time += duration;
        self.end_time += duration;
        self.duration += duration;
        if build_schedule {
            self.schedule.push(Event::Wait(time_window));
        }
    }
    pub fn add_traveling(&mut self, time_window: TimeWindow, build_schedule: bool) {
        let duration = time_window.duration();
        self.traveling_time += duration;
        self.end_time += duration;
        self.duration += duration;
        if build_schedule {
            self.schedule.push(Event::Travel(time_window));
        }
    }
    pub fn add_working(&mut self, location: usize, time_window: TimeWindow, build_schedule: bool) {
        let duration = time_window.duration();
        self.working_time += duration;
        self.end_time += duration;
        self.duration += duration;
        if build_schedule {
            self.schedule.push(Event::Work(time_window, location));
        }
    }
    pub fn add_split(&mut self) {
        self.job_splits += 1;
    }
    pub fn add_lateness(&mut self, lateness: chrono::Duration) {
        self.lateness += lateness;
    }
    pub fn complete(self) -> TimeOutput<Complete> {
        TimeOutput {
            start_time: self.start_time,
            end_time: self.end_time,
            duration: self.duration,
            lateness: self.lateness,
            working_time: self.working_time,
            waiting_time: self.waiting_time,
            traveling_time: self.traveling_time,
            job_splits: self.job_splits,
            schedule: self.schedule,
            phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Work(TimeWindow, usize),
    Travel(TimeWindow),
    Wait(TimeWindow),
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use chrono::TimeZone;

//     #[test]
//     fn test_action_report() {
//         let start = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
//         let time_windows = TimeWindows::new(vec![]);
//         let operation_times = OperationTimes::new(
//             chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
//             chrono::NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
//         );
//         let report = action_report(
//             chrono::Duration::minutes(30),
//             Some(&time_windows),
//             Some(&operation_times),
//             start,
//             false,
//             Some(0),
//             true,
//         );
//         assert_eq!(report.schedule.len(), 1);
//         assert_eq!(
//             report.schedule[0],
//             Event::Work(
//                 TimeWindow::new(start, start + chrono::Duration::minutes(30)),
//                 0
//             )
//         );
//         assert_eq!(report.lateness, chrono::Duration::zero());
//         assert_eq!(report.waiting_time, chrono::Duration::zero());
//         assert_eq!(report.traveling_time, chrono::Duration::zero());
//         assert_eq!(report.working_time, chrono::Duration::minutes(30));
//         assert_eq!(report.duration, chrono::Duration::minutes(30));
//         assert_eq!(report.start_time, start);
//         assert_eq!(report.end_time, start + chrono::Duration::minutes(30));
//     }

//     #[test]
//     fn test_action_report_with_time_windows() {
//         let start = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
//         let mut time_windows = TimeWindows::new(vec![]);
//         time_windows.add_window(TimeWindow::new(
//             Utc.with_ymd_and_hms(2021, 1, 1, 0, 30, 0).unwrap(),
//             Utc.with_ymd_and_hms(2021, 1, 1, 1, 0, 0).unwrap(),
//         ));
//         let operation_times = OperationTimes::new(
//             chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
//             chrono::NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
//         );
//         let report = action_report(
//             chrono::Duration::minutes(30),
//             Some(&time_windows),
//             Some(&operation_times),
//             start,
//             false,
//             Some(0),
//             true,
//         );
//         assert_eq!(report.schedule.len(), 2);
//         assert_eq!(
//             report.schedule[0],
//             Event::Wait(TimeWindow::new(
//                 start,
//                 start + chrono::Duration::minutes(30)
//             ))
//         );
//         assert_eq!(
//             report.schedule[1],
//             Event::Work(
//                 TimeWindow::new(
//                     start + chrono::Duration::minutes(30),
//                     start + chrono::Duration::minutes(60)
//                 ),
//                 0
//             )
//         );
//         assert_eq!(report.lateness, chrono::Duration::zero());
//         assert_eq!(report.waiting_time, chrono::Duration::minutes(30));
//         assert_eq!(report.traveling_time, chrono::Duration::zero());
//         assert_eq!(report.working_time, chrono::Duration::minutes(30));
//         assert_eq!(report.duration, chrono::Duration::minutes(60));
//         assert_eq!(report.start_time, start);
//         assert_eq!(report.end_time, start + chrono::Duration::minutes(60));
//     }
//     #[test]
//     fn test_action_report_wait_for_operation_time() {
//         let start = Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap();
//         let mut time_windows = TimeWindows::new(vec![]);
//         time_windows.add_window(TimeWindow::new(
//             Utc.with_ymd_and_hms(2021, 1, 1, 6, 0, 0).unwrap(),
//             Utc.with_ymd_and_hms(2021, 1, 2, 18, 0, 0).unwrap(),
//         ));
//         let operation_times = OperationTimes::new(
//             chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
//             chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
//         );
//         let report = action_report(
//             chrono::Duration::minutes(180),
//             Some(&time_windows),
//             Some(&operation_times),
//             start,
//             false,
//             Some(0),
//             true,
//         );
//         assert_eq!(report.schedule.len(), 2);
//         assert_eq!(
//             report.schedule[0],
//             Event::Wait(TimeWindow::new(
//                 Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
//                 Utc.with_ymd_and_hms(2021, 1, 1, 9, 0, 0).unwrap(),
//             ))
//         );
//         assert_eq!(
//             report.schedule[1],
//             Event::Work(
//                 TimeWindow::new(
//                     Utc.with_ymd_and_hms(2021, 1, 1, 9, 0, 0).unwrap(),
//                     Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).unwrap(),
//                 ),
//                 0
//             )
//         );
//         assert_eq!(report.lateness, chrono::Duration::zero());
//         assert_eq!(report.waiting_time, chrono::Duration::minutes(60));
//         assert_eq!(report.traveling_time, chrono::Duration::zero());
//         assert_eq!(report.working_time, chrono::Duration::minutes(180));
//         assert_eq!(report.duration, chrono::Duration::minutes(240));
//         assert_eq!(report.start_time, start);
//         assert_eq!(report.end_time, start + chrono::Duration::minutes(240));
//     }

//     #[test]
//     fn test_action_report_wait_for_time_window() {
//         let start = Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap();
//         let mut time_windows = TimeWindows::new(vec![]);
//         time_windows.add_window(TimeWindow::new(
//             Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap(),
//             Utc.with_ymd_and_hms(2021, 1, 2, 18, 0, 0).unwrap(),
//         ));
//         let operation_times = OperationTimes::new(
//             chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
//             chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
//         );
//         let report = action_report(
//             chrono::Duration::minutes(180),
//             Some(&time_windows),
//             Some(&operation_times),
//             start,
//             false,
//             Some(0),
//             true,
//         );
//         assert_eq!(report.schedule.len(), 2);
//         assert_eq!(
//             report.schedule[0],
//             Event::Wait(TimeWindow::new(
//                 Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
//                 Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap(),
//             ))
//         );
//         assert_eq!(
//             report.schedule[1],
//             Event::Work(
//                 TimeWindow::new(
//                     Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap(),
//                     Utc.with_ymd_and_hms(2021, 1, 1, 13, 0, 0).unwrap(),
//                 ),
//                 0
//             )
//         );
//         assert_eq!(report.lateness, chrono::Duration::zero());
//         assert_eq!(report.waiting_time, chrono::Duration::minutes(120));
//         assert_eq!(report.traveling_time, chrono::Duration::zero());
//         assert_eq!(report.working_time, chrono::Duration::minutes(180));
//         assert_eq!(report.duration, chrono::Duration::minutes(300));
//         assert_eq!(report.start_time, start);
//         assert_eq!(report.end_time, start + chrono::Duration::minutes(300));
//     }

//     #[test]
//     fn test_action_report_lateness() {
//         let start = Utc.with_ymd_and_hms(2021, 1, 3, 8, 0, 0).unwrap();
//         let mut time_windows = TimeWindows::new(vec![]);
//         time_windows.add_window(TimeWindow::new(
//             Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap(),
//             Utc.with_ymd_and_hms(2021, 1, 2, 18, 0, 0).unwrap(),
//         ));
//         let operation_times = OperationTimes::new(
//             chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
//             chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
//         );
//         let report = action_report(
//             chrono::Duration::minutes(180),
//             Some(&time_windows),
//             Some(&operation_times),
//             start,
//             false,
//             Some(0),
//             true,
//         );
//         assert_eq!(report.schedule.len(), 2);
//         assert_eq!(
//             report.schedule[0],
//             Event::Wait(TimeWindow::new(
//                 Utc.with_ymd_and_hms(2021, 1, 3, 8, 0, 0).unwrap(),
//                 Utc.with_ymd_and_hms(2021, 1, 3, 9, 0, 0).unwrap(),
//             ))
//         );
//         assert_eq!(
//             report.schedule[1],
//             Event::Work(
//                 TimeWindow::new(
//                     Utc.with_ymd_and_hms(2021, 1, 3, 9, 0, 0).unwrap(),
//                     Utc.with_ymd_and_hms(2021, 1, 3, 12, 0, 0).unwrap(),
//                 ),
//                 0
//             )
//         );
//         assert_eq!(report.lateness, chrono::Duration::hours(18));
//         assert_eq!(report.waiting_time, chrono::Duration::minutes(60));
//         assert_eq!(report.traveling_time, chrono::Duration::zero());
//         assert_eq!(report.working_time, chrono::Duration::minutes(180));
//         assert_eq!(report.duration, chrono::Duration::minutes(240));
//         assert_eq!(report.start_time, start);
//         assert_eq!(report.end_time, start + chrono::Duration::minutes(240));
//     }

//     #[test]
//     fn test_action_report_tw_too_small_for_job() {
//         let start = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
//         let mut time_windows = TimeWindows::new(vec![]);
//         time_windows.add_window(TimeWindow::new(
//             Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
//             Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap(),
//         ));
//         let operation_times = OperationTimes::new(
//             chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
//             chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
//         );
//         let report = action_report(
//             chrono::Duration::minutes(180),
//             Some(&time_windows),
//             Some(&operation_times),
//             start,
//             false,
//             Some(0),
//             true,
//         );
//         assert_eq!(report.schedule.len(), 1);
//         assert_eq!(
//             report.schedule[0],
//             Event::Work(
//                 TimeWindow::new(
//                     Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
//                     Utc.with_ymd_and_hms(2021, 1, 1, 3, 0, 0).unwrap(),
//                 ),
//                 0
//             )
//         );
//         assert_eq!(report.lateness, chrono::Duration::hours(1));
//         assert_eq!(report.waiting_time, chrono::Duration::minutes(0));
//         assert_eq!(report.traveling_time, chrono::Duration::zero());
//         assert_eq!(report.working_time, chrono::Duration::minutes(180));
//         assert_eq!(report.duration, chrono::Duration::minutes(180));
//         assert_eq!(report.start_time, start);
//         assert_eq!(report.end_time, start + chrono::Duration::minutes(180));
//     }

//     #[test]
//     fn test_action_report_ot_too_small_for_job() {
//         let start = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
//         let mut time_windows = TimeWindows::new(vec![]);
//         time_windows.add_window(TimeWindow::new(
//             Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
//             Utc.with_ymd_and_hms(2021, 1, 1, 3, 0, 0).unwrap(),
//         ));
//         let operation_times = OperationTimes::new(
//             chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
//             chrono::NaiveTime::from_hms_opt(2, 0, 0).unwrap(),
//         );
//         let report = action_report(
//             chrono::Duration::minutes(180),
//             Some(&time_windows),
//             Some(&operation_times),
//             start,
//             false,
//             Some(0),
//             true,
//         );
//         assert_eq!(report.schedule.len(), 3);
//         assert_eq!(
//             report.schedule[0],
//             Event::Work(
//                 TimeWindow::new(
//                     Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
//                     Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap(),
//                 ),
//                 0
//             )
//         );
//         assert_eq!(
//             report.schedule[1],
//             Event::Wait(TimeWindow::new(
//                 Utc.with_ymd_and_hms(2021, 1, 1, 2, 0, 0).unwrap(),
//                 Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap(),
//             ))
//         );
//         assert_eq!(
//             report.schedule[2],
//             Event::Work(
//                 TimeWindow::new(
//                     Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap(),
//                     Utc.with_ymd_and_hms(2021, 1, 2, 1, 0, 0).unwrap(),
//                 ),
//                 0
//             )
//         );
//         assert_eq!(report.lateness, chrono::Duration::hours(22));
//         assert_eq!(report.waiting_time, chrono::Duration::hours(22));
//         assert_eq!(report.traveling_time, chrono::Duration::zero());
//         assert_eq!(report.working_time, chrono::Duration::minutes(180));
//         assert_eq!(report.duration, chrono::Duration::hours(25));
//         assert_eq!(report.start_time, start);
//         assert_eq!(report.end_time, start + chrono::Duration::hours(25));
//     }

//     #[test]
//     fn test_action_report_travel_many_days() {
//         let start = Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap();
//         let operation_times = OperationTimes::new(
//             chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
//             chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
//         );
//         let report = action_report(
//             chrono::Duration::days(1),
//             None,
//             Some(&operation_times),
//             start,
//             true,
//             None,
//             true,
//         );
//         assert_eq!(report.schedule.len(), 5);
//         assert_eq!(
//             report.schedule[0],
//             Event::Travel(TimeWindow::new(
//                 Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
//                 Utc.with_ymd_and_hms(2021, 1, 1, 17, 0, 0).unwrap(),
//             ))
//         );
//         assert_eq!(
//             report.schedule[1],
//             Event::Wait(TimeWindow::new(
//                 Utc.with_ymd_and_hms(2021, 1, 1, 17, 0, 0).unwrap(),
//                 Utc.with_ymd_and_hms(2021, 1, 2, 8, 0, 0).unwrap(),
//             ))
//         );
//         assert_eq!(
//             report.schedule[2],
//             Event::Travel(TimeWindow::new(
//                 Utc.with_ymd_and_hms(2021, 1, 2, 8, 0, 0).unwrap(),
//                 Utc.with_ymd_and_hms(2021, 1, 2, 17, 0, 0).unwrap(),
//             ))
//         );
//         assert_eq!(
//             report.schedule[3],
//             Event::Wait(TimeWindow::new(
//                 Utc.with_ymd_and_hms(2021, 1, 2, 17, 0, 0).unwrap(),
//                 Utc.with_ymd_and_hms(2021, 1, 3, 8, 0, 0).unwrap(),
//             ))
//         );
//         assert_eq!(
//             report.schedule[4],
//             Event::Travel(TimeWindow::new(
//                 Utc.with_ymd_and_hms(2021, 1, 3, 8, 0, 0).unwrap(),
//                 Utc.with_ymd_and_hms(2021, 1, 3, 14, 0, 0).unwrap(),
//             ))
//         );
//         assert_eq!(report.lateness, chrono::Duration::zero());
//         assert_eq!(report.waiting_time, chrono::Duration::hours(30));
//         assert_eq!(report.traveling_time, chrono::Duration::hours(24));
//         assert_eq!(report.working_time, chrono::Duration::zero());
//         assert_eq!(report.duration, chrono::Duration::hours(54));
//         assert_eq!(report.start_time, start);
//         assert_eq!(report.end_time, start + chrono::Duration::hours(54));
//     }
//     #[test]
//     fn test_action_report_travel_no_operation_times() {
//         let start = Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap();
//         let report = action_report(
//             chrono::Duration::days(1),
//             None,
//             None,
//             start,
//             true,
//             None,
//             true,
//         );
//         assert_eq!(report.schedule.len(), 1);
//         assert_eq!(
//             report.schedule[0],
//             Event::Travel(TimeWindow::new(
//                 Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
//                 Utc.with_ymd_and_hms(2021, 1, 2, 8, 0, 0).unwrap(),
//             ))
//         );
//         assert_eq!(report.lateness, chrono::Duration::zero());
//         assert_eq!(report.waiting_time, chrono::Duration::zero());
//         assert_eq!(report.traveling_time, chrono::Duration::days(1));
//         assert_eq!(report.working_time, chrono::Duration::zero());
//         assert_eq!(report.duration, chrono::Duration::days(1));
//         assert_eq!(report.start_time, start);
//         assert_eq!(report.end_time, start + chrono::Duration::days(1));
//     }
//     #[test]
//     fn test_action_report_work_no_operation_times() {
//         let start = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
//         let mut time_windows = TimeWindows::new(vec![]);
//         time_windows.add_window(TimeWindow::new(
//             Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap(),
//             Utc.with_ymd_and_hms(2021, 1, 2, 3, 0, 0).unwrap(),
//         ));
//         let report = action_report(
//             chrono::Duration::hours(3),
//             Some(&time_windows),
//             None,
//             start,
//             false,
//             Some(0),
//             true,
//         );
//         assert_eq!(report.schedule.len(), 2);
//         assert_eq!(
//             report.schedule[0],
//             Event::Wait(TimeWindow::new(
//                 Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
//                 Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap(),
//             ))
//         );
//         assert_eq!(
//             report.schedule[1],
//             Event::Work(
//                 TimeWindow::new(
//                     Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap(),
//                     Utc.with_ymd_and_hms(2021, 1, 2, 3, 0, 0).unwrap(),
//                 ),
//                 0
//             )
//         );
//         assert_eq!(report.lateness, chrono::Duration::zero());
//         assert_eq!(report.waiting_time, chrono::Duration::days(1));
//         assert_eq!(report.traveling_time, chrono::Duration::zero());
//         assert_eq!(report.working_time, chrono::Duration::minutes(180));
//         assert_eq!(report.duration, chrono::Duration::hours(27));
//         assert_eq!(report.start_time, start);
//         assert_eq!(report.end_time, start + chrono::Duration::hours(27));
//     }
// }
