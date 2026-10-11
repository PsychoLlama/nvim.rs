//! Escaping and quoting.
//!
//! [`escaped_with`] is the general one -- a copy with a chosen escape
//! character inserted before every byte in a set, optionally including the
//! backslashes `rem_backslash()` would eat -- and [`shellescape`] the
//! specialised one, which wraps a string in single quotes for `system()` and
//! knows what csh and fish need escaped on top of that. [`unquoted`] is the
//! inverse used by `shell_build_argv`: it resolves shell double-quoting,
//! [`unquote`] being the byte walk it shares with its tests.
//!
//! Every input is the bytes of a C string: each walk reads up to the end of
//! the slice or a NUL, whichever comes first, and measures characters as the
//! pointer forms did (`c_cluster_len`), so a sequence the end cuts short is
//! an invalid byte rather than one still to come.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::find_char;
use crate::charset::escapes_next;
use crate::ex_docmd::cmdline_var_at;
use crate::mbyte::c_cluster_len;
use crate::memory::XString;
use crate::option::{csh_like_shell, fish_like_shell};

/// The bytes of `text` up to its first NUL.
fn until_nul(text: &[u8]) -> &[u8] {
    &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())]
}

/// Walk `src` as `vim_strnsave_unquoted` reads it, feeding kept bytes to
/// `emit`: unescaped double quotes toggle quote mode and vanish; inside
/// quotes `\\` and `\"` collapse to the escaped byte; everything else
/// (including other backslash sequences) passes through untouched.
pub(crate) fn unquote(src: &[u8], emit: &mut impl FnMut(u8)) {
    let mut inquote = false;
    let mut i = 0;
    while i < src.len() {
        let b = src[i];
        if b == b'"' {
            inquote = !inquote;
        } else if b == b'\\' && inquote && i + 1 < src.len() && matches!(src[i + 1], b'\\' | b'"') {
            i += 1;
            emit(src[i]);
        } else {
            emit(b);
        }
        i += 1;
    }
}

/// `src` with shell-style double-quoting resolved (see [`unquote`]).
///
/// Every byte of `src` is read, a NUL included: the C was handed a length.
pub fn unquoted(src: &[u8]) -> XString {
    let mut out = Vec::with_capacity(src.len());
    unquote(src, &mut |b| out.push(b));
    // A NUL among the bytes ends the string, as it did the C's copy.
    out.truncate(until_nul(&out).len());
    XString::from_vec(out)
}

/// A copy of `text` with a backslash before every byte in `chars`,
/// multibyte characters copied whole; [`escaped_with`] with `\` and no
/// backslash rule.
pub fn escaped_bytes(text: &[u8], chars: &[u8]) -> XString {
    escaped_with(text, chars, b'\\', false)
}

/// A copy of `text`, prefixing `cc` to every byte in `chars` (and, with
/// `bsl`, to the backslashes [`escapes_next`] flags). A multibyte character
/// is copied whole and never escaped.
///
/// A byte is a member of `chars` the way `vim_strchr` answers it: as a
/// codepoint, so a lone `0xE9` byte matches an `é` in the set.
pub fn escaped_with(text: &[u8], chars: &[u8], cc: u8, bsl: bool) -> XString {
    let (text, chars) = (until_nul(text), until_nul(chars));
    let mut out = Vec::with_capacity(text.len() + text.len() / 4 + 1);
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        let len = c_cluster_len(rest);
        if len > 1 {
            out.extend_from_slice(&rest[..len]);
            at += len;
            continue;
        }
        if find_char(chars, c_int::from(rest[0])).is_some() || (bsl && escapes_next(rest)) {
            out.push(cc);
        }
        out.push(rest[0]);
        at += 1;
    }
    XString::from_vec(out)
}

/// Single-quote `text` for the shell, doubling embedded quotes
/// (`'` → `'\''`) and -- depending on the shell flavor and flags --
/// escaping newlines, `!`, `\`, and `%`/`#` cmdline specials.
pub fn shellescape(text: &[u8], do_special: bool, do_newline: bool) -> XString {
    let text = until_nul(text);
    let csh_like = csh_like_shell();
    let fish_like = fish_like_shell();
    let mut out = Vec::with_capacity(text.len() + 3);
    out.push(b'\'');
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        let b = rest[0];
        if b == b'\'' {
            // A single-quoted string cannot contain a quote: close it,
            // emit an escaped quote, and reopen.
            out.extend_from_slice(b"'\\''");
            at += 1;
            continue;
        }
        if (b == b'\n' && (csh_like || do_newline)) || (b == b'!' && (csh_like || do_special)) {
            out.push(b'\\');
            if csh_like && do_special {
                out.push(b'\\');
            }
            out.push(b);
            at += 1;
            continue;
        }
        if do_special && let Some((_, len)) = cmdline_var_at(rest) {
            out.push(b'\\'); // insert backslash
            out.extend_from_slice(&rest[..len]); // copy the var
            at += len;
            continue;
        }
        if b == b'\\' && fish_like {
            out.extend_from_slice(b"\\\\");
            at += 1;
            continue;
        }
        // One whole character, composing marks included.
        let len = c_cluster_len(rest);
        out.extend_from_slice(&rest[..len]);
        at += len;
    }
    out.push(b'\'');
    XString::from_vec(out)
}
