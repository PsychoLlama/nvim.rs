//! Core helper functions for command history
//!
//! Includes history name table, entry free/clear, type lookup, and index calculation.

use std::ffi::{c_char, c_int};

use crate::ffi;
use crate::{HIST_COUNT, HIST_DEFAULT, HIST_INVALID};

// =============================================================================
// History Names Table
// =============================================================================

/// History name table used by `:history` command and `hist*()` functions.
/// Entries correspond to HIST_CMD..HIST_DEBUG (indices 0..4).
/// The last entry is an empty sentinel (like C's NULL terminator).
pub(crate) const HISTORY_NAMES: [&[u8]; 6] = [
    b"cmd\0",
    b"search\0",
    b"expr\0",
    b"input\0",
    b"debug\0",
    b"\0", // sentinel
];

// =============================================================================
// hist_free_entry / clear_hist_entry
// =============================================================================

/// Free the string and additional data of a history entry, then clear it.
///
/// # Safety
/// `hisptr` must be a valid pointer to a `histentry_T`.
#[allow(dead_code)] // Used in Phase 2+
pub(crate) unsafe fn hist_free_entry(hisptr: ffi::HistEntryPtr) {
    let hisstr = ffi::nvim_cmdhist_he_get_hisstr(hisptr);
    if !hisstr.is_null() {
        ffi::nvim_cmdhist_xfree(hisstr.cast());
    }
    let ad = ffi::nvim_cmdhist_he_get_additional_data(hisptr);
    if !ad.is_null() {
        ffi::nvim_cmdhist_xfree(ad);
    }
    ffi::nvim_cmdhist_he_clear(hisptr);
}

/// Zero-fill a history entry (equivalent to CLEAR_POINTER).
///
/// # Safety
/// `hisptr` must be a valid pointer to a `histentry_T`.
#[allow(dead_code)] // Used in Phase 3+
pub(crate) unsafe fn clear_hist_entry(hisptr: ffi::HistEntryPtr) {
    ffi::nvim_cmdhist_he_clear(hisptr);
}

// =============================================================================
// get_histtype
// =============================================================================

/// Convert history name to its HIST_ equivalent.
///
/// When `name` is empty returns currently active history or `HIST_DEFAULT`,
/// depending on `return_default` argument.
///
/// # Safety
/// `name` must be a valid pointer to at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_get_histtype(
    name: *const c_char,
    len: usize,
    return_default: c_int,
) -> c_int {
    // No argument: use current history.
    if len == 0 {
        return if return_default != 0 {
            HIST_DEFAULT
        } else {
            crate::rs_hist_char2type(ffi::nvim_cmdhist_get_cmdline_firstc())
        };
    }

    // Check against history_names table
    for i in 0..HIST_COUNT {
        let hist_name = HISTORY_NAMES[i as usize].as_ptr().cast::<c_char>();
        if ffi::nvim_cmdhist_strnicmp(name, hist_name, len) == 0 {
            return i;
        }
    }

    // Check single-character shortcuts
    let first_byte = *name as u8;
    if len == 1 {
        let shortcuts = b":=@>?/\0";
        if !ffi::nvim_cmdhist_vim_strchr(shortcuts.as_ptr().cast(), c_int::from(first_byte))
            .is_null()
        {
            return crate::rs_hist_char2type(c_int::from(first_byte));
        }
    }

    HIST_INVALID
}

// =============================================================================
// get_history_idx
// =============================================================================

/// Get identifier of newest history entry.
///
/// # Safety
/// Accesses C history arrays via FFI.
#[allow(dead_code)] // Used in Phase 4+
pub(crate) unsafe fn get_history_idx(histype: c_int) -> c_int {
    let hislen = ffi::nvim_get_hislen();
    if hislen == 0 || !(0..HIST_COUNT).contains(&histype) {
        return -1;
    }
    let idx = *ffi::get_hisidx(histype);
    if idx < 0 {
        return -1;
    }

    let hist = ffi::get_histentry(histype);
    let entry = ffi::nvim_cmdhist_he_at(hist, idx);
    ffi::nvim_cmdhist_he_get_hisnum(entry)
}

// =============================================================================
// calc_hist_idx
// =============================================================================

/// Calculate history index from a number.
///
/// `num > 0`: seen as identifying number of a history entry.
/// `num < 0`: relative position in history wrt newest entry.
///
/// # Safety
/// Accesses C history arrays via FFI.
#[allow(dead_code)] // Used in Phase 3+
pub(crate) unsafe fn calc_hist_idx(histype: c_int, num: c_int) -> c_int {
    let hislen = ffi::nvim_get_hislen();
    if hislen == 0 || !(0..HIST_COUNT).contains(&histype) || num == 0 {
        return -1;
    }
    let mut i = *ffi::get_hisidx(histype);
    if i < 0 {
        return -1;
    }

    let hist = ffi::get_histentry(histype);

    if num > 0 {
        let mut wrapped = false;
        while ffi::nvim_cmdhist_he_get_hisnum(ffi::nvim_cmdhist_he_at(hist, i)) > num {
            i -= 1;
            if i < 0 {
                if wrapped {
                    break;
                }
                i += hislen;
                wrapped = true;
            }
        }
        if i >= 0 {
            let entry = ffi::nvim_cmdhist_he_at(hist, i);
            if ffi::nvim_cmdhist_he_get_hisnum(entry) == num
                && !ffi::nvim_cmdhist_he_get_hisstr(entry).is_null()
            {
                return i;
            }
        }
    } else if -num <= hislen {
        i += num + 1;
        if i < 0 {
            i += hislen;
        }
        let entry = ffi::nvim_cmdhist_he_at(hist, i);
        if !ffi::nvim_cmdhist_he_get_hisstr(entry).is_null() {
            return i;
        }
    }
    -1
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_names_count() {
        // 5 valid names + 1 sentinel
        assert_eq!(HISTORY_NAMES.len(), 6);
    }

    #[test]
    fn test_history_names_content() {
        assert_eq!(HISTORY_NAMES[0], b"cmd\0");
        assert_eq!(HISTORY_NAMES[1], b"search\0");
        assert_eq!(HISTORY_NAMES[2], b"expr\0");
        assert_eq!(HISTORY_NAMES[3], b"input\0");
        assert_eq!(HISTORY_NAMES[4], b"debug\0");
        assert_eq!(HISTORY_NAMES[5], b"\0");
    }

    #[test]
    fn test_history_names_nul_terminated() {
        for name in &HISTORY_NAMES {
            assert_eq!(*name.last().unwrap(), 0u8, "name must be NUL-terminated");
        }
    }
}
