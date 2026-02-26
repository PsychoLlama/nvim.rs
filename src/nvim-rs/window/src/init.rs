//! Window initialization during split/tabpage creation.
//!
//! This module provides the Rust implementation of `win_init`, which copies
//! window state (cursor, topline, tags, folds, options, arglist) from an
//! existing window to a new window during split or tabpage creation.

#![allow(clippy::cast_possible_truncation)]

use std::ffi::c_int;

use crate::WinHandle;

// WSP_NEWLOC flag: don't copy the location list.
const WSP_NEWLOC: c_int = 0x100;

// =============================================================================
// External C accessor functions needed for win_init
// =============================================================================

extern "C" {
    /// Copy buffer link: dst->w_buffer = src->w_buffer, dst->w_s = &src->w_buffer->b_s,
    /// src->w_buffer->b_nwindows++.
    fn nvim_win_copy_buffer_link(dst: WinHandle, src: WinHandle);

    /// Copy cursor position: dst->w_cursor = src->w_cursor.
    fn nvim_win_copy_cursor(dst: WinHandle, src: WinHandle);

    /// Set w_valid to 0.
    fn nvim_win_set_valid(wp: WinHandle, val: c_int);

    /// Get w_curswant.
    fn nvim_win_get_curswant(wp: WinHandle) -> c_int;

    /// Set w_curswant.
    fn nvim_win_set_curswant(wp: WinHandle, val: c_int);

    /// Get w_set_curswant.
    fn nvim_win_get_set_curswant(wp: WinHandle) -> c_int;

    /// Set w_set_curswant.
    fn nvim_win_set_set_curswant(wp: WinHandle, val: c_int);

    /// Get w_topline.
    fn nvim_win_get_topline(wp: WinHandle) -> c_int;

    /// Set w_topline.
    fn nvim_win_set_topline(wp: WinHandle, val: c_int);

    /// Get w_topfill.
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;

    /// Set w_topfill.
    fn nvim_win_set_topfill(wp: WinHandle, val: c_int);

    /// Get w_leftcol.
    fn nvim_win_get_leftcol(wp: WinHandle) -> c_int;

    /// Set w_leftcol.
    fn nvim_win_set_leftcol(wp: WinHandle, val: c_int);

    /// Copy pcmarks: dst->w_pcmark = src->w_pcmark, dst->w_prev_pcmark = src->w_prev_pcmark.
    fn nvim_win_copy_pcmarks(dst: WinHandle, src: WinHandle);

    /// Get w_alt_fnum.
    fn nvim_win_get_alt_fnum(wp: WinHandle) -> c_int;

    /// Set w_alt_fnum.
    fn nvim_win_set_alt_fnum(wp: WinHandle, val: c_int);

    /// Get w_wrow.
    fn nvim_win_get_wrow(wp: WinHandle) -> c_int;

    /// Set w_wrow.
    fn nvim_win_set_wrow(wp: WinHandle, val: c_int);

    /// Get w_fraction.
    fn nvim_win_get_fraction(wp: WinHandle) -> c_int;

    /// Set w_fraction.
    fn nvim_win_set_fraction(wp: WinHandle, val: c_int);

    /// Get w_prev_fraction_row.
    fn nvim_win_get_prev_fraction_row(wp: WinHandle) -> c_int;

    /// Set w_prev_fraction_row.
    fn nvim_win_set_prev_fraction_row(wp: WinHandle, val: c_int);

    /// Wrap copy_jumplist(old, new).
    fn nvim_copy_jumplist_wrapper(old: WinHandle, new: WinHandle);

    /// Wrap copy_loclist_stack(old, new).
    fn nvim_copy_loclist_stack_wrapper(old: WinHandle, new: WinHandle);

    /// Set w_llist = NULL, w_llist_ref = NULL.
    fn nvim_win_clear_loclist(wp: WinHandle);

    /// Copy localdir strings: xstrdup of w_localdir and w_prevdir (handles NULL).
    fn nvim_win_copy_localdir(dst: WinHandle, src: WinHandle);

    /// Get *p_spk character: 'c' = cursor, 's' = screen, 't' = topline.
    fn nvim_win_get_p_spk_char() -> c_int;

    /// Get w_skipcol.
    fn nvim_win_get_skipcol(wp: WinHandle) -> c_int;

    /// Set w_skipcol.
    fn nvim_win_set_skipcol(wp: WinHandle, val: c_int);

    /// Get w_botline.
    fn nvim_win_get_botline(wp: WinHandle) -> c_int;

    /// Set w_botline.
    fn nvim_win_set_botline(wp: WinHandle, val: c_int);

    /// Get w_height.
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;

    /// Set w_prev_height.
    fn nvim_win_set_prev_height(wp: WinHandle, val: c_int);

    /// Get w_winrow.
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;

    /// Set w_prev_winrow.
    fn nvim_win_set_prev_winrow(wp: WinHandle, val: c_int);

    /// Copy entire tagstack with xstrdup of tagname/user_data strings.
    fn nvim_win_copy_tagstack(dst: WinHandle, src: WinHandle);

    /// Get w_changelistidx.
    fn nvim_win_get_changelistidx(wp: WinHandle) -> c_int;

    /// Set w_changelistidx.
    fn nvim_win_set_changelistidx(wp: WinHandle, val: c_int);

    /// Call rs_copyFoldingState(old, new).
    fn nvim_copy_folding_state_wrapper(old: WinHandle, new: WinHandle);

    /// Copy alist: dst->w_alist = src->w_alist; al_refcount++; dst->w_arg_idx = src->w_arg_idx.
    fn nvim_win_copy_alist(dst: WinHandle, src: WinHandle);

    /// Wrap win_copy_options(old, new).
    fn nvim_win_copy_options_wrapper(old: WinHandle, new: WinHandle);

    /// Get w_winbar_height.
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;

    /// Set w_winbar_height.
    fn nvim_win_set_winbar_height(wp: WinHandle, val: c_int);
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
        // Copy buffer link (w_buffer, w_s, b_nwindows++)
        nvim_win_copy_buffer_link(newp, oldp);

        // Copy cursor position
        nvim_win_copy_cursor(newp, oldp);

        // Reset w_valid
        nvim_win_set_valid(newp, 0);

        // Copy scalar view fields
        nvim_win_set_curswant(newp, nvim_win_get_curswant(oldp));
        nvim_win_set_set_curswant(newp, nvim_win_get_set_curswant(oldp));
        nvim_win_set_topline(newp, nvim_win_get_topline(oldp));
        nvim_win_set_topfill(newp, nvim_win_get_topfill(oldp));
        nvim_win_set_leftcol(newp, nvim_win_get_leftcol(oldp));

        // Copy pcmarks (w_pcmark, w_prev_pcmark)
        nvim_win_copy_pcmarks(newp, oldp);

        // Copy alternate file number
        nvim_win_set_alt_fnum(newp, nvim_win_get_alt_fnum(oldp));

        // Copy w_wrow, w_fraction, w_prev_fraction_row
        nvim_win_set_wrow(newp, nvim_win_get_wrow(oldp));
        nvim_win_set_fraction(newp, nvim_win_get_fraction(oldp));
        nvim_win_set_prev_fraction_row(newp, nvim_win_get_prev_fraction_row(oldp));

        // Copy jumplist
        nvim_copy_jumplist_wrapper(oldp, newp);

        // Location list: either skip (WSP_NEWLOC) or copy
        if (flags & WSP_NEWLOC) != 0 {
            nvim_win_clear_loclist(newp);
        } else {
            nvim_copy_loclist_stack_wrapper(oldp, newp);
        }

        // Copy local directory strings (handles NULL)
        nvim_win_copy_localdir(newp, oldp);

        // Handle splitkeep fields
        let p_spk = nvim_win_get_p_spk_char();
        if p_spk != c_int::from(b'c') {
            if p_spk == c_int::from(b't') {
                nvim_win_set_skipcol(newp, nvim_win_get_skipcol(oldp));
            }
            nvim_win_set_botline(newp, nvim_win_get_botline(oldp));
            // w_prev_height = oldp->w_height
            nvim_win_set_prev_height(newp, nvim_win_get_w_height(oldp));
            // w_prev_winrow = oldp->w_winrow
            nvim_win_set_prev_winrow(newp, nvim_win_get_winrow(oldp));
        }

        // Copy tagstack (compound: copies array with xstrdup of strings)
        nvim_win_copy_tagstack(newp, oldp);

        // Copy changelist position
        nvim_win_set_changelistidx(newp, nvim_win_get_changelistidx(oldp));

        // Copy folding state (calls rs_copyFoldingState via C wrapper)
        nvim_copy_folding_state_wrapper(oldp, newp);

        // Copy alist (w_alist, al_refcount++, w_arg_idx)
        nvim_win_copy_alist(newp, oldp);

        // Copy window options
        nvim_win_copy_options_wrapper(oldp, newp);

        // Copy winbar height
        nvim_win_set_winbar_height(newp, nvim_win_get_winbar_height(oldp));
    }
}

/// FFI export for `win_init`.
///
/// Signature: `void rs_win_init(win_T *newp, win_T *oldp, int flags)`
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_init(newp: WinHandle, oldp: WinHandle, flags: c_int) {
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
