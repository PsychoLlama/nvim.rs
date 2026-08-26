//! The C++ shapes: `namespace`, `extern "C"` and a base-class list.
//!
//! Three 'cinoptions' letters live here.  `N` (`b_ind_cpp_namespace`) and `E`
//! (`b_ind_cpp_extern_c`) are the two block openers whose contents upstream
//! does not want indented, and both are recognised from the *opening* line.
//! `k` (`b_ind_cpp_baseclass`) is the harder one: [`cin_is_cpp_baseclass`]
//! decides whether a line is inside a constructor's initialiser list or a
//! class's base clause, which needs a scan back to the `class`/`:` that
//! started it -- so it caches its answer in the [`cpp_baseclass_cache_T`] its
//! caller owns.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int};

/// Whether `s` opens a `namespace` block -- 'cinoptions' `N`.
///
/// `inline` and `export` may precede it in any order, and the name may be a
/// C++17 nested one (`a::b::c`), but *two* names in a row is not a namespace
/// declaration: that is what `has_name`/`has_name_start` are tracking.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_is_cpp_namespace(s: *const c_char) -> bool {
    // SAFETY: the caller's promise -- `s` is NUL-terminated, and
    // `cin_skipcomment` answers no further than its NUL.
    let mut s = unsafe { cin_skipcomment(s) };
    let mut has_name = false;
    let mut has_name_start = false;

    // Skip over "inline" and "export" in any order.
    loop {
        // SAFETY: `s` points inside a NUL-terminated string.  `s.add(6)` runs
        // only once the six bytes of "inline"/"export" have been seen there,
        // so it is at worst that string's NUL, and neither skipper walks past
        // one.
        unsafe {
            let bytes = CStr::from_ptr(s).to_bytes();
            if !(bytes.starts_with(b"inline") || bytes.starts_with(b"export"))
                || vim_iswordc(c_int::from(byte_at(bytes, 6)))
            {
                break;
            }
            s = cin_skipcomment(skipwhite(s.add(6)));
        }
    }

    // SAFETY: as above -- `s.add(9)` runs only once "namespace" has been seen
    // at `s`.
    let mut p = unsafe {
        let bytes = CStr::from_ptr(s).to_bytes();
        if !bytes.starts_with(b"namespace") || vim_iswordc(c_int::from(byte_at(bytes, 9))) {
            return false;
        }
        cin_skipcomment(skipwhite(s.add(9)))
    };

    loop {
        // SAFETY: `p` points inside a NUL-terminated string.
        let c = unsafe { *p as u8 };
        if c == 0 {
            break;
        }
        if ascii_iswhite(c_int::from(c)) {
            has_name = true; // found the end of a name
            // SAFETY: neither skipper walks past the string's NUL.
            p = unsafe { cin_skipcomment(skipwhite(p)) };
        } else if c == b'{' {
            break;
        // SAFETY: `vim_iswordc` reads the current buffer's 'iskeyword' table.
        } else if unsafe { vim_iswordc(c_int::from(c)) } {
            if has_name {
                return false; // a word character after a finished name
            }
            has_name_start = true;
            // SAFETY: `c` is not the NUL, so `p.add(1)` is at worst it.
            p = unsafe { p.add(1) };
        } else if c == b':'
            // SAFETY: `c` is not the NUL, so `p.add(1)` is inside the string;
            // the second `:` in front of `p.add(2)` is what says that one is
            // too, and the chain is left whole so that it keeps doing so.
            && unsafe { *p.add(1) as u8 == b':' && vim_iswordc(c_int::from(*p.add(2) as u8)) }
        {
            if !has_name_start || has_name {
                return false;
            }
            // SAFETY: the three bytes `::x` were just read at `p`.
            p = unsafe { p.add(3) }; // C++17 nested namespace
        } else {
            return false;
        }
    }
    true
}

/// Whether `s` opens an `extern "C"` or `extern "C++"` linkage block --
/// 'cinoptions' `E`.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_is_cpp_extern_c(s: *const c_char) -> bool {
    // SAFETY: the caller's promise -- `s` is NUL-terminated, and
    // `cin_skipcomment` answers no further than its NUL.
    let s = unsafe { cin_skipcomment(s) };
    // SAFETY: `s.add(6)` runs only once "extern" has been seen at `s`, so it
    // is at worst that string's NUL, and neither skipper walks past one.
    let mut p = unsafe {
        let bytes = CStr::from_ptr(s).to_bytes();
        if !bytes.starts_with(b"extern") || vim_iswordc(c_int::from(byte_at(bytes, 6))) {
            return false;
        }
        cin_skipcomment(skipwhite(s.add(6)))
    };

    let mut has_string_literal = false;
    loop {
        // SAFETY: `p` points inside a NUL-terminated string.
        let (c, tail) = unsafe { (*p as u8, CStr::from_ptr(p).to_bytes()) };
        if c == 0 {
            break;
        }
        if ascii_iswhite(c_int::from(c)) {
            // SAFETY: neither skipper walks past the string's NUL.
            p = unsafe { cin_skipcomment(skipwhite(p)) };
        } else if c == b'{' {
            break;
        } else if let Some(lang) = [&b"\"C\""[..], &b"\"C++\""[..]]
            .into_iter()
            .find(|lang| tail.starts_with(lang))
        {
            if has_string_literal {
                return false; // only one linkage string
            }
            has_string_literal = true;
            // SAFETY: `tail` starts with `lang`, so `p.add(lang.len())` is at
            // worst the string's NUL.
            p = unsafe { p.add(lang.len()) };
        } else {
            return false;
        }
    }
    has_string_literal
}

/// Whether the cursor's line is inside a C++ base-class list or a
/// constructor's initialiser list -- 'cinoptions' `k`.
///
/// ```text
/// class MyClass :
///      baseClass               <-- here
/// class MyClass : public baseClass,
///      anotherBaseClass        <-- here
/// MyClass::MyClass(...) :
///      baseClass(...)          <-- here (constructor-initialization)
/// ```
///
/// This is a lot of guessing -- `cond ? func() : foo` is the shape it must
/// not mistake for one -- so it first walks back to a line it can trust as a
/// statement boundary (`#`, empty, ending in `;`, or holding a brace) and
/// then scans *forward* from there to the cursor, tracking whether it has
/// seen a `class`/`struct` or a `)` that a following `:` would belong to.
///
/// `cached` holds the answer and the column to line up with; the scan is
/// re-run only once the cursor has moved above the line it was computed for,
/// which is exactly how the backwards scan in `engine` walks.
///
/// # Safety
/// Reads the cursor and the buffer; may unlock the current line.
pub(crate) unsafe fn cin_is_cpp_baseclass(cached: &mut cpp_baseclass_cache_T) -> bool {
    let mut lnum = cur_win().w_cursor.lnum;
    // SAFETY: the cursor is on a line of the current buffer.
    let mut line = get_cursor_line_ptr().cast_const();

    if cached.lpos.lnum <= lnum {
        return cached.found != 0; // use the cached result
    }
    cached.lpos.col = 0;

    // SAFETY: `line` is a NUL-terminated line, so `skipwhite` stops inside it
    // and `cin_skipcomment` answers no further than its NUL.
    let mut s = unsafe { skipwhite(line) }.cast_const();
    // SAFETY: `s` points inside that line.
    if unsafe { *s } as u8 == b'#' {
        return false; // skip #define FOO x ? (x) : x
    }
    // SAFETY: as above.
    s = unsafe { cin_skipcomment(s) };
    // SAFETY: as above.
    if unsafe { *s } == 0 {
        return false;
    }

    let mut cpp_base_class = false;
    let mut lookfor_ctor_init = false;
    let mut class_or_struct = false;

    // Walk back to a line starting with '#', empty, ending in ';' or
    // holding a '{' or '}', and start below it.  That handles:
    //    a = cond ?
    //          func() :
    //               asdf;
    //    Foo::Foo (int one, int two)
    //            : something(4),
    //            somethingelse(3)
    //    {}
    while lnum > 1 {
        // SAFETY: `lnum - 1` is at least 1, so it is a line of the current
        // buffer and `ml_get` hands back a NUL-terminated one; `skipwhite`
        // stops inside it and neither `cin_skipcomment` nor `cin_nocode`
        // walks past its NUL.  Every step is a pointer operation, so one
        // region around the whole walk is as tight as this gets.
        let stop = unsafe {
            line = ml_get(lnum - 1);
            s = skipwhite(line);
            if *s as u8 == b'#' || *s == 0 {
                true
            } else {
                while *s != 0 {
                    s = cin_skipcomment(s);
                    if *s as u8 == b'{'
                        || *s as u8 == b'}'
                        || (*s as u8 == b';' && cin_nocode(s.add(1)))
                    {
                        break;
                    }
                    if *s != 0 {
                        s = s.add(1);
                    }
                }
                *s != 0
            }
        };
        if stop {
            break;
        }
        lnum -= 1;
    }

    cached.lpos.lnum = lnum;
    // SAFETY: `lnum` is a line of the current buffer -- the walk above only
    // ever moved it down towards 1.
    line = ml_get(lnum);
    s = line;
    loop {
        // SAFETY: `s` points inside a NUL-terminated line.
        if unsafe { *s } == 0 {
            if lnum == cur_win().w_cursor.lnum {
                break;
            }
            lnum += 1; // continue into the cursor's line
            // SAFETY: the walk above stopped at or above the cursor's line
            // and this one stops there, so `lnum` is a line of the buffer.
            line = ml_get(lnum);
            s = line;
        }
        if s == line {
            // Do not recognise "case (foo):" as a base class.
            // SAFETY: `s` is a NUL-terminated line.
            if unsafe { cin_iscase(s, false) } {
                break;
            }
            // SAFETY: as above; `cin_skipcomment` stops at that line's NUL.
            s = unsafe { cin_skipcomment(line) };
            // SAFETY: `s` points inside the line.
            if unsafe { *s } == 0 {
                continue;
            }
        }

        // Past the test above, `s` is never on the line's NUL, so `s.add(1)`
        // is at worst that NUL -- which is what every step below rests on.
        // SAFETY: `s` points inside a NUL-terminated line.
        let c = unsafe { *s as u8 };
        // SAFETY: `c` is not the NUL, so `s.add(1)` is inside the line; the
        // chain is left whole so that the `R` keeps guarding the read.
        if unsafe { c == b'"' || (c == b'R' && *s.add(1) as u8 == b'"') } {
            // SAFETY: `s` starts a string constant of a NUL-terminated line,
            // so `skip_string` answers inside it and `add(1)` is at worst its
            // NUL.
            s = unsafe { skip_string(s).add(1) };
        } else if c == b':' {
            // SAFETY: `s.add(1)` is at worst the line's NUL.
            if unsafe { *s.add(1) as u8 == b':' } {
                // A double colon: no longer a constructor initialisation.
                lookfor_ctor_init = false;
                // SAFETY: two colons were just read at `s`, so `s.add(2)` is
                // at worst the line's NUL.
                s = unsafe { cin_skipcomment(s.add(2)) };
            } else {
                if lookfor_ctor_init || class_or_struct {
                    // The start of a base-class declaration or of a
                    // constructor initialisation.
                    cpp_base_class = true;
                    lookfor_ctor_init = false;
                    class_or_struct = false;
                    cached.lpos.col = 0;
                }
                // SAFETY: `s.add(1)` is at worst the line's NUL.
                s = unsafe { cin_skipcomment(s.add(1)) };
            }
        } else if let Some(word) = [&b"class"[..], &b"struct"[..]].into_iter().find(|word| {
            // SAFETY: `s` points inside a NUL-terminated line.
            let bytes = unsafe { CStr::from_ptr(s) }.to_bytes();
            // SAFETY: `vim_is_ident_char` reads the 'isident' table, which is
            // set up long before any buffer is indented.
            bytes.starts_with(word)
                && !unsafe { vim_is_ident_char(byte_at(bytes, word.len()).into()) }
        }) {
            class_or_struct = true;
            lookfor_ctor_init = false;
            // SAFETY: `word` was just matched at `s`, so `s.add(word.len())`
            // is at worst the line's NUL.
            s = unsafe { cin_skipcomment(s.add(word.len())) };
        } else {
            if c == b'{' || c == b'}' || c == b';' {
                cpp_base_class = false;
                lookfor_ctor_init = false;
                class_or_struct = false;
            } else if c == b')' {
                // "):" is assumed to be a constructor initialisation.
                class_or_struct = false;
                lookfor_ctor_init = true;
            } else if c == b'?' {
                // Do not see the '() :' after a '?' as a constructor init.
                return false;
            // SAFETY: `vim_is_ident_char` reads the 'isident' table.
            } else if !unsafe { vim_is_ident_char(c_int::from(c)) } {
                // Not an identifier: we are wrong.
                class_or_struct = false;
                lookfor_ctor_init = false;
            } else if cached.lpos.col == 0 {
                lookfor_ctor_init = false;
                // The first statement starts here; line up with it.
                if cpp_base_class {
                    // SAFETY: `s` and `line` point into the same line.
                    cached.lpos.col = unsafe { s.offset_from(line) } as colnr_T;
                }
            }

            // When the line ends in a comma, do not align with it.
            if lnum == cur_win().w_cursor.lnum
                // SAFETY: `s.add(1)` is at worst the line's NUL, which is all
                // `cin_nocode` asks for.
                && unsafe { c == b',' && cin_nocode(s.add(1)) }
            {
                cached.lpos.col = 0;
            }
            // SAFETY: `s.add(1)` is at worst the line's NUL.
            s = unsafe { cin_skipcomment(s.add(1)) };
        }
    }

    cached.found = c_int::from(cpp_base_class);
    if cpp_base_class {
        cached.lpos.lnum = lnum;
    }
    cpp_base_class
}

/// The indent for a line inside a base-class or initialiser list, given the
/// column [`cin_is_cpp_baseclass`] chose (0 meaning "nothing to line up
/// with").
///
/// # Safety
/// Reads and writes the cursor; may unlock the current line.
pub(crate) unsafe fn get_baseclass_amount(col: c_int) -> c_int {
    let mut amount = if col == 0 {
        // SAFETY: reads the cursor's line of the current buffer.
        let mut amount = get_indent();
        // SAFETY: `get_cursor_line_ptr` hands back the cursor's
        // NUL-terminated line; both searches work on the current buffer, and
        // `find_match_paren` runs only when `find_last_paren` found one, as
        // upstream has it.
        let opening = unsafe {
            find_last_paren(get_cursor_line_ptr(), b'(', b')')
                .then(|| find_match_paren(cur_buf().b_ind_maxparen))
                .flatten()
        };
        if let Some(trypos) = opening {
            // SAFETY: `trypos` is a position in the current buffer.
            amount = unsafe { get_indent_lnum(trypos.lnum) };
        }
        // SAFETY: the cursor's line is NUL-terminated.
        if !unsafe { cin_ends_in(get_cursor_line_ptr(), b",") } {
            amount += cur_buf().b_ind_cpp_baseclass;
        }
        amount
    } else {
        let mut win = cur_win();
        win.w_cursor.col = col;
        win.vcol(win.cursor())
    };
    if amount < cur_buf().b_ind_cpp_baseclass {
        amount = cur_buf().b_ind_cpp_baseclass;
    }
    amount
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
