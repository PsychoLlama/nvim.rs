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

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::decoration::kVPosWinCol;
use crate::src::nvim::keycodes::Ctrl_V;
use crate::src::nvim::pos::MAXCOL;

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
    pub mod_top: linenr_T,
    /// First buffer line after the change; 0 when nothing changed.
    pub mod_bot: linenr_T,
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
    fn redraw_all(&mut self, wp: *mut win_T) {
        // SAFETY: a live window.
        self.mid_start = 0;
        self.mid_end = unsafe { (*wp).w_view_height };
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
pub(crate) unsafe fn win_update(wp: *mut win_T) {
    // SAFETY: a live window of the current layout, during a redraw.
    unsafe {
        // Return early when the window would overflow a shrunk terminal, which
        // would draw out of bounds and trip an assertion.
        if (*wp).w_grid.target == default_grid.ptr() && (*wp).w_wincol >= Columns.get() {
            return;
        }

        let mut rg = Regions::new((*wp).w_redr_type);
        if rg.redr_type >= UPD_NOT_VALID {
            (*wp).w_redr_status = true;
            (*wp).w_lines_valid = 0;
        }

        // A window with no room for text only needs its separator.
        if (*wp).w_view_height == 0 {
            draw_hsep_win(wp);
            (*wp).w_redr_type = 0;
            return;
        }
        if (*wp).w_view_width == 0 {
            draw_vsep_win(wp);
            (*wp).w_redr_type = 0;
            return;
        }

        let buf = (*wp).w_buffer;

        // Reset `got_int`, otherwise the regexp engine will not work.
        let save_got_int = got_int.get();
        got_int.set(false);
        // Bound syntax highlighting by 'redrawtime'.
        let mut syntax_tm = profile_setlimit(p_rdt.get() as int64_t);
        syn_set_timeout(&raw mut syntax_tm);

        win_extmark_arr.with_mut(Vec::clear);

        decor_redraw_reset(wp, decor_state.ptr());
        decor_providers_invoke_win(wp);

        add_suspended_terminal_note(buf);

        // The sign column width is per buffer, so a change to it invalidates
        // every window showing that buffer -- including this one.
        for win in windows_in_curtab() {
            if (*win).w_buffer == buf && win_redraw_signcols(win) {
                changed_line_abv_curs_win(win);
                redraw_later(win, UPD_NOT_VALID);
            }
        }
        (*buf).b_signcols.last_max = (*buf).b_signcols.max;

        // Validate `w_virtcol` here: it can change the redraw type, which is
        // why the type is read again from the window afterwards.
        validate_virtcol(wp);
        rg.redr_type = (*wp).w_redr_type;

        init_search_hl(wp, screen_search_hl.ptr());

        clamp_skipcol(wp);

        let nrwidth_before = (*wp).w_nrwidth;
        let nrwidth_new = if (*wp).w_onebuf_opt.wo_nu != 0
            || (*wp).w_onebuf_opt.wo_rnu != 0
            || *(*wp).w_onebuf_opt.wo_stc != 0
        {
            number_width(wp)
        } else {
            0
        };
        if (*wp).w_nrwidth != nrwidth_new {
            // Every line's columns shift; nothing can be reused.
            rg.redr_type = UPD_NOT_VALID;
            changed_line_abv_curs_win(wp);
            (*wp).w_nrwidth = nrwidth_new;
        } else {
            find_changed_lines(wp, buf, &mut rg);
        }

        (*wp).w_redraw_top = 0; // reset for next time
        (*wp).w_redraw_bot = 0;
        search_hl_has_cursor_lnum.set(0);

        // UPD_REDRAW_TOP: only the top `w_upd_rows` lines, used when the window
        // scrolled down for `msg_scrolled`.
        if rg.redr_type == UPD_REDRAW_TOP {
            let mut rows = 0;
            for i in 0..(*wp).w_lines_valid {
                rows += (*(*wp).w_lines.add(i as usize)).wl_size as c_int;
                if rows >= (*wp).w_upd_rows {
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

        plan_scroll(wp, buf, &mut rg);

        if rg.redr_type == UPD_SOME_VALID {
            rg.redraw_all(wp);
            rg.redr_type = UPD_NOT_VALID;
        }

        plan_visual_area(wp, buf, &mut rg);
        remember_visual_area(wp, buf);

        let mut cursorline_fi = foldinfo_T::default();
        win_update_cursorline(wp, &raw mut cursorline_fi);
        if wp == curwin.get() {
            conceal_cursor_used.set(conceal_cursor_line(curwin.get()));
        }

        win_check_ns_hl(wp);

        let mut spv = spellvars_T::default();
        if spell_check_window(wp) {
            spv.spv_has_spell = true;
            spv.spv_unchanged = rg.mod_top == 0;
        }

        let old_botline = draw_window_lines(wp, buf, &mut rg, cursorline_fi, &mut spv);

        if (*wp).w_redr_type >= UPD_REDRAW_TOP {
            draw_vsep_win(wp);
            draw_hsep_win(wp);
        }
        syn_set_timeout(::core::ptr::null_mut());

        // The window has been updated.
        (*wp).w_redr_type = 0;
        (*wp).w_old_topfill = (*wp).w_topfill;
        (*wp).w_old_botfill = (*wp).w_botfill;

        send_win_extmarks(wp);

        finish_botline(wp, buf, old_botline, nrwidth_before);

        // Restore `got_int`, unless CTRL-C was hit while redrawing.
        if !got_int.get() {
            got_int.set(save_got_int);
        }
    }
}

/// Show `[Process suspended]` over the last line of a suspended `:terminal`.
///
/// The note is a decoration rather than drawn text, so it goes through the
/// decoration state the rest of the redraw already reads.
///
/// # Safety
/// Called from [`win_update`] with the decoration state reset for this window.
unsafe fn add_suspended_terminal_note(buf: *mut buf_T) {
    // SAFETY: the caller's buffer and the redraw's decoration state.
    unsafe {
        if (*buf).terminal.is_null() || !terminal_suspended((*buf).terminal) {
            return;
        }
        // Both live for the whole process: `decor_range_add_virt` stores the
        // pointer and the range is dropped at the end of the redraw.
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
            data: C2Rust_Unnamed_2 {
                virt_text: VirtText {
                    size: 1,
                    capacity: 0,
                    items: CHUNK.as_raw().cast::<VirtTextChunk>(),
                },
            },
            next: ::core::ptr::null_mut(),
        });
        let last = (*buf).b_ml.ml_line_count - 1;
        decor_range_add_virt(decor_state.ptr(), last, 0, last, 0, VIRT_TEXT.ptr(), false);
    }
}

/// Round `w_skipcol` down to a column a wrapped line actually starts a screen
/// row at.
///
/// It depends on the window width and on several options, any of which may have
/// changed since it was set.
///
/// # Safety
/// `wp` must be a live window.
unsafe fn clamp_skipcol(wp: *mut win_T) {
    // SAFETY: a live window.
    unsafe {
        if (*wp).w_skipcol <= 0 || (*wp).w_view_width <= win_col_off(wp) {
            return;
        }
        let width1 = (*wp).w_view_width - win_col_off(wp);
        let width2 = width1 + win_col_off2(wp);

        // The first screen row of a wrapped line is `width1` wide and every
        // later one `width2`, so the valid skip columns are that series.
        let mut at = 0;
        let mut step = width1;
        while at < (*wp).w_skipcol {
            if at > 0 {
                step = width2;
            }
            at += step;
        }
        if at != (*wp).w_skipcol {
            // Always round down; the higher value may not be valid.
            (*wp).w_skipcol = at - step;
        }
    }
}

/// Work out which buffer lines changed since the last redraw.
///
/// Fills `mod_top` / `mod_bot`, and `top_to_mod` when a multi-line pattern
/// means a change can invalidate highlighting *above* itself. Only reached when
/// the number column kept its width -- otherwise the whole window is redrawn
/// and none of this matters.
///
/// # Safety
/// `wp` must be a live window and `buf` its buffer.
unsafe fn find_changed_lines(wp: *mut win_T, buf: *mut buf_T, rg: &mut Regions) {
    // SAFETY: the caller's window and buffer.
    unsafe {
        // What `redraw_win_range_later` asked for.
        rg.mod_top = (*wp).w_redraw_top;
        rg.mod_bot = if (*wp).w_redraw_bot != 0 {
            (*wp).w_redraw_bot + 1
        } else {
            0
        };

        if (*buf).b_mod_set {
            if rg.mod_top == 0 || rg.mod_top > (*buf).b_mod_top {
                rg.mod_top = (*buf).b_mod_top;
                // Lines above the change may be included in a pattern match.
                if syntax_present(wp) {
                    rg.mod_top -= (*buf).b_s.b_syn_sync_linebreaks;
                    rg.mod_top = rg.mod_top.max(1);
                }
            }
            if rg.mod_bot == 0 || rg.mod_bot < (*buf).b_mod_bot {
                rg.mod_bot = (*buf).b_mod_bot;
            }

            // With a multi-line 'hlsearch' or :match pattern, a change in one
            // line can invalidate the highlighting of an earlier one. Simple
            // solution: redraw every visible line above the change.
            rg.top_to_mod = if !(*screen_search_hl.ptr()).rm.regprog.is_null() {
                re_multiline((*screen_search_hl.ptr()).rm.regprog) != 0
            } else {
                false
            };
            if !rg.top_to_mod {
                let mut cur = (*wp).w_match_head;
                while !cur.is_null() {
                    if !(*cur).mit_match.regprog.is_null()
                        && re_multiline((*cur).mit_match.regprog) != 0
                    {
                        rg.top_to_mod = true;
                        break;
                    }
                    cur = (*cur).mit_next;
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

        if rg.mod_top != 0 && win_lines_concealed(wp) {
            widen_over_folds(wp, rg);
        }

        // A change that starts above `w_topline` and ends below it starts the
        // redraw at `w_topline`. One that ends above it needs only the first
        // line redrawn, to pick up the syntax state.
        if rg.mod_top != 0 && rg.mod_top < (*wp).w_topline {
            if rg.mod_bot > (*wp).w_topline {
                rg.mod_top = (*wp).w_topline;
            } else if syntax_present(wp) {
                rg.top_end = 1;
            }
        }
    }
}

/// Widen the changed range to whole folds and past concealed lines.
///
/// A change in one line can fold or unfold the lines around it, so the range
/// has to reach the first displayed line that could be affected: whichever is
/// higher of the fold `mod_top` is in and the line after the last still-valid
/// `w_lines[]` entry above it, and symmetrically below.
///
/// # Safety
/// `wp` must be a live window.
unsafe fn widen_over_folds(wp: *mut win_T, rg: &mut Regions) {
    // SAFETY: the caller's window and its `w_lines` array.
    unsafe {
        // The line below the last valid entry above `mod_top`, and the first
        // valid entry at or below `mod_bot`.
        let mut lnumt = (*wp).w_topline;
        let mut lnumb = MAXLNUM as linenr_T;
        for i in 0..(*wp).w_lines_valid {
            let wl = (*wp).w_lines.add(i as usize);
            if !(*wl).wl_valid {
                continue;
            }
            if (*wl).wl_lastlnum < rg.mod_top {
                lnumt = (*wl).wl_lastlnum + 1;
            }
            if lnumb == MAXLNUM as linenr_T && (*wl).wl_lnum >= rg.mod_bot {
                lnumb = (*wl).wl_lnum;
                // A fold column may need updating on the next line as well
                // ("J" just above an open fold).
                if compute_foldcolumn(wp, 0) > 0 {
                    lnumb += 1;
                }
            }
        }

        hasFolding(wp, rg.mod_top, &raw mut rg.mod_top, ::core::ptr::null_mut());
        rg.mod_top = rg.mod_top.min(lnumt);

        // The same for the bottom, on the line one above `mod_bot`.
        rg.mod_bot -= 1;
        hasFolding(wp, rg.mod_bot, ::core::ptr::null_mut(), &raw mut rg.mod_bot);
        rg.mod_bot += 1;
        rg.mod_bot = rg.mod_bot.max(lnumb);
    }
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
///
/// # Safety
/// `wp` must be a live window and `buf` its buffer.
unsafe fn plan_scroll(wp: *mut win_T, buf: *mut buf_T, rg: &mut Regions) {
    // SAFETY: the caller's window, its buffer and its `w_lines` array.
    unsafe {
        // `w_lines[0].wl_lnum` can be below `w_topline` when the top line is
        // concealed, which would read as a scroll that did not happen. Compare
        // against a topline adjusted the same way.
        //
        // This runs whether or not the window is scrollable, as upstream has
        // it: `decor_conceal_line` invokes the decoration providers, so
        // skipping it on the non-scrollable path would be a change.
        let mut topline_conceal = (*wp).w_topline;
        while topline_conceal < (*buf).b_ml.ml_line_count
            && decor_conceal_line(wp, topline_conceal - 1, false)
        {
            topline_conceal += 1;
            hasFolding(
                wp,
                topline_conceal,
                ::core::ptr::null_mut(),
                &raw mut topline_conceal,
            );
        }

        let scrollable = matches!(
            rg.redr_type,
            UPD_VALID | UPD_SOME_VALID | UPD_INVERTED | UPD_INVERTED_ALL
        ) && !(*wp).w_botfill
            && !(*wp).w_old_botfill;
        if !scrollable {
            // Not UPD_VALID or UPD_INVERTED: redraw all lines.
            rg.redraw_all(wp);
            return;
        }

        let first = (*wp).w_lines;
        if rg.mod_top != 0
            && (*wp).w_topline == rg.mod_top
            && (!(*first).wl_valid || topline_conceal == (*first).wl_lnum)
        {
            // `w_topline` is the first changed line and the window did not
            // scroll: the line loop scrolls for the changed lines instead.
        } else if (*first).wl_valid
            && (topline_conceal < (*first).wl_lnum
                || (topline_conceal == (*first).wl_lnum && (*wp).w_topfill > (*wp).w_old_topfill))
        {
            scroll_down(wp, rg);
        } else {
            scroll_up(wp, rg);
        }

        // Redrawing from the first row means redrawing everything.
        if rg.mid_start == 0 {
            rg.mid_end = (*wp).w_view_height;
        }
    }
}

/// The new topline is above the old one: insert rows at the top.
///
/// # Safety
/// `wp` must be a live window.
unsafe fn scroll_down(wp: *mut win_T, rg: &mut Regions) {
    // SAFETY: the caller's window and its `w_lines` array.
    unsafe {
        let first_lnum = (*(*wp).w_lines).wl_lnum;

        // How many lines the window is off by, counting a run of folded lines
        // as one and skipping concealed ones.
        let off = if win_lines_concealed(wp) {
            let mut count = 0;
            let mut ln = (*wp).w_topline;
            while ln < first_lnum {
                count += c_int::from(!decor_conceal_line(wp, ln - 1, false));
                if count >= (*wp).w_view_height - 2 {
                    break;
                }
                hasFolding(wp, ln, ::core::ptr::null_mut(), &raw mut ln);
                ln += 1;
            }
            count
        } else {
            first_lnum - (*wp).w_topline
        };

        if off >= (*wp).w_view_height - 2 {
            rg.mid_start = 0; // too far off: redraw all lines
            return;
        }

        let mut rows = plines_m_win(wp, (*wp).w_topline, first_lnum - 1, (*wp).w_view_height);
        // Extra rows for filler lines that were not visible before.
        if first_lnum != (*wp).w_topline {
            rows += win_get_fill(wp, first_lnum) - (*wp).w_old_topfill;
        }
        if rows == 0 || rows >= (*wp).w_view_height - 2 {
            rg.mid_start = 0; // a screen or more off: redraw all lines
            return;
        }

        // Insert that many rows; if this is not the last window the rows at the
        // bottom are deleted. May fail if the terminal cannot do it.
        win_scroll_lines(wp, 0, rows);
        rg.bot_scroll_start = 0;
        if (*wp).w_lines_valid == 0 {
            return;
        }

        // The rows that are new have to be drawn, and the entries that were
        // scrolled move with them.
        rg.top_end = rows;
        rg.scrolled_down = true;

        (*wp).w_lines_valid = ((*wp).w_lines_valid + off).min((*wp).w_view_height);
        let mut idx = (*wp).w_lines_valid;
        while idx - off >= 0 {
            *(*wp).w_lines.add(idx as usize) = *(*wp).w_lines.add((idx - off) as usize);
            idx -= 1;
        }
        // The entries the scrolled ones vacated describe lines that are gone.
        while idx >= 0 {
            (*(*wp).w_lines.add(idx as usize)).wl_valid = false;
            idx -= 1;
        }
    }
}

/// The new topline is at or below the old one: delete rows at the top, or find
/// the first `w_lines[]` entry that is stale.
///
/// # Safety
/// `wp` must be a live window.
unsafe fn scroll_up(wp: *mut win_T, rg: &mut Regions) {
    // SAFETY: the caller's window and its `w_lines` array.
    unsafe {
        // Find `w_topline` in `w_lines[]`, counting the rows above it.
        let mut at = -1;
        let mut rows = 0;
        for i in 0..(*wp).w_lines_valid {
            let wl = (*wp).w_lines.add(i as usize);
            if (*wl).wl_valid && (*wl).wl_lnum == (*wp).w_topline {
                at = i;
                break;
            }
            rows += (*wl).wl_size as c_int;
        }
        let Ok(mut at) = usize::try_from(at) else {
            // Not displayed at all: redraw everything.
            rg.mid_start = 0;
            return;
        };

        // Delete the filler lines of the old topline if it did not move, of the
        // new one otherwise -- but keep the new topline's own filler lines.
        if (*(*wp).w_lines).wl_lnum == (*wp).w_topline {
            rows += (*wp).w_old_topfill;
        } else {
            rows += win_get_fill(wp, (*wp).w_topline);
        }
        rows -= (*wp).w_topfill;

        if rows > 0 {
            win_scroll_lines(wp, 0, -rows);
            rg.bot_start = (*wp).w_view_height - rows;
            rg.bot_scroll_start = rg.bot_start;
        }
        if (rows != 0 && rg.bot_start >= NO_ROW) || (*wp).w_lines_valid == 0 {
            return;
        }

        // The entries below the deleted rows are still valid; copy them up to
        // compensate, and set `bot_start` to the first row that does need
        // drawing.
        rg.bot_start = 0;
        let mut idx = 0usize;
        loop {
            *(*wp).w_lines.add(idx) = *(*wp).w_lines.add(at);
            // Stop at a line that did not fit -- unless nothing was deleted, in
            // which case it is still valid where it is.
            if rows > 0
                && rg.bot_start + rows + (*(*wp).w_lines.add(at)).wl_size as c_int
                    > (*wp).w_view_height
            {
                (*wp).w_lines_valid = idx as c_int + 1;
                break;
            }
            rg.bot_start += (*(*wp).w_lines.add(idx)).wl_size as c_int;
            idx += 1;
            at += 1;
            // Stop at the last valid entry.
            if at >= (*wp).w_lines_valid as usize {
                (*wp).w_lines_valid = idx as c_int;
                break;
            }
        }

        // Correct the first entry for filler lines at the top when it is not
        // going to be drawn below.
        if win_may_fill(wp) && rg.bot_start > 0 {
            (*(*wp).w_lines).wl_size =
                plines_correct_topline(wp, (*wp).w_topline, ::core::ptr::null_mut(), true)
                    as uint16_t;
        }
    }
}

/// Widen the mid area to cover the Visual selection that is being drawn or
/// taken away.
///
/// # Safety
/// `wp` must be a live window and `buf` its buffer.
unsafe fn plan_visual_area(wp: *mut win_T, buf: *mut buf_T, rg: &mut Regions) {
    // SAFETY: the caller's window, its buffer and the global Visual state.
    unsafe {
        let showing_visual = VIsual_active.get() && buf == (*curwin.get()).w_buffer;
        if !showing_visual && !((*wp).w_old_cursor_lnum != 0 && rg.redr_type != UPD_NOT_VALID) {
            return;
        }

        let (mut from, mut to) = if showing_visual {
            visual_line_range(wp, rg.redr_type)
        } else {
            // The selection is gone; use the line numbers of the old one.
            let a = (*wp).w_old_cursor_lnum;
            let b = (*wp).w_old_visual_lnum;
            (a.min(b), a.max(b))
        };

        // No need to update lines above the top of the window.
        from = from.max((*wp).w_topline);
        // If `w_botline` is known, restrict to what is visible.
        if (*wp).w_valid & VALID_BOTLINE != 0 {
            from = from.min((*wp).w_botline - 1);
            to = to.min((*wp).w_botline - 1);
        }

        // Find the minimal row range covering `from ..= to`. Entries that
        // scrolling made invalid still count towards `srow`, because a middle
        // mouse click with a selection can change the text above the Visual
        // area and reset `wl_valid`.
        if rg.mid_start <= 0 {
            return;
        }
        let mut lnum = (*wp).w_topline;
        let mut idx = 0;
        let mut srow = 0;
        rg.mid_start = if rg.scrolled_down { rg.top_end } else { 0 };

        while lnum < from && idx < (*wp).w_lines_valid {
            let wl = (*wp).w_lines.add(idx as usize);
            if (*wl).wl_valid {
                rg.mid_start += (*wl).wl_size as c_int;
            } else if !rg.scrolled_down {
                srow += (*wl).wl_size as c_int;
            }
            idx += 1;
            lnum = if idx < (*wp).w_lines_valid && (*(*wp).w_lines.add(idx as usize)).wl_valid {
                (*(*wp).w_lines.add(idx as usize)).wl_lnum
            } else {
                lnum + 1
            };
        }
        srow += rg.mid_start;

        rg.mid_end = (*wp).w_view_height;
        while idx < (*wp).w_lines_valid {
            let wl = (*wp).w_lines.add(idx as usize);
            if (*wl).wl_valid && (*wl).wl_lnum >= to + 1 {
                // Only update to the first row of this line.
                rg.mid_end = srow;
                break;
            }
            srow += (*wl).wl_size as c_int;
            idx += 1;
        }
    }
}

/// The buffer lines the Visual selection needs redrawn.
///
/// Either the whole selection (its kind changed, or the caller asked for all of
/// it) or just the lines between where the cursor and the anchor were and where
/// they are now.
///
/// # Safety
/// Called with `VIsual_active` and `wp` showing the current buffer.
unsafe fn visual_line_range(wp: *mut win_T, redr_type: c_int) -> (linenr_T, linenr_T) {
    // SAFETY: the caller's window and the global Visual state.
    unsafe {
        let cursor = (*curwin.get()).w_cursor.lnum;
        let anchor = (*VIsual.ptr()).lnum;

        let (mut from, mut to) = if VIsual_mode.get() != (*wp).w_old_visual_mode as c_int
            || redr_type == UPD_INVERTED_ALL
        {
            // The kind of selection changed, or the X selection was gained or
            // lost: redraw all of it, and the lines the cursor moved over.
            let (a, b) = (cursor.min(anchor), cursor.max(anchor));
            (
                a.min((*wp).w_old_cursor_lnum).min((*wp).w_old_visual_lnum),
                b.max((*wp).w_old_cursor_lnum).max((*wp).w_old_visual_lnum),
            )
        } else {
            // Just the lines between the old cursor position and the new one,
            // plus the anchor if it moved.
            let (mut from, to) = if cursor < (*wp).w_old_cursor_lnum {
                (cursor, (*wp).w_old_cursor_lnum)
            } else if (*wp).w_old_cursor_lnum == 0 {
                (cursor, cursor) // Visual mode just started
            } else {
                ((*wp).w_old_cursor_lnum, cursor)
            };
            let mut to = to;
            if anchor != (*wp).w_old_visual_lnum || (*VIsual.ptr()).col != (*wp).w_old_visual_col {
                if (*wp).w_old_visual_lnum < from && (*wp).w_old_visual_lnum != 0 {
                    from = (*wp).w_old_visual_lnum;
                }
                to = to.max((*wp).w_old_visual_lnum).max(anchor);
                from = from.min(anchor);
            }
            (from, to)
        };

        // Blockwise: a changed column or `w_curswant` means every line of the
        // selection has to be redrawn, so the actual columns are computed here.
        if VIsual_mode.get() == Ctrl_V {
            let (fromc, toc) = visual_block_columns(wp);
            if fromc != (*wp).w_old_cursor_fcol || toc != (*wp).w_old_cursor_lcol {
                from = from.min(anchor);
                to = to.max(anchor);
            }
            (*wp).w_old_cursor_fcol = fromc;
            (*wp).w_old_cursor_lcol = toc;
        }

        (from, to)
    }
}

/// The first and last screen columns of a blockwise Visual selection.
///
/// # Safety
/// Called with `VIsual_active` and a blockwise selection.
unsafe fn visual_block_columns(wp: *mut win_T) -> (colnr_T, colnr_T) {
    // SAFETY: the caller's window and the global Visual state.
    unsafe {
        let mut fromc = 0;
        let mut toc = 0;

        // With 'linebreak' the columns are computed as if 'virtualedit' were
        // "all", because that is how the selection is drawn.
        let save_ve_flags = (*curwin.get()).w_onebuf_opt.wo_ve_flags;
        if (*curwin.get()).w_onebuf_opt.wo_lbr != 0 {
            (*curwin.get()).w_onebuf_opt.wo_ve_flags = kOptVeFlagAll;
        }
        getvcols(
            wp,
            VIsual.ptr(),
            &raw mut (*curwin.get()).w_cursor,
            &raw mut fromc,
            &raw mut toc,
        );
        toc += 1;
        (*curwin.get()).w_onebuf_opt.wo_ve_flags = save_ve_flags;

        if (*curwin.get()).w_curswant != MAXCOL as colnr_T {
            return (fromc, toc);
        }

        // `$` in blockwise mode: highlight to the end of every line, unless
        // 'virtualedit' has "block", in which case it stops at the longest one.
        if get_ve_flags(curwin.get()) & kOptVeFlagBlock == 0 {
            return (fromc, MAXCOL as colnr_T);
        }

        let cursor_lnum = (*curwin.get()).w_cursor.lnum;
        let anchor_lnum = (*VIsual.ptr()).lnum;
        let cursor_above = cursor_lnum < anchor_lnum;
        let mut pos = pos_T::default();
        toc = 0;
        let mut lnum = cursor_lnum;
        while if cursor_above {
            lnum <= anchor_lnum
        } else {
            lnum >= anchor_lnum
        } {
            pos.lnum = lnum;
            pos.col = ml_get_buf_len((*wp).w_buffer, lnum);
            let mut end = 0;
            getvvcol(
                wp,
                &raw mut pos,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                &raw mut end,
            );
            toc = toc.max(end);
            lnum += if cursor_above { 1 } else { -1 };
        }
        (fromc, toc + 1)
    }
}

/// Record the Visual selection this redraw drew, so the next one can tell what
/// moved.
///
/// # Safety
/// `wp` must be a live window and `buf` its buffer.
unsafe fn remember_visual_area(wp: *mut win_T, buf: *mut buf_T) {
    // SAFETY: the caller's window and the global Visual state.
    unsafe {
        if VIsual_active.get() && buf == (*curwin.get()).w_buffer {
            (*wp).w_old_visual_mode = VIsual_mode.get() as c_char;
            (*wp).w_old_cursor_lnum = (*curwin.get()).w_cursor.lnum;
            (*wp).w_old_visual_lnum = (*VIsual.ptr()).lnum;
            (*wp).w_old_visual_col = (*VIsual.ptr()).col;
            (*wp).w_old_curswant = (*curwin.get()).w_curswant;
        } else {
            (*wp).w_old_visual_mode = 0;
            (*wp).w_old_cursor_lnum = 0;
            (*wp).w_old_visual_lnum = 0;
            (*wp).w_old_visual_col = 0;
        }
    }
}

/// Report the `ui_watched` extmarks this redraw passed to the UI.
///
/// # Safety
/// `wp` must be the window that was just drawn.
unsafe fn send_win_extmarks(wp: *mut win_T) {
    // SAFETY: the caller's window; the list is filled by this redraw only.
    unsafe {
        win_extmark_arr.with(|marks| {
            for m in marks {
                ui_call_win_extmark(
                    (*wp).w_grid_alloc.handle as Integer,
                    (*wp).handle as Window,
                    m.ns_id as Integer,
                    m.mark_id as Integer,
                    m.win_row as Integer,
                    m.win_col as Integer,
                );
            }
        });
    }
}

/// Publish `w_botline`, and redraw once more if it turned out to be wrong.
///
/// `w_botline` is deliberately approximated between redraws -- keeping it exact
/// would mean a `plines_win` walk on every change -- so this is where the
/// approximation is checked against what was actually drawn. When it was wrong
/// the cursor may be off screen, and the fix is another `win_update`.
///
/// `old_botline` is what `w_botline` held before the line loop replaced it.
///
/// # Safety
/// `wp` must be the window that was just drawn and `buf` its buffer.
unsafe fn finish_botline(
    wp: *mut win_T,
    buf: *mut buf_T,
    old_botline: linenr_T,
    nrwidth_before: c_int,
) {
    // Recursion guard: the second pass must not start a third.
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);

    // SAFETY: the caller's window and buffer.
    unsafe {
        // `dollar_vcol >= 0` means the cursor line is showing a `$` for a change
        // command and was not fully drawn, so its height is not known here.
        if dollar_vcol.get() == -1 || wp != curwin.get() {
            (*wp).w_valid |= VALID_BOTLINE;
            (*wp).w_viewport_invalid = true;
            if wp == curwin.get() && (*wp).w_botline != old_botline && !RECURSIVE.get() {
                RECURSIVE.set(true);
                (*curwin.get()).w_valid &= !VALID_TOPLINE;
                update_topline(curwin.get()); // may invalidate w_botline again
                // A new redraw, either from a moved topline or a reset skipcol.
                if must_redraw.get() != 0 {
                    // Do not update for the buffer changes a second time.
                    let mod_set = (*curbuf.get()).b_mod_set;
                    (*curbuf.get()).b_mod_set = false;
                    curs_columns(curwin.get(), c_int::from(true));
                    win_update(curwin.get());
                    must_redraw.set(0);
                    (*curbuf.get()).b_mod_set = mod_set;
                }
                RECURSIVE.set(false);
            }
        }

        if nrwidth_before != (*wp).w_nrwidth && !(*buf).terminal.is_null() {
            terminal_check_size((*buf).terminal);
        }
    }
}
