//! Marking things to be redrawn later.
//!
//! None of this draws: every function sets a flag that [`update_screen`] reads
//! on the next pass through the main loop. [`redraw_later`] is the primitive --
//! it raises one window's `w_redr_type` and the global `MUST_REDRAW` -- and the
//! rest name a scope: all windows ([`redraw_all_later`]), every window on one
//! buffer ([`redraw_buf_later`]), a line range ([`redraw_win_range_later`]), a
//! status line ([`status_redraw_buf`]).
//!
//! [`show_cursor_info_later`] is the one that decides *whether* anything
//! changed: it compares the cursor position, the Visual selection and the
//! recording state against what the status line was last drawn with.

#![deny(unsafe_op_in_unsafe_fn)]
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::normal::{visual_active, visual_anchor, visual_mode};
use crate::option::defs::StlSyntax;
use crate::option::vars::P_WBR;
use crate::optionstr::OptString;
use crate::window::FR_COL;
use crate::window::FR_LEAF;
use crate::window::FR_ROW;
use crate::winlayer::Buf;
use crate::winlayer::FrameRef;
use crate::winlayer::Win;

/// Mark the title and icon for redraw if either of them uses statusline format.
///
/// Answers whether either does.
pub(crate) fn redraw_custom_title_later() -> bool {
    let custom = (p_icon() && STL_SYNTAX.get().has(StlSyntax::ICON))
        || (p_title() && STL_SYNTAX.get().has(StlSyntax::TITLE));
    if custom {
        NEED_MAKETITLE.set(true);
    }
    custom
}

/// Show the current cursor position in the ruler and everywhere else that
/// reports it.
///
/// Nothing is drawn here either: what this decides is whether anything the
/// status line, the window bar or the ruler shows has *changed* since they were
/// last drawn, and if so which of them to mark. `force` reports unconditionally.
pub fn show_cursor_info_later(force: bool) {
    let mut wp = Win::current();
    let state = get_real_state();
    // "The cursor is on an empty line" is a status-line item of its own, and
    // in Insert mode it is deliberately always reported as false.
    let empty_line =
        STATE.get() & MODE_INSERT == 0 && wp.buffer().lines().line(wp.w_cursor.lnum).is_empty();

    validate_virtcol(wp);

    let visual_moved = visual_active()
        && (visual_mode().raw() != wp.w_stl_visual_mode || visual_anchor() != wp.w_stl_visual_pos);
    if force
        || wp.w_cursor != wp.w_stl_cursor
        || wp.w_virtcol != wp.w_stl_virtcol
        || wp.w_topline != wp.w_stl_topline
        || wp.w_buffer.b_ml.ml_line_count != wp.w_stl_line_count
        || wp.w_topfill != wp.w_stl_topfill
        || empty_line != wp.w_stl_empty
        || REG_RECORDING.get() != wp.w_stl_recording
        || state != wp.w_stl_state
        || visual_moved
    {
        if wp.w_status_height != 0 || global_stl_height() != 0 {
            wp.w_redr_status = true;
        } else {
            REDRAW_CMDLINE.set(true);
        }
        // A window bar can show the same items, and it is never on the
        // command line, so it needs the status-line treatment either way.
        if P_WBR.first_byte() != 0 || wp.w_onebuf_opt.wo_wbr.first_byte() != 0 {
            wp.w_redr_status = true;
        }
        redraw_custom_title_later();
    }

    wp.w_stl_cursor = wp.w_cursor;
    wp.w_stl_virtcol = wp.w_virtcol;
    wp.w_stl_empty = empty_line;
    wp.w_stl_topline = wp.w_topline;
    wp.w_stl_line_count = wp.w_buffer.b_ml.ml_line_count;
    wp.w_stl_topfill = wp.w_topfill;
    wp.w_stl_recording = REG_RECORDING.get().byte();
    wp.w_stl_state = state;
    // Upstream leaves the remembered Visual position alone when Visual mode
    // is not active, so that leaving and re-entering it on the same
    // selection does not count as a change. Reproduced.
    if visual_active() {
        wp.w_stl_visual_mode = visual_mode().raw();
        wp.w_stl_visual_pos = visual_anchor();
    }
}

/// Redraw window `window` later, with `w_redr_type` at least `redr_type`.
///
/// `MUST_REDRAW` is the maximum over all windows, so it only ever rises here;
/// [`update_screen`] resets it.
pub fn redraw_later(mut window: Win, redr_type: c_int) {
    if EXITING.get() || REDRAW_NOT_ALLOWED.get() {
        return;
    }
    if window.w_redr_type < redr_type {
        window.w_redr_type = redr_type;
        if redr_type >= UPD_NOT_VALID {
            window.w_lines_valid = 0;
        }
        set_must_redraw_unchecked(redr_type);
    }
}

/// Mark every window of the current tab page for redraw.
pub fn redraw_all_later(redr_type: c_int) {
    for wp in winlayer::windows() {
        redraw_later(wp, redr_type);
    }
    // Needed as well when switching tab pages: the windows marked above are
    // not the ones that will be drawn.
    set_must_redraw(redr_type);
}

/// Raise `MUST_REDRAW` to `redr_type`, unless redrawing is currently forbidden.
pub fn set_must_redraw(redr_type: c_int) {
    if !REDRAW_NOT_ALLOWED.get() {
        set_must_redraw_unchecked(redr_type);
    }
}

/// [`set_must_redraw`] without the `REDRAW_NOT_ALLOWED` test, for callers that
/// have already made it.
fn set_must_redraw_unchecked(redr_type: c_int) {
    MUST_REDRAW.set(MUST_REDRAW.get().max(redr_type));
}

/// Drop every window's cached attribute state; used when the highlight tables
/// are rebuilt.
pub fn screen_invalidate_highlights() {
    for mut wp in winlayer::windows() {
        redraw_later(wp, UPD_NOT_VALID);
        wp.w_grid_alloc.valid = false;
    }
}

/// Mark every window showing the current buffer.
///
/// Safe: the only promise is that the editor exists, which `CURBUF` carries
/// from startup to exit.
pub fn redraw_curbuf_later(redr_type: c_int) {
    redraw_buf_later(Buf::current(), redr_type)
}

/// Mark every window showing `buffer`.
pub fn redraw_buf_later(buffer: Buf, redr_type: c_int) {
    for wp in winlayer::windows() {
        if wp.w_buffer == buffer {
            redraw_later(wp, redr_type);
        }
    }
}

/// Mark line `line` of `buffer` in every window showing it.
///
/// `force` also marks a line *past* the end of the buffer, which is how a
/// deletion gets the rows it used to occupy redrawn.
pub fn redraw_buf_line_later(buffer: Buf, line: LineNr, force: bool) {
    for mut wp in winlayer::windows() {
        if wp.w_buffer == buffer {
            redraw_win_line(wp, line.min(buffer.b_ml.ml_line_count));
            if force && line > buffer.b_ml.ml_line_count {
                wp.w_redraw_bot = line;
            }
        }
    }
}

/// Widen window `window`'s pending redraw range to cover lines `first..=last`.
///
/// Nothing is marked when the range is entirely outside the window.
pub fn redraw_win_range_later(window: Win, first: LineNr, last: LineNr) {
    let mut win = window;
    if last >= win.w_topline && first < win.w_botline {
        if win.w_redraw_top == LineNr::ZERO || win.w_redraw_top > first {
            win.w_redraw_top = first;
        }
        if win.w_redraw_bot == LineNr::ZERO || win.w_redraw_bot < last {
            win.w_redraw_bot = last;
        }
        redraw_later(window, UPD_VALID);
    }
}

/// Mark one line of window `window`.
///
/// Inserting or deleting lines invalidates the range this widens, so a caller
/// that does either has to mark the whole window instead.
pub fn redraw_win_line(window: Win, lnum: LineNr) {
    redraw_win_range_later(window, lnum, lnum)
}

/// Mark lines `first..=last` of `buffer` in every window showing it.
pub fn redraw_buf_range_later(buffer: Buf, first: LineNr, last: LineNr) {
    for wp in winlayer::windows() {
        if wp.w_buffer == buffer {
            redraw_win_range_later(wp, first, last);
        }
    }
}

/// Mark the status lines and window bars of every window showing `buffer`.
pub fn redraw_buf_status_later(buffer: Buf) {
    for mut wp in winlayer::windows() {
        if wp.w_buffer == buffer
            && (wp.w_status_height != 0
                || (wp.is_current() && global_stl_height() != 0)
                || wp.w_winbar_height != 0)
        {
            wp.w_redr_status = true;
            set_must_redraw(UPD_VALID);
        }
    }
}

/// Mark every status line and window bar; used after the first `:cd`.
pub fn status_redraw_all() {
    let is_stl_global = global_stl_height() != 0;
    for mut wp in winlayer::windows() {
        if (!is_stl_global && wp.w_status_height != 0) || wp.is_current() || wp.w_winbar_height != 0
        {
            wp.w_redr_status = true;
            redraw_later(wp, UPD_VALID);
        }
    }
}

/// Mark the status lines and window bars of the current buffer.
pub fn status_redraw_curbuf() {
    status_redraw_buf(Buf::current())
}

/// Mark the status lines and window bars of `buffer`.
pub fn status_redraw_buf(buffer: Buf) {
    let is_stl_global = global_stl_height() != 0;
    for mut wp in winlayer::windows() {
        if wp.w_buffer == buffer
            && ((!is_stl_global && wp.w_status_height != 0)
                || (is_stl_global && wp.is_current())
                || wp.w_winbar_height != 0)
        {
            wp.w_redr_status = true;
            redraw_later(wp, UPD_VALID);
        }
    }
    // With no status line at all the ruler lives on the command line, so it
    // has to be marked separately -- but only if the loop above did not
    // already mark the current window.
    let wp = Win::current();
    if p_ru() && wp.w_status_height == 0 && !wp.w_redr_status {
        REDRAW_CMDLINE.set(true);
        redraw_later(wp, UPD_VALID);
    }
}

/// Draw every status line and window bar that is marked, plus the tab line and
/// the title.
pub fn redraw_statuslines() {
    for wp in winlayer::windows() {
        if wp.w_redr_status {
            win_check_ns_hl(Some(wp));
            win_redr_winbar(wp);
            win_redr_status(wp);
        }
    }
    win_check_ns_hl(None);

    if REDRAW_TABLINE.get() {
        draw_tabline();
    }
    if NEED_MAKETITLE.get() {
        maketitle();
    }
}

impl FrameRef {
    /// Mark the status lines at the bottom of this frame.
    ///
    /// One per column of a row frame; the last one of a column frame.
    pub fn redraw_last_status(self) {
        match c_int::from(self.fr_layout) {
            FR_LEAF => {
                let mut win = self.win().expect("a leaf frame holds a window");
                win.w_redr_status = true;
            }
            FR_ROW => {
                for child in self.children() {
                    child.redraw_last_status();
                }
            }
            layout => {
                debug_assert!(layout == FR_COL, "frp->fr_layout == FR_COL");
                let last = self.children().last().expect("a column frame has a child");
                last.redraw_last_status();
            }
        }
    }
}

/// The redraw-level arithmetic `update_screen` is fed by: which request
/// raises a window's `w_redr_type` and the global `MUST_REDRAW`, which is
/// swallowed, and how a line range widens. Driven on the fixture's bare
/// current window, which is in no window list, so only the per-window
/// entry points are reachable; `MUST_REDRAW` and the window's fields are put
/// back as they were found.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_fixture::Fixture;

    /// The fixture, with `MUST_REDRAW` and the window's redraw state zeroed
    /// for the case and put back when it goes.
    #[derive(Debug)]
    struct Redraw {
        fx: Fixture,
        saved_must_redraw: c_int,
    }

    impl Redraw {
        fn new() -> Redraw {
            let fx = Fixture::new();
            let saved_must_redraw = MUST_REDRAW.replace(0);
            let mut win = Win::current();
            win.w_redr_type = 0;
            win.w_lines_valid = 0;
            win.w_redraw_top = LineNr::ZERO;
            win.w_redraw_bot = LineNr::ZERO;
            Redraw {
                fx,
                saved_must_redraw,
            }
        }

        fn win(&self) -> Win {
            let _ = &self.fx;
            Win::current()
        }
    }

    impl Drop for Redraw {
        fn drop(&mut self) {
            MUST_REDRAW.set(self.saved_must_redraw);
            REDRAW_NOT_ALLOWED.set(false);
            EXITING.set(false);
        }
    }

    #[test]
    fn redraw_later_raises_the_window_and_must_redraw_together() {
        let t = Redraw::new();
        redraw_later(t.win(), UPD_VALID);
        assert_eq!(
            (t.win().w_redr_type, MUST_REDRAW.get()),
            (UPD_VALID, UPD_VALID)
        );
        redraw_later(t.win(), UPD_INVERTED);
        assert_eq!(
            (t.win().w_redr_type, MUST_REDRAW.get()),
            (UPD_INVERTED, UPD_INVERTED)
        );
    }

    #[test]
    fn a_lower_request_than_the_window_has_is_swallowed_whole() {
        // `MUST_REDRAW` is raised only when the *window's* level rises: a
        // window already at NOT_VALID leaves a zero `MUST_REDRAW` at zero.
        let t = Redraw::new();
        let mut win = t.win();
        win.w_redr_type = UPD_NOT_VALID;
        redraw_later(win, UPD_VALID);
        redraw_later(win, UPD_NOT_VALID);
        assert_eq!((win.w_redr_type, MUST_REDRAW.get()), (UPD_NOT_VALID, 0));
    }

    #[test]
    fn must_redraw_never_falls_below_another_windows_request() {
        let t = Redraw::new();
        MUST_REDRAW.set(UPD_CLEAR);
        redraw_later(t.win(), UPD_SOME_VALID);
        assert_eq!(t.win().w_redr_type, UPD_SOME_VALID);
        assert_eq!(MUST_REDRAW.get(), UPD_CLEAR);
    }

    #[test]
    fn not_valid_and_above_forget_the_windows_valid_lines() {
        let t = Redraw::new();
        let mut win = t.win();
        win.w_lines_valid = 5;
        redraw_later(win, UPD_SOME_VALID);
        assert_eq!(win.w_lines_valid, 5, "SOME_VALID keeps them");
        redraw_later(win, UPD_NOT_VALID);
        assert_eq!(win.w_lines_valid, 0);
        win.w_lines_valid = 5;
        redraw_later(win, UPD_CLEAR);
        assert_eq!(win.w_lines_valid, 0);
    }

    #[test]
    fn exiting_or_redraw_not_allowed_drops_the_request() {
        let t = Redraw::new();
        REDRAW_NOT_ALLOWED.set(true);
        redraw_later(t.win(), UPD_CLEAR);
        set_must_redraw(UPD_CLEAR);
        assert_eq!((t.win().w_redr_type, MUST_REDRAW.get()), (0, 0));
        REDRAW_NOT_ALLOWED.set(false);
        EXITING.set(true);
        redraw_later(t.win(), UPD_CLEAR);
        assert_eq!(t.win().w_redr_type, 0);
        // `set_must_redraw` asks only about `REDRAW_NOT_ALLOWED`.
        set_must_redraw(UPD_VALID);
        assert_eq!(MUST_REDRAW.get(), UPD_VALID);
    }

    #[test]
    fn set_must_redraw_keeps_the_maximum() {
        let _t = Redraw::new();
        set_must_redraw(UPD_NOT_VALID);
        set_must_redraw(UPD_VALID);
        assert_eq!(MUST_REDRAW.get(), UPD_NOT_VALID);
        set_must_redraw(UPD_CLEAR);
        assert_eq!(MUST_REDRAW.get(), UPD_CLEAR);
    }

    #[test]
    fn a_line_range_widens_the_pending_lines_and_asks_for_valid() {
        let t = Redraw::new();
        let mut win = t.win();
        win.w_topline = LineNr::new(10);
        win.w_botline = LineNr::new(30); // the first line below the window
        let range = |w: Win| (w.w_redraw_top.get(), w.w_redraw_bot.get());

        redraw_win_range_later(win, LineNr::new(15), LineNr::new(17));
        assert_eq!(range(win), (15, 17));
        assert_eq!((win.w_redr_type, MUST_REDRAW.get()), (UPD_VALID, UPD_VALID));
        redraw_win_range_later(win, LineNr::new(12), LineNr::new(16));
        assert_eq!(range(win), (12, 17), "the top moves up, the bottom stays");
        redraw_win_line(win, LineNr::new(20));
        assert_eq!(range(win), (12, 20));
        // A range that starts above the window but reaches into it counts.
        redraw_win_range_later(win, LineNr::new(1), LineNr::new(10));
        assert_eq!(range(win), (1, 20));
    }

    #[test]
    fn a_line_range_outside_the_window_marks_nothing() {
        let t = Redraw::new();
        let mut win = t.win();
        win.w_topline = LineNr::new(10);
        win.w_botline = LineNr::new(30);
        // Ends just above the top line; starts on the line below the window.
        redraw_win_range_later(win, LineNr::new(1), LineNr::new(9));
        redraw_win_range_later(win, LineNr::new(30), LineNr::new(40));
        assert_eq!(
            (win.w_redraw_top, win.w_redraw_bot),
            (LineNr::ZERO, LineNr::ZERO)
        );
        assert_eq!((win.w_redr_type, MUST_REDRAW.get()), (0, 0));
    }
}
