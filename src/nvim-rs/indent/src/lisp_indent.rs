//! Implementation of `get_lisp_indent()` — Lisp-style indentation.

use std::ffi::{c_char, c_int};

use crate::getters::rs_get_indent;

type LinenrT = i32;
type ColnrT = i32;

const NUL: c_char = 0;

/// Position in buffer (line, column, coladd).
#[repr(C)]
#[derive(Clone, Copy)]
struct PosT {
    lnum: LinenrT,
    col: ColnrT,
    coladd: ColnrT,
}

// C accessor functions
extern "C" {
    // Cursor
    fn nvim_get_curwin_cursor_lnum() -> LinenrT;
    fn nvim_set_curwin_cursor_lnum(lnum: LinenrT);
    fn nvim_set_curwin_cursor_col(col: ColnrT);

    // Save/restore full cursor position
    fn nvim_change_get_curwin_cursor() -> PosT;
    fn nvim_set_curwin_cursor(pos: PosT);

    // Line operations
    fn nvim_get_cursor_line_ptr() -> *mut c_char;
    fn nvim_linewhite(lnum: LinenrT) -> bool;

    // Matching
    fn nvim_findmatch(initc: *mut c_char, ch: c_char) -> *mut PosT;

    // Lisp-specific charsize helpers (indent_ffi.c)
    fn nvim_lisp_vcol_at_col(
        lnum: LinenrT,
        line: *mut c_char,
        col: ColnrT,
        out_ptr: *mut *mut c_char,
    ) -> c_int;
    fn nvim_lisp_skip_whitespace(
        lnum: LinenrT,
        line: *mut c_char,
        amount: c_int,
        ptr: *mut c_char,
        out_ptr: *mut *mut c_char,
    ) -> c_int;
    fn nvim_lisp_skip_word(
        lnum: LinenrT,
        line: *mut c_char,
        amount: c_int,
        ptr: *mut c_char,
        out_ptr: *mut *mut c_char,
    ) -> c_int;
    fn nvim_utf_ptr2char_value(ptr: *const c_char) -> c_int;

    // Lisp match (already in checks.rs)
    fn rs_lisp_match(p: *const c_char) -> c_int;
}

/// Determine lisp indentation for the current line.
///
/// # Safety
/// - Accesses global editor state (current window, buffer, cursor).
/// - Single-threaded (Neovim guarantee).
#[no_mangle]
pub unsafe extern "C" fn rs_get_lisp_indent() -> c_int {
    let realpos = nvim_change_get_curwin_cursor();
    nvim_set_curwin_cursor_col(0);

    // Find matching '(' or '['
    let mut pos = nvim_findmatch(std::ptr::null_mut(), b'(' as c_char);
    let mut paren;

    if pos.is_null() {
        pos = nvim_findmatch(std::ptr::null_mut(), b'[' as c_char);
    } else {
        paren = *pos;
        pos = nvim_findmatch(std::ptr::null_mut(), b'[' as c_char);

        if pos.is_null() || lt(*pos, paren) {
            pos = &mut paren;
        }
    }

    let amount = if !pos.is_null() {
        compute_indent_from_match(&*pos, realpos)
    } else {
        0 // No matching '(' or '[' found, use zero indent.
    };

    nvim_set_curwin_cursor(realpos);
    amount
}

/// Returns true if `a` is before `b` in the buffer.
#[inline]
fn lt(a: PosT, b: PosT) -> bool {
    a.lnum < b.lnum || (a.lnum == b.lnum && a.col < b.col)
}

/// Compute the indent amount based on the matched paren position.
unsafe fn compute_indent_from_match(pos: &PosT, _realpos: PosT) -> c_int {
    // Extra trick: Take the indent of the first previous non-white
    // line that is at the same () level.
    let mut amount: c_int = -1;
    let mut parencount: c_int = 0;

    let mut lnum = nvim_get_curwin_cursor_lnum();
    lnum -= 1;
    while lnum >= pos.lnum {
        nvim_set_curwin_cursor_lnum(lnum);
        if nvim_linewhite(lnum) {
            lnum -= 1;
            continue;
        }

        let line_ptr = nvim_get_cursor_line_ptr();
        let mut that = line_ptr;
        while *that != NUL {
            if *that == b';' as c_char {
                // Skip to end of line (comment)
                while *that.add(1) != NUL {
                    that = that.add(1);
                }
                that = that.add(1);
                continue;
            }

            if *that == b'\\' as c_char {
                if *that.add(1) != NUL {
                    that = that.add(1);
                }
                that = that.add(1);
                continue;
            }

            if *that == b'"' as c_char && *that.add(1) != NUL {
                that = that.add(1);
                while *that != NUL && *that != b'"' as c_char {
                    // Skipping escaped characters in the string
                    if *that == b'\\' as c_char {
                        that = that.add(1);
                        if *that == NUL {
                            break;
                        }
                        if *that.add(1) == NUL {
                            that = that.add(1);
                            break;
                        }
                    }
                    that = that.add(1);
                }
                if *that == NUL {
                    break;
                }
            }
            if *that == b'(' as c_char || *that == b'[' as c_char {
                parencount += 1;
            } else if *that == b')' as c_char || *that == b']' as c_char {
                parencount -= 1;
            }
            that = that.add(1);
        }

        if parencount == 0 {
            amount = rs_get_indent();
            break;
        }
        lnum -= 1;
    }

    if amount == -1 {
        amount = compute_indent_at_paren(pos);
    }

    amount
}

/// Compute indent based on the character at the matched paren position.
unsafe fn compute_indent_at_paren(pos: &PosT) -> c_int {
    nvim_set_curwin_cursor_lnum(pos.lnum);
    nvim_set_curwin_cursor_col(pos.col);

    let line = nvim_get_cursor_line_ptr();

    let mut that: *mut c_char = std::ptr::null_mut();
    let mut amount = nvim_lisp_vcol_at_col(pos.lnum, line, pos.col, &mut that);

    // Some keywords require "body" indenting rules
    if (*that == b'(' as c_char || *that == b'[' as c_char) && rs_lisp_match(that.add(1)) != 0 {
        amount += 2;
    } else {
        if *that != NUL {
            that = that.add(1);
            amount += 1;
        }
        let firsttry = amount;

        amount = nvim_lisp_skip_whitespace(pos.lnum, line, amount, that, &mut that);

        if *that != NUL && *that != b';' as c_char {
            // Not a comment line.
            let mut adjusted_firsttry = firsttry;
            if *that != b'(' as c_char && *that != b'[' as c_char {
                adjusted_firsttry += 1;
            }

            let ci_value = nvim_utf_ptr2char_value(that);
            if ci_value != i32::from(b'"')
                && ci_value != i32::from(b'\'')
                && ci_value != i32::from(b'#')
                && (ci_value < i32::from(b'0') || ci_value > i32::from(b'9'))
            {
                amount = nvim_lisp_skip_word(pos.lnum, line, amount, that, &mut that);
            }

            amount = nvim_lisp_skip_whitespace(pos.lnum, line, amount, that, &mut that);

            if *that == NUL || *that == b';' as c_char {
                amount = adjusted_firsttry;
            }
        }
    }

    amount
}
