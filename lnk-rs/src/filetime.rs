use std::fmt;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

/// The FILETIME structure is a 64-bit value that represents the number of
/// 100-nanosecond intervals that have elapsed since January 1, 1601,
/// Coordinated Universal Time (UTC).
#[derive(Clone, Copy)]
pub struct FileTime {
    low_date_time: u32,
    high_date_time: u32,
}

impl fmt::Debug for FileTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.datetime())
    }
}

impl FileTime {
    fn epoch() -> NaiveDateTime {
        let epoch_date = NaiveDate::from_ymd_opt(1601, 1, 1).unwrap();
        let epoch_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        NaiveDateTime::new(epoch_date, epoch_time)
    }

    /// Convert the `FileTime` object to a [[]]
    pub fn datetime(&self) -> NaiveDateTime {
        let hundred_nanos_after_epoch: u64 = Self::into(*self);
        Self::epoch() + Duration::microseconds((hundred_nanos_after_epoch as f64 / 10f64) as i64)
    }

    /// Create a new `FileTime` object representing now.
    pub fn now() -> Self {
        Self::from(chrono::Local::now().naive_local())
    }
}

impl From<NaiveDateTime> for FileTime {
    fn from(value: NaiveDateTime) -> Self {
        let duration = value - Self::epoch();
        let val = duration.num_microseconds().unwrap() * 10;
        let val = val as u64;
        Self::from(val)
    }
}

impl From<u64> for FileTime {
    fn from(value: u64) -> Self {
        let low_date_time = (value & 0xFFFF_FFFF) as u32;
        let high_date_time = ((value >> 32) & 0xFFFF_FFFF) as u32;
        Self {
            low_date_time,
            high_date_time,
        }
    }
}

impl Into<u64> for FileTime {
    fn into(self) -> u64 {
        u64::from(self.low_date_time) + (u64::from(self.high_date_time) << 32)
    }
}
