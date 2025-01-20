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

use solver::Solver;

/// Solving the Traveling Salesman Problem with Time Windows.
#[pyfunction]
#[pyo3(signature = (distance_matrix, duration_matrix=None, job_durations=None, time_windows=None, operation_times=None, working_days=None, travel_duration_until_break=None, break_duration=None, time_limit=None, init_route=None))]
fn solve(
    distance_matrix: Vec<Vec<u64>>,
    duration_matrix: Option<Vec<Vec<u64>>>,
    job_durations: Option<Vec<u64>>,
    time_windows: Option<Vec<Vec<(u64, u64)>>>,
    operation_times: Option<(u64, u64)>,
    working_days: Option<Vec<bool>>,
    travel_duration_until_break: Option<u64>,
    break_duration: Option<u64>,
    time_limit: Option<u64>,
    init_route: Option<Vec<usize>>,
) -> PyResult<PyOutput> {
    let input = input::get_input_from_raw(
        distance_matrix,
        duration_matrix,
        job_durations,
        time_windows,
        operation_times,
        working_days,
        travel_duration_until_break,
        break_duration,
        time_limit,
        init_route,
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
