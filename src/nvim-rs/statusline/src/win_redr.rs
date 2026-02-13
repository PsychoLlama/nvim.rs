//! Custom window redraw implementation - direct port of `nvim_stl_win_redr_custom_impl`
//!
//! Orchestrates rendering of statusline, winbar, tabline, and rulerformat
//! by determining mode/parameters, calling build_stl_str_hl, then drawing
//! the result to the grid with highlight records.

use std::cell::Cell;
use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use nvim_window::WinHandle;

use crate::ScharT;

// =============================================================================
// Constants (verified via _Static_assert in statusline.c)
// =============================================================================

const MAXPATHL: usize = 4096;
const NUL: u8 = 0;

const K_OPT_INVALID: c_int = -1;
const K_OPT_TABLINE: c_int = 302;
const K_OPT_WINBAR: c_int = 355;
const K_OPT_STATUSLINE: c_int = 294;
const K_OPT_RULERFORMAT: c_int = 241;
const OPT_LOCAL: c_int = 0x02;

// HLF constants
const HLF_TPF: c_int = 54;
const HLF_WBR: c_int = 65;
const HLF_WBRNC: c_int = 66;
const HLF_MSG: c_int = 63;

/// Opaque handle for stl_hlrec_t
type StlHlrecPtr = *mut c_void;
/// Opaque handle for StlClickRecord
type StlClickRecordPtr = *mut c_void;
/// Opaque handle for ScreenGrid
type GridHandle = *mut c_void;
/// Opaque handle for click definitions
type ClickDefsHandle = *mut c_void;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Window layout
    fn nvim_stl_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_status_height(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_winrow_off(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_wincol_off(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_view_height(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_view_width(wp: WinHandle) -> c_int;
    fn nvim_stl_W_ENDROW(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_wincol(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_fcs_wbr(wp: WinHandle) -> ScharT;

    // Global state
    fn nvim_stl_get_Columns() -> c_int;
    fn nvim_stl_get_Rows() -> c_int;
    fn nvim_stl_get_p_ch() -> i64;
    fn nvim_stl_get_ru_col() -> c_int;
    fn nvim_stl_get_curwin() -> WinHandle;
    fn nvim_global_stl_height() -> c_int;

    // Option strings
    fn nvim_stl_get_p_tal() -> *mut c_char;
    fn nvim_stl_get_p_ruf() -> *mut c_char;
    fn nvim_stl_get_p_stl() -> *const c_char;
    fn nvim_stl_win_get_p_stl(wp: WinHandle) -> *const c_char;
    fn nvim_stl_get_p_wbr() -> *const c_char;
    fn nvim_stl_win_get_p_wbr(wp: WinHandle) -> *mut c_char;

    // Fill character
    fn nvim_stl_fillchar_status(group: *mut c_int, wp: WinHandle) -> ScharT;
    fn nvim_stl_schar_from_ascii_char(c: c_char) -> ScharT;

    // Grid operations
    fn nvim_stl_grid_adjust_win(wp: WinHandle, row: *mut c_int, col: *mut c_int) -> GridHandle;
    fn nvim_stl_grid_adjust_msg(row: *mut c_int, col: *mut c_int) -> GridHandle;
    fn nvim_stl_get_default_grid() -> GridHandle;
    fn nvim_stl_win_get_grid_alloc(wp: WinHandle) -> GridHandle;
    fn nvim_stl_screengrid_line_start(grid: GridHandle, row: c_int, col: c_int);
    fn nvim_stl_grid_line_puts(
        col: c_int,
        text: *const c_char,
        textlen: c_int,
        attr: c_int,
    ) -> c_int;
    fn nvim_stl_grid_line_fill(start: c_int, end: c_int, fillchar: ScharT, attr: c_int);
    fn nvim_stl_grid_line_flush();

    // Highlight
    fn nvim_stl_win_hl_attr(wp: WinHandle, hlf: c_int) -> c_int;
    fn nvim_stl_hl_combine_attr(a: c_int, b: c_int) -> c_int;
    fn nvim_stl_syn_id2attr(id: c_int) -> c_int;
    fn nvim_stl_HL_ATTR(hlf: c_int) -> c_int;
    fn nvim_stl_highlight_user_arr(index: c_int) -> c_int;
    fn nvim_stl_highlight_stlnc_arr(index: c_int) -> c_int;
    fn nvim_stl_syn_name2id_len(name: *const c_char, len: c_int) -> c_int;

    // String operations
    fn nvim_stl_transstr_buf(
        s: *const c_char,
        len: c_int,
        buf: *mut c_char,
        buflen: usize,
    ) -> usize;
    fn nvim_stl_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_stl_xfree(ptr: *mut c_void);

    // Click definitions
    fn nvim_stl_clear_click_defs_wrap(defs: ClickDefsHandle, size: usize);
    fn nvim_stl_alloc_click_defs_wrap(
        cdp: ClickDefsHandle,
        width: c_int,
        size: *mut usize,
    ) -> ClickDefsHandle;
    fn nvim_stl_fill_click_defs_wrap(
        defs: ClickDefsHandle,
        recs: StlClickRecordPtr,
        buf: *const c_char,
        width: c_int,
        tabline: bool,
    );
    fn nvim_stl_get_tab_page_click_defs() -> ClickDefsHandle;

    // Window click defs
    fn nvim_stl_win_get_status_click_defs(wp: WinHandle) -> ClickDefsHandle;
    fn nvim_stl_win_get_status_click_defs_size(wp: WinHandle) -> usize;
    fn nvim_stl_win_set_status_click_defs(wp: WinHandle, defs: ClickDefsHandle);
    fn nvim_stl_win_set_status_click_defs_size(wp: WinHandle, size: usize);
    fn nvim_stl_win_get_winbar_click_defs(wp: WinHandle) -> ClickDefsHandle;
    fn nvim_stl_win_get_winbar_click_defs_size(wp: WinHandle) -> usize;
    fn nvim_stl_win_set_winbar_click_defs(wp: WinHandle, defs: ClickDefsHandle);
    fn nvim_stl_win_set_winbar_click_defs_size(wp: WinHandle, size: usize);

    // Window width
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;

    // Cursorbind
    fn nvim_stl_win_get_p_crb(wp: WinHandle) -> c_int;
    fn nvim_stl_win_set_p_crb(wp: WinHandle, val: c_int);

    // UI events
    fn nvim_stl_ui_call_msg_ruler_content(
        attrs: *const c_int,
        texts: *const *const c_char,
        tsizes: *const usize,
        groups: *const c_int,
        count: c_int,
    );

    // build_stl_str_hl (Rust, cross-crate via FFI)
    fn rs_build_stl_str_hl_wrap(
        wp: WinHandle,
        out: *mut c_char,
        outlen: usize,
        fmt: *mut c_char,
        opt_idx: c_int,
        opt_scope: c_int,
        fillchar: ScharT,
        maxwidth: c_int,
        hltab: *mut *mut StlHlRec,
        hltab_len: *mut usize,
        tabtab: *mut StlClickRecordPtr,
        stcp: *mut c_void,
    ) -> c_int;

    // strlen
    fn strlen(s: *const c_char) -> usize;
}

// =============================================================================
// Highlight record structure (matches C stl_hlrec_t)
// =============================================================================

/// Matches the C stl_hlrec_t structure.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct StlHlRec {
    start: *mut c_char,
    userhl: c_int,
}

// =============================================================================
// Recursion guard
// =============================================================================

thread_local! {
    static ENTERED: Cell<bool> = const { Cell::new(false) };
}

// =============================================================================
// Main implementation
// =============================================================================

/// Rust implementation of `win_redr_custom_impl`.
///
/// Renders a custom statusline, winbar, tabline, or ruler format string
/// to the grid, handling highlight records, click definitions, and UI events.
///
/// # Safety
/// - `wp` may be null (for tabline rendering).
/// - Accesses global C state (grid, highlight arrays, options).
#[allow(clippy::too_many_lines)]
pub unsafe fn win_redr_custom(wp: WinHandle, draw_winbar: bool, draw_ruler: bool, ui_event: bool) {
    // Recursion guard
    if ENTERED.with(|e| e.get()) {
        return;
    }
    ENTERED.with(|e| e.set(true));

    let mut col: c_int = 0;
    let mut row: c_int;
    let mut maxwidth: c_int;
    let mut group: c_int = 0;
    let mut fillchar: ScharT;
    let mut buf = [0u8; MAXPATHL];
    let mut stl: *mut c_char;
    let mut opt_idx: c_int = K_OPT_INVALID;
    let mut opt_scope: c_int = 0;
    let mut hltab: *mut StlHlRec = ptr::null_mut();
    let mut tabtab: StlClickRecordPtr = ptr::null_mut();
    let is_stl_global = nvim_global_stl_height() > 0;

    let grid: GridHandle = if !wp.is_null() && nvim_stl_win_get_floating(wp) != 0 && !is_stl_global
    {
        nvim_stl_win_get_grid_alloc(wp)
    } else {
        nvim_stl_get_default_grid()
    };

    // Setup environment based on mode
    if wp.is_null() {
        // Tabline mode
        stl = nvim_stl_get_p_tal();
        row = 0;
        fillchar = nvim_stl_schar_from_ascii_char(b' ' as c_char);
        group = HLF_TPF;
        let attr = nvim_stl_HL_ATTR(group);
        maxwidth = nvim_stl_get_Columns();
        opt_idx = K_OPT_TABLINE;

        // Early exit check
        if maxwidth <= 0 {
            ENTERED.with(|e| e.set(false));
            return;
        }

        // Temporarily reset cursorbind
        let ewp = nvim_stl_get_curwin();
        let p_crb_save = nvim_stl_win_get_p_crb(ewp);
        nvim_stl_win_set_p_crb(ewp, 0);

        // Make a copy
        stl = nvim_stl_xstrdup(stl);
        rs_build_stl_str_hl_wrap(
            ewp,
            buf.as_mut_ptr().cast(),
            buf.len(),
            stl,
            opt_idx,
            opt_scope,
            fillchar,
            maxwidth,
            &mut hltab,
            ptr::null_mut(),
            &mut tabtab,
            ptr::null_mut(),
        );
        nvim_stl_xfree(stl.cast());
        nvim_stl_win_set_p_crb(ewp, p_crb_save);

        draw_result(
            wp,
            grid,
            row,
            col,
            maxwidth,
            attr,
            group,
            fillchar,
            buf.as_ptr().cast(),
            &mut hltab,
            &mut tabtab,
            ui_event,
            draw_winbar,
            is_stl_global,
        );
        ENTERED.with(|e| e.set(false));
        return;
    }

    let attr: c_int;

    if draw_winbar {
        // Winbar mode
        opt_idx = K_OPT_WINBAR;
        let w_p_wbr = nvim_stl_win_get_p_wbr(wp);
        let p_wbr = nvim_stl_get_p_wbr();
        if !w_p_wbr.is_null() && *w_p_wbr != NUL as c_char {
            stl = w_p_wbr;
            opt_scope = OPT_LOCAL;
        } else {
            stl = p_wbr as *mut c_char;
            opt_scope = 0;
        }
        row = -1; // row zero is first row of text
        col = 0;
        let _grid = nvim_stl_grid_adjust_win(wp, &mut row, &mut col);

        if row < 0 {
            ENTERED.with(|e| e.set(false));
            return;
        }

        fillchar = nvim_stl_win_get_fcs_wbr(wp);
        let curwin = nvim_stl_get_curwin();
        group = if wp == curwin { HLF_WBR } else { HLF_WBRNC };
        attr = nvim_stl_win_hl_attr(wp, group);
        maxwidth = nvim_stl_win_get_view_width(wp);

        // Clear and allocate click defs
        let click_defs = nvim_stl_win_get_winbar_click_defs(wp);
        let click_defs_size = nvim_stl_win_get_winbar_click_defs_size(wp);
        nvim_stl_clear_click_defs_wrap(click_defs, click_defs_size);
        let mut new_size = click_defs_size;
        let new_defs = nvim_stl_alloc_click_defs_wrap(click_defs, maxwidth, &mut new_size);
        nvim_stl_win_set_winbar_click_defs(wp, new_defs);
        nvim_stl_win_set_winbar_click_defs_size(wp, new_size);
    } else {
        // Statusline or Ruler mode
        let in_status_line = nvim_stl_win_get_status_height(wp) != 0 || is_stl_global;
        if nvim_stl_win_get_floating(wp) != 0 && !is_stl_global && !draw_ruler {
            row = nvim_stl_win_get_winrow_off(wp) + nvim_stl_win_get_view_height(wp);
            col = nvim_stl_win_get_wincol_off(wp);
            maxwidth = nvim_stl_win_get_view_width(wp);
        } else {
            row = if is_stl_global {
                nvim_stl_get_Rows() - nvim_stl_get_p_ch() as c_int - 1
            } else {
                nvim_stl_W_ENDROW(wp)
            };
            maxwidth = if in_status_line && !is_stl_global {
                nvim_win_get_w_width(wp)
            } else {
                nvim_stl_get_Columns()
            };
        }

        fillchar = nvim_stl_fillchar_status(&mut group, wp);

        // Clear and allocate click defs
        let click_defs = nvim_stl_win_get_status_click_defs(wp);
        let click_defs_size = nvim_stl_win_get_status_click_defs_size(wp);
        nvim_stl_clear_click_defs_wrap(click_defs, click_defs_size);
        let mut new_size = click_defs_size;
        let new_defs = nvim_stl_alloc_click_defs_wrap(click_defs, maxwidth, &mut new_size);
        nvim_stl_win_set_status_click_defs(wp, new_defs);
        nvim_stl_win_set_status_click_defs_size(wp, new_size);

        if draw_ruler {
            stl = nvim_stl_get_p_ruf();
            opt_idx = K_OPT_RULERFORMAT;
            // Advance past leading group spec - implicit in ru_col
            if !stl.is_null() && *stl as u8 == b'%' {
                stl = stl.add(1);
                if *stl as u8 == b'-' {
                    stl = stl.add(1);
                }
                // Skip digits
                while (*stl as u8).is_ascii_digit() {
                    stl = stl.add(1);
                }
                if *stl as u8 != b'(' {
                    stl = nvim_stl_get_p_ruf();
                } else {
                    stl = stl.add(1);
                }
            }
            let columns = nvim_stl_get_Columns();
            let ru_col = nvim_stl_get_ru_col();
            let offset = ru_col - (columns - maxwidth);
            let half = (maxwidth + 1) / 2;
            col = if offset > half { offset } else { half };
            maxwidth -= col;
            if !in_status_line {
                row = nvim_stl_get_Rows() - 1;
                let _grid = nvim_stl_grid_adjust_msg(&mut row, &mut col);
                maxwidth -= 1; // writing in last column may cause scrolling
                fillchar = nvim_stl_schar_from_ascii_char(b' ' as c_char);
                group = HLF_MSG;
            }
        } else {
            opt_idx = K_OPT_STATUSLINE;
            let w_p_stl = nvim_stl_win_get_p_stl(wp);
            let p_stl = nvim_stl_get_p_stl();
            if !w_p_stl.is_null() && *w_p_stl != NUL as c_char {
                stl = w_p_stl as *mut c_char;
                opt_scope = OPT_LOCAL;
            } else {
                stl = p_stl as *mut c_char;
                opt_scope = 0;
            }
        }

        attr = nvim_stl_win_hl_attr(wp, group);
        if nvim_stl_win_get_floating(wp) == 0 && in_status_line && !is_stl_global {
            col += nvim_stl_win_get_wincol(wp);
        }
    }

    if maxwidth <= 0 {
        ENTERED.with(|e| e.set(false));
        return;
    }

    // Temporarily reset cursorbind
    let ewp = wp;
    let p_crb_save = nvim_stl_win_get_p_crb(ewp);
    nvim_stl_win_set_p_crb(ewp, 0);

    // Make a copy of stl
    stl = nvim_stl_xstrdup(stl);
    rs_build_stl_str_hl_wrap(
        ewp,
        buf.as_mut_ptr().cast(),
        buf.len(),
        stl,
        opt_idx,
        opt_scope,
        fillchar,
        maxwidth,
        &mut hltab,
        ptr::null_mut(),
        &mut tabtab,
        ptr::null_mut(),
    );
    nvim_stl_xfree(stl.cast());
    nvim_stl_win_set_p_crb(ewp, p_crb_save);

    draw_result(
        wp,
        grid,
        row,
        col,
        maxwidth,
        attr,
        group,
        fillchar,
        buf.as_ptr().cast(),
        &mut hltab,
        &mut tabtab,
        ui_event,
        draw_winbar,
        is_stl_global,
    );

    ENTERED.with(|e| e.set(false));
}

/// Draw the formatted statusline result to the grid.
///
/// Iterates highlight records, calls grid_line_puts for each segment,
/// handles UI event path (msg_ruler), fills remaining space, and
/// populates click definitions.
#[allow(clippy::too_many_arguments)]
unsafe fn draw_result(
    wp: WinHandle,
    grid: GridHandle,
    row: c_int,
    col: c_int,
    maxwidth: c_int,
    attr: c_int,
    group: c_int,
    fillchar: ScharT,
    buf: *const c_char,
    hltab: &mut *mut StlHlRec,
    tabtab: &mut StlClickRecordPtr,
    ui_event: bool,
    draw_winbar: bool,
    is_stl_global: bool,
) {
    let len = strlen(buf) as c_int;
    let start_col = col;
    let mut col = col;
    let mut transbuf = [0u8; MAXPATHL];

    if !ui_event {
        nvim_stl_screengrid_line_start(grid, row, 0);
    }

    let mut p = buf as *mut c_char;
    let mut curattr = attr;
    let mut curgroup = group;

    // For UI event path, collect chunks
    let mut ui_attrs: Vec<c_int> = Vec::new();
    let mut ui_texts: Vec<*const c_char> = Vec::new();
    let mut ui_tsizes: Vec<usize> = Vec::new();
    let mut ui_groups: Vec<c_int> = Vec::new();

    let mut sp = *hltab;
    loop {
        let sp_start = (*sp).start;
        let textlen = if !sp_start.is_null() {
            sp_start.offset_from(p) as c_int
        } else {
            buf.add(len as usize).offset_from(p) as c_int
        };

        // Make all characters printable
        let src = if (p as *const c_char) >= buf.add(len as usize) {
            b"\0".as_ptr().cast()
        } else {
            p as *const c_char
        };
        let tsize =
            nvim_stl_transstr_buf(src, textlen, transbuf.as_mut_ptr().cast(), transbuf.len());

        if !ui_event {
            col += nvim_stl_grid_line_puts(col, transbuf.as_ptr().cast(), tsize as c_int, curattr);
        } else {
            // For UI event, we need to duplicate the text
            let dup = nvim_stl_xstrdup(transbuf.as_ptr().cast());
            ui_attrs.push(curattr);
            ui_texts.push(dup);
            ui_tsizes.push(tsize);
            ui_groups.push(curgroup);
        }

        p = sp_start;

        if p.is_null() {
            break;
        }

        let userhl = (*sp).userhl;
        if userhl == 0 {
            curattr = attr;
            curgroup = group;
        } else if userhl < 0 {
            curattr = nvim_stl_syn_id2attr(-userhl);
            curgroup = -userhl;
        } else {
            let curwin = nvim_stl_get_curwin();
            let use_stlnc =
                !wp.is_null() && wp != curwin && nvim_stl_win_get_status_height(wp) != 0;
            let user_attr = if use_stlnc {
                nvim_stl_highlight_stlnc_arr(userhl - 1)
            } else {
                nvim_stl_highlight_user_arr(userhl - 1)
            };
            // Build "User" + digit name for syn_name2id
            let mut userbuf = [0u8; 6];
            userbuf[0] = b'U';
            userbuf[1] = b's';
            userbuf[2] = b'e';
            userbuf[3] = b'r';
            userbuf[4] = userhl as u8 + b'0';
            curattr = user_attr;
            curgroup = nvim_stl_syn_name2id_len(userbuf.as_ptr().cast(), 5);
        }

        if curattr != attr {
            curattr = nvim_stl_hl_combine_attr(attr, curattr);
        }

        sp = sp.add(1);
    }

    if ui_event {
        // Call ui_call_msg_ruler with accumulated content
        if !ui_attrs.is_empty() {
            nvim_stl_ui_call_msg_ruler_content(
                ui_attrs.as_ptr(),
                ui_texts.as_ptr(),
                ui_tsizes.as_ptr(),
                ui_groups.as_ptr(),
                ui_attrs.len() as c_int,
            );
        } else {
            // Empty ruler
            nvim_stl_ui_call_msg_ruler_content(
                ptr::null(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                0,
            );
        }
        // Free duplicated text strings
        for text in &ui_texts {
            nvim_stl_xfree(*text as *mut c_void);
        }
        ENTERED.with(|e| e.set(false));
        return;
    }

    let maxcol = start_col + maxwidth;

    // Fill remaining with fillchar
    nvim_stl_grid_line_fill(col, maxcol, fillchar, curattr);
    nvim_stl_grid_line_flush();

    // Fill click definitions
    let click_defs: ClickDefsHandle = if wp.is_null() {
        nvim_stl_get_tab_page_click_defs()
    } else if draw_winbar {
        nvim_stl_win_get_winbar_click_defs(wp)
    } else {
        nvim_stl_win_get_status_click_defs(wp)
    };

    nvim_stl_fill_click_defs_wrap(click_defs, *tabtab, buf, maxwidth, wp.is_null());
}
