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
type LinenrT = i32;

/// Column number type.
type ColnrT = i32;

/// A repr(C) kvec_t (generic, matches C kvec_t layout exactly).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KVec<T> {
    pub size: usize,
    pub capacity: usize,
    pub items: *mut T,
}

impl<T> KVec<T> {
    /// Create an empty KVec (matches VIRTTEXT_EMPTY).
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            size: 0,
            capacity: 0,
            items: std::ptr::null_mut(),
        }
    }
}

/// VirtTextChunk - a single virtual text chunk (char *text, int hl_id).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtTextChunkC {
    pub text: *mut c_char,
    pub hl_id: c_int,
}

/// SignTextAttrs - a sign text attribute (schar_T text[2], int hl_id).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SignTextAttrsC {
    pub text: [ScharT; 2],
    pub hl_id: c_int,
}

/// WinLineVars - the line drawing state struct, repr(C) matching winlinevars_T.
///
/// Field order and types must exactly match the C struct in drawline.c.
#[repr(C)]
pub struct WinLineVars {
    /// line number to be drawn (const in C)
    pub lnum: LinenrT,
    /// fold info for this line (const in C)
    pub foldinfo: FoldInfo,
    /// first row in the window to be drawn (const in C)
    pub startrow: c_int,
    /// row in the window, excl w_winrow
    pub row: c_int,
    /// virtual column, before wrapping
    pub vcol: ColnrT,
    /// visual column on screen, after wrapping
    pub col: c_int,
    /// nonexistent columns added to "col" to force wrapping
    pub boguscols: c_int,
    /// bogus boguscols
    pub old_boguscols: c_int,
    /// offset for concealed characters
    pub vcol_off_co: c_int,
    /// offset relative start of line
    pub off: c_int,
    /// set when 'cursorline' active
    pub cul_attr: c_int,
    /// attribute for the whole line
    pub line_attr: c_int,
    /// low-priority attribute for the line
    pub line_attr_lowprio: c_int,
    /// line number attribute (sign numhl)
    pub sign_num_attr: c_int,
    /// previous line's number attribute (sign numhl)
    pub prev_num_attr: c_int,
    /// cursorline sign attribute (sign culhl)
    pub sign_cul_attr: c_int,
    /// start of inverting
    pub fromcol: c_int,
    /// end of inverting
    pub tocol: c_int,
    /// virtual column after showbreak
    pub vcol_sbr: ColnrT,
    /// overlong line, skipping first x chars
    pub need_showbreak: bool,
    /// attributes for next character
    pub char_attr: c_int,
    /// number of extra bytes
    pub n_extra: c_int,
    /// chars with special attr
    pub n_attr: c_int,
    /// string of extra chars
    pub p_extra: *mut c_char,
    /// attributes for p_extra
    pub extra_attr: c_int,
    /// extra chars, all the same
    pub sc_extra: ScharT,
    /// final char, mandatory if set
    pub sc_final: ScharT,
    /// n_extra set for inline virtual text
    pub extra_for_extmark: bool,
    /// must be as large as transchar_charbuf[] in charset.c
    pub extra: [c_char; 11],
    /// type of diff highlighting (hlf_T as c_int)
    pub diff_hlf: c_int,
    /// nr of virtual lines
    pub n_virt_lines: c_int,
    /// nr of virtual lines belonging to previous line
    pub n_virt_below: c_int,
    /// nr of filler lines to be drawn
    pub filler_lines: c_int,
    /// nr of filler lines still to do + 1
    pub filler_todo: c_int,
    /// sign attributes for the sign column (SIGN_SHOW_MAX = 9)
    pub sattrs: [SignTextAttrsC; 9],
    /// do consider wrapping in linebreak mode only after non whitespace
    pub need_lbr: bool,
    /// virtual inline text (VirtText kvec)
    pub virt_inline: KVec<VirtTextChunkC>,
    /// current position in virt_inline
    pub virt_inline_i: usize,
    /// hl_mode for virt_inline (HlMode as c_int)
    pub virt_inline_hl_mode: c_int,
    /// reset extra attr flag
    pub reset_extra_attr: bool,
    /// nr of cells to skip for w_leftcol or w_skipcol or concealing
    pub skip_cells: c_int,
    /// nr of skipped cells for virtual text
    pub skipped_cells: c_int,
    /// if not NULL, highlight colorcolumn using according columns array
    pub color_cols: *mut c_int,
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
pub const HLF_DED: c_int = 32; // DiffDelete (deleted diff line)
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
    #[link_name = "win_col_off"]
    fn rs_win_col_off(wp: WinHandle) -> c_int;
    #[link_name = "win_col_off2"]
    fn rs_win_col_off2(wp: WinHandle) -> c_int;
    #[link_name = "number_width"]
    fn rs_number_width(wp: WinHandle) -> c_int;

    // Linebuf access
    fn nvim_get_linebuf_char() -> *mut ScharT;
    fn nvim_get_linebuf_attr() -> *mut c_int;
    fn nvim_get_linebuf_vcol() -> *mut ColnrT;

    // WLV accessor functions

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
    fn utf_ptr2cells(p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // schar functions from grid (rs_schar_from_char already declared above)
    fn rs_utfc_ptr2schar(p: *const c_char, firstc: *mut c_int) -> ScharT;

    // Tab padding from indent crate
    #[link_name = "tabstop_padding"]
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

    // Additional WLV accessors for handle_inline_virtual_text

    // Additional DecorRange accessors for handle_inline_virtual_text
    fn nvim_decor_range_get_start_col(range: *mut c_void) -> c_int;
    fn nvim_decor_init_draw_col(win_col: c_int, hidden: bool, item: *mut c_void);
    fn nvim_decor_range_get_virt_inline_data(range: *mut c_void) -> *mut c_void;
    fn nvim_decor_range_get_virt_inline_hl_mode(range: *mut c_void) -> c_int;

    // Multibyte functions from mbyte crate
    fn mb_charlen(s: *const c_char) -> c_int;
    fn mb_string2cells(s: *const c_char) -> usize;

    // wlv_put_linebuf accessors
    fn nvim_win_get_w_grid(wp: WinHandle) -> *mut c_void;
    fn nvim_win_get_lcs_prec(wp: WinHandle) -> u32;
    fn rs_linebuf_mirror(firstp: *mut c_int, lastp: *mut c_int, clearp: *mut c_int, width: c_int);
    fn rs_get_showbreak_value(wp: WinHandle) -> *const c_char;
    fn rs_grid_adjust(view: *mut c_void, row_off: *mut c_int, col_off: *mut c_int) -> *mut c_void;
    fn rs_grid_put_linebuf(
        grid: *mut c_void,
        row: c_int,
        coloff: c_int,
        col: c_int,
        endcol: c_int,
        clear_width: c_int,
        bg_attr: c_int,
        clear_attr: c_int,
        last_vcol: ColnrT,
        flags: c_int,
    );
    fn nvim_get_hl_attr_active() -> *const c_int;
    fn rs_schar_get_ascii(sc: ScharT) -> c_char;

    // draw_statuscol FFI (Phase 1)
    fn nvim_win_get_nrwidth(wp: WinHandle) -> c_int;
    fn nvim_win_set_nrwidth(wp: WinHandle, val: c_int);
    fn nvim_win_get_statuscol_line_count(wp: WinHandle) -> LinenrT;
    fn nvim_win_set_statuscol_line_count(wp: WinHandle, val: LinenrT);
    fn nvim_win_set_redr_statuscol(wp: WinHandle, val: bool);
    fn nvim_win_get_p_stc(wp: WinHandle) -> *const c_char;
    // nvim_win_get_p_nu and nvim_win_get_p_rnu already declared above
    fn nvim_win_get_nrwidth_line_count(wp: WinHandle) -> LinenrT;
    fn nvim_win_set_nrwidth_line_count(wp: WinHandle, val: LinenrT);
    fn nvim_win_set_nrwidth_width(wp: WinHandle, val: c_int);
    fn nvim_win_clear_valid_bits(wp: WinHandle, bits: c_int);
    // statuscol_T opaque accessors
    fn nvim_stcp_get_width(stcp: *mut c_void) -> c_int;
    fn nvim_stcp_set_width(stcp: *mut c_void, val: c_int);
    fn nvim_stcp_get_hlrec(stcp: *mut c_void) -> *mut c_void;
    fn nvim_stcp_get_fold_vcol(stcp: *mut c_void) -> *mut ColnrT;
    // stl_hlrec_t opaque accessors
    fn nvim_hlrec_get_start(sp: *mut c_void) -> *mut c_char;
    fn nvim_hlrec_get_item(sp: *mut c_void) -> c_int;
    fn nvim_hlrec_get_userhl(sp: *mut c_void) -> c_int;
    fn nvim_hlrec_next(sp: *mut c_void) -> *mut c_void;
    // build_statuscol_str wrapper
    fn nvim_build_statuscol_str(
        wp: WinHandle,
        lnum: LinenrT,
        relnum: LinenrT,
        buf: *mut c_char,
        stcp: *mut c_void,
    ) -> c_int;
    // get_cursor_rel_lnum wrapper
    fn nvim_get_cursor_rel_lnum(wp: WinHandle, lnum: LinenrT) -> LinenrT;
    // transstr_buf wrapper
    fn nvim_transstr_buf(s: *const c_char, slen: isize, buf: *mut c_char, buflen: usize) -> usize;
    // set_vim_var_nr wrapper (from fold_shim.c)
    fn nvim_fold_set_vim_var_nr(vv_idx: c_int, val: i64);
    // number_width (already available as rs_number_width, keep consistent)
    // use_cursor_line_highlight and get_line_number_attr are Rust-exported; we use them directly
    // draw_col_buf and draw_col_fill are Rust-exported; call impl functions directly
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

/// VV_VIRTNUM index (from eval/vars.c).
const VV_VIRTNUM: c_int = 102;

/// MAXPATHL constant (maximum path length).
const MAXPATHL: usize = 4096;

/// MAX_STCWIDTH = MAX_NUMBERWIDTH + SIGN_SHOW_MAX * SIGN_WIDTH + 9 = 20 + 18 + 9 = 47.
const MAX_STCWIDTH: c_int = 47;

/// VALID_WCOL bit (from buffer_defs.h).
const VALID_WCOL: c_int = 0x02;

/// StlFlag: STL_FOLDCOL = 'C', STL_SIGNCOL = 's'.
const STL_FOLDCOL: c_int = b'C' as c_int;
const STL_SIGNCOL: c_int = b's' as c_int;

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
fn draw_col_fill_impl(wlv: *mut WinLineVars, fillchar: ScharT, width: c_int, attr: c_int) {
    unsafe {
        let linebuf_char = nvim_get_linebuf_char();
        let linebuf_attr = nvim_get_linebuf_attr();
        let mut off = (*wlv).off;

        for _ in 0..width {
            *linebuf_char.add(off as usize) = fillchar;
            *linebuf_attr.add(off as usize) = attr;
            off += 1;
        }

        (*wlv).off = off;
    }
}

/// Return true if CursorLineNr highlight is to be used for the number column.
fn use_cursor_line_nr_impl(wp: WinHandle, wlv: *mut WinLineVars) -> bool {
    unsafe {
        let p_cul = nvim_win_get_p_cul(wp) != 0;
        let lnum = (*wlv).lnum;
        let cursorline = nvim_win_get_cursorline(wp);
        let culopt_flags = nvim_win_get_p_culopt_flags(wp);
        let row = (*wlv).row;
        let startrow = (*wlv).startrow;
        let filler_lines = (*wlv).filler_lines;

        p_cul
            && lnum == cursorline
            && (culopt_flags & K_OPT_CULOPT_FLAG_NUMBER) != 0
            && (row == startrow + filler_lines
                || (row > startrow + filler_lines && (culopt_flags & K_OPT_CULOPT_FLAG_LINE) != 0))
    }
}

/// Return line number attribute, combining the appropriate LineNr* highlight
/// with the highest priority sign numhl highlight, if any.
fn get_line_number_attr_impl(wp: WinHandle, wlv: *mut WinLineVars) -> c_int {
    unsafe {
        let mut numhl_attr = (*wlv).sign_num_attr;

        // Get previous sign numhl for virt_lines belonging to the previous line.
        let n_virt_lines = (*wlv).n_virt_lines;
        let filler_todo = (*wlv).filler_todo;
        let n_virt_below = (*wlv).n_virt_below;

        if (n_virt_lines - filler_todo) < n_virt_below {
            let mut prev = (*wlv).prev_num_attr;
            if prev == -1 {
                let lnum = (*wlv).lnum;
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
                (*wlv).prev_num_attr = prev;
            }
            numhl_attr = prev;
        }

        if use_cursor_line_nr_impl(wp, wlv) {
            return hl_combine_attr(nvim_win_hl_attr(wp, HLF_CLN), numhl_attr);
        }

        if nvim_win_get_p_rnu(wp) != 0 {
            let lnum = (*wlv).lnum;
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
fn draw_foldcolumn_impl(wp: WinHandle, wlv: *mut WinLineVars) {
    unsafe {
        // compute_foldcolumn is now exported directly with its C name
        extern "C" {
            #[link_name = "compute_foldcolumn"]
            fn rs_compute_foldcolumn(wp: WinHandle, col: c_int) -> c_int;
        }

        let fdc = rs_compute_foldcolumn(wp, 0);
        if fdc > 0 {
            let lnum = (*wlv).lnum;
            let foldinfo = (*wlv).foldinfo;
            let hlf = if use_cursor_line_highlight_impl(wp, lnum) {
                HLF_CLF
            } else {
                HLF_FC
            };
            let attr = nvim_win_hl_attr(wp, hlf);
            let mut off = (*wlv).off;
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
            (*wlv).off = off;
        }
    }
}

/// Draw a sign in the sign or number column.
fn draw_sign_impl(nrcol: bool, wp: WinHandle, wlv: *mut WinLineVars, sign_idx: c_int) {
    unsafe {
        let lnum = (*wlv).lnum;
        let sattr_text0 = (*wlv).sattrs[sign_idx as usize].text[0];
        let scl_attr = nvim_win_hl_attr(
            wp,
            if use_cursor_line_highlight_impl(wp, lnum) {
                HLF_CLS
            } else {
                HLF_SC
            },
        );

        let row = (*wlv).row;
        let startrow = (*wlv).startrow;
        let filler_lines = (*wlv).filler_lines;
        let filler_todo = (*wlv).filler_todo;

        if sattr_text0 != 0 && row == startrow + filler_lines && filler_todo <= 0 {
            let fill = if nrcol {
                rs_number_width(wp) + 1
            } else {
                SIGN_WIDTH
            };

            let sign_cul_attr = (*wlv).sign_cul_attr;
            let hl_id = (*wlv).sattrs[sign_idx as usize].hl_id;
            let mut attr = if sign_cul_attr != 0 {
                sign_cul_attr
            } else if hl_id != 0 {
                syn_id2attr(hl_id)
            } else {
                0
            };
            attr = hl_combine_attr(scl_attr, attr);

            draw_col_fill_impl(wlv, rs_schar_from_char(c_int::from(b' ')), fill, attr);

            let off = (*wlv).off;
            let sign_pos = off - SIGN_WIDTH - c_int::from(nrcol);
            debug_assert!(sign_pos >= 0);

            let linebuf_char = nvim_get_linebuf_char();
            *linebuf_char.add(sign_pos as usize) = sattr_text0;
            *linebuf_char.add((sign_pos + 1) as usize) = (*wlv).sattrs[sign_idx as usize].text[1];
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
fn draw_lnum_col_impl(wp: WinHandle, wlv: *mut WinLineVars) {
    unsafe {
        extern "C" {
            fn vim_strchr(s: *const i8, c: c_int) -> *mut i8;
            fn nvim_get_p_cpo() -> *const i8;
            fn get_cursor_rel_lnum(wp: WinHandle, lnum: LinenrT) -> LinenrT;
        }

        const CPO_NUMCOL: c_int = b'n' as c_int;

        let p_cpo = nvim_get_p_cpo();
        let has_cpo_n = !vim_strchr(p_cpo, CPO_NUMCOL).is_null();
        let p_nu = nvim_win_get_p_nu(wp) != 0;
        let p_rnu = nvim_win_get_p_rnu(wp) != 0;

        let row = (*wlv).row;
        let startrow = (*wlv).startrow;
        let filler_lines = (*wlv).filler_lines;
        let lnum = (*wlv).lnum;
        let p_bri = nvim_win_get_p_bri(wp) != 0;
        let skipcol = nvim_win_get_skipcol(wp);
        let topline = nvim_win_get_topline(wp);

        if (p_nu || p_rnu)
            && (row == startrow + filler_lines || !has_cpo_n)
            && !((has_cpo_n && !p_bri) && skipcol > 0 && lnum == topline)
        {
            let minscwidth = nvim_win_get_minscwidth(wp);
            let sattr_text0 = (*wlv).sattrs[0].text[0];
            let filler_todo = (*wlv).filler_todo;

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
                    let mut off = (*wlv).off;

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

                    (*wlv).off = off;
                } else {
                    draw_col_fill_impl(wlv, rs_schar_from_char(c_int::from(b' ')), width, attr);
                }
            }
        }
    }
}

// New WLV accessor functions
extern "C" {

    // Additional wlv accessors for win_line_start and fix_for_boguscols

    // HLF constants
    fn nvim_get_hlf_mc() -> c_int;
    fn nvim_get_hlf_cul() -> c_int;

    // Buffer handle for win
    fn nvim_win_get_w_buffer(wp: WinHandle) -> BufHandle;

    // State and quickfix functions for apply_cursorline_highlight
    fn nvim_get_state() -> c_int;
    fn rs_bt_quickfix(buf: BufHandle) -> bool;
    #[link_name = "qf_current_entry"]
    fn nvim_qf_current_entry(wp: WinHandle) -> LinenrT;

    // Diff highlight accessor

    // Reset extra attribute accessors

    // Highlight functions for set_line_attr_for_diff (Rust-exported)
    fn hl_get_underline() -> c_int;

    // handle_breakindent accessors
    fn nvim_win_get_briopt_sbr(wp: WinHandle) -> bool;
    fn nvim_get_breakindent_win_lnum(wp: WinHandle, lnum: LinenrT) -> c_int;

    // fromcol/tocol accessors

    // need_showbreak accessors

    // handle_showbreak_and_filler accessors (not yet declared above)
    fn nvim_win_get_fcs_diff(wp: WinHandle) -> ScharT;
    // rs_get_showbreak_value already declared above

    // has_more_inline_virt accessors
    fn nvim_decor_state_get_future_begin(state: *mut c_void) -> c_int;
    fn nvim_decor_state_get_ranges_count(state: *mut c_void) -> c_int;
    fn nvim_decor_state_get_range_by_idx(state: *mut c_void, idx: c_int) -> *mut c_void;
    // nvim_decor_range_get_start_col is declared above with handle_inline_virtual_text accessors
    // nvim_decor_virt_text_get_width already declared above

    // handle_inline_virtual_text additional accessors (p_extra, sc_extra, etc.)
}

/// Advance wlv->color_cols past the current vcol.
fn advance_color_col_impl(wlv: *mut WinLineVars, vcol: c_int) {
    unsafe {
        let mut color_cols = (*wlv).color_cols;
        if color_cols.is_null() {
            return;
        }

        while *color_cols >= 0 && vcol > *color_cols {
            color_cols = color_cols.add(1);
        }

        if *color_cols < 0 {
            (*wlv).color_cols = std::ptr::null_mut();
        } else {
            (*wlv).color_cols = color_cols;
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
    let mut cells = utf_ptr2cells(p);
    let c_len = utfc_ptr2len(p);

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
#[must_use]
#[export_name = "get_lcs_ext"]
pub extern "C" fn rs_get_lcs_ext(wp: WinHandle) -> ScharT {
    get_lcs_ext_impl(wp)
}

/// Get the rightmost virtual column that needs drawing.
#[must_use]
#[export_name = "get_rightmost_vcol"]
pub extern "C" fn rs_get_rightmost_vcol(wp: WinHandle, color_cols: *const c_int) -> c_int {
    get_rightmost_vcol_impl(wp, color_cols)
}

/// Compute cursorline margins (with caching matching the C margin_columns_win).
#[allow(clippy::too_many_lines)]
#[export_name = "margin_columns_win"]
pub unsafe extern "C" fn rs_margin_columns_win(
    wp: WinHandle,
    left_col: *mut c_int,
    right_col: *mut c_int,
) {
    use std::cell::Cell;
    use std::sync::OnceLock;

    // Cache state: matches the C static variables in margin_columns_win.
    struct Cache {
        saved_w_virtcol: Cell<ColnrT>,
        prev_wp: Cell<WinHandle>,
        prev_width1: Cell<c_int>,
        prev_width2: Cell<c_int>,
        prev_left_col: Cell<c_int>,
        prev_right_col: Cell<c_int>,
    }

    // Safety: single-threaded Neovim main loop; WinHandle is a raw pointer
    // which is not Send/Sync by default, but Neovim is single-threaded so
    // this is safe.
    #[allow(clippy::non_send_fields_in_send_ty)]
    unsafe impl Send for Cache {}
    unsafe impl Sync for Cache {}

    static CACHE: OnceLock<Cache> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Cache {
        saved_w_virtcol: Cell::new(-1),
        prev_wp: Cell::new(WinHandle::null()),
        prev_width1: Cell::new(-1),
        prev_width2: Cell::new(-1),
        prev_left_col: Cell::new(0),
        prev_right_col: Cell::new(0),
    });

    let cur_col_off = rs_win_col_off(wp);
    let width1 = nvim_win_get_view_width(wp) - cur_col_off;
    let width2 = width1 + rs_win_col_off2(wp);
    let virtcol = nvim_win_get_virtcol(wp);

    if cache.saved_w_virtcol.get() == virtcol
        && cache.prev_wp.get().as_ptr() == wp.as_ptr()
        && cache.prev_width1.get() == width1
        && cache.prev_width2.get() == width2
    {
        if !right_col.is_null() {
            *right_col = cache.prev_right_col.get();
        }
        if !left_col.is_null() {
            *left_col = cache.prev_left_col.get();
        }
        return;
    }

    let (left, right) = margin_columns_win_impl(wp);

    cache.prev_left_col.set(left);
    cache.prev_right_col.set(right);
    cache.prev_wp.set(wp);
    cache.prev_width1.set(width1);
    cache.prev_width2.set(width2);
    cache.saved_w_virtcol.set(virtcol);

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
#[must_use]
#[export_name = "use_cursor_line_highlight"]
pub extern "C" fn rs_use_cursor_line_highlight(wp: WinHandle, lnum: LinenrT) -> bool {
    use_cursor_line_highlight_impl(wp, lnum)
}

/// Apply cursorline highlight to the line attributes.
///
/// We make a compromise here (#7383):
/// - low-priority CursorLine if fg is not set
/// - high-priority ("same as Vim" priority) CursorLine if fg is set
unsafe fn apply_cursorline_highlight_impl(wp: WinHandle, wlv: *mut WinLineVars) {
    let hlf_cul = nvim_get_hlf_cul();
    let cul_attr = nvim_win_hl_attr(wp, hlf_cul);
    (*wlv).cul_attr = cul_attr;

    let ae: HlAttrs = rs_syn_attr2entry(cul_attr);

    if ae.rgb_fg_color == -1 && ae.cterm_fg_color == 0 {
        // Low-priority CursorLine when fg is not set
        (*wlv).line_attr_lowprio = cul_attr;
    } else {
        // High-priority CursorLine when fg is set
        let state = nvim_get_state();
        let buf = nvim_win_get_w_buffer(wp);
        let lnum = (*wlv).lnum;

        if (state & MODE_INSERT) == 0 && rs_bt_quickfix(buf) && nvim_qf_current_entry(wp) == lnum {
            // In quickfix window, combine with existing line_attr
            let line_attr = (*wlv).line_attr;
            (*wlv).line_attr = hl_combine_attr(cul_attr, line_attr);
        } else {
            (*wlv).line_attr = cul_attr;
        }
    }
}

/// Apply cursorline highlight (FFI export).
#[export_name = "apply_cursorline_highlight"]
pub unsafe extern "C" fn rs_apply_cursorline_highlight(wp: WinHandle, wlv: *mut WinLineVars) {
    apply_cursorline_highlight_impl(wp, wlv);
}

/// Set line attribute for diff mode highlight.
///
/// Overlay CursorLine onto diff-mode highlight when applicable.
unsafe fn set_line_attr_for_diff_impl(wp: WinHandle, wlv: *mut WinLineVars) {
    let diff_hlf = (*wlv).diff_hlf;
    let line_attr = nvim_win_hl_attr(wp, diff_hlf);
    (*wlv).line_attr = line_attr;

    let cul_attr = (*wlv).cul_attr;
    if cul_attr != 0 {
        let line_attr_lowprio = (*wlv).line_attr_lowprio;
        let new_attr = if line_attr_lowprio != 0 {
            // Low-priority CursorLine: combine with underline
            let combined = hl_combine_attr(cul_attr, line_attr);
            hl_combine_attr(combined, hl_get_underline())
        } else {
            // High-priority CursorLine
            hl_combine_attr(line_attr, cul_attr)
        };
        (*wlv).line_attr = new_attr;
    }
}

/// Set line attribute for diff mode (FFI export).
#[export_name = "set_line_attr_for_diff"]
pub unsafe extern "C" fn rs_set_line_attr_for_diff(wp: WinHandle, wlv: *mut WinLineVars) {
    set_line_attr_for_diff_impl(wp, wlv);
}

/// Handle breakindent: draw indent for wrapped text.
///
/// If need_showbreak is set, breakindent also applies.
unsafe fn handle_breakindent_impl(wp: WinHandle, wlv: *mut WinLineVars) {
    let p_bri = nvim_win_get_p_bri(wp);
    let row = (*wlv).row;
    let startrow = (*wlv).startrow;
    let filler_lines = (*wlv).filler_lines;
    let need_showbreak = (*wlv).need_showbreak;

    if p_bri != 0 && (row > startrow + filler_lines || need_showbreak) {
        let mut attr = 0;
        let diff_hlf = (*wlv).diff_hlf;
        if diff_hlf != 0 {
            attr = nvim_win_hl_attr(wp, diff_hlf);
        }

        let lnum = (*wlv).lnum;
        let mut num = nvim_get_breakindent_win_lnum(wp, lnum);

        if row == startrow {
            num -= rs_win_col_off2(wp);
            let n_extra = (*wlv).n_extra;
            if n_extra < 0 {
                num = 0;
            }
        }

        let vcol_before = (*wlv).vcol;
        let hlf_mc = nvim_get_hlf_mc();

        let linebuf_char = nvim_get_linebuf_char();
        let linebuf_attr = nvim_get_linebuf_attr();
        let linebuf_vcol = nvim_get_linebuf_vcol();

        let space_schar = rs_schar_from_char(c_int::from(b' '));

        for _ in 0..num {
            let off = (*wlv).off;
            *linebuf_char.add(off as usize) = space_schar;

            let vcol = (*wlv).vcol;
            advance_color_col_impl(wlv, vcol);

            let mut myattr = attr;
            let color_cols = (*wlv).color_cols;
            if !color_cols.is_null() && vcol == *color_cols {
                myattr = hl_combine_attr(nvim_win_hl_attr(wp, hlf_mc), myattr);
            }

            *linebuf_attr.add(off as usize) = myattr;
            *linebuf_vcol.add(off as usize) = vcol;
            (*wlv).vcol = vcol + 1;
            (*wlv).off = off + 1;
        }

        // Correct start of highlighted area for 'breakindent'
        let fromcol = (*wlv).fromcol;
        let vcol = (*wlv).vcol;
        if fromcol >= vcol_before && fromcol < vcol {
            (*wlv).fromcol = vcol;
        }

        // Correct end of highlighted area for 'breakindent'
        let tocol = (*wlv).tocol;
        if tocol == vcol_before {
            (*wlv).tocol = vcol;
        }
    }

    // Handle need_showbreak clearing
    let skipcol = nvim_win_get_skipcol(wp);
    let startrow = (*wlv).startrow;
    let p_wrap = nvim_win_get_p_wrap(wp);
    let briopt_sbr = nvim_win_get_briopt_sbr(wp);

    if skipcol > 0 && startrow == 0 && p_wrap != 0 && briopt_sbr {
        (*wlv).need_showbreak = false;
    }
}

/// Handle breakindent (FFI export).
#[export_name = "handle_breakindent"]
pub unsafe extern "C" fn rs_handle_breakindent(wp: WinHandle, wlv: *mut WinLineVars) {
    handle_breakindent_impl(wp, wlv);
}

/// Handle showbreak and filler lines.
///
/// Draws filler content for virtual lines and 'showbreak' string.
unsafe fn handle_showbreak_and_filler_impl(wp: WinHandle, wlv: *mut WinLineVars) {
    let view_width = nvim_win_get_view_width(wp);
    let off = (*wlv).off;
    let remaining = view_width - off;

    let filler_todo = (*wlv).filler_todo;
    let filler_lines = (*wlv).filler_lines;
    let n_virt_lines = (*wlv).n_virt_lines;

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
    let sbr = rs_get_showbreak_value(wp);
    let need_showbreak = (*wlv).need_showbreak;

    if !sbr.is_null() && *sbr != 0 && need_showbreak {
        // Combine 'showbreak' with 'cursorline', prioritizing 'showbreak'
        let cul_attr = (*wlv).cul_attr;
        let at_attr = nvim_win_hl_attr(wp, HLF_AT);
        let attr = hl_combine_attr(cul_attr, at_attr);
        let vcol_before = (*wlv).vcol;

        // Get showbreak string length
        let mut sbr_len: usize = 0;
        let mut p = sbr;
        while *p != 0 {
            sbr_len += 1;
            p = p.add(1);
        }
        draw_col_buf_impl(wp, wlv, sbr, sbr_len, attr, std::ptr::null(), true);

        let vcol = (*wlv).vcol;
        (*wlv).vcol_sbr = vcol;

        // Correct start of highlighted area for 'showbreak'
        let fromcol = (*wlv).fromcol;
        if fromcol >= vcol_before && fromcol < vcol {
            (*wlv).fromcol = vcol;
        }

        // Correct end of highlighted area for 'showbreak'
        let tocol = (*wlv).tocol;
        if tocol == vcol_before {
            (*wlv).tocol = vcol;
        }
    }

    // Clear need_showbreak flag when appropriate
    let skipcol = nvim_win_get_skipcol(wp);
    let startrow = (*wlv).startrow;
    let p_wrap = nvim_win_get_p_wrap(wp);
    let briopt_sbr = nvim_win_get_briopt_sbr(wp);

    if skipcol == 0 || startrow > 0 || p_wrap == 0 || !briopt_sbr {
        (*wlv).need_showbreak = false;
    }
}

/// Handle showbreak and filler (FFI export).
#[export_name = "handle_showbreak_and_filler"]
pub unsafe extern "C" fn rs_handle_showbreak_and_filler(wp: WinHandle, wlv: *mut WinLineVars) {
    handle_showbreak_and_filler_impl(wp, wlv);
}

/// Check if there is more inline virtual text to be drawn.
///
/// Returns true if there are more inline virtual text chunks to draw at or after
/// the given column position `v`.
unsafe fn has_more_inline_virt_impl(wlv: *mut WinLineVars, v: isize) -> bool {
    // Check if still inside current virt_inline
    let virt_inline_i = (*wlv).virt_inline_i;
    let virt_inline_size = (*wlv).virt_inline.size;
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
#[export_name = "has_more_inline_virt"]
pub unsafe extern "C" fn rs_has_more_inline_virt(wlv: *mut WinLineVars, v: isize) -> bool {
    has_more_inline_virt_impl(wlv, v)
}

/// Fill cells with a character.
#[export_name = "draw_col_fill"]
pub extern "C" fn rs_draw_col_fill(
    wlv: *mut WinLineVars,
    fillchar: ScharT,
    width: c_int,
    attr: c_int,
) {
    draw_col_fill_impl(wlv, fillchar, width, attr);
}

/// Fill fold column (full implementation with linebuf support).
#[export_name = "fill_foldcolumn"]
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
#[export_name = "draw_foldcolumn"]
pub extern "C" fn rs_draw_foldcolumn(wp: WinHandle, wlv: *mut WinLineVars) {
    draw_foldcolumn_impl(wp, wlv);
}

/// Check if cursor line number highlight should be used.
#[export_name = "use_cursor_line_nr"]
pub extern "C" fn rs_use_cursor_line_nr(wp: WinHandle, wlv: *mut WinLineVars) -> bool {
    use_cursor_line_nr_impl(wp, wlv)
}

/// Get line number attribute.
#[export_name = "get_line_number_attr"]
pub extern "C" fn rs_get_line_number_attr(wp: WinHandle, wlv: *mut WinLineVars) -> c_int {
    get_line_number_attr_impl(wp, wlv)
}

/// Draw sign in sign or number column.
#[export_name = "draw_sign"]
pub extern "C" fn rs_draw_sign(nrcol: bool, wp: WinHandle, wlv: *mut WinLineVars, sign_idx: c_int) {
    draw_sign_impl(nrcol, wp, wlv, sign_idx);
}

/// Draw line number column.
#[export_name = "draw_lnum_col"]
pub extern "C" fn rs_draw_lnum_col(wp: WinHandle, wlv: *mut WinLineVars) {
    draw_lnum_col_impl(wp, wlv);
}

/// Advance color_cols past current vcol.
#[export_name = "advance_color_col"]
pub extern "C" fn rs_advance_color_col(wlv: *mut WinLineVars, vcol: c_int) {
    advance_color_col_impl(wlv, vcol);
}

/// Put a character in the screen buffer for line drawing.
///
/// # Safety
/// - `pp` must be a valid pointer to a pointer to a NUL-terminated UTF-8 string
/// - `dest` must be a valid pointer to at least `maxcells` schar_T values
/// - `dest[0]` must not be 0 (caller ensures not overwriting right half of double-width)
#[export_name = "line_putchar"]
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
            let c_len = utfc_ptr2len(virt_str);
            let cells = if *virt_str == TAB {
                let ts = nvim_buf_get_p_ts(buf);
                let vts = nvim_buf_get_p_vts_array(buf);
                rs_tabstop_padding(vcol, ts, vts)
            } else {
                utf_ptr2cells(virt_str)
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
#[export_name = "draw_virt_text_item"]
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
#[export_name = "draw_virt_text"]
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
unsafe fn win_line_start_impl(wp: WinHandle, wlv: *mut WinLineVars) {
    (*wlv).col = 0;
    (*wlv).off = 0;
    (*wlv).need_lbr = false;

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
#[export_name = "win_line_start"]
pub unsafe extern "C" fn rs_win_line_start(wp: WinHandle, wlv: *mut WinLineVars) {
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
unsafe fn fix_for_boguscols_impl(wlv: *mut WinLineVars) {
    let vcol_off_co = (*wlv).vcol_off_co;
    let boguscols = (*wlv).boguscols;

    // wlv->n_extra += wlv->vcol_off_co
    let n_extra = (*wlv).n_extra;
    (*wlv).n_extra = n_extra + vcol_off_co;

    // wlv->vcol -= wlv->vcol_off_co
    let vcol = (*wlv).vcol;
    (*wlv).vcol = vcol - vcol_off_co as ColnrT;

    // wlv->vcol_off_co = 0
    (*wlv).vcol_off_co = 0;

    // wlv->col -= wlv->boguscols
    let col = (*wlv).col;
    (*wlv).col = col - boguscols;

    // wlv->old_boguscols = wlv->boguscols
    (*wlv).old_boguscols = boguscols;

    // wlv->boguscols = 0
    (*wlv).boguscols = 0;
}

/// FFI export for fix_for_boguscols.
#[export_name = "fix_for_boguscols"]
pub unsafe extern "C" fn rs_fix_for_boguscols(wlv: *mut WinLineVars) {
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
    wlv: *mut WinLineVars,
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

    while ptr < text_end && (*wlv).off < view_width {
        let off = (*wlv).off;

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
            advance_color_col_impl(wlv, (*wlv).vcol);
            let color_cols = (*wlv).color_cols;
            if !color_cols.is_null() && (*wlv).vcol == *color_cols {
                myattr = hl_combine_attr(nvim_win_hl_attr(wp, hlf_mc), myattr);
            }
        }

        // Fill in attr and vcol for each cell
        for _ in 0..cells {
            let current_off = (*wlv).off;
            *linebuf_attr.add(current_off as usize) = myattr;

            let vcol_val = if inc_vcol {
                let v = (*wlv).vcol;
                (*wlv).vcol = v + 1;
                v
            } else if !fold_vcol_ptr.is_null() {
                let v = *fold_vcol_ptr;
                fold_vcol_ptr = fold_vcol_ptr.add(1);
                v
            } else {
                -1
            };
            *linebuf_vcol.add(current_off as usize) = vcol_val;

            (*wlv).off = current_off + 1;
        }
    }
}

/// FFI export for draw_col_buf.
#[export_name = "draw_col_buf"]
pub unsafe extern "C" fn rs_draw_col_buf(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    text: *const c_char,
    len: usize,
    attr: c_int,
    fold_vcol: *const ColnrT,
    inc_vcol: bool,
) {
    draw_col_buf_impl(wp, wlv, text, len, attr, fold_vcol, inc_vcol);
}

// ============================================================================
// handle_inline_virtual_text - Handle inline virtual text rendering
// ============================================================================

/// INT_MIN constant for marking draw_col as processed.
const INT_MIN: c_int = c_int::MIN;

/// Handle inline virtual text processing.
///
/// This function iterates through decoration state to find inline virtual text
/// at the current position, setting up wlv fields for rendering.
///
/// # Safety
/// - `wp` must be a valid window handle
/// - `wlv` must be a valid winlinevars_T handle
/// - `v` is the current column position
/// - `selected` indicates if the position is selected (for overlay visibility)
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_possible_truncation)]
unsafe fn handle_inline_virtual_text_impl(
    _wp: WinHandle,
    wlv: *mut WinLineVars,
    v: isize,
    selected: bool,
) {
    loop {
        let n_extra = (*wlv).n_extra;
        if n_extra != 0 {
            break;
        }

        let virt_inline_i = (*wlv).virt_inline_i;
        let virt_inline_size = (*wlv).virt_inline.size;

        if virt_inline_i >= virt_inline_size {
            // Need to find inline virtual text
            (*wlv).virt_inline = KVec::empty();
            (*wlv).virt_inline_i = 0;

            let state = nvim_get_decor_state();
            let end = nvim_decor_state_get_current_end(state);
            let row = nvim_decor_state_get_row(state);
            let off = (*wlv).off;

            let mut found = false;
            for i in 0..end {
                let item = nvim_decor_state_get_active_range(state, i);
                if item.is_null() {
                    continue;
                }

                let draw_col = nvim_decor_range_get_draw_col(item);
                if draw_col == -3 {
                    // No more inline virtual text before this non-inline virtual text item,
                    // so its position can be decided now.
                    nvim_decor_init_draw_col(off, selected, item);
                }

                let start_row = nvim_decor_range_get_start_row(item);
                let kind = nvim_decor_range_get_kind(item);
                let vt = nvim_decor_range_get_virt_text(item);

                if start_row != row || kind != K_DECOR_KIND_VIRT_TEXT || vt.is_null() {
                    continue;
                }

                let pos = nvim_decor_virt_text_get_pos(vt);
                let width = nvim_decor_virt_text_get_width(vt);

                if pos != K_VPOS_INLINE || width == 0 {
                    continue;
                }

                let draw_col = nvim_decor_range_get_draw_col(item);
                let start_col = nvim_decor_range_get_start_col(item);

                if draw_col >= -1 && start_col == v as c_int {
                    // Found matching inline virtual text
                    let virt_inline_data = nvim_decor_range_get_virt_inline_data(item);
                    let hl_mode = nvim_decor_range_get_virt_inline_hl_mode(item);
                    (*wlv).virt_inline = if virt_inline_data.is_null() {
                        KVec::empty()
                    } else {
                        *(virt_inline_data as *const KVec<VirtTextChunkC>)
                    };
                    (*wlv).virt_inline_hl_mode = hl_mode;
                    nvim_decor_range_set_draw_col(item, INT_MIN);
                    found = true;
                    break;
                }
            }

            if !found {
                // No more inline virtual text here
                break;
            }
        } else {
            // Already inside existing inline virtual text with multiple chunks
            let virt_inline_ptr =
                std::ptr::from_mut::<KVec<VirtTextChunkC>>(&mut (*wlv).virt_inline)
                    .cast::<c_void>();
            let mut pos = (*wlv).virt_inline_i;
            let mut attr: c_int = 0;

            let text = nvim_next_virt_text_chunk(virt_inline_ptr, &mut pos, &mut attr);
            (*wlv).virt_inline_i = pos;

            if text.is_null() {
                continue;
            }

            // Calculate text length manually (no libc::strlen)
            let mut text_len: c_int = 0;
            {
                let mut p = text;
                while *p != 0 {
                    text_len += 1;
                    p = p.add(1);
                }
            }

            if text_len == 0 {
                continue;
            }

            (*wlv).p_extra = text.cast_mut();
            (*wlv).n_extra = text_len;
            (*wlv).sc_extra = 0; // NUL
            (*wlv).sc_final = 0; // NUL
            (*wlv).extra_attr = attr;

            let n_attr = mb_charlen(text);
            (*wlv).n_attr = n_attr;

            // If the text didn't reach until the first window column we need to skip cells.
            let skip_cells = (*wlv).skip_cells;
            if skip_cells > 0 {
                let p_extra = (*wlv).p_extra;
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                let virt_text_width = mb_string2cells(p_extra) as c_int;

                if virt_text_width > skip_cells {
                    let mut skip_cells_remaining = skip_cells;
                    let mut p = p_extra;
                    let mut n_extra_val = (*wlv).n_extra;
                    let mut n_attr_val = (*wlv).n_attr;

                    // Skip cells in the text
                    while skip_cells_remaining > 0 {
                        let cells = utf_ptr2cells(p);
                        if cells > skip_cells_remaining {
                            break;
                        }
                        let c_len = utfc_ptr2len(p);
                        skip_cells_remaining -= cells;
                        p = p.add(c_len as usize);
                        n_extra_val -= c_len;
                        n_attr_val -= 1;
                    }

                    (*wlv).p_extra = p;
                    (*wlv).n_extra = n_extra_val;
                    (*wlv).n_attr = n_attr_val;

                    // Skipped cells needed to be accounted for in vcol
                    let skipped_cells = (*wlv).skipped_cells;
                    (*wlv).skipped_cells = skipped_cells + skip_cells - skip_cells_remaining;
                    (*wlv).skip_cells = skip_cells_remaining;
                } else {
                    // The whole text is left of the window, drop it and advance to the next one
                    (*wlv).skip_cells = skip_cells - virt_text_width;

                    // Skipped cells needed to be accounted for in vcol
                    let skipped_cells = (*wlv).skipped_cells;
                    (*wlv).skipped_cells = skipped_cells + virt_text_width;
                    (*wlv).n_attr = 0;
                    (*wlv).n_extra = 0;

                    // Go to the start so the next virtual text chunk can be selected
                    continue;
                }
            }

            // Assert n_extra > 0
            debug_assert!((*wlv).n_extra > 0);
            (*wlv).extra_for_extmark = true;
        }

        // Break after successfully processing (either found new or processed existing)
        break;
    }
}

/// FFI export for handle_inline_virtual_text.
#[export_name = "handle_inline_virtual_text"]
pub unsafe extern "C" fn rs_handle_inline_virtual_text(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    v: isize,
    selected: bool,
) {
    handle_inline_virtual_text_impl(wp, wlv, v, selected);
}

// ============================================================================
// wlv_put_linebuf - Put line buffer to screen
// ============================================================================

/// SLF_RIGHTLEFT flag from grid.h.
const SLF_RIGHTLEFT: c_int = 1;

/// Put the rendered line buffer to the screen.
///
/// # Safety
/// - `wp` must be a valid window handle
/// - `wlv` must be a valid winlinevars_T handle
#[allow(clippy::too_many_lines)]
unsafe fn wlv_put_linebuf_impl(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    endcol: c_int,
    clear_end: bool,
    bg_attr: c_int,
    flags: c_int,
) {
    let grid = nvim_win_get_w_grid(wp);
    let view_width = nvim_win_get_view_width(wp);

    let mut startcol: c_int = 0;
    let mut endcol_mut = endcol;
    let mut clear_width = if clear_end { view_width } else { endcol };
    let mut flags_mut = flags;

    // assert!(!(flags & SLF_RIGHTLEFT));
    debug_assert!(flags & SLF_RIGHTLEFT == 0);

    if nvim_win_get_p_rl(wp) != 0 {
        rs_linebuf_mirror(&mut startcol, &mut endcol_mut, &mut clear_width, view_width);
        flags_mut |= SLF_RIGHTLEFT;
    }

    // Take care of putting "<<<" on the first line for 'smoothscroll'.
    let wlv_row = (*wlv).row;
    let skipcol = nvim_win_get_skipcol(wp);
    let showbreak = rs_get_showbreak_value(wp);
    let p_nu = nvim_win_get_p_nu(wp);
    let p_rnu = nvim_win_get_p_rnu(wp);
    let p_list = nvim_win_get_p_list(wp);
    let lcs_prec = nvim_win_get_lcs_prec(wp);

    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();
    let hl_attr_active = nvim_get_hl_attr_active();

    if wlv_row == 0
        && skipcol > 0
        // do not overwrite the 'showbreak' text with "<<<"
        && (showbreak.is_null() || *showbreak == 0)
        // do not overwrite the 'listchars' "precedes" text with "<<<"
        && !(p_list != 0 && lcs_prec != 0)
    {
        let mut off: c_int = 0;
        if p_nu != 0 && p_rnu != 0 {
            // do not overwrite the line number, change "123 text" to "123<<<xt".
            while off < view_width {
                let sc = *linebuf_char.add(off as usize);
                let ascii_char = rs_schar_get_ascii(sc);
                if !((ascii_char as u8).is_ascii_digit()) {
                    break;
                }
                off += 1;
            }
        }

        // Draw "<<<" characters
        for _i in 0..3 {
            if off >= view_width {
                break;
            }
            if off + 1 < view_width && *linebuf_char.add((off + 1) as usize) == 0 {
                // When the first half of a double-width character is
                // overwritten, change the second half to a space.
                *linebuf_char.add((off + 1) as usize) = rs_schar_from_char(c_int::from(b' '));
            }
            *linebuf_char.add(off as usize) = rs_schar_from_char(c_int::from(b'<'));
            // HL_ATTR(HLF_AT) = hl_attr_active[HLF_AT]
            *linebuf_attr.add(off as usize) = *hl_attr_active.add(HLF_AT as usize);
            off += 1;
        }
    }

    let row = wlv_row;
    let mut row_adjusted = row;
    let mut coloff: c_int = 0;
    let g = rs_grid_adjust(grid, &mut row_adjusted, &mut coloff);

    let vcol = (*wlv).vcol;
    rs_grid_put_linebuf(
        g,
        row_adjusted,
        coloff,
        startcol,
        endcol_mut,
        clear_width,
        bg_attr,
        0,
        vcol - 1,
        flags_mut,
    );
}

/// FFI export for wlv_put_linebuf.
#[export_name = "wlv_put_linebuf"]
pub unsafe extern "C" fn rs_wlv_put_linebuf(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    endcol: c_int,
    clear_end: bool,
    bg_attr: c_int,
    flags: c_int,
) {
    wlv_put_linebuf_impl(wp, wlv, endcol, clear_end, bg_attr, flags);
}

// ============================================================================
// Phase 2: Line Drawing State Helpers
// ============================================================================

// Additional WLV accessors for state management (char_attr exists in C)

/// Check if there are filler lines remaining to draw.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_has_filler_todo(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).filler_todo > 0)
}

/// Check if all filler lines have been drawn.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_filler_complete(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).filler_todo == 0)
}

/// Get the number of filler lines for diff display.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_filler_lines(wlv: *mut WinLineVars) -> c_int {
    (*wlv).filler_lines
}

/// Check if this is a virtual line (filler line within actual text).
///
/// Returns true if filler_todo > filler_lines (i.e., drawing virtual lines
/// that come before the diff filler lines).
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_is_virtual_line(wlv: *mut WinLineVars) -> c_int {
    let filler_todo = (*wlv).filler_todo;
    let filler_lines = (*wlv).filler_lines;
    c_int::from(filler_todo > filler_lines)
}

/// Get the character attribute for the current cell.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_char_attr(wlv: *mut WinLineVars) -> c_int {
    (*wlv).char_attr
}

/// Set the character attribute for the current cell.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_char_attr(wlv: *mut WinLineVars, attr: c_int) {
    (*wlv).char_attr = attr;
}

/// Combine an attribute with the current character attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_combine_char_attr(wlv: *mut WinLineVars, attr: c_int) {
    let current = (*wlv).char_attr;
    let combined = hl_combine_attr(current, attr);
    (*wlv).char_attr = combined;
}

/// Get the number of extra characters to display.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_n_extra(wlv: *mut WinLineVars) -> c_int {
    (*wlv).n_extra
}

/// Check if there are extra characters to display.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_has_n_extra(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).n_extra > 0)
}

/// Decrement the n_extra counter.
///
/// Returns the new value after decrement.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_dec_n_extra(wlv: *mut WinLineVars) -> c_int {
    let current = (*wlv).n_extra;
    let new_val = current - 1;
    (*wlv).n_extra = new_val;
    new_val
}

/// Get the number of cells to skip.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_skip_cells(wlv: *mut WinLineVars) -> c_int {
    (*wlv).skip_cells
}

/// Set the number of cells to skip.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_skip_cells(wlv: *mut WinLineVars, val: c_int) {
    (*wlv).skip_cells = val;
}

/// Decrement the skip_cells counter.
///
/// Returns the new value after decrement.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_dec_skip_cells(wlv: *mut WinLineVars) -> c_int {
    let current = (*wlv).skip_cells;
    let new_val = if current > 0 { current - 1 } else { 0 };
    (*wlv).skip_cells = new_val;
    new_val
}

/// Check if we should skip the current cell.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_should_skip(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).skip_cells > 0)
}

/// Get the number of attribute cells remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_n_attr(wlv: *mut WinLineVars) -> c_int {
    (*wlv).n_attr
}

/// Set the number of attribute cells.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_n_attr(wlv: *mut WinLineVars, val: c_int) {
    (*wlv).n_attr = val;
}

/// Check if there are attribute cells remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_has_n_attr(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).n_attr > 0)
}

/// Decrement the n_attr counter.
///
/// Returns the new value after decrement.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_dec_n_attr(wlv: *mut WinLineVars) -> c_int {
    let current = (*wlv).n_attr;
    let new_val = if current > 0 { current - 1 } else { 0 };
    (*wlv).n_attr = new_val;
    new_val
}

/// Get the row number being drawn.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_row(wlv: *mut WinLineVars) -> c_int {
    (*wlv).row
}

/// Get the starting row for this line.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_startrow(wlv: *mut WinLineVars) -> c_int {
    (*wlv).startrow
}

/// Check if we are on the first row of a wrapped line.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_is_first_row(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).row == (*wlv).startrow)
}

/// Get the line number being drawn.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_lnum(wlv: *mut WinLineVars) -> LinenrT {
    (*wlv).lnum
}

/// Get the current column offset.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_off(wlv: *mut WinLineVars) -> c_int {
    (*wlv).off
}

/// Set the current column offset.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_off(wlv: *mut WinLineVars, val: c_int) {
    (*wlv).off = val;
}

/// Advance the column offset by a given amount.
///
/// Returns the new offset value.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_advance_off(wlv: *mut WinLineVars, delta: c_int) -> c_int {
    let current = (*wlv).off;
    let new_val = current + delta;
    (*wlv).off = new_val;
    new_val
}

/// Get the fold info for the current line.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_fold_level(wlv: *mut WinLineVars) -> c_int {
    let fi = (*wlv).foldinfo;
    fi.fi_level
}

/// Check if the current line is folded.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_is_folded(wlv: *mut WinLineVars) -> c_int {
    let fi = (*wlv).foldinfo;
    c_int::from(fi.fi_lines > 0)
}

/// Get the number of lines in the current fold.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_fold_lines(wlv: *mut WinLineVars) -> LinenrT {
    let fi = (*wlv).foldinfo;
    fi.fi_lines
}

/// Get the number of virtual lines above this line.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_n_virt_lines(wlv: *mut WinLineVars) -> c_int {
    (*wlv).n_virt_lines
}

/// Get the number of virtual lines below this line.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_n_virt_below(wlv: *mut WinLineVars) -> c_int {
    (*wlv).n_virt_below
}

/// Check if there are any virtual lines for this line.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_has_virt_lines(wlv: *mut WinLineVars) -> c_int {
    let above = (*wlv).n_virt_lines;
    let below = (*wlv).n_virt_below;
    c_int::from(above > 0 || below > 0)
}

// ============================================================================
// Phase 3: Extmark Decoration Attribute Helpers
// ============================================================================

/// Check if n_extra is set for inline virtual text (extmark).
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_is_extmark_extra(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).extra_for_extmark)
}

/// Set the extmark extra flag.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_extmark_extra(wlv: *mut WinLineVars, val: c_int) {
    (*wlv).extra_for_extmark = val != 0;
}

/// Check if we should apply extmark attributes.
///
/// Returns true if either there's no n_extra or it's not for extmark.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_apply_extmark_attr(wlv: *mut WinLineVars) -> c_int {
    let n_extra = (*wlv).n_extra;
    let extra_for_extmark = (*wlv).extra_for_extmark;
    c_int::from(n_extra == 0 || !extra_for_extmark)
}

/// Get the virtual inline highlight mode.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_virt_inline_hl_mode(wlv: *mut WinLineVars) -> c_int {
    (*wlv).virt_inline_hl_mode
}

/// Set the virtual inline highlight mode.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_virt_inline_hl_mode(wlv: *mut WinLineVars, mode: c_int) {
    (*wlv).virt_inline_hl_mode = mode;
}

/// Check if the virtual inline highlight mode is replace.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_virt_inline_replaces(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).virt_inline_hl_mode <= HL_MODE_REPLACE)
}

/// Get the number of skipped cells for virtual text.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_skipped_cells(wlv: *mut WinLineVars) -> c_int {
    (*wlv).skipped_cells
}

/// Set the number of skipped cells for virtual text.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_skipped_cells(wlv: *mut WinLineVars, val: c_int) {
    (*wlv).skipped_cells = val;
}

/// Add to the number of skipped cells.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_add_skipped_cells(wlv: *mut WinLineVars, delta: c_int) -> c_int {
    let current = (*wlv).skipped_cells;
    let new_val = current + delta;
    (*wlv).skipped_cells = new_val;
    new_val
}

/// Get the p_extra pointer (extra text to display).
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_p_extra(wlv: *mut WinLineVars) -> *const c_char {
    (*wlv).p_extra
}

/// Set the p_extra pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_p_extra(wlv: *mut WinLineVars, ptr: *const c_char) {
    (*wlv).p_extra = ptr.cast_mut();
}

/// Check if there's extra text to display.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_has_p_extra(wlv: *mut WinLineVars) -> c_int {
    let p = (*wlv).p_extra;
    c_int::from(!p.is_null())
}

/// Get the sc_extra character (repeated extra character).
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_sc_extra(wlv: *mut WinLineVars) -> ScharT {
    (*wlv).sc_extra
}

/// Set the sc_extra character.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_sc_extra(wlv: *mut WinLineVars, c: ScharT) {
    (*wlv).sc_extra = c;
}

/// Get the sc_final character (terminating extra character).
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_sc_final(wlv: *mut WinLineVars) -> ScharT {
    (*wlv).sc_final
}

/// Set the sc_final character.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_sc_final(wlv: *mut WinLineVars, c: ScharT) {
    (*wlv).sc_final = c;
}

/// Check if using repeated character for extra (sc_extra != NUL).
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_uses_sc_extra(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).sc_extra != 0)
}

/// Clear the extra text state.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_clear_extra(wlv: *mut WinLineVars) {
    (*wlv).n_extra = 0;
    (*wlv).p_extra = std::ptr::null_mut();
    (*wlv).sc_extra = 0; // NUL
    (*wlv).sc_final = 0; // NUL
    (*wlv).extra_for_extmark = false;
}

/// Setup extra text from a string.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_setup_extra(
    wlv: *mut WinLineVars,
    text: *const c_char,
    len: c_int,
    attr: c_int,
    for_extmark: c_int,
) {
    (*wlv).p_extra = text.cast_mut();
    (*wlv).n_extra = len;
    (*wlv).n_attr = len;
    (*wlv).extra_attr = attr;
    (*wlv).sc_extra = 0; // NUL
    (*wlv).sc_final = 0; // NUL
    (*wlv).extra_for_extmark = for_extmark != 0;
}

/// Setup extra text with a repeated character.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_setup_extra_schar(
    wlv: *mut WinLineVars,
    sc: ScharT,
    count: c_int,
    attr: c_int,
) {
    (*wlv).p_extra = std::ptr::null_mut();
    (*wlv).n_extra = count;
    (*wlv).n_attr = count;
    (*wlv).extra_attr = attr;
    (*wlv).sc_extra = sc;
    (*wlv).sc_final = 0; // NUL
    (*wlv).extra_for_extmark = false;
}

/// Get the extra attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_extra_attr(wlv: *mut WinLineVars) -> c_int {
    (*wlv).extra_attr
}

/// Set the extra attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_extra_attr(wlv: *mut WinLineVars, attr: c_int) {
    (*wlv).extra_attr = attr;
}

/// Get the vcol_off_co (conceal offset).
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_vcol_off_co(wlv: *mut WinLineVars) -> c_int {
    (*wlv).vcol_off_co
}

/// Set the vcol_off_co (conceal offset).
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_vcol_off_co(wlv: *mut WinLineVars, val: c_int) {
    (*wlv).vcol_off_co = val;
}

/// Increment the vcol_off_co.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_inc_vcol_off_co(wlv: *mut WinLineVars) -> c_int {
    let val = (*wlv).vcol_off_co + 1;
    (*wlv).vcol_off_co = val;
    val
}

/// Get the bogus columns count.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_boguscols(wlv: *mut WinLineVars) -> c_int {
    (*wlv).boguscols
}

/// Set the bogus columns count.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_boguscols(wlv: *mut WinLineVars, val: c_int) {
    (*wlv).boguscols = val;
}

/// Get the old bogus columns count.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_old_boguscols(wlv: *mut WinLineVars) -> c_int {
    (*wlv).old_boguscols
}

/// Check if we need to handle bogus columns.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_has_boguscols(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).boguscols > 0)
}

/// Check if we need line break handling.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_needs_lbr(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).need_lbr)
}

/// Set the line break needed flag.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_needs_lbr(wlv: *mut WinLineVars, val: c_int) {
    (*wlv).need_lbr = val != 0;
}

// ============================================================================
// Phase 4: Syntax & Highlighting Attribute Helpers
// ============================================================================

/// Get the line attribute for the whole line.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_line_attr(wlv: *mut WinLineVars) -> c_int {
    (*wlv).line_attr
}

/// Set the line attribute for the whole line.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_line_attr(wlv: *mut WinLineVars, attr: c_int) {
    (*wlv).line_attr = attr;
}

/// Get the low-priority line attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_line_attr_lowprio(wlv: *mut WinLineVars) -> c_int {
    (*wlv).line_attr_lowprio
}

/// Set the low-priority line attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_line_attr_lowprio(wlv: *mut WinLineVars, attr: c_int) {
    (*wlv).line_attr_lowprio = attr;
}

/// Check if the line has a line attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_has_line_attr(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).line_attr != 0 || (*wlv).line_attr_lowprio != 0)
}

/// Combine an attribute with the line attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_combine_line_attr(wlv: *mut WinLineVars, attr: c_int) {
    let current = (*wlv).line_attr;
    let combined = hl_combine_attr(current, attr);
    (*wlv).line_attr = combined;
}

/// Combine an attribute with the low-priority line attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_combine_line_attr_lowprio(wlv: *mut WinLineVars, attr: c_int) {
    let current = (*wlv).line_attr_lowprio;
    let combined = hl_combine_attr(current, attr);
    (*wlv).line_attr_lowprio = combined;
}

/// Get the effective line attribute (combines line_attr and line_attr_lowprio).
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_effective_line_attr(wlv: *mut WinLineVars) -> c_int {
    let line_attr = (*wlv).line_attr;
    let line_attr_lowprio = (*wlv).line_attr_lowprio;
    hl_combine_attr(line_attr_lowprio, line_attr)
}

/// Clear the line attributes.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_clear_line_attr(wlv: *mut WinLineVars) {
    (*wlv).line_attr = 0;
    (*wlv).line_attr_lowprio = 0;
}

/// Combine multiple attributes.
///
/// Takes a base attribute and combines it with up to 3 overlay attributes.
/// Null (0) attributes are skipped.
#[no_mangle]
pub unsafe extern "C" fn rs_combine_multi_attrs(
    base: c_int,
    attr1: c_int,
    attr2: c_int,
    attr3: c_int,
) -> c_int {
    let mut result = base;
    if attr1 != 0 {
        result = hl_combine_attr(result, attr1);
    }
    if attr2 != 0 {
        result = hl_combine_attr(result, attr2);
    }
    if attr3 != 0 {
        result = hl_combine_attr(result, attr3);
    }
    result
}

/// Check if an attribute is non-default (has highlighting).
#[no_mangle]
pub extern "C" fn rs_attr_has_highlight(attr: c_int) -> c_int {
    c_int::from(attr != 0)
}

/// Get the sign cul attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_sign_cul_attr(wlv: *mut WinLineVars) -> c_int {
    (*wlv).sign_cul_attr
}

/// Get the sign num attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_sign_num_attr(wlv: *mut WinLineVars) -> c_int {
    (*wlv).sign_num_attr
}

/// Check if there's a sign-related CursorLine attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_has_sign_cul_attr(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).sign_cul_attr != 0)
}

/// Check if there's a sign-related number attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_has_sign_num_attr(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).sign_num_attr != 0)
}

/// Get the CursorLine attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_cul_attr(wlv: *mut WinLineVars) -> c_int {
    (*wlv).cul_attr
}

/// Set the CursorLine attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_cul_attr(wlv: *mut WinLineVars, attr: c_int) {
    (*wlv).cul_attr = attr;
}

/// Check if there's a CursorLine attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_has_cul_attr(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).cul_attr != 0)
}

/// Get the diff highlight flag.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_diff_hlf(wlv: *mut WinLineVars) -> c_int {
    (*wlv).diff_hlf
}

/// Set the diff highlight flag.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_diff_hlf(wlv: *mut WinLineVars, hlf: c_int) {
    (*wlv).diff_hlf = hlf;
}

/// Check if this line has diff highlighting.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_has_diff(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).diff_hlf != 0)
}

/// Get the visual selection fromcol.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_fromcol(wlv: *mut WinLineVars) -> c_int {
    (*wlv).fromcol
}

/// Set the visual selection fromcol.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_fromcol(wlv: *mut WinLineVars, col: c_int) {
    (*wlv).fromcol = col;
}

/// Get the visual selection tocol.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_tocol(wlv: *mut WinLineVars) -> c_int {
    (*wlv).tocol
}

/// Set the visual selection tocol.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_tocol(wlv: *mut WinLineVars, col: c_int) {
    (*wlv).tocol = col;
}

/// Check if a column is within the visual selection range.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_in_visual_range(wlv: *mut WinLineVars, col: c_int) -> c_int {
    let fromcol = (*wlv).fromcol;
    let tocol = (*wlv).tocol;
    c_int::from(col >= fromcol && col < tocol)
}

/// Check if visual selection is active on this line.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_has_visual(wlv: *mut WinLineVars) -> c_int {
    let tocol = (*wlv).tocol;
    c_int::from(tocol > 0)
}

/// Get the previous number attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_prev_num_attr(wlv: *mut WinLineVars) -> c_int {
    (*wlv).prev_num_attr
}

/// Set the previous number attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_prev_num_attr(wlv: *mut WinLineVars, attr: c_int) {
    (*wlv).prev_num_attr = attr;
}

/// Check if the number attribute has changed.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_num_attr_changed(wlv: *mut WinLineVars, attr: c_int) -> c_int {
    c_int::from((*wlv).prev_num_attr != attr)
}

/// Check if should reset extra attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_reset_extra_attr(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).reset_extra_attr)
}

/// Set the reset extra attribute flag.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_reset_extra_attr(wlv: *mut WinLineVars, val: c_int) {
    (*wlv).reset_extra_attr = val != 0;
}

/// Check if showing showbreak.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_need_showbreak(wlv: *mut WinLineVars) -> c_int {
    c_int::from((*wlv).need_showbreak)
}

/// Set the need showbreak flag.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_need_showbreak(wlv: *mut WinLineVars, val: c_int) {
    (*wlv).need_showbreak = val != 0;
}

// ============================================================================
// Phase 5: Line Numbers & Signs - Gutter Rendering Helpers
// ============================================================================

/// Check if we're on the text row (not filler).
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_on_text_row(wlv: *mut WinLineVars) -> c_int {
    let filler_todo = (*wlv).filler_todo;
    c_int::from(filler_todo <= 0)
}

/// Get sign text character at position.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_sign_text(
    wlv: *mut WinLineVars,
    sign_idx: c_int,
    pos: c_int,
) -> ScharT {
    (*wlv).sattrs[sign_idx as usize].text[pos as usize]
}

/// Get sign highlight id.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_sign_hl_id(wlv: *mut WinLineVars, sign_idx: c_int) -> c_int {
    (*wlv).sattrs[sign_idx as usize].hl_id
}

/// Check if sign has text.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_sign_has_text(wlv: *mut WinLineVars, sign_idx: c_int) -> c_int {
    c_int::from((*wlv).sattrs[sign_idx as usize].text[0] != 0)
}

/// Get the fold info for the current line.
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_foldinfo(wlv: *mut WinLineVars) -> FoldInfo {
    (*wlv).foldinfo
}

/// Get the vcol_sbr (showbreak vcol).
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_get_vcol_sbr(wlv: *mut WinLineVars) -> ColnrT {
    (*wlv).vcol_sbr
}

/// Set the vcol_sbr (showbreak vcol).
#[no_mangle]
pub unsafe extern "C" fn rs_wlv_set_vcol_sbr(wlv: *mut WinLineVars, val: ColnrT) {
    (*wlv).vcol_sbr = val;
}

/// Get the window highlight attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_win_hl_attr(wp: WinHandle, hlf: c_int) -> c_int {
    nvim_win_hl_attr(wp, hlf)
}

/// Check if line numbers are enabled for this window.
#[no_mangle]
pub unsafe extern "C" fn rs_win_has_line_numbers(wp: WinHandle) -> c_int {
    c_int::from(nvim_win_get_p_nu(wp) != 0 || nvim_win_get_p_rnu(wp) != 0)
}

/// Check if absolute line numbers are enabled.
#[no_mangle]
pub unsafe extern "C" fn rs_win_has_number(wp: WinHandle) -> c_int {
    c_int::from(nvim_win_get_p_nu(wp) != 0)
}

/// Check if relative line numbers are enabled.
#[no_mangle]
pub unsafe extern "C" fn rs_win_has_relativenumber(wp: WinHandle) -> c_int {
    c_int::from(nvim_win_get_p_rnu(wp) != 0)
}

/// Check if using rightleft mode.
#[no_mangle]
pub unsafe extern "C" fn rs_win_is_rightleft(wp: WinHandle) -> c_int {
    nvim_win_get_p_rl(wp)
}

/// Get the cursor line number.
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_cursor_lnum(wp: WinHandle) -> LinenrT {
    nvim_win_get_cursor_lnum(wp)
}

/// Check if lnum is the cursor line.
#[no_mangle]
pub unsafe extern "C" fn rs_is_cursor_line(wp: WinHandle, lnum: LinenrT) -> c_int {
    c_int::from(nvim_win_get_cursor_lnum(wp) == lnum)
}

/// Get the window's view width.
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_view_width(wp: WinHandle) -> c_int {
    nvim_win_get_view_width(wp)
}

/// Get the window's skip column (for smooth scrolling).
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_skipcol(wp: WinHandle) -> ColnrT {
    nvim_win_get_skipcol(wp)
}

/// Check if there's a skipcol set.
#[no_mangle]
pub unsafe extern "C" fn rs_win_has_skipcol(wp: WinHandle) -> c_int {
    c_int::from(nvim_win_get_skipcol(wp) > 0)
}

/// Get the list characters "precedes" character.
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_lcs_prec(wp: WinHandle) -> ScharT {
    nvim_win_get_lcs_prec(wp)
}

/// Check if list mode is enabled.
#[no_mangle]
pub unsafe extern "C" fn rs_win_has_list(wp: WinHandle) -> c_int {
    c_int::from(nvim_win_get_p_list(wp) != 0)
}

// =============================================================================
// Phase D2: Line rendering state helpers
// =============================================================================

// Additional C function declarations for line rendering
extern "C" {
    // Visual mode state
    fn nvim_get_VIsual_active() -> c_int;

    // Buffer state
    fn nvim_buf_get_line_count(buf: BufHandle) -> LinenrT;

    // Window comparison
    fn nvim_win_is_curwin(wp: WinHandle) -> c_int;

    // Syntax state
    fn syntax_present(wp: WinHandle) -> bool;

    // Marktree size
    fn nvim_buf_get_marktree_n_keys(buf: *mut c_void) -> c_int;

    // Concealcursor option
    fn nvim_win_get_p_cocu(wp: WinHandle) -> *const c_char;
}

/// Check if Visual mode is active and the window is the current window.
///
/// Returns 1 if Visual mode highlighting should be applied for this window.
#[no_mangle]
pub unsafe extern "C" fn rs_should_apply_visual(wp: WinHandle) -> c_int {
    let visual_active = nvim_get_VIsual_active() != 0;
    if !visual_active {
        return 0;
    }

    nvim_win_is_curwin(wp)
}

/// Check if a line is the last line in the buffer.
///
/// Returns 1 if lnum equals buf's line count.
#[no_mangle]
pub unsafe extern "C" fn rs_is_last_line(buf: BufHandle, lnum: LinenrT) -> c_int {
    let line_count = nvim_buf_get_line_count(buf);
    c_int::from(lnum == line_count)
}

/// Check if syntax highlighting is present and not in error state.
///
/// Returns 1 if syntax highlighting should be processed.
#[no_mangle]
pub unsafe extern "C" fn rs_has_syntax(wp: WinHandle) -> c_int {
    c_int::from(syntax_present(wp))
}

/// Calculate the effective highlight attribute for a character.
///
/// Combines base, priority and area attributes according to Neovim rules.
/// This mimics the char_attr calculation in win_line().
#[no_mangle]
pub const extern "C" fn rs_combine_char_attr(
    char_attr_base: c_int,
    char_attr_pri: c_int,
    area_attr: c_int,
    search_attr: c_int,
) -> c_int {
    // Priority: search_attr > area_attr > char_attr_pri > char_attr_base
    if search_attr != 0 {
        return search_attr;
    }
    if area_attr != 0 {
        return area_attr;
    }
    if char_attr_pri != 0 {
        return char_attr_pri;
    }
    char_attr_base
}

/// Clamp a column value to the view width.
#[no_mangle]
pub unsafe extern "C" fn rs_clamp_col_to_view(wp: WinHandle, col: c_int) -> c_int {
    let view_width = nvim_win_get_view_width(wp);
    col.clamp(0, view_width - 1)
}

/// Calculate the number of cells for tab expansion.
///
/// Returns the number of cells a tab character should occupy
/// given the current virtual column.
#[no_mangle]
pub unsafe extern "C" fn rs_tab_cells(wp: WinHandle, vcol: c_int) -> c_int {
    let buf = nvim_win_get_w_buffer(wp);
    // b_p_ts is OptInt (i64); clamp to c_int range (tabstop won't exceed that in practice).
    #[allow(clippy::cast_possible_truncation)]
    let tabstop = nvim_buf_get_p_ts(buf).min(i64::from(c_int::MAX)) as c_int;
    if tabstop == 0 {
        return 1;
    }
    tabstop - (vcol % tabstop)
}

/// Check if the character should be concealed.
///
/// Returns 1 if conceal level >= 2 or (level >= 1 and not on cursor line).
#[no_mangle]
pub unsafe extern "C" fn rs_should_conceal(
    wp: WinHandle,
    conceal_level: c_int,
    on_cursor_line: c_int,
) -> c_int {
    if conceal_level >= 2 {
        return 1;
    }
    if conceal_level >= 1 && on_cursor_line == 0 {
        // Level 1: conceal when not on cursor line (unless concealcursor is set)
        let cocu = nvim_win_get_p_cocu(wp);
        // concealcursor is NUL or empty string when not set
        let has_concealcursor = !cocu.is_null() && *cocu != 0;
        if !has_concealcursor {
            return 1;
        }
    }
    0
}

/// Check if we need to draw the end-of-line character.
///
/// Returns 1 if list mode is on and we have an eol character set.
#[no_mangle]
pub unsafe extern "C" fn rs_need_eol_char(wp: WinHandle) -> c_int {
    if nvim_win_get_p_list(wp) == 0 {
        return 0;
    }
    let eol_char = nvim_win_get_lcs_eol(wp);
    c_int::from(eol_char != 0)
}

extern "C" {
    fn nvim_win_get_lcs_eol(wp: WinHandle) -> ScharT;
}

/// Get the end-of-line character for list mode.
#[no_mangle]
pub unsafe extern "C" fn rs_get_eol_char(wp: WinHandle) -> ScharT {
    nvim_win_get_lcs_eol(wp)
}

/// Check if the line has virtual text that needs drawing.
/// Returns nonzero if the buffer has any extmarks (marktree has entries).
#[no_mangle]
pub unsafe extern "C" fn rs_line_has_virt_text(wp: WinHandle, _lnum: LinenrT) -> c_int {
    let buf = nvim_win_get_buffer(wp);
    if buf.is_null() {
        return 0;
    }
    nvim_buf_get_marktree_n_keys(buf)
}

/// Calculate the column position for cursor column highlight.
///
/// Returns the screen column for cursorcolumn highlighting, or -1 if not applicable.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_column_pos(wp: WinHandle) -> c_int {
    if nvim_win_get_p_cuc(wp) == 0 {
        return -1;
    }

    // Only draw cursor column when it's visible in the window
    let virtcol = nvim_win_get_virtcol(wp);
    let view_width = nvim_win_get_view_width(wp);

    if virtcol >= view_width {
        return -1;
    }

    virtcol
}

/// Validate screen row is within window bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_screen_row(wp: WinHandle, row: c_int) -> c_int {
    let view_height = nvim_win_get_view_height(wp);
    c_int::from(row >= 0 && row < view_height)
}

extern "C" {
    fn nvim_win_get_view_height(wp: WinHandle) -> c_int;
}

// ============================================================================
// Phase 1: draw_statuscol migration
// ============================================================================

/// Rust implementation of draw_statuscol.
///
/// Builds and draws the 'statuscolumn' string for line "lnum" in window "wp".
/// `stcp` is an opaque pointer to `statuscol_T`.
/// `wlv` is an opaque pointer to `winlinevars_T`.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_draw_statuscol(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    virtnum: c_int,
    col_rows: c_int,
    stcp: *mut c_void,
) {
    // Adjust lnum for filler lines belonging to the line above.
    let raw_lnum = (*wlv).lnum;
    let n_virt_lines = (*wlv).n_virt_lines;
    let n_virt_below = (*wlv).n_virt_below;
    let filler_todo = (*wlv).filler_todo;

    let adjust = c_int::from((n_virt_lines - filler_todo) < n_virt_below);
    let lnum = raw_lnum - adjust;

    // Compute relnum: absolute relative cursor line number for first/last rows.
    let filler_lines = (*wlv).filler_lines;
    let is_first_row =
        virtnum == -filler_lines || virtnum == 0 || virtnum == (n_virt_below - filler_lines);
    let relnum: LinenrT = if is_first_row {
        nvim_get_cursor_rel_lnum(wp, lnum).abs()
    } else {
        -1
    };

    // Buffer for building the statuscol string.
    let mut buf = [0i8; MAXPATHL];

    // When the buffer's line count has changed, rebuild for the widest possible line.
    let statuscol_line_count = nvim_win_get_statuscol_line_count(wp);
    let nrwidth_line_count = nvim_win_get_nrwidth_line_count(wp);
    if statuscol_line_count != nrwidth_line_count {
        nvim_win_set_statuscol_line_count(wp, nrwidth_line_count);
        nvim_fold_set_vim_var_nr(VV_VIRTNUM, 0);
        let width = nvim_build_statuscol_str(
            wp,
            nrwidth_line_count,
            nrwidth_line_count,
            buf.as_mut_ptr(),
            stcp,
        );
        let stcp_width = nvim_stcp_get_width(stcp);
        if width > stcp_width {
            let addwidth = (width - stcp_width).min(MAX_STCWIDTH - stcp_width);
            let new_nrwidth = nvim_win_get_nrwidth(wp) + addwidth;
            nvim_win_set_nrwidth(wp, new_nrwidth);
            nvim_win_set_nrwidth_width(wp, new_nrwidth);
            if col_rows > 0 {
                // Only column is being redrawn; need to redraw text too.
                nvim_win_set_redr_statuscol(wp, true);
                return;
            }
            nvim_stcp_set_width(stcp, stcp_width + addwidth);
            nvim_win_clear_valid_bits(wp, VALID_WCOL);
        }
    }

    nvim_fold_set_vim_var_nr(VV_VIRTNUM, i64::from(virtnum));

    let stcp_width = nvim_stcp_get_width(stcp);
    let width = nvim_build_statuscol_str(wp, lnum, relnum, buf.as_mut_ptr(), stcp);

    // Force a redraw on error or truncation.
    let p_stc = nvim_win_get_p_stc(wp);
    if *p_stc == 0 || (width > stcp_width && stcp_width < MAX_STCWIDTH) {
        if *p_stc == 0 {
            // 'statuscolumn' reset due to error.
            nvim_win_set_nrwidth_line_count(wp, 0);
            let nu = nvim_win_get_p_nu(wp);
            let rnu = nvim_win_get_p_rnu(wp);
            let new_nrwidth = c_int::from(nu != 0 || rnu != 0) * rs_number_width(wp);
            nvim_win_set_nrwidth(wp, new_nrwidth);
        } else {
            // Avoid truncating 'statuscolumn'.
            let addwidth = (width - stcp_width).min(MAX_STCWIDTH - stcp_width);
            let new_nrwidth = nvim_win_get_nrwidth(wp) + addwidth;
            nvim_win_set_nrwidth(wp, new_nrwidth);
            nvim_win_set_nrwidth_width(wp, new_nrwidth);
        }
        nvim_win_set_redr_statuscol(wp, true);
        return;
    }

    // Draw each highlighted segment.
    let buf_len = {
        let mut i = 0usize;
        while buf[i] != 0 {
            i += 1;
        }
        i
    };

    let scl_attr = nvim_win_hl_attr(
        wp,
        if use_cursor_line_highlight_impl(wp, raw_lnum) {
            HLF_CLS
        } else {
            HLF_SC
        },
    );
    let num_attr = get_line_number_attr_impl(wp, wlv);
    let mut cur_attr = num_attr;
    let mut fold_vcol: *const ColnrT = std::ptr::null();

    let mut p = buf.as_mut_ptr();
    let buf_end = buf.as_mut_ptr().add(buf_len);
    let mut sp = nvim_stcp_get_hlrec(stcp);

    // Iterate over highlight records (terminated by start == NULL).
    loop {
        let sp_start = nvim_hlrec_get_start(sp);
        if sp_start.is_null() {
            break;
        }
        // Draw text from p up to sp->start.
        let textlen = sp_start.offset_from(p);
        let mut transbuf = [0i8; MAXPATHL];
        let translen = nvim_transstr_buf(p, textlen, transbuf.as_mut_ptr(), MAXPATHL);
        draw_col_buf_impl(
            wp,
            wlv,
            transbuf.as_ptr(),
            translen,
            cur_attr,
            fold_vcol,
            false,
        );

        let item = nvim_hlrec_get_item(sp);
        let attr = if item == STL_SIGNCOL {
            scl_attr
        } else if item == STL_FOLDCOL {
            0
        } else {
            num_attr
        };
        let userhl = nvim_hlrec_get_userhl(sp);
        let userhl_attr = if userhl < 0 { syn_id2attr(-userhl) } else { 0 };
        cur_attr = hl_combine_attr(attr, userhl_attr);

        fold_vcol = if item == STL_FOLDCOL {
            nvim_stcp_get_fold_vcol(stcp)
        } else {
            std::ptr::null()
        };

        p = sp_start;
        sp = nvim_hlrec_next(sp);
    }

    // Draw the final segment.
    let textlen = buf_end.offset_from(p);
    let mut transbuf = [0i8; MAXPATHL];
    let translen = nvim_transstr_buf(p, textlen, transbuf.as_mut_ptr(), MAXPATHL);
    draw_col_buf_impl(
        wp,
        wlv,
        transbuf.as_ptr(),
        translen,
        cur_attr,
        fold_vcol,
        false,
    );
    draw_col_fill_impl(
        wlv,
        rs_schar_from_char(c_int::from(b' ')),
        stcp_width - width,
        cur_attr,
    );
}

// ============================================================================
// Phase 1: Scratch buffer (get_extra_buf / drawline_free_all_mem)
// ============================================================================

use std::sync::Mutex;

static EXTRA_BUF: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// Rust replacement for get_extra_buf: returns a pointer to a scratch buffer
/// of at least `size` bytes. The pointer is valid until the next call.
///
/// # Safety
/// Caller must not use the pointer after the mutex is released or after the
/// next call to this function from any thread.
unsafe fn get_extra_buf_impl(size: usize) -> *mut c_char {
    let min = if size < 64 { 64 } else { size };
    let mut guard = EXTRA_BUF.lock().unwrap();
    if guard.len() < min {
        guard.resize(min, 0);
    }
    guard.as_mut_ptr().cast::<c_char>()
}

/// Free the scratch buffer (EXITFREE).
///
/// # Panics
///
/// Panics if the internal mutex is poisoned.
#[no_mangle]
pub unsafe extern "C" fn drawline_free_all_mem() {
    let mut guard = EXTRA_BUF.lock().unwrap();
    *guard = Vec::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foldinfo_layout() {
        assert!(std::mem::size_of::<FoldInfo>() > 0);
    }

    #[test]
    fn test_winlinevars_layout() {
        // WinLineVars must be non-zero size
        assert!(std::mem::size_of::<WinLineVars>() > 0);
    }

    #[test]
    fn test_highlight_group_constants() {
        // Verify highlight group constants match C definitions
        assert_eq!(HLF_AT, 4); // NonText
        assert_eq!(HLF_N, 12); // LineNr
        assert_eq!(HLF_LNA, 13); // LineNrAbove
        assert_eq!(HLF_LNB, 14); // LineNrBelow
        assert_eq!(HLF_CLN, 15); // CursorLineNr
        assert_eq!(HLF_CLS, 16); // CursorLineSign
        assert_eq!(HLF_CLF, 17); // CursorLineFold
        assert_eq!(HLF_FC, 29); // FoldColumn
        assert_eq!(HLF_DED, 32); // DiffDelete
        assert_eq!(HLF_SC, 35); // SignColumn
    }

    #[test]
    fn test_cursorlineopt_flags() {
        // Verify cursorlineopt flags match C definitions
        assert_eq!(K_OPT_CULOPT_FLAG_LINE, 0x01);
        assert_eq!(K_OPT_CULOPT_FLAG_SCREENLINE, 0x02);
        assert_eq!(K_OPT_CULOPT_FLAG_NUMBER, 0x04);
        // Flags should be distinct powers of 2
        let flags = [
            K_OPT_CULOPT_FLAG_LINE,
            K_OPT_CULOPT_FLAG_SCREENLINE,
            K_OPT_CULOPT_FLAG_NUMBER,
        ];
        for (i, &a) in flags.iter().enumerate() {
            for (j, &b) in flags.iter().enumerate() {
                if i != j {
                    assert_eq!(a & b, 0, "flags {i} and {j} should not overlap");
                }
            }
        }
    }

    #[test]
    fn test_sign_width_constant() {
        assert_eq!(SIGN_WIDTH, 2);
    }

    #[test]
    fn test_foldinfo_default() {
        let fi = FoldInfo {
            fi_lnum: 0,
            fi_level: 0,
            fi_low_level: 0,
            fi_lines: 0,
        };
        assert_eq!(fi.fi_level, 0);
        assert_eq!(fi.fi_lines, 0);
    }

    #[test]
    fn test_kvecc_empty() {
        let v: KVec<u8> = KVec::empty();
        assert_eq!(v.size, 0);
        assert_eq!(v.capacity, 0);
        assert!(v.items.is_null());
    }

    #[test]
    fn test_type_alias_sizes() {
        // Verify type alias sizes match expected C types
        assert_eq!(std::mem::size_of::<ScharT>(), 4);
        assert_eq!(std::mem::size_of::<LinenrT>(), 4);
        assert_eq!(std::mem::size_of::<ColnrT>(), 4);
    }

    #[test]
    fn test_highlight_groups_sequential() {
        // LineNr groups are sequential
        assert_eq!(HLF_LNA, HLF_N + 1);
        assert_eq!(HLF_LNB, HLF_N + 2);
        // CursorLine groups are sequential
        assert_eq!(HLF_CLS, HLF_CLN + 1);
        assert_eq!(HLF_CLF, HLF_CLN + 2);
    }
}
