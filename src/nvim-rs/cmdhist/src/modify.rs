//! History modification functions
//!
//! in_history, init_history, add_to_history, clr_history

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::ffi;
use crate::helpers::hist_free_entry;
use crate::{HIST_COUNT, HIST_DEFAULT, HIST_INVALID, HIST_SEARCH};

/// CMOD_KEEPPATTERNS flag value (verified by _Static_assert in C).
const CMOD_KEEPPATTERNS: c_int = 0x1000;

/// OK/FAIL return values.
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Last seen maptick (moved from C static).
static mut LAST_MAPTICK: c_int = -1;

// =============================================================================
// in_history (internal)
// =============================================================================

/// Check if command line `str` is already in history.
/// If `move_to_front` is true, matching entry is moved to end of history.
///
/// # Safety
/// Accesses C history arrays via FFI. `str` must be a valid C string.
unsafe fn in_history(
    hist_type: c_int,
    str_ptr: *const c_char,
    move_to_front: c_int,
    sep: c_int,
) -> c_int {
    let hislen = ffi::nvim_get_hislen();
    let hisidx_ptr = ffi::get_hisidx(hist_type);

    if *hisidx_ptr < 0 {
        return 0;
    }

    let hist = ffi::get_histentry(hist_type);
    let mut i = *hisidx_ptr;

    loop {
        let entry = ffi::nvim_cmdhist_he_at(hist, i);
        let hisstr = ffi::nvim_cmdhist_he_get_hisstr(entry);
        if hisstr.is_null() {
            return 0;
        }

        // For search history, check that the separator character matches as well.
        if ffi::nvim_cmdhist_strcmp(str_ptr, hisstr) == 0
            && (hist_type != HIST_SEARCH
                || sep == c_int::from(*hisstr.add(ffi::nvim_cmdhist_he_get_hisstrlen(entry) + 1)))
        {
            if move_to_front == 0 {
                return 1;
            }
            // Found a match, move to front
            break;
        }

        i -= 1;
        if i < 0 {
            i = hislen - 1;
        }
        if i == *hisidx_ptr {
            // Wrapped around without finding match
            return 0;
        }
    }

    // `i` points to the matched entry — move it to front
    let last_i_start = i;
    let ad = ffi::nvim_cmdhist_he_get_additional_data(ffi::nvim_cmdhist_he_at(hist, i));
    let save_hisstr = ffi::nvim_cmdhist_he_get_hisstr(ffi::nvim_cmdhist_he_at(hist, i));
    let save_hisstrlen = ffi::nvim_cmdhist_he_get_hisstrlen(ffi::nvim_cmdhist_he_at(hist, i));

    let mut last_i = last_i_start;
    while i != *hisidx_ptr {
        i += 1;
        if i >= hislen {
            i = 0;
        }
        ffi::nvim_cmdhist_he_copy(
            ffi::nvim_cmdhist_he_at(hist, last_i),
            ffi::nvim_cmdhist_he_at(hist, i),
        );
        last_i = i;
    }

    if !ad.is_null() {
        ffi::nvim_cmdhist_xfree(ad);
    }

    let entry = ffi::nvim_cmdhist_he_at(hist, i);
    let hisnum_ptr = ffi::get_hisnum(hist_type);
    *hisnum_ptr += 1;
    ffi::nvim_cmdhist_he_set_hisnum(entry, *hisnum_ptr);
    ffi::nvim_cmdhist_he_set_hisstr(entry, save_hisstr);
    ffi::nvim_cmdhist_he_set_hisstrlen(entry, save_hisstrlen);
    ffi::nvim_cmdhist_he_set_timestamp(entry, ffi::nvim_cmdhist_os_time());
    ffi::nvim_cmdhist_he_set_additional_data(entry, ptr::null_mut());
    1
}

// =============================================================================
// init_history
// =============================================================================

/// Initialize command line history.
/// Also used to re-allocate history tables when size changes.
///
/// # Safety
/// Accesses and modifies C history state.
#[export_name = "init_history"]
pub unsafe extern "C" fn rs_init_history() {
    let p_hi = ffi::nvim_cmdhist_get_p_hi();
    assert!(p_hi >= 0 && p_hi <= i64::from(c_int::MAX));
    #[allow(clippy::cast_possible_truncation)]
    let newlen = p_hi as c_int;
    let oldlen = ffi::nvim_get_hislen();

    if newlen == oldlen {
        return;
    }

    let entry_size = ffi::nvim_cmdhist_sizeof_histentry();

    for hist_type in 0..HIST_COUNT {
        let temp = if newlen > 0 {
            ffi::nvim_cmdhist_xmalloc(newlen as usize * entry_size).cast::<c_char>()
        } else {
            ptr::null_mut()
        };
        let temp_he = temp.cast::<std::ffi::c_void>();

        let j = *ffi::get_hisidx(hist_type);
        let hist = ffi::get_histentry(hist_type);

        if j >= 0 {
            // old array gets partitioned this way:
            // [0       , i1     ) --> newest entries to be deleted
            // [i1      , i1 + l1) --> newest entries to be copied
            // [i1 + l1 , i2     ) --> oldest entries to be deleted
            // [i2      , i2 + l2) --> oldest entries to be copied
            let l1 = c_int::min(j + 1, newlen);
            let l2 = c_int::min(newlen, oldlen) - l1;
            let i1 = j + 1 - l1;
            let i2 = c_int::max(l1, oldlen - newlen + l1);

            if newlen > 0 {
                // copy oldest entries
                ffi::nvim_cmdhist_memcpy_entries(
                    ffi::nvim_cmdhist_he_at(temp_he, 0),
                    ffi::nvim_cmdhist_he_at(hist, i2),
                    l2,
                );
                // copy newest entries
                ffi::nvim_cmdhist_memcpy_entries(
                    ffi::nvim_cmdhist_he_at(temp_he, l2),
                    ffi::nvim_cmdhist_he_at(hist, i1),
                    l1,
                );
            }

            // delete entries that don't fit in newlen
            for idx in 0..i1 {
                hist_free_entry(ffi::nvim_cmdhist_he_at(hist, idx));
            }
            for idx in (i1 + l1)..i2 {
                hist_free_entry(ffi::nvim_cmdhist_he_at(hist, idx));
            }
        }

        // clear remaining space
        let l3 = if j < 0 { 0 } else { c_int::min(newlen, oldlen) };
        if newlen > 0 {
            ffi::nvim_cmdhist_memset_entries(ffi::nvim_cmdhist_he_at(temp_he, l3), newlen - l3);
        }

        *ffi::get_hisidx(hist_type) = l3 - 1;
        ffi::nvim_cmdhist_xfree(hist);
        ffi::set_histentry(hist_type, temp_he);
    }
    ffi::nvim_cmdhist_set_hislen(newlen);
}

// =============================================================================
// add_to_history
// =============================================================================

/// Add the given string to the given history. If the string is already in the
/// history then it is moved to the front.
///
/// # Safety
/// `new_entry` must be a valid C string with at least `new_entrylen` bytes.
#[export_name = "add_to_history"]
pub unsafe extern "C" fn rs_add_to_history(
    histype: c_int,
    new_entry: *const c_char,
    new_entrylen: usize,
    in_map: bool,
    sep: c_int,
) {
    let hislen = ffi::nvim_get_hislen();
    if hislen == 0 || histype == HIST_INVALID {
        return;
    }
    assert!(histype != HIST_DEFAULT);

    if (ffi::nvim_cmdhist_get_cmdmod_cmod_flags() & CMOD_KEEPPATTERNS) != 0
        && histype == HIST_SEARCH
    {
        return;
    }

    // Searches inside the same mapping overwrite each other
    if histype == HIST_SEARCH && in_map {
        let hisidx_ptr = ffi::get_hisidx(HIST_SEARCH);
        if ffi::nvim_cmdhist_get_maptick() == LAST_MAPTICK && *hisidx_ptr >= 0 {
            let hist = ffi::get_histentry(HIST_SEARCH);
            let hisptr = ffi::nvim_cmdhist_he_at(hist, *hisidx_ptr);
            hist_free_entry(hisptr);
            let hisnum_ptr = ffi::get_hisnum(histype);
            *hisnum_ptr -= 1;
            *hisidx_ptr -= 1;
            if *hisidx_ptr < 0 {
                *hisidx_ptr = hislen - 1;
            }
        }
        LAST_MAPTICK = -1;
    }

    if in_history(histype, new_entry, 1, sep) != 0 {
        return;
    }

    let hisidx_ptr = ffi::get_hisidx(histype);
    *hisidx_ptr += 1;
    if *hisidx_ptr == hislen {
        *hisidx_ptr = 0;
    }
    let hist = ffi::get_histentry(histype);
    let hisptr = ffi::nvim_cmdhist_he_at(hist, *hisidx_ptr);
    hist_free_entry(hisptr);

    // Store the separator after the NUL of the string.
    let hisstr = ffi::nvim_cmdhist_xstrnsave(new_entry, new_entrylen + 2);
    ffi::nvim_cmdhist_he_set_hisstr(hisptr, hisstr);
    ffi::nvim_cmdhist_he_set_timestamp(hisptr, ffi::nvim_cmdhist_os_time());
    ffi::nvim_cmdhist_he_set_additional_data(hisptr, ptr::null_mut());
    *hisstr.add(new_entrylen + 1) = sep as u8 as c_char;
    ffi::nvim_cmdhist_he_set_hisstrlen(hisptr, new_entrylen);

    let hisnum_ptr = ffi::get_hisnum(histype);
    *hisnum_ptr += 1;
    ffi::nvim_cmdhist_he_set_hisnum(hisptr, *hisnum_ptr);
    if histype == HIST_SEARCH && in_map {
        LAST_MAPTICK = ffi::nvim_cmdhist_get_maptick();
    }
}

// =============================================================================
// clr_history
// =============================================================================

/// Clear all entries in a history.
///
/// # Safety
/// Accesses C history arrays via FFI.
#[export_name = "clr_history"]
#[must_use]
pub unsafe extern "C" fn rs_clr_history(histype: c_int) -> c_int {
    let hislen = ffi::nvim_get_hislen();
    if hislen != 0 && (0..HIST_COUNT).contains(&histype) {
        let hist = ffi::get_histentry(histype);
        for idx in 0..hislen {
            hist_free_entry(ffi::nvim_cmdhist_he_at(hist, idx));
        }
        *ffi::get_hisidx(histype) = -1;
        *ffi::get_hisnum(histype) = 0;
        return OK;
    }
    FAIL
}

/// Get `LAST_MAPTICK` value.
///
/// # Safety
/// Accesses static mutable.
#[no_mangle]
pub unsafe extern "C" fn nvim_cmdhist_get_last_maptick() -> c_int {
    LAST_MAPTICK
}

/// Set `LAST_MAPTICK` value.
///
/// # Safety
/// Modifies static mutable.
#[no_mangle]
pub unsafe extern "C" fn nvim_cmdhist_set_last_maptick(val: c_int) {
    LAST_MAPTICK = val;
}
