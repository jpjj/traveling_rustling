// this will be what we get from the outside world and what will be inserted into the solver

use crate::{
    penalties::{self, distance::DistanceMatrix, time::time_input::TimeScheduler},
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

pub(crate) fn get_input_from_raw(
    distance_matrix: Vec<Vec<u64>>,
    duration_matrix: Option<Vec<Vec<u64>>>,
    job_durations: Option<Vec<u64>>,
    time_windows: Option<Vec<Vec<(u64, u64)>>>,
    operation_times: Option<(u64, u64)>,
    time_limit: Option<u64>,
) -> Input {
    let real_distance_matrix = DistanceMatrix::new(distance_matrix);
    let time_input = penalties::time::time_input::transform(
        duration_matrix,
        job_durations,
        time_windows,
        operation_times,
    );
    let time_limit = match time_limit {
        Some(limit) => Some(chrono::Duration::seconds(limit as i64)),
        None => None,
    };
    Input::new(real_distance_matrix, time_input, time_limit, None)
}
