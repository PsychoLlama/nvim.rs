//! History deletion functions
//!
//! del_history_entry (regex-based), del_history_idx (index-based)

use std::ffi::{c_char, c_int};

use crate::ffi;
use crate::helpers::{calc_hist_idx, clear_hist_entry, hist_free_entry};
use crate::{HIST_COUNT, HIST_SEARCH};

/// RE_MAGIC + RE_STRING flags (verified by _Static_assert in C).
const RE_MAGIC: c_int = 1;
const RE_STRING: c_int = 2;

// =============================================================================
// del_history_entry
// =============================================================================

/// Remove all entries matching `str` (regex) from a history.
///
/// # Safety
/// `str` must be a valid C string. Accesses C history arrays via FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_del_history_entry(histype: c_int, str: *const c_char) -> c_int {
    let hislen = ffi::nvim_get_hislen();
    if hislen == 0
        || !(0..HIST_COUNT).contains(&histype)
        || *str == 0
        || *ffi::get_hisidx(histype) < 0
    {
        return 0;
    }

    let idx = *ffi::get_hisidx(histype);
    let rm = ffi::nvim_cmdhist_regcomp(str, RE_MAGIC + RE_STRING);
    if rm.is_null() {
        return 0;
    }

    let hist = ffi::get_histentry(histype);
    let mut found = false;
    let mut i = idx;
    let mut last = idx;

    loop {
        let hisptr = ffi::nvim_cmdhist_he_at(hist, i);
        let hisstr = ffi::nvim_cmdhist_he_get_hisstr(hisptr);
        if hisstr.is_null() {
            break;
        }
        if ffi::nvim_cmdhist_regexec(rm, hisstr) != 0 {
            found = true;
            hist_free_entry(hisptr);
        } else {
            if i != last {
                ffi::nvim_cmdhist_he_copy(ffi::nvim_cmdhist_he_at(hist, last), hisptr);
                clear_hist_entry(hisptr);
            }
            last -= 1;
            if last < 0 {
                last += hislen;
            }
        }
        i -= 1;
        if i < 0 {
            i += hislen;
        }
        if i == idx {
            break;
        }
    }

    let top_entry = ffi::nvim_cmdhist_he_at(hist, idx);
    if ffi::nvim_cmdhist_he_get_hisstr(top_entry).is_null() {
        *ffi::get_hisidx(histype) = -1;
    }

    ffi::nvim_cmdhist_regfree(rm);
    c_int::from(found)
}

// =============================================================================
// del_history_idx
// =============================================================================

/// Remove an indexed entry from a history.
///
/// # Safety
/// Accesses C history arrays via FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_del_history_idx(histype: c_int, idx: c_int) -> c_int {
    let mut i = calc_hist_idx(histype, idx);
    if i < 0 {
        return 0;
    }
    let hislen = ffi::nvim_get_hislen();
    let hisidx = *ffi::get_hisidx(histype);
    let hist = ffi::get_histentry(histype);

    hist_free_entry(ffi::nvim_cmdhist_he_at(hist, i));

    // When deleting the last added search string in a mapping, reset
    // last_maptick, so that the last added search string isn't deleted again.
    if histype == HIST_SEARCH
        && ffi::nvim_cmdhist_get_maptick() == crate::modify::nvim_cmdhist_get_last_maptick()
        && i == hisidx
    {
        crate::modify::nvim_cmdhist_set_last_maptick(-1);
    }

    while i != hisidx {
        let j = (i + 1) % hislen;
        ffi::nvim_cmdhist_he_copy(
            ffi::nvim_cmdhist_he_at(hist, i),
            ffi::nvim_cmdhist_he_at(hist, j),
        );
        i = j;
    }
    clear_hist_entry(ffi::nvim_cmdhist_he_at(hist, hisidx));
    i -= 1;
    if i < 0 {
        i += hislen;
    }
    *ffi::get_hisidx(histype) = i;
    1
}
