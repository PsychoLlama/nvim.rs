//! Query functions for the argument list
//!
//! Phase 5: alist_name, get_arglist_name, editing_arg_idx, check_arg_idx, arg_all

use std::ffi::{c_char, c_int, c_void};

use crate::ffi::{self, WinPtr};
use crate::K_EQUAL_FILES;

const NUL_CHAR: c_char = 0;
#[allow(clippy::cast_possible_wrap)]
const SPACE: c_char = b' ' as c_char;
#[allow(clippy::cast_possible_wrap)]
const BACKSLASH: c_char = b'\\' as c_char;
#[allow(clippy::cast_possible_wrap)]
const BACKTICK: c_char = b'`' as c_char;

// =============================================================================
// alist_name
// =============================================================================

/// Get the file name for an argument list entry.
/// Uses the name from the associated buffer if it exists.
pub(crate) unsafe fn alist_name(aep: ffi::AentryPtr) -> *mut c_char {
    let fnum = ffi::nvim_al_ae_get_fnum(aep);
    let bp = ffi::nvim_al_buflist_findnr(fnum);
    if bp.is_null() {
        return ffi::nvim_al_ae_get_fname(aep);
    }
    let fname = ffi::nvim_al_buf_get_fname(bp);
    if fname.is_null() {
        return ffi::nvim_al_ae_get_fname(aep);
    }
    fname
}

#[export_name = "alist_name"]
pub extern "C" fn rs_alist_name(aep: ffi::AentryPtr) -> *mut c_char {
    unsafe { alist_name(aep) }
}

// =============================================================================
// get_arglist_name
// =============================================================================

#[export_name = "get_arglist_name"]
pub extern "C" fn rs_get_arglist_name(_xp: *mut c_void, idx: c_int) -> *mut c_char {
    unsafe {
        if idx >= ffi::nvim_al_ARGCOUNT() {
            return std::ptr::null_mut();
        }
        let ae = ffi::nvim_al_AARGLIST(ffi::nvim_al_ALIST_curwin(), idx);
        alist_name(ae)
    }
}

// =============================================================================
// editing_arg_idx
// =============================================================================

/// Returns true if window "win" is editing the file at the current argument index.
pub(crate) unsafe fn editing_arg_idx(win: WinPtr) -> bool {
    let arg_idx = ffi::nvim_al_win_get_arg_idx(win);
    let wargcount = ffi::nvim_al_WARGCOUNT(win);
    if arg_idx >= wargcount {
        return false;
    }

    let win_alist = ffi::nvim_al_win_get_alist(win);
    let ae = ffi::nvim_al_AARGLIST(win_alist, arg_idx);
    let ae_fnum = ffi::nvim_al_ae_get_fnum(ae);

    let buf = ffi::nvim_al_win_get_buffer(win);
    let buf_fnum = ffi::nvim_al_buf_get_fnum(buf);

    if buf_fnum == ae_fnum {
        return true;
    }

    let buf_ffname = ffi::nvim_al_buf_get_ffname(buf);
    if buf_ffname.is_null() {
        return false;
    }

    let ae_name = alist_name(ae);
    (ffi::nvim_al_path_full_compare(ae_name, buf_ffname, 1, 1) & K_EQUAL_FILES) != 0
}

#[export_name = "editing_arg_idx"]
pub extern "C" fn rs_editing_arg_idx(win: WinPtr) -> c_int {
    unsafe { c_int::from(editing_arg_idx(win)) }
}

// =============================================================================
// check_arg_idx
// =============================================================================

#[export_name = "check_arg_idx"]
pub extern "C" fn rs_check_arg_idx(win: WinPtr) {
    unsafe {
        let wargcount = ffi::nvim_al_WARGCOUNT(win);
        if wargcount > 1 && !editing_arg_idx(win) {
            // Not editing the current entry in the argument list.
            ffi::nvim_al_win_set_arg_idx_invalid(win, 1);
            let arg_idx = ffi::nvim_al_win_get_arg_idx(win);
            if arg_idx != wargcount - 1
                && ffi::nvim_al_get_arg_had_last() == 0
                && ffi::nvim_al_win_get_alist(win) == ffi::nvim_al_get_global_alist()
                && ffi::nvim_al_GARGCOUNT() > 0
                && arg_idx < ffi::nvim_al_GARGCOUNT()
            {
                let gargcount = ffi::nvim_al_GARGCOUNT();
                let last_ae = ffi::nvim_al_AARGLIST(ffi::nvim_al_get_global_alist(), gargcount - 1);
                let last_fnum = ffi::nvim_al_ae_get_fnum(last_ae);
                let buf = ffi::nvim_al_win_get_buffer(win);
                let buf_fnum = ffi::nvim_al_buf_get_fnum(buf);
                let buf_ffname = ffi::nvim_al_buf_get_ffname(buf);

                if buf_fnum == last_fnum
                    || (!buf_ffname.is_null()
                        && (ffi::nvim_al_path_full_compare(alist_name(last_ae), buf_ffname, 1, 1)
                            & K_EQUAL_FILES)
                            != 0)
                {
                    ffi::nvim_al_set_arg_had_last(1);
                }
            }
        } else {
            // Editing the current entry.
            ffi::nvim_al_win_set_arg_idx_invalid(win, 0);
            let arg_idx = ffi::nvim_al_win_get_arg_idx(win);
            let wargcount = ffi::nvim_al_WARGCOUNT(win);
            if arg_idx == wargcount - 1
                && ffi::nvim_al_win_get_alist(win) == ffi::nvim_al_get_global_alist()
            {
                ffi::nvim_al_set_arg_had_last(1);
            }
        }
    }
}

// =============================================================================
// arg_all
// =============================================================================

/// Concatenate all files in the argument list, separated by spaces.
/// Spaces and backslashes in the file names are escaped with a backslash.
#[export_name = "arg_all"]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_arg_all() -> *mut c_char {
    unsafe {
        let argcount = ffi::nvim_al_ARGCOUNT();
        let al = ffi::nvim_al_ALIST_curwin();

        // First pass: compute total length
        let mut len: usize = 0;
        for idx in 0..argcount {
            let ae = ffi::nvim_al_AARGLIST(al, idx);
            let p = alist_name(ae);
            if p.is_null() {
                continue;
            }
            if len > 0 {
                len += 1; // space separator
            }
            let mut c = p;
            while *c != NUL_CHAR {
                if *c == SPACE || *c == BACKSLASH || *c == BACKTICK {
                    len += 1; // backslash escape
                }
                len += 1;
                c = c.add(1);
            }
        }

        // Allocate
        let retval = ffi::nvim_al_xmalloc(len + 1).cast::<c_char>();

        // Second pass: fill in
        let mut pos: usize = 0;
        for idx in 0..argcount {
            let ae = ffi::nvim_al_AARGLIST(al, idx);
            let p = alist_name(ae);
            if p.is_null() {
                continue;
            }
            if pos > 0 {
                *retval.add(pos) = SPACE;
                pos += 1;
            }
            let mut c = p;
            while *c != NUL_CHAR {
                if *c == SPACE || *c == BACKSLASH || *c == BACKTICK {
                    *retval.add(pos) = BACKSLASH;
                    pos += 1;
                }
                *retval.add(pos) = *c;
                pos += 1;
                c = c.add(1);
            }
        }
        *retval.add(pos) = NUL_CHAR;

        retval
    }
}
