#![forbid(unsafe_code)]

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::buffer::{buf_is_quickfix, buf_meta_total};
use crate::charset::{
    CHAR_DISPLAY_LEN, CharDisplay, byte2cells, mirror_ascii, transchar_buf, transchar_hex_into,
    transstr_into, vim_isbreak, vim_isprintc,
};
use crate::cursor::get_cursor_rel_lnum;
use crate::cursor_shape::cursor_is_block_during_visual;
use crate::decoration::{
    DECOR_ID_INVALID, VirtChunks, VirtLineRef, decor_conceal, decor_has_more_decorations,
    decor_init_draw_col, decor_recheck_draw_col, decor_redraw_col, decor_redraw_eol,
    decor_redraw_line, decor_redraw_signs, decor_spell, decor_virt_lines, decor_virt_pos,
    decor_virt_pos_kind, kHlModeUnknown, with_decor_virt,
};
use crate::decoration_provider::{decor_providers_invoke_line, decor_providers_invoke_range};
use crate::diff::{diff_check_with_linestatus, diff_find_change};
use crate::drawscreen::state::{dollar_vcol, win_extmark_arr};
use crate::drawscreen::{compute_foldcolumn, conceal_cursor_line, number_width, win_draw_end};
use crate::eval::vars::set_vim_var_nr;
use crate::fold::{FOLD_TEXT_LEN, FoldText, VIRTTEXT_EMPTY, get_foldtext};
use crate::global_cell::GlobalCell;
use crate::grid::{
    LineAttrs, LineSpan, SLF_RIGHTLEFT, grid_adjust, grid_put_linebuf, linebuf_mirror, schar_cells,
    schar_from_ascii, schar_from_char, schar_get_adv, schar_get_ascii, schar_get_first_codepoint,
    schar_len, with_linebuf,
};
use crate::highlight::default_hl_attr;
use crate::highlight::namespace::hl_attr;
use crate::highlight::state::{cterm_normal_bg_color, normal_bg};
use crate::highlight::{
    hl_blend_attrs, hl_combine_attr, hl_get_underline, syn_attr2entry, win_bg_attr, win_hl_attr,
};
use crate::highlight_group::{
    HLF_0, HLF_8, HLF_ADD, HLF_AT, HLF_CHD, HLF_CLF, HLF_CLN, HLF_CLS, HLF_CONCEAL, HLF_COUNT,
    HLF_CUC, HLF_CUL, HLF_DED, HLF_FC, HLF_FL, HLF_I, HLF_LNA, HLF_LNB, HLF_MC, HLF_N, HLF_NONE,
    HLF_QFL, HLF_SC, HLF_TXA, HLF_TXD, HLF_V, syn_id2attr,
};
use crate::indent::get_breakindent_win;
use crate::insexpand::{ins_compl_col_range_attr, ins_compl_lnum_in_range, ins_compl_win_active};
use crate::r#match::{
    get_prevcol_hl_flag, get_search_match_hl, prepare_search_hl_line, update_search_hl,
};
use crate::mbyte::{
    c_cells_at, c_char_info, c_cluster_cell, c_cluster_len, c_next_cluster, c_string_cells,
    char_count, head_off,
};
use crate::memline::gchar_pos;
use crate::message::state::did_emsg;
use crate::r#move::{set_empty_rows, validate_virtcol, win_col_off2};
use crate::option::kOptFlagInsecure;
use crate::option::vars::dy_flags;
use crate::options::{
    kOptCuloptFlagLine, kOptCuloptFlagNumber, kOptCuloptFlagScreenline, kOptDyFlagUhex,
    kOptSpoFlagNoplainbuffer,
};
use crate::plines::{getvcol, getvvcol, win_charsize};
use crate::pos::{MAXCOL, ltoreq};
use crate::search::FORWARD;
use crate::search::state::{highlight_match, search_match_endcol, search_match_lines};
use crate::spell::spell_redraw_lnum;
use crate::spell::{check_need_cap, spell_cat_line, spell_check, spell_move_to, spell_to_word_end};
use crate::state::mode::State;
use crate::state::{MODE_INSERT, virtual_active};
use crate::statusline::{SIGN_SHOW_MAX, build_statuscol_str};
use crate::syntax::{SynFlags, get_syntax_attr, syn_get_sub_char, syntax_present};
use crate::terminal::terminal_get_line_attributes;
use crate::types::TAB;
use crate::types::{
    CharSize, CharsizeArg, ColNr, DecorRange, DiffLine, FoldInfo, GridView, HlMode, Hlf, LineNr,
    NS, NUL, Pos, RgbValue, ScreenAttr, ScreenChar, SignTextAttrs, SpellVars, StatusCol, VarNumber,
    VirtText, VirtTextPos, WinExtmark, ptrdiff_t, size_t, uint8_t, uint32_t, uint64_t,
};
use crate::ui::ui_rgb_attached;
use crate::winlayer::Win;
use crate::winlayer::graph::{cmdwin_type, cmdwin_win};
use core::mem::ManuallyDrop;

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
use self::attrs::attr_has_line_deco;
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
/// The most negative `int`, which `draw_virt_text` uses as "this virtual
/// text has no column yet".
pub const INT_MIN: ::core::ffi::c_int = ::core::ffi::c_int::MIN;
pub const MAX_NUMBERWIDTH: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
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
pub fn win_line(
    window: Win,
    lnum: LineNr,
    startrow: ::core::ffi::c_int,
    endrow: ::core::ffi::c_int,
    col_rows: ::core::ffi::c_int,
    concealed: bool,
    spv: &mut SpellVars,
    foldinfo: FoldInfo,
) -> ::core::ffi::c_int {
    debug_assert!(startrow < endrow);

    // Plain construction, so it sits outside the promise below.
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
        extra: ExtraText::take(),
        extra_attr: 0,
        extra_fill: 0,
        extra_last: 0,
        extra_is_virt_text: false,
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
        virt_inline: DECOR_ID_INVALID,
        virt_inline_len: 0,
        virt_inline_i: 0,
        virt_inline_hl_mode: kHlModeUnknown,
        reset_extra_attr: false,
        skip_cells: 0,
        skipped_cells: 0,
        color_col: NO_COLOR_COL,
    };

    // The spell look-ahead, filled by the setup half and read by the loop.
    let mut nextline: SpellLookahead = [0; SPELL_LOOKAHEAD * 2];

    let setup = prepare_line(
        &mut wlv,
        window,
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
        terminal_get_line_attributes(window.buffer(), lnum, &mut term_attrs);
    }

    let mut frame = LineFrame {
        endrow,
        col_rows,
        spv,
        statuscol: setup.statuscol,
        term_attrs: &term_attrs,
        nextline: &nextline,
    };
    Cells::new(setup).run(&mut wlv, window, window.buffer(), &mut frame)
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
fn wlv_put_linebuf(
    window: Win,
    wlv: &WinLineVars,
    mut endcol: ::core::ffi::c_int,
    clear_end: bool,
    bg_attr: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) {
    let grid: GridView = window.w_grid;
    let mut startcol = 0;
    let mut clear_width = if clear_end {
        window.w_view_width
    } else {
        endcol
    };

    debug_assert!(flags & SLF_RIGHTLEFT as ::core::ffi::c_int == 0);
    if window.w_onebuf_opt.wo_rl != 0 {
        linebuf_mirror(
            &mut startcol,
            &mut endcol,
            &mut clear_width,
            window.w_view_width,
        );
        flags |= SLF_RIGHTLEFT as ::core::ffi::c_int;
    }

    if wlv.row == 0
        && window.w_skipcol > 0
        // Do not overwrite the 'showbreak' text with "<<<" ...
        && !window.has_showbreak()
        // ... nor the 'listchars' "precedes" text.
        && !(window.w_onebuf_opt.wo_list != 0 && window.w_p_lcs_chars.prec != 0)
    {
        let at_attr = hl_attr(HLF_AT) as ScreenAttr;
        with_linebuf(|line| {
            let mut off = 0;
            if window.w_onebuf_opt.wo_nu != 0 && window.w_onebuf_opt.wo_rnu != 0 {
                // Do not overwrite the line number either: "123 text"
                // becomes "123<<<xt".
                while off < window.w_view_width
                    && ascii_isdigit(
                        schar_get_ascii(line.chars()[off as usize]) as ::core::ffi::c_int
                    )
                {
                    off += 1;
                }
            }
            for _ in 0..3 {
                if off >= window.w_view_width {
                    break;
                }
                if off + 1 < window.w_view_width
                    && line.chars()[off as usize + 1] == NUL as ScreenChar
                {
                    // The first half of a double-width character is being
                    // overwritten; blank its second half.
                    line.chars_mut()[off as usize + 1] = schar_from_ascii(b' ');
                }
                line.chars_mut()[off as usize] = schar_from_ascii(b'<');
                line.attrs_mut()[off as usize] = at_attr;
                off += 1;
            }
        });
    }

    let mut row = wlv.row;
    let mut coloff = 0;
    let g = grid_adjust(grid, &mut row, &mut coloff);
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

/// Tell the decoration providers that a line is about to be drawn, and ask
/// them for the first chunk of it.
///
/// The chunk size is only an approximation of how many bytes will be drawn:
/// it assumes single-cell ASCII and ignores `'linebreak'`, `'breakindent'`
/// and the rest. The character loop asks for more when it walks past what the
/// answer covered.
fn decor_providers_setup(
    rows_to_draw: ::core::ffi::c_int,
    draw_from_line_start: bool,
    lnum: LineNr,
    col: ColNr,
    window: Win,
) -> ::core::ffi::c_int {
    let rem_vcols = if window.w_onebuf_opt.wo_wrap != 0 {
        let width = window.w_view_width - window.col_off();
        let width2 = width + win_col_off2(window);
        let first_row_width = if draw_from_line_start { width } else { width2 };
        first_row_width + (rows_to_draw - 1) * width2
    } else {
        window.w_view_width - window.col_off()
    };

    // Called here because the line pointer has to be invalidated anyway.
    decor_providers_invoke_line(window, lnum - 1);
    validate_virtcol(window);

    invoke_range_next(window, lnum, col, rem_vcols + 1)
}

/// Drive the decoration providers over the next span of a line.
///
/// Answers the byte column their answers now reach, or `INT_MAX` once the
/// span runs to the end of the line.
fn invoke_range_next(
    window: Win,
    lnum: LineNr,
    begin_col: ColNr,
    col_off: ColNr,
) -> ::core::ffi::c_int {
    let col_off = col_off.max(1);
    // The line only for the arithmetic: the callbacks re-enter the editor.
    let end_col = {
        let mut lines = window.buffer().lines();
        let line = lines.line(lnum);
        let line_len = ColNr::try_from(line.len()).unwrap_or(ColNr::MAX);
        (col_off <= line_len - begin_col).then(|| {
            let end_col = begin_col + col_off;
            // Do not cut a character in half: forward to the start of the
            // next one when `end_col` is inside one.
            let at = usize::try_from(end_col).unwrap_or(0);
            let back = head_off(line, at);
            let fwd = if back == 0 {
                0
            } else {
                c_cluster_len(&line[at - back..]) - back
            };
            end_col + ColNr::try_from(fwd).unwrap_or(0)
        })
    };

    if let Some(end_col) = end_col {
        decor_providers_invoke_range(window, lnum - 1, begin_col, lnum - 1, end_col);
        validate_virtcol(window);
        end_col
    } else {
        decor_providers_invoke_range(window, lnum - 1, begin_col, lnum, 0);
        validate_virtcol(window);
        ::core::ffi::c_int::MAX
    }
}
