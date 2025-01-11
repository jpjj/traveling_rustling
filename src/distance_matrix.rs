pub struct DistanceMatrix {
    matrix: Vec<Vec<u64>>,
}

impl DistanceMatrix {
    pub fn new(matrix: Vec<Vec<u64>>) -> DistanceMatrix {
        DistanceMatrix {
            matrix,
        }
    }

    pub fn distance(&self, i: usize, j: usize) -> u64 {
        self.matrix[i][j]
    }

    pub fn len(&self) -> usize {
        self.matrix.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_matrix() {
        let matrix = vec![
            vec![0, 1, 2],
            vec![30, 0, 40],
            vec![500, 600, 0],
        ];
        let distance_matrix = DistanceMatrix::new(matrix);
        assert_eq!(distance_matrix.distance(0, 1), 1);
        assert_eq!(distance_matrix.distance(1, 2), 40);
        assert_eq!(distance_matrix.distance(2, 0), 500);

    }
}