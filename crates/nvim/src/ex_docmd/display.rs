//! Commands about what is on the screen rather than in the buffer:
//! redrawing, `:redir`, highlighting and the digraph table.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::guard::{Allow, Saved, Suppress};
use crate::message_fmt::msg_bytes;
use crate::semsg;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::buffer::maketitle;

use crate::digraph::{listdigraphs, putdigraph};
use crate::drawscreen::{
    UPD_INVERTED, UPD_NOT_VALID, UPD_SOME_VALID, redraw_curbuf_later, redraw_statuslines,
    status_redraw_all, status_redraw_curbuf,
};

use crate::drawscreen::state::{need_maketitle, redraw_cmdline};
use crate::eval::eval_to_string;
use crate::eval::vars::{set_vim_var_nr, var_redir_start, var_redir_stop};
use crate::ex_docmd::argopt::open_exfile;
use crate::ex_docmd::ex_pressedreturn;
use crate::ex_docmd::xfree;
use crate::ex_getln::state::cmdpreview;
use crate::highlight_group::{do_highlight, load_colors};
use crate::memory::xstrdup;
use crate::message::state::{
    msg_col, msg_didout, need_wait_return, redir_fd, redir_off, redir_reg, redir_vname,
};
use crate::option::vars::{P_LZ, p_hls, p_lz};
use crate::search::state::no_hlsearch;
use crate::state::mode::State;

use crate::message::msg_ext_set_kind;

use crate::r#move::{update_topline, validate_cursor};
use crate::normal::visual_active;

use crate::os::env::expand_env_save;
use crate::register::{valid_yank_reg, write_reg_contents};
use crate::state::MODE_CMDLINE;
use crate::statusline::draw_tabline;
use crate::types::{ExArg, FILE, Failed, VarNumber, Vv, ssize_t};

use crate::winlayer::Win;
use ::libc::fclose;

/// `:colorscheme` — with no argument, report `g:colors_name`.
pub(crate) fn ex_colorscheme(excmd: &mut ExArg) {
    if excmd.line.byte_at(excmd.line.arg) != 0 {
        if load_colors(excmd.line.cstr_from(excmd.line.arg)).is_err() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = msg_bytes(excmd.line.arg());
            semsg!("E185: Cannot find color scheme '{arg}'");
        }
        return;
    }
    // The variable may not exist, which is not an error here: an
    // unnamed scheme reports `default`.
    let expr = unsafe { xstrdup(c"g:colors_name".as_ptr()) };
    let no_emsg = Suppress::emsg();
    let name = unsafe { eval_to_string(expr, false, false) };
    drop(no_emsg);
    xfree(expr as *mut c_void);

    msg_ext_set_kind(c"list_cmd");
    if name.is_null() {
        msg(c"default".as_ptr(), 0);
    } else {
        msg(name, 0);
        xfree(name as *mut c_void);
    }
}

/// `:highlight`, and the greeting `:hi!` prints on its own.
pub(crate) fn ex_highlight(excmd: &mut ExArg) {
    if excmd.line.byte_at(excmd.line.arg) == 0 && excmd.line.byte_at(excmd.line.cmd + 2) == b'!' {
        msg(gettext(c"Greetings, Vim user!".as_ptr()), 0);
    }
    do_highlight(excmd.line.cstr_from(excmd.line.arg), excmd.forceit, false);
}

/// `:redir` — send message output to a file, a register or a variable
/// until `:redir END`.
///
/// Only one destination at a time: every form closes whatever was open
/// first.
pub(crate) fn ex_redir(excmd: &mut ExArg) {
    let mut at = excmd.line.arg;
    if excmd.line.arg().eq_ignore_ascii_case(b"END") {
        close_redir();
    } else if excmd.line.byte_at(at) == b'>' {
        // `:redir > file` truncates, `:redir >> file` appends.
        at += 1;
        let mode = if excmd.line.byte_at(at) == b'>' {
            at += 1;
            c"a".as_ptr() as *mut c_char
        } else {
            c"w".as_ptr() as *mut c_char
        };
        at = excmd.line.skip_white(at);
        close_redir();
        // SAFETY: the rest of the command's own NUL-terminated line.
        let fname = unsafe { expand_env_save(excmd.line.ptr_at(at)) };
        if fname.is_null() {
            return;
        }
        redir_fd.set(unsafe { open_exfile(fname, c_int::from(excmd.forceit), mode) });
        xfree(fname as *mut c_void);
    } else if excmd.line.byte_at(at) == b'@' {
        close_redir();
        at += 1;
        let name = excmd.line.byte_at(at);
        if valid_yank_reg(c_int::from(name), true) && name != b'_' {
            redir_reg.set(c_int::from(name));
            at += 1;
            if excmd.line.byte_at(at) == b'>' && excmd.line.byte_at(at + 1) == b'>' {
                // `:redir @a>>` appends.
                at += 2;
            } else {
                if excmd.line.byte_at(at) == b'>' {
                    at += 1;
                }
                // A lower-case register name overwrites, so empty it
                // now; an upper-case one always appends.
                if excmd.line.byte_at(at) == 0 && !(redir_reg.get() as u8).is_ascii_uppercase() {
                    unsafe { write_reg_contents(redir_reg.get(), c"".as_ptr(), 0 as ssize_t, 0) };
                }
            }
        }
        if excmd.line.byte_at(at) != 0 {
            redir_reg.set(0);
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = msg_bytes(excmd.line.arg());
            semsg!("E475: Invalid argument: {arg}");
        }
    } else if excmd.line.byte_at(at) == b'=' && excmd.line.byte_at(at + 1) == b'>' {
        close_redir();
        at += 2;
        let append = excmd.line.byte_at(at) == b'>';
        if append {
            at += 1;
        }
        let name = excmd.line.ptr_at(excmd.line.skip_white(at));
        // SAFETY: the variable name, inside the command's own line.
        if unsafe { var_redir_start(name, append) }.is_ok() {
            redir_vname.set(true);
        }
    } else {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = msg_bytes(excmd.line.arg());
        semsg!("E475: Invalid argument: {arg}");
    }
    // Whichever form succeeded, output is being captured again.
    if !redir_fd.get().is_null() || redir_reg.get() != 0 || redir_vname.get() {
        redir_off.set(false);
    }
}

/// `:redraw` — draw now, with 'lazyredraw' and the redraw suppression
/// counter out of the way.
pub(crate) fn ex_redraw(excmd: &mut ExArg) {
    if cmdpreview.get() {
        return;
    }
    let lazyredraw_off = suspend_lazyredraw();
    validate_cursor(Win::current());
    update_topline(Win::current());
    if excmd.forceit {
        redraw_all_later(UPD_NOT_VALID);
        redraw_cmdline.set(true);
    } else if visual_active() {
        redraw_curbuf_later(UPD_INVERTED);
    }
    let _ = update_screen();
    if need_maketitle.get() {
        maketitle();
    }
    drop(lazyredraw_off);
    // The command line is clean again after a full redraw.
    msg_didout.set(false);
    msg_col.set(0);
    need_wait_return.set(false);
    ui_flush();
}

/// `:redrawstatus` — the status lines only, unless a full redraw is
/// needed to show them.
pub(crate) fn ex_redrawstatus(excmd: &mut ExArg) {
    if cmdpreview.get() {
        return;
    }
    if excmd.forceit {
        status_redraw_all();
    } else {
        status_redraw_curbuf();
    }
    let lazyredraw_off = suspend_lazyredraw();
    if State.get() & MODE_CMDLINE != 0 {
        redraw_statuslines();
    } else {
        if visual_active() {
            redraw_curbuf_later(UPD_INVERTED);
        }
        let _ = update_screen();
    }
    drop(lazyredraw_off);
    ui_flush();
}

/// `:redrawtabline`.
pub(crate) fn ex_redrawtabline(_excmd: &mut ExArg) {
    let lazyredraw_off = suspend_lazyredraw();
    draw_tabline();
    drop(lazyredraw_off);
    ui_flush();
}

/// The redraw suppression counter and 'lazyredraw' held out of the way,
/// and put back when the guard is dropped.
struct LazyRedrawOff {
    _redraw: Saved,
    p_lz: bool,
}

impl Drop for LazyRedrawOff {
    fn drop(&mut self) {
        P_LZ.set(self.p_lz);
    }
}

/// Take both out of the way until the answer is dropped.
fn suspend_lazyredraw() -> LazyRedrawOff {
    let off = LazyRedrawOff {
        _redraw: Allow::redraw(),
        p_lz: p_lz(),
    };
    P_LZ.set(false);
    off
}

/// Stop capturing message output, whichever destination is open.
pub(crate) fn close_redir() {
    if !redir_fd.get().is_null() {
        unsafe { fclose(redir_fd.get()) };
        redir_fd.set(ptr::null_mut::<FILE>());
    }
    redir_reg.set(0);
    if redir_vname.get() {
        var_redir_stop();
        redir_vname.set(false);
    }
}

/// `:digraphs` — define digraphs, or list them.
pub(crate) fn ex_digraphs(excmd: &mut ExArg) {
    if excmd.line.byte_at(excmd.line.arg) != 0 {
        putdigraph(excmd.line.arg());
    } else {
        listdigraphs(excmd.forceit);
    }
}

/// Set 'no_hlsearch', keeping `v:hlsearch` in step.
pub fn set_no_hlsearch(flag: bool) {
    no_hlsearch.set(flag);
    set_vim_var_nr(Vv::Hlsearch, (!no_hlsearch.get() && p_hls()) as VarNumber);
}

/// `:nohlsearch`.
pub(crate) fn ex_nohlsearch(_excmd: &mut ExArg) {
    set_no_hlsearch(true);
    redraw_all_later(UPD_SOME_VALID);
}

/// Did the last Ex-mode line end with a bare Return?
pub fn get_pressedreturn() -> bool {
    ex_pressedreturn.get()
}

/// Record whether it did.
pub fn set_pressedreturn(val: bool) {
    ex_pressedreturn.set(val);
}

/// `gettext()` as checked code.
fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    // SAFETY: a NUL-terminated message; `gettext` answers one too.
    unsafe { crate::os::cshim::gettext_ptr(__msgid).as_ptr().cast_mut() }
}

/// `msg()` as checked code.
fn msg(s: *const c_char, hl_id: c_int) -> bool {
    // SAFETY: a NUL-terminated message.
    unsafe { crate::message::msg_ptr(s, hl_id) }
}

/// `redraw_all_later()` as checked code.
fn redraw_all_later(redr_type: c_int) {
    crate::drawscreen::redraw_all_later(redr_type)
}

/// `ui_flush()` as checked code.
fn ui_flush() {
    crate::ui::ui_flush()
}

/// `update_screen()` as checked code.
fn update_screen() -> Result<(), Failed> {
    crate::drawscreen::update_screen()
}
