//! Popup menu (PUM) management for command-line completion.
//!
//! Provides `cmdline_pum_display`, `cmdline_pum_remove`, `cmdline_pum_cleanup`,
//! `cmdline_compl_pattern`, `cmdline_compl_is_fuzzy`, and `cmdline_pum_create`.

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

    /// Allocate `compl_match_array` with `numMatches` entries, return pointer.
    fn nvim_cmdexpand_alloc_compl_match_array(num_matches: c_int) -> *mut libc::c_void;

    /// Set `compl_match_array[i]` to all-NULL + `pum_text`=text, user attrs=-1.
    fn nvim_cmdexpand_set_pum_text(i: c_int, text: *mut c_char);

    /// Set `compl_match_arraysize`.
    fn nvim_cmdexpand_set_compl_match_arraysize(val: c_int);

    /// Set `compl_startcol`.
    fn nvim_cmdexpand_set_compl_startcol(val: c_int);

    /// Get `cmdbuff` from `get_cmdline_info()`.
    fn nvim_cmdexpand_get_cmdbuff() -> *mut c_char;

    /// Check if cmdline PUM should be used (`ui_has(kUICmdline) && cmdline_win == NULL`).
    fn nvim_get_cmdline_win_is_null() -> c_int;

    /// `ui_has(cap)` — check UI capability.
    fn ui_has(cap: c_int) -> bool;

    /// `cmd_screencol(bytepos)` — screen column for given byte position.
    fn cmd_screencol(bytepos: c_int) -> c_int;
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

// =============================================================================
// cmdline_pum_create
// =============================================================================

/// `kUICmdline` enum value from `ui_defs.h`.
const K_UI_CMDLINE: c_int = 0;

/// Create completion popup menu with items from `matches`.
///
/// Allocates and populates `compl_match_array`, then computes `compl_startcol`
/// based on the pattern position in the cmdline buffer.
///
/// # Panics
///
/// Panics if `num_matches` is negative.
///
/// # Safety
///
/// - `xp` must be a valid `expand_T` pointer.
/// - `matches` must point to at least `num_matches` valid C string pointers.
/// - Must be called from cmdline context where `get_cmdline_info()` is valid.
#[unsafe(export_name = "cmdline_pum_create")]
pub unsafe extern "C" fn rs_cmdline_pum_create(
    _ccline: *mut libc::c_void,
    xp: *mut crate::ExpandT,
    matches: *mut *mut c_char,
    num_matches: c_int,
    showtail: bool,
    noselect: bool,
) {
    assert!(num_matches >= 0);

    // Allocate and fill compl_match_array
    nvim_cmdexpand_alloc_compl_match_array(num_matches);
    nvim_cmdexpand_set_compl_match_arraysize(num_matches);

    for i in 0..num_matches {
        let text = if showtail {
            crate::helpers::rs_showmatches_gettail(*matches.add(i as usize), 0)
        } else {
            *matches.add(i as usize)
        };
        nvim_cmdexpand_set_pum_text(i, text);
    }

    // Compute popup menu starting column
    let cmdbuff = nvim_cmdexpand_get_cmdbuff();
    let endpos: *const c_char = if showtail {
        crate::helpers::rs_showmatches_gettail((*xp).xp_pattern, c_int::from(noselect))
    } else {
        (*xp).xp_pattern
    };

    let byte_offset = if cmdbuff.is_null() || endpos.is_null() {
        0
    } else {
        (endpos as usize).wrapping_sub(cmdbuff as usize) as c_int
    };

    let startcol = if ui_has(K_UI_CMDLINE) && nvim_get_cmdline_win_is_null() != 0 {
        byte_offset
    } else {
        cmd_screencol(byte_offset)
    };
    nvim_cmdexpand_set_compl_startcol(startcol);
}
