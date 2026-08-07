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
use ::core::ffi::{CStr, c_char, c_int};

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
    unsafe {
        let mut l = l;
        while *l != 0 {
            if *l as u8 == b':' {
                if *l.add(1) as u8 == b':' {
                    l = l.add(1); // skip over "::" for C++
                } else if !cin_iscase(l.add(1), false) {
                    break;
                }
            } else if *l as u8 == b'\'' && *l.add(1) != 0 && *l.add(2) as u8 == b'\'' {
                l = l.add(2); // skip over 'x'
            }
            l = l.add(1);
        }
        if *l == 0 {
            return ::core::ptr::null::<c_char>();
        }
        let l = cin_skipcomment(l.add(1));
        if *l == 0 {
            ::core::ptr::null::<c_char>()
        } else {
            l
        }
    }
}

/// The screen column the code *after* a label on line `lnum` starts at, or 0
/// when there is nothing after it.
///
/// # Safety
/// `lnum` must be a valid line; may unlock the current line.
pub(crate) unsafe fn get_indent_nolabel(lnum: linenr_T) -> c_int {
    unsafe {
        let l = ml_get(lnum);
        let p = after_label(l);
        if p.is_null() {
            return 0;
        }
        line_vcol(lnum, p.offset_from(l) as colnr_T)
    }
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
    unsafe {
        let cursor_save = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor.lnum = lnum;
        let l = get_cursor_line_ptr().cast_const();

        let (amount, mut text) = if cin_iscase(l, false) || cin_isscopedecl(l) || cin_islabel() {
            (get_indent_nolabel(lnum), after_label(get_cursor_line_ptr()))
        } else {
            (get_indent(), get_cursor_line_ptr().cast_const())
        };
        if text.is_null() {
            text = get_cursor_line_ptr(); // just in case
        }
        *pp = text;

        (*curwin.get()).w_cursor = cursor_save;
        amount
    }
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
    unsafe {
        let line = get_cursor_line_ptr().cast_const();
        let mut p = skipwhite(line).cast_const();

        // Step over the storage class and the type's first word, so that the
        // identifier the answer is about is what `p` ends on.
        let mut len = skiptowhite(p).offset_from(p) as usize;
        if len == 6 && CStr::from_ptr(p).to_bytes().starts_with(b"static") {
            p = skipwhite(p.add(6));
            len = skiptowhite(p).offset_from(p) as usize;
        }
        let word = CStr::from_ptr(p).to_bytes();
        if len == 6 && word.starts_with(b"struct") {
            p = skipwhite(p.add(6));
        } else if len == 4 && word.starts_with(b"enum") {
            p = skipwhite(p.add(4));
        } else if (len == 8 && word.starts_with(b"unsigned"))
            || (len == 6 && word.starts_with(b"signed"))
        {
            // `unsigned`/`signed` only prefixes a type; take the type with it.
            let s = skipwhite(p.add(len)).cast_const();
            let rest = CStr::from_ptr(s).to_bytes();
            let takes_type = [&b"int"[..], b"long", b"short", b"char"]
                .into_iter()
                .any(|kw| {
                    rest.starts_with(kw) && ascii_iswhite(c_int::from(byte_at(rest, kw.len())))
                });
            if takes_type {
                p = s.cast_mut();
            }
        }

        let mut len = 0usize;
        while vim_isIDc(c_int::from(*p.add(len) as u8)) {
            len += 1;
        }
        if len == 0 || !ascii_iswhite(c_int::from(*p.add(len) as u8)) || cin_nocode(p) {
            return 0;
        }

        let p = skipwhite(p.add(len)).cast_const();
        line_vcol(
            (*curwin.get()).w_cursor.lnum,
            p.offset_from(line) as colnr_T,
        )
    }
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
    unsafe {
        if lnum > 1 {
            let above = ml_get(lnum - 1);
            if *above != 0 && *above.add(strlen(above) - 1) as u8 == b'\\' {
                return -1;
            }
        }

        let line = ml_get(lnum).cast_const();
        let mut s = line;
        while *s != 0 && vim_strchr(c"=;{}\"'".as_ptr(), c_int::from(*s as u8)).is_null() {
            if cin_iscomment(s) {
                s = cin_skipcomment(s);
            } else {
                s = s.add(1);
            }
        }
        if *s as u8 != b'=' {
            return 0;
        }

        let mut s = skipwhite(s.add(1)).cast_const();
        if cin_nocode(s) {
            return 0;
        }
        if *s as u8 == b'"' {
            s = s.add(1); // nice alignment for continued strings
        }
        line_vcol(lnum, s.offset_from(line) as colnr_T)
    }
}
