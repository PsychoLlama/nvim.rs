//! Environment variable operations
//!
//! Provides portable access to environment variables.
//!
//! Note: Functions that return allocated strings use nvim's `xmallocz`
//! allocator, so callers should free them with `xfree`.

use std::env;
use std::ffi::{c_char, c_int, CStr};
use std::ptr;

use nvim_memory::NvimString;

/// Get the value of an environment variable.
///
/// Returns a newly allocated string (via `xmallocz`) that must be freed
/// with `xfree`, or NULL if the variable is not set or empty.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_getenv(name: *const c_char) -> *mut c_char {
    if name.is_null() {
        return ptr::null_mut();
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    if name_str.is_empty() {
        return ptr::null_mut();
    }

    match env::var(name_str) {
        Ok(value) if !value.is_empty() => {
            // Use NvimString which allocates with xmallocz
            // The string contains no interior NULs (env::var returns valid UTF-8)
            NvimString::new(&value).into_raw()
        }
        _ => ptr::null_mut(),
    }
}

// Note: rs_os_getenv_free is no longer needed since rs_os_getenv now uses
// xmallocz. Callers should use xfree() directly to free returned strings.

/// Check if an environment variable exists.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_env_exists(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    if name_str.is_empty() {
        return 0;
    }

    c_int::from(env::var_os(name_str).is_some())
}

/// Set an environment variable.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `name` and `value` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_os_setenv(
    name: *const c_char,
    value: *const c_char,
    _overwrite: c_int,
) -> c_int {
    if name.is_null() || value.is_null() {
        return -1;
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let value_cstr = unsafe { CStr::from_ptr(value) };

    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let value_str = match value_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    if name_str.is_empty() {
        return -1;
    }

    // Note: std::env::set_var is marked unsafe in newer Rust versions
    // because it's not thread-safe. For now, we proceed with caution.
    env::set_var(name_str, value_str);
    0
}

/// Unset an environment variable.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_unsetenv(name: *const c_char) -> c_int {
    if name.is_null() {
        return -1;
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    if name_str.is_empty() {
        return -1;
    }

    env::remove_var(name_str);
    0
}

/// Get the current process ID.
#[no_mangle]
pub extern "C" fn rs_os_get_pid() -> i64 {
    i64::from(std::process::id())
}

/// Get the hostname.
///
/// Writes the hostname to `buf` (max `size` bytes including NUL).
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `buf` must point to a buffer of at least `size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_os_get_hostname(buf: *mut c_char, size: usize) -> c_int {
    if buf.is_null() || size == 0 {
        return -1;
    }

    #[cfg(unix)]
    {
        let mut hostname_buf = vec![0u8; size];
        let result = unsafe { libc::gethostname(hostname_buf.as_mut_ptr().cast(), size) };

        if result != 0 {
            return -1;
        }

        // Ensure null termination
        hostname_buf[size - 1] = 0;

        // Copy to output buffer
        unsafe {
            ptr::copy_nonoverlapping(hostname_buf.as_ptr(), buf.cast(), size);
        }
        0
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::SystemInformation::ComputerNameDnsHostname;
        use windows_sys::Win32::System::SystemInformation::GetComputerNameExA;

        let mut len = size as u32;
        let result = unsafe { GetComputerNameExA(ComputerNameDnsHostname, buf.cast(), &mut len) };

        if result == 0 {
            -1
        } else {
            0
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        // Fallback: return empty string
        unsafe {
            *buf = 0;
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_get_pid() {
        let pid = rs_os_get_pid();
        assert!(pid > 0);
    }

    #[test]
    fn test_env_exists() {
        // PATH should exist on all systems
        let name = CString::new("PATH").unwrap();
        let exists = unsafe { rs_os_env_exists(name.as_ptr()) };
        assert_eq!(exists, 1);

        // Non-existent variable
        let name = CString::new("NVIM_RS_TEST_NONEXISTENT_VAR_12345").unwrap();
        let exists = unsafe { rs_os_env_exists(name.as_ptr()) };
        assert_eq!(exists, 0);
    }

    #[test]
    fn test_setenv_unsetenv() {
        let name = CString::new("NVIM_RS_TEST_VAR").unwrap();
        let value = CString::new("test_value").unwrap();

        // Set the variable
        let result = unsafe { rs_os_setenv(name.as_ptr(), value.as_ptr(), 1) };
        assert_eq!(result, 0);

        // Check it exists
        let exists = unsafe { rs_os_env_exists(name.as_ptr()) };
        assert_eq!(exists, 1);

        // Unset it
        let result = unsafe { rs_os_unsetenv(name.as_ptr()) };
        assert_eq!(result, 0);

        // Check it's gone
        let exists = unsafe { rs_os_env_exists(name.as_ptr()) };
        assert_eq!(exists, 0);
    }

    // Note: test_getenv is disabled because rs_os_getenv now uses
    // nvim's xmallocz allocator which requires linking with nvim.
    // The function will be tested via nvim's integration tests.
    //
    // #[test]
    // fn test_getenv() {
    //     // This test requires nvim's allocator to be linked
    // }

    #[test]
    fn test_hostname() {
        let mut buf = [0i8; 256];
        let result = unsafe { rs_os_get_hostname(buf.as_mut_ptr(), buf.len()) };
        assert_eq!(result, 0);

        // Hostname should be non-empty on most systems
        let hostname = unsafe { CStr::from_ptr(buf.as_ptr()) };
        // Just verify it's valid UTF-8 and doesn't crash
        let _ = hostname.to_str();
    }
}
