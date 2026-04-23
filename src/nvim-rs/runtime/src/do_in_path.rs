//! Rust implementations of do_in_path and do_in_cached_path.
//!
//! These functions search path lists for runtime files and invoke callbacks
//! on matches.

use std::ffi::{c_char, c_int, c_void};

use crate::constants::{EW_DIR, EW_FILE, EW_NOBREAK, MAXPATHL};
use crate::dip;
use crate::globals;

// =============================================================================
// Type aliases
// =============================================================================

type DoInRuntimepathCB =
    Option<unsafe extern "C" fn(c_int, *mut *mut c_char, bool, *mut c_void) -> bool>;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Memory management
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xmallocz(size: usize) -> *mut c_void;
    fn xfree(p: *mut c_void);

    // Path utilities
    #[link_name = "path_is_after"]
    fn nvim_rt_path_is_after(buf: *const c_char, buflen: usize) -> bool;
    #[link_name = "add_pathsep"]
    fn nvim_rt_add_pathsep(p: *mut c_char);

    // Verbose messaging
    #[link_name = "verbose_enter"]
    fn nvim_rt_verbose_enter();
    #[link_name = "verbose_leave"]
    fn nvim_rt_verbose_leave();

    // Smsg wrappers for search messages
    fn nvim_rt_smsg_searching_prefix(
        name: *const c_char,
        prefix: *const c_char,
        path: *const c_char,
    );
    fn nvim_rt_smsg_searching_in(name: *const c_char, path: *const c_char);
    fn nvim_rt_smsg_searching(buf: *const c_char);
    fn nvim_rt_semsg_dirnotf(basepath: *const c_char, name: *const c_char);
    fn nvim_rt_smsg_notfound_in(basepath: *const c_char, name: *const c_char);
    fn nvim_rt_smsg_notfound_rtp(name: *const c_char);
    fn nvim_rt_smsg_searching_rtp(name: *const c_char);

    // copy_option_part: advance pointer through comma-separated value
    #[link_name = "copy_option_part"]
    fn nvim_rt_copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    );

    // Wildcard expansion with callback (the Rust gen_expand_wildcards_and_cb)
    fn gen_expand_wildcards_and_cb(
        num_pat: c_int,
        pats: *mut *mut c_char,
        flags: c_int,
        all: bool,
        callback: DoInRuntimepathCB,
        cookie: *mut c_void,
    ) -> c_int;

    // p_rtp for comparison in error messages
    fn nvim_rt_get_p_rtp() -> *const c_char;

    // RuntimeSearchPath accessors (for do_in_cached_path)
    fn nvim_rt_rsp_get_cached_size(ref_: *mut c_int) -> usize;
    fn nvim_rt_rsp_get_item_path(idx: usize) -> *const c_char;
    fn nvim_rt_rsp_get_item_after(idx: usize) -> bool;
    fn nvim_rt_rsp_unref(ref_: *const c_int);
}

// =============================================================================
// Constants
// =============================================================================

const FAIL: c_int = 0;
const OK: c_int = 1;

// =============================================================================
// Implementation
// =============================================================================

/// Search for patterns in `name` in all directories in `path`, invoking
/// `callback(fname, cookie)` for each match.
///
/// `prefix` is prepended to each pattern in `name`.
/// When `flags` has `DIP_ALL`: source all files, otherwise only the first one.
/// When `flags` has `DIP_DIR`: find directories instead of files.
/// When `flags` has `DIP_ERR`: give an error message if there is no match.
///
/// Returns `FAIL` when no file could be sourced, `OK` otherwise.
///
/// # Safety
/// This function is called from C and manipulates raw pointers.
#[unsafe(export_name = "do_in_path")]
pub unsafe extern "C" fn rs_do_in_path(
    path: *const c_char,
    prefix: *const c_char,
    name: *mut c_char,
    flags: c_int,
    callback: DoInRuntimepathCB,
    cookie: *mut c_void,
) -> c_int {
    let mut did_one = false;

    // Make a copy of the path. Invoking the callback may change the value.
    let rtp_copy = xstrdup(path);
    let mut buf = xmallocz(MAXPATHL).cast::<c_char>();

    {
        if globals::get_p_verbose() > 10 && !name.is_null() {
            nvim_rt_verbose_enter();
            if unsafe { *prefix } != 0 {
                nvim_rt_smsg_searching_prefix(name, prefix, path);
            } else {
                nvim_rt_smsg_searching_in(name, path);
            }
            nvim_rt_verbose_leave();
        }

        let do_all = (flags & dip::ALL) != 0;

        // Loop over all entries in 'runtimepath'.
        let mut rtp = rtp_copy;
        while unsafe { *rtp } != 0 && (do_all || !did_one) {
            // Copy the path from 'runtimepath' to buf[].
            nvim_rt_copy_option_part(&raw mut rtp, buf, MAXPATHL, c",".as_ptr());
            let buflen = libc::strlen(buf.cast());

            // Skip after or non-after directories.
            if (flags & (dip::NOAFTER | dip::AFTER)) != 0 {
                let is_after = nvim_rt_path_is_after(buf, buflen);
                if (is_after && (flags & dip::NOAFTER) != 0)
                    || (!is_after && (flags & dip::AFTER) != 0)
                {
                    continue;
                }
            }

            if name.is_null() {
                if let Some(cb) = callback {
                    cb(1, &raw mut buf, do_all, cookie);
                }
                did_one = true;
            } else if buflen + 2 + libc::strlen(prefix.cast()) + libc::strlen(name.cast())
                < MAXPATHL
            {
                nvim_rt_add_pathsep(buf);
                libc::strcat(buf.cast(), prefix.cast());
                let tail_offset = libc::strlen(buf.cast());
                let tail = buf.add(tail_offset);

                // Loop over all patterns in "name"
                let mut np = name;
                while unsafe { *np } != 0 && (do_all || !did_one) {
                    // Append the pattern from "name" to buf[].
                    let remaining = MAXPATHL - tail_offset;
                    nvim_rt_copy_option_part(&raw mut np, tail, remaining, c"\t ".as_ptr());

                    if globals::get_p_verbose() > 10 {
                        nvim_rt_verbose_enter();
                        nvim_rt_smsg_searching(buf);
                        nvim_rt_verbose_leave();
                    }

                    let ew_flags = (if (flags & dip::DIR) != 0 {
                        EW_DIR
                    } else {
                        EW_FILE
                    }) | (if (flags & dip::DIRFILE) != 0 {
                        EW_DIR | EW_FILE
                    } else {
                        0
                    });

                    did_one |= gen_expand_wildcards_and_cb(
                        1,
                        &raw mut buf,
                        ew_flags,
                        do_all,
                        callback,
                        cookie,
                    ) == OK;
                }
            }
        }
    }

    xfree(buf.cast::<c_void>());
    xfree(rtp_copy.cast::<c_void>());

    if !did_one && !name.is_null() {
        let p_rtp = nvim_rt_get_p_rtp();
        let basepath = if path == p_rtp {
            c"runtimepath".as_ptr()
        } else {
            c"packpath".as_ptr()
        };

        if (flags & dip::ERR) != 0 {
            nvim_rt_semsg_dirnotf(basepath, name);
        } else if globals::get_p_verbose() > 1 {
            nvim_rt_verbose_enter();
            nvim_rt_smsg_notfound_in(basepath, name);
            nvim_rt_verbose_leave();
        }
    }

    if did_one {
        OK
    } else {
        FAIL
    }
}

/// Search for `name` in the cached runtime search path, invoking `callback`
/// for each match.
///
/// `name` can contain wildcards.
/// When "flags" has DIP_ALL: source all files, otherwise only the first one.
/// When "flags" has DIP_DIR: find directories instead of files.
/// When "flags" has DIP_ERR: give an error message if there is no match.
///
/// Return FAIL when no file could be sourced, OK otherwise.
///
/// # Safety
/// This function is called from C and manipulates raw pointers.
#[unsafe(export_name = "do_in_cached_path")]
pub unsafe extern "C" fn rs_do_in_cached_path(
    name: *mut c_char,
    flags: c_int,
    callback: DoInRuntimepathCB,
    cookie: *mut c_void,
) -> c_int {
    let mut did_one = false;

    if globals::get_p_verbose() > 10 && !name.is_null() {
        nvim_rt_verbose_enter();
        nvim_rt_smsg_searching_rtp(name);
        nvim_rt_verbose_leave();
    }

    let mut ref_: c_int = 0;
    let path_size = nvim_rt_rsp_get_cached_size(&raw mut ref_);

    let do_all = (flags & dip::ALL) != 0;

    // Stack-allocate a buffer of MAXPATHL bytes.
    // We use a Vec as a stack-like buffer to avoid a heap allocation per call.
    let mut buf_vec: Vec<u8> = vec![0u8; MAXPATHL];
    let buf = buf_vec.as_mut_ptr().cast::<c_char>();

    // Loop over all entries in cached path
    for j in 0..path_size {
        if !do_all && did_one {
            break;
        }

        let item_path = nvim_rt_rsp_get_item_path(j);
        let item_after = nvim_rt_rsp_get_item_after(j);
        let buflen = libc::strlen(item_path.cast());

        // Skip after or non-after directories.
        if (flags & (dip::NOAFTER | dip::AFTER)) != 0
            && ((item_after && (flags & dip::NOAFTER) != 0)
                || (!item_after && (flags & dip::AFTER) != 0))
        {
            continue;
        }

        if name.is_null() {
            // Callback for directory entries when name is NULL
            let mut item_path_mut = item_path.cast_mut();
            if let Some(cb) = callback {
                cb(1, &raw mut item_path_mut, do_all, cookie);
            }
        } else if buflen + libc::strlen(name.cast()) + 2 < MAXPATHL {
            libc::strcpy(buf.cast(), item_path.cast());
            nvim_rt_add_pathsep(buf);
            let tail_offset = libc::strlen(buf.cast());
            let tail = buf.add(tail_offset);

            // Loop over all patterns in "name"
            let mut np = name;
            while unsafe { *np } != 0 && (do_all || !did_one) {
                // Append the pattern from "name" to buf[].
                let remaining = MAXPATHL - tail_offset;
                nvim_rt_copy_option_part(&raw mut np, tail, remaining, c"\t ".as_ptr());

                if globals::get_p_verbose() > 10 {
                    nvim_rt_verbose_enter();
                    nvim_rt_smsg_searching(buf);
                    nvim_rt_verbose_leave();
                }

                let ew_flags = (if (flags & dip::DIR) != 0 {
                    EW_DIR
                } else {
                    EW_FILE
                }) | (if (flags & dip::DIRFILE) != 0 {
                    EW_DIR | EW_FILE
                } else {
                    0
                }) | EW_NOBREAK;

                // Expand wildcards, invoke the callback for each match.
                let mut pat_ptr = buf;
                did_one |= gen_expand_wildcards_and_cb(
                    1,
                    &raw mut pat_ptr,
                    ew_flags,
                    do_all,
                    callback,
                    cookie,
                ) == OK;
            }
        }
    }

    if !did_one && !name.is_null() {
        if (flags & dip::ERR) != 0 {
            nvim_rt_semsg_dirnotf(c"runtime path".as_ptr(), name);
        } else if globals::get_p_verbose() > 1 {
            nvim_rt_verbose_enter();
            nvim_rt_smsg_notfound_rtp(name);
            nvim_rt_verbose_leave();
        }
    }

    nvim_rt_rsp_unref(&raw const ref_);

    if did_one {
        OK
    } else {
        FAIL
    }
}
