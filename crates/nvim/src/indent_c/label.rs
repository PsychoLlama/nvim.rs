//! Indent amounts read off a line rather than computed.
//!
//! [`skip_label`] and [`get_indent_nolabel`] answer "how indented is this
//! line, not counting a jump label in front of it"; [`after_label`] is the
//! text past one.  [`cin_first_id_amount`] is 'cinoptions' `+`'s
//! continuation base -- the column of the first identifier after a type --
//! and [`cin_get_equal_amount`] the column after a trailing `=`, which is
//! what a `\`-continued assignment lines up with.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::Win;
use core::ffi::{CStr, c_char, c_int};

/// The first non-white non-comment character after a `:` label in `l`, or
/// null.
///
/// ```text
///    case 234:    a = b;
///                 ^
/// ```
///
/// `::` is C++ scope resolution, and a `:` that opens *another* `case` is not
/// the end of this label.
///
/// # Safety
/// `l` must point at a NUL-terminated string.
pub(crate) unsafe fn after_label(l: *const c_char) -> *const c_char {
    // SAFETY: the caller's promise.  Every step stays inside the string:
    // the `l.add(1)`/`l.add(2)` skips run only after the byte before them
    // has been seen to be non-NUL, which the `&&` chains keep in front.
    let mut l = l;
    while unsafe { *l } != 0 {
        if unsafe { *l } as u8 == b':' {
            if unsafe { *l.add(1) } as u8 == b':' {
                l = unsafe { l.add(1) }; // skip over "::" for C++
            } else if !unsafe { cin_iscase(l.add(1), false) } {
                break;
            }
        } else if unsafe { *l } as u8 == b'\''
            && unsafe { *l.add(1) } != 0
            && unsafe { *l.add(2) } as u8 == b'\''
        {
            l = unsafe { l.add(2) }; // skip over 'x'
        }
        l = unsafe { l.add(1) };
    }
    if unsafe { *l } == 0 {
        return ::core::ptr::null::<c_char>();
    }
    let l = unsafe { cin_skipcomment(l.add(1)) };
    if unsafe { *l } == 0 {
        ::core::ptr::null::<c_char>()
    } else {
        l
    }
}

/// The screen column the code *after* a label on line `lnum` starts at, or 0
/// when there is nothing after it.
///
/// # Safety
/// `lnum` must be a valid line; may unlock the current line.
pub(crate) unsafe fn get_indent_nolabel(lnum: linenr_T) -> c_int {
    // SAFETY: on the main thread with a current buffer; `after_label` is
    // handed the NUL-terminated line `ml_get` answered with, and gives back
    // either null or a pointer inside it.
    let (l, p) = unsafe {
        let l = ml_get(lnum);
        (l, after_label(l))
    };
    if p.is_null() {
        return 0;
    }
    // SAFETY: `p` points inside line `lnum`.
    unsafe { line_vcol(lnum, p.offset_from(l) as colnr_T) }
}

/// The indent of line `lnum` ignoring any case or jump label, with `pp` left
/// pointing at the text the amount belongs to.
///
/// ```text
///   label:     if (asdf && asdfasdf)
///              ^
/// ```
///
/// # Safety
/// Moves the cursor and restores it; may unlock the current line.
pub(crate) unsafe fn skip_label(lnum: linenr_T, pp: &mut *const c_char) -> c_int {
    let cursor_save = cur_win().w_cursor;
    cur_win().w_cursor.lnum = lnum;
    // SAFETY: the cursor now sits on line `lnum` of the current buffer, so
    // `get_cursor_line_ptr` answers with that NUL-terminated line.
    let (amount, mut text) = unsafe {
        let l = get_cursor_line_ptr().cast_const();
        if cin_iscase(l, false) || cin_isscopedecl(l) || cin_islabel() {
            (get_indent_nolabel(lnum), after_label(get_cursor_line_ptr()))
        } else {
            (get_indent(), get_cursor_line_ptr().cast_const())
        }
    };
    if text.is_null() {
        // SAFETY: the cursor is still on line `lnum`.
        text = get_cursor_line_ptr(); // just in case
    }
    *pp = text;

    cur_win().w_cursor = cursor_save;
    amount
}

/// The screen column of the first variable name after a type in a
/// declaration -- 'cinoptions' `+`'s base for a continued declaration.
///
/// ```text
///  int     a,                  the column of "a"
///  static struct foo    b,     the column of "b"
///  enum bla    c,              the column of "c"
/// ```
///
/// Zero when the line does not look like a declaration.
///
/// # Safety
/// Reads the cursor; may unlock the current line.
pub(crate) unsafe fn cin_first_id_amount() -> c_int {
    // SAFETY: the cursor is on a line of the current buffer, and every walk
    // below starts from the NUL-terminated line it names.  `p.add(len)` is
    // inside it because `len` counts bytes the walk has already read.
    let line = get_cursor_line_ptr().cast_const();
    let mut p = unsafe { skipwhite(line) }.cast_const();

    // Step over the storage class and the type's first word, so that the
    // identifier the answer is about is what `p` ends on.
    let mut len = unsafe { skiptowhite(p).offset_from(p) } as usize;
    if len == 6
        && unsafe { CStr::from_ptr(p) }
            .to_bytes()
            .starts_with(b"static")
    {
        p = unsafe { skipwhite(p.add(6)) };
        len = unsafe { skiptowhite(p).offset_from(p) } as usize;
    }
    let word = unsafe { CStr::from_ptr(p) }.to_bytes();
    if len == 6 && word.starts_with(b"struct") {
        p = unsafe { skipwhite(p.add(6)) };
    } else if len == 4 && word.starts_with(b"enum") {
        p = unsafe { skipwhite(p.add(4)) };
    } else if (len == 8 && word.starts_with(b"unsigned"))
        || (len == 6 && word.starts_with(b"signed"))
    {
        // `unsigned`/`signed` only prefixes a type; take the type with it.
        let s = unsafe { skipwhite(p.add(len)) }.cast_const();
        let rest = unsafe { CStr::from_ptr(s) }.to_bytes();
        let takes_type = [&b"int"[..], b"long", b"short", b"char"]
            .into_iter()
            .any(|kw| rest.starts_with(kw) && ascii_iswhite(c_int::from(byte_at(rest, kw.len()))));
        if takes_type {
            p = s.cast_mut();
        }
    }

    let mut len = 0usize;
    while unsafe { vim_is_ident_char(c_int::from(*p.add(len) as u8)) } {
        len += 1;
    }
    if len == 0
        || !ascii_iswhite(c_int::from(unsafe { *p.add(len) } as u8))
        || unsafe { cin_nocode(p) }
    {
        return 0;
    }

    let p = unsafe { skipwhite(p.add(len)) }.cast_const();
    unsafe { line_vcol(cur_win().w_cursor.lnum, p.offset_from(line) as colnr_T) }
}

/// The screen column of the first non-blank after an `=` on line `lnum`.
///
/// ```text
///       char *foo = "here";
///                    ^
/// ```
///
/// Zero when there is no useful `=`, and **-1** when the line *above* `lnum`
/// ends in a backslash -- the assignment started further up, so this line's
/// `=` is not the one to line up with.
///
/// # Safety
/// `lnum` must be a valid line; may unlock the current line.
pub(crate) unsafe fn cin_get_equal_amount(lnum: linenr_T) -> c_int {
    if lnum > 1 {
        // SAFETY: on the main thread with a current buffer; `ml_get` hands
        // back a NUL-terminated line.
        if unsafe { cin_ends_in_backslash(ml_get(lnum - 1)) } {
            return -1;
        }
    }

    // SAFETY: the same, and every walk below stays inside that line: the
    // `s.add(1)` steps run only past a byte already seen to be non-NUL.
    let line = ml_get(lnum).cast_const();
    let mut s = line;
    while unsafe { *s } != 0
        && unsafe { vim_strchr(c"=;{}\"'".as_ptr(), c_int::from(*s as u8)) }.is_null()
    {
        if unsafe { cin_iscomment(s) } {
            s = unsafe { cin_skipcomment(s) };
        } else {
            s = unsafe { s.add(1) };
        }
    }
    if unsafe { *s } as u8 != b'=' {
        return 0;
    }

    let mut s = unsafe { skipwhite(s.add(1)) }.cast_const();
    if unsafe { cin_nocode(s) } {
        return 0;
    }
    if unsafe { *s } as u8 == b'"' {
        s = unsafe { s.add(1) }; // nice alignment for continued strings
    }
    unsafe { line_vcol(lnum, s.offset_from(line) as colnr_T) }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
