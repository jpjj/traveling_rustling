use super::{operation_times::OperationTimes, time_windows::TimeWindows};

/// input for time window constraints

pub struct TimeScheduler {
    pub duration_matrix: Vec<Vec<chrono::Duration>>,
    pub job_durations: Vec<chrono::Duration>,
    pub time_windows: Vec<TimeWindows>,
    pub operation_times: OperationTimes,
}

impl TimeScheduler {
    // pub fn new(duration_matrix: Vec<Vec<chrono::Duration>>, job_durations: Vec<chrono::Duration>, time_windows: Vec<TimeWindows>, operation_times: OperationTimes) -> TimeScheduler{
    //     TimeScheduler{
    //         duration_matrix,
    //         job_durations,
    //         time_windows,
    //         operation_times
    //     }
    // }

    pub fn travel_time(&self, from: usize, to: usize) -> chrono::Duration {
        self.duration_matrix[from][to]
    }
}
