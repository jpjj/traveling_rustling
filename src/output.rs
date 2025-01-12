use std::cmp::Ordering;

use crate::{penalties::time::time_output::TimeReport, route::Route};

pub struct Output {
    pub solution: Solution,
    pub iterations: u64,
    pub time_taken: chrono::Duration,
}

#[derive(Clone)]
pub struct Solution {
    pub route: Route,
    pub distance: u64,
    pub time_report: Option<TimeReport>,
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
