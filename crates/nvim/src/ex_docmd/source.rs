//! What surrounds a `do_cmdline` call: the exception state it saves and
//! restores for the debugger, the line getter it reads through, the loop
//! line store `:while` and `:for` replay from, and Ex mode.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::buffer::buf_get_changedtick;
use crate::strings::vim_snprintf;

use crate::getchar::typeahead;
use crate::guard::Suppress;
use crate::memline::MlFlags;
use crate::smsg_c;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::ptr;
use std::ffi::CString;

use crate::clipboard::{end_batch_changes, start_batch_changes};
use crate::cstr;
use crate::drawscreen::{UPD_NOT_VALID, redraw_all_later, update_screen};
use crate::eval::vars::set_vim_var_string;

use crate::ex_cmds::print_line_no_prefix;
use crate::ex_docmd::cmdline::{do_cmdline, sourcing_entry};

use crate::ex_docmd::{
    DoCmdOpts, ETYPE_EXCEPT, MSG_BUF_LEN, cmdline_call_depth, dbg_stuff, ex_pressedreturn,
    loop_cookie, wcmd_T,
};
use crate::ex_eval::discard_current_exception;
use crate::ex_getln::{getcmdline, getexline};
use crate::garray::ga_append_via_ptr;
use crate::highlight_group::HLF_E;
use crate::main::{
    KeyTyped, Rows, State, caught_stack, check_cstack, cmdline_row, curbuf, current_exception,
    did_emsg, did_throw, e_empty_buffer, emsg_silent, ex_no_reprint, ex_normal_busy, exiting,
    exmode_active, force_abort, global_busy, got_int, lines_left, msg_col, msg_row, msg_scroll,
    msg_silent, need_rethrow, need_wait_return, p_mfd, suppress_errthrow, trylevel,
};

use crate::message::{
    emsg_multiline, msg, msg_clr_eos, msg_puts, msg_scroll_flush, verbose_enter_scroll,
    verbose_leave_scroll,
};

use crate::runtime::{estack_pop, estack_push, set_sourcing_lnum};
use crate::state::{MODE_NORMAL, may_trigger_modechanged};

use crate::types::{
    FAIL, IOSIZE, LineGetter, OK, OptInt, Vv, except_T, garray_T, linenr_T, msglist_T, ptrdiff_t,
    size_t,
};

use crate::winlayer::{Buf, Live, Win};

/// The debugger's saved exception environment, whose caller has promised it
/// outlives the value: `save_dbg_stuff`/`restore_dbg_stuff` are handed a
/// `dbg_stuff` the debugger's own frame owns.
type Dbg = Live<dbg_stuff>;

/// The exception `handle_did_throw` is reporting, live until it discards it.
type Exc = Live<except_T>;

/// The stored lines a `:while`/`:for` body is replayed from, owned by the
/// frame running the loop.
type Lc = Live<loop_cookie>;

/// Take the whole exception environment out of the way, and answer it.
///
/// Used only by the debugger: a `>quit` at a breakpoint must not be
/// swallowed by whatever `:try` the script had open.
pub(crate) unsafe fn save_dbg_stuff(dsp: *mut dbg_stuff) {
    // SAFETY: the caller's own `dbg_stuff`, live for the call.
    let mut d = unsafe { Dbg::new(dsp) };
    d.trylevel = trylevel.get();
    trylevel.set(0);
    d.force_abort = force_abort.get() as c_int;
    force_abort.set(false);
    d.caught_stack = caught_stack.get();
    caught_stack.set(ptr::null_mut());
    // Both of these answer the old value and clear it.
    d.vv_exception = v_exception(ptr::null_mut());
    d.vv_throwpoint = v_throwpoint(ptr::null_mut());
    d.did_emsg = did_emsg.get();
    did_emsg.set(0);
    d.got_int = got_int.get() as c_int;
    got_int.set(false);
    d.did_throw = did_throw.get();
    did_throw.set(false);
    d.need_rethrow = need_rethrow.get() as c_int;
    need_rethrow.set(false);
    d.check_cstack = check_cstack.get() as c_int;
    check_cstack.set(false);
    d.current_exception = current_exception.get();
    current_exception.set(ptr::null_mut());
}

/// Put it all back.
pub(crate) unsafe fn restore_dbg_stuff(dsp: *mut dbg_stuff) {
    // SAFETY: as `save_dbg_stuff`.
    let d = unsafe { Dbg::new(dsp) };
    suppress_errthrow.set(false);
    trylevel.set(d.trylevel);
    force_abort.set(d.force_abort != 0);
    caught_stack.set(d.caught_stack);
    v_exception(d.vv_exception);
    v_throwpoint(d.vv_throwpoint);
    did_emsg.set(d.did_emsg);
    got_int.set(d.got_int != 0);
    did_throw.set(d.did_throw);
    need_rethrow.set(d.need_rethrow != 0);
    check_cstack.set(d.check_cstack != 0);
    current_exception.set(d.current_exception);
}

/// Ex mode: read and run one command line at a time, printing the current
/// line after each one that moved the cursor or changed the buffer.
pub unsafe fn do_exmode() {
    exmode_active.set(true);
    State.set(MODE_NORMAL);
    unsafe { may_trigger_modechanged() };

    // `:global` runs Ex mode for each line itself; there is no prompt
    // to give.
    if global_busy.get() != 0 {
        return;
    }

    let save_msg_scroll = msg_scroll.get();
    let redraw_off = Suppress::redraw();
    let no_prompt = Suppress::wait_return();
    unsafe {
        msg(
            gettext(c"Entering Ex mode.  Type \"visual\" to go to Normal mode.".as_ptr()),
            0,
        )
    };

    while exmode_active.get() {
        // `:normal` that ran out of keys leaves Ex mode rather than
        // waiting for more.
        if ex_normal_busy.get() > 0 && typeahead().is_empty() {
            exmode_active.set(false);
            break;
        }

        msg_scroll.set(1);
        need_wait_return.set(false);
        ex_pressedreturn.set(false);
        ex_no_reprint.set(false);
        let changedtick = buf_get_changedtick(unsafe { Buf::new(curbuf.get()) });
        let prev_msg_row = msg_row.get();
        let prev_line = cur_win().w_cursor.lnum;
        cmdline_row.set(msg_row.get());

        let plain = DoCmdOpts::NONE;
        unsafe { do_cmdline(ptr::null_mut(), Some(getexline), ptr::null_mut(), plain) };
        lines_left.set(Rows.get() - 1);

        let moved = prev_line != cur_win().w_cursor.lnum
            || changedtick != buf_get_changedtick(unsafe { Buf::new(curbuf.get()) });
        if moved && !ex_no_reprint.get() {
            if cur_buf().b_ml.ml_flags.has(MlFlags::EMPTY) {
                emsg(gettext(e_empty_buffer.as_ptr()));
            } else {
                // A bare Return already scrolled; print over that line
                // rather than under it.
                if ex_pressedreturn.get() {
                    unsafe { msg_scroll_flush() };
                    msg_row.set(prev_msg_row);
                    if prev_msg_row == Rows.get() - 1 {
                        msg_row.set(msg_row.get() - 1);
                    }
                }
                msg_col.set(0);
                unsafe { print_line_no_prefix(cur_win().w_cursor.lnum, false, false) };
                unsafe { msg_clr_eos() };
            }
        } else if ex_pressedreturn.get() && !ex_no_reprint.get() {
            // Return on the last line: there is nothing to print.
            if cur_buf().b_ml.ml_flags.has(MlFlags::EMPTY) {
                emsg(gettext(e_empty_buffer.as_ptr()));
            } else {
                emsg(gettext(c"E501: At end-of-file".as_ptr()));
            }
        }
    }

    drop(redraw_off);
    drop(no_prompt);
    unsafe { redraw_all_later(UPD_NOT_VALID) };
    unsafe { update_screen() };
    need_wait_return.set(false);
    msg_scroll.set(save_msg_scroll);
}

/// `:verbose` >= 15: report the command about to run, and which line of
/// which script it is.
pub(crate) unsafe fn msg_verbose_cmd(lnum: linenr_T, cmd: *mut c_char) {
    let _no_prompt = Suppress::wait_return();
    unsafe { verbose_enter_scroll() };
    if lnum == 0 {
        unsafe { smsg_c!(0, gettext(c"Executing: %s".as_ptr()), cmd) };
    } else {
        unsafe { smsg_c!(0, gettext(c"line %d: %s".as_ptr()), lnum, cmd) };
    }
    if msg_silent.get() == 0 {
        unsafe { msg_puts(c"\n".as_ptr()) };
    }
    unsafe { verbose_leave_scroll() };
}

/// Enter a `do_cmdline` call, refusing to nest past 'maxfuncdepth'.
///
/// The limit only bites above a floor of 200: a low 'maxfuncdepth' must
/// still leave room for the editor's own nesting.
pub(crate) fn do_cmdline_start() -> c_int {
    debug_assert!(cmdline_call_depth.get() >= 0);
    if cmdline_call_depth.get() >= 200 && cmdline_call_depth.get() as OptInt >= p_mfd.get() {
        return FAIL;
    }
    cmdline_call_depth.set(cmdline_call_depth.get() + 1);
    // Clipboard writes are batched across the whole command line, so
    // that a `:while` that yanks repeatedly sets the selection once.
    start_batch_changes();
    OK
}

/// Leave it.
pub(crate) fn do_cmdline_end() {
    cmdline_call_depth.set(cmdline_call_depth.get() - 1);
    debug_assert!(cmdline_call_depth.get() >= 0);
    end_batch_changes();
}

/// Report an exception that reached the outermost `:try`, and discard it.
///
/// A user exception (`:throw`) is reported as E605; an error exception
/// replays the messages it was built from, so that the original error text
/// is what the user sees; an interrupt says nothing, because the interrupt
/// message is given elsewhere.
pub unsafe fn handle_did_throw() {
    debug_assert!(!current_exception.get().is_null());
    // SAFETY: non-null by the assert above, and live until
    // `discard_current_exception` below.
    let mut exception = unsafe { Exc::new(current_exception.get()) };
    let mut reported: *mut c_char = ptr::null_mut();
    let mut messages: *mut msglist_T = ptr::null_mut();

    match exception.type_0 as c_uint {
        0 => {
            // ET_USER
            let mut buf = [0 as c_char; IOSIZE as usize];
            unsafe {
                vim_snprintf(
                    buf.as_mut_ptr(),
                    IOSIZE as size_t,
                    gettext(c"E605: Exception not caught: %s".as_ptr()),
                    exception.value,
                )
            };
            reported = xstrdup(buf.as_ptr());
        }
        1 => {
            // ET_ERROR: take the messages, so that discarding the
            // exception does not free them.
            messages = exception.messages;
            exception.messages = ptr::null_mut();
        }
        // ET_INTERRUPT, and anything else.
        _ => {}
    }

    // Report against where the exception was thrown, not where it was
    // caught.
    estack_push(ETYPE_EXCEPT, exception.throw_name, exception.throw_lnum);
    exception.throw_name = ptr::null_mut();
    unsafe { discard_current_exception() };

    // `:silent!` makes even an uncaught exception non-fatal.
    if emsg_silent.get() == 0 {
        suppress_errthrow.set(true);
        force_abort.set(true);
    }

    if !messages.is_null() {
        let mut m = messages;
        while !m.is_null() {
            let next = unsafe { (*m).next };
            unsafe { emsg_multiline((*m).msg, c"emsg".as_ptr(), HLF_E, (*m).multiline) };
            unsafe { xfree((*m).msg as *mut c_void) };
            unsafe { xfree((*m).sfile as *mut c_void) };
            xfree(m as *mut c_void);
            m = next;
        }
    } else if !reported.is_null() {
        emsg(reported);
        xfree(reported as *mut c_void);
    }

    xfree(sourcing_entry().es_name as *mut c_void);
    estack_pop();
}

/// The line getter `do_one_cmd` is handed inside a `:while` or `:for`.
///
/// Replays a stored line when there is one, and otherwise reads a new line
/// from the getter underneath and stores it on the way through. That is
/// what lets a `:function` be defined inside a loop: its body is read once
/// and replayed with everything else.
///
/// Keeps the raw signature: it is stored as a `LineGetter`.
pub(crate) unsafe fn get_loop_line(
    c: c_int,
    cookie: *mut c_void,
    indent: c_int,
    do_concat: bool,
) -> *mut c_char {
    // SAFETY: the cookie is the `:while`/`:for` frame's own, live for as
    // long as the loop it drives.
    let mut cp = unsafe { Lc::new(cookie as *mut loop_cookie) };
    if cp.current_line + 1 >= unsafe { (*cp.lines_gap).ga_len } {
        // Past the end of what was stored. On a repeat pass that means
        // the loop body is over.
        if cp.repeating != 0 {
            return ptr::null_mut();
        }
        let line = match cp.lc_getline {
            Some(get) => unsafe { get(c, cp.cookie, indent, do_concat) },
            None => unsafe { getcmdline(c, 0, indent, do_concat) },
        };
        if !line.is_null() {
            unsafe { store_loop_line(cp.lines_gap, line) };
            cp.current_line += 1;
        }
        return line;
    }
    // A replayed line was not typed.
    KeyTyped.set(false);
    cp.current_line += 1;
    let wp = unsafe { ((*cp.lines_gap).ga_data as *mut wcmd_T).offset(cp.current_line as isize) };
    set_sourcing_lnum(unsafe { (*wp).lnum });
    unsafe { xstrdup((*wp).line) }
}

/// Remember a line, with the source line number it came from.
pub(crate) unsafe fn store_loop_line(gap: *mut garray_T, line: *mut c_char) {
    let p = unsafe { ga_append_via_ptr(gap, size_of::<wcmd_T>()) } as *mut wcmd_T;
    unsafe { (*p).line = xstrdup(line) };
    unsafe { (*p).lnum = sourcing_entry().es_lnum };
}

/// Are these the same line getter? Spelled out so the intent survives the
/// `unpredictable_function_pointer_comparisons` lint.
pub(crate) fn line_getter_eq(a: LineGetter, b: LineGetter) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => ptr::fn_addr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

/// Is `fgetline` — or whatever it is wrapping — this line getter?
///
/// `get_loop_line` wraps another getter, and a loop inside a loop wraps it
/// again, so the chain has to be walked before the comparison means
/// anything.
pub unsafe fn getline_equal(fgetline: LineGetter, cookie: *mut c_void, func: LineGetter) -> bool {
    let (gp, _) = unwrap_loop_getter(fgetline, cookie);
    line_getter_eq(gp, func)
}

/// The cookie at the bottom of that chain — the function or script the
/// lines really come from.
pub unsafe fn getline_cookie(fgetline: LineGetter, cookie: *mut c_void) -> *mut c_void {
    let (_, cp) = unwrap_loop_getter(fgetline, cookie);
    cp as *mut c_void
}

/// Walk out of every `get_loop_line` wrapper.
fn unwrap_loop_getter(fgetline: LineGetter, cookie: *mut c_void) -> (LineGetter, *mut loop_cookie) {
    let mut gp = fgetline;
    let mut cp = cookie as *mut loop_cookie;
    while line_getter_eq(gp, Some(get_loop_line)) {
        gp = unsafe { (*cp).lc_getline };
        cp = unsafe { (*cp).cookie } as *mut loop_cookie;
    }
    (gp, cp)
}

/// A translated message, copied into an owned Ex-command error message.
///
/// Every producer of an `exarg_T::errmsg` — and of the `errormsg`
/// out-parameter the parser threads — answers a buffer of its own, so that
/// raising a second error before the first is reported cannot overwrite it.
/// Upstream shared two static buffers here (`IObuff` and `ex_error_buf`)
/// and `emsg` runs autocommands, so the overwrite was reachable.
///
/// # Safety
///
/// `msg` must be NUL-terminated.
pub(crate) unsafe fn ex_msg(msg: *const c_char) -> CString {
    // SAFETY: the caller's NUL-terminated message; `gettext` answers it or
    // a translation of it, equally NUL-terminated.
    unsafe { CStr::from_ptr(gettext(msg)) }.to_owned()
}

/// [`ex_msg`] for a message with one `%s` in it.
///
/// # Safety
///
/// `msg_0` must be a NUL-terminated format taking one `%s`, and `arg` a
/// NUL-terminated string.
pub(crate) unsafe fn ex_errmsg(msg_0: *const c_char, arg: *const c_char) -> CString {
    let mut buf = [0 as c_char; MSG_BUF_LEN as usize];
    let size = MSG_BUF_LEN as size_t;
    // SAFETY: the caller's format and argument, and the whole of `buf`.
    unsafe { vim_snprintf(buf.as_mut_ptr(), size, gettext(msg_0), arg) };
    cstr::in_chars(&buf).to_owned()
}

/// Cancel an exit that a QuitPre or ExitPre autocommand called off.
pub unsafe fn not_exiting(save_exiting: bool) {
    exiting.set(save_exiting);
    unsafe { set_vim_var_string(Vv::Exitreason, ptr::null(), -1 as ptrdiff_t) };
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// `emsg()` as checked code.
fn emsg(s: *const c_char) -> bool {
    // SAFETY: a NUL-terminated message.
    unsafe { crate::message::emsg(s) }
}

/// `gettext()` as checked code.
fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    // SAFETY: a NUL-terminated message; `gettext` answers one too.
    unsafe { crate::os::cshim::gettext(__msgid) }
}

/// `v_exception()` as checked code.
fn v_exception(oldval: *mut c_char) -> *mut c_char {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::eval::vars::v_exception(oldval) }
}

/// `v_throwpoint()` as checked code.
fn v_throwpoint(oldval: *mut c_char) -> *mut c_char {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::eval::vars::v_throwpoint(oldval) }
}

/// `xfree()` as checked code.
fn xfree(ptr: *mut c_void) {
    // SAFETY: `xmalloc`ed, or null.
    unsafe { crate::memory::xfree(ptr) }
}

/// `xstrdup()` as checked code.
fn xstrdup(str: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::memory::xstrdup(str) }
}
