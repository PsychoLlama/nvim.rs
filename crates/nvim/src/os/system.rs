//! Running a child process to completion and collecting what it wrote.
//!
//! [`os_system_capture`] is the public entry point; everything `:!`, `system()` and
//! wildcard expansion do goes through [`do_os_system`], which spawns through
//! libuv, pumps the loop until the child exits, and then either hands the
//! output back to the caller or leaves it on the screen. What the command
//! is, and what becomes of its output on screen, is `crate::shell`'s.
//!
//! [`argv_into_raw`] and [`free_raw_argv`] are the one place an [`Argv`]
//! becomes the NULL-terminated `char **` a spawned child owns, and back.
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

use crate::cstr;
use crate::event::libuv_proc::libuv_proc_init;
use crate::event::r#loop::loop_poll_events;
use crate::event::proc::{proc_spawn, proc_stop, proc_wait};
use crate::event::rstream::{rstream_init, rstream_start};
use crate::event::stream::stream_may_close;
use crate::event::wstream::{
    wstream_init, wstream_new_buffer, wstream_set_write_cb, wstream_write,
};
use crate::event_queue::{self, ChildQueue};
use crate::getchar::state::got_int;
use crate::guard::Suppress;
use crate::memory::ThinCString;
use crate::message::msg_str;
use crate::message::state::{lines_left, msg_no_more};
use crate::message::{msg_display, msg_end, msg_putchar, msg_sb_eol, msg_start};
use crate::message_fmt::msg_cstr;
use crate::msg_schedule_semsg;
use crate::os::cshim::gettext;
use crate::os::fs::{os_err_name, os_strerror};
use crate::shell::Argv;
use crate::shell::throttle::{Ring, out_data, out_data_decide_throttle, out_data_ring};
use crate::startup::main_loop;
use crate::types::{LibuvProc, MAXPATHL, Proc, RStream, Stream, WBuffer};
use crate::types::{size_t, stream_read_cb};
use crate::ui::ui_flush;
use crate::ui::{ui_busy_start, ui_busy_stop};
use core::ffi::{c_char, c_int, c_void};
use core::{ptr, slice};

/// Synchronously run `argv` (in the shell only if it says so) with `input`
/// on standard input, answering the exit code (-1 if the child could not be
/// started) and everything it wrote, or `None` when it wrote nothing.
pub fn os_system_capture(argv: Argv, input: &[u8]) -> (c_int, Option<Vec<u8>>) {
    do_os_system(argv, input, true, true, false)
}

/// Lay `argv` out as the NULL-terminated vector of NUL-terminated words a
/// child is spawned with; the child owns it until [`free_raw_argv`].
pub(crate) fn argv_into_raw(argv: Argv) -> *mut *mut c_char {
    let slots: Box<[*mut c_char]> = argv
        .into_words()
        .into_iter()
        .map(ThinCString::into_raw)
        .chain([ptr::null_mut()])
        .collect();
    Box::into_raw(slots).cast::<*mut c_char>()
}

/// Release a vector [`argv_into_raw`] laid out.
///
/// # Safety
/// `argv` is null or an answer of [`argv_into_raw`], released once.
pub(crate) unsafe fn free_raw_argv(argv: *mut *mut c_char) {
    if argv.is_null() {
        return;
    }
    // SAFETY: the caller's contract -- a boxed slice of words ending in the
    // null, each word a `ThinCString` taken over once.
    unsafe {
        let mut len = 0;
        while !(*argv.add(len)).is_null() {
            drop(ThinCString::from_raw(*argv.add(len)));
            len += 1;
        }
        drop(Box::from_raw(ptr::slice_from_raw_parts_mut(argv, len + 1)));
    }
}

/// The one implementation behind [`os_system_capture`] and
/// `crate::shell::run_shell`: run `argv`, feeding it `input`, and answer its exit
/// code (-1 when it could not be started or fed) and, with `capture`, what
/// it wrote — `None` when that was nothing.
///
/// `silent` suppresses the "shell failed to start" report and
/// `forward_output` sends the child's output to the screen rather than to
/// the answer.
pub(crate) fn do_os_system(
    argv: Argv,
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
        Some(screen_data_cb)
    } else if !capture {
        None
    } else {
        Some(system_data_cb)
    };

    // Copy the program name in case it has to be reported, cut where
    // upstream's `MAXPATHL` buffer cuts it.
    let prog = argv.program().to_bytes();
    let prog = cstr::owned(&prog[..prog.len().min(MAXPATHL as usize - 1)]);
    let argv = argv_into_raw(argv);

    // SAFETY: `main_loop` is the process's event loop, and `proc` points
    // into `uvproc`, which outlives every use of it, as `buf` outlives every
    // callback that is handed its address; the child owns `argv`.
    unsafe {
        let mut uvproc: LibuvProc = libuv_proc_init(main_loop.ptr(), (&raw mut buf).cast());
        let proc: *mut Proc = &raw mut uvproc.proc;
        let events = ChildQueue::of_main_loop();
        (*proc).events = Some(events.id());
        (*proc).argv = argv;

        let status = proc_spawn(proc, has_input, true, true);
        if status != 0 {
            loop_poll_events(main_loop.ptr(), 0);
            // Probably 'shell' is not executable.
            if !silent {
                msg_str(gettext(c"\nshell failed to start: "));
                msg_display(os_strerror(status), 0, false);
                msg_str(c": ");
                msg_display(&prog, 0, false);
                msg_putchar('\n' as c_int);
            }
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
        exitcode = proc_wait(proc, -1, None);
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

        debug_assert!(event_queue::is_empty(events.id()));
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

/// The `stream_read_cb` `do_os_system` uses when the output goes to the
/// screen rather than to the caller.
///
/// # Safety
/// A `stream_read_cb`: `data` must be readable for `count` bytes and `stream`
/// live.
unsafe fn screen_data_cb(
    stream: *mut RStream,
    data: *const c_char,
    count: size_t,
    _cookie: *mut c_void,
    eof: bool,
) -> size_t {
    if count == 0 {
        return count;
    }
    // SAFETY: the caller's contract.
    let (data, fd) = unsafe {
        (
            slice::from_raw_parts(data.cast::<u8>(), count),
            (*stream).s.fd,
        )
    };
    out_data(data, fd, eof)
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
                msg_cstr(os_err_name(status))
            );
        }
        stream_may_close(stream);
    }
}
