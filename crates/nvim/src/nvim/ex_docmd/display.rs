//! Commands about what is on the screen rather than in the buffer:
//! redrawing, `:redir`, highlighting and the digraph table.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::buffer::maketitle;
use crate::src::nvim::charset::skipwhite;
use crate::src::nvim::digraph::{listdigraphs, putdigraph};
use crate::src::nvim::drawscreen::{
    UPD_INVERTED, UPD_NOT_VALID, UPD_SOME_VALID, redraw_all_later, redraw_curbuf_later,
    redraw_statuslines, status_redraw_all, status_redraw_curbuf, update_screen,
};
use crate::src::nvim::eval::eval_to_string;
use crate::src::nvim::eval::vars::{set_vim_var_nr, var_redir_start, var_redir_stop};
use crate::src::nvim::ex_docmd::argopt::open_exfile;
use crate::src::nvim::ex_docmd::{FAIL, NUL, OK, VV_HLSEARCH, ex_pressedreturn};
use crate::src::nvim::highlight_group::{do_highlight, load_colors};
use crate::src::nvim::main::{
    RedrawingDisabled, State, VIsual_active, cmdpreview, curwin, e_invarg2, emsg_off, msg_col,
    msg_didout, need_maketitle, need_wait_return, no_hlsearch, p_hls, p_lz, redir_fd, redir_off,
    redir_reg, redir_vname, redraw_cmdline,
};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::message::{msg, msg_ext_set_kind, semsg};
use crate::src::nvim::r#move::{update_topline, validate_cursor};
use crate::src::nvim::os::env::expand_env_save;
use crate::src::nvim::os::libc::{fclose, gettext, strcasecmp};
use crate::src::nvim::register::{valid_yank_reg, write_reg_contents};
use crate::src::nvim::state::MODE_CMDLINE;
use crate::src::nvim::statusline::draw_tabline;
use crate::src::nvim::types::{FILE, exarg_T, ssize_t, uint8_t, varnumber_T};
use crate::src::nvim::ui::ui_flush;

/// `:colorscheme` — with no argument, report `g:colors_name`.
pub(crate) unsafe fn ex_colorscheme(eap: *mut exarg_T) {
    unsafe {
        if *(*eap).arg as c_int != NUL {
            if load_colors((*eap).arg) == FAIL {
                semsg(
                    gettext(c"E185: Cannot find color scheme '%s'".as_ptr()),
                    (*eap).arg,
                );
            }
            return;
        }
        // The variable may not exist, which is not an error here: an
        // unnamed scheme reports `default`.
        let expr = xstrdup(c"g:colors_name".as_ptr());
        *emsg_off.ptr() += 1;
        let name = eval_to_string(expr, false, false);
        *emsg_off.ptr() -= 1;
        xfree(expr as *mut c_void);

        msg_ext_set_kind(c"list_cmd".as_ptr());
        if name.is_null() {
            msg(c"default".as_ptr(), 0);
        } else {
            msg(name, 0);
            xfree(name as *mut c_void);
        }
    }
}

/// `:highlight`, and the greeting `:hi!` prints on its own.
pub(crate) unsafe fn ex_highlight(eap: *mut exarg_T) {
    unsafe {
        if *(*eap).arg as c_int == NUL && *(*eap).cmd.add(2) as c_int == '!' as c_int {
            msg(gettext(c"Greetings, Vim user!".as_ptr()), 0);
        }
        do_highlight((*eap).arg, (*eap).forceit != 0, false);
    }
}

/// `:redir` — send message output to a file, a register or a variable
/// until `:redir END`.
///
/// Only one destination at a time: every form closes whatever was open
/// first.
pub(crate) unsafe fn ex_redir(eap: *mut exarg_T) {
    unsafe {
        let mut arg = (*eap).arg;
        if strcasecmp((*eap).arg, c"END".as_ptr() as *mut c_char) == 0 {
            close_redir();
        } else if *arg as c_int == '>' as c_int {
            // `:redir > file` truncates, `:redir >> file` appends.
            arg = arg.add(1);
            let mode = if *arg as c_int == '>' as c_int {
                arg = arg.add(1);
                c"a".as_ptr() as *mut c_char
            } else {
                c"w".as_ptr() as *mut c_char
            };
            arg = skipwhite(arg);
            close_redir();
            let fname = expand_env_save(arg);
            if fname.is_null() {
                return;
            }
            redir_fd.set(open_exfile(fname, (*eap).forceit, mode));
            xfree(fname as *mut c_void);
        } else if *arg as c_int == '@' as c_int {
            close_redir();
            arg = arg.add(1);
            if valid_yank_reg(*arg as c_int, true) && *arg as c_int != '_' as c_int {
                redir_reg.set(*arg as uint8_t as c_int);
                arg = arg.add(1);
                if *arg as c_int == '>' as c_int && *arg.add(1) as c_int == '>' as c_int {
                    // `:redir @a>>` appends.
                    arg = arg.add(2);
                } else {
                    if *arg as c_int == '>' as c_int {
                        arg = arg.add(1);
                    }
                    // A lower-case register name overwrites, so empty it
                    // now; an upper-case one always appends.
                    if *arg as c_int == NUL && !(redir_reg.get() as u8).is_ascii_uppercase() {
                        write_reg_contents(redir_reg.get(), c"".as_ptr(), 0 as ssize_t, 0);
                    }
                }
            }
            if *arg as c_int != NUL {
                redir_reg.set(0);
                semsg(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
            }
        } else if *arg as c_int == '=' as c_int && *arg.add(1) as c_int == '>' as c_int {
            close_redir();
            arg = arg.add(2);
            let append = *arg as c_int == '>' as c_int;
            if append {
                arg = arg.add(1);
            }
            if var_redir_start(skipwhite(arg), append) == OK {
                redir_vname.set(true);
            }
        } else {
            semsg(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
        }
        // Whichever form succeeded, output is being captured again.
        if !(*redir_fd.ptr()).is_null() || redir_reg.get() != 0 || redir_vname.get() {
            redir_off.set(false);
        }
    }
}

/// `:redraw` — draw now, with 'lazyredraw' and the redraw suppression
/// counter out of the way.
pub(crate) unsafe fn ex_redraw(eap: *mut exarg_T) {
    unsafe {
        if cmdpreview.get() {
            return;
        }
        let (r, p) = suspend_lazyredraw();
        validate_cursor(curwin.get());
        update_topline(curwin.get());
        if (*eap).forceit != 0 {
            redraw_all_later(UPD_NOT_VALID);
            redraw_cmdline.set(true);
        } else if VIsual_active.get() {
            redraw_curbuf_later(UPD_INVERTED);
        }
        update_screen();
        if need_maketitle.get() {
            maketitle();
        }
        resume_lazyredraw(r, p);
        // The command line is clean again after a full redraw.
        msg_didout.set(false);
        msg_col.set(0);
        need_wait_return.set(false);
        ui_flush();
    }
}

/// `:redrawstatus` — the status lines only, unless a full redraw is
/// needed to show them.
pub(crate) unsafe fn ex_redrawstatus(eap: *mut exarg_T) {
    unsafe {
        if cmdpreview.get() {
            return;
        }
        if (*eap).forceit != 0 {
            status_redraw_all();
        } else {
            status_redraw_curbuf();
        }
        let (r, p) = suspend_lazyredraw();
        if State.get() & MODE_CMDLINE != 0 {
            redraw_statuslines();
        } else {
            if VIsual_active.get() {
                redraw_curbuf_later(UPD_INVERTED);
            }
            update_screen();
        }
        resume_lazyredraw(r, p);
        ui_flush();
    }
}

/// `:redrawtabline`.
pub(crate) unsafe fn ex_redrawtabline(_eap: *mut exarg_T) {
    unsafe {
        let (r, p) = suspend_lazyredraw();
        draw_tabline();
        resume_lazyredraw(r, p);
        ui_flush();
    }
}

/// Take the redraw suppression counter and 'lazyredraw' out of the way,
/// answering what they were.
fn suspend_lazyredraw() -> (c_int, c_int) {
    let saved = (RedrawingDisabled.get(), p_lz.get());
    RedrawingDisabled.set(0);
    p_lz.set(0);
    saved
}

/// Put both back.
fn resume_lazyredraw(r: c_int, p: c_int) {
    RedrawingDisabled.set(r);
    p_lz.set(p);
}

/// Stop capturing message output, whichever destination is open.
pub(crate) unsafe fn close_redir() {
    unsafe {
        if !(*redir_fd.ptr()).is_null() {
            fclose(redir_fd.get());
            redir_fd.set(ptr::null_mut::<FILE>());
        }
        redir_reg.set(0);
        if redir_vname.get() {
            var_redir_stop();
            redir_vname.set(false);
        }
    }
}

/// `:digraphs` — define digraphs, or list them.
pub(crate) unsafe fn ex_digraphs(eap: *mut exarg_T) {
    unsafe {
        if *(*eap).arg as c_int != NUL {
            putdigraph(core::ffi::CStr::from_ptr((*eap).arg).to_bytes());
        } else {
            listdigraphs((*eap).forceit != 0);
        }
    }
}

/// Set 'no_hlsearch', keeping `v:hlsearch` in step.
pub unsafe fn set_no_hlsearch(flag: bool) {
    unsafe {
        no_hlsearch.set(flag);
        set_vim_var_nr(
            VV_HLSEARCH,
            (!no_hlsearch.get() && p_hls.get() != 0) as varnumber_T,
        );
    }
}

/// `:nohlsearch`.
pub(crate) unsafe fn ex_nohlsearch(_eap: *mut exarg_T) {
    unsafe {
        set_no_hlsearch(true);
        redraw_all_later(UPD_SOME_VALID);
    }
}

/// Did the last Ex-mode line end with a bare Return?
pub fn get_pressedreturn() -> bool {
    ex_pressedreturn.get()
}

/// Record whether it did.
pub fn set_pressedreturn(val: bool) {
    ex_pressedreturn.set(val);
}
