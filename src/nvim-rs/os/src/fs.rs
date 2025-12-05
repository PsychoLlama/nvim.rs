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
