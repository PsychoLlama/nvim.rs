//! Scrolling and the view: the page and half-page commands, 'scrollbind',
//! and the `Z` pair.
//!
//! The `z` prefix tree is [`self::zet`].

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::keycodes::ModMask;
use crate::types::Key;
use crate::winlayer::BufId;
use crate::winlayer::WinId;
use crate::winlayer::{Buf, Win, windows};

use crate::cursor::set_leftcol;
use crate::diff::diff_set_topline;
use crate::drawscreen::{UPD_VALID, redraw_later};
use crate::ex_docmd::do_cmdline_cmd;
use crate::getchar::state::mod_mask;
use crate::global_cell::GlobalCell;
use crate::normal::{
    check_clear_op, check_clear_op_quit, clear_op_beep, set_visual_active, set_visual_select,
    visual_active, visual_select,
};
use crate::option::vars::p_sbo;
use crate::plines::plines_m_win_fill;
use crate::state::mode::did_syncbind;
use crate::types::{CmdArg, ColNr, Direction, LineNr};
use crate::window::goto_tabpage;
use core::ffi::c_int;

use crate::r#move::{cursor_correct, pagescroll, scroll_redraw, scrolldown, scrollup};
use crate::search::{BACKWARD, FORWARD};

mod zet;
pub(crate) use self::zet::*;

/// The window's top line counted in *screen* rows from the start of the
/// buffer, which is what 'scrollbind' has to keep equal between windows: two
/// windows showing the same buffer at different widths wrap it differently,
/// so buffer line numbers would not line up.
pub(crate) fn get_vtopline(window: Win) -> c_int {
    plines_m_win_fill(window, LineNr::ONE, window.w_topline) - window.w_topfill
}

/// After a command that may have scrolled: bring the 'scrollbind' windows
/// along, and remember where this one is for next time.
pub(crate) fn do_check_scrollbind(check: bool) {
    // The previous call's answers. They are what makes this a *difference*
    // rather than an absolute position, so that a window bound to two others
    // does not fight itself.
    static OLD_CURWIN: GlobalCell<Option<WinId>> = GlobalCell::new(None);
    static OLD_VTOPLINE: GlobalCell<LineNr> = GlobalCell::new(LineNr::ZERO);
    static OLD_BUF: GlobalCell<Option<BufId>> = GlobalCell::new(None);
    static OLD_LEFTCOL: GlobalCell<ColNr> = GlobalCell::new(ColNr::ZERO);

    let mut win = Win::current();
    let vtopline = get_vtopline(win);
    if check && win.w_onebuf_opt.wo_scb != 0 {
        if did_syncbind.get() {
            // `:syncbind` has just set every bound window itself.
            did_syncbind.set(false);
        } else if OLD_CURWIN.get() == Some(win.id()) {
            if (win.buffer().try_id() == OLD_BUF.get() || win.w_onebuf_opt.wo_diff != 0)
                && (LineNr::new(vtopline) != OLD_VTOPLINE.get()
                    || win.w_leftcol != OLD_LEFTCOL.get())
            {
                let down = LineNr::new(vtopline) - OLD_VTOPLINE.get();
                check_scrollbind(down, win.w_leftcol - OLD_LEFTCOL.get());
            }
        } else if p_sbo(|value| value.to_bytes().contains(&b'j')) {
            // Just moved into this window, and 'scrollopt' has "jump":
            // bring it back to where the binding says it should be.
            check_scrollbind((LineNr::new(vtopline) - win.w_scbind_pos).get(), 0);
        }
        win.w_scbind_pos = vtopline;
    }
    OLD_CURWIN.set(Some(win.id()));
    OLD_VTOPLINE.set(LineNr::new(vtopline));
    OLD_BUF.set(win.buffer().try_id());
    OLD_LEFTCOL.set(win.w_leftcol);
}

/// Scroll every other 'scrollbind' window by the same amount this one just
/// moved.
///
/// Each window is made current in turn, because the scrolling functions work
/// on `curwin`. Any Visual selection is put down for the duration so that
/// nothing extends it.
pub(crate) fn check_scrollbind(vtopline_diff: i32, leftcol_diff: c_int) {
    let (old_curwin, old_curbuf) = (Win::current(), Buf::current());
    let old_visual_select = visual_select();
    let old_visual_active = visual_active();
    let tgt_leftcol = old_curwin.w_leftcol;
    // Two windows in diff mode are always bound vertically; otherwise
    // 'scrollopt' says so.
    let want_ver = old_curwin.w_onebuf_opt.wo_diff != 0
        || (p_sbo(|value| value.to_bytes().contains(&b'v')) && vtopline_diff != 0);
    let want_hor = p_sbo(|value| value.to_bytes().contains(&b'h'))
        && (leftcol_diff != 0 || vtopline_diff != 0);
    set_visual_active(false);
    set_visual_select(visual_active());

    // Upstream asks `curtab == curtab`, so this always walks the current
    // tab page's windows however it reads. Nothing in the body can free one.
    for mut win in windows() {
        win.make_current();
        win.buffer().make_current();
        if win != old_curwin && win.w_onebuf_opt.wo_scb != 0 {
            if want_ver {
                if old_curwin.w_onebuf_opt.wo_diff != 0 && win.w_onebuf_opt.wo_diff != 0 {
                    // Both windows are live for this walk.
                    diff_set_topline(old_curwin, win);
                } else {
                    // The bound position may run past the end of this
                    // window's buffer; the *position* keeps the overshoot
                    // so that scrolling back lines up again.
                    win.w_scbind_pos += vtopline_diff as c_int;
                    let curr_vtopline = get_vtopline(win);
                    let last = Buf::current().b_ml.ml_line_count;
                    let filled = plines_m_win_fill(win, win.w_topline + 1, last);
                    let max_vtopline = curr_vtopline + win.w_topfill + filled;
                    let new_vtopline = win.w_scbind_pos.min(max_vtopline).max(1);
                    let y = new_vtopline - curr_vtopline;
                    if y > 0 {
                        scrollup(win, y, false);
                    } else {
                        scrolldown(win, -y, false);
                    }
                }
                redraw_later(win, UPD_VALID);
                cursor_correct(win);
                win.w_redr_status = true;
            }
            if want_hor {
                set_leftcol(tgt_leftcol);
            }
        }
    }

    set_visual_select(old_visual_select);
    set_visual_active(old_visual_active);
    old_curwin.make_current();
    old_curbuf.make_current();
}

/// `CTRL-F` and `CTRL-B`: a page forwards or backwards. With CTRL held they
/// are a tab page instead.
pub(crate) fn nv_page(cmd_arg: &mut CmdArg) {
    if check_clear_op(cmd_arg.op()) {
        return;
    }
    if mod_mask.get().has(ModMask::CTRL) {
        if cmd_arg.arg == BACKWARD as c_int {
            goto_tabpage(-cmd_arg.count1);
        } else {
            goto_tabpage(cmd_arg.count0);
        }
    } else {
        pagescroll(cmd_arg.arg as Direction, cmd_arg.count1, false);
    }
}

/// `CTRL-E` and `CTRL-Y`: scroll one line, leaving the cursor where it is on
/// the screen for as long as it can.
pub(crate) fn nv_scroll_line(cmd_arg: &mut CmdArg) {
    if !check_clear_op(cmd_arg.op()) {
        scroll_redraw(cmd_arg.arg, cmd_arg.count1);
    }
}

/// `CTRL-D` and `CTRL-U`: half a page.
pub(crate) fn nv_halfpage(cmd_arg: &mut CmdArg) {
    if !check_clear_op(cmd_arg.op()) {
        let dir = if cmd_arg.cmdchar == Key::CTRL_D {
            FORWARD as c_int
        } else {
            BACKWARD as c_int
        };
        // A count here also sets 'scroll', which `pagescroll` does.
        pagescroll(dir as Direction, cmd_arg.count0, true);
    }
}

/// `ZZ`, `ZQ` and `ZR`: the two-key ways out.
pub(crate) fn nv_exit_command(cmd_arg: &mut CmdArg) {
    if check_clear_op_quit(cmd_arg.op()) {
        return;
    }
    let cmd = match cmd_arg.nchar.as_byte() {
        // Write this file if it changed, then quit.
        Some(b'Z') => c"x",
        // Quit without writing.
        Some(b'Q') => c"q!",
        // Restart, restoring the session. A count skips the session; a
        // count of 9 also abandons unsaved changes.
        Some(b'R') if (1..=8).contains(&cmd_arg.count0) => c"restart!",
        Some(b'R') if cmd_arg.count0 == 9 => c"restart! +qall!",
        Some(b'R') => c"restart",
        _ => {
            clear_op_beep(cmd_arg.op());
            return;
        }
    };
    let _ = do_cmdline_cmd(cmd);
}
