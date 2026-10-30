//! Running a child process to completion and collecting what it wrote.
//!
//! [`os_system_capture`] is the public entry point; everything `:!`, `system()` and
//! wildcard expansion do goes through [`do_os_system`], which spawns through
//! libuv, pumps the loop until the child exits, and then either hands the
//! output back to the caller or leaves it on the screen.
//!
//! Only the POSIX build was transpiled: upstream also forces the Windows
//! console code page to UTF-8 around the spawn and clears
//! `$NoDefaultCurrentDirectoryInExePath`.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::throttle::{Ring, out_data_cb, out_data_decide_throttle, out_data_ring};
use super::*;
use crate::cstr;
use crate::event::libuv::{uv_err_name, uv_strerror};
use crate::event::libuv_proc::libuv_proc_init;
use crate::event::r#loop::loop_poll_events;
use crate::event::multiqueue::{multiqueue_empty, multiqueue_free, multiqueue_new_child};
use crate::event::proc::{proc_spawn, proc_stop, proc_wait};
use crate::event::rstream::{rstream_init, rstream_start};
use crate::event::stream::stream_may_close;
use crate::event::wstream::{
    wstream_init, wstream_new_buffer, wstream_set_write_cb, wstream_write,
};
use crate::getchar::state::got_int;
use crate::guard::Suppress;
use crate::memory::xstrlcpy;
use crate::message::state::{lines_left, msg_no_more};
use crate::message::{msg_display, msg_end, msg_putchar, msg_sb_eol, msg_start};
use crate::message_fmt::c_str;
use crate::msg_schedule_semsg;
use crate::os::cshim::gettext;
use crate::types::{LibuvProc, MAXPATHL, MultiQueue, Proc, RStream, Stream, WBuffer};
use crate::ui::{ui_busy_start, ui_busy_stop};

/// Synchronously run `argv` (in the shell only if it says so) with `input`
/// on standard input, answering the exit code (-1 if the child could not be
/// started) and everything it wrote, or `None` when it wrote nothing.
pub fn os_system_capture(argv: Argv, input: &[u8]) -> (c_int, Option<Vec<u8>>) {
    // SAFETY: the vector is handed over; the child releases it.
    unsafe { do_os_system(argv.into_raw(), input, true, true, false) }
}

/// The one implementation behind [`os_system_capture`] and
/// [`os_call_shell`]: run `argv`, feeding it `input`, and answer its exit
/// code (-1 when it could not be started or fed) and, with `capture`, what
/// it wrote — `None` when that was nothing.
///
/// `silent` suppresses the "shell failed to start" report and
/// `forward_output` sends the child's output to the screen rather than to
/// the answer.
///
/// # Safety
/// `argv` must be a NULL-terminated, owned argument vector, which the child
/// releases.
pub(crate) unsafe fn do_os_system(
    argv: *mut *mut c_char,
    input: &[u8],
    capture: bool,
    silent: bool,
    forward_output: bool,
) -> (c_int, Option<Vec<u8>>) {
    let mut exitcode = -1;

    out_data_decide_throttle(0); // Initialise the throttle decider.
    out_data_ring(Ring::Reset); // Initialise the output ring buffer.
    let has_input = !input.is_empty();

    // Where the output accumulates, and what libuv calls with each chunk.
    let mut buf: Vec<u8> = Vec::new();
    let data_cb: stream_read_cb = if forward_output {
        Some(out_data_cb)
    } else if !capture {
        None
    } else {
        Some(system_data_cb)
    };

    // SAFETY: the caller's contract; `main_loop` is the process's event loop,
    // and `proc` points into `uvproc`, which outlives every use of it, as
    // `buf` outlives every callback that is handed its address.
    unsafe {
        // Copy the program name in case it has to be reported.
        let mut prog: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
        xstrlcpy(prog.as_mut_ptr(), *argv, MAXPATHL as usize);

        let mut uvproc: LibuvProc = libuv_proc_init(main_loop.ptr(), (&raw mut buf).cast());
        let proc: *mut Proc = &raw mut uvproc.proc;
        let events: *mut MultiQueue = multiqueue_new_child((*main_loop.ptr()).events);
        (*proc).events = events;
        (*proc).argv = argv;

        let status = proc_spawn(proc, has_input, true, true);
        if status != 0 {
            loop_poll_events(main_loop.ptr(), 0);
            // Probably 'shell' is not executable.
            if !silent {
                msg_str(gettext(c"\nshell failed to start: "));
                msg_display(cstr::at(uv_strerror(status)), 0, false);
                msg_str(c": ");
                msg_display(cstr::in_chars(&prog), 0, false);
                msg_putchar('\n' as c_int);
            }
            multiqueue_free(events);
            return (exitcode, None);
        }

        // Unlike process events, stream events are not queued: they are dealt
        // with as fast as possible so the streams are not closed while the OS
        // buffer still holds data the child wrote before exiting.
        if has_input {
            wstream_init(&raw mut (*proc).in_0, 0);
        }
        rstream_init(&raw mut (*proc).out);
        rstream_start(&raw mut (*proc).out, data_cb, (&raw mut buf).cast());
        rstream_init(&raw mut (*proc).err);
        rstream_start(&raw mut (*proc).err, data_cb, (&raw mut buf).cast());

        if has_input {
            // Borrowed, not released (no free callback): the write is done
            // with it before `proc_wait` returns.
            let data = input.as_ptr().cast::<c_char>().cast_mut();
            let input_buffer: *mut WBuffer = wstream_new_buffer(data, input.len(), 1, None);
            if wstream_write(&raw mut (*proc).in_0, input_buffer) != 0 {
                // Could not write: stop the child and tell the user.
                proc_stop(proc);
                multiqueue_free(events);
                return (exitcode, None);
            }
            // Close stdin once everything has been written.
            wstream_set_write_cb(&raw mut (*proc).in_0, Some(shell_write_cb), ptr::null_mut());
        }

        // Start the busy indicator here so pumping the loop below does not
        // change the busy state.
        ui_busy_start();
        ui_flush();
        if forward_output {
            msg_sb_eol();
            msg_start();
            msg_no_more.set(true);
            lines_left.set(-1);
        }
        exitcode = proc_wait(proc, -1, ptr::null_mut());
        if !got_int.get() && out_data_decide_throttle(0) {
            // The last chunk of output was skipped; show it now.
            out_data_ring(Ring::Print);
        }
        if forward_output {
            // The caller decides whether `wait_return()` is invoked.
            let no_prompt = Suppress::wait_return();
            msg_end();
            drop(no_prompt);
            msg_no_more.set(false);
        }
        ui_busy_stop();

        debug_assert!(multiqueue_empty(events));
        multiqueue_free(events);
    }
    (exitcode, (capture && !buf.is_empty()).then_some(buf))
}

/// The `stream_read_cb` that accumulates output into the caller's buffer.
///
/// # Safety
/// An `stream_read_cb`: `buf` readable for `count` bytes, `data` a live
/// `Vec<u8>`.
unsafe fn system_data_cb(
    _stream: *mut RStream,
    buf: *const c_char,
    count: size_t,
    data: *mut c_void,
    _eof: bool,
) -> size_t {
    if count == 0 {
        return count;
    }
    // SAFETY: the caller's contract: `data` is `do_os_system`'s buffer, and
    // `buf` is the stream's own, so the two do not overlap.
    unsafe {
        let output = &mut *data.cast::<Vec<u8>>();
        output.extend_from_slice(slice::from_raw_parts(buf.cast::<u8>(), count));
    }
    count
}

/// Report a failed write to the child's stdin, then close it.
///
/// # Safety
/// A `stream_write_cb`; `stream` must be live.
unsafe fn shell_write_cb(stream: *mut Stream, _data: *mut c_void, status: c_int) {
    // SAFETY: the caller's contract; `msg_schedule_semsg` is printf-shaped
    // and `uv_err_name` answers a static string.
    unsafe {
        if status != 0 {
            // Happens when input is sent to a backgrounded shell command:
            // `:call system("cat - &", "foo")`. #3529 #5241
            msg_schedule_semsg!(
                "E5677: Error writing input to shell-command: {}",
                c_str(uv_err_name(status))
            );
        }
        stream_may_close(stream);
    }
}
