//! Handing buffer text to a shell -- `:!cmd`, `:range!cmd` and `:shell`.
//!
//! [`do_bang`] is the command-line half: it expands `!` to the previous command
//! ([`prevcmd`]), applies 'shellquote', and decides whether this is a filter (a
//! range was given) or a plain `:!`.  `do_filter` is the buffer half: write the
//! range to a temp file, run the command with the file redirected in and its
//! output redirected out, read the output back over the range, and fix the
//! cursor.  [`make_filter_cmd`] and [`append_redir`] build that shell line from
//! 'shell', 'shellredir' and 'shellpipe'; [`print_line`] is `:print`'s and
//! `:number`'s output, shared with `:global`.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::say;
use super::{READ_FILTER, buf_autocmd, check_secure, kExtmarkNOOP};
use crate::types::AutoEvent;
use crate::winlayer::{Buf, Win};

use crate::autocmd::state::autocmd_busy;
use crate::bufwrite::{WriteRequest, buf_write};
use crate::change::{appended_lines_mark, del_lines};
use crate::charset::skipwhite;
use crate::cstr;
use crate::drawscreen::{UPD_VALID, number_width, redraw_curbuf_later};
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds2::autowrite_all;
use crate::ex_docmd::cmdmod_has;
use crate::ex_docmd::state::{cmdmod, global_busy};
use crate::ex_eval::aborting;
use crate::fileio::state::{did_check_timestamps, need_check_timestamps};
use crate::fileio::{readfile, vim_tempname, write_lnum_adjust};
use crate::fold::fold_update;
use crate::getchar::state::{bangredo, got_int};
use crate::getchar::{append_to_redobuff, append_to_redobuff_literally};
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::highlight_group::HLF_N;
use crate::mark::mark_adjust;
use crate::memline::ml_get;
use crate::memory::xfree;
use crate::message::state::{info_message, msg_col, msg_didout, msg_row, msg_scroll, msg_silent};
use crate::message::{
    MSG_BUF_LEN, emsg, message_filtered, msg_display, msg_ext_set_kind, msg_prt_line, msg_ptr,
    msg_str_hl, set_keep_msg, wait_return,
};
use crate::message::{e_noprev, e_notmp};
use crate::message_fmt::c_str;
use crate::r#move::{changed_line_abv_curs, invalidate_botline_win};
use crate::option::cpo_has;
use crate::option::vars::{P_SHQ, p_report, p_sh, p_srr, p_stmp, p_warn};
use crate::os::cshim::gettext;
use crate::os::fs::os_remove;
use crate::os::input::os_breakcheck;
use crate::os::shell::{ShellOpts, call_shell};
use crate::path::invocation_path_tail;
use crate::pos::MAXLNUM;
use crate::semsg;
use crate::startup::silent_mode;
use crate::strings::{vim_snprintf, vim_strsave_escaped};
use crate::types::ui::kUIMessages;
use crate::types::{CmdModFlags, CpoFlag, ExArg, LineNr, NUL, OptInt};
use crate::ui::state::Rows;
use crate::ui::{ui_cursor_goto, ui_has};
use crate::undo::{buf_is_changed, u_save};
use crate::winlayer::buffers;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;
use std::ffi::CString;

/// The last `:!` command, so that a later `!` in the argument can stand for
/// it.  Owned, without a terminator; empty means there has not been one --
/// which is upstream's NULL, and is not reachable any other way, because an
/// empty command is never remembered.
static prevcmd: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());

/// Check that [`prevcmd`] is set; if it is not, report it.
fn prevcmd_is_set() -> bool {
    if prevcmd.with(Vec::is_empty) {
        emsg(gettext(e_noprev));
        return false;
    }
    true
}

/// Handle `:!cmd`, and the `:r !cmd` / `:w !cmd` forms.
///
/// Bangs in the argument stand for the previously entered command, which this
/// then remembers.
pub fn do_bang(addr_count: c_int, args: &mut ExArg, forceit: bool, do_in: bool, do_out: bool) {
    let (arg, line1, line2) = (args.arg_ptr(), args.line1, args.line2);
    let scroll_save = msg_scroll.get();
    // Disallow shell commands in secure mode.
    // SAFETY: main thread, message state.
    if check_secure() {
        return;
    }

    if addr_count == 0 {
        // ":!" -- the shell may look at the files on disk, so 'autowriteall'
        // gets to put them there first.  Don't scroll here.
        msg_scroll.set(0);
        autowrite_all();
        msg_scroll.set(scroll_save);
    }

    // Assemble the command out of the argument's `!`-separated pieces, each
    // bang standing for the whole of the previous command.
    let mut ins_prevcmd = forceit;
    // SAFETY: `arg` is the command's NUL-terminated argument.
    let mut trail = unsafe { cstr::bytes_at(skipwhite(arg)) }.to_vec();
    let mut head: Vec<u8> = Vec::new();
    let assembled = loop {
        if ins_prevcmd && !prevcmd_is_set() {
            return;
        }
        let mut text = head;
        if ins_prevcmd {
            prevcmd.with(|cmd| text.extend_from_slice(cmd));
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
    // `prevcmd` is exactly "nothing has been".
    if !assembled.is_empty() {
        prevcmd.set(assembled.clone());
    }
    let mut newcmd = assembled;
    newcmd.push(NUL as u8);

    'theend: {
        if bangredo.get() {
            // Put the command in the redo buffer.
            if !prevcmd_is_set() {
                break 'theend;
            }
            let mut remembered = prevcmd.with(Vec::clone);
            remembered.push(NUL as u8);
            // SAFETY: `remembered` is this call's own NUL-terminated copy,
            // and the escaped answer is ours to free.
            unsafe {
                let escaped = vim_strsave_escaped(remembered.as_ptr().cast(), c"%#".as_ptr());
                append_to_redobuff_literally(escaped, -1);
                xfree(escaped.cast());
                append_to_redobuff(c"\n".as_ptr());
            };
            bangredo.set(false);
        }

        // A copy: the quoting below outlives the projection's borrow.
        let shellquote = P_SHQ.get();
        let shq = &*shellquote;
        if !shq.is_empty() {
            // `prevcmd` is set -- either `prevcmd_is_set` passed above, or
            // the assembled command was just stored in it.
            let mut quoted = shq.to_vec();
            prevcmd.with(|cmd| quoted.extend_from_slice(cmd));
            quoted.extend_from_slice(shq);
            quoted.push(NUL as u8);
            newcmd = quoted;
        }

        let cmd = newcmd.as_mut_ptr().cast::<c_char>();
        if addr_count == 0 {
            // Echo the command; it is not remembered in the message history.
            say::start();
            // SAFETY: main thread, message state; `cmd` is a live string.
            msg_ext_set_kind(c"shell_cmd");
            say::putchar(':' as c_int);
            say::putchar('!' as c_int);
            msg_display(unsafe { cstr::at(cmd) }, 0, false);
            say::clear_eos();
            ui_cursor_goto(msg_row.get(), msg_col.get());
            // SAFETY: as above.
            unsafe { do_shell(cmd, ShellOpts::NONE) };
        } else {
            // SAFETY: `cmd` is a live string; the autocommand runs with the
            // current buffer.
            unsafe { do_filter(line1, line2, args, cmd, do_in, do_out) };
            buf_autocmd(AutoEvent::ShellFilterPost, Buf::current());
        }
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

/// A `vim_tempname` allocation: taken off disk and freed when it goes out of
/// scope, which is what upstream's `filterend` label does by hand.
struct TempFile(*mut c_char);

impl TempFile {
    fn new() -> Option<TempFile> {
        let name = vim_tempname();
        (!name.is_null()).then_some(TempFile(name))
    }

    /// The file name, or NULL when there is no such file.
    fn name(this: &Option<TempFile>) -> *mut c_char {
        this.as_ref().map_or(ptr::null_mut(), |f| f.0)
    }

    /// The name as a string, for the callers that no longer take a pointer.
    ///
    /// # Safety
    /// The name is `vim_tempname`'s own NUL-terminated allocation.
    unsafe fn as_cstr(this: &Option<TempFile>) -> Option<&CStr> {
        // SAFETY: caller's contract.
        this.as_ref().map(|f| unsafe { cstr::at(f.0) })
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // SAFETY: our own `vim_tempname` allocation.
        unsafe { os_remove(cstr::at(self.0)) };
        unsafe { xfree(self.0.cast()) };
    }
}

/// Run `cmd` over lines `line1`..`line2`, replacing them with its output.
///
/// `do_in` asks for the range on the command's stdin, `do_out` for its stdout
/// back into the buffer; `:w !cmd` is the first alone, `:r !cmd` the second.
/// Either side travels through a pipe unless 'shelltemp' asks for files.
///
/// # Safety
/// `cmd` must be live, and the range must be lines of the current
/// buffer.
unsafe fn do_filter(
    line1: LineNr,
    line2: LineNr,
    args: &mut ExArg,
    cmd: *mut c_char,
    do_in: bool,
    do_out: bool,
) {
    // SAFETY: caller's contract.
    if unsafe { *cmd } as c_int == NUL {
        return; // no filter command
    }

    let old_curbuf = Buf::current_raw();
    // SAFETY: `curbuf` and `curwin` are the live current buffer and window.
    let (orig_start, orig_end, cursor_save) = (
        Buf::current().b_op_start,
        Buf::current().b_op_end,
        Win::current().w_cursor,
    );
    let stmp = p_stmp();

    // Temporarily disable lockmarks since that's needed to propagate changed
    // regions of the buffer for fold_update(), linecount, etc.
    // Released at four different exits, one of them past the end of the
    // block below, so the guard is held in an `Option` rather than by scope.
    let mut no_prompt = None;
    let save_cmod_flags = cmdmod.with(|mods| mods.cmod_flags);
    cmdmod.with_mut(|mods| mods.cmod_flags.clear(CmdModFlags::LOCKMARKS));

    let mut linecount = line2 - line1 + 1;
    // SAFETY: `curwin` is the live current window and `line1` a line of it.
    Win::current().w_cursor.lnum = line1;
    Win::current().w_cursor.col = 0;
    changed_line_abv_curs();
    invalidate_botline_win(Win::current());

    // When using temp files:
    // 1. * Form temp file names
    // 2. * Write the lines to a temp file
    // 3.   Run the filter command on the temp file
    // 4. * Read the output of the command into the buffer
    // 5. * Delete the original lines to be filtered
    // 6. * Remove the temp files
    //
    // When writing the input with a pipe or when catching the output with a
    // pipe only need to do 3.
    let mut shell_flags = if do_out {
        ShellOpts::DO_OUT
    } else {
        ShellOpts::NONE
    };
    let mut itmp = None;
    let mut otmp = None;
    let mut no_tempname = false;
    if !stmp && (do_in || do_out) {
        if do_in {
            shell_flags |= ShellOpts::WRITE;
            // SAFETY: `curbuf` is live.
            Buf::current().b_op_start.lnum = line1;
            Buf::current().b_op_end.lnum = line2;
        }
        if do_out {
            shell_flags |= ShellOpts::READ;
            // SAFETY: `curwin` is live.
            Win::current().w_cursor.lnum = line2;
        }
    } else {
        if do_in {
            itmp = TempFile::new();
            no_tempname = itmp.is_none();
        }
        if !no_tempname && do_out {
            otmp = TempFile::new();
            no_tempname = otmp.is_none();
        }
        if no_tempname {
            emsg(gettext(e_notmp));
        }
    }

    'filterend: {
        if no_tempname {
            break 'filterend;
        }

        // The writing and reading of temp files will not be shown.
        // Vi also doesn't do this and the messages are not very informative.
        no_prompt = Some(Suppress::wait_return()); // don't wait_return() while busy
        if itmp.is_some()
            // SAFETY: the range is the current buffer's.
            && unsafe {
                buf_write(
                    Buf::current(),
                    TempFile::name(&itmp),
                    ptr::null_mut(),
                    line1,
                    line2,
                    Some(args,),
                    WriteRequest::filter(),
                )
            }.is_err()
        {
            if !ui_has(kUIMessages) {
                // SAFETY: message state. Keep message from buf_write().
                say::putchar('\n' as c_int);
            }
            drop(no_prompt.take());
            if !aborting() {
                // SAFETY: one `%s` for one string. Will call wait_return().
                let arg0 = unsafe { c_str(TempFile::name(&itmp)) };
                semsg!("E482: Can't create file {arg0}");
            }
            break 'filterend;
        }
        if Buf::current_raw() != old_curbuf {
            break 'filterend;
        }

        if !do_out {
            // SAFETY: message state.
            say::putchar('\n' as c_int);
        }

        'error: {
            // SAFETY: `cmd` is live and the temp names are ours.
            let cmd_buf = unsafe {
                make_filter_cmd(
                    cstr::at(cmd),
                    TempFile::as_cstr(&itmp),
                    TempFile::as_cstr(&otmp),
                    do_in,
                )
            };
            ui_cursor_goto(Rows.get() - 1, 0);

            if do_out {
                // SAFETY: `line2` is a line of the current buffer.
                if u_save(line2, line2 + 1).is_err() {
                    break 'error;
                }
                // SAFETY: main thread, redraw state.
                redraw_curbuf_later(UPD_VALID);
            }
            // SAFETY: `curbuf` is live.
            let mut read_linecount = Buf::current().b_ml.ml_line_count;

            // SAFETY: `cmd_buf` is a live command line and ours to free.
            // Pass on the DO_OUT flag when the output is redirected.
            unsafe {
                call_shell(
                    cmd_buf.as_ptr().cast_mut(),
                    ShellOpts::FILTER | shell_flags,
                    ptr::null_mut(),
                )
            };
            drop(cmd_buf);

            did_check_timestamps.set(false);
            need_check_timestamps.set(true);

            // When interrupting the shell command, it may still have produced
            // some useful output.  Reset got_int here, so that readfile()
            // won't cancel reading.
            os_breakcheck();
            got_int.set(false);

            if !do_out {
                break 'error;
            }

            if otmp.is_some() {
                // SAFETY: `otmp` is a live file name.
                let read = unsafe {
                    readfile(
                        TempFile::name(&otmp),
                        ptr::null_mut(),
                        line2,
                        0,
                        MAXLNUM,
                        Some(args),
                        READ_FILTER as c_int,
                        false,
                    )
                };
                if read.is_err() {
                    if !aborting() {
                        // SAFETY: message state; one `%s` for one string.
                        say::putchar('\n' as c_int);
                        // SAFETY: a message argument the caller holds as a NUL-terminated string.
                        let arg0 = unsafe { c_str(TempFile::name(&otmp)) };
                        semsg!("E485: Can't read file {arg0}");
                    }
                    break 'error;
                }
                if Buf::current_raw() != old_curbuf {
                    break 'filterend;
                }
            }

            // SAFETY: `curbuf` is live.
            read_linecount = Buf::current().b_ml.ml_line_count - read_linecount;

            if shell_flags.has(ShellOpts::READ) {
                // SAFETY: as above; the read appended after `line2`.
                Buf::current().b_op_start.lnum = line2 + 1;
                Buf::current().b_op_end.lnum = Win::current().w_cursor.lnum;
                appended_lines_mark(line2, read_linecount as c_int);
            }

            if do_in {
                if cmdmod_has(CmdModFlags::KEEPMARKS) || !cpo_has(CpoFlag::REMMARK) {
                    // TODO(bfredl): Currently not active for extmarks. What
                    // would we do if columns don't match, assume added/deleted
                    // bytes at the end of each line?
                    if read_linecount >= linecount {
                        // move all marks from old lines to new lines
                        mark_adjust(line1, line2, linecount, 0, kExtmarkNOOP);
                    } else {
                        // move marks from old lines to new lines, delete
                        // marks that are in deleted lines
                        mark_adjust(
                            line1,
                            line1 + read_linecount - 1,
                            linecount,
                            0,
                            kExtmarkNOOP,
                        );
                        mark_adjust(line1 + read_linecount, line2, MAXLNUM, 0, kExtmarkNOOP);
                    }
                }

                // Put cursor on first filtered line for ":range!cmd".
                // Adjust '[ and '] (set by buf_write()).
                // SAFETY: the original range is still in the buffer, ahead of
                // what the filter appended.
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
                // SAFETY: `curbuf`/`curwin` are live.
                linecount = Buf::current().b_op_end.lnum - Buf::current().b_op_start.lnum + 1;
                Win::current().w_cursor.lnum = Buf::current().b_op_end.lnum;
            }

            // SAFETY: cursor on first non-blank.
            beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
            drop(no_prompt.take());

            if linecount as OptInt > p_report() {
                if do_in {
                    report_filtered(linecount);
                } else {
                    // SAFETY: message state.
                    say::more(linecount as c_int);
                }
            }
            break 'filterend;
        }

        // put cursor back in same position for ":w !cmd"
        // SAFETY: `curwin` is live and `cursor_save` came from it.
        Win::current().w_cursor = cursor_save;
        drop(no_prompt.take());
        // SAFETY: message state.
        wait_return(0);
    }

    cmdmod.with_mut(|mods| mods.cmod_flags = save_cmod_flags);
    if Buf::current_raw() != old_curbuf {
        // The C decrements here even on the ":w !cmd" path that already
        // did, which would take the counter below where it started; the
        // guard releases once.
        drop(no_prompt.take());
        emsg(gettext(
            c"E135: *Filter* Autocommands must not change current buffer",
        ));
    } else if cmdmod_has(CmdModFlags::LOCKMARKS) {
        // SAFETY: `curbuf` is live and the marks came from it.
        Buf::current().b_op_start = orig_start;
        Buf::current().b_op_end = orig_end;
    }
}

/// `:range!cmd`'s "N lines filtered". `set_keep_msg` takes a copy, so it
/// survives the redraw without a buffer outliving this call.
fn report_filtered(linecount: LineNr) {
    let mut scratch = [0 as c_char; MSG_BUF_LEN as usize];
    let buf = scratch.as_mut_ptr();
    // SAFETY: `scratch` is `MSG_BUF_LEN` bytes and outlives the call; one
    // `%ld` for one `int64_t`.  `msg` and `set_keep_msg` copy what they are
    // given.
    unsafe {
        vim_snprintf(
            buf,
            MSG_BUF_LEN as usize,
            gettext(c"%ld lines filtered").as_ptr(),
            linecount as i64,
        )
    };
    if unsafe { msg_ptr(buf, 0) } && msg_scroll.get() == 0 {
        // save message to display it after redraw
        unsafe { set_keep_msg(buf, 0) };
    }
}

/// Call a shell to execute `cmd`; a NULL `cmd` starts an interactive shell.
///
/// `flags` may be [`ShellOpts::DO_OUT`] when the output is redirected.
///
/// # Safety
/// `cmd` must be a live C string, or NULL.
pub unsafe fn do_shell(cmd: *mut c_char, flags: ShellOpts) {
    // SAFETY: main thread, message state.
    if check_secure() {
        say::end();
        return;
    }

    // For the sake of the terminal, the shell's output starts on a fresh line.
    // SAFETY: message state.
    say::putchar('\r' as c_int);
    say::putchar('\n' as c_int);

    if p_warn() && !autocmd_busy.get() && msg_silent.get() == 0 && buffers().any(buf_is_changed) {
        // SAFETY: a live message string.
        say::puts(gettext(c"[No write since last change]\n"));
    }

    ui_cursor_goto(msg_row.get(), msg_col.get());
    // SAFETY: `cmd` is the caller's live command line.
    unsafe { call_shell(cmd, flags, ptr::null_mut()) };

    if msg_silent.get() == 0 {
        msg_didout.set(true);
    }
    did_check_timestamps.set(false);
    need_check_timestamps.set(true);

    // Put the cursor back where it was: the shell wrote over the screen.
    msg_row.set(Rows.get() - 1);
    msg_col.set(0);
    buf_autocmd(AutoEvent::ShellCmdPost, Buf::current());
}

/// Which shell 'shell' names, as far as building a command line goes.
#[derive(Clone, Copy, PartialEq, Eq)]
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
    // SAFETY: caller's contract; a NULL length asks only for the tail.
    let tail = p_sh(|value| unsafe {
        cstr::bytes_at(invocation_path_tail(
            value.as_ptr().cast_mut(),
            ptr::null_mut(),
        ))
    });
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
    // SAFETY: a zero-length destination writes nothing and only measures; the
    // format is the caller's and `fname` is the one string its `%s` names.
    let needed = unsafe { vim_snprintf(ptr::null_mut(), 0, format.as_ptr(), fname.as_ptr()) };
    let Ok(needed) = usize::try_from(needed) else {
        return; // a format `vim_snprintf` could not render at all
    };
    let at = buf.len();
    buf.resize(at + needed + 1, 0); // `vim_snprintf` writes its own NUL
    // SAFETY: `needed + 1` writable bytes at `at`, which is what the
    // measuring call above asked for.
    unsafe {
        vim_snprintf(
            buf.as_mut_ptr().add(at).cast(),
            needed + 1,
            format.as_ptr(),
            fname.as_ptr(),
        )
    };
    buf.truncate(at + needed); // the sink carries no terminator of its own
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
    // SAFETY: `curwin` is the live current window.
    if Win::current().w_onebuf_opt.wo_nu != 0 || use_number {
        let mut numbuf: [c_char; 30] = [0; 30];
        // SAFETY: a `%*d` for the width and the line number, into a buffer of
        // its own size.  Highlight line nrs.
        unsafe {
            vim_snprintf(
                numbuf.as_mut_ptr(),
                numbuf.len(),
                c"%*d ".as_ptr(),
                number_width(Win::current()),
                lnum,
            )
        };
        msg_str_hl(cstr::in_chars(&numbuf), HLF_N + 1, false);
    }
    // SAFETY: caller's contract.
    unsafe { msg_prt_line(ml_get(lnum), list) };
}

/// Start a new message only once during `:global`.
pub(crate) static global_need_msg_kind: GlobalCell<bool> = GlobalCell::new(false);

/// Print a text line.  Also in silent mode (`ex -s`).
pub fn print_line(lnum: LineNr, use_number: bool, list: bool, first: bool) {
    let save_silent = silent_mode.get();

    // apply :filter /pat/
    // SAFETY: caller's contract.
    if message_filtered(unsafe { cstr::at(ml_get(lnum)) }) {
        return;
    }

    silent_mode.set(false);
    info_message.set(true); // use stdout, not stderr
    if (global_busy.get() == 0 || global_need_msg_kind.get()) && first {
        say::start();
        msg_ext_set_kind(c"list_cmd");
        global_need_msg_kind.set(false);
    } else if !save_silent {
        // don't want trailing newline with regular messaging
        say::putchar('\n' as c_int);
    }

    print_line_no_prefix(lnum, use_number, list);
    if save_silent {
        say::putchar('\n' as c_int);
        silent_mode.set(save_silent);
    }
    info_message.set(false);
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
