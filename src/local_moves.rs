use crate::route::Route;
/// for all moves it holdds that 0 <= i < j < n.
pub fn two_opt(route: &mut Route, i: usize, j: usize){
    route.sequence[i..j+1].reverse();
}

pub fn swap(route: &mut Route, i: usize, j: usize){
    route.sequence.swap(i, j);
}

pub fn one_shift_left(route: &mut Route, i: usize, j: usize){
    route.sequence[i..j+1].rotate_left(1);
}

pub fn two_shift_left(route: &mut Route, i: usize, j: usize){
    route.sequence[i..j+1].rotate_left(2);
}

pub fn three_shift_left(route: &mut Route, i: usize, j: usize){
    route.sequence[i..j+1].rotate_left(3);
}

pub fn one_shift_right(route: &mut Route, i: usize, j: usize){
    route.sequence[i..j+1].rotate_right(1);
}

pub fn two_shift_right(route: &mut Route, i: usize, j: usize){
    route.sequence[i..j+1].rotate_right(2);
}

pub fn three_shift_right(route: &mut Route, i: usize, j: usize){
    route.sequence[i..j+1].rotate_right(3);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route::Route;

    #[test]
    fn test_two_opt() {
        let mut route = Route::new(vec![0, 1, 2, 3, 4]);
        two_opt(&mut route, 0, 4);
        assert_eq!(route.sequence, vec![4, 3, 2, 1, 0]);
        two_opt(&mut route, 0, 1);
        assert_eq!(route.sequence, vec![3, 4, 2, 1, 0]);
    }

    #[test]
    fn test_swap() {
        let mut route = Route::new(vec![0, 1, 2, 3, 4]);
        swap(&mut route, 0, 4);
        assert_eq!(route.sequence, vec![4, 1, 2, 3, 0]);
    }

    #[test]
    fn test_one_left_shift() {
        let mut route = Route::new(vec![0, 1, 2, 3, 4]);
        one_shift_left(&mut route, 1, 3);
        assert_eq!(route.sequence, vec![0, 2, 3, 1, 4]);
    }

    #[test]
    fn test_two_left_shift() {
        let mut route = Route::new(vec![0, 1, 2, 3, 4]);
        two_shift_left(&mut route, 1, 3);
        assert_eq!(route.sequence, vec![0, 3, 1, 2, 4]);
    }

    #[test]
    fn test_three_left_shift() {
        let mut route = Route::new(vec![0, 1, 2, 3, 4]);
        three_shift_left(&mut route, 1, 3);
        assert_eq!(route.sequence, vec![0, 1, 2, 3, 4]);
    }
}