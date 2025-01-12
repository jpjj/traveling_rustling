use std::ops::Index;

#[derive(Debug, Clone)]
pub struct Route {
    pub sequence: Vec<usize>,
}

impl Route {
    pub fn new(sequence: Vec<usize>) -> Route {
        Route { sequence }
    }
    pub fn len(&self) -> usize {
        self.sequence.len()
    }
}
impl Index<usize> for Route {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.sequence[index]
    }
}
