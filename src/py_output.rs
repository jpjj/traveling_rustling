use crate::output::Solution;
use crate::penalties::time::time_output::Event;
use pyo3::prelude::*;
use pyo3::{pyclass, pymethods, types::PyDelta};

#[pyclass]
pub struct PyOutput {
    pub solution: Solution,
    pub iterations: u64,
    pub time_taken: chrono::Duration,
}

impl PyOutput {
    pub fn new(solution: Solution, iterations: u64, time_taken: chrono::Duration) -> Self {
        PyOutput {
            solution,
            iterations,
            time_taken,
        }
    }
}

#[pymethods]
impl PyOutput {
    #[getter]
    fn distance(&self) -> u64 {
        self.solution.distance
    }
    #[getter]
    fn route(&self) -> Vec<usize> {
        self.solution.route.sequence.clone()
    }
    #[getter]
    fn lateness(&self) -> Option<u64> {
        match &self.solution.time_report {
            Some(time_report) => Some(time_report.lateness.num_seconds() as u64),
            None => None,
        }
    }
    #[getter]
    fn duration(&self) -> Option<u64> {
        match &self.solution.time_report {
            Some(time_report) => Some(time_report.duration.num_seconds() as u64),
            None => None,
        }
    }
    #[getter]
    fn working_time(&self) -> Option<u64> {
        match &self.solution.time_report {
            Some(time_report) => Some(time_report.working_time.num_seconds() as u64),
            None => None,
        }
    }
    #[getter]
    fn waiting_time(&self) -> Option<u64> {
        match &self.solution.time_report {
            Some(time_report) => Some(time_report.waiting_time.num_seconds() as u64),
            None => None,
        }
    }
    #[getter]
    fn traveling_time(&self) -> Option<u64> {
        match &self.solution.time_report {
            Some(time_report) => Some(time_report.traveling_time.num_seconds() as u64),
            None => None,
        }
    }
    #[getter]
    fn start_time(&self) -> Option<u64> {
        match &self.solution.time_report {
            Some(time_report) => Some(time_report.start_time.timestamp() as u64),
            None => None,
        }
    }
    #[getter]
    fn end_time(&self) -> Option<u64> {
        match &self.solution.time_report {
            Some(time_report) => Some(time_report.end_time.timestamp() as u64),
            None => None,
        }
    }
    #[getter]
    fn iterations(&self) -> u64 {
        self.iterations
    }
    #[getter]
    fn time_taken_microseconds(&self) -> u64 {
        self.time_taken.num_microseconds().unwrap() as u64
    }
    #[getter]
    fn schedule(&self) -> Option<Vec<PyEvent>> {
        match &self.solution.time_report {
            Some(time_report) => Some(
                time_report
                    .schedule
                    .iter()
                    .map(|event| match event {
                        Event::Work(window, location) => PyEvent::Work(PyWork {
                            window: (
                                window.start.timestamp() as u64,
                                window.end.timestamp() as u64,
                            ),
                            location: *location,
                        }),
                        Event::Wait(window) => PyEvent::Wait(PyWait {
                            window: (
                                window.start.timestamp() as u64,
                                window.end.timestamp() as u64,
                            ),
                        }),
                        Event::Travel(window) => PyEvent::Travel(PyTravel {
                            window: (
                                window.start.timestamp() as u64,
                                window.end.timestamp() as u64,
                            ),
                        }),
                    })
                    .collect(),
            ),
            None => None,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyWork {
    pub window: (u64, u64),
    pub location: usize,
}

#[pymethods]
impl PyWork {
    #[getter]
    fn location(&self) -> usize {
        self.location
    }
    #[getter]
    fn window(&self) -> (u64, u64) {
        self.window
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyWait {
    pub window: (u64, u64),
}
#[pymethods]
impl PyWait {
    #[getter]
    fn window(&self) -> (u64, u64) {
        self.window
    }
}
#[pyclass]
#[derive(Clone)]
pub struct PyTravel {
    pub window: (u64, u64),
}
#[pymethods]
impl PyTravel {
    #[getter]
    fn window(&self) -> (u64, u64) {
        self.window
    }
}
#[pyclass]
enum PyEvent {
    Work(PyWork),
    Wait(PyWait),
    Travel(PyTravel),
}
