/// Translates Unix Epoch seconds directly to a date/time Gregorian calendar string.
#[must_use]
pub fn format_epoch(epoch_secs: i64, include_time: bool) -> String {
    if epoch_secs <= 0 || epoch_secs > 253_402_300_799 {
        return if include_time {
            "Unknown".to_string()
        } else {
            "Pre-1970".to_string()
        };
    }
    let days = epoch_secs / 86400;
    let secs_in_day = epoch_secs % 86400;

    let mut year = 1970;
    let mut days_left = days;

    loop {
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if is_leap { 366 } else { 365 };
        if days_left < days_in_year {
            break;
        }
        days_left -= days_in_year;
        year += 1;
    }

    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_days = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    let mut day = days_left + 1;
    for &days_in_m in &month_days {
        if day <= days_in_m {
            break;
        }
        day -= days_in_m;
        month += 1;
    }

    if include_time {
        let hour = secs_in_day / 3600;
        let minute = (secs_in_day % 3600) / 60;
        let second = secs_in_day % 60;
        format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
    } else {
        format!("{year:04}-{month:02}-{day:02}")
    }
}

/// Safely translates `SystemTime` to seconds since Unix Epoch, maintaining signs for pre-1970 dates.
#[must_use]
pub fn system_time_to_unix_timestamp(t: std::time::SystemTime) -> i64 {
    match t.duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as i64,
        Err(err) => {
            let neg_duration = err.duration();
            -(neg_duration.as_secs() as i64)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::*;

    #[test]
    fn test_format_epoch_pre_1970() {
        assert_eq!(format_epoch(0, false), "Pre-1970");
        assert_eq!(format_epoch(0, true), "Unknown");
        assert_eq!(format_epoch(-50, false), "Pre-1970");
        assert_eq!(format_epoch(-50, true), "Unknown");
    }

    #[test]
    fn test_format_epoch_post_maximum() {
        assert_eq!(format_epoch(253_402_300_800, false), "Pre-1970");
        assert_eq!(format_epoch(253_402_300_800, true), "Unknown");
    }

    #[test]
    fn test_format_epoch_standard_no_time() {
        assert_eq!(format_epoch(1_686_614_400, false), "2023-06-13");
    }

    #[test]
    fn test_format_epoch_standard_with_time() {
        assert_eq!(format_epoch(1_686_657_845, true), "2023-06-13 12:04:05");
    }

    #[test]
    fn test_format_epoch_leap_year() {
        assert_eq!(format_epoch(1_582_977_600, true), "2020-02-29 12:00:00");
    }

    #[test]
    fn test_format_epoch_non_leap_year() {
        assert_eq!(format_epoch(1_614_513_600, true), "2021-02-28 12:00:00");
    }

    #[test]
    fn test_system_time_to_unix_timestamp_epoch() {
        let t = SystemTime::UNIX_EPOCH;
        assert_eq!(system_time_to_unix_timestamp(t), 0);
    }

    #[test]
    fn test_system_time_to_unix_timestamp_future() {
        let t = SystemTime::UNIX_EPOCH + Duration::from_secs(123_456_789);
        assert_eq!(system_time_to_unix_timestamp(t), 123_456_789);
    }

    #[test]
    fn test_system_time_to_unix_timestamp_past() {
        let t = SystemTime::UNIX_EPOCH - Duration::from_secs(98765);
        assert_eq!(system_time_to_unix_timestamp(t), -98765);
    }
}
