//! Running the user's `'shell'`.
//!
//! # Boundary
//!
//! Nothing here talks to the OS: the child is spawned and pumped by
//! `crate::shell::system`, and its output reaches the screen through the
//! message layer ([`throttle`]). What this module owns is everything
//! *around* that -- turning `'shell'`, `'shellcmdflag'`, `'shellxquote'`
//! and `'shellxescape'` into an argument vector, deciding where the output
//! goes, and pushing it into the current buffer for `:r !cmd`.
//!
//! - [`Argv`]: the argument vector.
//! - [`run_shell`] / [`call_shell`]: run a command, honouring the
//!   [`ShellOpts`] below. `call_shell` is the one that also does
//!   `'verbose'`, `:profile` and `v:shell_error`.
//! - [`get_cmd_output`]: run a command and hand back its stdout.
//! - [`expand`]: wildcard expansion, which is a shell command too.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

mod expand;
pub mod system;
pub(crate) mod throttle;

use crate::ascii::NL;
use crate::eval::vars::set_vim_var_nr;
use crate::ex_cmds::{check_secure, make_filter_cmd};
use crate::fileio::temp_name;
use crate::guard::Suppress;
use crate::memline::ml_append_terminated;
use crate::memory::{ThinCString, XString};
use crate::message::state::EMSG_SILENT;
use crate::message::{E_NOTMP, E_SHELLEMPTY};
use crate::message::{
    msg_ext_set_kind, msg_outnum, msg_putchar, msg_str, verbose_enter, verbose_leave,
};
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::option::vars::{P_SH, P_SHCF, P_SXE, P_SXQ, p_sh, p_verbose};
use crate::os::cshim::gettext;
use crate::os::fs::os_remove;
use crate::os::signal::{signal_accept_deadly, signal_reject_deadly};
use crate::profile::{DO_PROFILING, PROF_YES};
use crate::profile::{prof_child_enter, prof_child_exit};
use crate::shell::system::do_os_system;
use crate::smsg;
use crate::state::MODE_EXTERNCMD;
use crate::state::mode::STATE;
use crate::strings::{escaped_with, unquoted};
use crate::tag::tag_freematch;
use crate::tr;
use crate::types::{Failed, LineNr, ProfTime, VarNumber, Vv};
use crate::ui::state::UI_MESSAGES;
use crate::ui::{ui_flush, ui_has};
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_int};
pub(crate) use expand::shell_expand_wildcards;
use std::os::unix::ffi::OsStrExt;

crate::flag_set! {
    /// How a shell command's input and output are wired up — upstream's
    /// `ShellOpts` enum, whose values are always combined with `|`.
    pub struct ShellOpts;

    /// The command filters the buffer (`:{range}!cmd`). Only `ex_cmds/`
    /// and `diff/` set it, and only [`call_shell`]'s `'shelltemp'` path
    /// and the `:diff` writer read it back.
    const FILTER = 1;
    /// The command is a wildcard expansion, not something the user typed:
    /// no mode change, no output forwarding.
    const EXPAND = 2;
    /// The caller has already redirected the output somewhere.
    const DO_OUT = 4;
    /// Do not report a non-zero exit code.
    const SILENT = 8;
    /// Capture the output and write it into the editor.
    const READ = 16;
    /// Feed the current buffer to the command's standard input.
    const WRITE = 32;
    /// Run without any message at all, not even the mode change.
    const HIDE_MESS = 64;
}

/// Characters a shell would act on, so they get a backslash when a pattern is
/// pasted into a command line.
const SHELL_SPECIAL: &[u8] = b"\t \"&'$;<>()\\|\n";

/// An owned argument vector, the program first: what a child process is
/// started with. `crate::shell::system` lays it out as the NULL-terminated
/// `char **` libuv wants when the child is spawned.
#[derive(Debug)]
pub struct Argv(Vec<ThinCString>);

impl Argv {
    /// `'shell'` running `cmd` through `'shellcmdflag'`.
    pub fn shell(cmd: &CStr) -> Self {
        Self::shell_with(Some(cmd.to_bytes()), None)
    }

    /// `'shell'` on its own, as an interactive shell runs.
    pub fn interactive_shell() -> Self {
        Self::shell_with(None, None)
    }

    /// The argument vector for running `'shell'` with an optional command
    /// prefixed by `'shellcmdflag'`, e.g.
    /// `["shell", "-extra_args", "-shellcmdflag", "command with spaces"]`.
    pub fn shell_with(cmd: Option<&[u8]>, extra_args: Option<&CStr>) -> Self {
        let mut words = tokenize(&P_SH.get());
        if let Some(extra_args) = extra_args {
            words.push(ThinCString::from_cstr(extra_args));
        }
        if let Some(cmd) = cmd {
            words.extend(tokenize(&P_SHCF.get()));
            words.push(shell_xescape_xquote(cmd));
        }
        debug_assert!(!words.is_empty());
        Argv(words)
    }

    /// `words`, in order, run as they are.
    ///
    /// # Panics
    /// When `words` is empty: a vector names at least its program.
    pub fn from_words(words: Vec<ThinCString>) -> Self {
        assert!(!words.is_empty(), "an argument vector names its program");
        Argv(words)
    }

    /// Every word, the program first.
    pub fn words(&self) -> impl Iterator<Item = &CStr> {
        self.0.iter().map(ThinCString::as_cstr)
    }

    /// The program, `argv[0]` -- empty only for an interactive shell with
    /// an empty `'shell'`, which upstream asserts never happens.
    pub fn program(&self) -> &CStr {
        self.0.first().map_or(c"", ThinCString::as_cstr)
    }

    /// The words quoted and joined, for reporting. Truncated with an
    /// ellipsis if it does not fit in 256 bytes.
    pub fn to_display(&self) -> XString {
        /// The buffer upstream rendered into, kept because the ellipsis is
        /// placed by it: a result that does not fit ends at byte 255 with
        /// the three bytes before it replaced.
        const MAXSIZE: usize = 256;
        let mut rendered = XString::new();
        for word in self.words() {
            rendered.push_byte(b'\'');
            rendered.push_bytes(word.to_bytes());
            rendered.push_bytes(b"' ");
        }
        if rendered.is_empty() {
            return rendered;
        }
        if rendered.len() < MAXSIZE {
            // Drop the trailing space.
            rendered.truncate(rendered.len() - 1);
        } else {
            // Too long: "/bin/bash 'foo' 'bar'..."
            rendered.truncate(MAXSIZE - 4);
            rendered.push_str("...");
        }
        rendered
    }

    /// The words, for the spawner that lays them out for libuv.
    pub(crate) fn into_words(self) -> Vec<ThinCString> {
        self.0
    }
}

/// Run `cmd` through `'shell'`, or start an interactive shell when it is
/// `None`. `opts` says how the command's streams are wired up.
///
/// Answers the command's exit code.
pub(crate) fn run_shell(cmd: Option<&CStr>, opts: ShellOpts, extra_args: Option<&CStr>) -> c_int {
    let mut input: Vec<u8> = Vec::new();
    let mut capture = false;
    let current_state = STATE.get();
    let mut forward_output = true;

    // Terminating signals are ignored while the child runs.
    signal_reject_deadly();

    if opts.has(ShellOpts::HIDE_MESS | ShellOpts::EXPAND) {
        forward_output = false;
    } else {
        STATE.set(MODE_EXTERNCMD);
        if opts.has(ShellOpts::WRITE) {
            read_input(&mut input);
        }
        if opts.has(ShellOpts::READ) {
            capture = true;
            forward_output = false;
        } else if opts.has(ShellOpts::DO_OUT) {
            // The caller has already redirected the output.
            forward_output = false;
        }
    }

    let argv = Argv::shell_with(cmd.map(CStr::to_bytes), extra_args);
    let (exitcode, output) = do_os_system(
        argv,
        &input,
        capture,
        EMSG_SILENT.get() != 0,
        forward_output,
    );
    drop(input);

    if let Some(mut output) = output {
        write_output(&mut output, true);
    }

    if EMSG_SILENT.get() == 0 && exitcode != 0 && !opts.has(ShellOpts::SILENT) {
        msg_ext_set_kind(c"shell_ret");
        if !ui_has(UI_MESSAGES) {
            msg_putchar(c_int::from(NL));
        }
        msg_str(gettext(c"shell returned "));
        msg_outnum(exitcode);
    }

    STATE.set(current_state);
    signal_accept_deadly();
    exitcode
}

/// Why the shell did not run a command, or its output could not be read
/// back. Said once, by [`call_shell`] or [`get_cmd_output`].
#[derive(Debug, thiserror::Error)]
enum ShellError {
    /// E91: 'shell' is empty.
    #[error("{}", msg_cstr(gettext(E_SHELLEMPTY)))]
    EmptyShell,
    /// E483: no temp file name for the output.
    #[error("{}", msg_cstr(gettext(E_NOTMP)))]
    NoTempFile,
    /// The output file could not be opened or measured.
    #[error("{}", tr!("E282: Cannot read from \"{}\"", msg_cstr(.0.as_cstr())))]
    CannotRead(ThinCString),
    /// The output file read short.
    #[error("{}", tr!("E485: Can't read file {}", msg_cstr(.0.as_cstr())))]
    ReadShort(ThinCString),
}

const _: () = assert!(size_of::<ShellError>() <= 16);

impl ShellError {
    /// Puts the message out.
    fn report(&self) -> Failed {
        emsg_text(self.to_string());
        Failed
    }
}

/// [`call_shell`] of a command line the caller holds, with no extra shell
/// argument, for a caller that reads no exit status.
pub(crate) fn call_shell_text(cmd: &CStr, opts: ShellOpts) {
    let _ = call_shell(Some(cmd), opts, None);
}

/// [`run_shell`] plus the editor-level bookkeeping: `'verbose'`,
/// `:profile`, `v:shell_error`, and invalidating the cached tags.
///
/// Answers the command's exit status, or `Err` -- said, and with
/// `v:shell_error` at -1 -- when 'shell' is empty.
pub(crate) fn call_shell(
    cmd: Option<&CStr>,
    opts: ShellOpts,
    extra_shell_arg: Option<&CStr>,
) -> Result<c_int, Failed> {
    let mut wait_time: ProfTime = 0;
    if p_verbose() > 3 {
        verbose_enter();
        let shell = P_SH.get();
        smsg!(
            0,
            "Executing command: \"{}\"",
            msg_cstr(cmd.unwrap_or(shell.as_cstr()))
        );
        msg_putchar(c_int::from(NL));
        verbose_leave();
    }

    if DO_PROFILING.get() == PROF_YES {
        wait_time = prof_child_enter();
    }

    let retval = if p_sh(CStr::is_empty) {
        Err(ShellError::EmptyShell.report())
    } else {
        // The command may have updated a tags file.
        tag_freematch();
        Ok(run_shell(cmd, opts, extra_shell_arg))
    };

    set_vim_var_nr(
        Vv::ShellError,
        VarNumber::from(*retval.as_ref().unwrap_or(&-1)),
    );
    if DO_PROFILING.get() == PROF_YES {
        prof_child_exit(wait_time);
    }
    retval
}

/// The stdout of an external command, or `Err` having said why not. Every
/// NUL byte in the output becomes SOH, because the answer is used as a C
/// string.
pub(crate) fn get_cmd_output(cmd: &CStr, flags: ShellOpts) -> Result<XString, Failed> {
    if check_secure() {
        return Err(Failed);
    }
    let tempname = temp_name()
        .ok_or(ShellError::NoTempFile)
        .map_err(|e| e.report())?;

    // Add the redirection, and run it. Errors are ignored, and timestamps
    // deliberately not checked.
    let command = make_filter_cmd(cmd, None, Some(tempname.as_cstr()), false);
    let unchecked = Suppress::timestamp_checks();
    let _ = call_shell(
        Some(command.as_c_str()),
        ShellOpts::DO_OUT | ShellOpts::EXPAND | flags,
        None,
    );
    drop(unchecked);
    drop(command);

    let mut buffer = read_output(tempname.as_cstr()).map_err(|e| e.report())?;
    // A NUL would truncate the string, so it becomes SOH instead.
    for b in &mut buffer {
        if *b == 0 {
            *b = 1;
        }
    }
    Ok(XString::from_vec(buffer))
}

/// Read back what [`get_cmd_output`]'s command redirected into `tempname`.
fn read_output(tempname: &CStr) -> Result<Vec<u8>, ShellError> {
    use std::io::{Read, Seek, SeekFrom};
    // Not being able to seek means the file cannot be read.
    let opened = std::fs::File::open(std::ffi::OsStr::from_bytes(tempname.to_bytes()))
        .and_then(|mut file| Ok((file.seek(SeekFrom::End(0))?, file)));
    let Ok((len, mut file)) = opened.and_then(|(len, mut file)| {
        file.rewind()?;
        Ok((len, file))
    }) else {
        return Err(ShellError::CannotRead(ThinCString::from_cstr(tempname)));
    };
    let mut buffer = Vec::new();
    let read = file.by_ref().take(len).read_to_end(&mut buffer);
    drop(file);
    os_remove(tempname);
    if read.ok() != usize::try_from(len).ok() {
        return Err(ShellError::ReadShort(ThinCString::from_cstr(tempname)));
    }
    Ok(buffer)
}

/// Split a command string into words, respecting double quotes; each word
/// is unquoted.
fn tokenize(text: &[u8]) -> Vec<ThinCString> {
    let mut words = Vec::new();
    let mut at = 0;
    while at < text.len() {
        let len = word_length(&text[at..]);
        words.push(ThinCString::from(unquoted(&text[at..at + len])));
        at += len;
        while at < text.len() && (text[at] == b' ' || text[at] == b'\t') {
            at += 1;
        }
    }
    words
}

/// How long the shell word at the start of `bytes` is.
fn word_length(bytes: &[u8]) -> usize {
    let mut inquote = false;
    let mut at = 0;
    // Advance while inside a quote, or on a non-whitespace character.
    while at < bytes.len() && (inquote || (bytes[at] != b' ' && bytes[at] != b'\t')) {
        if bytes[at] == b'"' {
            inquote = !inquote;
        } else if bytes[at] == b'\\' && inquote {
            // The escaped byte is part of the word whatever it is.
            at += 1;
        }
        at += 1;
    }
    at.min(bytes.len())
}

/// Copy the range `:!` was given into `buf`, before the event loop starts.
///
/// The whole text is copied up front rather than written in `ml_get` chunks,
/// because reading from the child can trigger a change to the very buffer
/// still being written from.
fn read_input(buf: &mut Vec<u8>) {
    let (start, end) = (Buf::current().b_op_start.lnum, Buf::current().b_op_end.lnum);
    Buf::current().read_into(start, end, buf);
}

/// Append the child's output to the current buffer, one line per newline
/// (CRLF too, except in binary mode, where the CR is kept). A NUL becomes a
/// newline in the text.
///
/// Answers how many bytes were consumed: a trailing partial line is left for
/// the next call unless `eof`. The line ends are overwritten in place with
/// the NUL each line is appended with.
fn write_output(output: &mut [u8], eof: bool) -> usize {
    let mut start = 0;
    let mut off = 0;
    while start + off < output.len() {
        let binary = Buf::current().b_p_bin != 0;
        let at = start + off;
        let byte = output[at];
        // CRLF, except in binary mode, where the CR is kept.
        let skip = if byte == b'\r' && output.get(at + 1) == Some(&b'\n') && !binary {
            off + 2
        } else if (byte == b'\r' && !binary) || byte == b'\n' {
            off + 1
        } else {
            if byte == 0 {
                // A NUL becomes a newline.
                output[at] = b'\n';
            }
            off += 1;
            continue;
        };
        output[at] = 0;
        append_at_cursor(&output[start..=at]);
        start += skip;
        off = 0;
    }

    if eof {
        if start < output.len() {
            // An unfinished last line, and a note that its ending was
            // missing.
            let mut line = output[start..].to_vec();
            line.push(0);
            append_at_cursor(&line);
            Buf::current().b_no_eol_lnum = Win::current().w_cursor.lnum;
            start = output.len();
        } else {
            Buf::current().b_no_eol_lnum = LineNr::ZERO;
        }
    }

    ui_flush();
    start
}

/// Append one terminated `line` below the cursor, and move the cursor onto
/// it.
fn append_at_cursor(line: &[u8]) {
    let lnum = Win::current().w_cursor.lnum;
    Win::current().w_cursor.lnum += 1;
    let _ = ml_append_terminated(lnum, line, false);
}

/// Apply `'shellxescape'` and `'shellxquote'` to a command.
fn shell_xescape_xquote(cmd: &[u8]) -> ThinCString {
    let (sxq, sxe) = (P_SXQ.get(), P_SXE.get());
    if sxq.is_empty() {
        return ThinCString::from(cmd);
    }
    let escaped = (!sxe.is_empty() && sxq == c"(").then(|| escaped_with(cmd, &sxe, b'^', false));
    let ecmd = escaped
        .as_ref()
        .map_or(cmd, |escaped| escaped.as_cstr().to_bytes());

    // 'shellxquote' of "(" appends ")", of "\"(" appends ")\"".
    let (open, close): (&[u8], &[u8]) = if sxq == c"(" {
        (b"(", b")")
    } else if sxq == c"\"(" {
        (b"\"(", b")\"")
    } else {
        (&sxq, &sxq)
    };
    let mut ncmd = XString::with_capacity(open.len() + ecmd.len() + close.len());
    ncmd.push_bytes(open);
    ncmd.push_bytes(ecmd);
    ncmd.push_bytes(close);
    ThinCString::from(ncmd)
}
