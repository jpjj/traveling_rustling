pub mod operation_times;
pub mod time_input;
pub mod time_output;
pub mod time_windows;
use std::cmp::max;

use chrono::{Duration, Utc};
use time_input::TimeInput;
use time_output::{Complete, Incomplete, TimeOutput};
use time_windows::TimeWindow;

use crate::route::Route;

struct WorkingTimePenalizer<'a> {
    time_input: TimeInput,
    route: &'a Route,
    time_output: TimeOutput<Incomplete>,
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
            time_output: TimeOutput::new(start_time),
            build_schedule,
        }
    }

    fn finish_schedule(self) -> TimeOutput<Complete> {
        for (i, _) in self.route.sequence.iter().enumerate() {
            self.execute_job(i);
            self.execute_travel(i);
        }

        self.time_output.complete()
    }
    fn add_job(&mut self, location: usize, time_window: TimeWindow) {
        // Add waiting between time_output.current_time and time_window.start
        // Add waiting time to time_output
        let waiting_duration = time_window
            .start
            .signed_duration_since(self.time_output.end_time);
        self.add_waiting(waiting_duration);
        self.time_output.add_job(location, time_window);
    }
    fn add_split(&mut self) {
        self.time_output.add_split();
    }
    fn add_travel(&mut self, time_window: TimeWindow) {
        let waiting_duration = time_window
            .start
            .signed_duration_since(self.time_output.end_time);
        self.add_waiting(waiting_duration);
        self.time_output
            .add_travel(time_window, self.build_schedule);
    }
    fn add_waiting(&mut self, duration: Duration) {
        if duration > chrono::Duration::zero() {
            self.time_output.add_waiting(
                TimeWindow::new(
                    self.time_output.end_time,
                    self.time_output.end_time + duration,
                ),
                self.build_schedule,
            );
        }
    }

    fn execute_job(&self, i: usize) {
        // We assume that we are at the current location
        let location = self.route.sequence[i];
        let job_duration = self.time_input.job_durations[location];
        let time_windows = self.time_input.time_windows[location];
        let operation_times = self.time_input.operation_times.unwrap(); // TODO there should always be operation times here. If we work 24/7, this should be handled in operation times.
        let mut current_time = self.time_output.current_time;
        let mut tent_current_time = current_time;
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
        let mut must_fit = false;

        while !job_completed {
            // We first check if we are within a time window
            // that is big enough to fit the job duration
            let maybe_next_time_tw =
                time_windows.find_next_fitting_time(current_time, job_duration, must_fit);
            let maybe_next_time_op =
                operation_times.find_next_fitting_time(current_time, job_duration, must_fit);
            match (maybe_next_time_tw, maybe_next_time_op) {
                (Some(next_time_tw), Some(next_time_op)) => {
                    if next_time_tw == next_time_op {
                        // Tentative time output will take care that there is waiting in between
                        self.add_job(i, next_time_tw);
                        job_duration -= next_time_tw.duration();
                        if job_duration == chrono::Duration::zero() {
                            job_completed = true;
                        }
                        current_time = self.time_output.current_time;
                    } else {
                        // Otherwise, we have to wait until the next fitting time window
                        current_time = max(next_time_op.start, next_time_tw.start);
                    }
                }
                (None, Some(next_time_op)) => {
                    // There is no time window left, but for operation times, there is
                    // add job will thereby create lateness
                    self.add_job(i, next_time_op);
                    job_duration -= next_time_op.duration();
                    if job_duration == chrono::Duration::zero() {
                        job_completed = true;
                    }
                    current_time = self.time_output.current_time;
                }
                (_, None) => {
                    // There is no feasible time window for operation time,
                    // this can only happen if the job is too long,
                    // hence we know that the job has to be split
                    must_fit = false;
                    self.add_split();
                    current_time = self.time_output.current_time;
                }
            }
        }
    }
    fn execute_travel(&self, i: usize) {
        // for add travel, we have to take a look at the travel duration between the current location and the next location
        // also, we have to consider the working times as well as te breaks we do after a certain amount of travel time
        let location = self.route.sequence[i - 1];
        let next_location = self.route.sequence[i];
        let travel_duration = self.time_input.duration_matrix[location][next_location];
        let operation_times = self.time_input.operation_times.unwrap(); // TODO there should always be operation times here. If we work 24/7, this should be handled in operation times.
        let mut current_time = self.time_output.current_time;
        let mut remaining_travel_duration = travel_duration;
        // TODO also consider breaks
        while remaining_travel_duration > chrono::Duration::zero() {
            let maybe_next_time_op = operation_times.find_next_fitting_time(
                current_time,
                remaining_travel_duration,
                false,
            );
            match maybe_next_time_op {
                Some(next_time_op) => {
                    self.time_output.add_travel(next_location, next_time_op);
                    remaining_travel_duration -= next_time_op.duration();
                    current_time = self.time_output.current_time;
                }
                None => {
                    // This should never happen, panic
                    panic!("Panic because None was returned by operationtimes.find_nect_fitting_time with must_fit = false");
                }
            }
        }
    }
}

pub struct TimePenalizer {
    time_input: TimeInput,
}

impl TimePenalizer {
    pub fn new(time_input: TimeInput) -> TimePenalizer {
        TimePenalizer { time_input }
    }
    pub fn penalize(&self, route: &Route, build_schedule: bool) -> TimeOutput<Complete> {
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
