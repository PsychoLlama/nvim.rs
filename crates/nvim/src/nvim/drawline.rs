use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::buffer::bt_quickfix;
use crate::src::nvim::buffer::buf_meta_total;
use crate::src::nvim::charset::vim_isbreak;
use crate::src::nvim::charset::{
    byte2cells, rl_mirror_ascii, skiptowhite, skipwhite, transchar_buf, transchar_hex,
    transstr_buf, vim_isprintc,
};
use crate::src::nvim::cursor::get_cursor_rel_lnum;
use crate::src::nvim::cursor_shape::cursor_is_block_during_visual;
use crate::src::nvim::decoration::decor_redraw_col;
use crate::src::nvim::decoration::{
    clear_virttext, decor_has_more_decorations, decor_init_draw_col, decor_range_at,
    decor_range_count, decor_recheck_draw_col, decor_redraw_eol, decor_redraw_line,
    decor_redraw_signs, decor_virt_lines, decor_virt_pos, decor_virt_pos_kind,
    next_virt_text_chunk,
};
use crate::src::nvim::decoration_provider::{
    decor_providers_invoke_line, decor_providers_invoke_range,
};
use crate::src::nvim::diff::{diff_change_parse, diff_check_with_linestatus, diff_find_change};
use crate::src::nvim::drawscreen::{
    compute_foldcolumn, conceal_cursor_line, number_width, win_draw_end,
};
use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::fold::get_foldtext;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{
    LineAttrs, LineSpan, grid_adjust, grid_put_linebuf, linebuf_mirror, schar_cells,
    schar_from_ascii, schar_from_char, schar_get_adv, schar_get_ascii, schar_get_first_codepoint,
    schar_len,
};
use crate::src::nvim::highlight::win_hl_attr;
use crate::src::nvim::highlight::{
    hl_blend_attrs, hl_combine_attr, hl_get_underline, syn_attr2entry, win_bg_attr,
};
use crate::src::nvim::highlight_group::{
    HLF_0, HLF_8, HLF_ADD, HLF_AT, HLF_CHD, HLF_CLF, HLF_CLN, HLF_CLS, HLF_CONCEAL, HLF_COUNT,
    HLF_CUC, HLF_CUL, HLF_DED, HLF_FC, HLF_FL, HLF_I, HLF_LNA, HLF_LNB, HLF_MC, HLF_N, HLF_NONE,
    HLF_QFL, HLF_SC, HLF_TXA, HLF_TXD, HLF_V, syn_id2attr,
};
use crate::src::nvim::indent::{get_breakindent_win, tabstop_padding};
use crate::src::nvim::insexpand::{
    ins_compl_col_range_attr, ins_compl_lnum_in_range, ins_compl_win_active,
};
use crate::src::nvim::main::{
    State, VIsual, VIsual_active, VIsual_mode, cmdwin_type, cmdwin_win, cterm_normal_bg_color,
    curwin, decor_state, did_emsg, dollar_vcol, dy_flags, highlight_attr, highlight_match,
    hl_attr_active, linebuf_attr, linebuf_char, linebuf_vcol, normal_bg, p_cpo, p_sel,
    screen_search_hl, search_match_endcol, search_match_lines, spell_redraw_lnum, win_extmark_arr,
};
use crate::src::nvim::r#match::{
    get_prevcol_hl_flag, get_search_match_hl, prepare_search_hl_line, update_search_hl,
};
use crate::src::nvim::mbyte::{
    mb_charlen, mb_off_next, mb_ptr2char_adv, mb_string2cells, utf_head_off, utf_ptr2CharInfo,
    utf_ptr2StrCharInfo, utf_ptr2cells, utfc_next, utfc_ptr2len, utfc_ptr2schar,
};
use crate::src::nvim::memline::{gchar_pos, ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{xfree, xmalloc, xrealloc};
use crate::src::nvim::r#move::{set_empty_rows, validate_virtcol, win_col_off, win_col_off2};
use crate::src::nvim::option::{get_showbreak_value, kOptFlagInsecure};
use crate::src::nvim::options::{
    kOptCuloptFlagLine, kOptCuloptFlagNumber, kOptCuloptFlagScreenline, kOptDyFlagUhex,
    kOptSpoFlagNoplainbuffer,
};
use crate::src::nvim::os::libc::{__assert_fail, abs, memcpy, memset, snprintf, strlen};
use crate::src::nvim::plines::{getvcol, getvvcol, init_charsize_arg, win_charsize};
use crate::src::nvim::pos::ltoreq;
use crate::src::nvim::quickfix::qf_current_entry;
use crate::src::nvim::search::FORWARD;
use crate::src::nvim::spell::{
    check_need_cap, spell_cat_line, spell_check, spell_move_to, spell_to_word_end,
};
use crate::src::nvim::state::{MODE_INSERT, virtual_active};
use crate::src::nvim::statusline::build_statuscol_str;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::syntax::{
    HL_CONCEAL, get_syntax_attr, get_syntax_info, syn_get_sub_char, syntax_present, syntax_start,
};
use crate::src::nvim::terminal::terminal_get_line_attributes;
use crate::src::nvim::types::{
    CharSize, CharsizeArg, CharsizeKind, DecorRange, DecorRangeKind, DecorVirtText, GridView,
    HlAttrs, MetaIndex, NS, OptInt, RgbValue, ScreenGrid, SignTextAttrs, StlFlag, TriState,
    VimVarIndex, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinExtmark, buf_T, colnr_T,
    diffline_T, foldinfo_T, hlf_T, linenr_T, pos_T, ptrdiff_t, sattr_T, schar_T, size_t, smt_T,
    spellvars_T, ssize_t, statuscol_T, uint8_t, uint32_t, uint64_t, varnumber_T, virt_line, win_T,
};
use crate::src::nvim::ui::ui_rgb_attached;

// The carve of the transpiled module; see each child's docs.
mod state;
pub use self::state::*;
mod columns;
pub use self::columns::*;
mod virttext;
pub(crate) use self::virttext::*;
mod prologue;
pub(crate) use self::prologue::*;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed = 2147483647;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const SIGN_WIDTH: C2Rust_Unnamed_0 = 2;
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kVLScroll: C2Rust_Unnamed_14 = 2;
pub const kVLLeftcol: C2Rust_Unnamed_14 = 1;
pub type HlMode = ::core::ffi::c_uint;
pub const kHlModeBlend: HlMode = 3;
pub const kHlModeCombine: HlMode = 2;
pub const kHlModeReplace: HlMode = 1;
pub const kHlModeUnknown: HlMode = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const kVTRepeatLinebreak: C2Rust_Unnamed_15 = 8;
pub const kMTMetaInline: MetaIndex = 0;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const FOLD_TEXT_LEN: C2Rust_Unnamed_19 = 51;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const SIGN_SHOW_MAX: C2Rust_Unnamed_20 = 9;
pub const STL_SIGNCOL: StlFlag = 115;
pub const STL_FOLDCOL: StlFlag = 67;
/// A `DecorRange` that draws virtual text of its own.
pub const kDecorKindVirtText: DecorRangeKind = 2;
/// A `DecorRange` that draws nothing and reports its position to the UI.
pub const kDecorKindUIWatched: DecorRangeKind = 4;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const TERM_ATTRS_MAX: C2Rust_Unnamed_29 = 1024;
pub const SLF_WRAP: C2Rust_Unnamed_32 = 2;
pub const SLF_RIGHTLEFT: C2Rust_Unnamed_32 = 1;
pub const SLF_INC_VCOL: C2Rust_Unnamed_32 = 4;
pub const VV_VIRTNUM: VimVarIndex = 103;
pub const SMT_ALL: smt_T = 0;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const VIRTTEXT_EMPTY: VirtText = VirtText {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<VirtTextChunk>(),
};
pub const CPO_NUMCOL: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const MAX_NUMBERWIDTH: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const SCL_NUM: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const VALID_CHEIGHT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VALID_CROW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub unsafe extern "C" fn win_line(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut startrow: ::core::ffi::c_int,
    mut endrow: ::core::ffi::c_int,
    mut col_rows: ::core::ffi::c_int,
    mut concealed: bool,
    mut spv: *mut spellvars_T,
    mut foldinfo: foldinfo_T,
) -> ::core::ffi::c_int {
    let mut vcol_prev: colnr_T = -1 as colnr_T;
    let mut grid: *mut GridView = &raw mut (*wp).w_grid;
    let mut saved_attr2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n_attr3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut saved_attr3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut char_attr_pri: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut char_attr_base: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut area_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut vcol_save_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut decor_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut folded_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut eol_hl_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut multi_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut mb_l: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut mb_c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut mb_schar: schar_T = 0 as schar_T;
    let mut n_extra_next: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut extra_attr_next: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut saved_search_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut saved_area_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut saved_decor_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut saved_search_attr_from_match: bool = false_0 != 0;
    let mut win_col_offset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut area_active: bool = false_0 != 0;
    let mut decor_need_recheck: bool = false_0 != 0;
    let mut buf_fold: [::core::ffi::c_char; 51] = [0; 51];
    let mut fold_vt: VirtText = VIRTTEXT_EMPTY;
    let mut foldtext_free: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut match_conc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut on_last_col: bool = false_0 != 0;
    let mut syntax_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut syntax_seqnr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev_syntax_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_concealing: bool = false_0 != 0;
    let mut did_wcol: bool = false_0 != 0;
    '_c2rust_label: {
        if startrow < endrow {
        } else {
            __assert_fail(
                b"startrow < endrow\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/drawline.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1168 as ::core::ffi::c_uint,
                b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut wlv: WinLineVars = WinLineVars {
        lnum: lnum,
        foldinfo: foldinfo,
        startrow: startrow,
        row: startrow,
        vcol: 0,
        col: 0,
        boguscols: 0,
        old_boguscols: 0 as ::core::ffi::c_int,
        vcol_off_co: 0,
        off: 0,
        cursorline_attr: 0,
        line_attr: 0,
        line_attr_lowprio: 0,
        sign_num_attr: 0,
        prev_num_attr: -1 as ::core::ffi::c_int,
        sign_cul_attr: 0,
        fromcol: -10 as ::core::ffi::c_int,
        tocol: MAXCOL as ::core::ffi::c_int,
        showbreak_vcol: -1 as colnr_T,
        need_showbreak: false,
        char_attr: 0,
        n_extra: 0,
        n_attr: 0,
        p_extra: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        extra_attr: 0,
        sc_extra: 0,
        sc_final: 0,
        extra_for_extmark: false,
        extra: [0; 11],
        diff_hlf: HLF_NONE,
        n_virt_lines: 0,
        n_virt_below: 0,
        filler_lines: 0,
        filler_todo: 0,
        sign_attrs: [SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        }; 9],
        need_lbr: false,
        virt_inline: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        virt_inline_i: 0,
        virt_inline_hl_mode: kHlModeUnknown,
        reset_extra_attr: false,
        skip_cells: 0,
        skipped_cells: 0,
        color_cols: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    };
    let buf: *mut buf_T = (*wp).w_buffer;
    let mut nextline: SpellLookahead = [0; SPWORDLEN as usize * 2];
    let LineSetup {
        view_width,
        view_height,
        in_curline,
        has_fold,
        has_foldtext,
        is_wrapped,
        draw_text,
        start_vcol,
        bg_attr,
        conceal_attr,
        may_have_inline_virt,
        has_terminal,
        mut line,
        mut ptr,
        trailcol,
        leadcol,
        lcs_eol,
        mut lcs_prec_todo,
        mut in_multispace,
        mut multispace_pos,
        mut area_highlighting,
        mut extra_check,
        mut has_syntax,
        mut has_decor,
        vi_attr,
        mut search_attr,
        mut search_attr_from_match,
        noinvcur,
        fromcol_prev,
        lnum_in_visual_area,
        cul_screenline,
        left_curline_col,
        right_curline_col,
        line_attr_save,
        line_attr_lowprio_save,
        mut line_changes,
        mut change_index,
        mut change_start,
        mut change_end,
        mut statuscol,
        mut virt_lines,
        check_decor_providers,
        mut decor_provider_end_col,
        nextlinecol,
        nextline_idx,
        mut spell_attr,
        mut word_end,
        cur_checked_col,
    } = prepare_line(
        &mut wlv,
        wp,
        endrow,
        col_rows,
        concealed,
        spv,
        &mut nextline,
    );
    // Zeroed here rather than in the setup half so that it is zeroed only
    // for a `:terminal` buffer; see `LineSetup::has_terminal`.
    let mut term_attrs: [::core::ffi::c_int; TERM_ATTRS_MAX as usize] =
        [0; TERM_ATTRS_MAX as usize];
    if has_terminal {
        terminal_get_line_attributes(
            (*(*wp).w_buffer).terminal,
            wp,
            lnum,
            term_attrs.as_mut_ptr(),
        );
    }
    statuscol.sattrs = &raw mut wlv.sign_attrs as *mut SignTextAttrs;
    let mut lcs_eol_todo: bool = true_0 != 0;
    let mut draw_cols: bool = true_0 != 0;
    let mut leftcols_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut virt_line_index: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut virt_line_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut draw_folded: bool = false;
    let mut extmark_attr: ::core::ffi::c_int = 0;
    let mut lcs_ext: schar_T = 0;
    's_5143: loop {
        let mut has_match_conc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut decor_conceal: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut did_decrement_ptr: bool = false_0 != 0;
        if check_decor_providers as ::core::ffi::c_int != 0
            && ptr.offset_from(line) as ::core::ffi::c_int >= decor_provider_end_col
        {
            let col_0: ::core::ffi::c_int = ptr.offset_from(line) as ::core::ffi::c_int;
            decor_provider_end_col = invoke_range_next(
                wp,
                lnum as ::core::ffi::c_int,
                col_0 as colnr_T,
                100 as colnr_T,
            );
            line = ml_get_buf((*wp).w_buffer, lnum);
            ptr = line.offset(col_0 as isize);
            if !has_decor
                && decor_has_more_decorations(
                    decor_state.ptr(),
                    lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
            {
                has_decor = true_0 != 0;
                extra_check = true_0 != 0;
            }
        }
        '_end_check: {
            if draw_cols {
                if cul_screenline {
                    wlv.cursorline_attr = 0 as ::core::ffi::c_int;
                    wlv.line_attr = line_attr_save;
                    wlv.line_attr_lowprio = line_attr_lowprio_save;
                }
                '_c2rust_label_1: {
                    if wlv.off == 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"wlv.off == 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/drawline.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            1726 as ::core::ffi::c_uint,
                            b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if wp == cmdwin_win.get() {
                    wlv.draw_col_fill(cmdwin_type.get() as schar_T, 1, win_hl_attr(wp, HLF_AT));
                }
                if wlv.filler_todo > 0 as ::core::ffi::c_int {
                    let mut index: ::core::ffi::c_int =
                        wlv.filler_todo - (wlv.filler_lines - wlv.n_virt_lines);
                    if index > 0 as ::core::ffi::c_int {
                        virt_line_index = virt_lines.size as ::core::ffi::c_int - index;
                        '_c2rust_label_2: {
                            if virt_line_index >= 0 as ::core::ffi::c_int {
                            } else {
                                __assert_fail(
                                    b"virt_line_index >= 0\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/drawline.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    1737 as ::core::ffi::c_uint,
                                    b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        virt_line_flags =
                            (*virt_lines.items.offset(virt_line_index as isize)).flags;
                    }
                }
                if !(virt_line_index >= 0 as ::core::ffi::c_int
                    && virt_line_flags & kVLLeftcol as ::core::ffi::c_int != 0)
                {
                    if statuscol.draw {
                        let v_0: ::core::ffi::c_int = ptr.offset_from(line) as ::core::ffi::c_int;
                        wlv.draw_statuscol(
                            wp,
                            wlv.row - startrow - wlv.filler_lines,
                            col_rows,
                            &raw mut statuscol,
                        );
                        if (*wp).w_redr_statuscol {
                            break 's_5143;
                        }
                        if draw_text {
                            line = ml_get_buf((*wp).w_buffer, lnum);
                            ptr = line.offset(v_0 as isize);
                        }
                    } else {
                        wlv.draw_foldcolumn(wp);
                        let mut sign_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while sign_idx < (*wp).w_scwidth {
                            wlv.draw_sign(false, wp, sign_idx);
                            sign_idx += 1;
                        }
                        wlv.draw_lnum_col(wp);
                    }
                }
                win_col_offset = wlv.off;
                if col_rows > 0 as ::core::ffi::c_int {
                    wlv_put_linebuf(
                        wp,
                        &raw mut wlv,
                        if wlv.off < view_width {
                            wlv.off
                        } else {
                            view_width
                        },
                        false_0 != 0,
                        bg_attr,
                        0 as ::core::ffi::c_int,
                    );
                    if !(wlv.row + 1 as ::core::ffi::c_int - wlv.startrow < col_rows
                        && (statuscol.draw as ::core::ffi::c_int != 0
                            || win_hl_attr(wp, HLF_LNA) != win_hl_attr(wp, HLF_N)
                            || win_hl_attr(wp, HLF_LNB) != win_hl_attr(wp, HLF_N))
                        || wlv.filler_todo > 0 as ::core::ffi::c_int)
                    {
                        break 's_5143;
                    }
                    wlv.row += 1;
                    if wlv.row == endrow {
                        break 's_5143;
                    }
                    wlv.filler_todo -= 1;
                    virt_line_index = -1 as ::core::ffi::c_int;
                    if wlv.filler_todo == 0 as ::core::ffi::c_int
                        && ((*wp).w_botfill as ::core::ffi::c_int != 0 || !draw_text)
                    {
                        break 's_5143;
                    }
                    wlv.col = 0 as ::core::ffi::c_int;
                    wlv.off = 0 as ::core::ffi::c_int;
                    continue 's_5143;
                } else {
                    if !(*wp).w_briopt_sbr {
                        wlv.handle_breakindent(wp);
                    }
                    wlv.handle_showbreak_and_filler(wp);
                    if (*wp).w_briopt_sbr {
                        wlv.handle_breakindent(wp);
                    }
                    wlv.col = wlv.off;
                    draw_cols = false_0 != 0;
                    if wlv.filler_todo <= 0 as ::core::ffi::c_int {
                        leftcols_width = wlv.off;
                    }
                    if has_decor as ::core::ffi::c_int != 0
                        && wlv.row == startrow + wlv.filler_lines
                    {
                        decor_redraw_col(
                            wp,
                            ptr.offset_from(line) as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                            wlv.off,
                            true_0 != 0,
                            decor_state.ptr(),
                            decor_provider_end_col - 1 as ::core::ffi::c_int,
                        );
                    }
                    if wlv.col >= view_width {
                        wlv.off = view_width;
                        wlv.col = wlv.off;
                        break '_end_check;
                    }
                }
            }
            if cul_screenline as ::core::ffi::c_int != 0
                && wlv.filler_todo <= 0 as ::core::ffi::c_int
                && wlv.vcol >= left_curline_col
                && wlv.vcol < right_curline_col
            {
                wlv.apply_cursorline_highlight(wp);
            }
            if dollar_vcol.get() >= 0 as ::core::ffi::c_int
                && in_curline as ::core::ffi::c_int != 0
                && wlv.vcol >= (*wp).w_virtcol
            {
                draw_virt_text(wp, buf, win_col_offset, &mut wlv.col, wlv.row);
                wlv_put_linebuf(
                    wp,
                    &raw mut wlv,
                    wlv.col,
                    false_0 != 0,
                    bg_attr,
                    0 as ::core::ffi::c_int,
                );
                if (*wp).w_onebuf_opt.wo_cuc != 0 {
                    wlv.row = (*wp).w_cline_row + (*wp).w_cline_height;
                } else {
                    wlv.row = view_height;
                }
                break 's_5143;
            } else {
                draw_folded =
                    has_fold as ::core::ffi::c_int != 0 && wlv.row == startrow + wlv.filler_lines;
                if draw_folded as ::core::ffi::c_int != 0 && wlv.n_extra == 0 as ::core::ffi::c_int
                {
                    folded_attr = win_hl_attr(wp, HLF_FL);
                    wlv.char_attr = folded_attr;
                    decor_attr = 0 as ::core::ffi::c_int;
                }
                extmark_attr = 0 as ::core::ffi::c_int;
                if wlv.filler_todo <= 0 as ::core::ffi::c_int
                    && (area_highlighting as ::core::ffi::c_int != 0
                        || (*spv).spv_has_spell as ::core::ffi::c_int != 0
                        || extra_check as ::core::ffi::c_int != 0)
                {
                    if wlv.n_extra == 0 as ::core::ffi::c_int || !wlv.extra_for_extmark {
                        wlv.reset_extra_attr = false_0 != 0;
                    }
                    if has_decor as ::core::ffi::c_int != 0
                        && wlv.n_extra == 0 as ::core::ffi::c_int
                    {
                        if wlv.vcol == wlv.fromcol
                            || wlv.vcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                                == wlv.fromcol
                                && (wlv.n_extra == 0 as ::core::ffi::c_int
                                    && utf_ptr2cells(ptr) > 1 as ::core::ffi::c_int)
                            || vcol_prev == fromcol_prev
                                && vcol_prev < wlv.vcol
                                && wlv.vcol < wlv.tocol
                        {
                            area_active = true_0 != 0;
                        } else if area_active as ::core::ffi::c_int != 0
                            && (wlv.vcol == wlv.tocol
                                || noinvcur as ::core::ffi::c_int != 0
                                    && wlv.vcol == (*wp).w_virtcol)
                        {
                            area_active = false_0 != 0;
                        }
                        let mut selected: bool = area_active as ::core::ffi::c_int != 0
                            || area_highlighting as ::core::ffi::c_int != 0
                                && noinvcur as ::core::ffi::c_int != 0
                                && wlv.vcol == (*wp).w_virtcol;
                        if decor_need_recheck {
                            if !may_have_inline_virt {
                                decor_recheck_draw_col(wlv.off, selected, decor_state.ptr());
                            }
                            decor_need_recheck = false_0 != 0;
                        }
                        extmark_attr = decor_redraw_col(
                            wp,
                            ptr.offset_from(line) as ::core::ffi::c_int,
                            if may_have_inline_virt as ::core::ffi::c_int != 0 {
                                -3 as ::core::ffi::c_int
                            } else {
                                wlv.off
                            },
                            selected,
                            decor_state.ptr(),
                            decor_provider_end_col - 1 as ::core::ffi::c_int,
                        );
                        if may_have_inline_virt {
                            wlv.handle_inline_virtual_text(ptr.offset_from(line), selected);
                            if wlv.n_extra > 0 as ::core::ffi::c_int
                                && wlv.virt_inline_hl_mode as ::core::ffi::c_uint
                                    <= kHlModeReplace as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                saved_search_attr = search_attr;
                                saved_area_attr = area_attr;
                                saved_decor_attr = decor_attr;
                                saved_search_attr_from_match = search_attr_from_match;
                                search_attr = 0 as ::core::ffi::c_int;
                                area_attr = 0 as ::core::ffi::c_int;
                                decor_attr = 0 as ::core::ffi::c_int;
                                search_attr_from_match = false_0 != 0;
                            }
                        }
                    }
                    let mut area_attr_p: *mut ::core::ffi::c_int =
                        if wlv.extra_for_extmark as ::core::ffi::c_int != 0
                            && wlv.virt_inline_hl_mode as ::core::ffi::c_uint
                                <= kHlModeReplace as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            &raw mut saved_area_attr
                        } else {
                            &raw mut area_attr
                        };
                    if wlv.vcol == wlv.fromcol
                        || wlv.vcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int == wlv.fromcol
                            && (wlv.n_extra == 0 as ::core::ffi::c_int
                                && utf_ptr2cells(ptr) > 1 as ::core::ffi::c_int
                                || wlv.n_extra > 0 as ::core::ffi::c_int
                                    && !wlv.p_extra.is_null()
                                    && utf_ptr2cells(wlv.p_extra) > 1 as ::core::ffi::c_int)
                        || vcol_prev == fromcol_prev && vcol_prev < wlv.vcol && wlv.vcol < wlv.tocol
                    {
                        *area_attr_p = vi_attr;
                        area_active = true_0 != 0;
                    } else if *area_attr_p != 0 as ::core::ffi::c_int
                        && (wlv.vcol == wlv.tocol
                            || noinvcur as ::core::ffi::c_int != 0 && wlv.vcol == (*wp).w_virtcol)
                    {
                        *area_attr_p = 0 as ::core::ffi::c_int;
                        area_active = false_0 != 0;
                    }
                    if !has_foldtext && wlv.n_extra == 0 as ::core::ffi::c_int {
                        let v_1: ::core::ffi::c_int = ptr.offset_from(line) as ::core::ffi::c_int;
                        search_attr = update_search_hl(
                            wp,
                            lnum,
                            v_1 as colnr_T,
                            &raw mut line,
                            screen_search_hl.ptr(),
                            &raw mut has_match_conc,
                            &raw mut match_conc,
                            lcs_eol_todo,
                            &raw mut on_last_col,
                            &raw mut search_attr_from_match,
                        );
                        ptr = line.offset(v_1 as isize);
                        if *ptr as ::core::ffi::c_int == NUL {
                            has_match_conc = 0 as ::core::ffi::c_int;
                        }
                        if State.get() & MODE_INSERT != 0
                            && ins_compl_win_active(wp) as ::core::ffi::c_int != 0
                            && (in_curline as ::core::ffi::c_int != 0
                                || ins_compl_lnum_in_range(lnum) as ::core::ffi::c_int != 0)
                        {
                            let mut ins_match_attr: ::core::ffi::c_int = ins_compl_col_range_attr(
                                lnum,
                                ptr.offset_from(line) as ::core::ffi::c_int,
                            );
                            if ins_match_attr > 0 as ::core::ffi::c_int {
                                search_attr = hl_combine_attr(search_attr, ins_match_attr);
                            }
                        }
                    }
                    if wlv.diff_hlf as ::core::ffi::c_uint != HLF_NONE as ::core::ffi::c_uint {
                        if line_changes.num_changes > 0 as ::core::ffi::c_int
                            && change_index >= 0 as ::core::ffi::c_int
                            && change_index < line_changes.num_changes - 1 as ::core::ffi::c_int
                        {
                            if ptr.offset_from(line)
                                >= (*line_changes
                                    .changes
                                    .offset((change_index + 1 as ::core::ffi::c_int) as isize))
                                .dc_start[line_changes.bufidx as usize]
                                    as isize
                            {
                                change_index += 1 as ::core::ffi::c_int;
                            }
                        }
                        let mut added_0: bool = false_0 != 0;
                        if line_changes.num_changes > 0 as ::core::ffi::c_int
                            && change_index >= 0 as ::core::ffi::c_int
                            && change_index < line_changes.num_changes
                        {
                            added_0 = diff_change_parse(
                                &raw mut line_changes,
                                line_changes.changes.offset(change_index as isize),
                                &raw mut change_start,
                                &raw mut change_end,
                            );
                        }
                        if wlv.diff_hlf as ::core::ffi::c_uint == HLF_CHD as ::core::ffi::c_uint
                            && ptr.offset_from(line) >= change_start as isize
                            && wlv.n_extra == 0 as ::core::ffi::c_int
                        {
                            wlv.diff_hlf = (if added_0 as ::core::ffi::c_int != 0 {
                                HLF_TXA
                            } else {
                                HLF_TXD
                            }) as hlf_T;
                        }
                        if (wlv.diff_hlf as ::core::ffi::c_uint == HLF_TXD as ::core::ffi::c_uint
                            || wlv.diff_hlf as ::core::ffi::c_uint
                                == HLF_TXA as ::core::ffi::c_uint)
                            && (ptr.offset_from(line) >= change_end as isize
                                && wlv.n_extra == 0 as ::core::ffi::c_int
                                || wlv.n_extra > 0 as ::core::ffi::c_int
                                    && wlv.extra_for_extmark as ::core::ffi::c_int != 0)
                        {
                            wlv.diff_hlf = HLF_CHD;
                        }
                        wlv.set_line_attr_for_diff(wp);
                    }
                    if area_attr != 0 as ::core::ffi::c_int {
                        char_attr_pri = hl_combine_attr(wlv.line_attr, area_attr);
                        if !highlight_match.get() {
                            char_attr_pri = hl_combine_attr(search_attr, char_attr_pri);
                        }
                    } else if search_attr != 0 as ::core::ffi::c_int {
                        char_attr_pri = hl_combine_attr(wlv.line_attr, search_attr);
                    } else if wlv.line_attr != 0 as ::core::ffi::c_int
                        && (wlv.fromcol == -10 as ::core::ffi::c_int
                            && wlv.tocol == MAXCOL as ::core::ffi::c_int
                            || wlv.vcol < wlv.fromcol
                            || vcol_prev < fromcol_prev
                            || wlv.vcol >= wlv.tocol)
                    {
                        char_attr_pri = wlv.line_attr;
                    } else {
                        char_attr_pri = 0 as ::core::ffi::c_int;
                    }
                    char_attr_base = hl_combine_attr(folded_attr, decor_attr);
                    wlv.char_attr = hl_combine_attr(char_attr_base, char_attr_pri);
                }
                if draw_folded as ::core::ffi::c_int != 0
                    && has_foldtext as ::core::ffi::c_int != 0
                    && wlv.n_extra == 0 as ::core::ffi::c_int
                    && wlv.col == win_col_offset
                {
                    let v_2: ::core::ffi::c_int = ptr.offset_from(line) as ::core::ffi::c_int;
                    let mut lnume: linenr_T = lnum + foldinfo.fi_lines - 1 as linenr_T;
                    memset(
                        &raw mut buf_fold as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                        ' ' as ::core::ffi::c_int,
                        FOLD_TEXT_LEN as ::core::ffi::c_int as size_t,
                    );
                    wlv.p_extra = get_foldtext(
                        wp,
                        lnum,
                        lnume,
                        foldinfo,
                        &raw mut buf_fold as *mut ::core::ffi::c_char,
                        &raw mut fold_vt,
                    );
                    wlv.n_extra = strlen(wlv.p_extra) as ::core::ffi::c_int;
                    if wlv.p_extra != &raw mut buf_fold as *mut ::core::ffi::c_char {
                        '_c2rust_label_3: {
                            if foldtext_free.is_null() {
                            } else {
                                __assert_fail(
                                    b"foldtext_free == NULL\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/drawline.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2012 as ::core::ffi::c_uint,
                                    b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        foldtext_free = wlv.p_extra;
                    }
                    wlv.sc_extra = NUL as schar_T;
                    wlv.sc_final = NUL as schar_T;
                    *wlv.p_extra.offset(wlv.n_extra as isize) = NUL as ::core::ffi::c_char;
                    line = ml_get_buf((*wp).w_buffer, lnum);
                    ptr = line.offset(v_2 as isize);
                }
                if draw_folded as ::core::ffi::c_int != 0
                    && wlv.n_extra == 0 as ::core::ffi::c_int
                    && wlv.col < view_width
                    && (has_foldtext as ::core::ffi::c_int != 0
                        || *ptr as ::core::ffi::c_int == NUL
                            && ((*wp).w_onebuf_opt.wo_list == 0
                                || !lcs_eol_todo
                                || lcs_eol == NUL as schar_T))
                {
                    wlv.sc_extra = (*wp).w_p_fcs_chars.fold;
                    wlv.sc_final = NUL as schar_T;
                    wlv.n_extra = view_width - wlv.col;
                    search_attr = 0 as ::core::ffi::c_int;
                }
                if draw_folded as ::core::ffi::c_int != 0
                    && wlv.n_extra != 0 as ::core::ffi::c_int
                    && wlv.col >= view_width
                {
                    wlv.n_extra = 0 as ::core::ffi::c_int;
                }
                if wlv.n_extra > 0 as ::core::ffi::c_int {
                    if wlv.sc_extra != NUL as schar_T
                        || wlv.n_extra == 1 as ::core::ffi::c_int && wlv.sc_final != NUL as schar_T
                    {
                        mb_schar = if wlv.n_extra == 1 as ::core::ffi::c_int
                            && wlv.sc_final != NUL as schar_T
                        {
                            wlv.sc_final
                        } else {
                            wlv.sc_extra
                        };
                        mb_c = schar_get_first_codepoint(mb_schar);
                        wlv.n_extra -= 1;
                    } else {
                        '_c2rust_label_4: {
                            if !wlv.p_extra.is_null() {
                            } else {
                                __assert_fail(
                                    b"wlv.p_extra != NULL\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/drawline.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2055 as ::core::ffi::c_uint,
                                    b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        mb_l = utfc_ptr2len(wlv.p_extra);
                        mb_schar = utfc_ptr2schar(wlv.p_extra, &raw mut mb_c);
                        if mb_l > wlv.n_extra || mb_l == 0 as ::core::ffi::c_int {
                            mb_l = 1 as ::core::ffi::c_int;
                        }
                        if wlv.col >= view_width - 1 as ::core::ffi::c_int
                            && schar_cells(mb_schar) == 2 as ::core::ffi::c_int
                        {
                            mb_c = '>' as ::core::ffi::c_int;
                            mb_l = 1 as ::core::ffi::c_int;
                            mb_schar = mb_c as schar_T;
                            multi_attr = win_hl_attr(wp, HLF_AT);
                            if wlv.cursorline_attr != 0 {
                                multi_attr = if 0 as ::core::ffi::c_int != wlv.line_attr_lowprio {
                                    hl_combine_attr(wlv.cursorline_attr, multi_attr)
                                } else {
                                    hl_combine_attr(multi_attr, wlv.cursorline_attr)
                                };
                            }
                        } else {
                            wlv.n_extra -= mb_l;
                            wlv.p_extra = wlv.p_extra.offset(mb_l as isize);
                        }
                        if wlv.filler_todo <= 0 as ::core::ffi::c_int
                            && wlv.skip_cells > 0 as ::core::ffi::c_int
                            && mb_l > 1 as ::core::ffi::c_int
                        {
                            if wlv.n_extra > 0 as ::core::ffi::c_int {
                                n_extra_next = wlv.n_extra;
                                extra_attr_next = wlv.extra_attr;
                            }
                            wlv.n_extra = 1 as ::core::ffi::c_int;
                            wlv.sc_extra = '<' as ::core::ffi::c_int as schar_T;
                            wlv.sc_final = NUL as schar_T;
                            mb_schar = ' ' as ::core::ffi::c_int as schar_T;
                            mb_c = ' ' as ::core::ffi::c_int;
                            mb_l = 1 as ::core::ffi::c_int;
                            wlv.n_attr += 1;
                            wlv.extra_attr = win_hl_attr(wp, HLF_AT);
                        }
                    }
                    if wlv.n_extra <= 0 as ::core::ffi::c_int {
                        if n_extra_next <= 0 as ::core::ffi::c_int {
                            if search_attr == 0 as ::core::ffi::c_int {
                                search_attr = saved_search_attr;
                                saved_search_attr = 0 as ::core::ffi::c_int;
                            }
                            if area_attr == 0 as ::core::ffi::c_int
                                && *ptr as ::core::ffi::c_int != NUL
                            {
                                area_attr = saved_area_attr;
                                saved_area_attr = 0 as ::core::ffi::c_int;
                            }
                            if decor_attr == 0 as ::core::ffi::c_int {
                                decor_attr = saved_decor_attr;
                                saved_decor_attr = 0 as ::core::ffi::c_int;
                            }
                            if wlv.extra_for_extmark {
                                wlv.reset_extra_attr = true_0 != 0;
                                extra_attr_next = -1 as ::core::ffi::c_int;
                            }
                            wlv.extra_for_extmark = false_0 != 0;
                        } else {
                            '_c2rust_label_5: {
                                if wlv.sc_extra != '\0' as schar_T
                                    || wlv.sc_final != '\0' as schar_T
                                {
                                } else {
                                    __assert_fail(
                                        b"wlv.sc_extra != NUL || wlv.sc_final != NUL\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/drawline.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2121 as ::core::ffi::c_uint,
                                        b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            '_c2rust_label_6: {
                                if !wlv.p_extra.is_null() {
                                } else {
                                    __assert_fail(
                                        b"wlv.p_extra != NULL\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/drawline.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2122 as ::core::ffi::c_uint,
                                        b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            wlv.sc_extra = NUL as schar_T;
                            wlv.sc_final = NUL as schar_T;
                            wlv.n_extra = n_extra_next;
                            n_extra_next = 0 as ::core::ffi::c_int;
                            wlv.reset_extra_attr = true_0 != 0;
                            '_c2rust_label_7: {
                                if extra_attr_next >= 0 as ::core::ffi::c_int {
                                } else {
                                    __assert_fail(
                                        b"extra_attr_next >= 0\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/drawline.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2130 as ::core::ffi::c_uint,
                                        b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                        }
                    }
                } else if wlv.filler_todo > 0 as ::core::ffi::c_int {
                    mb_c = ' ' as ::core::ffi::c_int;
                    mb_schar = ' ' as ::core::ffi::c_int as schar_T;
                } else if has_foldtext as ::core::ffi::c_int != 0
                    || has_fold as ::core::ffi::c_int != 0 && wlv.col >= view_width
                {
                    mb_schar = NUL as schar_T;
                } else {
                    let mut prev_ptr_0: *const ::core::ffi::c_char = ptr;
                    let mut c0: ::core::ffi::c_int = *ptr as uint8_t as ::core::ffi::c_int;
                    if c0 == NUL {
                        wlv.skip_cells = 0 as ::core::ffi::c_int;
                    }
                    mb_l = utfc_ptr2len(ptr);
                    mb_schar = utfc_ptr2schar(ptr, &raw mut mb_c);
                    if mb_l > 1 as ::core::ffi::c_int && mb_c < 0x80 as ::core::ffi::c_int {
                        c0 = mb_c;
                    }
                    if mb_l == 1 as ::core::ffi::c_int && c0 >= 0x80 as ::core::ffi::c_int
                        || mb_l >= 1 as ::core::ffi::c_int && mb_c == 0 as ::core::ffi::c_int
                        || mb_l > 1 as ::core::ffi::c_int && !vim_isprintc(mb_c)
                    {
                        transchar_hex(&raw mut wlv.extra as *mut ::core::ffi::c_char, mb_c);
                        if (*wp).w_onebuf_opt.wo_rl != 0 {
                            rl_mirror_ascii(
                                &raw mut wlv.extra as *mut ::core::ffi::c_char,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            );
                        }
                        wlv.p_extra = &raw mut wlv.extra as *mut ::core::ffi::c_char;
                        mb_c = mb_ptr2char_adv(
                            &raw mut wlv.p_extra as *mut *const ::core::ffi::c_char,
                        );
                        mb_schar = schar_from_char(mb_c);
                        wlv.n_extra = strlen(wlv.p_extra) as ::core::ffi::c_int;
                        wlv.sc_extra = NUL as schar_T;
                        wlv.sc_final = NUL as schar_T;
                        if area_attr == 0 as ::core::ffi::c_int
                            && search_attr == 0 as ::core::ffi::c_int
                        {
                            wlv.n_attr = wlv.n_extra + 1 as ::core::ffi::c_int;
                            wlv.extra_attr = win_hl_attr(wp, HLF_8);
                            saved_attr2 = wlv.char_attr;
                        }
                    } else if mb_l == 0 as ::core::ffi::c_int {
                        mb_l = 1 as ::core::ffi::c_int;
                    }
                    if wlv.col >= view_width - 1 as ::core::ffi::c_int
                        && schar_cells(mb_schar) == 2 as ::core::ffi::c_int
                    {
                        mb_schar = '>' as ::core::ffi::c_int as schar_T;
                        mb_c = '>' as ::core::ffi::c_int;
                        mb_l = 1 as ::core::ffi::c_int;
                        multi_attr = win_hl_attr(wp, HLF_AT);
                        ptr = ptr.offset(-1);
                        did_decrement_ptr = true_0 != 0;
                    } else if *ptr as ::core::ffi::c_int != NUL {
                        ptr = ptr.offset((mb_l - 1 as ::core::ffi::c_int) as isize);
                    }
                    if wlv.skip_cells > 0 as ::core::ffi::c_int
                        && mb_l > 1 as ::core::ffi::c_int
                        && wlv.n_extra == 0 as ::core::ffi::c_int
                    {
                        wlv.n_extra = 1 as ::core::ffi::c_int;
                        wlv.sc_extra = '<' as ::core::ffi::c_int as schar_T;
                        wlv.sc_final = NUL as schar_T;
                        mb_schar = ' ' as ::core::ffi::c_int as schar_T;
                        mb_c = ' ' as ::core::ffi::c_int;
                        mb_l = 1 as ::core::ffi::c_int;
                        if area_attr == 0 as ::core::ffi::c_int
                            && search_attr == 0 as ::core::ffi::c_int
                        {
                            wlv.n_attr = wlv.n_extra + 1 as ::core::ffi::c_int;
                            wlv.extra_attr = win_hl_attr(wp, HLF_AT);
                            saved_attr2 = wlv.char_attr;
                        }
                    }
                    ptr = ptr.offset(1);
                    decor_attr = 0 as ::core::ffi::c_int;
                    if extra_check {
                        let no_plain_buffer: bool = (*(*wp).w_s).b_p_spo_flags
                            & kOptSpoFlagNoplainbuffer as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0 as ::core::ffi::c_uint;
                        let mut can_spell: bool = !no_plain_buffer;
                        let v_3: ::core::ffi::c_int = ptr.offset_from(line) as ::core::ffi::c_int;
                        let prev_v: ptrdiff_t = prev_ptr_0.offset_from(line);
                        if has_syntax as ::core::ffi::c_int != 0 && v_3 > 0 as ::core::ffi::c_int {
                            let mut save_did_emsg_0: ::core::ffi::c_int = did_emsg.get();
                            did_emsg.set(false_0);
                            decor_attr = get_syntax_attr(
                                v_3 as colnr_T - 1 as colnr_T,
                                if (*spv).spv_has_spell as ::core::ffi::c_int != 0 {
                                    &raw mut can_spell
                                } else {
                                    ::core::ptr::null_mut::<bool>()
                                },
                                false_0 != 0,
                            );
                            if did_emsg.get() != 0 {
                                (*(*wp).w_s).b_syn_error = true_0 != 0;
                                has_syntax = false_0 != 0;
                            } else {
                                did_emsg.set(save_did_emsg_0);
                            }
                            if (*(*wp).w_s).b_syn_slow {
                                has_syntax = false_0 != 0;
                            }
                            line = ml_get_buf((*wp).w_buffer, lnum);
                            ptr = line.offset(v_3 as isize);
                            prev_ptr_0 = line.offset(prev_v as isize);
                            syntax_flags = if mb_schar == 0 as schar_T {
                                0 as ::core::ffi::c_int
                            } else {
                                get_syntax_info(&raw mut syntax_seqnr)
                            };
                        }
                        if has_decor as ::core::ffi::c_int != 0 && v_3 > 0 as ::core::ffi::c_int {
                            decor_attr = hl_combine_attr(decor_attr, extmark_attr);
                            decor_conceal = (*decor_state.ptr()).conceal;
                            can_spell = if (*decor_state.ptr()).spell as ::core::ffi::c_int
                                == kTrue as ::core::ffi::c_int
                            {
                                true_0
                            } else if (*decor_state.ptr()).spell as ::core::ffi::c_int
                                == kFalse as ::core::ffi::c_int
                            {
                                false_0
                            } else {
                                can_spell as ::core::ffi::c_int
                            } != 0;
                        }
                        char_attr_base = hl_combine_attr(folded_attr, decor_attr);
                        wlv.char_attr = hl_combine_attr(char_attr_base, char_attr_pri);
                        let mut v1: ::core::ffi::c_int =
                            ptr.offset_from(line) as ::core::ffi::c_int;
                        if (*spv).spv_has_spell as ::core::ffi::c_int != 0
                            && v1 >= word_end
                            && v1 > cur_checked_col
                        {
                            spell_attr = 0 as ::core::ffi::c_int;
                            if mb_schar != 0 as schar_T
                                && *skipwhite(prev_ptr_0) as ::core::ffi::c_int != NUL
                                && can_spell as ::core::ffi::c_int != 0
                            {
                                let mut p: *mut ::core::ffi::c_char =
                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                let mut spell_hlf_0: hlf_T = HLF_COUNT;
                                v1 -= mb_l - 1 as ::core::ffi::c_int;
                                if prev_ptr_0.offset_from(line) - nextlinecol as isize >= 0 as isize
                                {
                                    p = (nextline.as_mut_ptr()).offset(
                                        (prev_ptr_0.offset_from(line) - nextlinecol as isize)
                                            as isize,
                                    );
                                } else {
                                    p = prev_ptr_0 as *mut ::core::ffi::c_char;
                                }
                                (*spv).spv_cap_col -=
                                    prev_ptr_0.offset_from(line) as ::core::ffi::c_int;
                                let mut tmplen: size_t = spell_check(
                                    wp,
                                    p,
                                    &raw mut spell_hlf_0,
                                    &raw mut (*spv).spv_cap_col,
                                    (*spv).spv_unchanged,
                                );
                                '_c2rust_label_8: {
                                    if tmplen <= 2147483647 as ::core::ffi::c_int as size_t {
                                    } else {
                                        __assert_fail(
                                            b"tmplen <= INT_MAX\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                            b"src/nvim/drawline.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            2290 as ::core::ffi::c_uint,
                                            b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                                let mut len_0: ::core::ffi::c_int = tmplen as ::core::ffi::c_int;
                                word_end = v1 + len_0;
                                if spell_hlf_0 as ::core::ffi::c_uint
                                    != HLF_COUNT as ::core::ffi::c_uint
                                    && State.get() & MODE_INSERT != 0
                                    && (*wp).w_cursor.lnum == lnum
                                    && (*wp).w_cursor.col >= prev_ptr_0.offset_from(line) as colnr_T
                                    && (*wp).w_cursor.col < word_end
                                {
                                    spell_hlf_0 = HLF_COUNT;
                                    spell_redraw_lnum.set(lnum);
                                }
                                if spell_hlf_0 as ::core::ffi::c_uint
                                    == HLF_COUNT as ::core::ffi::c_uint
                                    && p != prev_ptr_0 as *mut ::core::ffi::c_char
                                    && p.offset_from(nextline.as_mut_ptr()) + len_0 as isize
                                        > nextline_idx as isize
                                {
                                    (*spv).spv_checked_lnum = lnum + 1 as linenr_T;
                                    (*spv).spv_checked_col = (p.offset_from(nextline.as_mut_ptr())
                                        + len_0 as isize
                                        - nextline_idx as isize)
                                        as ::core::ffi::c_int;
                                }
                                if spell_hlf_0 as ::core::ffi::c_uint
                                    != HLF_COUNT as ::core::ffi::c_uint
                                {
                                    spell_attr = (*highlight_attr.ptr())[spell_hlf_0 as usize];
                                }
                                if (*spv).spv_cap_col > 0 as ::core::ffi::c_int {
                                    if p != prev_ptr_0 as *mut ::core::ffi::c_char
                                        && p.offset_from(nextline.as_mut_ptr())
                                            + (*spv).spv_cap_col as isize
                                            >= nextline_idx as isize
                                    {
                                        (*spv).spv_capcol_lnum = lnum + 1 as linenr_T;
                                        (*spv).spv_cap_col = (p.offset_from(nextline.as_mut_ptr())
                                            + (*spv).spv_cap_col as isize
                                            - nextline_idx as isize)
                                            as ::core::ffi::c_int;
                                    } else {
                                        (*spv).spv_cap_col +=
                                            prev_ptr_0.offset_from(line) as ::core::ffi::c_int;
                                    }
                                }
                            }
                        }
                        if spell_attr != 0 as ::core::ffi::c_int {
                            char_attr_base = hl_combine_attr(char_attr_base, spell_attr);
                            wlv.char_attr = hl_combine_attr(char_attr_base, char_attr_pri);
                        }
                        if !(*(*wp).w_buffer).terminal.is_null() {
                            wlv.char_attr = hl_combine_attr(
                                if wlv.vcol < TERM_ATTRS_MAX as ::core::ffi::c_int {
                                    term_attrs[wlv.vcol as usize]
                                } else {
                                    0 as ::core::ffi::c_int
                                },
                                wlv.char_attr,
                            );
                        }
                        if (*wp).w_onebuf_opt.wo_lbr != 0
                            && !wlv.need_lbr
                            && mb_schar != NUL as schar_T
                            && !vim_isbreak(*ptr as uint8_t as ::core::ffi::c_int)
                        {
                            wlv.need_lbr = true_0 != 0;
                        }
                        if (*wp).w_onebuf_opt.wo_lbr != 0
                            && c0 == mb_c
                            && mb_c < 128 as ::core::ffi::c_int
                            && wlv.need_lbr as ::core::ffi::c_int != 0
                            && vim_isbreak(mb_c) as ::core::ffi::c_int != 0
                            && !vim_isbreak(*ptr as uint8_t as ::core::ffi::c_int)
                        {
                            let mut mb_off: ::core::ffi::c_int =
                                utf_head_off(line, ptr.offset(-(1 as ::core::ffi::c_int as isize)));
                            let mut p_0: *mut ::core::ffi::c_char =
                                ptr.offset(-((mb_off + 1 as ::core::ffi::c_int) as isize));
                            let mut csarg_0: CharsizeArg = CharsizeArg::default();
                            let mut cstype_0: CharsizeKind =
                                init_charsize_arg(&mut csarg_0, wp, 0 as linenr_T, line);
                            wlv.n_extra = win_charsize(
                                cstype_0,
                                wlv.vcol as ::core::ffi::c_int,
                                p_0,
                                utf_ptr2CharInfo(p_0).value,
                                &mut csarg_0,
                            )
                            .width
                                - 1 as ::core::ffi::c_int;
                            if on_last_col as ::core::ffi::c_int != 0 && mb_c != TAB {
                                search_attr = 0 as ::core::ffi::c_int;
                            }
                            if mb_c == TAB && wlv.n_extra + wlv.col > view_width {
                                wlv.n_extra = tabstop_padding(
                                    wlv.vcol,
                                    (*(*wp).w_buffer).b_p_ts,
                                    (*(*wp).w_buffer).b_p_vts_array,
                                ) - 1 as ::core::ffi::c_int;
                            }
                            wlv.sc_extra = (if mb_off > 0 as ::core::ffi::c_int {
                                '<' as ::core::ffi::c_int
                            } else {
                                ' ' as ::core::ffi::c_int
                            }) as schar_T;
                            wlv.sc_final = NUL as schar_T;
                            if mb_c < 128 as ::core::ffi::c_int
                                && ascii_iswhite(mb_c) as ::core::ffi::c_int != 0
                            {
                                if mb_c == TAB {
                                    wlv.fix_for_boguscols();
                                }
                                if (*wp).w_onebuf_opt.wo_list == 0 {
                                    mb_c = ' ' as ::core::ffi::c_int;
                                    mb_schar = mb_c as schar_T;
                                }
                            }
                        }
                        if (*wp).w_onebuf_opt.wo_list != 0 {
                            in_multispace = mb_c == ' ' as ::core::ffi::c_int
                                && (*ptr as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                                    || prev_ptr_0 > line as *const ::core::ffi::c_char
                                        && *prev_ptr_0.offset(-1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == ' ' as ::core::ffi::c_int);
                            if !in_multispace {
                                multispace_pos = 0 as ::core::ffi::c_int;
                            }
                        }
                        if (*wp).w_onebuf_opt.wo_list != 0
                            && ((mb_c == 160 as ::core::ffi::c_int
                                && mb_l == 2 as ::core::ffi::c_int
                                || mb_c == 0x202f as ::core::ffi::c_int
                                    && mb_l == 3 as ::core::ffi::c_int)
                                && (*wp).w_p_lcs_chars.nbsp != 0
                                || mb_c == ' ' as ::core::ffi::c_int
                                    && mb_l == 1 as ::core::ffi::c_int
                                    && ((*wp).w_p_lcs_chars.space != 0
                                        || in_multispace as ::core::ffi::c_int != 0
                                            && !(*wp).w_p_lcs_chars.multispace.is_null())
                                    && ptr.offset_from(line) >= leadcol as isize
                                    && ptr.offset_from(line) <= trailcol as isize)
                        {
                            if in_multispace as ::core::ffi::c_int != 0
                                && !(*wp).w_p_lcs_chars.multispace.is_null()
                            {
                                let c2rust_fresh1 = multispace_pos;
                                multispace_pos = multispace_pos + 1;
                                mb_schar = *(*wp)
                                    .w_p_lcs_chars
                                    .multispace
                                    .offset(c2rust_fresh1 as isize);
                                if *(*wp)
                                    .w_p_lcs_chars
                                    .multispace
                                    .offset(multispace_pos as isize)
                                    == NUL as schar_T
                                {
                                    multispace_pos = 0 as ::core::ffi::c_int;
                                }
                            } else {
                                mb_schar = if mb_c == ' ' as ::core::ffi::c_int {
                                    (*wp).w_p_lcs_chars.space
                                } else {
                                    (*wp).w_p_lcs_chars.nbsp
                                };
                            }
                            wlv.n_attr = 1 as ::core::ffi::c_int;
                            wlv.extra_attr = win_hl_attr(wp, HLF_0);
                            saved_attr2 = wlv.char_attr;
                            mb_c = schar_get_first_codepoint(mb_schar);
                        }
                        if mb_c == ' ' as ::core::ffi::c_int
                            && mb_l == 1 as ::core::ffi::c_int
                            && (trailcol != MAXCOL as ::core::ffi::c_int
                                && ptr > line.offset(trailcol as isize)
                                || leadcol != 0 as ::core::ffi::c_int
                                    && ptr < line.offset(leadcol as isize))
                        {
                            if leadcol != 0 as ::core::ffi::c_int
                                && in_multispace as ::core::ffi::c_int != 0
                                && ptr < line.offset(leadcol as isize)
                                && !(*wp).w_p_lcs_chars.leadmultispace.is_null()
                            {
                                let c2rust_fresh2 = multispace_pos;
                                multispace_pos = multispace_pos + 1;
                                mb_schar = *(*wp)
                                    .w_p_lcs_chars
                                    .leadmultispace
                                    .offset(c2rust_fresh2 as isize);
                                if *(*wp)
                                    .w_p_lcs_chars
                                    .leadmultispace
                                    .offset(multispace_pos as isize)
                                    == NUL as schar_T
                                {
                                    multispace_pos = 0 as ::core::ffi::c_int;
                                }
                            } else if ptr > line.offset(trailcol as isize)
                                && (*wp).w_p_lcs_chars.trail != 0
                            {
                                mb_schar = (*wp).w_p_lcs_chars.trail;
                            } else if ptr < line.offset(leadcol as isize)
                                && (*wp).w_p_lcs_chars.lead != 0
                            {
                                mb_schar = (*wp).w_p_lcs_chars.lead;
                            } else if leadcol != 0 as ::core::ffi::c_int
                                && (*wp).w_p_lcs_chars.space != 0
                            {
                                mb_schar = (*wp).w_p_lcs_chars.space;
                            }
                            wlv.n_attr = 1 as ::core::ffi::c_int;
                            wlv.extra_attr = win_hl_attr(wp, HLF_0);
                            saved_attr2 = wlv.char_attr;
                            mb_c = schar_get_first_codepoint(mb_schar);
                        }
                    }
                    if !vim_isprintc(mb_c) {
                        if mb_c == TAB
                            && ((*wp).w_onebuf_opt.wo_list == 0 || (*wp).w_p_lcs_chars.tab1 != 0)
                        {
                            let mut tab_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            let mut vcol_adjusted: colnr_T = wlv.vcol;
                            let mut lcs_tab1: schar_T = (*wp).w_p_lcs_chars.tab1;
                            let mut lcs_tab2: schar_T = (*wp).w_p_lcs_chars.tab2;
                            let mut lcs_tab3: schar_T = (*wp).w_p_lcs_chars.tab3;
                            if (*wp).w_onebuf_opt.wo_list != 0
                                && (*wp).w_p_lcs_chars.leadtab1 != NUL as schar_T
                                && ptr < line.offset(leadcol as isize)
                            {
                                lcs_tab1 = (*wp).w_p_lcs_chars.leadtab1;
                                lcs_tab2 = (*wp).w_p_lcs_chars.leadtab2;
                                lcs_tab3 = (*wp).w_p_lcs_chars.leadtab3;
                            }
                            let sbr: *mut ::core::ffi::c_char = get_showbreak_value(wp);
                            if *sbr as ::core::ffi::c_int != NUL
                                && wlv.vcol == wlv.showbreak_vcol
                                && (*wp).w_onebuf_opt.wo_wrap != 0
                            {
                                vcol_adjusted =
                                    (wlv.vcol as ::core::ffi::c_int - mb_charlen(sbr)) as colnr_T;
                            }
                            tab_len = tabstop_padding(
                                vcol_adjusted,
                                (*(*wp).w_buffer).b_p_ts,
                                (*(*wp).w_buffer).b_p_vts_array,
                            ) - 1 as ::core::ffi::c_int;
                            if (*wp).w_onebuf_opt.wo_lbr == 0 || (*wp).w_onebuf_opt.wo_list == 0 {
                                wlv.n_extra = tab_len;
                            } else {
                                let mut saved_nextra: ::core::ffi::c_int = wlv.n_extra;
                                if wlv.vcol_off_co > 0 as ::core::ffi::c_int {
                                    tab_len += wlv.vcol_off_co;
                                }
                                if lcs_tab1 != 0
                                    && wlv.old_boguscols > 0 as ::core::ffi::c_int
                                    && wlv.n_extra > tab_len
                                {
                                    tab_len += wlv.n_extra - tab_len;
                                }
                                if tab_len > 0 as ::core::ffi::c_int {
                                    let mut tab2_len: size_t = schar_len(lcs_tab2);
                                    let mut len_1: size_t =
                                        (tab_len as size_t).wrapping_mul(tab2_len);
                                    if lcs_tab3 != 0 {
                                        len_1 = len_1.wrapping_add(
                                            schar_len(lcs_tab3).wrapping_sub(tab2_len),
                                        );
                                    }
                                    if wlv.n_extra > 0 as ::core::ffi::c_int {
                                        len_1 =
                                            len_1.wrapping_add((wlv.n_extra - tab_len) as size_t);
                                    }
                                    mb_schar = lcs_tab1;
                                    mb_c = schar_get_first_codepoint(mb_schar);
                                    let mut p_1: *mut ::core::ffi::c_char =
                                        get_extra_buf(len_1.wrapping_add(1 as size_t));
                                    memset(
                                        p_1 as *mut ::core::ffi::c_void,
                                        ' ' as ::core::ffi::c_int,
                                        len_1,
                                    );
                                    *p_1.offset(len_1 as isize) = NUL as ::core::ffi::c_char;
                                    wlv.p_extra = p_1;
                                    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while i < tab_len {
                                        if *p_1 as ::core::ffi::c_int == NUL {
                                            tab_len = i;
                                            break;
                                        } else {
                                            let mut lcs: schar_T = lcs_tab2;
                                            if lcs_tab3 != 0
                                                && i == tab_len - 1 as ::core::ffi::c_int
                                            {
                                                lcs = lcs_tab3;
                                            }
                                            let mut slen: size_t = schar_get_adv(&raw mut p_1, lcs);
                                            wlv.n_extra += slen as ::core::ffi::c_int
                                                - (if saved_nextra > 0 as ::core::ffi::c_int {
                                                    1 as ::core::ffi::c_int
                                                } else {
                                                    0 as ::core::ffi::c_int
                                                });
                                            i += 1;
                                        }
                                    }
                                    if wlv.vcol_off_co > 0 as ::core::ffi::c_int {
                                        wlv.n_extra -= wlv.vcol_off_co;
                                    }
                                }
                            }
                            let mut vc_saved: ::core::ffi::c_int = wlv.vcol_off_co;
                            wlv.fix_for_boguscols();
                            if wlv.n_extra == tab_len + vc_saved
                                && (*wp).w_onebuf_opt.wo_list != 0
                                && (*wp).w_p_lcs_chars.tab1 != 0
                            {
                                tab_len += vc_saved;
                            }
                            if (*wp).w_onebuf_opt.wo_list != 0 {
                                mb_schar =
                                    if wlv.n_extra == 0 as ::core::ffi::c_int && lcs_tab3 != 0 {
                                        lcs_tab3
                                    } else {
                                        lcs_tab1
                                    };
                                if (*wp).w_onebuf_opt.wo_lbr != 0
                                    && !wlv.p_extra.is_null()
                                    && *wlv.p_extra as ::core::ffi::c_int != NUL
                                {
                                    wlv.sc_extra = NUL as schar_T;
                                } else {
                                    wlv.sc_extra = lcs_tab2;
                                }
                                wlv.sc_final = lcs_tab3;
                                wlv.n_attr = tab_len + 1 as ::core::ffi::c_int;
                                wlv.extra_attr = win_hl_attr(wp, HLF_0);
                                saved_attr2 = wlv.char_attr;
                            } else {
                                wlv.sc_final = NUL as schar_T;
                                wlv.sc_extra = ' ' as ::core::ffi::c_int as schar_T;
                                mb_schar = ' ' as ::core::ffi::c_int as schar_T;
                            }
                            mb_c = schar_get_first_codepoint(mb_schar);
                        } else if mb_schar == NUL as schar_T
                            && ((*wp).w_onebuf_opt.wo_list != 0
                                || (wlv.fromcol >= 0 as ::core::ffi::c_int
                                    || fromcol_prev >= 0 as ::core::ffi::c_int)
                                    && wlv.tocol > wlv.vcol
                                    && VIsual_mode.get() != Ctrl_V
                                    && wlv.col < view_width
                                    && !(noinvcur as ::core::ffi::c_int != 0
                                        && lnum == (*wp).w_cursor.lnum
                                        && wlv.vcol == (*wp).w_virtcol))
                            && lcs_eol_todo as ::core::ffi::c_int != 0
                            && lcs_eol != NUL as schar_T
                        {
                            if wlv.diff_hlf as ::core::ffi::c_uint
                                == HLF_NONE as ::core::ffi::c_uint
                                && wlv.line_attr == 0 as ::core::ffi::c_int
                                && wlv.line_attr_lowprio == 0 as ::core::ffi::c_int
                            {
                                if !(area_highlighting as ::core::ffi::c_int != 0
                                    && virtual_active(wp) as ::core::ffi::c_int != 0
                                    && wlv.tocol != MAXCOL as ::core::ffi::c_int
                                    && wlv.vcol < wlv.tocol)
                                {
                                    wlv.p_extra = b"\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char;
                                }
                                wlv.n_extra = 0 as ::core::ffi::c_int;
                            }
                            if (*wp).w_onebuf_opt.wo_list != 0
                                && (*wp).w_p_lcs_chars.eol > 0 as schar_T
                            {
                                mb_schar = (*wp).w_p_lcs_chars.eol;
                            } else {
                                mb_schar = ' ' as ::core::ffi::c_int as schar_T;
                            }
                            lcs_eol_todo = false_0 != 0;
                            ptr = ptr.offset(-1);
                            wlv.extra_attr = win_hl_attr(wp, HLF_AT);
                            wlv.n_attr = 1 as ::core::ffi::c_int;
                            mb_c = schar_get_first_codepoint(mb_schar);
                        } else if mb_schar != NUL as schar_T {
                            wlv.p_extra = transchar_buf((*wp).w_buffer, mb_c);
                            if wlv.n_extra == 0 as ::core::ffi::c_int {
                                wlv.n_extra = byte2cells(mb_c) - 1 as ::core::ffi::c_int;
                            }
                            if dy_flags.get()
                                & kOptDyFlagUhex as ::core::ffi::c_int as ::core::ffi::c_uint
                                != 0
                                && (*wp).w_onebuf_opt.wo_rl != 0
                            {
                                rl_mirror_ascii(
                                    wlv.p_extra,
                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                );
                            }
                            wlv.sc_extra = NUL as schar_T;
                            wlv.sc_final = NUL as schar_T;
                            if (*wp).w_onebuf_opt.wo_lbr != 0 {
                                mb_c = *wlv.p_extra as uint8_t as ::core::ffi::c_int;
                                let mut p_2: *mut ::core::ffi::c_char = get_extra_buf(
                                    (wlv.n_extra as size_t).wrapping_add(1 as size_t),
                                );
                                memset(
                                    p_2 as *mut ::core::ffi::c_void,
                                    ' ' as ::core::ffi::c_int,
                                    wlv.n_extra as size_t,
                                );
                                memcpy(
                                    p_2 as *mut ::core::ffi::c_void,
                                    wlv.p_extra.offset(1 as ::core::ffi::c_int as isize)
                                        as *const ::core::ffi::c_void,
                                    strlen(wlv.p_extra).wrapping_sub(1 as size_t),
                                );
                                *p_2.offset(wlv.n_extra as isize) = NUL as ::core::ffi::c_char;
                                wlv.p_extra = p_2;
                            } else {
                                wlv.n_extra = byte2cells(mb_c) - 1 as ::core::ffi::c_int;
                                let c2rust_fresh3 = wlv.p_extra;
                                wlv.p_extra = wlv.p_extra.offset(1);
                                mb_c = *c2rust_fresh3 as uint8_t as ::core::ffi::c_int;
                            }
                            wlv.n_attr = wlv.n_extra + 1 as ::core::ffi::c_int;
                            wlv.extra_attr = win_hl_attr(wp, HLF_8);
                            saved_attr2 = wlv.char_attr;
                            mb_schar = mb_c as schar_T;
                        } else if VIsual_active.get() as ::core::ffi::c_int != 0
                            && (VIsual_mode.get() == Ctrl_V
                                || VIsual_mode.get() == 'v' as ::core::ffi::c_int)
                            && virtual_active(wp) as ::core::ffi::c_int != 0
                            && wlv.tocol != MAXCOL as ::core::ffi::c_int
                            && wlv.vcol < wlv.tocol
                            && wlv.col < view_width
                        {
                            mb_c = ' ' as ::core::ffi::c_int;
                            mb_schar = schar_from_char(mb_c);
                            ptr = ptr.offset(-1);
                        }
                    }
                    if (*wp).w_onebuf_opt.wo_cole > 0 as OptInt
                        && (wp != curwin.get()
                            || lnum != (*wp).w_cursor.lnum
                            || conceal_cursor_line(wp) as ::core::ffi::c_int != 0)
                        && (syntax_flags & HL_CONCEAL != 0 as ::core::ffi::c_int
                            || has_match_conc > 0 as ::core::ffi::c_int
                            || decor_conceal > 0 as ::core::ffi::c_int)
                        && !(lnum_in_visual_area as ::core::ffi::c_int != 0
                            && vim_strchr((*wp).w_onebuf_opt.wo_cocu, 'v' as ::core::ffi::c_int)
                                .is_null())
                    {
                        let mut syntax_conceal: bool =
                            syntax_flags & HL_CONCEAL != 0 as ::core::ffi::c_int;
                        wlv.char_attr = conceal_attr;
                        if (prev_syntax_id != syntax_seqnr
                            && syntax_conceal as ::core::ffi::c_int != 0
                            || has_match_conc > 1 as ::core::ffi::c_int
                            || decor_conceal > 1 as ::core::ffi::c_int)
                            && (syntax_conceal as ::core::ffi::c_int != 0
                                && syn_get_sub_char() != NUL
                                || has_match_conc != 0 && match_conc != 0
                                || decor_conceal != 0 && (*decor_state.ptr()).conceal_char != 0
                                || (*wp).w_onebuf_opt.wo_cole == 1 as OptInt)
                            && (*wp).w_onebuf_opt.wo_cole != 3 as OptInt
                        {
                            if schar_cells(mb_schar) > 1 as ::core::ffi::c_int {
                                wlv.n_extra += 1;
                            }
                            if has_match_conc != 0 && match_conc != 0 {
                                mb_schar = schar_from_char(match_conc);
                            } else if decor_conceal != 0 && (*decor_state.ptr()).conceal_char != 0 {
                                mb_schar = (*decor_state.ptr()).conceal_char;
                                if (*decor_state.ptr()).conceal_attr != 0 {
                                    wlv.char_attr = (*decor_state.ptr()).conceal_attr;
                                }
                            } else if syntax_conceal as ::core::ffi::c_int != 0
                                && syn_get_sub_char() != NUL
                            {
                                mb_schar = schar_from_char(syn_get_sub_char());
                            } else if (*wp).w_p_lcs_chars.conceal != NUL as schar_T {
                                mb_schar = (*wp).w_p_lcs_chars.conceal;
                            } else {
                                mb_schar = ' ' as ::core::ffi::c_int as schar_T;
                            }
                            mb_c = schar_get_first_codepoint(mb_schar);
                            prev_syntax_id = syntax_seqnr;
                            if wlv.n_extra > 0 as ::core::ffi::c_int {
                                wlv.vcol_off_co += wlv.n_extra;
                            }
                            wlv.vcol += wlv.n_extra;
                            if is_wrapped as ::core::ffi::c_int != 0
                                && wlv.n_extra > 0 as ::core::ffi::c_int
                            {
                                wlv.boguscols += wlv.n_extra;
                                wlv.col += wlv.n_extra;
                            }
                            wlv.n_extra = 0 as ::core::ffi::c_int;
                            wlv.n_attr = 0 as ::core::ffi::c_int;
                        } else if wlv.skip_cells == 0 as ::core::ffi::c_int {
                            is_concealing = true_0 != 0;
                            wlv.skip_cells = 1 as ::core::ffi::c_int;
                        }
                    } else {
                        prev_syntax_id = 0 as ::core::ffi::c_int;
                        is_concealing = false_0 != 0;
                    }
                    if wlv.skip_cells > 0 as ::core::ffi::c_int
                        && did_decrement_ptr as ::core::ffi::c_int != 0
                    {
                        ptr = ptr.offset(1);
                    }
                }
                if !did_wcol
                    && wlv.filler_todo <= 0 as ::core::ffi::c_int
                    && in_curline as ::core::ffi::c_int != 0
                    && conceal_cursor_line(wp) as ::core::ffi::c_int != 0
                    && (wlv.vcol as ::core::ffi::c_int + wlv.skip_cells >= (*wp).w_virtcol
                        || mb_schar == NUL as schar_T)
                {
                    (*wp).w_wcol = wlv.col - wlv.boguscols;
                    if wlv.vcol as ::core::ffi::c_int + wlv.skip_cells < (*wp).w_virtcol {
                        (*wp).w_wcol += (*wp).w_virtcol as ::core::ffi::c_int
                            - wlv.vcol as ::core::ffi::c_int
                            - wlv.skip_cells;
                    }
                    (*wp).w_wrow = wlv.row;
                    did_wcol = true_0 != 0;
                    (*wp).w_valid |= VALID_WCOL | VALID_WROW | VALID_VIRTCOL;
                }
                if wlv.n_attr > 0 as ::core::ffi::c_int && !search_attr_from_match {
                    wlv.char_attr = hl_combine_attr(wlv.char_attr, wlv.extra_attr);
                    if wlv.reset_extra_attr {
                        wlv.reset_extra_attr = false_0 != 0;
                        if extra_attr_next >= 0 as ::core::ffi::c_int {
                            wlv.extra_attr = extra_attr_next;
                            extra_attr_next = -1 as ::core::ffi::c_int;
                        } else {
                            wlv.extra_attr = 0 as ::core::ffi::c_int;
                            search_attr_from_match = saved_search_attr_from_match;
                        }
                    }
                }
                if lcs_prec_todo != NUL as schar_T
                    && (*wp).w_onebuf_opt.wo_list != 0
                    && (if (*wp).w_onebuf_opt.wo_wrap != 0 {
                        ((*wp).w_skipcol > 0 as ::core::ffi::c_int
                            && wlv.row == 0 as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                    } else {
                        ((*wp).w_leftcol > 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                    }) != 0
                    && wlv.filler_todo <= 0 as ::core::ffi::c_int
                    && wlv.skip_cells <= 0 as ::core::ffi::c_int
                    && mb_schar != NUL as schar_T
                {
                    lcs_prec_todo = NUL as schar_T;
                    if schar_cells(mb_schar) > 1 as ::core::ffi::c_int {
                        wlv.sc_extra = '<' as ::core::ffi::c_int as schar_T;
                        wlv.sc_final = NUL as schar_T;
                        if wlv.n_extra > 0 as ::core::ffi::c_int {
                            '_c2rust_label_9: {
                                if !wlv.p_extra.is_null() {
                                } else {
                                    __assert_fail(
                                        b"wlv.p_extra != NULL\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/drawline.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2749 as ::core::ffi::c_uint,
                                        b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            n_extra_next = wlv.n_extra;
                            extra_attr_next = wlv.extra_attr;
                            wlv.n_attr =
                                if wlv.n_attr + 1 as ::core::ffi::c_int > 2 as ::core::ffi::c_int {
                                    wlv.n_attr + 1 as ::core::ffi::c_int
                                } else {
                                    2 as ::core::ffi::c_int
                                };
                        } else {
                            wlv.n_attr = 2 as ::core::ffi::c_int;
                        }
                        wlv.n_extra = 1 as ::core::ffi::c_int;
                        wlv.extra_attr = win_hl_attr(wp, HLF_AT);
                    }
                    mb_schar = (*wp).w_p_lcs_chars.prec;
                    mb_c = schar_get_first_codepoint(mb_schar);
                    saved_attr3 = wlv.char_attr;
                    wlv.char_attr = win_hl_attr(wp, HLF_AT);
                    n_attr3 = 1 as ::core::ffi::c_int;
                }
                if mb_schar == NUL as schar_T && eol_hl_off == 0 as ::core::ffi::c_int {
                    let prevcol_hl_flag: bool = get_prevcol_hl_flag(
                        wp,
                        screen_search_hl.ptr(),
                        ptr.offset_from(line) as colnr_T - 1 as colnr_T,
                    );
                    if lcs_eol_todo as ::core::ffi::c_int != 0
                        && (area_attr != 0 as ::core::ffi::c_int
                            && wlv.vcol == wlv.fromcol
                            && (VIsual_mode.get() != Ctrl_V
                                || lnum == (*VIsual.ptr()).lnum
                                || lnum == (*curwin.get()).w_cursor.lnum)
                            || prevcol_hl_flag as ::core::ffi::c_int != 0)
                    {
                        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if wlv.col >= view_width {
                            n = -1 as ::core::ffi::c_int;
                        }
                        if n != 0 as ::core::ffi::c_int {
                            wlv.off += n;
                            wlv.col += n;
                        } else {
                            *(*linebuf_char.ptr()).offset(wlv.off as isize) =
                                ' ' as ::core::ffi::c_int as schar_T;
                        }
                        if area_attr == 0 as ::core::ffi::c_int && !has_fold {
                            get_search_match_hl(
                                wp,
                                screen_search_hl.ptr(),
                                ptr.offset_from(line) as colnr_T,
                                &raw mut wlv.char_attr,
                            );
                        }
                        let eol_attr: ::core::ffi::c_int = if wlv.cursorline_attr != 0 {
                            hl_combine_attr(wlv.cursorline_attr, wlv.char_attr)
                        } else {
                            wlv.char_attr
                        };
                        *(*linebuf_attr.ptr()).offset(wlv.off as isize) = eol_attr as sattr_T;
                        *(*linebuf_vcol.ptr()).offset(wlv.off as isize) = wlv.vcol;
                        wlv.col += 1;
                        wlv.off += 1;
                        wlv.vcol += 1;
                        eol_hl_off = 1 as ::core::ffi::c_int;
                    }
                }
                if mb_schar == NUL as schar_T {
                    wlv.vcol = (if wlv.vcol > start_vcol + wlv.col - win_col_off(wp) {
                        wlv.vcol as ::core::ffi::c_int
                    } else {
                        start_vcol + wlv.col - win_col_off(wp)
                    }) as colnr_T;
                    wlv.col -= wlv.boguscols;
                    wlv.boguscols = 0 as ::core::ffi::c_int;
                    wlv.advance_color_col(wlv.vcol - wlv.vcol_off_co);
                    let eol_skip: ::core::ffi::c_int = if lcs_eol_todo as ::core::ffi::c_int != 0
                        && eol_hl_off == 0 as ::core::ffi::c_int
                    {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                    if has_decor {
                        decor_redraw_eol(
                            wp,
                            decor_state.ptr(),
                            &raw mut wlv.line_attr,
                            wlv.col + eol_skip,
                        );
                    }
                    let mut i_0: ::core::ffi::c_int = wlv.col;
                    while i_0 < view_width {
                        *(*linebuf_vcol.ptr()).offset((wlv.off + (i_0 - wlv.col)) as isize) =
                            (wlv.vcol as ::core::ffi::c_int + (i_0 - wlv.col)) as colnr_T;
                        i_0 += 1;
                    }
                    if (*wp).w_onebuf_opt.wo_cuc != 0
                        && (*wp).w_virtcol
                            >= wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co - eol_hl_off
                        && ((*wp).w_virtcol as ptrdiff_t)
                            < view_width as ptrdiff_t
                                * (wlv.row - startrow + 1 as ::core::ffi::c_int) as ptrdiff_t
                                + start_vcol as ptrdiff_t
                        && lnum != (*wp).w_cursor.lnum
                        || !wlv.color_cols.is_null()
                        || wlv.line_attr_lowprio != 0
                        || wlv.line_attr != 0
                        || wlv.diff_hlf as ::core::ffi::c_uint != 0 as ::core::ffi::c_uint
                        || !(*(*wp).w_buffer).terminal.is_null()
                    {
                        let mut rightmost_vcol: ::core::ffi::c_int =
                            get_rightmost_vcol(wp, wlv.color_cols);
                        let cuc_attr: ::core::ffi::c_int = win_hl_attr(wp, HLF_CUC);
                        let mc_attr: ::core::ffi::c_int = win_hl_attr(wp, HLF_MC);
                        if wlv.diff_hlf as ::core::ffi::c_uint == HLF_TXD as ::core::ffi::c_uint
                            || wlv.diff_hlf as ::core::ffi::c_uint == HLF_TXA as ::core::ffi::c_uint
                        {
                            wlv.diff_hlf = HLF_CHD;
                            wlv.set_line_attr_for_diff(wp);
                        }
                        let diff_attr: ::core::ffi::c_int =
                            if wlv.diff_hlf as ::core::ffi::c_uint != 0 as ::core::ffi::c_uint {
                                win_hl_attr(wp, wlv.diff_hlf as ::core::ffi::c_int)
                            } else {
                                0 as ::core::ffi::c_int
                            };
                        let base_attr: ::core::ffi::c_int =
                            hl_combine_attr(wlv.line_attr_lowprio, diff_attr);
                        if base_attr != 0
                            || wlv.line_attr != 0
                            || !(*(*wp).w_buffer).terminal.is_null()
                        {
                            rightmost_vcol = INT_MAX;
                        }
                        while wlv.col < view_width {
                            *(*linebuf_char.ptr()).offset(wlv.off as isize) =
                                ' ' as ::core::ffi::c_int as schar_T;
                            wlv.advance_color_col(wlv.vcol - wlv.vcol_off_co);
                            let mut col_attr: ::core::ffi::c_int = base_attr;
                            if (*wp).w_onebuf_opt.wo_cuc != 0
                                && wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co
                                    == (*wp).w_virtcol
                                && lnum != (*wp).w_cursor.lnum
                            {
                                col_attr = hl_combine_attr(col_attr, cuc_attr);
                            } else if !wlv.color_cols.is_null()
                                && wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co
                                    == *wlv.color_cols
                            {
                                col_attr = hl_combine_attr(col_attr, mc_attr);
                            }
                            if !(*(*wp).w_buffer).terminal.is_null()
                                && wlv.vcol < TERM_ATTRS_MAX as ::core::ffi::c_int
                            {
                                col_attr = hl_combine_attr(col_attr, term_attrs[wlv.vcol as usize]);
                            }
                            col_attr = hl_combine_attr(col_attr, wlv.line_attr);
                            *(*linebuf_attr.ptr()).offset(wlv.off as isize) = col_attr as sattr_T;
                            wlv.off += 1;
                            wlv.col += 1;
                            wlv.vcol += 1;
                            if wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co > rightmost_vcol {
                                break;
                            }
                        }
                    }
                    if fold_vt.size > 0 as size_t {
                        draw_virt_text_item(
                            buf,
                            win_col_offset,
                            fold_vt,
                            kHlModeCombine,
                            view_width,
                            0 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                    }
                    draw_virt_text(wp, buf, win_col_offset, &mut wlv.col, wlv.row);
                    wlv_put_linebuf(
                        wp,
                        &raw mut wlv,
                        wlv.col,
                        true_0 != 0,
                        bg_attr,
                        SLF_INC_VCOL as ::core::ffi::c_int,
                    );
                    wlv.row += 1;
                    if in_curline {
                        (*curwin.get()).w_cline_row = startrow;
                        (*curwin.get()).w_cline_height = wlv.row - startrow;
                        (*curwin.get()).w_cline_folded = has_fold;
                        (*curwin.get()).w_valid |= VALID_CHEIGHT | VALID_CROW;
                    }
                    break 's_5143;
                } else {
                    lcs_ext = get_lcs_ext(wp);
                    if lcs_ext != NUL as schar_T
                        && wlv.filler_todo <= 0 as ::core::ffi::c_int
                        && wlv.col == view_width - 1 as ::core::ffi::c_int
                        && !has_foldtext
                    {
                        if has_decor as ::core::ffi::c_int != 0
                            && *ptr as ::core::ffi::c_int == NUL
                            && lcs_eol == 0 as schar_T
                            && lcs_eol_todo as ::core::ffi::c_int != 0
                        {
                            decor_redraw_col(
                                wp,
                                ptr.offset_from(line) as ::core::ffi::c_int,
                                -1 as ::core::ffi::c_int,
                                false_0 != 0,
                                decor_state.ptr(),
                                decor_provider_end_col - 1 as ::core::ffi::c_int,
                            );
                        }
                        if *ptr as ::core::ffi::c_int != NUL
                            || lcs_eol > 0 as schar_T && lcs_eol_todo as ::core::ffi::c_int != 0
                            || wlv.n_extra > 0 as ::core::ffi::c_int
                                && (wlv.sc_extra != NUL as schar_T
                                    || *wlv.p_extra as ::core::ffi::c_int != NUL)
                            || may_have_inline_virt as ::core::ffi::c_int != 0
                                && wlv.has_more_inline_virt(ptr.offset_from(line))
                                    as ::core::ffi::c_int
                                    != 0
                        {
                            mb_schar = lcs_ext;
                            wlv.char_attr = win_hl_attr(wp, HLF_AT);
                            mb_c = schar_get_first_codepoint(mb_schar);
                        }
                    }
                    wlv.advance_color_col(wlv.vcol - wlv.vcol_off_co);
                    vcol_save_attr = -1 as ::core::ffi::c_int;
                    if !lnum_in_visual_area
                        && search_attr == 0 as ::core::ffi::c_int
                        && area_attr == 0 as ::core::ffi::c_int
                        && wlv.filler_todo <= 0 as ::core::ffi::c_int
                    {
                        if (*wp).w_onebuf_opt.wo_cuc != 0
                            && wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co == (*wp).w_virtcol
                            && lnum != (*wp).w_cursor.lnum
                        {
                            vcol_save_attr = wlv.char_attr;
                            wlv.char_attr =
                                hl_combine_attr(win_hl_attr(wp, HLF_CUC), wlv.char_attr);
                        } else if !wlv.color_cols.is_null()
                            && wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co == *wlv.color_cols
                        {
                            vcol_save_attr = wlv.char_attr;
                            wlv.char_attr = hl_combine_attr(win_hl_attr(wp, HLF_MC), wlv.char_attr);
                        }
                    }
                    if wlv.filler_todo <= 0 as ::core::ffi::c_int {
                        let mut low: ::core::ffi::c_int = wlv.line_attr_lowprio;
                        let mut high: ::core::ffi::c_int = wlv.char_attr;
                        if wlv.line_attr_lowprio != 0 as ::core::ffi::c_int {
                            let mut line_ae: HlAttrs = syn_attr2entry(wlv.line_attr_lowprio);
                            let mut char_ae: HlAttrs = syn_attr2entry(wlv.char_attr);
                            let mut win_normal_bg: ::core::ffi::c_int =
                                normal_bg.get() as ::core::ffi::c_int;
                            let mut win_normal_cterm_bg: ::core::ffi::c_int =
                                cterm_normal_bg_color.get();
                            if bg_attr != 0 as ::core::ffi::c_int {
                                let mut norm_ae: HlAttrs = syn_attr2entry(bg_attr);
                                win_normal_bg = norm_ae.rgb_bg_color as ::core::ffi::c_int;
                                win_normal_cterm_bg = norm_ae.cterm_bg_color as ::core::ffi::c_int;
                            }
                            let mut char_is_normal_bg: bool =
                                if ui_rgb_attached() as ::core::ffi::c_int != 0 {
                                    (char_ae.rgb_bg_color == win_normal_bg as RgbValue)
                                        as ::core::ffi::c_int
                                } else {
                                    (char_ae.cterm_bg_color as ::core::ffi::c_int
                                        == win_normal_cterm_bg)
                                        as ::core::ffi::c_int
                                } != 0;
                            if (line_ae.rgb_bg_color >= 0 as RgbValue
                                || line_ae.cterm_bg_color as ::core::ffi::c_int
                                    > 0 as ::core::ffi::c_int)
                                && char_is_normal_bg as ::core::ffi::c_int != 0
                            {
                                low = wlv.char_attr;
                                high = wlv.line_attr_lowprio;
                            }
                        }
                        wlv.char_attr = hl_combine_attr(low, high);
                    }
                    if wlv.filler_todo <= 0 as ::core::ffi::c_int {
                        vcol_prev = wlv.vcol;
                    }
                    if wlv.filler_todo <= 0 as ::core::ffi::c_int {
                        if wlv.skip_cells <= 0 as ::core::ffi::c_int {
                            *(*linebuf_char.ptr()).offset(wlv.off as isize) = mb_schar;
                            if multi_attr != 0 {
                                *(*linebuf_attr.ptr()).offset(wlv.off as isize) =
                                    multi_attr as sattr_T;
                                multi_attr = 0 as ::core::ffi::c_int;
                            } else {
                                *(*linebuf_attr.ptr()).offset(wlv.off as isize) =
                                    wlv.char_attr as sattr_T;
                            }
                            *(*linebuf_vcol.ptr()).offset(wlv.off as isize) = wlv.vcol;
                            if schar_cells(mb_schar) > 1 as ::core::ffi::c_int {
                                wlv.off += 1;
                                wlv.col += 1;
                                *(*linebuf_char.ptr()).offset(wlv.off as isize) = 0 as schar_T;
                                *(*linebuf_attr.ptr()).offset(wlv.off as isize) = *(*linebuf_attr
                                    .ptr())
                                .offset((wlv.off - 1 as ::core::ffi::c_int) as isize);
                                wlv.vcol += 1;
                                *(*linebuf_vcol.ptr()).offset(wlv.off as isize) = wlv.vcol;
                                if wlv.tocol == wlv.vcol {
                                    wlv.tocol += 1;
                                }
                            }
                            wlv.off += 1;
                            wlv.col += 1;
                        } else if (*wp).w_onebuf_opt.wo_cole > 0 as OptInt
                            && is_concealing as ::core::ffi::c_int != 0
                        {
                            let mut concealed_wide: bool =
                                schar_cells(mb_schar) > 1 as ::core::ffi::c_int;
                            wlv.skip_cells -= 1;
                            wlv.vcol_off_co += 1;
                            if concealed_wide {
                                wlv.vcol += 1;
                                wlv.vcol_off_co += 1;
                            }
                            if wlv.n_extra > 0 as ::core::ffi::c_int {
                                wlv.vcol_off_co += wlv.n_extra;
                            }
                            if is_wrapped {
                                if wlv.n_extra > 0 as ::core::ffi::c_int {
                                    wlv.vcol += wlv.n_extra;
                                    wlv.col += wlv.n_extra;
                                    wlv.boguscols += wlv.n_extra;
                                    wlv.n_extra = 0 as ::core::ffi::c_int;
                                    wlv.n_attr = 0 as ::core::ffi::c_int;
                                }
                                if concealed_wide {
                                    wlv.boguscols += 1;
                                    wlv.col += 1;
                                }
                                wlv.boguscols += 1;
                                wlv.col += 1;
                            } else if wlv.n_extra > 0 as ::core::ffi::c_int {
                                wlv.vcol += wlv.n_extra;
                                wlv.n_extra = 0 as ::core::ffi::c_int;
                                wlv.n_attr = 0 as ::core::ffi::c_int;
                            }
                        } else {
                            wlv.skip_cells -= 1;
                        }
                    }
                    if wlv.skipped_cells > 0 as ::core::ffi::c_int {
                        wlv.vcol += wlv.skipped_cells;
                        wlv.skipped_cells = 0 as ::core::ffi::c_int;
                    }
                    if wlv.filler_todo <= 0 as ::core::ffi::c_int {
                        wlv.vcol += 1;
                    }
                    if vcol_save_attr >= 0 as ::core::ffi::c_int {
                        wlv.char_attr = vcol_save_attr;
                    }
                    if n_attr3 > 0 as ::core::ffi::c_int && {
                        n_attr3 -= 1;
                        n_attr3 == 0 as ::core::ffi::c_int
                    } {
                        wlv.char_attr = saved_attr3;
                    }
                    if wlv.n_attr > 0 as ::core::ffi::c_int && {
                        wlv.n_attr -= 1;
                        wlv.n_attr == 0 as ::core::ffi::c_int
                    } {
                        wlv.char_attr = saved_attr2;
                    }
                    if has_decor as ::core::ffi::c_int != 0
                        && wlv.filler_todo <= 0 as ::core::ffi::c_int
                        && wlv.col >= view_width
                    {
                        if is_wrapped as ::core::ffi::c_int != 0
                            && wlv.n_extra == 0 as ::core::ffi::c_int
                        {
                            decor_redraw_col(
                                wp,
                                ptr.offset_from(line) as ::core::ffi::c_int,
                                -3 as ::core::ffi::c_int,
                                false_0 != 0,
                                decor_state.ptr(),
                                decor_provider_end_col - 1 as ::core::ffi::c_int,
                            );
                            decor_need_recheck = true_0 != 0;
                        } else if !is_wrapped {
                            decor_recheck_draw_col(
                                -1 as ::core::ffi::c_int,
                                true_0 != 0,
                                decor_state.ptr(),
                            );
                            decor_redraw_col(
                                wp,
                                MAXCOL as ::core::ffi::c_int,
                                -1 as ::core::ffi::c_int,
                                true_0 != 0,
                                decor_state.ptr(),
                                decor_provider_end_col - 1 as ::core::ffi::c_int,
                            );
                        }
                    }
                }
            }
        }
        if !(wlv.col >= view_width
            && (!has_foldtext || wlv.filler_todo > 0 as ::core::ffi::c_int)
            && (wlv.col <= leftcols_width
                || *ptr as ::core::ffi::c_int != NUL
                || wlv.filler_todo > 0 as ::core::ffi::c_int
                || (*wp).w_onebuf_opt.wo_list != 0
                    && (*wp).w_p_lcs_chars.eol != NUL as schar_T
                    && lcs_eol_todo as ::core::ffi::c_int != 0
                || wlv.n_extra != 0 as ::core::ffi::c_int
                    && (wlv.sc_extra != NUL as schar_T
                        || *wlv.p_extra as ::core::ffi::c_int != NUL)
                || may_have_inline_virt as ::core::ffi::c_int != 0
                    && wlv.has_more_inline_virt(ptr.offset_from(line)) as ::core::ffi::c_int != 0))
        {
            continue;
        }
        let mut grid_width: ::core::ffi::c_int = (*(*wp).w_grid.target).cols;
        let wrap: bool = is_wrapped as ::core::ffi::c_int != 0
            && wlv.filler_todo <= 0 as ::core::ffi::c_int
            && lcs_eol_todo as ::core::ffi::c_int != 0
            && wlv.row != endrow - 1 as ::core::ffi::c_int
            && view_width == grid_width
            && (*wp).w_onebuf_opt.wo_rl == 0;
        let mut draw_col: ::core::ffi::c_int = wlv.col - wlv.boguscols;
        let mut i_1: ::core::ffi::c_int = draw_col;
        while i_1 < view_width {
            *(*linebuf_vcol.ptr()).offset((wlv.off + (i_1 - draw_col)) as isize) =
                (wlv.vcol as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
            i_1 += 1;
        }
        if wlv.boguscols != 0 as ::core::ffi::c_int
            && (wlv.line_attr_lowprio != 0 as ::core::ffi::c_int
                || wlv.line_attr != 0 as ::core::ffi::c_int)
        {
            let mut attr: ::core::ffi::c_int =
                hl_combine_attr(wlv.line_attr_lowprio, wlv.line_attr);
            while draw_col < view_width {
                *(*linebuf_char.ptr()).offset(wlv.off as isize) =
                    schar_from_char(' ' as ::core::ffi::c_int);
                *(*linebuf_attr.ptr()).offset(wlv.off as isize) = attr as sattr_T;
                wlv.off += 1;
                draw_col += 1;
            }
        }
        if virt_line_index >= 0 as ::core::ffi::c_int {
            draw_virt_text_item(
                buf,
                if virt_line_flags & kVLLeftcol as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    win_col_offset
                },
                (*virt_lines.items.offset(virt_line_index as isize)).line,
                kHlModeReplace,
                view_width,
                0 as ::core::ffi::c_int,
                if virt_line_flags & kVLScroll as ::core::ffi::c_int != 0 {
                    (*wp).w_leftcol as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                },
            );
        } else if wlv.filler_todo <= 0 as ::core::ffi::c_int {
            draw_virt_text(wp, buf, win_col_offset, &mut draw_col, wlv.row);
        }
        wlv_put_linebuf(
            wp,
            &raw mut wlv,
            draw_col,
            true_0 != 0,
            bg_attr,
            if wrap as ::core::ffi::c_int != 0 {
                SLF_WRAP as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        );
        if wrap {
            let mut current_row: ::core::ffi::c_int = wlv.row;
            let mut dummy_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut current_grid: *mut ScreenGrid =
                grid_adjust(grid, &raw mut current_row, &raw mut dummy_col);
            *(*current_grid).attrs.offset(
                *(*current_grid)
                    .line_offset
                    .offset((current_row + 1 as ::core::ffi::c_int) as isize)
                    as isize,
            ) = -1 as ::core::ffi::c_int as sattr_T;
        }
        wlv.boguscols = 0 as ::core::ffi::c_int;
        wlv.vcol_off_co = 0 as ::core::ffi::c_int;
        wlv.row += 1;
        if !is_wrapped && wlv.filler_todo <= 0 as ::core::ffi::c_int {
            break;
        }
        if wlv.col <= leftcols_width {
            win_draw_end(
                wp,
                '@' as ::core::ffi::c_int as schar_T,
                true_0 != 0,
                wlv.row,
                (*wp).w_view_height,
                HLF_AT,
            );
            set_empty_rows(wp, wlv.row);
            wlv.row = endrow;
        }
        if wlv.row == endrow {
            wlv.row += 1;
            break;
        } else {
            wlv.start_line(wp);
            draw_cols = true_0 != 0;
            lcs_prec_todo = (*wp).w_p_lcs_chars.prec;
            if wlv.filler_todo <= 0 as ::core::ffi::c_int {
                wlv.need_showbreak = true_0 != 0;
            }
            if statuscol.draw as ::core::ffi::c_int != 0
                && !vim_strchr(p_cpo.get(), CPO_NUMCOL).is_null()
                && wlv.row > startrow + wlv.filler_lines
            {
                statuscol.draw = false_0 != 0;
            }
            wlv.filler_todo -= 1;
            virt_line_index = -1 as ::core::ffi::c_int;
            virt_line_flags = 0 as ::core::ffi::c_int;
            if wlv.filler_todo == 0 as ::core::ffi::c_int
                && ((*wp).w_botfill as ::core::ffi::c_int != 0 || !draw_text)
            {
                break;
            }
        }
    }
    clear_virttext(&raw mut fold_vt);
    xfree(virt_lines.items as *mut ::core::ffi::c_void);
    virt_lines.capacity = 0 as size_t;
    virt_lines.size = virt_lines.capacity;
    virt_lines.items = ::core::ptr::null_mut::<virt_line>();
    xfree(foldtext_free as *mut ::core::ffi::c_void);
    return wlv.row;
}
pub const SPWORDLEN: ::core::ffi::c_int = 150 as ::core::ffi::c_int;
unsafe extern "C" fn wlv_put_linebuf(
    mut wp: *mut win_T,
    mut wlv: *const WinLineVars,
    mut endcol: ::core::ffi::c_int,
    mut clear_end: bool,
    mut bg_attr: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) {
    let mut grid: *mut GridView = &raw mut (*wp).w_grid;
    let mut startcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut clear_width: ::core::ffi::c_int = if clear_end as ::core::ffi::c_int != 0 {
        (*wp).w_view_width
    } else {
        endcol
    };
    '_c2rust_label: {
        if flags & SLF_RIGHTLEFT as ::core::ffi::c_int == 0 {
        } else {
            __assert_fail(
                b"!(flags & SLF_RIGHTLEFT)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3253 as ::core::ffi::c_uint,
                b"void wlv_put_linebuf(win_T *, const WinLineVars *, int, _Bool, int, int)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if (*wp).w_onebuf_opt.wo_rl != 0 {
        linebuf_mirror(
            &mut startcol,
            &mut endcol,
            &mut clear_width,
            (*wp).w_view_width,
        );
        flags |= SLF_RIGHTLEFT as ::core::ffi::c_int;
    }
    if (*wlv).row == 0 as ::core::ffi::c_int
        && (*wp).w_skipcol > 0 as ::core::ffi::c_int
        && *get_showbreak_value(wp) as ::core::ffi::c_int == NUL
        && !((*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.prec != 0 as schar_T)
    {
        let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*wp).w_onebuf_opt.wo_nu != 0 && (*wp).w_onebuf_opt.wo_rnu != 0 {
            while off < (*wp).w_view_width
                && ascii_isdigit(schar_get_ascii(*(*linebuf_char.ptr()).offset(off as isize))
                    as ::core::ffi::c_int) as ::core::ffi::c_int
                    != 0
            {
                off += 1;
            }
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 3 as ::core::ffi::c_int && off < (*wp).w_view_width {
            if (off + 1 as ::core::ffi::c_int) < (*wp).w_view_width
                && *(*linebuf_char.ptr()).offset((off + 1 as ::core::ffi::c_int) as isize)
                    == NUL as schar_T
            {
                *(*linebuf_char.ptr()).offset((off + 1 as ::core::ffi::c_int) as isize) =
                    ' ' as ::core::ffi::c_int as schar_T;
            }
            *(*linebuf_char.ptr()).offset(off as isize) = '<' as ::core::ffi::c_int as schar_T;
            *(*linebuf_attr.ptr()).offset(off as isize) =
                *(*hl_attr_active.ptr()).offset(HLF_AT as isize) as sattr_T;
            off += 1;
            i += 1;
        }
    }
    let mut row: ::core::ffi::c_int = (*wlv).row;
    let mut coloff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut g: *mut ScreenGrid = grid_adjust(grid, &raw mut row, &raw mut coloff);
    grid_put_linebuf(
        g,
        row,
        coloff,
        LineSpan {
            col: startcol,
            endcol,
            clear_width,
        },
        LineAttrs {
            bg: bg_attr,
            clear: 0,
        },
        (*wlv).vcol - 1 as colnr_T,
        flags,
    );
}
unsafe extern "C" fn decor_providers_setup(
    mut rows_to_draw: ::core::ffi::c_int,
    mut draw_from_line_start: bool,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut wp: *mut win_T,
) -> ::core::ffi::c_int {
    let mut rem_vcols: ::core::ffi::c_int = 0;
    if (*wp).w_onebuf_opt.wo_wrap != 0 {
        let mut width: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
        let mut width2: ::core::ffi::c_int = width + win_col_off2(wp);
        let mut first_row_width: ::core::ffi::c_int =
            if draw_from_line_start as ::core::ffi::c_int != 0 {
                width
            } else {
                width2
            };
        rem_vcols = first_row_width + (rows_to_draw - 1 as ::core::ffi::c_int) * width2;
    } else {
        rem_vcols = (*wp).w_view_width - win_col_off(wp);
    }
    decor_providers_invoke_line(wp, lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    validate_virtcol(wp);
    return invoke_range_next(
        wp,
        lnum as ::core::ffi::c_int,
        col,
        rem_vcols as colnr_T + 1 as colnr_T,
    );
}
unsafe extern "C" fn invoke_range_next(
    mut wp: *mut win_T,
    mut lnum: ::core::ffi::c_int,
    mut begin_col: colnr_T,
    mut col_off: colnr_T,
) -> ::core::ffi::c_int {
    let line: *const ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum as linenr_T);
    let line_len: ::core::ffi::c_int = ml_get_buf_len((*wp).w_buffer, lnum as linenr_T);
    col_off = (if col_off > 1 as ::core::ffi::c_int {
        col_off as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    }) as colnr_T;
    let mut new_col: colnr_T = 0;
    if col_off <= line_len as colnr_T - begin_col {
        let mut end_col: ::core::ffi::c_int =
            begin_col as ::core::ffi::c_int + col_off as ::core::ffi::c_int;
        end_col += mb_off_next(line, line.offset(end_col as isize));
        decor_providers_invoke_range(
            wp,
            lnum - 1 as ::core::ffi::c_int,
            begin_col as ::core::ffi::c_int,
            lnum - 1 as ::core::ffi::c_int,
            end_col,
        );
        validate_virtcol(wp);
        new_col = end_col as colnr_T;
    } else {
        decor_providers_invoke_range(
            wp,
            lnum - 1 as ::core::ffi::c_int,
            begin_col as ::core::ffi::c_int,
            lnum,
            0 as ::core::ffi::c_int,
        );
        validate_virtcol(wp);
        new_col = INT_MAX as colnr_T;
    }
    return new_col as ::core::ffi::c_int;
}
pub const INT_MIN: ::core::ffi::c_int = -INT_MAX - 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
