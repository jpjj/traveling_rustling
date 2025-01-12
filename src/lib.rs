// mod time_windows;
mod solver;

mod input;
mod local_moves;
mod output;
mod penalizer;
mod penalties;
mod route;
use pyo3::prelude::*;

use penalties::distance::DistanceMatrix;
use solver::Solver;
/// Formats the sum of two numbers as string.
#[pyfunction]
fn solve(distance_matrix: Vec<Vec<u64>>) -> PyResult<Vec<usize>> {
    let real_distance_matrix = DistanceMatrix::new(distance_matrix);
    let input = input::Input::new(real_distance_matrix, None, None, None);
    let mut solver = Solver::new(input);
    solver.solve();

    Ok(solver.get_best_sequence())
}

/// A Python module implemented in Rust.
#[pymodule]
fn traveling_rustling(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve, m)?)?;
    Ok(())
}
