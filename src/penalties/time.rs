pub mod operation_times;
pub mod time_input;
pub mod time_output;
pub mod time_windows;
use chrono::Utc;
use time_input::TimeInput;
use time_output::TimeOutput;

use crate::route::Route;

struct TentativeTimeOutput {
    time_output: TimeOutput,
    current_time: chrono::DateTime<Utc>,
}

impl TentativeTimeOutput {
    pub fn new(start_time: chrono::DateTime<Utc>) -> TentativeTimeOutput {
        TentativeTimeOutput {
            time_output: TimeOutput::new(start_time),
            current_time: start_time,
        }
    }

    pub fn complete(self) -> TimeOutput {
        self.time_output
    }
}

struct WorkingTimePenalizer<'a> {
    time_input: TimeInput,
    route: &'a Route,
    tentative_time_output: TentativeTimeOutput,
    build_schedule: bool,
}

impl<'a> WorkingTimePenalizer<'a> {
    fn new(
        time_input: TimeInput,
        route: &Route,
        start_time: chrono::DateTime<Utc>,
        build_schedule: bool,
    ) -> WorkingTimePenalizer {
        WorkingTimePenalizer {
            time_input,
            route,
            tentative_time_output: TentativeTimeOutput::new(start_time),
            build_schedule,
        }
    }
    fn finish_schedule(self) -> TimeOutput {
        for (i, _) in self.route.sequence.iter().enumerate() {
            self.add_job(i);
            self.add_travel(i);
        }

        self.tentative_time_output.complete()
    }

    fn add_job(&self, i: usize) {
        // We assume that we are at the current location
        // There are a lot of things to check.
        // RULE 1: We can't start a job before the opening time
        // RULE 2: The job has to be finished before the closing time
        // This means we have to wait until we are before or within a time window
        // that is big enough to fit the job duration.
        let location = self.route.sequence[i];
        let job_duration = self.time_input.job_durations[location];
        let time_windows = self.time_input.time_windows[location];
        let current_time = self.tentative_time_output.current_time;

        let mut job_completed = false;

        // it follows a while loop that searches for a time when
        // 1. there is a time window of the location that fits the job
        // 2. the job can be done within the operation times
        // Different cases can happen:
        // A. We find a slot that works for time window and opening times
        // B. There is no possible time anymore where the job can be done in one go.
        // If B happens, we allow the job to be splitable, but we increase the split penalty by one.
        // We run the loop again with the split option.
        // Now, we work the job off incrementally until it is done.
        // if in this case, we get past the last time window, we continue by applying lateness penalty.

        // This results in having a new penalty in our lexicographic optimization:
        // 1. Minimize the number of splits
        // 2. Minimize the lateness
        // 3. Minimize the travel time
        // 4. Minimize the makespan
        // 5. Minimize the waiting time
        // our solution is feasible, if there are no splits nor lateness.
        
        while !job_completed {
            // We first check if we are within a time window
            // that is big enough to fit the job duration
            let maybe_next_fitting_time_tw = time_windows.find_next_fitting_time(current_time, job_duration);
            match maybe_next_fitting_time_tw {
                Some(next_fitting_time_tw) => {
                    // We have found a fitting time window
                    // We have to check if the job can be done within the operation times
                    let maybe_next_fitting_time_op = self.time_input.operation_times.find_next_fitting_time(next_fitting_time_tw.start, job_duration);
                    match maybe_next_fitting_time_op {
                        Some(next_fitting_time_op) => {
                            // We have found a fitting operation time
                            // If the two fitting times are the same, we can do the job now
                            if next_fitting_time_tw == next_fitting_time_op {
                                self.tentative_time_output.time_output.add_job(i, next_fitting_time_op, next_fitting_time_op);
                                job_completed = true;
                            } else {
                                // Otherwise, we have to wait until the next fitting time window
                                current_time = next_fitting_time_op;
                                job_completed = false;
                            }
                        }
                        None => {
                            // We have not found a fitting operation time
                            // This basically means that the job never fits our operation times
                            // In this case, we set the job to be splitable, but increase the "split penalty" by one.
                            self.tentative_time_output.time_output.add_waiting(location, next_fitting_time_tw.start);
                            current_time = next_fitting_time_tw.start;
                        }
                    }
                }
                None => {
                    // We have not found a fitting time window
                    // We have to wait until the next fitting time window
                    self.tentative_time_output.time_output.add_waiting(location, time_windows.find_next_fitting_time(current_time, 0).unwrap().start);
                    current_time = time_windows.find_next_fitting_time(current_time, 0).unwrap().start;
                }
            }
        // if None is returned, it means that there is no fitting time window left.
        // Hence, we might as well do the job now.
    }
    fn add_travel(&self, i: usize) {}
}

pub struct TimePenalizer {
    time_input: TimeInput,
}

impl TimePenalizer {
    pub fn new(time_input: TimeInput) -> TimePenalizer {
        TimePenalizer { time_input }
    }
    pub fn penalize(&self, route: &Route, build_schedule: bool) -> TimeOutput {
        // Here comes the functionalities of the time penalizer
        // We go through the route one location after the other
        // we fist assume that we are at current location.
        // then we call a cycle function that will calculate
        // a subsequence of waiting and working or waiting and moving
        // until either the job duration is over or the traveling time is over
        // all inside the operation times.

        // we start at the first opening time of the first location
        let mut start_time = self.time_input.time_windows[route.sequence[0]][0].start;
        let mut working_time_penalizer =
            WorkingTimePenalizer::new(self.time_input, route, start_time, build_schedule);
        working_time_penalizer.finish_schedule()
    }
}
