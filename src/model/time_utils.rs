use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// A generic time format wrapper holding a serialized `strftime` representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeFormat(pub String);

impl Default for TimeFormat {
    fn default() -> Self {
        Self(CommonTimeFormat::Iso8601.as_str().to_string())
    }
}

/// Common international date/time display formats for UI selection.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CommonTimeFormat {
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

impl CommonTimeFormat {
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

    /// `strftime` compatible representation of the format.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Iso8601 => "%Y-%m-%d %H:%M:%S",
            Self::Iso8601T => "%Y-%m-%dT%H:%M:%S",
            Self::EuropeanSlash => "%d/%m/%Y %H:%M:%S",
            Self::EuropeanDot => "%d.%m.%Y %H:%M:%S",
            Self::UsSlash => "%m/%d/%Y %I:%M:%S %p",
            Self::EuropeanShort => "%d/%m/%y %H:%M",
            Self::DotSeparated => "%Y.%m.%d %H:%M:%S",
            Self::LongMonthName => "%d %b %Y %H:%M:%S",
            Self::UnixTimestamp => "%s",
            Self::DateOnly => "%Y-%m-%d",
        }
    }
}

/// Translates Unix Epoch seconds to a date/time string using the given `TimeFormat`.
///
/// Returns `"Unknown"` for timestamps that cannot be represented (≤ 0 or beyond year 9999).
#[must_use]
pub fn format_epoch(epoch_secs: i64, fmt: &TimeFormat) -> String {
    if epoch_secs <= 0 || epoch_secs > 253_402_300_799 {
        if fmt.0 == CommonTimeFormat::UnixTimestamp.as_str() {
            return epoch_secs.to_string();
        } else if fmt.0 == CommonTimeFormat::DateOnly.as_str() {
            return "Pre-1970".to_string();
        }
        return "Unknown".to_string();
    }

    // Special-case: Unix timestamp needs no calendar decomposition.
    if fmt.0 == CommonTimeFormat::UnixTimestamp.as_str() {
        return epoch_secs.to_string();
    }

    Utc.timestamp_opt(epoch_secs, 0)
        .single()
        .map_or_else(|| "Unknown".to_string(), |dt| dt.format(&fmt.0).to_string())
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

    fn tf(fmt: CommonTimeFormat) -> TimeFormat {
        TimeFormat(fmt.as_str().to_string())
    }

    #[test]
    fn test_format_epoch_pre_1970() {
        assert_eq!(format_epoch(0, &tf(CommonTimeFormat::DateOnly)), "Pre-1970");
        assert_eq!(format_epoch(0, &tf(CommonTimeFormat::Iso8601)), "Unknown");
        assert_eq!(
            format_epoch(-50, &tf(CommonTimeFormat::DateOnly)),
            "Pre-1970"
        );
        assert_eq!(format_epoch(-50, &tf(CommonTimeFormat::Iso8601)), "Unknown");
    }

    #[test]
    fn test_format_epoch_post_maximum() {
        assert_eq!(
            format_epoch(253_402_300_800, &tf(CommonTimeFormat::DateOnly)),
            "Pre-1970"
        );
        assert_eq!(
            format_epoch(253_402_300_800, &tf(CommonTimeFormat::Iso8601)),
            "Unknown"
        );
    }

    #[test]
    fn test_format_epoch_standard_iso() {
        assert_eq!(
            format_epoch(1_686_614_400, &tf(CommonTimeFormat::DateOnly)),
            "2023-06-13"
        );
        assert_eq!(
            format_epoch(1_686_657_845, &tf(CommonTimeFormat::Iso8601)),
            "2023-06-13 12:04:05"
        );
    }

    #[test]
    fn test_format_epoch_leap_year() {
        assert_eq!(
            format_epoch(1_582_977_600, &tf(CommonTimeFormat::Iso8601)),
            "2020-02-29 12:00:00"
        );
    }

    #[test]
    fn test_format_epoch_non_leap_year() {
        assert_eq!(
            format_epoch(1_614_513_600, &tf(CommonTimeFormat::Iso8601)),
            "2021-02-28 12:00:00"
        );
    }

    #[test]
    fn test_format_epoch_european() {
        assert_eq!(
            format_epoch(1_686_657_845, &tf(CommonTimeFormat::EuropeanSlash)),
            "13/06/2023 12:04:05"
        );
        assert_eq!(
            format_epoch(1_686_657_845, &tf(CommonTimeFormat::EuropeanDot)),
            "13.06.2023 12:04:05"
        );
    }

    #[test]
    fn test_format_epoch_us() {
        assert_eq!(
            format_epoch(1_686_657_845, &tf(CommonTimeFormat::UsSlash)),
            "06/13/2023 12:04:05 PM"
        );
    }

    #[test]
    fn test_format_epoch_unix_timestamp() {
        assert_eq!(
            format_epoch(1_686_657_845, &tf(CommonTimeFormat::UnixTimestamp)),
            "1686657845"
        );
        assert_eq!(format_epoch(0, &tf(CommonTimeFormat::UnixTimestamp)), "0");
    }

    #[test]
    fn test_format_epoch_long_month_name() {
        assert_eq!(
            format_epoch(1_686_657_845, &tf(CommonTimeFormat::LongMonthName)),
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
