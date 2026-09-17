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
//! ([`semsg!`](crate::semsg) and friends, defined in
//! [`crate::message_fmt`]): each renders a `format_args!` the compiler has
//! checked, truncates it to the buffer size the C wrapper owned, and hands
//! the bytes to the reporting functions here. Same bytes, same truncation --
//! and no C-variadic definition, which only a nightly compiler can write.
//!
//! The shared message text -- upstream's `errors.h`, every `e_*` wording
//! raised from more than one place -- sits at the end of the file.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::cstr;
use crate::guard::Suppress;
use crate::log::logmsg;
use crate::message_fmt::c_str;
use crate::strings::has_char;
use core::ffi::{CStr, c_char, c_int, c_long, c_void};
use core::ptr;

/// The script/function the last error was reported from.
static last_sourcing_name: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

/// The line the last error was reported from.
static last_sourcing_lnum: GlobalCell<c_int> = GlobalCell::new(0);

/// Forget where the last error came from, so the next one names its source
/// again.
pub fn reset_last_sourcing() {
    unsafe { xfree(last_sourcing_name.get().cast()) };
    last_sourcing_name.set(ptr::null_mut());
    last_sourcing_lnum.set(0);
}

/// Is the innermost script/function a different one from the last error's?
fn other_sourcing_name() -> bool {
    if exestack_has_name() {
        if !last_sourcing_name.get().is_null() {
            return !unsafe { cstr::eq(sourcing_top().es_name, last_sourcing_name.get()) };
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
    if !exestack_has_name() || !other_sourcing_name() {
        return ptr::null_mut();
    }
    let tofree = estack_sfile(ESTACK_NONE);
    let sname = if tofree.is_null() {
        sourcing_top().es_name
    } else {
        tofree
    };
    let p = gettext(c"Error in %s:");
    let buf_len = unsafe { cstr::bytes_at(sname) }.len() + p.count_bytes() + 1;
    let buf: *mut c_char = unsafe { xmalloc(buf_len) }.cast();
    unsafe { snprintf(buf, buf_len, p.as_ptr(), sname) };
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
        || !(other_sourcing_name() || sourcing_top().es_lnum != last_sourcing_lnum.get())
        || sourcing_top().es_lnum == 0
    {
        return ptr::null_mut();
    }
    let p = gettext(c"line %4d:");
    let buf_len = 20 + p.count_bytes();
    let buf: *mut c_char = unsafe { xmalloc(buf_len) }.cast();
    unsafe { snprintf(buf, buf_len, p.as_ptr(), sourcing_top().es_lnum) };
    buf
}

/// Display the source of an error message, if it has not been shown already.
pub fn msg_source(hl_id: c_int) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false);
    if recursive.get() {
        return;
    }
    recursive.set(true);

    let no_prompt = Suppress::wait_return();
    let p = unsafe { get_emsg_source() };
    if !p.is_null() {
        msg_scroll.set(1);
        unsafe { msg_ptr(p, hl_id) };
        unsafe { xfree(p.cast()) };
    }
    let p = unsafe { get_emsg_lnum() };
    if !p.is_null() {
        unsafe { msg_ptr(p, HLF_N) };
        unsafe { xfree(p.cast()) };
        last_sourcing_lnum.set(sourcing_top().es_lnum as c_int);
    }

    // Remember the source name and line number, so we can tell when
    // the message changes.
    if sourcing_top().es_name.is_null() || other_sourcing_name() {
        unsafe { xfree(last_sourcing_name.get().cast()) };
        last_sourcing_name.set(ptr::null_mut());
        if !sourcing_top().es_name.is_null() {
            last_sourcing_name.set(unsafe { xstrdup(sourcing_top().es_name) });
            if !redirecting() {
                msg_putchar_hl(b'\n' as c_int, hl_id);
            }
        }
    }
    drop(no_prompt);
    recursive.set(false);
}

/// Is this a bad time to show an error?
pub(crate) fn emsg_not_now() -> bool {
    (emsg_off.get() > 0
        && !has_char(unsafe { cstr::at(p_debug()) }, b'm' as c_int)
        && !has_char(unsafe { cstr::at(p_debug()) }, b't' as c_int))
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
    if emsg_not_now() {
        return true;
    }
    called_emsg.set(called_emsg.get() + 1);

    // Reset the "severe" flag: it applies to this message only.
    let severe = emsg_severe.get();
    emsg_severe.set(false);

    if emsg_off.get() == 0 || has_char(unsafe { cstr::at(p_debug()) }, b't' as c_int) {
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
                msg_start();
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
                        let len = unsafe { cstr::bytes_at(line) }.len();
                        unsafe { *line.add(len) = b'\n' as c_char };
                        redir_write(unsafe { cstr::slice_at(line, len + 1) });
                        unsafe { xfree(line.cast()) };
                    }
                };
                write_line(unsafe { get_emsg_source() });
                write_line(unsafe { get_emsg_lnum() });
                redir_write(unsafe { cstr::bytes_at(s) });
            }
            // SAFETY: the message being reported, and the exec stack's own
            // name for where it came from -- both NUL-terminated.
            let text = unsafe { c_str(s) };
            if !sourcing_top().es_name.is_null() && sourcing_top().es_lnum != 0 {
                let name = unsafe { c_str(sourcing_top().es_name) };
                let lnum = sourcing_top().es_lnum;
                logmsg!(
                    LOGLVL_DBG,
                    c"emsg_multiline",
                    845,
                    "(:silent) {text} ({name} (line {lnum}))"
                );
            } else {
                logmsg!(LOGLVL_DBG, c"emsg_multiline", 847, "(:silent) {text}");
            }
            return true;
        }

        // SAFETY: as above.
        let text = unsafe { c_str(s) };
        if !sourcing_top().es_name.is_null() && sourcing_top().es_lnum != 0 {
            let name = unsafe { c_str(sourcing_top().es_name) };
            let lnum = sourcing_top().es_lnum;
            logmsg!(
                LOGLVL_INF,
                c"emsg_multiline",
                855,
                "{text} ({name} (line {lnum}))"
            );
        } else {
            logmsg!(LOGLVL_INF, c"emsg_multiline", 857, "{text}");
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
        if p_eb() {
            beep_flush();
        } else {
            flush_buffers(FLUSH_MINIMAL);
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
    msg_source(hl_id);
    msg_nowait.set(false); // wait for this msg
    let rv = unsafe { msg_keep(s, hl_id, false, multiline) };
    msg_ext_skip_flush.set(save_msg_skip_flush);
    rv
}

/// Show an error message.
pub fn emsg(s: &CStr) -> bool {
    // SAFETY: a `CStr` is a valid C string, which is the whole contract.
    unsafe { emsg_multiline(s.as_ptr(), c"emsg".as_ptr(), HLF_E, false) }
}

/// [`emsg`] for a message still held as a raw pointer.
///
/// # Safety
/// `s` must be a valid C string.
pub(crate) unsafe fn emsg_ptr(s: *const c_char) -> bool {
    // SAFETY: the caller's contract.
    unsafe { emsg_multiline(s, c"emsg".as_ptr(), HLF_E, false) }
}

/// "E354: Invalid register name" for register `name`.
pub fn emsg_invreg(name: c_int) {
    let display = transchar_buf(None, name);
    unsafe { crate::semsg!("E354: Invalid register name: '{}'", c_str(display.as_ptr())) };
}

/// How much of an error message [`semsg!`](crate::semsg) keeps: the size of
/// the buffer the C wrapper formatted into, so the truncation point is the
/// one upstream's `vim_snprintf` chose.
pub const SEMSG_ERRBUF_LEN: size_t = 1025;

/// [`SEMSG_ERRBUF_LEN`] for [`semsg_multiline!`](crate::semsg_multiline). A
/// multiline error can be much longer than one line's worth.
pub const SEMSG_MULTILINE_ERRBUF_LEN: size_t = 8192;

/// [`iemsg`] for a message still held as a raw pointer.
///
/// # Safety
/// `s` must be a valid C string.
pub(crate) unsafe fn iemsg_ptr(s: *const c_char) {
    // SAFETY: reads message-state globals on the main thread, as every
    // message call does.
    if emsg_not_now() {
        return;
    }
    // SAFETY: the caller's contract.
    unsafe { emsg_ptr(s) };
}

/// An internal error: same as [`emsg`], but skipped when errors are off.
pub fn iemsg(s: &CStr) {
    if emsg_not_now() {
        return;
    }
    emsg(s);
}

/// "E5555: API call: <where>", for a reached-the-unreachable case.
///
/// # Safety
/// `where_0` must be a valid C string.
pub unsafe fn internal_error(where_0: *const c_char) {
    unsafe { crate::siemsg!("E685: Internal error: {}", c_str(where_0)) };
}

/// Report `text` as a multiline error of `ext_messages` kind `kind`, keeping
/// its embedded newlines. [`semsg_multiline!`](crate::semsg_multiline)'s tail.
#[doc(hidden)]
pub(crate) fn emsg_multiline_text(text: &CStr, kind: &CStr) -> bool {
    // SAFETY: both are valid C strings, which is the whole contract.
    unsafe { emsg_multiline(text.as_ptr(), kind.as_ptr(), HLF_E, true) }
}

/// Show `text` as a warning. [`swmsg!`](crate::swmsg)'s tail.
#[doc(hidden)]
pub(crate) fn swmsg_text(text: &CStr, hl: bool) {
    // SAFETY: a `CStr` is a valid C string, which is the whole contract.
    unsafe { give_warning(text.as_ptr(), hl, true) }
}

/// Hand `text` to the main loop as an error, `multiline` keeping its embedded
/// newlines. [`msg_schedule_semsg!`](crate::msg_schedule_semsg)'s tail.
#[doc(hidden)]
pub(crate) fn msg_schedule_semsg_text(text: &CStr, multiline: bool) {
    let handler = if multiline {
        msg_semsg_multiline_event
    } else {
        msg_semsg_event
    };
    // SAFETY: `text` is a valid C string; the copy is owned by the event,
    // whose handler frees it, and the main loop is live wherever a message
    // can be scheduled.
    unsafe {
        let event = Event::new(Some(handler), [xstrdup(text.as_ptr()).cast::<c_void>()]);
        loop_schedule_deferred(main_loop.ptr(), event);
    }
}

/// Deferred-event handler for [`msg_schedule_semsg`].
///
/// # Safety
/// `argv[0]` must be an allocated C string this call takes ownership of.
pub(crate) unsafe extern "C" fn msg_semsg_event(argv: *mut *mut c_void) {
    let s: *mut c_char = unsafe { (*argv).cast() };
    unsafe { emsg_ptr(s) };
    unsafe { xfree(s.cast()) };
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

    if msg_ext_kind.with(String_0::is_null) {
        unsafe { msg_ext_set_kind(c"wmsg".as_ptr()) };
    }
    if unsafe { msg_ptr(message, keep_msg_hl_id.get()) } && msg_scrolled.get() == 0 {
        unsafe { set_keep_msg(message, keep_msg_hl_id.get()) };
    }
    msg_didout.set(false); // overwrite this message
    msg_nowait.set(true); // don't wait for this message
    msg_col.set(0);

    drop(no_prompt);
    msg_hist_off.set(save_msg_hist_off);
}

// The shared message text. Upstream keeps these in `errors.h` as one
// `EXTERN char[]` per message so that a wording appears once however many
// call sites raise it, and c2rust parked the header in `startup/mod.rs`. Only
// the ones raised from more than one place are here; a message with a single
// caller is written at the call.
//
// The name is upstream's and follows the wording, not the error number: the
// `E<n>` code is part of the text.

pub(crate) static e_api_spawn_failed: &CStr = c"E903: Could not spawn API job";
pub(crate) static e_argreq: &CStr = c"E471: Argument required";
pub(crate) static e_backslash: &CStr = c"E10: \\ should be followed by /, ? or &";
pub(crate) static e_cmdwin: &CStr =
    c"E11: Invalid in command-line window; <CR> executes, CTRL-C quits";
pub(crate) static e_curdir: &CStr =
    c"E12: Command not allowed in secure mode in current dir or tag search";
pub(crate) static e_command_too_recursive: &CStr = c"E169: Command too recursive";
pub(crate) static e_buffer_is_not_loaded: &CStr = c"E681: Buffer is not loaded";
pub(crate) static e_endif: &CStr = c"E171: Missing :endif";
pub(crate) static e_endtry: &CStr = c"E600: Missing :endtry";
pub(crate) static e_endwhile: &CStr = c"E170: Missing :endwhile";
pub(crate) static e_endfor: &CStr = c"E170: Missing :endfor";
pub(crate) static e_while: &CStr = c"E588: :endwhile without :while";
pub(crate) static e_for: &CStr = c"E588: :endfor without :for";
pub(crate) static e_exists: &CStr = c"E13: File exists (add ! to override)";
pub(crate) static e_failed: &CStr = c"E472: Command failed";
pub(crate) static e_intern2: &CStr = c"E685: Internal error: %s";
pub(crate) static e_interr: &CStr = c"Interrupted";
pub(crate) static e_invarg: &CStr = c"E474: Invalid argument";
pub(crate) static e_invarg2: &CStr = c"E475: Invalid argument: %s";
pub(crate) static e_invargval: &CStr = c"E475: Invalid value for argument %s";
pub(crate) static e_invargNval: &CStr = c"E475: Invalid value for argument %s: %s";
pub(crate) static e_invexpr2: &CStr = c"E15: Invalid expression: \"%s\"";
pub(crate) static e_invrange: &CStr = c"E16: Invalid range";
pub(crate) static e_invcmd: &CStr = c"E476: Invalid command";
pub(crate) static e_isadir2: &CStr = c"E17: \"%s\" is a directory";
pub(crate) static e_no_spell: &CStr = c"E756: Spell checking is not possible";
pub(crate) static e_invchan: &CStr = c"E900: Invalid channel id";
pub(crate) static e_invchanjob: &CStr = c"E900: Invalid channel id: not a job";
pub(crate) static e_channotpty: &CStr = c"E904: channel is not a pty";
pub(crate) static e_invstream: &CStr = c"E906: invalid stream for channel";
pub(crate) static e_invstreamrpc: &CStr = c"E906: invalid stream for rpc channel, use 'rpc'";
pub(crate) static e_fsync: &CStr = c"E667: Fsync failed: %s";
pub(crate) static e_mkdir: &CStr = c"E739: Cannot create directory %s: %s";
pub(crate) static e_markinval: &CStr = c"E19: Mark has invalid line number";
pub(crate) static e_marknotset: &CStr = c"E20: Mark not set";
pub(crate) static e_modifiable: &CStr = c"E21: Cannot make changes, 'modifiable' is off";
pub(crate) static e_nesting: &CStr = c"E22: Scripts nested too deep";
pub(crate) static e_noalt: &CStr = c"E23: No alternate file";
pub(crate) static e_noabbr: &CStr = c"E24: No such abbreviation";
pub(crate) static e_nobang: &CStr = c"E477: No ! allowed";
pub(crate) static e_noinstext: &CStr = c"E29: No inserted text yet";
pub(crate) static e_nolastcmd: &CStr = c"E30: No previous command line";
pub(crate) static e_nomap: &CStr = c"E31: No such mapping";
pub(crate) static e_noident: &CStr = c"E349: No identifier under cursor";
pub(crate) static e_nomatch: &CStr = c"E479: No match";
pub(crate) static e_noname: &CStr = c"E32: No file name";
pub(crate) static e_nopresub: &CStr = c"E33: No previous substitute regular expression";
pub(crate) static e_noprev: &CStr = c"E34: No previous command";
pub(crate) static e_noprevre: &CStr = c"E35: No previous regular expression";
pub(crate) static e_norange: &CStr = c"E481: No range allowed";
pub(crate) static e_noroom: &CStr = c"E36: Not enough room";
pub(crate) static e_notmp: &CStr = c"E483: Can't get temp file name";
pub(crate) static e_notopen: &CStr = c"E484: Can't open file %s";
pub(crate) static e_cant_read_file_str: &CStr = c"E485: Can't read file %s";
pub(crate) static e_null: &CStr = c"E38: Null argument";
pub(crate) static e_outofmem: &CStr = c"E41: Out of memory!";
pub(crate) static e_patnotf: &CStr = c"Pattern not found";
pub(crate) static e_patnotf2: &CStr = c"E486: Pattern not found: %s";
pub(crate) static e_positive: &CStr = c"E487: Argument must be positive";
pub(crate) static e_prev_dir: &CStr = c"E459: Cannot go back to previous directory";
pub(crate) static e_no_errors: &CStr = c"E42: No Errors";
pub(crate) static e_loclist: &CStr = c"E776: No location list";
pub(crate) static e_re_damg: &CStr = c"E43: Damaged match string";
pub(crate) static e_re_corr: &CStr = c"E44: Corrupted regexp program";
pub(crate) static e_readonly: &CStr = c"E45: 'readonly' option is set (add ! to override)";
pub(crate) static e_cannot_mod: &CStr = c"E995: Cannot modify existing variable";
pub(crate) static e_cannot_change_readonly_variable_str: &CStr =
    c"E46: Cannot change read-only variable \"%.*s\"";
pub(crate) static e_dictreq: &CStr = c"E715: Dictionary required";
pub(crate) static e_invalblob: &CStr = c"E978: Invalid operation for Blob";
pub(crate) static e_toomanyarg: &CStr = c"E118: Too many arguments for function: %s";
pub(crate) static e_toofewarg: &CStr = c"E119: Not enough arguments for function: %s";
pub(crate) static e_listreq: &CStr = c"E714: List required";
pub(crate) static e_listblobreq: &CStr = c"E897: List or Blob required";
pub(crate) static e_listblobarg: &CStr = c"E899: Argument of %s must be a List or Blob";
pub(crate) static e_listdictarg: &CStr = c"E712: Argument of %s must be a List or Dictionary";
pub(crate) static e_listdictblobarg: &CStr =
    c"E896: Argument of %s must be a List, Dictionary or Blob";
pub(crate) static e_readerrf: &CStr = c"E47: Error while reading errorfile";
pub(crate) static e_sandbox: &CStr = c"E48: Not allowed in sandbox";
pub(crate) static e_secure: &CStr = c"E523: Not allowed here";
pub(crate) static e_textlock: &CStr = c"E565: Not allowed to change text or change window";
pub(crate) static e_screenmode: &CStr = c"E359: Screen mode setting not supported";
pub(crate) static e_scroll: &CStr = c"E49: Invalid scroll size";
pub(crate) static e_shellempty: &CStr = c"E91: 'shell' option is empty";
pub(crate) static e_swapclose: &CStr = c"E72: Close error on swap file";
pub(crate) static e_toocompl: &CStr = c"E74: Command too complex";
pub(crate) static e_longname: &CStr = c"E75: Name too long";
pub(crate) static e_toomany: &CStr = c"E77: Too many file names";
pub(crate) static e_trailing: &CStr = c"E488: Trailing characters";
pub(crate) static e_trailing_arg: &CStr = c"E488: Trailing characters: %s";
pub(crate) static e_umark: &CStr = c"E78: Unknown mark";
pub(crate) static e_wildexpand: &CStr = c"E79: Cannot expand wildcards";
pub(crate) static e_winheight: &CStr = c"E591: 'winheight' cannot be smaller than 'winminheight'";
pub(crate) static e_winwidth: &CStr = c"E592: 'winwidth' cannot be smaller than 'winminwidth'";
pub(crate) static e_write: &CStr = c"E80: Error while writing";
pub(crate) static e_zerocount: &CStr = c"E939: Positive count required";
pub(crate) static e_usingsid: &CStr = c"E81: Using <SID> not in a script context";
pub(crate) static e_empty_buffer: &CStr = c"E749: Empty buffer";
pub(crate) static e_no_write_since_last_change: &CStr = c"E37: No write since last change";
pub(crate) static e_no_write_since_last_change_add_bang_to_override: &CStr =
    c"E37: No write since last change (add ! to override)";
pub(crate) static e_buffer_nr_not_found: &CStr = c"E92: Buffer %d not found";
pub(crate) static e_unknown_function_str: &CStr = c"E117: Unknown function: %s";
pub(crate) static e_job_still_running: &CStr = c"E948: Job still running";
pub(crate) static e_job_still_running_add_bang_to_end_the_job: &CStr =
    c"E948: Job still running (add ! to end the job)";
pub(crate) static e_invalpat: &CStr = c"E682: Invalid search pattern or delimiter";
pub(crate) static e_bufloaded: &CStr = c"E139: File is loaded in another buffer";
pub(crate) static e_au_recursive: &CStr = c"E952: Autocommand caused recursive behavior";
pub(crate) static e_menu_only_exists_in_another_mode: &CStr =
    c"E328: Menu only exists in another mode";
pub(crate) static e_autocmd_close: &CStr = c"E813: Cannot close autocmd window";
pub(crate) static e_list_index_out_of_range_nr: &CStr = c"E684: List index out of range: %ld";
pub(crate) static e_unsupportedoption: &CStr = c"E519: Option not supported";
pub(crate) static e_fnametoolong: &CStr = c"E856: Filename too long";
pub(crate) static e_using_float_as_string: &CStr = c"E806: Using a Float as a String";
pub(crate) static e_cannot_edit_other_buf: &CStr = c"E788: Not allowed to edit another buffer now";
pub(crate) static e_auabort: &CStr = c"E855: Autocommands caused command to abort";
pub(crate) static e_fast_api_disabled: &CStr =
    c"E5560: %s must not be called in a fast event context";
pub(crate) static e_floatonly: &CStr =
    c"E5601: Cannot close window, only floating window would remain";
pub(crate) static e_floatexchange: &CStr = c"E5602: Cannot exchange or rotate float";
pub(crate) static e_cant_find_directory_str_in_cdpath: &CStr =
    c"E344: Can't find directory \"%s\" in cdpath";
pub(crate) static e_cant_find_file_str_in_path: &CStr = c"E345: Can't find file \"%s\" in path";
pub(crate) static e_no_more_directory_str_found_in_cdpath: &CStr =
    c"E346: No more directory \"%s\" found in cdpath";
pub(crate) static e_no_more_file_str_found_in_path: &CStr =
    c"E347: No more file \"%s\" found in path";
pub(crate) static e_value_is_locked: &CStr = c"E741: Value is locked";
pub(crate) static e_value_is_locked_str: &CStr = c"E741: Value is locked: %.*s";
pub(crate) static e_cannot_change_value: &CStr = c"E742: Cannot change value";
pub(crate) static e_cannot_change_value_of_str: &CStr = c"E742: Cannot change value of %.*s";
pub(crate) static e_cannot_set_variable_in_sandbox_str: &CStr =
    c"E794: Cannot set variable in the sandbox: \"%.*s\"";
pub(crate) static e_invalwindow: &CStr = c"E957: Invalid window number";
pub(crate) static e_problem_creating_internal_diff: &CStr =
    c"E960: Problem creating the internal diff";
pub(crate) static e_cannot_define_autocommands_for_all_events: &CStr =
    c"E1155: Cannot define autocommands for ALL events";
pub(crate) static e_resulting_text_too_long: &CStr = c"E1240: Resulting text too long";
pub(crate) static e_line_number_out_of_range: &CStr = c"E1247: Line number out of range";
pub(crate) static e_highlight_group_name_invalid_char: &CStr =
    c"E5248: Invalid character in group name";
pub(crate) static e_highlight_group_name_too_long: &CStr = c"E1249: Highlight group name too long";
pub(crate) static e_string_required: &CStr = c"E928: String required";
pub(crate) static e_cannot_change_menus_while_listing: &CStr =
    c"E1310: Cannot change menus while listing";
pub(crate) static e_not_allowed_to_change_window_layout_in_this_autocmd: &CStr =
    c"E1312: Not allowed to change the window layout in this autocmd";
pub(crate) static e_undobang_cannot_redo_or_move_branch: &CStr =
    c"E5767: Cannot use :undo! to redo or move to a different undo branch";
pub(crate) static e_winfixbuf_cannot_go_to_buffer: &CStr =
    c"E1513: Cannot switch buffer. 'winfixbuf' is enabled";
pub(crate) static e_invalid_return_type_from_findfunc: &CStr =
    c"E1514: 'findfunc' did not return a List type";
pub(crate) static e_cannot_switch_to_a_closing_buffer: &CStr =
    c"E1546: Cannot switch to a closing buffer";
pub(crate) static e_failed_to_find_all_diff_anchors: &CStr =
    c"E1550: Failed to find all diff anchors";
pub(crate) static e_diff_anchors_with_hidden_windows: &CStr =
    c"E1562: Diff anchors cannot be used with hidden diff windows";
pub(crate) static e_leadtab_requires_tab: &CStr =
    c"E1572: 'listchars' field \"leadtab\" requires \"tab\" to be specified";
pub(crate) static e_invalid_format_string_single_percent_s: &CStr =
    c"E1577: Invalid format string, only one \"%s\" is allowed";
