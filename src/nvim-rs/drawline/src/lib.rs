//! Line drawing functions for Neovim
//!
//! This crate provides Rust implementations of line drawing functions
//! from `src/nvim/drawline.c`, focusing on column rendering and helpers.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::if_not_else)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::missing_safety_doc)]
#![allow(dead_code)]

use std::ffi::c_int;
use std::ffi::c_void;

use nvim_decoration::{
    decor_range_draw_col, decor_range_has_virt_pos, decor_range_kind, decor_range_set_draw_col,
    decor_range_start_col, decor_range_start_row, decor_range_ui_mark_id, decor_range_ui_ns_id,
    decor_range_virt_pos_kind, decor_range_virt_text, decor_state_current_end, decor_state_eol_col,
    decor_state_get_active_range, decor_state_get_eol_right_width, decor_state_get_range_by_idx,
    decor_state_row, decor_state_set_eol_col, get_decor_state, next_virt_text_chunk, virt_text_col,
    virt_text_flags, virt_text_get_virt_text, virt_text_hl_mode, virt_text_pos, virt_text_width,
    win_extmark_push, DecorKind, DecorRangeHandle, DecorVirtTextHandle, VirtTextHandle,
    VirtTextPos,
};
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
pub const HLF_CUL: c_int = 56; // CursorLine
pub const HLF_MC: c_int = 57; // ColorColumn
pub const HLF_QFL: c_int = 58; // QuickFixLine
                               // Highlight groups used by win_line init block:
pub const HLF_I: c_int = 7; // IncSearch
pub const HLF_V: c_int = 24; // Visual
pub const HLF_ADD: c_int = 30; // DiffAdd
pub const HLF_CHD: c_int = 31; // DiffChange
pub const HLF_TXD: c_int = 33; // DiffText (changed text)
pub const HLF_TXA: c_int = 34; // DiffText (added text)
pub const HLF_CONCEAL: c_int = 36; // Conceal
pub const MAXCOL: c_int = i32::MAX; // MAXCOL from pos_defs.h

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

    // Additional WLV accessors for handle_inline_virtual_text
    #[link_name = "decor_init_draw_col"]
    fn nvim_decor_init_draw_col(win_col: c_int, hidden: bool, item: DecorRangeHandle);

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

    // Phase 2: decor_providers_setup / invoke_range_next FFI
    fn nvim_win_ml_get_buf(wp: WinHandle, lnum: LinenrT) -> *const c_char;
    fn nvim_win_ml_get_buf_len(wp: WinHandle, lnum: LinenrT) -> ColnrT;
    fn decor_providers_invoke_line(wp: WinHandle, row: c_int);
    fn decor_providers_invoke_range(
        wp: WinHandle,
        start_row: c_int,
        start_col: c_int,
        end_row: c_int,
        end_col: c_int,
    );
    fn validate_virtcol(wp: WinHandle);
    fn mb_off_next(base: *const c_char, p: *const c_char) -> c_int;

    // Phase 2 (win_line migration): additional FFI declarations

    // Syntax highlighting
    fn syntax_start(wp: WinHandle, lnum: LinenrT);
    fn get_syntax_attr(col: ColnrT, can_spell: *mut bool, keep_state: bool) -> c_int;
    fn get_syntax_info(seqnrp: *mut c_int) -> c_int;
    fn syn_get_sub_char() -> c_int;

    // Spell checking
    fn spell_move_to(
        wp: WinHandle,
        dir: c_int,
        behaviour: c_int,
        curline: bool,
        attrp: *mut c_int,
    ) -> usize;
    fn spell_to_word_end(start: *const c_char, win: WinHandle) -> *const c_char;
    fn spell_check(
        wp: WinHandle,
        ptr: *mut c_char,
        attrp: *mut c_int,
        capcol: *mut c_int,
        docount: bool,
    ) -> usize;
    fn spell_cat_line(buf: *mut c_char, line: *const c_char, maxlen: c_int);
    fn check_need_cap(wp: WinHandle, lnum: LinenrT, col: ColnrT) -> bool;

    // Search highlighting (match_T passed as opaque *mut c_void)
    fn prepare_search_hl_line(
        wp: WinHandle,
        lnum: LinenrT,
        mincol: ColnrT,
        line: *mut *const c_char,
        search_hl: *mut c_void,
        search_attr: *mut c_int,
        search_attr_from_match: *mut bool,
    ) -> bool;
    fn update_search_hl(
        wp: WinHandle,
        lnum: LinenrT,
        col: ColnrT,
        line: *mut *const c_char,
        search_hl: *mut c_void,
        has_match_conc: *mut c_int,
        match_conc: *mut c_int,
        lcs_eol_todo: bool,
        on_last_col: *mut bool,
        search_attr_from_match: *mut bool,
    ) -> c_int;
    fn get_prevcol_hl_flag(wp: WinHandle, search_hl: *mut c_void, curcol: ColnrT) -> bool;
    fn get_search_match_hl(
        wp: WinHandle,
        search_hl: *mut c_void,
        col: ColnrT,
        char_attr: *mut c_int,
    );

    // Decoration
    fn decor_redraw_line(wp: WinHandle, row: c_int, state: *mut c_void);
    fn decor_redraw_col_impl(
        wp: WinHandle,
        col: ColnrT,
        win_col: c_int,
        hidden: bool,
        state: *mut c_void,
    ) -> c_int;
    fn decor_redraw_eol(
        wp: WinHandle,
        state: *mut c_void,
        eol_attr: *mut c_int,
        eol_col: c_int,
    ) -> bool;
    fn decor_has_more_decorations(state: *mut c_void, row: c_int) -> bool;
    fn decor_recheck_draw_col(win_col: c_int, hidden: bool, state: *mut c_void);
    fn decor_virt_lines(
        wp: WinHandle,
        start_row: c_int,
        end_row: c_int,
        num_below: *mut c_int,
        lines: *mut c_void,
        apply_folds: bool,
    ) -> c_int;

    // Grid
    fn win_draw_end(
        wp: WinHandle,
        c1: ScharT,
        draw_margin: bool,
        startrow: c_int,
        endrow: c_int,
        hl: c_int,
    );
    fn set_empty_rows(wp: WinHandle, used: c_int);
    fn win_bg_attr(wp: WinHandle) -> c_int;

    // Multibyte / charset
    fn transchar_buf(buf: *mut c_void, c: c_int) -> *mut c_char;
    fn transchar_hex(buf: *mut c_char, c: c_int) -> usize;
    fn rl_mirror_ascii(str: *mut c_char, end: *mut c_char);
    fn byte2cells(b: c_int) -> c_int;
    fn clear_virttext(text: *mut c_void);
    fn skipwhite(p: *const c_char) -> *const c_char;
    fn plines_win(wp: WinHandle, lnum: LinenrT, limit_winheight: bool) -> c_int;
    fn utf_ptr2CharInfo_impl(p: *const u8, len: usize) -> i32;
    // utfc_next_impl is implemented in Rust (mbyte crate) - call via utfc_ptr2schar wrappers
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn mb_ptr2char_adv(pp: *mut *const c_char) -> c_int;
    fn schar_get_adv(buf_out: *mut *mut c_char, sc: ScharT) -> usize;
    fn schar_len(sc: ScharT) -> usize;
    fn schar_cells(sc: ScharT) -> c_int;
    fn schar_get_first_codepoint(sc: ScharT) -> c_int;
    fn ml_get_buf(buf: *mut c_void, lnum: LinenrT) -> *const c_char;

    fn nvim_get_state() -> c_int;
    fn nvim_win_is_curwin(wp: WinHandle) -> c_int;
}

/// Opaque handle to buffer (buf_T).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    // Buffer handle for win
    fn nvim_win_get_w_buffer(wp: WinHandle) -> BufHandle;

    // State and quickfix functions for apply_cursorline_highlight
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
    let cul_attr = nvim_win_hl_attr(wp, HLF_CUL);
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
        let hlf_mc = HLF_MC;

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

    let state = get_decor_state();
    let count = (*state).ranges_i.size as c_int;
    let cur_end = decor_state_current_end(state);
    let fut_beg = (*state).future_begin;
    let state_row = decor_state_row(state);

    // Check both ranges: [0, cur_end) and [fut_beg, count)
    let beg_pos = [0, fut_beg];
    let end_pos = [cur_end, count];

    for pos_i in 0..2 {
        for i in beg_pos[pos_i]..end_pos[pos_i] {
            let range = decor_state_get_range_by_idx(state, i);
            if range.is_null() {
                continue;
            }

            let start_row = decor_range_start_row(range);
            let kind = decor_range_kind(range);
            let draw_col = decor_range_draw_col(range);
            let start_col = decor_range_start_col(range);

            if start_row != state_row || kind != Some(DecorKind::VirtText) {
                continue;
            }

            // Get virt text position and width
            let vt = decor_range_virt_text(range);
            if vt.is_null() {
                continue;
            }

            let vt_pos = virt_text_pos(vt);
            let vt_width = virt_text_width(vt);

            if vt_pos != Some(VirtTextPos::Inline) || vt_width == 0 {
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
            match next_virt_text_chunk(VirtTextHandle(vt), &mut virt_pos, &mut virt_attr) {
                Some(p) => virt_str = p,
                None => break,
            }
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
    let state = get_decor_state();
    let max_col = nvim_win_get_view_width(wp);
    let mut right_pos = max_col;
    let eol_col = decor_state_eol_col(state);
    let do_eol = eol_col > -1;
    let row = decor_state_row(state);
    let current_end = decor_state_current_end(state);

    let mut total_width_eol_right = 0;

    for i in 0..current_end {
        let item = decor_state_get_active_range(state, i);
        if item.is_null() {
            continue;
        }

        // Skip if not on current row or not a virtual position
        if decor_range_start_row(item) != row || !decor_range_has_virt_pos(item) {
            continue;
        }

        let kind = decor_range_kind(item);
        let vt: DecorVirtTextHandle = if kind == Some(DecorKind::VirtText) {
            decor_range_virt_text(item)
        } else {
            std::ptr::null_mut()
        };

        let draw_col = decor_range_draw_col(item);
        if decor_range_has_virt_pos(item) && draw_col == -1 {
            let mut updated = true;
            let pos = decor_range_virt_pos_kind(item);

            if do_eol && pos == Some(VirtTextPos::EndOfLineRightAlign) {
                let mut eol_offset = 0;
                if total_width_eol_right == 0 {
                    // Calculate total width of right-aligned EOL virtual text
                    total_width_eol_right = decor_state_get_eol_right_width(state, i);

                    let current_eol_col = decor_state_eol_col(state);
                    if total_width_eol_right <= right_pos - current_eol_col {
                        eol_offset = right_pos - total_width_eol_right - current_eol_col;
                    }
                }

                let new_draw_col = decor_state_eol_col(state) + eol_offset;
                decor_range_set_draw_col(item, new_draw_col);
            } else if pos == Some(VirtTextPos::RightAlign) {
                if !vt.is_null() {
                    right_pos -= virt_text_width(vt);
                }
                decor_range_set_draw_col(item, right_pos);
            } else if pos == Some(VirtTextPos::EndOfLine) && do_eol {
                decor_range_set_draw_col(item, decor_state_eol_col(state));
            } else if pos == Some(VirtTextPos::WinCol) {
                if !vt.is_null() {
                    let vt_col = virt_text_col(vt);
                    let new_col = std::cmp::max(col_off + vt_col, 0);
                    decor_range_set_draw_col(item, new_col);
                }
            } else {
                updated = false;
            }

            if updated {
                let new_draw_col = decor_range_draw_col(item);
                if new_draw_col < 0 || new_draw_col >= max_col {
                    // Out of window, don't draw at all
                    decor_range_set_draw_col(item, c_int::MIN);
                }
            }
        }

        let draw_col = decor_range_draw_col(item);
        if draw_col < 0 {
            continue;
        }

        // Handle UIWatched marks
        if kind == Some(DecorKind::UIWatched) {
            let ns_id = decor_range_ui_ns_id(item);
            let mark_id = u64::from(decor_range_ui_mark_id(item));
            win_extmark_push(ns_id, mark_id, win_row, draw_col);
        }

        // Render virtual text
        if !vt.is_null() {
            let vcol = draw_col - col_off;
            let virt_text = virt_text_get_virt_text(vt);
            let hl_mode = virt_text_hl_mode(vt).map_or(0, |m| m as c_int);

            let col =
                draw_virt_text_item_impl(buf, draw_col, virt_text.0, hl_mode, max_col, vcol, 0);

            let vt_pos = virt_text_pos(vt);
            if do_eol
                && (vt_pos == Some(VirtTextPos::EndOfLine)
                    || vt_pos == Some(VirtTextPos::EndOfLineRightAlign))
            {
                decor_state_set_eol_col(state, col + 1);
            }

            if !end_col.is_null() {
                *end_col = std::cmp::max(*end_col, col);
            }
        }

        // Deactivate unless it should repeat on linebreak
        let flags = if !vt.is_null() {
            virt_text_flags(vt)
        } else {
            0
        };
        if vt.is_null() || (flags & K_VT_REPEAT_LINEBREAK) == 0 {
            decor_range_set_draw_col(item, c_int::MIN);
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
    let hlf_mc = HLF_MC;

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

            let state = get_decor_state();
            let end = decor_state_current_end(state);
            let row = decor_state_row(state);
            let off = (*wlv).off;

            let mut found = false;
            for i in 0..end {
                let item = decor_state_get_active_range(state, i);
                if item.is_null() {
                    continue;
                }

                let draw_col = decor_range_draw_col(item);
                if draw_col == -3 {
                    // No more inline virtual text before this non-inline virtual text item,
                    // so its position can be decided now.
                    nvim_decor_init_draw_col(off, selected, item);
                }

                let start_row = decor_range_start_row(item);
                let kind = decor_range_kind(item);
                let vt = decor_range_virt_text(item);

                if start_row != row || kind != Some(DecorKind::VirtText) || vt.is_null() {
                    continue;
                }

                let pos = virt_text_pos(vt);
                let width = virt_text_width(vt);

                if pos != Some(VirtTextPos::Inline) || width == 0 {
                    continue;
                }

                let draw_col = decor_range_draw_col(item);
                let start_col = decor_range_start_col(item);

                if draw_col >= -1 && start_col == v as c_int {
                    // Found matching inline virtual text -- access vt->data.virt_text directly
                    let virt_inline_data: *mut KVec<VirtTextChunkC> =
                        std::ptr::addr_of_mut!((*vt).data.virt_text).cast();
                    let hl_mode = c_int::from((*vt).hl_mode);
                    (*wlv).virt_inline = if virt_inline_data.is_null() {
                        KVec::empty()
                    } else {
                        *virt_inline_data
                    };
                    (*wlv).virt_inline_hl_mode = hl_mode;
                    decor_range_set_draw_col(item, INT_MIN);
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
            let virt_inline_ptr = VirtTextHandle(
                std::ptr::from_mut::<KVec<VirtTextChunkC>>(&mut (*wlv).virt_inline)
                    .cast::<c_void>(),
            );
            let mut pos = (*wlv).virt_inline_i;
            let mut attr: c_int = 0;

            let Some(text) = next_virt_text_chunk(virt_inline_ptr, &mut pos, &mut attr) else {
                (*wlv).virt_inline_i = pos;
                continue;
            };
            (*wlv).virt_inline_i = pos;

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
    static mut VIsual_active: bool;

    // Buffer state
    fn nvim_buf_get_line_count(buf: BufHandle) -> LinenrT;

    // Window comparison

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
    let visual_active = VIsual_active;
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

/// Exported Rust wrapper for `get_extra_buf` (called from C `win_line`).
///
/// Returns a pointer to a scratch buffer of at least `size` bytes.
///
/// # Safety
/// Caller must not use the pointer after the next call to this function from any thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_extra_buf(size: usize) -> *mut c_char {
    get_extra_buf_impl(size)
}

// ============================================================================
// Phase 2: decor_providers_setup and invoke_range_next migration
// ============================================================================

/// Rust equivalent of invoke_range_next.
///
/// Invokes `decor_providers_invoke_range` for the next segment starting at `begin_col`
/// and spanning `col_off` bytes. Returns the new begin column, or `i32::MAX`.
unsafe fn invoke_range_next_impl(
    wp: WinHandle,
    lnum: LinenrT,
    begin_col: ColnrT,
    col_off: ColnrT,
) -> c_int {
    let line = nvim_win_ml_get_buf(wp, lnum);
    let line_len = nvim_win_ml_get_buf_len(wp, lnum);
    let col_off = col_off.max(1);

    if col_off <= line_len - begin_col {
        let mut end_col = begin_col + col_off;
        end_col += mb_off_next(line, line.offset(end_col as isize));
        decor_providers_invoke_range(wp, lnum - 1, begin_col, lnum - 1, end_col);
        validate_virtcol(wp);
        end_col
    } else {
        decor_providers_invoke_range(wp, lnum - 1, begin_col, lnum, 0);
        validate_virtcol(wp);
        c_int::MAX
    }
}

/// Exported Rust wrapper for `invoke_range_next` (called from C `win_line`).
#[no_mangle]
pub unsafe extern "C" fn rs_invoke_range_next(
    wp: WinHandle,
    lnum: LinenrT,
    begin_col: ColnrT,
    col_off: ColnrT,
) -> c_int {
    invoke_range_next_impl(wp, lnum, begin_col, col_off)
}

/// Rust equivalent of decor_providers_setup.
///
/// Approximates the number of bytes that will be drawn and sets up decoration
/// provider ranges for the current line. Returns the new begin column.
unsafe fn decor_providers_setup_impl(
    rows_to_draw: c_int,
    draw_from_line_start: bool,
    lnum: LinenrT,
    col: ColnrT,
    wp: WinHandle,
) -> c_int {
    let rem_vcols = if nvim_win_get_p_wrap(wp) != 0 {
        let view_width = nvim_win_get_view_width(wp);
        let width = view_width - rs_win_col_off(wp);
        let width2 = width + rs_win_col_off2(wp);
        let first_row_width = if draw_from_line_start { width } else { width2 };
        first_row_width + (rows_to_draw - 1) * width2
    } else {
        nvim_win_get_view_height(wp) - rs_win_col_off(wp)
    };

    // Call it here since we need to invalidate the line pointer anyway.
    decor_providers_invoke_line(wp, lnum - 1);
    validate_virtcol(wp);

    invoke_range_next_impl(wp, lnum, col, rem_vcols + 1)
}

/// Exported Rust wrapper for `decor_providers_setup` (called from C `win_line`).
#[no_mangle]
pub unsafe extern "C" fn rs_decor_providers_setup(
    rows_to_draw: c_int,
    draw_from_line_start: bool,
    lnum: LinenrT,
    col: ColnrT,
    wp: WinHandle,
) -> c_int {
    decor_providers_setup_impl(rows_to_draw, draw_from_line_start, lnum, col, wp)
}

// =============================================================================
// Phase W1: win_line_init migration
// =============================================================================

/// pos_T repr(C) -- mirrors the C pos_T struct (lnum, col, coladd).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PosT {
    pub lnum: LinenrT,
    pub col: ColnrT,
    pub coladd: ColnrT,
}

/// Repr(C) mirror of C diffline_T.
/// Must match: { diffline_change_T *changes; int num_changes; int bufidx; int lineoff; }
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DifflineStateC {
    pub changes: *mut c_void,
    pub num_changes: c_int,
    pub bufidx: c_int,
    pub lineoff: c_int,
}

impl Default for DifflineStateC {
    fn default() -> Self {
        Self {
            changes: std::ptr::null_mut(),
            num_changes: 0,
            bufidx: 0,
            lineoff: 0,
        }
    }
}

/// Local variable state for win_line() that must persist across the main loop.
///
/// This holds all the non-WinLineVars local variables of win_line() needed
/// after the initialization block. The struct is repr(C) so that C code can
/// take a pointer to it on the stack.
#[repr(C)]
pub struct WinLineState {
    pub vcol_prev: ColnrT,
    pub fromcol_prev: c_int,
    pub noinvcur: bool,
    pub lnum_in_visual_area: bool,
    pub char_attr_pri: c_int,
    pub char_attr_base: c_int,
    pub area_highlighting: bool,
    pub vi_attr: c_int,
    pub area_attr: c_int,
    pub search_attr: c_int,
    pub vcol_save_attr: c_int,
    pub decor_attr: c_int,
    pub has_syntax: bool,
    pub folded_attr: c_int,
    pub eol_hl_off: c_int,
    /// nextline[SPWORDLEN * 2]: text with start of the next line (SPWORDLEN=150)
    pub nextline: [c_char; 300],
    pub nextlinecol: c_int,
    pub nextline_idx: c_int,
    pub spell_attr: c_int,
    pub word_end: c_int,
    pub cur_checked_col: c_int,
    pub extra_check: bool,
    pub multi_attr: c_int,
    pub mb_l: c_int,
    pub mb_c: c_int,
    pub mb_schar: ScharT,
    pub change_start: c_int,
    pub change_end: c_int,
    pub in_multispace: bool,
    pub multispace_pos: c_int,
    pub n_extra_next: c_int,
    pub extra_attr_next: c_int,
    pub search_attr_from_match: bool,
    pub has_decor: bool,
    pub saved_search_attr: c_int,
    pub saved_area_attr: c_int,
    pub saved_decor_attr: c_int,
    pub saved_search_attr_from_match: bool,
    pub win_col_offset: c_int,
    pub area_active: bool,
    pub decor_need_recheck: bool,
    /// buf_fold[FOLD_TEXT_LEN=51] - buffer for fold text
    pub buf_fold: [c_char; 51],
    pub fold_vt: KVec<VirtTextChunkC>,
    pub foldtext_free: *mut c_char,
    pub cul_screenline: bool,
    pub left_curline_col: c_int,
    pub right_curline_col: c_int,
    pub match_conc: c_int,
    pub on_last_col: bool,
    pub syntax_flags: c_int,
    pub syntax_seqnr: c_int,
    pub prev_syntax_id: c_int,
    pub conceal_attr: c_int,
    pub is_concealing: bool,
    pub did_wcol: bool,
    pub bg_attr: c_int,
    pub draw_text: bool,
    pub has_fold: bool,
    pub has_foldtext: bool,
    pub is_wrapped: bool,
    pub in_curline: bool,
    pub view_width: c_int,
    pub view_height: c_int,
    pub line_attr_save: c_int,
    pub line_attr_lowprio_save: c_int,
    pub check_decor_providers: bool,
    pub decor_provider_end_col: c_int,
    pub linestatus: c_int,
    pub change_index: c_int,
    /// diffline_T struct containing change info for inline diff.
    /// The `changes` pointer inside points to long-lived C data.
    pub line_changes: DifflineStateC,
    pub virt_lines: KVec<u8>,
    pub saved_attr2: c_int,
    pub n_attr3: c_int,
    pub saved_attr3: c_int,
}

impl Default for WinLineState {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// Additional FFI bindings for win_line_init
extern "C" {
    fn nvim_get_VIsual_mode() -> c_int;
    fn nvim_get_VIsual_lnum() -> LinenrT;
    fn nvim_get_VIsual_col() -> ColnrT;
    fn nvim_get_VIsual_coladd() -> ColnrT;
    fn nvim_get_highlight_match() -> c_int;
    fn nvim_get_search_match_lines() -> c_int;
    fn nvim_get_search_match_endcol() -> c_int;
    fn nvim_get_p_sel_first() -> c_char;
    fn nvim_getvvcol(
        wp: WinHandle,
        pos: *mut PosT,
        scol: *mut ColnrT,
        ccol: *mut ColnrT,
        ecol: *mut ColnrT,
    );
    fn nvim_getvcol(
        wp: WinHandle,
        pos: *mut PosT,
        scol: *mut ColnrT,
        ccol: *mut ColnrT,
        ecol: *mut ColnrT,
    );
    fn nvim_cursor_is_block_during_visual(exclusive: c_int) -> c_int;
    fn nvim_gchar_pos_lnum_col(lnum: LinenrT, col: ColnrT, coladd: ColnrT) -> c_int;
    fn nvim_win_get_old_cursor_fcol(wp: WinHandle) -> ColnrT;
    fn nvim_win_get_old_cursor_lcol(wp: WinHandle) -> ColnrT;
    fn nvim_get_cmdwin_win() -> WinHandle;
    fn nvim_buf_get_terminal(buf: BufHandle) -> c_int;
    // rs_diff_check_with_linestatus, rs_diff_find_change, rs_diff_change_parse
    // are already exported by the diff crate; call them via extern.
    fn rs_diff_check_with_linestatus(wp: WinHandle, lnum: LinenrT, linestatus: *mut c_int)
        -> c_int;
    fn rs_diff_find_change(wp: WinHandle, lnum: LinenrT, diffline: *mut c_void) -> bool;
    fn rs_diff_change_parse(
        diffline: *mut c_void,
        change: *mut c_void,
        change_start: *mut c_int,
        change_end: *mut c_int,
    ) -> bool;
    // nvim_win_get_cursor_col and nvim_win_get_cursor_coladd from win_struct.rs:
    fn nvim_win_get_cursor_col(wp: WinHandle) -> ColnrT;
    fn nvim_win_get_cursor_coladd(wp: WinHandle) -> ColnrT;
    // nvim_win_ml_get_buf already declared above at line ~392
    // nvim_win_get_cursorline, nvim_win_get_p_cul, nvim_win_get_p_culopt_flags already declared above

    // diff accessor: get nth change from diffline_T
    fn nvim_diff_diffline_get_change(dl: *mut c_void, i: c_int) -> *mut c_void;
    fn nvim_diffchange_get_start(change: *mut c_void, idx: c_int) -> ColnrT;

    // insexpand functions (exported from insexpand crate)
    fn rs_ins_compl_win_active(wp: WinHandle) -> c_int;
    fn rs_ins_compl_lnum_in_range(lnum: c_int) -> c_int;
    fn rs_ins_compl_col_range_attr(lnum: c_int, col: c_int) -> c_int;
}

// use_cursor_line_highlight: already exported from this crate, but also needed as FFI call
extern "C" {
    fn use_cursor_line_highlight(wp: WinHandle, lnum: LinenrT) -> bool;
}

/// Initialize the line state variables for win_line().
///
/// This implements lines 232-610 of the C win_line() function.
/// Returns the initialized WinLineState via out-pointer.
///
/// # Safety
/// All pointers must be valid. `spv` must point to a valid spellvars_T.
#[no_mangle]
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_win_line_init(
    wp: WinHandle,
    lnum: LinenrT,
    _startrow: c_int,
    col_rows: c_int,
    concealed: bool,
    spv: *mut c_void,
    foldinfo: FoldInfo,
    wlv: *mut WinLineVars,
    state_out: *mut WinLineState,
) {
    let state = &mut *state_out;

    // Initialize WinLineVars fields that win_line sets up
    (*wlv).fromcol = -10;
    (*wlv).tocol = MAXCOL;

    // Grid not used in init (set by win_line body in C), view dimensions:
    let view_width = nvim_win_get_view_width(wp);
    let view_height = nvim_win_get_view_height(wp);
    state.view_width = view_width;
    state.view_height = view_height;

    // in_curline: wp == curwin && lnum == curwin->w_cursor.lnum
    let is_curwin = nvim_win_is_curwin(wp) != 0;
    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    state.in_curline = is_curwin && lnum == cursor_lnum;

    // has_fold / has_foldtext / is_wrapped
    let has_fold = foldinfo.fi_level != 0 && foldinfo.fi_lines > 0;
    let p_fdt_ptr = nvim_win_get_p_fdt(wp);
    let has_foldtext = has_fold && !p_fdt_ptr.is_null() && *p_fdt_ptr != 0;
    let is_wrapped = nvim_win_get_p_wrap(wp) != 0 && !has_fold;
    state.has_fold = has_fold;
    state.has_foldtext = has_foldtext;
    state.is_wrapped = is_wrapped;

    // Initial local vars
    state.vcol_prev = -1;
    state.fromcol_prev = -2;
    state.noinvcur = false;
    state.lnum_in_visual_area = false;
    state.char_attr_pri = 0;
    state.char_attr_base = 0;
    state.area_highlighting = false;
    state.vi_attr = 0;
    state.area_attr = 0;
    state.search_attr = 0;
    state.vcol_save_attr = 0;
    state.decor_attr = 0;
    state.has_syntax = false;
    state.folded_attr = 0;
    state.eol_hl_off = 0;
    state.nextlinecol = 0;
    state.nextline_idx = 0;
    state.spell_attr = 0;
    state.word_end = 0;
    state.cur_checked_col = 0;
    state.extra_check = false;
    state.multi_attr = 0;
    state.mb_l = 1;
    state.mb_c = 0;
    state.mb_schar = 0;
    state.change_start = MAXCOL;
    state.change_end = -1;
    state.in_multispace = false;
    state.multispace_pos = 0;
    state.n_extra_next = 0;
    state.extra_attr_next = -1;
    state.search_attr_from_match = false;
    state.has_decor = false;
    state.saved_search_attr = 0;
    state.saved_area_attr = 0;
    state.saved_decor_attr = 0;
    state.saved_search_attr_from_match = false;
    state.win_col_offset = 0;
    state.area_active = false;
    state.decor_need_recheck = false;
    state.fold_vt = KVec::empty();
    state.foldtext_free = std::ptr::null_mut();
    state.cul_screenline = false;
    state.left_curline_col = 0;
    state.right_curline_col = 0;
    state.match_conc = 0;
    state.on_last_col = false;
    state.syntax_flags = 0;
    state.syntax_seqnr = 0;
    state.prev_syntax_id = 0;
    state.is_concealing = false;
    state.did_wcol = false;
    state.saved_attr2 = 0;
    state.n_attr3 = 0;
    state.saved_attr3 = 0;
    state.linestatus = 0;
    state.change_index = -1;
    state.check_decor_providers = false;
    state.decor_provider_end_col = 0;

    // conceal_attr
    state.conceal_attr = nvim_win_hl_attr(wp, HLF_CONCEAL);

    // draw_text: !concealed && lnum != buf->b_ml.ml_line_count + 1
    let buf = nvim_win_get_w_buffer(wp);
    let line_count = nvim_buf_get_line_count(buf);
    state.draw_text = !concealed && (lnum != line_count + 1);

    // color_cols (set on wlv): buf->terminal ? NULL : wp->w_p_cc_cols
    // We set this below after the draw_text check.

    if col_rows == 0 && state.draw_text {
        // extra_check: wp->w_p_lbr
        state.extra_check = nvim_win_get_p_lbr(wp) != 0;

        // Syntax highlighting setup
        if syntax_present(wp)
            && !nvim_win_get_syn_error(wp)
            && !nvim_win_get_syn_slow(wp)
            && !has_foldtext
        {
            let save_did_emsg = nvim_get_did_emsg();
            nvim_set_did_emsg(false);
            syntax_start(wp, lnum);
            if nvim_get_did_emsg() {
                nvim_win_set_syn_error(wp, true);
            } else {
                nvim_set_did_emsg_int(save_did_emsg);
                if !nvim_win_get_syn_slow(wp) {
                    state.has_syntax = true;
                    state.extra_check = true;
                }
            }
        }

        state.check_decor_providers = true;

        // color_cols setup
        if nvim_buf_get_terminal(buf) == 0 {
            (*wlv).color_cols = nvim_win_get_p_cc_cols(wp);
        } else {
            (*wlv).color_cols = std::ptr::null_mut();
        }
        advance_color_col_impl(wlv, (*wlv).vcol - (*wlv).vcol_off_co);

        // Visual area highlighting
        if VIsual_active && buf == nvim_curwin_get_buffer() {
            let vis_lnum = nvim_get_VIsual_lnum();
            let vis_col = nvim_get_VIsual_col();
            let vis_coladd = nvim_get_VIsual_coladd();
            let cursor_lnum2 = nvim_win_get_cursor_lnum(wp);
            let cursor_col = nvim_win_get_cursor_col(wp);
            let cursor_coladd = nvim_win_get_cursor_coladd(wp);

            let mut top: PosT;
            let mut bot: PosT;
            // ltoreq: cursor <= VIsual means cursor is top, VIsual is bot
            let cursor_lt_vis =
                cursor_lnum2 < vis_lnum || (cursor_lnum2 == vis_lnum && cursor_col <= vis_col);
            if cursor_lt_vis {
                top = PosT {
                    lnum: cursor_lnum2,
                    col: cursor_col,
                    coladd: cursor_coladd,
                };
                bot = PosT {
                    lnum: vis_lnum,
                    col: vis_col,
                    coladd: vis_coladd,
                };
            } else {
                top = PosT {
                    lnum: vis_lnum,
                    col: vis_col,
                    coladd: vis_coladd,
                };
                bot = PosT {
                    lnum: cursor_lnum2,
                    col: cursor_col,
                    coladd: cursor_coladd,
                };
            }

            state.lnum_in_visual_area = lnum >= top.lnum && lnum <= bot.lnum;
            let vis_mode = nvim_get_VIsual_mode();
            if vis_mode == i32::from(b'\x16') {
                // Ctrl-V block mode
                if state.lnum_in_visual_area {
                    (*wlv).fromcol = nvim_win_get_old_cursor_fcol(wp);
                    (*wlv).tocol = nvim_win_get_old_cursor_lcol(wp);
                }
            } else {
                // non-block mode
                if lnum > top.lnum && lnum <= bot.lnum {
                    (*wlv).fromcol = 0;
                } else if lnum == top.lnum {
                    if vis_mode == i32::from(b'V') {
                        // linewise
                        (*wlv).fromcol = 0;
                    } else {
                        let mut scol: ColnrT = 0;
                        nvim_getvvcol(
                            wp,
                            &mut top,
                            &mut scol,
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                        );
                        (*wlv).fromcol = scol;
                        if nvim_gchar_pos_lnum_col(top.lnum, top.col, top.coladd) == 0 {
                            // NUL char at top pos -> extend tocol
                            (*wlv).tocol = (*wlv).fromcol + 1;
                        }
                    }
                }
                if vis_mode != i32::from(b'V') && lnum == bot.lnum {
                    let p_sel_char = nvim_get_p_sel_first();
                    if p_sel_char == b'e' as c_char && bot.col == 0 && bot.coladd == 0 {
                        (*wlv).fromcol = -10;
                        (*wlv).tocol = MAXCOL;
                    } else if bot.col == MAXCOL {
                        (*wlv).tocol = MAXCOL;
                    } else {
                        let mut tocol: ColnrT = 0;
                        if p_sel_char == b'e' as c_char {
                            nvim_getvvcol(
                                wp,
                                &mut bot,
                                &mut tocol,
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                            );
                        } else {
                            nvim_getvvcol(
                                wp,
                                &mut bot,
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                                &mut tocol,
                            );
                            tocol += 1;
                        }
                        (*wlv).tocol = tocol;
                    }
                }
            }

            // Check if the char under the cursor should be inverted
            if nvim_get_highlight_match() == 0 && state.in_curline {
                let p_sel_char = nvim_get_p_sel_first();
                let exclusive = c_int::from(p_sel_char == b'e' as c_char);
                if nvim_cursor_is_block_during_visual(exclusive) != 0 {
                    state.noinvcur = true;
                }
            }

            // if inverting in this line set area_highlighting
            if (*wlv).fromcol >= 0 {
                state.area_highlighting = true;
                state.vi_attr = nvim_win_hl_attr(wp, HLF_V);
            }
        } else if nvim_get_highlight_match() != 0
            && is_curwin
            && !has_foldtext
            && lnum >= cursor_lnum
            && lnum <= cursor_lnum + nvim_get_search_match_lines()
        {
            // incsearch / :s///c highlighting
            if lnum == cursor_lnum {
                let mut cur_pos = PosT {
                    lnum: cursor_lnum,
                    col: nvim_win_get_cursor_col(wp),
                    coladd: nvim_win_get_cursor_coladd(wp),
                };
                let mut scol: ColnrT = 0;
                nvim_getvcol(
                    wp,
                    &mut cur_pos,
                    &mut scol,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
                (*wlv).fromcol = scol;
            } else {
                (*wlv).fromcol = 0;
            }
            let match_lines = nvim_get_search_match_lines();
            if lnum == cursor_lnum + match_lines {
                let endcol = nvim_get_search_match_endcol();
                let mut pos = PosT {
                    lnum,
                    col: endcol,
                    coladd: 0,
                };
                let mut tocol: ColnrT = 0;
                nvim_getvcol(
                    wp,
                    &mut pos,
                    &mut tocol,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
                (*wlv).tocol = tocol;
            }
            // do at least one character; happens when past end of line
            let endcol = nvim_get_search_match_endcol();
            if (*wlv).fromcol == (*wlv).tocol && endcol != 0 {
                (*wlv).tocol = (*wlv).fromcol + 1;
            }
            state.area_highlighting = true;
            state.vi_attr = nvim_win_hl_attr(wp, HLF_I);
        }
    }

    // bg_attr
    state.bg_attr = win_bg_attr(wp);

    // Diff highlighting setup
    state.linestatus = 0;
    let filler_lines = rs_diff_check_with_linestatus(wp, lnum, &mut state.linestatus);
    (*wlv).filler_lines = filler_lines;
    state.line_changes = DifflineStateC::default();
    state.change_index = -1;
    if state.linestatus < 0 {
        if state.linestatus == -1 {
            if rs_diff_find_change(wp, lnum, (&raw mut state.line_changes).cast()) {
                (*wlv).diff_hlf = HLF_ADD;
            } else if state.line_changes.num_changes > 0 {
                let added = rs_diff_change_parse(
                    (&raw mut state.line_changes).cast(),
                    state.line_changes.changes,
                    &mut state.change_start,
                    &mut state.change_end,
                );
                if state.change_start == 0 {
                    if added {
                        (*wlv).diff_hlf = HLF_TXA;
                    } else {
                        (*wlv).diff_hlf = HLF_TXD;
                    }
                } else {
                    (*wlv).diff_hlf = HLF_CHD;
                }
                state.change_index = 0;
            } else {
                (*wlv).diff_hlf = HLF_CHD;
                state.change_index = 0;
            }
        } else {
            (*wlv).diff_hlf = HLF_ADD;
        }
        state.area_highlighting = true;
    }

    // Virtual lines / filler setup
    state.virt_lines = KVec::empty();
    let n_virt_lines = decor_virt_lines(
        wp,
        lnum - 1,
        lnum,
        &mut (*wlv).n_virt_below,
        (&raw mut state.virt_lines).cast(),
        true,
    );
    (*wlv).n_virt_lines = n_virt_lines;
    (*wlv).filler_lines += n_virt_lines;
    if lnum == nvim_win_get_topline(wp) {
        (*wlv).filler_lines = nvim_win_get_topfill(wp);
        (*wlv).n_virt_lines = (*wlv).n_virt_lines.min((*wlv).filler_lines);
    }
    (*wlv).filler_todo = (*wlv).filler_lines;

    // Cursor line highlighting
    let w_p_cul = nvim_win_get_p_cul(wp) != 0;
    let culopt_flags = nvim_win_get_p_culopt_flags(wp);
    let cursorline = nvim_win_get_cursorline(wp);
    let k_opt_culopt_flag_number = 0x04;
    if w_p_cul
        && (culopt_flags & k_opt_culopt_flag_number) == 0
        && lnum == cursorline
        && !(is_curwin && VIsual_active)
    {
        let k_opt_culopt_flag_screenline = 0x02;
        state.cul_screenline = is_wrapped && (culopt_flags & k_opt_culopt_flag_screenline) != 0;
        if !state.cul_screenline {
            apply_cursorline_highlight_impl(wp, wlv);
        } else {
            // margin_columns_win is already in Rust (exported)
            let (left, right) = margin_columns_win_impl(wp);
            state.left_curline_col = left;
            state.right_curline_col = right;
        }
        state.area_highlighting = true;
    }

    // Sign/statuscolumn setup
    let mut sign_line_attr: c_int = 0;
    decor_redraw_signs(
        wp,
        buf.0,
        (*wlv).lnum - 1,
        (*wlv).sattrs.as_mut_ptr().cast(),
        &mut sign_line_attr,
        &mut (*wlv).sign_cul_attr,
        &mut (*wlv).sign_num_attr,
    );

    // statuscol: handled by the C side (the C win_line calls rs_draw_statuscol separately)
    // Here we just mirror the sign highlight logic.
    let stc_ptr = nvim_win_get_p_stc(wp);
    if stc_ptr.is_null() || *stc_ptr == 0 {
        // No statuscolumn: apply sign_cul and sign_num attrs directly
        if (*wlv).sign_cul_attr > 0 {
            (*wlv).sign_cul_attr = if use_cursor_line_highlight(wp, lnum) {
                syn_id2attr((*wlv).sign_cul_attr)
            } else {
                0
            };
        }
    }
    if (*wlv).sign_num_attr > 0 {
        (*wlv).sign_num_attr = syn_id2attr((*wlv).sign_num_attr);
    }
    if sign_line_attr > 0 {
        (*wlv).line_attr = syn_id2attr(sign_line_attr);
    }

    // Quickfix line highlight
    if rs_bt_quickfix(buf) && nvim_qf_current_entry(wp) == lnum {
        (*wlv).line_attr = nvim_win_hl_attr(wp, HLF_QFL);
    }

    if (*wlv).line_attr_lowprio != 0 || (*wlv).line_attr != 0 {
        state.area_highlighting = true;
    }

    // Save line attrs for later restore
    state.line_attr_save = (*wlv).line_attr;
    state.line_attr_lowprio_save = (*wlv).line_attr_lowprio;

    // Spell checking preparation
    let spv_has_spell = nvim_spv_get_has_spell(spv);
    if spv_has_spell && col_rows == 0 && state.draw_text {
        state.extra_check = true;

        // When a word wrapped from the previous line the start of the
        // current line is valid.
        let spv_checked_lnum = nvim_spv_get_checked_lnum(spv);
        if lnum == spv_checked_lnum {
            state.cur_checked_col = nvim_spv_get_checked_col(spv);
        }

        // Previous line was not spell checked, check for capital.
        let spv_capcol_lnum = nvim_spv_get_capcol_lnum(spv);
        if spv_capcol_lnum == 0 && check_need_cap(wp, lnum, 0) {
            nvim_spv_set_cap_col(spv, 0);
        } else if lnum != spv_capcol_lnum {
            nvim_spv_set_cap_col(spv, -1);
        }
        nvim_spv_set_checked_lnum(spv, 0);

        // Get the start of the next line for spell checking.
        // Fill nextline[SPWORDLEN..] from the next line first.
        let spwordlen = 150_i32;
        state.nextline[spwordlen as usize] = 0;
        if lnum < line_count {
            let next_line = nvim_win_ml_get_buf(wp, lnum + 1);
            spell_cat_line(
                state.nextline.as_mut_ptr().add(spwordlen as usize),
                next_line,
                spwordlen,
            );
        }

        let line_ptr = nvim_win_ml_get_buf(wp, lnum);
        let ptr = skipwhite(line_ptr);

        if *ptr == 0 {
            // Empty line: check first word in next line for capital.
            nvim_spv_set_cap_col(spv, 0);
            nvim_spv_set_capcol_lnum(spv, lnum + 1);
        } else {
            let spv_cap_col = nvim_spv_get_cap_col(spv);
            if spv_cap_col == 0 {
                nvim_spv_set_cap_col(spv, (ptr as usize - line_ptr as usize) as c_int);
            }
        }

        // Copy the end of the current line into nextline[].
        if state.nextline[spwordlen as usize] == 0 {
            // No next line or it is empty.
            state.nextlinecol = MAXCOL;
            state.nextline_idx = 0;
        } else {
            let line_len = nvim_win_ml_get_buf_len(wp, lnum);
            if line_len < spwordlen {
                state.nextlinecol = 0;
                libc_memmove(
                    state.nextline.as_mut_ptr().cast(),
                    line_ptr.cast(),
                    line_len as usize,
                );
                // STRMOVE: move nextline[spwordlen..] to nextline[line_len..]
                let src = state.nextline.as_ptr().add(spwordlen as usize);
                let dst = state.nextline.as_mut_ptr().add(line_len as usize);
                std::ptr::copy(src, dst, 300 - line_len as usize);
                state.nextline_idx = line_len + 1;
            } else {
                state.nextlinecol = line_len - spwordlen;
                libc_memmove(
                    state.nextline.as_mut_ptr().cast(),
                    line_ptr.add(state.nextlinecol as usize).cast(),
                    spwordlen as usize,
                );
                state.nextline_idx = spwordlen + 1;
            }
        }
    }
}

// Additional FFI for win_line_init
extern "C" {
    // spellvars_T opaque accessors
    fn nvim_spv_get_has_spell(spv: *mut c_void) -> bool;
    fn nvim_spv_get_checked_lnum(spv: *mut c_void) -> LinenrT;
    fn nvim_spv_get_checked_col(spv: *mut c_void) -> c_int;
    fn nvim_spv_get_capcol_lnum(spv: *mut c_void) -> LinenrT;
    fn nvim_spv_get_cap_col(spv: *mut c_void) -> c_int;
    fn nvim_spv_set_cap_col(spv: *mut c_void, val: c_int);
    fn nvim_spv_set_checked_lnum(spv: *mut c_void, val: LinenrT);
    fn nvim_spv_set_capcol_lnum(spv: *mut c_void, val: LinenrT);
    // win_T accessors not yet bound
    fn nvim_win_get_p_fdt(wp: WinHandle) -> *const c_char;
    fn nvim_win_get_p_lbr(wp: WinHandle) -> c_int;
    fn nvim_win_get_syn_error(wp: WinHandle) -> bool;
    fn nvim_win_get_syn_slow(wp: WinHandle) -> bool;
    fn nvim_win_set_syn_error(wp: WinHandle, val: bool);
    fn nvim_get_did_emsg() -> bool;
    fn nvim_set_did_emsg(val: bool);
    fn nvim_set_did_emsg_int(val: bool);
    fn nvim_win_get_p_cc_cols(wp: WinHandle) -> *mut c_int;
    fn nvim_curwin_get_buffer() -> BufHandle;
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_fdt_is_null(wp: WinHandle) -> bool;
}

/// Thin wrapper for memmove from C's libc.
#[allow(clippy::missing_const_for_fn)]
unsafe fn libc_memmove(dst: *mut u8, src: *const u8, n: usize) {
    // SAFETY: Caller ensures dst and src are valid for n bytes
    unsafe { std::ptr::copy(src, dst, n) };
}

// ============================================================================
// win_line migration phases: helper constants and FFI
// ============================================================================

/// SLF_WRAP flag from grid.h.
const SLF_WRAP: c_int = 2;
/// SLF_INC_VCOL flag from grid.h.
const SLF_INC_VCOL: c_int = 4;

/// HLF_CUC: 'cursorcolumn' highlight group (index 55).
const HLF_CUC: c_int = 55;

/// Ctrl-V codepoint for visual block mode check.
const CTRL_V: c_int = 0x16;

/// TERM_ATTRS_MAX: maximum columns for terminal highlight attributes.
const TERM_ATTRS_MAX: c_int = 1024;

/// MB_FILLER_CHAR: character used when a double-width character doesn't fit.
const MB_FILLER_CHAR: c_int = b'<' as c_int;

/// VALID_WCOL | VALID_WROW | VALID_VIRTCOL bits.
const VALID_WCOL_WROW_VIRTCOL: c_int = 0x07;

/// kVLLeftcol flag: start at left window edge.
const K_VL_LEFTCOL: c_int = 1;
/// kVLScroll flag: can scroll horizontally.
const K_VL_SCROLL: c_int = 2;

// Phase-functions FFI (new C accessors added to drawscreen_shim.c / drawline.c)
extern "C" {
    /// Get pointer to global screen_search_hl (match_T*).
    fn nvim_get_screen_search_hl_ptr() -> *mut c_void;
    /// Update curwin->w_cline_{row,height,folded} and w_valid.
    fn nvim_curwin_update_cline(startrow: c_int, row: c_int, has_fold: bool);
    /// Invalidate first column of next row in grid after a line wrap.
    fn nvim_grid_invalidate_next_row(grid: *mut c_void, row: c_int);
    /// wp->w_grid.target->cols
    fn nvim_win_get_w_grid_target_cols(wp: WinHandle) -> c_int;
    /// Get pointer to virt_lines[idx].line (VirtText*) as void*.
    fn nvim_virt_lines_get_line(virt_lines: *mut c_void, idx: c_int) -> *mut c_void;
    /// conceal_cursor_line: true if cursor line conceal should apply.
    fn conceal_cursor_line(wp: WinHandle) -> bool;
    /// wp->w_botfill
    fn nvim_win_get_botfill(wp: WinHandle) -> bool;
    /// schar_from_char
    fn schar_from_char(c: c_int) -> ScharT;
    /// wp->w_p_cole
    fn nvim_win_get_p_cole(wp: WinHandle) -> i64;
    /// wp->w_leftcol
    fn nvim_win_get_leftcol(wp: WinHandle) -> ColnrT;
    /// wp->w_virtcol (alias for nvim_win_get_virtcol which already exists)
    // (already declared above as nvim_win_get_virtcol)
    /// wp->w_wcol
    fn nvim_win_get_wcol(wp: WinHandle) -> ColnrT;
    /// Set wp->w_wcol
    fn nvim_win_set_wcol(wp: WinHandle, val: c_int);
    /// wp->w_wrow
    fn nvim_win_get_wrow(wp: WinHandle) -> c_int;
    /// Set wp->w_wrow
    fn nvim_win_set_wrow(wp: WinHandle, val: c_int);
    /// wp->w_valid |= bits
    fn nvim_win_set_valid_bits(wp: WinHandle, bits: c_int);
    /// Check p_cpo for character c
    fn nvim_vim_strchr_p_cpo(c: c_int) -> bool;
    // nvim_win_get_p_list already declared above
    // nvim_win_get_skipcol already declared above
    // nvim_win_get_p_wrap already declared above
    // nvim_win_get_lcs_prec already declared above (returns u32=ScharT)
    // nvim_win_get_lcs_eol already declared above
    // nvim_win_get_view_height already declared in separate extern block
    // nvim_win_get_p_rl already declared above
}

// ============================================================================
// Phase 1: EOL highlight, EOL fill, cursorcolumn
// ============================================================================

/// Handle highlighting at end of text line (C win_line lines 1728-1779).
///
/// Called when `mb_schar == NUL && eol_hl_off == 0`.
/// Returns new `eol_hl_off` (1 if highlight was placed, 0 otherwise).
///
/// # Safety
/// All pointers must be valid. `screen_search_hl` must point to a valid `match_T`.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rs_win_line_eol_highlight(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    state: *const WinLineState,
    lcs_eol_todo: bool,
    area_attr: c_int,
    ptr_col: ColnrT,
    screen_search_hl: *mut c_void,
) -> c_int {
    let has_fold = (*state).has_fold;
    let view_width = (*state).view_width;
    let lnum = (*wlv).lnum;

    let prevcol_hl_flag = get_prevcol_hl_flag(wp, screen_search_hl, ptr_col - 1);

    let visual_check = {
        let vis_mode = nvim_get_VIsual_mode();
        let vis_lnum = nvim_get_VIsual_lnum();
        let cursor_lnum = nvim_win_get_cursor_lnum(wp);
        vis_mode != CTRL_V || lnum == vis_lnum || lnum == cursor_lnum
    };

    if lcs_eol_todo
        && ((area_attr != 0 && (*wlv).vcol == (*wlv).fromcol && visual_check) || prevcol_hl_flag)
    {
        let linebuf_char = nvim_get_linebuf_char();
        let linebuf_attr = nvim_get_linebuf_attr();
        let linebuf_vcol = nvim_get_linebuf_vcol();

        let n = if (*wlv).col >= view_width { -1 } else { 0 };

        if n != 0 {
            (*wlv).off += n;
            (*wlv).col += n;
        } else {
            *linebuf_char.add((*wlv).off as usize) = schar_from_char(c_int::from(b' '));
        }

        if area_attr == 0 && !has_fold {
            get_search_match_hl(wp, screen_search_hl, ptr_col, &mut (*wlv).char_attr);
        }

        let eol_attr = if (*wlv).cul_attr != 0 {
            hl_combine_attr((*wlv).cul_attr, (*wlv).char_attr)
        } else {
            (*wlv).char_attr
        };

        *linebuf_attr.add((*wlv).off as usize) = eol_attr;
        *linebuf_vcol.add((*wlv).off as usize) = (*wlv).vcol;
        (*wlv).col += 1;
        (*wlv).off += 1;
        (*wlv).vcol += 1;
        1
    } else {
        0
    }
}

/// Fill past end-of-line with column/diff/terminal highlights and virtual text
/// (C win_line lines 1782-1881).
///
/// Called when `mb_schar == NUL`.
/// Returns `true` to signal the caller's while loop to `break`.
///
/// # Safety
/// All pointers must be valid. `term_attrs` must be at least `TERM_ATTRS_MAX` ints.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_win_line_eol_fill(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    state: *mut WinLineState,
    start_vcol: c_int,
    lcs_eol_todo: bool,
    eol_hl_off: c_int,
    term_attrs: *const c_int,
    has_decor: bool,
) -> bool {
    let view_width = (*state).view_width;
    let win_col_offset = (*state).win_col_offset;
    let in_curline = (*state).in_curline;
    let has_fold = (*state).has_fold;
    let bg_attr = (*state).bg_attr;
    let startrow = (*wlv).startrow;

    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();
    let linebuf_vcol = nvim_get_linebuf_vcol();

    // check if line ends before left margin
    let col_off_val = rs_win_col_off(wp);
    let new_vcol = (start_vcol + (*wlv).col - col_off_val).max((*wlv).vcol);
    (*wlv).vcol = new_vcol;
    // Get rid of boguscols.
    (*wlv).col -= (*wlv).boguscols;
    (*wlv).boguscols = 0;

    advance_color_col_impl(wlv, (*wlv).vcol - (*wlv).vcol_off_co);

    let eol_skip = c_int::from(lcs_eol_todo && eol_hl_off == 0);

    if has_decor {
        let decor_state = get_decor_state().cast::<c_void>();
        decor_redraw_eol(
            wp,
            decor_state,
            &mut (*wlv).line_attr,
            (*wlv).col + eol_skip,
        );
    }

    for i in (*wlv).col..view_width {
        *linebuf_vcol.add(((*wlv).off + (i - (*wlv).col)) as usize) =
            (*wlv).vcol + (i - (*wlv).col);
    }

    let buf = nvim_win_get_buffer(wp);
    let terminal = nvim_buf_get_terminal(BufHandle(buf));
    let lnum = (*wlv).lnum;
    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    let virtcol = nvim_win_get_virtcol(wp);
    let vcol_hlc = (*wlv).vcol - (*wlv).vcol_off_co;
    let view_width_isize = view_width as isize;
    let row_factor = ((*wlv).row - startrow + 1) as isize;

    let need_fill = (nvim_win_get_p_cuc(wp) != 0
        && virtcol >= vcol_hlc - eol_hl_off
        && (virtcol as isize) < view_width_isize * row_factor + start_vcol as isize
        && lnum != cursor_lnum)
        || !(*wlv).color_cols.is_null()
        || (*wlv).line_attr_lowprio != 0
        || (*wlv).line_attr != 0
        || (*wlv).diff_hlf != 0
        || terminal != 0;

    if need_fill {
        let mut rightmost_vcol = get_rightmost_vcol_impl(wp, (*wlv).color_cols);
        let cuc_attr = nvim_win_hl_attr(wp, HLF_CUC);
        let mc_attr = nvim_win_hl_attr(wp, HLF_MC);

        if (*wlv).diff_hlf == HLF_TXD || (*wlv).diff_hlf == HLF_TXA {
            (*wlv).diff_hlf = HLF_CHD;
            set_line_attr_for_diff_impl(wp, wlv);
        }

        let diff_attr = if (*wlv).diff_hlf != 0 {
            nvim_win_hl_attr(wp, (*wlv).diff_hlf)
        } else {
            0
        };

        let base_attr = hl_combine_attr((*wlv).line_attr_lowprio, diff_attr);
        if base_attr != 0 || (*wlv).line_attr != 0 || terminal != 0 {
            rightmost_vcol = c_int::MAX;
        }

        while (*wlv).col < view_width {
            *linebuf_char.add((*wlv).off as usize) = schar_from_char(c_int::from(b' '));

            advance_color_col_impl(wlv, (*wlv).vcol - (*wlv).vcol_off_co);

            let cur_vcol_hlc = (*wlv).vcol - (*wlv).vcol_off_co;
            let mut col_attr = base_attr;

            if nvim_win_get_p_cuc(wp) != 0 && cur_vcol_hlc == virtcol && lnum != cursor_lnum {
                col_attr = hl_combine_attr(col_attr, cuc_attr);
            } else if !(*wlv).color_cols.is_null() && cur_vcol_hlc == *(*wlv).color_cols {
                col_attr = hl_combine_attr(col_attr, mc_attr);
            }

            if terminal != 0 && (*wlv).vcol < TERM_ATTRS_MAX {
                col_attr = hl_combine_attr(col_attr, *term_attrs.add((*wlv).vcol as usize));
            }

            col_attr = hl_combine_attr(col_attr, (*wlv).line_attr);

            *linebuf_attr.add((*wlv).off as usize) = col_attr;
            (*wlv).off += 1;
            (*wlv).col += 1;
            (*wlv).vcol += 1;

            if ((*wlv).vcol - (*wlv).vcol_off_co) > rightmost_vcol {
                break;
            }
        }
    }

    // Draw fold virtual text if any.
    let fold_vt_ptr =
        std::ptr::from_mut::<KVec<VirtTextChunkC>>(&mut (*state).fold_vt).cast::<c_void>();
    if (*state).fold_vt.size > 0 {
        draw_virt_text_item_impl(
            BufHandle(buf),
            win_col_offset,
            fold_vt_ptr,
            HL_MODE_COMBINE,
            view_width,
            0,
            0,
        );
    }

    draw_virt_text_impl(
        wp,
        BufHandle(buf),
        win_col_offset,
        &mut (*wlv).col,
        (*wlv).row,
    );
    wlv_put_linebuf_impl(wp, wlv, (*wlv).col, true, bg_attr, SLF_INC_VCOL);
    (*wlv).row += 1;

    if in_curline {
        nvim_curwin_update_cline(startrow, (*wlv).row, has_fold);
    }

    true // caller should break
}

/// Highlight cursorcolumn and colorcolumn for current character position
/// (C win_line lines 1903-1927, including the `advance_color_col` at line 1903).
///
/// Returns the new `vcol_save_attr` (-1 if no override was applied).
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_win_line_cursorcolumn(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    lnum_in_visual_area: bool,
    search_attr: c_int,
    area_attr: c_int,
) -> c_int {
    advance_color_col_impl(wlv, (*wlv).vcol - (*wlv).vcol_off_co);

    let mut vcol_save_attr: c_int = -1;

    if !lnum_in_visual_area && search_attr == 0 && area_attr == 0 && (*wlv).filler_todo <= 0 {
        let virtcol = nvim_win_get_virtcol(wp);
        let lnum = (*wlv).lnum;
        let cursor_lnum = nvim_win_get_cursor_lnum(wp);
        let vcol_hlc = (*wlv).vcol - (*wlv).vcol_off_co;

        if nvim_win_get_p_cuc(wp) != 0 && vcol_hlc == virtcol && lnum != cursor_lnum {
            vcol_save_attr = (*wlv).char_attr;
            (*wlv).char_attr = hl_combine_attr(nvim_win_hl_attr(wp, HLF_CUC), (*wlv).char_attr);
        } else if !(*wlv).color_cols.is_null() && vcol_hlc == *(*wlv).color_cols {
            vcol_save_attr = (*wlv).char_attr;
            (*wlv).char_attr = hl_combine_attr(nvim_win_hl_attr(wp, HLF_MC), (*wlv).char_attr);
        }
    }

    // Apply lowest-priority line attr (lines 1924-1927).
    if (*wlv).filler_todo <= 0 {
        (*wlv).char_attr = hl_combine_attr((*wlv).line_attr_lowprio, (*wlv).char_attr);
    }

    vcol_save_attr
}

// ============================================================================
// Phase 2: Store character and post-store
// ============================================================================

/// Store character to linebuf and handle skip/conceal logic
/// (C win_line lines 1933-2021).
///
/// `mb_schar`, `multi_attr`, `is_concealing` are C local variables passed directly.
/// `is_wrapped` is read from `state` (it is const throughout the loop).
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_win_line_store_char(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    state: *const WinLineState,
    mb_schar: ScharT,
    multi_attr: c_int,
    is_concealing: bool,
) {
    let is_wrapped = (*state).is_wrapped;

    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();
    let linebuf_vcol = nvim_get_linebuf_vcol();

    if (*wlv).filler_todo > 0 {
        // No-op: wait for filler lines to finish.
    } else if (*wlv).skip_cells <= 0 {
        *linebuf_char.add((*wlv).off as usize) = mb_schar;
        if multi_attr != 0 {
            *linebuf_attr.add((*wlv).off as usize) = multi_attr;
        } else {
            *linebuf_attr.add((*wlv).off as usize) = (*wlv).char_attr;
        }
        *linebuf_vcol.add((*wlv).off as usize) = (*wlv).vcol;

        if schar_cells(mb_schar) > 1 {
            (*wlv).off += 1;
            (*wlv).col += 1;
            *linebuf_char.add((*wlv).off as usize) = 0;
            *linebuf_attr.add((*wlv).off as usize) = *linebuf_attr.add((*wlv).off as usize - 1);
            (*wlv).vcol += 1;
            *linebuf_vcol.add((*wlv).off as usize) = (*wlv).vcol;
            if (*wlv).tocol == (*wlv).vcol {
                (*wlv).tocol += 1;
            }
        }
        (*wlv).off += 1;
        (*wlv).col += 1;
    } else if nvim_win_get_p_cole(wp) > 0 && is_concealing {
        let concealed_wide = schar_cells(mb_schar) > 1;
        (*wlv).skip_cells -= 1;
        (*wlv).vcol_off_co += 1;
        if concealed_wide {
            (*wlv).vcol += 1;
            (*wlv).vcol_off_co += 1;
        }
        if (*wlv).n_extra > 0 {
            (*wlv).vcol_off_co += (*wlv).n_extra;
        }
        if is_wrapped {
            if (*wlv).n_extra > 0 {
                (*wlv).vcol += (*wlv).n_extra;
                (*wlv).col += (*wlv).n_extra;
                (*wlv).boguscols += (*wlv).n_extra;
                (*wlv).n_extra = 0;
                (*wlv).n_attr = 0;
            }
            if concealed_wide {
                (*wlv).boguscols += 1;
                (*wlv).col += 1;
            }
            (*wlv).boguscols += 1;
            (*wlv).col += 1;
        } else if (*wlv).n_extra > 0 {
            (*wlv).vcol += (*wlv).n_extra;
            (*wlv).n_extra = 0;
            (*wlv).n_attr = 0;
        }
    } else {
        (*wlv).skip_cells -= 1;
    }
}

/// Post-store: advance vcol, restore attributes, peek decorations
/// (C win_line lines 2023-2062).
///
/// `vcol_save_attr` is from `rs_win_line_cursorcolumn`.
/// `ptr_col` is `(colnr_T)(ptr - line)`.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_win_line_post_store(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    state: *mut WinLineState,
    vcol_save_attr: c_int,
    ptr_col: ColnrT,
) {
    if (*wlv).skipped_cells > 0 {
        (*wlv).vcol += (*wlv).skipped_cells;
        (*wlv).skipped_cells = 0;
    }
    if (*wlv).filler_todo <= 0 {
        (*wlv).vcol += 1;
    }
    if vcol_save_attr >= 0 {
        (*wlv).char_attr = vcol_save_attr;
    }
    if (*state).n_attr3 > 0 {
        (*state).n_attr3 -= 1;
        if (*state).n_attr3 == 0 {
            (*wlv).char_attr = (*state).saved_attr3;
        }
    }
    if (*wlv).n_attr > 0 {
        (*wlv).n_attr -= 1;
        if (*wlv).n_attr == 0 {
            (*wlv).char_attr = (*state).saved_attr2;
        }
    }

    let view_width = (*state).view_width;
    let is_wrapped = (*state).is_wrapped;

    if (*state).has_decor && (*wlv).filler_todo <= 0 && (*wlv).col >= view_width {
        let decor_state = get_decor_state().cast::<c_void>();
        if is_wrapped && (*wlv).n_extra == 0 {
            decor_redraw_col_impl(wp, ptr_col, -3, false, decor_state);
            (*state).decor_need_recheck = true;
        } else if !is_wrapped {
            decor_recheck_draw_col(-1, true, decor_state);
            decor_redraw_col_impl(wp, MAXCOL, -1, true, decor_state);
        }
    }
}

// ============================================================================
// Phase 3: End-check and line wrapping
// ============================================================================

/// At end of screen line: handle line wrapping, virt_line rendering, etc.
/// (C win_line lines 2064-2163).
///
/// This is called after the outer `if` condition has already been checked.
///
/// Returns `true` if the C while loop should `break`.
/// `draw_cols_out` is set to `true` to signal the caller to set `draw_cols = true`.
/// `virt_line_index_out` and `virt_line_flags_out` are reset to -1/0.
/// `lcs_prec_todo_out` is set from `wp->w_p_lcs_chars.prec`.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_win_line_end_check(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    state: *mut WinLineState,
    endrow: c_int,
    leftcols_width: c_int,
    virt_line_index: c_int,
    virt_line_flags: c_int,
    _ptr_is_nul: bool,
    lcs_eol_todo: bool,
    virt_lines_ptr: *mut c_void,
    bg_attr: c_int,
    statuscol_draw: *mut bool,
    draw_cols_out: *mut bool,
    virt_line_index_out: *mut c_int,
    virt_line_flags_out: *mut c_int,
    lcs_prec_todo_out: *mut ScharT,
) -> bool {
    let view_width = (*state).view_width;
    let is_wrapped = (*state).is_wrapped;
    // has_foldtext is checked externally before calling this function
    let win_col_offset = (*state).win_col_offset;
    let buf = nvim_win_get_buffer(wp);
    let startrow = (*wlv).startrow;

    let grid_width = nvim_win_get_w_grid_target_cols(wp);
    let wrap = is_wrapped
        && (*wlv).filler_todo <= 0
        && lcs_eol_todo
        && (*wlv).row != endrow - 1
        && view_width == grid_width
        && nvim_win_get_p_rl(wp) == 0;

    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();
    let linebuf_vcol = nvim_get_linebuf_vcol();
    let _ = linebuf_vcol; // already filled

    let mut draw_col = (*wlv).col - (*wlv).boguscols;

    for i in draw_col..view_width {
        *linebuf_vcol.add(((*wlv).off + (i - draw_col)) as usize) = (*wlv).vcol - 1;
    }

    if (*wlv).boguscols != 0 && ((*wlv).line_attr_lowprio != 0 || (*wlv).line_attr != 0) {
        let attr = hl_combine_attr((*wlv).line_attr_lowprio, (*wlv).line_attr);
        while draw_col < view_width {
            *linebuf_char.add((*wlv).off as usize) = schar_from_char(c_int::from(b' '));
            *linebuf_attr.add((*wlv).off as usize) = attr;
            (*wlv).off += 1;
            draw_col += 1;
        }
    }

    if virt_line_index >= 0 {
        let leftcol = if virt_line_flags & K_VL_LEFTCOL != 0 {
            0
        } else {
            win_col_offset
        };
        let scroll_left = if virt_line_flags & K_VL_SCROLL != 0 {
            nvim_win_get_leftcol(wp)
        } else {
            0
        };
        let line_vt = nvim_virt_lines_get_line(virt_lines_ptr, virt_line_index);
        draw_virt_text_item_impl(
            BufHandle(buf),
            leftcol,
            line_vt,
            HL_MODE_REPLACE,
            view_width,
            0,
            scroll_left,
        );
    } else if (*wlv).filler_todo <= 0 {
        draw_virt_text_impl(
            wp,
            BufHandle(buf),
            win_col_offset,
            &mut draw_col,
            (*wlv).row,
        );
    }

    wlv_put_linebuf_impl(
        wp,
        wlv,
        draw_col,
        true,
        bg_attr,
        if wrap { SLF_WRAP } else { 0 },
    );

    if wrap {
        let grid_ptr = nvim_win_get_w_grid(wp);
        nvim_grid_invalidate_next_row(grid_ptr, (*wlv).row);
    }

    (*wlv).boguscols = 0;
    (*wlv).vcol_off_co = 0;
    (*wlv).row += 1;

    if !is_wrapped && (*wlv).filler_todo <= 0 {
        return true;
    }

    if (*wlv).col <= leftcols_width {
        let view_height = (*state).view_height;
        win_draw_end(
            wp,
            schar_from_char(c_int::from(b'@')),
            true,
            (*wlv).row,
            view_height,
            HLF_AT,
        );
        set_empty_rows(wp, (*wlv).row);
        (*wlv).row = endrow;
    }

    if (*wlv).row == endrow {
        (*wlv).row += 1;
        return true;
    }

    rs_win_line_start(wp, wlv);
    if !draw_cols_out.is_null() {
        *draw_cols_out = true;
    }

    if !lcs_prec_todo_out.is_null() {
        *lcs_prec_todo_out = nvim_win_get_lcs_prec(wp);
    }

    if (*wlv).filler_todo <= 0 {
        (*wlv).need_showbreak = true;
    }

    if !statuscol_draw.is_null()
        && *statuscol_draw
        && nvim_vim_strchr_p_cpo(c_int::from(b'n'))
        && (*wlv).row > startrow + (*wlv).filler_lines
    {
        *statuscol_draw = false;
    }

    (*wlv).filler_todo -= 1;

    if !virt_line_index_out.is_null() {
        *virt_line_index_out = -1;
    }
    if !virt_line_flags_out.is_null() {
        *virt_line_flags_out = 0;
    }

    let botfill = nvim_win_get_botfill(wp);
    let draw_text = (*state).draw_text;
    if (*wlv).filler_todo == 0 && (botfill || !draw_text) {
        return true;
    }

    false // continue
}

// ============================================================================
// Phase 4: Extra attr restore, extends char, cursor conceal correct
// ============================================================================

/// Restore char_attr after special characters and handle precedes listchar
/// (C win_line lines 1678-1725).
///
/// Returns updated `lcs_prec_todo`.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rs_win_line_extra_attr_restore(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    state: *mut WinLineState,
    lcs_prec_todo: ScharT,
) -> ScharT {
    if (*wlv).n_attr > 0 && !(*state).search_attr_from_match {
        (*wlv).char_attr = hl_combine_attr((*wlv).char_attr, (*wlv).extra_attr);
        if (*wlv).reset_extra_attr {
            (*wlv).reset_extra_attr = false;
            if (*state).extra_attr_next >= 0 {
                (*wlv).extra_attr = (*state).extra_attr_next;
                (*state).extra_attr_next = -1;
            } else {
                (*wlv).extra_attr = 0;
                (*state).search_attr_from_match = (*state).saved_search_attr_from_match;
            }
        }
    }

    let mut new_lcs_prec_todo = lcs_prec_todo;
    let mb_schar = (*state).mb_schar;

    if lcs_prec_todo != 0
        && nvim_win_get_p_list(wp) != 0
        && (if nvim_win_get_p_wrap(wp) != 0 {
            nvim_win_get_skipcol(wp) > 0 && (*wlv).row == 0
        } else {
            nvim_win_get_leftcol(wp) > 0
        })
        && (*wlv).filler_todo <= 0
        && (*wlv).skip_cells <= 0
        && mb_schar != 0
    {
        new_lcs_prec_todo = 0;
        if schar_cells(mb_schar) > 1 {
            (*wlv).sc_extra = schar_from_char(MB_FILLER_CHAR);
            (*wlv).sc_final = 0;
            if (*wlv).n_extra > 0 {
                (*state).n_extra_next = (*wlv).n_extra;
                (*state).extra_attr_next = (*wlv).extra_attr;
                (*wlv).n_attr = 2.max((*wlv).n_attr + 1);
            } else {
                (*wlv).n_attr = 2;
            }
            (*wlv).n_extra = 1;
            (*wlv).extra_attr = nvim_win_hl_attr(wp, HLF_AT);
        }
        let prec_char = nvim_win_get_lcs_prec(wp);
        (*state).mb_schar = prec_char;
        (*state).mb_c = schar_get_first_codepoint(prec_char);
        (*state).saved_attr3 = (*wlv).char_attr;
        (*wlv).char_attr = nvim_win_hl_attr(wp, HLF_AT);
        (*state).n_attr3 = 1;
    }

    new_lcs_prec_todo
}

/// Show 'extends' character from 'listchars' if beyond the line end
/// (C win_line lines 1883-1901).
///
/// Updates `state->mb_schar`, `state->mb_c`, and `wlv->char_attr` if extends
/// character should be shown.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rs_win_line_extends_char(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    state: *mut WinLineState,
    ptr_col: ColnrT,
    ptr_is_nul: bool,
    lcs_eol: ScharT,
    lcs_eol_todo: bool,
    may_have_inline_virt: bool,
) {
    let lcs_ext = get_lcs_ext_impl(wp);
    if lcs_ext == 0 {
        return;
    }
    if (*wlv).filler_todo > 0 {
        return;
    }
    if (*wlv).col != (*state).view_width - 1 {
        return;
    }
    if (*state).has_foldtext {
        return;
    }

    if (*state).has_decor && ptr_is_nul && lcs_eol == 0 && lcs_eol_todo {
        let decor_state = get_decor_state().cast::<c_void>();
        decor_redraw_col_impl(wp, ptr_col, -1, false, decor_state);
    }

    if !ptr_is_nul
        || (lcs_eol > 0 && lcs_eol_todo)
        || ((*wlv).n_extra > 0
            && ((*wlv).sc_extra != 0 || (!(*wlv).p_extra.is_null() && *(*wlv).p_extra != 0)))
        || (may_have_inline_virt && has_more_inline_virt_impl(wlv, ptr_col as isize))
    {
        (*state).mb_schar = lcs_ext;
        (*wlv).char_attr = nvim_win_hl_attr(wp, HLF_AT);
        (*state).mb_c = schar_get_first_codepoint(lcs_ext);
    }
}

/// Correct cursor column when concealing characters (C win_line lines 1661-1676).
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_win_line_cursor_conceal_correct(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    state: *mut WinLineState,
    in_curline: bool,
    ptr_col: ColnrT,
) {
    let _ = ptr_col; // not used in this section
    if (*state).did_wcol {
        return;
    }
    if (*wlv).filler_todo > 0 {
        return;
    }
    if !in_curline {
        return;
    }
    if !conceal_cursor_line(wp) {
        return;
    }
    let mb_schar = (*state).mb_schar;
    let virtcol = nvim_win_get_virtcol(wp);
    if (*wlv).vcol + (*wlv).skip_cells < virtcol && mb_schar != 0 {
        return;
    }

    let wcol = (*wlv).col - (*wlv).boguscols;
    let wrow = nvim_win_get_wrow(wp); // use row from wlv
    let wrow_val = (*wlv).row;
    nvim_win_set_wcol(wp, wcol);
    nvim_win_set_wrow(wp, wrow_val);
    let _ = wrow;
    nvim_win_set_valid_bits(wp, VALID_WCOL_WROW_VIRTCOL);

    if (*wlv).vcol + (*wlv).skip_cells < virtcol {
        let extra = virtcol - (*wlv).vcol - (*wlv).skip_cells;
        let new_wcol = nvim_win_get_wcol(wp) + extra;
        nvim_win_set_wcol(wp, new_wcol);
    }

    (*state).did_wcol = true;
}

// ============================================================================
// Phase 5: N_extra processing
// ============================================================================

/// Return values from rs_win_line_process_n_extra.
#[repr(C)]
pub struct NExtraResult {
    /// Updated mb_schar value.
    pub mb_schar: ScharT,
    /// Updated mb_c value.
    pub mb_c: c_int,
    /// Updated mb_l value.
    pub mb_l: c_int,
}

/// Process n_extra chars (C win_line lines 1019-1102).
///
/// Called when `wlv->n_extra > 0`.
/// Returns updated `(mb_schar, mb_c, mb_l)`.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_win_line_process_n_extra(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    state: *mut WinLineState,
    ptr_is_nul: bool,
) -> NExtraResult {
    let view_width = (*state).view_width;

    let (mb_schar, mb_c, mb_l) =
        if (*wlv).sc_extra != 0 || ((*wlv).n_extra == 1 && (*wlv).sc_final != 0) {
            let sc = if (*wlv).n_extra == 1 && (*wlv).sc_final != 0 {
                (*wlv).sc_final
            } else {
                (*wlv).sc_extra
            };
            (*wlv).n_extra -= 1;
            (sc, schar_get_first_codepoint(sc), 1)
        } else {
            debug_assert!(!(*wlv).p_extra.is_null());
            let p = (*wlv).p_extra.cast_const();
            let mut firstc: c_int = 0;
            let sc = rs_utfc_ptr2schar(p, &mut firstc);
            let raw_l = utfc_ptr2len(p);
            let l = if raw_l > (*wlv).n_extra || raw_l == 0 {
                1
            } else {
                raw_l
            };

            if (*wlv).col >= view_width - 1 && schar_cells(sc) == 2 {
                let mc = c_int::from(b'>');
                let mat = nvim_win_hl_attr(wp, HLF_AT);
                let combined = if (*wlv).cul_attr != 0 {
                    if (*wlv).line_attr_lowprio != 0 {
                        hl_combine_attr((*wlv).cul_attr, mat)
                    } else {
                        hl_combine_attr(mat, (*wlv).cul_attr)
                    }
                } else {
                    mat
                };
                (*state).multi_attr = combined;
                (schar_from_char(mc), mc, 1)
            } else {
                (*wlv).n_extra -= l;
                (*wlv).p_extra = (*wlv).p_extra.add(l as usize);

                if (*wlv).filler_todo <= 0 && (*wlv).skip_cells > 0 && l > 1 {
                    if (*wlv).n_extra > 0 {
                        (*state).n_extra_next = (*wlv).n_extra;
                        (*state).extra_attr_next = (*wlv).extra_attr;
                    }
                    (*wlv).n_extra = 1;
                    (*wlv).sc_extra = schar_from_char(MB_FILLER_CHAR);
                    (*wlv).sc_final = 0;
                    (*wlv).n_attr += 1;
                    (*wlv).extra_attr = nvim_win_hl_attr(wp, HLF_AT);
                    (schar_from_char(c_int::from(b' ')), c_int::from(b' '), 1)
                } else {
                    (sc, firstc, l)
                }
            }
        };

    if (*wlv).n_extra <= 0 {
        if (*state).n_extra_next <= 0 {
            if (*state).search_attr == 0 {
                (*state).search_attr = (*state).saved_search_attr;
                (*state).saved_search_attr = 0;
            }
            if (*state).area_attr == 0 && !ptr_is_nul {
                (*state).area_attr = (*state).saved_area_attr;
                (*state).saved_area_attr = 0;
            }
            if (*state).decor_attr == 0 {
                (*state).decor_attr = (*state).saved_decor_attr;
                (*state).saved_decor_attr = 0;
            }
            if (*wlv).extra_for_extmark {
                (*wlv).reset_extra_attr = true;
                (*state).extra_attr_next = -1;
            }
            (*wlv).extra_for_extmark = false;
        } else {
            (*wlv).sc_extra = 0;
            (*wlv).sc_final = 0;
            (*wlv).n_extra = (*state).n_extra_next;
            (*state).n_extra_next = 0;
            (*wlv).reset_extra_attr = true;
        }
    }

    NExtraResult {
        mb_schar,
        mb_c,
        mb_l,
    }
}

/// Result returned by rs_win_line_highlight_attrs (Phase 2 migration).
///
/// Must match C typedef `HighlightResult` in drawline.c.
#[repr(C)]
pub struct HighlightResult {
    pub extmark_attr: c_int,
    pub has_match_conc: c_int,
}

/// Phase 2: Area highlighting + attr decision for one character position.
///
/// Absorbs lines 838-1016 of the original win_line() (C lines 946-1097 after Phase 1).
/// Called once per character in the main while loop, only when
/// `filler_todo <= 0 && (area_highlighting || spv_has_spell || extra_check)`.
///
/// All state mutations go directly through `state` (WinLineState fields).
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_win_line_highlight_attrs(
    wp: WinHandle,
    wlv: *mut WinLineVars,
    state: *mut WinLineState,
    ptr_col: ColnrT,
    lcs_eol_todo: bool,
    may_have_inline_virt: bool,
    lnum: LinenrT,
    screen_search_hl: *mut c_void,
) -> HighlightResult {
    let mut extmark_attr: c_int = 0;
    let mut has_match_conc: c_int = 0;

    // Reset extra_attr flag for this character position when not in extra-text mode.
    if (*wlv).n_extra == 0 || !(*wlv).extra_for_extmark {
        (*wlv).reset_extra_attr = false;
    }

    // Handle extmark/decor highlights when not in n_extra mode.
    if (*state).has_decor && (*wlv).n_extra == 0 {
        // Duplicate Visual area check to decide `selected` for decor calls.
        let vcol = (*wlv).vcol;
        let fromcol = (*wlv).fromcol;
        let tocol = (*wlv).tocol;
        let vcol_prev = (*state).vcol_prev;
        let fromcol_prev = (*state).fromcol_prev;
        let virtcol = nvim_win_get_virtcol(wp);

        let n_extra_cells = if (*wlv).n_extra == 0 {
            utf_ptr2cells_at(wp, lnum, ptr_col)
        } else {
            1
        };

        if vcol == fromcol
            || (vcol + 1 == fromcol && n_extra_cells > 1)
            || (vcol_prev == fromcol_prev && vcol_prev < vcol && vcol < tocol)
        {
            (*state).area_active = true;
        } else if (*state).area_active && (vcol == tocol || ((*state).noinvcur && vcol == virtcol))
        {
            (*state).area_active = false;
        }

        let selected = (*state).area_active
            || ((*state).area_highlighting && (*state).noinvcur && vcol == virtcol);

        // Recheck non-inline virt text draw column if needed.
        if (*state).decor_need_recheck {
            if !may_have_inline_virt {
                let ds = get_decor_state().cast::<c_void>();
                decor_recheck_draw_col((*wlv).off, selected, ds);
            }
            (*state).decor_need_recheck = false;
        }

        let decor_state = get_decor_state().cast::<c_void>();
        let win_col = if may_have_inline_virt { -3 } else { (*wlv).off };
        extmark_attr = decor_redraw_col_impl(wp, ptr_col, win_col, selected, decor_state);

        if may_have_inline_virt {
            handle_inline_virtual_text_impl(wp, wlv, ptr_col as isize, selected);
            if (*wlv).n_extra > 0 && (*wlv).virt_inline_hl_mode <= HL_MODE_REPLACE {
                // Save current attrs and reset for inline virt text rendering.
                (*state).saved_search_attr = (*state).search_attr;
                (*state).saved_area_attr = (*state).area_attr;
                (*state).saved_decor_attr = (*state).decor_attr;
                (*state).saved_search_attr_from_match = (*state).search_attr_from_match;
                (*state).search_attr = 0;
                (*state).area_attr = 0;
                (*state).decor_attr = 0;
                (*state).search_attr_from_match = false;
            }
        }
    }

    // Determine which area_attr to update (normal vs saved for inline virt).
    let use_saved_area = (*wlv).extra_for_extmark && (*wlv).virt_inline_hl_mode <= HL_MODE_REPLACE;

    // Handle Visual area or match highlighting.
    {
        let vcol = (*wlv).vcol;
        let fromcol = (*wlv).fromcol;
        let tocol = (*wlv).tocol;
        let vcol_prev = (*state).vcol_prev;
        let fromcol_prev = (*state).fromcol_prev;
        let noinvcur = (*state).noinvcur;
        let virtcol = nvim_win_get_virtcol(wp);

        let n_extra_cells = if (*wlv).n_extra == 0 {
            utf_ptr2cells_at(wp, lnum, ptr_col)
        } else if !(*wlv).p_extra.is_null() {
            utf_ptr2cells((*wlv).p_extra.cast_const())
        } else {
            1
        };

        let area_attr_ref = if use_saved_area {
            &mut (*state).saved_area_attr
        } else {
            &mut (*state).area_attr
        };

        if vcol == fromcol
            || (vcol + 1 == fromcol && n_extra_cells > 1)
            || (vcol_prev == fromcol_prev && vcol_prev < vcol && vcol < tocol)
        {
            *area_attr_ref = (*state).vi_attr;
            (*state).area_active = true;
        } else if *area_attr_ref != 0 && (vcol == tocol || (noinvcur && vcol == virtcol)) {
            *area_attr_ref = 0;
            (*state).area_active = false;
        }
    }

    // Update search highlighting (only when not in fold text or extra chars).
    if !(*state).has_foldtext && (*wlv).n_extra == 0 {
        // update_search_hl can change the line pointer; we pass a local copy.
        // The caller always re-fetches line after this call returns.
        let mut line_ptr: *const c_char = nvim_win_ml_get_buf(wp, lnum);
        let mut match_conc = (*state).match_conc;
        let mut on_last_col = (*state).on_last_col;
        let mut search_attr_from_match = (*state).search_attr_from_match;

        (*state).search_attr = update_search_hl(
            wp,
            lnum,
            ptr_col,
            &mut line_ptr,
            screen_search_hl,
            &mut has_match_conc,
            &mut match_conc,
            lcs_eol_todo,
            &mut on_last_col,
            &mut search_attr_from_match,
        );

        (*state).match_conc = match_conc;
        (*state).on_last_col = on_last_col;
        (*state).search_attr_from_match = search_attr_from_match;

        // Check if at NUL: re-fetch line_ptr in case update_search_hl changed it.
        // If at NUL (end of line), concealing is not allowed.
        let line_after = nvim_win_ml_get_buf(wp, lnum);
        if *line_after.add(ptr_col as usize) == 0 {
            has_match_conc = 0;
        }

        // Check ComplMatchIns highlight.
        if (nvim_get_state() & MODE_INSERT) != 0
            && rs_ins_compl_win_active(wp) != 0
            && ((*state).in_curline || rs_ins_compl_lnum_in_range(lnum) != 0)
        {
            let ins_match_attr = rs_ins_compl_col_range_attr(lnum, ptr_col);
            if ins_match_attr > 0 {
                (*state).search_attr = hl_combine_attr((*state).search_attr, ins_match_attr);
            }
        }
    }

    // Update diff highlighting.
    if (*wlv).diff_hlf != 0 {
        let num_changes = (*state).line_changes.num_changes;
        let bufidx = (*state).line_changes.bufidx;
        let change_index = (*state).change_index;

        // Advance change_index if ptr has passed the start of the next change.
        if num_changes > 0 && change_index >= 0 && change_index < num_changes - 1 {
            let next_change =
                nvim_diff_diffline_get_change((*state).line_changes.changes, change_index + 1);
            if !next_change.is_null() {
                let next_start = nvim_diffchange_get_start(next_change, bufidx) as c_int;
                if ptr_col >= next_start {
                    (*state).change_index += 1;
                }
            }
        }

        // Parse current change boundaries.
        let change_index = (*state).change_index;
        let mut added = false;
        if num_changes > 0 && change_index >= 0 && change_index < num_changes {
            let cur_change =
                nvim_diff_diffline_get_change((*state).line_changes.changes, change_index);
            if !cur_change.is_null() {
                added = rs_diff_change_parse(
                    (*state).line_changes.changes,
                    cur_change,
                    &mut (*state).change_start,
                    &mut (*state).change_end,
                );
            }
        }

        // Switch diff_hlf based on position within changed region.
        if (*wlv).diff_hlf == HLF_CHD && ptr_col >= (*state).change_start && (*wlv).n_extra == 0 {
            (*wlv).diff_hlf = if added { HLF_TXA } else { HLF_TXD };
        }
        if ((*wlv).diff_hlf == HLF_TXD || (*wlv).diff_hlf == HLF_TXA)
            && ((ptr_col >= (*state).change_end && (*wlv).n_extra == 0)
                || ((*wlv).n_extra > 0 && (*wlv).extra_for_extmark))
        {
            (*wlv).diff_hlf = HLF_CHD;
        }
        set_line_attr_for_diff_impl(wp, wlv);
    }

    // Decide which highlight attribute to use.
    let area_attr = (*state).area_attr;
    let search_attr = (*state).search_attr;
    let highlight_match = nvim_get_highlight_match();
    let folded_attr = (*state).folded_attr;
    let decor_attr = (*state).decor_attr;

    (*state).char_attr_pri = if area_attr != 0 {
        let combined = hl_combine_attr((*wlv).line_attr, area_attr);
        if highlight_match == 0 {
            // let search highlight show in Visual area if possible
            hl_combine_attr(search_attr, combined)
        } else {
            combined
        }
    } else if search_attr != 0 {
        hl_combine_attr((*wlv).line_attr, search_attr)
    } else if (*wlv).line_attr != 0
        && (((*wlv).fromcol == -10 && (*wlv).tocol == MAXCOL)
            || (*wlv).vcol < (*wlv).fromcol
            || (*state).vcol_prev < (*state).fromcol_prev
            || (*wlv).vcol >= (*wlv).tocol)
    {
        // Use line_attr when not in Visual or 'incsearch' area.
        (*wlv).line_attr
    } else {
        0
    };

    (*state).char_attr_base = hl_combine_attr(folded_attr, decor_attr);
    (*wlv).char_attr = hl_combine_attr((*state).char_attr_base, (*state).char_attr_pri);

    HighlightResult {
        extmark_attr,
        has_match_conc,
    }
}

/// Return the number of cells for the character at byte offset `col` in the buffer line.
/// Helper for rs_win_line_highlight_attrs.
#[inline]
unsafe fn utf_ptr2cells_at(wp: WinHandle, lnum: LinenrT, col: ColnrT) -> c_int {
    let line = nvim_win_ml_get_buf(wp, lnum);
    utf_ptr2cells(line.add(col as usize))
}

// ============================================================================
// Phase 1: c_win_line_pre_loop migration
// ============================================================================

/// Result type for nvim_c_advance_to_start_vcol (matches C AdvanceToStartVcolResult).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct AdvanceToStartVcolResult {
    ptr_offset: c_int,
    vcol: c_int,
    in_multispace: bool,
    multispace_pos: c_int,
    skip_cells: c_int,
    fromcol: c_int,
    need_showbreak: bool,
}

/// PreLoopResult: output from rs_c_win_line_pre_loop (matches C PreLoopResult).
///
/// Size verified by `_Static_assert(sizeof(PreLoopResult) == 60, ...)` in drawline.c.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PreLoopResult {
    pub ptr_offset: c_int,
    pub trailcol: ColnrT,
    pub leadcol: ColnrT,
    pub lcs_eol_todo: bool,
    pub lcs_eol: ScharT,
    pub lcs_prec_todo: ScharT,
    pub start_vcol: c_int,
    pub may_have_inline_virt: bool,
    pub virt_line_index: c_int,
    pub virt_line_flags: c_int,
    pub draw_cols: bool,
    pub leftcols_width: c_int,
    pub statuscol_draw: bool,
    pub statuscol_width: c_int,
    pub statuscol_sign_cul_id: c_int,
}

// FFI: new drawline_ffi.c accessor functions for Phase 1
extern "C" {
    /// Charsize iteration wrapper: advance ptr to start_vcol.
    fn nvim_c_advance_to_start_vcol(
        wp: WinHandle,
        lnum: LinenrT,
        line: *mut c_char,
        start_vcol: c_int,
        wlv_vcol: c_int,
        wlv_tocol: c_int,
        wlv_fromcol: c_int,
        has_fold: bool,
        p_list: bool,
        p_wrap: bool,
        leadcol: c_int,
        in_ms: bool,
        ms_pos: c_int,
    ) -> AdvanceToStartVcolResult;

    /// Return true if virtual editing is active for the given window.
    fn nvim_win_virtual_active_drawline(wp: WinHandle) -> bool;
    /// Return true if wp->w_buffer->terminal is non-NULL.
    fn nvim_win_buf_has_terminal_drawline(wp: WinHandle) -> bool;
    /// Get buf_meta_total(wp->w_buffer, kMTMetaInline) > 0.
    fn nvim_win_buf_meta_total_inline(wp: WinHandle) -> bool;
    /// Get highlight_attr[hlf].
    fn nvim_get_highlight_attr_hlf(hlf: c_int) -> c_int;
    /// Wrap terminal_get_line_attributes.
    fn nvim_win_terminal_get_line_attrs(wp: WinHandle, lnum: c_int, term_attrs: *mut c_int);
    /// Get length of wp->w_p_lcs_chars.multispace array.
    fn nvim_win_lcs_multispace_len(wp: WinHandle) -> c_int;
    /// Get length of wp->w_p_lcs_chars.leadmultispace array.
    fn nvim_win_lcs_leadmultispace_len(wp: WinHandle) -> c_int;
    /// Return true if *wp->w_p_stc != NUL.
    fn nvim_win_get_w_p_stc_is_set(wp: WinHandle) -> bool;
    /// wp->w_p_lcs_chars.space (already in window crate, declare here for local use)
    fn nvim_win_lcs_space(wp: WinHandle) -> ScharT;
    /// wp->w_p_lcs_chars.trail
    fn nvim_win_lcs_trail(wp: WinHandle) -> ScharT;
    /// wp->w_p_lcs_chars.lead
    fn nvim_win_lcs_lead(wp: WinHandle) -> ScharT;
    /// wp->w_p_lcs_chars.nbsp
    fn nvim_win_lcs_nbsp(wp: WinHandle) -> ScharT;
    /// Set wp->w_cursor.lnum
    fn nvim_win_set_cursor_lnum(wp: WinHandle, lnum: LinenrT);
    /// Set wp->w_cursor.col
    fn nvim_win_set_cursor_col(wp: WinHandle, col: ColnrT);
    /// Set wp->w_cursor.coladd
    fn nvim_win_set_cursor_coladd(wp: WinHandle, coladd: ColnrT);
}

/// Implement `c_win_line_pre_loop` in Rust.
///
/// Sets up pre-loop state: statuscol init, trailing/leading whitespace detection,
/// vcol advancement with charsize iteration (via C wrapper), spell setup,
/// decoration redraw, search highlight preparation, terminal attributes.
///
/// # Safety
/// All pointers must be valid. `line` must point to the start of the buffer line.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_c_win_line_pre_loop(
    wp: WinHandle,
    lnum: LinenrT,
    wlv: *mut WinLineVars,
    wls: *mut WinLineState,
    spv: *mut c_void,
    _foldinfo: FoldInfo,
    startrow: c_int,
    endrow: c_int,
    col_rows: c_int,
    term_attrs: *mut c_int,
) -> PreLoopResult {
    let state = &mut *wls;

    let draw_text = state.draw_text;
    let has_foldtext = state.has_foldtext;
    let has_fold = state.has_fold;

    // Build initial result
    let lcs_eol = nvim_win_get_lcs_eol(wp);
    let lcs_prec_todo = nvim_win_get_lcs_prec(wp);
    let mut res = PreLoopResult {
        ptr_offset: 0,
        trailcol: MAXCOL,
        leadcol: 0,
        lcs_eol_todo: true,
        lcs_eol,
        lcs_prec_todo,
        start_vcol: 0,
        may_have_inline_virt: false,
        virt_line_index: -1,
        virt_line_flags: 0,
        draw_cols: true,
        leftcols_width: 0,
        statuscol_draw: false,
        statuscol_width: 0,
        statuscol_sign_cul_id: 0,
    };

    // statuscol initialization
    if nvim_win_get_w_p_stc_is_set(wp) {
        res.statuscol_draw = true;
        let cmdwin_win = nvim_get_cmdwin_win();
        let cmdwin_offset = c_int::from(wp == cmdwin_win);
        res.statuscol_width = rs_win_col_off(wp) - cmdwin_offset;
        res.statuscol_sign_cul_id = if use_cursor_line_highlight(wp, lnum) {
            (*wlv).sign_cul_attr
        } else {
            0
        };
    }

    // current line pointer (as byte offset into the buffer line)
    let mut ptr_col: c_int = 0; // offset of ptr into line

    let p_list = nvim_win_get_p_list(wp) != 0;
    let p_wrap = nvim_win_get_p_wrap(wp) != 0;

    if p_list && !has_foldtext && draw_text {
        // Check if any listchars that affect whitespace are set
        let has_space = nvim_win_lcs_space(wp) != 0
            || nvim_win_lcs_multispace_len(wp) > 0
            || nvim_win_lcs_leadmultispace_len(wp) > 0
            || nvim_win_lcs_trail(wp) != 0
            || nvim_win_lcs_lead(wp) != 0
            || nvim_win_lcs_nbsp(wp) != 0;
        if has_space {
            state.extra_check = true;
        }

        // Find start of trailing whitespace
        if nvim_win_lcs_trail(wp) != 0 {
            let line_len = nvim_win_ml_get_buf_len(wp, lnum);
            let line_ptr = nvim_win_ml_get_buf(wp, lnum);
            let mut trailcol = line_len;
            // ascii_iswhite: space or tab
            while trailcol > 0 {
                let c = *line_ptr.add((trailcol - 1) as usize) as u8;
                if c != b' ' && c != b'\t' {
                    break;
                }
                trailcol -= 1;
            }
            res.trailcol = trailcol + ptr_col;
        }

        // Find end of leading whitespace
        if nvim_win_lcs_lead(wp) != 0 || nvim_win_lcs_leadmultispace_len(wp) > 0 {
            let line_ptr = nvim_win_ml_get_buf(wp, lnum);
            let mut leadcol: c_int = 0;
            loop {
                let c = *line_ptr.add(leadcol as usize) as u8;
                if c != b' ' && c != b'\t' {
                    break;
                }
                leadcol += 1;
            }
            // Check if line is all spaces (leadcol at NUL)
            if *line_ptr.add(leadcol as usize) == NUL {
                res.leadcol = 0;
            } else {
                res.leadcol = leadcol + ptr_col + 1;
            }
        }
    }

    // Determine start_vcol
    res.start_vcol = if p_wrap {
        if startrow == 0 {
            nvim_win_get_skipcol(wp)
        } else {
            0
        }
    } else {
        nvim_win_get_leftcol(wp)
    };

    if has_foldtext {
        (*wlv).vcol = res.start_vcol;
    } else if res.start_vcol > 0 && col_rows == 0 {
        // Use the C charsize iteration wrapper
        let line_ptr = if draw_text {
            nvim_win_ml_get_buf(wp, lnum).cast_mut()
        } else {
            c"".as_ptr().cast_mut()
        };

        let adv = nvim_c_advance_to_start_vcol(
            wp,
            lnum,
            line_ptr,
            res.start_vcol,
            (*wlv).vcol,
            (*wlv).tocol,
            (*wlv).fromcol,
            has_fold,
            p_list,
            p_wrap,
            res.leadcol,
            state.in_multispace,
            state.multispace_pos,
        );

        (*wlv).vcol = adv.vcol;
        (*wlv).skip_cells = adv.skip_cells;
        (*wlv).fromcol = adv.fromcol;
        (*wlv).need_showbreak = adv.need_showbreak;
        state.in_multispace = adv.in_multispace;
        state.multispace_pos = adv.multispace_pos;
        ptr_col = adv.ptr_offset;

        // Spell check at start of line (when line starts mid-word)
        if nvim_spv_get_has_spell(spv) {
            let linecol = ptr_col;
            let mut spell_hlf: c_int = 76; // HLF_COUNT

            // Save/restore cursor around spell_move_to
            // (spell_move_to modifies wp->w_cursor then we restore it)
            let save_cursor_lnum = nvim_win_get_cursor_lnum(wp);
            let save_cursor_col = nvim_win_get_cursor_col(wp);
            let save_cursor_coladd = nvim_win_get_cursor_coladd(wp);

            // Set cursor to our position
            nvim_win_set_cursor_lnum(wp, lnum);
            nvim_win_set_cursor_col(wp, linecol);

            let forward: c_int = 1;
            let smt_all: c_int = 3;
            let len = spell_move_to(wp, forward, smt_all, true, &mut spell_hlf);

            // spell_move_to() may call ml_get() and make "line" invalid
            // Re-fetch line pointer (ptr_col stays valid as byte offset)
            let line_ptr2 = nvim_win_ml_get_buf(wp, lnum);
            let cursor_col_after = nvim_win_get_cursor_col(wp);

            let hlf_count: c_int = 76; // HLF_COUNT
            if len == 0 || cursor_col_after > linecol {
                // no bad word found at line start, don't check until end of a word
                let ptr2 = line_ptr2.add(linecol as usize);
                state.word_end =
                    (spell_to_word_end(ptr2, wp) as usize - line_ptr2 as usize + 1) as c_int;
            } else {
                // bad word found, use attributes until end of word
                state.word_end = cursor_col_after + len as c_int + 1;
                if spell_hlf != hlf_count {
                    state.spell_attr = nvim_get_highlight_attr_hlf(spell_hlf);
                }
            }

            // Restore cursor
            nvim_win_set_cursor_lnum(wp, save_cursor_lnum);
            nvim_win_set_cursor_col(wp, save_cursor_col);
            nvim_win_set_cursor_coladd(wp, save_cursor_coladd);

            // Need to restart syntax highlighting for this line.
            if state.has_syntax {
                syntax_start(wp, lnum);
            }
        }
    }

    if state.check_decor_providers {
        state.decor_provider_end_col =
            rs_decor_providers_setup(endrow - startrow, res.start_vcol == 0, lnum, ptr_col, wp);
        // rs_decor_providers_setup may invalidate line pointer; re-fetch not needed
        // since we only use ptr_col (integer offset)
    }

    decor_redraw_line(wp, lnum - 1, get_decor_state().cast::<c_void>());
    if !state.has_decor && decor_has_more_decorations(get_decor_state().cast::<c_void>(), lnum - 1)
    {
        state.has_decor = true;
        state.extra_check = true;
    }

    // Correct highlighting for cursor that can't be disabled.
    if (*wlv).fromcol >= 0 {
        if state.noinvcur {
            let virtcol = nvim_win_get_virtcol(wp);
            if (*wlv).fromcol as ColnrT == virtcol {
                state.fromcol_prev = (*wlv).fromcol;
                (*wlv).fromcol = -1;
            } else if ((*wlv).fromcol as ColnrT) < virtcol {
                state.fromcol_prev = virtcol;
            }
        }
        if (*wlv).fromcol >= (*wlv).tocol {
            (*wlv).fromcol = -1;
        }
    }

    if col_rows == 0 && draw_text && !has_foldtext {
        // prepare_search_hl_line: update line pointer (via *line)
        let screen_search_hl = nvim_get_screen_search_hl_ptr();
        let mut line_for_search: *const c_char = nvim_win_ml_get_buf(wp, lnum);
        let area = prepare_search_hl_line(
            wp,
            lnum,
            ptr_col,
            &mut line_for_search,
            screen_search_hl,
            &mut state.search_attr,
            &mut state.search_attr_from_match,
        );
        state.area_highlighting |= area;
        // Update ptr_col if line pointer moved (prepare_search_hl_line may update *line)
        // ptr_col stays valid as an offset; line base may have changed but offset is the same
    }

    if (nvim_get_state() & MODE_INSERT) != 0
        && rs_ins_compl_win_active(wp) != 0
        && (state.in_curline || rs_ins_compl_lnum_in_range(lnum as c_int) != 0)
    {
        state.area_highlighting = true;
    }

    rs_win_line_start(wp, wlv);

    if nvim_win_buf_has_terminal_drawline(wp) {
        nvim_win_terminal_get_line_attrs(wp, lnum, term_attrs);
        state.extra_check = true;
    }

    res.may_have_inline_virt = !has_foldtext && nvim_win_buf_meta_total_inline(wp);
    res.ptr_offset = ptr_col;
    res
}

// ============================================================================
// Phase 2: draw_cols block migration
// ============================================================================

/// Action codes returned by rs_win_line_draw_cols.
/// Must match DRAW_COLS_ACTION_* constants used in C wrapper.
const DRAW_COLS_ACTION_FALLTHROUGH: c_int = 0;
const DRAW_COLS_ACTION_BREAK: c_int = 1;
const DRAW_COLS_ACTION_CONTINUE: c_int = 2;
const DRAW_COLS_ACTION_GOTO_END_CHECK: c_int = 3;

/// Return struct for rs_win_line_draw_cols (matches C DrawColsResult).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrawColsResult {
    /// Control flow action for the caller.
    pub action: c_int,
    /// Updated draw_cols flag.
    pub draw_cols: bool,
    /// Updated leftcols_width.
    pub leftcols_width: c_int,
    /// Updated virt_line_index.
    pub virt_line_index: c_int,
    /// Updated virt_line_flags.
    pub virt_line_flags: c_int,
    /// Updated win_col_offset.
    pub win_col_offset: c_int,
    /// Updated ptr byte offset (for re-fetching ptr = line + ptr_offset).
    pub ptr_offset: c_int,
}

// FFI for Phase 2
extern "C" {
    /// kv_size(*vl) for VirtLines.
    fn nvim_kv_size_virt_lines(vl: *mut c_void) -> c_int;
    /// kv_A(*vl, idx).flags for VirtLines.
    fn nvim_kv_A_virt_lines_flags(vl: *mut c_void, idx: c_int) -> c_int;
    /// wp->w_p_fcs_chars.fold.
    fn nvim_win_get_fcs_fold(wp: WinHandle) -> ScharT;
    /// nvim_get_cmdwin_type (cmdwin_type global, c_int).
    fn nvim_get_cmdwin_type() -> c_int;
    /// wp->w_redr_statuscol.
    fn nvim_win_get_redr_statuscol(wp: WinHandle) -> bool;
    /// wp->w_scwidth.
    #[link_name = "nvim_win_get_scwidth"]
    fn nvim_win_get_w_scwidth_drawline(wp: WinHandle) -> c_int;
    /// Convert ASCII byte to schar_T (from grid crate).
    fn rs_schar_from_ascii(c: c_int) -> ScharT;
}

/// Handle the `draw_cols` block from win_line.
///
/// Implements the `if (draw_cols)` block from the main loop body.
/// Returns a `DrawColsResult` with a control-flow action code:
/// - DRAW_COLS_ACTION_FALLTHROUGH: continue with loop body normally
/// - DRAW_COLS_ACTION_BREAK: break the outer while loop
/// - DRAW_COLS_ACTION_CONTINUE: continue to next iteration
/// - DRAW_COLS_ACTION_GOTO_END_CHECK: jump to `end_check:` label
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_win_line_draw_cols(
    wp: WinHandle,
    _lnum: LinenrT,
    wlv: *mut WinLineVars,
    wls: *const WinLineState,
    statuscol: *mut c_void,
    statuscol_draw: bool,
    virt_lines: *mut c_void,
    ptr_col: c_int,
    startrow: c_int,
    endrow: c_int,
    col_rows: c_int,
    virt_line_index_in: c_int,
    virt_line_flags_in: c_int,
    leftcols_width_in: c_int,
    win_col_offset_in: c_int,
    draw_text: bool,
    has_decor: bool,
    bg_attr: c_int,
) -> DrawColsResult {
    let state = &*wls;
    let mut res = DrawColsResult {
        action: DRAW_COLS_ACTION_FALLTHROUGH,
        draw_cols: true,
        leftcols_width: leftcols_width_in,
        virt_line_index: virt_line_index_in,
        virt_line_flags: virt_line_flags_in,
        win_col_offset: win_col_offset_in,
        ptr_offset: ptr_col,
    };

    // Restore cul_screenline line attrs
    if state.cul_screenline {
        (*wlv).cul_attr = 0;
        (*wlv).line_attr = state.line_attr_save;
        (*wlv).line_attr_lowprio = state.line_attr_lowprio_save;
    }

    // assert(wlv.off == 0)
    debug_assert_eq!((*wlv).off, 0);

    // Draw cmdwin char
    let cmdwin_win = nvim_get_cmdwin_win();
    if wp == cmdwin_win {
        let cmdwin_type = nvim_get_cmdwin_type();
        let hl_at = nvim_win_hl_attr(wp, HLF_AT);
        draw_col_fill_impl(wlv, rs_schar_from_ascii(cmdwin_type), 1, hl_at);
    }

    // Compute virt_line_index from filler state
    let mut virt_line_index = virt_line_index_in;
    let mut virt_line_flags = virt_line_flags_in;
    let filler_todo = (*wlv).filler_todo;
    let filler_lines = (*wlv).filler_lines;
    let n_virt_lines = (*wlv).n_virt_lines;
    if filler_todo > 0 {
        let index = filler_todo - (filler_lines - n_virt_lines);
        if index > 0 {
            let vl_size = nvim_kv_size_virt_lines(virt_lines);
            virt_line_index = vl_size - index;
            virt_line_flags = nvim_kv_A_virt_lines_flags(virt_lines, virt_line_index);
        }
    }
    res.virt_line_index = virt_line_index;
    res.virt_line_flags = virt_line_flags;

    // Draw columns
    if virt_line_index >= 0 && (virt_line_flags & K_VL_LEFTCOL) != 0 {
        // skip columns (kVLLeftcol)
    } else if statuscol_draw {
        // Draw 'statuscolumn'
        let v = ptr_col;
        rs_draw_statuscol(
            wp,
            wlv,
            (*wlv).row - startrow - (*wlv).filler_lines,
            col_rows,
            statuscol,
        );
        if nvim_win_get_redr_statuscol(wp) {
            res.action = DRAW_COLS_ACTION_BREAK;
            res.win_col_offset = (*wlv).off;
            res.ptr_offset = v;
            return res;
        }
        if draw_text {
            // ptr_col stays valid as offset; re-fetch line pointer in caller
            res.ptr_offset = v;
        }
    } else {
        // Draw builtin info columns: fold, sign, number
        rs_draw_foldcolumn(wp, wlv);
        let scwidth = nvim_win_get_w_scwidth_drawline(wp);
        for sign_idx in 0..scwidth {
            rs_draw_sign(false, wp, wlv, sign_idx);
        }
        rs_draw_lnum_col(wp, wlv);
    }

    res.win_col_offset = (*wlv).off;

    // When only updating the columns and that's done, stop here.
    if col_rows > 0 {
        rs_wlv_put_linebuf(wp, wlv, (*wlv).off.min(state.view_width), false, bg_attr, 0);

        let need_more = ((*wlv).row + 1 - (*wlv).startrow < col_rows
            && (statuscol_draw
                || nvim_win_hl_attr(wp, HLF_LNA) != nvim_win_hl_attr(wp, HLF_N)
                || nvim_win_hl_attr(wp, HLF_LNB) != nvim_win_hl_attr(wp, HLF_N)))
            || filler_todo > 0;

        if need_more {
            (*wlv).row += 1;
            if (*wlv).row == endrow {
                res.action = DRAW_COLS_ACTION_BREAK;
                return res;
            }
            (*wlv).filler_todo -= 1;
            res.virt_line_index = -1;
            if (*wlv).filler_todo == 0 && (nvim_win_get_botfill(wp) || !draw_text) {
                res.action = DRAW_COLS_ACTION_BREAK;
                return res;
            }
            (*wlv).col = 0;
            (*wlv).off = 0;
            res.action = DRAW_COLS_ACTION_CONTINUE;
        } else {
            res.action = DRAW_COLS_ACTION_BREAK;
        }
        return res;
    }

    // Check if 'breakindent' applies and show it.
    let briopt_sbr = nvim_win_get_briopt_sbr(wp);
    if !briopt_sbr {
        rs_handle_breakindent(wp, wlv);
    }
    rs_handle_showbreak_and_filler(wp, wlv);
    if briopt_sbr {
        rs_handle_breakindent(wp, wlv);
    }

    (*wlv).col = (*wlv).off;
    res.draw_cols = false;
    if filler_todo <= 0 {
        res.leftcols_width = (*wlv).off;
    }
    if has_decor && (*wlv).row == startrow + (*wlv).filler_lines {
        // hide virt_text on text hidden by 'nowrap' or 'smoothscroll'
        let decor_state = get_decor_state().cast::<c_void>();
        decor_redraw_col_impl(wp, ptr_col - 1, (*wlv).off, true, decor_state);
    }
    if (*wlv).col >= state.view_width {
        (*wlv).col = state.view_width;
        (*wlv).off = state.view_width;
        res.action = DRAW_COLS_ACTION_GOTO_END_CHECK;
        return res;
    }

    res
}

// ============================================================================
// Phase 3: Character processing (the `else` branch of the main loop)
// ============================================================================

/// HLF constants used in Phase 3.
const HLF_8: c_int = 1; // SpecialKey / non-printable
const HLF_FL: c_int = 28; // Folded
const HLF_0: c_int = 59; // Whitespace (listchars)

// FFI for Phase 3
extern "C" {
    /// vim_isprintc: return true if c is printable.
    fn nvim_vim_isprintc(c: c_int) -> bool;
    /// vim_isbreak: return true if c is in 'breakat'.
    fn nvim_vim_isbreak(c: c_int) -> c_int;
    /// decor_state.conceal.
    fn nvim_get_decor_state_conceal() -> c_int;
    /// decor_state.spell (as int: kFalse=0, kTrue=1, kNone=-1).
    fn nvim_get_decor_state_spell() -> c_int;
    /// decor_state.conceal_char.
    fn nvim_get_decor_state_conceal_char() -> ScharT;
    /// decor_state.conceal_attr.
    fn nvim_get_decor_state_conceal_attr() -> c_int;
    /// Compute charsize width for linebreak using CharsizeArg (lnum=0).
    fn nvim_c_win_charsize_for_lbr(
        wp: WinHandle,
        line: *const c_char,
        ptr_off: c_int,
        vcol: c_int,
    ) -> c_int;
    /// Get dy_flags (option flags).
    fn nvim_get_dy_flags() -> c_int;
    /// Set spell_redraw_lnum global.
    fn nvim_set_spell_redraw_lnum(lnum: LinenrT);
    /// Get State global.
    fn nvim_get_State() -> c_int;
    /// ascii_iswhite: return true if char is whitespace (space/tab).
    fn nvim_ascii_iswhite(c: c_char) -> bool;
    /// noplainbuffer spelloptions flag check.
    fn nvim_spell_win_noplainbuffer(wp: WinHandle) -> bool;
    /// Get curwin as WinHandle.
    fn nvim_get_curwin() -> WinHandle;
    /// VIsual_active as int (non-zero = true).
    fn nvim_get_VIsual_active() -> c_int;
    /// wp->w_p_lcs_chars.tab1 (exported as nvim_win_get_lcs_tab1 from window crate).
    #[link_name = "nvim_win_get_lcs_tab1"]
    fn nvim_win_lcs_tab1(wp: WinHandle) -> ScharT;
    /// wp->w_p_lcs_chars.tab2.
    fn nvim_win_lcs_tab2(wp: WinHandle) -> ScharT;
    /// wp->w_p_lcs_chars.tab3.
    fn nvim_win_lcs_tab3(wp: WinHandle) -> ScharT;
    /// wp->w_p_lcs_chars.eol.
    fn nvim_win_lcs_eol(wp: WinHandle) -> ScharT;
    /// wp->w_p_lcs_chars.conceal.
    fn nvim_win_lcs_conceal(wp: WinHandle) -> ScharT;
    /// spv->spv_unchanged.
    fn nvim_spv_get_unchanged(spv: *mut c_void) -> bool;
    /// Set spv->spv_checked_col.
    fn nvim_spv_set_checked_col(spv: *mut c_void, val: c_int);
    /// wp->w_p_lcs_chars.multispace[pos] (0 if NULL or at NUL).
    fn nvim_win_lcs_multispace_char_at(wp: WinHandle, pos: c_int) -> ScharT;
    /// wp->w_p_lcs_chars.leadmultispace[pos] (0 if NULL or at NUL).
    fn nvim_win_lcs_leadmultispace_char_at(wp: WinHandle, pos: c_int) -> ScharT;
    /// strlen from C stdlib.
    fn strlen(s: *const c_char) -> usize;
    /// memset from C stdlib.
    fn memset(dst: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    /// memcpy from C stdlib.
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

/// Return value for rs_win_line_process_char.
/// Most state changes go through wls in-place; this carries fields
/// that are not in WinLineState.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProcessCharResult {
    /// Updated mb_schar.
    pub mb_schar: ScharT,
    /// Updated mb_c.
    pub mb_c: c_int,
    /// Updated mb_l.
    pub mb_l: c_int,
    /// Updated ptr byte offset into line.
    pub ptr_col: c_int,
    /// Updated prev_ptr byte offset into line.
    pub prev_ptr_col: c_int,
    /// Whether ptr was decremented (double-width overflow).
    pub did_decrement_ptr: bool,
    /// Updated lcs_eol_todo (may be cleared to false by EOL display).
    pub lcs_eol_todo: bool,
    /// Updated search_attr (may be cleared in linebreak path).
    pub search_attr: c_int,
    /// Updated decor_attr.
    pub decor_attr: c_int,
}

/// Process a character from the buffer line in the main win_line loop.
///
/// This implements the `else` branch (non-n_extra path) of the character
/// processing section. It reads from the buffer via ptr_col offset, applies
/// all list/spell/syntax/conceal transformations, and returns the final
/// character to display plus updated state.
///
/// # Safety
/// All pointers must be valid. `line` must point to the current buffer line.
/// `term_attrs` must point to a TERM_ATTRS_MAX array.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_win_line_process_char(
    wp: WinHandle,
    lnum: LinenrT,
    wlv: *mut WinLineVars,
    wls: *mut WinLineState,
    spv: *mut c_void,
    line: *const c_char,
    ptr_col: c_int,
    trailcol: ColnrT,
    leadcol: ColnrT,
    extmark_attr: c_int,
    lcs_eol_todo: bool,
    search_attr_in: c_int,
    area_attr: c_int,
    term_attrs: *const c_int,
    has_match_conc: c_int,
) -> ProcessCharResult {
    let state = &*wls;
    let view_width = state.view_width;

    // Initialize result with current input values that may be updated.
    let mut res = ProcessCharResult {
        mb_schar: 0,
        mb_c: 0,
        mb_l: 0,
        ptr_col,
        prev_ptr_col: ptr_col,
        did_decrement_ptr: false,
        lcs_eol_todo,
        search_attr: search_attr_in,
        decor_attr: 0,
    };

    let ptr = line.add(ptr_col as usize);

    // First byte of next char.
    let c0 = if ptr.is_null() {
        0
    } else {
        c_int::from(*ptr as u8)
    };
    if c0 == 0 {
        // no more cells to skip
        (*wlv).skip_cells = 0;
    }

    // Get a character from the line itself.
    let mb_l_raw = utfc_ptr2len(ptr);
    let mut mb_c: c_int = 0;
    let mut mb_schar = rs_utfc_ptr2schar(ptr, &raw mut mb_c);
    let mut mb_l = mb_l_raw;

    // Overlong encoded ASCII or ASCII with composing char is displayed normally, except NUL.
    let c0 = if mb_l > 1 && mb_c < 0x80 { mb_c } else { c0 };

    if (mb_l == 1 && c0 >= 0x80)
        || (mb_l >= 1 && mb_c == 0)
        || (mb_l > 1 && !nvim_vim_isprintc(mb_c))
    {
        // Illegal UTF-8 byte: display as <xx>. Non-printable: display as ? or fullwidth ?.
        transchar_hex((*wlv).extra.as_mut_ptr().cast(), mb_c);
        if nvim_win_get_p_rl(wp) != 0 {
            // reverse
            rl_mirror_ascii((*wlv).extra.as_mut_ptr().cast(), std::ptr::null_mut());
        }
        (*wlv).p_extra = (*wlv).extra.as_mut_ptr().cast();
        {
            let mut p_extra_const: *const c_char = (*wlv).p_extra.cast_const();
            mb_c = mb_ptr2char_adv(&raw mut p_extra_const);
            (*wlv).p_extra = p_extra_const.cast_mut();
        }
        mb_schar = rs_schar_from_char(mb_c);
        (*wlv).n_extra = strlen((*wlv).p_extra.cast()) as c_int;
        (*wlv).sc_extra = 0; // NUL
        (*wlv).sc_final = 0; // NUL
        if area_attr == 0 && search_attr_in == 0 {
            (*wlv).n_attr = (*wlv).n_extra + 1;
            (*wlv).extra_attr = nvim_win_hl_attr(wp, HLF_8);
            (*wls).saved_attr2 = (*wlv).char_attr;
        }
    } else if mb_l == 0 {
        // at the NUL at end-of-line
        mb_l = 1;
    }

    // If a double-width char doesn't fit in the last column, display '>'.
    if (*wlv).col >= view_width - 1 && schar_cells(mb_schar) == 2 {
        mb_schar = rs_schar_from_ascii(c_int::from(b'>'));
        mb_c = c_int::from(b'>');
        mb_l = 1;
        (*wls).multi_attr = nvim_win_hl_attr(wp, HLF_AT);
        // Put pointer back so char will be displayed at start of next line.
        res.ptr_col = ptr_col - 1;
        res.did_decrement_ptr = true;
    } else if *ptr != 0 {
        // advance by mb_l - 1 bytes (we will add 1 more below)
        res.ptr_col = ptr_col + mb_l - 1;
    }

    // If a double-width char doesn't fit at the left side, display '<'.
    if (*wlv).skip_cells > 0 && mb_l > 1 && (*wlv).n_extra == 0 {
        (*wlv).n_extra = 1;
        (*wlv).sc_extra = rs_schar_from_ascii(c_int::from(b'<')); // MB_FILLER_CHAR
        (*wlv).sc_final = 0; // NUL
        mb_schar = rs_schar_from_ascii(c_int::from(b' '));
        mb_c = c_int::from(b' ');
        mb_l = 1;
        if area_attr == 0 && search_attr_in == 0 {
            (*wlv).n_attr = (*wlv).n_extra + 1;
            (*wlv).extra_attr = nvim_win_hl_attr(wp, HLF_AT);
            (*wls).saved_attr2 = (*wlv).char_attr;
        }
    }
    // ptr++ equivalent
    res.ptr_col += 1;

    // Remember prev_ptr position (before the ptr++ above, after any mb_l-1 advance)
    // prev_ptr was the original ptr before any advancement
    res.prev_ptr_col = ptr_col;

    res.decor_attr = 0;

    if state.extra_check {
        let no_plain_buffer = nvim_spell_win_noplainbuffer(wp);
        let mut can_spell = !no_plain_buffer;

        // Get extmark and syntax attributes, unless still at the start of the line.
        let v = res.ptr_col; // ptr - line after advancement
        let prev_v = res.prev_ptr_col;

        let mut has_syntax = state.has_syntax;
        let syntax_flags;
        let mut syntax_seqnr = state.syntax_seqnr;

        if has_syntax && v > 0 {
            let save_did_emsg = nvim_get_did_emsg();
            nvim_set_did_emsg(false);

            res.decor_attr = get_syntax_attr(
                v - 1,
                if nvim_spv_get_has_spell(spv) {
                    &raw mut can_spell
                } else {
                    std::ptr::null_mut()
                },
                false,
            );

            if nvim_get_did_emsg() {
                nvim_win_set_syn_error(wp, true);
                has_syntax = false;
            } else {
                nvim_set_did_emsg(save_did_emsg);
            }

            if nvim_win_get_syn_slow(wp) {
                has_syntax = false;
            }

            // Sync syntax state changes back.
            (*wls).has_syntax = has_syntax;

            // Note: line pointer has been re-fetched by caller after this call.
            // We don't refetch here; ptr offsets remain valid.

            // no concealing past the end of the line.
            syntax_flags = if mb_schar == 0 {
                0
            } else {
                get_syntax_info(&raw mut syntax_seqnr)
            };
            (*wls).syntax_flags = syntax_flags;
            (*wls).syntax_seqnr = syntax_seqnr;
        }

        if state.has_decor && v > 0 {
            // extmarks take precedence over syntax.c
            res.decor_attr = hl_combine_attr(res.decor_attr, extmark_attr);
            // decor_conceal is read via nvim_get_decor_state_conceal() at point of use.
            // TRISTATE_TO_BOOL(decor_state.spell, can_spell)
            let ds_spell = nvim_get_decor_state_spell();
            can_spell = if ds_spell == 1 {
                true
            } else if ds_spell == 0 {
                false
            } else {
                can_spell
            };
        }

        let char_attr_base = hl_combine_attr(state.folded_attr, res.decor_attr);
        (*wlv).char_attr = hl_combine_attr(char_attr_base, state.char_attr_pri);
        (*wls).char_attr_base = char_attr_base;

        // Check spelling.
        let v1 = v; // ptr - line
        let word_end = state.word_end;
        let cur_checked_col = state.cur_checked_col;
        if nvim_spv_get_has_spell(spv) && v1 >= word_end && v1 > cur_checked_col {
            let mut spell_attr = 0;
            // do not calculate cap_col at end of line or when only whitespace follows
            if mb_schar != 0 && *skipwhite(line.add(prev_v as usize)) != 0 && can_spell {
                let hlf_count: c_int = 76; // HLF_COUNT
                let mut spell_hlf: c_int = hlf_count;
                let v1_adj = v1 - (mb_l - 1);

                let nextline = state.nextline.as_ptr();
                let nextlinecol = state.nextlinecol;
                let nextline_idx = state.nextline_idx;

                let p: *const c_char = if (prev_v as isize) - (nextlinecol as isize) >= 0 {
                    nextline
                        .add(((prev_v as isize) - (nextlinecol as isize)) as usize)
                        .cast()
                } else {
                    line.add(prev_v as usize)
                };

                let cap_col_adj = nvim_spv_get_cap_col(spv) - prev_v;
                nvim_spv_set_cap_col(spv, cap_col_adj);

                // spell_check modifies cap_col in place via pointer; use a local
                let mut local_cap_col: c_int = nvim_spv_get_cap_col(spv);
                let tmplen = spell_check(
                    wp,
                    p.cast_mut(),
                    &raw mut spell_hlf,
                    &raw mut local_cap_col,
                    nvim_spv_get_unchanged(spv),
                );
                nvim_spv_set_cap_col(spv, local_cap_col);
                // Write back cap_col is now handled above via local_cap_col.
                let word_end_new = v1_adj + tmplen as c_int;
                (*wls).word_end = word_end_new;

                let state_val = nvim_get_State();
                if spell_hlf != hlf_count
                    && (state_val & MODE_INSERT) != 0
                    && nvim_win_get_cursor_lnum(wp) == lnum
                    && nvim_win_get_cursor_col(wp) >= prev_v as ColnrT
                    && nvim_win_get_cursor_col(wp) < word_end_new as ColnrT
                {
                    spell_hlf = hlf_count;
                    nvim_set_spell_redraw_lnum(lnum);
                }

                if spell_hlf == hlf_count
                    && !std::ptr::eq(p, line.add(prev_v as usize))
                    && (p.offset_from(nextline.cast::<c_char>()) as c_int) + tmplen as c_int
                        > nextline_idx
                {
                    nvim_spv_set_checked_lnum(spv, lnum + 1);
                    let checked_col = (p.offset_from(nextline.cast::<c_char>()) as c_int)
                        + tmplen as c_int
                        - nextline_idx;
                    nvim_spv_set_checked_col(spv, checked_col);
                }

                if spell_hlf != hlf_count {
                    spell_attr = nvim_get_highlight_attr_hlf(spell_hlf);
                }

                let cap_col = nvim_spv_get_cap_col(spv);
                if cap_col > 0 {
                    if !std::ptr::eq(p, line.add(prev_v as usize))
                        && (p.offset_from(nextline.cast::<c_char>()) as c_int) + cap_col
                            >= nextline_idx
                    {
                        nvim_spv_set_capcol_lnum(spv, lnum + 1);
                        let new_cap_col = (p.offset_from(nextline.cast::<c_char>()) as c_int)
                            + cap_col
                            - nextline_idx;
                        nvim_spv_set_cap_col(spv, new_cap_col);
                    } else {
                        nvim_spv_set_cap_col(spv, cap_col + prev_v);
                    }
                }

                (*wls).spell_attr = spell_attr;
            } else {
                (*wls).spell_attr = 0;
            }
            let spell_attr = (*wls).spell_attr;
            if spell_attr != 0 {
                let char_attr_base = hl_combine_attr((*wls).char_attr_base, spell_attr);
                (*wlv).char_attr = hl_combine_attr(char_attr_base, (*wls).char_attr_pri);
                (*wls).char_attr_base = char_attr_base;
            }
        }

        // Terminal: combine terminal attributes.
        if nvim_win_buf_has_terminal_drawline(wp) {
            let vcol = (*wlv).vcol;
            let term_attr = if vcol < 1024 {
                *term_attrs.add(vcol as usize)
            } else {
                0
            };
            (*wlv).char_attr = hl_combine_attr(term_attr, (*wlv).char_attr);
        }

        // linebreak: only allow once we have found chars not in 'breakat'.
        if nvim_win_get_p_lbr(wp) != 0
            && !(*wlv).need_lbr
            && mb_schar != 0
            && nvim_vim_isbreak(c_int::from(*line.add(res.ptr_col as usize) as u8)) == 0
        {
            (*wlv).need_lbr = true;
        }
        // Found last space before word: check for line break.
        if nvim_win_get_p_lbr(wp) != 0
            && c0 == mb_c
            && mb_c < 128
            && (*wlv).need_lbr
            && nvim_vim_isbreak(mb_c) != 0
            && nvim_vim_isbreak(c_int::from(*line.add(res.ptr_col as usize) as u8)) == 0
        {
            let mb_off = utf_head_off(line, line.add(res.ptr_col as usize - 1));
            let p_off = res.ptr_col - 1 - mb_off;
            let charsize_w = nvim_c_win_charsize_for_lbr(wp, line, p_off, (*wlv).vcol);
            (*wlv).n_extra = charsize_w - 1;

            if (*wls).on_last_col && mb_c != c_int::from(b'\t') {
                // Do not continue search/match highlighting over line break,
                // but for TABs highlighting should include complete width.
                res.search_attr = 0;
            }

            if mb_c == c_int::from(b'\t') && (*wlv).n_extra + (*wlv).col > view_width {
                let buf = nvim_win_get_w_buffer(wp);
                let ts = nvim_buf_get_p_ts(buf);
                let vts = nvim_buf_get_p_vts_array(buf);
                (*wlv).n_extra = rs_tabstop_padding((*wlv).vcol, ts, vts) - 1;
            }
            (*wlv).sc_extra = rs_schar_from_ascii(if mb_off > 0 {
                c_int::from(b'<')
            } else {
                c_int::from(b' ')
            });
            (*wlv).sc_final = 0; // NUL
            if mb_c < 128 && nvim_ascii_iswhite(mb_c as c_char) {
                if mb_c == c_int::from(b'\t') {
                    // Tab alignment.
                    fix_for_boguscols_impl(wlv);
                }
                if nvim_win_get_p_list(wp) == 0 {
                    mb_c = c_int::from(b' ');
                    mb_schar = rs_schar_from_ascii(mb_c);
                }
            }
        }

        // Update in_multispace tracking.
        if nvim_win_get_p_list(wp) != 0 {
            let next_char = c_int::from(*line.add(res.ptr_col as usize) as u8);
            let prev_char = if ptr_col > 0 {
                c_int::from(*line.add(ptr_col as usize - 1) as u8)
            } else {
                -1
            };
            let in_multispace = mb_c == c_int::from(b' ')
                && (next_char == c_int::from(b' ')
                    || (ptr_col > 0 && prev_char == c_int::from(b' ')));
            (*wls).in_multispace = in_multispace;
            if !in_multispace {
                (*wls).multispace_pos = 0;
            }
        }

        // 'list': Change char 160/nbsp/space to listchars equivalents.
        let p_list = nvim_win_get_p_list(wp) != 0;
        let ptr_after = res.ptr_col; // ptr after increment

        // nbsp/space listchar substitution
        let lcs_nbsp = nvim_win_lcs_nbsp(wp);
        let lcs_space = nvim_win_lcs_space(wp);
        let lcs_ms = nvim_win_lcs_multispace_char_at(wp, 0);

        if p_list
            && (((mb_c == 160 && mb_l == 2) || (mb_c == 0x202f && mb_l == 3)) && lcs_nbsp != 0
                || (mb_c == c_int::from(b' ')
                    && mb_l == 1
                    && (lcs_space != 0 || ((*wls).in_multispace && lcs_ms != 0))
                    && ptr_after as ColnrT >= leadcol
                    && ptr_after as ColnrT <= trailcol))
        {
            if (*wls).in_multispace && lcs_ms != 0 {
                let pos = (*wls).multispace_pos;
                mb_schar = nvim_win_lcs_multispace_char_at(wp, pos);
                let new_pos = pos + 1;
                if nvim_win_lcs_multispace_char_at(wp, new_pos) == 0 {
                    (*wls).multispace_pos = 0;
                } else {
                    (*wls).multispace_pos = new_pos;
                }
            } else {
                mb_schar = if mb_c == c_int::from(b' ') {
                    lcs_space
                } else {
                    lcs_nbsp
                };
            }
            (*wlv).n_attr = 1;
            (*wlv).extra_attr = nvim_win_hl_attr(wp, HLF_0);
            (*wls).saved_attr2 = (*wlv).char_attr;
            mb_c = schar_get_first_codepoint(mb_schar);
        }

        // trail/lead listchar substitution.
        let lcs_trail = nvim_win_lcs_trail(wp);
        let lcs_lead = nvim_win_lcs_lead(wp);
        let lcs_leadms = nvim_win_lcs_leadmultispace_char_at(wp, 0);

        if mb_c == c_int::from(b' ')
            && mb_l == 1
            && ((trailcol != ColnrT::MAX && ptr_after as ColnrT > trailcol)
                || (leadcol != 0 && (ptr_after as ColnrT) < leadcol))
        {
            if leadcol != 0
                && (*wls).in_multispace
                && (ptr_after as ColnrT) < leadcol
                && lcs_leadms != 0
            {
                let pos = (*wls).multispace_pos;
                mb_schar = nvim_win_lcs_leadmultispace_char_at(wp, pos);
                let new_pos = pos + 1;
                if nvim_win_lcs_leadmultispace_char_at(wp, new_pos) == 0 {
                    (*wls).multispace_pos = 0;
                } else {
                    (*wls).multispace_pos = new_pos;
                }
            } else if ptr_after as ColnrT > trailcol && lcs_trail != 0 {
                mb_schar = lcs_trail;
            } else if (ptr_after as ColnrT) < leadcol && lcs_lead != 0 {
                mb_schar = lcs_lead;
            } else if leadcol != 0 && lcs_space != 0 {
                mb_schar = lcs_space;
            }
            (*wlv).n_attr = 1;
            (*wlv).extra_attr = nvim_win_hl_attr(wp, HLF_0);
            (*wls).saved_attr2 = (*wlv).char_attr;
            mb_c = schar_get_first_codepoint(mb_schar);
        }
    } // end extra_check

    // Handling of non-printable characters.
    if !nvim_vim_isprintc(mb_c) {
        if mb_c == c_int::from(b'\t')
            && (nvim_win_get_p_list(wp) == 0 || nvim_win_lcs_tab1(wp) != 0)
        {
            // Tab handling.
            let mut tab_len: c_int;
            let mut vcol_adjusted = (*wlv).vcol; // removed showbreak length
            let sbr = rs_get_showbreak_value(wp);
            if *sbr != 0 && (*wlv).vcol == (*wlv).vcol_sbr && nvim_win_get_p_wrap(wp) != 0 {
                vcol_adjusted = (*wlv).vcol - mb_charlen(sbr);
            }

            let buf = nvim_win_get_w_buffer(wp);
            let ts = nvim_buf_get_p_ts(buf);
            let vts = nvim_buf_get_p_vts_array(buf);
            tab_len = rs_tabstop_padding(vcol_adjusted, ts, vts) - 1;

            if nvim_win_get_p_lbr(wp) == 0 || nvim_win_get_p_list(wp) == 0 {
                (*wlv).n_extra = tab_len;
            } else {
                let saved_nextra = (*wlv).n_extra;
                let vc_off = (*wlv).vcol_off_co;

                if vc_off > 0 {
                    tab_len += vc_off;
                }
                let lcs_tab1 = nvim_win_lcs_tab1(wp);
                if lcs_tab1 != 0 && (*wlv).old_boguscols > 0 && (*wlv).n_extra > tab_len {
                    tab_len += (*wlv).n_extra - tab_len;
                }

                if tab_len > 0 {
                    let lcs_tab2 = nvim_win_lcs_tab2(wp);
                    let lcs_tab3 = nvim_win_lcs_tab3(wp);
                    let tab2_len = schar_len(lcs_tab2);
                    let mut len = (tab_len as usize) * tab2_len;
                    if lcs_tab3 != 0 {
                        len += schar_len(lcs_tab3) - tab2_len;
                    }
                    if saved_nextra > 0 {
                        len += (saved_nextra - tab_len) as usize;
                    }
                    let p = rs_get_extra_buf(len + 1);
                    memset(p.cast(), c_int::from(b' '), len);
                    *p.add(len) = 0;
                    (*wlv).p_extra = p;
                    let mut i = 0;
                    while i < tab_len {
                        if *(*wlv).p_extra == 0 {
                            tab_len = i;
                            break;
                        }
                        let lcs = if lcs_tab3 != 0 && i == tab_len - 1 {
                            lcs_tab3
                        } else {
                            lcs_tab2
                        };
                        let slen = schar_get_adv(&raw mut (*wlv).p_extra, lcs);
                        let decr = i32::from(saved_nextra > 0);
                        (*wlv).n_extra += slen as c_int - decr;
                        i += 1;
                    }
                    if vc_off > 0 {
                        (*wlv).n_extra -= vc_off;
                    }
                }
            }

            {
                let vc_saved = (*wlv).vcol_off_co;
                fix_for_boguscols_impl(wlv);
                let lcs_tab1 = nvim_win_lcs_tab1(wp);
                if (*wlv).n_extra == tab_len + vc_saved
                    && nvim_win_get_p_list(wp) != 0
                    && lcs_tab1 != 0
                {
                    tab_len += vc_saved;
                }
            }

            if nvim_win_get_p_list(wp) != 0 {
                let lcs_tab1 = nvim_win_lcs_tab1(wp);
                let lcs_tab3 = nvim_win_lcs_tab3(wp);
                let lcs_tab2 = nvim_win_lcs_tab2(wp);
                mb_schar = if (*wlv).n_extra == 0 && lcs_tab3 != 0 {
                    lcs_tab3
                } else {
                    lcs_tab1
                };
                if nvim_win_get_p_lbr(wp) != 0 && !(*wlv).p_extra.is_null() && *(*wlv).p_extra != 0
                {
                    (*wlv).sc_extra = 0; // using p_extra from above
                } else {
                    (*wlv).sc_extra = lcs_tab2;
                }
                (*wlv).sc_final = lcs_tab3;
                (*wlv).n_attr = tab_len + 1;
                (*wlv).extra_attr = nvim_win_hl_attr(wp, HLF_0);
                (*wls).saved_attr2 = (*wlv).char_attr;
            } else {
                (*wlv).sc_final = 0; // NUL
                (*wlv).sc_extra = rs_schar_from_ascii(c_int::from(b' '));
                mb_schar = rs_schar_from_ascii(c_int::from(b' '));
            }
            mb_c = schar_get_first_codepoint(mb_schar);
        } else if mb_schar == 0
            && (nvim_win_get_p_list(wp) != 0
                || (((*wlv).fromcol >= 0 || (*wls).fromcol_prev >= 0)
                    && (*wlv).tocol > (*wlv).vcol
                    && nvim_get_VIsual_mode() != c_int::from(b'\x16') // Ctrl_V
                    && (*wlv).col < view_width
                    && !((*wls).noinvcur
                        && lnum == nvim_win_get_cursor_lnum(wp)
                        && (*wlv).vcol == nvim_win_get_virtcol(wp))))
            && res.lcs_eol_todo
            && nvim_win_lcs_eol(wp) != 0
        {
            // Display a '$' after the line or highlight an extra character.
            if (*wlv).diff_hlf == 0 && (*wlv).line_attr == 0 && (*wlv).line_attr_lowprio == 0 {
                // In virtualedit, visual selections may extend beyond end of line
                if !(state.area_highlighting
                    && nvim_win_virtual_active_drawline(wp)
                    && (*wlv).tocol != ColnrT::MAX
                    && (*wlv).vcol < (*wlv).tocol)
                {
                    (*wlv).p_extra = c"".as_ptr().cast_mut();
                }
                (*wlv).n_extra = 0;
            }
            let lcs_eol = nvim_win_lcs_eol(wp);
            if nvim_win_get_p_list(wp) != 0 && lcs_eol > 0 {
                mb_schar = lcs_eol;
            } else {
                mb_schar = rs_schar_from_ascii(c_int::from(b' '));
            }
            res.lcs_eol_todo = false;
            res.ptr_col -= 1; // put it back at the NUL
            (*wlv).extra_attr = nvim_win_hl_attr(wp, HLF_AT);
            (*wlv).n_attr = 1;
            mb_c = schar_get_first_codepoint(mb_schar);
        } else if mb_schar != 0 {
            // Non-printable, non-TAB, non-NUL: display as hex or transchar.
            (*wlv).p_extra = transchar_buf(nvim_win_get_w_buffer(wp).0, mb_c);
            if (*wlv).n_extra == 0 {
                (*wlv).n_extra = byte2cells(mb_c) - 1;
            }
            if (nvim_get_dy_flags() & 0x100) != 0 && nvim_win_get_p_rl(wp) != 0 {
                // (dy_flags & kOptDyFlagUhex) = 0x100
                rl_mirror_ascii((*wlv).p_extra, std::ptr::null_mut());
            }
            (*wlv).sc_extra = 0; // NUL
            (*wlv).sc_final = 0; // NUL
            if nvim_win_get_p_lbr(wp) != 0 {
                mb_c = c_int::from(*(*wlv).p_extra as u8);
                let p = rs_get_extra_buf(((*wlv).n_extra + 1) as usize);
                memset(p.cast(), c_int::from(b' '), (*wlv).n_extra as usize);
                let pextra_str_len = strlen((*wlv).p_extra.cast());
                if pextra_str_len > 1 {
                    memcpy(p.cast(), (*wlv).p_extra.add(1).cast(), pextra_str_len - 1);
                }
                *p.add((*wlv).n_extra as usize) = 0;
                (*wlv).p_extra = p;
            } else {
                (*wlv).n_extra = byte2cells(mb_c) - 1;
                mb_c = c_int::from(*(*wlv).p_extra as u8);
                (*wlv).p_extra = (*wlv).p_extra.add(1);
            }
            (*wlv).n_attr = (*wlv).n_extra + 1;
            (*wlv).extra_attr = nvim_win_hl_attr(wp, HLF_8);
            (*wls).saved_attr2 = (*wlv).char_attr;
            mb_schar = rs_schar_from_ascii(mb_c);
        } else if nvim_get_VIsual_active() != 0
            && (nvim_get_VIsual_mode() == c_int::from(b'v')
                || nvim_get_VIsual_mode() == c_int::from(b'\x16')) // Ctrl_V
            && nvim_win_virtual_active_drawline(wp)
            && (*wlv).tocol != ColnrT::MAX
            && (*wlv).vcol < (*wlv).tocol
            && (*wlv).col < view_width
        {
            mb_c = c_int::from(b' ');
            mb_schar = rs_schar_from_char(mb_c);
            res.ptr_col -= 1; // put it back at the NUL
        }
    } // end non-printable handling

    // Concealment
    if nvim_win_get_p_cole(wp) > 0
        && (wp != nvim_get_curwin()
            || lnum != nvim_win_get_cursor_lnum(wp)
            || conceal_cursor_line(wp))
        && (state.syntax_flags & 0x04 != 0 // HL_CONCEAL
            || has_match_conc > 0
            || nvim_get_decor_state_conceal() > 0)
        && !((*wls).lnum_in_visual_area && {
            let cocu = nvim_win_get_p_cocu(wp);
            let v_ptr = {
                extern "C" {
                    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
                }
                vim_strchr(cocu, c_int::from(b'v'))
            };
            v_ptr.is_null()
        })
    {
        (*wlv).char_attr = state.conceal_attr;
        let prev_syntax_id = state.prev_syntax_id;
        if ((prev_syntax_id != state.syntax_seqnr && (state.syntax_flags & 0x04) != 0)
            || has_match_conc > 1
            || nvim_get_decor_state_conceal() > 1)
            && (syn_get_sub_char() != 0
                || (has_match_conc != 0 && (*wls).match_conc != 0)
                || (nvim_get_decor_state_conceal() != 0
                    && nvim_get_decor_state_conceal_char() != 0)
                || nvim_win_get_p_cole(wp) == 1)
            && nvim_win_get_p_cole(wp) != 3
        {
            if schar_cells(mb_schar) > 1 {
                // Double-width concealed char: advance one more virtual column.
                (*wlv).n_extra += 1;
            }
            if has_match_conc != 0 && (*wls).match_conc != 0 {
                mb_schar = rs_schar_from_char((*wls).match_conc);
            } else if nvim_get_decor_state_conceal() != 0
                && nvim_get_decor_state_conceal_char() != 0
            {
                mb_schar = nvim_get_decor_state_conceal_char();
                let conceal_attr = nvim_get_decor_state_conceal_attr();
                if conceal_attr != 0 {
                    (*wlv).char_attr = conceal_attr;
                }
            } else if syn_get_sub_char() != 0 {
                mb_schar = rs_schar_from_char(syn_get_sub_char());
            } else if nvim_win_lcs_conceal(wp) != 0 {
                mb_schar = nvim_win_lcs_conceal(wp);
            } else {
                mb_schar = rs_schar_from_ascii(c_int::from(b' '));
            }
            mb_c = schar_get_first_codepoint(mb_schar);
            (*wls).prev_syntax_id = state.syntax_seqnr;

            let n_extra = (*wlv).n_extra;
            if n_extra > 0 {
                (*wlv).vcol_off_co += n_extra;
            }
            (*wlv).vcol += n_extra;
            if state.is_wrapped && n_extra > 0 {
                (*wlv).boguscols += n_extra;
                (*wlv).col += n_extra;
            }
            (*wlv).n_extra = 0;
            (*wlv).n_attr = 0;
        } else if (*wlv).skip_cells == 0 {
            (*wls).is_concealing = true;
            (*wlv).skip_cells = 1;
        }
    } else {
        (*wls).prev_syntax_id = 0;
        (*wls).is_concealing = false;
    }

    if (*wlv).skip_cells > 0 && res.did_decrement_ptr {
        // not showing the '>'; put pointer back to avoid getting stuck
        res.ptr_col += 1;
    }

    res.mb_schar = mb_schar;
    res.mb_c = mb_c;
    res.mb_l = mb_l;
    res
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
