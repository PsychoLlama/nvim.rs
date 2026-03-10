//! Tabline drawing implementation - direct port of `nvim_stl_draw_tabline_impl`
//!
//! Checks for external UI tabline, custom tabline option, or draws the
//! built-in tabline with grid operations.

use std::ffi::{c_char, c_int, c_void};

use nvim_window::WinHandle;

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

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
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
    fn nvim_stl_win_hl_attr(wp: WinHandle, hlf: c_int) -> c_int;
    #[link_name = "hl_combine_attr"]
    fn nvim_stl_hl_combine_attr(a: c_int, b: c_int) -> c_int;

    // Character
    #[link_name = "rs_schar_from_ascii"]
    fn nvim_stl_schar_from_ascii_char(c: c_char) -> ScharT;

    // Global state
    #[link_name = "nvim_option_get_t_colors"]
    fn nvim_stl_get_t_colors() -> c_int;
    #[link_name = "nvim_get_default_grid_has_chars"]
    fn nvim_stl_default_grid_has_chars() -> c_int;
    fn nvim_stl_set_redraw_tabline(val: c_int);
    #[link_name = "nvim_ui_has_tabline"]
    fn nvim_stl_ui_has_tabline() -> c_int;
    #[link_name = "rs_tabline_height"]
    fn nvim_stl_tabline_height() -> c_int;
    #[link_name = "nvim_get_Columns"]
    fn nvim_stl_get_Columns() -> c_int;
    fn nvim_stl_get_p_tal() -> *mut c_char;
    fn nvim_stl_get_p_sc() -> c_int;
    fn nvim_stl_showcmd_loc_is_tabline() -> c_int;
    fn nvim_stl_get_showcmd_buf() -> *const c_char;

    // Click defs
    fn nvim_stl_get_tab_page_click_defs_size() -> usize;
    #[link_name = "stl_clear_click_defs"]
    fn nvim_stl_clear_click_defs_wrap(defs: *mut c_void, size: usize);
    fn nvim_stl_get_tab_page_click_defs() -> *mut c_void;
    fn nvim_stl_set_tab_click_def(col: c_int, click_type: c_int, tabnr: c_int);

    // Tab info collection
    fn nvim_stl_collect_tab_info(out_count: *mut c_int) -> *mut c_void;
    fn nvim_stl_tab_info_size() -> usize;
    fn nvim_stl_tab_info_get_cwp(ptr: *mut c_void) -> WinHandle;
    fn nvim_stl_tab_info_get_wincount(ptr: *mut c_void) -> c_int;
    fn nvim_stl_tab_info_get_modified(ptr: *mut c_void) -> c_int;
    fn nvim_stl_tab_info_get_is_curtab(ptr: *mut c_void) -> c_int;
    fn nvim_stl_tab_info_get_topframe_match(ptr: *mut c_void) -> c_int;
    fn nvim_stl_tab_info_get_name(ptr: *mut c_void) -> *const c_char;
    fn nvim_stl_tab_info_get_name_len(ptr: *mut c_void) -> c_int;

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

    // Memory (direct link to C)
    #[link_name = "xfree"]
    fn nvim_stl_xfree(ptr: *mut c_void);
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
    let use_sep_chars = nvim_stl_get_t_colors() < 8;

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
    let columns = nvim_stl_get_Columns();
    let click_defs = nvim_stl_get_tab_page_click_defs();
    let click_defs_size = nvim_stl_get_tab_page_click_defs_size();
    debug_assert!(click_defs_size >= columns as usize);
    nvim_stl_clear_click_defs_wrap(click_defs, click_defs_size);

    // Use the 'tabline' option if it's set.
    let p_tal = nvim_stl_get_p_tal();
    if !p_tal.is_null() && *p_tal != 0 {
        rs_win_redr_custom(WinHandle::null(), false, false, false);
    } else {
        draw_builtin_tabline(columns, attr_nosel, attr_fill, use_sep_chars);
    }

    // Reset the flag here again, in case evaluating 'tabline' causes it to be set.
    nvim_stl_set_redraw_tabline(0);
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

    // Collect tab info from C
    let mut tabcount: c_int = 0;
    let tab_info_arr = nvim_stl_collect_tab_info(&mut tabcount);
    if tab_info_arr.is_null() || tabcount == 0 {
        // No tabs - just fill
        let c = if use_sep_chars { b'_' } else { b' ' };
        nvim_stl_grid_line_fill(
            col,
            columns,
            nvim_stl_schar_from_ascii_char(c as c_char),
            attr_fill,
        );
        nvim_stl_grid_line_flush();
        if !tab_info_arr.is_null() {
            nvim_stl_xfree(tab_info_arr);
        }
        return;
    }

    let tab_info_size = nvim_stl_tab_info_size();
    let tabwidth = rs_tabwidth_calc(columns, tabcount);

    let mut attr = attr_nosel;
    let mut drawn_tabcount = 0;

    for i in 0..tabcount {
        if col >= columns - 4 {
            break;
        }

        let tab_ptr = (tab_info_arr as *mut u8).add(i as usize * tab_info_size) as *mut c_void;
        let cwp = nvim_stl_tab_info_get_cwp(tab_ptr);
        let wincount = nvim_stl_tab_info_get_wincount(tab_ptr);
        let modified = nvim_stl_tab_info_get_modified(tab_ptr) != 0;
        let topframe_match = nvim_stl_tab_info_get_topframe_match(tab_ptr) != 0;

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
            let name_ptr = nvim_stl_tab_info_get_name(tab_ptr);
            let mut name_display_len = nvim_stl_tab_info_get_name_len(tab_ptr);

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
            let showcmd_buf = nvim_stl_get_showcmd_buf();
            nvim_stl_grid_line_puts(
                columns - sc_width - (if drawn_tabcount > 1 { 2 } else { 0 }),
                showcmd_buf,
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
    nvim_stl_xfree(tab_info_arr);
}

/// Set click definitions for a range of columns.
unsafe fn set_tab_click_range(start: c_int, end: c_int, tabnr: c_int) {
    for col in start..end {
        nvim_stl_set_tab_click_def(col, K_STL_CLICK_TAB_SWITCH, tabnr);
    }
}
