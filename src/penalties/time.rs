pub mod operation_times;
pub mod time_input;
pub mod time_output;
pub mod time_windows;

pub struct TimePenalizer {
    time_input: TimeInput,
    time_output: TimeOutput,
}

impl TimePenalizer {
    pub fn new(time_input: TimeInput, time_output: TimeOutput) -> TimePenalizer {
        TimePenalizer {
            time_input,
            time_output,
        }
    }

    pub fn penalize(&self, route: &Route) -> u64 {
        let mut penalty = 0;
        let mut current_time = chrono::Duration::zero();
        let mut current_location = 0;
        for next_location in route.iter() {
            let travel_time = self.time_input.travel_time(current_location, next_location);
            current_time = current_time + travel_time;
            let job_duration = self.time_input.job_durations[next_location];
            current_time = current_time + job_duration;
            let time_window = self.time_input.time_windows[next_location];
            if current_time < time_window.0 {
                penalty += (time_window.0 - current_time).num_seconds() as u64;
            } else if current_time > time_window.1 {
                penalty += (current_time - time_window.1).num_seconds() as u64;
            }
            current_location = next_location;
        }
        penalty
    }
}
