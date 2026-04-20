//! Window initialization during split/tabpage creation.
//!
//! This module provides the Rust implementation of `win_init`, which copies
//! window state (cursor, topline, tags, folds, options, arglist) from an
//! existing window to a new window during split or tabpage creation.

#![allow(clippy::cast_possible_truncation)]

use std::ffi::c_int;

use crate::win_struct::{win_mut, win_ref};
use crate::WinHandle;

// WSP_NEWLOC flag: don't copy the location list.
const WSP_NEWLOC: c_int = 0x100;

// =============================================================================
// External C accessor functions needed for win_init
// =============================================================================

extern "C" {
    /// Copy all compound init data from src to dst.
    /// flags: WSP_NEWLOC to skip location list copy.
    fn nvim_win_init_copy_compound(dst: WinHandle, src: WinHandle, flags: c_int);

    /// Copy cursor position: dst->w_cursor = src->w_cursor.
    fn nvim_win_copy_cursor(dst: WinHandle, src: WinHandle);

    /// Get *p_spk character: 'c' = cursor, 's' = screen, 't' = topline.
    fn nvim_win_get_p_spk_char() -> c_int;
}

// =============================================================================
// win_init implementation
// =============================================================================

/// Initialize window `newp` from window `oldp`.
///
/// Used when splitting a window and when creating a new tab page.
/// The windows will both edit the same buffer.
/// WSP_NEWLOC may be specified in flags to prevent the location list from
/// being copied.
///
/// This is the Rust equivalent of `win_init(newp, oldp, flags)`.
fn win_init_impl(newp: WinHandle, oldp: WinHandle, flags: c_int) {
    // SAFETY: All calls below are safe C accessor functions operating on valid window handles.
    unsafe {
        // Copy all compound data (buffer link, pcmarks, jumplist, loclist,
        // localdir, tagstack, alist, options, folding state).
        nvim_win_init_copy_compound(newp, oldp, flags);

        // Copy cursor position
        nvim_win_copy_cursor(newp, oldp);

        // Reset w_valid
        win_mut(newp).w_valid = 0;

        // Copy scalar view fields
        win_mut(newp).w_curswant = win_ref(oldp).w_curswant;
        win_mut(newp).w_set_curswant = win_ref(oldp).w_set_curswant;
        win_mut(newp).w_topline = win_ref(oldp).w_topline;
        win_mut(newp).w_topfill = win_ref(oldp).w_topfill;
        win_mut(newp).w_leftcol = win_ref(oldp).w_leftcol;

        // Copy alternate file number
        win_mut(newp).w_alt_fnum = win_ref(oldp).w_alt_fnum;

        // Copy w_wrow, w_fraction, w_prev_fraction_row
        win_mut(newp).w_wrow = win_ref(oldp).w_wrow;
        win_mut(newp).w_fraction = win_ref(oldp).w_fraction;
        win_mut(newp).w_prev_fraction_row = win_ref(oldp).w_prev_fraction_row;

        // Handle splitkeep fields
        let p_spk = nvim_win_get_p_spk_char();
        if p_spk != c_int::from(b'c') {
            if p_spk == c_int::from(b't') {
                win_mut(newp).w_skipcol = win_ref(oldp).w_skipcol;
            }
            win_mut(newp).w_botline = win_ref(oldp).w_botline;
            // w_prev_height = oldp->w_height
            win_mut(newp).w_prev_height = win_ref(oldp).w_height;
            // w_prev_winrow = oldp->w_winrow
            win_mut(newp).w_prev_winrow = win_ref(oldp).w_winrow;
        }

        // Copy changelist position
        win_mut(newp).w_changelistidx = win_ref(oldp).w_changelistidx;

        // Copy winbar height
        win_mut(newp).w_winbar_height = win_ref(oldp).w_winbar_height;
    }
}

/// FFI export for `win_init`.
///
/// Signature: `void rs_win_init(win_T *newp, win_T *oldp, int flags)`
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_init(newp: WinHandle, oldp: WinHandle, flags: c_int) {
    win_init_impl(newp, oldp, flags);
}

/// C export: `win_init` — eliminates the C thin wrapper.
#[unsafe(export_name = "win_init")]
pub extern "C" fn win_init(newp: WinHandle, oldp: WinHandle, flags: c_int) {
    win_init_impl(newp, oldp, flags);
}

// =============================================================================
// Static assertions via tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wsp_newloc_constant() {
        // WSP_NEWLOC must match the C value 0x100
        assert_eq!(WSP_NEWLOC, 0x100);
    }
}
