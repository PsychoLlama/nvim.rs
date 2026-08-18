#![deny(unsafe_op_in_unsafe_fn)]

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::buffer::{bt_quickfix, buf_meta_total};
use crate::charset::{
    byte2cells, rl_mirror_ascii, skiptowhite, skipwhite, transchar_buf, transchar_hex,
    transstr_buf, vim_isbreak, vim_isprintc,
};
use crate::cursor::get_cursor_rel_lnum;
use crate::cursor_shape::cursor_is_block_during_visual;
use crate::decoration::{
    clear_virttext, decor_has_more_decorations, decor_init_draw_col, decor_range_at,
    decor_range_count, decor_recheck_draw_col, decor_redraw_col, decor_redraw_eol,
    decor_redraw_line, decor_redraw_signs, decor_virt_lines, decor_virt_pos, decor_virt_pos_kind,
    kHlModeUnknown, next_virt_text_chunk,
};
use crate::decoration_provider::{decor_providers_invoke_line, decor_providers_invoke_range};
use crate::diff::{diff_change_parse, diff_check_with_linestatus, diff_find_change};
use crate::drawscreen::{compute_foldcolumn, conceal_cursor_line, number_width, win_draw_end};
use crate::eval::vars::set_vim_var_nr;
use crate::fold::{FOLD_TEXT_LEN, VIRTTEXT_EMPTY, get_foldtext};
use crate::global_cell::GlobalCell;
use crate::grid::{
    LineAttrs, LineSpan, SLF_RIGHTLEFT, grid_adjust, grid_put_linebuf, linebuf_mirror, schar_cells,
    schar_from_ascii, schar_from_char, schar_get_adv, schar_get_ascii, schar_get_first_codepoint,
    schar_len,
};
use crate::highlight::{
    hl_blend_attrs, hl_combine_attr, hl_get_underline, syn_attr2entry, win_bg_attr, win_hl_attr,
};
use crate::highlight_group::{
    HLF_0, HLF_8, HLF_ADD, HLF_AT, HLF_CHD, HLF_CLF, HLF_CLN, HLF_CLS, HLF_CONCEAL, HLF_COUNT,
    HLF_CUC, HLF_CUL, HLF_DED, HLF_FC, HLF_FL, HLF_I, HLF_LNA, HLF_LNB, HLF_MC, HLF_N, HLF_NONE,
    HLF_QFL, HLF_SC, HLF_TXA, HLF_TXD, HLF_V, syn_id2attr,
};
use crate::indent::{get_breakindent_win, tabstop_padding};
use crate::insexpand::{ins_compl_col_range_attr, ins_compl_lnum_in_range, ins_compl_win_active};
use crate::main::{
    State, VIsual, VIsual_active, VIsual_mode, cmdwin_type, cmdwin_win, cterm_normal_bg_color,
    curwin, decor_state, did_emsg, dollar_vcol, dy_flags, highlight_attr, highlight_match,
    hl_attr_active, linebuf_attr, linebuf_char, linebuf_vcol, normal_bg, p_cpo, p_sel,
    screen_search_hl, search_match_endcol, search_match_lines, spell_redraw_lnum, win_extmark_arr,
};
use crate::r#match::{
    get_prevcol_hl_flag, get_search_match_hl, prepare_search_hl_line, update_search_hl,
};
use crate::mbyte::{
    mb_charlen, mb_off_next, mb_ptr2char_adv, mb_string2cells, utf_head_off, utf_ptr2CharInfo,
    utf_ptr2StrCharInfo, utf_ptr2cells, utfc_next, utfc_ptr2len, utfc_ptr2schar,
};
use crate::memline::{gchar_pos, ml_get_buf, ml_get_buf_len};
use crate::memory::{xfree, xmalloc};
use crate::r#move::{set_empty_rows, validate_virtcol, win_col_off, win_col_off2};
use crate::option::{get_showbreak_value, kOptFlagInsecure};
use crate::options::{
    kOptCuloptFlagLine, kOptCuloptFlagNumber, kOptCuloptFlagScreenline, kOptDyFlagUhex,
    kOptSpoFlagNoplainbuffer,
};
use crate::os::cshim::snprintf;
use crate::plines::{getvcol, getvvcol, init_charsize_arg, win_charsize};
use crate::pos::{MAXCOL, ltoreq};
use crate::quickfix::qf_current_entry;
use crate::search::FORWARD;
use crate::spell::{check_need_cap, spell_cat_line, spell_check, spell_move_to, spell_to_word_end};
use crate::state::{MODE_INSERT, virtual_active};
use crate::statusline::{SIGN_SHOW_MAX, build_statuscol_str};
use crate::strings::vim_strchr;
use crate::syntax::{
    HL_CONCEAL, get_syntax_attr, get_syntax_info, syn_get_sub_char, syntax_present, syntax_start,
};
use crate::terminal::terminal_get_line_attributes;
use crate::types::{
    CharSize, CharsizeArg, DecorRange, DecorVirtText, GridView, HlMode, NS, RgbValue,
    SignTextAttrs, VirtLines, VirtText, WinExtmark, buf_T, colnr_T, diffline_T, foldinfo_T, hlf_T,
    linenr_T, pos_T, ptrdiff_t, sattr_T, schar_T, size_t, spellvars_T, ssize_t, statuscol_T,
    uint8_t, uint32_t, uint64_t, varnumber_T, virt_line, win_T,
};
use crate::ui::ui_rgb_attached;
use ::libc::{abs, memcpy, memset, strlen};

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
/// A `DecorRange` that draws virtual text of its own.
/// A `DecorRange` that draws nothing and reports its position to the UI.
/// Columns of a `:terminal` line whose attributes `win_line` will look up.
///
/// The scratch array is that many entries; past it a terminal cell just takes
/// the window's own attributes.
pub const TERM_ATTRS_MAX: ::core::ffi::c_int = 1024;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
/// The most negative `int`, which `draw_virt_text` uses as "this virtual
/// text has no column yet".
pub const INT_MIN: ::core::ffi::c_int = ::core::ffi::c_int::MIN;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const CPO_NUMCOL: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const MAX_NUMBERWIDTH: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
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
        debug_assert!(startrow < endrow);

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
            extra_todo: 0,
            n_attr: 0,
            extra_text: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            extra_attr: 0,
            extra_fill: 0,
            extra_last: 0,
            extra_is_virt_text: false,
            escape_buf: [0; 11],
            diff_hlf: HLF_NONE,
            n_virt_lines: 0,
            n_virt_below: 0,
            filler_lines: 0,
            filler_todo: 0,
            sign_attrs: [SignTextAttrs {
                text: [0; 2],
                hl_id: 0,
            }; 9],
            linebreak_armed: false,
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
/// Bytes of the following line the spell checker looks ahead over, which is
/// the longest word it will ever consider.
pub const SPWORDLEN: ::core::ffi::c_int = 150;
/// Hand the line buffer to the grid.
///
/// `endcol` is one past the last column drawn; `clear_end` clears from there
/// to the right edge of the window. `flags` are `grid_put_linebuf`'s and may
/// not already carry `SLF_RIGHTLEFT` — this is where that is decided.
///
/// It is also where the `<<<` marker goes on the first row when
/// `'smoothscroll'` has taken part of the line off the top.
///
/// # Safety
/// `wp` must be a live window and the line buffers filled for it.
unsafe fn wlv_put_linebuf(
    wp: *mut win_T,
    wlv: &WinLineVars,
    mut endcol: ::core::ffi::c_int,
    clear_end: bool,
    bg_attr: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) {
    // SAFETY: the caller's window and line buffers.
    unsafe {
        let grid: *mut GridView = &raw mut (*wp).w_grid;
        let mut startcol = 0;
        let mut clear_width = if clear_end {
            (*wp).w_view_width
        } else {
            endcol
        };

        debug_assert!(flags & SLF_RIGHTLEFT as ::core::ffi::c_int == 0);
        if (*wp).w_onebuf_opt.wo_rl != 0 {
            linebuf_mirror(
                &mut startcol,
                &mut endcol,
                &mut clear_width,
                (*wp).w_view_width,
            );
            flags |= SLF_RIGHTLEFT as ::core::ffi::c_int;
        }

        if wlv.row == 0
            && (*wp).w_skipcol > 0
            // Do not overwrite the 'showbreak' text with "<<<" ...
            && *get_showbreak_value(wp) as ::core::ffi::c_int == NUL
            // ... nor the 'listchars' "precedes" text.
            && !((*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.prec != 0)
        {
            let mut off = 0;
            if (*wp).w_onebuf_opt.wo_nu != 0 && (*wp).w_onebuf_opt.wo_rnu != 0 {
                // Do not overwrite the line number either: "123 text" becomes
                // "123<<<xt".
                while off < (*wp).w_view_width
                    && ascii_isdigit(schar_get_ascii(*linebuf_char.get().add(off as usize))
                        as ::core::ffi::c_int)
                {
                    off += 1;
                }
            }
            for _ in 0..3 {
                if off >= (*wp).w_view_width {
                    break;
                }
                if off + 1 < (*wp).w_view_width
                    && *linebuf_char.get().add(off as usize + 1) == NUL as schar_T
                {
                    // The first half of a double-width character is being
                    // overwritten; blank its second half.
                    *linebuf_char.get().add(off as usize + 1) = schar_from_ascii(b' ');
                }
                *linebuf_char.get().add(off as usize) = schar_from_ascii(b'<');
                *linebuf_attr.get().add(off as usize) =
                    *hl_attr_active.get().add(HLF_AT as usize) as sattr_T;
                off += 1;
            }
        }

        let mut row = wlv.row;
        let mut coloff = 0;
        let g = grid_adjust(grid, &raw mut row, &raw mut coloff);
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
            wlv.vcol - 1,
            flags,
        );
    }
}

/// Tell the decoration providers that a line is about to be drawn, and ask
/// them for the first chunk of it.
///
/// The chunk size is only an approximation of how many bytes will be drawn:
/// it assumes single-cell ASCII and ignores `'linebreak'`, `'breakindent'`
/// and the rest. The character loop asks for more when it walks past what the
/// answer covered.
///
/// # Safety
/// `wp` must be a live window and `lnum` one of its buffer's lines.
unsafe fn decor_providers_setup(
    rows_to_draw: ::core::ffi::c_int,
    draw_from_line_start: bool,
    lnum: linenr_T,
    col: colnr_T,
    wp: *mut win_T,
) -> ::core::ffi::c_int {
    // SAFETY: the caller's window and line; the callbacks re-enter the editor.
    unsafe {
        let rem_vcols = if (*wp).w_onebuf_opt.wo_wrap != 0 {
            let width = (*wp).w_view_width - win_col_off(wp);
            let width2 = width + win_col_off2(wp);
            let first_row_width = if draw_from_line_start { width } else { width2 };
            first_row_width + (rows_to_draw - 1) * width2
        } else {
            (*wp).w_view_width - win_col_off(wp)
        };

        // Called here because the line pointer has to be invalidated anyway.
        decor_providers_invoke_line(wp, lnum - 1);
        validate_virtcol(wp);

        invoke_range_next(wp, lnum, col, rem_vcols + 1)
    }
}

/// Drive the decoration providers over the next span of a line.
///
/// Answers the byte column their answers now reach, or `INT_MAX` once the
/// span runs to the end of the line.
///
/// # Safety
/// `wp` must be a live window and `lnum` one of its buffer's lines.
unsafe fn invoke_range_next(
    wp: *mut win_T,
    lnum: linenr_T,
    begin_col: colnr_T,
    col_off: colnr_T,
) -> ::core::ffi::c_int {
    // SAFETY: the caller's window and line; the callbacks re-enter the editor.
    unsafe {
        let line = ml_get_buf((*wp).w_buffer, lnum);
        let line_len = ml_get_buf_len((*wp).w_buffer, lnum);
        let col_off = col_off.max(1);

        if col_off <= line_len - begin_col {
            let mut end_col = begin_col + col_off;
            // Do not cut a character in half.
            end_col += mb_off_next(line, line.offset(end_col as isize));
            decor_providers_invoke_range(wp, lnum - 1, begin_col, lnum - 1, end_col);
            validate_virtcol(wp);
            end_col
        } else {
            decor_providers_invoke_range(wp, lnum - 1, begin_col, lnum, 0);
            validate_virtcol(wp);
            ::core::ffi::c_int::MAX
        }
    }
}
