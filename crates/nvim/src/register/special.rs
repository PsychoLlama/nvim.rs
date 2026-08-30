//! Registers whose contents are computed, not stored.
//!
//! `"=` is an expression: [`get_expr_register`] prompts for it,
//! [`set_expr_line`] keeps the source for a repeat, and [`get_expr_line`]
//! evaluates it -- so *reading* this register runs arbitrary Vimscript, which
//! is why every caller has to cope with the buffer having changed underneath
//! it, and why the evaluation is depth-limited.
//!
//! [`get_spec_reg`] is the rest of the read-only set: `"%` the file name, `"#`
//! the alternate file, `":` the last command line, `"/` the last search
//! pattern, `".` the last insert, `"_` the black hole -- plus the four that
//! read the *buffer* around the cursor, which are what CTRL-R CTRL-W and its
//! friends insert.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::guard::Depth;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::file_search::FileNameOpts;
use crate::types::NUL;

/// Prompt for the `"=` expression on the command line.
///
/// Answers `'='` once it is stored, or `NUL` if the prompt was abandoned. An
/// empty answer leaves the previous expression in place, so that `"=<CR>`
/// repeats it.
///
/// # Safety
/// Runs the command line, and so arbitrary autocommands.
pub unsafe fn get_expr_register() -> c_int {
    // SAFETY: running the command line is the caller's promise.
    let new_line = unsafe { getcmdline('=' as c_int, 0, 0, true) };
    if new_line.is_null() {
        return NUL; // cancelled
    }
    // SAFETY: a non-null answer is an allocated, NUL-terminated string, so
    // its first byte is readable.
    if c_int::from(unsafe { *new_line }) == NUL {
        // SAFETY: the empty answer is ours and nothing else points at it.
        unsafe { xfree(new_line as *mut c_void) }; // keep the previous expression
    } else {
        // SAFETY: an allocated, NUL-terminated string, handed over.
        unsafe { set_expr_line(new_line) };
    }
    '=' as c_int
}

/// Set the `"=` expression, taking ownership of `new_line`.
///
/// # Safety
/// `new_line` must be an allocated, NUL-terminated string.
pub unsafe fn set_expr_line(new_line: *mut c_char) {
    // SAFETY: `expr_line` holds an allocation this module made, or null.
    unsafe { xfree(expr_line.get() as *mut c_void) };
    expr_line.set(new_line);
}

/// Evaluate the `"=` expression and answer the result, allocated.
///
/// Null when no expression has been set. The evaluation is nested at most ten
/// deep: past that the *source* is answered instead, which is what stops
/// `let @= = '@='` from recursing forever.
///
/// # Safety
/// Runs arbitrary Vimscript.
pub unsafe fn get_expr_line() -> *mut c_char {
    static nested: GlobalCell<c_int> = GlobalCell::new(0);

    if expr_line.get().is_null() {
        return ::core::ptr::null_mut();
    }
    // Evaluating may set `expr_line` again, so work on a copy.
    //
    // SAFETY: tested non-null just above, and it is a NUL-terminated string.
    let expr_copy = unsafe { xstrdup(expr_line.get()) };
    if nested.get() >= 10 {
        return expr_copy;
    }
    let nesting = Depth::of(&nested);
    // SAFETY: running Vimscript is the caller's promise, and `expr_copy` is a
    // NUL-terminated string this call owns for the duration.
    let rv = unsafe { eval_to_string(expr_copy, true, false) };
    drop(nesting);
    // SAFETY: `expr_copy` is ours and `eval_to_string` kept no pointer to it.
    unsafe { xfree(expr_copy as *mut c_void) };
    rv
}

/// The `"=` expression itself, allocated, without evaluating it.
///
/// # Safety
/// Reads the register store; main thread only.
pub unsafe fn get_expr_line_src() -> *mut c_char {
    if expr_line.get().is_null() {
        return ::core::ptr::null_mut();
    }
    // SAFETY: tested non-null just above, and it is a NUL-terminated string.
    unsafe { xstrdup(expr_line.get()) }
}

/// The contents of a computed register.
///
/// Answers false when `regname` is not one of them (or when a buffer-reading
/// one is asked for without `errmsg`, which is the caller saying it only
/// wants an answer it can get without side effects). `*allocated` says
/// whether the caller must free `*argp`.
///
/// `errmsg` also turns on the error messages for a register that has nothing
/// in it -- E29 for `".`, E30 for `":`, E35 for `"/`.
///
/// # Safety
/// `argp` and `allocated` must be writable. `"=` runs arbitrary Vimscript.
pub unsafe fn get_spec_reg(
    regname: c_int,
    argp: *mut *mut c_char,
    allocated: *mut bool,
    errmsg: bool,
) -> bool {
    // The answer is built here and handed over in one place at the end, so
    // that the two writes through the caller's pointers are the whole of
    // this function's unchecked surface.
    let mut value: *mut c_char = ::core::ptr::null_mut();
    let mut owned = false;

    let found = match regname {
        // `"%` -- the current file name.
        c if c == '%' as c_int => {
            if errmsg {
                // SAFETY: main thread, with a current buffer; it only reports.
                let _ = unsafe { check_fname() }; // will give an error message
            }
            value = cur_buf().b_fname;
            true
        }
        // `"#` -- the alternate file name.
        c if c == '#' as c_int => {
            // SAFETY: main thread, with the alternate-file list set up.
            value = unsafe { getaltfname(errmsg) };
            true
        }
        // `"=` -- the expression, evaluated.
        c if c == '=' as c_int => {
            // SAFETY: running Vimscript is this function's own promise.
            value = unsafe { get_expr_line() };
            owned = true;
            true
        }
        // `":` -- the last command line.
        c if c == ':' as c_int => {
            if last_cmdline.get().is_null() && errmsg {
                emsg(gettext(e_nolastcmd));
            }
            value = last_cmdline.get();
            true
        }
        // `"/` -- the last search pattern.
        c if c == '/' as c_int => {
            if last_search_pat().is_null() && errmsg {
                emsg(gettext(e_noprevre));
            }
            value = last_search_pat();
            true
        }
        // `".` -- the last inserted text.
        c if c == '.' as c_int => {
            // SAFETY: main thread; it answers a fresh allocation or null.
            value = unsafe { get_last_insert_save() };
            owned = true;
            if value.is_null() && errmsg {
                emsg(gettext(e_noinstext));
            }
            true
        }
        // CTRL-R CTRL-F / CTRL-P -- the file name under the cursor, the
        // second form expanded to a full path.  Reading the buffer is a side
        // effect, so it only happens for a caller that asked for messages.
        Ctrl_F | Ctrl_P if errmsg => {
            let opts =
                FileNameOpts::MESS | FileNameOpts::HYP | FileNameOpts::EXP.when(regname == Ctrl_P);
            // SAFETY: main thread, with a cursor on a line of the buffer; a
            // null `count` is how it is told there is no count to report.
            value = unsafe { file_name_at_cursor(opts, 1, ::core::ptr::null_mut()) };
            owned = true;
            true
        }
        // CTRL-R CTRL-W / CTRL-A -- the word, or the WORD, under the cursor.
        Ctrl_W | Ctrl_A if errmsg => {
            let find = if regname == Ctrl_W {
                FIND_IDENT | FIND_STRING
            } else {
                FIND_STRING
            };
            // The identifier is found in the buffer's own line, so it has to
            // be copied out before anything else can move the line.
            let mut ident: *mut c_char = ::core::ptr::null_mut();
            // SAFETY: main thread, with a cursor on a line of the buffer;
            // `ident` is a writable local and a null `textcol` asks for none.
            let cnt =
                unsafe { find_ident_under_cursor(&raw mut ident, find, ::core::ptr::null_mut()) };
            value = if cnt != 0 {
                // SAFETY: a non-zero answer means `ident` points at that many
                // bytes of the cursor's line.
                unsafe { xmemdupz(ident as *const c_void, cnt) as *mut c_char }
            } else {
                ::core::ptr::null_mut()
            };
            owned = true;
            true
        }
        // CTRL-R CTRL-L -- the whole cursor line.
        Ctrl_L if errmsg => {
            // SAFETY: main thread; the cursor is on a line of its own window's
            // buffer, and the line stays put until the buffer changes.
            value = unsafe { ml_get_buf(cur_win().w_buffer, cur_win().w_cursor.lnum) };
            true
        }
        // `"_` -- the black hole, which reads as empty.
        c if c == '_' as c_int => {
            value = c"".as_ptr().cast_mut();
            true
        }
        _ => false,
    };

    // SAFETY: the caller promises both pointers are writable.
    unsafe { *argp = value };
    // SAFETY: as above.
    unsafe { *allocated = owned };
    found
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
