//! `:buffer`, `:bnext`, `:bdelete` and friends -- `do_buffer()`.
//!
//! [`do_buffer_ext`] is the whole `:buffer`-family command: resolve the
//! argument to a buffer (by number, by name, relative to the current one, or
//! by the alternate file), decide whether the current buffer may be
//! abandoned, and then either switch to the target, unload it, delete it or
//! wipe it.  [`do_bufdel`] is the range form, [`goto_buffer`] the split/hide
//! wrapper, and [`empty_curbuf`] the "there is nothing left to show" fallback
//! when the last listed buffer goes away.
//!
//! The unload half is where the re-entrancy lives: `win_close`,
//! `close_windows`, the `:confirm` dialogs and `close_buffer` all fire
//! autocommands, and every one of them may free the buffer being deleted or
//! the one that was going to replace it.  Each is guarded by a [`BufRef`]
//! taken before the call, exactly as upstream's `bufref` locals are.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use crate::smsg_c;
use core::ffi::{CStr, c_char, c_int, c_ulong};
use core::ptr;
use std::ffi::CString;

use super::*;
use crate::ascii::ascii_isdigit;
use crate::autocmd::is_aucmd_win;
use crate::charset::{getdigits_int, skiptowhite_esc, skipwhite};
use crate::ex_cmds2::{can_abandon, dialog_changed, dialog_close_terminal};
use crate::ex_docmd::{cmdmod_has, ex_errmsg, ex_msg};
use crate::ex_eval::{enter_cleanup, leave_cleanup};
use crate::main::{
    au_new_curbuf, cmdline_row, curbuf, e_cannot_switch_to_a_closing_buffer,
    e_no_write_since_last_change_for_buffer_nr_add_bang_to_override, e_nobufnr, e_trailing_arg,
    got_int, jop_flags, msg_row, msg_scroll, need_fileinfo, p_confirm, p_report, p_write,
    swap_exists_action, swap_exists_did_quit,
};
use crate::mark::mark_jumplist_forget_file;
use crate::memline::ml_recover;
use crate::message::msg_puts;
use crate::options::kOptJopFlagClean;
use crate::os::cshim::ngettext;
use crate::os::input::os_breakcheck;
use crate::search::FORWARD;
use crate::terminal::terminal_running;
use crate::types::{
    CMD_bNext, CMD_bnext, CMD_bprevious, CMD_sbNext, CMD_sbnext, CMD_sbprevious, CmdModFlags, FAIL,
    NUL, OK, OptInt, OptionSetFlags, cleanup_T, exarg_T, int64_t, linenr_T, win_T,
};
use crate::window::{
    check_can_set_curbuf_forceit, last_window, swbuf_goto_win_with_buf, win_close, win_locked,
    win_split,
};
use crate::winlayer::{buffers, last_window as last_listed_window, windows};

use super::expand::find_buf;
use crate::normal::visual_active;

/// A pristine `cleanup_T` for [`enter_cleanup_now`] to fill in.
const NO_CLEANUP: cleanup_T = cleanup_T {
    pending: 0,
    exception: ptr::null_mut(),
};

// ---------------------------------------------------------------------------
// The neighbours, wrapped
//
// One safe wrapper per distinct neighbour, each taking the live buffer or
// window the callee needs, so that the stages below are ordinary code. Each
// one-line body is the only unchecked line the neighbour costs, however many
// times it is called.

/// Reset the error/interrupt/exception state, so that `aborting()` answers
/// false while a window or buffer is closed. Paired with [`leave_cleanup_now`].
fn enter_cleanup_now(cs: &mut cleanup_T) {
    // SAFETY: a local to save the pending state into.
    unsafe { enter_cleanup(cs) };
}

/// Restore what [`enter_cleanup_now`] saved, unless a new aborting error,
/// interrupt or uncaught exception has discarded it.
fn leave_cleanup_now(cs: &mut cleanup_T) {
    // SAFETY: the state `enter_cleanup` has just saved.
    unsafe { leave_cleanup(cs) };
}
fn close_win(mut win: Win, free_buf: bool, force: bool) -> c_int {
    // SAFETY: a live window.
    unsafe { win_close(win.raw(), free_buf, force) }
}
fn window_locked(mut win: Win) -> bool {
    // SAFETY: a live window.
    unsafe { win_locked(win.raw()) != 0 }
}
fn is_last_window(mut win: Win) -> bool {
    // SAFETY: a live window.
    unsafe { last_window(win.raw()) }
}
fn is_autocmd_window(win: *mut win_T) -> bool {
    // SAFETY: the pointer is only compared against the autocommand windows.
    is_aucmd_win(win)
}
fn split_window() -> c_int {
    win_split(0, 0)
}

/// Jump to a window of this tab page already showing `buf`, if `'switchbuf'`
/// says to; the answer is whether one was found.
fn window_showing(mut buf: Buf) -> bool {
    // SAFETY: a live buffer.
    !unsafe { swbuf_goto_win_with_buf(buf.raw()) }.is_null()
}
fn may_change_buffer(forceit: bool) -> bool {
    check_can_set_curbuf_forceit(forceit as c_int)
}
fn forget_jumps(mut win: Win, fnum: c_int) {
    // SAFETY: a live window.
    unsafe { mark_jumplist_forget_file(win.raw(), fnum) };
}
fn may_abandon(mut buf: Buf, forceit: bool) -> bool {
    // SAFETY: a live buffer.
    unsafe { can_abandon(buf.raw(), forceit) }
}

/// The "save changes?" dialog. Re-enters, and may free the buffer.
fn ask_about_changes(mut buf: Buf) {
    // SAFETY: a live buffer; `false` is upstream's `checkall`.
    unsafe { dialog_changed(buf.raw(), false) };
}
fn ask_about_terminal(mut buf: Buf) -> bool {
    // SAFETY: a live buffer with a live terminal.
    unsafe { dialog_close_terminal(buf.raw()) }
}
fn terminal_alive(mut buf: Buf) -> bool {
    // SAFETY: a live terminal, the caller having ruled out null.
    unsafe { terminal_running(buf.terminal) }
}
fn is_quickfix(buf: Buf) -> bool {
    buf_is_quickfix(Some(buf))
}
fn recover_swapfile() {
    // SAFETY: reads the current buffer; `false` is upstream's `checkext`.
    unsafe { ml_recover(false) };
}
fn put_message(msg: &CStr) {
    // SAFETY: a NUL-terminated literal.
    unsafe { msg_puts(msg.as_ptr()) };
}
fn err_nobufnr<T: crate::message_fmt::CArg>(n: T) {
    err_num(tr_raw(e_nobufnr.as_ptr()), n);
}

/// `smsg(0, NGETTEXT(one, many, n), n)`: the "N buffers deleted" report.
fn report_count(one: &'static CStr, many: &'static CStr, n: c_int) {
    let fmt = ngettext(one, many, n as c_ulong);
    // SAFETY: a translated format taking one number.
    let _: c_int = unsafe { smsg_c!(0 as c_int, fmt.as_ptr(), n) };
}

/// A translated message, as the owned error this family answers with.
fn owned_err(msg: &CStr) -> Option<CString> {
    // SAFETY: a NUL-terminated message.
    Some(unsafe { ex_msg(msg.as_ptr()) })
}
fn skip_white(arg: *mut c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated argument.
    unsafe { skipwhite(arg) }
}
fn skip_to_white(arg: *mut c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated argument.
    unsafe { skiptowhite_esc(arg) }
}
fn head(arg: *mut c_char) -> c_int {
    // SAFETY: a NUL-terminated argument, so its first byte is readable.
    unsafe { *arg as c_int }
}
fn take_number(arg: &mut *mut c_char) -> c_int {
    // SAFETY: a NUL-terminated argument starting with a digit.
    unsafe { getdigits_int(arg, false, 0) }
}
fn find_by_pattern(pattern: *mut c_char, end: *mut c_char, unlisted: bool) -> c_int {
    // SAFETY: a NUL-terminated pattern and a pointer into it.
    unsafe { buflist_findpat(pattern, end, unlisted, false, false) }
}
fn trailing_arg_error(arg: *mut c_char) -> Option<CString> {
    // SAFETY: the message static and the caller's NUL-terminated argument.
    Some(unsafe { ex_errmsg(e_trailing_arg.as_ptr(), arg) })
}
fn jop_clean() -> bool {
    jop_flags.get() & kOptJopFlagClean as c_int as ::core::ffi::c_uint != 0
}
fn confirming() -> bool {
    p_confirm.get() != 0 || cmdmod_has(CmdModFlags::CONFIRM)
}

// ---------------------------------------------------------------------------
// The `:buffer` family's entry points

/// Go to another buffer, handling the result of the ATTENTION dialog.
///
/// # Safety
/// `eap` must be a live `exarg_T`.
pub unsafe fn goto_buffer(eap: *mut exarg_T, start: c_int, dir: c_int, count: c_int) {
    let save_sea = swap_exists_action.get();
    // SAFETY: the caller's promise -- a live command, whose `cmd` is a
    // NUL-terminated pointer into the command line.
    let (cmdidx, split) = unsafe { ((*eap).cmdidx, *(*eap).cmd as c_int == 's' as c_int) };
    // SAFETY: as above.
    let forceit = unsafe { (*eap).forceit != 0 };

    let skip_help_buf = matches!(
        cmdidx,
        CMD_bnext | CMD_sbnext | CMD_bNext | CMD_bprevious | CMD_sbNext | CMD_sbprevious
    );

    let old_curbuf = BufRef::of(cur_buf());

    if swap_exists_action.get() == SEA_NONE {
        swap_exists_action.set(SEA_DIALOG);
    }
    let action = if split { DOBUF_SPLIT } else { DOBUF_GOTO } as c_int;
    let skip = if skip_help_buf {
        DOBUF_SKIPHELP as c_int
    } else {
        0
    };
    let _ = do_buffer_ext(
        action,
        start,
        dir,
        count,
        forceit_flag(forceit as c_int) | skip,
    );

    if swap_exists_action.get() == SEA_QUIT && split {
        // Reset the error state, so aborting() is false while the window
        // closes; `leave_cleanup_now` restores it.
        let mut cs = NO_CLEANUP;
        enter_cleanup_now(&mut cs);
        // Quitting means closing the split window, nothing else.
        close_win(cur_win(), true, false);
        swap_exists_action.set(save_sea);
        swap_exists_did_quit.set(true);
        leave_cleanup_now(&mut cs);
    } else {
        handle_swap_exists(Some(old_curbuf));
    }
}

/// Handle the situation of `swap_exists_action` being set.  `old_curbuf` is
/// the buffer to go back to, `None` where the C passed a NULL `bufref_T *`;
/// it is only ever re-validated, never trusted.
pub(crate) fn handle_swap_exists(old_curbuf: Option<BufRef>) {
    let old_tw: OptInt = cur_buf().b_p_tw;

    if swap_exists_action.get() == SEA_QUIT {
        // Reset the error state, so aborting() is false while the buffer
        // closes; `leave_cleanup_now` restores it.
        let mut cs = NO_CLEANUP;
        enter_cleanup_now(&mut cs);
        // User selected Quit at ATTENTION prompt.  Go back to previous buffer.
        // If that buffer is gone or the same as the current one, open a new,
        // empty buffer.
        swap_exists_action.set(SEA_NONE); // don't want it again
        swap_exists_did_quit.set(true);
        let unload = DOBUF_UNLOAD as c_int;
        // SAFETY: the current window and the current buffer.
        unsafe { close_buffer(Some(Win::current()), Buf::current(), unload, false, false) };

        let kept = old_curbuf
            .and_then(BufRef::get)
            .filter(|b| b.raw() != curbuf.get());
        let buf = match kept {
            Some(buf) => Some(buf),
            None => {
                // Block autocommands here because curwin->w_buffer is NULL.
                block_autocmds_now();
                let flags = BLN_CURBUF as c_int | BLN_LISTED as c_int;
                // SAFETY: two null names ask for a nameless buffer.
                let buf = unsafe { buflist_new(ptr::null_mut(), ptr::null_mut(), 1, flags) };
                unblock_autocmds_now();
                // SAFETY: `buflist_new` answers a live buffer or null.
                (!buf.is_null()).then(|| unsafe { Buf::new(buf) })
            }
        };
        if let Some(buf) = buf {
            enter_buffer(buf);
            if old_tw != cur_buf().b_p_tw {
                recheck_colorcolumn(cur_win());
            }
        }
        // If "old_curbuf" is NULL we are in big trouble here...
        leave_cleanup_now(&mut cs);
    } else if swap_exists_action.get() == SEA_RECOVER {
        let mut cs = NO_CLEANUP;
        enter_cleanup_now(&mut cs);
        // User selected Recover at ATTENTION prompt.
        msg_scroll.set(1);
        recover_swapfile();
        put_message(c"\n"); // don't overwrite the last message
        cmdline_row.set(msg_row.get());
        do_modelines(OptionSetFlags::NONE);
        leave_cleanup_now(&mut cs);
    }
    swap_exists_action.set(SEA_NONE);
}

/// Delete or unload buffers -- the `:bdelete`/`:bunload`/`:bwipeout` range
/// form, answering an error message or null.
///
/// `addr_count` 0 is the current buffer, 1 is `end_bnr` followed by whatever
/// `arg` names, and 2 is the range `start_bnr` to `end_bnr`;  `command` is
/// `DOBUF_UNLOAD`, `DOBUF_DEL` or `DOBUF_WIPE`.
///
/// # Safety
/// `arg` must be NUL-terminated.
pub unsafe fn do_bufdel(
    command: c_int,
    arg: *mut c_char,
    addr_count: c_int,
    start_bnr: c_int,
    end_bnr: c_int,
    forceit: c_int,
) -> Option<CString> {
    let mut arg = arg;
    let mut do_current = 0; // delete current buffer?
    let mut deleted = 0; // number of buffers deleted

    if addr_count == 0 {
        do_buffer_ext(
            command,
            DOBUF_CURRENT as c_int,
            FORWARD as c_int,
            0,
            forceit_flag(forceit),
        );
        return None;
    }

    let mut bnr = if addr_count == 2 {
        if head(arg) != 0 {
            // both range and argument is not allowed
            return trailing_arg_error(arg);
        }
        start_bnr
    } else {
        end_bnr // addr_count == 1
    };

    while !got_int.get() {
        // delete the current buffer last, otherwise when the current buffer is
        // deleted, the next buffer becomes the current one and will be loaded,
        // which may then also be deleted, etc.
        if bnr == cur_buf().handle {
            do_current = bnr;
        } else if delete_one(command, bnr, forceit) {
            deleted += 1;
        }

        // find next buffer number to delete/unload
        if addr_count == 2 {
            bnr += 1;
            if bnr > end_bnr {
                break;
            }
        } else {
            // addr_count == 1
            arg = skip_white(arg);
            if head(arg) == NUL {
                break;
            }
            if ascii_isdigit(head(arg)) {
                bnr = take_number(&mut arg);
            } else {
                let end = skip_to_white(arg);
                bnr = find_by_pattern(arg, end, command == DOBUF_WIPE as c_int);
                if bnr < 0 {
                    break; // failed
                }
                arg = end;
            }
        }
        os_breakcheck();
    }
    if !got_int.get() && do_current != 0 && delete_one(command, do_current, forceit) {
        deleted += 1;
    }

    if deleted == 0 {
        return owned_err(match command {
            c if c == DOBUF_UNLOAD as c_int => c"E515: No buffers were unloaded",
            c if c == DOBUF_DEL as c_int => c"E516: No buffers were deleted",
            _ => c"E517: No buffers were wiped out",
        });
    }
    if deleted as OptInt >= p_report.get() {
        match command {
            c if c == DOBUF_UNLOAD as c_int => {
                report_count(c"%d buffer unloaded", c"%d buffers unloaded", deleted);
            }
            c if c == DOBUF_DEL as c_int => {
                report_count(c"%d buffer deleted", c"%d buffers deleted", deleted);
            }
            _ => report_count(c"%d buffer wiped out", c"%d buffers wiped out", deleted),
        }
    }
    None
}

fn forceit_flag(forceit: c_int) -> c_int {
    if forceit != 0 {
        DOBUF_FORCEIT as c_int
    } else {
        0
    }
}

/// One step of [`do_bufdel`]: delete buffer number `bnr`.
fn delete_one(command: c_int, bnr: c_int, forceit: c_int) -> bool {
    let start = DOBUF_FIRST as c_int;
    do_buffer_ext(command, start, FORWARD as c_int, bnr, forceit_flag(forceit)) == OK
}

/// Make the current buffer empty, for when it is wiped out and it is the last
/// one.
fn empty_curbuf(close_others: bool, forceit: c_int, action: c_int) -> c_int {
    let buf = cur_buf();

    if action == DOBUF_UNLOAD as c_int {
        err(c"E90: Cannot unload last buffer");
        return FAIL;
    }

    let bufref = BufRef::of(buf);

    if close_others {
        // Closing all other windows with this buffer may leave only floating
        // windows -- unless another non-floating window holds a different
        // (probably unlisted) buffer, in which case it is fine.  When it is
        // not, `close_windows` would refuse to close the last non-floating
        // window, so it is allowed to close the current one instead.
        let can_close_all_others = !cur_win().w_floating
            || windows()
                .take_while(|wp| !wp.w_floating)
                .any(|wp| wp.w_buffer != curbuf.get());
        close_all_windows(buf, can_close_all_others);
    }

    set_pcmark();
    let none = ptr::null_mut::<c_char>();
    let one = ECMD_ONE as c_int as linenr_T;
    let flags = if forceit != 0 {
        ECMD_FORCEIT as c_int
    } else {
        0
    };
    let retval = edit_file(0, none, none, ptr::null_mut(), one, flags, cur_win());

    // do_ecmd() may create a new buffer, then we have to delete the old one.
    // But do_ecmd() may have done that already, check if the buffer still
    // exists.
    if let Some(old) = bufref
        .get()
        .filter(|b| b.raw() != curbuf.get() && b.b_nwindows == 0)
    {
        close_buffer(None, old, action, false, false);
    }

    if !close_others {
        need_fileinfo.set(false);
    }

    retval
}

// ---------------------------------------------------------------------------
// The command itself

/// The commands for the buffer list.  `action` is `DOBUF_GOTO`/`DOBUF_SPLIT`
/// or one of `DOBUF_UNLOAD`/`DEL`/`WIPE`; `start` says where counting begins
/// (`DOBUF_CURRENT`/`FIRST`/`LAST`/`MOD`); `dir` is `FORWARD` or `BACKWARD`;
/// `count` is a buffer number or a number of buffers.
fn do_buffer_ext(action: c_int, start: c_int, dir: c_int, count: c_int, flags: c_int) -> c_int {
    let mut update_jumplist = true;
    let unload = action == DOBUF_UNLOAD as c_int
        || action == DOBUF_DEL as c_int
        || action == DOBUF_WIPE as c_int;
    let forceit = flags & DOBUF_FORCEIT as c_int != 0;

    let Some(buf) = locate(start, dir, count, flags, unload) else {
        return FAIL;
    };

    if action == DOBUF_GOTO as c_int && buf.raw() != curbuf.get() && !may_change_buffer(forceit) {
        // disallow navigating to another buffer when 'winfixbuf' is applied
        return FAIL;
    }

    if (action == DOBUF_GOTO as c_int || action == DOBUF_SPLIT as c_int)
        && buf.b_flags.has(BufFlags::DUMMY)
    {
        // disallow navigating to the dummy buffer
        err_nobufnr(count);
        return FAIL;
    }

    // delete buffer "buf" from memory and/or the list
    let mut target = Some(buf);
    if unload {
        match unload_buffer(buf, action, flags, &mut update_jumplist) {
            Unloaded::Done(rc) => return rc,
            Unloaded::Replace(replacement) => target = replacement,
        }
    }

    let Some(buf) = target else {
        // Autocommands must have wiped out all other buffers.  Only option now
        // is to make the current buffer empty.
        return empty_curbuf(false, flags & DOBUF_FORCEIT as c_int, action);
    };

    // make "buf" the current buffer
    // If 'switchbuf' is set jump to the window containing "buf".
    if action == DOBUF_SPLIT as c_int && window_showing(buf) {
        return OK;
    }
    // Whether splitting or not, don't open a closing buffer in more windows.
    if buf.raw() != curbuf.get() && buf.b_locked_split != 0 {
        err_raw(tr_raw(e_cannot_switch_to_a_closing_buffer.as_ptr()));
        return FAIL;
    }
    if action == DOBUF_SPLIT as c_int && split_window() == FAIL {
        return FAIL; // split window first
    }

    // go to current buffer - nothing to do
    if buf.raw() == curbuf.get() {
        return OK;
    }

    // Check if the current buffer may be abandoned.
    if action == DOBUF_GOTO as c_int && !may_abandon(cur_buf(), forceit) {
        if confirming() && p_write.get() != 0 {
            let bufref = BufRef::of(buf);
            ask_about_changes(cur_buf());
            if !bufref.valid() {
                // Autocommand deleted buffer, oops!
                return FAIL;
            }
        }
        if is_changed(cur_buf()) {
            no_write_message();
            return FAIL;
        }
    }

    // Go to the other buffer.
    // SAFETY: upstream's own assumption, recorded rather than fixed --
    // `swbuf_goto_win_with_buf`, `win_split` and the `:confirm` dialog above
    // all re-enter, and only the dialog re-validates `buf` afterwards.
    unsafe { set_curbuf(buf, action, update_jumplist) };

    if action == DOBUF_SPLIT as c_int {
        let mut win = cur_win(); // reset 'scrollbind' and 'cursorbind'
        win.w_onebuf_opt.wo_scb = 0;
        win.w_onebuf_opt.wo_crb = 0;
    }

    if aborting_now() {
        return FAIL; // autocmds may abort script processing
    }

    OK
}

/// Which buffer `start`/`dir`/`count` name, with the error already reported
/// when there is none.
fn locate(start: c_int, dir: c_int, count: c_int, flags: c_int, unload: bool) -> Option<Buf> {
    let found = locate_arm(start, dir, count, flags, unload).ok()?;
    if found.is_none() {
        // could not find it
        if start == DOBUF_FIRST as c_int {
            // don't warn when deleting
            if !unload {
                // The two `e_nobufnr` sites pass different widths -- an
                // `int64_t` here and a plain `int` for the dummy buffer below;
                // both are kept as upstream writes them.
                err_nobufnr(count as int64_t);
            }
        } else if dir == FORWARD as c_int {
            err(c"E87: Cannot go beyond last buffer");
        } else {
            err(c"E88: Cannot go before first buffer");
        }
    }
    found
}

/// A located buffer, or none; `Err` means the arm has already reported its
/// own error (E84 or E85).
type Located = Result<Option<Buf>, ()>;

/// The three ways [`locate`] can be asked for a buffer.
fn locate_arm(start: c_int, dir: c_int, count: c_int, flags: c_int, unload: bool) -> Located {
    let mut count = count;
    let from = match start {
        s if s == DOBUF_FIRST as c_int => first_buf(),
        // `DOBUF_LAST`, which c2rust dropped for want of a use.
        2 => last_buf(),
        _ => current_buf(),
    };

    if start == DOBUF_MOD as c_int {
        // find next modified buffer
        let Some(mut buf) = from else {
            return Ok(None);
        };
        while count > 0 {
            count -= 1;
            loop {
                let Some(next) = buf.next().or_else(first_buf) else {
                    return Ok(None);
                };
                buf = next;
                if buf == cur_buf() || is_changed(buf) {
                    break;
                }
            }
        }
        if !is_changed(buf) {
            err(c"E84: No modified buffer found");
            return Err(());
        }
        return Ok(Some(buf));
    }

    if start == DOBUF_FIRST as c_int && count != 0 {
        // find specified buffer number
        let mut buf = from;
        while let Some(b) = buf {
            if b.handle == count {
                break;
            }
            buf = b.next();
        }
        return Ok(buf);
    }

    match from {
        None => Ok(None),
        Some(buf) => step_to_listed(buf, dir, count, flags, unload),
    }
}

/// Step `count` listed buffers away from `buf`, wrapping at either end of the
/// list -- what `:bnext`, `:bprevious` and a bare `:buffer` do.
fn step_to_listed(buf: Buf, dir: c_int, count: c_int, flags: c_int, unload: bool) -> Located {
    let mut buf = buf;
    let mut count = count;
    let skip_help = flags & DOBUF_SKIPHELP as c_int != 0;
    let help_only = skip_help && buf.b_help;

    // remember the buffer where we start, we come back there when all buffers
    // are unlisted.
    let mut bp: Option<Buf> = None;
    while count > 0
        || bp != Some(buf)
            && !unload
            && !(if help_only {
                buf.b_help
            } else {
                buf.b_p_bl != 0
            })
    {
        if bp.is_none() {
            bp = Some(buf);
        }
        let step = if dir == FORWARD as c_int {
            buf.next().or_else(first_buf)
        } else {
            buf.prev().or_else(last_buf)
        };
        let Some(next) = step else {
            return Ok(None);
        };
        buf = next;
        // Avoid non-help buffers if the starting point was a help buffer and
        // vice-versa.  Don't count unlisted buffers.
        let counts = if help_only {
            buf.b_help
        } else {
            buf.b_p_bl != 0 && (!skip_help || !buf.b_help)
        };
        if unload || counts {
            count -= 1;
            bp = None; // use this buffer as new starting point
        }
        if bp == Some(buf) {
            // back where we started, didn't find anything.
            err(c"E85: There is no listed buffer");
            return Err(());
        }
    }
    Ok(Some(buf))
}

/// What the unload half of [`do_buffer_ext`] decided.
enum Unloaded {
    /// The command is finished; this is its result.
    Done(c_int),
    /// The buffer was unloaded; this is the one to go to instead, if any.
    Replace(Option<Buf>),
}

/// Unload, delete or wipe `buf`, and pick the buffer to show in its place.
fn unload_buffer(buf: Buf, action: c_int, flags: c_int, update_jumplist: &mut bool) -> Unloaded {
    if !can_unload_buffer(buf) {
        return Unloaded::Done(FAIL);
    }
    let bufref = BufRef::of(buf);

    // When unloading or deleting a buffer that's already unloaded and
    // unlisted: fail silently.
    if action != DOBUF_WIPE as c_int && buf.b_ml.ml_mfp.is_null() && buf.b_p_bl == 0 {
        return Unloaded::Done(FAIL);
    }

    if let Some(rc) = refuse_unload(buf, bufref, flags) {
        return Unloaded::Done(rc);
    }

    let buf_fnum = buf.handle as c_int;

    // When closing the current buffer stop Visual mode.
    if buf.raw() == curbuf.get() && visual_active() {
        end_visual();
    }

    // If deleting the last (listed) buffer, make it empty.
    // The last (listed) buffer cannot be unloaded.
    if !buffers().any(|b| b.b_p_bl != 0 && b != buf) && buf.raw() == curbuf.get() {
        let forceit = flags & DOBUF_FORCEIT as c_int;
        return Unloaded::Done(empty_curbuf(true, forceit, action));
    }

    // If the deleted buffer is the current one, close the current window
    // (unless it's the only non-floating window), for as long as we end up in
    // a window with this buffer.
    while buf.raw() == curbuf.get()
        && !(window_locked(cur_win()) || cur_win().buffer().b_locked > 0)
        && (last_listed_window().is_some_and(|wp| is_autocmd_window(wp.raw()))
            || !is_last_window(cur_win()))
    {
        if close_win(cur_win(), false, false) == FAIL {
            break;
        }
    }

    // If the buffer to be deleted is not the current one, delete it here.
    if buf.raw() != curbuf.get() {
        if jop_clean() {
            // Remove the buffer to be deleted from the jump list.
            forget_jumps(cur_win(), buf_fnum);
        }

        close_all_windows(buf, false);

        if let Some(gone) = bufref
            .get()
            .filter(|b| b.raw() != curbuf.get() && b.b_nwindows <= 0)
        {
            close_buffer(None, gone, action, false, false);
        }
        return Unloaded::Done(OK);
    }

    Unloaded::Replace(pick_replacement(buf_fnum, update_jumplist))
}

/// The refusals: an unsaved buffer without `!`, and a running terminal job.
///
/// `Some(FAIL)` means the caller must stop; the dialogs re-enter, so `bufref`
/// re-validates `buf` after each.
fn refuse_unload(buf: Buf, bufref: BufRef, flags: c_int) -> Option<c_int> {
    if flags & DOBUF_FORCEIT as c_int == 0 && is_changed(buf) {
        if confirming() && p_write.get() != 0 {
            ask_about_changes(buf);
            // Autocommand deleted buffer, oops! It's not changed now.  If it's
            // still changed fail silently, the dialog already mentioned why it
            // fails.
            let Some(buf) = bufref.get() else {
                return Some(FAIL);
            };
            if is_changed(buf) {
                return Some(FAIL);
            }
        } else {
            let fmt = e_no_write_since_last_change_for_buffer_nr_add_bang_to_override;
            err_num(tr_raw(fmt.as_ptr()), buf.handle as c_int);
            return Some(FAIL);
        }
    }

    if flags & DOBUF_FORCEIT as c_int == 0 && !buf.terminal.is_null() && terminal_alive(buf) {
        if confirming() {
            if !ask_about_terminal(buf) {
                return Some(FAIL);
            }
        } else {
            err_fname(c"E89: %s will be killed (add ! to override)", buf);
            return Some(FAIL);
        }
    }
    None
}

/// Deleting the current buffer: find another to go to.  In order:
/// `au_new_curbuf`, the buffer most recently visited, a loaded one after --
/// then before -- the current buffer, and finally any buffer at all, skipping
/// buffers that are closing.  Autocommands may have deleted them all.
fn pick_replacement(buf_fnum: c_int, update_jumplist: &mut bool) -> Option<Buf> {
    // Used when no loaded buffer found.
    let mut unloaded: Option<Buf> = None;

    // First use au_new_curbuf.br_buf, if it is valid and not closing.
    let mut buf = BufRef::of_record(au_new_curbuf.get())
        .get()
        .filter(|b| b.b_locked_split == 0);
    if buf.is_none() && cur_win().w_jumplistlen > 0 {
        if jop_clean() {
            // Remove the buffer from the jump list.
            forget_jumps(cur_win(), buf_fnum);
        }
        // It's possible that we removed all jump list entries, in that case we
        // need to try another approach.
        if cur_win().w_jumplistlen > 0 {
            buf = walk_jumplist(&mut unloaded, update_jumplist);
        }
    }

    if buf.is_none() {
        // No previous buffer, Try 2'nd approach
        buf = walk_neighbours(&mut unloaded);
    }
    if buf.is_none() {
        // No loaded buffer, use unloaded one
        buf = unloaded;
    }
    if buf.is_none() {
        // No loaded buffer, find listed one
        buf = buffers().find(|b| {
            b.b_p_bl != 0 && b.raw() != curbuf.get() && !is_quickfix(*b) && b.b_locked_split == 0
        });
    }
    if buf.is_none() {
        // Still no buffer, just take one.  Upstream tests the answer without
        // checking it for null first; with both neighbours gone there is
        // nothing to test, which is what `filter` says here.
        let cur = cur_buf();
        buf = cur
            .next()
            .or_else(|| cur.prev())
            .filter(|b| !is_quickfix(*b) && !(b.raw() != curbuf.get() && b.b_locked_split != 0));
    }
    buf
}

/// The jump list, newest first, for the most recently visited buffer that is
/// listed, loaded and not closing.
fn walk_jumplist(unloaded: &mut Option<Buf>, update_jumplist: &mut bool) -> Option<Buf> {
    let mut win = cur_win();
    let mut jumpidx = win.w_jumplistidx;

    if jop_clean() {
        // If the index is the same as the length, the current position was not
        // yet added to the jump list. So we can safely go back to the last
        // entry and search from there.
        if jumpidx == win.w_jumplistlen {
            win.w_jumplistidx = win.w_jumplistlen - 1;
            jumpidx = win.w_jumplistidx;
        }
    } else {
        jumpidx -= 1;
        if jumpidx < 0 {
            jumpidx = win.w_jumplistlen - 1;
        }
    }

    let forward = jumpidx;
    while jop_clean() || jumpidx != win.w_jumplistidx {
        let mut buf = find_buf(win.w_jumplist[jumpidx as usize].fmark.fnum);

        if let Some(b) = buf {
            // Skip current and unlisted bufs.  Also skip a quickfix or closing
            // buffer, it might be deleted soon.
            if b.raw() == curbuf.get() || b.b_p_bl == 0 || is_quickfix(b) || b.b_locked_split != 0 {
                buf = None;
            } else if b.b_ml.ml_mfp.is_null() {
                // skip unloaded buf, but may keep it for later
                unloaded.get_or_insert(b);
                buf = None;
            }
        }
        if buf.is_some() {
            // found a valid buffer: stop searching
            if jop_clean() {
                win.w_jumplistidx = jumpidx;
                *update_jumplist = false;
            }
            return buf;
        }
        // advance to older entry in jump list
        if jumpidx == 0 && win.w_jumplistidx == win.w_jumplistlen {
            return None;
        }
        jumpidx -= 1;
        if jumpidx < 0 {
            jumpidx = win.w_jumplistlen - 1;
        }
        if jumpidx == forward {
            return None; // List exhausted for sure
        }
    }
    None
}

/// The buffers after, then before, the current one, for the first loaded and
/// listed buffer of the same help-ness.
fn walk_neighbours(unloaded: &mut Option<Buf>) -> Option<Buf> {
    let mut forward = true;
    let cur = cur_buf();
    let mut buf = cur.next();
    loop {
        let Some(b) = buf else {
            if !forward {
                return None; // tried both directions
            }
            buf = cur.prev();
            forward = false;
            continue;
        };
        // in non-help buffer, try to skip help buffers, and vv
        if b.b_help == cur.b_help && b.b_p_bl != 0 && !is_quickfix(b) && b.b_locked_split == 0 {
            if !b.b_ml.ml_mfp.is_null() {
                return Some(b); // found loaded buffer
            }
            unloaded.get_or_insert(b); // remember unloaded buf for later
        }
        buf = if forward { b.next() } else { b.prev() };
    }
}

/// `semsg(fmt, buf->b_fname)`.
fn err_fname(fmt: &CStr, mut buf: Buf) {
    let (fmt, name) = (tr(fmt), buf.b_fname);
    // SAFETY: a translated format taking one string, and a buffer's own name.
    let _: bool = unsafe { semsg_c!(fmt, name) };
}

/// [`do_buffer_ext`] with just the `forceit` flag.
pub fn do_buffer(action: c_int, start: c_int, dir: c_int, count: c_int, forceit: c_int) -> c_int {
    do_buffer_ext(action, start, dir, count, forceit_flag(forceit))
}
