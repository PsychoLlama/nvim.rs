//! Directory scanning and recursive deletion for Neovim.
//!
//! This module provides:
//! - `readdir_core`: directory listing with optional filtering
//! - `delete_recursive`: recursive directory/file deletion

#![allow(unsafe_code)]

use std::ffi::{c_char, c_int, c_void};

// sizeof(Directory) = 456 on Linux x86-64 with libuv 1.51.
// Directory contains uv_fs_t (440 bytes) + uv_dirent_t (16 bytes).
// This must match the C struct layout; guarded by the static assertion below.
const DIRECTORY_SIZE: usize = 456;

/// Matches C `garray_T`.
#[repr(C)]
pub struct GarrayT {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

/// Callback type matching C `CheckItem`.
/// Returns < 0 to abort, 0 to skip, > 0 to include.
type CheckItem = Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> i64>;

extern "C" {
    fn os_scandir(dir: *mut c_void, path: *const c_char) -> bool;
    fn os_scandir_next(dir: *mut c_void) -> *const c_char;
    fn os_closedir(dir: *mut c_void);
    fn os_isrealdir(name: *const c_char) -> bool;
    fn os_remove(path: *const c_char) -> c_int;
    fn os_rmdir(path: *const c_char) -> c_int;

    fn ga_init(gap: *mut GarrayT, itemsize: c_int, growsize: c_int);
    fn ga_grow(gap: *mut GarrayT, n: c_int);
    fn ga_clear_strings(gap: *mut GarrayT);
    fn sort_strings(files: *mut c_void, count: c_int);

    fn smsg(priority: c_int, fmt: *const c_char, ...) -> c_int;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn vim_snprintf(buf: *mut c_char, buflen: usize, fmt: *const c_char, ...) -> c_int;

    static mut NameBuff: [c_char; 4096];
}

// OK/FAIL constants matching Neovim C convention.
const OK: c_int = 1;
const FAIL: c_int = 0;

// NUL char
const NUL: u8 = 0;

/// Read the contents of a directory into `gap`.
///
/// The `checkitem` callback (if non-null) is called for each entry;
/// entries where it returns <= 0 are skipped; < 0 aborts the scan.
///
/// Directly replaces the C `readdir_core` symbol.
///
/// # Safety
/// - `gap` must point to a valid uninitialized (or zeroed) `garray_T`.
/// - `path` must be a valid NUL-terminated string.
/// - `context` and `checkitem` are passed through to the callback.
#[export_name = "readdir_core"]
pub unsafe extern "C" fn rs_readdir_core(
    gap: *mut GarrayT,
    path: *const c_char,
    context: *mut c_void,
    checkitem: CheckItem,
) -> c_int {
    // sizeof(char *) = pointer size (8 on 64-bit platforms)
    unsafe { ga_init(gap, std::mem::size_of::<*mut c_char>() as c_int, 20) };

    // Allocate an opaque Directory on the heap (avoids unsafe stack layout assumptions).
    let mut dir_buf = Box::new([0u8; DIRECTORY_SIZE]);
    let dir_ptr = dir_buf.as_mut_ptr() as *mut c_void;

    if !unsafe { os_scandir(dir_ptr, path) } {
        let e_notopen = c"E484: Can't open file %s".as_ptr();
        unsafe { smsg(0, e_notopen, path) };
        return FAIL;
    }

    loop {
        let p = unsafe { os_scandir_next(dir_ptr) };
        if p.is_null() {
            break;
        }

        // Skip "." and ".."
        let p_bytes = p as *const u8;
        let b0 = unsafe { *p_bytes };
        let ignore = if b0 == b'.' {
            let b1 = unsafe { *p_bytes.add(1) };
            b1 == NUL || (b1 == b'.' && unsafe { *p_bytes.add(2) } == NUL)
        } else {
            false
        };

        let mut ignore = ignore;
        if !ignore {
            if let Some(cb) = checkitem {
                let r = unsafe { cb(context, p) };
                if r < 0 {
                    break;
                }
                if r == 0 {
                    ignore = true;
                }
            }
        }

        if !ignore {
            unsafe { ga_grow(gap, 1) };
            let slot = (unsafe { (*gap).ga_data } as *mut *mut c_char)
                .add(unsafe { (*gap).ga_len } as usize);
            unsafe { *slot = xstrdup(p) };
            unsafe { (*gap).ga_len += 1 };
        }
    }

    unsafe { os_closedir(dir_ptr) };

    if unsafe { (*gap).ga_len } > 0 {
        unsafe { sort_strings((*gap).ga_data, (*gap).ga_len) };
    }

    OK
}

/// Delete `name` and everything in it, recursively.
///
/// Directly replaces the C `delete_recursive` symbol.
///
/// # Safety
/// `name` must be a valid NUL-terminated path string.
#[export_name = "delete_recursive"]
pub unsafe extern "C" fn rs_delete_recursive(name: *const c_char) -> c_int {
    let mut result: c_int = 0;

    if unsafe { os_isrealdir(name) } {
        let exp = unsafe { xstrdup(name) };
        let mut ga = GarrayT {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: std::ptr::null_mut(),
        };

        if unsafe { rs_readdir_core(&raw mut ga, exp, std::ptr::null_mut(), None) } == OK {
            for i in 0..ga.ga_len as usize {
                let entry = unsafe { *(ga.ga_data as *mut *mut c_char).add(i) };
                // Build full path: exp + "/" + entry -> NameBuff
                let namebuff = std::ptr::addr_of_mut!(NameBuff) as *mut c_char;
                unsafe { vim_snprintf(namebuff, 4096, c"%s/%s".as_ptr(), exp, entry) };
                if unsafe { rs_delete_recursive(namebuff as *const c_char) } != 0 {
                    result = -1;
                }
            }
            unsafe { ga_clear_strings(&raw mut ga) };
            if unsafe { os_rmdir(exp) } != 0 {
                result = -1;
            }
        } else {
            result = -1;
        }

        unsafe { xfree(exp as *mut c_void) };
    } else {
        // Delete symlink only.
        result = if unsafe { os_remove(name) } == 0 {
            0
        } else {
            -1
        };
    }

    result
}
