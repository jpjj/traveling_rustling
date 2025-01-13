use chrono::TimeDelta;

use crate::{
    input::Input,
    local_moves::{
        one_shift_left, one_shift_right, swap, three_shift_left, three_shift_right, two_opt,
        two_shift_left, two_shift_right,
    },
    output::Solution,
    penalizer::Penalizer,
    route::Route,
};

use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct Solver {
    n: usize,
    penalizer: Penalizer,
    current_solution: Solution,
    pub best_solution: Solution,
    time_limit: Option<TimeDelta>,
    start: chrono::DateTime<chrono::Utc>,
    pub iterations: u64,
    pub time_taken: chrono::Duration,
}

impl Solver {
    pub fn new(input: Input) -> Solver {
        let n = input.distance_matrix.len();
        let distance_matrix = input.distance_matrix;
        let time_limit = input.time_limit;
        let penalizer: Penalizer = Penalizer::new(distance_matrix, input.time_input);
        let route = match input.init_route {
            Some(route) => route,
            None => Route::new((0..n).collect()),
        };
        let current_solution = penalizer.penalize(route, false);
        let best_solution = current_solution.clone();
        let start = chrono::Utc::now();
        Solver {
            n,
            penalizer,
            current_solution,
            best_solution,
            time_limit,
            start,
            iterations: 0,
            time_taken: chrono::Duration::zero(),
        }
    }

    fn generate_initial_solution(&self) -> Solution {
        let mut sequence = (0..=self.n - 1).collect::<Vec<usize>>();
        sequence.shuffle(&mut thread_rng());
        let route = Route::new(sequence);
        self.penalizer.penalize(route, false)
    }

    fn run_move(
        &mut self,
        local_move: &mut dyn FnMut(&mut Route, usize, usize),
        min_margin: usize,
    ) -> bool {
        let mut improved = false;
        for i in 0..self.n {
            for j in i + 1 + min_margin..self.n {
                let mut new_route = self.current_solution.route.clone();
                local_move(&mut new_route, i, j);
                let new_solution = self.penalizer.penalize(new_route, false);
                if self
                    .penalizer
                    .is_better(&new_solution, &self.current_solution)
                {
                    self.current_solution = new_solution;
                    improved = true;
                }
            }
        }
        improved
    }
    fn run_heuristics(&mut self) -> bool {
        let mut improved = false;
        improved |= self.run_move(&mut two_opt, 0);
        // for 0 and 1, we have the same move as for 2opt
        improved |= self.run_move(&mut swap, 2);
        // for 0, it is like swapping neighbors
        improved |= self.run_move(&mut one_shift_left, 1);
        improved |= self.run_move(&mut one_shift_right, 1);
        // 0 would be a two city intervall being rotated by 2, so no change
        // 1 would be like a 3 city intervall being rotated by 1 in the other direction
        improved |= self.run_move(&mut two_shift_left, 2);
        // 2 would be like a 4 city intervall being roated by 2, already done in other direction
        improved |= self.run_move(&mut two_shift_right, 3);

        // 0 would lead to an error.
        // 1 would be a 3 city intervall being rotated by 3, so no change.
        // 2 would be a 4 city intervall being rotated by 1 in the other direction
        // 3 would be a 5 city intervall being rotated by 2 in the other direction
        improved |= self.run_move(&mut three_shift_left, 4);
        // 4 would be like a 6 city intervall being roated by 3, already done in other direction
        improved |= self.run_move(&mut three_shift_right, 5);
        improved
    }

    fn termination_criterion(&self) -> bool {
        // returns true if the termination criterion is met
        // no time limit means we always continue
        match self.time_limit {
            Some(limit) => chrono::Utc::now() - self.start <= limit,
            None => true,
        }
    }

    fn one_time(&self) -> bool {
        self.time_limit.is_none()
    }

    pub fn solve(&mut self) {
        let mut improved = true;
        self.start = chrono::Utc::now();
        while self.termination_criterion() {
            self.iterations += 1;
            improved = true;
            while improved & self.termination_criterion() {
                improved = self.run_heuristics()
            }

            if self
                .penalizer
                .is_better(&self.current_solution, &self.best_solution)
            {
                self.best_solution = self.current_solution.clone();
            }
            self.current_solution = self.generate_initial_solution();

            if self.one_time() {
                break;
            }
        }
        // finally, we also build the schedule
        self.best_solution = self
            .penalizer
            .penalize(self.best_solution.route.clone(), true);
        self.time_taken = chrono::Utc::now() - self.start;
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveTime, TimeZone, Utc};

    use crate::{
        input,
        penalties::{
            distance::DistanceMatrix,
            time::{
                operation_times::OperationTimes,
                time_input::TimeScheduler,
                time_windows::{TimeWindow, TimeWindows},
            },
        },
    };

    use super::*;

    #[test]
    fn test_solver() {
        let matrix = DistanceMatrix::new(vec![vec![0, 2, 1], vec![40, 0, 30], vec![600, 500, 0]]);
        let input = Input::new(matrix, None, None, None);
        let mut solver = Solver::new(input);
        solver.solve();
        assert_eq!(solver.best_solution.distance, 541);
        assert_eq!(solver.best_solution.route.sequence, vec![1, 0, 2]);
    }

    #[test]
    fn test_solver_time_limit() {
        let matrix = DistanceMatrix::new(vec![vec![0, 2, 1], vec![40, 0, 30], vec![600, 500, 0]]);
        let input = Input::new(matrix, None, Some(TimeDelta::milliseconds(100)), None);
        let mut solver = Solver::new(input);
        let start = chrono::Utc::now();
        solver.solve();
        let end = chrono::Utc::now();
        assert!(end - start >= TimeDelta::milliseconds(100));
        assert!(end - start < TimeDelta::milliseconds(150));
        assert_eq!(solver.best_solution.distance, 541);
        assert_eq!(solver.best_solution.route.sequence, vec![1, 0, 2]);
    }

    #[test]
    fn test_solver_time_input() {
        let distance_matrix =
            DistanceMatrix::new(vec![vec![0, 1, 2], vec![1, 0, 3], vec![2, 3, 0]]);
        let time_input = Some(TimeScheduler {
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
        });
        let input = Input::new(distance_matrix, time_input, None, None);
        let mut solver = Solver::new(input);
        solver.solve();
        assert_eq!(solver.best_solution.route.sequence, vec![2, 1, 0]);
    }

    #[test]
    fn test_solve_raw_input() {
        let distance_matrix = vec![vec![0]];
        let duration_matrix = Some(vec![vec![0]]);
        let job_durations = Some(vec![10800]);
        let time_windows = Some(vec![vec![(1735689600, 1736035200)]]);
        let operation_times = Some((0, 82800));
        let input = input::get_input_from_raw(
            distance_matrix,
            duration_matrix,
            job_durations,
            time_windows,
            operation_times,
            None,
        );
        let mut solver = Solver::new(input);
        solver.solve();
        let solution = solver.best_solution.clone();
        assert_eq!(solution.route.sequence, vec![0]);
        assert_eq!(
            chrono::DateTime::from_timestamp(1735689600, 0).unwrap(),
            chrono::Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()
        );
        assert_eq!(
            chrono::DateTime::from_timestamp(1736035200, 0).unwrap(),
            chrono::Utc.with_ymd_and_hms(2025, 1, 5, 0, 0, 0).unwrap()
        );
        assert_eq!(
            solution.time_report.unwrap().waiting_time,
            chrono::Duration::zero()
        );
    }
}
