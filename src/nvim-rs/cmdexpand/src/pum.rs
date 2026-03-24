//! Popup menu (PUM) management for command-line completion.
//!
//! Provides `cmdline_pum_display`, `cmdline_pum_remove`, `cmdline_pum_cleanup`,
//! `cmdline_compl_pattern`, and `cmdline_compl_is_fuzzy`.

use libc::{c_char, c_int};

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    /// Run `pum_display` with the current `compl_*` statics (avoids exporting them).
    fn nvim_cmdexpand_do_pum_display(changed_array: c_int);

    /// Run `pum_undisplay` + free `compl_match_array` + reset arraysize.
    fn nvim_cmdexpand_do_pum_remove(defer_redraw: c_int);

    /// Run `cmdline_pum_remove(false)` + `wildmenu_cleanup(get_cmdline_info())`.
    fn nvim_cmdexpand_do_pum_cleanup();

    /// Return `get_cmdline_info()->xpc->xp_orig` or NULL if `xpc` is NULL.
    fn nvim_cmdexpand_get_compl_pattern() -> *mut c_char;

    /// Return `get_cmdline_info()->xpc` if non-null and context supports fuzzy, else 0.
    fn nvim_cmdexpand_ccline_xpc_supports_fuzzy() -> c_int;

}

// =============================================================================
// cmdline_pum_display
// =============================================================================

/// Display the cmdline completion popup menu.
///
/// Delegates to `pum_display` with the cached `compl_*` statics.
///
/// # Safety
///
/// Must only be called when `compl_match_array` is not NULL.
#[unsafe(export_name = "cmdline_pum_display")]
pub unsafe extern "C" fn rs_cmdline_pum_display(changed_array: bool) {
    nvim_cmdexpand_do_pum_display(c_int::from(changed_array));
}

// =============================================================================
// cmdline_pum_remove
// =============================================================================

/// Remove the cmdline completion popup menu and free the match array.
///
/// # Safety
///
/// Safe to call even when no PUM is displayed (the C wrapper is a no-op then).
#[unsafe(export_name = "cmdline_pum_remove")]
pub unsafe extern "C" fn rs_cmdline_pum_remove(defer_redraw: bool) {
    nvim_cmdexpand_do_pum_remove(c_int::from(defer_redraw));
}

// =============================================================================
// cmdline_pum_cleanup
// =============================================================================

/// Remove the PUM and clean up the wildmenu.
///
/// # Safety
///
/// Must be called from cmdline context where `get_cmdline_info()` is valid.
#[unsafe(export_name = "cmdline_pum_cleanup")]
pub unsafe extern "C" fn rs_cmdline_pum_cleanup(_cclp: *mut libc::c_void) {
    nvim_cmdexpand_do_pum_cleanup();
}

// =============================================================================
// cmdline_compl_pattern
// =============================================================================

/// Returns the current cmdline completion pattern (`xpc->xp_orig`).
///
/// Returns NULL if the `xpc` field of the current cmdline info is NULL.
///
/// # Safety
///
/// Returned pointer is valid as long as no completion re-initialization occurs.
#[must_use]
#[unsafe(export_name = "cmdline_compl_pattern")]
pub unsafe extern "C" fn rs_cmdline_compl_pattern() -> *mut c_char {
    nvim_cmdexpand_get_compl_pattern()
}

// =============================================================================
// cmdline_compl_is_fuzzy
// =============================================================================

/// Returns true if fuzzy cmdline completion is currently active.
///
/// # Safety
///
/// Must be called from cmdline context where `get_cmdline_info()` is valid.
#[must_use]
#[unsafe(export_name = "cmdline_compl_is_fuzzy")]
pub unsafe extern "C" fn rs_cmdline_compl_is_fuzzy() -> bool {
    nvim_cmdexpand_ccline_xpc_supports_fuzzy() != 0
}
