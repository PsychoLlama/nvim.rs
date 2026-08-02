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
    CharSize, CharsizeArg, DecorRange, DecorRangeKind, DecorVirtText, GridView, MetaIndex, NS,
    RgbValue, ScreenGrid, SignTextAttrs, StlFlag, TriState, VimVarIndex, VirtLines, VirtText,
    VirtTextChunk, VirtTextPos, WinExtmark, buf_T, colnr_T, diffline_T, foldinfo_T, hlf_T,
    linenr_T, pos_T, ptrdiff_t, sattr_T, schar_T, size_t, smt_T, spellvars_T, ssize_t, statuscol_T,
    uint8_t, uint32_t, uint64_t, varnumber_T, virt_line, win_T,
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
mod cells;
pub(crate) use self::cells::*;
mod attrs;
mod chars;
mod rows;
mod special;
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
/// Draw one buffer line, and answer the window row after it.
///
/// The work is in two halves: [`prepare_line`] decides everything that is
/// true of the line as a whole, and [`Cells::run`] walks it cell by cell for
/// as many screen rows as it takes.
///
/// `startrow` is the first window row the line may use and `endrow` one past
/// the last; `col_rows` is non-zero when only the info columns are being
/// redrawn, over that many rows. `concealed` says the whole line is hidden by
/// a decoration, `spv` carries the redraw's spell state and `foldinfo`
/// whatever `win_update` found out about folds here.
///
/// # Safety
/// `wp` must be a live window, `lnum` one of its buffer's lines, and `spv` a
/// live `spellvars_T`.
pub unsafe extern "C" fn win_line(
    wp: *mut win_T,
    lnum: linenr_T,
    startrow: ::core::ffi::c_int,
    endrow: ::core::ffi::c_int,
    col_rows: ::core::ffi::c_int,
    concealed: bool,
    spv: *mut spellvars_T,
    foldinfo: foldinfo_T,
) -> ::core::ffi::c_int {
    // SAFETY: the caller's window, line and spell state.
    unsafe {
        assert!(startrow < endrow);

        let mut wlv = WinLineVars {
            lnum,
            foldinfo,
            startrow,
            row: startrow,
            vcol: 0,
            col: 0,
            boguscols: 0,
            old_boguscols: 0,
            vcol_off_co: 0,
            off: 0,
            cursorline_attr: 0,
            line_attr: 0,
            line_attr_lowprio: 0,
            sign_num_attr: 0,
            prev_num_attr: -1,
            sign_cul_attr: 0,
            // `fromcol` -10 and `tocol` MAXCOL mean "no inverted range at
            // all", which is a different thing from an empty one.
            fromcol: -10,
            tocol: MAXCOL as ::core::ffi::c_int,
            showbreak_vcol: -1,
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
            virt_inline: VIRTTEXT_EMPTY,
            virt_inline_i: 0,
            virt_inline_hl_mode: kHlModeUnknown,
            reset_extra_attr: false,
            skip_cells: 0,
            skipped_cells: 0,
            color_cols: ::core::ptr::null_mut::<::core::ffi::c_int>(),
        };
        let buf: *mut buf_T = (*wp).w_buffer;

        // The two scratch buffers the loop needs but never owns: the spell
        // look-ahead, filled by the setup half, and the fold text.
        let mut nextline: SpellLookahead = [0; SPELL_LOOKAHEAD * 2];
        let mut fold_buf: [::core::ffi::c_char; FOLD_TEXT_LEN as usize] =
            [0; FOLD_TEXT_LEN as usize];

        let setup = prepare_line(
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
        if setup.has_terminal {
            terminal_get_line_attributes(
                (*(*wp).w_buffer).terminal,
                wp,
                lnum,
                term_attrs.as_mut_ptr(),
            );
        }

        // `sattrs` is attached here rather than by the setup half: it points
        // into `wlv`, which lives in this frame, and a pointer derived from
        // the borrow the setup half takes would not outlive it.
        let mut statuscol = setup.statuscol;
        statuscol.sattrs = &raw mut wlv.sign_attrs as *mut SignTextAttrs;

        let frame = LineFrame {
            endrow,
            col_rows,
            spv,
            statuscol: &raw mut statuscol,
            term_attrs: term_attrs.as_ptr(),
            nextline: nextline.as_mut_ptr(),
            fold_buf: fold_buf.as_mut_ptr(),
        };
        Cells::new(setup).run(&mut wlv, wp, buf, &frame)
    }
}

/// How many bytes of the next line the spell checker joins onto this one, so
/// that a word wrapping across the line break can be checked whole.
pub const SPWORDLEN: ::core::ffi::c_int = 150;
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
