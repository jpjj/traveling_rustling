pub mod distance_matrix;

pub use distance_matrix::DistanceMatrix;

use crate::route::Route;

pub struct DistancePenalizer {
    distance_matrix: DistanceMatrix,
}

impl DistancePenalizer {
    pub fn new(distance_matrix: DistanceMatrix) -> DistancePenalizer {
        DistancePenalizer { distance_matrix }
    }

    pub fn penalize(&self, route: &Route) -> u64 {
        let mut distance = 0;
        for i in 0..route.len() - 1 {
            distance += self.distance_matrix.distance(route[i], route[i + 1]);
        }
        distance
            + self
                .distance_matrix
                .distance(route[route.len() - 1], route[0])
    }
}
