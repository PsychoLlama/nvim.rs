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
    unsafe {
        let new_line = getcmdline('=' as c_int, 0, 0, true);
        if new_line.is_null() {
            return NUL; // cancelled
        }
        if c_int::from(*new_line) == NUL {
            xfree(new_line as *mut c_void); // keep the previous expression
        } else {
            set_expr_line(new_line);
        }
        '=' as c_int
    }
}

/// Set the `"=` expression, taking ownership of `new_line`.
///
/// # Safety
/// `new_line` must be an allocated, NUL-terminated string.
pub unsafe fn set_expr_line(new_line: *mut c_char) {
    unsafe {
        xfree(expr_line.get() as *mut c_void);
        expr_line.set(new_line);
    }
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
    unsafe {
        static nested: GlobalCell<c_int> = GlobalCell::new(0);

        if expr_line.get().is_null() {
            return ::core::ptr::null_mut();
        }
        // Evaluating may set `expr_line` again, so work on a copy.
        let expr_copy = xstrdup(expr_line.get());
        if nested.get() >= 10 {
            return expr_copy;
        }
        *nested.ptr() += 1;
        let rv = eval_to_string(expr_copy, true, false);
        *nested.ptr() -= 1;
        xfree(expr_copy as *mut c_void);
        rv
    }
}

/// The `"=` expression itself, allocated, without evaluating it.
///
/// # Safety
/// Reads the register store; main thread only.
pub unsafe fn get_expr_line_src() -> *mut c_char {
    unsafe {
        if expr_line.get().is_null() {
            return ::core::ptr::null_mut();
        }
        xstrdup(expr_line.get())
    }
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
    unsafe {
        *argp = ::core::ptr::null_mut();
        *allocated = false;

        match regname {
            // `"%` -- the current file name.
            c if c == '%' as c_int => {
                if errmsg {
                    check_fname(); // will give an error message
                }
                *argp = (*curbuf.get()).b_fname;
                true
            }
            // `"#` -- the alternate file name.
            c if c == '#' as c_int => {
                *argp = getaltfname(errmsg);
                true
            }
            // `"=` -- the expression, evaluated.
            c if c == '=' as c_int => {
                *argp = get_expr_line();
                *allocated = true;
                true
            }
            // `":` -- the last command line.
            c if c == ':' as c_int => {
                if last_cmdline.get().is_null() && errmsg {
                    emsg(gettext(&raw const e_nolastcmd as *const c_char));
                }
                *argp = last_cmdline.get();
                true
            }
            // `"/` -- the last search pattern.
            c if c == '/' as c_int => {
                if last_search_pat().is_null() && errmsg {
                    emsg(gettext(&raw const e_noprevre as *const c_char));
                }
                *argp = last_search_pat();
                true
            }
            // `".` -- the last inserted text.
            c if c == '.' as c_int => {
                *argp = get_last_insert_save();
                *allocated = true;
                if (*argp).is_null() && errmsg {
                    emsg(gettext(&raw const e_noinstext as *const c_char));
                }
                true
            }
            // CTRL-R CTRL-F / CTRL-P -- the file name under the cursor, the
            // second form expanded to a full path.
            Ctrl_F | Ctrl_P => {
                if !errmsg {
                    return false;
                }
                *argp = file_name_at_cursor(
                    FileNameOpts::MESS
                        | FileNameOpts::HYP
                        | FileNameOpts::EXP.when(regname == Ctrl_P),
                    1,
                    ::core::ptr::null_mut(),
                );
                *allocated = true;
                true
            }
            // CTRL-R CTRL-W / CTRL-A -- the word, or the WORD, under the
            // cursor.
            Ctrl_W | Ctrl_A => {
                if !errmsg {
                    return false;
                }
                let cnt = find_ident_under_cursor(
                    argp,
                    if regname == Ctrl_W {
                        FIND_IDENT | FIND_STRING
                    } else {
                        FIND_STRING
                    },
                    ::core::ptr::null_mut(),
                );
                *argp = if cnt != 0 {
                    xmemdupz(*argp as *const c_void, cnt) as *mut c_char
                } else {
                    ::core::ptr::null_mut()
                };
                *allocated = true;
                true
            }
            // CTRL-R CTRL-L -- the whole cursor line.
            Ctrl_L => {
                if !errmsg {
                    return false;
                }
                *argp = ml_get_buf((*curwin.get()).w_buffer, (*curwin.get()).w_cursor.lnum);
                true
            }
            // `"_` -- the black hole, which reads as empty.
            c if c == '_' as c_int => {
                *argp = c"".as_ptr().cast_mut();
                true
            }
            _ => false,
        }
    }
}
