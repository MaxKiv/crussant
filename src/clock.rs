use embassy_time::Instant;
use time::{OffsetDateTime, UtcOffset};

use time::error::ComponentRange as TimeComponentRange;

const AUCKLAND_UTC_OFFSET_HOURS: i8 = 13;

#[derive(Clone, Debug)]
pub struct Clock {
    // boot time in Unix timestamp: seconds from Unix Epoch
    boot_time: u64,

    // time offset
    offset: UtcOffset,
}

impl Clock {
    /// Construct a new [`Clock`] with time defined during compilation
    pub fn new() -> Self {
        // Get current time from compilation, env defined in build.rs
        let boot_time = env!("BUILD_TIME").parse::<u64>().unwrap();
        // Get current offset
        let offset = UtcOffset::from_hms(AUCKLAND_UTC_OFFSET_HOURS, 0, 0).unwrap();
        Clock { boot_time, offset }
    }

    pub fn now(&self) -> Result<OffsetDateTime, Error> {
        let now = self.now_as_unix_timestamp();
        let utc = OffsetDateTime::from_unix_timestamp(now as i64)?;
        let local = utc
            .checked_to_offset(self.offset)
            .ok_or(Error::InvalidInOffset)?;
        Ok(local)
    }

    /// Return current time as a Unix epoch
    pub fn now_as_unix_timestamp(&self) -> u64 {
        let from_boot = Instant::now().as_secs();
        self.boot_time + from_boot
    }

    // TODO: save clock in rtc_fast and retrieve it from there on boot
}

/// A clock error
#[derive(Debug)]
pub enum Error {
    /// A time component is out of range
    TimeComponentRange(#[allow(unused)] TimeComponentRange),

    /// The clocks utc offset is invalid
    InvalidInOffset,
}

impl From<TimeComponentRange> for Error {
    fn from(error: TimeComponentRange) -> Self {
        Self::TimeComponentRange(error)
    }
}
