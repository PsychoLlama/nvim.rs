//! Runtime path searching and callback dispatching
//!
//! This module implements path-based searching for runtime files with various
//! callback mechanisms to process found files.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::dip;
use crate::doso;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Path utilities
    fn path_with_extension(path: *const c_char, extension: *const c_char) -> bool;

    // Script sourcing
    fn do_source(
        fname: *mut c_char,
        check_other: bool,
        is_vimrc: c_int,
        ret_sid: *mut c_void,
    ) -> c_int;

    // Wildcard expansion
    fn gen_expand_wildcards(
        num_pat: c_int,
        pat: *mut *mut c_char,
        num_file: *mut c_int,
        file: *mut *mut *mut c_char,
        flags: c_int,
    ) -> c_int;
    fn FreeWild(count: c_int, files: *mut *mut c_char);

    // Path searching (remains in C)
    fn do_in_path(
        path: *const c_char,
        prefix: *const c_char,
        name: *mut c_char,
        flags: c_int,
        callback: Option<unsafe extern "C" fn(c_int, *mut *mut c_char, bool, *mut c_void) -> bool>,
        cookie: *mut c_void,
    ) -> c_int;

    fn do_in_cached_path(
        name: *mut c_char,
        flags: c_int,
        callback: Option<unsafe extern "C" fn(c_int, *mut *mut c_char, bool, *mut c_void) -> bool>,
        cookie: *mut c_void,
    ) -> c_int;

    // Global options
    static p_pp: *mut c_char; // packpath
    static p_rtp: *mut c_char; // runtimepath
}

// =============================================================================
// Constants
// =============================================================================

const FAIL: c_int = 0;
const OK: c_int = 1;

// =============================================================================
// Callback Functions
// =============================================================================

/// Source all .vim and .lua files in "fnames" with .vim files being sourced first.
///
/// # Safety
/// This function is called from C and must match the DoInRuntimepathCB signature.
#[no_mangle]
pub unsafe extern "C" fn rs_source_callback_vim_lua(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) -> bool {
    let mut did_one = false;

    // Source all .vim files first
    for i in 0..num_fnames {
        let fname = *fnames.add(i as usize);
        if path_with_extension(fname, c"vim".as_ptr()) {
            do_source(fname, false, doso::NONE, cookie);
            did_one = true;
            if !all {
                return true;
            }
        }
    }

    // Then source all .lua files
    for i in 0..num_fnames {
        let fname = *fnames.add(i as usize);
        if path_with_extension(fname, c"lua".as_ptr()) {
            do_source(fname, false, doso::NONE, cookie);
            did_one = true;
            if !all {
                return true;
            }
        }
    }

    did_one
}

/// Source all files in "fnames" with .vim files sourced first, .lua files
/// sourced second, and any remaining files sourced last.
///
/// # Safety
/// This function is called from C and must match the DoInRuntimepathCB signature.
#[no_mangle]
pub unsafe extern "C" fn rs_source_callback(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) -> bool {
    // First source .vim and .lua files
    let did_one = rs_source_callback_vim_lua(num_fnames, fnames, all, cookie);

    if !all && did_one {
        return true;
    }

    // Then source remaining files
    let mut did_any = did_one;
    for i in 0..num_fnames {
        let fname = *fnames.add(i as usize);
        if !path_with_extension(fname, c"vim".as_ptr())
            && !path_with_extension(fname, c"lua".as_ptr())
        {
            do_source(fname, false, doso::NONE, cookie);
            did_any = true;
            if !all {
                return true;
            }
        }
    }

    did_any
}

// =============================================================================
// Wildcard Expansion with Callback
// =============================================================================

/// Expand wildcards in "pats" and invoke callback on matches.
///
/// # Safety
/// This function calls C functions that manipulate pointers.
#[export_name = "gen_expand_wildcards_and_cb"]
pub unsafe extern "C" fn rs_gen_expand_wildcards_and_cb(
    num_pat: c_int,
    pats: *mut *mut c_char,
    flags: c_int,
    all: bool,
    callback: Option<unsafe extern "C" fn(c_int, *mut *mut c_char, bool, *mut c_void) -> bool>,
    cookie: *mut c_void,
) -> c_int {
    let mut num_files: c_int = 0;
    let mut files: *mut *mut c_char = ptr::null_mut();

    if gen_expand_wildcards(num_pat, pats, &raw mut num_files, &raw mut files, flags) != OK {
        return FAIL;
    }

    if let Some(cb) = callback {
        cb(num_files, files, all, cookie);
    }

    FreeWild(num_files, files);
    OK
}

// =============================================================================
// Path and Packpath Searching
// =============================================================================

/// Find "name" in "path" and packpath.
///
/// When found, invoke the callback function for it: callback(fname, cookie).
/// When "flags" has DIP_ALL repeat for all matches, otherwise only the first
/// one is used.
///
/// Returns OK when at least one match found, FAIL otherwise.
/// If "name" is NULL calls callback for each entry in "path". Cookie is
/// passed by reference in this case, setting it to NULL indicates that callback
/// has done its job.
///
/// # Safety
/// This function calls C functions that manipulate pointers.
#[export_name = "do_in_path_and_pp"]
pub unsafe extern "C" fn rs_do_in_path_and_pp(
    path: *mut c_char,
    name: *mut c_char,
    flags: c_int,
    callback: Option<unsafe extern "C" fn(c_int, *mut *mut c_char, bool, *mut c_void) -> bool>,
    cookie: *mut c_void,
) -> c_int {
    let mut done = FAIL;

    if (flags & dip::NORTP) == 0 {
        let search_name = if !name.is_null() && unsafe { *name } == 0 {
            ptr::null_mut()
        } else {
            name
        };
        done |= do_in_path(path, c"".as_ptr(), search_name, flags, callback, cookie);
    }

    if (done == FAIL || (flags & dip::ALL) != 0) && (flags & dip::START) != 0 {
        let prefix = if (flags & dip::AFTER) != 0 {
            c"pack/*/start/*/after/".as_ptr()
        } else {
            c"pack/*/start/*/".as_ptr()
        };
        done |= do_in_path(p_pp, prefix, name, flags & !dip::AFTER, callback, cookie);

        if done == FAIL || (flags & dip::ALL) != 0 {
            let prefix = if (flags & dip::AFTER) != 0 {
                c"start/*/after/".as_ptr()
            } else {
                c"start/*/".as_ptr()
            };
            done |= do_in_path(p_pp, prefix, name, flags & !dip::AFTER, callback, cookie);
        }
    }

    if (done == FAIL || (flags & dip::ALL) != 0) && (flags & dip::OPT) != 0 {
        done |= do_in_path(
            p_pp,
            c"pack/*/opt/*/".as_ptr(),
            name,
            flags,
            callback,
            cookie,
        );

        if done == FAIL || (flags & dip::ALL) != 0 {
            done |= do_in_path(p_pp, c"opt/*/".as_ptr(), name, flags, callback, cookie);
        }
    }

    done
}

// =============================================================================
// Runtime Path Dispatching
// =============================================================================

/// Just like do_in_path_and_pp(), using 'runtimepath' for "path".
///
/// # Safety
/// This function calls C functions that manipulate pointers.
#[export_name = "do_in_runtimepath"]
pub unsafe extern "C" fn rs_do_in_runtimepath(
    name: *mut c_char,
    flags: c_int,
    callback: Option<unsafe extern "C" fn(c_int, *mut *mut c_char, bool, *mut c_void) -> bool>,
    cookie: *mut c_void,
) -> c_int {
    let mut success = FAIL;

    let modified_flags = if (flags & dip::NORTP) == 0 {
        let search_name = if !name.is_null() && unsafe { *name } == 0 {
            ptr::null_mut()
        } else {
            name
        };
        success |= do_in_cached_path(search_name, flags, callback, cookie);
        (flags & !dip::START) | dip::NORTP
    } else {
        flags
    };

    // TODO(bfredl): we could integrate disabled OPT dirs into the cached path
    // which would effectivize ":packadd myoptpack" as well
    if (modified_flags & (dip::START | dip::OPT)) != 0
        && (success == FAIL || (modified_flags & dip::ALL) != 0)
    {
        success |= rs_do_in_path_and_pp(p_rtp, name, modified_flags, callback, cookie);
    }

    success
}

// =============================================================================
// Public API Functions
// =============================================================================

/// Source the file "name" from all directories in 'runtimepath'.
/// "name" can contain wildcards.
/// When "flags" has DIP_ALL: source all files, otherwise only the first one.
///
/// Return FAIL when no file could be sourced, OK otherwise.
///
/// # Safety
/// This function calls C functions that manipulate pointers.
#[export_name = "source_runtime"]
pub unsafe extern "C" fn rs_source_runtime(name: *mut c_char, flags: c_int) -> c_int {
    rs_do_in_runtimepath(name, flags, Some(rs_source_callback), ptr::null_mut())
}

/// Just like source_runtime(), but only source vim and lua files.
///
/// # Safety
/// This function calls C functions that manipulate pointers.
#[export_name = "source_runtime_vim_lua"]
pub unsafe extern "C" fn rs_source_runtime_vim_lua(name: *mut c_char, flags: c_int) -> c_int {
    rs_do_in_runtimepath(
        name,
        flags,
        Some(rs_source_callback_vim_lua),
        ptr::null_mut(),
    )
}

/// Just like source_runtime(), but:
/// - use "path" instead of 'runtimepath'.
/// - only source .vim and .lua files
///
/// # Safety
/// This function calls C functions that manipulate pointers.
#[export_name = "source_in_path_vim_lua"]
pub unsafe extern "C" fn rs_source_in_path_vim_lua(
    path: *mut c_char,
    name: *mut c_char,
    flags: c_int,
) -> c_int {
    rs_do_in_path_and_pp(
        path,
        name,
        flags,
        Some(rs_source_callback_vim_lua),
        ptr::null_mut(),
    )
}
