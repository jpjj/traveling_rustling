use crate::{
    output::Solution,
    penalties::{
        distance::DistancePenalizer,
        time::{
            time_output::{Complete, TimeOutput},
            TimePenalizer,
        },
    },
    route::Route,
};

pub struct Penalizer {
    pub distance_penalizer: DistancePenalizer,
    pub time_penalizer: Option<TimePenalizer>,
}

impl Penalizer {
    pub fn new(
        distance_penalizer: DistancePenalizer,
        time_penalizer: Option<TimePenalizer>,
    ) -> Penalizer {
        Penalizer {
            distance_penalizer,
            time_penalizer,
        }
    }

    pub fn penalize(&self, route: Route, build_schedule: bool) -> Solution {
        let distance = self.distance_penalizer.penalize(&route);
        let time_report = self.time(&route, build_schedule);
        Solution {
            route,
            distance,
            time_report,
        }
    }

    pub fn is_better(&self, sol1: &Solution, sol2: &Solution) -> bool {
        match &self.time_penalizer {
            None => sol1.distance < sol2.distance,
            Some(_) => {
                let time_report1 = sol1.time_report.as_ref().unwrap();
                let time_report2 = sol2.time_report.as_ref().unwrap();
                if time_report1.job_splits < time_report2.job_splits {
                    return true;
                }
                if time_report1.job_splits > time_report2.job_splits {
                    return false;
                }
                if time_report1.lateness < time_report2.lateness {
                    return true;
                }
                if time_report1.lateness > time_report2.lateness {
                    return false;
                }
                if time_report1.traveling_time < time_report2.traveling_time {
                    return true;
                }
                if time_report1.traveling_time > time_report2.traveling_time {
                    return false;
                }
                if time_report1.duration < time_report2.duration {
                    return true;
                }
                if time_report1.duration > time_report2.duration {
                    return false;
                }
                if time_report1.waiting_time < time_report2.waiting_time {
                    return true;
                }
                if time_report1.waiting_time > time_report2.waiting_time {
                    return false;
                }
                sol1.distance < sol2.distance
            }
        }
    }

    pub fn time(&self, route: &Route, build_schedule: bool) -> Option<TimeOutput<Complete>> {
        match &self.time_penalizer {
            None => None,
            Some(time_penalizer) => Some(time_penalizer.penalize(&route, build_schedule)),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::penalties::{
        distance::DistanceMatrix,
        time::{
            operation_times::OperationTimes,
            time_input::TimeInput,
            time_output::Event,
            time_windows::{TimeWindow, TimeWindows},
        },
    };
    use chrono::{NaiveTime, TimeZone, Utc};

    #[test]
    fn test_penalizer() {
        let distance_matrix =
            DistanceMatrix::new(vec![vec![0, 1, 2], vec![1, 0, 3], vec![2, 3, 0]]);
        let time_input = Some(TimeInput {
            job_durations: vec![chrono::Duration::hours(3); 3],
            time_windows: vec![
                TimeWindows::new(vec![
                    TimeWindow::new(
                        Utc.with_ymd_and_hms(2021, 1, 1, 6, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).unwrap(),
                    ),
                    TimeWindow::new(
                        Utc.with_ymd_and_hms(2021, 1, 2, 6, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2021, 1, 2, 12, 0, 0).unwrap(),
                    ),
                ]),
                TimeWindows::new(vec![
                    TimeWindow::new(
                        Utc.with_ymd_and_hms(2021, 1, 1, 6, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).unwrap(),
                    ),
                    TimeWindow::new(
                        Utc.with_ymd_and_hms(2021, 1, 2, 6, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2021, 1, 2, 12, 0, 0).unwrap(),
                    ),
                ]),
                TimeWindows::new(vec![
                    TimeWindow::new(
                        Utc.with_ymd_and_hms(2021, 1, 1, 6, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).unwrap(),
                    ),
                    TimeWindow::new(
                        Utc.with_ymd_and_hms(2021, 1, 2, 6, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2021, 1, 2, 12, 0, 0).unwrap(),
                    ),
                ]),
            ],
            operation_times: Some(OperationTimes::new(
                NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
                None,
            )),
            duration_matrix: vec![
                vec![
                    chrono::Duration::hours(0),
                    chrono::Duration::hours(1),
                    chrono::Duration::hours(2),
                ],
                vec![
                    chrono::Duration::hours(1),
                    chrono::Duration::hours(0),
                    chrono::Duration::hours(3),
                ],
                vec![
                    chrono::Duration::hours(2),
                    chrono::Duration::hours(3),
                    chrono::Duration::hours(0),
                ],
            ],
            travel_duration_until_break: None,
            break_duration: None,
        });
        let distance_penalizer = DistancePenalizer::new(distance_matrix);
        let time_penalizer = TimePenalizer::new(time_input.unwrap());
        let penalizer = Penalizer::new(distance_penalizer, Some(time_penalizer));
        let route = Route::new(vec![0, 1, 2]);
        let solution = penalizer.penalize(route, true);
        assert_eq!(solution.distance, 6);
        let time_report = solution.time_report.unwrap();
        assert_eq!(
            time_report.start_time,
            Utc.with_ymd_and_hms(2021, 1, 1, 6, 0, 0).unwrap()
        );
        assert_eq!(
            time_report.end_time,
            Utc.with_ymd_and_hms(2021, 1, 3, 13, 0, 0).unwrap()
        );
        assert_eq!(time_report.duration, chrono::Duration::hours(55));
        assert_eq!(time_report.waiting_time, chrono::Duration::hours(40));
        assert_eq!(time_report.working_time, chrono::Duration::hours(9));
        assert_eq!(time_report.traveling_time, chrono::Duration::hours(6));
        assert_eq!(time_report.lateness, chrono::Duration::hours(23));
        assert_eq!(time_report.schedule.len(), 9);
        assert_eq!(
            time_report.schedule[0],
            Event::Wait(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 6, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            time_report.schedule[1],
            Event::Work(
                TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 1, 11, 0, 0).unwrap(),
                ),
                0
            )
        );
        assert_eq!(
            time_report.schedule[2],
            Event::Travel(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 11, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            time_report.schedule[3],
            Event::Wait(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 2, 8, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            time_report.schedule[4],
            Event::Work(
                TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 2, 8, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 2, 11, 0, 0).unwrap(),
                ),
                1
            )
        );
        assert_eq!(
            time_report.schedule[5],
            Event::Travel(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 2, 11, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 2, 14, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            time_report.schedule[6],
            Event::Wait(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 2, 14, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 3, 8, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            time_report.schedule[7],
            Event::Work(
                TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 3, 8, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 3, 11, 0, 0).unwrap(),
                ),
                2,
            )
        );
        assert_eq!(
            time_report.schedule[8],
            Event::Travel(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 3, 11, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 3, 13, 0, 0).unwrap(),
            ))
        );
    }

    #[test]
    fn test_is_better() {
        let distance_matrix =
            DistanceMatrix::new(vec![vec![0, 1, 2], vec![1, 0, 3], vec![2, 3, 0]]);
        let time_input = Some(TimeInput {
            job_durations: vec![chrono::Duration::hours(3); 3],
            time_windows: vec![
                TimeWindows::new(vec![
                    TimeWindow::new(
                        Utc.with_ymd_and_hms(2021, 1, 1, 6, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).unwrap(),
                    ),
                    TimeWindow::new(
                        Utc.with_ymd_and_hms(2021, 1, 3, 6, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2021, 1, 3, 12, 0, 0).unwrap(),
                    ),
                ]),
                TimeWindows::new(vec![
                    TimeWindow::new(
                        Utc.with_ymd_and_hms(2021, 1, 1, 6, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).unwrap(),
                    ),
                    TimeWindow::new(
                        Utc.with_ymd_and_hms(2021, 1, 2, 6, 0, 0).unwrap(),
                        Utc.with_ymd_and_hms(2021, 1, 2, 12, 0, 0).unwrap(),
                    ),
                ]),
                TimeWindows::new(vec![TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 1, 6, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).unwrap(),
                )]),
            ],
            operation_times: Some(OperationTimes::new(
                NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
                None,
            )),
            duration_matrix: vec![
                vec![
                    chrono::Duration::hours(0),
                    chrono::Duration::hours(1),
                    chrono::Duration::hours(2),
                ],
                vec![
                    chrono::Duration::hours(1),
                    chrono::Duration::hours(0),
                    chrono::Duration::hours(3),
                ],
                vec![
                    chrono::Duration::hours(2),
                    chrono::Duration::hours(3),
                    chrono::Duration::hours(0),
                ],
            ],
            travel_duration_until_break: None,
            break_duration: None,
        });
        let distance_penalizer = DistancePenalizer::new(distance_matrix);
        let time_penalizer = TimePenalizer::new(time_input.unwrap());
        let penalizer = Penalizer::new(distance_penalizer, Some(time_penalizer));
        let route1 = Route::new(vec![0, 1, 2]);
        let route2 = Route::new(vec![2, 1, 0]);
        let solution1 = penalizer.penalize(route1, true);
        let solution2 = penalizer.penalize(route2, true);
        assert!(penalizer.is_better(&solution2, &solution1));
    }
}
