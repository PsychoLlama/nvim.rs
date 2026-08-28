//! Leaving: the orderly path through `VimLeavePre`/`VimLeave`, and the two
//! that skip it.
//!
//! [`getout`] is the only path that runs autocommands; [`os_exit`] is what
//! every path funnels into, and [`preserve_exit`] is for when the process is
//! already too broken to do either properly -- it is reachable from a deadly
//! signal handler and from the allocator, so it does as little as it can.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use crate::api::private::helpers::cstr_as_string;
use crate::autocmd::{
    EVENT_BUFUNLOAD, EVENT_BUFWINLEAVE, EVENT_VIMLEAVE, EVENT_VIMLEAVEPRE, apply_autocmds,
    block_autocmds, is_autocmd_blocked, unblock_autocmds,
};
use crate::buffer::{BufRef, buf_get_changedtick, buf_set_changedtick, buf_valid};
use crate::eval::garbage_collect;
use crate::eval::userfunc::invoke_all_defer;
use crate::eval::vars::{get_vim_var_str, set_vim_var_nr, set_vim_var_string, set_vim_var_type};
use crate::event::stream::stream_set_blocking;
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_INF, logmsg_c};
use crate::main::entry::event_teardown;
use crate::main::{
    curbuf, did_emsg, ex_exitval, exiting, exmode_active, garbage_collect_at_exit, no_wait_return,
    p_shada, p_title, p_titleold, stderr_isatty, stdout_isatty, ui_client_channel_id,
    ui_client_exit_status, used_stdin, v_dying,
};
use crate::memfile::mf_fname;
use crate::memline::{ml_close_all, ml_close_notmod, ml_sync_all};
use crate::message::wait_return;
use crate::os::cshim::stderr;
use crate::os::signal::signal_reject_deadly;
use crate::profile::{profile_dump, time_finish};
use crate::shada::shada_write_file;
use crate::types::libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use crate::types::{NUL, VAR_NUMBER, Vv, varnumber_T};
use crate::ui::{ui_call_set_title, ui_call_stop, ui_flush};
use crate::ui_client::ui_client_stop;
use ::libc::{exit, fprintf, strlen, tcdrain};

use crate::winlayer::{Buf, WinId, buffers, first_buffer, first_tab, first_window};
/// Shut the process down. Every exit path ends here, including the ones that
/// skipped the autocommands.
///
/// `r` is the status, except that a UI client with a status of its own wins
/// and a botched event-loop teardown turns a success into a failure.
pub unsafe fn os_exit(mut r: c_int) -> ! {
    exiting.set(true);

    // SAFETY: shuts down the singleton UI, event loop and memfiles, in that
    // order, and never returns.
    if ui_client_channel_id.get() != 0 {
        unsafe { ui_client_stop() };
        if r == 0 {
            r = ui_client_exit_status.get();
        }
    } else {
        unsafe { ui_flush() };
        ui_call_stop();
    }

    if !unsafe { event_teardown() } && r == 0 {
        // The main loop did not come down cleanly; say so in the status.
        r = 1;
    }

    if ui_client_channel_id.get() != 0 {
        // The last output to a TTY is sometimes lost (at least on
        // FreeBSD). Drain it, after `event_teardown`, since libuv events
        // may still have written to stderr.
        if stdout_isatty.get() {
            unsafe { tcdrain(STDOUT_FILENO) };
        }
        if stderr_isatty.get() {
            unsafe { tcdrain(STDERR_FILENO) };
        }
    } else {
        unsafe { ml_close_all(true) };
    }

    if used_stdin.get() {
        // Put the stream back the way we found it (#2598).
        stream_set_blocking(STDIN_FILENO, true);
    }

    let (site, fmt) = (c"os_exit".as_ptr(), c"Nvim exit: %d".as_ptr());
    unsafe { logmsg_c!(LOGLVL_INF, ptr::null(), site, 737, true, fmt, r) };

    unsafe { exit(r) };
}

/// Exit properly: the only path that lets the user's autocommands see it
/// coming.
///
/// Runs `BufWinLeave`, `BufUnload`, `VimLeavePre` and `VimLeave`, writes the
/// ShaDa file and then hands over to [`os_exit`]. A deadly signal has raised
/// `v_dying` by the time it gets here, and the autocommands are skipped --
/// running user code inside a signal handler is how one crash becomes two.
pub unsafe fn getout(mut exitval: c_int) -> ! {
    debug_assert!(
        ui_client_channel_id.get() == 0,
        "getout() in a UI client, which has no editor state to shut down"
    );
    exiting.set(true);

    // Make sure the startup times have been flushed.
    time_finish();

    // POSIX asks for a non-zero status after an error in Ex mode. The
    // standard is not 100% clear about it, but every other vi does this.
    if exmode_active.get() {
        exitval += ex_exitval.get();
    }

    // SAFETY: walks the tab pages, windows and buffers, each of which the
    // autocommands below may free -- hence the `bufref` liveness checks.
    unsafe { set_vim_var_type(Vv::Exiting, VAR_NUMBER) };
    unsafe { set_vim_var_nr(Vv::Exiting, exitval as varnumber_T) };

    // `:restart` and friends set a reason of their own first.
    if unsafe { *get_vim_var_str(Vv::Exitreason) } as c_int == NUL {
        unsafe { set_vim_var_string(Vv::Exitreason, c"quit".as_ptr(), 4) };
    }

    // Every `:defer`red function still on the stack.
    unsafe { invoke_all_defer() };

    if v_dying.get() <= 1 {
        // `BufWinLeave` for every window, but only once per buffer: the
        // changedtick is set to -1 to mark a buffer as already done.
        let mut tab = first_tab();
        while let Some(tp) = tab {
            let mut next_tp = tp.next();
            let mut win = match tp.is_current() {
                true => first_window(),
                false => tp.tp_firstwin.and_then(WinId::get),
            };
            while let Some(wp) = win {
                // An autocommand may already have closed the buffer.
                let buf = wp.w_buffer;
                // `buf_valid` does the null test itself.
                if unsafe { buf_valid(buf) } && buf_get_changedtick(unsafe { Buf::new(buf) }) != -1
                {
                    let bufref = BufRef::of_opt(unsafe { Buf::from_raw(buf) });
                    unsafe {
                        apply_autocmds(
                            EVENT_BUFWINLEAVE,
                            (*buf).b_fname,
                            (*buf).b_fname,
                            false,
                            buf,
                        )
                    };
                    if bufref.valid() {
                        unsafe { buf_set_changedtick(buf, -1) };
                    }
                    // The autocommands may have rearranged both lists;
                    // start the whole walk again.
                    next_tp = first_tab();
                    break;
                }
                win = wp.next();
            }
            tab = next_tp;
        }

        // `BufUnload` for every loaded buffer.
        let mut cur = first_buffer();
        while let Some(mut buf) = cur {
            if !buf.b_ml.ml_mfp.is_null() {
                let bufref = BufRef::of(buf);
                let (name, raw) = (buf.b_fname, buf.raw());
                unsafe { apply_autocmds(EVENT_BUFUNLOAD, name, name, false, raw) };
                if !bufref.valid() {
                    // An autocommand deleted the buffer we were standing
                    // on, so the `b_next` link is gone with it.
                    break;
                }
            }
            cur = buf.next();
        }

        unsafe { with_autocmds_unblocked(EVENT_VIMLEAVEPRE) };
    }

    if !p_shada.get().is_null() && unsafe { *p_shada.get() } as c_int != NUL {
        // The registers, history, marks and the rest.
        unsafe { shada_write_file(ptr::null(), false) };
    }

    if v_dying.get() <= 1 {
        unsafe { with_autocmds_unblocked(EVENT_VIMLEAVE) };
    }

    profile_dump();

    if did_emsg.get() != 0 {
        // Give the user a chance to read the error.
        no_wait_return.set(0);
        // NB: upstream notes this may itself call getout(0) and clobber
        // `exitval`.
        unsafe { wait_return(0) };
    }

    if p_title.get() != 0 && unsafe { *p_titleold.get() } as c_int != NUL {
        ui_call_set_title(unsafe { cstr_as_string(p_titleold.get()) });
    }

    if garbage_collect_at_exit.get() {
        unsafe { garbage_collect(false) };
    }

    unsafe { os_exit(exitval) };
}

/// Fire one of the leave events even if autocommands are blocked, and leave
/// the block the way it was found.
///
/// `deathtrap()` blocks autocommands on the way in, but `VimLeavePre` and
/// `VimLeave` are exactly the two the user still expects to see.
unsafe fn with_autocmds_unblocked(event: crate::types::event_T) {
    // SAFETY: the block counter and the autocommand tables are global.
    let blocked = is_autocmd_blocked();
    if blocked {
        unsafe { unblock_autocmds() };
    }
    unsafe { apply_autocmds(event, ptr::null_mut(), ptr::null_mut(), false, curbuf.get()) };
    if blocked {
        unsafe { block_autocmds() };
    }
}

/// Preserve the swap files, print `errmsg`, and exit 1.
///
/// Reachable from `deadly_signal()` and from the allocator when it is out of
/// memory, so it must not allocate and must survive being called twice: the
/// second call exits 2 without touching anything.
///
/// A null `errmsg` means "say nothing", which is how the process-teardown
/// path reaches it.
pub unsafe fn preserve_exit(errmsg: *const c_char) -> ! {
    /// Set once we are certain we are going down, e.g. after a deadly signal.
    static really_exiting: GlobalCell<bool> = GlobalCell::new(false);

    // SAFETY: `errmsg`, when non-null, is a NUL-terminated string that
    // outlives the call; the buffer list is global.
    if really_exiting.get() {
        if used_stdin.get() {
            // Put the stream back the way we found it (#2598).
            stream_set_blocking(STDIN_FILENO, true);
        }
        unsafe { exit(2) };
    }
    really_exiting.set(true);

    // Ignore SIGHUP now that we are already on the way out (#9274).
    signal_reject_deadly();

    if ui_client_channel_id.get() != 0 {
        // Leave the alternate screen so the message below can be read.
        unsafe { ui_client_stop() };
    }

    if !errmsg.is_null() && unsafe { *errmsg } as c_int != NUL {
        let has_eol = unsafe { *errmsg.add(strlen(errmsg) - 1) } as u8 == b'\n';
        unsafe {
            fprintf(
                stderr,
                if has_eol {
                    c"%s".as_ptr()
                } else {
                    c"%s\n".as_ptr()
                },
                errmsg,
            )
        };
    }

    if ui_client_channel_id.get() != 0 {
        // A UI client has no buffers to preserve.
        unsafe { os_exit(1) };
    }

    unsafe { ml_close_notmod() };

    for buf in buffers() {
        let memfile = buf.b_ml.ml_mfp;
        if !memfile.is_null() && !unsafe { mf_fname(memfile) }.is_null() {
            if !errmsg.is_null() {
                unsafe { fprintf(stderr, c"Nvim: preserving files...\n".as_ptr()) };
            }
            // One sync writes every swap file, so stop at the first
            // buffer that has one.
            unsafe { ml_sync_all(0, 0, true) };
            break;
        }
    }

    // Close the memfiles without deleting them.
    unsafe { ml_close_all(false) };

    if !errmsg.is_null() {
        unsafe { fprintf(stderr, c"Nvim: Finished.\n".as_ptr()) };
    }

    unsafe { getout(1) };
}
