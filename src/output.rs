use crate::{penalties::time::time_output::TimeOutput, route::Route};

pub struct Output {
    pub solution: Solution,
    pub iterations: u64,
    pub time_taken: chrono::Duration,
}

#[derive(Clone)]
pub struct Solution {
    pub route: Route,
    pub distance: u64,
    pub time_report: Option<TimeOutput<Complete>>,
}
