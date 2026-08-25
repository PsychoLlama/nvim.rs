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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::memline::MlFlags;
use core::ffi::{c_char, c_int};

use super::*;
use crate::api::private::helpers::{api_free_array, cstr_as_string};
use crate::ascii::ascii_isdigit;
use crate::autocmd::is_aucmd_win;
use crate::buffer::{col_print, get_rel_pos};
use crate::charset::{transstr_buf, vim_strsize};
use crate::cstr;
use crate::global_cell::GlobalCell;
use crate::grid::{GridRef, schar_from_ascii, schar_get};
use crate::highlight_group::{HLF_MSG, HLF_TPF, HLF_WBR, HLF_WBRNC, syn_id2attr, syn_name2id_len};
use crate::kvec::Kvec;
use crate::main::{
    Columns, Rows, State, default_grid, edit_submode, highlight_stlnc, highlight_user, msg_col,
    msg_row, p_ch, p_ru, p_ruf, p_stl, p_tal, p_wbr, ru_col, tab_page_click_defs,
    tab_page_click_defs_size,
};
use crate::mbyte::{utf_ptr2cells, utfc_ptr2len};
use crate::memline::ml_get_buf;
use crate::memory::xmemdupz;
use crate::message::{msg_clr_eos, msg_grid_view};
use crate::options::{kOptRulerformat, kOptStatusline, kOptTabline, kOptWinbar};
use crate::os::cshim::gettext;
use crate::state::MODE_INSERT;
use crate::strings::vim_snprintf;
use crate::types::ui::kUIMessages;
use crate::types::{
    Array, Integer, MAXPATHL, NUL, Object, OptIndex, OptInt, OptionSetFlags, String_0, colnr_T,
    hlf_T, int64_t, schar_T, ssize_t, tabpage_T, win_T,
};
use crate::ui::{ui_call_msg_ruler, ui_has};
use crate::window::lastwin_nofloating;
use ::libc::{atoi, strlen};

static DID_SHOW_EXT_RULER: GlobalCell<bool> = GlobalCell::new(false);

/// The screen grid itself, which is what everything but a floating window's
/// status line and the ruler-in-the-message-area is drawn on.
fn screen_canvas() -> Canvas {
    // SAFETY: `default_grid` is live for the process's lifetime.
    unsafe { Canvas::new(GridRef::new(default_grid.ptr())) }
}

/// Where one of the four formats is drawn, and what it is drawn with.
struct Target {
    /// The grid and the row of it the line goes on.
    canvas: Canvas,
    row: c_int,
    /// The column the text starts at, and how many cells it may take.
    col: c_int,
    maxwidth: c_int,
    /// The pad character and the highlight the untouched cells carry.
    fillchar: schar_T,
    group: hlf_T,
    attr: c_int,
}

/// The format to expand, and which option it came from.
struct Source {
    fmt: Fmt,
    opt: (OptIndex, OptionSetFlags),
}

impl Target {
    /// Set up for the task at hand: `wp` null draws `'tabline'`, otherwise
    /// `draw_winbar` draws `'winbar'`, `draw_ruler` draws `'rulerformat'`
    /// and neither draws `'statusline'`.
    ///
    /// Answers `None` when there is nothing to draw -- a winbar scrolled off
    /// the top, or a width of nothing.
    ///
    /// # Safety
    /// `wp` must be null or a live window.
    unsafe fn of(wp: *mut win_T, draw_winbar: bool, draw_ruler: bool) -> Option<(Target, Source)> {
        // SAFETY: the caller's promise.
        let win = unsafe { win_opt(wp) };
        let is_stl_global = stl_is_global();
        let floating = win.is_some_and(|w| w.w_floating) && !is_stl_global;
        // SAFETY: a floating window owns its grid allocation.
        let own = || unsafe { Canvas::new(GridRef::new(&raw mut (*wp).w_grid_alloc)) };
        let mut canvas = if floating { own() } else { screen_canvas() };
        let mut col = 0;

        let Some(mut win) = win else {
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
                // SAFETY: the option's own string.
                fmt: unsafe { Fmt::copy_of(p_tal.get()) },
                opt: (kOptTabline, OptionSetFlags::NONE),
            };
            return (target.maxwidth > 0).then_some((target, source));
        };

        if draw_winbar {
            let local = !opt_is_empty(win.w_onebuf_opt.wo_wbr);
            let mut row = -1; // Row zero is the first row of text.
            // SAFETY: a live window whose grid view is live.
            canvas = unsafe { Canvas::adjust((*wp).w_grid, &mut row, &mut col) };
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
            let wbr = if local {
                win.w_onebuf_opt.wo_wbr
            } else {
                p_wbr.get()
            };
            let source = Source {
                // SAFETY: the option's own string.
                fmt: unsafe { Fmt::copy_of(wbr) },
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
                Rows.get() - p_ch.get() as c_int - 1
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
            // SAFETY: the option's own string.
            let fmt = unsafe { Fmt::copy_of(ruler_body(p_ruf.get())) };
            col = (ru_col.get() - (Columns.get() - maxwidth)).max((maxwidth + 1) / 2);
            maxwidth -= col;
            if !in_status_line {
                row = Rows.get() - 1;
                // SAFETY: the message grid's view is live.
                canvas = unsafe { Canvas::adjust(msg_grid_view(), &mut row, &mut col) };
                maxwidth -= 1; // Writing in the last column may scroll.
                fillchar = schar_from_ascii(b' ');
                group = HLF_MSG;
            }
            Source {
                fmt,
                opt: (kOptRulerformat, OptionSetFlags::NONE),
            }
        } else {
            let local = !opt_is_empty(win.w_onebuf_opt.wo_stl);
            let stl = if local {
                win.w_onebuf_opt.wo_stl
            } else {
                p_stl.get()
            };
            Source {
                // SAFETY: the option's own string.
                fmt: unsafe { Fmt::copy_of(stl) },
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
///
/// # Safety
/// `ruf` must be a NUL-terminated string.
unsafe fn ruler_body(ruf: *mut c_char) -> *mut c_char {
    // SAFETY: the caller's promise.
    let text = unsafe { CStr::from_ptr(ruf) }.to_bytes();
    let Some(&b'%') = text.first() else {
        return ruf;
    };
    let mut at = 1 + usize::from(text.get(1) == Some(&b'-'));
    // SAFETY: `at` is inside the string, so the number starts there. C reads
    // it with `atoi`, which stops at the first byte that is not a digit --
    // and answers 0 for a leading zero run, which then is not skipped.
    if unsafe { atoi(ruf.add(at)) } != 0 {
        while text.get(at).is_some_and(|&b| ascii_isdigit(c_int::from(b))) {
            at += 1;
        }
    }
    // Upstream reads the byte and steps past it in one `*stl++ != '('`, so a
    // format that is only a group spec ends up pointing at the terminator.
    // SAFETY: `at` is at most the terminator's index, so `at + 1` is at most
    // one past it, which is still a valid pointer.
    if text.get(at) == Some(&b'(') {
        unsafe { ruf.add(at + 1) }
    } else {
        ruf
    }
}

/// Which of a window's click-definition arenas is meant.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClickKind {
    Status,
    Winbar,
}

/// Free the click definitions recorded last time and make room for `width`
/// cells of new ones.
fn reset_click_defs(mut win: Win, kind: ClickKind, width: c_int) {
    let (defs, size) = match kind {
        ClickKind::Status => (win.w_status_click_defs, win.w_status_click_defs_size),
        ClickKind::Winbar => (win.w_winbar_click_defs, win.w_winbar_click_defs_size),
    };
    // SAFETY: the window's own arena and its recorded size.
    let mut arena = unsafe { ClickArena::new(defs, size) };
    arena.clear();
    arena.reserve(width);
    let (defs, size) = arena.parts();
    match kind {
        ClickKind::Status => {
            win.w_status_click_defs = defs;
            win.w_status_click_defs_size = size;
        }
        ClickKind::Winbar => {
            win.w_winbar_click_defs = defs;
            win.w_winbar_click_defs_size = size;
        }
    }
}

/// The arena this draw's click records belong in: the tab page line's, or
/// one of the window's two.
fn click_arena(win: Option<Win>, draw_winbar: bool) -> ClickArena {
    let (defs, size) = match win {
        None => (tab_page_click_defs.get(), tab_page_click_defs_size.get()),
        Some(w) if draw_winbar => (w.w_winbar_click_defs, w.w_winbar_click_defs_size),
        Some(w) => (w.w_status_click_defs, w.w_status_click_defs_size),
    };
    // SAFETY: `Target::of` sized the window's arenas a moment ago, and the
    // tab line's is sized by the screen resize.
    unsafe { ClickArena::new(defs, size) }
}

/// The expanded line, as the painter walks it: a NUL-terminated buffer and
/// how far into it the text runs.
#[derive(Clone, Copy)]
struct Expanded {
    start: *mut c_char,
    len: isize,
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
    line: Expanded,
    runs: HlRuns,
    win: Option<Win>,
    mut content: Option<&mut Array>,
) -> (c_int, c_int) {
    let mut transbuf = [0 as c_char; MAXPATHL as usize];
    let mut col = target.col;
    let mut curattr = target.attr;
    let mut curgroup = target.group as c_int;
    // SAFETY: one past the last byte of the expanded line, which is where
    // its terminator sits.
    let end = unsafe { line.start.offset(line.len) };
    let mut p = line.start;

    // The terminating run -- the one whose `start` is null -- closes the
    // last stretch, so it takes one more turn than there are runs.
    for run in runs.iter().map(Some).chain(core::iter::once(None)) {
        let stop = run.map_or(end, |r| r.start);
        // SAFETY: `p` and `stop` are positions in the expanded line.
        let textlen = unsafe { stop.offset_from(p) } as c_int;
        // Make all characters printable. `p` can be past the end of the
        // line -- the expander records a run at the truncation point -- and
        // there is then nothing left to transform.
        let src = if p >= end { c"".as_ptr() } else { p };
        let (out, room) = (transbuf.as_mut_ptr(), transbuf.len());
        // SAFETY: `src` holds `textlen` readable bytes, and `transbuf` is
        // told its own length.
        let tsize = unsafe { transstr_buf(src, textlen as ssize_t, out, room, true) };
        match content.as_deref_mut() {
            None => col += paint_text(col, &transbuf[..tsize], curattr),
            Some(content) => push_chunk(content, curattr, &transbuf[..tsize], curgroup),
        }

        let Some(run) = run else { break };
        p = run.start;
        (curattr, curgroup) = run_highlight(run, target, curattr, win);
        if curattr != target.attr {
            curattr = combine_attr(target.attr, curattr);
        }
    }
    (col, curattr)
}

/// The attribute and highlight group one run switches to.
fn run_highlight(
    run: stl_hlrec_t,
    target: &Target,
    curattr: c_int,
    win: Option<Win>,
) -> (c_int, c_int) {
    if run.userhl == 0 {
        return (target.attr, target.group as c_int);
    }
    if run.userhl < 0 {
        // A named group -- `%#Group#`, or the sign and fold columns' own.
        // SAFETY: a group id the expander resolved.
        let new_attr = unsafe { syn_id2attr(-run.userhl) };
        let attr = if run.item == STL_HIGHLIGHT_COMB {
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
    let mut name = *b"User\0";
    name[4] = run.userhl as u8 + b'0';
    // SAFETY: five bytes of a local array.
    let group = unsafe { syn_name2id_len(name.as_mut_ptr().cast(), 5) };
    (attr, group)
}

/// Append one `[attr, text, group]` chunk to an `ext_messages` array.
fn push_chunk(content: &mut Array, attr: c_int, text: &[c_char], group: c_int) {
    let mut chunk = ARRAY_DICT_INIT;
    // SAFETY: both are kvecs this frame owns, so growing them is ours to
    // do; `text` holds its own length in readable bytes, which `xmemdupz`
    // copies into a string the event owns.
    // SAFETY: `text` holds its own length in readable bytes, and the copy
    // becomes the event's to free.
    let copy = unsafe { xmemdupz(text.as_ptr().cast(), text.len()) };
    let owned = String_0::from_raw_parts(copy.cast(), text.len());
    let parts = [
        Object::integer(attr as Integer),
        Object::string(owned),
        Object::integer(group as Integer),
    ];
    {
        let mut c = Kvec::new(&mut chunk.size, &mut chunk.capacity, &mut chunk.items);
        // SAFETY: a kvec this frame owns, so growing it is ours to do.
        parts.into_iter().for_each(|part| unsafe { c.push(part) });
    }
    let mut a = Kvec::new(&mut content.size, &mut content.capacity, &mut content.items);
    // SAFETY: as above.
    unsafe { a.push(Object::array(chunk)) };
}

/// Redraw the status line, window bar, ruler or tab line of `wp` -- null for
/// `'tabline'`.
///
/// # Safety
/// `wp` must be null or a live window. Expanding the format re-enters the
/// editor, so nothing may be held across this.
pub(crate) unsafe fn win_redr_custom(
    wp: *mut win_T,
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
    // SAFETY: the caller's promise.
    unsafe { draw_custom(wp, draw_winbar, draw_ruler, ui_event) };
    ENTERED.set(false);
}

/// [`win_redr_custom`] without the recursion guard.
///
/// # Safety
/// As [`win_redr_custom`].
unsafe fn draw_custom(wp: *mut win_T, draw_winbar: bool, draw_ruler: bool, ui_event: bool) {
    // SAFETY: the caller's promise.
    let Some((target, source)) = (unsafe { Target::of(wp, draw_winbar, draw_ruler) }) else {
        return;
    };
    // SAFETY: the caller's promise; `curwin` is live from startup to exit.
    let (win, mut ewp) = unsafe { (win_opt(wp), win_opt(wp).unwrap_or(Win::current())) };
    let _ = &ewp;

    // Temporarily reset 'cursorbind': a side effect from moving the cursor
    // away and back is not wanted.
    let crb_save = ewp.w_onebuf_opt.wo_crb;
    ewp.w_onebuf_opt.wo_crb = 0;
    let mut buf = [0 as c_char; MAXPATHL as usize];
    let job = StlJob {
        win: ewp,
        fmt: source.fmt,
        opt: source.opt,
        fillchar: target.fillchar,
        maxwidth: target.maxwidth,
        hl: HlDest::Runs,
        want_clicks: true,
        stcp: None,
    };
    // SAFETY: `buf` is this frame's own and is not `NameBuff`. The expander
    // re-enters the editor; nothing is held across it.
    let built = unsafe { job.run(&mut buf) };
    ewp.w_onebuf_opt.wo_crb = crb_save;

    let line = Expanded {
        start: buf.as_mut_ptr(),
        // SAFETY: the expander NUL-terminates its output.
        len: unsafe { strlen(buf.as_ptr()) } as isize,
    };
    let runs = built.hl.expect("hltab was asked for");
    let start_col = target.col;

    if ui_event {
        let mut content = ARRAY_DICT_INIT;
        paint_chunks(&target, line, runs, win, Some(&mut content));
        ui_call_msg_ruler(content);
        DID_SHOW_EXT_RULER.set(true);
        // SAFETY: the array and every string in it were built above.
        unsafe { api_free_array(content) };
        return;
    }

    // SAFETY: the target's grid is live and the batch is flushed below.
    unsafe { target.canvas.line_start(target.row, 0) };
    let (col, curattr) = paint_chunks(&target, line, runs, win, None);
    paint_fill(col, start_col + target.maxwidth, target.fillchar, curattr);
    paint_flush();

    // Record where each click in the tab page line, status line or window
    // bar lands.
    if let Some(clicks) = built.clicks {
        click_arena(win, draw_winbar).fill(clicks, line.start, target.maxwidth, win.is_none());
    }
}

/// Redraw `wp`'s window bar from `'winbar'`.
///
/// # Safety
/// `wp` must be a live window. This evaluates the option, so it re-enters
/// the editor.
pub unsafe fn win_redr_winbar(wp: *mut win_T) {
    static ENTERED: GlobalCell<bool> = GlobalCell::new(false);
    // Reached recursively when the winbar contains an expression that
    // triggers a redraw.
    if ENTERED.get() {
        return;
    }
    ENTERED.set(true);
    // SAFETY: the caller's promise.
    let win = unsafe { Win::new(wp) };
    if win.w_winbar_height != 0
        && is_redrawing()
        && (!opt_is_empty(p_wbr.get()) || !opt_is_empty(win.w_onebuf_opt.wo_wbr))
    {
        // SAFETY: a live window; this evaluates the option.
        unsafe { win_redr_custom(wp, true, false, false) };
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
///
/// # Safety
/// The editor must be up. This evaluates `'rulerformat'`, so it re-enters.
pub unsafe fn redraw_ruler() {
    // The ruler belongs to the window it describes, unless that window has a
    // status line of its own to put it on -- then it is the last window's.
    // SAFETY: `curwin` is live from startup to exit.
    let cur = unsafe { Win::current() };
    // SAFETY: a live window.
    let use_cur = unsafe { !is_aucmd_win(cur.raw()) } && cur.w_status_height == 0;
    // SAFETY: `lastwin_nofloating` answers a live window of this tab page.
    let mut win = if use_cur {
        cur
    } else {
        unsafe { Win::new(lastwin_nofloating(ptr::null_mut::<tabpage_T>())) }
    };
    let is_stl_global = stl_is_global();

    // Should the ruler be drawn at all? If not, clear what was drawn before.
    if p_ru.get() == 0
        || win.w_status_height > 0
        || is_stl_global
        || (p_ch.get() == 0 as OptInt && !ui_has(kUIMessages))
    {
        if DID_SHOW_EXT_RULER.get() && ui_has(kUIMessages) {
            ui_call_msg_ruler(ARRAY_DICT_INIT);
            DID_SHOW_EXT_RULER.set(false);
        } else if DID_RULER_COL.get() > 0 {
            msg_col.set(DID_RULER_COL.get());
            msg_row.set(Rows.get() - 1);
            // SAFETY: clears the message area of the screen.
            unsafe { msg_clr_eos() };
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
    if win.w_status_height == 0 && !is_stl_global && !edit_submode.get().is_null() {
        return;
    }

    let part_of_status = win.w_status_height != 0 || is_stl_global;
    if !opt_is_empty(p_ruf.get())
        && (p_ch.get() > 0 as OptInt || (ui_has(kUIMessages) && !part_of_status))
    {
        // SAFETY: a live window; this evaluates the option.
        unsafe { win_redr_custom(win.raw(), false, true, ui_has(kUIMessages)) };
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
    if win.w_onebuf_opt.wo_list != 0 && win.w_p_lcs_chars.tab1 == NUL as schar_T {
        win.w_onebuf_opt.wo_list = 0;
        virtcol = win.virtual_cursor_vcol(win.cursor());
        win.w_onebuf_opt.wo_list = 1;
    }

    let mut buffer = [0 as c_char; RULER_BUF_LEN as usize];
    let mut bufferlen = ruler_position(win, virtcol, &mut buffer);

    // Add a "50%" if there is room for it. On the last line, don't print in
    // the last column: that scrolls the screen up on some terminals.
    let mut rel_pos = [0 as c_char; RULER_BUF_LEN as usize];
    // SAFETY: `rel_pos` is `RULER_BUF_LEN` bytes of this frame, which
    // `get_rel_pos` fills and NUL-terminates.
    let rel_poslen = unsafe { get_rel_pos(win.raw(), rel_pos.as_mut_ptr(), RULER_BUF_LEN) };
    // SAFETY: as above.
    let mut n1 = bufferlen + unsafe { vim_strsize(rel_pos.as_ptr()) };
    if win.w_status_height == 0 && !is_stl_global {
        n1 += 1; // Can't use the last character of the screen.
    }

    // Never use more than half the window/screen width, so that there is a
    // half left for the file name.
    let this_ru_col = (ru_col.get() - (Columns.get() - width)).max((width + 1) / 2);
    if this_ru_col + n1 < width {
        // Pad up to `this_ru_col`, but leave room for rel_pos and its NUL.
        while this_ru_col + n1 < width && RULER_BUF_LEN > bufferlen + rel_poslen + 1 {
            let at = &mut buffer[bufferlen as usize..];
            // SAFETY: the loop's own bound leaves room for the glyph.
            bufferlen += unsafe { schar_get(at.as_mut_ptr(), fillchar) } as c_int;
            n1 += 1;
        }
        let (at, room) = (
            &mut buffer[bufferlen as usize..],
            (RULER_BUF_LEN as size_t).wrapping_sub(bufferlen as size_t),
        );
        // SAFETY: `room` bytes of `buffer` and a NUL-terminated `rel_pos`.
        bufferlen +=
            unsafe { vim_snprintf(at.as_mut_ptr(), room, c"%s".as_ptr(), rel_pos.as_ptr()) };
    }
    let _ = bufferlen;

    if ui_has(kUIMessages) && !part_of_status {
        show_ext_ruler(&buffer, attr);
        return;
    }
    if DID_SHOW_EXT_RULER.get() {
        ui_call_msg_ruler(ARRAY_DICT_INIT);
        DID_SHOW_EXT_RULER.set(false);
    }
    truncate_at_width(&mut buffer, this_ru_col, width);

    // SAFETY: the message grid's view is live; the batch is flushed below.
    unsafe { view_line_start(msg_grid_view(), Rows.get() - 1) };
    DID_RULER_COL.set(off + this_ru_col);
    let w = paint_cstr(DID_RULER_COL.get(), cstr::in_chars(&buffer), attr);
    paint_fill(DID_RULER_COL.get() + w, off + width, fillchar, attr);
    paint_flush();
}

/// Write `line,col` into `buffer`, answering how many bytes that took.
///
/// The line number is 0 for an empty buffer, and the column is 0 outside
/// Insert mode on an empty line -- which is what makes that read "0-1".
fn ruler_position(win: Win, virtcol: colnr_T, buffer: &mut [c_char]) -> c_int {
    let empty_buffer = win.buffer().b_ml.ml_flags.has(MlFlags::EMPTY);
    // SAFETY: a live window's cursor line, which is NUL-terminated.
    let first = unsafe { *ml_get_buf(win.buffer().raw(), win.w_cursor.lnum) };
    let empty_line = State.get() & MODE_INSERT == 0 && c_int::from(first) == NUL;
    let lnum = if empty_buffer {
        0 as int64_t
    } else {
        win.w_cursor.lnum as int64_t
    };
    // l10n: leave as-is unless a space after the comma is preferred
    // l10n: do not add any row/column label, due to the limited space
    // SAFETY: a message catalogue lookup of a literal.
    let fmt = unsafe { gettext(c"%ld,".as_ptr()) };
    let (out, room) = (buffer.as_mut_ptr(), RULER_BUF_LEN as size_t);
    // SAFETY: `buffer` is `RULER_BUF_LEN` bytes of the caller's frame, and
    // the format takes exactly the one integer.
    let mut len = unsafe { vim_snprintf(out, room, fmt, lnum) };
    let (at, room) = (
        &mut buffer[len as usize..],
        (RULER_BUF_LEN as size_t).wrapping_sub(len as size_t),
    );
    let col = if empty_line { 0 } else { win.w_cursor.col + 1 };
    // SAFETY: `room` bytes of `buffer`, which is what is left of it.
    len += unsafe { col_print(at.as_mut_ptr(), room, col, virtcol + 1) };
    len
}

/// Cut `buffer` short at the first character that would not fit.
fn truncate_at_width(buffer: &mut [c_char], this_ru_col: c_int, width: c_int) {
    let mut at = 0;
    let mut cells = 0;
    while buffer[at as usize] != 0 {
        let p = buffer[at as usize..].as_ptr();
        // SAFETY: `at` is a character boundary inside the NUL-terminated
        // buffer, which is where the step below leaves it.
        cells += unsafe { utf_ptr2cells(p) };
        if this_ru_col + cells > width {
            buffer[at as usize] = NUL as c_char;
            return;
        }
        // SAFETY: as above.
        at += unsafe { utfc_ptr2len(p) };
    }
}

/// Send the ruler to a UI that has taken the message area over.
fn show_ext_ruler(buffer: &[c_char], attr: c_int) {
    let mut content_items = [Object::NIL; 1];
    let mut chunk_items = [Object::NIL; 3];
    let mut content = Array {
        size: 0,
        capacity: 1,
        items: content_items.as_mut_ptr(),
    };
    let mut chunk = Array {
        size: 0,
        capacity: 3,
        items: chunk_items.as_mut_ptr(),
    };
    debug_assert!(
        attr == hl_attr(HLF_MSG as c_int),
        "attr == HL_ATTR(HLF_MSG)"
    );
    // SAFETY: `buffer` is NUL-terminated, and the string it becomes is
    // borrowed for the call only.
    let text = unsafe { cstr_as_string(buffer.as_ptr().cast_mut()) };
    push(&mut chunk, Object::integer(attr as Integer));
    push(&mut chunk, Object::string(text));
    push(&mut chunk, Object::integer(HLF_MSG as Integer));
    push(&mut content, Object::array(chunk));
    ui_call_msg_ruler(content);
    DID_SHOW_EXT_RULER.set(true);
    DID_RULER_COL.set(1);
}
