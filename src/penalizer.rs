use crate::{
    output::Solution,
    penalties::{
        distance::DistanceMatrix,
        time::{time_input::TimeScheduler, time_output::TimeReport},
    },
    route::Route,
};

pub struct Penalizer {
    pub distance_matrix: DistanceMatrix,
    pub time_input: Option<TimeScheduler>,
}

impl Penalizer {
    pub fn new(distance_matrix: DistanceMatrix, time_input: Option<TimeScheduler>) -> Penalizer {
        Penalizer {
            distance_matrix,
            time_input,
        }
    }

    pub fn penalize(&self, route: Route) -> Solution {
        let distance = self.distance(&route);
        let time_report = self.time(&route);
        Solution {
            route,
            distance,
            time_report,
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

    pub fn time(&self, route: &Route) -> Option<TimeReport> {
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
                let current_time = time_input.time_windows[route.sequence[0]][0].start;
                let _start_time = current_time;
                let _total_working_time = chrono::Duration::zero();
                let _total_waiting_time = chrono::Duration::zero();
                let _total_traveling_time = chrono::Duration::zero();
                for (_i, _location) in route.sequence.iter().enumerate() {}
                None
            }
        }
    }
}
