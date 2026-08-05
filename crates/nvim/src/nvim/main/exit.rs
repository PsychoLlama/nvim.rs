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

use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::autocmd::{
    EVENT_BUFUNLOAD, EVENT_BUFWINLEAVE, EVENT_VIMLEAVE, EVENT_VIMLEAVEPRE, apply_autocmds,
    block_autocmds, is_autocmd_blocked, unblock_autocmds,
};
use crate::src::nvim::buffer::{
    buf_get_changedtick, buf_set_changedtick, buf_valid, bufref_valid, set_bufref,
};
use crate::src::nvim::eval::garbage_collect;
use crate::src::nvim::eval::userfunc::invoke_all_defer;
use crate::src::nvim::eval::vars::{
    get_vim_var_str, set_vim_var_nr, set_vim_var_string, set_vim_var_type,
};
use crate::src::nvim::event::stream::stream_set_blocking;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::{LOGLVL_INF, logmsg};
use crate::src::nvim::main::entry::event_teardown;
use crate::src::nvim::main::{
    NUL, curbuf, curtab, did_emsg, ex_exitval, exiting, exmode_active, first_tabpage, firstbuf,
    firstwin, garbage_collect_at_exit, no_wait_return, p_shada, p_title, p_titleold, stderr_isatty,
    stdout_isatty, ui_client_channel_id, ui_client_exit_status, used_stdin, v_dying,
};
use crate::src::nvim::memfile::mf_fname;
use crate::src::nvim::memline::{ml_close_all, ml_close_notmod, ml_sync_all};
use crate::src::nvim::message::wait_return;
use crate::src::nvim::os::libc::{exit, fprintf, stderr, strlen, tcdrain};
use crate::src::nvim::os::signal::signal_reject_deadly;
use crate::src::nvim::profile::{profile_dump, time_finish};
use crate::src::nvim::shada::shada_write_file;
use crate::src::nvim::types::libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use crate::src::nvim::types::{
    VAR_NUMBER, VV_EXITING, VV_EXITREASON, bufref_T, tabpage_T, varnumber_T,
};
use crate::src::nvim::ui::{ui_call_set_title, ui_call_stop, ui_flush};
use crate::src::nvim::ui_client::ui_client_stop;

/// Shut the process down. Every exit path ends here, including the ones that
/// skipped the autocommands.
///
/// `r` is the status, except that a UI client with a status of its own wins
/// and a botched event-loop teardown turns a success into a failure.
pub unsafe fn os_exit(mut r: c_int) -> ! {
    exiting.set(true);

    // SAFETY: shuts down the singleton UI, event loop and memfiles, in that
    // order, and never returns.
    unsafe {
        if ui_client_channel_id.get() != 0 {
            ui_client_stop();
            if r == 0 {
                r = ui_client_exit_status.get();
            }
        } else {
            ui_flush();
            ui_call_stop();
        }

        if !event_teardown() && r == 0 {
            // The main loop did not come down cleanly; say so in the status.
            r = 1;
        }

        if ui_client_channel_id.get() != 0 {
            // The last output to a TTY is sometimes lost (at least on
            // FreeBSD). Drain it, after `event_teardown`, since libuv events
            // may still have written to stderr.
            if stdout_isatty.get() {
                tcdrain(STDOUT_FILENO);
            }
            if stderr_isatty.get() {
                tcdrain(STDERR_FILENO);
            }
        } else {
            ml_close_all(true);
        }

        if used_stdin.get() {
            // Put the stream back the way we found it (#2598).
            stream_set_blocking(STDIN_FILENO, true);
        }

        logmsg(
            LOGLVL_INF,
            ptr::null(),
            c"os_exit".as_ptr(),
            737,
            true,
            c"Nvim exit: %d".as_ptr(),
            r,
        );

        exit(r);
    }
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
    unsafe {
        set_vim_var_type(VV_EXITING, VAR_NUMBER);
        set_vim_var_nr(VV_EXITING, exitval as varnumber_T);

        // `:restart` and friends set a reason of their own first.
        if *get_vim_var_str(VV_EXITREASON) as c_int == NUL {
            set_vim_var_string(VV_EXITREASON, c"quit".as_ptr(), 4);
        }

        // Every `:defer`red function still on the stack.
        invoke_all_defer();

        if v_dying.get() <= 1 {
            // `BufWinLeave` for every window, but only once per buffer: the
            // changedtick is set to -1 to mark a buffer as already done.
            let mut tp: *const tabpage_T = first_tabpage.get();
            while !tp.is_null() {
                let mut next_tp = (*tp).tp_next;
                let mut wp = if tp == curtab.get() as *const tabpage_T {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
                while !wp.is_null() {
                    // An autocommand may already have closed the buffer.
                    let buf = (*wp).w_buffer;
                    if !buf.is_null() && buf_valid(buf) && buf_get_changedtick(buf) != -1 {
                        let mut bufref = bufref_T::default();
                        set_bufref(&raw mut bufref, buf);
                        apply_autocmds(
                            EVENT_BUFWINLEAVE,
                            (*buf).b_fname,
                            (*buf).b_fname,
                            false,
                            buf,
                        );
                        if bufref_valid(&raw mut bufref) {
                            buf_set_changedtick(buf, -1);
                        }
                        // The autocommands may have rearranged both lists;
                        // start the whole walk again.
                        next_tp = first_tabpage.get();
                        break;
                    }
                    wp = (*wp).w_next;
                }
                tp = next_tp;
            }

            // `BufUnload` for every loaded buffer.
            let mut buf = firstbuf.get();
            while !buf.is_null() {
                if !(*buf).b_ml.ml_mfp.is_null() {
                    let mut bufref = bufref_T::default();
                    set_bufref(&raw mut bufref, buf);
                    apply_autocmds(EVENT_BUFUNLOAD, (*buf).b_fname, (*buf).b_fname, false, buf);
                    if !bufref_valid(&raw mut bufref) {
                        // An autocommand deleted the buffer we were standing
                        // on, so the `b_next` link is gone with it.
                        break;
                    }
                }
                buf = (*buf).b_next;
            }

            with_autocmds_unblocked(EVENT_VIMLEAVEPRE);
        }

        if !p_shada.get().is_null() && *p_shada.get() as c_int != NUL {
            // The registers, history, marks and the rest.
            shada_write_file(ptr::null(), false);
        }

        if v_dying.get() <= 1 {
            with_autocmds_unblocked(EVENT_VIMLEAVE);
        }

        profile_dump();

        if did_emsg.get() != 0 {
            // Give the user a chance to read the error.
            no_wait_return.set(0);
            // NB: upstream notes this may itself call getout(0) and clobber
            // `exitval`.
            wait_return(0);
        }

        if p_title.get() != 0 && *p_titleold.get() as c_int != NUL {
            ui_call_set_title(cstr_as_string(p_titleold.get()));
        }

        if garbage_collect_at_exit.get() {
            garbage_collect(false);
        }

        os_exit(exitval);
    }
}

/// Fire one of the leave events even if autocommands are blocked, and leave
/// the block the way it was found.
///
/// `deathtrap()` blocks autocommands on the way in, but `VimLeavePre` and
/// `VimLeave` are exactly the two the user still expects to see.
unsafe fn with_autocmds_unblocked(event: crate::src::nvim::types::event_T) {
    // SAFETY: the block counter and the autocommand tables are global.
    unsafe {
        let blocked = is_autocmd_blocked();
        if blocked {
            unblock_autocmds();
        }
        apply_autocmds(event, ptr::null_mut(), ptr::null_mut(), false, curbuf.get());
        if blocked {
            block_autocmds();
        }
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
    unsafe {
        if really_exiting.get() {
            if used_stdin.get() {
                // Put the stream back the way we found it (#2598).
                stream_set_blocking(STDIN_FILENO, true);
            }
            exit(2);
        }
        really_exiting.set(true);

        // Ignore SIGHUP now that we are already on the way out (#9274).
        signal_reject_deadly();

        if ui_client_channel_id.get() != 0 {
            // Leave the alternate screen so the message below can be read.
            ui_client_stop();
        }

        if !errmsg.is_null() && *errmsg as c_int != NUL {
            let has_eol = *errmsg.add(strlen(errmsg) - 1) as u8 == b'\n';
            fprintf(
                stderr,
                if has_eol {
                    c"%s".as_ptr()
                } else {
                    c"%s\n".as_ptr()
                },
                errmsg,
            );
        }

        if ui_client_channel_id.get() != 0 {
            // A UI client has no buffers to preserve.
            os_exit(1);
        }

        ml_close_notmod();

        let mut buf = firstbuf.get();
        while !buf.is_null() {
            let memfile = (*buf).b_ml.ml_mfp;
            if !memfile.is_null() && !mf_fname(memfile).is_null() {
                if !errmsg.is_null() {
                    fprintf(stderr, c"Nvim: preserving files...\n".as_ptr());
                }
                // One sync writes every swap file, so stop at the first
                // buffer that has one.
                ml_sync_all(0, 0, true);
                break;
            }
            buf = (*buf).b_next;
        }

        // Close the memfiles without deleting them.
        ml_close_all(false);

        if !errmsg.is_null() {
            fprintf(stderr, c"Nvim: Finished.\n".as_ptr());
        }

        getout(1);
    }
}
