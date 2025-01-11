use chrono::TimeDelta;

use crate::{distance_matrix::DistanceMatrix, local_moves::{one_shift_left, one_shift_right, swap, three_shift_left, three_shift_right, two_opt, two_shift_left, two_shift_right}, route::{self, Route}};
use std::cmp::Ordering;
use rand::thread_rng;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
struct Solution {
    route: Route,
    distance: u64,
}

impl Ord for Solution {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.cmp(&other.distance)
    }
}
impl PartialOrd for Solution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Solution {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}
impl Eq for Solution {}


pub struct Solver {
    distance_matrix: DistanceMatrix,
    n: usize,
    current_solution: Solution,
    best_solution: Solution,
    time_limit: Option<TimeDelta>,
    start: chrono::DateTime<chrono::Utc>,
}

impl Solver {
    pub fn new(distance_matrix: DistanceMatrix, time_limit: Option<TimeDelta>) -> Solver {

        let n = distance_matrix.len();
        let best_route = Route::new((0..n).collect());
        let best_distance = best_route.distance(&distance_matrix);
        let best_solution = Solution {
            route: best_route,
            distance: best_distance,
        };
        let current_solution = best_solution.clone();
        let start = chrono::Utc::now();
        Solver {
            distance_matrix,
            n,
            current_solution,
            best_solution,
            time_limit,
            start,
        }
    }

    fn evaluate(&self, route: Route) -> Solution {
        let distance = route.distance(&self.distance_matrix);
        Solution {
            route,
            distance,
        }
    }

    fn generate_initial_solution(&self) -> Solution {
        let mut sequence = (0..=self.n-1).collect::<Vec<usize>>();
        sequence.shuffle(&mut thread_rng());
        let route = Route::new(sequence);
        self.evaluate(route)
    }


    fn run_move(&mut self, local_move: &mut dyn FnMut(&mut Route, usize, usize), min_margin: usize) -> bool {
        let mut improved = false;
        for i in 0..self.n {
            for j in i+1+min_margin..self.n {
                let mut new_route = self.current_solution.route.clone();
                local_move(&mut new_route, i, j);
                let new_solution = self.evaluate(new_route);
                if new_solution < self.current_solution {
                    self.current_solution = new_solution;
                    improved = true;
                }
            }
        }
        improved
    }
    fn run_heuristics(&mut self) -> bool{
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
        match self.time_limit {
            Some(limit) => {
                chrono::Utc::now() - self.start > limit
            }
            None => true
        }
    }

    fn one_time(&self) -> bool {
        return self.time_limit.is_none();
    }

    pub fn solve(&mut self) {
        let mut improved = true;
        self.start = chrono::Utc::now();
        let time_left = true;
        while self.termination_criterion() {
            improved = true;
            while improved & self.termination_criterion(){
                improved = self.run_heuristics()
            }

            if self.current_solution < self.best_solution {
                self.best_solution = self.current_solution.clone();
            }
            self.current_solution = self.generate_initial_solution();

            if self.one_time() {
                break;
            }
        }
        
    }

    pub fn get_best_sequence(&self) -> Vec<usize>{
        self.best_solution.route.sequence.clone()
    }
}


#[cfg(test)]
mod tests {
    use core::time;

    use super::*;
    use crate::distance_matrix::DistanceMatrix;

    #[test]
    fn test_solver() {
        let matrix = DistanceMatrix::new(vec![
            vec![0, 2, 1],
            vec![40, 0, 30],
            vec![600, 500, 0],
        ]);
        let mut solver = Solver::new(matrix, None);
        solver.solve();
        assert_eq!(solver.best_solution.distance, 541);
        assert_eq!(solver.best_solution.route.sequence, vec![1, 0, 2]);

    }
}
