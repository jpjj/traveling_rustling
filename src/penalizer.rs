use crate::{
    output::Solution,
    penalties::{
        distance::DistanceMatrix,
        time::{
            time_input::TimeInput,
            time_output::{action_report, TimeReport},
        },
    },
    route::Route,
};

pub struct Penalizer {
    pub distance_matrix: DistanceMatrix,
    pub time_input: Option<TimeInput>,
}

impl Penalizer {
    pub fn new(distance_matrix: DistanceMatrix, time_input: Option<TimeInput>) -> Penalizer {
        Penalizer {
            distance_matrix,
            time_input,
        }
    }

    pub fn penalize(&self, route: Route, build_schedule: bool) -> Solution {
        let distance = self.distance(&route);
        let time_report = self.time(&route, build_schedule);
        Solution {
            route,
            distance,
            time_report,
        }
    }

    pub fn is_better(&self, sol1: &Solution, sol2: &Solution) -> bool {
        match &self.time_input {
            None => sol1.distance < sol2.distance,
            Some(_) => {
                let time_report1 = sol1.time_report.as_ref().unwrap();
                let time_report2 = sol2.time_report.as_ref().unwrap();
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

    pub fn distance(&self, route: &Route) -> u64 {
        let mut distance = 0;
        for i in 0..route.len() - 1 {
            distance += self.distance_matrix.distance(route[i], route[i + 1]);
        }
        distance
            + self
                .distance_matrix
                .distance(route[route.len() - 1], route[0])
    }

    pub fn time(&self, route: &Route, build_schedule: bool) -> Option<TimeReport> {
        match &self.time_input {
            None => None,
            Some(time_input) => {
                // Here comes the functionalities of the time penalizer
                // We go through the route one location after the other
                // we fist assume that we are at current location.
                // then we call a cycle function that will calculate
                // a subsequence of waiting and working or waiting and moving
                // until either the job duration is over or the traveling time is over
                // all inside the operation times.

                // we start at the first opening time of the first location
                let mut start_time = time_input.time_windows[route.sequence[0]][0].start;
                let mut time_report = TimeReport::new(
                    start_time,
                    start_time,
                    chrono::Duration::zero(),
                    chrono::Duration::zero(),
                    chrono::Duration::zero(),
                    chrono::Duration::zero(),
                    chrono::Duration::zero(),
                    vec![],
                );
                let operation_times = time_input.operation_times.as_ref();
                for (i, &location) in route.sequence.iter().enumerate() {
                    start_time = time_report.end_time;
                    let job_duration = time_input.job_durations[location];
                    let job_time_windows = &time_input.time_windows[location];

                    time_report.extend(action_report(
                        job_duration,
                        Some(job_time_windows),
                        operation_times,
                        start_time,
                        false,
                        Some(location),
                        build_schedule,
                    ));
                    let next_i = (i + 1) % route.sequence.len();
                    let travel_duration =
                        time_input.travel_time(route.sequence[i], route.sequence[next_i]);
                    start_time = time_report.end_time;

                    time_report.extend(action_report(
                        travel_duration,
                        None,
                        operation_times,
                        start_time,
                        true,
                        None,
                        build_schedule,
                    ));
                }
                Some(time_report)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::penalties::time::{
        operation_times::OperationTimes,
        time_output::Event,
        time_windows::{TimeWindow, TimeWindows},
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
            working_days: None,
            travel_duration_until_break: None,
            break_duration: None,
        });
        let penalizer = Penalizer::new(distance_matrix, time_input);
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
            Utc.with_ymd_and_hms(2021, 1, 3, 11, 0, 0).unwrap()
        );
        assert_eq!(time_report.duration, chrono::Duration::hours(53));
        assert_eq!(time_report.waiting_time, chrono::Duration::hours(38));
        assert_eq!(time_report.working_time, chrono::Duration::hours(9));
        assert_eq!(time_report.traveling_time, chrono::Duration::hours(6));
        assert_eq!(time_report.lateness, chrono::Duration::hours(21));
        assert_eq!(time_report.schedule.len(), 10);
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
            Event::Work(
                TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 2, 14, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 2, 16, 0, 0).unwrap(),
                ),
                2
            )
        );
        assert_eq!(
            time_report.schedule[7],
            Event::Wait(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 2, 16, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 3, 8, 0, 0).unwrap(),
            ))
        );
        assert_eq!(
            time_report.schedule[8],
            Event::Work(
                TimeWindow::new(
                    Utc.with_ymd_and_hms(2021, 1, 3, 8, 0, 0).unwrap(),
                    Utc.with_ymd_and_hms(2021, 1, 3, 9, 0, 0).unwrap(),
                ),
                2
            )
        );
        assert_eq!(
            time_report.schedule[9],
            Event::Travel(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 3, 9, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 3, 11, 0, 0).unwrap(),
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
            working_days: None,
            travel_duration_until_break: None,
            break_duration: None,
        });
        let penalizer = Penalizer::new(distance_matrix, time_input);
        let route1 = Route::new(vec![0, 1, 2]);
        let route2 = Route::new(vec![2, 1, 0]);
        let solution1 = penalizer.penalize(route1, true);
        let solution2 = penalizer.penalize(route2, true);
        assert!(penalizer.is_better(&solution2, &solution1));
    }
}
