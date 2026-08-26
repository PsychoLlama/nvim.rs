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

use core::ffi::{c_char, c_int};

use crate::change::*;
use crate::types::NUL;
use crate::winlayer::{Buf, Win};

/// A cursor into the NUL-terminated line 'smartindent' is looking at.
///
/// Every walk below tests the byte under the cursor before stepping, so the
/// cursor stays inside the line -- which leaves *taking* it as the one
/// unchecked step and makes every read after it ordinary code.
#[derive(Clone, Copy)]
struct Ln(*mut c_char);

impl Ln {
    /// # Safety
    /// `p` must point inside a NUL-terminated line and stay valid for as long
    /// as the cursor is used.
    unsafe fn new(p: *mut c_char) -> Self {
        Self(p)
    }

    /// The byte under the cursor.
    fn byte(self) -> c_int {
        // SAFETY: the constructor's promise, and no walk here steps past the
        // terminating NUL or before the start of the line.
        c_int::from(unsafe { *self.0 })
    }

    /// The byte `off` bytes from the cursor.
    fn byte_at(self, off: isize) -> c_int {
        // SAFETY: as [`Ln::byte`]; `off` names a byte the walk has already
        // passed or is about to reach.
        c_int::from(unsafe { *self.0.offset(off) })
    }

    /// Whether the byte under the cursor is white space.
    fn white(self) -> bool {
        ascii_iswhite(self.byte())
    }

    fn step(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    fn back(&mut self) {
        self.0 = self.0.wrapping_sub(1);
    }

    fn raw(self) -> *mut c_char {
        self.0
    }

    /// How far into `start` the cursor is.
    fn col_in(self, start: Ln) -> colnr_T {
        // SAFETY: both cursors are inside the same line.
        unsafe { self.0.offset_from(start.0) as colnr_T }
    }
}

/// The indent of the line the cursor is on.
fn indent_here() -> c_int {
    // SAFETY: the cursor is on a valid line of the current buffer.
    get_indent()
}

/// Line `lnum` of the current buffer.
fn line_at(lnum: linenr_T) -> *mut c_char {
    // SAFETY: every caller has just clamped `lnum` into the buffer.
    ml_get(lnum)
}

/// `findmatch` from the cursor, for `initc`.
fn find_match(initc: c_int) -> Option<pos_T> {
    // SAFETY: the cursor is on a valid line of the current buffer.
    unsafe { findmatch(::core::ptr::null_mut(), initc) }
}

/// [`get_leader_len`] when only the length is wanted.
fn leader_len_of(ptr: *mut c_char) -> c_int {
    // SAFETY: a NUL-terminated line, and no flags are asked for.
    unsafe { get_leader_len(ptr, ::core::ptr::null_mut(), false, true) }
}

/// The first non-white byte of `ptr`, as a cursor.
///
/// # Safety
/// `ptr` must be NUL-terminated.
unsafe fn skip_white(ptr: *mut c_char) -> Ln {
    // SAFETY: the caller's NUL-terminated line; `skipwhite` stops at its NUL
    // at the latest.
    unsafe { Ln::new(skipwhite(ptr)) }
}

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
unsafe fn indent_of_comment_start(ptr: *mut c_char) -> Option<c_int> {
    // SAFETY: the caller's NUL-terminated line.
    let mut start = unsafe { Ln::new(ptr) };
    // SAFETY: as above.
    let mut p = unsafe { skip_white(ptr) };
    if p.byte() == '/' as c_int && p.byte_at(1) == '*' as c_int {
        p.step();
    }
    if p.byte() != '*' as c_int {
        return None;
    }
    p.step();
    while p.byte() != 0 {
        if p.byte() == '/' as c_int && p.byte_at(-1) == '*' as c_int {
            // End of a C comment: line the indent up with the line
            // holding the start of it.
            cur_win().w_cursor.col = p.col_in(start);
            if let Some(pos) = find_match(NUL) {
                cur_win().w_cursor.lnum = pos.lnum;
                return Some(indent_here());
            }
            // findmatch may have made `ptr` stale; fetch it again.
            let at = line_at(cur_win().w_cursor.lnum);
            // SAFETY: a line of the current buffer, and the cursor column is
            // inside it.
            start = unsafe { Ln::new(at) };
            p = unsafe { Ln::new(at.wrapping_offset(cur_win().w_cursor.col as isize)) };
        }
        p.step();
    }
    None
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
    // SAFETY: the caller's NUL-terminated line.
    let mut start = unsafe { Ln::new(ptr) };
    // Skip preprocessor directives, unless they are comments.
    if lead_len == 0 && start.byte() == '#' as c_int {
        while start.byte() == '#' as c_int && cur_win().w_cursor.lnum > 1 {
            cur_win().w_cursor.lnum -= 1;
            ptr = line_at(cur_win().w_cursor.lnum);
            // SAFETY: a line of the current buffer.
            start = unsafe { Ln::new(ptr) };
        }
        newindent = indent_here();
    }
    // Re-measure: the `#` walk above may have landed on another line.
    let lead_len = if flags & OPENLINE_DO_COM != 0 {
        leader_len_of(ptr)
    } else {
        0
    };

    if lead_len > 0 {
        // SAFETY: `ptr` is a NUL-terminated line of the current buffer.
        if let Some(indent) = unsafe { indent_of_comment_start(ptr) } {
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
    //
    // SAFETY: the line's own NUL, stepped back one -- which is the read
    // upstream makes.
    let end = unsafe { ptr.add(strlen(ptr)) }.wrapping_offset(-1);
    // SAFETY: as above.
    let mut p = unsafe { Ln::new(end) };
    while p.raw() > start.raw() && p.white() {
        p.back();
    }
    let last_char = p.byte();

    // Step back over the `{` or `;` to whatever came before it.
    if last_char == '{' as c_int || last_char == ';' as c_int {
        if p.raw() > start.raw() {
            p.back();
        }
        while p.raw() > start.raw() && p.white() {
            p.back();
        }
    }

    // A statement split over several lines lines up with the line the
    // condition started on:
    //     if (condition &&
    //             condition) {
    //         Should line up here!
    //     }
    if p.byte() == ')' as c_int {
        cur_win().w_cursor.col = p.col_in(start);
        if let Some(pos) = find_match('(' as c_int) {
            cur_win().w_cursor.lnum = pos.lnum;
            newindent = indent_here();
            // SAFETY: the cursor is on a valid line of the current buffer.
            ptr = get_cursor_line_ptr();
        }
    }

    let mut no_si = false;
    if last_char == '{' as c_int {
        // A trailing `{` indents, with no need to look for an `if`.
        did_si.set(true);
        no_si = true; // ... and typing `{` must not un-indent it again
    } else if last_char != ';' as c_int
        && last_char != '}' as c_int
        // SAFETY: a NUL-terminated line of the current buffer.
        && unsafe { cin_is_cinword(ptr) }
    {
        // One of 'cinwords', and the line before did not finish a
        // statement.
        did_si.set(true);
    }
    (newindent, no_si)
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
    // SAFETY: the caller's NUL-terminated line.
    let mut start = unsafe { Ln::new(ptr) };
    // Skip preprocessor directives, unless they are comments. A `\`
    // continuation carries the directive onto the next line.
    if lead_len == 0 && start.byte() == '#' as c_int {
        let mut was_backslashed = false;
        while (start.byte() == '#' as c_int || was_backslashed)
            && cur_win().w_cursor.lnum < cur_buf().b_ml.ml_line_count
        {
            // SAFETY: `ptr` is NUL-terminated and not empty, as just tested.
            was_backslashed = start.byte() != 0
                && unsafe { c_int::from(*ptr.add(strlen(ptr).wrapping_sub(1))) } == '\\' as c_int;
            cur_win().w_cursor.lnum += 1;
            ptr = line_at(cur_win().w_cursor.lnum);
            // SAFETY: a line of the current buffer.
            start = unsafe { Ln::new(ptr) };
        }
        newindent = if was_backslashed {
            0 // ran off the end of the file
        } else {
            indent_here()
        };
    }

    // SAFETY: `ptr` is a NUL-terminated line of the current buffer.
    if unsafe { skip_white(ptr) }.byte() == '}' as c_int {
        did_si.set(true); // a line starting with `}` indents
    } else {
        can_si_back.set(true); // a `{` typed next can delete the indent
    }
    newindent
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
    let old_cursor = cur_win().w_cursor;
    let ptr = saved_line;
    let lead_len = if flags & OPENLINE_DO_COM != 0 {
        leader_len_of(ptr)
    } else {
        0
    };
    // SAFETY: the caller's copy of the cursor line.
    let answer = unsafe {
        if dir == FORWARD {
            smart_indent_forward(ptr, flags, lead_len, newindent)
        } else {
            (smart_indent_backward(ptr, lead_len, newindent), false)
        }
    };
    cur_win().w_cursor = old_cursor;
    answer
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
