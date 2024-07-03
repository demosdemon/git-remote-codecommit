use std::time::SystemTime;

const SIGV4_DATE: &str = "%Y%m%d";
const SIGV4_TIMESTAMP: &str = "%Y%m%dT%H%M%S";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeWithFormat<'a> {
    time: SystemTime,
    format: &'a str,
}

impl core::fmt::Display for TimeWithFormat<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        chrono::DateTime::<chrono::Utc>::from(self.time)
            .format(self.format)
            .fmt(f)
    }
}

pub trait TimestampExt {
    fn sigv4_date(self) -> TimeWithFormat<'static>;

    fn sigv4_timestamp(self) -> TimeWithFormat<'static>;
}

impl TimestampExt for SystemTime {
    fn sigv4_date(self) -> TimeWithFormat<'static> {
        TimeWithFormat {
            time: self,
            format: SIGV4_DATE,
        }
    }

    fn sigv4_timestamp(self) -> TimeWithFormat<'static> {
        TimeWithFormat {
            time: self,
            format: SIGV4_TIMESTAMP,
        }
    }
}
