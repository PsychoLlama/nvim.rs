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
use ::core::ffi::{CStr, c_char, c_int};

/// Whether `s` opens a `namespace` block -- 'cinoptions' `N`.
///
/// `inline` and `export` may precede it in any order, and the name may be a
/// C++17 nested one (`a::b::c`), but *two* names in a row is not a namespace
/// declaration: that is what `has_name`/`has_name_start` are tracking.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_is_cpp_namespace(s: *const c_char) -> bool {
    unsafe {
        let mut s = cin_skipcomment(s);
        let mut has_name = false;
        let mut has_name_start = false;

        // Skip over "inline" and "export" in any order.
        loop {
            let bytes = CStr::from_ptr(s).to_bytes();
            if !(bytes.starts_with(b"inline") || bytes.starts_with(b"export"))
                || vim_iswordc(c_int::from(byte_at(bytes, 6)))
            {
                break;
            }
            s = cin_skipcomment(skipwhite(s.add(6)));
        }

        let bytes = CStr::from_ptr(s).to_bytes();
        if !bytes.starts_with(b"namespace") || vim_iswordc(c_int::from(byte_at(bytes, 9))) {
            return false;
        }

        let mut p = cin_skipcomment(skipwhite(s.add(9)));
        while *p != 0 {
            if ascii_iswhite(c_int::from(*p as u8)) {
                has_name = true; // found the end of a name
                p = cin_skipcomment(skipwhite(p));
            } else if *p as u8 == b'{' {
                break;
            } else if vim_iswordc(c_int::from(*p as u8)) {
                if has_name {
                    return false; // a word character after a finished name
                }
                has_name_start = true;
                p = p.add(1);
            } else if *p as u8 == b':'
                && *p.add(1) as u8 == b':'
                && vim_iswordc(c_int::from(*p.add(2) as u8))
            {
                if !has_name_start || has_name {
                    return false;
                }
                p = p.add(3); // C++17 nested namespace
            } else {
                return false;
            }
        }
        true
    }
}

/// Whether `s` opens an `extern "C"` or `extern "C++"` linkage block --
/// 'cinoptions' `E`.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub(crate) unsafe fn cin_is_cpp_extern_c(s: *const c_char) -> bool {
    unsafe {
        let s = cin_skipcomment(s);
        let bytes = CStr::from_ptr(s).to_bytes();
        if !bytes.starts_with(b"extern") || vim_iswordc(c_int::from(byte_at(bytes, 6))) {
            return false;
        }

        let mut has_string_literal = false;
        let mut p = cin_skipcomment(skipwhite(s.add(6)));
        while *p != 0 {
            let tail = CStr::from_ptr(p).to_bytes();
            if ascii_iswhite(c_int::from(*p as u8)) {
                p = cin_skipcomment(skipwhite(p));
            } else if *p as u8 == b'{' {
                break;
            } else if let Some(lang) = [&b"\"C\""[..], &b"\"C++\""[..]]
                .into_iter()
                .find(|lang| tail.starts_with(lang))
            {
                if has_string_literal {
                    return false; // only one linkage string
                }
                has_string_literal = true;
                p = p.add(lang.len());
            } else {
                return false;
            }
        }
        has_string_literal
    }
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
    unsafe {
        let mut lnum = (*curwin.get()).w_cursor.lnum;
        let mut line = get_cursor_line_ptr().cast_const();

        if cached.lpos.lnum <= lnum {
            return cached.found != 0; // use the cached result
        }
        cached.lpos.col = 0;

        let mut s = skipwhite(line).cast_const();
        if *s as u8 == b'#' {
            return false; // skip #define FOO x ? (x) : x
        }
        s = cin_skipcomment(s);
        if *s == 0 {
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
            line = ml_get(lnum - 1);
            s = skipwhite(line);
            if *s as u8 == b'#' || *s == 0 {
                break;
            }
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
            if *s != 0 {
                break;
            }
            lnum -= 1;
        }

        cached.lpos.lnum = lnum;
        line = ml_get(lnum);
        s = line;
        loop {
            if *s == 0 {
                if lnum == (*curwin.get()).w_cursor.lnum {
                    break;
                }
                lnum += 1; // continue into the cursor's line
                line = ml_get(lnum);
                s = line;
            }
            if s == line {
                // Do not recognise "case (foo):" as a base class.
                if cin_iscase(s, false) {
                    break;
                }
                s = cin_skipcomment(line);
                if *s == 0 {
                    continue;
                }
            }

            if *s as u8 == b'"' || (*s as u8 == b'R' && *s.add(1) as u8 == b'"') {
                s = skip_string(s).add(1);
            } else if *s as u8 == b':' {
                if *s.add(1) as u8 == b':' {
                    // A double colon: no longer a constructor initialisation.
                    lookfor_ctor_init = false;
                    s = cin_skipcomment(s.add(2));
                } else {
                    if lookfor_ctor_init || class_or_struct {
                        // The start of a base-class declaration or of a
                        // constructor initialisation.
                        cpp_base_class = true;
                        lookfor_ctor_init = false;
                        class_or_struct = false;
                        cached.lpos.col = 0;
                    }
                    s = cin_skipcomment(s.add(1));
                }
            } else if let Some(word) = [&b"class"[..], &b"struct"[..]].into_iter().find(|word| {
                let bytes = CStr::from_ptr(s).to_bytes();
                bytes.starts_with(word) && !vim_isIDc(c_int::from(byte_at(bytes, word.len())))
            }) {
                class_or_struct = true;
                lookfor_ctor_init = false;
                s = cin_skipcomment(s.add(word.len()));
            } else {
                if *s as u8 == b'{' || *s as u8 == b'}' || *s as u8 == b';' {
                    cpp_base_class = false;
                    lookfor_ctor_init = false;
                    class_or_struct = false;
                } else if *s as u8 == b')' {
                    // "):" is assumed to be a constructor initialisation.
                    class_or_struct = false;
                    lookfor_ctor_init = true;
                } else if *s as u8 == b'?' {
                    // Do not see the '() :' after a '?' as a constructor init.
                    return false;
                } else if !vim_isIDc(c_int::from(*s as u8)) {
                    // Not an identifier: we are wrong.
                    class_or_struct = false;
                    lookfor_ctor_init = false;
                } else if cached.lpos.col == 0 {
                    lookfor_ctor_init = false;
                    // The first statement starts here; line up with it.
                    if cpp_base_class {
                        cached.lpos.col = s.offset_from(line) as colnr_T;
                    }
                }

                // When the line ends in a comma, do not align with it.
                if lnum == (*curwin.get()).w_cursor.lnum && *s as u8 == b',' && cin_nocode(s.add(1))
                {
                    cached.lpos.col = 0;
                }
                s = cin_skipcomment(s.add(1));
            }
        }

        cached.found = c_int::from(cpp_base_class);
        if cpp_base_class {
            cached.lpos.lnum = lnum;
        }
        cpp_base_class
    }
}

/// The indent for a line inside a base-class or initialiser list, given the
/// column [`cin_is_cpp_baseclass`] chose (0 meaning "nothing to line up
/// with").
///
/// # Safety
/// Reads and writes the cursor; may unlock the current line.
pub(crate) unsafe fn get_baseclass_amount(col: c_int) -> c_int {
    unsafe {
        let mut amount = if col == 0 {
            let mut amount = get_indent();
            if find_last_paren(get_cursor_line_ptr(), b'(', b')') {
                let trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                if !trypos.is_null() {
                    amount = get_indent_lnum((*trypos).lnum);
                }
            }
            if !cin_ends_in(get_cursor_line_ptr(), b",") {
                amount += (*curbuf.get()).b_ind_cpp_baseclass;
            }
            amount
        } else {
            (*curwin.get()).w_cursor.col = col;
            let mut vcol: colnr_T = 0;
            getvcol(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                &raw mut vcol,
                ::core::ptr::null_mut::<colnr_T>(),
                ::core::ptr::null_mut::<colnr_T>(),
            );
            vcol
        };
        if amount < (*curbuf.get()).b_ind_cpp_baseclass {
            amount = (*curbuf.get()).b_ind_cpp_baseclass;
        }
        amount
    }
}
