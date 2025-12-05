//! Time operations
//!
//! Provides portable time functions.

use std::ffi::c_int;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

// Store the program start time for relative timing
static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn get_start_time() -> &'static Instant {
    START_TIME.get_or_init(Instant::now)
}

/// Get a high-resolution, monotonically-increasing time in nanoseconds.
///
/// This is relative to an arbitrary time in the past and is not
/// related to wall-clock time.
#[no_mangle]
pub extern "C" fn rs_os_hrtime() -> u64 {
    get_start_time().elapsed().as_nanos() as u64
}

/// Get the current Unix timestamp in seconds.
///
/// Returns seconds since Unix epoch as unsigned 64-bit integer,
/// matching nvim's `Timestamp` type (`uint64_t`).
#[no_mangle]
pub extern "C" fn rs_os_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Get the current Unix timestamp in milliseconds.
#[no_mangle]
pub extern "C" fn rs_os_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Sleep for the specified number of milliseconds.
///
/// This is a simple blocking sleep - it does not check for interrupts
/// or process events. For nvim's normal delay function, use the C
/// version which integrates with the event loop.
#[no_mangle]
pub extern "C" fn rs_os_sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

/// Represents a broken-down time (like C's struct tm).
#[repr(C)]
pub struct RsTime {
    /// Seconds (0-60)
    pub sec: c_int,
    /// Minutes (0-59)
    pub min: c_int,
    /// Hours (0-23)
    pub hour: c_int,
    /// Day of month (1-31)
    pub mday: c_int,
    /// Month (0-11)
    pub mon: c_int,
    /// Year - 1900
    pub year: c_int,
    /// Day of week (0-6, Sunday = 0)
    pub wday: c_int,
    /// Day of year (0-365)
    pub yday: c_int,
    /// Daylight saving time flag
    pub isdst: c_int,
}

impl Default for RsTime {
    fn default() -> Self {
        Self {
            sec: 0,
            min: 0,
            hour: 0,
            mday: 1,
            mon: 0,
            year: 70,
            wday: 0,
            yday: 0,
            isdst: -1,
        }
    }
}

/// Get the current local time as a broken-down time structure.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `result` must point to a valid `RsTime` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_os_localtime(result: *mut RsTime) -> c_int {
    if result.is_null() {
        return -1;
    }

    #[cfg(unix)]
    {
        let now = rs_os_time();
        let mut tm: libc::tm = std::mem::zeroed();
        let time_t = now as libc::time_t;

        if libc::localtime_r(&time_t, &mut tm).is_null() {
            return -1;
        }

        unsafe {
            (*result).sec = tm.tm_sec;
            (*result).min = tm.tm_min;
            (*result).hour = tm.tm_hour;
            (*result).mday = tm.tm_mday;
            (*result).mon = tm.tm_mon;
            (*result).year = tm.tm_year;
            (*result).wday = tm.tm_wday;
            (*result).yday = tm.tm_yday;
            (*result).isdst = tm.tm_isdst;
        }
        0
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::SYSTEMTIME;
        use windows_sys::Win32::System::Time::GetLocalTime;

        let mut st: SYSTEMTIME = std::mem::zeroed();
        unsafe { GetLocalTime(&mut st) };

        // Calculate day of year (approximate)
        let days_in_months = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
        let yday = days_in_months[(st.wMonth as usize).saturating_sub(1)] + st.wDay as i32 - 1;

        unsafe {
            (*result).sec = st.wSecond as c_int;
            (*result).min = st.wMinute as c_int;
            (*result).hour = st.wHour as c_int;
            (*result).mday = st.wDay as c_int;
            (*result).mon = (st.wMonth as c_int) - 1;
            (*result).year = (st.wYear as c_int) - 1900;
            (*result).wday = st.wDayOfWeek as c_int;
            (*result).yday = yday;
            (*result).isdst = -1; // Unknown
        }
        0
    }

    #[cfg(not(any(unix, windows)))]
    {
        // Fallback: return epoch
        unsafe {
            *result = RsTime::default();
        }
        0
    }
}

/// Convert a timestamp to local time.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `result` must point to a valid `RsTime` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_os_localtime_r(timestamp: i64, result: *mut RsTime) -> c_int {
    if result.is_null() {
        return -1;
    }

    #[cfg(unix)]
    {
        let mut tm: libc::tm = std::mem::zeroed();
        let time_t = timestamp as libc::time_t;

        if libc::localtime_r(&time_t, &mut tm).is_null() {
            return -1;
        }

        unsafe {
            (*result).sec = tm.tm_sec;
            (*result).min = tm.tm_min;
            (*result).hour = tm.tm_hour;
            (*result).mday = tm.tm_mday;
            (*result).mon = tm.tm_mon;
            (*result).year = tm.tm_year;
            (*result).wday = tm.tm_wday;
            (*result).yday = tm.tm_yday;
            (*result).isdst = tm.tm_isdst;
        }
        0
    }

    #[cfg(not(unix))]
    {
        // For non-Unix, fall back to current time
        rs_os_localtime(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hrtime() {
        let t1 = rs_os_hrtime();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = rs_os_hrtime();
        assert!(t2 > t1);
        // Should have elapsed at least 10ms = 10_000_000 ns
        assert!(t2 - t1 >= 10_000_000);
    }

    #[test]
    fn test_time() {
        let t = rs_os_time();
        // Should be a reasonable timestamp (after 2020)
        assert!(t > 1_577_836_800); // 2020-01-01
    }

    #[test]
    fn test_time_ms() {
        let t = rs_os_time_ms();
        // Should be a reasonable timestamp in ms
        assert!(t > 1_577_836_800_000); // 2020-01-01 in ms
    }

    #[test]
    fn test_localtime() {
        let mut tm = RsTime::default();
        let result = unsafe { rs_os_localtime(&mut tm) };
        assert_eq!(result, 0);

        // Basic sanity checks
        assert!(tm.sec >= 0 && tm.sec <= 60);
        assert!(tm.min >= 0 && tm.min <= 59);
        assert!(tm.hour >= 0 && tm.hour <= 23);
        assert!(tm.mday >= 1 && tm.mday <= 31);
        assert!(tm.mon >= 0 && tm.mon <= 11);
        assert!(tm.year >= 120); // At least year 2020 (120 = 2020 - 1900)
    }

    #[test]
    fn test_localtime_r() {
        let timestamp: i64 = 1_700_000_000; // Nov 14, 2023
        let mut tm = RsTime::default();
        let result = unsafe { rs_os_localtime_r(timestamp, &mut tm) };
        assert_eq!(result, 0);

        // Year should be 2023
        assert_eq!(tm.year, 123); // 2023 - 1900
    }
}
