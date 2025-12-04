//! Filesystem operations
//!
//! Provides portable filesystem abstractions.

use std::ffi::{c_char, c_int, CStr};
use std::fs;
use std::os::raw::c_uint;
use std::path::Path;
use std::ptr;

use nvim_memory::NvimString;

use crate::{FAIL, OK};

/// Check if a path exists.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_path_exists(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    c_int::from(Path::new(path_str).exists())
}

/// Check if a path is a directory.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_isdir(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    c_int::from(Path::new(path_str).is_dir())
}

/// Check if a path is a regular file.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_isfile(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    c_int::from(Path::new(path_str).is_file())
}

/// Check if a path is a symbolic link.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_islink(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    c_int::from(Path::new(path_str).is_symlink())
}

/// Check if a file is readable.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_file_is_readable(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    // Try to open the file for reading
    c_int::from(fs::File::open(path_str).is_ok())
}

/// Check if a file is writable.
///
/// Note: This checks if we can open the file for writing, not just
/// the permission bits.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_file_is_writable(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let path = Path::new(path_str);

    // If file doesn't exist, check if parent directory is writable
    if !path.exists() {
        return match path.parent() {
            Some(parent) if parent.exists() => {
                // Try to create a temp file in the parent
                c_int::from(
                    fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(path)
                        .map(|_| {
                            let _ = fs::remove_file(path);
                        })
                        .is_ok(),
                )
            }
            _ => 0,
        };
    }

    // Try to open existing file for writing (append mode to not truncate)
    c_int::from(fs::OpenOptions::new().append(true).open(path_str).is_ok())
}

/// Get file permissions (mode bits).
///
/// Returns the mode on success, -1 on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_getperm(path: *const c_char) -> i32 {
    if path.is_null() {
        return -1;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        match fs::metadata(path_str) {
            Ok(meta) => meta.mode() as i32,
            Err(_) => -1,
        }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, return a synthetic mode
        match fs::metadata(path_str) {
            Ok(meta) => {
                let mut mode = 0o444; // Default readable
                if !meta.permissions().readonly() {
                    mode |= 0o222; // Writable
                }
                if meta.is_dir() {
                    mode |= 0o111 | 0o040000; // Executable + directory bit
                } else {
                    mode |= 0o100000; // Regular file
                }
                mode
            }
            Err(_) => -1,
        }
    }
}

/// Get file size in bytes.
///
/// Returns the size on success, -1 on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_filesize(path: *const c_char) -> i64 {
    if path.is_null() {
        return -1;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match fs::metadata(path_str) {
        Ok(meta) => meta.len() as i64,
        Err(_) => -1,
    }
}

/// Create a directory.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_mkdir(path: *const c_char, _mode: c_uint) -> c_int {
    if path.is_null() {
        return -1;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match fs::create_dir(path_str) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Create a directory and all parent directories.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_mkdir_all(path: *const c_char, _mode: c_uint) -> c_int {
    if path.is_null() {
        return -1;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match fs::create_dir_all(path_str) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Remove a file.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_remove(path: *const c_char) -> c_int {
    if path.is_null() {
        return -1;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match fs::remove_file(path_str) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Remove a directory (must be empty).
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_rmdir(path: *const c_char) -> c_int {
    if path.is_null() {
        return -1;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match fs::remove_dir(path_str) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Rename/move a file or directory.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `from` and `to` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_os_rename(from: *const c_char, to: *const c_char) -> c_int {
    if from.is_null() || to.is_null() {
        return -1;
    }

    let from_cstr = unsafe { CStr::from_ptr(from) };
    let to_cstr = unsafe { CStr::from_ptr(to) };

    let from_str = match from_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let to_str = match to_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match fs::rename(from_str, to_str) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Get the current working directory.
///
/// Returns OK on success, FAIL on failure.
///
/// # Safety
///
/// `buf` must point to a buffer of at least `size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_os_dirname(buf: *mut c_char, size: usize) -> c_int {
    if buf.is_null() || size == 0 {
        return FAIL;
    }

    match std::env::current_dir() {
        Ok(path) => match path.to_str() {
            Some(s) => {
                let bytes = s.as_bytes();
                if bytes.len() >= size {
                    return FAIL;
                }
                unsafe {
                    ptr::copy_nonoverlapping(bytes.as_ptr(), buf.cast(), bytes.len());
                    *buf.add(bytes.len()) = 0;
                }
                OK
            }
            None => FAIL,
        },
        Err(_) => FAIL,
    }
}

/// Change the current working directory.
///
/// Returns 0 on success, negative error code on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_chdir(path: *const c_char) -> c_int {
    if path.is_null() {
        return -1;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match std::env::set_current_dir(path_str) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Read the target of a symbolic link.
///
/// Returns a newly allocated string (via `xmallocz`) that must be freed
/// with `xfree`, or NULL on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_readlink(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    match fs::read_link(path_str) {
        Ok(target) => match target.to_str() {
            Some(s) => {
                // Use NvimString which allocates with xmallocz
                NvimString::new(s).into_raw()
            }
            None => ptr::null_mut(),
        },
        Err(_) => ptr::null_mut(),
    }
}

// Note: rs_os_readlink_free is no longer needed since rs_os_readlink now uses
// xmallocz. Callers should use xfree() directly to free returned strings.

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_path_exists() {
        // Current directory should exist
        let path = CString::new(".").unwrap();
        let exists = unsafe { rs_os_path_exists(path.as_ptr()) };
        assert_eq!(exists, 1);

        // Non-existent path
        let path = CString::new("/nonexistent/path/12345").unwrap();
        let exists = unsafe { rs_os_path_exists(path.as_ptr()) };
        assert_eq!(exists, 0);
    }

    #[test]
    fn test_isdir() {
        // Current directory is a directory
        let path = CString::new(".").unwrap();
        let isdir = unsafe { rs_os_isdir(path.as_ptr()) };
        assert_eq!(isdir, 1);

        // Cargo.toml is not a directory (if it exists)
        if Path::new("Cargo.toml").exists() {
            let path = CString::new("Cargo.toml").unwrap();
            let isdir = unsafe { rs_os_isdir(path.as_ptr()) };
            assert_eq!(isdir, 0);
        }
    }

    #[test]
    fn test_dirname() {
        let mut buf = [0i8; 1024];
        let result = unsafe { rs_os_dirname(buf.as_mut_ptr(), buf.len()) };
        assert_eq!(result, OK);

        let cwd = unsafe { CStr::from_ptr(buf.as_ptr()) };
        let cwd_str = cwd.to_str().unwrap();
        assert!(!cwd_str.is_empty());
    }

    #[test]
    fn test_getperm() {
        // Current directory should have permissions
        let path = CString::new(".").unwrap();
        let perm = unsafe { rs_os_getperm(path.as_ptr()) };
        assert!(perm > 0);
    }
}
