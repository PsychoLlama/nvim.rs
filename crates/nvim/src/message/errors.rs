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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::event::r#loop::schedule_deferred_job;
use crate::guard::Suppress;
use crate::log::{LOG_DEBUG, logmsg};
use crate::message_fmt::{msg_bytes as text_arg, msg_cstr};
use crate::runtime::{estack_sfile, sourcing_lnum, sourcing_name_copy, with_sourcing_name};
use crate::types::LineNr;
use core::ffi::{CStr, c_int, c_long};
use std::ffi::CString;

/// Forget where the last error came from, so the next one names its source
/// again.
pub fn reset_last_sourcing() {
    LAST_SOURCING_NAME.set(None);
    LAST_SOURCING_LNUM.set(0);
}

/// The innermost script or function's name, when it has one.
fn sourcing_name_text() -> Option<Vec<u8>> {
    sourcing_name_copy()
}

/// Is the innermost script/function a different one from the last error's?
/// False when it has no name.
fn other_sourcing_name() -> bool {
    with_sourcing_name(|name| {
        name.is_some_and(|name| LAST_SOURCING_NAME.with(|last| last.as_deref() != Some(name)))
    })
}

/// The "Error in <script>:" line, or `None` when the source has already been
/// reported.
fn emsg_source_line() -> Option<CString> {
    if !other_sourcing_name() {
        return None;
    }
    let sname = match estack_sfile(ESTACK_NONE) {
        Some(stack) => stack.as_cstr().to_bytes().to_vec(),
        None => sourcing_name_text()?,
    };
    let text = crate::formatted!(usize::MAX, gettext(c"Error in %s:").to_bytes(), &sname[..]);
    Some(CString::new(text).unwrap_or_default())
}

/// The "line NNNN:" line, or `None` when the line has already been reported
/// (or there is none).
fn emsg_lnum_line() -> Option<CString> {
    // Show the source of the error, but not if it is the same as the last
    // time.
    let lnum = sourcing_lnum();
    if !with_sourcing_name(|name| name.is_some())
        || !(other_sourcing_name() || lnum != LineNr::new(LAST_SOURCING_LNUM.get()))
        || lnum == LineNr::ZERO
    {
        return None;
    }
    let text = crate::formatted!(usize::MAX, gettext(c"line %4d:").to_bytes(), lnum);
    Some(CString::new(text).unwrap_or_default())
}

/// Display the source of an error message, if it has not been shown already.
pub fn msg_source(hl_id: c_int) {
    if MSG_SOURCE_BUSY.get() {
        return;
    }
    MSG_SOURCE_BUSY.set(true);

    let no_prompt = Suppress::wait_return();
    if let Some(line) = emsg_source_line() {
        MSG_SCROLL.set(1);
        msg(&line, hl_id);
    }
    if let Some(line) = emsg_lnum_line() {
        msg(&line, HLF_N);
        LAST_SOURCING_LNUM.set(sourcing_lnum().get());
    }

    // Remember the source name and line number, so we can tell when
    // the message changes.
    if !with_sourcing_name(|name| name.is_some()) || other_sourcing_name() {
        LAST_SOURCING_NAME.set(None);
        if let Some(name) = sourcing_name_text() {
            LAST_SOURCING_NAME.set(Some(XString::from_bytes(&name)));
            if !redirecting() {
                msg_putchar_hl(c_int::from(b'\n'), hl_id);
            }
        }
    }
    drop(no_prompt);
    MSG_SOURCE_BUSY.set(false);
}

/// Is this a bad time to show an error?
pub(crate) fn emsg_not_now() -> bool {
    (EMSG_OFF.get() > 0
        && !p_debug(|value| value.to_bytes().contains(&b'm'))
        && !p_debug(|value| value.to_bytes().contains(&b't')))
        || EMSG_SKIP.get() > 0
}

/// Show an error message, possibly spanning several lines.
///
/// Answers true when the message was shown (or deliberately swallowed).
/// `kind` is the `ext_messages` kind; `multiline` keeps embedded newlines
/// rather than escaping them.
pub fn emsg_multiline(s: &CStr, kind: Option<&CStr>, hl_id: c_int, multiline: bool) -> bool {
    if emsg_not_now() {
        return true;
    }
    CALLED_EMSG.set(CALLED_EMSG.get() + 1);

    // Reset the "severe" flag: it applies to this message only.
    let severe = EMSG_SEVERE.get();
    EMSG_SEVERE.set(false);

    if EMSG_OFF.get() == 0 || p_debug(|value| value.to_bytes().contains(&b't')) {
        // Cause a throw of an error exception if appropriate. Don't display
        // the error message in this case.
        let mut ignore = false;
        if cause_errthrow(s, multiline, IS_MULTIHL.get() > 1, severe, &mut ignore) {
            if !ignore {
                DID_EMSG.set(DID_EMSG.get() + 1);
            }
            return true;
        }

        if IN_ASSERT_FAILS.get() && EMSG_ASSERT_FAILS_MSG.with(Option::is_none) {
            EMSG_ASSERT_FAILS_MSG.set(Some(XString::from_cstr(s)));
            EMSG_ASSERT_FAILS_LNUM.set(c_long::from(sourcing_lnum()));
            let context = sourcing_name_text().unwrap_or_default();
            EMSG_ASSERT_FAILS_CONTEXT.set(Some(XString::from_bytes(&context)));
        }

        // set "v:errmsg", also when using ":silent! cmd"
        set_vim_var_string(Vv::Errmsg, Some(s.to_bytes()));

        // When using ":silent! cmd" don't display the error message, but
        // do write it to the redirection and the log.
        if EMSG_SILENT.get() != 0 {
            if !EMSG_NOREDIR.get() {
                msg_start();
                // Each source line is redirected with a newline appended.
                //
                // Called one at a time rather than collected first: the
                // redirection in between can reach `:redir => var`, and
                // the line number's answer depends on state a redirection
                // could in principle move.
                let write_line = |line: Option<CString>| {
                    if let Some(line) = line {
                        let mut text = line.into_bytes();
                        text.push(b'\n');
                        redir_write(&text);
                    }
                };
                write_line(emsg_source_line());
                write_line(emsg_lnum_line());
                redir_write(s.to_bytes());
            }
            if LOG_DEBUG {
                let text = msg_cstr(s);
                let lnum = sourcing_lnum();
                with_sourcing_name(|name| match name {
                    Some(name) if lnum != LineNr::ZERO => {
                        let name = text_arg(name);
                        logmsg!(
                            LOGLVL_DBG,
                            c"emsg_multiline",
                            845,
                            "(:silent) {text} ({name} (line {lnum}))"
                        );
                    }
                    _ => {
                        logmsg!(LOGLVL_DBG, c"emsg_multiline", 847, "(:silent) {text}");
                    }
                });
            }
            return true;
        }

        if LOG_DEBUG {
            let text = msg_cstr(s);
            let lnum = sourcing_lnum();
            with_sourcing_name(|name| match name {
                Some(name) if lnum != LineNr::ZERO => {
                    let name = text_arg(name);
                    logmsg!(
                        LOGLVL_INF,
                        c"emsg_multiline",
                        855,
                        "{text} ({name} (line {lnum}))"
                    );
                }
                _ => {
                    logmsg!(LOGLVL_INF, c"emsg_multiline", 857, "{text}");
                }
            });
        }

        EX_EXITVAL.set(1);

        // Reset msg_silent, an error causes messages to be visible again.
        MSG_SILENT.set(0);
        CMD_SILENT.set(false);

        if GLOBAL_BUSY.get() != 0 {
            // Break out of the :global command.
            GLOBAL_BUSY.set(GLOBAL_BUSY.get() + 1);
        }

        // Now that we have a message, flush the input buffer or beep.
        if p_eb() {
            beep_flush();
        } else {
            flush_buffers(FLUSH_MINIMAL);
        }
        DID_EMSG.set(DID_EMSG.get() + 1);
    }

    EMSG_ON_DISPLAY.set(true); // remember there is an error message
    if MSG_SCROLLED.get() != 0 {
        NEED_WAIT_RETURN.set(true); // needed in case emsg() is called after wait_return() has cleared it
    }
    msg_ext_set_kind_opt(kind);
    MSG_SCROLL.set(1); // don't overwrite a previous message

    // Skip the flush until the whole message has been written, so that the
    // source line and the error arrive as one ext_messages event.
    let save_msg_skip_flush = MSG_EXT_SKIP_FLUSH.get();
    MSG_EXT_SKIP_FLUSH.set(true);
    msg_source(hl_id);
    MSG_NOWAIT.set(false); // wait for this msg
    let rv = show_message(s, hl_id, false, multiline);
    MSG_EXT_SKIP_FLUSH.set(save_msg_skip_flush);
    rv
}

/// Show an error message.
pub fn emsg(s: &CStr) -> bool {
    emsg_multiline(s, Some(c"emsg"), HLF_E, false)
}

/// "E354: Invalid register name" for register `name`.
pub fn emsg_invreg(name: c_int) {
    let display = transchar_buf(None, name);
    let display = msg_cstr(cstr::in_bytes(&display));
    crate::semsg!("E354: Invalid register name: '{}'", display);
}

/// How much of an error message [`semsg!`](crate::semsg) keeps: the size of
/// the buffer the C wrapper formatted into, so the truncation point is the
/// one upstream's `vim_snprintf` chose.
pub const SEMSG_ERRBUF_LEN: size_t = 1025;

/// [`SEMSG_ERRBUF_LEN`] for [`semsg_multiline!`](crate::semsg_multiline). A
/// multiline error can be much longer than one line's worth.
pub const SEMSG_MULTILINE_ERRBUF_LEN: size_t = 8192;

/// An internal error: same as [`emsg`], but skipped when errors are off.
pub fn iemsg(s: &CStr) {
    if emsg_not_now() {
        return;
    }
    emsg(s);
}

/// "E685: Internal error: <where>", for a reached-the-unreachable case.
pub fn internal_error(where_0: &CStr) {
    crate::siemsg!("E685: Internal error: {}", msg_cstr(where_0));
}

/// Report `text` as a multiline error of `ext_messages` kind `kind`, keeping
/// its embedded newlines. [`semsg_multiline!`](crate::semsg_multiline)'s tail.
#[doc(hidden)]
pub(crate) fn emsg_multiline_text(text: &CStr, kind: &CStr) -> bool {
    emsg_multiline(text, Some(kind), HLF_E, true)
}

/// Show `text` as a warning. [`swmsg!`](crate::swmsg)'s tail.
#[doc(hidden)]
pub(crate) fn swmsg_text(text: &CStr, hl: bool) {
    give_warning(text, hl, true);
}

/// Hand `text` to the main loop as an error, `multiline` keeping its embedded
/// newlines. [`msg_schedule_semsg!`](crate::msg_schedule_semsg)'s tail.
#[doc(hidden)]
pub(crate) fn msg_schedule_semsg_text(text: &CStr, multiline: bool) {
    let text = text.to_owned();
    schedule_deferred_job(Box::new(move || {
        emsg_multiline(&text, Some(c"emsg"), HLF_E, multiline);
    }));
}

/// Show a warning, which `'warningmsg'` highlighting and `v:warningmsg` pick
/// up. Repeated after a redraw, unlike an error.
pub fn give_warning(message: &CStr, hl: bool, hist: bool) {
    // Don't do this for ":silent".
    if MSG_SILENT.get() != 0 {
        return;
    }
    let save_msg_hist_off = MSG_HIST_OFF.get();
    MSG_HIST_OFF.set(!hist);

    let no_prompt = Suppress::wait_return();
    set_vim_var_string(Vv::Warningmsg, Some(message.to_bytes()));
    KEEP_MSG.set(None);
    KEEP_MSG_HL_ID.set(if hl { HLF_W } else { 0 });

    if MSG_EXT_KIND.with(ApiString::is_null) {
        msg_ext_set_kind(c"wmsg");
    }
    if msg(message, KEEP_MSG_HL_ID.get()) && MSG_SCROLLED.get() == 0 {
        set_keep_msg(Some(message), KEEP_MSG_HL_ID.get());
    }
    MSG_DIDOUT.set(false); // overwrite this message
    MSG_NOWAIT.set(true); // don't wait for this message
    MSG_COL.set(0);

    drop(no_prompt);
    MSG_HIST_OFF.set(save_msg_hist_off);
}

// The shared message text. Upstream keeps these in `errors.h` as one
// `EXTERN char[]` per message so that a wording appears once however many
// call sites raise it, and c2rust parked the header in `startup/mod.rs`. Only
// the ones raised from more than one place are here; a message with a single
// caller is written at the call.
//
// The name is upstream's and follows the wording, not the error number: the
// `E<n>` code is part of the text.

pub(crate) static E_API_SPAWN_FAILED: &CStr = c"E903: Could not spawn API job";
pub(crate) static E_ARGREQ: &CStr = c"E471: Argument required";
pub(crate) static E_BACKSLASH: &CStr = c"E10: \\ should be followed by /, ? or &";
pub(crate) static E_CMDWIN: &CStr =
    c"E11: Invalid in command-line window; <CR> executes, CTRL-C quits";
pub(crate) static E_CURDIR: &CStr =
    c"E12: Command not allowed in secure mode in current dir or tag search";
pub(crate) static E_COMMAND_TOO_RECURSIVE: &CStr = c"E169: Command too recursive";
pub(crate) static E_BUFFER_IS_NOT_LOADED: &CStr = c"E681: Buffer is not loaded";
pub(crate) static E_ENDIF: &CStr = c"E171: Missing :endif";
pub(crate) static E_ENDTRY: &CStr = c"E600: Missing :endtry";
pub(crate) static E_ENDWHILE: &CStr = c"E170: Missing :endwhile";
pub(crate) static E_ENDFOR: &CStr = c"E170: Missing :endfor";
pub(crate) static E_WHILE: &CStr = c"E588: :endwhile without :while";
pub(crate) static E_FOR: &CStr = c"E588: :endfor without :for";
pub(crate) static E_EXISTS: &CStr = c"E13: File exists (add ! to override)";
pub(crate) static E_FAILED: &CStr = c"E472: Command failed";
pub(crate) static E_INTERN2: &CStr = c"E685: Internal error: %s";
pub(crate) static E_INTERR: &CStr = c"Interrupted";
pub(crate) static E_INVARG: &CStr = c"E474: Invalid argument";
pub(crate) static E_INVARG2: &CStr = c"E475: Invalid argument: %s";
pub(crate) static E_INVARGVAL: &CStr = c"E475: Invalid value for argument %s";
pub(crate) static E_INVRANGE: &CStr = c"E16: Invalid range";
pub(crate) static E_INVCMD: &CStr = c"E476: Invalid command";
pub(crate) static E_NO_SPELL: &CStr = c"E756: Spell checking is not possible";
pub(crate) static E_INVCHAN: &CStr = c"E900: Invalid channel id";
pub(crate) static E_INVCHANJOB: &CStr = c"E900: Invalid channel id: not a job";
pub(crate) static E_CHANNOTPTY: &CStr = c"E904: channel is not a pty";
pub(crate) static E_INVSTREAM: &CStr = c"E906: invalid stream for channel";
pub(crate) static E_INVSTREAMRPC: &CStr = c"E906: invalid stream for rpc channel, use 'rpc'";
pub(crate) static E_FSYNC: &CStr = c"E667: Fsync failed: %s";
pub(crate) static E_MARKINVAL: &CStr = c"E19: Mark has invalid line number";
pub(crate) static E_MARKNOTSET: &CStr = c"E20: Mark not set";
pub(crate) static E_MODIFIABLE: &CStr = c"E21: Cannot make changes, 'modifiable' is off";
pub(crate) static E_NESTING: &CStr = c"E22: Scripts nested too deep";
pub(crate) static E_NOALT: &CStr = c"E23: No alternate file";
pub(crate) static E_NOABBR: &CStr = c"E24: No such abbreviation";
pub(crate) static E_NOBANG: &CStr = c"E477: No ! allowed";
pub(crate) static E_NOINSTEXT: &CStr = c"E29: No inserted text yet";
pub(crate) static E_NOLASTCMD: &CStr = c"E30: No previous command line";
pub(crate) static E_NOMAP: &CStr = c"E31: No such mapping";
pub(crate) static E_NOIDENT: &CStr = c"E349: No identifier under cursor";
pub(crate) static E_NOMATCH: &CStr = c"E479: No match";
pub(crate) static E_NONAME: &CStr = c"E32: No file name";
pub(crate) static E_NOPRESUB: &CStr = c"E33: No previous substitute regular expression";
pub(crate) static E_NOPREV: &CStr = c"E34: No previous command";
pub(crate) static E_NOPREVRE: &CStr = c"E35: No previous regular expression";
pub(crate) static E_NORANGE: &CStr = c"E481: No range allowed";
pub(crate) static E_NOROOM: &CStr = c"E36: Not enough room";
pub(crate) static E_NOTMP: &CStr = c"E483: Can't get temp file name";
pub(crate) static E_CANT_READ_FILE_STR: &CStr = c"E485: Can't read file %s";
pub(crate) static E_NULL: &CStr = c"E38: Null argument";
pub(crate) static E_OUTOFMEM: &CStr = c"E41: Out of memory!";
pub(crate) static E_PATNOTF: &CStr = c"Pattern not found";
pub(crate) static E_POSITIVE: &CStr = c"E487: Argument must be positive";
pub(crate) static E_PREV_DIR: &CStr = c"E459: Cannot go back to previous directory";
pub(crate) static E_NO_ERRORS: &CStr = c"E42: No Errors";
pub(crate) static E_LOCLIST: &CStr = c"E776: No location list";
pub(crate) static E_RE_DAMG: &CStr = c"E43: Damaged match string";
pub(crate) static E_RE_CORR: &CStr = c"E44: Corrupted regexp program";
pub(crate) static E_READONLY: &CStr = c"E45: 'readonly' option is set (add ! to override)";
pub(crate) static E_CANNOT_MOD: &CStr = c"E995: Cannot modify existing variable";
pub(crate) static E_CANNOT_CHANGE_READONLY_VARIABLE_STR: &CStr =
    c"E46: Cannot change read-only variable \"%.*s\"";
pub(crate) static E_DICTREQ: &CStr = c"E715: Dictionary required";
pub(crate) static E_INVALBLOB: &CStr = c"E978: Invalid operation for Blob";
pub(crate) static E_TOOMANYARG: &CStr = c"E118: Too many arguments for function: %s";
pub(crate) static E_TOOFEWARG: &CStr = c"E119: Not enough arguments for function: %s";
pub(crate) static E_LISTREQ: &CStr = c"E714: List required";
pub(crate) static E_LISTBLOBREQ: &CStr = c"E897: List or Blob required";
pub(crate) static E_SANDBOX: &CStr = c"E48: Not allowed in sandbox";
pub(crate) static E_SECURE: &CStr = c"E523: Not allowed here";
pub(crate) static E_TEXTLOCK: &CStr = c"E565: Not allowed to change text or change window";
pub(crate) static E_SCREENMODE: &CStr = c"E359: Screen mode setting not supported";
pub(crate) static E_SCROLL: &CStr = c"E49: Invalid scroll size";
pub(crate) static E_SHELLEMPTY: &CStr = c"E91: 'shell' option is empty";
pub(crate) static E_SWAPCLOSE: &CStr = c"E72: Close error on swap file";
pub(crate) static E_TOOCOMPL: &CStr = c"E74: Command too complex";
pub(crate) static E_LONGNAME: &CStr = c"E75: Name too long";
pub(crate) static E_TOOMANY: &CStr = c"E77: Too many file names";
pub(crate) static E_TRAILING: &CStr = c"E488: Trailing characters";
pub(crate) static E_TRAILING_ARG: &CStr = c"E488: Trailing characters: %s";
pub(crate) static E_UMARK: &CStr = c"E78: Unknown mark";
pub(crate) static E_WILDEXPAND: &CStr = c"E79: Cannot expand wildcards";
pub(crate) static E_WINHEIGHT: &CStr = c"E591: 'winheight' cannot be smaller than 'winminheight'";
pub(crate) static E_WINWIDTH: &CStr = c"E592: 'winwidth' cannot be smaller than 'winminwidth'";
pub(crate) static E_WRITE: &CStr = c"E80: Error while writing";
pub(crate) static E_ZEROCOUNT: &CStr = c"E939: Positive count required";
pub(crate) static E_USINGSID: &CStr = c"E81: Using <SID> not in a script context";
pub(crate) static E_EMPTY_BUFFER: &CStr = c"E749: Empty buffer";
pub(crate) static E_NO_WRITE_SINCE_LAST_CHANGE: &CStr = c"E37: No write since last change";
pub(crate) static E_NO_WRITE_SINCE_LAST_CHANGE_ADD_BANG_TO_OVERRIDE: &CStr =
    c"E37: No write since last change (add ! to override)";
pub(crate) static E_BUFFER_NR_NOT_FOUND: &CStr = c"E92: Buffer %d not found";
pub(crate) static E_UNKNOWN_FUNCTION_STR: &CStr = c"E117: Unknown function: %s";
pub(crate) static E_JOB_STILL_RUNNING: &CStr = c"E948: Job still running";
pub(crate) static E_JOB_STILL_RUNNING_ADD_BANG_TO_END_THE_JOB: &CStr =
    c"E948: Job still running (add ! to end the job)";
pub(crate) static E_INVALPAT: &CStr = c"E682: Invalid search pattern or delimiter";
pub(crate) static E_BUFLOADED: &CStr = c"E139: File is loaded in another buffer";
pub(crate) static E_AU_RECURSIVE: &CStr = c"E952: Autocommand caused recursive behavior";
pub(crate) static E_MENU_ONLY_EXISTS_IN_ANOTHER_MODE: &CStr =
    c"E328: Menu only exists in another mode";
pub(crate) static E_AUTOCMD_CLOSE: &CStr = c"E813: Cannot close autocmd window";
pub(crate) static E_UNSUPPORTEDOPTION: &CStr = c"E519: Option not supported";
pub(crate) static E_FNAMETOOLONG: &CStr = c"E856: Filename too long";
pub(crate) static E_USING_FLOAT_AS_STRING: &CStr = c"E806: Using a Float as a String";
pub(crate) static E_CANNOT_EDIT_OTHER_BUF: &CStr = c"E788: Not allowed to edit another buffer now";
pub(crate) static E_AUABORT: &CStr = c"E855: Autocommands caused command to abort";
pub(crate) static E_FAST_API_DISABLED: &CStr =
    c"E5560: %s must not be called in a fast event context";
pub(crate) static E_FLOATONLY: &CStr =
    c"E5601: Cannot close window, only floating window would remain";
pub(crate) static E_FLOATEXCHANGE: &CStr = c"E5602: Cannot exchange or rotate float";
pub(crate) static E_VALUE_IS_LOCKED: &CStr = c"E741: Value is locked";
pub(crate) static E_VALUE_IS_LOCKED_STR: &CStr = c"E741: Value is locked: %.*s";
pub(crate) static E_CANNOT_CHANGE_VALUE: &CStr = c"E742: Cannot change value";
pub(crate) static E_CANNOT_CHANGE_VALUE_OF_STR: &CStr = c"E742: Cannot change value of %.*s";
pub(crate) static E_CANNOT_SET_VARIABLE_IN_SANDBOX_STR: &CStr =
    c"E794: Cannot set variable in the sandbox: \"%.*s\"";
pub(crate) static E_INVALWINDOW: &CStr = c"E957: Invalid window number";
pub(crate) static E_PROBLEM_CREATING_INTERNAL_DIFF: &CStr =
    c"E960: Problem creating the internal diff";
pub(crate) static E_CANNOT_DEFINE_AUTOCOMMANDS_FOR_ALL_EVENTS: &CStr =
    c"E1155: Cannot define autocommands for ALL events";
pub(crate) static E_RESULTING_TEXT_TOO_LONG: &CStr = c"E1240: Resulting text too long";
pub(crate) static E_LINE_NUMBER_OUT_OF_RANGE: &CStr = c"E1247: Line number out of range";
pub(crate) static E_HIGHLIGHT_GROUP_NAME_INVALID_CHAR: &CStr =
    c"E5248: Invalid character in group name";
pub(crate) static E_HIGHLIGHT_GROUP_NAME_TOO_LONG: &CStr = c"E1249: Highlight group name too long";
pub(crate) static E_STRING_REQUIRED: &CStr = c"E928: String required";
pub(crate) static E_CANNOT_CHANGE_MENUS_WHILE_LISTING: &CStr =
    c"E1310: Cannot change menus while listing";
pub(crate) static E_NOT_ALLOWED_TO_CHANGE_WINDOW_LAYOUT_IN_THIS_AUTOCMD: &CStr =
    c"E1312: Not allowed to change the window layout in this autocmd";
pub(crate) static E_UNDOBANG_CANNOT_REDO_OR_MOVE_BRANCH: &CStr =
    c"E5767: Cannot use :undo! to redo or move to a different undo branch";
pub(crate) static E_WINFIXBUF_CANNOT_GO_TO_BUFFER: &CStr =
    c"E1513: Cannot switch buffer. 'winfixbuf' is enabled";
pub(crate) static E_INVALID_RETURN_TYPE_FROM_FINDFUNC: &CStr =
    c"E1514: 'findfunc' did not return a List type";
pub(crate) static E_CANNOT_SWITCH_TO_A_CLOSING_BUFFER: &CStr =
    c"E1546: Cannot switch to a closing buffer";
pub(crate) static E_FAILED_TO_FIND_ALL_DIFF_ANCHORS: &CStr =
    c"E1550: Failed to find all diff anchors";
pub(crate) static E_DIFF_ANCHORS_WITH_HIDDEN_WINDOWS: &CStr =
    c"E1562: Diff anchors cannot be used with hidden diff windows";
pub(crate) static E_LEADTAB_REQUIRES_TAB: &CStr =
    c"E1572: 'listchars' field \"leadtab\" requires \"tab\" to be specified";
pub(crate) static E_INVALID_FORMAT_STRING_SINGLE_PERCENT_S: &CStr =
    c"E1577: Invalid format string, only one \"%s\" is allowed";
