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

use super::say;
use super::{READ_FILTER, buf_autocmd, check_secure, kExtmarkNOOP};
use super::{cur_buf, cur_win};
use crate::types::AutoEvent;

use crate::bufwrite::{WriteRequest, buf_write};
use crate::change::{appended_lines_mark, del_lines};
use crate::charset::skipwhite;
use crate::cstr;
use crate::drawscreen::{UPD_VALID, number_width, redraw_curbuf_later};
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds2::autowrite_all;
use crate::ex_docmd::cmdmod_has;
use crate::ex_eval::aborting;
use crate::fileio::{readfile, vim_tempname, write_lnum_adjust};
use crate::fold::fold_update;
use crate::getchar::{append_to_redobuff, append_to_redobuff_literally};
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::highlight_group::HLF_N;
use crate::main::{
    Rows, autocmd_busy, bangredo, cmdmod, did_check_timestamps, e_noprev, e_notmp, global_busy,
    got_int, info_message, msg_col, msg_didout, msg_row, msg_scroll, msg_silent,
    need_check_timestamps, p_report, p_sh, p_shq, p_srr, p_stmp, p_warn, silent_mode,
};
use crate::mark::mark_adjust;
use crate::memline::ml_get;
use crate::memory::{xfree, xmalloc};
use crate::message::{
    MSG_BUF_LEN, emsg, message_filtered, msg_ext_set_kind, msg_outtrans, msg_prt_line, msg_ptr,
    msg_puts_hl, set_keep_msg, wait_return,
};
use crate::message_fmt::c_str;
use crate::r#move::{changed_line_abv_curs, invalidate_botline_win};
use crate::option::cpo_has;
use crate::os::cshim::gettext;
use crate::os::fs::os_remove;
use crate::os::input::os_breakcheck;
use crate::os::shell::{ShellOpts, call_shell};
use crate::path::invocation_path_tail;
use crate::pos::MAXLNUM;
use crate::semsg;
use crate::strings::{vim_snprintf, vim_strsave_escaped};
use crate::types::ui::kUIMessages;
use crate::types::{CmdModFlags, CpoFlag, NUL, OptInt, exarg_T, linenr_T};
use crate::ui::{ui_cursor_goto, ui_has};
use crate::undo::{buf_is_changed, u_save};
use crate::winlayer::buffers;
use core::ffi::{c_char, c_int};
use core::ptr;

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
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn do_bang(
    addr_count: c_int,
    eap: &mut exarg_T,
    forceit: bool,
    do_in: bool,
    do_out: bool,
) {
    let (arg, line1, line2) = (eap.arg, eap.line1, eap.line2);
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
        // SAFETY: main thread; writing every changed buffer is re-entrant but
        // holds no state of ours.
        unsafe { autowrite_all() };
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

        // SAFETY: 'shellquote' is a live option string.
        let shq = unsafe { cstr::bytes_at(p_shq.get()) };
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
            unsafe { msg_ext_set_kind(c"shell_cmd".as_ptr()) };
            say::putchar(':' as c_int);
            say::putchar('!' as c_int);
            unsafe { msg_outtrans(cmd, 0, false) };
            say::clear_eos();
            ui_cursor_goto(msg_row.get(), msg_col.get());
            // SAFETY: as above.
            unsafe { do_shell(cmd, ShellOpts::NONE) };
        } else {
            // SAFETY: `cmd` is a live string; the autocommand runs with the
            // current buffer.
            unsafe { do_filter(line1, line2, eap, cmd, do_in, do_out) };
            buf_autocmd(AutoEvent::ShellFilterPost, cur_buf());
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
    /// # Safety
    /// Main thread; the temp directory must be available.
    unsafe fn new() -> Option<TempFile> {
        // SAFETY: caller's contract.
        let name = unsafe { vim_tempname() };
        (!name.is_null()).then_some(TempFile(name))
    }

    /// The file name, or NULL when there is no such file.
    fn name(this: &Option<TempFile>) -> *mut c_char {
        this.as_ref().map_or(ptr::null_mut(), |f| f.0)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // SAFETY: our own `vim_tempname` allocation.
        unsafe { os_remove(self.0) };
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
/// `eap` and `cmd` must be live, and the range must be lines of the current
/// buffer.
unsafe fn do_filter(
    line1: linenr_T,
    line2: linenr_T,
    eap: &mut exarg_T,
    cmd: *mut c_char,
    do_in: bool,
    do_out: bool,
) {
    // SAFETY: caller's contract.
    if unsafe { *cmd } as c_int == NUL {
        return; // no filter command
    }

    let old_curbuf = cur_buf().raw();
    // SAFETY: `curbuf` and `curwin` are the live current buffer and window.
    let (orig_start, orig_end, cursor_save) =
        (cur_buf().b_op_start, cur_buf().b_op_end, cur_win().w_cursor);
    let stmp = p_stmp.get();

    // Temporarily disable lockmarks since that's needed to propagate changed
    // regions of the buffer for fold_update(), linecount, etc.
    // Released at four different exits, one of them past the end of the
    // block below, so the guard is held in an `Option` rather than by scope.
    let mut no_prompt = None;
    let save_cmod_flags = cmdmod.with(|mods| mods.cmod_flags);
    cmdmod.with_mut(|mods| mods.cmod_flags.clear(CmdModFlags::LOCKMARKS));

    let mut linecount = line2 - line1 + 1;
    // SAFETY: `curwin` is the live current window and `line1` a line of it.
    cur_win().w_cursor.lnum = line1;
    cur_win().w_cursor.col = 0;
    unsafe { changed_line_abv_curs() };
    invalidate_botline_win(cur_win());

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
    if stmp == 0 && (do_in || do_out) {
        if do_in {
            shell_flags |= ShellOpts::WRITE;
            // SAFETY: `curbuf` is live.
            cur_buf().b_op_start.lnum = line1;
            cur_buf().b_op_end.lnum = line2;
        }
        if do_out {
            shell_flags |= ShellOpts::READ;
            // SAFETY: `curwin` is live.
            cur_win().w_cursor.lnum = line2;
        }
    } else {
        if do_in {
            // SAFETY: main thread.
            itmp = unsafe { TempFile::new() };
            no_tempname = itmp.is_none();
        }
        if !no_tempname && do_out {
            // SAFETY: as above.
            otmp = unsafe { TempFile::new() };
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
            // SAFETY: `eap` is live and the range is the current buffer's.
            && unsafe {
                buf_write(
                    cur_buf().raw(),
                    TempFile::name(&itmp),
                    ptr::null_mut(),
                    line1,
                    line2,
                    &raw mut *eap,
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
        if cur_buf().raw() != old_curbuf {
            break 'filterend;
        }

        if !do_out {
            // SAFETY: message state.
            say::putchar('\n' as c_int);
        }

        'error: {
            // SAFETY: `cmd` is live and the temp names are ours.
            let cmd_buf = unsafe {
                make_filter_cmd(cmd, TempFile::name(&itmp), TempFile::name(&otmp), do_in)
            };
            ui_cursor_goto(Rows.get() - 1, 0);

            if do_out {
                // SAFETY: `line2` is a line of the current buffer.
                if u_save(line2, line2 + 1).is_err() {
                    // SAFETY: `cmd_buf` is our own allocation.
                    unsafe { xfree(cmd_buf.cast()) };
                    break 'error;
                }
                // SAFETY: main thread, redraw state.
                redraw_curbuf_later(UPD_VALID);
            }
            // SAFETY: `curbuf` is live.
            let mut read_linecount = cur_buf().b_ml.ml_line_count;

            // SAFETY: `cmd_buf` is a live command line and ours to free.
            // Pass on the DO_OUT flag when the output is redirected.
            unsafe { call_shell(cmd_buf, ShellOpts::FILTER | shell_flags, ptr::null_mut()) };
            unsafe { xfree(cmd_buf.cast()) };

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
                // SAFETY: `otmp` is a live file name and `eap` the caller's.
                let read = unsafe {
                    readfile(
                        TempFile::name(&otmp),
                        ptr::null_mut(),
                        line2,
                        0,
                        MAXLNUM as linenr_T,
                        &raw mut *eap,
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
                if cur_buf().raw() != old_curbuf {
                    break 'filterend;
                }
            }

            // SAFETY: `curbuf` is live.
            read_linecount = cur_buf().b_ml.ml_line_count - read_linecount;

            if shell_flags.has(ShellOpts::READ) {
                // SAFETY: as above; the read appended after `line2`.
                cur_buf().b_op_start.lnum = line2 + 1;
                cur_buf().b_op_end.lnum = cur_win().w_cursor.lnum;
                unsafe { appended_lines_mark(line2, read_linecount as c_int) };
            }

            if do_in {
                if cmdmod_has(CmdModFlags::KEEPMARKS)
                    // SAFETY: 'cpoptions' is a live option string.
                    || !cpo_has(CpoFlag::REMMARK)
                {
                    // TODO(bfredl): Currently not active for extmarks. What
                    // would we do if columns don't match, assume added/deleted
                    // bytes at the end of each line?
                    // SAFETY: the two ranges are lines of the current buffer.
                    if read_linecount >= linecount {
                        // move all marks from old lines to new lines
                        unsafe { mark_adjust(line1, line2, linecount, 0, kExtmarkNOOP) };
                    } else {
                        // move marks from old lines to new lines, delete
                        // marks that are in deleted lines
                        unsafe {
                            mark_adjust(
                                line1,
                                line1 + read_linecount - 1,
                                linecount,
                                0,
                                kExtmarkNOOP,
                            )
                        };
                        unsafe {
                            mark_adjust(
                                line1 + read_linecount,
                                line2,
                                MAXLNUM as linenr_T,
                                0,
                                kExtmarkNOOP,
                            )
                        };
                    }
                }

                // Put cursor on first filtered line for ":range!cmd".
                // Adjust '[ and '] (set by buf_write()).
                // SAFETY: the original range is still in the buffer, ahead of
                // what the filter appended.
                cur_win().w_cursor.lnum = line1;
                unsafe { del_lines(linecount, true) };
                cur_buf().b_op_start.lnum -= linecount;
                cur_buf().b_op_end.lnum -= linecount;
                // adjust last line for next write
                unsafe { write_lnum_adjust(-linecount) };
                fold_update(
                    cur_win(),
                    cur_buf().b_op_start.lnum,
                    cur_buf().b_op_end.lnum,
                );
            } else {
                // Put cursor on last new line for ":r !cmd".
                // SAFETY: `curbuf`/`curwin` are live.
                linecount = cur_buf().b_op_end.lnum - cur_buf().b_op_start.lnum + 1;
                cur_win().w_cursor.lnum = cur_buf().b_op_end.lnum;
            }

            // SAFETY: cursor on first non-blank.
            beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
            drop(no_prompt.take());

            if linecount as OptInt > p_report.get() {
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
        cur_win().w_cursor = cursor_save;
        drop(no_prompt.take());
        // SAFETY: message state.
        unsafe { wait_return(0) };
    }

    cmdmod.with_mut(|mods| mods.cmod_flags = save_cmod_flags);
    if cur_buf().raw() != old_curbuf {
        // The C decrements here even on the ":w !cmd" path that already
        // did, which would take the counter below where it started; the
        // guard releases once.
        drop(no_prompt.take());
        emsg(gettext(
            c"E135: *Filter* Autocommands must not change current buffer",
        ));
    } else if cmdmod_has(CmdModFlags::LOCKMARKS) {
        // SAFETY: `curbuf` is live and the marks came from it.
        cur_buf().b_op_start = orig_start;
        cur_buf().b_op_end = orig_end;
    }
}

/// `:range!cmd`'s "N lines filtered". `set_keep_msg` takes a copy, so it
/// survives the redraw without a buffer outliving this call.
fn report_filtered(linecount: linenr_T) {
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

    if p_warn.get() != 0
        && !autocmd_busy.get()
        && msg_silent.get() == 0
        && buffers().any(buf_is_changed)
    {
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
    buf_autocmd(AutoEvent::ShellCmdPost, cur_buf());
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
///
/// # Safety
/// Main thread; 'shell' must be a live option string.
unsafe fn shell_kind() -> Shell {
    // SAFETY: caller's contract; a NULL length asks only for the tail.
    let tail = unsafe { cstr::bytes_at(invocation_path_tail(p_sh.get(), ptr::null_mut())) };
    if tail.starts_with(b"fish") {
        Shell::Fish
    } else if tail.starts_with(b"pwsh") || tail.starts_with(b"powershell") {
        Shell::Pwsh
    } else {
        Shell::Posix
    }
}

/// The shell command that runs `cmd` with `itmp` as its input file and `otmp`
/// as its output file, either of which may be NULL.  `do_in` says whether the
/// command is fed anything on stdin at all.
///
/// # Safety
/// The three strings must be live, apart from the NULLs allowed above.  The
/// result is the caller's to `xfree`.
pub unsafe fn make_filter_cmd(
    cmd: *mut c_char,
    itmp: *mut c_char,
    otmp: *mut c_char,
    do_in: bool,
) -> *mut c_char {
    // SAFETY: caller's contract, plus the live option strings.
    let (shell, cmd_bytes, itmp_bytes, srr_len) = unsafe {
        (
            shell_kind(),
            cstr::bytes_at(cmd),
            (!itmp.is_null()).then(|| cstr::bytes_at(itmp)),
            if otmp.is_null() {
                0
            } else {
                cstr::bytes_at(p_srr.get()).len()
            },
        )
    };

    // Upstream sizes the buffer up front and `append_redir` writes into what
    // is left over, so the allocation has to carry the redirection's room too
    // even though nothing has been written there yet.  The `sizeof(...) - 1`
    // additions upstream spells out are the literals below.
    let mut len = cmd_bytes.len() + 1; // at least enough space for cmd + NUL
    len += match shell {
        Shell::Fish => b"begin; ; end".len(),
        Shell::Pwsh => 0,
        Shell::Posix => b"()".len(),
    };
    if let Some(itmp_bytes) = itmp_bytes {
        len += itmp_bytes.len()
            + match shell {
                // +6: #20530
                Shell::Pwsh => b"& { Get-Content  | &  }".len() + 6,
                _ => b" {  <  } ".len(),
            };
    }
    if do_in && shell == Shell::Pwsh {
        len += b" $input | ".len() + 1; // upstream counts the NUL here
    }
    if !otmp.is_null() {
        // SAFETY: checked non-NULL.
        len += unsafe { cstr::bytes_at(otmp) }.len() + srr_len + 2; // two extra spaces
    }

    let text = filter_cmd_text(shell, cmd_bytes, itmp_bytes, !otmp.is_null(), do_in);
    debug_assert!(text.len() < len, "make_filter_cmd undersized its buffer");
    // SAFETY: `len` is at least `text.len() + 1` and `append_redir` writes
    // only within the remainder.
    let buf = unsafe { xmalloc(len) } as *mut c_char;
    unsafe { ptr::copy_nonoverlapping(text.as_ptr().cast::<c_char>(), buf, text.len()) };
    unsafe { *buf.add(text.len()) = 0 };
    if !otmp.is_null() {
        unsafe { append_redir(buf, len, p_srr.get(), otmp) };
    }
    buf
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
///
/// # Safety
/// `buf` must be a NUL-terminated string in an allocation of `buflen` bytes,
/// with room left for what is appended; `opt` and `fname` must be live.
pub unsafe fn append_redir(
    buf: *mut c_char,
    buflen: usize,
    opt: *const c_char,
    fname: *const c_char,
) {
    // SAFETY: caller's contract.
    let used = unsafe { cstr::bytes_at(buf) }.len();
    // SAFETY: as above.
    let formats = has_percent_s(unsafe { cstr::bytes_at(opt) });
    // SAFETY: `used` is inside the allocation, and the writes below stay
    // within `buflen`.  One `%s` for one string in either format.
    if formats {
        // not really needed?  Not with sh, ksh or bash
        unsafe { *buf.add(used) = b' ' as c_char };
        unsafe { vim_snprintf(buf.add(used + 1), buflen - used - 1, opt, fname) };
    } else {
        unsafe { vim_snprintf(buf.add(used), buflen - used, c" %s %s".as_ptr(), opt, fname) };
    }
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
///
/// # Safety
/// `lnum` must be a line of the current buffer.
pub unsafe fn print_line_no_prefix(lnum: linenr_T, use_number: bool, list: bool) {
    // SAFETY: `curwin` is the live current window.
    if cur_win().w_onebuf_opt.wo_nu != 0 || use_number {
        let mut numbuf: [c_char; 30] = [0; 30];
        // SAFETY: a `%*d` for the width and the line number, into a buffer of
        // its own size.  Highlight line nrs.
        unsafe {
            vim_snprintf(
                numbuf.as_mut_ptr(),
                numbuf.len(),
                c"%*d ".as_ptr(),
                number_width(cur_win().raw()),
                lnum,
            )
        };
        unsafe { msg_puts_hl(numbuf.as_ptr(), HLF_N + 1, false) };
    }
    // SAFETY: caller's contract.
    unsafe { msg_prt_line(ml_get(lnum), list) };
}

/// Start a new message only once during `:global`.
pub(crate) static global_need_msg_kind: GlobalCell<bool> = GlobalCell::new(false);

/// Print a text line.  Also in silent mode (`ex -s`).
///
/// # Safety
/// `lnum` must be a line of the current buffer.
pub unsafe fn print_line(lnum: linenr_T, use_number: bool, list: bool, first: bool) {
    let save_silent = silent_mode.get();

    // apply :filter /pat/
    // SAFETY: caller's contract.
    if unsafe { message_filtered(ml_get(lnum)) } {
        return;
    }

    silent_mode.set(false);
    info_message.set(true); // use stdout, not stderr
    if (global_busy.get() == 0 || global_need_msg_kind.get()) && first {
        // SAFETY: message state.
        say::start();
        unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
        global_need_msg_kind.set(false);
    } else if !save_silent {
        // don't want trailing newline with regular messaging
        // SAFETY: message state.
        say::putchar('\n' as c_int);
    }

    // SAFETY: caller's contract.
    unsafe { print_line_no_prefix(lnum, use_number, list) };
    if save_silent {
        // SAFETY: message state.
        say::putchar('\n' as c_int);
        silent_mode.set(save_silent);
    }
    info_message.set(false);
}
