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

use crate::ex_getln::cmdline_input;
use crate::guard::Depth;
use crate::memory::XString;
use crate::message::e_noalt;
use crate::normal::ident_under_cursor;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::file_search::FileNameOpts;
use crate::types::NUL;

/// Prompt for the `"=` expression on the command line.
///
/// Answers `'='` once it is stored, or `NUL` if the prompt was abandoned. An
/// empty answer leaves the previous expression in place, so that `"=<CR>`
/// repeats it.
pub fn get_expr_register() -> c_int {
    let Some(new_line) = cmdline_input('=' as c_int, 0, true) else {
        return NUL; // cancelled
    };
    // An empty answer keeps the previous expression.
    if !new_line.is_empty() {
        set_expr_line(new_line);
    }
    '=' as c_int
}

/// Set the `"=` expression.
pub fn set_expr_line(new_line: XString) {
    expr_line.set(Some(new_line));
}

/// Evaluate the `"=` expression and answer the result.
///
/// `None` when no expression has been set, or it fails. The evaluation is
/// nested at most ten deep: past that the *source* is answered instead,
/// which is what stops `let @= = '@='` from recursing forever.
///
/// Runs arbitrary Vimscript.
pub fn get_expr_line() -> Option<XString> {
    static nested: GlobalCell<c_int> = GlobalCell::new(0);

    // Evaluating may set `expr_line` again, so work on a copy.
    let expression = expr_line.with(Clone::clone)?;
    if nested.get() >= 10 {
        return Some(expression);
    }
    let _nesting = Depth::of(&nested);
    // The copy is this call's own, whatever the expression does to `"=`.
    eval_to_string(&expression, true, false)
}

/// A copy of the `"=` expression itself, without evaluating it.
pub fn get_expr_line_src() -> Option<XString> {
    expr_line.with(Clone::clone)
}

/// The contents of a computed register, copied.
///
/// `None` when `regname` is not one of them (or when a buffer-reading one is
/// asked for without `errmsg`, which is the caller saying it only wants an
/// answer it can get without side effects); `Some(None)` for one that has
/// nothing in it.
///
/// `errmsg` also turns on the error messages for a register that has nothing
/// in it -- E29 for `".`, E30 for `":`, E35 for `"/`. `"=` runs arbitrary
/// Vimscript.
pub fn get_spec_reg(regname: c_int, errmsg: bool) -> Option<Option<XString>> {
    let value = match regname {
        // `"%` -- the current file name.
        c if c == '%' as c_int => {
            if errmsg {
                let _ = check_fname(); // will give an error message
            }
            Buf::current().name.shown().map(XString::from_cstr)
        }
        // `"#` -- the alternate file name.
        c if c == '#' as c_int => match buflist_name_nr(0) {
            Some((alt, _)) => alt.name.shown().map(XString::from_cstr),
            None => {
                if errmsg {
                    emsg(gettext(e_noalt));
                }
                None
            }
        },
        // `"=` -- the expression, evaluated.
        c if c == '=' as c_int => get_expr_line(),
        // `":` -- the last command line.
        c if c == ':' as c_int => {
            // A copy: the caller may run commands while it holds it.
            let line = last_cmdline.with(Clone::clone);
            if line.is_none() && errmsg {
                emsg(gettext(e_nolastcmd));
            }
            line
        }
        // `"/` -- the last search pattern.
        c if c == '/' as c_int => {
            let pat = last_search_pat();
            if pat.is_none() && errmsg {
                emsg(gettext(e_noprevre));
            }
            pat
        }
        // `".` -- the last inserted text.
        c if c == '.' as c_int => {
            let text = get_last_insert_save();
            if text.is_none() && errmsg {
                emsg(gettext(e_noinstext));
            }
            text
        }
        // CTRL-R CTRL-F / CTRL-P -- the file name under the cursor, the
        // second form expanded to a full path.  Reading the buffer is a side
        // effect, so it only happens for a caller that asked for messages.
        Ctrl_F | Ctrl_P if errmsg => {
            let opts =
                FileNameOpts::MESS | FileNameOpts::HYP | FileNameOpts::EXP.when(regname == Ctrl_P);
            file_name_at_cursor(opts, 1, None)
        }
        // CTRL-R CTRL-W / CTRL-A -- the word, or the WORD, under the cursor.
        Ctrl_W | Ctrl_A if errmsg => {
            let find = if regname == Ctrl_W {
                FIND_IDENT | FIND_STRING
            } else {
                FIND_STRING
            };
            // A copy out of the buffer's own line.
            let ident = ident_under_cursor(find);
            (!ident.is_empty()).then(|| XString::from_vec(ident))
        }
        // CTRL-R CTRL-L -- the whole cursor line.
        Ctrl_L if errmsg => {
            // Copied rather than lent: every caller here goes on to insert
            // the text, and inserting re-enters the editor, which may move
            // or free the block the line was read out of.
            let win = Win::current();
            Some(XString::from_bytes(
                win.buffer().lines().line(win.w_cursor.lnum),
            ))
        }
        // `"_` -- the black hole, which reads as empty.
        c if c == '_' as c_int => Some(XString::new()),
        _ => return None,
    };
    Some(value)
}
