//! Simple timing wrappers for profiling.
//!
//! These functions wrap `os_hrtime()` to provide timing primitives used
//! throughout the profiling subsystem.

use std::os::raw::{c_char, c_int};

use crate::Proftime;

extern "C" {
    fn os_hrtime() -> u64;
    fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
}

/// Wait time accumulated while waiting for user input.
/// Replaces `static proftime_T prof_wait_time` in profile.c.
static mut PROF_WAIT_TIME: Proftime = 0;

/// Gets the current time (high-resolution monotonic clock).
///
/// # Safety
///
/// Calls external C function `os_hrtime`.
#[export_name = "profile_start"]
pub unsafe extern "C" fn rs_profile_start() -> Proftime {
    os_hrtime()
}

/// Computes the time elapsed from `tm` until now.
///
/// # Safety
///
/// Calls external C function `os_hrtime`.
#[export_name = "profile_end"]
pub unsafe extern "C" fn rs_profile_end(tm: Proftime) -> Proftime {
    crate::rs_profile_sub(os_hrtime(), tm)
}

/// Gets a string representing time `tm`.
///
/// Returns a pointer to a static buffer in the form "seconds.microseconds".
/// Not thread-safe, not reentrant.
///
/// # Safety
///
/// Returns pointer to a static buffer. Caller must not free or use
/// concurrently.
#[export_name = "profile_msg"]
pub unsafe extern "C" fn rs_profile_msg(tm: Proftime) -> *const c_char {
    static mut BUF: [u8; 50] = [0u8; 50];
    let val = crate::rs_profile_signed(tm) as f64 / 1_000_000_000.0;
    let buf_ptr = std::ptr::addr_of_mut!(BUF);
    snprintf(
        (*buf_ptr).as_mut_ptr().cast::<c_char>(),
        (*buf_ptr).len(),
        c"%10.6lf".as_ptr(),
        val,
    );
    (*buf_ptr).as_ptr().cast::<c_char>()
}

/// Gets the time `msec` into the future.
///
/// If msec <= 0, returns the zero time (no limit).
///
/// # Safety
///
/// Calls external C function `os_hrtime`.
#[export_name = "profile_setlimit"]
pub unsafe extern "C" fn rs_profile_setlimit(msec: i64) -> Proftime {
    if msec <= 0 {
        return crate::rs_profile_zero();
    }
    debug_assert!(msec < i64::MAX / 1_000_000);
    let nsec: Proftime = (msec as u64).wrapping_mul(1_000_000);
    os_hrtime() + nsec
}

/// Checks if current time has passed `tm`.
///
/// Returns true if the current time is past `tm`, false if not or if
/// the timer was not set (tm == 0).
///
/// # Safety
///
/// Calls external C function `os_hrtime`.
#[export_name = "profile_passed_limit"]
pub unsafe extern "C" fn rs_profile_passed_limit(tm: Proftime) -> bool {
    if tm == 0 {
        return false;
    }
    crate::rs_profile_cmp(os_hrtime(), tm) < 0
}

/// Sets the current waittime.
///
/// # Safety
///
/// Accesses mutable static. Single-threaded profiling context only.
#[export_name = "profile_set_wait"]
pub unsafe extern "C" fn rs_profile_set_wait(wait: Proftime) {
    let ptr = std::ptr::addr_of_mut!(PROF_WAIT_TIME);
    ptr.write(wait);
}

/// Gets the current waittime.
///
/// # Safety
///
/// Accesses mutable static. Single-threaded profiling context only.
#[no_mangle]
pub unsafe extern "C" fn rs_profile_get_wait_time() -> Proftime {
    let ptr = std::ptr::addr_of!(PROF_WAIT_TIME);
    ptr.read()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prof_wait_time_default() {
        // The static should default to zero
        unsafe {
            std::ptr::addr_of_mut!(PROF_WAIT_TIME).write(0);
            assert_eq!(rs_profile_get_wait_time(), 0);
        }
    }

    #[test]
    fn test_set_get_wait_time() {
        unsafe {
            rs_profile_set_wait(42);
            assert_eq!(rs_profile_get_wait_time(), 42);
            // Reset
            rs_profile_set_wait(0);
        }
    }
}
