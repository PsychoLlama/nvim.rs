//! Environment variable operations
//!
//! Provides portable access to environment variables.
//!
//! Note: Functions that return allocated strings use nvim's `xmallocz`
//! allocator, so callers should free them with `xfree`.

use std::env;
use std::ffi::{c_char, c_int, CStr, CString};
use std::ptr;

use nvim_memory::NvimString;

extern "C" {
    #[link_name = "xfree"]
    fn nvim_xfree(ptr: *mut c_char);
    #[link_name = "xstrdup"]
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
}

/// Get the value of an environment variable.
///
/// Returns a newly allocated string (via `xmallocz`) that must be freed
/// with `xfree`, or NULL if the variable is not set or empty.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[export_name = "os_getenv"]
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

/// Check if an environment variable exists (and optionally is non-empty).
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[export_name = "os_env_exists"]
pub unsafe extern "C" fn rs_os_env_exists(name: *const c_char, nonempty: bool) -> bool {
    if name.is_null() {
        return false;
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    if name_str.is_empty() {
        return false;
    }

    if nonempty {
        // Must be set and non-empty
        match env::var(name_str) {
            Ok(v) => !v.is_empty(),
            Err(_) => false,
        }
    } else {
        // Only needs to be set (even if empty)
        env::var_os(name_str).is_some()
    }
}

/// Set an environment variable.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `name` and `value` must be valid null-terminated C strings.
#[export_name = "os_setenv"]
pub unsafe extern "C" fn rs_os_setenv(
    name: *const c_char,
    value: *const c_char,
    overwrite: c_int,
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

    #[cfg(windows)]
    {
        if overwrite == 0 && !unsafe { rs_os_env_exists(name, true) } {
            return 0;
        }
        // Windows (Vim-compat): Empty string undefines the env var.
        if value_str.is_empty() {
            return unsafe { rs_os_unsetenv(name) };
        }
    }

    #[cfg(not(windows))]
    {
        if overwrite == 0 && unsafe { rs_os_env_exists(name, false) } {
            return 0;
        }
    }

    // Note: std::env::set_var is marked unsafe in newer Rust versions
    // because it's not thread-safe. For now, we proceed with caution.
    #[allow(deprecated)]
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
#[export_name = "os_unsetenv"]
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

    #[allow(deprecated)]
    env::remove_var(name_str);
    0
}

/// Get the current process ID.
#[export_name = "os_get_pid"]
pub extern "C" fn rs_os_get_pid() -> i64 {
    i64::from(std::process::id())
}

/// Gets the hostname of the current machine.
///
/// Writes the hostname to `buf` (max `size` bytes including NUL).
/// On failure, sets `buf[0]` to NUL.
///
/// # Safety
///
/// `buf` must point to a buffer of at least `size` bytes.
#[export_name = "os_get_hostname"]
pub unsafe extern "C" fn rs_os_get_hostname(buf: *mut c_char, size: usize) {
    if buf.is_null() || size == 0 {
        return;
    }

    #[cfg(unix)]
    {
        let mut hostname_buf = vec![0u8; size];
        let result = unsafe { libc::gethostname(hostname_buf.as_mut_ptr().cast(), size) };

        if result != 0 {
            unsafe {
                *buf = 0;
            }
            return;
        }

        // Ensure null termination
        hostname_buf[size - 1] = 0;

        // Copy to output buffer
        unsafe {
            ptr::copy_nonoverlapping(hostname_buf.as_ptr(), buf.cast(), size);
        }
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::SystemInformation::ComputerNameDnsHostname;
        use windows_sys::Win32::System::SystemInformation::GetComputerNameExA;

        let mut len = size as u32;
        let result = unsafe { GetComputerNameExA(ComputerNameDnsHostname, buf.cast(), &mut len) };

        if result == 0 {
            unsafe {
                *buf = 0;
            }
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        unsafe {
            *buf = 0;
        }
    }
}

/// Returns number of variables in the current environment variables block.
#[export_name = "os_get_fullenv_size"]
pub extern "C" fn rs_os_get_fullenv_size() -> usize {
    env::vars_os().count()
}

/// Copies the current environment variables into the given array `env`.
/// Each element is of the form "NAME=VALUE", allocated with xstrdup.
///
/// # Safety
///
/// `env` must point to an array of at least `env_size` pointers.
#[export_name = "os_copy_fullenv"]
pub unsafe extern "C" fn rs_os_copy_fullenv(env: *mut *mut c_char, env_size: usize) {
    if env.is_null() || env_size == 0 {
        return;
    }
    let mut i = 0usize;
    for (key, val) in env::vars() {
        if i >= env_size {
            break;
        }
        let entry = format!("{key}={val}");
        if let Ok(cstr) = CString::new(entry) {
            let ptr = unsafe { nvim_xstrdup(cstr.as_ptr()) };
            unsafe {
                *env.add(i) = ptr;
            }
            i += 1;
        }
    }
}

/// Frees an environment array allocated by `os_copy_fullenv`.
///
/// # Safety
///
/// `env` must be a null-terminated array of pointers allocated with xmalloc,
/// and the array itself must also be xmalloc-allocated.
#[export_name = "os_free_fullenv"]
pub unsafe extern "C" fn rs_os_free_fullenv(env: *mut *mut c_char) {
    if env.is_null() {
        return;
    }
    let mut p = env;
    while !unsafe { *p }.is_null() {
        unsafe {
            nvim_xfree(*p);
            *p = ptr::null_mut();
            p = p.add(1);
        }
    }
    unsafe {
        nvim_xfree(env.cast());
    }
}

/// Signals to the OS that Nvim is an application for "interactive work"
/// which should be prioritized similar to a GUI app.
#[export_name = "os_hint_priority"]
pub extern "C" fn rs_os_hint_priority() {
    #[cfg(target_os = "macos")]
    {
        extern "C" {
            fn mach_task_self_() -> u32;
            fn task_policy_set(task: u32, flavor: u32, policy_info: *const i32, count: u32) -> i32;
        }
        // TASK_CATEGORY_POLICY = 1
        // TASK_DEFAULT_APPLICATION = 4 (integer_t)
        let policy: i32 = 4;
        unsafe {
            task_policy_set(mach_task_self_(), 1, &policy, 1);
        }
    }
}

/// Gets the name of the environment variable at `index`.
///
/// Result must be freed by the caller with `xfree`.
///
/// # Safety
///
/// Always safe to call.
#[export_name = "os_getenvname_at_index"]
pub extern "C" fn rs_os_getenvname_at_index(index: usize) -> *mut c_char {
    match env::vars().nth(index) {
        Some((key, _)) => match CString::new(key) {
            Ok(cstr) => unsafe { nvim_xstrdup(cstr.as_ptr()) },
            Err(_) => ptr::null_mut(),
        },
        None => ptr::null_mut(),
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
    fn test_env_exists_nonempty() {
        // PATH should exist and be non-empty on all systems
        let name = CString::new("PATH").unwrap();
        let exists = unsafe { rs_os_env_exists(name.as_ptr(), true) };
        assert!(exists);

        // Non-existent variable
        let name = CString::new("NVIM_RS_TEST_NONEXISTENT_VAR_12345").unwrap();
        let exists = unsafe { rs_os_env_exists(name.as_ptr(), false) };
        assert!(!exists);
    }

    #[test]
    fn test_setenv_unsetenv() {
        let name = CString::new("NVIM_RS_TEST_VAR_PHASE1").unwrap();
        let value = CString::new("test_value").unwrap();

        // Set the variable
        let result = unsafe { rs_os_setenv(name.as_ptr(), value.as_ptr(), 1) };
        assert_eq!(result, 0);

        // Check it exists
        let exists = unsafe { rs_os_env_exists(name.as_ptr(), false) };
        assert!(exists);

        // Unset it
        let result = unsafe { rs_os_unsetenv(name.as_ptr()) };
        assert_eq!(result, 0);

        // Check it's gone
        let exists = unsafe { rs_os_env_exists(name.as_ptr(), false) };
        assert!(!exists);
    }

    // Note: test_getenvname_at_index is disabled because rs_os_getenvname_at_index
    // calls nvim's xstrdup allocator which requires linking with nvim.
    // The function will be tested via nvim's integration tests.

    #[test]
    fn test_fullenv_size() {
        let size = rs_os_get_fullenv_size();
        // There should always be at least one env var (e.g., PATH)
        assert!(size > 0);
    }

    // Note: test_getenv is disabled because rs_os_getenv now uses
    // nvim's xmallocz allocator which requires linking with nvim.
    // The function will be tested via nvim's integration tests.
}
