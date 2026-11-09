//! Running a child process to completion and collecting what it wrote.
//!
//! [`os_system_capture`] is the public entry point; everything `:!`, `system()` and
//! wildcard expansion do goes through [`do_os_system`], which has the event
//! loop spawn the child and pump until it exits
//! ([`run_to_exit`]), and then either
//! hands the output back to the caller or leaves it on the screen. What the
//! command is, and what becomes of its output on screen, is `crate::shell`'s.
//!
//! Only the POSIX build was transpiled: upstream also forces the Windows
//! console code page to UTF-8 around the spawn and clears
//! `$NoDefaultCurrentDirectoryInExePath`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::event::libuv_proc::{ChildRun, OutputSink, run_to_exit};
use crate::getchar::state::GOT_INT;
use crate::guard::Suppress;
use crate::message::msg_str;
use crate::message::state::{LINES_LEFT, MSG_NO_MORE};
use crate::message::{msg_display, msg_end, msg_putchar, msg_sb_eol, msg_start};
use crate::message_fmt::msg_cstr;
use crate::msg_schedule_semsg;
use crate::os::cshim::gettext;
use crate::os::fs::{os_err_name, os_strerror};
use crate::path::MAXPATHL;
use crate::shell::Argv;
use crate::shell::throttle::{Ring, out_data, out_data_decide_throttle, out_data_ring};
use crate::ui::ui_flush;
use crate::ui::{ui_busy_start, ui_busy_stop};
use core::ffi::c_int;

/// Synchronously run `argv` (in the shell only if it says so) with `input`
/// on standard input, answering the exit code (-1 if the child could not be
/// started) and everything it wrote, or `None` when it wrote nothing.
pub fn os_system_capture(argv: Argv, input: &[u8]) -> (c_int, Option<Vec<u8>>) {
    do_os_system(argv, input, true, true, false)
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
    out_data_decide_throttle(0); // Initialise the throttle decider.
    out_data_ring(Ring::Reset); // Initialise the output ring buffer.

    // Copy the program name in case it has to be reported, cut where
    // upstream's `MAXPATHL` buffer cuts it.
    let prog = argv.program().to_bytes();
    let prog = cstr::owned(&prog[..prog.len().min(MAXPATHL as usize - 1)]);

    // Where the output accumulates, or how it reaches the screen.
    let mut buf: Vec<u8> = Vec::new();
    let mut collect = |bytes: &[u8], _fd: c_int, _eof: bool| {
        buf.extend_from_slice(bytes);
        bytes.len()
    };
    let mut to_screen = |bytes: &[u8], fd: c_int, eof: bool| out_data(bytes, fd, eof);
    let output: Option<OutputSink<'_>> = if forward_output {
        Some(&mut to_screen)
    } else if capture {
        Some(&mut collect)
    } else {
        None
    };

    let before_wait = || {
        // Start the busy indicator here so pumping the loop does not change
        // the busy state.
        ui_busy_start();
        ui_flush();
        if forward_output {
            msg_sb_eol();
            msg_start();
            MSG_NO_MORE.set(true);
            LINES_LEFT.set(-1);
        }
    };
    let exitcode = match run_to_exit(argv, input, output, report_input_written, before_wait) {
        ChildRun::NotStarted(status) => {
            // Probably 'shell' is not executable.
            if !silent {
                msg_str(gettext(c"\nshell failed to start: "));
                msg_display(os_strerror(status), 0, false);
                msg_str(c": ");
                msg_display(&prog, 0, false);
                msg_putchar('\n' as c_int);
            }
            return (-1, None);
        }
        // Could not write: the child is stopped.
        ChildRun::InputRefused => return (-1, None),
        ChildRun::Exited(code) => code,
    };
    if !GOT_INT.get() && out_data_decide_throttle(0) {
        // The last chunk of output was skipped; show it now.
        out_data_ring(Ring::Print);
    }
    if forward_output {
        // The caller decides whether `wait_return()` is invoked.
        let no_prompt = Suppress::wait_return();
        msg_end();
        drop(no_prompt);
        MSG_NO_MORE.set(false);
    }
    ui_busy_stop();
    (exitcode, (capture && !buf.is_empty()).then_some(buf))
}

/// A failed write to the child's stdin; the stdin is closed either way.
fn report_input_written(status: c_int) {
    if status != 0 {
        // Happens when input is sent to a backgrounded shell command:
        // `:call system("cat - &", "foo")`. #3529 #5241
        msg_schedule_semsg!(
            "E5677: Error writing input to shell-command: {}",
            msg_cstr(os_err_name(status))
        );
    }
}
