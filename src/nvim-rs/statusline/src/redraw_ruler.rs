//! Ruler redraw implementation - direct port of `nvim_stl_redraw_ruler_impl`
//!
//! Checks if ruler should display, delegates to `win_redr_custom` for custom
//! `rulerformat`, or formats standard "line,col percentage" string and draws
//! to grid or emits UI event.

use std::cell::Cell;
use std::ffi::{c_char, c_int};
use std::io::Write;

use nvim_window::WinHandle;

use crate::ScharT;

// =============================================================================
// Constants (verified via _Static_assert in statusline.c)
// =============================================================================

const RULER_BUF_LEN: usize = 70;
const MODE_INSERT: c_int = 0x10;
const HLF_MSG: c_int = 63;
const NUL: c_int = 0;

// =============================================================================
// Thread-local static state
// =============================================================================

thread_local! {
    static DID_RULER_COL: Cell<c_int> = const { Cell::new(-1) };
    static DID_SHOW_EXT_RULER: Cell<bool> = const { Cell::new(false) };
}

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Window accessors
    fn nvim_stl_win_get_status_height(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_wincol(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_w_virtcol(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_cursor_col(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_cursor_lnum(wp: WinHandle) -> c_int;
    fn nvim_stl_win_cursor_invalid(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_p_list(wp: WinHandle) -> c_int;
    fn nvim_stl_win_set_p_list(wp: WinHandle, val: c_int);
    fn nvim_stl_win_get_lcs_tab1(wp: WinHandle) -> c_int;
    fn nvim_stl_getvvcol_cursor(wp: WinHandle) -> c_int;
    fn nvim_stl_ml_get_buf_first_char(wp: WinHandle) -> c_int;

    // Global state
    fn nvim_stl_get_curwin() -> WinHandle;
    fn nvim_stl_lastwin_nofloating() -> WinHandle;
    #[link_name = "rs_global_stl_height"]
    fn nvim_global_stl_height() -> c_int;
    fn nvim_stl_get_p_ru() -> c_int;
    fn nvim_stl_get_p_ch() -> i64;
    fn nvim_stl_get_p_ruf() -> *mut c_char;
    fn nvim_stl_get_Columns() -> c_int;
    fn nvim_stl_get_Rows() -> c_int;
    fn nvim_stl_get_ru_col() -> c_int;
    fn nvim_stl_get_State() -> c_int;
    fn nvim_stl_edit_submode_not_null() -> c_int;
    fn nvim_stl_ui_has_messages() -> c_int;
    fn nvim_stl_buf_ml_empty(buf: *mut std::ffi::c_void) -> c_int;
    fn nvim_win_get_buffer(wp: WinHandle) -> *mut std::ffi::c_void;

    // Fill character / highlight
    fn nvim_stl_fillchar_status(group: *mut c_int, wp: WinHandle) -> ScharT;
    #[link_name = "rs_schar_from_ascii"]
    fn nvim_stl_schar_from_ascii_char(c: c_char) -> ScharT;
    fn nvim_stl_win_hl_attr(wp: WinHandle, hlf: c_int) -> c_int;
    fn nvim_stl_HL_ATTR(hlf: c_int) -> c_int;

    // String operations (direct link to Rust/C implementations)
    #[link_name = "schar_get"]
    fn nvim_stl_schar_get(buf: *mut c_char, c: ScharT) -> usize;
    #[link_name = "vim_strsize"]
    fn nvim_stl_vim_strsize(s: *const c_char) -> c_int;
    #[link_name = "utfc_ptr2len"]
    fn nvim_stl_utfc_ptr2len(s: *const c_char) -> c_int;
    #[link_name = "ptr2cells"]
    fn nvim_stl_ptr2cells(s: *const c_char) -> c_int;

    // Relative position (C wrapper that calls get_rel_pos)
    fn nvim_stl_get_rel_pos(wp: WinHandle, buf: *mut c_char, buflen: c_int) -> c_int;

    // Grid operations
    fn nvim_stl_msg_grid_line_start(row: c_int);
    fn nvim_stl_grid_line_puts(
        col: c_int,
        text: *const c_char,
        textlen: c_int,
        attr: c_int,
    ) -> c_int;
    fn nvim_stl_grid_line_fill(start: c_int, end: c_int, fillchar: ScharT, attr: c_int);
    fn nvim_stl_grid_line_flush();

    // Message area
    fn nvim_stl_set_msg_col(col: c_int);
    fn nvim_stl_set_msg_row(row: c_int);
    fn nvim_stl_msg_clr_eos();

    // UI events
    fn nvim_stl_ui_call_msg_ruler_empty();
    fn nvim_stl_ui_call_msg_ruler_content(
        attrs: *const c_int,
        texts: *const *const c_char,
        tsizes: *const usize,
        groups: *const c_int,
        count: c_int,
    );

    // win_redr_custom (from Rust, already migrated)
    fn rs_win_redr_custom(wp: WinHandle, draw_winbar: bool, draw_ruler: bool, ui_event: bool);
}

// =============================================================================
// Main implementation
// =============================================================================

/// Redraw the ruler - main entry point.
///
/// # Safety
/// Accesses global C state.
pub unsafe fn redraw_ruler() {
    let curwin = nvim_stl_get_curwin();
    let wp = if nvim_stl_win_get_status_height(curwin) == 0 {
        curwin
    } else {
        nvim_stl_lastwin_nofloating()
    };
    let is_stl_global = nvim_global_stl_height() > 0;

    let p_ru = nvim_stl_get_p_ru() != 0;
    let status_height = nvim_stl_win_get_status_height(wp);
    let p_ch = nvim_stl_get_p_ch();
    let ui_has_messages = nvim_stl_ui_has_messages() != 0;

    // Check if ruler should be drawn, clear if it was drawn before.
    if !p_ru || status_height > 0 || is_stl_global || (p_ch == 0 && !ui_has_messages) {
        let did_ruler_col = DID_RULER_COL.get();
        if did_ruler_col > 0 && ui_has_messages {
            nvim_stl_ui_call_msg_ruler_empty();
            DID_SHOW_EXT_RULER.set(false);
        } else if did_ruler_col > 0 {
            nvim_stl_set_msg_col(did_ruler_col);
            nvim_stl_set_msg_row(nvim_stl_get_Rows() - 1);
            nvim_stl_msg_clr_eos();
        }
        DID_RULER_COL.set(-1);
        return;
    }

    // Check if cursor.lnum is valid
    if nvim_stl_win_cursor_invalid(wp) != 0 {
        return;
    }

    // Don't draw the ruler while doing insert-completion
    if status_height == 0 && !is_stl_global && nvim_stl_edit_submode_not_null() != 0 {
        return;
    }

    let part_of_status = status_height != 0 || is_stl_global;
    let p_ruf = nvim_stl_get_p_ruf();
    if !p_ruf.is_null() && *p_ruf != 0 && (p_ch > 0 || (ui_has_messages && !part_of_status)) {
        rs_win_redr_custom(wp, false, true, ui_has_messages);
        return;
    }

    let mut group: c_int = HLF_MSG;
    let off = if status_height != 0 {
        nvim_stl_win_get_wincol(wp)
    } else {
        0
    };
    let width = if status_height != 0 {
        nvim_win_get_w_width(wp)
    } else {
        nvim_stl_get_Columns()
    };
    let fillchar = if part_of_status {
        nvim_stl_fillchar_status(&mut group, wp)
    } else {
        nvim_stl_schar_from_ascii_char(b' ' as c_char)
    };
    let attr = nvim_stl_win_hl_attr(wp, group);

    // In list mode virtcol needs to be recomputed
    let mut virtcol = nvim_stl_win_get_w_virtcol(wp);
    if nvim_stl_win_get_p_list(wp) != 0 && nvim_stl_win_get_lcs_tab1(wp) == NUL {
        nvim_stl_win_set_p_list(wp, 0);
        virtcol = nvim_stl_getvvcol_cursor(wp);
        nvim_stl_win_set_p_list(wp, 1);
    }

    // Check if not in Insert mode and the line is empty (will show "0-1").
    let empty_line =
        (nvim_stl_get_State() & MODE_INSERT) == 0 && nvim_stl_ml_get_buf_first_char(wp) == NUL;

    let mut buffer = [0u8; RULER_BUF_LEN];

    // Format "lnum," part
    let buf = nvim_win_get_buffer(wp);
    let ml_empty = if buf.is_null() {
        false
    } else {
        nvim_stl_buf_ml_empty(buf) != 0
    };

    let cursor_lnum = nvim_stl_win_get_cursor_lnum(wp);
    let lnum_val: i64 = if ml_empty { 0 } else { i64::from(cursor_lnum) };

    let mut cursor = std::io::Cursor::new(&mut buffer[..]);
    let _ = write!(cursor, "{lnum_val},");
    let mut bufferlen = cursor.position() as usize;

    // col_print: format col[-vcol]
    let col_val = if empty_line {
        0
    } else {
        nvim_stl_win_get_cursor_col(wp) + 1
    };
    let vcol_val = virtcol + 1;

    let col_buf = &mut buffer[bufferlen..];
    let mut col_cursor = std::io::Cursor::new(col_buf);
    if col_val == vcol_val {
        let _ = write!(col_cursor, "{col_val}");
    } else {
        let _ = write!(col_cursor, "{col_val}-{vcol_val}");
    }
    bufferlen += col_cursor.position() as usize;

    // Add a "50%" if there is room for it.
    let mut rel_pos = [0u8; RULER_BUF_LEN];
    let rel_poslen = nvim_stl_get_rel_pos(
        wp,
        rel_pos.as_mut_ptr().cast::<c_char>(),
        RULER_BUF_LEN as c_int,
    );

    // NUL-terminate rel_pos for vim_strsize
    if (rel_poslen as usize) < RULER_BUF_LEN {
        rel_pos[rel_poslen as usize] = 0;
    }

    let n1_base = bufferlen as c_int + nvim_stl_vim_strsize(rel_pos.as_ptr().cast::<c_char>());
    let mut n1 = n1_base;
    if status_height == 0 && !is_stl_global {
        // can't use last char of screen
        n1 += 1;
    }

    let columns = nvim_stl_get_Columns();
    let ru_col = nvim_stl_get_ru_col();
    let mut this_ru_col = ru_col - (columns - width);
    // Never use more than half the window/screen width
    let n2 = (width + 1) / 2;
    if this_ru_col < n2 {
        this_ru_col = n2;
    }

    if this_ru_col + n1 < width {
        // need at least space for rel_pos + NUL
        while this_ru_col + n1 < width && RULER_BUF_LEN > bufferlen + (rel_poslen as usize) + 1 {
            let written =
                nvim_stl_schar_get(buffer[bufferlen..].as_mut_ptr().cast::<c_char>(), fillchar);
            bufferlen += written;
            n1 += 1;
        }
        // Append rel_pos
        let rel_bytes = rel_poslen as usize;
        if bufferlen + rel_bytes < RULER_BUF_LEN {
            buffer[bufferlen..bufferlen + rel_bytes].copy_from_slice(&rel_pos[..rel_bytes]);
            bufferlen += rel_bytes;
        }
    }

    // NUL-terminate buffer
    if bufferlen < RULER_BUF_LEN {
        buffer[bufferlen] = 0;
    }

    if ui_has_messages && !part_of_status {
        // Send ruler content via UI event
        let attr_arr = [attr];
        let text_ptr = buffer.as_ptr().cast::<c_char>();
        let texts = [text_ptr];
        let tsizes = [bufferlen];
        let groups = [HLF_MSG];

        // assert: attr == HL_ATTR(HLF_MSG)
        debug_assert_eq!(attr, nvim_stl_HL_ATTR(HLF_MSG));

        nvim_stl_ui_call_msg_ruler_content(
            attr_arr.as_ptr(),
            texts.as_ptr(),
            tsizes.as_ptr(),
            groups.as_ptr(),
            1,
        );
        DID_SHOW_EXT_RULER.set(true);
        DID_RULER_COL.set(1);
    } else {
        if DID_SHOW_EXT_RULER.get() {
            nvim_stl_ui_call_msg_ruler_empty();
            DID_SHOW_EXT_RULER.set(false);
        }

        // Truncate at window boundary
        let mut byte_pos: usize = 0;
        let mut cell_width: c_int = 0;
        while byte_pos < bufferlen && buffer[byte_pos] != 0 {
            let ptr = buffer[byte_pos..].as_ptr().cast::<c_char>();
            let char_cells = nvim_stl_ptr2cells(ptr);
            if this_ru_col + cell_width + char_cells > width {
                bufferlen = byte_pos;
                buffer[bufferlen] = 0;
                break;
            }
            cell_width += char_cells;
            byte_pos += nvim_stl_utfc_ptr2len(ptr) as usize;
        }

        let rows = nvim_stl_get_Rows();
        nvim_stl_msg_grid_line_start(rows - 1);
        let did_ruler_col = off + this_ru_col;
        DID_RULER_COL.set(did_ruler_col);
        let w = nvim_stl_grid_line_puts(did_ruler_col, buffer.as_ptr().cast::<c_char>(), -1, attr);
        nvim_stl_grid_line_fill(did_ruler_col + w, off + width, fillchar, attr);
        nvim_stl_grid_line_flush();
    }
}
