//! The `emsg` family: errors, warnings and where they came from.
//!
//! [`emsg_multiline`] is the funnel -- it consults `'debug'`, the `:try`
//! stack and `v:errmsg` before anything is displayed -- and
//! [`get_emsg_source`] is what prefixes the message with the script and line
//! that raised it.
//!
//! The `s`-prefixed entry points (`semsg`, `siemsg`, `swmsg`,
//! `msg_schedule_semsg` and friends) were C variadics, called ~700 times
//! across the tree as `printf`-style forwarders. They are macros now
//! ([`semsg_c!`](crate::semsg_c) and friends, defined in
//! [`crate::message_fmt`]): each expands to a `vim_snprintf` into
//! the scratch buffer the wrapper owned, then the `*_finish` tail below that
//! reports it. Same bytes, same buffer sizes, same truncation — and no
//! C-variadic definition, which only a nightly compiler can write.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::guard::Suppress;
use crate::log::logmsg_c;
use core::ffi::{c_char, c_int, c_long, c_void};
use core::ptr;

/// The script/function the last error was reported from.
static last_sourcing_name: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

/// The line the last error was reported from.
static last_sourcing_lnum: GlobalCell<c_int> = GlobalCell::new(0);

/// Forget where the last error came from, so the next one names its source
/// again.
///
/// # Safety
/// Only that nothing else holds the stored name.
pub unsafe fn reset_last_sourcing() {
    unsafe { xfree(last_sourcing_name.get().cast()) };
    last_sourcing_name.set(ptr::null_mut());
    last_sourcing_lnum.set(0);
}

/// Is the innermost script/function a different one from the last error's?
///
/// # Safety
/// Only that the exec stack is well formed.
unsafe fn other_sourcing_name() -> bool {
    if exestack_has_name() {
        if !last_sourcing_name.get().is_null() {
            return unsafe { strcmp(sourcing_top().es_name, last_sourcing_name.get()) } != 0;
        }
        return true;
    }
    false
}

/// Is there an innermost exec-stack entry, and does it name a source?
fn exestack_has_name() -> bool {
    crate::runtime::innermost().is_some_and(|entry| !entry.es_name.is_null())
}

/// An allocated "Error in <script>:" line, or null when the source has
/// already been reported.
///
/// # Safety
/// Only that the exec stack is well formed.
unsafe fn get_emsg_source() -> *mut c_char {
    if !exestack_has_name() || !unsafe { other_sourcing_name() } {
        return ptr::null_mut();
    }
    let tofree = unsafe { estack_sfile(ESTACK_NONE) };
    let sname = if tofree.is_null() {
        sourcing_top().es_name
    } else {
        tofree
    };
    let p = unsafe { gettext(c"Error in %s:".as_ptr()) };
    let buf_len = unsafe { strlen(sname) } + unsafe { strlen(p) } + 1;
    let buf: *mut c_char = unsafe { xmalloc(buf_len) }.cast();
    unsafe { snprintf(buf, buf_len, p, sname) };
    unsafe { xfree(tofree.cast()) };
    buf
}

/// An allocated "line NNNN:" line, or null when the line has already been
/// reported (or there is none).
///
/// # Safety
/// Only that the exec stack is well formed.
unsafe fn get_emsg_lnum() -> *mut c_char {
    // Show the source of the error, but not if it is the same as the last
    // time.
    if sourcing_top().es_name.is_null()
        || !(unsafe { other_sourcing_name() } || sourcing_top().es_lnum != last_sourcing_lnum.get())
        || sourcing_top().es_lnum == 0
    {
        return ptr::null_mut();
    }
    let p = unsafe { gettext(c"line %4d:".as_ptr()) };
    let buf_len = 20 + unsafe { strlen(p) };
    let buf: *mut c_char = unsafe { xmalloc(buf_len) }.cast();
    unsafe { snprintf(buf, buf_len, p, sourcing_top().es_lnum) };
    buf
}

/// Display the source of an error message, if it has not been shown already.
///
/// # Safety
/// Only that the exec stack is well formed.
pub unsafe fn msg_source(hl_id: c_int) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false);
    if recursive.get() {
        return;
    }
    recursive.set(true);

    let no_prompt = Suppress::wait_return();
    let p = unsafe { get_emsg_source() };
    if !p.is_null() {
        msg_scroll.set(1);
        unsafe { msg(p, hl_id) };
        unsafe { xfree(p.cast()) };
    }
    let p = unsafe { get_emsg_lnum() };
    if !p.is_null() {
        unsafe { msg(p, HLF_N) };
        unsafe { xfree(p.cast()) };
        last_sourcing_lnum.set(sourcing_top().es_lnum as c_int);
    }

    // Remember the source name and line number, so we can tell when
    // the message changes.
    if sourcing_top().es_name.is_null() || unsafe { other_sourcing_name() } {
        unsafe { xfree(last_sourcing_name.get().cast()) };
        last_sourcing_name.set(ptr::null_mut());
        if !sourcing_top().es_name.is_null() {
            last_sourcing_name.set(unsafe { xstrdup(sourcing_top().es_name) });
            if !unsafe { redirecting() } {
                unsafe { msg_putchar_hl(b'\n' as c_int, hl_id) };
            }
        }
    }
    drop(no_prompt);
    recursive.set(false);
}

/// Is this a bad time to show an error?
///
/// # Safety
/// Only that `'debug'` holds a valid string.
pub(crate) unsafe fn emsg_not_now() -> bool {
    (emsg_off.get() > 0
        && unsafe { vim_strchr(p_debug.get(), b'm' as c_int) }.is_null()
        && unsafe { vim_strchr(p_debug.get(), b't' as c_int) }.is_null())
        || emsg_skip.get() > 0
}

/// Show an error message, possibly spanning several lines.
///
/// Answers true when the message was shown (or deliberately swallowed).
/// `kind` is the `ext_messages` kind; `multiline` keeps embedded newlines
/// rather than escaping them.
///
/// # Safety
/// `s` must be a valid C string; `kind` may be null.
pub unsafe fn emsg_multiline(
    s: *const c_char,
    kind: *const c_char,
    hl_id: c_int,
    multiline: bool,
) -> bool {
    if unsafe { emsg_not_now() } {
        return true;
    }
    called_emsg.set(called_emsg.get() + 1);

    // Reset the "severe" flag: it applies to this message only.
    let severe = emsg_severe.get();
    emsg_severe.set(false);

    if emsg_off.get() == 0 || !unsafe { vim_strchr(p_debug.get(), b't' as c_int) }.is_null() {
        // Cause a throw of an error exception if appropriate. Don't display
        // the error message in this case.
        let mut ignore = false;
        if unsafe { cause_errthrow(s, multiline, is_multihl.get() > 1, severe, &raw mut ignore) } {
            if !ignore {
                did_emsg.set(did_emsg.get() + 1);
            }
            return true;
        }

        if in_assert_fails.get() && emsg_assert_fails_msg.get().is_null() {
            emsg_assert_fails_msg.set(unsafe { xstrdup(s) });
            emsg_assert_fails_lnum.set(sourcing_top().es_lnum as c_long);
            unsafe { xfree(emsg_assert_fails_context.get().cast()) };
            let context = if sourcing_top().es_name.is_null() {
                c"".as_ptr()
            } else {
                sourcing_top().es_name
            };
            emsg_assert_fails_context.set(unsafe { xstrdup(context) });
        }

        // set "v:errmsg", also when using ":silent! cmd"
        unsafe { set_vim_var_string(Vv::Errmsg, s, -1) };

        // When using ":silent! cmd" don't display the error message, but
        // do write it to the redirection and the log.
        if emsg_silent.get() != 0 {
            if !emsg_noredir.get() {
                unsafe { msg_start() };
                // Each source line is redirected with a newline appended.
                // Both helpers size their buffer for one byte more than
                // the text, so the terminator's slot takes it.
                //
                // Called one at a time rather than collected first: the
                // redirection in between can reach `:redir => var`, and
                // `get_emsg_lnum`'s answer depends on state a redirection
                // could in principle move.
                let write_line = |line: *mut c_char| {
                    if !line.is_null() {
                        let len = unsafe { strlen(line) };
                        unsafe { *line.add(len) = b'\n' as c_char };
                        unsafe { redir_write(line, len as ptrdiff_t + 1) };
                        unsafe { xfree(line.cast()) };
                    }
                };
                write_line(unsafe { get_emsg_source() });
                write_line(unsafe { get_emsg_lnum() });
                unsafe { redir_write(s, strlen(s) as ptrdiff_t) };
            }
            let at = c"emsg_multiline".as_ptr();
            if !sourcing_top().es_name.is_null() && sourcing_top().es_lnum != 0 {
                let fmt = c"(:silent) %s (%s (line %d))".as_ptr();
                let name = sourcing_top().es_name;
                let lnum = sourcing_top().es_lnum;
                unsafe { logmsg_c!(LOGLVL_DBG, ptr::null(), at, 845, true, fmt, s, name, lnum) };
            } else {
                let fmt = c"(:silent) %s".as_ptr();
                unsafe { logmsg_c!(LOGLVL_DBG, ptr::null(), at, 847, true, fmt, s) };
            }
            return true;
        }

        let at = c"emsg_multiline".as_ptr();
        if !sourcing_top().es_name.is_null() && sourcing_top().es_lnum != 0 {
            let fmt = c"%s (%s (line %d))".as_ptr();
            let name = sourcing_top().es_name;
            let lnum = sourcing_top().es_lnum;
            unsafe { logmsg_c!(LOGLVL_INF, ptr::null(), at, 855, true, fmt, s, name, lnum) };
        } else {
            let fmt = c"%s".as_ptr();
            unsafe { logmsg_c!(LOGLVL_INF, ptr::null(), at, 857, true, fmt, s) };
        }

        ex_exitval.set(1);

        // Reset msg_silent, an error causes messages to be visible again.
        msg_silent.set(0);
        cmd_silent.set(false);

        if global_busy.get() != 0 {
            // Break out of the :global command.
            global_busy.set(global_busy.get() + 1);
        }

        // Now that we have a message, flush the input buffer or beep.
        if p_eb.get() != 0 {
            beep_flush();
        } else {
            unsafe { flush_buffers(FLUSH_MINIMAL) };
        }
        did_emsg.set(did_emsg.get() + 1);
    }

    emsg_on_display.set(true); // remember there is an error message
    if msg_scrolled.get() != 0 {
        need_wait_return.set(true); // needed in case emsg() is called after wait_return() has cleared it
    }
    unsafe { msg_ext_set_kind(kind) };
    msg_scroll.set(1); // don't overwrite a previous message

    // Skip the flush until the whole message has been written, so that the
    // source line and the error arrive as one ext_messages event.
    let save_msg_skip_flush = msg_ext_skip_flush.get();
    msg_ext_skip_flush.set(true);
    unsafe { msg_source(hl_id) };
    msg_nowait.set(false); // wait for this msg
    let rv = unsafe { msg_keep(s, hl_id, false, multiline) };
    msg_ext_skip_flush.set(save_msg_skip_flush);
    rv
}

/// Show an error message. Exported for the unit specs.
///
/// # Safety
/// `s` must be a valid C string.
pub unsafe extern "C" fn emsg(s: *const c_char) -> bool {
    unsafe { emsg_multiline(s, c"emsg".as_ptr(), HLF_E, false) }
}

/// "E354: Invalid register name" for register `name`.
///
/// # Safety
/// Only that `name` is a character code.
pub unsafe fn emsg_invreg(name: c_int) {
    // SAFETY: a character code, rendered into this frame's own buffer, and
    // a one-string format.
    let display = unsafe { transchar_buf(ptr::null(), name) };
    let fmt = unsafe { gettext(c"E354: Invalid register name: '%s'".as_ptr()) };
    unsafe { crate::semsg_c!(fmt, display.as_ptr()) };
}

/// How much of an error message [`semsg_c!`](crate::semsg_c) keeps.
pub const SEMSG_ERRBUF_LEN: size_t = 1025;

/// How much of an error message [`semsg_multiline_c!`](crate::semsg_multiline_c)
/// keeps.
pub const SEMSG_MULTILINE_ERRBUF_LEN: size_t = 8192;

/// Where [`semsg_c!`](crate::semsg_c) formats: a buffer belonging to the
/// expansion, not the shared one upstream reuses. Formatting an error can
/// run an autocommand -- `emsg` does -- and that autocommand can raise an
/// error of its own, which used to overwrite the message being assembled.
/// The macro's first half; not meant to be called directly.
#[doc(hidden)]
pub fn semsg_errbuf() -> [c_char; SEMSG_ERRBUF_LEN] {
    [0; SEMSG_ERRBUF_LEN]
}

/// [`semsg_errbuf`] for [`semsg_multiline_c!`](crate::semsg_multiline_c). A
/// multiline error can be much longer than one line's worth.
#[doc(hidden)]
pub fn semsg_multiline_errbuf() -> [c_char; SEMSG_MULTILINE_ERRBUF_LEN] {
    [0; SEMSG_MULTILINE_ERRBUF_LEN]
}

/// Report what was formatted into `buf` as an error. The second half of
/// [`semsg_c!`](crate::semsg_c); not meant to be called directly. The macro
/// has already established that errors are on.
///
/// # Safety
/// Only that the message state is the main thread's.
#[doc(hidden)]
pub unsafe fn semsg_report(buf: &[c_char; SEMSG_ERRBUF_LEN]) -> bool {
    unsafe { emsg(buf.as_ptr()) }
}

/// Report what was formatted into `buf` as a multiline error of kind
/// `kind`. The second half of
/// [`semsg_multiline_c!`](crate::semsg_multiline_c).
///
/// # Safety
/// `kind` must be a valid C string.
#[doc(hidden)]
pub unsafe fn semsg_multiline_report(
    buf: &[c_char; SEMSG_MULTILINE_ERRBUF_LEN],
    kind: *const c_char,
) -> bool {
    unsafe { emsg_multiline(buf.as_ptr(), kind, HLF_E, true) }
}

/// An internal error: same as [`emsg`], but skipped when errors are off.
///
/// # Safety
/// `s` must be a valid C string.
pub unsafe fn iemsg(s: *const c_char) {
    if unsafe { emsg_not_now() } {
        return;
    }
    unsafe { emsg(s) };
}

/// "E5555: API call: <where>", for a reached-the-unreachable case.
///
/// # Safety
/// `where_0` must be a valid C string.
pub unsafe fn internal_error(where_0: *const c_char) {
    unsafe { crate::siemsg_c!(gettext(e_intern2.as_ptr()), where_0) };
}

/// Deferred-event handler for [`msg_schedule_semsg`].
///
/// # Safety
/// `argv[0]` must be an allocated C string this call takes ownership of.
pub(crate) unsafe extern "C" fn msg_semsg_event(argv: *mut *mut c_void) {
    let s: *mut c_char = unsafe { (*argv).cast() };
    unsafe { emsg(s) };
    unsafe { xfree(s.cast()) };
}

/// Hand whatever was formatted into [`msg_iobuff`] to the main loop as an
/// error. The second half of
/// [`msg_schedule_semsg_c!`](crate::msg_schedule_semsg_c).
///
/// # Safety
/// Only that the main loop is live.
#[doc(hidden)]
pub unsafe fn msg_schedule_semsg_finish(buf: &[c_char; MSG_IOBUFF_LEN]) {
    let s = unsafe { xstrdup(buf.as_ptr()) };
    let event = Event::new(Some(msg_semsg_event), [s.cast::<c_void>()]);
    unsafe { loop_schedule_deferred(main_loop.ptr(), event) };
}

/// Deferred-event handler for [`msg_schedule_semsg_multiline`].
///
/// # Safety
/// As [`msg_semsg_event`].
pub(crate) unsafe extern "C" fn msg_semsg_multiline_event(argv: *mut *mut c_void) {
    let s: *mut c_char = unsafe { (*argv).cast() };
    unsafe { emsg_multiline(s, c"emsg".as_ptr(), HLF_E, true) };
    unsafe { xfree(s.cast()) };
}

/// Hand whatever was formatted into [`msg_iobuff`] to the main loop as a
/// multiline error. The second half of
/// [`msg_schedule_semsg_multiline_c!`](crate::msg_schedule_semsg_multiline_c).
///
/// # Safety
/// Only that the main loop is live.
#[doc(hidden)]
pub unsafe fn msg_schedule_semsg_multiline_finish(buf: &[c_char; MSG_IOBUFF_LEN]) {
    let s = unsafe { xstrdup(buf.as_ptr()) };
    let event = Event::new(Some(msg_semsg_multiline_event), [s.cast::<c_void>()]);
    unsafe { loop_schedule_deferred(main_loop.ptr(), event) };
}

/// Show a warning, which `'warningmsg'` highlighting and `v:warningmsg` pick
/// up. Repeated after a redraw, unlike an error.
///
/// # Safety
/// `message` must be a valid C string.
pub unsafe fn give_warning(message: *const c_char, hl: bool, hist: bool) {
    // Don't do this for ":silent".
    if msg_silent.get() != 0 {
        return;
    }
    let save_msg_hist_off = msg_hist_off.get();
    msg_hist_off.set(!hist);

    let no_prompt = Suppress::wait_return();
    unsafe { set_vim_var_string(Vv::Warningmsg, message, -1) };
    unsafe { xfree(keep_msg.get().cast()) };
    keep_msg.set(ptr::null_mut());
    keep_msg_hl_id.set(if hl { HLF_W } else { 0 });

    if msg_ext_kind.get().is_null() {
        unsafe { msg_ext_set_kind(c"wmsg".as_ptr()) };
    }
    if unsafe { msg(message, keep_msg_hl_id.get()) } && msg_scrolled.get() == 0 {
        unsafe { set_keep_msg(message, keep_msg_hl_id.get()) };
    }
    msg_didout.set(false); // overwrite this message
    msg_nowait.set(true); // don't wait for this message
    msg_col.set(0);

    drop(no_prompt);
    msg_hist_off.set(save_msg_hist_off);
}

/// Show whatever was formatted into [`msg_iobuff`] as a warning. The second
/// half of [`swmsg_c!`](crate::swmsg_c).
///
/// # Safety
/// Only that the message state is the main thread's.
#[doc(hidden)]
pub unsafe fn swmsg_finish(buf: &[c_char; MSG_IOBUFF_LEN], hl: bool) {
    unsafe { give_warning(buf.as_ptr(), hl, true) }
}
