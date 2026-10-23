//! Drawing a user format -- `'statusline'`, `'tabline'`, `'winbar'` and
//! `'rulerformat'`.
//!
//! [`win_redr_custom`] is the one renderer all four share, and it runs in
//! three stages: [`Target::of`] decides *where* to draw (which grid, which
//! row, how wide, with which fill character and highlight) and picks the
//! format up out of the right option; the expansion is then one [`StlJob`];
//! and [`paint_chunks`] walks the highlight runs the expander recorded,
//! painting each stretch of text with the attribute in force there -- or,
//! for `ext_messages`, packing the same stretches into the `msg_ruler`
//! event instead of drawing them.
//!
//! [`win_redr_winbar`] is the winbar entry point. [`redraw_ruler`] is the
//! `'ruler'` one: it hands over to [`win_redr_custom`] when `'rulerformat'`
//! is set, and otherwise builds the plain `line,col   50%` text itself and
//! draws it on the last line -- which is why it is here rather than in
//! [`super::status`].
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use crate::memline::MlFlags;
use crate::optionstr::{OptString, local_or_global};
use core::ffi::c_int;

use super::*;
use crate::ascii::ascii_isdigit;
use crate::autocmd::is_aucmd_win;
use crate::buffer::{col_text, get_rel_pos};
use crate::charset::{strsize, transstr_into};
use crate::drawscreen::state::ru_col;
use crate::global_cell::GlobalCell;
use crate::grid::{SCHAR_BUF_SIZE, default_grid_ref, schar_copy, schar_from_ascii};
use crate::highlight::state::{highlight_stlnc, highlight_user};
use crate::highlight_group::{
    HLF_MSG, HLF_TPF, HLF_WBR, HLF_WBRNC, syn_id2attr, syn_name2id_bytes,
};
use crate::mbyte::c_cluster_len;
use crate::message::state::{msg_col, msg_row};
use crate::message::{msg_clr_eos, msg_grid_view};
use crate::option::vars::{P_RUF, P_STL, P_WBR, p_ch, p_ru, p_ruf, p_tal};
use crate::options::{kOptRulerformat, kOptStatusline, kOptTabline, kOptWinbar};
use crate::optionstr::OptStringRef;
use crate::os::cshim::gettext;
use crate::state::MODE_INSERT;
use crate::state::mode::{State, edit_submode};
use crate::strings::{FmtArg, formatted};
use crate::types::ui::kUIMessages;
use crate::types::{
    Array, ColNr, Hlf, Integer, MAXPATHL, NUL, Object, OptIndex, OptInt, OptionSetFlags,
    ScreenChar, StlHlRun, StlOpt, String_0, int64_t,
};
use crate::ui::state::{Columns, Rows};
use crate::ui::{ui_call_msg_ruler, ui_has};
use crate::ui_compositor::win_grid;
use crate::window::lastwin_nofloating;

static DID_SHOW_EXT_RULER: GlobalCell<bool> = GlobalCell::new(false);

/// The screen grid itself, which is what everything but a floating window's
/// status line and the ruler-in-the-message-area is drawn on.
fn screen_canvas() -> GridRef {
    default_grid_ref()
}

/// Where one of the four formats is drawn, and what it is drawn with.
struct Target {
    /// The grid and the row of it the line goes on.
    canvas: GridRef,
    row: c_int,
    /// The column the text starts at, and how many cells it may take.
    col: c_int,
    maxwidth: c_int,
    /// The pad character and the highlight the untouched cells carry.
    fillchar: ScreenChar,
    group: Hlf,
    attr: c_int,
}

/// The format to expand, a copy of the option's value, and which option it
/// came from.
struct Source {
    fmt: Vec<u8>,
    opt: (OptIndex, OptionSetFlags),
}

impl Target {
    /// Set up for the task at hand: no `win` draws `'tabline'`, which belongs
    /// to no window; otherwise `draw_winbar` draws `'winbar'`, `draw_ruler`
    /// draws `'rulerformat'` and neither draws `'statusline'`.
    ///
    /// Answers `None` when there is nothing to draw -- a winbar scrolled off
    /// the top, or a width of nothing.
    fn of(win: Option<Win>, draw_winbar: bool, draw_ruler: bool) -> Option<(Target, Source)> {
        let is_stl_global = stl_is_global();
        let floating = win.is_some_and(|w| w.w_floating) && !is_stl_global;
        // A floating window owns its grid allocation.
        let own = || win_grid(win.expect("a float is a window"));
        let mut canvas = if floating { own() } else { screen_canvas() };
        let mut col = 0;

        let Some(win) = win else {
            // 'tabline', always on the first row of the screen.
            let target = Target {
                canvas,
                row: 0,
                col,
                maxwidth: Columns.get(),
                fillchar: schar_from_ascii(b' '),
                group: HLF_TPF,
                attr: hl_attr(HLF_TPF),
            };
            let source = Source {
                fmt: p_tal(|value| value.to_bytes().to_vec()),
                opt: (kOptTabline, OptionSetFlags::NONE),
            };
            return (target.maxwidth > 0).then_some((target, source));
        };

        if draw_winbar {
            let local = !win.w_onebuf_opt.wo_wbr.bytes().is_empty();
            let mut row = -1; // Row zero is the first row of text.
            canvas = grid_adjust(win.w_grid, &mut row, &mut col);
            if row < 0 {
                return None;
            }
            let group = if win.is_current() { HLF_WBR } else { HLF_WBRNC };
            let maxwidth = win.w_view_width;
            reset_click_defs(win, ClickKind::Winbar, maxwidth);
            let target = Target {
                canvas,
                row,
                col,
                maxwidth,
                fillchar: win.w_p_fcs_chars.wbr,
                group,
                attr: win_hl(win, group as c_int),
            };
            let wbr = local_or_global(&win.w_onebuf_opt.wo_wbr, P_WBR).get();
            let source = Source {
                fmt: wbr.to_vec(),
                opt: (
                    kOptWinbar,
                    if local {
                        OptionSetFlags::LOCAL
                    } else {
                        OptionSetFlags::NONE
                    },
                ),
            };
            return (target.maxwidth > 0).then_some((target, source));
        }

        // 'statusline' or 'rulerformat'.
        let in_status_line = win.w_status_height != 0 || is_stl_global;
        let mut row;
        let mut maxwidth;
        if win.w_floating && !is_stl_global && !draw_ruler {
            row = win.w_winrow_off + win.w_view_height;
            col = win.w_wincol_off;
            maxwidth = win.w_view_width;
        } else {
            row = if is_stl_global {
                Rows.get() - p_ch() as c_int - 1
            } else {
                win.w_winrow + win.w_height
            };
            maxwidth = if in_status_line && !is_stl_global {
                win.w_width
            } else {
                Columns.get()
            };
        }
        let (mut group, mut fillchar) = fillchar_status_of(win);
        reset_click_defs(win, ClickKind::Status, maxwidth);

        let source = if draw_ruler {
            let fmt = p_ruf(|value| ruler_body(value.to_bytes()).to_vec());
            col = (ru_col.get() - (Columns.get() - maxwidth)).max((maxwidth + 1) / 2);
            maxwidth -= col;
            if !in_status_line {
                row = Rows.get() - 1;
                canvas = grid_adjust(msg_grid_view(), &mut row, &mut col);
                maxwidth -= 1; // Writing in the last column may scroll.
                fillchar = schar_from_ascii(b' ');
                group = HLF_MSG;
            }
            Source {
                fmt,
                opt: (kOptRulerformat, OptionSetFlags::NONE),
            }
        } else {
            let local = !win.w_onebuf_opt.wo_stl.bytes().is_empty();
            let stl = local_or_global(&win.w_onebuf_opt.wo_stl, P_STL).get();
            Source {
                fmt: stl.to_vec(),
                opt: (
                    kOptStatusline,
                    if local {
                        OptionSetFlags::LOCAL
                    } else {
                        OptionSetFlags::NONE
                    },
                ),
            }
        };

        let attr = win_hl(win, group as c_int);
        if !win.w_floating && in_status_line && !is_stl_global {
            col += win.w_wincol;
        }
        let target = Target {
            canvas,
            row,
            col,
            maxwidth,
            fillchar,
            group,
            attr,
        };
        (target.maxwidth > 0).then_some((target, source))
    }
}

/// `'rulerformat'` may open with `%-<width>(`, which reserves that many
/// columns; `ru_col` already accounts for it, so the group spec is stepped
/// over here rather than expanded. Anything else after the `%` is a plain
/// item and the whole format is used.
fn ruler_body(ruf: &[u8]) -> &[u8] {
    let Some(&b'%') = ruf.first() else {
        return ruf;
    };
    let mut at = 1 + usize::from(ruf.get(1) == Some(&b'-'));
    // C reads the number with `atoi`, which stops at the first byte that is
    // not a digit -- and answers 0 for a leading zero run, which then is not
    // skipped. (A sign or blanks `atoi` would also take cannot start a
    // width here: the parser would not have accepted the option.)
    let digits = ruf[at..]
        .iter()
        .take_while(|&&b| ascii_isdigit(c_int::from(b)));
    // glibc's `atoi` is `(int) strtol`: saturated at a `long`, then cut.
    let value = digits.clone().fold(0i64, |acc, &digit| {
        acc.saturating_mul(10)
            .saturating_add(i64::from(digit - b'0'))
    });
    if value as c_int != 0 {
        at += digits.count();
    }
    // Upstream reads the byte and steps past it in one `*stl++ != '('`, so a
    // format that is only a group spec ends up at the terminator.
    if ruf.get(at) == Some(&b'(') {
        &ruf[at + 1..]
    } else {
        ruf
    }
}

/// Which of a window's click definitions are meant.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClickKind {
    Status,
    Winbar,
}

/// Free the click definitions recorded last time and make room for `width`
/// cells of new ones.
fn reset_click_defs(mut win: Win, kind: ClickKind, width: c_int) {
    let defs = match kind {
        ClickKind::Status => &mut win.w_click_defs.status,
        ClickKind::Winbar => &mut win.w_click_defs.winbar,
    };
    clear_click_defs(defs);
    reserve_click_defs(defs, width);
}

/// Spread this draw's click records over the definitions they belong in:
/// the tab page line's, or one of the window's two.
fn record_clicks(
    win: Option<Win>,
    draw_winbar: bool,
    recs: Vec<StlClickRecord>,
    text: &[u8],
    width: c_int,
) {
    match win {
        None => with_tab_click_defs(|defs| fill_click_defs(defs, recs, text, width, true)),
        // `Target::of` sized the window's definitions a moment ago.
        Some(mut w) => {
            let defs = if draw_winbar {
                &mut w.w_click_defs.winbar
            } else {
                &mut w.w_click_defs.status
            };
            fill_click_defs(defs, recs, text, width, false);
        }
    }
}

/// Paint the expanded line, one stretch per highlight run.
///
/// `content` packs the same stretches into an `ext_messages` array instead of
/// drawing them, which is how the ruler reaches a UI that has taken the
/// message area over.
///
/// Answers the column painting stopped at and the attribute in force there,
/// both of which the fill after it needs.
fn paint_chunks(
    target: &Target,
    line: &[u8],
    runs: &[StlHlRun],
    win: Option<Win>,
    mut content: Option<&mut Array>,
) -> (c_int, c_int) {
    let mut transbuf = [0u8; MAXPATHL as usize];
    let mut col = target.col;
    let mut curattr = target.attr;
    let mut curgroup = target.group as c_int;
    let end = line.len();
    let mut p = 0;

    // A last turn past the runs closes the last stretch, at the end of the
    // line.
    for run in runs.iter().map(Some).chain(core::iter::once(None)) {
        let stop = run.map_or(end, |r| r.start);
        let textlen = stop as isize - p as isize;
        // Make all characters printable. `p` can be past the end of the
        // line -- the expander records a run at the truncation point -- and
        // there is then nothing left to transform.
        let src = line.get(p..).unwrap_or_default();
        let tsize = transstr_into(src, usize::try_from(textlen).ok(), &mut transbuf, true);
        match content.as_deref_mut() {
            None => col += paint_text(col, &transbuf[..tsize], curattr),
            Some(content) => push_chunk(content, curattr, &transbuf[..tsize], curgroup),
        }

        let Some(run) = run else { break };
        p = run.start;
        (curattr, curgroup) = run_highlight(*run, target, curattr, win);
        if curattr != target.attr {
            curattr = combine_attr(target.attr, curattr);
        }
    }
    (col, curattr)
}

/// The attribute and highlight group one run switches to.
fn run_highlight(
    run: StlHlRun,
    target: &Target,
    curattr: c_int,
    win: Option<Win>,
) -> (c_int, c_int) {
    if run.userhl == 0 {
        return (target.attr, target.group as c_int);
    }
    if run.userhl < 0 {
        // A named group -- `%#Group#`, or the sign and fold columns' own.
        let new_attr = syn_id2attr(-run.userhl);
        let attr = if run.item == Some(StlOpt::HighlightComb) {
            combine_attr(curattr, new_attr)
        } else {
            new_attr
        };
        return (attr, -run.userhl);
    }
    // `%N*`: one of the nine User highlights -- or their "status line of a
    // window that is not the current one" counterparts, which only exist
    // for a window that has a status line of its own.
    let stlnc = win.is_some_and(|w| !w.is_current() && w.w_status_height != 0);
    let idx = (run.userhl - 1) as usize;
    let attr = if stlnc {
        highlight_stlnc.with(|hl| hl[idx])
    } else {
        highlight_user.with(|hl| hl[idx])
    };
    let name = [b'U', b's', b'e', b'r', run.userhl as u8 + b'0'];
    let group = syn_name2id_bytes(&name);
    (attr, group)
}

/// Append one `[attr, text, group]` chunk to an `ext_messages` array.
fn push_chunk(content: &mut Array, attr: c_int, text: &[u8], group: c_int) {
    let mut chunk = Array::with_capacity(3);
    // The text is copied: the event outlives the buffer it was built in.
    chunk.push(Object::integer(attr as Integer));
    chunk.push(Object::string(String_0::from_bytes(text)));
    chunk.push(Object::integer(group as Integer));
    content.push(Object::array(chunk));
}

/// Redraw the status line, window bar, ruler or tab line of `window` -- null for
/// `'tabline'`.
pub(crate) fn win_redr_custom(
    window: Option<Win>,
    draw_winbar: bool,
    draw_ruler: bool,
    ui_event: bool,
) {
    static ENTERED: GlobalCell<bool> = GlobalCell::new(false);
    // There is a tiny chance of getting here recursively: redrawing a status
    // line can trigger redrawing the ruler or the tab line.
    if ENTERED.get() {
        return;
    }
    ENTERED.set(true);
    draw_custom(window, draw_winbar, draw_ruler, ui_event);
    ENTERED.set(false);
}

/// [`win_redr_custom`] without the recursion guard.
fn draw_custom(window: Option<Win>, draw_winbar: bool, draw_ruler: bool, ui_event: bool) {
    let Some((target, source)) = Target::of(window, draw_winbar, draw_ruler) else {
        return;
    };
    let (win, mut ewp) = (window, window.unwrap_or_else(Win::current));

    // Temporarily reset 'cursorbind': a side effect from moving the cursor
    // away and back is not wanted.
    let crb_save = ewp.w_onebuf_opt.wo_crb;
    ewp.w_onebuf_opt.wo_crb = 0;
    let job = StlJob {
        win: ewp,
        fmt: source.fmt,
        opt: source.opt,
        fillchar: target.fillchar,
        maxwidth: target.maxwidth,
        room: MAXPATHL as usize,
        want_hl: true,
        want_clicks: true,
        stc: None,
    };
    // The expander re-enters the editor; nothing is held across it.
    let built = job.run();
    ewp.w_onebuf_opt.wo_crb = crb_save;

    let start_col = target.col;

    if ui_event {
        let mut content = Array::EMPTY;
        paint_chunks(&target, &built.text, &built.hl, win, Some(&mut content));
        ui_call_msg_ruler(content);
        DID_SHOW_EXT_RULER.set(true);
        return;
    }

    screengrid_line_start(target.canvas, target.row, 0);
    let (col, curattr) = paint_chunks(&target, &built.text, &built.hl, win, None);
    paint_fill(col, start_col + target.maxwidth, target.fillchar, curattr);
    paint_flush();

    // Record where each click in the tab page line, status line or window
    // bar lands.
    record_clicks(win, draw_winbar, built.clicks, &built.text, target.maxwidth);
}

/// Redraw `window`'s window bar from `'winbar'`.
pub fn win_redr_winbar(window: Win) {
    static ENTERED: GlobalCell<bool> = GlobalCell::new(false);
    // Reached recursively when the winbar contains an expression that
    // triggers a redraw.
    if ENTERED.get() {
        return;
    }
    ENTERED.set(true);
    let win = window;
    if win.w_winbar_height != 0
        && is_redrawing()
        && (P_WBR.first_byte() != 0 || !win.w_onebuf_opt.wo_wbr.bytes().is_empty())
    {
        win_redr_custom(Some(window), true, false, false);
    }
    ENTERED.set(false);
}

// ---------------------------------------------------------------------------
// The ruler

/// Where the ruler was last drawn, so that it can be cleared again. -1 means
/// "nowhere".
static DID_RULER_COL: GlobalCell<c_int> = GlobalCell::new(-1);

/// Redraw the ruler: `'rulerformat'` if it is set, else `line,col` and the
/// relative position, right-aligned at `ru_col`.
pub fn redraw_ruler() {
    // The ruler belongs to the window it describes, unless that window has a
    // status line of its own to put it on -- then it is the last window's.
    let cur = Win::current();
    let use_cur = !is_aucmd_win(cur) && cur.w_status_height == 0;
    let mut win = if use_cur {
        cur
    } else {
        lastwin_nofloating(None)
    };
    let is_stl_global = stl_is_global();

    // Should the ruler be drawn at all? If not, clear what was drawn before.
    if !p_ru()
        || win.w_status_height > 0
        || is_stl_global
        || (p_ch() == 0 as OptInt && !ui_has(kUIMessages))
    {
        if DID_SHOW_EXT_RULER.get() && ui_has(kUIMessages) {
            ui_call_msg_ruler(ARRAY_DICT_INIT);
            DID_SHOW_EXT_RULER.set(false);
        } else if DID_RULER_COL.get() > 0 {
            msg_col.set(DID_RULER_COL.get());
            msg_row.set(Rows.get() - 1);
            msg_clr_eos();
        }
        DID_RULER_COL.set(-1);
        return;
    }

    // `redraw_ruler()` can be called after deleting lines but before the
    // cursor has been corrected, so the line number may be out of range.
    if win.w_cursor.lnum > win.buffer().line_count() {
        return;
    }
    // Not while insert-completion is running: it might overwrite the (long)
    // mode message.
    if win.w_status_height == 0 && !is_stl_global && edit_submode.get().is_some() {
        return;
    }

    let part_of_status = win.w_status_height != 0 || is_stl_global;
    if P_RUF.first_byte() != 0 && (p_ch() > 0 as OptInt || (ui_has(kUIMessages) && !part_of_status))
    {
        win_redr_custom(Some(win), false, true, ui_has(kUIMessages));
        return;
    }

    let mut group = HLF_MSG;
    let off = if win.w_status_height != 0 {
        win.w_wincol
    } else {
        0
    };
    let width = if win.w_status_height != 0 {
        win.w_width
    } else {
        Columns.get()
    };
    let fillchar = if part_of_status {
        let (g, fillchar) = fillchar_status_of(win);
        group = g;
        fillchar
    } else {
        schar_from_ascii(b' ')
    };
    let attr = if part_of_status {
        win_hl(win, group as c_int)
    } else {
        hl_attr(group as c_int)
    };

    // In list mode the virtual column has to be recomputed, because the
    // cursor is drawn where the tab is shown rather than where it ends.
    let mut virtcol = win.w_virtcol;
    if win.w_onebuf_opt.wo_list != 0 && win.w_p_lcs_chars.tab1 == NUL as ScreenChar {
        win.w_onebuf_opt.wo_list = 0;
        virtcol = win.virtual_cursor_vcol(win.cursor());
        win.w_onebuf_opt.wo_list = 1;
    }

    // The text as upstream's `RULER_BUF_LEN` buffer would hold it.
    let room = RULER_BUF_LEN as usize;
    let mut buffer = ruler_position(win, virtcol);

    // Add a "50%" if there is room for it. On the last line, don't print in
    // the last column: that scrolls the screen up on some terminals.
    let rel_pos = get_rel_pos(win, room);
    let mut n1 = buffer.len() as c_int + strsize(&rel_pos);
    if win.w_status_height == 0 && !is_stl_global {
        n1 += 1; // Can't use the last character of the screen.
    }

    // Never use more than half the window/screen width, so that there is a
    // half left for the file name.
    let this_ru_col = (ru_col.get() - (Columns.get() - width)).max((width + 1) / 2);
    if this_ru_col + n1 < width {
        // Pad up to `this_ru_col`, but leave room for rel_pos and its NUL.
        let mut glyph = [0u8; SCHAR_BUF_SIZE];
        let glyph_len = schar_copy(&mut glyph, fillchar);
        while this_ru_col + n1 < width && room > buffer.len() + rel_pos.len() + 1 {
            buffer.extend_from_slice(&glyph[..glyph_len]);
            n1 += 1;
        }
        let keep = rel_pos.len().min(room.saturating_sub(buffer.len() + 1));
        buffer.extend_from_slice(&rel_pos[..keep]);
    }

    if ui_has(kUIMessages) && !part_of_status {
        show_ext_ruler(&buffer, attr);
        return;
    }
    if DID_SHOW_EXT_RULER.get() {
        ui_call_msg_ruler(ARRAY_DICT_INIT);
        DID_SHOW_EXT_RULER.set(false);
    }
    truncate_at_width(&mut buffer, this_ru_col, width);

    view_line_start(msg_grid_view(), Rows.get() - 1);
    DID_RULER_COL.set(off + this_ru_col);
    let w = paint_text(DID_RULER_COL.get(), &buffer, attr);
    paint_fill(DID_RULER_COL.get() + w, off + width, fillchar, attr);
    paint_flush();
}

/// `line,col`, as the start of a `RULER_BUF_LEN` buffer holds it.
///
/// The line number is 0 for an empty buffer, and the column is 0 outside
/// Insert mode on an empty line -- which is what makes that read "0-1".
fn ruler_position(win: Win, virtcol: ColNr) -> Vec<u8> {
    let empty_buffer = win.buffer().b_ml.ml_flags.has(MlFlags::EMPTY);
    let cursor_line_empty = win.buffer().lines().line(win.w_cursor.lnum).is_empty();
    let empty_line = State.get() & MODE_INSERT == 0 && cursor_line_empty;
    let lnum = if empty_buffer {
        0 as int64_t
    } else {
        win.w_cursor.lnum as int64_t
    };
    let room = RULER_BUF_LEN as usize;
    // l10n: leave as-is unless a space after the comma is preferred
    // l10n: do not add any row/column label, due to the limited space
    let mut text = formatted(room, gettext(c"%ld,").to_bytes(), &[FmtArg::from(lnum)]);
    let col = if empty_line { 0 } else { win.w_cursor.col + 1 };
    let col = col_text(col, virtcol + 1);
    let keep = col.len().min(room.saturating_sub(text.len() + 1));
    text.extend_from_slice(&col.as_bytes()[..keep]);
    text
}

/// Cut `buffer` short at the first character that would not fit.
fn truncate_at_width(buffer: &mut Vec<u8>, this_ru_col: c_int, width: c_int) {
    let mut at = 0;
    let mut cells = 0;
    while at < buffer.len() {
        cells += crate::mbyte::cells_at(&buffer[at..]);
        if this_ru_col + cells > width {
            buffer.truncate(at);
            return;
        }
        at += c_cluster_len(&buffer[at..]);
    }
}

/// Send the ruler to a UI that has taken the message area over.
fn show_ext_ruler(buffer: &[u8], attr: c_int) {
    let mut content = Array::with_capacity(1);
    let mut chunk = Array::with_capacity(3);
    debug_assert!(
        attr == hl_attr(HLF_MSG as c_int),
        "attr == HL_ATTR(HLF_MSG)"
    );
    chunk.push(Object::integer(attr as Integer));
    chunk.push(Object::string(String_0::from_bytes(buffer)));
    chunk.push(Object::integer(HLF_MSG as Integer));
    content.push(Object::array(chunk));
    ui_call_msg_ruler(content);
    DID_SHOW_EXT_RULER.set(true);
    DID_RULER_COL.set(1);
}
