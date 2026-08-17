//! Scrolling and the view: the page and half-page commands, 'scrollbind',
//! and the `Z` pair.
//!
//! The `z` prefix tree is [`self::zet`].

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use crate::cursor::set_leftcol;
use crate::diff::diff_set_topline;
use crate::drawscreen::{UPD_VALID, redraw_later};
use crate::ex_docmd::do_cmdline_cmd;
use crate::global_cell::GlobalCell;
use crate::main::{
    VIsual_active, VIsual_select, curbuf, curwin, did_syncbind, firstwin, mod_mask, p_sbo,
};
use crate::normal::{MOD_MASK_CTRL, checkclearop, checkclearopq, clearopbeep};
use crate::plines::plines_m_win_fill;
use crate::strings::vim_strchr;
use crate::types::{Direction, buf_T, cmdarg_T, colnr_T, linenr_T, win_T};
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
pub unsafe fn get_vtopline(wp: *mut win_T) -> c_int {
    // SAFETY: `wp` is a live window.
    unsafe { plines_m_win_fill(wp, 1, (*wp).w_topline) - (*wp).w_topfill }
}

/// After a command that may have scrolled: bring the 'scrollbind' windows
/// along, and remember where this one is for next time.
pub unsafe fn do_check_scrollbind(check: bool) {
    // The previous call's answers. They are what makes this a *difference*
    // rather than an absolute position, so that a window bound to two others
    // does not fight itself.
    static old_curwin: GlobalCell<*mut win_T> = GlobalCell::new(ptr::null_mut());
    static old_vtopline: GlobalCell<linenr_T> = GlobalCell::new(0);
    static old_buf: GlobalCell<*mut buf_T> = GlobalCell::new(ptr::null_mut());
    static old_leftcol: GlobalCell<colnr_T> = GlobalCell::new(0);

    // SAFETY: reads the current window and the remembered previous one.
    unsafe {
        let win = curwin.get();
        let vtopline = get_vtopline(win);
        if check && (*win).w_onebuf_opt.wo_scb != 0 {
            if did_syncbind.get() {
                // `:syncbind` has just set every bound window itself.
                did_syncbind.set(false);
            } else if win == old_curwin.get() {
                if ((*win).w_buffer == old_buf.get() || (*win).w_onebuf_opt.wo_diff != 0)
                    && (vtopline as linenr_T != old_vtopline.get()
                        || (*win).w_leftcol != old_leftcol.get())
                {
                    check_scrollbind(
                        vtopline as linenr_T - old_vtopline.get(),
                        (*win).w_leftcol - old_leftcol.get(),
                    );
                }
            } else if !vim_strchr(p_sbo.get(), 'j' as c_int).is_null() {
                // Just moved into this window, and 'scrollopt' has "jump":
                // bring it back to where the binding says it should be.
                check_scrollbind(vtopline as linenr_T - (*win).w_scbind_pos as linenr_T, 0);
            }
            (*win).w_scbind_pos = vtopline;
        }
        old_curwin.set(win);
        old_vtopline.set(vtopline as linenr_T);
        old_buf.set((*win).w_buffer);
        old_leftcol.set((*win).w_leftcol);
    }
}

/// Scroll every other 'scrollbind' window by the same amount this one just
/// moved.
///
/// Each window is made current in turn, because the scrolling functions work
/// on `curwin`. Any Visual selection is put down for the duration so that
/// nothing extends it.
pub unsafe fn check_scrollbind(vtopline_diff: linenr_T, leftcol_diff: c_int) {
    // SAFETY: walks the current tab page's window list, restoring `curwin`
    // and `curbuf` before returning.
    unsafe {
        let old_curwin = curwin.get();
        let old_curbuf = curbuf.get();
        let old_visual_select = VIsual_select.get();
        let old_visual_active = VIsual_active.get();
        let tgt_leftcol = (*old_curwin).w_leftcol;
        // Two windows in diff mode are always bound vertically; otherwise
        // 'scrollopt' says so.
        let want_ver = (*old_curwin).w_onebuf_opt.wo_diff != 0
            || (!vim_strchr(p_sbo.get(), 'v' as c_int).is_null() && vtopline_diff != 0);
        let want_hor = !vim_strchr(p_sbo.get(), 'h' as c_int).is_null()
            && (leftcol_diff != 0 || vtopline_diff != 0);
        VIsual_active.set(false);
        VIsual_select.set(VIsual_active.get());

        // Upstream asks `curtab == curtab`, so this always walks the current
        // tab page's windows however it reads.
        let mut wp = firstwin.get();
        while !wp.is_null() {
            curwin.set(wp);
            curbuf.set((*wp).w_buffer);
            if wp != old_curwin && (*wp).w_onebuf_opt.wo_scb != 0 {
                if want_ver {
                    if (*old_curwin).w_onebuf_opt.wo_diff != 0 && (*wp).w_onebuf_opt.wo_diff != 0 {
                        diff_set_topline(old_curwin, wp);
                    } else {
                        // The bound position may run past the end of this
                        // window's buffer; the *position* keeps the overshoot
                        // so that scrolling back lines up again.
                        (*wp).w_scbind_pos += vtopline_diff as c_int;
                        let curr_vtopline = get_vtopline(wp);
                        let max_vtopline = curr_vtopline
                            + (*wp).w_topfill
                            + plines_m_win_fill(
                                wp,
                                (*wp).w_topline + 1,
                                (*curbuf.get()).b_ml.ml_line_count,
                            );
                        let new_vtopline = (*wp).w_scbind_pos.min(max_vtopline).max(1);
                        let y = new_vtopline - curr_vtopline;
                        if y > 0 {
                            scrollup(wp, y as linenr_T, false);
                        } else {
                            scrolldown(wp, -(y as linenr_T), false);
                        }
                    }
                    redraw_later(wp, UPD_VALID);
                    cursor_correct(wp);
                    (*wp).w_redr_status = true;
                }
                if want_hor {
                    set_leftcol(tgt_leftcol);
                }
            }
            wp = (*wp).w_next;
        }

        VIsual_select.set(old_visual_select);
        VIsual_active.set(old_visual_active);
        curwin.set(old_curwin);
        curbuf.set(old_curbuf);
    }
}

/// `CTRL-F` and `CTRL-B`: a page forwards or backwards. With CTRL held they
/// are a tab page instead.
pub(crate) unsafe fn nv_page(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearop((*cap).oap) {
            return;
        }
        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            if (*cap).arg == BACKWARD as c_int {
                goto_tabpage(-(*cap).count1);
            } else {
                goto_tabpage((*cap).count0);
            }
        } else {
            pagescroll((*cap).arg as Direction, (*cap).count1, false);
        }
    }
}

/// `CTRL-E` and `CTRL-Y`: scroll one line, leaving the cursor where it is on
/// the screen for as long as it can.
pub unsafe fn nv_scroll_line(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if !checkclearop((*cap).oap) {
            scroll_redraw((*cap).arg, (*cap).count1 as linenr_T);
        }
    }
}

/// `CTRL-D` and `CTRL-U`: half a page.
pub(crate) unsafe fn nv_halfpage(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if !checkclearop((*cap).oap) {
            let dir = if (*cap).cmdchar == Ctrl_D {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            };
            // A count here also sets 'scroll', which `pagescroll` does.
            pagescroll(dir as Direction, (*cap).count0, true);
        }
    }
}

/// `ZZ`, `ZQ` and `ZR`: the two-key ways out.
pub(crate) unsafe fn nv_Zet(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearopq((*cap).oap) {
            return;
        }
        match u8::try_from((*cap).nchar) {
            // Write this file if it changed, then quit.
            Ok(b'Z') => do_cmdline_cmd(c"x".as_ptr()),
            // Quit without writing.
            Ok(b'Q') => do_cmdline_cmd(c"q!".as_ptr()),
            // Restart. A count means "and abandon every other window too".
            Ok(b'R') => {
                if (*cap).count0 >= 1 {
                    do_cmdline_cmd(c"restart +qall!".as_ptr())
                } else {
                    do_cmdline_cmd(c"restart".as_ptr())
                }
            }
            _ => {
                clearopbeep((*cap).oap);
                return;
            }
        };
    }
}
