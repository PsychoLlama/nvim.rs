//! The C++ shapes: `namespace`, `extern "C"` and a base-class list.
//!
//! Three 'cinoptions' letters live here.  `N` (`b_ind_cpp_namespace`) and `E`
//! (`b_ind_cpp_extern_c`) are the two block openers whose contents upstream
//! does not want indented, and both are recognised from the *opening* line.
//! `k` (`b_ind_cpp_baseclass`) is the harder one: [`cin_is_cpp_baseclass`]
//! decides whether a line is inside a constructor's initialiser list or a
//! class's base clause, which needs a scan back to the `class`/`:` that
//! started it -- so it caches its answer in the [`CppBaseclassCache`] its
//! caller owns.
//!
//! | C | here |
//! | --- | --- |
//! | `cin_is_cpp_namespace` | [`opens_namespace`] |
//! | `cin_is_cpp_extern_c` | [`opens_extern_c`] |
//! | `cin_is_cpp_baseclass` | [`in_baseclass_list`] |

#![forbid(unsafe_code)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

/// Whether `line[at..]` opens a `namespace` block -- 'cinoptions' `N`.
///
/// `inline` and `export` may precede it in any order, and the name may be a
/// C++17 nested one (`a::b::c`), but *two* names in a row is not a namespace
/// declaration: that is what `has_name`/`has_name_start` are tracking.
pub(crate) fn opens_namespace(line: &[u8], at: usize) -> bool {
    let mut at = code_at(line, at);
    let mut has_name = false;
    let mut has_name_start = false;

    // Skip over "inline" and "export" in any order.
    loop {
        let tail = &line[at.min(line.len())..];
        // SAFETY: `vim_iswordc` reads the current buffer's 'iskeyword' table.
        if !(tail.starts_with(b"inline") || tail.starts_with(b"export"))
            || vim_iswordc(c_int::from(byte_at(line, at + 6)))
        {
            break;
        }
        at += 6;
        at = code_at(line, at + skip::white(&line[at.min(line.len())..]));
    }

    if !line[at.min(line.len())..].starts_with(b"namespace")
        || vim_iswordc(c_int::from(byte_at(line, at + 9)))
    {
        return false;
    }
    at += 9;
    let mut at = code_at(line, at + skip::white(&line[at.min(line.len())..]));

    loop {
        let c = byte_at(line, at);
        if c == 0 {
            break;
        }
        if ascii_iswhite(c_int::from(c)) {
            has_name = true; // found the end of a name
            at = code_at(line, at + skip::white(&line[at.min(line.len())..]));
        } else if c == b'{' {
            break;
        // SAFETY: `vim_iswordc` reads the current buffer's 'iskeyword' table.
        } else if vim_iswordc(c_int::from(c)) {
            if has_name {
                return false; // a word character after a finished name
            }
            has_name_start = true;
            at += 1;
        } else if c == b':'
            && byte_at(line, at + 1) == b':'
            && vim_iswordc(c_int::from(byte_at(line, at + 2)))
        {
            if !has_name_start || has_name {
                return false;
            }
            at += 3; // C++17 nested namespace
        } else {
            return false;
        }
    }
    true
}

/// Whether `line[at..]` opens an `extern "C"` or `extern "C++"` linkage block
/// -- 'cinoptions' `E`.
pub(crate) fn opens_extern_c(line: &[u8], at: usize) -> bool {
    let at = code_at(line, at);
    // SAFETY: `vim_iswordc` reads the current buffer's 'iskeyword' table.
    if !line[at.min(line.len())..].starts_with(b"extern")
        || vim_iswordc(c_int::from(byte_at(line, at + 6)))
    {
        return false;
    }
    let at = at + 6;
    let mut at = code_at(line, at + skip::white(&line[at.min(line.len())..]));

    let mut has_string_literal = false;
    loop {
        let c = byte_at(line, at);
        if c == 0 {
            break;
        }
        let tail = &line[at.min(line.len())..];
        if ascii_iswhite(c_int::from(c)) {
            at = code_at(line, at + skip::white(tail));
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
            at += lang.len();
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
pub(crate) fn in_baseclass_list(cached: &mut CppBaseclassCache) -> bool {
    let mut lnum = Win::current().w_cursor.lnum;
    if cached.lpos.lnum <= lnum {
        return cached.found != 0; // use the cached result
    }
    cached.lpos.col = 0;

    let mut lines = Lines::current();
    {
        let line = lines.line(lnum);
        let at = skip::white(line);
        if byte_at(line, at) == b'#' {
            return false; // skip #define FOO x ? (x) : x
        }
        if only_comment_left(line, at) {
            return false;
        }
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
        let line = lines.line(lnum - 1);
        let mut at = skip::white(line);
        let stop = if byte_at(line, at) == b'#' || byte_at(line, at) == 0 {
            true
        } else {
            while byte_at(line, at) != 0 {
                at = code_at(line, at);
                if byte_at(line, at) == b'{'
                    || byte_at(line, at) == b'}'
                    || (byte_at(line, at) == b';' && only_comment_left(line, at + 1))
                {
                    break;
                }
                if byte_at(line, at) != 0 {
                    at += 1;
                }
            }
            byte_at(line, at) != 0
        };
        if stop {
            break;
        }
        lnum -= 1;
    }

    cached.lpos.lnum = lnum;
    let mut at = 0usize;
    'lines: loop {
        let line = lines.line(lnum);
        loop {
            if byte_at(line, at) == 0 {
                if lnum == Win::current().w_cursor.lnum {
                    break 'lines;
                }
                lnum += 1; // continue into the cursor's line
                at = 0;
                continue 'lines;
            }
            if at == 0 {
                // Do not recognise "case (foo):" as a base class.
                if is_case_label(line, 0, false) {
                    break 'lines;
                }
                at = code_at(line, 0);
                if byte_at(line, at) == 0 {
                    continue;
                }
            }

            // Past the test above, `at` is never on the line's end, so
            // `at + 1` is at worst one past it -- which is what every step
            // below rests on.
            let c = byte_at(line, at);
            if c == b'"' || (c == b'R' && byte_at(line, at + 1) == b'"') {
                at = string_end_at(line, at) + 1;
            } else if c == b':' {
                if byte_at(line, at + 1) == b':' {
                    // A double colon: no longer a constructor initialisation.
                    lookfor_ctor_init = false;
                    at = code_at(line, at + 2);
                } else {
                    if lookfor_ctor_init || class_or_struct {
                        // The start of a base-class declaration or of a
                        // constructor initialisation.
                        cpp_base_class = true;
                        lookfor_ctor_init = false;
                        class_or_struct = false;
                        cached.lpos.col = 0;
                    }
                    at = code_at(line, at + 1);
                }
            } else if let Some(word) = [&b"class"[..], &b"struct"[..]]
                .into_iter()
                .find(|word| starts_with_word(line, at, word))
            {
                class_or_struct = true;
                lookfor_ctor_init = false;
                at = code_at(line, at + word.len());
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
                } else if !vim_is_ident_char(c_int::from(c)) {
                    // Not an identifier: we are wrong.
                    class_or_struct = false;
                    lookfor_ctor_init = false;
                } else if cached.lpos.col == 0 {
                    lookfor_ctor_init = false;
                    // The first statement starts here; line up with it.
                    if cpp_base_class {
                        cached.lpos.col = at as ColNr;
                    }
                }

                // When the line ends in a comma, do not align with it.
                if lnum == Win::current().w_cursor.lnum
                    && c == b','
                    && only_comment_left(line, at + 1)
                {
                    cached.lpos.col = 0;
                }
                at = code_at(line, at + 1);
            }
        }
    }

    cached.found = c_int::from(cpp_base_class);
    if cpp_base_class {
        cached.lpos.lnum = lnum;
    }
    cpp_base_class
}

/// The indent for a line inside a base-class or initialiser list, given the
/// column [`in_baseclass_list`] chose (0 meaning "nothing to line up with").
pub(crate) fn get_baseclass_amount(col: c_int) -> c_int {
    let mut amount = if col == 0 {
        // SAFETY: reads the cursor's line of the current buffer.
        let mut amount = get_indent();
        // The borrow ends with the statement: the match search reads other
        // lines, and only runs when a paren was found, as upstream has it.
        let cursor_lnum = Win::current().w_cursor.lnum;
        let has_paren = find_last_paren(Lines::current().line(cursor_lnum), b'(', b')');
        if let Some(trypos) = has_paren
            .then(|| find_match_paren(Buf::current().b_ind_maxparen))
            .flatten()
        {
            // SAFETY: `trypos` is a position in the current buffer.
            amount = get_indent_lnum(trypos.lnum);
        }
        let cursor_lnum = Win::current().w_cursor.lnum;
        if !ends_in(Lines::current().line(cursor_lnum), 0, b",") {
            amount += Buf::current().b_ind_cpp_baseclass;
        }
        amount
    } else {
        let mut win = Win::current();
        win.w_cursor.col = col;
        win.vcol(win.cursor())
    };
    if amount < Buf::current().b_ind_cpp_baseclass {
        amount = Buf::current().b_ind_cpp_baseclass;
    }
    amount
}
