//! File and directory completion expansion.
//!
//! Ports `expand_files_and_dirs` and `globpath` from `cmdexpand.c` to Rust.

#![allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]

use libc::{c_char, c_int};
use std::ffi::c_void;

use crate::context::backslash::{XP_BS_COMMA, XP_BS_NONE, XP_BS_ONE, XP_BS_THREE};
use crate::context::ew_flags::{EW_CDPATH, EW_DIR, EW_FILE, EW_ICASE, EW_PATH};
use crate::context::wild_options::WILD_ICASE;
use crate::context::wild_options::WILD_SILENT;
use crate::ExpandT;

/// Growing array matching C `garray_T` layout (24 bytes on 64-bit).
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct GArray {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

extern "C" {
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_char;
    fn xmemcpyz(dst: *mut c_char, src: *const c_char, len: usize);
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

    // globpath helpers
    fn ExpandInit(xp: *mut ExpandT);
    fn ExpandFromContext(
        xp: *mut ExpandT,
        pat: *mut c_char,
        matches: *mut *mut *mut c_char,
        num_matches: *mut c_int,
        options: c_int,
    ) -> c_int;
    fn rs_expand_escape(
        xp: *mut ExpandT,
        str_: *mut c_char,
        numfiles: c_int,
        files: *mut *mut c_char,
        options: c_int,
    );
    fn ga_grow(gap: *mut GArray, n: c_int);
    fn after_pathsep(b: *const c_char, p: *const c_char) -> c_int;
    fn copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;
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

// =============================================================================
// globpath -- migrated from cmdexpand.c
// =============================================================================

/// Maximum path length (`MAXPATHL`).
const MAXPATHL: usize = 4096;
/// Path separator string on Linux (same as `PATHSEPSTR`).
const PATHSEPSTR: &[u8] = b"/";

/// Expand `file` for all comma-separated directories in `path`.
/// Appends matches to `ga`. If `dirs` is true only expand directory names.
///
/// Direct replacement for C `globpath` (exported by name).
///
/// # Safety
///
/// `path` and `file` must be valid non-null C strings.
/// `ga` must be a valid pointer to an initialized `garray_T`.
#[unsafe(export_name = "globpath")]
pub unsafe extern "C" fn rs_globpath(
    path: *mut c_char,
    file: *mut c_char,
    ga_ptr: *mut c_void,
    expand_options: c_int,
    dirs: bool,
) {
    let ga = ga_ptr.cast::<GArray>();
    let buf = xmalloc(MAXPATHL);

    let mut xpc = ExpandT::zeroed();
    ExpandInit(&raw mut xpc);
    xpc.xp_context = if dirs {
        crate::context::ExpandContext::Directories as c_int
    } else {
        crate::context::ExpandContext::Files as c_int
    };

    let filelen = libc::strlen(file);

    // Loop over all comma-separated entries in `path`.
    let mut path = path;
    while *path != 0 {
        let pathlen = copy_option_part(&raw mut path, buf, MAXPATHL, c",".as_ptr());

        // Add separator if the buf doesn't already end with one.
        let seplen = if *buf != 0 && after_pathsep(buf, buf.add(pathlen)) == 0 {
            PATHSEPSTR.len()
        } else {
            0
        };

        if pathlen + seplen + filelen < MAXPATHL {
            if seplen > 0 {
                xmemcpyz(buf.add(pathlen), PATHSEPSTR.as_ptr().cast(), seplen);
            }
            xmemcpyz(buf.add(pathlen + seplen), file, filelen);

            let mut p: *mut *mut c_char = std::ptr::null_mut();
            let mut num_p: c_int = 0;
            ExpandFromContext(
                &raw mut xpc,
                buf,
                &raw mut p,
                &raw mut num_p,
                WILD_SILENT | expand_options,
            );
            if num_p > 0 {
                rs_expand_escape(&raw mut xpc, buf, num_p, p, WILD_SILENT | expand_options);

                ga_grow(ga, num_p);
                for i in 0..num_p as usize {
                    let slot = (*ga)
                        .ga_data
                        .cast::<*mut c_char>()
                        .add((*ga).ga_len as usize);
                    *slot = *p.add(i);
                    (*ga).ga_len += 1;
                }
                xfree(p.cast::<c_void>());
            }
        }
    }

    xfree(buf.cast::<c_void>());
}
