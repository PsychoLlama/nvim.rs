//! History iteration and array access functions (ShaDa integration)
//!
//! hist_iter (circular buffer iterator), hist_get_array (array accessor)

use std::ffi::{c_int, c_void};

use crate::ffi::{self, HistEntryPtr};
use crate::helpers::clear_hist_entry;

// =============================================================================
// hist_iter
// =============================================================================

/// Iterate over history items.
///
/// The iterator traverses the circular history buffer starting from the entry
/// after the most recent one (i.e., the oldest entry), wrapping around, and
/// ending at the most recent entry (`hisidx[history_type]`).
///
/// When `iter` is NULL, finds the first non-empty entry after `hisidx`.
/// When `iter` is non-NULL, it should be a pointer previously returned by
/// this function.
///
/// The `hist` output parameter is populated with a copy of the current
/// entry's data. If `zero` is true, the entry is zeroed after copying
/// (caller must call `clr_history()` after iteration completes).
///
/// Returns the next iterator pointer, or NULL when iteration is done.
///
/// # Safety
/// `hist` must be a valid pointer to a `histentry_T`. `iter`, if non-NULL,
/// must be a valid pointer previously returned by this function.
/// Accesses C history arrays via FFI.
#[export_name = "hist_iter"]
#[must_use]
pub unsafe extern "C" fn rs_hist_iter(
    iter: *const c_void,
    history_type: u8,
    zero: bool,
    hist: HistEntryPtr,
) -> *const c_void {
    // Clear output entry
    ffi::nvim_cmdhist_he_clear(hist);

    let hisidx_val = *ffi::get_hisidx(c_int::from(history_type));
    if hisidx_val == -1 {
        return std::ptr::null();
    }

    let hislen = ffi::nvim_get_hislen();
    let hist_base = ffi::get_histentry(c_int::from(history_type));

    // Compute index of the iterator entry
    let hiter_idx: c_int;
    if iter.is_null() {
        // Find first non-empty entry starting after hisidx (oldest entry)
        let mut idx = hisidx_val + 1;
        if idx >= hislen {
            idx = 0;
        }
        loop {
            let entry = ffi::nvim_cmdhist_he_at(hist_base, idx);
            if !ffi::nvim_cmdhist_he_get_hisstr(entry).is_null() {
                break;
            }
            idx += 1;
            if idx >= hislen {
                idx = 0;
            }
            if idx == (hisidx_val + 1) % hislen {
                // Wrapped all the way around without finding a non-empty entry
                break;
            }
        }
        hiter_idx = idx;
    } else {
        // Convert pointer back to index
        hiter_idx = nvim_cmdhist_ptr_to_idx(hist_base, iter.cast_mut() as HistEntryPtr);
    }

    // Copy entry to output
    let hiter_entry = ffi::nvim_cmdhist_he_at(hist_base, hiter_idx);
    ffi::nvim_cmdhist_he_copy(hist, hiter_entry);

    if zero {
        clear_hist_entry(hiter_entry);
    }

    // If we're at the last (most recent) entry, iteration is done
    if hiter_idx == hisidx_val {
        return std::ptr::null();
    }

    // Advance to next entry
    let mut next_idx = hiter_idx + 1;
    if next_idx >= hislen {
        next_idx = 0;
    }

    ffi::nvim_cmdhist_he_at(hist_base, next_idx).cast()
}

extern "C" {
    fn nvim_cmdhist_ptr_to_idx(base: HistEntryPtr, ptr: HistEntryPtr) -> c_int;
}

// =============================================================================
// hist_get_array
// =============================================================================

/// Get array of history items.
///
/// Initializes history if needed, then returns a pointer to the history array
/// along with pointers to the index and number counters.
///
/// # Safety
/// `new_hisidx` and `new_hisnum` must be valid non-null pointers to `int *`.
/// Accesses C history arrays via FFI.
#[export_name = "hist_get_array"]
#[must_use]
pub unsafe extern "C" fn rs_hist_get_array(
    history_type: u8,
    new_hisidx: *mut *mut c_int,
    new_hisnum: *mut *mut c_int,
) -> HistEntryPtr {
    crate::modify::rs_init_history();
    *new_hisidx = ffi::get_hisidx(c_int::from(history_type));
    *new_hisnum = ffi::get_hisnum(c_int::from(history_type));
    ffi::get_histentry(c_int::from(history_type))
}
