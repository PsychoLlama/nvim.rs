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
pub const HLF_N: c_int = 12; // LineNr
pub const HLF_LNA: c_int = 13; // LineNrAbove
pub const HLF_LNB: c_int = 14; // LineNrBelow
pub const HLF_CLN: c_int = 15; // CursorLineNr
pub const HLF_CLS: c_int = 16; // CursorLineSign
pub const HLF_CLF: c_int = 17; // CursorLineFold
pub const HLF_FC: c_int = 29; // FoldColumn
pub const HLF_SC: c_int = 35; // SignColumn

// Cursorlineopt flags (from option_vars.generated.h)
pub const K_OPT_CULOPT_FLAG_LINE: c_int = 0x01;
pub const K_OPT_CULOPT_FLAG_SCREENLINE: c_int = 0x02;
pub const K_OPT_CULOPT_FLAG_NUMBER: c_int = 0x04;

/// Sign width constant.
pub const SIGN_WIDTH: c_int = 2;

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
}

/// Flag for insecure wrap option.
const K_OPT_FLAG_INSECURE: c_int = 0x04;

/// SCL_NUM constant for signcolumn='number'.
const SCL_NUM: c_int = -1;

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
