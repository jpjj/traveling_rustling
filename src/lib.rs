// mod time_windows;
mod input;
mod local_moves;
mod output;
mod penalizer;
mod penalties;
mod py_output;
mod route;
mod solver;

use py_output::PyOutput;
use pyo3::prelude::*;

use penalties::distance::DistanceMatrix;
use solver::Solver;

/// Solving the Traveling Salesman Problem with Time Windows.
#[pyfunction]
#[pyo3(signature = (distance_matrix, duration_matrix=None, job_durations=None, time_windows=None, operation_times=None, time_limit=None))]
fn solve(
    distance_matrix: Vec<Vec<u64>>,
    duration_matrix: Option<Vec<Vec<u64>>>,
    job_durations: Option<Vec<u64>>,
    time_windows: Option<Vec<Vec<(u64, u64)>>>,
    operation_times: Option<(u64, u64)>,
    time_limit: Option<u64>,
) -> PyResult<PyOutput> {
    let input = input::get_input_from_raw(
        distance_matrix,
        duration_matrix,
        job_durations,
        time_windows,
        operation_times,
        time_limit,
    );
    let mut solver = Solver::new(input);
    solver.solve();

    //Ok(solver.get_best_sequence())
    Ok(PyOutput::new(
        solver.best_solution.clone(),
        solver.iterations,
        solver.time_taken,
    ))
}

/// A Python module implemented in Rust.
#[pymodule]
fn traveling_rustling(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve, m)?)?;
    m.add_class::<PyOutput>()?;
    Ok(())
}
