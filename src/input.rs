// this will be what we get from the outside world and what will be inserted into the solver

use crate::{
    penalties::{distance::DistanceMatrix, time::time_input::TimeScheduler},
    route::Route,
};

pub struct Input {
    pub distance_matrix: DistanceMatrix,
    pub time_input: Option<TimeScheduler>,
    pub time_limit: Option<chrono::Duration>,
    pub init_route: Option<Route>,
}

impl Input {
    pub fn new(
        distance_matrix: DistanceMatrix,
        time_input: Option<TimeScheduler>,
        time_limit: Option<chrono::Duration>,
        init_route: Option<Route>,
    ) -> Input {
        Input {
            distance_matrix,
            time_input,
            time_limit,
            init_route,
        }
    }
}
