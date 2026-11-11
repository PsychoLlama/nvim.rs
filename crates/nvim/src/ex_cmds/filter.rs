//! Handing buffer text to a shell -- `:!cmd`, `:range!cmd` and `:shell`.
//!
//! [`do_bang`] is the command-line half: it expands `!` to the previous command
//! ([`PREVCMD`]), applies 'shellquote', and decides whether this is a filter (a
//! range was given) or a plain `:!`.  `do_filter` is the buffer half: write the
//! range to a temp file, run the command with the file redirected in and its
//! output redirected out, read the output back over the range, and fix the
//! cursor.  [`make_filter_cmd`] and [`append_redir`] build that shell line from
//! 'shell', 'shellredir' and 'shellpipe'; [`print_line`] is `:print`'s and
//! `:number`'s output, shared with `:global`.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::say;
use super::{buf_autocmd, check_secure};
use crate::autocmd::defs::AutoEvent;
use crate::extmark::EXTMARK_NOOP;
use crate::memline::Lines;
use crate::types::{ColNr, Pos};
use crate::winlayer::{Buf, Win};

use crate::autocmd::state::AUTOCMD_BUSY;
use crate::bufwrite::{WriteName, WriteRequest};
use crate::change::{appended_lines_mark, del_lines};
use crate::cstr;
use crate::drawscreen::{UPD_VALID, number_width, redraw_curbuf_later};
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds2::autowrite_all;
use crate::ex_docmd::cmdmod_has;
use crate::ex_docmd::state::{CMDMOD, GLOBAL_BUSY};
use crate::ex_eval::aborting;
use crate::fileio::state::{DID_CHECK_TIMESTAMPS, NEED_CHECK_TIMESTAMPS};
use crate::fileio::{ReadFlags, readfile_named, temp_name, write_lnum_adjust};
use crate::fold::fold_update;
use crate::getchar::state::{BANGREDO, GOT_INT};
use crate::getchar::{append_to_redobuff, append_to_redobuff_literally};
use crate::global_cell::GlobalCell;
use crate::guard::{MsgBump, Suppress};
use crate::highlight_group::HLF_N;
use crate::mark::mark_adjust;
use crate::memory::XString;
use crate::message::state::{INFO_MESSAGE, MSG_COL, MSG_DIDOUT, MSG_ROW, MSG_SCROLL, MSG_SILENT};
use crate::message::{E_NOPREV, E_NOTMP};
use crate::message::{
    MSG_BUF_LEN, PromptRedraw, emsg, message_filtered, msg, msg_bytes, msg_display,
    msg_ext_set_kind, msg_prt_line, set_keep_msg, wait_return,
};
use crate::message_fmt::{msg_cstr, to_message};
use crate::r#move::{changed_line_abv_curs, invalidate_botline_win};
use crate::option::cpo_has;
use crate::option::defs::CpoFlag;
use crate::option::vars::{P_SHQ, p_report, p_sh, p_srr, p_stmp, p_warn};
use crate::os::cshim::gettext;
use crate::os::fs::os_remove;
use crate::os::input::os_breakcheck;
use crate::path::invocation_path_tail;
use crate::pos::MAXLNUM;
use crate::semsg;
use crate::shell::{ShellOpts, call_shell_text};
use crate::startup::SILENT_MODE;
use crate::strings::escaped_bytes;
use crate::tr;
use crate::types::{CmdModFlags, ExArg, Failed, LineNr, OptInt};
use crate::ui::state::ROWS;
use crate::ui::state::UI_MESSAGES;
use crate::ui::{ui_cursor_goto, ui_has};
use crate::undo::{buf_is_changed, u_save};
use crate::winlayer::buffers;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

/// The last `:!` command, so that a later `!` in the argument can stand for
/// it.  Owned, without a terminator; empty means there has not been one --
/// which is upstream's NULL, and is not reachable any other way, because an
/// empty command is never remembered.
static PREVCMD: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());

/// Check that [`PREVCMD`] is set: E34 when it is not.
fn prevcmd_is_set() -> Result<(), Failed> {
    if PREVCMD.with(Vec::is_empty) {
        emsg(gettext(E_NOPREV));
        return Err(Failed);
    }
    Ok(())
}

/// Handle `:!cmd`, and the `:r !cmd` / `:w !cmd` forms.
///
/// Bangs in the argument stand for the previously entered command, which this
/// then remembers.
pub fn do_bang(addr_count: c_int, args: &mut ExArg, forceit: bool, do_in: bool, do_out: bool) {
    let (line1, line2) = (args.line1, args.line2);
    let scroll_save = MSG_SCROLL.get();
    // Disallow shell commands in secure mode.
    if check_secure() {
        return;
    }

    if addr_count == 0 {
        // ":!" -- the shell may look at the files on disk, so 'autowriteall'
        // gets to put them there first.  Don't scroll here.
        MSG_SCROLL.set(0);
        autowrite_all();
        MSG_SCROLL.set(scroll_save);
    }

    // Assemble the command out of the argument's `!`-separated pieces, each
    // bang standing for the whole of the previous command.
    let mut ins_prevcmd = forceit;
    let mut trail = args
        .line
        .rest_of(args.line.skip_white(args.line.arg))
        .to_vec();
    let mut head: Vec<u8> = Vec::new();
    let assembled = loop {
        if ins_prevcmd && prevcmd_is_set().is_err() {
            return;
        }
        let mut text = head;
        if ins_prevcmd {
            PREVCMD.with(|cmd| text.extend_from_slice(cmd));
        }
        // Only the newly appended argument is scanned for a bang, but the
        // escape test may look one byte back into what came before it.
        let scan_from = text.len();
        text.extend_from_slice(&trail);
        match split_at_bang(&mut text, scan_from) {
            Some(at) => {
                trail = text[at + 1..].to_vec();
                text.truncate(at);
                head = text;
                ins_prevcmd = true;
            }
            None => break text,
        }
    };

    // An empty command is not worth remembering, and upstream's NULL
    // `PREVCMD` is exactly "nothing has been".
    if !assembled.is_empty() {
        PREVCMD.set(assembled.clone());
    }
    let mut newcmd = assembled;
    newcmd.push(0);

    if BANGREDO.get() {
        // Put the command in the redo buffer.
        if prevcmd_is_set().is_err() {
            return;
        }
        let escaped = PREVCMD.with(|cmd| escaped_bytes(cmd, b"%#"));
        append_to_redobuff_literally(escaped.as_cstr().to_bytes());
        append_to_redobuff(b"\n");
        BANGREDO.set(false);
    }

    // A copy: the quoting below outlives the projection's borrow.
    let shellquote = P_SHQ.get();
    let shq = &*shellquote;
    if !shq.is_empty() {
        // `PREVCMD` is set -- either `prevcmd_is_set` passed above, or
        // the assembled command was just stored in it.
        let mut quoted = shq.to_vec();
        PREVCMD.with(|cmd| quoted.extend_from_slice(cmd));
        quoted.extend_from_slice(shq);
        quoted.push(0);
        newcmd = quoted;
    }

    let cmd = CStr::from_bytes_until_nul(&newcmd).expect("the command is terminated");
    if addr_count == 0 {
        // Echo the command; it is not remembered in the message history.
        say::start();
        msg_ext_set_kind(c"shell_cmd");
        say::putchar(b':');
        say::putchar(b'!');
        msg_display(cmd, 0, false);
        say::clear_eos();
        ui_cursor_goto(MSG_ROW.get(), MSG_COL.get());
        do_shell(cmd, ShellOpts::NONE);
    } else {
        // The autocommand runs with the current buffer.
        do_filter(line1, line2, args, cmd, do_in, do_out);
        buf_autocmd(AutoEvent::ShellFilterPost, Buf::current());
    }
}

/// Find the next unescaped `!` in `text` at or after `from`, removing the
/// backslash from each escaped one on the way.
///
/// Upstream removes the backslash with a `memmove` and then steps the scan
/// past the byte that slid into its place, so `\!x` swallows the `x` as well;
/// that quirk is user-visible and is kept.
fn split_at_bang(text: &mut Vec<u8>, from: usize) -> Option<usize> {
    let mut p = from;
    while p < text.len() {
        if text[p] == b'!' {
            if p > 0 && text[p - 1] == b'\\' {
                text.remove(p - 1);
            } else {
                return Some(p);
            }
        }
        p += 1;
    }
    None
}

/// A temp file's name: the file is taken off disk when it goes out of scope,
/// which is what upstream's `filterend` label does by hand.
#[derive(Debug)]
struct TempFile(XString);

impl TempFile {
    fn new() -> Option<TempFile> {
        temp_name().map(TempFile)
    }

    /// The file name, if there is such a file.
    fn name(this: &Option<TempFile>) -> Option<&CStr> {
        this.as_ref().map(|file| file.0.as_cstr())
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        os_remove(self.0.as_cstr());
    }
}

/// Run `cmd` over lines `line1`..`line2`, replacing them with its output.
///
/// `do_in` asks for the range on the command's stdin, `do_out` for its stdout
/// back into the buffer; `:w !cmd` is the first alone, `:r !cmd` the second.
/// Either side travels through a pipe unless 'shelltemp' asks for files.
/// The range must be lines of the current buffer.
fn do_filter(
    line1: LineNr,
    line2: LineNr,
    args: &mut ExArg,
    cmd: &CStr,
    do_in: bool,
    do_out: bool,
) {
    if cmd.is_empty() {
        return; // no filter command
    }

    let (orig_start, orig_end) = (Buf::current().b_op_start, Buf::current().b_op_end);
    let mut filter = Filter {
        line1,
        line2,
        do_in,
        do_out,
        old_curbuf: Buf::current(),
        cursor_save: Win::current().w_cursor,
        linecount: line2 - line1 + 1,
        shell_flags: ShellOpts::NONE,
        otmp: None,
        itmp: None,
        no_prompt: None,
    };

    // Temporarily disable lockmarks since that's needed to propagate changed
    // regions of the buffer for fold_update(), linecount, etc.
    let save_cmod_flags = CMDMOD.with(|mods| mods.cmod_flags);
    CMDMOD.with_mut(|mods| mods.cmod_flags.clear(CmdModFlags::LOCKMARKS));

    Win::current().w_cursor.lnum = line1;
    Win::current().w_cursor.col = ColNr::ZERO;
    changed_line_abv_curs();
    invalidate_botline_win(Win::current());

    if filter.set_up_transport() {
        filter.run(args, cmd);
    }

    CMDMOD.with_mut(|mods| mods.cmod_flags = save_cmod_flags);
    if filter.curbuf_changed() {
        // The C decrements here even on the ":w !cmd" path that already
        // did, which would take the counter below where it started; the
        // guard releases once.
        drop(filter.no_prompt.take());
        emsg(gettext(
            c"E135: *Filter* Autocommands must not change current buffer",
        ));
    } else if cmdmod_has(CmdModFlags::LOCKMARKS) {
        Buf::current().b_op_start = orig_start;
        Buf::current().b_op_end = orig_end;
    }
}

/// One run of [`do_filter`]: the range, how its text travels to and from
/// the command, and what has to be put back.
///
/// When using temp files:
/// 1. * Form temp file names
/// 2. * Write the lines to a temp file
/// 3.   Run the filter command on the temp file
/// 4. * Read the output of the command into the buffer
/// 5. * Delete the original lines to be filtered
/// 6. * Remove the temp files
///
/// When writing the input with a pipe or when catching the output with a
/// pipe only need to do 3.
#[derive(Debug)]
struct Filter {
    line1: LineNr,
    line2: LineNr,
    do_in: bool,
    do_out: bool,
    /// The buffer the range is in; autocommands must not change it.
    old_curbuf: Buf,
    /// Where `:w !cmd` puts the cursor back.
    cursor_save: Pos,
    /// The lines filtered, then the lines read for `:r !cmd`.
    linecount: i32,
    shell_flags: ShellOpts,
    // Dropped in this order (output file, input file, then the prompt
    // guard), as the C's locals were.
    otmp: Option<TempFile>,
    itmp: Option<TempFile>,
    /// Temporarily no `wait_return()`. Released at four different exits,
    /// one of them in [`do_filter`] after the run, so the guard is held in
    /// an `Option` rather than by scope.
    no_prompt: Option<MsgBump>,
}

impl Filter {
    /// Whether autocommands made another buffer current.
    fn curbuf_changed(&self) -> bool {
        !self.old_curbuf.is_current()
    }

    /// Choose pipes or temp files for each side ('shelltemp'), creating the
    /// temp files' names. False (E_NOTMP said) when a name cannot be had.
    fn set_up_transport(&mut self) -> bool {
        if self.do_out {
            self.shell_flags = ShellOpts::DO_OUT;
        }
        if !p_stmp() && (self.do_in || self.do_out) {
            if self.do_in {
                self.shell_flags |= ShellOpts::WRITE;
                Buf::current().b_op_start.lnum = self.line1;
                Buf::current().b_op_end.lnum = self.line2;
            }
            if self.do_out {
                self.shell_flags |= ShellOpts::READ;
                Win::current().w_cursor.lnum = self.line2;
            }
            return true;
        }
        let mut no_tempname = false;
        if self.do_in {
            self.itmp = TempFile::new();
            no_tempname = self.itmp.is_none();
        }
        if !no_tempname && self.do_out {
            self.otmp = TempFile::new();
            no_tempname = self.otmp.is_none();
        }
        if no_tempname {
            emsg(gettext(E_NOTMP));
        }
        !no_tempname
    }

    /// Write the input, run the command, read its output back.
    fn run(&mut self, args: &mut ExArg, cmd: &CStr) {
        // The writing and reading of temp files will not be shown.
        // Vi also doesn't do this and the messages are not very informative.
        self.no_prompt = Some(Suppress::wait_return()); // don't wait_return() while busy
        if !self.write_input(args) || self.curbuf_changed() {
            return;
        }
        if !self.do_out {
            say::putchar(b'\n');
        }
        if self.shell(args, cmd) == Ran::PutBack {
            // put cursor back in same position for ":w !cmd"
            Win::current().w_cursor = self.cursor_save;
            drop(self.no_prompt.take());
            wait_return(PromptRedraw::IfScrolled);
        }
    }

    /// Write the range to the input temp file, when there is one. False
    /// (E482 said) when it cannot be written.
    fn write_input(&mut self, args: &mut ExArg) -> bool {
        let Some(itmp_name) = TempFile::name(&self.itmp) else {
            return true;
        };
        let written = Buf::current().write_file(
            WriteName::Other(itmp_name),
            None,
            self.line1,
            self.line2,
            Some(args),
            WriteRequest::filter(),
        );
        if written.is_ok() {
            return true;
        }
        if !ui_has(UI_MESSAGES) {
            say::putchar(b'\n');
        }
        drop(self.no_prompt.take());
        if !aborting() {
            // Will call wait_return().
            let arg0 = msg_cstr(itmp_name);
            semsg!("E482: Can't create file {arg0}");
        }
        false
    }

    /// Run the command and, for `do_out`, put its output in the buffer.
    fn shell(&mut self, args: &mut ExArg, cmd: &CStr) -> Ran {
        let cmd_buf = make_filter_cmd(
            cmd,
            TempFile::name(&self.itmp),
            TempFile::name(&self.otmp),
            self.do_in,
        );
        ui_cursor_goto(ROWS.get() - 1, 0);

        if self.do_out {
            if u_save(self.line2, self.line2 + 1).is_err() {
                return Ran::PutBack;
            }
            redraw_curbuf_later(UPD_VALID);
        }
        let line_count_before = Buf::current().b_ml.ml_line_count;

        // Pass on the DO_OUT flag when the output is redirected.
        call_shell_text(&cmd_buf, ShellOpts::FILTER | self.shell_flags);
        drop(cmd_buf);

        DID_CHECK_TIMESTAMPS.set(false);
        NEED_CHECK_TIMESTAMPS.set(true);

        // When interrupting the shell command, it may still have produced
        // some useful output.  Reset got_int here, so that readfile()
        // won't cancel reading.
        os_breakcheck();
        GOT_INT.set(false);

        if !self.do_out {
            return Ran::PutBack;
        }

        if let Some(otmp_name) = TempFile::name(&self.otmp) {
            let read = readfile_named(
                Some(otmp_name),
                None,
                self.line2,
                0,
                MAXLNUM.get(),
                Some(args),
                ReadFlags::FILTER,
                false,
            );
            if read.is_err() {
                if !aborting() {
                    say::putchar(b'\n');
                    let arg0 = msg_cstr(otmp_name);
                    semsg!("E485: Can't read file {arg0}");
                }
                return Ran::PutBack;
            }
            if self.curbuf_changed() {
                return Ran::Done;
            }
        }

        let read_linecount = LineNr::new(Buf::current().b_ml.ml_line_count - line_count_before);
        self.take_output(read_linecount);
        Ran::Done
    }

    /// The command's `read_linecount` lines are in after `line2`: replace
    /// the range with them (`:range!cmd`) or move onto them (`:r !cmd`),
    /// and report.
    fn take_output(&mut self, read_linecount: LineNr) {
        let (line1, line2) = (self.line1, self.line2);
        if self.shell_flags.has(ShellOpts::READ) {
            Buf::current().b_op_start.lnum = line2 + 1;
            Buf::current().b_op_end.lnum = Win::current().w_cursor.lnum;
            appended_lines_mark(line2, read_linecount.get());
        }

        if self.do_in {
            let linecount = self.linecount;
            if cmdmod_has(CmdModFlags::KEEPMARKS) || !cpo_has(CpoFlag::REMMARK) {
                // TODO(bfredl): Currently not active for extmarks. What
                // would we do if columns don't match, assume added/deleted
                // bytes at the end of each line?
                if read_linecount >= LineNr::new(linecount) {
                    // move all marks from old lines to new lines
                    mark_adjust(line1, line2, linecount, 0, EXTMARK_NOOP);
                } else {
                    // move marks from old lines to new lines, delete
                    // marks that are in deleted lines
                    mark_adjust(
                        line1,
                        line1 + read_linecount.get() - 1,
                        linecount,
                        0,
                        EXTMARK_NOOP,
                    );
                    mark_adjust(
                        line1 + read_linecount.get(),
                        line2,
                        MAXLNUM.get(),
                        0,
                        EXTMARK_NOOP,
                    );
                }
            }

            // Put cursor on first filtered line for ":range!cmd".
            // Adjust '[ and '] (set by buf_write()).
            Win::current().w_cursor.lnum = line1;
            del_lines(linecount, true);
            Buf::current().b_op_start.lnum -= linecount;
            Buf::current().b_op_end.lnum -= linecount;
            // adjust last line for next write
            write_lnum_adjust(-linecount);
            fold_update(
                Win::current(),
                Buf::current().b_op_start.lnum,
                Buf::current().b_op_end.lnum,
            );
        } else {
            // Put cursor on last new line for ":r !cmd".
            self.linecount = Buf::current().b_op_end.lnum - Buf::current().b_op_start.lnum + 1;
            Win::current().w_cursor.lnum = Buf::current().b_op_end.lnum;
        }

        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
        drop(self.no_prompt.take());

        if OptInt::from(self.linecount) > p_report() {
            if self.do_in {
                report_filtered(self.linecount);
            } else {
                say::more(self.linecount as c_int);
            }
        }
    }
}

/// How [`Filter::shell`] ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ran {
    /// Nothing more to do.
    Done,
    /// No output was taken (`:w !cmd`, or a failure): the cursor goes back
    /// and the user sees the command's output.
    PutBack,
}

/// `:range!cmd`'s "N lines filtered". `set_keep_msg` takes a copy, so it
/// survives the redraw without a buffer outliving this call.
fn report_filtered(linecount: i32) {
    let count = i64::from(linecount);
    // `MSG_BUF_LEN` is where upstream's scratch buffer truncated this.
    let shown = to_message(tr!("{count} lines filtered"), MSG_BUF_LEN as usize);
    if msg(&shown, 0) && MSG_SCROLL.get() == 0 {
        // save message to display it after redraw
        set_keep_msg(Some(&shown), 0);
    }
}

/// Call a shell to execute `cmd`.
///
/// `flags` may be [`ShellOpts::DO_OUT`] when the output is redirected.
pub(crate) fn do_shell(cmd: &CStr, flags: ShellOpts) {
    if check_secure() {
        say::end();
        return;
    }

    // For the sake of the terminal, the shell's output starts on a fresh line.
    say::putchar(b'\r');
    say::putchar(b'\n');

    if p_warn() && !AUTOCMD_BUSY.get() && MSG_SILENT.get() == 0 && buffers().any(buf_is_changed) {
        say::puts(gettext(c"[No write since last change]\n"));
    }

    ui_cursor_goto(MSG_ROW.get(), MSG_COL.get());
    call_shell_text(cmd, flags);

    if MSG_SILENT.get() == 0 {
        MSG_DIDOUT.set(true);
    }
    DID_CHECK_TIMESTAMPS.set(false);
    NEED_CHECK_TIMESTAMPS.set(true);

    // Put the cursor back where it was: the shell wrote over the screen.
    MSG_ROW.set(ROWS.get() - 1);
    MSG_COL.set(0);
    buf_autocmd(AutoEvent::ShellCmdPost, Buf::current());
}

/// Which shell 'shell' names, as far as building a command line goes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Shell {
    /// `begin; ...; end` instead of `(...)`.
    Fish,
    /// PowerShell: no grouping, and `Get-Content` instead of `<`.
    Pwsh,
    /// Anything else, treated as Bourne-compatible.
    Posix,
}

/// Classify 'shell' by the tail of its invocation path.
fn shell_kind() -> Shell {
    let tail = p_sh(|value| invocation_path_tail(value).to_bytes().to_vec());
    if tail.starts_with(b"fish") {
        Shell::Fish
    } else if tail.starts_with(b"pwsh") || tail.starts_with(b"powershell") {
        Shell::Pwsh
    } else {
        Shell::Posix
    }
}

/// The shell command that runs `cmd` with `itmp` as its input file and `otmp`
/// as its output file, either of which may be absent.  `do_in` says whether
/// the command is fed anything on stdin at all.
///
/// Upstream sizes one `xmalloc`ed buffer up front and lets `append_redir`
/// write into what is left over; the sink grows itself instead, so the
/// arithmetic that had to predict the result's length is gone.
pub(crate) fn make_filter_cmd(
    cmd: &CStr,
    itmp: Option<&CStr>,
    otmp: Option<&CStr>,
    do_in: bool,
) -> CString {
    let shell = shell_kind();
    let mut text = filter_cmd_text(
        shell,
        cmd.to_bytes(),
        itmp.map(CStr::to_bytes),
        otmp.is_some(),
        do_in,
    );
    if let Some(otmp) = otmp {
        p_srr(|srr| append_redir(&mut text, srr, otmp));
    }
    cstr::owned(&text)
}

/// The command line itself, before any output redirection is appended.
fn filter_cmd_text(
    shell: Shell,
    cmd: &[u8],
    itmp: Option<&[u8]>,
    has_otmp: bool,
    do_in: bool,
) -> Vec<u8> {
    let mut buf = Vec::new();
    match (shell, itmp) {
        // FIXME: should we add "-Encoding utf8"?
        // FIXME: add `&` ourself or leave to user?
        (Shell::Pwsh, Some(itmp)) => {
            buf.extend_from_slice(b"& { Get-Content ");
            buf.extend_from_slice(itmp);
            buf.extend_from_slice(b" | & ");
            buf.extend_from_slice(cmd);
            buf.extend_from_slice(b" }");
        }
        (Shell::Pwsh, None) => {
            if do_in {
                buf.extend_from_slice(b" $input | ");
            }
            buf.extend_from_slice(cmd);
        }
        (_, itmp) => {
            // Put delimiters around the command (for concatenated commands)
            // when redirecting input and/or output.
            if itmp.is_some() || has_otmp {
                wrap_group(shell, cmd, &mut buf);
            } else {
                buf.extend_from_slice(cmd);
            }
            if let Some(itmp) = itmp {
                buf.extend_from_slice(b" < ");
                buf.extend_from_slice(itmp);
            }
        }
    }
    buf
}

/// `(cmd)`, or fish's `begin; cmd; end`.
fn wrap_group(shell: Shell, cmd: &[u8], buf: &mut Vec<u8>) {
    if shell == Shell::Fish {
        buf.extend_from_slice(b"begin; ");
        buf.extend_from_slice(cmd);
        buf.extend_from_slice(b"; end");
    } else {
        buf.push(b'(');
        buf.extend_from_slice(cmd);
        buf.push(b')');
    }
}

/// Append output redirection for `fname` to the end of `buf`.
///
/// `opt` is a separator or a format string: a `%s` in it is replaced by
/// `fname`, and otherwise a space, `opt`, a space and `fname` are appended.
pub(crate) fn append_redir(buf: &mut Vec<u8>, opt: &CStr, fname: &CStr) {
    buf.push(b' ');
    if has_percent_s(opt.to_bytes()) {
        // not really needed?  Not with sh, ksh or bash
        push_formatted(buf, opt, fname);
    } else {
        // Upstream spells this as `vim_snprintf(" %s %s", opt, fname)`, which
        // is the two strings with the separators around them and nothing a
        // format can do to it.
        buf.extend_from_slice(opt.to_bytes());
        buf.push(b' ');
        buf.extend_from_slice(fname.to_bytes());
    }
}

/// Append `format` with `fname` for its one `%s` to `buf`.
///
/// The format is a user's option value, so it goes through `vim_snprintf`
/// rather than a substitution of our own: whatever it does with a conversion
/// that is not the `%s` this was written for, it does exactly as before.
fn push_formatted(buf: &mut Vec<u8>, format: &CStr, fname: &CStr) {
    buf.extend_from_slice(&crate::formatted!(usize::MAX, format.to_bytes(), fname));
}

/// Does `opt` carry a `%s` conversion?
///
/// A `%%` is an escaped percent and the byte after it is skipped, so `"%%s"`
/// answers false -- which a plain search for `"%s"` would get wrong.
fn has_percent_s(opt: &[u8]) -> bool {
    let mut i = 0;
    while i < opt.len() {
        if opt[i] == b'%' {
            match opt.get(i + 1) {
                Some(b's') => return true,
                Some(b'%') => i += 1, // skip %%
                _ => {}
            }
        }
        i += 1;
    }
    false
}

/// Print line `lnum`, without the leading newline `:print` puts out.
pub fn print_line_no_prefix(lnum: LineNr, use_number: bool, list: bool) {
    if Win::current().w_onebuf_opt.wo_nu != 0 || use_number {
        // Highlight line nrs, right-aligned in 'numberwidth' columns.
        let width = usize::try_from(number_width(Win::current())).unwrap_or(0);
        msg_bytes(format!("{lnum:>width$} ").as_bytes(), HLF_N + 1, false);
    }
    // The message path re-enters the editor (`msg_putchar` can reach the
    // hit-enter prompt), so the line is copied rather than lent out of the
    // memline cache.
    let line = Lines::current().line_copy(lnum);
    msg_prt_line(line.as_cstr(), list);
}

/// Start a new message only once during `:global`.
pub(crate) static GLOBAL_NEED_MSG_KIND: GlobalCell<bool> = GlobalCell::new(false);

/// Print a text line.  Also in silent mode (`ex -s`).
pub fn print_line(lnum: LineNr, use_number: bool, list: bool, first: bool) {
    let save_silent = SILENT_MODE.get();

    // apply :filter /pat/
    let line = Lines::current().line_copy(lnum);
    if message_filtered(line.as_cstr()) {
        return;
    }

    SILENT_MODE.set(false);
    INFO_MESSAGE.set(true); // use stdout, not stderr
    if (GLOBAL_BUSY.get() == 0 || GLOBAL_NEED_MSG_KIND.get()) && first {
        say::start();
        msg_ext_set_kind(c"list_cmd");
        GLOBAL_NEED_MSG_KIND.set(false);
    } else if !save_silent {
        // don't want trailing newline with regular messaging
        say::putchar(b'\n');
    }

    print_line_no_prefix(lnum, use_number, list);
    if save_silent {
        say::putchar(b'\n');
        SILENT_MODE.set(save_silent);
    }
    INFO_MESSAGE.set(false);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The command line for a shell kind, as `make_filter_cmd` assembles it
    /// before the redirection is appended.
    fn assembled(
        shell: Shell,
        cmd: &str,
        itmp: Option<&str>,
        has_otmp: bool,
        do_in: bool,
    ) -> String {
        let text = filter_cmd_text(
            shell,
            cmd.as_bytes(),
            itmp.map(str::as_bytes),
            has_otmp,
            do_in,
        );
        String::from_utf8(text).expect("ASCII in, ASCII out")
    }

    /// `append_redir`'s answer on its own, starting from an empty sink.
    fn redirected(opt: &CStr, fname: &CStr) -> String {
        let mut buf = Vec::new();
        append_redir(&mut buf, opt, fname);
        String::from_utf8(buf).expect("ASCII in, ASCII out")
    }

    #[test]
    fn a_bare_command_is_passed_through_undecorated() {
        assert_eq!(assembled(Shell::Posix, "sort", None, false, false), "sort");
        assert_eq!(assembled(Shell::Fish, "sort", None, false, false), "sort");
    }

    #[test]
    fn redirection_puts_delimiters_around_the_command() {
        // Either redirection is enough to need the grouping, because the
        // command may itself be several commands.
        assert_eq!(assembled(Shell::Posix, "a; b", None, true, false), "(a; b)");
        assert_eq!(
            assembled(Shell::Fish, "a; b", None, true, false),
            "begin; a; b; end"
        );
        assert_eq!(
            assembled(Shell::Posix, "sort", Some("/tmp/in"), false, false),
            "(sort) < /tmp/in"
        );
        assert_eq!(
            assembled(Shell::Fish, "sort", Some("/tmp/in"), true, false),
            "begin; sort; end < /tmp/in"
        );
    }

    #[test]
    fn powershell_pipes_the_input_file_rather_than_redirecting_it() {
        assert_eq!(
            assembled(Shell::Pwsh, "sort", Some("/tmp/in"), true, false),
            "& { Get-Content /tmp/in | & sort }"
        );
        // With no input file, `do_in` is what asks for stdin.
        assert_eq!(
            assembled(Shell::Pwsh, "sort", None, true, true),
            " $input | sort"
        );
        assert_eq!(assembled(Shell::Pwsh, "sort", None, true, false), "sort");
    }

    #[test]
    fn a_shellredir_without_a_conversion_is_a_separator() {
        assert_eq!(redirected(c">", c"/tmp/out"), " > /tmp/out");
        assert_eq!(redirected(c">%s 2>&1", c"/tmp/out"), " >/tmp/out 2>&1");
    }

    #[test]
    fn a_percent_s_in_shellredir_is_where_the_file_name_goes() {
        // The `%s` may be anywhere, and the leading space is added either way.
        assert_eq!(
            redirected(c"2>&1 | tee %s", c"/tmp/out"),
            " 2>&1 | tee /tmp/out"
        );
        // `%%` is an escaped percent, so `%%s` is not a conversion and the
        // whole option is treated as a separator.
        assert!(!has_percent_s(b"%%s"));
        assert!(has_percent_s(b"%%%s"));
        assert!(!has_percent_s(b">"));
        assert!(has_percent_s(b"%s"));
    }

    #[test]
    fn the_redirection_lands_after_whatever_the_sink_already_holds() {
        let mut buf = b"(sort) < /tmp/in".to_vec();
        append_redir(&mut buf, c">", c"/tmp/out");
        assert_eq!(buf, b"(sort) < /tmp/in > /tmp/out");
    }
}
