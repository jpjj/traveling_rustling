use crate::distance_matrix::DistanceMatrix;

#[derive(Debug, Clone)]
pub struct Route{
    pub sequence: Vec<usize>,
}

impl Route{
    pub fn new(sequence: Vec<usize>) -> Route{
        Route{
            sequence,
        }
    }
    pub fn len(&self) -> usize{
        self.sequence.len()
    }

    pub fn distance(&self, matrix: &DistanceMatrix) -> u64{
        let mut distance = 0;
        for i in 0..self.sequence.len() - 1{
            distance += matrix.distance(self.sequence[i], self.sequence[i + 1]);
        }
        distance + matrix.distance(self.sequence[self.sequence.len() - 1], self.sequence[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distance_matrix::DistanceMatrix;

    #[test]
    fn test_route() {
        let matrix = DistanceMatrix::new(vec![
            vec![0, 1, 2],
            vec![30, 0, 40],
            vec![500, 600, 0],
        ]);
        let route = Route::new(vec![0, 1, 2]);
        assert_eq!(route.distance(&matrix), 541);
    }
}