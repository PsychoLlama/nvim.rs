//! Textwidth computation for text formatting.
//!
//! This module provides functions to compute the effective textwidth
//! based on buffer options and window dimensions.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::c_int;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a window (win_T*).
pub type WinHandle = *mut std::ffi::c_void;

/// Opaque handle to a buffer (buf_T*).
pub type BufHandle = *mut std::ffi::c_void;

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    /// Get curbuf->b_p_tw (textwidth option).
    fn nvim_textfmt_get_curbuf_b_p_tw() -> c_int;

    /// Get curbuf->b_p_wm (wrapmargin option).
    fn nvim_textfmt_get_curbuf_b_p_wm() -> c_int;

    /// Get curwin->w_view_width.
    fn nvim_textfmt_get_curwin_w_view_width() -> c_int;

    /// Get curbuf pointer.
    fn nvim_textfmt_get_curbuf() -> BufHandle;

    /// Get cmdwin_buf pointer.
    fn nvim_textfmt_get_cmdwin_buf() -> BufHandle;

    /// Get curwin pointer.
    fn nvim_textfmt_get_curwin() -> WinHandle;

    /// Get fold column count for window.
    fn rs_win_fdccol_count(win: WinHandle) -> c_int;

    /// Get curwin->w_scwidth (sign column width).
    fn nvim_textfmt_get_curwin_w_scwidth() -> c_int;

    /// Get curwin->w_p_nu (number option).
    fn nvim_textfmt_get_curwin_w_p_nu() -> bool;

    /// Get curwin->w_p_rnu (relativenumber option).
    fn nvim_textfmt_get_curwin_w_p_rnu() -> bool;
}

// =============================================================================
// Textwidth Computation
// =============================================================================

/// Find out textwidth to be used for formatting.
///
/// - If 'textwidth' option is set, use it
/// - Else if 'wrapmargin' option is set, use `w_view_width - wrapmargin`
/// - If invalid value, use 0
/// - Set default to window width (maximum 79) for "gq" operator
///
/// # Arguments
/// * `ff` - Force formatting (for "gq" command)
///
/// # Returns
/// The computed textwidth.
pub(crate) unsafe fn comp_textwidth_impl(ff: bool) -> c_int {
    let mut textwidth = nvim_textfmt_get_curbuf_b_p_tw();

    if textwidth == 0 && nvim_textfmt_get_curbuf_b_p_wm() != 0 {
        // The width is the window width minus 'wrapmargin' minus all the
        // things that add to the margin.
        textwidth = nvim_textfmt_get_curwin_w_view_width() - nvim_textfmt_get_curbuf_b_p_wm();

        // Check if in command window
        if nvim_textfmt_get_curbuf() == nvim_textfmt_get_cmdwin_buf() {
            textwidth -= 1;
        }

        textwidth -= rs_win_fdccol_count(nvim_textfmt_get_curwin());
        textwidth -= nvim_textfmt_get_curwin_w_scwidth();

        if nvim_textfmt_get_curwin_w_p_nu() || nvim_textfmt_get_curwin_w_p_rnu() {
            textwidth -= 8;
        }
    }

    textwidth = textwidth.max(0);

    if ff && textwidth == 0 {
        textwidth = (nvim_textfmt_get_curwin_w_view_width() - 1).min(79);
    }

    textwidth
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Compute textwidth for formatting.
///
/// # Safety
/// Accesses global window and buffer state via C functions.
#[export_name = "comp_textwidth"]
pub unsafe extern "C" fn rs_comp_textwidth(ff: c_int) -> c_int {
    comp_textwidth_impl(ff != 0)
}

#[cfg(test)]
mod tests {
    // Integration testing is done via the full Neovim build.
}
