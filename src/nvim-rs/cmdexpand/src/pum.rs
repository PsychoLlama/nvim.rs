//! Popup menu (PUM) management for command-line completion.
//!
//! Provides `cmdline_pum_display`, `cmdline_pum_remove`, `cmdline_pum_cleanup`,
//! `cmdline_compl_pattern`, `cmdline_compl_is_fuzzy`, and `cmdline_pum_create`.

use libc::{c_char, c_int};

// =============================================================================
// pumitem_T repr(C) (matches C layout, sizeof=48)
// =============================================================================

/// Popup menu item matching `pumitem_T` from `popupmenu.h` (sizeof=48).
///
/// Layout verified in `popupmenu/src/item.rs`:
/// - `pum_text`@0, `pum_kind`@8, `pum_extra`@16, `pum_info`@24 (pointers)
/// - `pum_cpt_source_idx`@32, `pum_user_abbr_hlattr`@36, `pum_user_kind_hlattr`@40, pad@44
#[repr(C)]
struct PumItem {
    pum_text: *mut c_char,
    pum_kind: *mut c_char,
    pum_extra: *mut c_char,
    pum_info: *mut c_char,
    pum_cpt_source_idx: c_int,
    pum_user_abbr_hlattr: c_int,
    pum_user_kind_hlattr: c_int,
    _pad: c_int,
}

// =============================================================================
// Rust-owned compl_* statics (formerly in cmdexpand.c)
// =============================================================================

/// Currently displayed list of entries in the popup menu (NULL when not shown).
static mut COMPL_MATCH_ARRAY: *mut PumItem = std::ptr::null_mut();
/// Number of entries in `COMPL_MATCH_ARRAY`.
static mut COMPL_MATCH_ARRAYSIZE: c_int = 0;
/// First column in cmdline of the matched item for completion.
static mut COMPL_STARTCOL: c_int = 0;
/// Currently selected completion item index.
static mut COMPL_SELECTED: c_int = 0;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    /// Run `cmdline_pum_remove(false)` + `wildmenu_cleanup(get_cmdline_info())`.
    fn nvim_cmdexpand_do_pum_cleanup();

    /// Return `get_cmdline_info()->xpc->xp_orig` or NULL if `xpc` is NULL.
    fn nvim_cmdexpand_get_compl_pattern() -> *mut c_char;

    /// Return `get_cmdline_info()->xpc` if non-null and context supports fuzzy, else 0.
    fn nvim_cmdexpand_ccline_xpc_supports_fuzzy() -> c_int;

    /// Get `cmdbuff` from `get_cmdline_info()`.
    fn nvim_cmdexpand_get_cmdbuff() -> *mut c_char;

    /// Check if cmdline PUM should be used (`ui_has(kUICmdline) && cmdline_win == NULL`).
    fn nvim_get_cmdline_win_is_null() -> c_int;

    /// `ui_has(cap)` — check UI capability.
    fn ui_has(cap: c_int) -> bool;

    /// `cmd_screencol(bytepos)` — screen column for given byte position.
    fn cmd_screencol(bytepos: c_int) -> c_int;

    /// `pum_display(array, size, selected, array_changed, cmd_startcol)`.
    fn pum_display(
        array: *mut PumItem,
        size: c_int,
        selected: c_int,
        array_changed: bool,
        cmd_startcol: c_int,
    );

    /// `pum_undisplay(force_redraw)`.
    fn pum_undisplay(force_redraw: bool);

    /// `xfree(ptr)`.
    fn xfree(ptr: *mut libc::c_void);

    /// `xmalloc(size)` — allocate memory (aborts on failure).
    fn xmalloc(size: usize) -> *mut c_char;
}

// =============================================================================
// Accessors for Rust-owned compl_* statics (called from navigation.rs, display.rs)
// =============================================================================

/// Returns non-zero if `COMPL_MATCH_ARRAY` is non-null.
#[unsafe(export_name = "nvim_get_compl_match_array_not_null")]
pub extern "C" fn rs_get_compl_match_array_not_null() -> c_int {
    // SAFETY: read of atomic-like static (single-threaded Neovim).
    c_int::from(unsafe { !COMPL_MATCH_ARRAY.is_null() })
}

/// Set `COMPL_SELECTED`.
#[unsafe(export_name = "nvim_set_compl_selected")]
pub extern "C" fn rs_set_compl_selected(val: c_int) {
    // SAFETY: single-threaded Neovim.
    unsafe { COMPL_SELECTED = val };
}

// =============================================================================
// cmdline_pum_display
// =============================================================================

/// Display the cmdline completion popup menu.
///
/// Calls `pum_display` with the Rust-owned `COMPL_*` statics.
///
/// # Safety
///
/// Must only be called when `COMPL_MATCH_ARRAY` is not NULL.
#[unsafe(export_name = "cmdline_pum_display")]
pub unsafe extern "C" fn rs_cmdline_pum_display(changed_array: bool) {
    pum_display(
        COMPL_MATCH_ARRAY,
        COMPL_MATCH_ARRAYSIZE,
        COMPL_SELECTED,
        changed_array,
        COMPL_STARTCOL,
    );
}

// =============================================================================
// cmdline_pum_remove
// =============================================================================

/// Remove the cmdline completion popup menu and free the match array.
///
/// # Safety
///
/// Safe to call even when no PUM is displayed.
#[unsafe(export_name = "cmdline_pum_remove")]
pub unsafe extern "C" fn rs_cmdline_pum_remove(defer_redraw: bool) {
    pum_undisplay(!defer_redraw);
    xfree(COMPL_MATCH_ARRAY.cast::<libc::c_void>());
    COMPL_MATCH_ARRAY = std::ptr::null_mut();
    COMPL_MATCH_ARRAYSIZE = 0;
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
/// Allocates and populates `COMPL_MATCH_ARRAY`, then computes `COMPL_STARTCOL`
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

    // Allocate and fill COMPL_MATCH_ARRAY
    let arr = xmalloc(std::mem::size_of::<PumItem>() * num_matches as usize).cast::<PumItem>();
    COMPL_MATCH_ARRAY = arr;
    COMPL_MATCH_ARRAYSIZE = num_matches;

    for i in 0..num_matches {
        let text = if showtail {
            crate::helpers::rs_showmatches_gettail(*matches.add(i as usize), 0)
        } else {
            *matches.add(i as usize)
        };
        arr.add(i as usize).write(PumItem {
            pum_text: text,
            pum_info: std::ptr::null_mut(),
            pum_extra: std::ptr::null_mut(),
            pum_kind: std::ptr::null_mut(),
            pum_cpt_source_idx: 0,
            pum_user_abbr_hlattr: -1,
            pum_user_kind_hlattr: -1,
            _pad: 0,
        });
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

    COMPL_STARTCOL = if ui_has(K_UI_CMDLINE) && nvim_get_cmdline_win_is_null() != 0 {
        byte_offset
    } else {
        cmd_screencol(byte_offset)
    };
}
