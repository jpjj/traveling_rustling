pub struct OperationTimes {
    daily_start: chrono::NaiveTime,
    daily_end: chrono::NaiveTime,
}

impl OperationTimes {
    pub fn new(daily_start: chrono::NaiveTime, daily_end: chrono::NaiveTime) -> OperationTimes {
        assert!(daily_start < daily_end);
        OperationTimes {
            daily_start,
            daily_end,
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
    pub fn contains(&self, time: chrono::NaiveTime) -> bool {
        self.daily_start <= time && time <= self.daily_end
    }
    pub fn waiting_time(&self, time: chrono::NaiveTime) -> chrono::Duration {
        if !self.contains(time) {
            if time < self.daily_start {
                return self.daily_start.signed_duration_since(time);
            }
            chrono::Duration::days(1) + self.daily_start.signed_duration_since(time)
        } else if time == self.daily_end {
            chrono::Duration::days(1) + self.daily_start.signed_duration_since(time)
        } else {
            chrono::Duration::zero()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveTime, TimeZone, Utc};

    #[test]
    fn test_operation_times() {
        let operation_times = OperationTimes::new(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
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
        assert!(operation_times.contains(NaiveTime::from_hms_opt(12, 0, 0).unwrap()));
        assert!(!operation_times.contains(NaiveTime::from_hms_opt(7, 0, 0).unwrap()));
        assert!(!operation_times.contains(NaiveTime::from_hms_opt(17, 0, 0).unwrap()));
        assert_eq!(
            operation_times.waiting_time(NaiveTime::from_hms_opt(7, 0, 0).unwrap()),
            chrono::Duration::hours(1)
        );
        assert_eq!(
            operation_times.waiting_time(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
            chrono::Duration::zero()
        );
        assert_eq!(
            operation_times.waiting_time(NaiveTime::from_hms_opt(16, 0, 0).unwrap()),
            chrono::Duration::hours(16)
        );
        assert_eq!(
            operation_times.waiting_time(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
            chrono::Duration::hours(15)
        );
    }

    #[test]
    fn test_operation_times_against_utc() {
        let operation_times = OperationTimes::new(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
        );
        let utc_datetime = Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap();
        assert!(operation_times.contains(utc_datetime.time()));
        let utc_datetime = Utc.with_ymd_and_hms(2021, 1, 1, 7, 0, 0).unwrap();
        assert!(!operation_times.contains(utc_datetime.time()));
    }
}
