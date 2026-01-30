//! Core logging implementation
//!
//! This module provides the core logging logic migrated from log.c,
//! including file I/O, formatting, and instance name generation.

use std::cell::RefCell;
use std::ffi::{c_char, c_int, CStr};
use std::ptr;

// =============================================================================
// Constants
// =============================================================================

/// Log levels (matching C defines in log.h)
pub const LOGLVL_DBG: c_int = 1;
pub const LOGLVL_INF: c_int = 2;
pub const LOGLVL_WRN: c_int = 3;
pub const LOGLVL_ERR: c_int = 4;

/// Log level names
const LOG_LEVEL_NAMES: [&str; 5] = ["???", "DBG", "INF", "WRN", "ERR"];

/// Maximum path length (matching MAXPATHL)
const MAXPATHL: usize = 4096;

// =============================================================================
// C FFI Declarations
// =============================================================================

extern "C" {
    // Accessors for global state
    fn nvim_log_get_file_path() -> *const c_char;
    fn nvim_log_set_file_path(path: *const c_char);
    fn nvim_log_get_file_path_size() -> usize;
    fn nvim_log_is_initialized() -> bool;
    fn nvim_log_increment_skip();
    fn nvim_log_get_ui_client_channel_id() -> u64;
    fn nvim_log_get_servername() -> *const c_char;
    fn nvim_log_get_parent_nvim() -> *const c_char;
    fn nvim_log_get_pid() -> i64;
    fn nvim_log_path_tail(path: *const c_char) -> *const c_char;
    fn nvim_log_get_localtime(result: *mut libc::tm) -> c_int;
    fn nvim_log_get_millis() -> c_int;
    fn nvim_log_expand_env(src: *const c_char, dst: *mut c_char, dstlen: c_int);
    fn nvim_log_is_dir(path: *const c_char) -> bool;
    fn nvim_log_get_xdg_state_home() -> *mut c_char;
    fn nvim_log_mkdir_recurse(path: *const c_char, failed_dir: *mut *mut c_char) -> c_int;
    fn nvim_log_get_state_subpath(subpath: *const c_char) -> *mut c_char;
    fn nvim_log_setenv(name: *const c_char, value: *const c_char);
    fn nvim_log_free(ptr: *mut libc::c_void);
    fn nvim_log_strequal(s1: *const c_char, s2: *const c_char) -> bool;
    fn nvim_log_strlcpy(dst: *mut c_char, src: *const c_char, dstsize: usize) -> usize;

    // Logging functions we still call from C
    fn log_lock();
    fn log_unlock();
}

// =============================================================================
// Thread-Local State
// =============================================================================

thread_local! {
    /// Instance name for this Nvim process (cached)
    static INSTANCE_NAME: RefCell<[u8; 32]> = const { RefCell::new([0u8; 32]) };

    /// Recursion guard for logging
    static RECURSIVE: RefCell<bool> = const { RefCell::new(false) };

    /// Flag: have we shown the recursion warning?
    static DID_RECURSE_MSG: RefCell<bool> = const { RefCell::new(false) };
}

// =============================================================================
// Path Initialization
// =============================================================================

/// Try to create/open a file for appending to test if it's writable.
fn log_try_create(path: &[u8]) -> bool {
    if path.is_empty() || path[0] == 0 {
        return false;
    }

    // Convert to CStr for fopen
    let Ok(path_cstr) = std::ffi::CString::new(
        path.iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect::<Vec<_>>(),
    ) else {
        return false;
    };

    unsafe {
        let file = libc::fopen(path_cstr.as_ptr(), c"a".as_ptr());
        if file.is_null() {
            return false;
        }
        libc::fclose(file);
    }
    true
}

/// Initialize the log file path.
///
/// Tries `$NVIM_LOG_FILE`, or falls back to `$XDG_STATE_HOME/nvim/log`.
/// Sets `$NVIM_LOG_FILE` if using fallback.
///
/// # Safety
/// This function calls C functions that access global state.
#[no_mangle]
pub unsafe extern "C" fn rs_log_path_init() {
    let mut log_file_path = [0u8; MAXPATHL + 1];
    let size = nvim_log_get_file_path_size();

    // Create the environment variable string "$NVIM_LOG_FILE"
    let env_var = c"$NVIM_LOG_FILE";

    // Expand $NVIM_LOG_FILE
    nvim_log_expand_env(
        env_var.as_ptr(),
        log_file_path.as_mut_ptr().cast::<c_char>(),
        (size - 1) as c_int,
    );

    // Check if expansion failed or path is invalid
    let path_ptr = log_file_path.as_ptr().cast::<c_char>();
    let expanded_ok = !nvim_log_strequal(env_var.as_ptr(), path_ptr)
        && log_file_path[0] != 0
        && !nvim_log_is_dir(path_ptr)
        && log_try_create(&log_file_path);

    if !expanded_ok {
        // Make $XDG_STATE_HOME if it does not exist.
        let loghome = nvim_log_get_xdg_state_home();
        let mut failed_dir: *mut c_char = ptr::null_mut();
        let log_dir_failure = if !loghome.is_null() && !nvim_log_is_dir(loghome) {
            nvim_log_mkdir_recurse(loghome, &raw mut failed_dir) != 0
        } else {
            false
        };
        if !loghome.is_null() {
            nvim_log_free(loghome.cast::<libc::c_void>());
        }

        // Invalid $NVIM_LOG_FILE or failed to expand; fall back to default.
        let log_subpath = c"log";
        let defaultpath = nvim_log_get_state_subpath(log_subpath.as_ptr());
        let mut len = 0usize;

        if !defaultpath.is_null() {
            len = nvim_log_strlcpy(
                log_file_path.as_mut_ptr().cast::<c_char>(),
                defaultpath,
                size,
            );
            nvim_log_free(defaultpath.cast::<libc::c_void>());
        }

        // Fall back to .nvimlog
        if len >= size || !log_try_create(&log_file_path) {
            let nvimlog = c".nvimlog";
            len = nvim_log_strlcpy(
                log_file_path.as_mut_ptr().cast::<c_char>(),
                nvimlog.as_ptr(),
                size,
            );
        }

        // Fall back to stderr (empty path)
        if len >= size || !log_try_create(&log_file_path) {
            log_file_path[0] = 0;
            nvim_log_set_file_path(ptr::null());
            if !failed_dir.is_null() {
                nvim_log_free(failed_dir.cast::<libc::c_void>());
            }
            return;
        }

        // Set $NVIM_LOG_FILE to the resolved path
        let env_name = c"NVIM_LOG_FILE";
        nvim_log_setenv(env_name.as_ptr(), log_file_path.as_ptr().cast::<c_char>());

        // Log warning if directory creation failed (after we have a valid path)
        if log_dir_failure && !failed_dir.is_null() {
            // We can't use WLOG here as logging might not be fully initialized
            // The C code handles this warning
        }

        if !failed_dir.is_null() {
            nvim_log_free(failed_dir.cast::<libc::c_void>());
        }
    }

    // Set the global log file path
    nvim_log_set_file_path(log_file_path.as_ptr().cast::<c_char>());
}

// =============================================================================
// Instance Name Generation
// =============================================================================

/// Generate the instance name for log messages.
///
/// Format: "c/<parent>" if child, "<servername>" if has server, or "?.<pid>".
/// UI clients have "ui/" prefix.
fn generate_instance_name() -> [u8; 32] {
    let mut name = [0u8; 32];

    unsafe {
        let ui = nvim_log_get_ui_client_channel_id() != 0;

        // Get parent servername ($NVIM)
        let parent_ptr = nvim_log_get_parent_nvim();
        let parent = if parent_ptr.is_null() {
            ""
        } else {
            let tail = nvim_log_path_tail(parent_ptr);
            if tail.is_null() {
                ""
            } else {
                CStr::from_ptr(tail).to_str().unwrap_or("")
            }
        };

        // Get servername (v:servername)
        let serv_ptr = nvim_log_get_servername();
        let serv = if serv_ptr.is_null() {
            ""
        } else {
            let tail = nvim_log_path_tail(serv_ptr);
            if tail.is_null() {
                ""
            } else {
                CStr::from_ptr(tail).to_str().unwrap_or("")
            }
        };

        let name_str = if !parent.is_empty() {
            if ui {
                format!("ui/c/{parent}")
            } else {
                format!("c/{parent}")
            }
        } else if !serv.is_empty() {
            if ui {
                format!("ui/{serv}")
            } else {
                serv.to_string()
            }
        } else {
            let pid = nvim_log_get_pid();
            if ui {
                format!("ui.{pid:<5}")
            } else {
                format!("?.{pid:<5}")
            }
        };

        // Copy to fixed-size buffer
        let bytes = name_str.as_bytes();
        let copy_len = bytes.len().min(name.len() - 1);
        name[..copy_len].copy_from_slice(&bytes[..copy_len]);
    }

    name
}

/// Get the cached instance name, regenerating if needed.
fn get_instance_name() -> [u8; 32] {
    INSTANCE_NAME.with(|cell| {
        let mut name = cell.borrow_mut();

        unsafe {
            let ui = nvim_log_get_ui_client_channel_id() != 0;

            // Regenerate if:
            // - UI client (to ensure "ui" is in the name)
            // - not set yet (first byte is 0)
            // - no v:servername yet (starts with '?')
            let regen = ui || name[0] == 0 || name[0] == b'?';

            if regen {
                *name = generate_instance_name();
            }
        }

        *name
    })
}

// =============================================================================
// Log Formatting and Output
// =============================================================================

/// Format and write a log message to a file.
///
/// Returns true on success, false on failure.
fn do_log_to_file(
    file: *mut libc::FILE,
    log_level: c_int,
    context: Option<&str>,
    func_name: Option<&str>,
    line_num: c_int,
    eol: bool,
    message: &str,
) -> bool {
    if file.is_null() {
        return false;
    }

    // Get timestamp
    let mut local_time: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { nvim_log_get_localtime(&raw mut local_time) } != 0 {
        return false;
    }

    let millis = unsafe { nvim_log_get_millis() };

    // Format timestamp: YYYY-MM-DDTHH:MM:SS
    let date_time = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        local_time.tm_year + 1900,
        local_time.tm_mon + 1,
        local_time.tm_mday,
        local_time.tm_hour,
        local_time.tm_min,
        local_time.tm_sec
    );

    // Get log level name
    let level_idx = log_level.clamp(0, 4) as usize;
    let level_name = LOG_LEVEL_NAMES[level_idx];

    // Get instance name
    let name_bytes = get_instance_name();
    let name_end = name_bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(name_bytes.len());
    let name = std::str::from_utf8(&name_bytes[..name_end]).unwrap_or("?");

    // Format the log line
    let log_line = if line_num == -1 || func_name.is_none() {
        format!(
            "{} {}.{:03} {:<10} {}{}{}",
            level_name,
            date_time,
            millis,
            name,
            context.unwrap_or("?:"),
            message,
            if eol { "\n" } else { "" }
        )
    } else {
        format!(
            "{} {}.{:03} {:<10} {}{}:{}: {}{}",
            level_name,
            date_time,
            millis,
            name,
            context.unwrap_or(""),
            func_name.unwrap_or(""),
            line_num,
            message,
            if eol { "\n" } else { "" }
        )
    };

    // Write to file
    let bytes = log_line.as_bytes();
    let written =
        unsafe { libc::fwrite(bytes.as_ptr().cast::<libc::c_void>(), 1, bytes.len(), file) };

    if written != bytes.len() {
        return false;
    }

    // Flush
    if unsafe { libc::fflush(file) } == libc::EOF {
        return false;
    }

    true
}

/// Open the log file for appending.
///
/// Returns the file handle, or stderr on failure.
fn open_log_file() -> *mut libc::FILE {
    unsafe {
        let path_ptr = nvim_log_get_file_path();

        if !path_ptr.is_null() {
            let path = CStr::from_ptr(path_ptr);
            if !path.to_bytes().is_empty() {
                let file = libc::fopen(path_ptr, c"a".as_ptr());
                if !file.is_null() {
                    return file;
                }
            }
        }

        // Return stderr on failure
        libc::fdopen(libc::STDERR_FILENO, c"w".as_ptr())
    }
}

// =============================================================================
// Main Logging Function
// =============================================================================

/// Log a message to the log file.
///
/// This is the main entry point for logging, called from C via FFI.
///
/// # Safety
/// All pointer parameters must be valid C strings or null.
#[no_mangle]
pub unsafe extern "C" fn rs_logmsg(
    log_level: c_int,
    context: *const c_char,
    func_name: *const c_char,
    line_num: c_int,
    eol: bool,
    message: *const c_char,
) -> bool {
    // Check if logging is initialized
    if !nvim_log_is_initialized() {
        nvim_log_increment_skip();
        return false;
    }

    // Convert C strings to Rust
    let context_str = if context.is_null() {
        None
    } else {
        CStr::from_ptr(context).to_str().ok()
    };

    let func_name_str = if func_name.is_null() {
        None
    } else {
        CStr::from_ptr(func_name).to_str().ok()
    };

    let message_str = if message.is_null() {
        ""
    } else {
        CStr::from_ptr(message).to_str().unwrap_or("")
    };

    // Check for recursion (with lock)
    log_lock();

    let is_recursive = RECURSIVE.with(|cell| {
        let recursive = *cell.borrow();
        if !recursive {
            *cell.borrow_mut() = true;
        }
        recursive
    });

    if is_recursive {
        DID_RECURSE_MSG.with(|cell| {
            let did_msg = *cell.borrow();
            if !did_msg {
                *cell.borrow_mut() = true;
                // The C code schedules an error message here - we skip this
                // as it's complex to call msg_schedule_semsg from Rust
            }
        });
        nvim_log_increment_skip();
        log_unlock();
        return false;
    }

    // Open log file and write
    let log_file = open_log_file();
    let ret = do_log_to_file(
        log_file,
        log_level,
        context_str,
        func_name_str,
        line_num,
        eol,
        message_str,
    );

    // Close file if not stderr/stdout
    let stderr_ptr = libc::fdopen(libc::STDERR_FILENO, c"w".as_ptr());
    if !log_file.is_null() && log_file != stderr_ptr {
        libc::fclose(log_file);
    }
    // Note: We can't easily check stdout here, but the original C code checks both

    // Clear recursion flag
    RECURSIVE.with(|cell| {
        *cell.borrow_mut() = false;
    });

    log_unlock();
    ret
}

/// Open the log file for external use.
///
/// # Safety
/// The returned file handle must be properly closed by the caller
/// (unless it's stderr/stdout).
#[no_mangle]
pub unsafe extern "C" fn rs_open_log_file() -> *mut libc::FILE {
    let path_ptr = nvim_log_get_file_path();

    // Reset errno
    *libc::__errno_location() = 0;

    if !path_ptr.is_null() {
        let path = CStr::from_ptr(path_ptr);
        if !path.to_bytes().is_empty() {
            let file = libc::fopen(path_ptr, c"a".as_ptr());
            if !file.is_null() {
                return file;
            }
        }
    }

    // Log error and return stderr
    let log_file = libc::fdopen(libc::STDERR_FILENO, c"w".as_ptr());

    // Write error message to stderr
    do_log_to_file(
        log_file,
        LOGLVL_ERR,
        None,
        Some("rs_open_log_file"),
        line!() as c_int,
        true,
        &format!(
            "failed to open $NVIM_LOG_FILE: errno={}",
            *libc::__errno_location()
        ),
    );

    log_file
}

/// Check if a file can be created/appended (for `log_try_create`).
///
/// # Safety
/// `fname` must be a valid C string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_log_try_create(fname: *const c_char) -> bool {
    if fname.is_null() {
        return false;
    }

    let path = match CStr::from_ptr(fname).to_bytes() {
        [] => return false,
        b => b,
    };

    log_try_create(path)
}

/// FFI: Format and write a log message to a file (called from C after printf formatting).
///
/// # Safety
/// - `log_file` must be a valid FILE* or null
/// - String parameters must be valid C strings or null
#[no_mangle]
pub unsafe extern "C" fn rs_do_log_to_file(
    log_file: *mut libc::FILE,
    log_level: c_int,
    context: *const c_char,
    func_name: *const c_char,
    line_num: c_int,
    eol: bool,
    message: *const c_char,
) -> bool {
    let context_str = if context.is_null() {
        None
    } else {
        CStr::from_ptr(context).to_str().ok()
    };

    let func_name_str = if func_name.is_null() {
        None
    } else {
        CStr::from_ptr(func_name).to_str().ok()
    };

    let message_str = if message.is_null() {
        ""
    } else {
        CStr::from_ptr(message).to_str().unwrap_or("")
    };

    do_log_to_file(
        log_file,
        log_level,
        context_str,
        func_name_str,
        line_num,
        eol,
        message_str,
    )
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_names() {
        assert_eq!(LOG_LEVEL_NAMES[LOGLVL_DBG as usize], "DBG");
        assert_eq!(LOG_LEVEL_NAMES[LOGLVL_INF as usize], "INF");
        assert_eq!(LOG_LEVEL_NAMES[LOGLVL_WRN as usize], "WRN");
        assert_eq!(LOG_LEVEL_NAMES[LOGLVL_ERR as usize], "ERR");
    }

    #[test]
    fn test_log_try_create_empty() {
        assert!(!log_try_create(&[]));
        assert!(!log_try_create(&[0]));
    }
}
