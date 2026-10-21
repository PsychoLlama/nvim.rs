//! `-s` script input: reading keys from a file.
//!
//! [`openscript`] pushes a file onto the `scriptin` stack (up to `NSCRIPT`
//! deep) and `inchar` reads a byte at a time from the innermost one until
//! EOF, when [`closescript`] pops it. [`updatescript`] is the other
//! direction: the `'scriptout'` copy of what was typed.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::message_fmt::msg_cstr;
use crate::os::env::expand_env_into;
use crate::os::fileio::ReadFile;
use crate::os::fs::os_strerror;
use crate::semsg;
use crate::types::MAXPATHL;
use core::ffi::{CStr, c_int};
use std::ffi::CString;
use std::io::Write;

/// Streams to read script (`-s` / `:source!`) input from, innermost last;
/// `curscript` names the innermost open one.
static scriptin: GlobalCell<[Option<ReadFile>; NSCRIPT as usize]> =
    GlobalCell::new([const { None }; NSCRIPT as usize]);

/// The stack slot of script `at`, which is in range of the stack.
fn slot(at: c_int) -> usize {
    usize::try_from(at).expect("a script on the stack")
}

/// Read into `buf` from script `at`: what [`ReadFile::read`] answers, or -1
/// when nothing is open there.
pub(crate) fn read_script(at: c_int, buf: &mut [u8]) -> ptrdiff_t {
    scriptin.with_mut(|stack| stack[slot(at)].as_mut().map_or(-1, |file| file.read(buf)))
}

/// Open a script file for `:source!`, and with `directly` run its commands
/// there and then.
pub fn openscript(name: &CStr, directly: bool) {
    if curscript.get() + 1 == NSCRIPT.cast_signed() {
        emsg(gettext(e_nesting));
        return;
    }
    // Not in the sandbox: the commands would run later, possibly outside
    // it.
    if check_secure() {
        return;
    }
    if ignore_script.get() {
        // Not reading from a script, so don't open one either.
        return;
    }

    curscript.set(curscript.get() + 1);
    // The expanded name; upstream expands into `NameBuff`, cut to
    // `MAXPATHL`.
    let expanded = CString::new(expand_env_into(name, MAXPATHL as usize + 1))
        .expect("an expansion up to its NUL");
    match ReadFile::open(&expanded, kFileReadOnly) {
        Ok(file) => scriptin.with_mut(|stack| stack[slot(curscript.get())] = Some(file)),
        Err(error) => {
            let (name, why) = (msg_cstr(name), msg_cstr(os_strerror(error)));
            semsg!("E484: Can't open file {name}: {why}");
            curscript.set(curscript.get() - 1);
            return;
        }
    }
    save_typebuf();

    if !directly {
        return;
    }

    // Run the commands right now, which is what `:source!` after
    // `:global` or `:argdo`, or inside a loop, or with another command
    // following, needs. The display is not updated while this runs. Not
    // done always -- "make test" would fail.
    let save_state = State.get();
    let save_restart_edit = restart_edit.get();
    let save_finish_op = finish_op.get();
    let save_msg_scroll = msg_scroll.get();

    State.set(MODE_NORMAL);
    msg_scroll.set(0); // no message scrolling in Normal mode
    restart_edit.set(0); // don't go to Insert mode
    finish_op.set(false);

    let started_at = curscript.get();
    crate::normal::with_frame_op(OpArg::default(), |op| {
        while {
            update_topline_cursor(); // cursor position and topline
            normal_cmd(op, false); // one command
            vpeekc(); // check for end of file
            curscript.get() >= started_at
        } {}
    });

    State.set(save_state);
    msg_scroll.set(save_msg_scroll);
    restart_edit.set(save_restart_edit);
    finish_op.set(save_finish_op);
}

/// Close the innermost script and put back the typeahead it displaced.
pub(crate) fn closescript() {
    debug_assert!(curscript.get() >= 0);
    restore_saved_typebuf(curscript.get());

    if let Some(file) = scriptin.with_mut(|stack| stack[slot(curscript.get())].take()) {
        let _ = file.close();
    }
    curscript.set(curscript.get() - 1);
}

/// Open the `-s` script, which is always the outermost one.
///
/// The name `-` means standard input. Answers false, with a message on
/// stderr, when the file cannot be read.
pub fn open_scriptin(scriptin_name: &CStr) -> bool {
    debug_assert!(curscript.get() == -1);
    curscript.set(curscript.get() + 1);

    let opened = if scriptin_name.to_bytes() == b"-" {
        ReadFile::stdin()
    } else {
        ReadFile::open(scriptin_name, kFileReadOnly | kFileNonBlocking)
    };
    match opened {
        Ok(file) => scriptin.with_mut(|stack| stack[0] = Some(file)),
        Err(error) => {
            let message = printf_s(
                gettext(c"Cannot open for reading: \"%s\": %s\n").to_bytes(),
                &[scriptin_name.to_bytes(), os_strerror(error).to_bytes()],
            );
            let _ = std::io::stderr().write_all(&message);
            curscript.set(curscript.get() - 1);
            return false;
        }
    }
    save_typebuf();
    true
}

/// `template` with each `%s` replaced by the next of `args`: the one
/// conversion the translated `-s` message has.
fn printf_s(template: &[u8], args: &[&[u8]]) -> Vec<u8> {
    let mut out = Vec::with_capacity(template.len() + 64);
    let mut args = args.iter();
    let mut rest = template;
    while let Some(at) = rest.windows(2).position(|pair| pair == b"%s") {
        out.extend_from_slice(&rest[..at]);
        out.extend_from_slice(args.next().copied().unwrap_or_default());
        rest = &rest[at + 2..];
    }
    out.extend_from_slice(rest);
    out
}

/// Whether keys are being read from a script file.
pub fn using_script() -> c_int {
    c_int::from(curscript.get() >= 0)
}

/// Called just before a blocking wait, so after waiting `'updatetime'` for a
/// character to arrive.
pub fn before_blocking() {
    updatescript(0);
    if may_garbage_collect.get() {
        garbage_collect(false);
    }
}

/// Copy a typed character to the `'scriptout'` file, and sync memfiles when
/// enough have gone by.
///
/// `c == 0` means "we have been waiting a while", which syncs unconditionally
/// and is where the idle fsync happens; otherwise the sync waits until
/// `'updatecount'` characters have been typed.
pub(crate) fn updatescript(c: c_int) {
    if c != 0 {
        // `putc`'s `(unsigned char)c`: every caller passes a byte.
        scriptout.with(|file| file.as_ref().map(|file| file.putc(c.to_le_bytes()[0])));
    }
    let idle = c == 0;
    if idle
        || (p_uc() > 0 && {
            typed_since_sync.set(typed_since_sync.get() + 1);
            OptInt::from(typed_since_sync.get()) >= p_uc()
        })
    {
        // Always fsync at idle (CursorHold).
        ml_sync_all(c_int::from(idle), 1, p_fs() || idle);
        typed_since_sync.set(0);
    }
}
