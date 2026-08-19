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

use super::{CPO_REMMARK, FAIL, READ_FILTER, buf_autocmd, check_secure, kExtmarkNOOP};
use crate::autocmd::{EVENT_SHELLCMDPOST, EVENT_SHELLFILTERPOST};
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
use crate::fold::foldUpdate;
use crate::getchar::{AppendToRedobuff, AppendToRedobuffLit};
use crate::global_cell::GlobalCell;
use crate::highlight_group::HLF_N;
use crate::main::{
    Rows, autocmd_busy, bangredo, cmdmod, curbuf, curwin, did_check_timestamps,
    e_cant_read_file_str, e_noprev, e_notmp, firstbuf, global_busy, got_int, info_message, msg_buf,
    msg_col, msg_didout, msg_row, msg_scroll, msg_silent, need_check_timestamps, no_wait_return,
    p_cpo, p_report, p_sh, p_shq, p_srr, p_stmp, p_warn, silent_mode,
};
use crate::mark::mark_adjust;
use crate::memline::ml_get;
use crate::memory::{xfree, xmalloc};
use crate::message::{
    MSG_BUF_LEN, emsg, message_filtered, msg, msg_clr_eos, msg_end, msg_ext_set_kind, msg_outtrans,
    msg_prt_line, msg_putchar, msg_puts, msg_puts_hl, msg_start, msgmore, set_keep_msg,
    wait_return,
};
use crate::r#move::{changed_line_abv_curs, invalidate_botline_win};
use crate::os::cshim::gettext;
use crate::os::fs::os_remove;
use crate::os::input::os_breakcheck;
use crate::os::shell::{ShellOpts, call_shell};
use crate::path::invocation_path_tail;
use crate::pos::MAXLNUM;
use crate::semsg_c;
use crate::strings::{vim_snprintf, vim_strchr, vim_strsave_escaped};
use crate::types::ui::kUIMessages;
use crate::types::{CmdModFlags, NUL, OK, OptInt, buf_T, exarg_T, linenr_T};
use crate::ui::{ui_cursor_goto, ui_has};
use crate::undo::{bufIsChanged, u_save};
use core::ffi::{c_char, c_int};
use core::ptr;

/// The last `:!` command, so that a later `!` in the argument can stand for
/// it.  An `xmalloc`ed C string, owned by this module.
static prevcmd: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

/// Copy `bytes` into a fresh `xmalloc` allocation, NUL-terminated.
///
/// # Safety
/// The caller owns the result and must release it with `xfree`.  `bytes` must
/// hold no interior NUL, or the C string will end early.
unsafe fn xmalloc_cstr(bytes: &[u8]) -> *mut c_char {
    // SAFETY: the allocation is `bytes.len() + 1` long, so the copy and the
    // terminator both land inside it.
    unsafe {
        let buf = xmalloc(bytes.len() + 1) as *mut c_char;
        ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), buf, bytes.len());
        *buf.add(bytes.len()) = 0;
        buf
    }
}

/// Check that [`prevcmd`] is set; if it is not, report it.
///
/// # Safety
/// Main thread, message state ready.
unsafe fn prevcmd_is_set() -> bool {
    if prevcmd.get().is_null() {
        // SAFETY: `e_noprev` is a NUL-terminated message.
        unsafe { emsg(gettext(&raw const e_noprev as *const c_char)) };
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
    eap: *mut exarg_T,
    forceit: bool,
    do_in: bool,
    do_out: bool,
) {
    // SAFETY: caller's contract.
    let (arg, line1, line2) = unsafe { ((*eap).arg, (*eap).line1, (*eap).line2) };
    let scroll_save = msg_scroll.get();
    // Disallow shell commands in secure mode.
    // SAFETY: main thread, message state.
    if unsafe { check_secure() } {
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
        // SAFETY: main thread, message state.
        if ins_prevcmd && !unsafe { prevcmd_is_set() } {
            return;
        }
        let mut text = head;
        if ins_prevcmd {
            // SAFETY: checked non-NULL just above.
            text.extend_from_slice(unsafe { cstr::bytes_at(prevcmd.get()) });
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

    // SAFETY: the bytes come from C strings, so they hold no interior NUL.
    let mut newcmd = unsafe { xmalloc_cstr(&assembled) };
    let mut free_newcmd = assembled.is_empty();
    if !free_newcmd {
        // SAFETY: `prevcmd` is our own allocation, or NULL.
        unsafe { xfree(prevcmd.get().cast()) };
        prevcmd.set(newcmd);
    }

    'theend: {
        if bangredo.get() {
            // Put the command in the redo buffer.
            // SAFETY: main thread, message state.
            if !unsafe { prevcmd_is_set() } {
                break 'theend;
            }
            // SAFETY: `prevcmd` is a live C string and `cmd` our own copy.
            unsafe {
                let cmd = vim_strsave_escaped(prevcmd.get(), c"%#".as_ptr());
                AppendToRedobuffLit(cmd, -1);
                xfree(cmd.cast());
                AppendToRedobuff(c"\n".as_ptr());
            }
            bangredo.set(false);
        }

        // SAFETY: 'shellquote' is a live option string.
        let shq = unsafe { cstr::bytes_at(p_shq.get()) };
        if !shq.is_empty() {
            if free_newcmd {
                // SAFETY: our own allocation.
                unsafe { xfree(newcmd.cast()) };
            }
            // SAFETY: `prevcmd` is live -- either `prevcmd_is_set` passed
            // above, or the assembled command was just stored in it.
            let mut quoted = shq.to_vec();
            quoted.extend_from_slice(unsafe { cstr::bytes_at(prevcmd.get()) });
            quoted.extend_from_slice(shq);
            // SAFETY: option and command bytes, so no interior NUL.
            newcmd = unsafe { xmalloc_cstr(&quoted) };
            free_newcmd = true;
        }

        if addr_count == 0 {
            // Echo the command; it is not remembered in the message history.
            // SAFETY: main thread, message state; `newcmd` is a live string.
            unsafe {
                msg_start();
                msg_ext_set_kind(c"shell_cmd".as_ptr());
                msg_putchar(':' as c_int);
                msg_putchar('!' as c_int);
                msg_outtrans(newcmd, 0, false);
                msg_clr_eos();
            }
            ui_cursor_goto(msg_row.get(), msg_col.get());
            // SAFETY: as above.
            unsafe { do_shell(newcmd, ShellOpts::NONE) };
        } else {
            // SAFETY: `eap` is the caller's live argument and `newcmd` a live
            // string; the autocommand runs with the current buffer.
            unsafe {
                do_filter(line1, line2, eap, newcmd, do_in, do_out);
                buf_autocmd(EVENT_SHELLFILTERPOST, curbuf.get());
            }
        }
    }

    if free_newcmd {
        // SAFETY: our own allocation.
        unsafe { xfree(newcmd.cast()) };
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
        unsafe {
            os_remove(self.0);
            xfree(self.0.cast());
        }
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
    eap: *mut exarg_T,
    cmd: *mut c_char,
    do_in: bool,
    do_out: bool,
) {
    // SAFETY: caller's contract.
    if unsafe { *cmd } as c_int == NUL {
        return; // no filter command
    }

    let old_curbuf = curbuf.get();
    // SAFETY: `curbuf` and `curwin` are the live current buffer and window.
    let (orig_start, orig_end, cursor_save) = unsafe {
        (
            (*curbuf.get()).b_op_start,
            (*curbuf.get()).b_op_end,
            (*curwin.get()).w_cursor,
        )
    };
    let stmp = p_stmp.get();

    // Temporarily disable lockmarks since that's needed to propagate changed
    // regions of the buffer for foldUpdate(), linecount, etc.
    let save_cmod_flags = cmdmod.with(|mods| mods.cmod_flags);
    cmdmod.with_mut(|mods| mods.cmod_flags.clear(CmdModFlags::LOCKMARKS));

    let mut linecount = line2 - line1 + 1;
    // SAFETY: `curwin` is the live current window and `line1` a line of it.
    unsafe {
        (*curwin.get()).w_cursor.lnum = line1;
        (*curwin.get()).w_cursor.col = 0;
        changed_line_abv_curs();
        invalidate_botline_win(curwin.get());
    }

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
            unsafe {
                (*curbuf.get()).b_op_start.lnum = line1;
                (*curbuf.get()).b_op_end.lnum = line2;
            }
        }
        if do_out {
            shell_flags |= ShellOpts::READ;
            // SAFETY: `curwin` is live.
            unsafe { (*curwin.get()).w_cursor.lnum = line2 };
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
            // SAFETY: a live message string.
            unsafe { emsg(gettext(&raw const e_notmp as *const c_char)) };
        }
    }

    'filterend: {
        if no_tempname {
            break 'filterend;
        }

        // The writing and reading of temp files will not be shown.
        // Vi also doesn't do this and the messages are not very informative.
        no_wait_return.set(no_wait_return.get() + 1); // don't wait_return() while busy
        if itmp.is_some()
            // SAFETY: `eap` is live and the range is the current buffer's.
            && unsafe {
                buf_write(
                    curbuf.get(),
                    TempFile::name(&itmp),
                    ptr::null_mut(),
                    line1,
                    line2,
                    eap,
                    WriteRequest::filter(),
                )
            } == FAIL
        {
            if !ui_has(kUIMessages) {
                // SAFETY: message state. Keep message from buf_write().
                unsafe { msg_putchar('\n' as c_int) };
            }
            no_wait_return.set(no_wait_return.get() - 1);
            if !aborting() {
                // SAFETY: one `%s` for one string. Will call wait_return().
                unsafe {
                    semsg_c!(
                        gettext(c"E482: Can't create file %s".as_ptr()),
                        TempFile::name(&itmp),
                    );
                }
            }
            break 'filterend;
        }
        if curbuf.get() != old_curbuf {
            break 'filterend;
        }

        if !do_out {
            // SAFETY: message state.
            unsafe { msg_putchar('\n' as c_int) };
        }

        'error: {
            // SAFETY: `cmd` is live and the temp names are ours.
            let cmd_buf = unsafe {
                make_filter_cmd(cmd, TempFile::name(&itmp), TempFile::name(&otmp), do_in)
            };
            ui_cursor_goto(Rows.get() - 1, 0);

            if do_out {
                // SAFETY: `line2` is a line of the current buffer.
                if unsafe { u_save(line2, line2 + 1) } == FAIL {
                    // SAFETY: `cmd_buf` is our own allocation.
                    unsafe { xfree(cmd_buf.cast()) };
                    break 'error;
                }
                // SAFETY: main thread, redraw state.
                unsafe { redraw_curbuf_later(UPD_VALID) };
            }
            // SAFETY: `curbuf` is live.
            let mut read_linecount = unsafe { (*curbuf.get()).b_ml.ml_line_count };

            // SAFETY: `cmd_buf` is a live command line and ours to free.
            // Pass on the DO_OUT flag when the output is redirected.
            unsafe {
                call_shell(cmd_buf, ShellOpts::FILTER | shell_flags, ptr::null_mut());
                xfree(cmd_buf.cast());
            }

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
                        eap,
                        READ_FILTER as c_int,
                        false,
                    )
                };
                if read != OK {
                    if !aborting() {
                        // SAFETY: message state; one `%s` for one string.
                        unsafe {
                            msg_putchar('\n' as c_int);
                            semsg_c!(
                                gettext(&raw const e_cant_read_file_str as *const c_char),
                                TempFile::name(&otmp),
                            );
                        }
                    }
                    break 'error;
                }
                if curbuf.get() != old_curbuf {
                    break 'filterend;
                }
            }

            // SAFETY: `curbuf` is live.
            read_linecount = unsafe { (*curbuf.get()).b_ml.ml_line_count } - read_linecount;

            if shell_flags.has(ShellOpts::READ) {
                // SAFETY: as above; the read appended after `line2`.
                unsafe {
                    (*curbuf.get()).b_op_start.lnum = line2 + 1;
                    (*curbuf.get()).b_op_end.lnum = (*curwin.get()).w_cursor.lnum;
                    appended_lines_mark(line2, read_linecount as c_int);
                }
            }

            if do_in {
                if cmdmod_has(CmdModFlags::KEEPMARKS)
                    // SAFETY: 'cpoptions' is a live option string.
                    || unsafe { vim_strchr(p_cpo.get(), CPO_REMMARK) }.is_null()
                {
                    // TODO(bfredl): Currently not active for extmarks. What
                    // would we do if columns don't match, assume added/deleted
                    // bytes at the end of each line?
                    // SAFETY: the two ranges are lines of the current buffer.
                    unsafe {
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
                            mark_adjust(
                                line1 + read_linecount,
                                line2,
                                MAXLNUM as linenr_T,
                                0,
                                kExtmarkNOOP,
                            );
                        }
                    }
                }

                // Put cursor on first filtered line for ":range!cmd".
                // Adjust '[ and '] (set by buf_write()).
                // SAFETY: the original range is still in the buffer, ahead of
                // what the filter appended.
                unsafe {
                    (*curwin.get()).w_cursor.lnum = line1;
                    del_lines(linecount, true);
                    (*curbuf.get()).b_op_start.lnum -= linecount;
                    (*curbuf.get()).b_op_end.lnum -= linecount;
                    // adjust last line for next write
                    write_lnum_adjust(-linecount);
                    foldUpdate(
                        curwin.get(),
                        (*curbuf.get()).b_op_start.lnum,
                        (*curbuf.get()).b_op_end.lnum,
                    );
                }
            } else {
                // Put cursor on last new line for ":r !cmd".
                // SAFETY: `curbuf`/`curwin` are live.
                unsafe {
                    linecount = (*curbuf.get()).b_op_end.lnum - (*curbuf.get()).b_op_start.lnum + 1;
                    (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_op_end.lnum;
                }
            }

            // SAFETY: cursor on first non-blank.
            unsafe { beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX) };
            no_wait_return.set(no_wait_return.get() - 1);

            if linecount as OptInt > p_report.get() {
                if do_in {
                    report_filtered(linecount);
                } else {
                    // SAFETY: message state.
                    unsafe { msgmore(linecount as c_int) };
                }
            }
            break 'filterend;
        }

        // put cursor back in same position for ":w !cmd"
        // SAFETY: `curwin` is live and `cursor_save` came from it.
        unsafe { (*curwin.get()).w_cursor = cursor_save };
        no_wait_return.set(no_wait_return.get() - 1);
        // SAFETY: message state.
        unsafe { wait_return(0) };
    }

    cmdmod.with_mut(|mods| mods.cmod_flags = save_cmod_flags);
    if curbuf.get() != old_curbuf {
        no_wait_return.set(no_wait_return.get() - 1);
        // SAFETY: a literal.
        unsafe {
            emsg(gettext(
                c"E135: *Filter* Autocommands must not change current buffer".as_ptr(),
            ));
        }
    } else if cmdmod_has(CmdModFlags::LOCKMARKS) {
        // SAFETY: `curbuf` is live and the marks came from it.
        unsafe {
            (*curbuf.get()).b_op_start = orig_start;
            (*curbuf.get()).b_op_end = orig_end;
        }
    }
}

/// `:range!cmd`'s "N lines filtered", kept in `msg_buf` so that it can survive
/// a redraw.
fn report_filtered(linecount: linenr_T) {
    let buf = msg_buf.ptr() as *mut c_char;
    // SAFETY: `msg_buf` is `MSG_BUF_LEN` bytes and no reference into it is
    // outstanding; one `%ld` for one `int64_t`.  `msg` and `set_keep_msg` copy
    // what they are given.
    unsafe {
        vim_snprintf(
            buf,
            MSG_BUF_LEN as usize,
            gettext(c"%ld lines filtered".as_ptr()),
            linecount as i64,
        );
        if msg(buf, 0) && msg_scroll.get() == 0 {
            // save message to display it after redraw
            set_keep_msg(buf, 0);
        }
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
    if unsafe { check_secure() } {
        unsafe { msg_end() };
        return;
    }

    // For the sake of the terminal, the shell's output starts on a fresh line.
    // SAFETY: message state.
    unsafe {
        msg_putchar('\r' as c_int);
        msg_putchar('\n' as c_int);
    }

    if p_warn.get() != 0 && !autocmd_busy.get() && msg_silent.get() == 0 {
        let mut buf: *mut buf_T = firstbuf.get();
        // SAFETY: the buffer list is the editor's own and is live.
        unsafe {
            while !buf.is_null() {
                if bufIsChanged(buf) {
                    msg_puts(gettext(c"[No write since last change]\n".as_ptr()));
                    break;
                }
                buf = (*buf).b_next;
            }
        }
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
    // SAFETY: the autocommand runs with the current buffer.
    unsafe {
        buf_autocmd(EVENT_SHELLCMDPOST, curbuf.get());
    }
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
    unsafe {
        let buf = xmalloc(len) as *mut c_char;
        ptr::copy_nonoverlapping(text.as_ptr().cast::<c_char>(), buf, text.len());
        *buf.add(text.len()) = 0;
        if !otmp.is_null() {
            append_redir(buf, len, p_srr.get(), otmp);
        }
        buf
    }
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
    unsafe {
        if formats {
            // not really needed?  Not with sh, ksh or bash
            *buf.add(used) = b' ' as c_char;
            vim_snprintf(buf.add(used + 1), buflen - used - 1, opt, fname);
        } else {
            vim_snprintf(buf.add(used), buflen - used, c" %s %s".as_ptr(), opt, fname);
        }
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
    if unsafe { (*curwin.get()).w_onebuf_opt.wo_nu } != 0 || use_number {
        let mut numbuf: [c_char; 30] = [0; 30];
        // SAFETY: a `%*d` for the width and the line number, into a buffer of
        // its own size.  Highlight line nrs.
        unsafe {
            vim_snprintf(
                numbuf.as_mut_ptr(),
                numbuf.len(),
                c"%*d ".as_ptr(),
                number_width(curwin.get()),
                lnum,
            );
            msg_puts_hl(numbuf.as_ptr(), HLF_N + 1, false);
        }
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
        unsafe {
            msg_start();
            msg_ext_set_kind(c"list_cmd".as_ptr());
        }
        global_need_msg_kind.set(false);
    } else if !save_silent {
        // don't want trailing newline with regular messaging
        // SAFETY: message state.
        unsafe { msg_putchar('\n' as c_int) };
    }

    // SAFETY: caller's contract.
    unsafe { print_line_no_prefix(lnum, use_number, list) };
    if save_silent {
        // SAFETY: message state.
        unsafe { msg_putchar('\n' as c_int) };
        silent_mode.set(save_silent);
    }
    info_message.set(false);
}
