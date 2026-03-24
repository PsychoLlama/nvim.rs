//! Wildmenu key translation, dispatching, and cleanup for command-line completion.
//!
//! Provides `wildmenu_translate_key`, `wildmenu_process_key`, and `wildmenu_cleanup`.

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
    fn nvim_cmdexpand_get_key_up() -> c_int;
    fn nvim_cmdexpand_get_key_kenter() -> c_int;

    /// Read the `wild_menu_showing` static variable (0, `WM_SHOWN`=1, `WM_SCROLLED`=2).
    fn nvim_get_wild_menu_showing() -> c_int;

    /// Check if cmdline PUM is active (`compl_match_array != NULL`).
    fn nvim_get_compl_match_array_not_null() -> c_int;

    /// Get `cmdbuff` from `get_cmdline_info()`.
    fn nvim_cmdexpand_get_cmdbuff() -> *mut libc::c_char;

    /// Get `cmdpos` from `get_cmdline_info()`.
    fn nvim_cmdexpand_get_cmdpos() -> c_int;

    /// Get `cmdlen` from `get_cmdline_info()`.
    fn nvim_cmdexpand_get_cmdlen() -> c_int;

    /// Get `p_wc` wildchar option (for `K_UP`/`K_DOWN` handling).
    fn nvim_cmdexpand_get_p_wc() -> c_int;

    /// Delete characters from `from` to `cmdpos` on the real ccline.
    fn nvim_cmdexpand_cmdline_del(from: c_int);

    /// Set `KeyTyped` global.
    fn nvim_cmdexpand_set_key_typed(val: c_int);

    /// Get `KeyTyped` global.
    fn nvim_cmdexpand_get_key_typed() -> c_int;

    /// Wrapper for `put_on_cmdline(str, len, redraw)`.
    fn nvim_cmdexpand_put_on_cmdline(str_: *const libc::c_char, len: c_int, redraw: c_int);

    /// Wrapper for `utf_head_off(base, p)`.
    fn nvim_cmdexpand_utf_head_off(base: *const libc::c_char, p: *const libc::c_char) -> c_int;

    /// Check if character is a path separator.
    fn nvim_cmdexpand_vim_ispathsep(c: c_int) -> c_int;

    /// Get the `PATHSEP` character constant.
    fn nvim_cmdexpand_get_pathsep() -> c_int;

    // Wildmenu cleanup functions
    /// Get `p_wmnu` (wildmenu option).
    fn nvim_get_p_wmnu() -> c_int;

    /// Get `input_fn` from `get_cmdline_info()`.
    fn nvim_cmdexpand_get_input_fn() -> c_int;

    /// Get `RedrawingDisabled` global.
    fn nvim_cmdexpand_get_redrawing_disabled() -> c_int;

    /// Set `RedrawingDisabled` global.
    fn nvim_cmdexpand_set_redrawing_disabled(val: c_int);

    /// Set `no_hlsearch` via `set_no_hlsearch(val)`.
    fn nvim_cmdexpand_set_no_hlsearch(val: c_int);

    /// Get `WM_SCROLLED` constant.
    fn nvim_cmdexpand_get_wm_scrolled() -> c_int;

    /// Decrement `cmdline_row`.
    fn nvim_cmdexpand_dec_cmdline_row();

    /// Wrapper for `redrawcmd()`.
    fn nvim_cmdexpand_redrawcmd();

    /// Set `wild_menu_showing`.
    fn nvim_cmdexpand_set_wild_menu_showing(val: c_int);

    /// Get `save_p_ls`.
    fn nvim_cmdexpand_get_save_p_ls() -> c_int;

    /// Set `save_p_ls`.
    fn nvim_cmdexpand_set_save_p_ls(val: c_int);

    /// Get `save_p_wmh`.
    fn nvim_cmdexpand_get_save_p_wmh() -> c_int;

    /// Set `p_ls`.
    fn nvim_cmdexpand_set_p_ls(val: i64);

    /// Set `p_wmh`.
    fn nvim_cmdexpand_set_p_wmh(val: i64);

    /// Wrapper for `rs_last_status(0)`.
    fn rs_last_status(morewin: c_int);

    /// Wrapper for `update_screen()`.
    fn nvim_cmdexpand_update_screen();

    /// Wrapper for `win_redraw_last_status(topframe)`.
    fn nvim_cmdexpand_win_redraw_last_status();

    /// Wrapper for `redraw_statuslines()`.
    fn nvim_cmdexpand_redraw_statuslines();
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
// wildmenu_process_key (internal helpers)
// =============================================================================

/// Handle `K_UP`/`K_DOWN` for menunames (`EXPAND_MENUNAMES`) wildmenu.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. Accesses and mutates the real ccline.
unsafe fn process_key_menunames(key: c_int, xp: ExpandHandle) -> c_int {
    let k_down = nvim_cmdexpand_get_key_down();
    let k_up = nvim_cmdexpand_get_key_up();
    let p_wc = nvim_cmdexpand_get_p_wc();
    let cmdpos = nvim_cmdexpand_get_cmdpos();
    let cmdbuff = nvim_cmdexpand_get_cmdbuff();

    let mut key = key;

    if key == k_down
        && cmdpos > 0
        && !cmdbuff.is_null()
        && *cmdbuff.add((cmdpos - 1) as usize) == b'.' as libc::c_char
    {
        // Hitting <Down> after "emenu Name.": complete submenu
        key = p_wc;
        nvim_cmdexpand_set_key_typed(1); // in case the key was mapped
    } else if key == k_up {
        // Hitting <Up>: remove one submenu name in front of the cursor
        let mut found = false;
        let xp_pattern = (*xp).xp_pattern;
        let j_start = if cmdbuff.is_null() || xp_pattern.is_null() {
            0
        } else {
            (xp_pattern as usize).wrapping_sub(cmdbuff as usize) as c_int
        };
        let mut j = j_start;
        let mut i = 0;

        while j > 0 {
            j -= 1;
            let ch = *cmdbuff.add(j as usize) as u8;
            // check for start of menu name
            if ch == b' ' && (j == 0 || *cmdbuff.add((j - 1) as usize) != b'\\' as libc::c_char) {
                i = j + 1;
                break;
            }
            // check for start of submenu name
            if ch == b'.' && (j == 0 || *cmdbuff.add((j - 1) as usize) != b'\\' as libc::c_char) {
                if found {
                    i = j + 1;
                    break;
                }
                found = true;
            }
        }

        if i > 0 {
            nvim_cmdexpand_cmdline_del(i);
        }
        key = p_wc;
        nvim_cmdexpand_set_key_typed(1); // in case the key was mapped
        (*xp).xp_context = ExpandContext::Nothing.to_raw();
    }

    key
}

/// Handle `K_UP`/`K_DOWN` for files/dirs/shellcmd wildmenu.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. Accesses and mutates the real ccline.
unsafe fn process_key_filenames(key: c_int, xp: ExpandHandle) -> c_int {
    let k_down = nvim_cmdexpand_get_key_down();
    let k_up = nvim_cmdexpand_get_key_up();
    let p_wc = nvim_cmdexpand_get_p_wc();
    let pathsep = nvim_cmdexpand_get_pathsep() as u8;
    let cmdpos = nvim_cmdexpand_get_cmdpos();
    let cmdlen = nvim_cmdexpand_get_cmdlen();
    let cmdbuff = nvim_cmdexpand_get_cmdbuff();
    let xp_pattern = (*xp).xp_pattern;

    let mut key = key;

    // Build the ".." path segment: PATHSEP + ".." + PATHSEP
    let upseg = [pathsep, b'.', b'.', pathsep, 0u8];

    if key == k_down
        && cmdpos > 0
        && !cmdbuff.is_null()
        && *cmdbuff.add((cmdpos - 1) as usize) as u8 == pathsep
        && (cmdpos < 3
            || *cmdbuff.add((cmdpos - 2) as usize) as u8 != b'.'
            || *cmdbuff.add((cmdpos - 3) as usize) as u8 != b'.')
    {
        // Go down a directory
        key = p_wc;
        nvim_cmdexpand_set_key_typed(1);
    } else if !xp_pattern.is_null()
        && libc::strncmp(xp_pattern, upseg.as_ptr().add(1).cast(), 3) == 0
        && key == k_down
    {
        // If in a direct ancestor, strip off one ../ to go down
        let i = if cmdbuff.is_null() || xp_pattern.is_null() {
            0
        } else {
            (xp_pattern as usize).wrapping_sub(cmdbuff as usize) as c_int
        };
        let mut j = cmdpos;
        let mut found = false;

        while j > i {
            j -= 1;
            j -= nvim_cmdexpand_utf_head_off(cmdbuff, cmdbuff.add(j as usize));
            if nvim_cmdexpand_vim_ispathsep(c_int::from(*cmdbuff.add(j as usize))) != 0 {
                found = true;
                break;
            }
        }

        if found
            && j >= 2
            && *cmdbuff.add((j - 1) as usize) as u8 == b'.'
            && *cmdbuff.add((j - 2) as usize) as u8 == b'.'
            && (nvim_cmdexpand_vim_ispathsep(c_int::from(*cmdbuff.add((j - 3) as usize))) != 0
                || j == i + 2)
        {
            nvim_cmdexpand_cmdline_del(j - 2);
            key = p_wc;
            nvim_cmdexpand_set_key_typed(1);
        }
    } else if key == k_up {
        // Go up a directory
        let xp_offset = if cmdbuff.is_null() || xp_pattern.is_null() {
            0
        } else {
            (xp_pattern as usize).wrapping_sub(cmdbuff as usize) as c_int
        };
        let mut j = cmdpos - 1;
        let mut found = false;
        let mut result_i = xp_offset; // final i after loop

        while j > xp_offset {
            j -= 1;
            j -= nvim_cmdexpand_utf_head_off(cmdbuff, cmdbuff.add(j as usize));
            if nvim_cmdexpand_vim_ispathsep(c_int::from(*cmdbuff.add(j as usize))) != 0 {
                if found {
                    result_i = j + 1;
                    break;
                }
                found = true;
            }
        }

        let new_j = if !found {
            result_i
        } else if libc::strncmp(cmdbuff.add(j as usize), upseg.as_ptr().cast(), 4) == 0 {
            j + 4
        } else if libc::strncmp(cmdbuff.add(j as usize), upseg.as_ptr().add(1).cast(), 3) == 0
            && j == xp_offset
        {
            j + 3
        } else {
            0
        };

        if new_j > 0 {
            nvim_cmdexpand_cmdline_del(new_j);
            nvim_cmdexpand_put_on_cmdline(upseg.as_ptr().add(1).cast(), 3, 0);
        } else {
            let cmdpos_now = nvim_cmdexpand_get_cmdpos();
            if cmdpos_now > xp_offset {
                nvim_cmdexpand_cmdline_del(xp_offset);
            }
        }

        let _ = cmdlen; // used for bounds checking in original
        key = p_wc;
        nvim_cmdexpand_set_key_typed(1);
    }

    key
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

    if ctx == ExpandContext::Menunames.to_raw() {
        return process_key_menunames(key, xp);
    }

    if ctx == ExpandContext::Files.to_raw()
        || ctx == ExpandContext::Directories.to_raw()
        || ctx == ExpandContext::Shellcmd.to_raw()
    {
        return process_key_filenames(key, xp);
    }

    key
}

// =============================================================================
// wildmenu_cleanup
// =============================================================================

/// Free expanded names when finished walking through the matches.
///
/// Restores `laststatus`, redraws the screen as appropriate.
///
/// # Safety
///
/// Must be called from cmdline context where `get_cmdline_info()` is valid.
#[unsafe(export_name = "wildmenu_cleanup")]
pub unsafe extern "C" fn rs_wildmenu_cleanup(_cclp: *mut libc::c_void) {
    if nvim_get_p_wmnu() == 0 || nvim_get_wild_menu_showing() == 0 {
        return;
    }

    let skt = nvim_cmdexpand_get_key_typed() != 0;
    let old_redrawing_disabled = nvim_cmdexpand_get_redrawing_disabled();

    let input_fn = nvim_cmdexpand_get_input_fn() != 0;
    if input_fn {
        nvim_cmdexpand_set_redrawing_disabled(0);
    }

    // Clear highlighting applied during wildmenu activity
    nvim_cmdexpand_set_no_hlsearch(1);

    let wm_scrolled = nvim_cmdexpand_get_wm_scrolled();

    if nvim_get_wild_menu_showing() == wm_scrolled {
        // Entered command line, move it up
        nvim_cmdexpand_dec_cmdline_row();
        nvim_cmdexpand_redrawcmd();
        nvim_cmdexpand_set_wild_menu_showing(0);
    } else if nvim_cmdexpand_get_save_p_ls() != -1 {
        // Restore 'laststatus' and 'winminheight'
        nvim_cmdexpand_set_p_ls(i64::from(nvim_cmdexpand_get_save_p_ls()));
        nvim_cmdexpand_set_p_wmh(i64::from(nvim_cmdexpand_get_save_p_wmh()));
        rs_last_status(0);
        nvim_cmdexpand_update_screen(); // redraw the screen NOW
        nvim_cmdexpand_redrawcmd();
        nvim_cmdexpand_set_save_p_ls(-1);
        nvim_cmdexpand_set_wild_menu_showing(0);
    } else {
        nvim_cmdexpand_win_redraw_last_status();
        nvim_cmdexpand_set_wild_menu_showing(0); // must be before redraw_statuslines #8385
        nvim_cmdexpand_redraw_statuslines();
    }

    nvim_cmdexpand_set_key_typed(c_int::from(skt));
    if input_fn {
        nvim_cmdexpand_set_redrawing_disabled(old_redrawing_disabled);
    }
}
