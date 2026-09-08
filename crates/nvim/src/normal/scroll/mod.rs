//! Scrolling and the view: the page and half-page commands, 'scrollbind',
//! and the `Z` pair.
//!
//! The `z` prefix tree is [`self::zet`].

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::keycodes::ModMask;
use crate::strings::has_char;
use crate::winlayer::{Buf, Win, windows};
use core::ptr;

use crate::cursor::set_leftcol;
use crate::diff::diff_set_topline;
use crate::drawscreen::{UPD_VALID, redraw_later};
use crate::ex_docmd::do_cmdline_cmd;
use crate::getchar::state::mod_mask;
use crate::global_cell::GlobalCell;
use crate::normal::{
    CmdArgRef, check_clear_op, check_clear_op_quit, clear_op_beep, set_visual_active,
    set_visual_select, visual_active, visual_select,
};
use crate::option::vars::p_sbo;
use crate::plines::plines_m_win_fill;
use crate::state::mode::did_syncbind;
use crate::types::{Buffer, CmdArg, ColNr, Direction, LineNr, Window};
use crate::window::goto_tabpage;
use core::ffi::c_int;

use crate::keycodes::Ctrl_D;
use crate::r#move::{cursor_correct, pagescroll, scroll_redraw, scrolldown, scrollup};
use crate::search::{BACKWARD, FORWARD};

mod zet;
pub(crate) use self::zet::*;

/// The window's top line counted in *screen* rows from the start of the
/// buffer, which is what 'scrollbind' has to keep equal between windows: two
/// windows showing the same buffer at different widths wrap it differently,
/// so buffer line numbers would not line up.
pub(crate) fn get_vtopline(window: Win) -> c_int {
    // SAFETY: a live window, by `Win`'s contract, which is the whole of what
    // `plines_m_win_fill` asks for.
    unsafe { plines_m_win_fill(window, 1, window.w_topline) - window.w_topfill }
}

/// After a command that may have scrolled: bring the 'scrollbind' windows
/// along, and remember where this one is for next time.
pub(crate) fn do_check_scrollbind(check: bool) {
    // The previous call's answers. They are what makes this a *difference*
    // rather than an absolute position, so that a window bound to two others
    // does not fight itself.
    static old_curwin: GlobalCell<*mut Window> = GlobalCell::new(ptr::null_mut());
    static old_vtopline: GlobalCell<LineNr> = GlobalCell::new(0);
    static old_buf: GlobalCell<*mut Buffer> = GlobalCell::new(ptr::null_mut());
    static old_leftcol: GlobalCell<ColNr> = GlobalCell::new(0);

    // SAFETY: reads the current window and the remembered previous one.
    let mut win = Win::current();
    let vtopline = get_vtopline(win);
    if check && win.w_onebuf_opt.wo_scb != 0 {
        if did_syncbind.get() {
            // `:syncbind` has just set every bound window itself.
            did_syncbind.set(false);
        } else if win.raw() == old_curwin.get() {
            if (win.w_buffer == old_buf.get() || win.w_onebuf_opt.wo_diff != 0)
                && (vtopline as LineNr != old_vtopline.get() || win.w_leftcol != old_leftcol.get())
            {
                let down = vtopline as LineNr - old_vtopline.get();
                check_scrollbind(down, win.w_leftcol - old_leftcol.get());
            }
        } else if has_char(unsafe { cstr::at(p_sbo.get()) }, 'j' as c_int) {
            // Just moved into this window, and 'scrollopt' has "jump":
            // bring it back to where the binding says it should be.
            check_scrollbind(vtopline as LineNr - win.w_scbind_pos as LineNr, 0);
        }
        win.w_scbind_pos = vtopline;
    }
    old_curwin.set(win.raw());
    old_vtopline.set(vtopline as LineNr);
    old_buf.set(win.w_buffer);
    old_leftcol.set(win.w_leftcol);
}

/// Scroll every other 'scrollbind' window by the same amount this one just
/// moved.
///
/// Each window is made current in turn, because the scrolling functions work
/// on `curwin`. Any Visual selection is put down for the duration so that
/// nothing extends it.
pub(crate) fn check_scrollbind(vtopline_diff: LineNr, leftcol_diff: c_int) {
    // SAFETY (throughout): walks the current tab page's window list, restoring `curwin`
    // and `curbuf` before returning.
    let (old_curwin, old_curbuf) = (Win::current(), Buf::current());
    let old_visual_select = visual_select();
    let old_visual_active = visual_active();
    let tgt_leftcol = old_curwin.w_leftcol;
    // Two windows in diff mode are always bound vertically; otherwise
    // 'scrollopt' says so.
    let want_ver = old_curwin.w_onebuf_opt.wo_diff != 0
        || (has_char(unsafe { cstr::at(p_sbo.get()) }, 'v' as c_int) && vtopline_diff != 0);
    let want_hor = has_char(unsafe { cstr::at(p_sbo.get()) }, 'h' as c_int)
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
                    // SAFETY: a live window of the walk above.
                    let curr_vtopline = get_vtopline(win);
                    let last = Buf::current().b_ml.ml_line_count;
                    let filled = unsafe { plines_m_win_fill(win, win.w_topline + 1, last) };
                    let max_vtopline = curr_vtopline + win.w_topfill + filled;
                    let new_vtopline = win.w_scbind_pos.min(max_vtopline).max(1);
                    let y = new_vtopline - curr_vtopline;
                    if y > 0 {
                        scrollup(win, y as LineNr, false);
                    } else {
                        scrolldown(win, -(y as LineNr), false);
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
///
/// # Safety
///
/// `cmd_arg` must point at the command's `CmdArg`, unaliased for the call.
pub(crate) unsafe fn nv_page(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    if check_clear_op(ca.op()) {
        return;
    }
    if mod_mask.get().has(ModMask::CTRL) {
        if ca.arg == BACKWARD as c_int {
            goto_tabpage(-ca.count1);
        } else {
            goto_tabpage(ca.count0);
        }
    } else {
        unsafe { pagescroll(ca.arg as Direction, ca.count1, false) };
    }
}

/// `CTRL-E` and `CTRL-Y`: scroll one line, leaving the cursor where it is on
/// the screen for as long as it can.
///
/// # Safety
///
/// `cmd_arg` must point at the command's `CmdArg`, unaliased for the call.
pub(crate) unsafe fn nv_scroll_line(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    if !check_clear_op(ca.op()) {
        scroll_redraw(ca.arg, ca.count1 as LineNr);
    }
}

/// `CTRL-D` and `CTRL-U`: half a page.
///
/// # Safety
///
/// `cmd_arg` must point at the command's `CmdArg`, unaliased for the call.
pub(crate) unsafe fn nv_halfpage(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    if !check_clear_op(ca.op()) {
        let dir = if ca.cmdchar == Ctrl_D {
            FORWARD as c_int
        } else {
            BACKWARD as c_int
        };
        // A count here also sets 'scroll', which `pagescroll` does.
        unsafe { pagescroll(dir as Direction, ca.count0, true) };
    }
}

/// `ZZ`, `ZQ` and `ZR`: the two-key ways out.
///
/// # Safety
///
/// `cmd_arg` must point at the command's `CmdArg`, unaliased for the call.
pub(crate) unsafe fn nv_exit_command(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    if check_clear_op_quit(ca.op()) {
        return;
    }
    let cmd = match u8::try_from(ca.nchar) {
        // Write this file if it changed, then quit.
        Ok(b'Z') => c"x",
        // Quit without writing.
        Ok(b'Q') => c"q!",
        // Restart. A count means "and abandon every other window too".
        Ok(b'R') if ca.count0 >= 1 => c"restart +qall!",
        Ok(b'R') => c"restart",
        _ => {
            clear_op_beep(ca.op());
            return;
        }
    };
    let _ = unsafe { do_cmdline_cmd(cmd.as_ptr()) };
}
