//! `$VAR` and `~` expansion inside a path.
//!
//! [`expand_env_bounded`] is the whole of it; the three entry points are its
//! defaults. It copies into a result bounded the way upstream's fixed
//! destination buffer was and, whenever anything at all goes wrong, keeps
//! what it has already copied -- which is why it is written as a copy loop
//! with an early-out per name rather than as a parser.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::charset::{vim_is_ident_char, vim_isfilec};
use crate::env::getenv_resolved;
use crate::eval::skip_expr;
use crate::mbyte::Codepoint;
use crate::memory::XString;
use crate::os::env::with_home_dir;
use crate::os::users::user_home_dir;
use crate::path::MAXPATHL;
use crate::path::{after_pathsep, vim_ispathsep};
use crate::strings::escaped_bytes;
use core::ffi::{CStr, c_int};

/// `$VAR` and `~` expanded in `src`, into a `MAXPATHL` result (cut where
/// that buffer cuts it). `one` treats `src` as a single file name, so only
/// a leading `~` is expanded.
pub(crate) fn expand_env(src: &CStr, one: bool) -> XString {
    XString::from_vec(expand_env_bounded(
        src.to_bytes(),
        MAXPATHL as usize,
        false,
        one,
        None,
    ))
}

/// [`expand_env`] with escaping and a restart prefix: see
/// [`expand_env_bounded`].
pub(crate) fn expand_env_escaped(
    src: &CStr,
    esc: bool,
    one: bool,
    prefix: Option<&CStr>,
) -> XString {
    let prefix = prefix.map(CStr::to_bytes);
    XString::from_vec(expand_env_bounded(
        src.to_bytes(),
        MAXPATHL as usize,
        esc,
        one,
        prefix,
    ))
}

/// [`expand_env`] into a result of at most `capacity - 1` bytes: the
/// expansion, cut to fit exactly as upstream's fixed-size destination cuts
/// it when told one byte less than it holds.
pub(crate) fn expand_env_within(src: &CStr, capacity: usize) -> Vec<u8> {
    expand_env_bounded(src.to_bytes(), capacity - 1, false, false, None)
}

/// What one `$VAR`, `~`, or `~user` at the cursor expands to.
#[derive(Debug)]
struct Expansion {
    /// The expansion, or `None` when the name did not resolve.
    value: Option<Vec<u8>>,
    /// Where in the source to carry on from.
    tail: usize,
}

/// The byte at `i`, or the NUL past the end.
fn byte_at(src: &[u8], i: usize) -> u8 {
    src.get(i).copied().unwrap_or(0)
}

/// Resolve the `$VAR` at `src[at]`. The name is cut at `room - 1` bytes,
/// as upstream's copy of it into the destination was.
fn resolve_env_var(src: &[u8], at: usize, room: usize) -> Expansion {
    let mut tail = at + 1;
    let mut name = Vec::new();
    let mut left = room.saturating_sub(1);

    // A `${VAR}` name may hold anything but '}'; a `$VAR` one is limited to
    // 'isident' characters -- which is why the check below asks about the
    // brace rather than about `braced`: 'isident' may itself contain '{'.
    let brace = byte_at(src, tail) == b'{';
    let braced = brace && !vim_is_ident_char(Codepoint::byte(b'{'));
    if braced {
        tail += 1; // ignore '{'
        while left > 0 && byte_at(src, tail) != 0 && byte_at(src, tail) != b'}' {
            left -= 1;
            name.push(src[tail]);
            tail += 1;
        }
    } else {
        while left > 0 && byte_at(src, tail) != 0 && vim_is_ident_char(Codepoint::from(src[tail])) {
            left -= 1;
            name.push(src[tail]);
            tail += 1;
        }
    }

    // A `${VAR}` that never reached its '}' is not a variable at all.
    if brace && byte_at(src, tail) != b'}' {
        return Expansion { value: None, tail };
    }
    if brace {
        tail += 1;
    }
    let value = getenv_resolved(XString::from_vec(name).as_cstr()).map(|value| value.to_vec());
    Expansion { value, tail }
}

/// Resolve the `~user` at `src[at]`; no directory when the user is not in
/// the password database. (Asking the shell instead would re-enter the
/// event loop, which aborts in a fast event.)
fn resolve_user_dir(src: &[u8], at: usize, room: usize) -> Expansion {
    let mut tail = at;
    let mut left = room.saturating_sub(1);
    while left > 0
        && byte_at(src, tail) != 0
        && vim_isfilec(Codepoint::from(src[tail]))
        && !vim_ispathsep(c_int::from(src[tail]))
    {
        left -= 1;
        tail += 1;
    }
    let value = if tail == at {
        None
    } else {
        user_home_dir(XString::from_bytes(&src[at + 1..tail]).as_cstr())
    };
    Expansion { value, tail }
}

/// Expand `$VAR` and `~` in a path, into at most `dstlen - 1` bytes.
///
/// `esc` backslash-escapes whitespace inside an expanded value, which
/// `:e ~/tt` needs when `$HOME` has a space in it. `one` treats `src` as a
/// single file name, so `~` after a space is *not* the start of a new name.
/// `prefix` restarts name recognition after each occurrence of it.
///
/// Whenever a name cannot be expanded, or its value does not fit, it is
/// copied through as it was.
pub fn expand_env_bounded(
    src: &[u8],
    dstlen: usize,
    esc: bool,
    one: bool,
    prefix: Option<&[u8]>,
) -> Vec<u8> {
    let mut out = Vec::new();
    // Leave one byte spare for a "\," escape.
    let Some(mut room) = dstlen.checked_sub(1) else {
        return out;
    };
    let mut i = src
        .iter()
        .position(|&b| b != b' ' && b != b'\t')
        .unwrap_or(src.len());
    // At the start of a name, which is what makes a `~` here mean the home
    // directory.
    let mut at_start = true;

    while i < src.len() && room > 0 {
        // Skip over `=expr`, which is not a path at all.
        if src[i] == b'`' && byte_at(src, i + 1) == b'=' {
            let start = i;
            i += 2;
            i += skip_expr(&src[i..]);
            if byte_at(src, i) == b'`' {
                i += 1;
            }
            let len = (i - start).min(room);
            out.extend_from_slice(&src[start..start + len]);
            room -= len;
            continue;
        }

        let mut copy_char = true;
        if src[i] == b'$' || (src[i] == b'~' && at_start) {
            let next = byte_at(src, i + 1);
            let Expansion {
                mut value,
                mut tail,
            } = if src[i] != b'~' {
                resolve_env_var(src, i, room)
            } else if next == 0 || vim_ispathsep(c_int::from(next)) || b" ,\t\n".contains(&next) {
                // The home directory itself.
                Expansion {
                    value: with_home_dir(|home| home.map(<[u8]>::to_vec)),
                    tail: i + 1,
                }
            } else {
                resolve_user_dir(src, i, room)
            };

            // Whitespace inside the value has to be escaped, or the
            // caller's ' '-separated list would gain an entry.
            if esc
                && let Some(text) = &value
                && text.iter().any(|&b| b == b' ' || b == b'\t')
            {
                value = Some(escaped_bytes(text, b" \t").to_vec());
            }

            if let Some(text) = value.filter(|text| !text.is_empty()) {
                let c = text.len();
                if c + (src.len() - tail.min(src.len())) + 1 < room {
                    out.extend_from_slice(&text);
                    room -= c;
                    // If the value ends in a path separator and the tail
                    // starts with one, drop one of them.
                    if after_pathsep(&text, c) && vim_ispathsep(c_int::from(byte_at(src, tail))) {
                        tail += 1;
                    }
                    i = tail;
                    copy_char = false;
                }
            }
        }

        if copy_char {
            // Copy at least one byte, and work out whether the *next* one
            // starts a new name -- which is what makes a `~` there mean the
            // home directory. Not when `one` is set, so `:edit foo ~ foo`
            // leaves the middle one alone.
            at_start = false;
            if src[i] == b'\\' && byte_at(src, i + 1) != 0 {
                out.push(src[i]);
                i += 1;
                room -= 1;
            } else if (src[i] == b' ' || src[i] == b',') && !one {
                at_start = true;
            }
            if room > 0 {
                out.push(src[i]);
                i += 1;
                room -= 1;
                if let Some(prefix) = prefix
                    && i >= prefix.len()
                    && src[i - prefix.len()..i] == *prefix
                {
                    at_start = true;
                }
            }
        }
    }
    out
}
