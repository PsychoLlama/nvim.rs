//! Commands about what is on the screen rather than in the buffer:
//! redrawing, `:redir`, highlighting and the digraph table.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::guard::{Allow, Saved, Suppress};
use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::buffer::maketitle;

use crate::digraph::{listdigraphs, putdigraph};
use crate::drawscreen::{
    UPD_INVERTED, UPD_NOT_VALID, UPD_SOME_VALID, redraw_curbuf_later, redraw_statuslines,
    status_redraw_all, status_redraw_curbuf,
};

use crate::eval::eval_to_string;
use crate::eval::vars::{set_vim_var_nr, var_redir_start, var_redir_stop};
use crate::ex_docmd::argopt::open_exfile;
use crate::ex_docmd::ex_pressedreturn;
use crate::highlight_group::{do_highlight, load_colors};
use crate::main::{
    State, cmdpreview, e_invarg2, msg_col, msg_didout, need_maketitle, need_wait_return,
    no_hlsearch, p_hls, p_lz, redir_fd, redir_off, redir_reg, redir_vname, redraw_cmdline,
};
use crate::memory::xstrdup;

use crate::message::msg_ext_set_kind;

use crate::r#move::{update_topline, validate_cursor};
use crate::normal::visual_active;

use crate::os::env::expand_env_save;
use crate::register::{valid_yank_reg, write_reg_contents};
use crate::state::MODE_CMDLINE;
use crate::statusline::draw_tabline;
use crate::types::{FAIL, FILE, NUL, OK, Vv, exarg_T, ssize_t, varnumber_T};

use crate::winlayer::{Ea, Win};
use ::libc::{fclose, strcasecmp};

/// `:colorscheme` — with no argument, report `g:colors_name`.
pub(crate) unsafe fn ex_colorscheme(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if byte(eap.arg) != NUL {
        if unsafe { load_colors(eap.arg) } == FAIL {
            unsafe {
                semsg_c!(
                    gettext(c"E185: Cannot find color scheme '%s'".as_ptr()),
                    eap.arg,
                )
            };
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

    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    if name.is_null() {
        msg(c"default".as_ptr(), 0);
    } else {
        msg(name, 0);
        xfree(name as *mut c_void);
    }
}

/// `:highlight`, and the greeting `:hi!` prints on its own.
pub(crate) unsafe fn ex_highlight(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if byte(eap.arg) == NUL && byte_at(eap.cmd, 2) == '!' as c_int {
        msg(gettext(c"Greetings, Vim user!".as_ptr()), 0);
    }
    unsafe { do_highlight(eap.arg, eap.forceit != 0, false) };
}

/// `:redir` — send message output to a file, a register or a variable
/// until `:redir END`.
///
/// Only one destination at a time: every form closes whatever was open
/// first.
pub(crate) unsafe fn ex_redir(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    let mut arg = eap.arg;
    if unsafe { strcasecmp(eap.arg, c"END".as_ptr() as *mut c_char) } == 0 {
        close_redir();
    } else if byte(arg) == '>' as c_int {
        // `:redir > file` truncates, `:redir >> file` appends.
        arg = unsafe { arg.add(1) };
        let mode = if byte(arg) == '>' as c_int {
            arg = unsafe { arg.add(1) };
            c"a".as_ptr() as *mut c_char
        } else {
            c"w".as_ptr() as *mut c_char
        };
        arg = skipwhite(arg);
        close_redir();
        let fname = unsafe { expand_env_save(arg) };
        if fname.is_null() {
            return;
        }
        redir_fd.set(unsafe { open_exfile(fname, eap.forceit, mode) });
        xfree(fname as *mut c_void);
    } else if byte(arg) == '@' as c_int {
        close_redir();
        arg = unsafe { arg.add(1) };
        if unsafe { valid_yank_reg(*arg as c_int, true) } && byte(arg) != '_' as c_int {
            redir_reg.set(ubyte(arg) as c_int);
            arg = unsafe { arg.add(1) };
            if byte(arg) == '>' as c_int && byte_at(arg, 1) == '>' as c_int {
                // `:redir @a>>` appends.
                arg = unsafe { arg.add(2) };
            } else {
                if byte(arg) == '>' as c_int {
                    arg = unsafe { arg.add(1) };
                }
                // A lower-case register name overwrites, so empty it
                // now; an upper-case one always appends.
                if byte(arg) == NUL && !(redir_reg.get() as u8).is_ascii_uppercase() {
                    unsafe { write_reg_contents(redir_reg.get(), c"".as_ptr(), 0 as ssize_t, 0) };
                }
            }
        }
        if byte(arg) != NUL {
            redir_reg.set(0);
            unsafe { semsg_c!(gettext(e_invarg2.as_ptr()), eap.arg) };
        }
    } else if byte(arg) == '=' as c_int && byte_at(arg, 1) == '>' as c_int {
        close_redir();
        arg = unsafe { arg.add(2) };
        let append = byte(arg) == '>' as c_int;
        if append {
            arg = unsafe { arg.add(1) };
        }
        if unsafe { var_redir_start(skipwhite(arg), append) } == OK {
            redir_vname.set(true);
        }
    } else {
        unsafe { semsg_c!(gettext(e_invarg2.as_ptr()), eap.arg) };
    }
    // Whichever form succeeded, output is being captured again.
    if !redir_fd.get().is_null() || redir_reg.get() != 0 || redir_vname.get() {
        redir_off.set(false);
    }
}

/// `:redraw` — draw now, with 'lazyredraw' and the redraw suppression
/// counter out of the way.
pub(crate) unsafe fn ex_redraw(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if cmdpreview.get() {
        return;
    }
    let lazyredraw_off = suspend_lazyredraw();
    validate_cursor(cur_win());
    update_topline(cur_win());
    if eap.forceit != 0 {
        redraw_all_later(UPD_NOT_VALID);
        redraw_cmdline.set(true);
    } else if visual_active() {
        redraw_curbuf_later(UPD_INVERTED);
    }
    update_screen();
    if need_maketitle.get() {
        unsafe { maketitle() };
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
pub(crate) unsafe fn ex_redrawstatus(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if cmdpreview.get() {
        return;
    }
    if eap.forceit != 0 {
        unsafe { status_redraw_all() };
    } else {
        unsafe { status_redraw_curbuf() };
    }
    let lazyredraw_off = suspend_lazyredraw();
    if State.get() & MODE_CMDLINE != 0 {
        unsafe { redraw_statuslines() };
    } else {
        if visual_active() {
            redraw_curbuf_later(UPD_INVERTED);
        }
        update_screen();
    }
    drop(lazyredraw_off);
    ui_flush();
}

/// `:redrawtabline`.
pub(crate) unsafe fn ex_redrawtabline(_eap: *mut exarg_T) {
    let lazyredraw_off = suspend_lazyredraw();
    unsafe { draw_tabline() };
    drop(lazyredraw_off);
    ui_flush();
}

/// The redraw suppression counter and 'lazyredraw' held out of the way,
/// and put back when the guard is dropped.
struct LazyRedrawOff {
    _redraw: Saved,
    p_lz: c_int,
}

impl Drop for LazyRedrawOff {
    fn drop(&mut self) {
        p_lz.set(self.p_lz);
    }
}

/// Take both out of the way until the answer is dropped.
fn suspend_lazyredraw() -> LazyRedrawOff {
    let off = LazyRedrawOff {
        _redraw: Allow::redraw(),
        p_lz: p_lz.get(),
    };
    p_lz.set(0);
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
        unsafe { var_redir_stop() };
        redir_vname.set(false);
    }
}

/// `:digraphs` — define digraphs, or list them.
pub(crate) unsafe fn ex_digraphs(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if byte(eap.arg) != NUL {
        putdigraph(unsafe { core::ffi::CStr::from_ptr(eap.arg) }.to_bytes());
    } else {
        listdigraphs(eap.forceit != 0);
    }
}

/// Set 'no_hlsearch', keeping `v:hlsearch` in step.
pub unsafe fn set_no_hlsearch(flag: bool) {
    no_hlsearch.set(flag);
    unsafe {
        set_vim_var_nr(
            Vv::Hlsearch,
            (!no_hlsearch.get() && p_hls.get() != 0) as varnumber_T,
        )
    };
}

/// `:nohlsearch`.
pub(crate) unsafe fn ex_nohlsearch(_eap: *mut exarg_T) {
    unsafe { set_no_hlsearch(true) };
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
    unsafe { crate::os::cshim::gettext(__msgid) }
}

/// `msg()` as checked code.
fn msg(s: *const c_char, hl_id: c_int) -> bool {
    // SAFETY: a NUL-terminated message.
    unsafe { crate::message::msg(s, hl_id) }
}

/// `redraw_all_later()` as checked code.
fn redraw_all_later(redr_type: c_int) {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::drawscreen::redraw_all_later(redr_type) }
}

/// `skipwhite()` as checked code.
fn skipwhite(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::charset::skipwhite(p) }
}

/// `ui_flush()` as checked code.
fn ui_flush() {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::ui::ui_flush() }
}

/// `update_screen()` as checked code.
fn update_screen() -> c_int {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::drawscreen::update_screen() }
}

/// `xfree()` as checked code.
fn xfree(ptr: *mut c_void) {
    // SAFETY: `xmalloc`ed, or null.
    unsafe { crate::memory::xfree(ptr) }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The byte `p` points at, as the C's `*p` reads it.
fn byte(p: *const c_char) -> c_int {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as c_int }
}

/// The byte `p` points at, unsigned, as the C's `(uint8_t)*p` reads it.
fn ubyte(p: *const c_char) -> u8 {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as u8 }
}

/// The byte at `p[i]`, as the C's `*(p + i)` reads it.
fn byte_at(p: *const c_char, i: isize) -> c_int {
    // SAFETY: an offset within the NUL-terminated string `p` points into.
    unsafe { *p.offset(i) as c_int }
}
