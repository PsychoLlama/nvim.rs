//! Running a child process to completion and collecting what it wrote.
//!
//! [`os_system`] is the public entry point; everything `:!`, `system()` and
//! wildcard expansion do goes through [`do_os_system`], which spawns through
//! libuv, pumps the loop until the child exits, and then either hands the
//! output back to the caller or leaves it on the screen.
//!
//! Only the POSIX build was transpiled: upstream also forces the Windows
//! console code page to UTF-8 around the spawn and clears
//! `$NoDefaultCurrentDirectoryInExePath`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::throttle::{Ring, out_data_cb, out_data_decide_throttle, out_data_ring};
use super::*;
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
use crate::guard::Suppress;
use crate::main::{got_int, lines_left, msg_no_more};
use crate::memory::{xfree, xrealloc, xstrlcpy};
use crate::message::{msg_end, msg_outtrans, msg_putchar, msg_sb_eol, msg_start};
use crate::message_fmt::c_str;
use crate::msg_schedule_semsg;
use crate::os::cshim::gettext;
use crate::types::{LibuvProc, MAXPATHL, MultiQueue, Proc, RStream, Stream, WBuffer};
use crate::ui::{ui_busy_start, ui_busy_stop};

/// Synchronously run a command, in the shell only if `argv` says so.
///
/// ```c
/// char *output = NULL;
/// size_t nread = 0;
/// char *argv[] = { "ls", "-la", NULL };
/// int exitcode = os_system(argv, NULL, 0, &output, &nread);
/// ```
///
/// `argv` is consumed. `input` is fed to the child's stdin (NULL for none) and
/// `len` is its length. `output` is where the collected output is stored — it
/// is set to NULL when the child wrote nothing, and passing NULL for it
/// discards the output entirely; `nread` then holds the length.
///
/// Answers the child's exit code, or -1 if it could not be started.
///
/// # Safety
/// `argv` must be a NULL-terminated, owned argument vector; `input` readable
/// for `len` bytes or NULL; `output` and `nread` writable or NULL.
pub unsafe fn os_system(
    argv: *mut *mut c_char,
    input: *const c_char,
    len: size_t,
    output: *mut *mut c_char,
    nread: *mut size_t,
) -> c_int {
    // SAFETY: the caller's contract.
    unsafe { do_os_system(argv, input, len, output, nread, true, false) }
}

/// The one implementation behind [`os_system`] and [`os_call_shell`].
///
/// `silent` suppresses the "shell failed to start" report and
/// `forward_output` sends the child's output to the screen rather than to
/// `output`.
///
/// # Safety
/// As [`os_system`].
pub(crate) unsafe fn do_os_system(
    argv: *mut *mut c_char,
    input: *const c_char,
    len: size_t,
    output: *mut *mut c_char,
    nread: *mut size_t,
    silent: bool,
    forward_output: bool,
) -> c_int {
    let mut exitcode = -1;

    out_data_decide_throttle(0); // Initialise the throttle decider.
    out_data_ring(Ring::Reset); // Initialise the output ring buffer.
    let has_input = !input.is_null() && len > 0;

    // Where the output accumulates, and what libuv calls with each chunk.
    let mut buf = STRINGBUILDER_INIT;
    let data_cb: stream_read_cb = if forward_output {
        Some(out_data_cb)
    } else if output.is_null() {
        None
    } else {
        Some(system_data_cb)
    };

    // SAFETY: the caller's contract; `main_loop` is the process's event loop,
    // and `proc` points into `uvproc`, which outlives every use of it.
    unsafe {
        if !nread.is_null() {
            *nread = 0;
        }

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
                msg_puts(gettext(c"\nshell failed to start: ").as_ptr());
                msg_outtrans(uv_strerror(status), 0, false);
                msg_puts(c": ".as_ptr());
                msg_outtrans(prog.as_ptr(), 0, false);
                msg_putchar('\n' as c_int);
            }
            multiqueue_free(events);
            return exitcode;
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
            let input_buffer: *mut WBuffer = wstream_new_buffer(input.cast_mut(), len, 1, None);
            if wstream_write(&raw mut (*proc).in_0, input_buffer) != 0 {
                // Could not write: stop the child and tell the user.
                proc_stop(proc);
                multiqueue_free(events);
                return exitcode;
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

        if !output.is_null() {
            debug_assert!(!nread.is_null());
            if buf.size == 0 {
                // Nothing came back.
                *output = ptr::null_mut();
                *nread = 0;
                xfree(buf.items.cast());
            } else {
                *nread = buf.size;
                // NUL-terminate so the output is usable as a C string.
                Kvec::new(&mut buf.size, &mut buf.capacity, &mut buf.items).push(0);
                *output = buf.items;
            }
        }

        debug_assert!(multiqueue_empty(events));
        multiqueue_free(events);
    }
    exitcode
}

/// The `stream_read_cb` that accumulates output into the caller's buffer.
///
/// # Safety
/// An `stream_read_cb`: `buf` readable for `count` bytes, `data` a live
/// [`StringBuilder`].
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
    // SAFETY: the caller's contract. `kv_concat_len` rounds the capacity up
    // to the next power of two, which the unit suite's `alloc_log` asserts on
    // — do not tighten it to an exact fit.
    unsafe {
        let dbuf = &mut *(data as *mut StringBuilder);
        if dbuf.capacity < dbuf.size + count {
            let mut capacity = dbuf.size + count - 1;
            capacity |= capacity >> 1;
            capacity |= capacity >> 2;
            capacity |= capacity >> 4;
            capacity |= capacity >> 8;
            capacity |= capacity >> 16;
            dbuf.capacity = capacity + 1;
            dbuf.items = xrealloc(dbuf.items.cast(), dbuf.capacity) as *mut c_char;
        }
        debug_assert!(!dbuf.items.is_null());
        ptr::copy_nonoverlapping(buf, dbuf.items.add(dbuf.size), count);
        dbuf.size += count;
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
