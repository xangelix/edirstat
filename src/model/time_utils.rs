/// Common international date/time display formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeFormat {
    /// ISO 8601 — `2024-06-13 12:04:05`  (international default)
    #[default]
    Iso8601,
    /// ISO 8601 with T separator — `2024-06-13T12:04:05`
    Iso8601T,
    /// European (day-first) — `13/06/2024 12:04:05`
    EuropeanSlash,
    /// European (day-first, dots) — `13.06.2024 12:04:05`
    EuropeanDot,
    /// US (month-first) — `06/13/2024 12:04:05 PM`
    UsSlash,
    /// Short year, European — `13/06/24 12:04`
    EuropeanShort,
    /// Year-month-day, dots — `2024.06.13 12:04:05`
    DotSeparated,
    /// Locale-friendly long — `13 Jun 2024 12:04:05`
    LongMonthName,
    /// Unix timestamp (seconds) — `1718273045`
    UnixTimestamp,
    /// Date only, ISO — `2024-06-13`
    DateOnly,
}

impl TimeFormat {
    /// All variants in display order
    pub const ALL: &'static [Self] = &[
        Self::Iso8601,
        Self::Iso8601T,
        Self::EuropeanSlash,
        Self::EuropeanDot,
        Self::UsSlash,
        Self::EuropeanShort,
        Self::DotSeparated,
        Self::LongMonthName,
        Self::UnixTimestamp,
        Self::DateOnly,
    ];

    /// Human-readable menu label for each format.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Iso8601 => "YYYY-MM-DD HH:MM:SS",
            Self::Iso8601T => "YYYY-MM-DDTHH:MM:SS",
            Self::EuropeanSlash => "DD/MM/YYYY HH:MM:SS",
            Self::EuropeanDot => "DD.MM.YYYY HH:MM:SS",
            Self::UsSlash => "MM/DD/YYYY HH:MM:SS AM/PM",
            Self::EuropeanShort => "DD/MM/YY HH:MM",
            Self::DotSeparated => "YYYY.MM.DD HH:MM:SS",
            Self::LongMonthName => "DD Mon YYYY HH:MM:SS",
            Self::UnixTimestamp => "Unix Timestamp",
            Self::DateOnly => "YYYY-MM-DD",
        }
    }
}

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Translates Unix Epoch seconds to a date/time string using the given `TimeFormat`.
///
/// Returns `"Unknown"` for timestamps that cannot be represented (≤ 0 or beyond year 9999).
#[must_use]
pub fn format_epoch(epoch_secs: i64, fmt: TimeFormat) -> String {
    if epoch_secs <= 0 || epoch_secs > 253_402_300_799 {
        return match fmt {
            TimeFormat::UnixTimestamp => epoch_secs.to_string(),
            TimeFormat::DateOnly => "Pre-1970".to_string(),
            _ => "Unknown".to_string(),
        };
    }

    // Special-case: Unix timestamp needs no calendar decomposition.
    if fmt == TimeFormat::UnixTimestamp {
        return epoch_secs.to_string();
    }

    let days = epoch_secs / 86400;
    let secs_in_day = epoch_secs % 86400;

    let mut year = 1970i64;
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

    let mut month: i64 = 1;
    let mut day = days_left + 1;
    for &days_in_m in &month_days {
        if day <= days_in_m {
            break;
        }
        day -= days_in_m;
        month += 1;
    }

    let hour = secs_in_day / 3600;
    let minute = (secs_in_day % 3600) / 60;
    let second = secs_in_day % 60;

    let year_short = year % 100;
    let month_name = MONTH_NAMES[(month - 1) as usize];

    match fmt {
        TimeFormat::Iso8601 => {
            format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
        }
        TimeFormat::Iso8601T => {
            format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}")
        }
        TimeFormat::EuropeanSlash => {
            format!("{day:02}/{month:02}/{year:04} {hour:02}:{minute:02}:{second:02}")
        }
        TimeFormat::EuropeanDot => {
            format!("{day:02}.{month:02}.{year:04} {hour:02}:{minute:02}:{second:02}")
        }
        TimeFormat::UsSlash => {
            let (display_hour, am_pm) = if hour == 0 {
                (12, "AM")
            } else if hour < 12 {
                (hour, "AM")
            } else if hour == 12 {
                (12, "PM")
            } else {
                (hour - 12, "PM")
            };
            format!(
                "{month:02}/{day:02}/{year:04} {display_hour:02}:{minute:02}:{second:02} {am_pm}"
            )
        }
        TimeFormat::EuropeanShort => {
            format!("{day:02}/{month:02}/{year_short:02} {hour:02}:{minute:02}")
        }
        TimeFormat::DotSeparated => {
            format!("{year:04}.{month:02}.{day:02} {hour:02}:{minute:02}:{second:02}")
        }
        TimeFormat::LongMonthName => {
            format!("{day:02} {month_name} {year:04} {hour:02}:{minute:02}:{second:02}")
        }
        TimeFormat::UnixTimestamp => unreachable!("handled above"),
        TimeFormat::DateOnly => {
            format!("{year:04}-{month:02}-{day:02}")
        }
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
        assert_eq!(format_epoch(0, TimeFormat::DateOnly), "Pre-1970");
        assert_eq!(format_epoch(0, TimeFormat::Iso8601), "Unknown");
        assert_eq!(format_epoch(-50, TimeFormat::DateOnly), "Pre-1970");
        assert_eq!(format_epoch(-50, TimeFormat::Iso8601), "Unknown");
    }

    #[test]
    fn test_format_epoch_post_maximum() {
        assert_eq!(
            format_epoch(253_402_300_800, TimeFormat::DateOnly),
            "Pre-1970"
        );
        assert_eq!(
            format_epoch(253_402_300_800, TimeFormat::Iso8601),
            "Unknown"
        );
    }

    #[test]
    fn test_format_epoch_standard_iso() {
        assert_eq!(
            format_epoch(1_686_614_400, TimeFormat::DateOnly),
            "2023-06-13"
        );
        assert_eq!(
            format_epoch(1_686_657_845, TimeFormat::Iso8601),
            "2023-06-13 12:04:05"
        );
    }

    #[test]
    fn test_format_epoch_leap_year() {
        assert_eq!(
            format_epoch(1_582_977_600, TimeFormat::Iso8601),
            "2020-02-29 12:00:00"
        );
    }

    #[test]
    fn test_format_epoch_non_leap_year() {
        assert_eq!(
            format_epoch(1_614_513_600, TimeFormat::Iso8601),
            "2021-02-28 12:00:00"
        );
    }

    #[test]
    fn test_format_epoch_european() {
        assert_eq!(
            format_epoch(1_686_657_845, TimeFormat::EuropeanSlash),
            "13/06/2023 12:04:05"
        );
        assert_eq!(
            format_epoch(1_686_657_845, TimeFormat::EuropeanDot),
            "13.06.2023 12:04:05"
        );
    }

    #[test]
    fn test_format_epoch_us() {
        assert_eq!(
            format_epoch(1_686_657_845, TimeFormat::UsSlash),
            "06/13/2023 12:04:05 PM"
        );
    }

    #[test]
    fn test_format_epoch_unix_timestamp() {
        assert_eq!(
            format_epoch(1_686_657_845, TimeFormat::UnixTimestamp),
            "1686657845"
        );
        // Even invalid timestamps are passed through for Unix format
        assert_eq!(format_epoch(0, TimeFormat::UnixTimestamp), "0");
    }

    #[test]
    fn test_format_epoch_long_month_name() {
        assert_eq!(
            format_epoch(1_686_657_845, TimeFormat::LongMonthName),
            "13 Jun 2023 12:04:05"
        );
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
