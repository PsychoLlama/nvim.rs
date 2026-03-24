//! Wildmenu key translation and dispatching for command-line completion.
//!
//! Provides `wildmenu_translate_key` and `wildmenu_process_key`.

use libc::c_int;

use crate::context::ExpandContext;
use crate::ExpandHandle;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    /// Get the correct `K_*` key code values from C (avoids hardcoding).
    fn nvim_cmdexpand_get_key_left() -> c_int;
    fn nvim_cmdexpand_get_key_right() -> c_int;
    fn nvim_cmdexpand_get_key_down() -> c_int;
    fn nvim_cmdexpand_get_key_kenter() -> c_int;

    /// Read the `wild_menu_showing` static variable (0, `WM_SHOWN`=1, `WM_SCROLLED`=2).
    fn nvim_get_wild_menu_showing() -> c_int;

    /// Check if cmdline PUM is active (`compl_match_array != NULL`).
    fn nvim_get_compl_match_array_not_null() -> c_int;

    /// Call the static C helper `wildmenu_process_key_menunames`.
    fn nvim_cmdexpand_process_key_menunames(
        key: c_int,
        xp: ExpandHandle,
        cmdpos: c_int,
        cmdbuff: *mut libc::c_char,
    ) -> c_int;

    /// Call the static C helper `wildmenu_process_key_filenames`.
    fn nvim_cmdexpand_process_key_filenames(
        key: c_int,
        xp: ExpandHandle,
        cmdpos: c_int,
        cmdbuff: *mut libc::c_char,
        cmdlen: c_int,
    ) -> c_int;

    /// Get `cmdbuff` from `get_cmdline_info()`.
    fn nvim_cmdexpand_get_cmdbuff() -> *mut libc::c_char;

    /// Get `cmdpos` from `get_cmdline_info()`.
    fn nvim_cmdexpand_get_cmdpos() -> c_int;

    /// Get `cmdlen` from `get_cmdline_info()`.
    fn nvim_cmdexpand_get_cmdlen() -> c_int;
}

// =============================================================================
// wildmenu_translate_key
// =============================================================================

/// Translate keys for the wildmenu.
///
/// Maps `K_LEFT`/`K_RIGHT` to `Ctrl_P`/`Ctrl_N` when wildmenu is visible, and maps
/// Enter keys to `K_DOWN` for menu name expansion.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `ccline` fields are read via C accessors.
#[unsafe(export_name = "wildmenu_translate_key")]
pub unsafe extern "C" fn rs_wildmenu_translate_key(
    _cclp: *mut libc::c_void,
    key: c_int,
    xp: ExpandHandle,
    did_wild_list: bool,
) -> c_int {
    // Get actual key constants from C to avoid hardcoding
    let k_left = nvim_cmdexpand_get_key_left();
    let k_right = nvim_cmdexpand_get_key_right();
    let k_down = nvim_cmdexpand_get_key_down();
    let k_kenter = nvim_cmdexpand_get_key_kenter();

    let mut c = key;
    let pum_active = nvim_get_compl_match_array_not_null() != 0;
    let wild_showing = nvim_get_wild_menu_showing() != 0;

    if pum_active || did_wild_list || wild_showing {
        if c == k_left {
            c = c_int::from(b'\x10'); // Ctrl_P
        } else if c == k_right {
            c = c_int::from(b'\x0e'); // Ctrl_N
        }
    }

    // Hitting CR after "emenu Name.": complete submenu
    if (*xp).xp_context == ExpandContext::Menunames.to_raw() {
        let cmdpos = nvim_cmdexpand_get_cmdpos();
        let cmdbuff = nvim_cmdexpand_get_cmdbuff();
        if cmdpos > 1
            && !cmdbuff.is_null()
            && *cmdbuff.add((cmdpos - 1) as usize) == b'.' as i8
            && *cmdbuff.add((cmdpos - 2) as usize) != b'\\' as i8
            && (c == c_int::from(b'\n') || c == c_int::from(b'\r') || c == k_kenter)
        {
            c = k_down;
        }
    }

    c
}

// =============================================================================
// wildmenu_process_key
// =============================================================================

/// Handle a key pressed when the wildmenu is displayed.
///
/// Dispatches to the appropriate handler based on the expansion context.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle.
#[unsafe(export_name = "wildmenu_process_key")]
pub unsafe extern "C" fn rs_wildmenu_process_key(
    _cclp: *mut libc::c_void,
    key: c_int,
    xp: ExpandHandle,
) -> c_int {
    let ctx = (*xp).xp_context;
    let cmdpos = nvim_cmdexpand_get_cmdpos();
    let cmdbuff = nvim_cmdexpand_get_cmdbuff();
    let cmdlen = nvim_cmdexpand_get_cmdlen();

    if ctx == ExpandContext::Menunames.to_raw() {
        return nvim_cmdexpand_process_key_menunames(key, xp, cmdpos, cmdbuff);
    }

    if ctx == ExpandContext::Files.to_raw()
        || ctx == ExpandContext::Directories.to_raw()
        || ctx == ExpandContext::Shellcmd.to_raw()
    {
        return nvim_cmdexpand_process_key_filenames(key, xp, cmdpos, cmdbuff, cmdlen);
    }

    key
}
