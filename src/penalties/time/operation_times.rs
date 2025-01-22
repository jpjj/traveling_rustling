use std::cmp::min;

use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};

use super::time_windows::TimeWindow;

struct WorkingDays {
    next_day_cache: [chrono::Weekday; 7],
}
impl WorkingDays {
    fn new(working_days: Vec<chrono::Weekday>) -> WorkingDays {
        let mut days = [false; 7];
        for day in working_days {
            days[day.num_days_from_monday() as usize] = true;
        }
        let mut next_day_cache = [chrono::Weekday::Mon; 7];
        for i in 0..7 {
            for j in 1..7 {
                let next_day = (i + j) % 7;
                if days[next_day] {
                    next_day_cache[i] = chrono::Weekday::try_from(next_day as u8).unwrap();
                    break;
                }
            }
        }
        WorkingDays { next_day_cache }
    }
    fn next_working_day(&self, date: NaiveDate) -> NaiveDate {
        let next_day = self.next_day_cache[date.weekday().num_days_from_monday() as usize];
        let mut days_to_add = (next_day.num_days_from_monday() as i8
            - date.weekday().num_days_from_monday() as i8)
            .rem_euclid(7) as i64;
        if days_to_add == 0 {
            days_to_add = 7; // because from monday to next monday is 7 days
        }
        date + chrono::Duration::days(days_to_add)
    }
}

pub struct OperationTimes {
    daily_start: chrono::NaiveTime,
    daily_end: chrono::NaiveTime,
    working_days: Option<WorkingDays>,
}

impl OperationTimes {
    pub fn new(
        daily_start: chrono::NaiveTime,
        daily_end: chrono::NaiveTime,
        working_days: Option<Vec<chrono::Weekday>>,
    ) -> OperationTimes {
        assert!(daily_start < daily_end);

        OperationTimes {
            daily_start,
            daily_end,
            working_days: working_days.map(|wd| WorkingDays::new(wd)),
        }
    }

    pub fn duration(&self) -> chrono::Duration {
        self.daily_end.signed_duration_since(self.daily_start)
    }

    pub fn start(&self) -> chrono::NaiveTime {
        self.daily_start
    }

    pub fn end(&self) -> chrono::NaiveTime {
        self.daily_end
    }
    pub fn contains(&self, time: DateTime<Utc>) -> bool {
        let time = time.time();
        self.daily_start <= time && time < self.daily_end
    }
    pub fn waiting_time(&self, current_time: DateTime<Utc>) -> chrono::Duration {
        let time = current_time.time();
        if !self.contains(current_time) {
            // it is before the daily start
            if time < self.daily_start {
                return self.daily_start.signed_duration_since(time);
            }
            // it is after the daily end
            self.start_next_day(current_time) - current_time
        } else {
            chrono::Duration::zero()
        }
    }

    pub fn next_day(&self, current_time: DateTime<Utc>) -> NaiveDate {
        match self.working_days {
            Some(ref working_days) => {
                return working_days.next_working_day(current_time.date_naive());
            }
            None => { current_time + chrono::Duration::days(1) }.date_naive(),
        }
    }

    pub fn start_next_day(&self, current_time: DateTime<Utc>) -> DateTime<Utc> {
        let next_day = self.next_day(current_time);
        Utc.from_utc_datetime(&next_day.and_time(self.daily_start))
    }

    pub fn find_next_fitting_time(
        &self,
        current_time: DateTime<Utc>,
        job_duration: chrono::Duration,
        must_fit: bool,
    ) -> Option<TimeWindow> {
        let waiting_time = self.waiting_time(current_time);
        let start_time = current_time + waiting_time;
        let end_time = min(
            start_time + job_duration,
            // the following unwrap is likely to be safe because the datetime is within the representable range for a DateTime
            start_time.with_time(self.daily_end).unwrap(),
        );
        let result_tw = TimeWindow::new(start_time, end_time);
        match must_fit {
            false => Some(result_tw),
            true => {
                // If the job duration is bigger than the proposed time window,
                // we have to wait until the next day and start fresh.
                // If it still does not fit, we return None.
                if result_tw.duration() == job_duration {
                    return Some(result_tw);
                }
                if job_duration > self.duration() {
                    return None;
                }
                let start_time = self.start_next_day(current_time);
                let end_time = start_time + job_duration;
                return Some(TimeWindow::new(start_time, end_time));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveTime, TimeZone, Utc};

    // Test for checking the general features of the OperationTimes struct
    #[test]
    fn test_operation_times() {
        let operation_times = OperationTimes::new(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            None,
        );
        assert_eq!(
            operation_times.start(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap()
        );
        assert_eq!(
            operation_times.end(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap()
        );
        assert_eq!(operation_times.duration(), chrono::Duration::hours(8));
        assert!(!operation_times.contains(Utc.with_ymd_and_hms(2021, 1, 1, 7, 0, 0).unwrap()));
        assert!(operation_times.contains(Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap()));
        assert!(operation_times.contains(Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap()));
        assert!(!operation_times.contains(Utc.with_ymd_and_hms(2021, 1, 1, 16, 0, 0).unwrap()));
        assert!(!operation_times.contains(Utc.with_ymd_and_hms(2021, 1, 1, 17, 0, 0).unwrap()));
        assert_eq!(
            operation_times.waiting_time(Utc.with_ymd_and_hms(2021, 1, 1, 7, 0, 0).unwrap()),
            chrono::Duration::hours(1)
        );
        assert_eq!(
            operation_times.waiting_time(Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap()),
            chrono::Duration::zero()
        );
        assert_eq!(
            operation_times.waiting_time(Utc.with_ymd_and_hms(2021, 1, 1, 16, 0, 0).unwrap()),
            chrono::Duration::hours(16)
        );
        assert_eq!(
            operation_times.waiting_time(Utc.with_ymd_and_hms(2021, 1, 1, 17, 0, 0).unwrap()),
            chrono::Duration::hours(15)
        );
    }

    // testing find_next_fitting_time with must_fit = false
    #[test]
    fn test_find_next_fitting_time_no_must_fit() {
        let operation_times = OperationTimes::new(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            None,
        );
        let job_duration = chrono::TimeDelta::hours(2);
        let current_time = Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap();
        let result = operation_times.find_next_fitting_time(current_time, job_duration, false);
        assert_eq!(
            result,
            Some(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap()
            ))
        );
        let current_time = Utc.with_ymd_and_hms(2021, 1, 1, 7, 0, 0).unwrap();
        let result = operation_times.find_next_fitting_time(current_time, job_duration, false);
        assert_eq!(
            result,
            Some(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap()
            ))
        );
        let current_time = Utc.with_ymd_and_hms(2021, 1, 1, 15, 0, 0).unwrap();
        let result = operation_times.find_next_fitting_time(current_time, job_duration, false);
        assert_eq!(
            result,
            Some(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 15, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 16, 0, 0).unwrap()
            ))
        );
        let current_time = Utc.with_ymd_and_hms(2021, 1, 1, 16, 0, 0).unwrap();
        let result = operation_times.find_next_fitting_time(current_time, job_duration, false);
        assert_eq!(
            result,
            Some(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 2, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 2, 10, 0, 0).unwrap()
            ))
        );
    }

    // testing find_next_fitting_time with must_fit = true
    #[test]
    fn test_find_next_fitting_time_must_fit() {
        let operation_times = OperationTimes::new(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            None,
        );
        let job_duration = chrono::TimeDelta::hours(2);

        // job too early
        let current_time = Utc.with_ymd_and_hms(2021, 1, 1, 7, 0, 0).unwrap();
        let result = operation_times.find_next_fitting_time(current_time, job_duration, true);
        assert_eq!(
            result,
            Some(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 10, 0, 0).unwrap()
            ))
        );
        // job inside time window
        let current_time = Utc.with_ymd_and_hms(2021, 1, 1, 9, 0, 0).unwrap();
        let result = operation_times.find_next_fitting_time(current_time, job_duration, true);
        assert_eq!(
            result,
            Some(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 1, 9, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 1, 11, 0, 0).unwrap()
            ))
        );
        // because complete job must fit, it will jump to the next day
        let current_time = Utc.with_ymd_and_hms(2021, 1, 1, 15, 0, 0).unwrap();
        let result = operation_times.find_next_fitting_time(current_time, job_duration, true);
        assert_eq!(
            result,
            Some(TimeWindow::new(
                Utc.with_ymd_and_hms(2021, 1, 2, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2021, 1, 2, 10, 0, 0).unwrap()
            ))
        );
        // creating a job that does never fit
        let current_time = Utc.with_ymd_and_hms(2021, 1, 1, 15, 0, 0).unwrap();
        let job_duration = chrono::TimeDelta::hours(9);
        let result = operation_times.find_next_fitting_time(current_time, job_duration, true);
        assert!(result.is_none());
    }
}
