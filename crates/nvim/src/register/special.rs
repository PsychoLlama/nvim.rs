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
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::guard::Depth;
use crate::memory::XString;
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
pub fn get_expr_register() -> c_int {
    let new_line = getcmdline('=' as c_int, 0, 0, true);
    if new_line.is_null() {
        return NUL; // cancelled
    }
    // SAFETY: a non-null answer is an allocated, NUL-terminated string, so
    // its first byte is readable.
    if c_int::from(unsafe { *new_line }) == NUL {
        // SAFETY: the empty answer is ours and nothing else points at it.
        unsafe { xfree(new_line.cast::<c_void>()) }; // keep the previous expression
    } else {
        // SAFETY: an allocated, NUL-terminated string, handed over.
        set_expr_line(unsafe { XString::from_raw(new_line) });
    }
    '=' as c_int
}

/// Set the `"=` expression.
pub fn set_expr_line(new_line: XString) {
    expr_line.set(Some(new_line));
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

    // Evaluating may set `expr_line` again, so work on a copy.
    let Some(expression) = expr_line.with(Clone::clone) else {
        return ::core::ptr::null_mut();
    };
    if nested.get() >= 10 {
        return expression.into_raw();
    }
    let nesting = Depth::of(&nested);
    // The copy is this call's own, whatever the expression does to `"=`.
    let rv = eval_to_string(&expression, true, false);
    drop(nesting);
    rv.map_or(::core::ptr::null_mut(), XString::into_raw)
}

/// The `"=` expression itself, allocated, without evaluating it.
///
/// # Safety
/// Reads the register store; main thread only.
pub unsafe fn get_expr_line_src() -> *mut c_char {
    expr_line.with(|line| {
        line.clone()
            .map_or(::core::ptr::null_mut(), XString::into_raw)
    })
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
                let _ = check_fname(); // will give an error message
            }
            value = Buf::current().name.shown_ptr();
            true
        }
        // `"#` -- the alternate file name.
        c if c == '#' as c_int => {
            value = getaltfname(errmsg);
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
            // A copy: the caller may run commands while it holds it.
            match last_cmdline.with(Clone::clone) {
                Some(line) => {
                    value = line.into_raw();
                    owned = true;
                }
                None if errmsg => {
                    emsg(gettext(e_nolastcmd));
                }
                None => {}
            }
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
                unsafe { xmemdupz(ident as *const c_void, cnt).cast::<c_char>() }
            } else {
                ::core::ptr::null_mut()
            };
            owned = true;
            true
        }
        // CTRL-R CTRL-L -- the whole cursor line.
        Ctrl_L if errmsg => {
            // Copied rather than lent: every caller here goes on to insert
            // the text, and inserting re-enters the editor, which may move
            // or free the block the line was read out of.
            let win = Win::current();
            value = XString::from_bytes(win.buffer().lines().line(win.w_cursor.lnum)).into_raw();
            owned = true;
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
