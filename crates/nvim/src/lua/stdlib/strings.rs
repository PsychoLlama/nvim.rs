//! The string helpers: UTF-8 indexing, comparison and `iconv`.
//!
//! [`str_utfindex`]/[`str_byteindex`] convert between byte and character
//! offsets, [`str_utf_pos`]/[`str_utf_start`]/[`str_utf_end`] report
//! codepoint boundaries, [`stricmp`] is the case-insensitive compare the Lua
//! side cannot express, and [`iconv`] is `vim.iconv()`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use crate::lua::stack::{At, Kind, Raise, Stack};
use crate::mbyte::{
    c_utf_index_to_byte, c_utf_lengths, canonize, cp_bounds, promised_char_len, skip_len,
};
use crate::os::cshim::cstr_ncasecmp;
use crate::types::{CONV_NONE_INIT, lua_Integer, lua_Number};

/// A count as the Lua integer it is pushed as.
fn as_lua(n: usize) -> lua_Integer {
    lua_Integer::try_from(n).unwrap_or(lua_Integer::MAX)
}

/// The string at slot 1, which a check has already made one.
fn subject(stack: &Stack) -> &[u8] {
    stack.string(At::arg(1)).unwrap_or_default()
}

/// `vim.str_utfindex()`: the UTF-32 and UTF-16 lengths of the string at slot 1
/// up to the byte index at slot 2, or of the whole string when it is absent.
///
/// Two nils for an index outside the string.
pub(crate) fn str_utfindex(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.check_bytes(At::arg(1))?;
    let idx = if stack.is_none_or_nil(At::arg(2)) {
        None
    } else {
        Some(stack.check_integer(At::arg(2))?)
    };
    let s1 = subject(stack);
    let idx = match idx {
        None => s1.len(),
        Some(idx) => match usize::try_from(idx).ok().filter(|&idx| idx <= s1.len()) {
            Some(idx) => idx,
            None => {
                stack.push_nil();
                stack.push_nil();
                return Ok(2);
            }
        },
    };

    let (codepoints, codeunits) = c_utf_lengths(s1, idx);
    stack.push_integer(as_lua(codepoints));
    stack.push_integer(as_lua(codeunits));
    Ok(2)
}

/// `vim.str_utf_pos()`: the 1-based byte index of every codepoint in the
/// string at slot 1, as a list. The walk stops at a NUL.
pub(crate) fn str_utf_pos(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.check_bytes(At::arg(1))?;
    let s1 = subject(stack);
    let mut starts = Vec::new();
    let mut i = 0;
    while i < s1.len() && s1[i] != 0 {
        starts.push(i);
        i += promised_char_len(&s1[i..]);
    }
    stack.new_table(0, 0);
    for (n, start) in (1..).zip(starts) {
        stack.push_integer(as_lua(start) + 1);
        stack.raw_set_index(At::from_top(2), n);
    }
    Ok(1)
}

/// The 0-based byte offset the 1-based offset at slot 2 names in the string
/// at slot 1, refused outside it.
fn checked_offset(stack: &mut Stack) -> Result<usize, Raise> {
    stack.check_bytes(At::arg(1))?;
    let offset = stack.check_integer(At::arg(2))?;
    let len = subject(stack).len();
    match usize::try_from(offset).ok().filter(|&n| n > 0 && n <= len) {
        Some(offset) => Ok(offset - 1),
        None => Err(Raise::message("index out of range")),
    }
}

/// `vim.str_utf_start()`: how far *back* from the 1-based byte offset at slot
/// 2 the codepoint containing it begins, as a non-positive number.
pub(crate) fn str_utf_start(stack: &mut Stack) -> Result<c_int, Raise> {
    let off = checked_offset(stack)?;
    let bounds = cp_bounds(subject(stack), off);
    stack.push_integer(-lua_Integer::from(bounds.begin_off));
    Ok(1)
}

/// `vim.str_utf_end()`: how far *on* from the 1-based byte offset at slot 2
/// the codepoint containing it ends.
pub(crate) fn str_utf_end(stack: &mut Stack) -> Result<c_int, Raise> {
    let off = checked_offset(stack)?;
    let bounds = cp_bounds(subject(stack), off);
    stack.push_integer(lua_Integer::from(bounds.end_off) - 1);
    Ok(1)
}

/// `vim.str_byteindex()`: the byte offset of the UTF-32 (or, with a truthy
/// slot 3, UTF-16) index at slot 2, or nil when the string is shorter.
pub(crate) fn str_byteindex(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.check_bytes(At::arg(1))?;
    let idx = stack.check_integer(At::arg(2))?;
    let Ok(idx) = usize::try_from(idx) else {
        stack.push_nil();
        return Ok(1);
    };
    let use_utf16 = stack.top().index() >= 3 && stack.to_boolean(At::arg(3));

    let s1 = subject(stack);
    match c_utf_index_to_byte(s1, s1.len(), idx, use_utf16) {
        Some(byteidx) => stack.push_integer(as_lua(byteidx)),
        None => stack.push_nil(),
    }
    Ok(1)
}

/// `strcasecmp` on two NUL-terminated runs.
fn casecmp(a: &CStr, b: &CStr) -> c_int {
    cstr_ncasecmp(a, b, usize::MAX)
}

/// `vim.stricmp()`: -1, 0 or 1 for the two strings at slots 1 and 2, ignoring
/// case.
///
/// A Lua string may contain NULs and `strcasecmp` stops at the first one, so
/// the comparison runs NUL-separated run by NUL-separated run; a string that
/// has one where the other has ended sorts after it. Uppercasing can change a
/// byte's length, so the two sides cannot be advanced by a common amount.
pub(crate) fn stricmp(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.check_bytes(At::arg(1))?;
    stack.check_bytes(At::arg(2))?;
    let ret = {
        // Each with its terminator, so every run is a C string in place.
        let mut s1 = stack.string_with_nul(At::arg(1)).unwrap_or(b"\0");
        let mut s2 = stack.string_with_nul(At::arg(2)).unwrap_or(b"\0");
        loop {
            let run1 = CStr::from_bytes_until_nul(s1).expect("a Lua string ends in a NUL");
            let run2 = CStr::from_bytes_until_nul(s2).expect("a Lua string ends in a NUL");
            let ret = casecmp(run1, run2);
            if ret != 0 {
                break ret;
            }
            // Whether the run ended at a NUL of the string's own rather than
            // at its terminator.
            let more1 = run1.count_bytes() + 1 < s1.len();
            let more2 = run2.count_bytes() + 1 < s2.len();
            // Compare "a\0" greater than "a".
            if more1 != more2 {
                break c_int::from(more1) - c_int::from(more2);
            }
            if !more1 {
                break 0;
            }
            s1 = &s1[run1.count_bytes() + 1..];
            s2 = &s2[run2.count_bytes() + 1..];
        }
    };

    stack.pop(2);
    stack.push_number(lua_Number::from(ret.signum()));
    Ok(1)
}

/// `vim.iconv()`: the string at slot 1 recoded from the encoding at slot 2 to
/// the one at slot 3, or nil when the conversion fails.
pub(crate) fn iconv(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() < 3 {
        return Err(Raise::message("Expected at least 3 arguments"));
    }
    for i in 1..=3 {
        if stack.kind(At::arg(i)) != Kind::String {
            return Err(Raise::argument(i, "expected string"));
        }
    }

    let converted = {
        // Each name as `lua_tostring`'s C string: up to its first NUL.
        let name = |at: c_int| {
            let name = stack.c_string(At::arg(at)).unwrap_or_default().to_bytes();
            canonize(&name[skip_len(name)..])
        };
        let (from, to) = (name(2), name(3));

        let mut vimconv = CONV_NONE_INIT;
        let _ = vimconv.setup_ext(Some(from.as_cstr()), false, Some(to.as_cstr()), false);
        vimconv.convert(subject(stack))
    };
    match converted {
        None => stack.push_nil(),
        Some(text) => stack.push_bytes(&text),
    }
    Ok(1)
}
