//! `-s` script input: reading keys from a file.
//!
//! [`openscript`] pushes a file onto the `scriptin` stack (up to `NSCRIPT`
//! deep) and `inchar` reads a byte at a time from the innermost one until
//! EOF, when [`closescript`] pops it. [`updatescript`] is the other
//! direction: the `'scriptout'` copy of what was typed.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::MAXPATHL;
use core::ffi::{c_char, c_int};

/// The `scriptin` entry `curscript` names.
///
/// # Safety
/// `at` must be a valid index into the stack.
pub(crate) unsafe fn script_at(at: c_int) -> *mut FileDescriptor {
    // SAFETY (this body): the caller's promise -- `at` is a valid index into
    // the fifteen-entry stack.
    unsafe { scriptin.ptr().cast::<FileDescriptor>().offset(at as isize) }
}

/// Open a script file for `:source!`, and with `directly` run its commands
/// there and then.
///
/// # Safety
/// `name` must point at a NUL-terminated file name.
pub unsafe fn openscript(name: *mut c_char, directly: bool) {
    // The expanded name; upstream shares `NameBuff`, which the error path
    // below reaches the message machinery through.
    let mut expanded = [0 as c_char; MAXPATHL as usize];
    if curscript.get() + 1 == NSCRIPT as c_int {
        // SAFETY (this body): `expanded` is this frame's own `MAXPATHL`
        // buffer, and `curscript` was just checked to be in range of the
        // stack.
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
    unsafe { expand_env(name, expanded.as_mut_ptr(), MAXPATHL) };
    let error = unsafe {
        file_open(
            script_at(curscript.get()),
            expanded.as_ptr(),
            kFileReadOnly as c_int,
            0,
        )
    };
    if error != 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
        let (name, arg1) = unsafe { (c_str(name), c_str(uv_strerror(error))) };
        semsg!("E484: Can't open file {name}: {arg1}");
        curscript.set(curscript.get() - 1);
        return;
    }
    unsafe { save_typebuf() };

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
    let mut oa: oparg_T = unsafe { core::mem::zeroed() };
    unsafe { clear_oparg(&raw mut oa) };
    finish_op.set(false);

    let started_at = curscript.get();
    while {
        unsafe { update_topline_cursor() }; // cursor position and topline
        unsafe { normal_cmd(&raw mut oa, false) }; // one command
        vpeekc(); // check for end of file
        curscript.get() >= started_at
    } {}

    State.set(save_state);
    msg_scroll.set(save_msg_scroll);
    restart_edit.set(save_restart_edit);
    finish_op.set(save_finish_op);
}

/// Close the innermost script and put back the typeahead it displaced.
///
/// # Safety
/// A script must be open.
pub(crate) unsafe fn closescript() {
    debug_assert!(curscript.get() >= 0);
    // SAFETY (this body): `curscript` names an open entry of the stack.
    unsafe { free_typebuf() };
    restore_saved_typebuf(curscript.get());

    unsafe { file_close(script_at(curscript.get()), false) };
    curscript.set(curscript.get() - 1);
}

/// Open the `-s` script, which is always the outermost one.
///
/// The name `-` means standard input. Answers false, with a message on
/// stderr, when the file cannot be read.
///
/// # Safety
/// `scriptin_name` must point at a NUL-terminated file name.
pub unsafe fn open_scriptin(scriptin_name: *mut c_char) -> bool {
    debug_assert!(curscript.get() == -1);
    curscript.set(curscript.get() + 1);

    // SAFETY (this body): as [`openscript`] -- the stack entry is in range.
    let error = if unsafe { strequal(scriptin_name, c"-".as_ptr()) } {
        unsafe { file_open_stdin(script_at(0)) }
    } else {
        unsafe {
            file_open(
                script_at(0),
                scriptin_name,
                kFileReadOnly as c_int | kFileNonBlocking as c_int,
                0,
            )
        }
    };
    if error != 0 {
        unsafe {
            fprintf(
                stderr,
                gettext(c"Cannot open for reading: \"%s\": %s\n").as_ptr(),
                scriptin_name,
                uv_strerror(error),
            )
        };
        curscript.set(curscript.get() - 1);
        return false;
    }
    unsafe { save_typebuf() };
    true
}

/// Whether keys are being read from a script file.
pub fn using_script() -> c_int {
    c_int::from(curscript.get() >= 0)
}

/// Called just before a blocking wait, so after waiting `'updatetime'` for a
/// character to arrive.
///
/// # Safety
/// Callable at any time.
pub unsafe fn before_blocking() {
    // SAFETY (this body): reads the script stack and syncs the editor's own
    // state.
    unsafe { updatescript(0) };
    if may_garbage_collect.get() {
        unsafe { garbage_collect(false) };
    }
}

/// Copy a typed character to the `'scriptout'` file, and sync memfiles when
/// enough have gone by.
///
/// `c == 0` means "we have been waiting a while", which syncs unconditionally
/// and is where the idle fsync happens; otherwise the sync waits until
/// `'updatecount'` characters have been typed.
///
/// # Safety
/// Callable at any time.
pub(crate) unsafe fn updatescript(c: c_int) {
    /// Characters typed since the last sync.
    static count: GlobalCell<c_int> = GlobalCell::new(0);

    if c != 0 && !scriptout.get().is_null() {
        // SAFETY (this body): `scriptout` is the open `'scriptout'` stream, or
        // null, which `putc` is not reached with.
        unsafe { putc(c, scriptout.get()) };
    }
    let idle = c == 0;
    if idle
        || (p_uc.get() > 0 && {
            count.set(count.get() + 1);
            count.get() as OptInt >= p_uc.get()
        })
    {
        // Always fsync at idle (CursorHold).
        unsafe { ml_sync_all(c_int::from(idle), 1, p_fs.get() != 0 || idle) };
        count.set(0);
    }
}
