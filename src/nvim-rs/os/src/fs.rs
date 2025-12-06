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

/// Convert a Rust `io::Error` to a libuv-compatible error code.
///
/// libuv uses negated errno values (e.g., `UV_ENOENT` = -2 = -ENOENT).
/// This function maps common Rust `io::ErrorKind` values to their libuv equivalents.
fn io_error_to_uv_error(err: &std::io::Error) -> i32 {
    use std::io::ErrorKind;

    // On Unix, we can get the raw os error directly
    #[cfg(unix)]
    if let Some(errno) = err.raw_os_error() {
        // libuv negates errno values
        return -errno;
    }

    // Fall back to mapping ErrorKind for cross-platform compatibility
    match err.kind() {
        ErrorKind::NotFound => -2,            // UV_ENOENT = -ENOENT
        ErrorKind::PermissionDenied => -13,   // UV_EACCES = -EACCES
        ErrorKind::AlreadyExists => -17,      // UV_EEXIST = -EEXIST
        ErrorKind::InvalidInput => -22,       // UV_EINVAL = -EINVAL
        ErrorKind::NotADirectory => -20,      // UV_ENOTDIR = -ENOTDIR
        ErrorKind::IsADirectory => -21,       // UV_EISDIR = -EISDIR
        ErrorKind::DirectoryNotEmpty => -39,  // UV_ENOTEMPTY = -ENOTEMPTY
        ErrorKind::ReadOnlyFilesystem => -30, // UV_EROFS = -EROFS
        ErrorKind::FileTooLarge => -27,       // UV_EFBIG = -EFBIG
        ErrorKind::StorageFull => -28,        // UV_ENOSPC = -ENOSPC
        ErrorKind::TimedOut => -110,          // UV_ETIMEDOUT = -ETIMEDOUT
        ErrorKind::Interrupted => -4,         // UV_EINTR = -EINTR
        _ => -1,                              // Generic error
    }
}

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

/// Check if a path is a directory and NOT a symlink to a directory.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_isrealdir(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let path = Path::new(path_str);

    // Use symlink_metadata (lstat) to check if it's a symlink
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            // If it's a symlink, return false
            if meta.file_type().is_symlink() {
                return 0;
            }
            // Otherwise, check if it's a directory
            c_int::from(meta.is_dir())
        }
        Err(_) => 0,
    }
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
/// Returns:
/// - 0 if `name` is not writable
/// - 1 if `name` is writable (file)
/// - 2 if `name` is a writable directory
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

    // Check if path exists
    if !path.exists() {
        return 0;
    }

    // Check write access - on Unix, we can use access() equivalent
    #[cfg(unix)]
    {
        // Get metadata
        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return 0,
        };

        // Try to open for writing to test actual access
        let is_writable = if meta.is_dir() {
            // For directories, try to create a temp file
            use std::io::ErrorKind;
            let test_path = path.join(".nvim_write_test");
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&test_path)
            {
                Ok(_) => {
                    let _ = fs::remove_file(&test_path);
                    true
                }
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                    // File exists, try to open it for writing (append implies write)
                    fs::OpenOptions::new().append(true).open(&test_path).is_ok()
                }
                Err(_) => false,
            }
        } else {
            // For files, try to open for writing (append mode to not truncate)
            fs::OpenOptions::new().append(true).open(path).is_ok()
        };

        if is_writable {
            if meta.is_dir() {
                2
            } else {
                1
            }
        } else {
            0
        }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, use simpler check
        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return 0,
        };

        if meta.permissions().readonly() {
            return 0;
        }

        if meta.is_dir() {
            2
        } else {
            1
        }
    }
}

/// Get file permissions (mode bits).
///
/// Returns the mode on success, libuv-compatible error code on failure
/// (e.g., `UV_ENOENT` = -2 for file not found).
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_getperm(path: *const c_char) -> i32 {
    if path.is_null() {
        return -22; // UV_EINVAL
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -22, // UV_EINVAL for invalid UTF-8
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        match fs::metadata(path_str) {
            Ok(meta) => meta.mode() as i32,
            Err(e) => io_error_to_uv_error(&e),
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
            Err(e) => io_error_to_uv_error(&e),
        }
    }
}

/// Set file permissions (mode bits).
///
/// Returns OK on success, FAIL on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_setperm(path: *const c_char, perm: c_int) -> c_int {
    if path.is_null() {
        return FAIL;
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return FAIL,
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // perm is a Unix mode (positive value), safe to cast
        #[allow(clippy::cast_sign_loss)]
        let permissions = fs::Permissions::from_mode(perm as u32);
        match fs::set_permissions(path_str, permissions) {
            Ok(()) => OK,
            Err(_) => FAIL,
        }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, we can only set readonly flag
        let meta = match fs::metadata(path_str) {
            Ok(m) => m,
            Err(_) => return FAIL,
        };
        let mut permissions = meta.permissions();
        // If write bits are clear, set readonly
        permissions.set_readonly((perm & 0o222) == 0);
        match fs::set_permissions(path_str, permissions) {
            Ok(()) => OK,
            Err(_) => FAIL,
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

/// Create a directory with the specified mode.
///
/// Returns 0 on success, libuv-compatible error code on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_mkdir(path: *const c_char, mode: c_uint) -> c_int {
    if path.is_null() {
        return -22; // UV_EINVAL
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -22, // UV_EINVAL for invalid UTF-8
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        let result = fs::DirBuilder::new().mode(mode).create(path_str);
        match result {
            Ok(()) => 0,
            Err(e) => io_error_to_uv_error(&e),
        }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, mode is ignored
        let _ = mode;
        match fs::create_dir(path_str) {
            Ok(()) => 0,
            Err(e) => io_error_to_uv_error(&e),
        }
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
/// Returns 0 on success, libuv-compatible error code on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_remove(path: *const c_char) -> c_int {
    if path.is_null() {
        return -22; // UV_EINVAL
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -22, // UV_EINVAL for invalid UTF-8
    };

    match fs::remove_file(path_str) {
        Ok(()) => 0,
        Err(e) => io_error_to_uv_error(&e),
    }
}

/// Remove a directory (must be empty).
///
/// Returns 0 on success, libuv-compatible error code on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_rmdir(path: *const c_char) -> c_int {
    if path.is_null() {
        return -22; // UV_EINVAL
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -22, // UV_EINVAL for invalid UTF-8
    };

    match fs::remove_dir(path_str) {
        Ok(()) => 0,
        Err(e) => io_error_to_uv_error(&e),
    }
}

/// Rename/move a file or directory.
///
/// Returns OK on success, FAIL on failure.
///
/// # Safety
///
/// `from` and `to` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_os_rename(from: *const c_char, to: *const c_char) -> c_int {
    if from.is_null() || to.is_null() {
        return FAIL;
    }

    let from_cstr = unsafe { CStr::from_ptr(from) };
    let to_cstr = unsafe { CStr::from_ptr(to) };

    let from_str = match from_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return FAIL,
    };

    let to_str = match to_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return FAIL,
    };

    match fs::rename(from_str, to_str) {
        Ok(()) => OK,
        Err(_) => FAIL,
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

/// Create a temporary directory from a template.
///
/// The template must end with "XXXXXX" which will be replaced by random characters.
/// On success, the created path is written to the output buffer.
///
/// Returns 0 on success, libuv-compatible error code on failure.
///
/// # Safety
///
/// - `template` must be a valid null-terminated C string ending with "XXXXXX"
/// - `path` must point to a buffer of at least `path_len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_os_mkdtemp(
    template: *const c_char,
    path: *mut c_char,
    path_len: usize,
) -> c_int {
    if template.is_null() || path.is_null() || path_len == 0 {
        return -22; // UV_EINVAL
    }

    #[cfg(unix)]
    {
        // mkdtemp modifies the template in place, so we need a mutable copy
        let template_cstr = unsafe { CStr::from_ptr(template) };
        let template_bytes = template_cstr.to_bytes_with_nul();

        // Check that template ends with XXXXXX
        if template_bytes.len() < 7 {
            return -22; // UV_EINVAL - template too short
        }

        // Create a mutable buffer for mkdtemp
        let mut buf: Vec<u8> = template_bytes.to_vec();

        let result = unsafe { libc::mkdtemp(buf.as_mut_ptr().cast()) };

        if result.is_null() {
            // mkdtemp failed, get errno
            let errno = *libc::__errno_location();
            return -errno;
        }

        // Copy the result to the output path
        let result_len = buf.len(); // includes null terminator
        if result_len > path_len {
            // Path too long for output buffer
            // Clean up by removing the created directory
            let _ = libc::rmdir(buf.as_ptr().cast());
            return -36; // UV_ENAMETOOLONG
        }

        unsafe {
            ptr::copy_nonoverlapping(buf.as_ptr(), path.cast(), result_len);
        }
        0
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, use a fallback with random directory name
        use std::time::SystemTime;

        let template_cstr = unsafe { CStr::from_ptr(template) };
        let template_str = match template_cstr.to_str() {
            Ok(s) => s,
            Err(_) => return -22, // UV_EINVAL
        };

        // Find and replace XXXXXX with random chars
        if !template_str.ends_with("XXXXXX") {
            return -22; // UV_EINVAL
        }

        // Generate a unique suffix based on time and process ID
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let pid = std::process::id();
        let suffix = format!("{:06x}", (now ^ u128::from(pid)) & 0xFFFFFF);

        let new_path = format!("{}{}", &template_str[..template_str.len() - 6], suffix);

        // Try to create the directory
        match fs::create_dir(&new_path) {
            Ok(()) => {
                let new_path_bytes = new_path.as_bytes();
                if new_path_bytes.len() + 1 > path_len {
                    let _ = fs::remove_dir(&new_path);
                    return -36; // UV_ENAMETOOLONG
                }
                unsafe {
                    ptr::copy_nonoverlapping(
                        new_path_bytes.as_ptr(),
                        path.cast(),
                        new_path_bytes.len(),
                    );
                    *path.add(new_path_bytes.len()) = 0;
                }
                0
            }
            Err(e) => io_error_to_uv_error(&e),
        }
    }
}

/// Change ownership of a file.
///
/// Returns 0 on success, libuv-compatible error code on failure.
/// If `owner` or `group` is -1 (max value for unsigned), that ID is not changed.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_chown(path: *const c_char, owner: u32, group: u32) -> c_int {
    if path.is_null() {
        return -22; // UV_EINVAL
    }

    #[cfg(unix)]
    {
        let path_cstr = unsafe { CStr::from_ptr(path) };

        // Convert -1 (max u32) to libc's convention
        let uid: libc::uid_t = if owner == u32::MAX {
            libc::uid_t::MAX // -1 in unsigned
        } else {
            owner as libc::uid_t
        };
        let gid: libc::gid_t = if group == u32::MAX {
            libc::gid_t::MAX // -1 in unsigned
        } else {
            group as libc::gid_t
        };

        let result = unsafe { libc::chown(path_cstr.as_ptr(), uid, gid) };

        if result == 0 {
            0
        } else {
            -(*libc::__errno_location())
        }
    }

    #[cfg(not(unix))]
    {
        // chown is not meaningful on non-Unix systems
        let _ = (path, owner, group);
        0 // Success (no-op)
    }
}

/// Change ownership of a file by file descriptor.
///
/// Returns 0 on success, libuv-compatible error code on failure.
/// If `owner` or `group` is -1 (max value for unsigned), that ID is not changed.
#[no_mangle]
pub extern "C" fn rs_os_fchown(fd: c_int, owner: u32, group: u32) -> c_int {
    #[cfg(unix)]
    {
        // Convert -1 (max u32) to libc's convention
        let uid: libc::uid_t = if owner == u32::MAX {
            libc::uid_t::MAX
        } else {
            owner as libc::uid_t
        };
        let gid: libc::gid_t = if group == u32::MAX {
            libc::gid_t::MAX
        } else {
            group as libc::gid_t
        };

        let result = unsafe { libc::fchown(fd, uid, gid) };

        if result == 0 {
            0
        } else {
            unsafe { -(*libc::__errno_location()) }
        }
    }

    #[cfg(not(unix))]
    {
        // fchown is not meaningful on non-Unix systems
        let _ = (fd, owner, group);
        0 // Success (no-op)
    }
}

/// Set file access and modification times.
///
/// Times are specified as seconds since the Unix epoch (as doubles to allow
/// sub-second precision, matching libuv's `uv_fs_utime`).
///
/// Returns 0 on success, libuv-compatible error code on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_file_settime(path: *const c_char, atime: f64, mtime: f64) -> c_int {
    if path.is_null() {
        return -22; // UV_EINVAL
    }

    #[cfg(unix)]
    {
        let path_cstr = unsafe { CStr::from_ptr(path) };

        // Convert double seconds to timeval (seconds + microseconds)
        let atime_sec = atime.trunc() as libc::time_t;
        let atime_usec = ((atime.fract()) * 1_000_000.0) as libc::suseconds_t;
        let mtime_sec = mtime.trunc() as libc::time_t;
        let mtime_usec = ((mtime.fract()) * 1_000_000.0) as libc::suseconds_t;

        let times = [
            libc::timeval {
                tv_sec: atime_sec,
                tv_usec: atime_usec,
            },
            libc::timeval {
                tv_sec: mtime_sec,
                tv_usec: mtime_usec,
            },
        ];

        let result = unsafe { libc::utimes(path_cstr.as_ptr(), times.as_ptr()) };

        if result == 0 {
            0
        } else {
            unsafe { -(*libc::__errno_location()) }
        }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, use filetime crate or Windows API
        // For now, return success (no-op)
        let _ = (path, atime, mtime);
        0
    }
}

/// Close a file descriptor.
///
/// Returns 0 on success, libuv-compatible error code on failure.
#[no_mangle]
pub extern "C" fn rs_os_close(fd: c_int) -> c_int {
    #[cfg(unix)]
    {
        let result = unsafe { libc::close(fd) };
        if result == 0 {
            0
        } else {
            unsafe { -(*libc::__errno_location()) }
        }
    }

    #[cfg(not(unix))]
    {
        // On Windows, use _close
        let result = unsafe { libc::close(fd) };
        if result == 0 {
            0
        } else {
            -1 // Generic error
        }
    }
}

/// Duplicate a file descriptor.
///
/// Returns the new file descriptor on success, libuv-compatible error code on failure.
#[no_mangle]
pub extern "C" fn rs_os_dup(fd: c_int) -> c_int {
    #[cfg(unix)]
    {
        loop {
            let result = unsafe { libc::dup(fd) };
            if result >= 0 {
                return result;
            }
            let errno = unsafe { *libc::__errno_location() };
            if errno == libc::EINTR {
                // Retry on EINTR
                continue;
            }
            return -errno;
        }
    }

    #[cfg(not(unix))]
    {
        let result = unsafe { libc::dup(fd) };
        if result >= 0 {
            result
        } else {
            -1 // Generic error
        }
    }
}

// libuv copy file flags
const UV_FS_COPYFILE_EXCL: c_int = 0x0001;
const UV_FS_COPYFILE_FICLONE: c_int = 0x0002;
#[allow(dead_code)]
const UV_FS_COPYFILE_FICLONE_FORCE: c_int = 0x0004;

/// Copy a file from one path to another.
///
/// Flags:
/// - `UV_FS_COPYFILE_EXCL` (0x0001): Fail if destination exists
/// - `UV_FS_COPYFILE_FICLONE` (0x0002): Attempt copy-on-write clone (best effort)
/// - `UV_FS_COPYFILE_FICLONE_FORCE` (0x0004): Require copy-on-write clone
///
/// Returns 0 on success, libuv-compatible error code on failure.
///
/// # Safety
///
/// `path` and `new_path` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_os_copy(
    path: *const c_char,
    new_path: *const c_char,
    flags: c_int,
) -> c_int {
    if path.is_null() || new_path.is_null() {
        return -22; // UV_EINVAL
    }

    let path_cstr = unsafe { CStr::from_ptr(path) };
    let new_path_cstr = unsafe { CStr::from_ptr(new_path) };

    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -22, // UV_EINVAL for invalid UTF-8
    };

    let new_path_str = match new_path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -22, // UV_EINVAL for invalid UTF-8
    };

    // Check if EXCL flag is set and destination exists
    if (flags & UV_FS_COPYFILE_EXCL) != 0 && Path::new(new_path_str).exists() {
        return -17; // UV_EEXIST
    }

    // Attempt copy with optional COW clone support
    #[cfg(target_os = "linux")]
    {
        // On Linux, try copy_file_range first which can do COW on supported filesystems
        if (flags & UV_FS_COPYFILE_FICLONE) != 0 {
            if let Ok(result) = try_clone_file(path_str, new_path_str) {
                if result == 0 {
                    return 0;
                }
                // Clone failed, fall through to regular copy
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = flags; // Silence unused warning for flags on non-Linux
    }

    // Regular copy using std::fs::copy
    match fs::copy(path_str, new_path_str) {
        Ok(_) => 0,
        Err(e) => io_error_to_uv_error(&e),
    }
}

/// Attempt to clone a file using Linux's FICLONE ioctl.
/// Returns Ok(0) on success, Ok(-1) if cloning is not supported, or Err on other failures.
#[cfg(target_os = "linux")]
fn try_clone_file(src: &str, dst: &str) -> Result<c_int, std::io::Error> {
    use std::fs::OpenOptions;
    use std::os::unix::io::AsRawFd;

    // FICLONE ioctl number (from linux/fs.h)
    // #define FICLONE _IOW(0x94, 9, int)
    // This is 0x40049409 on most architectures
    const FICLONE: libc::c_ulong = 0x4004_9409;

    // Open source file for reading
    let src_file = std::fs::File::open(src)?;

    // Create destination file (truncate if exists)
    let dst_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dst)?;

    let src_fd = src_file.as_raw_fd();
    let dst_fd = dst_file.as_raw_fd();

    // Try FICLONE ioctl
    let result = unsafe { libc::ioctl(dst_fd, FICLONE, src_fd) };

    if result == 0 {
        Ok(0)
    } else {
        // Clone failed, return -1 to indicate fallback needed
        // Remove the empty destination file we created
        let _ = std::fs::remove_file(dst);
        Ok(-1)
    }
}

// Node type constants (matching fs_defs.h)
const NODE_NORMAL: c_int = 0; // file or directory
const NODE_WRITABLE: c_int = 1; // something we can write to (char device, fifo, socket)
const NODE_OTHER: c_int = 2; // non-writable thing (e.g., block device)

/// Check what type of filesystem node `name` is.
///
/// Returns:
/// - `NODE_NORMAL` (0): file or directory (or doesn't exist)
/// - `NODE_WRITABLE` (1): writable device, socket, fifo, etc.
/// - `NODE_OTHER` (2): non-writable things (e.g., block device)
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_nodetype(name: *const c_char) -> c_int {
    if name.is_null() {
        return NODE_NORMAL;
    }

    let path_cstr = unsafe { CStr::from_ptr(name) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return NODE_NORMAL,
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;

        // Use symlink_metadata to get the actual file type (like stat, not lstat)
        // Actually, the C code uses uv_fs_stat which follows symlinks, so we use metadata
        let meta = match fs::metadata(path_str) {
            Ok(m) => m,
            Err(_) => return NODE_NORMAL, // File doesn't exist
        };

        let file_type = meta.file_type();

        // Regular file or directory
        if file_type.is_file() || file_type.is_dir() {
            return NODE_NORMAL;
        }

        // Block device is not writable (in the sense nvim uses)
        if file_type.is_block_device() {
            return NODE_OTHER;
        }

        // Everything else (char device, fifo, socket) is considered writable
        // This matches the C code comment: "Everything else is writable?"
        NODE_WRITABLE
    }

    #[cfg(not(unix))]
    {
        // On Windows, the C code uses uv_guess_handle which requires opening the file.
        // For now, we just check if it's a regular file or directory.
        let meta = match fs::metadata(path_str) {
            Ok(m) => m,
            Err(_) => return NODE_NORMAL, // File doesn't exist
        };

        if meta.is_file() || meta.is_dir() {
            NODE_NORMAL
        } else {
            // On Windows, other types are rare but could be pipes etc.
            NODE_OTHER
        }
    }
}

/// Set the close-on-exec flag on a file descriptor.
///
/// Returns 0 on success, -1 on failure.
#[no_mangle]
pub extern "C" fn rs_os_set_cloexec(fd: c_int) -> c_int {
    #[cfg(unix)]
    {
        // Get current flags
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if flags < 0 {
            return -1;
        }

        // Check if FD_CLOEXEC is already set
        if (flags & libc::FD_CLOEXEC) != 0 {
            return 0; // Already set
        }

        // Set FD_CLOEXEC
        let result = unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) };
        if result < 0 {
            return -1;
        }
        0
    }

    #[cfg(not(unix))]
    {
        // On Windows, files should be opened with O_NOINHERIT
        let _ = fd;
        -1
    }
}

/// Get the path to the currently running executable.
///
/// Returns 0 on success, libuv-compatible error code on failure.
/// On success, the path is written to `buffer` (null-terminated) and
/// `size` is updated to the actual length (not including null terminator).
///
/// # Safety
///
/// - `buffer` must point to a buffer of at least `*size` bytes
/// - `size` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_os_exepath(buffer: *mut c_char, size: *mut usize) -> c_int {
    if buffer.is_null() || size.is_null() {
        return -22; // UV_EINVAL
    }

    let buf_size = unsafe { *size };
    if buf_size == 0 {
        return -22; // UV_EINVAL
    }

    match std::env::current_exe() {
        Ok(path) => {
            match path.to_str() {
                Some(path_str) => {
                    let path_bytes = path_str.as_bytes();
                    if path_bytes.len() >= buf_size {
                        // Path is too long for buffer
                        return -7; // UV_E2BIG
                    }

                    unsafe {
                        ptr::copy_nonoverlapping(
                            path_bytes.as_ptr(),
                            buffer.cast(),
                            path_bytes.len(),
                        );
                        // Null terminate
                        *buffer.add(path_bytes.len()) = 0;
                        // Update size to actual length (not including null)
                        *size = path_bytes.len();
                    }
                    0
                }
                None => -22, // UV_EINVAL - path not valid UTF-8
            }
        }
        Err(e) => io_error_to_uv_error(&e),
    }
}

// libuv error codes for read/write
const UV_EINTR: c_int = -4;
const UV_EAGAIN: c_int = -11;

/// Convert errno to libuv error code.
fn errno_to_uv_error(errno: c_int) -> c_int {
    -errno
}

/// Read from a file descriptor.
///
/// Handles EINTR by retrying. When `non_blocking` is true, EAGAIN causes
/// immediate return instead of retry.
///
/// Returns the number of bytes read, or a negative libuv error code.
/// Sets `*ret_eof` to true if EOF was reached.
///
/// # Safety
///
/// - `ret_eof` must be a valid pointer
/// - `ret_buf` must point to a buffer of at least `size` bytes (or be NULL if size is 0)
#[no_mangle]
pub unsafe extern "C" fn rs_os_read(
    fd: c_int,
    ret_eof: *mut bool,
    ret_buf: *mut c_char,
    size: usize,
    non_blocking: bool,
) -> isize {
    if ret_eof.is_null() {
        return -22; // UV_EINVAL
    }

    unsafe { *ret_eof = false };

    if ret_buf.is_null() {
        if size == 0 {
            return 0;
        }
        return -22; // UV_EINVAL
    }

    let mut read_bytes: usize = 0;

    while read_bytes < size {
        let remaining = size - read_bytes;
        let buf_ptr = unsafe { ret_buf.add(read_bytes) };

        let cur_read = unsafe { libc::read(fd, buf_ptr.cast(), remaining) };

        match cur_read.cmp(&0) {
            std::cmp::Ordering::Greater => {
                // Safe: cur_read > 0, so cast to usize is safe
                #[allow(clippy::cast_sign_loss)]
                {
                    read_bytes += cur_read as usize;
                }
            }
            std::cmp::Ordering::Less => {
                let errno = unsafe { *libc::__errno_location() };
                unsafe { *libc::__errno_location() = 0 };
                let error = errno_to_uv_error(errno);

                if non_blocking && error == UV_EAGAIN {
                    break;
                }
                if error != UV_EINTR && error != UV_EAGAIN {
                    return error as isize;
                }
                // EINTR or EAGAIN (blocking): retry
            }
            std::cmp::Ordering::Equal => {
                // cur_read == 0 means EOF
                unsafe { *ret_eof = true };
                break;
            }
        }
    }

    read_bytes as isize
}

/// Write to a file descriptor.
///
/// Handles EINTR by retrying. When `non_blocking` is true, EAGAIN causes
/// immediate return instead of retry.
///
/// Returns the number of bytes written, or a negative libuv error code.
///
/// # Safety
///
/// - `buf` must point to a buffer of at least `size` bytes (or be NULL if size is 0)
#[no_mangle]
pub unsafe extern "C" fn rs_os_write(
    fd: c_int,
    buf: *const c_char,
    size: usize,
    non_blocking: bool,
) -> isize {
    if buf.is_null() {
        if size == 0 {
            return 0;
        }
        return -22; // UV_EINVAL
    }

    let mut written_bytes: usize = 0;

    while written_bytes < size {
        let remaining = size - written_bytes;
        let buf_ptr = unsafe { buf.add(written_bytes) };

        let cur_written = unsafe { libc::write(fd, buf_ptr.cast(), remaining) };

        match cur_written.cmp(&0) {
            std::cmp::Ordering::Greater => {
                // Safe: cur_written > 0, so cast to usize is safe
                #[allow(clippy::cast_sign_loss)]
                {
                    written_bytes += cur_written as usize;
                }
            }
            std::cmp::Ordering::Less => {
                let errno = unsafe { *libc::__errno_location() };
                unsafe { *libc::__errno_location() = 0 };
                let error = errno_to_uv_error(errno);

                if non_blocking && error == UV_EAGAIN {
                    break;
                }
                if error != UV_EINTR && error != UV_EAGAIN {
                    return error as isize;
                }
                // EINTR or EAGAIN (blocking): retry
            }
            std::cmp::Ordering::Equal => {
                // cur_written == 0 is unusual for write, treat as error
                return -5; // UV_EIO
            }
        }
    }

    written_bytes as isize
}

/// `FileID` structure matching nvim's `FileID` in `fs_defs.h`
///
/// Contains inode and `device_id` to uniquely identify a file.
#[repr(C)]
pub struct FileID {
    pub inode: u64,
    pub device_id: u64,
}

/// `uv_timespec_t` structure matching libuv's timespec.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UvTimespec {
    pub tv_sec: libc::c_long,
    pub tv_nsec: libc::c_long,
}

/// `uv_stat_t` structure matching libuv's stat structure.
///
/// This must match the exact layout of libuv's `uv_stat_t`.
#[repr(C)]
pub struct UvStat {
    pub st_dev: u64,
    pub st_mode: u64,
    pub st_nlink: u64,
    pub st_uid: u64,
    pub st_gid: u64,
    pub st_rdev: u64,
    pub st_ino: u64,
    pub st_size: u64,
    pub st_blksize: u64,
    pub st_blocks: u64,
    pub st_flags: u64,
    pub st_gen: u64,
    pub st_atim: UvTimespec,
    pub st_mtim: UvTimespec,
    pub st_ctim: UvTimespec,
    pub st_birthtim: UvTimespec,
}

/// `FileInfo` structure matching nvim's `FileInfo` in `fs_defs.h`
///
/// Wraps a `uv_stat_t` structure.
#[repr(C)]
pub struct FileInfo {
    pub stat: UvStat,
}

/// Check if two `FileID`s are equal.
///
/// # Safety
///
/// Both `file_id_1` and `file_id_2` must be valid pointers to `FileID` structs.
#[no_mangle]
pub unsafe extern "C" fn rs_os_fileid_equal(
    file_id_1: *const FileID,
    file_id_2: *const FileID,
) -> bool {
    if file_id_1.is_null() || file_id_2.is_null() {
        return false;
    }

    let id1 = unsafe { &*file_id_1 };
    let id2 = unsafe { &*file_id_2 };

    id1.inode == id2.inode && id1.device_id == id2.device_id
}

/// Check if a `FileID` equals a `FileInfo`.
///
/// Compares the inode and device from the `FileID` against the corresponding
/// fields in the `FileInfo`'s stat structure.
///
/// # Safety
///
/// Both `file_id` and `file_info` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_os_fileid_equal_fileinfo(
    file_id: *const FileID,
    file_info: *const FileInfo,
) -> bool {
    if file_id.is_null() || file_info.is_null() {
        return false;
    }

    let id = unsafe { &*file_id };
    let info = unsafe { &*file_info };

    id.inode == info.stat.st_ino && id.device_id == info.stat.st_dev
}

/// Compare the inodes of two `FileInfo`s.
///
/// # Safety
///
/// Both `file_info_1` and `file_info_2` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_os_fileinfo_id_equal(
    file_info_1: *const FileInfo,
    file_info_2: *const FileInfo,
) -> bool {
    if file_info_1.is_null() || file_info_2.is_null() {
        return false;
    }

    let info1 = unsafe { &*file_info_1 };
    let info2 = unsafe { &*file_info_2 };

    info1.stat.st_ino == info2.stat.st_ino && info1.stat.st_dev == info2.stat.st_dev
}

/// Get the `FileID` of a `FileInfo`.
///
/// # Safety
///
/// Both `file_info` and `file_id` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_os_fileinfo_id(file_info: *const FileInfo, file_id: *mut FileID) {
    if file_info.is_null() || file_id.is_null() {
        return;
    }

    let info = unsafe { &*file_info };
    let id = unsafe { &mut *file_id };

    id.inode = info.stat.st_ino;
    id.device_id = info.stat.st_dev;
}

/// Get the inode of a `FileInfo`.
///
/// # Safety
///
/// `file_info` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_os_fileinfo_inode(file_info: *const FileInfo) -> u64 {
    if file_info.is_null() {
        return 0;
    }

    let info = unsafe { &*file_info };
    info.stat.st_ino
}

/// Get the size of a file from a `FileInfo`.
///
/// # Safety
///
/// `file_info` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_os_fileinfo_size(file_info: *const FileInfo) -> u64 {
    if file_info.is_null() {
        return 0;
    }

    let info = unsafe { &*file_info };
    info.stat.st_size
}

/// Get the number of hardlinks from a `FileInfo`.
///
/// # Safety
///
/// `file_info` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_os_fileinfo_hardlinks(file_info: *const FileInfo) -> u64 {
    if file_info.is_null() {
        return 0;
    }

    let info = unsafe { &*file_info };
    info.stat.st_nlink
}

/// Get the blocksize from a `FileInfo`.
///
/// # Safety
///
/// `file_info` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_os_fileinfo_blocksize(file_info: *const FileInfo) -> u64 {
    if file_info.is_null() {
        return 0;
    }

    let info = unsafe { &*file_info };
    info.stat.st_blksize
}

/// Return the canonicalized absolute pathname.
///
/// Uses libc's `realpath()` to resolve the canonical path, then copies
/// the result to the provided buffer (or allocates one if `buf` is NULL).
///
/// # Safety
///
/// - `name` must be a valid null-terminated C string.
/// - If `buf` is not NULL, it must point to a buffer of at least `len` bytes.
/// - The caller is responsible for freeing any allocated buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_os_realpath(
    name: *const c_char,
    buf: *mut c_char,
    len: usize,
) -> *mut c_char {
    if name.is_null() {
        return ptr::null_mut();
    }

    #[cfg(unix)]
    {
        // Call libc::realpath with NULL to let it allocate the buffer
        let resolved = unsafe { libc::realpath(name, ptr::null_mut()) };
        if resolved.is_null() {
            return ptr::null_mut();
        }

        // Get the length of the resolved path
        let resolved_len = unsafe { libc::strlen(resolved) };

        // Determine the output buffer
        let out_buf = if buf.is_null() {
            // Allocate using nvim's allocator
            unsafe { nvim_memory::xmalloc(len).cast::<c_char>() }
        } else {
            buf
        };

        // Copy the resolved path to the output buffer, respecting the length limit
        let copy_len = std::cmp::min(resolved_len, len.saturating_sub(1));
        unsafe {
            ptr::copy_nonoverlapping(resolved, out_buf, copy_len);
            // Null-terminate
            *out_buf.add(copy_len) = 0;
        }

        // Free the buffer allocated by realpath
        unsafe { libc::free(resolved.cast::<libc::c_void>()) };

        out_buf
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, return NULL (not implemented)
        let _ = (name, buf, len);
        ptr::null_mut()
    }
}

/// Open or create a file.
///
/// Returns a file descriptor on success, or a negative libuv error code on failure.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_os_open(path: *const c_char, flags: c_int, mode: c_int) -> c_int {
    if path.is_null() {
        return -22; // UV_EINVAL
    }

    #[cfg(unix)]
    {
        let fd = unsafe { libc::open(path, flags, mode) };
        if fd < 0 {
            // Return libuv-compatible error code (negated errno)
            unsafe { -(*libc::__errno_location()) }
        } else {
            fd
        }
    }

    #[cfg(not(unix))]
    {
        let _ = (path, flags, mode);
        -38 // UV_ENOSYS
    }
}

/// Open a file using fopen-style flags.
///
/// Converts fopen-style mode strings ("r", "w", "a", "r+", "w+", "a+", with optional "b")
/// to `open()` flags and returns a `FILE*` pointer.
///
/// # Safety
///
/// - `path` must be a valid null-terminated C string.
/// - `flags` must be a valid null-terminated C string with length 1-2.
/// - The returned `FILE*` must be closed with `fclose()` when done.
#[no_mangle]
pub unsafe extern "C" fn rs_os_fopen(
    path: *const c_char,
    flags: *const c_char,
) -> *mut libc::FILE {
    if path.is_null() || flags.is_null() {
        return ptr::null_mut();
    }

    #[cfg(unix)]
    {
        let flags_cstr = unsafe { CStr::from_ptr(flags) };
        let flags_bytes = flags_cstr.to_bytes();

        if flags_bytes.is_empty() || flags_bytes.len() > 2 {
            return ptr::null_mut();
        }

        let iflags = if flags_bytes.len() == 1 || flags_bytes[1] == b'b' {
            // Single char or char + 'b'
            match flags_bytes[0] {
                b'r' => libc::O_RDONLY,
                b'w' => libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                b'a' => libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND,
                _ => return ptr::null_mut(),
            }
        } else if flags_bytes[1] == b'+' {
            // Char followed by '+'
            match flags_bytes[0] {
                b'r' => libc::O_RDWR,
                b'w' => libc::O_RDWR | libc::O_CREAT | libc::O_TRUNC,
                b'a' => libc::O_RDWR | libc::O_CREAT | libc::O_APPEND,
                _ => return ptr::null_mut(),
            }
        } else {
            return ptr::null_mut();
        };

        // Default mode 0666, will be umask-adjusted
        let fd = unsafe { libc::open(path, iflags, 0o666) };
        if fd < 0 {
            return ptr::null_mut();
        }

        unsafe { libc::fdopen(fd, flags) }
    }

    #[cfg(not(unix))]
    {
        let _ = (path, flags);
        ptr::null_mut()
    }
}

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

    #[test]
    fn test_fileid_equal() {
        let id1 = FileID {
            inode: 123,
            device_id: 456,
        };
        let id2 = FileID {
            inode: 123,
            device_id: 456,
        };
        let id3 = FileID {
            inode: 789,
            device_id: 456,
        };

        assert!(unsafe { rs_os_fileid_equal(&id1, &id2) });
        assert!(!unsafe { rs_os_fileid_equal(&id1, &id3) });
        assert!(!unsafe { rs_os_fileid_equal(std::ptr::null(), &id1) });
    }

    #[test]
    fn test_fileid_equal_fileinfo() {
        // Create a FileID
        let file_id = FileID {
            inode: 12345,
            device_id: 67890,
        };

        // Create a matching FileInfo with zero-initialized timespec fields
        let zero_ts = UvTimespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        let matching_info = FileInfo {
            stat: UvStat {
                st_dev: 67890,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 12345,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: zero_ts,
                st_mtim: zero_ts,
                st_ctim: zero_ts,
                st_birthtim: zero_ts,
            },
        };

        // Create a non-matching FileInfo (different inode)
        let non_matching_info = FileInfo {
            stat: UvStat {
                st_dev: 67890,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 99999, // Different inode
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: zero_ts,
                st_mtim: zero_ts,
                st_ctim: zero_ts,
                st_birthtim: zero_ts,
            },
        };

        assert!(unsafe { rs_os_fileid_equal_fileinfo(&file_id, &matching_info) });
        assert!(!unsafe { rs_os_fileid_equal_fileinfo(&file_id, &non_matching_info) });
        assert!(!unsafe { rs_os_fileid_equal_fileinfo(std::ptr::null(), &matching_info) });
    }
}
