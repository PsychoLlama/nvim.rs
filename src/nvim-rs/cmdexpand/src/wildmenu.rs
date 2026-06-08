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

    /// `wild_menu_showing` global (0, `WM_SHOWN`=1, `WM_SCROLLED`=2).
    static mut wild_menu_showing: c_int;

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
    fn vim_ispathsep(c: c_int) -> c_int;

    /// Get the `PATHSEP` character constant.
    fn nvim_cmdexpand_get_pathsep() -> c_int;

    // Wildmenu cleanup globals
    /// `p_wmnu` (wildmenu option).
    static p_wmnu: c_int;

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

    /// Direct: `redrawcmd()`.
    fn redrawcmd();

    /// `save_p_ls` global.
    static mut save_p_ls: c_int;

    /// `save_p_wmh` global.
    static mut save_p_wmh: c_int;

    /// `p_ls` global (`OptInt = i64`).
    static mut p_ls: i64;

    /// `p_wmh` global (`OptInt = i64`).
    static mut p_wmh: i64;

    /// Wrapper for `rs_last_status(0)`.
    fn rs_last_status(morewin: c_int);

    /// Direct: `update_screen()`.
    fn update_screen();

    /// Wrapper for `win_redraw_last_status(topframe)`.
    fn nvim_cmdexpand_win_redraw_last_status();

    /// Wrapper for `redraw_statuslines()`.
    fn nvim_cmdexpand_redraw_statuslines();

    /// Direct: `cursorcmd()`.
    fn cursorcmd();

    /// Direct: `ui_flush()`.
    fn ui_flush();

    /// Direct: `msg_puts(s)`.
    fn msg_puts(s: *const libc::c_char);

    /// `cmd_silent` global.
    static cmd_silent: bool;

    /// `got_int` global.
    static mut got_int: bool;

    /// Wrapper for `xstrnsave(s, n)`.
    fn nvim_cmdexpand_xstrnsave(s: *const libc::c_char, n: usize) -> *mut libc::c_char;

    /// Set `cmd_showtail`.
    fn nvim_cmdexpand_set_cmd_showtail(val: c_int);

    /// Set `may_expand_pattern`.
    fn nvim_cmdexpand_set_may_expand_pattern(val: c_int);

    /// Copy `pre_incsearch_pos` from `xp->xp_pre_incsearch_pos`.
    fn nvim_cmdexpand_copy_pre_incsearch_pos(xp: *mut crate::ExpandT);

    /// Free old `cmdline_orig` and set a new value (NULL to just free).
    fn nvim_cmdexpand_set_cmdline_orig(val: *mut libc::c_char);

    /// Apply expansion result into ccline->cmdbuff.
    fn nvim_cmdexpand_apply_expansion(
        xp: *mut crate::ExpandT,
        i: c_int,
        p: *const libc::c_char,
        plen: c_int,
    );

    /// Wrapper for `nlua_expand_pat(xp)`.
    fn nvim_cmdexpand_nlua_expand_pat(xp: *mut crate::ExpandT);

    /// Get `xp_context` from `get_cmdline_info()->xpc` (or `EXPAND_NOTHING` if NULL).
    fn nvim_cmdexpand_get_ccline_xp_context() -> c_int;

    /// Get `p_wic` option value (wildchar: use ignore case).
    fn nvim_cmdexpand_get_p_wic() -> c_int;

    /// `ExpandOne` (already in Rust, but callable via C ABI).
    fn ExpandOne(
        xp: *mut crate::ExpandT,
        str_: *mut libc::c_char,
        orig: *mut libc::c_char,
        options: c_int,
        mode: c_int,
    ) -> *mut libc::c_char;

    fn xfree(ptr: *mut libc::c_void);
    fn xmalloc(size: usize) -> *mut libc::c_char;

    fn beep_flush();

    fn ui_has(cap: c_int) -> bool;

    fn rs_expand_showtail(xp: *mut crate::ExpandT) -> c_int;

    fn rs_cmdline_fuzzy_completion_supported(context: c_int) -> c_int;

    fn addstar(fname: *mut libc::c_char, len: usize, context: c_int) -> *mut libc::c_char;

    // redraw_wildmenu dependencies
    /// `nvim_win_hl_attr(wp, hlf)` — get highlight attr for window+group.
    fn nvim_win_hl_attr(wp: *mut libc::c_void, hlf: c_int) -> c_int;

    /// `nvim_get_curwin()` — returns current window pointer.
    fn nvim_get_curwin() -> *mut libc::c_void;

    /// `nvim_get_lastwin()` — returns last window pointer.
    fn nvim_get_lastwin() -> *mut libc::c_void;

    /// `nvim_win_get_status_height(wp)` — get `w_status_height`.
    fn nvim_win_get_status_height(wp: *mut libc::c_void) -> c_int;

    /// `rs_global_stl_height()` — get global statusline height.
    fn rs_global_stl_height() -> c_int;

    /// `msg_scrolled` global.
    static mut msg_scrolled: c_int;

    /// `msg_scroll_up(may_throttle, zerocmd)`.
    fn msg_scroll_up(may_throttle: bool, zerocmd: bool);

    /// `grid_line_start(view, row)`.
    fn grid_line_start(view: *mut libc::c_void, row: c_int);

    /// `grid_line_puts(col, str, len, attr)` — writes a C string to the line.
    fn grid_line_puts(col: c_int, text: *const libc::c_char, textlen: c_int, attr: c_int) -> c_int;

    /// `grid_line_fill(start_col, end_col, sc, attr)`.
    fn grid_line_fill(start_col: c_int, end_col: c_int, sc: u32, attr: c_int);

    /// `grid_line_flush()`.
    fn grid_line_flush();

    /// `transchar(c)` — translate char to printable sequence.
    fn transchar(c: c_int) -> *mut libc::c_char;

    /// `transchar_byte(c)` — translate non-ASCII byte to printable sequence.
    fn transchar_byte(c: c_int) -> *mut libc::c_char;

    /// `utfc_ptr2len(p)` — byte length of the first UTF-8 sequence.
    fn utfc_ptr2len(p: *const libc::c_char) -> c_int;

    /// `ptr2cells(p)` — display width of first character in string.
    fn ptr2cells(p: *const libc::c_char) -> c_int;

    /// `menu_is_separator(name)`.
    fn menu_is_separator(name: *const libc::c_char) -> bool;

    /// `fillchar_status(group, wp)` — get fill char and group for status line.
    #[link_name = "fillchar_status"]
    fn nvim_fillchar_status(group: *mut c_int, wp: *mut libc::c_void) -> u32;

    /// Get pointer to `msg_grid_adj` `GridView`.
    fn nvim_cmdexpand_get_msg_grid_adj_ptr() -> *mut libc::c_void;

    /// Get pointer to `default_gridview` `GridView`.
    fn nvim_cmdexpand_get_default_gridview_ptr() -> *mut libc::c_void;

    /// `hl_attr_active` pointer (array of int, indexed by HLF_* enum value).
    static mut hl_attr_active: *mut c_int;

    /// `Columns` global — terminal width.
    static Columns: c_int;

    /// `Rows` global — terminal height.
    static Rows: c_int;

    /// `cmdline_row` global — row where the command line is displayed.
    static mut cmdline_row: c_int;

    /// `rs_wildmenu_match_len(xp, s)` — visible width of a completion match.
    fn rs_wildmenu_match_len(xp: *mut crate::ExpandT, s: *const libc::c_char) -> c_int;

    /// `rs_skip_wildmenu_char(xp, s)` — bytes to skip for non-displayable chars.
    fn rs_skip_wildmenu_char(xp: *const crate::ExpandT, s: *const libc::c_char) -> c_int;

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
    let wild_showing = wild_menu_showing != 0;

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
            if vim_ispathsep(c_int::from(*cmdbuff.add(j as usize))) != 0 {
                found = true;
                break;
            }
        }

        if found
            && j >= 2
            && *cmdbuff.add((j - 1) as usize) as u8 == b'.'
            && *cmdbuff.add((j - 2) as usize) as u8 == b'.'
            && (vim_ispathsep(c_int::from(*cmdbuff.add((j - 3) as usize))) != 0 || j == i + 2)
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
            if vim_ispathsep(c_int::from(*cmdbuff.add(j as usize))) != 0 {
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
    if p_wmnu == 0 || wild_menu_showing == 0 {
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

    if wild_menu_showing == wm_scrolled {
        // Entered command line, move it up
        nvim_cmdexpand_dec_cmdline_row();
        redrawcmd();
        wild_menu_showing = 0;
    } else if save_p_ls != -1 {
        // Restore 'laststatus' and 'winminheight'
        p_ls = i64::from(save_p_ls);
        p_wmh = i64::from(save_p_wmh);
        rs_last_status(0);
        update_screen(); // redraw the screen NOW
        redrawcmd();
        save_p_ls = -1;
        wild_menu_showing = 0;
    } else {
        nvim_cmdexpand_win_redraw_last_status();
        wild_menu_showing = 0; // must be before redraw_statuslines #8385
        nvim_cmdexpand_redraw_statuslines();
    }

    nvim_cmdexpand_set_key_typed(c_int::from(skt));
    if input_fn {
        nvim_cmdexpand_set_redrawing_disabled(old_redrawing_disabled);
    }
}

// =============================================================================
// nextwild
// =============================================================================

// =============================================================================
// redraw_wildmenu
// =============================================================================

/// `WM_SHOWN` — wildmenu is showing.
const WM_SHOWN: c_int = 1;
/// `WM_SCROLLED` — wildmenu showing with scroll.
const WM_SCROLLED: c_int = 2;
/// `HLF_WM` — wildmenu highlight (index in `hl_attr_active` array).
const HLF_WM: usize = 27;
/// `MB_MAXBYTES` — maximum bytes in a multibyte character.
const MB_MAXBYTES: usize = 21;
/// `EXPAND_MENUS` context.
const EXPAND_MENUS: c_int = 11;
/// `EXPAND_MENUNAMES` context.
const EXPAND_MENUNAMES: c_int = 21;

/// First visible match in the wildmenu (persists across calls like C `static`).
static mut FIRST_MATCH: c_int = 0;

/// Get a single match string (showtail or raw).
///
/// # Safety
/// `matches` must be valid, `m` must be in range.
#[inline]
unsafe fn show_match_str(
    matches: *mut *mut libc::c_char,
    m: c_int,
    showtail: bool,
) -> *mut libc::c_char {
    if showtail {
        crate::helpers::rs_showmatches_gettail(*matches.add(m as usize), 0)
    } else {
        *matches.add(m as usize)
    }
}

/// Show wildchar matches in the status line.
///
/// Show at least the `match_` item.  Items to the right of `match_` are shown
/// first; if there is room, items to the left are added.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` pointer.
/// `matches` must point to at least `num_matches` valid C string pointers (or be NULL).
#[allow(clippy::too_many_lines)]
#[unsafe(export_name = "redraw_wildmenu")]
pub unsafe extern "C" fn rs_redraw_wildmenu(
    xp: *mut crate::ExpandT,
    num_matches: c_int,
    matches: *mut *mut libc::c_char,
    match_: c_int,
    showtail: bool,
) {
    if matches.is_null() {
        // Interrupted completion
        return;
    }

    let columns = Columns;
    let buf = xmalloc(columns as usize * MB_MAXBYTES + 1);

    let (highlight, match_) = if match_ == -1 {
        // Don't show match but original text
        (false, 0i32)
    } else {
        (true, match_)
    };

    // Count 1 for the ending ">"
    let mut clen = rs_wildmenu_match_len(xp, show_match_str(matches, match_, showtail)) + 3;
    let mut add_left = false;

    if match_ == 0 {
        FIRST_MATCH = 0;
    } else if match_ < FIRST_MATCH {
        // Jumping left: go as far left as possible
        FIRST_MATCH = match_;
        add_left = true;
    } else {
        // Check if match fits on the screen
        let mut i = FIRST_MATCH;
        while i < match_ {
            clen += rs_wildmenu_match_len(xp, show_match_str(matches, i, showtail)) + 2;
            i += 1;
        }
        if FIRST_MATCH > 0 {
            clen += 2;
        }

        // Jumping right: put match at the left
        if clen > columns {
            FIRST_MATCH = match_;
            // If showing the last match, we can add some on the left
            clen = 2;
            let mut i = match_;
            while i < num_matches {
                clen += rs_wildmenu_match_len(xp, show_match_str(matches, i, showtail)) + 2;
                if clen >= columns {
                    break;
                }
                i += 1;
            }
            if i == num_matches {
                add_left = true;
            }
        }
    }
    if add_left {
        while FIRST_MATCH > 0 {
            clen +=
                rs_wildmenu_match_len(xp, show_match_str(matches, FIRST_MATCH - 1, showtail)) + 2;
            if clen >= columns {
                break;
            }
            FIRST_MATCH -= 1;
        }
    }

    let mut group: c_int = 0;
    let curwin = nvim_get_curwin();
    let fillchar = nvim_fillchar_status(&raw mut group, curwin);
    let attr = nvim_win_hl_attr(curwin, group);

    // Build the display string in buf
    let (mut len, mut clen) = if FIRST_MATCH == 0 {
        *buf = 0i8; // NUL
        (0i32, 0i32)
    } else {
        // "< "
        *buf = b'<' as libc::c_char;
        *buf.add(1) = b' ' as libc::c_char;
        (2i32, 2i32)
    };

    let mut selstart: *mut libc::c_char = std::ptr::null_mut();
    let mut selstart_col: c_int = 0;
    let mut selend: *mut libc::c_char = std::ptr::null_mut();

    let mut i = FIRST_MATCH;
    while clen + rs_wildmenu_match_len(xp, show_match_str(matches, i, showtail)) + 2 < columns {
        if i == match_ {
            selstart = buf.add(len as usize);
            selstart_col = clen;
        }

        let s = show_match_str(matches, i, showtail);
        let xp_context = (*xp).xp_context;
        let emenu = xp_context == EXPAND_MENUS || xp_context == EXPAND_MENUNAMES;
        if emenu && menu_is_separator(s) {
            let tc = transchar(c_int::from(b'|'));
            let tc_len = libc::strlen(tc) as usize;
            std::ptr::copy_nonoverlapping(tc, buf.add(len as usize), tc_len);
            len += tc_len as c_int;
            clen += tc_len as c_int;
        } else {
            let mut sp = s;
            while *sp != 0 {
                sp = sp.add(rs_skip_wildmenu_char(xp, sp) as usize);
                clen += ptr2cells(sp);
                let l = utfc_ptr2len(sp);
                if l > 1 {
                    std::ptr::copy_nonoverlapping(sp, buf.add(len as usize), l as usize);
                    sp = sp.add((l - 1) as usize);
                    len += l;
                } else {
                    let tb = transchar_byte(c_int::from(*sp as u8));
                    let tb_len = libc::strlen(tb) as usize;
                    std::ptr::copy_nonoverlapping(tb, buf.add(len as usize), tb_len);
                    len += tb_len as c_int;
                }
                sp = sp.add(1);
            }
        }

        if i == match_ {
            selend = buf.add(len as usize);
        }

        *buf.add(len as usize) = b' ' as libc::c_char;
        len += 1;
        *buf.add(len as usize) = b' ' as libc::c_char;
        len += 1;
        clen += 2;
        i += 1;
        if i == num_matches {
            break;
        }
    }

    if i != num_matches {
        *buf.add(len as usize) = b'>' as libc::c_char;
        len += 1;
        clen += 1;
    }

    *buf.add(len as usize) = 0; // NUL terminate

    let mut row = cmdline_row - 1;
    if row >= 0 {
        if wild_menu_showing == 0 {
            if msg_scrolled > 0 {
                // Put the wildmenu just above the command line.
                // If there is no room, scroll the screen one line up.
                if cmdline_row == Rows - 1 {
                    msg_scroll_up(false, false);
                    msg_scrolled += 1;
                } else {
                    cmdline_row += 1;
                    row += 1;
                }
                wild_menu_showing = WM_SCROLLED;
            } else {
                // Create status line if needed by setting 'laststatus' to 2.
                // Set 'winminheight' to zero to avoid window resizing.
                let lastwin = nvim_get_lastwin();
                if nvim_win_get_status_height(lastwin) == 0 && rs_global_stl_height() == 0 {
                    save_p_ls = p_ls as c_int;
                    save_p_wmh = p_wmh as c_int;
                    p_ls = 2;
                    p_wmh = 0;
                    rs_last_status(0);
                }
                wild_menu_showing = WM_SHOWN;
            }
        }

        // Start rendering the grid line
        let gridview = if wild_menu_showing == WM_SCROLLED {
            nvim_cmdexpand_get_msg_grid_adj_ptr()
        } else {
            nvim_cmdexpand_get_default_gridview_ptr()
        };
        grid_line_start(gridview, row);

        grid_line_puts(0, buf, -1, attr);

        if !selstart.is_null() && highlight {
            *selend = 0; // NUL-terminate at selend
            let hl_wm = *hl_attr_active.add(HLF_WM);
            grid_line_puts(selstart_col, selstart, -1, hl_wm);
        }

        grid_line_fill(clen, columns, fillchar, attr);
        grid_line_flush();
    }

    nvim_cmdexpand_win_redraw_last_status();
    xfree(buf.cast::<libc::c_void>());
}

use crate::context::wild_mode::{
    WILD_FREE, WILD_LONGEST, WILD_NEXT, WILD_PAGEDOWN, WILD_PAGEUP, WILD_PREV, WILD_PUM_WANT,
};
use crate::context::wild_options::{
    WILD_ADD_SLASH, WILD_ESCAPE, WILD_FUNC_TRIGGER, WILD_HOME_REPLACE, WILD_ICASE,
    WILD_MAY_EXPAND_PATTERN, WILD_NOSELECT, WILD_SILENT,
};

/// `kUICmdline` enum value from `ui_defs.h`.
const K_UI_CMDLINE: c_int = 0;
/// `kUIWildmenu` enum value from `ui_defs.h` (= 2 in the 0-based `UIExtType` enum).
const K_UI_WILDMENU: c_int = 2;

/// `EXPAND_COMMANDS` context constant.
const EXPAND_COMMANDS: c_int = 6;
/// `EXPAND_MAPPINGS` context constant.
const EXPAND_MAPPINGS: c_int = 7;
/// `EXPAND_UNSUCCESSFUL` context constant.
const EXPAND_UNSUCCESSFUL: c_int = -1;
/// `EXPAND_NOTHING` context constant.
const EXPAND_NOTHING: c_int = 0;
/// `EXPAND_PATTERN_IN_BUF` context constant.
const EXPAND_PATTERN_IN_BUF: c_int = 65;
/// `EXPAND_LUA` context constant.
const EXPAND_LUA: c_int = 74;

const OK: c_int = 1;
const FAIL: c_int = 0;

/// Main entry point for wildcard expansion on the command line.
///
/// Returns `OK` (1) on success, `FAIL` (0) to signal that the character should
/// be inserted literally (e.g., for `:map` context with no match).
///
/// # Safety
///
/// `xp` must be a valid `expand_T` pointer; `get_cmdline_info()` must return
/// a valid ccline for the current context.
///
/// # Panics
///
/// Panics (via `assert!`) if `cmdpos < i` (pattern offset exceeds cursor position).
#[allow(clippy::too_many_lines)]
#[unsafe(export_name = "nextwild")]
pub unsafe extern "C" fn rs_nextwild(
    xp: *mut crate::ExpandT,
    type_: c_int,
    options: c_int,
    escape: bool,
) -> c_int {
    let from_wildtrigger_func = (options & WILD_FUNC_TRIGGER) != 0;
    let wild_navigate = type_ == WILD_NEXT
        || type_ == WILD_PREV
        || type_ == WILD_PAGEUP
        || type_ == WILD_PAGEDOWN
        || type_ == WILD_PUM_WANT;

    if (*xp).xp_numfiles == -1 {
        // Copy pre_incsearch_pos from xp field
        nvim_cmdexpand_copy_pre_incsearch_pos(xp);

        let input_fn = nvim_cmdexpand_get_input_fn() != 0;
        let xp_context = nvim_cmdexpand_get_ccline_xp_context();
        if input_fn && xp_context == EXPAND_COMMANDS {
            // Expand commands typed in input() function
            let cmdbuff = nvim_cmdexpand_get_cmdbuff();
            let cmdlen = nvim_cmdexpand_get_cmdlen();
            let cmdpos = nvim_cmdexpand_get_cmdpos();
            crate::rs_set_cmd_context(xp, cmdbuff, cmdlen, cmdpos, 0);
        } else {
            let may_expand = (options & WILD_MAY_EXPAND_PATTERN) != 0;
            nvim_cmdexpand_set_may_expand_pattern(c_int::from(may_expand));
            crate::rs_set_expand_context(xp);
            nvim_cmdexpand_set_may_expand_pattern(0);
        }

        if (*xp).xp_context == EXPAND_LUA {
            nvim_cmdexpand_nlua_expand_pat(xp);
        }

        let showtail = rs_expand_showtail(xp) != 0;
        nvim_cmdexpand_set_cmd_showtail(c_int::from(showtail));
    }

    if (*xp).xp_context == EXPAND_UNSUCCESSFUL {
        beep_flush();
        return OK; // Something illegal on command line
    }
    if (*xp).xp_context == EXPAND_NOTHING {
        // Caller can use the character as a normal char instead
        return FAIL;
    }

    let cmdbuff = nvim_cmdexpand_get_cmdbuff();
    let cmdpos = nvim_cmdexpand_get_cmdpos();
    let i = if cmdbuff.is_null() || (*xp).xp_pattern.is_null() {
        0
    } else {
        ((*xp).xp_pattern as usize).wrapping_sub(cmdbuff as usize) as c_int
    };
    assert!(cmdpos >= i);
    (*xp).xp_pattern_len = (cmdpos - i) as usize;

    // Skip showing matches if prefix is invalid during wildtrigger()
    if from_wildtrigger_func && (*xp).xp_context == EXPAND_COMMANDS && (*xp).xp_pattern_len == 0 {
        return FAIL;
    }

    // If cmd_silent is set then don't show dots while busy
    if !cmd_silent
        && !from_wildtrigger_func
        && !wild_navigate
        && !ui_has(K_UI_CMDLINE)
        && !ui_has(K_UI_WILDMENU)
    {
        msg_puts(c"...".as_ptr());
        ui_flush();
    }

    let mut p: *mut libc::c_char;

    if wild_navigate {
        // Get next/previous match for a previously expanded pattern.
        p = ExpandOne(xp, std::ptr::null_mut(), std::ptr::null_mut(), 0, type_);
    } else {
        let xp_pattern = (*xp).xp_pattern;
        let xp_pattern_len = (*xp).xp_pattern_len;
        let xp_context = (*xp).xp_context;

        let tmp: *mut libc::c_char = if rs_cmdline_fuzzy_completion_supported(xp_context) != 0
            || xp_context == EXPAND_PATTERN_IN_BUF
        {
            // Don't modify the search string
            nvim_cmdexpand_xstrnsave(xp_pattern, xp_pattern_len)
        } else {
            addstar(xp_pattern, xp_pattern_len, xp_context)
        };

        let p_wic = nvim_cmdexpand_get_p_wic();
        let use_options = options
            | WILD_HOME_REPLACE
            | WILD_ADD_SLASH
            | WILD_SILENT
            | if escape { WILD_ESCAPE } else { 0 }
            | if p_wic != 0 { WILD_ICASE } else { 0 };

        let orig = nvim_cmdexpand_xstrnsave(cmdbuff.add(i as usize), xp_pattern_len);
        p = ExpandOne(xp, tmp, orig, use_options, type_);
        xfree(tmp.cast::<libc::c_void>());

        // Longest match: make sure it is not shorter, happens with :help.
        if !p.is_null() && type_ == WILD_LONGEST {
            // Count how many non-wildcard chars are in the original pattern.
            let mut j = 0usize;
            while j < xp_pattern_len {
                let ch = *cmdbuff.add(i as usize + j) as u8;
                if ch == b'*' || ch == b'?' {
                    break;
                }
                j += 1;
            }
            if libc::strlen(p) < j {
                xfree(p.cast::<libc::c_void>());
                p = std::ptr::null_mut();
            }
        }
    }

    // Save cmdline before inserting selected item
    if !wild_navigate && !cmdbuff.is_null() {
        let cmdlen = nvim_cmdexpand_get_cmdlen();
        let new_orig = nvim_cmdexpand_xstrnsave(cmdbuff, cmdlen as usize);
        nvim_cmdexpand_set_cmdline_orig(new_orig);
    }

    let got_int_val = got_int;

    if !p.is_null() && !got_int_val && (options & WILD_NOSELECT) == 0 {
        let plen = libc::strlen(p) as c_int;
        nvim_cmdexpand_apply_expansion(xp, i, p, plen);
    }

    redrawcmd();
    cursorcmd();

    // When expanding a ":map" command and no matches are found, assume that
    // the key is supposed to be inserted literally
    if (*xp).xp_context == EXPAND_MAPPINGS && p.is_null() {
        return FAIL;
    }

    if (*xp).xp_numfiles <= 0 && p.is_null() {
        beep_flush();
    } else if (*xp).xp_numfiles == 1 && (options & WILD_NOSELECT) == 0 && !wild_navigate {
        // free expanded pattern
        ExpandOne(xp, std::ptr::null_mut(), std::ptr::null_mut(), 0, WILD_FREE);
    }

    xfree(p.cast::<libc::c_void>());

    OK
}
