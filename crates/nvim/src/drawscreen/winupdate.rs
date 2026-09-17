//! `win_update`'s first half: deciding which rows of a window need redrawing.
//!
//! Nothing here draws. Between them, [`win_update`] and the helpers below turn
//! "this window is marked `w_redr_type`" into three row ranges -- a top, a mid
//! and a bot area -- plus a buffer-line range that changed, and they take the
//! scrolling shortcuts: when the new `w_topline` is a few lines above or below
//! the old one, the rows that are still correct are *moved* on the grid rather
//! than drawn again. `winlines.rs` then walks the window one line at a time
//! using what this decided.
//!
//! That is why a bug here shows up as STALE CELLS: every shortcut is a promise
//! that some rows do not have to be redrawn.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::decoration::{DecorStateRef, kVPosWinCol};
use crate::grid::default_grid_ref;
use crate::r#move::WinValid;
use crate::normal::{VisualSelection, visual_selection};
use crate::optionstr::LocalOptStr;
use crate::pos::MAXCOL;
use crate::winlayer::Buf;
use crate::winlayer::{self, Win};

/// A row index no window can reach, used as "this area is empty".
///
/// Upstream's `999`; the window height is bounded well below it.
pub(crate) const NO_ROW: c_int = 999;

/// The rows and buffer lines `win_update` decided have to be drawn again.
///
/// Three row areas, all in window rows:
///
/// - `0 .. top_end` -- lines scrolled in at the top,
/// - `mid_start .. mid_end` -- changed text, or a changed Visual selection,
/// - `bot_start ..` -- everything below a scroll or an insertion.
///
/// plus the buffer-line range `mod_top .. mod_bot` that changed, which the line
/// loop widens as it discovers folds and syntax that did not stop where the
/// change did.
pub(crate) struct Regions {
    /// Below the last row of the top area; 0 when there is none.
    pub top_end: c_int,
    /// First row of the mid area; [`NO_ROW`] when there is none.
    pub mid_start: c_int,
    /// Below the last row of the mid area; 0 when there is none.
    pub mid_end: c_int,
    /// First row of the bot area; [`NO_ROW`] when there is none.
    pub bot_start: c_int,
    /// First row a scroll left stale. Only the end-of-buffer fill reads it --
    /// `bot_start` is about *text*, this is about the blank area below it.
    pub bot_scroll_start: c_int,
    /// Whether the window was scrolled down, i.e. the top area was scrolled in.
    pub scrolled_down: bool,
    /// Whether everything from the top of the window down to `mod_top` still
    /// has to be redrawn (a multi-line pattern makes a change reach upwards).
    pub top_to_mod: bool,
    /// First changed buffer line; 0 when nothing changed.
    pub mod_top: LineNr,
    /// First buffer line after the change; 0 when nothing changed.
    pub mod_bot: LineNr,
    /// The redraw type after this half adjusted it -- it is re-read from the
    /// window after `validate_virtcol`, raised when the number column changed
    /// width, and lowered once `UPD_REDRAW_TOP` has been turned into a top area.
    pub redr_type: c_int,
}

impl Regions {
    /// Nothing to redraw yet.
    fn new(redr_type: c_int) -> Self {
        Self {
            top_end: 0,
            mid_start: NO_ROW,
            mid_end: 0,
            bot_start: NO_ROW,
            bot_scroll_start: NO_ROW,
            scrolled_down: false,
            top_to_mod: false,
            mod_top: 0,
            mod_bot: 0,
            redr_type,
        }
    }

    /// Redraw every row of the window.
    fn redraw_all(&mut self, window: Win) {
        // SAFETY: a live window.
        self.mid_start = 0;
        self.mid_end = window.w_view_height;
    }
}

/// Update a single window.
///
/// This may cause the windows below it to be redrawn as well, when clearing the
/// screen or scrolling lines.
///
/// How the window is redrawn depends on `w_redr_type`; each type also implies
/// the one below it:
///
/// - `UPD_NOT_VALID` -- redraw the whole window
/// - `UPD_SOME_VALID` -- redraw the whole window, but scroll where possible
/// - `UPD_REDRAW_TOP` -- redraw the top `w_upd_rows` lines, else like UPD_VALID
/// - `UPD_INVERTED` -- redraw the changed part of the Visual area
/// - `UPD_INVERTED_ALL` -- redraw the whole Visual area
/// - `UPD_VALID` -- scroll for a changed `w_topline`, redraw changed text, and
///   redraw the lines a scroll brought in at either end.
pub(crate) fn win_update(window: Win) {
    // SAFETY: the caller's promise, taken once for the whole body.
    let mut win = window;
    // SAFETY: a live window of the current layout, during a redraw.
    // Return early when the window would overflow a shrunk terminal, which
    // would draw out of bounds and trip an assertion.
    if win.w_grid.target == default_grid_ref().raw() && win.w_wincol >= Columns.get() {
        return;
    }

    let mut rg = Regions::new(win.w_redr_type);
    if rg.redr_type >= UPD_NOT_VALID {
        win.w_redr_status = true;
        win.w_lines_valid = 0;
    }

    // A window with no room for text only needs its separator.
    if win.w_view_height == 0 {
        draw_hsep_win(window);
        win.w_redr_type = 0;
        return;
    }
    if win.w_view_width == 0 {
        draw_vsep_win(window);
        win.w_redr_type = 0;
        return;
    }

    let mut buf = win.buffer();

    // Reset `got_int`, otherwise the regexp engine will not work.
    let save_got_int = got_int.get();
    got_int.set(false);
    // Bound syntax highlighting by 'redrawtime'.
    let mut syntax_tm = profile_setlimit(p_rdt() as int64_t);
    unsafe { syn_set_timeout(&raw mut syntax_tm) };

    win_extmark_arr.with_mut(Vec::clear);

    // The redraw's decoration state, acquired ONCE here and threaded all
    // the way down `drawline/`. A handle rather than a borrow because the
    // provider callbacks below re-enter: an `on_line` that sets an
    // ephemeral extmark comes back through `nvim_buf_set_extmark`, which
    // reaches the same state from the API side.
    let decor = unsafe { DecorStateRef::current() };
    decor_redraw_reset(window, decor);
    decor_providers_invoke_win(window, decor);

    add_suspended_terminal_note(buf, decor);

    // The sign column width is per buffer, so a change to it invalidates
    // every window showing that buffer -- including this one.
    for win in winlayer::windows() {
        if win.w_buffer == buf.raw() && win_redraw_signcols(win) {
            changed_line_abv_curs_win(win);
            redraw_later(win, UPD_NOT_VALID);
        }
    }
    buf.b_signcols.last_max = buf.b_signcols.max;

    // Validate `w_virtcol` here: it can change the redraw type, which is
    // why the type is read again from the window afterwards.
    validate_virtcol(win);
    rg.redr_type = win.w_redr_type;

    unsafe { init_search_hl(window, SearchHl::current().raw()) };

    clamp_skipcol(window);

    let nrwidth_before = win.w_nrwidth;
    let nrwidth_new = if win.w_onebuf_opt.wo_nu != 0
        || win.w_onebuf_opt.wo_rnu != 0
        || win.w_onebuf_opt.wo_stc.first_byte() != 0
    {
        number_width(window)
    } else {
        0
    };
    if win.w_nrwidth != nrwidth_new {
        // Every line's columns shift; nothing can be reused.
        rg.redr_type = UPD_NOT_VALID;
        changed_line_abv_curs_win(win);
        win.w_nrwidth = nrwidth_new;
    } else {
        find_changed_lines(win, buf, &mut rg);
    }

    win.w_redraw_top = 0; // reset for next time
    win.w_redraw_bot = 0;
    search_hl_has_cursor_lnum.set(0);

    // UPD_REDRAW_TOP: only the top `w_upd_rows` lines, used when the window
    // scrolled down for `msg_scrolled`.
    if rg.redr_type == UPD_REDRAW_TOP {
        let mut rows = 0;
        for i in 0..win.w_lines_valid {
            rows += unsafe { *win.w_lines.add(i as usize) }.wl_size as c_int;
            if rows >= win.w_upd_rows {
                rg.top_end = rows;
                break;
            }
        }
        rg.redr_type = if rg.top_end == 0 {
            UPD_NOT_VALID // not found (cannot happen?): redraw everything
        } else {
            UPD_VALID // top area defined, the rest is UPD_VALID
        };
    }

    plan_scroll(win, buf, &mut rg);

    if rg.redr_type == UPD_SOME_VALID {
        rg.redraw_all(window);
        rg.redr_type = UPD_NOT_VALID;
    }

    plan_visual_area(win, buf, &mut rg);
    remember_visual_area(window, buf);

    let mut cursorline_fi = FoldInfo::default();
    unsafe { win_update_cursorline(window, &raw mut cursorline_fi) };
    if window.raw() == Win::current_raw() {
        conceal_cursor_used.set(conceal_cursor_line(Win::current()));
    }

    win_check_ns_hl(Some(window));

    let mut spv = SpellVars::default();
    if spell_check_window(window) {
        spv.spv_has_spell = true;
        spv.spv_unchanged = rg.mod_top == 0;
    }

    let old_botline = draw_window_lines(window, buf, &mut rg, cursorline_fi, &mut spv, decor);

    if win.w_redr_type >= UPD_REDRAW_TOP {
        draw_vsep_win(window);
        draw_hsep_win(window);
    }
    unsafe { syn_set_timeout(::core::ptr::null_mut()) };

    // The window has been updated.
    win.w_redr_type = 0;
    win.w_old_topfill = win.w_topfill;
    win.w_old_botfill = win.w_botfill;

    send_win_extmarks(window);

    finish_botline(window, buf, old_botline, nrwidth_before);

    // Restore `got_int`, unless CTRL-C was hit while redrawing.
    if !got_int.get() {
        got_int.set(save_got_int);
    }
}

/// Show `[Process suspended]` over the last line of a suspended `:terminal`.
///
/// The note is a decoration rather than drawn text, so it goes through the
/// decoration state the rest of the redraw already reads.
fn add_suspended_terminal_note(buffer: Buf, state: DecorStateRef) {
    // Both live for the whole process: `decor_range_add_virt` stores the
    // pointer and the range is dropped at the end of the redraw. Declarations,
    // so they sit outside the promise below.
    static CHUNK: GlobalCell<VirtTextChunk> = GlobalCell::new(VirtTextChunk {
        text: c"[Process suspended]".as_ptr().cast_mut(),
        hl_id: -1,
    });
    static VIRT_TEXT: GlobalCell<DecorVirtText> = GlobalCell::new(DecorVirtText {
        flags: 0,
        hl_mode: 0,
        priority: DECOR_PRIORITY_BASE as DecorPriority,
        width: 0,
        col: 0,
        pos: kVPosWinCol,
        data: DecorVirtText_data::Text(VirtText {
            size: 1,
            capacity: 0,
            items: CHUNK.as_raw().cast::<VirtTextChunk>(),
        }),
        next: ::core::ptr::null_mut(),
    });

    // SAFETY: the caller's buffer.
    if buffer.terminal.is_null() || !unsafe { terminal_suspended(buffer.terminal) } {
        return;
    }
    let last = buffer.b_ml.ml_line_count - 1;
    unsafe { decor_range_add_virt(state, last, 0, last, 0, VIRT_TEXT.ptr(), false) };
}

/// Round `w_skipcol` down to a column a wrapped line actually starts a screen
/// row at.
///
/// It depends on the window width and on several options, any of which may have
/// changed since it was set.
fn clamp_skipcol(mut window: Win) {
    if window.w_skipcol <= 0 || window.w_view_width <= window.col_off() {
        return;
    }
    let width1 = window.w_view_width - window.col_off();
    let width2 = width1 + win_col_off2(window);

    // The first screen row of a wrapped line is `width1` wide and every
    // later one `width2`, so the valid skip columns are that series.
    let mut at = 0;
    let mut step = width1;
    while at < window.w_skipcol {
        if at > 0 {
            step = width2;
        }
        at += step;
    }
    if at != window.w_skipcol {
        // Always round down; the higher value may not be valid.
        window.w_skipcol = at - step;
    }
}

/// Work out which buffer lines changed since the last redraw.
///
/// Fills `mod_top` / `mod_bot`, and `top_to_mod` when a multi-line pattern
/// means a change can invalidate highlighting *above* itself. Only reached when
/// the number column kept its width -- otherwise the whole window is redrawn
/// and none of this matters.
fn find_changed_lines(win: Win, buffer: Buf, rg: &mut Regions) {
    // SAFETY: the caller's window and buffer.
    // What `redraw_win_range_later` asked for.
    rg.mod_top = win.w_redraw_top;
    rg.mod_bot = if win.w_redraw_bot != 0 {
        win.w_redraw_bot + 1
    } else {
        0
    };

    if buffer.b_mod_set {
        if rg.mod_top == 0 || rg.mod_top > buffer.b_mod_top {
            rg.mod_top = buffer.b_mod_top;
            // Lines above the change may be included in a pattern match.
            if syntax_present(win) {
                rg.mod_top -= buffer.b_s.b_syn_sync_linebreaks;
                rg.mod_top = rg.mod_top.max(1);
            }
        }
        if rg.mod_bot == 0 || rg.mod_bot < buffer.b_mod_bot {
            rg.mod_bot = buffer.b_mod_bot;
        }

        // With a multi-line 'hlsearch' or :match pattern, a change in one
        // line can invalidate the highlighting of an earlier one. Simple
        // solution: redraw every visible line above the change.
        let regprog = SearchHl::current().regprog();
        rg.top_to_mod = if !regprog.is_null() {
            unsafe { re_multiline(regprog) }
        } else {
            false
        };
        if !rg.top_to_mod {
            let mut cur = win.w_match_head;
            while !cur.is_null() {
                if !unsafe { (*cur).mit_match.regprog }.is_null()
                    && unsafe { re_multiline((*cur).mit_match.regprog) }
                {
                    rg.top_to_mod = true;
                    break;
                }
                cur = unsafe { (*cur).mit_next };
            }
        }
    }

    if search_hl_has_cursor_lnum.get() > 0 {
        // CurSearch was used last time; that line has to be redrawn or two
        // matches end up highlighted with it.
        let cursor_lnum = search_hl_has_cursor_lnum.get();
        if rg.mod_top == 0 || rg.mod_top > cursor_lnum {
            rg.mod_top = cursor_lnum;
        }
        if rg.mod_bot == 0 || rg.mod_bot < cursor_lnum + 1 {
            rg.mod_bot = cursor_lnum + 1;
        }
    }

    if rg.mod_top != 0 && win_lines_concealed(win) {
        widen_over_folds(win, rg);
    }

    // A change that starts above `w_topline` and ends below it starts the
    // redraw at `w_topline`. One that ends above it needs only the first
    // line redrawn, to pick up the syntax state.
    if rg.mod_top != 0 && rg.mod_top < win.w_topline {
        if rg.mod_bot > win.w_topline {
            rg.mod_top = win.w_topline;
        } else if syntax_present(win) {
            rg.top_end = 1;
        }
    }
}

/// Widen the changed range to whole folds and past concealed lines.
///
/// A change in one line can fold or unfold the lines around it, so the range
/// has to reach the first displayed line that could be affected: whichever is
/// higher of the fold `mod_top` is in and the line after the last still-valid
/// `w_lines[]` entry above it, and symmetrically below.
fn widen_over_folds(win: Win, rg: &mut Regions) {
    // SAFETY: the caller's window and its `w_lines` array.
    // The line below the last valid entry above `mod_top`, and the first
    // valid entry at or below `mod_bot`.
    let mut lnumt = win.w_topline;
    let mut lnumb = MAXLNUM;
    for i in 0..win.w_lines_valid {
        let wl = unsafe { win.w_lines.add(i as usize) };
        if !unsafe { (*wl).wl_valid } {
            continue;
        }
        if unsafe { (*wl).wl_lastlnum } < rg.mod_top {
            lnumt = unsafe { (*wl).wl_lastlnum } + 1;
        }
        if lnumb == MAXLNUM && unsafe { (*wl).wl_lnum } >= rg.mod_bot {
            lnumb = unsafe { (*wl).wl_lnum };
            // A fold column may need updating on the next line as well
            // ("J" just above an open fold).
            if compute_foldcolumn(win, 0) > 0 {
                lnumb += 1;
            }
        }
    }

    has_folding(win, rg.mod_top, Some(&mut rg.mod_top), None);
    rg.mod_top = rg.mod_top.min(lnumt);

    // The same for the bottom, on the line one above `mod_bot`.
    rg.mod_bot -= 1;
    has_folding(win, rg.mod_bot, None, Some(&mut rg.mod_bot));
    rg.mod_bot += 1;
    rg.mod_bot = rg.mod_bot.max(lnumb);
}

/// Take the scrolling shortcut, when there is one.
///
/// Three cases, all of them only reachable when nothing forces a full redraw:
///
/// 1. the window is a few lines off the top -- scroll down,
/// 2. `w_topline` is below `w_lines[0]` -- scroll up,
/// 3. `w_topline` did not move -- find the first stale `w_lines[]` entry.
///
/// Each one either moves rows on the grid and records which rows are now stale,
/// or gives up and marks the whole window.
fn plan_scroll(win: Win, buffer: Buf, rg: &mut Regions) {
    // SAFETY: the caller's window, its buffer and its `w_lines` array.
    // `w_lines[0].wl_lnum` can be below `w_topline` when the top line is
    // concealed, which would read as a scroll that did not happen. Compare
    // against a topline adjusted the same way.
    //
    // This runs whether or not the window is scrollable, as upstream has
    // it: `decor_conceal_line` invokes the decoration providers, so
    // skipping it on the non-scrollable path would be a change.
    let mut topline_conceal = win.w_topline;
    while topline_conceal < buffer.b_ml.ml_line_count
        && decor_conceal_line(win, topline_conceal - 1, false)
    {
        topline_conceal += 1;
        has_folding(win, topline_conceal, None, Some(&mut topline_conceal));
    }

    let scrollable = matches!(
        rg.redr_type,
        UPD_VALID | UPD_SOME_VALID | UPD_INVERTED | UPD_INVERTED_ALL
    ) && !win.w_botfill
        && !win.w_old_botfill;
    if !scrollable {
        // Not UPD_VALID or UPD_INVERTED: redraw all lines.
        rg.redraw_all(win);
        return;
    }

    let first = win.w_lines;
    if rg.mod_top != 0
        && win.w_topline == rg.mod_top
        && (!unsafe { (*first).wl_valid } || topline_conceal == unsafe { (*first).wl_lnum })
    {
        // `w_topline` is the first changed line and the window did not
        // scroll: the line loop scrolls for the changed lines instead.
    } else if unsafe { (*first).wl_valid }
        && (topline_conceal < unsafe { (*first).wl_lnum }
            || (topline_conceal == unsafe { (*first).wl_lnum }
                && win.w_topfill > win.w_old_topfill))
    {
        scroll_down(win, rg);
    } else {
        scroll_up(win, rg);
    }

    // Redrawing from the first row means redrawing everything.
    if rg.mid_start == 0 {
        rg.mid_end = win.w_view_height;
    }
}

/// The new topline is above the old one: insert rows at the top.
fn scroll_down(mut win: Win, rg: &mut Regions) {
    // SAFETY: the caller's window and its `w_lines` array.
    let first_lnum = unsafe { (*win.w_lines).wl_lnum };

    // How many lines the window is off by, counting a run of folded lines
    // as one and skipping concealed ones.
    let off = if win_lines_concealed(win) {
        let mut count = 0;
        let mut ln = win.w_topline;
        while ln < first_lnum {
            count += c_int::from(!decor_conceal_line(win, ln - 1, false));
            if count >= win.w_view_height - 2 {
                break;
            }
            has_folding(win, ln, None, Some(&mut ln));
            ln += 1;
        }
        count
    } else {
        first_lnum - win.w_topline
    };

    if off >= win.w_view_height - 2 {
        rg.mid_start = 0; // too far off: redraw all lines
        return;
    }

    let mut rows = plines_m_win(win, win.w_topline, first_lnum - 1, win.w_view_height);
    // Extra rows for filler lines that were not visible before.
    if first_lnum != win.w_topline {
        rows += win_get_fill(win, first_lnum) - win.w_old_topfill;
    }
    if rows == 0 || rows >= win.w_view_height - 2 {
        rg.mid_start = 0; // a screen or more off: redraw all lines
        return;
    }

    // Insert that many rows; if this is not the last window the rows at the
    // bottom are deleted. May fail if the terminal cannot do it.
    win_scroll_lines(win, 0, rows);
    rg.bot_scroll_start = 0;
    if win.w_lines_valid == 0 {
        return;
    }

    // The rows that are new have to be drawn, and the entries that were
    // scrolled move with them.
    rg.top_end = rows;
    rg.scrolled_down = true;

    win.w_lines_valid = (win.w_lines_valid + off).min(win.w_view_height);
    let mut idx = win.w_lines_valid;
    while idx - off >= 0 {
        unsafe { *win.w_lines.add(idx as usize) = *win.w_lines.add((idx - off) as usize) };
        idx -= 1;
    }
    // The entries the scrolled ones vacated describe lines that are gone.
    while idx >= 0 {
        unsafe { (*win.w_lines.add(idx as usize)).wl_valid = false };
        idx -= 1;
    }
}

/// The new topline is at or below the old one: delete rows at the top, or find
/// the first `w_lines[]` entry that is stale.
fn scroll_up(mut win: Win, rg: &mut Regions) {
    // Find `w_topline` in `w_lines[]`, counting the rows above it.
    let mut at = -1;
    let mut rows = 0;
    // SAFETY: the caller's window and its `w_lines` array.
    for i in 0..win.w_lines_valid {
        let wl = unsafe { win.w_lines.add(i as usize) };
        if unsafe { (*wl).wl_valid } && unsafe { (*wl).wl_lnum } == win.w_topline {
            at = i;
            break;
        }
        rows += unsafe { (*wl).wl_size } as c_int;
    }
    let Ok(mut at) = usize::try_from(at) else {
        // Not displayed at all: redraw everything.
        rg.mid_start = 0;
        return;
    };

    // Delete the filler lines of the old topline if it did not move, of the
    // new one otherwise -- but keep the new topline's own filler lines.
    if unsafe { (*win.w_lines).wl_lnum } == win.w_topline {
        rows += win.w_old_topfill;
    } else {
        rows += win_get_fill(win, win.w_topline);
    }
    rows -= win.w_topfill;

    if rows > 0 {
        win_scroll_lines(win, 0, -rows);
        rg.bot_start = win.w_view_height - rows;
        rg.bot_scroll_start = rg.bot_start;
    }
    if (rows != 0 && rg.bot_start >= NO_ROW) || win.w_lines_valid == 0 {
        return;
    }

    // The entries below the deleted rows are still valid; copy them up to
    // compensate, and set `bot_start` to the first row that does need
    // drawing.
    rg.bot_start = 0;
    let mut idx = 0usize;
    loop {
        unsafe { *win.w_lines.add(idx) = *win.w_lines.add(at) };
        // Stop at a line that did not fit -- unless nothing was deleted, in
        // which case it is still valid where it is.
        if rows > 0
            && rg.bot_start + rows + unsafe { (*win.w_lines.add(at)).wl_size } as c_int
                > win.w_view_height
        {
            win.w_lines_valid = idx as c_int + 1;
            break;
        }
        rg.bot_start += unsafe { (*win.w_lines.add(idx)).wl_size } as c_int;
        idx += 1;
        at += 1;
        // Stop at the last valid entry.
        if at >= win.w_lines_valid as usize {
            win.w_lines_valid = idx as c_int;
            break;
        }
    }

    // Correct the first entry for filler lines at the top when it is not
    // going to be drawn below.
    if win_may_fill(win) && rg.bot_start > 0 {
        unsafe {
            (*win.w_lines).wl_size = plines_correct_topline(win, win.w_topline, true).0 as uint16_t
        };
    }
}

/// Widen the mid area to cover the Visual selection that is being drawn or
/// taken away.
fn plan_visual_area(win: Win, buffer: Buf, rg: &mut Regions) {
    let shown = visual_selection().filter(|_| buffer == Win::current().buffer());
    if shown.is_none() && !(win.w_old_cursor_lnum != 0 && rg.redr_type != UPD_NOT_VALID) {
        return;
    }

    let (mut from, mut to) = if let Some(sel) = shown {
        visual_line_range(win, sel, rg.redr_type)
    } else {
        // The selection is gone; use the line numbers of the old one.
        let a = win.w_old_cursor_lnum;
        let b = win.w_old_visual_lnum;
        (a.min(b), a.max(b))
    };

    // No need to update lines above the top of the window.
    from = from.max(win.w_topline);
    // If `w_botline` is known, restrict to what is visible.
    if win.w_valid.has(WinValid::BOTLINE) {
        from = from.min(win.w_botline - 1);
        to = to.min(win.w_botline - 1);
    }

    // Find the minimal row range covering `from ..= to`. Entries that
    // scrolling made invalid still count towards `srow`, because a middle
    // mouse click with a selection can change the text above the Visual
    // area and reset `wl_valid`.
    if rg.mid_start <= 0 {
        return;
    }
    let mut lnum = win.w_topline;
    let mut idx = 0;
    let mut srow = 0;
    rg.mid_start = if rg.scrolled_down { rg.top_end } else { 0 };

    while lnum < from && idx < win.w_lines_valid {
        let wl = unsafe { win.w_lines.add(idx as usize) };
        if unsafe { (*wl).wl_valid } {
            rg.mid_start += unsafe { (*wl).wl_size } as c_int;
        } else if !rg.scrolled_down {
            srow += unsafe { (*wl).wl_size } as c_int;
        }
        idx += 1;
        lnum = if idx < win.w_lines_valid && unsafe { *win.w_lines.add(idx as usize) }.wl_valid {
            unsafe { *win.w_lines.add(idx as usize) }.wl_lnum
        } else {
            lnum + 1
        };
    }
    srow += rg.mid_start;

    rg.mid_end = win.w_view_height;
    while idx < win.w_lines_valid {
        let wl = unsafe { win.w_lines.add(idx as usize) };
        if unsafe { (*wl).wl_valid } && unsafe { (*wl).wl_lnum } > to {
            // Only update to the first row of this line.
            rg.mid_end = srow;
            break;
        }
        srow += unsafe { (*wl).wl_size } as c_int;
        idx += 1;
    }
}

/// The buffer lines the Visual selection needs redrawn.
///
/// Either the whole selection (its kind changed, or the caller asked for all of
/// it) or just the lines between where the cursor and the anchor were and where
/// they are now.
fn visual_line_range(mut win: Win, sel: VisualSelection, redr_type: c_int) -> (LineNr, LineNr) {
    // SAFETY: the caller's window.
    let cursor = Win::current().w_cursor.lnum;
    let anchor = sel.anchor.lnum;

    let (mut from, mut to) =
        if sel.mode.raw() != win.w_old_visual_mode as c_int || redr_type == UPD_INVERTED_ALL {
            // The kind of selection changed, or the X selection was gained or
            // lost: redraw all of it, and the lines the cursor moved over.
            let (a, b) = (cursor.min(anchor), cursor.max(anchor));
            (
                a.min(win.w_old_cursor_lnum).min(win.w_old_visual_lnum),
                b.max(win.w_old_cursor_lnum).max(win.w_old_visual_lnum),
            )
        } else {
            // Just the lines between the old cursor position and the new one,
            // plus the anchor if it moved.
            let (mut from, to) = if cursor < win.w_old_cursor_lnum {
                (cursor, win.w_old_cursor_lnum)
            } else if win.w_old_cursor_lnum == 0 {
                (cursor, cursor) // Visual mode just started
            } else {
                (win.w_old_cursor_lnum, cursor)
            };
            let mut to = to;
            if anchor != win.w_old_visual_lnum || sel.anchor.col != win.w_old_visual_col {
                if win.w_old_visual_lnum < from && win.w_old_visual_lnum != 0 {
                    from = win.w_old_visual_lnum;
                }
                to = to.max(win.w_old_visual_lnum).max(anchor);
                from = from.min(anchor);
            }
            (from, to)
        };

    // Blockwise: a changed column or `w_curswant` means every line of the
    // selection has to be redrawn, so the actual columns are computed here.
    if sel.mode.is_block() {
        let (fromc, toc) = visual_block_columns(win, sel);
        if fromc != win.w_old_cursor_fcol || toc != win.w_old_cursor_lcol {
            from = from.min(anchor);
            to = to.max(anchor);
        }
        win.w_old_cursor_fcol = fromc;
        win.w_old_cursor_lcol = toc;
    }

    (from, to)
}

/// The first and last screen columns of a blockwise Visual selection.
fn visual_block_columns(win: Win, sel: VisualSelection) -> (ColNr, ColNr) {
    // A copy of the anchor: `getvcols` only reads it.
    let mut anchor = sel.anchor;
    // SAFETY: the caller's window.
    let mut fromc = 0;
    let mut toc = 0;

    // With 'linebreak' the columns are computed as if 'virtualedit' were
    // "all", because that is how the selection is drawn.
    let save_ve_flags = Win::current().w_onebuf_opt.wo_ve_flags;
    if Win::current().w_onebuf_opt.wo_lbr != 0 {
        Win::current().w_onebuf_opt.wo_ve_flags = kOptVeFlagAll;
    }
    unsafe {
        getvcols(
            win,
            &raw mut anchor,
            &raw mut (*Win::current_raw()).w_cursor,
            &raw mut fromc,
            &raw mut toc,
        )
    };
    toc += 1;
    Win::current().w_onebuf_opt.wo_ve_flags = save_ve_flags;

    if Win::current().w_curswant != MAXCOL as ColNr {
        return (fromc, toc);
    }

    // `$` in blockwise mode: highlight to the end of every line, unless
    // 'virtualedit' has "block", in which case it stops at the longest one.
    if get_ve_flags(Win::current()) & kOptVeFlagBlock == 0 {
        return (fromc, MAXCOL as ColNr);
    }

    let cursor_lnum = Win::current().w_cursor.lnum;
    let anchor_lnum = sel.anchor.lnum;
    let cursor_above = cursor_lnum < anchor_lnum;
    let mut pos = Pos::default();
    toc = 0;
    let mut lnum = cursor_lnum;
    while if cursor_above {
        lnum <= anchor_lnum
    } else {
        lnum >= anchor_lnum
    } {
        pos.lnum = lnum;
        pos.col = ml_get_buf_len(win.buffer(), lnum);
        let mut end = 0;
        unsafe {
            getvvcol(
                win,
                &raw mut pos,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                &raw mut end,
            )
        };
        toc = toc.max(end);
        lnum += if cursor_above { 1 } else { -1 };
    }
    (fromc, toc + 1)
}

/// Record the Visual selection this redraw drew, so the next one can tell what
/// moved.
fn remember_visual_area(mut window: Win, buffer: Buf) {
    if let Some(sel) = visual_selection().filter(|_| buffer == Win::current().buffer()) {
        window.w_old_visual_mode = sel.mode.raw() as c_char;
        window.w_old_cursor_lnum = Win::current().w_cursor.lnum;
        window.w_old_visual_lnum = sel.anchor.lnum;
        window.w_old_visual_col = sel.anchor.col;
        window.w_old_curswant = Win::current().w_curswant;
    } else {
        window.w_old_visual_mode = 0;
        window.w_old_cursor_lnum = 0;
        window.w_old_visual_lnum = 0;
        window.w_old_visual_col = 0;
    }
}

/// Report the `ui_watched` extmarks this redraw passed to the UI.
fn send_win_extmarks(window: Win) {
    win_extmark_arr.with(|marks| {
        for m in marks {
            ui_call_win_extmark(
                window.w_grid_alloc.handle as Integer,
                window.handle as WindowHandle,
                m.ns_id as Integer,
                m.mark_id as Integer,
                m.win_row as Integer,
                m.win_col as Integer,
            );
        }
    });
}

/// Publish `w_botline`, and redraw once more if it turned out to be wrong.
///
/// `w_botline` is deliberately approximated between redraws -- keeping it exact
/// would mean a `plines_win` walk on every change -- so this is where the
/// approximation is checked against what was actually drawn. When it was wrong
/// the cursor may be off screen, and the fix is another `win_update`.
///
/// `old_botline` is what `w_botline` held before the line loop replaced it.
fn finish_botline(mut window: Win, buffer: Buf, old_botline: LineNr, nrwidth_before: c_int) {
    // Recursion guard: the second pass must not start a third.
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);

    if dollar_vcol.get() == -1 || window.raw() != Win::current_raw() {
        window.w_valid |= WinValid::BOTLINE;
        window.w_viewport_invalid = true;
        if window.raw() == Win::current_raw() && window.w_botline != old_botline && !RECURSIVE.get()
        {
            RECURSIVE.set(true);
            Win::current().w_valid.clear(WinValid::TOPLINE);
            update_topline(Win::current()); // may invalidate w_botline again
            // A new redraw, either from a moved topline or a reset skipcol.
            if must_redraw.get() != 0 {
                // Do not update for the buffer changes a second time.
                let mod_set = Buf::current().b_mod_set;
                Buf::current().b_mod_set = false;
                curs_columns(Win::current(), c_int::from(true));
                win_update(Win::current());
                must_redraw.set(0);
                Buf::current().b_mod_set = mod_set;
            }
            RECURSIVE.set(false);
        }
    }

    if nrwidth_before != window.w_nrwidth && !buffer.terminal.is_null() {
        unsafe { terminal_check_size(buffer.terminal) };
    }
}
