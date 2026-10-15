//! Leaving: the orderly path through `VimLeavePre`/`VimLeave`, and the two
//! that skip it.
//!
//! [`getout`] is the only path that runs autocommands; [`os_exit`] is what
//! every path funnels into, and [`preserve_exit`] is for when the process is
//! already too broken to do either properly -- it is reachable from a deadly
//! signal handler and from the allocator, so it does as little as it can.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::autocmd::{fire_autocmds, fire_autocmds_for};
use crate::types::{AutoEvent, String_0};
use core::ffi::CStr;
use core::ffi::c_int;

use crate::autocmd::{block_autocmds, is_autocmd_blocked, unblock_autocmds};
use crate::buffer::{BufRef, buf_get_changedtick, buf_set_changedtick};
use crate::eval::garbage_collect;
use crate::eval::gc::garbage_collect_at_exit;
use crate::eval::userfunc::invoke_all_defer;
use crate::eval::vars::{set_vim_var_nr, set_vim_var_string, set_vim_var_type, with_vim_var_str};
use crate::event::stream::stream_set_blocking;
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_INF, logmsg};
use crate::memline::{ml_close_all, ml_close_notmod, ml_sync_all};
use crate::message::state::{did_emsg, no_wait_return};
use crate::message::wait_return;
use crate::option::vars::{P_SHADA, P_TITLEOLD, p_title, p_titleold};
use crate::os::cshim::write_c_stderr;
use crate::os::input::tty_drain;
use crate::os::signal::signal_reject_deadly;
use crate::profile::{profile_dump, time_finish};
use crate::shada::shada_write_file;
use crate::startup::entry::event_teardown;
use crate::startup::{
    ex_exitval, exiting, stderr_isatty, stdout_isatty, ui_client_channel_id, ui_client_exit_status,
    used_stdin, v_dying,
};
use crate::state::mode::exmode_active;
use crate::types::libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use crate::types::{VAR_NUMBER, VarNumber, Vv};
use crate::ui::{ui_call_set_title, ui_call_stop, ui_flush};
use crate::ui_client::ui_client_stop;

use crate::winlayer::{Buf, WinId, buffers, first_buffer, first_tab, first_window};
/// Shut the process down. Every exit path ends here, including the ones that
/// skipped the autocommands.
///
/// `r` is the status, except that a UI client with a status of its own wins
/// and a botched event-loop teardown turns a success into a failure.
pub fn os_exit(mut r: c_int) -> ! {
    exiting.set(true);

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
            tty_drain(STDOUT_FILENO);
        }
        if stderr_isatty.get() {
            tty_drain(STDERR_FILENO);
        }
    } else {
        ml_close_all(true);
    }

    if used_stdin.get() {
        // Put the stream back the way we found it (#2598).
        stream_set_blocking(STDIN_FILENO, true);
    }

    logmsg!(LOGLVL_INF, c"os_exit", 737, "Nvim exit: {r}");

    std::process::exit(r);
}

/// Exit properly: the only path that lets the user's autocommands see it
/// coming.
///
/// Runs `BufWinLeave`, `BufUnload`, `VimLeavePre` and `VimLeave`, writes the
/// ShaDa file and then hands over to [`os_exit`]. A deadly signal has raised
/// `v_dying` by the time it gets here, and the autocommands are skipped --
/// running user code inside a signal handler is how one crash becomes two.
pub fn getout(mut exitval: c_int) -> ! {
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

    set_vim_var_type(Vv::Exiting, VAR_NUMBER);
    set_vim_var_nr(Vv::Exiting, exitval as VarNumber);

    // `:restart` and friends set a reason of their own first.
    if with_vim_var_str(Vv::Exitreason, CStr::is_empty) {
        set_vim_var_string(Vv::Exitreason, Some(b"quit"));
    }

    // Every `:defer`red function still on the stack.
    invoke_all_defer();

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
                // An autocommand may already have closed the buffer, so the
                // window's buffer is looked up by number before it is read.
                let live = wp.surviving_buffer();
                if let Some(buffer) = live.filter(|b| buf_get_changedtick(*b) != -1) {
                    let bufref = BufRef::of(buffer);
                    // A copy: the autocommands may rename the buffer.
                    let fname = buffer.name.shown().map(CStr::to_owned);
                    let (fname, event) = (fname.as_deref(), AutoEvent::BufWinLeave);
                    fire_autocmds_for(event, fname, fname, false, Some(buffer), None);
                    // The event may have wiped it; ask again before writing.
                    if let Some(buffer) = bufref.get() {
                        buf_set_changedtick(buffer, -1);
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
        while let Some(buf) = cur {
            if !buf.b_ml.ml_mfp.is_null() {
                let bufref = BufRef::of(buf);
                // A copy, as above.
                let name = buf.name.shown().map(CStr::to_owned);
                let (name, some) = (name.as_deref(), Some(buf));
                fire_autocmds_for(AutoEvent::BufUnload, name, name, false, some, None);
                if !bufref.valid() {
                    // An autocommand deleted the buffer we were standing
                    // on, so the `b_next` link is gone with it.
                    break;
                }
            }
            cur = buf.next();
        }

        with_autocmds_unblocked(AutoEvent::VimLeavePre);
    }

    if P_SHADA.first_byte() != 0 {
        // The registers, history, marks and the rest.
        shada_write_file(None, false);
    }

    if v_dying.get() <= 1 {
        with_autocmds_unblocked(AutoEvent::VimLeave);
    }

    profile_dump();

    if did_emsg.get() != 0 {
        // Give the user a chance to read the error.
        no_wait_return.set(0);
        // NB: upstream notes this may itself call getout(0) and clobber
        // `exitval`.
        wait_return(0);
    }

    if p_title() && P_TITLEOLD.first_byte() != 0 {
        ui_call_set_title(p_titleold(String_0::from_cstr));
    }

    if garbage_collect_at_exit.get() {
        garbage_collect(false);
    }

    os_exit(exitval);
}

/// Fire one of the leave events even if autocommands are blocked, and leave
/// the block the way it was found.
///
/// `deathtrap()` blocks autocommands on the way in, but `VimLeavePre` and
/// `VimLeave` are exactly the two the user still expects to see.
fn with_autocmds_unblocked(event: AutoEvent) {
    let blocked = is_autocmd_blocked();
    if blocked {
        unblock_autocmds();
    }
    fire_autocmds(event, false, Buf::current_or_none());
    if blocked {
        block_autocmds();
    }
}

/// Preserve the swap files, print `errmsg`, and exit 1.
///
/// Reachable from `deadly_signal()` and from the allocator when it is out of
/// memory, so it must not allocate and must survive being called twice: the
/// second call exits 2 without touching anything.
///
/// `None` means "say nothing", which is how the process-teardown path
/// reaches it.
pub fn preserve_exit(errmsg: Option<&CStr>) -> ! {
    /// Set once we are certain we are going down, e.g. after a deadly signal.
    static really_exiting: GlobalCell<bool> = GlobalCell::new(false);

    if really_exiting.get() {
        if used_stdin.get() {
            // Put the stream back the way we found it (#2598).
            stream_set_blocking(STDIN_FILENO, true);
        }
        std::process::exit(2);
    }
    really_exiting.set(true);

    // Ignore SIGHUP now that we are already on the way out (#9274).
    signal_reject_deadly();

    if ui_client_channel_id.get() != 0 {
        // Leave the alternate screen so the message below can be read.
        ui_client_stop();
    }

    if let Some(text) = errmsg.map(CStr::to_bytes).filter(|text| !text.is_empty()) {
        write_c_stderr(text);
        if !text.ends_with(b"\n") {
            write_c_stderr(b"\n");
        }
    }

    if ui_client_channel_id.get() != 0 {
        // A UI client has no buffers to preserve.
        os_exit(1);
    }

    ml_close_notmod();

    // One sync writes every swap file, so stop at the first buffer that has
    // one.
    if buffers().any(Buf::has_swap_file) {
        if errmsg.is_some() {
            write_c_stderr(b"Nvim: preserving files...\n");
        }
        ml_sync_all(0, 0, true);
    }

    // Close the memfiles without deleting them.
    ml_close_all(false);

    if errmsg.is_some() {
        write_c_stderr(b"Nvim: Finished.\n");
    }

    getout(1);
}
