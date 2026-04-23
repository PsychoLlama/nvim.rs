//! Tabline drawing implementation - direct port of `nvim_stl_draw_tabline_impl`
//!
//! Checks for external UI tabline, custom tabline option, or draws the
//! built-in tabline with grid operations.

use std::ffi::{c_char, c_int, c_void};

use nvim_window::{BufHandle, TabpageHandle, WinHandle};

use crate::ScharT;

// =============================================================================
// Constants (verified via _Static_assert in statusline.c)
// =============================================================================

const HLF_T: c_int = 23;
const HLF_TP: c_int = 52;
const HLF_TPS: c_int = 53;
const HLF_TPF: c_int = 54;
const K_STL_CLICK_TAB_SWITCH: c_int = 1;
const K_STL_CLICK_TAB_CLOSE: c_int = 2;

const MAXPATHL: usize = 4096;

// =============================================================================
// Tab info (collected in Rust, replacing the C TabInfo struct)
// =============================================================================

struct TabData {
    cwp: WinHandle,
    wincount: c_int,
    modified: bool,
    topframe_match: bool,
    name_buf: [u8; MAXPATHL],
    name_len: c_int,
}

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    static Rows: c_int;
    static Columns: c_int;
    // Grid operations
    fn nvim_stl_default_grid_line_start(row: c_int);
    #[link_name = "grid_line_put_schar"]
    fn nvim_stl_grid_line_put_schar(col: c_int, c: ScharT, attr: c_int);
    #[link_name = "grid_line_puts"]
    fn nvim_stl_grid_line_puts(
        col: c_int,
        text: *const c_char,
        textlen: c_int,
        attr: c_int,
    ) -> c_int;
    #[link_name = "grid_line_fill"]
    fn nvim_stl_grid_line_fill(start: c_int, end: c_int, fillchar: ScharT, attr: c_int) -> c_int;
    #[link_name = "grid_line_flush"]
    fn nvim_stl_grid_line_flush();

    // Highlight
    fn nvim_stl_HL_ATTR(hlf: c_int) -> c_int;
    #[link_name = "rs_win_hl_attr"]
    fn nvim_stl_win_hl_attr(wp: WinHandle, hlf: c_int) -> c_int;
    #[link_name = "hl_combine_attr"]
    fn nvim_stl_hl_combine_attr(a: c_int, b: c_int) -> c_int;

    // Character
    #[link_name = "rs_schar_from_ascii"]
    fn nvim_stl_schar_from_ascii_char(c: c_char) -> ScharT;

    // Global state
    static mut t_colors: c_int;
    #[link_name = "nvim_get_default_grid_has_chars"]
    fn nvim_stl_default_grid_has_chars() -> c_int;
    fn nvim_stl_set_redraw_tabline(val: c_int);
    #[link_name = "nvim_ui_has_tabline"]
    fn nvim_stl_ui_has_tabline() -> c_int;
    #[link_name = "rs_tabline_height"]
    fn nvim_stl_tabline_height() -> c_int;
    static mut p_tal: *mut c_char;
    fn nvim_stl_get_p_sc() -> c_int;
    fn nvim_stl_showcmd_loc_is_tabline() -> c_int;
    static showcmd_buf: [u8; 41];

    // Click defs
    fn nvim_stl_get_tab_page_click_defs_size() -> usize;
    #[link_name = "stl_clear_click_defs"]
    fn nvim_stl_clear_click_defs_wrap(defs: *mut c_void, size: usize);
    fn nvim_stl_get_tab_page_click_defs() -> *mut c_void;
    fn nvim_stl_set_tab_click_def(col: c_int, click_type: c_int, tabnr: c_int);

    // Tab / window iteration (implemented in nvim-window Rust crate)
    #[link_name = "nvim_get_first_tabpage"]
    fn nvim_stl_get_first_tabpage() -> TabpageHandle;
    #[link_name = "nvim_tabpage_get_next"]
    fn nvim_stl_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;
    #[link_name = "nvim_tabpage_get_curwin"]
    fn nvim_stl_tabpage_get_curwin(tp: TabpageHandle) -> WinHandle;
    #[link_name = "nvim_tabpage_get_firstwin"]
    fn nvim_stl_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;
    #[link_name = "nvim_win_get_next"]
    fn nvim_stl_win_get_next(wp: WinHandle) -> WinHandle;
    #[link_name = "nvim_get_curwin"]
    fn nvim_stl_get_curwin() -> WinHandle;
    // firstwin for current tab
    fn nvim_curtab_first_win() -> WinHandle;
    // window focusable/hidden check accessors
    fn nvim_win_get_focusable(wp: WinHandle) -> c_int;
    fn nvim_ex2_win_get_w_config_hide(wp: WinHandle) -> bool;
    // bufIsChanged via accessor
    fn nvim_win_bufIsChanged(wp: WinHandle) -> c_int;
    // tabpage topframe comparison
    fn nvim_stl_tabpage_is_curtab(tp: TabpageHandle) -> c_int;
    fn nvim_stl_tabpage_topframe_matches(tp: TabpageHandle) -> c_int;
    // buffer name
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    #[link_name = "get_trans_bufname"]
    fn nvim_stl_get_trans_bufname(buf: BufHandle);
    #[link_name = "shorten_dir"]
    fn nvim_stl_shorten_dir(s: *mut c_char);
    static mut NameBuff: [c_char; 4096];

    // String (direct link to Rust/C implementations)
    #[link_name = "vim_strsize"]
    fn nvim_stl_vim_strsize(s: *const c_char) -> c_int;
    #[link_name = "ptr2cells"]
    fn nvim_stl_ptr2cells(s: *const c_char) -> c_int;
    #[link_name = "utfc_ptr2len"]
    fn nvim_stl_utfc_ptr2len(s: *const c_char) -> c_int;

    // Tabwidth calc (from Rust)
    fn rs_tabwidth_calc(columns: c_int, tabcount: c_int) -> c_int;

    // Delegated functions (already in Rust)
    fn rs_ui_ext_tabline_update();
    fn rs_win_redr_custom(wp: WinHandle, draw_winbar: bool, draw_ruler: bool, ui_event: bool);
}

// =============================================================================
// Main implementation
// =============================================================================

/// Draw the tab line at the top of the Vim window.
///
/// # Safety
/// Accesses global C state.
pub unsafe fn draw_tabline() {
    let attr_nosel = nvim_stl_HL_ATTR(HLF_TP);
    let attr_fill = nvim_stl_HL_ATTR(HLF_TPF);
    let use_sep_chars = t_colors < 8;

    if nvim_stl_default_grid_has_chars() == 0 {
        return;
    }
    nvim_stl_set_redraw_tabline(0);

    if nvim_stl_ui_has_tabline() != 0 {
        rs_ui_ext_tabline_update();
        return;
    }

    if nvim_stl_tabline_height() < 1 {
        return;
    }

    // Clear tab_page_click_defs
    let columns = Columns;
    let click_defs = nvim_stl_get_tab_page_click_defs();
    let click_defs_size = nvim_stl_get_tab_page_click_defs_size();
    debug_assert!(click_defs_size >= columns as usize);
    nvim_stl_clear_click_defs_wrap(click_defs, click_defs_size);

    // Use the 'tabline' option if it's set.
    if !p_tal.is_null() && *p_tal != 0 {
        rs_win_redr_custom(WinHandle::null(), false, false, false);
    } else {
        draw_builtin_tabline(columns, attr_nosel, attr_fill, use_sep_chars);
    }

    // Reset the flag here again, in case evaluating 'tabline' causes it to be set.
    nvim_stl_set_redraw_tabline(0);
}

/// Collect per-tab info in Rust, replacing the C nvim_stl_collect_tab_info.
unsafe fn collect_tab_data() -> Vec<TabData> {
    let mut tabs: Vec<TabData> = Vec::new();
    let curwin = nvim_stl_get_curwin();

    let mut tp = nvim_stl_get_first_tabpage();
    while !tp.is_null() {
        let is_curtab = nvim_stl_tabpage_is_curtab(tp) != 0;
        let topframe_match = nvim_stl_tabpage_topframe_matches(tp) != 0;

        // Current window for this tab
        let cwp = if is_curtab {
            curwin
        } else {
            nvim_stl_tabpage_get_curwin(tp)
        };

        // Iterate windows to count focusable, non-hidden and check modified
        let firstwin = if is_curtab {
            nvim_curtab_first_win()
        } else {
            nvim_stl_tabpage_get_firstwin(tp)
        };

        let mut wincount: c_int = 0;
        let mut modified = false;
        let mut wp = firstwin;
        while !wp.is_null() {
            let focusable = nvim_win_get_focusable(wp) != 0;
            let hidden = nvim_ex2_win_get_w_config_hide(wp);
            if focusable && !hidden {
                wincount += 1;
                if nvim_win_bufIsChanged(wp) != 0 {
                    modified = true;
                }
            }
            wp = nvim_stl_win_get_next(wp);
        }

        // Get buffer name via get_trans_bufname + shorten_dir
        let buf = nvim_win_get_buffer(cwp);
        let mut name_buf = [0u8; MAXPATHL];
        if !buf.is_null() {
            nvim_stl_get_trans_bufname(buf);
            let namebuff = (&raw mut NameBuff).cast::<c_char>();
            nvim_stl_shorten_dir(namebuff);
            // Copy NameBuff into our local buffer
            let namebuff_slice = std::ffi::CStr::from_ptr(namebuff);
            let bytes = namebuff_slice.to_bytes();
            let copy_len = bytes.len().min(MAXPATHL - 1);
            name_buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
            name_buf[copy_len] = 0;
        }
        let name_len = nvim_stl_vim_strsize(name_buf.as_ptr().cast());

        tabs.push(TabData {
            cwp,
            wincount,
            modified,
            topframe_match,
            name_buf,
            name_len,
        });

        tp = nvim_stl_tabpage_get_next(tp);
    }

    tabs
}

/// Draw the built-in tabline (when no custom 'tabline' is set).
unsafe fn draw_builtin_tabline(
    columns: c_int,
    attr_nosel: c_int,
    attr_fill: c_int,
    use_sep_chars: bool,
) {
    let mut col: c_int = 0;

    nvim_stl_default_grid_line_start(0);

    // Collect tab info in Rust
    let tabs = collect_tab_data();
    let tabcount = tabs.len() as c_int;

    if tabcount == 0 {
        // No tabs - just fill
        let c = if use_sep_chars { b'_' } else { b' ' };
        nvim_stl_grid_line_fill(
            col,
            columns,
            nvim_stl_schar_from_ascii_char(c as c_char),
            attr_fill,
        );
        nvim_stl_grid_line_flush();
        return;
    }

    let tabwidth = rs_tabwidth_calc(columns, tabcount);

    let mut attr = attr_nosel;
    let mut drawn_tabcount = 0;

    for tab in &tabs {
        if col >= columns - 4 {
            break;
        }

        let cwp = tab.cwp;
        let wincount = tab.wincount;
        let modified = tab.modified;
        let topframe_match = tab.topframe_match;

        let scol = col;

        if topframe_match {
            attr = nvim_stl_win_hl_attr(cwp, HLF_TPS);
        }
        if use_sep_chars && col > 0 {
            nvim_stl_grid_line_put_schar(col, nvim_stl_schar_from_ascii_char(b'|' as c_char), attr);
            col += 1;
        }

        if !topframe_match {
            attr = nvim_stl_win_hl_attr(cwp, HLF_TP);
        }

        nvim_stl_grid_line_put_schar(col, nvim_stl_schar_from_ascii_char(b' ' as c_char), attr);
        col += 1;

        if modified || wincount > 1 {
            if wincount > 1 {
                // Format window count
                let wc_str = format!("{wincount}\0");
                let wc_bytes = wc_str.as_bytes();
                let len = wc_bytes.len() as c_int - 1; // exclude NUL
                if col + len >= columns - 3 {
                    // Set click defs up to current col
                    drawn_tabcount += 1;
                    set_tab_click_range(scol, col, drawn_tabcount);
                    break;
                }
                let combined = nvim_stl_hl_combine_attr(attr, nvim_stl_win_hl_attr(cwp, HLF_T));
                nvim_stl_grid_line_puts(col, wc_bytes.as_ptr().cast::<c_char>(), len, combined);
                col += len;
            }
            if modified {
                nvim_stl_grid_line_put_schar(
                    col,
                    nvim_stl_schar_from_ascii_char(b'+' as c_char),
                    attr,
                );
                col += 1;
            }
            nvim_stl_grid_line_put_schar(col, nvim_stl_schar_from_ascii_char(b' ' as c_char), attr);
            col += 1;
        }

        // Draw buffer name
        let room = scol - col + tabwidth - 1;
        if room > 0 {
            let name_ptr = tab.name_buf.as_ptr().cast::<c_char>();
            let mut name_display_len = tab.name_len;

            // Skip leading chars that don't fit
            let mut p = name_ptr;
            while name_display_len > room {
                name_display_len -= nvim_stl_ptr2cells(p);
                p = p.add(nvim_stl_utfc_ptr2len(p) as usize);
            }
            let n = columns - col - 1;
            let display_len = if name_display_len < n {
                name_display_len
            } else {
                n
            };

            nvim_stl_grid_line_puts(col, p, -1, attr);
            col += display_len;
        }
        nvim_stl_grid_line_put_schar(col, nvim_stl_schar_from_ascii_char(b' ' as c_char), attr);
        col += 1;

        // Store click defs
        drawn_tabcount += 1;
        set_tab_click_range(scol, col, drawn_tabcount);
    }

    // Fill remaining with click defs for "new tab" (tabnr=0)
    for scol in col..columns {
        nvim_stl_set_tab_click_def(scol, K_STL_CLICK_TAB_SWITCH, 0);
    }

    // Fill rest with fill char
    let c = if use_sep_chars { b'_' } else { b' ' };
    nvim_stl_grid_line_fill(
        col,
        columns,
        nvim_stl_schar_from_ascii_char(c as c_char),
        attr_fill,
    );

    // Draw showcmd if 'showcmdloc' == "tabline"
    if nvim_stl_get_p_sc() != 0 && nvim_stl_showcmd_loc_is_tabline() != 0 {
        let n = columns - col - (if drawn_tabcount > 1 { 3 } else { 0 });
        let sc_width = if 10 < n { 10 } else { n };

        if sc_width > 0 {
            nvim_stl_grid_line_puts(
                columns - sc_width - (if drawn_tabcount > 1 { 2 } else { 0 }),
                showcmd_buf.as_ptr().cast(),
                sc_width,
                attr_nosel,
            );
        }
    }

    // Put an "X" for closing the current tab if there are several.
    if drawn_tabcount > 1 {
        nvim_stl_grid_line_put_schar(
            columns - 1,
            nvim_stl_schar_from_ascii_char(b'X' as c_char),
            attr_nosel,
        );
        nvim_stl_set_tab_click_def(columns - 1, K_STL_CLICK_TAB_CLOSE, 999);
    }

    nvim_stl_grid_line_flush();
}

/// Set click definitions for a range of columns.
unsafe fn set_tab_click_range(start: c_int, end: c_int, tabnr: c_int) {
    for col in start..end {
        nvim_stl_set_tab_click_def(col, K_STL_CLICK_TAB_SWITCH, tabnr);
    }
}
