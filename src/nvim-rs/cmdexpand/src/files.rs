//! File and directory completion expansion.
//!
//! Ports `expand_files_and_dirs` from `cmdexpand.c` to Rust.

#![allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]

use libc::{c_char, c_int};
use std::ffi::c_void;

use crate::context::backslash::{XP_BS_COMMA, XP_BS_NONE, XP_BS_ONE, XP_BS_THREE};
use crate::context::ew_flags::{EW_CDPATH, EW_DIR, EW_FILE, EW_ICASE, EW_PATH};
use crate::context::wild_options::WILD_ICASE;
use crate::ExpandT;

extern "C" {
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn expand_wildcards_eval(
        pat: *mut *mut c_char,
        num_file: *mut c_int,
        file: *mut *mut *mut c_char,
        flags: c_int,
    ) -> c_int;
    fn expand_findfunc(
        pat: *mut c_char,
        files: *mut *mut *mut c_char,
        num_matches: *mut c_int,
    ) -> c_int;
}

// EXPAND context values from context.rs
use crate::context::ExpandContext;

/// Expand file/directory matches for the given pattern.
///
/// Ports the C `expand_files_and_dirs` static function to Rust.
///
/// # Safety
///
/// All pointers must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_expand_files_and_dirs(
    xp: *mut ExpandT,
    pat: *mut c_char,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
    flags: c_int,
    options: c_int,
) -> c_int {
    let xp = &mut *xp;
    let mut free_pat = false;
    let mut pat = pat;

    // Halve backslashes for escaped space (for ":set path=" and ":set tags=")
    if xp.xp_backslash != XP_BS_NONE {
        free_pat = true;
        let pat_len = libc::strlen(pat);
        pat = xstrnsave(pat, pat_len);

        let pat_end = pat.add(pat_len);
        let mut p = pat;
        while *p != 0 {
            if *p != b'\\' as c_char {
                p = p.add(1);
                continue;
            }

            if (xp.xp_backslash & XP_BS_THREE) != 0
                && *p.add(1) == b'\\' as c_char
                && *p.add(2) == b'\\' as c_char
                && *p.add(3) == b' ' as c_char
            {
                let from = p.add(3);
                let remaining = pat_end.offset_from(from) as usize + 1; // +1 for NUL
                std::ptr::copy(from, p, remaining);
                // pat_end -= 3 (not tracked as pointer, but loop continues correctly)
            } else if (xp.xp_backslash & XP_BS_ONE) != 0 && *p.add(1) == b' ' as c_char {
                let from = p.add(1);
                let remaining = pat_end.offset_from(from) as usize + 1;
                std::ptr::copy(from, p, remaining);
                // pat_end -= 1
            } else if (xp.xp_backslash & XP_BS_COMMA) != 0
                && *p.add(1) == b'\\' as c_char
                && *p.add(2) == b',' as c_char
            {
                let from = p.add(2);
                let remaining = pat_end.offset_from(from) as usize + 1;
                std::ptr::copy(from, p, remaining);
                // pat_end -= 2
            }

            p = p.add(1);
        }
    }

    let mut flags = flags;
    let ret;

    let ctx = ExpandContext::from_raw(xp.xp_context);
    if ctx == Some(ExpandContext::Findfunc) {
        ret = expand_findfunc(pat, matches, num_matches);
    } else if ctx == Some(ExpandContext::Files) {
        flags |= EW_FILE;
        if (options & WILD_ICASE) != 0 {
            flags |= EW_ICASE;
        }
        ret = expand_wildcards_eval(std::ptr::addr_of_mut!(pat), num_matches, matches, flags);
    } else if ctx == Some(ExpandContext::FilesInPath) {
        flags |= EW_FILE | EW_PATH;
        if (options & WILD_ICASE) != 0 {
            flags |= EW_ICASE;
        }
        ret = expand_wildcards_eval(std::ptr::addr_of_mut!(pat), num_matches, matches, flags);
    } else if ctx == Some(ExpandContext::DirsInCdpath) {
        flags = (flags | EW_DIR | EW_CDPATH) & !EW_FILE;
        if (options & WILD_ICASE) != 0 {
            flags |= EW_ICASE;
        }
        ret = expand_wildcards_eval(std::ptr::addr_of_mut!(pat), num_matches, matches, flags);
    } else {
        flags = (flags | EW_DIR) & !EW_FILE;
        if (options & WILD_ICASE) != 0 {
            flags |= EW_ICASE;
        }
        ret = expand_wildcards_eval(std::ptr::addr_of_mut!(pat), num_matches, matches, flags);
    }

    if free_pat {
        xfree(pat.cast::<c_void>());
    }

    ret
}
