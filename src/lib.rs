// mod time_windows;
mod solver;
mod distance_matrix;
mod route;
mod local_moves;
use distance_matrix::DistanceMatrix;
use pyo3::prelude::*;

use solver::Solver;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn solve(distance_matrix: Vec<Vec<u64>>) -> PyResult<Vec<usize>> {
    let real_distance_matrix = DistanceMatrix::new(distance_matrix);
    let mut solver = Solver::new(real_distance_matrix, None);
    solver.solve();

    Ok(solver.get_best_sequence())
}

/// A Python module implemented in Rust.
#[pymodule]
fn traveling_rustling(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve, m)?)?;
    Ok(())
}
