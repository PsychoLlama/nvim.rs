//! Line drawing functions for Neovim
//!
//! This crate provides Rust implementations of line drawing functions
//! from `src/nvim/drawline.c`, focusing on column rendering and helpers.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::if_not_else)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::missing_safety_doc)]
#![allow(dead_code)]

use std::ffi::c_int;
use std::ffi::c_void;

use nvim_highlight::{rs_syn_attr2entry, HlAttrs};
use nvim_window::WinHandle;

/// schar_T is stored as a u32.
type ScharT = u32;

/// Line number type.
type LinenrT = i64;

/// Column number type.
type ColnrT = i32;

/// Opaque handle to winlinevars_T (line drawing state).
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct WlvHandle(*mut c_void);

impl WlvHandle {
    /// Check if the handle is null.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Fold info structure (matching C foldinfo_T).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldInfo {
    /// Line number where fold starts.
    pub fi_lnum: LinenrT,
    /// Level of the fold (0 = no fold).
    pub fi_level: c_int,
    /// Lowest fold level that starts in the same line.
    pub fi_low_level: c_int,
    /// Number of lines the fold spans (0 if not closed).
    pub fi_lines: LinenrT,
}

// Highlight group constants (from highlight_defs.h)
pub const HLF_AT: c_int = 4; // NonText (@ characters)
pub const HLF_N: c_int = 12; // LineNr
pub const HLF_LNA: c_int = 13; // LineNrAbove
pub const HLF_LNB: c_int = 14; // LineNrBelow
pub const HLF_CLN: c_int = 15; // CursorLineNr
pub const HLF_CLS: c_int = 16; // CursorLineSign
pub const HLF_CLF: c_int = 17; // CursorLineFold
pub const HLF_FC: c_int = 29; // FoldColumn
pub const HLF_DED: c_int = 31; // DiffDelete (deleted diff line)
pub const HLF_SC: c_int = 35; // SignColumn

// Cursorlineopt flags (from option_vars.generated.h)
pub const K_OPT_CULOPT_FLAG_LINE: c_int = 0x01;
pub const K_OPT_CULOPT_FLAG_SCREENLINE: c_int = 0x02;
pub const K_OPT_CULOPT_FLAG_NUMBER: c_int = 0x04;

/// Sign width constant.
pub const SIGN_WIDTH: c_int = 2;

/// Mode constants (from state_defs.h)
const MODE_INSERT: c_int = 0x10;

// C accessor functions for window
extern "C" {
    fn nvim_win_get_p_wrap(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_list(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_cuc(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_cul(wp: WinHandle) -> c_int;
    fn nvim_win_get_wrap_flags(wp: WinHandle) -> c_int;
    fn nvim_win_get_lcs_ext(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_foldclosed(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_foldopen(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_foldsep(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_foldinner(wp: WinHandle) -> ScharT;
    fn nvim_win_get_view_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_virtcol(wp: WinHandle) -> ColnrT;
    fn nvim_win_get_cursorline(wp: WinHandle) -> LinenrT;
    fn nvim_win_get_p_culopt_flags(wp: WinHandle) -> c_int;
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> LinenrT;
    fn nvim_win_get_p_rnu(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_nu(wp: WinHandle) -> c_int;
    fn nvim_win_get_topline(wp: WinHandle) -> LinenrT;
    fn nvim_win_get_skipcol(wp: WinHandle) -> ColnrT;
    fn nvim_win_get_p_bri(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_rl(wp: WinHandle) -> c_int;
    fn nvim_win_get_minscwidth(wp: WinHandle) -> c_int;

    // Highlight functions
    fn nvim_win_hl_attr(wp: WinHandle, hlf: c_int) -> c_int;
    fn hl_combine_attr(char_attr: c_int, prim_attr: c_int) -> c_int;
    fn hl_blend_attrs(back_attr: c_int, front_attr: c_int, through: *mut bool) -> c_int;
    fn syn_id2attr(hl_id: c_int) -> c_int;

    // Grid functions for schar operations
    fn rs_schar_from_char(c: c_int) -> ScharT;

    // Display width functions
    fn rs_win_col_off(wp: WinHandle) -> c_int;
    fn rs_win_col_off2(wp: WinHandle) -> c_int;
    fn rs_number_width(wp: WinHandle) -> c_int;

    // Linebuf access
    fn nvim_get_linebuf_char() -> *mut ScharT;
    fn nvim_get_linebuf_attr() -> *mut c_int;
    fn nvim_get_linebuf_vcol() -> *mut ColnrT;

    // WLV accessor functions
    fn nvim_wlv_get_lnum(wlv: WlvHandle) -> LinenrT;
    fn nvim_wlv_get_foldinfo(wlv: WlvHandle) -> FoldInfo;
    fn nvim_wlv_get_row(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_get_startrow(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_get_off(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_set_off(wlv: WlvHandle, val: c_int);
    fn nvim_wlv_get_filler_lines(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_get_filler_todo(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_get_sign_num_attr(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_get_sign_cul_attr(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_get_prev_num_attr(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_set_prev_num_attr(wlv: WlvHandle, val: c_int);
    fn nvim_wlv_get_n_virt_lines(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_get_n_virt_below(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_get_sattr_text(wlv: WlvHandle, sign_idx: c_int, char_idx: c_int) -> ScharT;
    fn nvim_wlv_get_sattr_hl_id(wlv: WlvHandle, sign_idx: c_int) -> c_int;

    // Decoration function for sign numhl lookup
    fn decor_redraw_signs(
        wp: WinHandle,
        buf: *mut c_void,
        row: LinenrT,
        sattrs: *mut c_void,
        line_id: *mut c_int,
        cul_id: *mut c_int,
        num_id: *mut c_int,
    );

    // Buffer handle for decoration
    fn nvim_win_get_buffer(wp: WinHandle) -> *mut c_void;

    // Buffer accessor functions (for line_putchar)
    fn nvim_buf_get_p_ts(buf: BufHandle) -> i64;
    fn nvim_buf_get_p_vts_array(buf: BufHandle) -> *mut c_int;

    // UTF-8 functions from mbyte
    fn rs_utf_ptr2cells(p: *const c_char) -> c_int;
    fn rs_utfc_ptr2len(p: *const c_char) -> c_int;

    // schar functions from grid (rs_schar_from_char already declared above)
    fn rs_utfc_ptr2schar(p: *const c_char, firstc: *mut c_int) -> ScharT;

    // Tab padding from indent crate
    fn rs_tabstop_padding(col: c_int, ts: i64, vts: *const c_int) -> c_int;

    // VirtText iteration (from decoration.c)
    fn nvim_next_virt_text_chunk(
        vt: *mut c_void,
        pos: *mut usize,
        attr: *mut c_int,
    ) -> *const c_char;

    // DecorState accessors (from decoration.c)
    fn nvim_get_decor_state() -> *mut c_void;
    fn nvim_decor_state_get_row(state: *mut c_void) -> c_int;
    fn nvim_decor_state_get_eol_col(state: *mut c_void) -> c_int;
    fn nvim_decor_state_set_eol_col(state: *mut c_void, val: c_int);
    fn nvim_decor_state_get_current_end(state: *mut c_void) -> c_int;
    fn nvim_decor_state_get_active_range(state: *mut c_void, i: c_int) -> *mut c_void;
    fn nvim_decor_state_get_eol_right_width(state: *mut c_void, from_idx: c_int) -> c_int;

    // DecorRange accessors
    fn nvim_decor_range_get_start_row(range: *mut c_void) -> c_int;
    fn nvim_decor_range_get_draw_col(range: *mut c_void) -> c_int;
    fn nvim_decor_range_set_draw_col(range: *mut c_void, val: c_int);
    fn nvim_decor_range_get_kind(range: *mut c_void) -> c_int;
    fn nvim_decor_range_has_virt_pos(range: *mut c_void) -> bool;
    fn nvim_decor_range_get_virt_pos_kind(range: *mut c_void) -> c_int;
    fn nvim_decor_range_get_virt_text(range: *mut c_void) -> *mut c_void;
    fn nvim_decor_range_get_ui_ns_id(range: *mut c_void) -> u64;
    fn nvim_decor_range_get_ui_mark_id(range: *mut c_void) -> u32;

    // DecorVirtText accessors
    fn nvim_decor_virt_text_get_width(vt: *mut c_void) -> c_int;
    fn nvim_decor_virt_text_get_col(vt: *mut c_void) -> c_int;
    fn nvim_decor_virt_text_get_pos(vt: *mut c_void) -> c_int;
    fn nvim_decor_virt_text_get_hl_mode(vt: *mut c_void) -> c_int;
    fn nvim_decor_virt_text_get_flags(vt: *mut c_void) -> c_int;
    fn nvim_decor_virt_text_get_virt_text(vt: *mut c_void) -> *mut c_void;

    // win_extmark_arr push
    fn nvim_win_extmark_push(ns_id: u64, mark_id: u64, win_row: c_int, win_col: c_int);
}

/// Opaque handle to buffer (buf_T).
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct BufHandle(*mut c_void);

use std::ffi::c_char;

/// TAB character constant.
#[allow(clippy::cast_possible_wrap)]
const TAB: c_char = b'\t' as c_char;

/// Flag for insecure wrap option.
const K_OPT_FLAG_INSECURE: c_int = 0x04;

/// SCL_NUM constant for signcolumn='number'.
const SCL_NUM: c_int = -1;

/// HlMode constants (from decoration_defs.h).
const HL_MODE_UNKNOWN: c_int = 0;
const HL_MODE_REPLACE: c_int = 1;
const HL_MODE_COMBINE: c_int = 2;
const HL_MODE_BLEND: c_int = 3;

/// NUL character constant.
const NUL: c_char = 0;

/// Decor kind constants (from decoration_defs.h).
const K_DECOR_KIND_VIRT_TEXT: c_int = 2;
const K_DECOR_KIND_UI_WATCHED: c_int = 4;

/// VirtTextPos constants.
const K_VPOS_END_OF_LINE: c_int = 0;
const K_VPOS_END_OF_LINE_RIGHT_ALIGN: c_int = 1;
const K_VPOS_INLINE: c_int = 2;
const K_VPOS_OVERLAY: c_int = 3;
const K_VPOS_RIGHT_ALIGN: c_int = 4;
const K_VPOS_WIN_COL: c_int = 5;

/// VirtText flag for repeat linebreak.
const K_VT_REPEAT_LINEBREAK: c_int = 8;

// ============================================================================
// Implementation functions
// ============================================================================

/// Get the 'listchars' "extends" character to use for "wp", or 0 if it
/// shouldn't be used.
fn get_lcs_ext_impl(wp: WinHandle) -> ScharT {
    unsafe {
        // Line never continues beyond the right of the screen with 'wrap'.
        if nvim_win_get_p_wrap(wp) != 0 {
            return 0;
        }
        // If 'nowrap' was set from a modeline, forcibly use '>'.
        if nvim_win_get_wrap_flags(wp) & K_OPT_FLAG_INSECURE != 0 {
            return rs_schar_from_char(c_int::from(b'>'));
        }
        if nvim_win_get_p_list(wp) != 0 {
            nvim_win_get_lcs_ext(wp)
        } else {
            0
        }
    }
}

/// Compute the margins for 'cursorlineopt' "screenline".
#[allow(clippy::cast_possible_truncation)]
fn margin_columns_win_impl(wp: WinHandle) -> (c_int, c_int) {
    unsafe {
        let cur_col_off = rs_win_col_off(wp);
        let width1 = nvim_win_get_view_width(wp) - cur_col_off;
        let width2 = width1 + rs_win_col_off2(wp);
        let virtcol = nvim_win_get_virtcol(wp);

        let right_col = if virtcol >= width1 && width2 > 0 {
            width1 + ((virtcol - width1) / width2 + 1) as c_int * width2
        } else {
            width1
        };

        let left_col = if virtcol >= width1 && width2 > 0 {
            ((virtcol - width1) / width2) as c_int * width2 + width1
        } else {
            0
        };

        (left_col, right_col)
    }
}

/// Fill a fold column buffer with fold symbols.
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
fn compute_foldcolumn_symbols(
    wp: WinHandle,
    level: c_int,
    closed: bool,
    lnum: LinenrT,
    fi_lnum: LinenrT,
    fi_low_level: c_int,
    fdc: c_int,
) -> Vec<(ScharT, c_int)> {
    let first_level = (level - fdc - c_int::from(closed) + 1).max(1);
    let closedcol = fdc.min(level);

    let capacity = if fdc > 0 { fdc as usize } else { 0 };
    let mut result = Vec::with_capacity(capacity);

    unsafe {
        for i in 0..fdc {
            let symbol = if i >= level {
                rs_schar_from_char(c_int::from(b' '))
            } else if i == closedcol - 1 && closed {
                nvim_win_get_fcs_foldclosed(wp)
            } else if fi_lnum == lnum && first_level + i >= fi_low_level {
                nvim_win_get_fcs_foldopen(wp)
            } else if first_level == 1 {
                nvim_win_get_fcs_foldsep(wp)
            } else {
                let foldinner = nvim_win_get_fcs_foldinner(wp);
                if foldinner != 0 {
                    foldinner
                } else if first_level + i <= 9 {
                    rs_schar_from_char(c_int::from(b'0') + first_level + i)
                } else {
                    rs_schar_from_char(c_int::from(b'>'))
                }
            };

            let vcol = if i >= level {
                -1
            } else if i == closedcol - 1 && closed {
                -2
            } else {
                -3
            };

            result.push((symbol, vcol));
        }
    }

    result
}

/// Get the rightmost virtual column that needs to be drawn.
fn get_rightmost_vcol_impl(wp: WinHandle, color_cols: *const c_int) -> c_int {
    let mut ret = 0;

    unsafe {
        if nvim_win_get_p_cuc(wp) != 0 {
            ret = nvim_win_get_virtcol(wp);
        }

        if !color_cols.is_null() {
            let mut i = 0;
            loop {
                let col = *color_cols.add(i);
                if col < 0 {
                    break;
                }
                if col > ret {
                    ret = col;
                }
                i += 1;
            }
        }
    }

    ret
}

/// Return true if CursorLineSign/CursorLineFold highlight is to be used.
fn use_cursor_line_highlight_impl(wp: WinHandle, lnum: LinenrT) -> bool {
    unsafe {
        nvim_win_get_p_cul(wp) != 0
            && lnum == nvim_win_get_cursorline(wp)
            && (nvim_win_get_p_culopt_flags(wp) & K_OPT_CULOPT_FLAG_NUMBER) != 0
    }
}

/// Fill cells with a character (draw_col_fill).
fn draw_col_fill_impl(wlv: WlvHandle, fillchar: ScharT, width: c_int, attr: c_int) {
    unsafe {
        let linebuf_char = nvim_get_linebuf_char();
        let linebuf_attr = nvim_get_linebuf_attr();
        let mut off = nvim_wlv_get_off(wlv);

        for _ in 0..width {
            *linebuf_char.add(off as usize) = fillchar;
            *linebuf_attr.add(off as usize) = attr;
            off += 1;
        }

        nvim_wlv_set_off(wlv, off);
    }
}

/// Return true if CursorLineNr highlight is to be used for the number column.
fn use_cursor_line_nr_impl(wp: WinHandle, wlv: WlvHandle) -> bool {
    unsafe {
        let p_cul = nvim_win_get_p_cul(wp) != 0;
        let lnum = nvim_wlv_get_lnum(wlv);
        let cursorline = nvim_win_get_cursorline(wp);
        let culopt_flags = nvim_win_get_p_culopt_flags(wp);
        let row = nvim_wlv_get_row(wlv);
        let startrow = nvim_wlv_get_startrow(wlv);
        let filler_lines = nvim_wlv_get_filler_lines(wlv);

        p_cul
            && lnum == cursorline
            && (culopt_flags & K_OPT_CULOPT_FLAG_NUMBER) != 0
            && (row == startrow + filler_lines
                || (row > startrow + filler_lines && (culopt_flags & K_OPT_CULOPT_FLAG_LINE) != 0))
    }
}

/// Return line number attribute, combining the appropriate LineNr* highlight
/// with the highest priority sign numhl highlight, if any.
fn get_line_number_attr_impl(wp: WinHandle, wlv: WlvHandle) -> c_int {
    unsafe {
        let mut numhl_attr = nvim_wlv_get_sign_num_attr(wlv);

        // Get previous sign numhl for virt_lines belonging to the previous line.
        let n_virt_lines = nvim_wlv_get_n_virt_lines(wlv);
        let filler_todo = nvim_wlv_get_filler_todo(wlv);
        let n_virt_below = nvim_wlv_get_n_virt_below(wlv);

        if (n_virt_lines - filler_todo) < n_virt_below {
            let mut prev = nvim_wlv_get_prev_num_attr(wlv);
            if prev == -1 {
                let lnum = nvim_wlv_get_lnum(wlv);
                let buf = nvim_win_get_buffer(wp);
                decor_redraw_signs(
                    wp,
                    buf,
                    lnum - 2,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut prev,
                );
                if prev > 0 {
                    prev = syn_id2attr(prev);
                }
                nvim_wlv_set_prev_num_attr(wlv, prev);
            }
            numhl_attr = prev;
        }

        if use_cursor_line_nr_impl(wp, wlv) {
            return hl_combine_attr(nvim_win_hl_attr(wp, HLF_CLN), numhl_attr);
        }

        if nvim_win_get_p_rnu(wp) != 0 {
            let lnum = nvim_wlv_get_lnum(wlv);
            let cursor_lnum = nvim_win_get_cursor_lnum(wp);
            if lnum < cursor_lnum {
                return hl_combine_attr(nvim_win_hl_attr(wp, HLF_LNA), numhl_attr);
            }
            if lnum > cursor_lnum {
                return hl_combine_attr(nvim_win_hl_attr(wp, HLF_LNB), numhl_attr);
            }
        }

        hl_combine_attr(nvim_win_hl_attr(wp, HLF_N), numhl_attr)
    }
}

/// Fill fold column directly to linebuf or output buffers.
#[allow(clippy::cast_sign_loss)]
fn fill_foldcolumn_impl(
    wp: WinHandle,
    foldinfo: FoldInfo,
    lnum: LinenrT,
    attr: c_int,
    fdc: c_int,
    wlv_off: *mut c_int,
    out_vcol: *mut ColnrT,
    out_buffer: *mut ScharT,
) {
    let closed = foldinfo.fi_level != 0 && foldinfo.fi_lines > 0;
    let symbols = compute_foldcolumn_symbols(
        wp,
        foldinfo.fi_level,
        closed,
        lnum,
        foldinfo.fi_lnum,
        foldinfo.fi_low_level,
        fdc,
    );

    unsafe {
        if !out_buffer.is_null() {
            // Fill output buffers (for statuscolumn)
            for (i, (symbol, vcol)) in symbols.into_iter().enumerate() {
                *out_vcol.add(i) = vcol;
                *out_buffer.add(i) = symbol;
            }
        } else {
            // Write directly to linebuf
            let linebuf_char = nvim_get_linebuf_char();
            let linebuf_attr = nvim_get_linebuf_attr();
            let linebuf_vcol = nvim_get_linebuf_vcol();
            let mut off = *wlv_off;

            for (symbol, vcol) in symbols {
                *linebuf_vcol.add(off as usize) = vcol;
                *linebuf_attr.add(off as usize) = attr;
                *linebuf_char.add(off as usize) = symbol;
                off += 1;
            }

            *wlv_off = off;
        }
    }
}

/// Setup for drawing the 'foldcolumn', if there is one.
fn draw_foldcolumn_impl(wp: WinHandle, wlv: WlvHandle) {
    unsafe {
        // compute_foldcolumn is rs_compute_foldcolumn in C
        extern "C" {
            fn rs_compute_foldcolumn(wp: WinHandle, col: c_int) -> c_int;
        }

        let fdc = rs_compute_foldcolumn(wp, 0);
        if fdc > 0 {
            let lnum = nvim_wlv_get_lnum(wlv);
            let foldinfo = nvim_wlv_get_foldinfo(wlv);
            let hlf = if use_cursor_line_highlight_impl(wp, lnum) {
                HLF_CLF
            } else {
                HLF_FC
            };
            let attr = nvim_win_hl_attr(wp, hlf);
            let mut off = nvim_wlv_get_off(wlv);
            fill_foldcolumn_impl(
                wp,
                foldinfo,
                lnum,
                attr,
                fdc,
                &mut off,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            nvim_wlv_set_off(wlv, off);
        }
    }
}

/// Draw a sign in the sign or number column.
fn draw_sign_impl(nrcol: bool, wp: WinHandle, wlv: WlvHandle, sign_idx: c_int) {
    unsafe {
        let lnum = nvim_wlv_get_lnum(wlv);
        let sattr_text0 = nvim_wlv_get_sattr_text(wlv, sign_idx, 0);
        let scl_attr = nvim_win_hl_attr(
            wp,
            if use_cursor_line_highlight_impl(wp, lnum) {
                HLF_CLS
            } else {
                HLF_SC
            },
        );

        let row = nvim_wlv_get_row(wlv);
        let startrow = nvim_wlv_get_startrow(wlv);
        let filler_lines = nvim_wlv_get_filler_lines(wlv);
        let filler_todo = nvim_wlv_get_filler_todo(wlv);

        if sattr_text0 != 0 && row == startrow + filler_lines && filler_todo <= 0 {
            let fill = if nrcol {
                rs_number_width(wp) + 1
            } else {
                SIGN_WIDTH
            };

            let sign_cul_attr = nvim_wlv_get_sign_cul_attr(wlv);
            let hl_id = nvim_wlv_get_sattr_hl_id(wlv, sign_idx);
            let mut attr = if sign_cul_attr != 0 {
                sign_cul_attr
            } else if hl_id != 0 {
                syn_id2attr(hl_id)
            } else {
                0
            };
            attr = hl_combine_attr(scl_attr, attr);

            draw_col_fill_impl(wlv, rs_schar_from_char(c_int::from(b' ')), fill, attr);

            let off = nvim_wlv_get_off(wlv);
            let sign_pos = off - SIGN_WIDTH - c_int::from(nrcol);
            debug_assert!(sign_pos >= 0);

            let linebuf_char = nvim_get_linebuf_char();
            *linebuf_char.add(sign_pos as usize) = sattr_text0;
            *linebuf_char.add((sign_pos + 1) as usize) = nvim_wlv_get_sattr_text(wlv, sign_idx, 1);
        } else {
            debug_assert!(!nrcol); // handled in draw_lnum_col()
            draw_col_fill_impl(
                wlv,
                rs_schar_from_char(c_int::from(b' ')),
                SIGN_WIDTH,
                scl_attr,
            );
        }
    }
}

/// Display the absolute or relative line number.
fn draw_lnum_col_impl(wp: WinHandle, wlv: WlvHandle) {
    unsafe {
        extern "C" {
            fn vim_strchr(s: *const i8, c: c_int) -> *const i8;
            fn nvim_get_p_cpo() -> *const i8;
            fn get_cursor_rel_lnum(wp: WinHandle, lnum: LinenrT) -> LinenrT;
        }

        const CPO_NUMCOL: c_int = b'n' as c_int;

        let p_cpo = nvim_get_p_cpo();
        let has_cpo_n = !vim_strchr(p_cpo, CPO_NUMCOL).is_null();
        let p_nu = nvim_win_get_p_nu(wp) != 0;
        let p_rnu = nvim_win_get_p_rnu(wp) != 0;

        let row = nvim_wlv_get_row(wlv);
        let startrow = nvim_wlv_get_startrow(wlv);
        let filler_lines = nvim_wlv_get_filler_lines(wlv);
        let lnum = nvim_wlv_get_lnum(wlv);
        let p_bri = nvim_win_get_p_bri(wp) != 0;
        let skipcol = nvim_win_get_skipcol(wp);
        let topline = nvim_win_get_topline(wp);

        if (p_nu || p_rnu)
            && (row == startrow + filler_lines || !has_cpo_n)
            && !((has_cpo_n && !p_bri) && skipcol > 0 && lnum == topline)
        {
            let minscwidth = nvim_win_get_minscwidth(wp);
            let sattr_text0 = nvim_wlv_get_sattr_text(wlv, 0, 0);
            let filler_todo = nvim_wlv_get_filler_todo(wlv);

            if minscwidth == SCL_NUM
                && sattr_text0 != 0
                && row == startrow + filler_lines
                && filler_todo <= 0
            {
                draw_sign_impl(true, wp, wlv, 0);
            } else {
                let width = rs_number_width(wp) + 1;
                let attr = get_line_number_attr_impl(wp, wlv);

                if row == startrow + filler_lines && (skipcol == 0 || row > 0 || (p_nu && p_rnu)) {
                    // Format the line number
                    let num_width = rs_number_width(wp);
                    let (num, left_align) = if p_nu && !p_rnu {
                        (lnum, false)
                    } else {
                        let rel = get_cursor_rel_lnum(wp, lnum).abs();
                        if rel == 0 && p_nu && p_rnu {
                            (lnum, true)
                        } else {
                            (rel, false)
                        }
                    };

                    // Format number into buffer
                    let mut buf = [0u8; 32];
                    let s = if left_align {
                        format!("{:<width$} ", num, width = num_width as usize)
                    } else {
                        format!("{:>width$} ", num, width = num_width as usize)
                    };
                    let bytes = s.as_bytes();
                    let len = bytes.len().min(31);
                    buf[..len].copy_from_slice(&bytes[..len]);

                    // Replace leading spaces with '-' if skipcol > 0 && startrow == 0
                    if skipcol > 0 && startrow == 0 {
                        for b in buf.iter_mut() {
                            if *b == b' ' {
                                *b = b'-';
                            } else {
                                break;
                            }
                        }
                    }

                    // TODO: Handle w_p_rl (right-to-left) mode

                    // Draw the line number using draw_col_buf logic
                    let linebuf_char = nvim_get_linebuf_char();
                    let linebuf_attr = nvim_get_linebuf_attr();
                    let mut off = nvim_wlv_get_off(wlv);

                    for i in 0..width {
                        let c = if (i as usize) < len {
                            buf[i as usize]
                        } else {
                            b' '
                        };
                        *linebuf_char.add(off as usize) = rs_schar_from_char(c_int::from(c));
                        *linebuf_attr.add(off as usize) = attr;
                        off += 1;
                    }

                    nvim_wlv_set_off(wlv, off);
                } else {
                    draw_col_fill_impl(wlv, rs_schar_from_char(c_int::from(b' ')), width, attr);
                }
            }
        }
    }
}

// New WLV accessor functions
extern "C" {
    fn nvim_wlv_get_color_cols(wlv: WlvHandle) -> *mut c_int;
    fn nvim_wlv_set_color_cols(wlv: WlvHandle, val: *mut c_int);
    fn nvim_wlv_get_vcol(wlv: WlvHandle) -> ColnrT;
    fn nvim_wlv_set_vcol(wlv: WlvHandle, val: ColnrT);
    fn nvim_wlv_get_vcol_off_co(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_get_col(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_set_col(wlv: WlvHandle, val: c_int);
    fn nvim_wlv_get_boguscols(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_set_boguscols(wlv: WlvHandle, val: c_int);
    fn nvim_wlv_get_old_boguscols(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_set_old_boguscols(wlv: WlvHandle, val: c_int);
    fn nvim_wlv_get_cul_attr(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_set_cul_attr(wlv: WlvHandle, val: c_int);
    fn nvim_wlv_get_line_attr(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_set_line_attr(wlv: WlvHandle, val: c_int);
    fn nvim_wlv_get_line_attr_lowprio(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_set_line_attr_lowprio(wlv: WlvHandle, val: c_int);

    // Additional wlv accessors for win_line_start and fix_for_boguscols
    fn nvim_wlv_get_n_extra(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_set_n_extra(wlv: WlvHandle, val: c_int);
    fn nvim_wlv_set_vcol_off_co(wlv: WlvHandle, val: c_int);
    fn nvim_wlv_get_need_lbr(wlv: WlvHandle) -> bool;
    fn nvim_wlv_set_need_lbr(wlv: WlvHandle, val: bool);

    // HLF constants
    fn nvim_get_hlf_mc() -> c_int;
    fn nvim_get_hlf_cul() -> c_int;

    // Buffer handle for win
    fn nvim_win_get_w_buffer(wp: WinHandle) -> BufHandle;

    // State and quickfix functions for apply_cursorline_highlight
    fn nvim_get_state() -> c_int;
    fn rs_bt_quickfix(buf: BufHandle) -> bool;
    fn nvim_qf_current_entry(wp: WinHandle) -> LinenrT;

    // Diff highlight accessor
    fn nvim_wlv_get_diff_hlf(wlv: WlvHandle) -> c_int;

    // Highlight functions for set_line_attr_for_diff
    fn rs_hl_get_underline() -> c_int;

    // handle_breakindent accessors
    fn nvim_win_get_briopt_sbr(wp: WinHandle) -> bool;
    fn nvim_get_breakindent_win_lnum(wp: WinHandle, lnum: LinenrT) -> c_int;

    // fromcol/tocol accessors
    fn nvim_wlv_get_fromcol(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_set_fromcol(wlv: WlvHandle, val: c_int);
    fn nvim_wlv_get_tocol(wlv: WlvHandle) -> c_int;
    fn nvim_wlv_set_tocol(wlv: WlvHandle, val: c_int);

    // need_showbreak accessors
    fn nvim_wlv_get_need_showbreak(wlv: WlvHandle) -> bool;
    fn nvim_wlv_set_need_showbreak(wlv: WlvHandle, val: bool);

    // handle_showbreak_and_filler accessors (not yet declared above)
    fn nvim_wlv_get_vcol_sbr(wlv: WlvHandle) -> ColnrT;
    fn nvim_wlv_set_vcol_sbr(wlv: WlvHandle, val: ColnrT);
    fn nvim_win_get_fcs_diff(wp: WinHandle) -> ScharT;
    fn get_showbreak_value(wp: WinHandle) -> *const c_char;

    // has_more_inline_virt accessors
    fn nvim_wlv_get_virt_inline_i(wlv: WlvHandle) -> usize;
    fn nvim_wlv_get_virt_inline_size(wlv: WlvHandle) -> usize;
    fn nvim_decor_state_get_future_begin(state: *mut c_void) -> c_int;
    fn nvim_decor_state_get_ranges_count(state: *mut c_void) -> c_int;
    fn nvim_decor_state_get_range_by_idx(state: *mut c_void, idx: c_int) -> *mut c_void;
    fn nvim_decor_range_get_start_col(range: *mut c_void) -> c_int;
    // nvim_decor_virt_text_get_width already declared above
}

/// Advance wlv->color_cols past the current vcol.
fn advance_color_col_impl(wlv: WlvHandle, vcol: c_int) {
    unsafe {
        let mut color_cols = nvim_wlv_get_color_cols(wlv);
        if color_cols.is_null() {
            return;
        }

        while *color_cols >= 0 && vcol > *color_cols {
            color_cols = color_cols.add(1);
        }

        if *color_cols < 0 {
            nvim_wlv_set_color_cols(wlv, std::ptr::null_mut());
        } else {
            nvim_wlv_set_color_cols(wlv, color_cols);
        }
    }
}

/// Put a character in the screen buffer for line drawing.
///
/// This implements the C `line_putchar` function. Returns the number of cells
/// used, and advances `*pp` past the character.
///
/// # Safety
/// - `pp` must be a valid pointer to a pointer to a NUL-terminated UTF-8 string
/// - `dest` must be a valid pointer to at least `maxcells` schar_T values
/// - `dest[0]` must not be 0 (caller ensures not overwriting right half of double-width)
unsafe fn line_putchar_impl(
    buf: BufHandle,
    pp: *mut *const c_char,
    dest: *mut ScharT,
    maxcells: c_int,
    vcol: c_int,
) -> c_int {
    // Caller should handle overwriting the right half of a double-width char.
    debug_assert!(*dest != 0, "dest[0] must not be 0");

    let p = *pp;
    let mut cells = rs_utf_ptr2cells(p);
    let c_len = rs_utfc_ptr2len(p);

    debug_assert!(maxcells > 0, "maxcells must be > 0");
    if cells > maxcells {
        *dest = rs_schar_from_char(c_int::from(b' '));
        return 1;
    }

    if *p == TAB {
        let ts = nvim_buf_get_p_ts(buf);
        let vts = nvim_buf_get_p_vts_array(buf);
        cells = rs_tabstop_padding(vcol, ts, vts);
        if cells > maxcells {
            cells = maxcells;
        }
    }

    // When overwriting the left half of a double-width char, clear the right half.
    if cells < maxcells && *dest.add(cells as usize) == 0 {
        *dest.add(cells as usize) = rs_schar_from_char(c_int::from(b' '));
    }

    if *p == TAB {
        for c in 0..cells {
            *dest.add(c as usize) = rs_schar_from_char(c_int::from(b' '));
        }
    } else {
        let mut u8c: c_int = 0;
        *dest = rs_utfc_ptr2schar(p, &mut u8c);
        if cells > 1 {
            *dest.add(1) = 0;
        }
    }

    *pp = p.add(c_len as usize);
    cells
}

// ============================================================================
// FFI exports
// ============================================================================

/// Get the 'listchars' "extends" character.
#[no_mangle]
pub extern "C" fn rs_get_lcs_ext(wp: WinHandle) -> ScharT {
    get_lcs_ext_impl(wp)
}

/// Get the rightmost virtual column that needs drawing.
#[no_mangle]
pub extern "C" fn rs_get_rightmost_vcol(wp: WinHandle, color_cols: *const c_int) -> c_int {
    get_rightmost_vcol_impl(wp, color_cols)
}

/// Compute cursorline margins.
#[no_mangle]
pub unsafe extern "C" fn rs_margin_columns_win(
    wp: WinHandle,
    left_col: *mut c_int,
    right_col: *mut c_int,
) {
    let (left, right) = margin_columns_win_impl(wp);
    if !left_col.is_null() {
        *left_col = left;
    }
    if !right_col.is_null() {
        *right_col = right;
    }
}

/// Fill fold column output buffer with symbols (legacy API).
#[no_mangle]
pub unsafe extern "C" fn rs_fill_foldcolumn_buffer(
    wp: WinHandle,
    foldinfo: FoldInfo,
    lnum: LinenrT,
    fdc: c_int,
    out_buffer: *mut ScharT,
    out_vcol: *mut ColnrT,
) {
    if out_buffer.is_null() || out_vcol.is_null() || fdc <= 0 {
        return;
    }

    let closed = foldinfo.fi_level != 0 && foldinfo.fi_lines > 0;
    let symbols = compute_foldcolumn_symbols(
        wp,
        foldinfo.fi_level,
        closed,
        lnum,
        foldinfo.fi_lnum,
        foldinfo.fi_low_level,
        fdc,
    );

    for (i, (symbol, vcol)) in symbols.into_iter().enumerate() {
        *out_buffer.add(i) = symbol;
        *out_vcol.add(i) = vcol;
    }
}

/// Check if cursor line highlight should be used.
#[no_mangle]
pub extern "C" fn rs_use_cursor_line_highlight(wp: WinHandle, lnum: LinenrT) -> bool {
    use_cursor_line_highlight_impl(wp, lnum)
}

/// Apply cursorline highlight to the line attributes.
///
/// We make a compromise here (#7383):
/// - low-priority CursorLine if fg is not set
/// - high-priority ("same as Vim" priority) CursorLine if fg is set
unsafe fn apply_cursorline_highlight_impl(wp: WinHandle, wlv: WlvHandle) {
    let hlf_cul = nvim_get_hlf_cul();
    let cul_attr = nvim_win_hl_attr(wp, hlf_cul);
    nvim_wlv_set_cul_attr(wlv, cul_attr);

    let ae: HlAttrs = rs_syn_attr2entry(cul_attr);

    if ae.rgb_fg_color == -1 && ae.cterm_fg_color == 0 {
        // Low-priority CursorLine when fg is not set
        nvim_wlv_set_line_attr_lowprio(wlv, cul_attr);
    } else {
        // High-priority CursorLine when fg is set
        let state = nvim_get_state();
        let buf = nvim_win_get_w_buffer(wp);
        let lnum = nvim_wlv_get_lnum(wlv);

        if (state & MODE_INSERT) == 0 && rs_bt_quickfix(buf) && nvim_qf_current_entry(wp) == lnum {
            // In quickfix window, combine with existing line_attr
            let line_attr = nvim_wlv_get_line_attr(wlv);
            nvim_wlv_set_line_attr(wlv, hl_combine_attr(cul_attr, line_attr));
        } else {
            nvim_wlv_set_line_attr(wlv, cul_attr);
        }
    }
}

/// Apply cursorline highlight (FFI export).
#[no_mangle]
pub unsafe extern "C" fn rs_apply_cursorline_highlight(wp: WinHandle, wlv: WlvHandle) {
    apply_cursorline_highlight_impl(wp, wlv);
}

/// Set line attribute for diff mode highlight.
///
/// Overlay CursorLine onto diff-mode highlight when applicable.
unsafe fn set_line_attr_for_diff_impl(wp: WinHandle, wlv: WlvHandle) {
    let diff_hlf = nvim_wlv_get_diff_hlf(wlv);
    let line_attr = nvim_win_hl_attr(wp, diff_hlf);
    nvim_wlv_set_line_attr(wlv, line_attr);

    let cul_attr = nvim_wlv_get_cul_attr(wlv);
    if cul_attr != 0 {
        let line_attr_lowprio = nvim_wlv_get_line_attr_lowprio(wlv);
        let new_attr = if line_attr_lowprio != 0 {
            // Low-priority CursorLine: combine with underline
            let combined = hl_combine_attr(cul_attr, line_attr);
            hl_combine_attr(combined, rs_hl_get_underline())
        } else {
            // High-priority CursorLine
            hl_combine_attr(line_attr, cul_attr)
        };
        nvim_wlv_set_line_attr(wlv, new_attr);
    }
}

/// Set line attribute for diff mode (FFI export).
#[no_mangle]
pub unsafe extern "C" fn rs_set_line_attr_for_diff(wp: WinHandle, wlv: WlvHandle) {
    set_line_attr_for_diff_impl(wp, wlv);
}

/// Handle breakindent: draw indent for wrapped text.
///
/// If need_showbreak is set, breakindent also applies.
unsafe fn handle_breakindent_impl(wp: WinHandle, wlv: WlvHandle) {
    let p_bri = nvim_win_get_p_bri(wp);
    let row = nvim_wlv_get_row(wlv);
    let startrow = nvim_wlv_get_startrow(wlv);
    let filler_lines = nvim_wlv_get_filler_lines(wlv);
    let need_showbreak = nvim_wlv_get_need_showbreak(wlv);

    if p_bri != 0 && (row > startrow + filler_lines || need_showbreak) {
        let mut attr = 0;
        let diff_hlf = nvim_wlv_get_diff_hlf(wlv);
        if diff_hlf != 0 {
            attr = nvim_win_hl_attr(wp, diff_hlf);
        }

        let lnum = nvim_wlv_get_lnum(wlv);
        let mut num = nvim_get_breakindent_win_lnum(wp, lnum);

        if row == startrow {
            num -= rs_win_col_off2(wp);
            let n_extra = nvim_wlv_get_n_extra(wlv);
            if n_extra < 0 {
                num = 0;
            }
        }

        let vcol_before = nvim_wlv_get_vcol(wlv);
        let hlf_mc = nvim_get_hlf_mc();

        let linebuf_char = nvim_get_linebuf_char();
        let linebuf_attr = nvim_get_linebuf_attr();
        let linebuf_vcol = nvim_get_linebuf_vcol();

        let space_schar = rs_schar_from_char(c_int::from(b' '));

        for _ in 0..num {
            let off = nvim_wlv_get_off(wlv);
            *linebuf_char.add(off as usize) = space_schar;

            let vcol = nvim_wlv_get_vcol(wlv);
            advance_color_col_impl(wlv, vcol);

            let mut myattr = attr;
            let color_cols = nvim_wlv_get_color_cols(wlv);
            if !color_cols.is_null() && vcol == *color_cols {
                myattr = hl_combine_attr(nvim_win_hl_attr(wp, hlf_mc), myattr);
            }

            *linebuf_attr.add(off as usize) = myattr;
            *linebuf_vcol.add(off as usize) = vcol;
            nvim_wlv_set_vcol(wlv, vcol + 1);
            nvim_wlv_set_off(wlv, off + 1);
        }

        // Correct start of highlighted area for 'breakindent'
        let fromcol = nvim_wlv_get_fromcol(wlv);
        let vcol = nvim_wlv_get_vcol(wlv);
        if fromcol >= vcol_before && fromcol < vcol {
            nvim_wlv_set_fromcol(wlv, vcol);
        }

        // Correct end of highlighted area for 'breakindent'
        let tocol = nvim_wlv_get_tocol(wlv);
        if tocol == vcol_before {
            nvim_wlv_set_tocol(wlv, vcol);
        }
    }

    // Handle need_showbreak clearing
    let skipcol = nvim_win_get_skipcol(wp);
    let startrow = nvim_wlv_get_startrow(wlv);
    let p_wrap = nvim_win_get_p_wrap(wp);
    let briopt_sbr = nvim_win_get_briopt_sbr(wp);

    if skipcol > 0 && startrow == 0 && p_wrap != 0 && briopt_sbr {
        nvim_wlv_set_need_showbreak(wlv, false);
    }
}

/// Handle breakindent (FFI export).
#[no_mangle]
pub unsafe extern "C" fn rs_handle_breakindent(wp: WinHandle, wlv: WlvHandle) {
    handle_breakindent_impl(wp, wlv);
}

/// Handle showbreak and filler lines.
///
/// Draws filler content for virtual lines and 'showbreak' string.
unsafe fn handle_showbreak_and_filler_impl(wp: WinHandle, wlv: WlvHandle) {
    let view_width = nvim_win_get_view_width(wp);
    let off = nvim_wlv_get_off(wlv);
    let remaining = view_width - off;

    let filler_todo = nvim_wlv_get_filler_todo(wlv);
    let filler_lines = nvim_wlv_get_filler_lines(wlv);
    let n_virt_lines = nvim_wlv_get_n_virt_lines(wlv);

    if filler_todo > filler_lines - n_virt_lines {
        // Fill with spaces for virtual lines
        let space_schar = rs_schar_from_char(c_int::from(b' '));
        draw_col_fill_impl(wlv, space_schar, remaining, 0);
    } else if filler_todo > 0 {
        // Draw "deleted" diff line(s)
        let diff_char = nvim_win_get_fcs_diff(wp);
        let attr = nvim_win_hl_attr(wp, HLF_DED);
        draw_col_fill_impl(wlv, diff_char, remaining, attr);
    }

    // Draw 'showbreak' at the start of each broken line
    let sbr = get_showbreak_value(wp);
    let need_showbreak = nvim_wlv_get_need_showbreak(wlv);

    if !sbr.is_null() && *sbr != 0 && need_showbreak {
        // Combine 'showbreak' with 'cursorline', prioritizing 'showbreak'
        let cul_attr = nvim_wlv_get_cul_attr(wlv);
        let at_attr = nvim_win_hl_attr(wp, HLF_AT);
        let attr = hl_combine_attr(cul_attr, at_attr);
        let vcol_before = nvim_wlv_get_vcol(wlv);

        // Get showbreak string length
        let mut sbr_len: usize = 0;
        let mut p = sbr;
        while *p != 0 {
            sbr_len += 1;
            p = p.add(1);
        }
        draw_col_buf_impl(wp, wlv, sbr, sbr_len, attr, std::ptr::null(), true);

        let vcol = nvim_wlv_get_vcol(wlv);
        nvim_wlv_set_vcol_sbr(wlv, vcol);

        // Correct start of highlighted area for 'showbreak'
        let fromcol = nvim_wlv_get_fromcol(wlv);
        if fromcol >= vcol_before && fromcol < vcol {
            nvim_wlv_set_fromcol(wlv, vcol);
        }

        // Correct end of highlighted area for 'showbreak'
        let tocol = nvim_wlv_get_tocol(wlv);
        if tocol == vcol_before {
            nvim_wlv_set_tocol(wlv, vcol);
        }
    }

    // Clear need_showbreak flag when appropriate
    let skipcol = nvim_win_get_skipcol(wp);
    let startrow = nvim_wlv_get_startrow(wlv);
    let p_wrap = nvim_win_get_p_wrap(wp);
    let briopt_sbr = nvim_win_get_briopt_sbr(wp);

    if skipcol == 0 || startrow > 0 || p_wrap == 0 || !briopt_sbr {
        nvim_wlv_set_need_showbreak(wlv, false);
    }
}

/// Handle showbreak and filler (FFI export).
#[no_mangle]
pub unsafe extern "C" fn rs_handle_showbreak_and_filler(wp: WinHandle, wlv: WlvHandle) {
    handle_showbreak_and_filler_impl(wp, wlv);
}

/// Check if there is more inline virtual text to be drawn.
///
/// Returns true if there are more inline virtual text chunks to draw at or after
/// the given column position `v`.
unsafe fn has_more_inline_virt_impl(wlv: WlvHandle, v: isize) -> bool {
    // Check if still inside current virt_inline
    let virt_inline_i = nvim_wlv_get_virt_inline_i(wlv);
    let virt_inline_size = nvim_wlv_get_virt_inline_size(wlv);
    if virt_inline_i < virt_inline_size {
        return true;
    }

    let state = nvim_get_decor_state();
    let count = nvim_decor_state_get_ranges_count(state);
    let cur_end = nvim_decor_state_get_current_end(state);
    let fut_beg = nvim_decor_state_get_future_begin(state);
    let state_row = nvim_decor_state_get_row(state);

    // Check both ranges: [0, cur_end) and [fut_beg, count)
    let beg_pos = [0, fut_beg];
    let end_pos = [cur_end, count];

    for pos_i in 0..2 {
        for i in beg_pos[pos_i]..end_pos[pos_i] {
            let range = nvim_decor_state_get_range_by_idx(state, i);
            if range.is_null() {
                continue;
            }

            let start_row = nvim_decor_range_get_start_row(range);
            let kind = nvim_decor_range_get_kind(range);
            let draw_col = nvim_decor_range_get_draw_col(range);
            let start_col = nvim_decor_range_get_start_col(range);

            if start_row != state_row || kind != K_DECOR_KIND_VIRT_TEXT {
                continue;
            }

            // Get virt text position and width
            let vt = nvim_decor_range_get_virt_text(range);
            if vt.is_null() {
                continue;
            }

            let vt_pos = nvim_decor_virt_text_get_pos(vt);
            let vt_width = nvim_decor_virt_text_get_width(vt);

            if vt_pos != K_VPOS_INLINE || vt_width == 0 {
                continue;
            }

            if draw_col >= -1 && (start_col as isize) >= v {
                return true;
            }
        }
    }
    false
}

/// Check for more inline virtual text (FFI export).
#[no_mangle]
pub unsafe extern "C" fn rs_has_more_inline_virt(wlv: WlvHandle, v: isize) -> bool {
    has_more_inline_virt_impl(wlv, v)
}

/// Fill cells with a character.
#[no_mangle]
pub extern "C" fn rs_draw_col_fill(wlv: WlvHandle, fillchar: ScharT, width: c_int, attr: c_int) {
    draw_col_fill_impl(wlv, fillchar, width, attr);
}

/// Fill fold column (full implementation with linebuf support).
#[no_mangle]
pub unsafe extern "C" fn rs_fill_foldcolumn(
    wp: WinHandle,
    foldinfo: FoldInfo,
    lnum: LinenrT,
    attr: c_int,
    fdc: c_int,
    wlv_off: *mut c_int,
    out_vcol: *mut ColnrT,
    out_buffer: *mut ScharT,
) {
    fill_foldcolumn_impl(wp, foldinfo, lnum, attr, fdc, wlv_off, out_vcol, out_buffer);
}

/// Draw fold column.
#[no_mangle]
pub extern "C" fn rs_draw_foldcolumn(wp: WinHandle, wlv: WlvHandle) {
    draw_foldcolumn_impl(wp, wlv);
}

/// Check if cursor line number highlight should be used.
#[no_mangle]
pub extern "C" fn rs_use_cursor_line_nr(wp: WinHandle, wlv: WlvHandle) -> bool {
    use_cursor_line_nr_impl(wp, wlv)
}

/// Get line number attribute.
#[no_mangle]
pub extern "C" fn rs_get_line_number_attr(wp: WinHandle, wlv: WlvHandle) -> c_int {
    get_line_number_attr_impl(wp, wlv)
}

/// Draw sign in sign or number column.
#[no_mangle]
pub extern "C" fn rs_draw_sign(nrcol: bool, wp: WinHandle, wlv: WlvHandle, sign_idx: c_int) {
    draw_sign_impl(nrcol, wp, wlv, sign_idx);
}

/// Draw line number column.
#[no_mangle]
pub extern "C" fn rs_draw_lnum_col(wp: WinHandle, wlv: WlvHandle) {
    draw_lnum_col_impl(wp, wlv);
}

/// Advance color_cols past current vcol.
#[no_mangle]
pub extern "C" fn rs_advance_color_col(wlv: WlvHandle, vcol: c_int) {
    advance_color_col_impl(wlv, vcol);
}

/// Put a character in the screen buffer for line drawing.
///
/// # Safety
/// - `pp` must be a valid pointer to a pointer to a NUL-terminated UTF-8 string
/// - `dest` must be a valid pointer to at least `maxcells` schar_T values
/// - `dest[0]` must not be 0 (caller ensures not overwriting right half of double-width)
#[no_mangle]
pub unsafe extern "C" fn rs_line_putchar(
    buf: BufHandle,
    pp: *mut *const c_char,
    dest: *mut ScharT,
    maxcells: c_int,
    vcol: c_int,
) -> c_int {
    line_putchar_impl(buf, pp, dest, maxcells, vcol)
}

// ============================================================================
// Virtual text rendering
// ============================================================================

/// Draw a virtual text item.
///
/// Renders virtual text chunks to the line buffer, handling highlight modes
/// (replace, combine, blend) and character/cell alignment.
///
/// # Arguments
/// * `buf` - Buffer handle for tab expansion
/// * `col` - Starting column in linebuf
/// * `vt` - VirtText pointer (kvec_t of VirtTextChunk)
/// * `hl_mode` - Highlight mode (0=unknown, 1=replace, 2=combine, 3=blend)
/// * `max_col` - Maximum column (window width)
/// * `vcol` - Virtual column for tab expansion
/// * `skip_cells` - Number of cells to skip (for partial rendering)
///
/// # Returns
/// The column after the last rendered character.
///
/// # Safety
/// - `vt` must be a valid pointer to VirtText (kvec_t)
/// - linebuf_char and linebuf_attr must be valid global arrays
#[allow(clippy::too_many_lines)]
unsafe fn draw_virt_text_item_impl(
    buf: BufHandle,
    mut col: c_int,
    vt: *mut c_void,
    hl_mode: c_int,
    max_col: c_int,
    mut vcol: c_int,
    mut skip_cells: c_int,
) -> c_int {
    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();

    let mut virt_str: *const c_char = c"".as_ptr();
    let mut virt_attr: c_int = 0;
    let mut virt_pos: usize = 0;

    while col < max_col {
        // Get next chunk if current string is exhausted
        if skip_cells >= 0 && *virt_str == NUL {
            let next = nvim_next_virt_text_chunk(vt, &mut virt_pos, &mut virt_attr);
            if next.is_null() {
                break;
            }
            virt_str = next;
        }

        // Skip cells in the text
        while skip_cells > 0 && *virt_str != NUL {
            let c_len = rs_utfc_ptr2len(virt_str);
            let cells = if *virt_str == TAB {
                let ts = nvim_buf_get_p_ts(buf);
                let vts = nvim_buf_get_p_vts_array(buf);
                rs_tabstop_padding(vcol, ts, vts)
            } else {
                rs_utf_ptr2cells(virt_str)
            };
            skip_cells -= cells;
            vcol += cells;
            virt_str = virt_str.add(c_len as usize);
        }

        // If a double-width char or TAB doesn't fit, pad with spaces
        let draw_str: *const c_char = if skip_cells < 0 {
            c" ".as_ptr()
        } else {
            virt_str
        };

        if *draw_str == NUL {
            continue;
        }

        debug_assert!(skip_cells <= 0);

        // Calculate attribute based on highlight mode
        let mut through = false;
        #[allow(clippy::cast_possible_wrap)]
        let space_char: c_char = b' ' as c_char;
        let attr = match hl_mode {
            HL_MODE_COMBINE => hl_combine_attr(*linebuf_attr.add(col as usize), virt_attr),
            HL_MODE_BLEND => {
                through = *draw_str == space_char;
                hl_blend_attrs(*linebuf_attr.add(col as usize), virt_attr, &mut through)
            }
            _ => virt_attr, // HL_MODE_REPLACE or HL_MODE_UNKNOWN
        };

        // Prepare dummy buffer for "through" mode
        let mut dummy: [ScharT; 2] = [
            rs_schar_from_char(c_int::from(b' ')),
            rs_schar_from_char(c_int::from(b' ')),
        ];

        let maxcells = max_col - col;

        // When overwriting the right half of a double-width char, clear the left half
        if !through && *linebuf_char.add(col as usize) == 0 {
            debug_assert!(col > 0);
            *linebuf_char.add((col - 1) as usize) = rs_schar_from_char(c_int::from(b' '));
            // Clear the right half as well for the assertion in line_putchar
            *linebuf_char.add(col as usize) = rs_schar_from_char(c_int::from(b' '));
        }

        // Draw the character
        let dest = if through {
            dummy.as_mut_ptr()
        } else {
            linebuf_char.add(col as usize)
        };

        let mut draw_str_ptr = draw_str;
        let cells = line_putchar_impl(buf, &mut draw_str_ptr, dest, maxcells, vcol);

        // Update attributes for all cells
        for _ in 0..cells {
            *linebuf_attr.add(col as usize) = attr;
            col += 1;
        }

        // Update state
        if skip_cells < 0 {
            skip_cells += 1;
        } else {
            vcol += cells;
            virt_str = draw_str_ptr;
        }
    }

    col
}

/// Draw a virtual text item (FFI export).
#[no_mangle]
pub unsafe extern "C" fn rs_draw_virt_text_item(
    buf: BufHandle,
    col: c_int,
    vt: *mut c_void,
    hl_mode: c_int,
    max_col: c_int,
    vcol: c_int,
    skip_cells: c_int,
) -> c_int {
    draw_virt_text_item_impl(buf, col, vt, hl_mode, max_col, vcol, skip_cells)
}

// ============================================================================
// draw_virt_text - virtual text positioning and rendering
// ============================================================================

/// Draw virtual text items for the current line.
///
/// This function positions and renders virtual text with various alignment modes:
/// - EndOfLine: right after end of line
/// - EndOfLineRightAlign: right-aligned at end of window
/// - RightAlign: right-aligned from right edge
/// - WinCol: at specific window column
///
/// # Arguments
/// * `wp` - Window handle
/// * `buf` - Buffer handle
/// * `col_off` - Column offset
/// * `end_col` - Pointer to track end column (updated)
/// * `win_row` - Window row for UI watched marks
///
/// # Safety
/// - `end_col` must be a valid pointer
/// - wp and buf must be valid handles
#[allow(clippy::too_many_lines)]
unsafe fn draw_virt_text_impl(
    wp: WinHandle,
    buf: BufHandle,
    col_off: c_int,
    end_col: *mut c_int,
    win_row: c_int,
) {
    let state = nvim_get_decor_state();
    let max_col = nvim_win_get_view_width(wp);
    let mut right_pos = max_col;
    let eol_col = nvim_decor_state_get_eol_col(state);
    let do_eol = eol_col > -1;
    let row = nvim_decor_state_get_row(state);
    let current_end = nvim_decor_state_get_current_end(state);

    let mut total_width_eol_right = 0;

    for i in 0..current_end {
        let item = nvim_decor_state_get_active_range(state, i);
        if item.is_null() {
            continue;
        }

        // Skip if not on current row or not a virtual position
        if nvim_decor_range_get_start_row(item) != row || !nvim_decor_range_has_virt_pos(item) {
            continue;
        }

        let kind = nvim_decor_range_get_kind(item);
        let vt = if kind == K_DECOR_KIND_VIRT_TEXT {
            nvim_decor_range_get_virt_text(item)
        } else {
            std::ptr::null_mut()
        };

        let draw_col = nvim_decor_range_get_draw_col(item);
        if nvim_decor_range_has_virt_pos(item) && draw_col == -1 {
            let mut updated = true;
            let pos = nvim_decor_range_get_virt_pos_kind(item);

            if do_eol && pos == K_VPOS_END_OF_LINE_RIGHT_ALIGN {
                let mut eol_offset = 0;
                if total_width_eol_right == 0 {
                    // Calculate total width of right-aligned EOL virtual text
                    total_width_eol_right = nvim_decor_state_get_eol_right_width(state, i);

                    let current_eol_col = nvim_decor_state_get_eol_col(state);
                    if total_width_eol_right <= right_pos - current_eol_col {
                        eol_offset = right_pos - total_width_eol_right - current_eol_col;
                    }
                }

                let new_draw_col = nvim_decor_state_get_eol_col(state) + eol_offset;
                nvim_decor_range_set_draw_col(item, new_draw_col);
            } else if pos == K_VPOS_RIGHT_ALIGN {
                if !vt.is_null() {
                    right_pos -= nvim_decor_virt_text_get_width(vt);
                }
                nvim_decor_range_set_draw_col(item, right_pos);
            } else if pos == K_VPOS_END_OF_LINE && do_eol {
                nvim_decor_range_set_draw_col(item, nvim_decor_state_get_eol_col(state));
            } else if pos == K_VPOS_WIN_COL {
                if !vt.is_null() {
                    let vt_col = nvim_decor_virt_text_get_col(vt);
                    let new_col = std::cmp::max(col_off + vt_col, 0);
                    nvim_decor_range_set_draw_col(item, new_col);
                }
            } else {
                updated = false;
            }

            if updated {
                let new_draw_col = nvim_decor_range_get_draw_col(item);
                if new_draw_col < 0 || new_draw_col >= max_col {
                    // Out of window, don't draw at all
                    nvim_decor_range_set_draw_col(item, c_int::MIN);
                }
            }
        }

        let draw_col = nvim_decor_range_get_draw_col(item);
        if draw_col < 0 {
            continue;
        }

        // Handle UIWatched marks
        if kind == K_DECOR_KIND_UI_WATCHED {
            let ns_id = nvim_decor_range_get_ui_ns_id(item);
            let mark_id = u64::from(nvim_decor_range_get_ui_mark_id(item));
            nvim_win_extmark_push(ns_id, mark_id, win_row, draw_col);
        }

        // Render virtual text
        if !vt.is_null() {
            let vcol = draw_col - col_off;
            let virt_text = nvim_decor_virt_text_get_virt_text(vt);
            let hl_mode = nvim_decor_virt_text_get_hl_mode(vt);

            let col = draw_virt_text_item_impl(buf, draw_col, virt_text, hl_mode, max_col, vcol, 0);

            let vt_pos = nvim_decor_virt_text_get_pos(vt);
            if do_eol && (vt_pos == K_VPOS_END_OF_LINE || vt_pos == K_VPOS_END_OF_LINE_RIGHT_ALIGN)
            {
                nvim_decor_state_set_eol_col(state, col + 1);
            }

            if !end_col.is_null() {
                *end_col = std::cmp::max(*end_col, col);
            }
        }

        // Deactivate unless it should repeat on linebreak
        let flags = if !vt.is_null() {
            nvim_decor_virt_text_get_flags(vt)
        } else {
            0
        };
        if vt.is_null() || (flags & K_VT_REPEAT_LINEBREAK) == 0 {
            nvim_decor_range_set_draw_col(item, c_int::MIN);
        }
    }
}

/// Draw virtual text (FFI export).
#[no_mangle]
pub unsafe extern "C" fn rs_draw_virt_text(
    wp: WinHandle,
    buf: BufHandle,
    col_off: c_int,
    end_col: *mut c_int,
    win_row: c_int,
) {
    draw_virt_text_impl(wp, buf, col_off, end_col, win_row);
}

// ============================================================================
// Line initialization functions
// ============================================================================

/// Initialize the line buffer for rendering.
///
/// Resets wlv->col, wlv->off, and wlv->need_lbr to initial values, and fills
/// the linebuf arrays with spaces/zeros.
///
/// # Safety
/// - `wp` must be a valid window handle
/// - `wlv` must be a valid winlinevars_T handle
unsafe fn win_line_start_impl(wp: WinHandle, wlv: WlvHandle) {
    nvim_wlv_set_col(wlv, 0);
    nvim_wlv_set_off(wlv, 0);
    nvim_wlv_set_need_lbr(wlv, false);

    let view_width = nvim_win_get_view_width(wp);
    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();
    let linebuf_vcol = nvim_get_linebuf_vcol();

    // schar_from_ascii(' ') - space character in native byte order
    let space_schar = rs_schar_from_char(c_int::from(b' '));

    for i in 0..view_width {
        *linebuf_char.add(i as usize) = space_schar;
        *linebuf_attr.add(i as usize) = 0;
        *linebuf_vcol.add(i as usize) = -1;
    }
}

/// FFI export for win_line_start.
#[no_mangle]
pub unsafe extern "C" fn rs_win_line_start(wp: WinHandle, wlv: WlvHandle) {
    win_line_start_impl(wp, wlv);
}

/// Fix up the linebuf for bogus columns.
///
/// This adjusts n_extra, vcol, vcol_off_co, col, boguscols, and old_boguscols
/// after handling bogus columns (extra columns for composing characters or
/// other special cases).
///
/// # Safety
/// - `wlv` must be a valid winlinevars_T handle
unsafe fn fix_for_boguscols_impl(wlv: WlvHandle) {
    let vcol_off_co = nvim_wlv_get_vcol_off_co(wlv);
    let boguscols = nvim_wlv_get_boguscols(wlv);

    // wlv->n_extra += wlv->vcol_off_co
    let n_extra = nvim_wlv_get_n_extra(wlv);
    nvim_wlv_set_n_extra(wlv, n_extra + vcol_off_co);

    // wlv->vcol -= wlv->vcol_off_co
    let vcol = nvim_wlv_get_vcol(wlv);
    nvim_wlv_set_vcol(wlv, vcol - vcol_off_co as ColnrT);

    // wlv->vcol_off_co = 0
    nvim_wlv_set_vcol_off_co(wlv, 0);

    // wlv->col -= wlv->boguscols
    let col = nvim_wlv_get_col(wlv);
    nvim_wlv_set_col(wlv, col - boguscols);

    // wlv->old_boguscols = wlv->boguscols
    nvim_wlv_set_old_boguscols(wlv, boguscols);

    // wlv->boguscols = 0
    nvim_wlv_set_boguscols(wlv, 0);
}

/// FFI export for fix_for_boguscols.
#[no_mangle]
pub unsafe extern "C" fn rs_fix_for_boguscols(wlv: WlvHandle) {
    fix_for_boguscols_impl(wlv);
}

// ============================================================================
// draw_col_buf - Draw text into the line buffer
// ============================================================================

/// Draw text into the line buffer, handling color columns.
///
/// This function draws text from `text` (with length `len`) into the line buffer,
/// advancing wlv->off and optionally wlv->vcol. It also handles colorcolumn highlighting.
///
/// # Arguments
/// * `wp` - Window handle
/// * `wlv` - winlinevars_T handle
/// * `text` - Pointer to UTF-8 text to draw
/// * `len` - Length of text in bytes
/// * `attr` - Highlight attribute to use
/// * `fold_vcol` - Optional pointer to vcol values for folded text (NULL if inc_vcol)
/// * `inc_vcol` - If true, increment vcol for each cell
///
/// # Safety
/// - All pointers must be valid
/// - text must point to len bytes of valid UTF-8
unsafe fn draw_col_buf_impl(
    wp: WinHandle,
    wlv: WlvHandle,
    text: *const c_char,
    len: usize,
    attr: c_int,
    fold_vcol: *const ColnrT,
    inc_vcol: bool,
) {
    let buf = nvim_win_get_w_buffer(wp);
    let view_width = nvim_win_get_view_width(wp);
    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();
    let linebuf_vcol = nvim_get_linebuf_vcol();
    let hlf_mc = nvim_get_hlf_mc();

    let mut ptr = text;
    let text_end = text.add(len);
    let mut fold_vcol_ptr = fold_vcol;

    while ptr < text_end && nvim_wlv_get_off(wlv) < view_width {
        let off = nvim_wlv_get_off(wlv);

        // Call line_putchar to render the character
        let cells = line_putchar_impl(
            buf,
            &mut ptr,
            linebuf_char.add(off as usize),
            view_width - off,
            off,
        );

        let mut myattr = attr;
        if inc_vcol {
            advance_color_col_impl(wlv, nvim_wlv_get_vcol(wlv));
            let color_cols = nvim_wlv_get_color_cols(wlv);
            if !color_cols.is_null() && nvim_wlv_get_vcol(wlv) == *color_cols {
                myattr = hl_combine_attr(nvim_win_hl_attr(wp, hlf_mc), myattr);
            }
        }

        // Fill in attr and vcol for each cell
        for _ in 0..cells {
            let current_off = nvim_wlv_get_off(wlv);
            *linebuf_attr.add(current_off as usize) = myattr;

            let vcol_val = if inc_vcol {
                let v = nvim_wlv_get_vcol(wlv);
                nvim_wlv_set_vcol(wlv, v + 1);
                v
            } else if !fold_vcol_ptr.is_null() {
                let v = *fold_vcol_ptr;
                fold_vcol_ptr = fold_vcol_ptr.add(1);
                v
            } else {
                -1
            };
            *linebuf_vcol.add(current_off as usize) = vcol_val;

            nvim_wlv_set_off(wlv, current_off + 1);
        }
    }
}

/// FFI export for draw_col_buf.
#[no_mangle]
pub unsafe extern "C" fn rs_draw_col_buf(
    wp: WinHandle,
    wlv: WlvHandle,
    text: *const c_char,
    len: usize,
    attr: c_int,
    fold_vcol: *const ColnrT,
    inc_vcol: bool,
) {
    draw_col_buf_impl(wp, wlv, text, len, attr, fold_vcol, inc_vcol);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foldinfo_layout() {
        assert!(std::mem::size_of::<FoldInfo>() > 0);
    }

    #[test]
    fn test_wlv_handle_layout() {
        assert_eq!(
            std::mem::size_of::<WlvHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
    }
}
