//! Time helpers shared by all repositories.

use time::{Duration, OffsetDateTime};

/// Current time in UTC, truncated to whole seconds to match storage precision.
#[must_use]
pub fn now() -> OffsetDateTime {
    let now = OffsetDateTime::now_utc();
    now.replace_nanosecond(0).unwrap_or(now)
}

/// Current time plus `duration`.
#[must_use]
pub fn now_plus(duration: Duration) -> OffsetDateTime {
    now() + duration
}

/// Converts a timestamp to storage representation (UNIX seconds).
#[must_use]
pub fn unix(timestamp: OffsetDateTime) -> i64 {
    timestamp.unix_timestamp()
}

/// Converts a stored UNIX timestamp back to a UTC timestamp.
#[must_use]
pub fn from_unix(seconds: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(seconds).unwrap_or(OffsetDateTime::UNIX_EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_seconds() {
        let stamp = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("valid");
        assert_eq!(from_unix(unix(stamp)), stamp);
    }

    #[test]
    fn out_of_range_falls_back_to_epoch() {
        assert_eq!(from_unix(i64::MAX), OffsetDateTime::UNIX_EPOCH);
    }
}
