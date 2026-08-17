//! Guessing the new line's indent from the old one: 'autoindent' and
//! 'smartindent'.
//!
//! Everything here runs *before* the line is opened and only ever computes a
//! column, but it does it by moving the cursor around -- `get_indent` reads
//! the cursor line and `findmatch` searches from the cursor -- so the caller
//! saves and restores `w_cursor` around the whole block.
//!
//! Besides the answer, this sets three of 'smartindent''s globals:
//! `did_si` ("indent the new line one level further"), `can_si_back` ("a `{`
//! typed on the new line may un-indent it") and, through the `no_si` half of
//! the answer, whether `did_si` should be cleared again once the indent has
//! been applied.

#![deny(unsafe_op_in_unsafe_fn)]

use ::core::ffi::{c_char, c_int};

use super::super::*;

/// Where the C comment that ends on this line began, as an indent.
///
/// Walks forward from the start of a `/*`-style leader looking for the `*/`
/// that closes it, and if `findmatch` can pair it up, answers the indent of
/// the line the comment *started* on. `None` leaves the indent alone.
///
/// ```text
///     /*
///      * A comment.
///      */
///     #define IN_THE_WAY
///     This should line up here;
/// ```
///
/// # Safety
/// `ptr` must be the current cursor line, NUL-terminated.
unsafe fn indent_of_comment_start(mut ptr: *mut c_char) -> Option<c_int> {
    unsafe {
        let mut p = skipwhite(ptr);
        if c_int::from(*p) == '/' as c_int && c_int::from(*p.add(1)) == '*' as c_int {
            p = p.add(1);
        }
        if c_int::from(*p) != '*' as c_int {
            return None;
        }
        p = p.add(1);
        while *p != 0 {
            if c_int::from(*p) == '/' as c_int && c_int::from(*p.offset(-1)) == '*' as c_int {
                // End of a C comment: line the indent up with the line
                // holding the start of it.
                (*curwin.get()).w_cursor.col = p.offset_from(ptr) as colnr_T;
                let pos = findmatch(::core::ptr::null_mut(), NUL);
                if !pos.is_null() {
                    (*curwin.get()).w_cursor.lnum = (*pos).lnum;
                    return Some(get_indent());
                }
                // findmatch may have made `ptr` stale; fetch it again.
                ptr = ml_get((*curwin.get()).w_cursor.lnum);
                p = ptr.offset((*curwin.get()).w_cursor.col as isize);
            }
            p = p.add(1);
        }
        None
    }
}

/// 'smartindent' looking *down* the file, for `o` and `<CR>`.
///
/// Answers the new indent and whether `did_si` was set by a `{` -- which the
/// caller has to undo after applying the indent, so that typing `{` on the
/// new line does not un-indent it a second time.
///
/// # Safety
/// `ptr` must be the cursor line and `newindent` its measured indent. The
/// caller must restore `w_cursor` afterwards.
unsafe fn smart_indent_forward(
    mut ptr: *mut c_char,
    flags: c_int,
    lead_len: c_int,
    mut newindent: c_int,
) -> (c_int, bool) {
    unsafe {
        // Skip preprocessor directives, unless they are comments.
        if lead_len == 0 && c_int::from(*ptr) == '#' as c_int {
            while c_int::from(*ptr) == '#' as c_int && (*curwin.get()).w_cursor.lnum > 1 {
                (*curwin.get()).w_cursor.lnum -= 1;
                ptr = ml_get((*curwin.get()).w_cursor.lnum);
            }
            newindent = get_indent();
        }
        // Re-measure: the `#` walk above may have landed on another line.
        let lead_len = if flags & OPENLINE_DO_COM != 0 {
            get_leader_len(ptr, ::core::ptr::null_mut(), false, true)
        } else {
            0
        };

        if lead_len > 0 {
            if let Some(indent) = indent_of_comment_start(ptr) {
                newindent = indent;
            }
            return (newindent, false);
        }

        // Not a comment line: look at what the line ends with.
        //
        // `wrapping_offset`, not `offset`: on an empty line upstream forms
        // `ptr - 1` and reads it, which is out of bounds. The `#` walk above
        // can land on an empty line, so this is reachable (O-B15-20); the
        // read is kept as upstream has it, only the pointer arithmetic is
        // spelled so that forming the address is not itself UB.
        let mut p = ptr.add(strlen(ptr)).wrapping_offset(-1);
        while p > ptr && ascii_iswhite(c_int::from(*p)) {
            p = p.offset(-1);
        }
        let last_char = *p;

        // Step back over the `{` or `;` to whatever came before it.
        if c_int::from(last_char) == '{' as c_int || c_int::from(last_char) == ';' as c_int {
            if p > ptr {
                p = p.offset(-1);
            }
            while p > ptr && ascii_iswhite(c_int::from(*p)) {
                p = p.offset(-1);
            }
        }

        // A statement split over several lines lines up with the line the
        // condition started on:
        //     if (condition &&
        //             condition) {
        //         Should line up here!
        //     }
        if c_int::from(*p) == ')' as c_int {
            (*curwin.get()).w_cursor.col = p.offset_from(ptr) as colnr_T;
            let pos = findmatch(::core::ptr::null_mut(), '(' as c_int);
            if !pos.is_null() {
                (*curwin.get()).w_cursor.lnum = (*pos).lnum;
                newindent = get_indent();
                ptr = get_cursor_line_ptr();
            }
        }

        let mut no_si = false;
        if c_int::from(last_char) == '{' as c_int {
            // A trailing `{` indents, with no need to look for an `if`.
            did_si.set(true);
            no_si = true; // ... and typing `{` must not un-indent it again
        } else if c_int::from(last_char) != ';' as c_int
            && c_int::from(last_char) != '}' as c_int
            && cin_is_cinword(ptr)
        {
            // One of 'cinwords', and the line before did not finish a
            // statement.
            did_si.set(true);
        }
        (newindent, no_si)
    }
}

/// 'smartindent' looking *up* the file, for `O`.
///
/// # Safety
/// `ptr` must be the cursor line. The caller must restore `w_cursor`
/// afterwards.
unsafe fn smart_indent_backward(
    mut ptr: *mut c_char,
    lead_len: c_int,
    mut newindent: c_int,
) -> c_int {
    unsafe {
        // Skip preprocessor directives, unless they are comments. A `\`
        // continuation carries the directive onto the next line.
        if lead_len == 0 && c_int::from(*ptr) == '#' as c_int {
            let mut was_backslashed = false;
            while (c_int::from(*ptr) == '#' as c_int || was_backslashed)
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
            {
                was_backslashed = *ptr != 0
                    && c_int::from(*ptr.add(strlen(ptr).wrapping_sub(1))) == '\\' as c_int;
                (*curwin.get()).w_cursor.lnum += 1;
                ptr = ml_get((*curwin.get()).w_cursor.lnum);
            }
            newindent = if was_backslashed {
                0 // ran off the end of the file
            } else {
                get_indent()
            };
        }

        if c_int::from(*skipwhite(ptr)) == '}' as c_int {
            did_si.set(true); // a line starting with `}` indents
        } else {
            can_si_back.set(true); // a `{` typed next can delete the indent
        }
        newindent
    }
}

/// The whole 'smartindent' guess, for either direction.
///
/// Answers the new indent and the `no_si` flag (see [`smart_indent_forward`]).
/// The cursor is saved and restored around it.
///
/// # Safety
/// `saved_line` must be a NUL-terminated copy of the cursor line.
pub(crate) unsafe fn smart_indent(
    dir: c_int,
    flags: c_int,
    saved_line: *mut c_char,
    newindent: c_int,
) -> (c_int, bool) {
    unsafe {
        let old_cursor = (*curwin.get()).w_cursor;
        let ptr = saved_line;
        let lead_len = if flags & OPENLINE_DO_COM != 0 {
            get_leader_len(ptr, ::core::ptr::null_mut(), false, true)
        } else {
            0
        };
        let answer = if dir == FORWARD {
            smart_indent_forward(ptr, flags, lead_len, newindent)
        } else {
            (smart_indent_backward(ptr, lead_len, newindent), false)
        };
        (*curwin.get()).w_cursor = old_cursor;
        answer
    }
}
