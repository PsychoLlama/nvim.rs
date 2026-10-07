//! Scanning a variable, function or option name out of an expression.
//!
//! Three character classes decide where a name ends and they are not the
//! same: `eval_isnamec1` is what may *start* one, `eval_isnamec` what may
//! continue it (which includes `:` and `#`), and `eval_isdictc` what a
//! `.key` may contain (which includes neither).
//!
//! The scanners read a slice and answer lengths and offsets. The end of the
//! slice reads as the terminator the C strings had, so a scan over the rest
//! of a line stops exactly where the pointer form stopped.

#![forbid(unsafe_code)]

use crate::ascii::ascii_isdigit;
use crate::charset::vim_is_ident_char;
use crate::cstr::byte_at;
use crate::eval::typval::PartialRef;
use crate::eval::userfunc::fname_script_len;
use crate::eval::vars::is_lua_partial;
use crate::eval::{
    AUTOLOAD_CHAR, Cursor, FNE_CHECK_START, FNE_INCL_BR, char_len_at, eval_to_string,
    namespace_char,
};
use crate::keycodes::{K_SPECIAL, KE_SNR, KS_EXTRA};
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::option::option_end;
use crate::semsg;
use crate::strings::has_char;
use crate::types::{NUL, OptIndex, OptionSetFlags, TypVal};
use core::ffi::c_int;

/// The NUL byte, as the scans below compare it.
const END: u8 = NUL as u8;

/// How long the environment-variable name `text` starts with is. Zero when
/// there is none.
pub(crate) fn env_name_len(text: &[u8]) -> usize {
    text.iter()
        .take_while(|&&b| vim_is_ident_char(c_int::from(b)))
        .count()
}

/// How long the plain identifier `text` starts with is. Zero when there is
/// none.
///
/// A `:` is part of the name only as a leading namespace letter; anywhere
/// else it ends it.
pub(crate) fn id_len(text: &[u8]) -> usize {
    for (p, &c) in text.iter().enumerate() {
        let scope_ends = || p > 1 || (p == 1 && !has_char(namespace_char, c_int::from(text[0])));
        if !eval_isnamec(c_int::from(c)) || (c == b':' && scope_ends()) {
            return p;
        }
    }
    text.len()
}

/// The identifier at the cursor, which is left on the first non-blank after
/// it: its length, zero (and the cursor unmoved) when there is none.
fn take_id(cursor: &mut Cursor<'_>) -> usize {
    let len = id_len(cursor.rest());
    if len > 0 {
        cursor.bump(len);
        cursor.skip_white();
    }
    len
}

/// A name's length as the callers count it, which a name never overflows.
fn name_len(len: usize) -> c_int {
    c_int::try_from(len).unwrap_or(c_int::MAX)
}

/// The length of the name at the cursor, expanding a `{...}` in it. The
/// cursor is left on the first non-blank after the name.
///
/// When the name held curly braces and `evaluate` is set, the expanded
/// spelling comes back as well and the answer is *its* length rather than
/// the source text's. -1 means the expansion failed.
pub(crate) fn get_name_len(
    cursor: &mut Cursor<'_>,
    evaluate: bool,
    verbose: bool,
) -> (c_int, Option<XString>) {
    // A `<SNR>` prefix arrives as the three-byte key encoding.
    let snr = [K_SPECIAL, KS_EXTRA, KE_SNR as c_int];
    if (0..3).all(|i| c_int::from(cursor.at(i)) == snr[i]) {
        cursor.bump(3);
        return (name_len(take_id(cursor) + 3), None);
    }

    // `s:` and `<SID>` are a prefix on top of the name proper.
    let prefix = fname_script_len(cursor.rest());
    cursor.bump(prefix);
    let flags = if prefix > 0 { 0 } else { FNE_CHECK_START };
    // A name with curly braces in it is an identifier run up to a `{`, so
    // one that does not stop at one has none: the plain identifier is the
    // whole name, and the brace-aware scan has nothing to add.
    let rest = cursor.rest();
    let id = id_len(rest);
    if byte_at(rest, id) != b'{' {
        if id > 0 {
            cursor.bump(id);
            cursor.skip_white();
        }
        return plain_name(cursor, prefix + id, verbose);
    }
    let found = name_end(rest, flags);

    if let Some(open) = found.brace_open {
        if evaluate {
            // The prefix is part of the name being expanded.
            let start = cursor.offset() - prefix;
            let name = &cursor.text()[start..cursor.offset() + found.end];
            let close = found.brace_close.map(|close| prefix + close);
            let Some(expanded) = expanded_name(name, prefix + open, close) else {
                return (-1, None);
            };
            cursor.bump(found.end);
            cursor.skip_white();
            return (name_len(expanded.len()), Some(expanded));
        }
        cursor.bump(found.end);
        cursor.skip_white();
        return (name_len(prefix + found.end), None);
    }

    let len = prefix + take_id(cursor);
    plain_name(cursor, len, verbose)
}

/// [`get_name_len`]'s answer for a name without curly braces, `len` bytes
/// long and already stepped over: an empty one is reported when `verbose`.
fn plain_name(cursor: &Cursor<'_>, len: usize, verbose: bool) -> (c_int, Option<XString>) {
    if len == 0 && verbose && cursor.byte() != END {
        let rest = msg_bytes(cursor.rest());
        semsg!("E15: Invalid expression: \"{rest}\"");
    }
    (name_len(len), None)
}

/// Where a name ends, and where its outermost pair of curly braces is.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct NameEnd {
    /// The offset of the first byte after the name.
    pub(crate) end: usize,
    /// The first `{`, when the name has one.
    pub(crate) brace_open: Option<usize>,
    /// The `}` that closes the outermost pair, when it was reached.
    pub(crate) brace_close: Option<usize>,
}

/// The end of the name `text` starts with, stepping over `{...}` and, with
/// `FNE_INCL_BR`, over `[...]` and `.key` subscripts too. With
/// `FNE_CHECK_START`, `text` must start with a name character or a `{`.
pub(crate) fn name_end(text: &[u8], flags: c_int) -> NameEnd {
    let at = |i: usize| byte_at(text, i);
    let mut found = NameEnd::default();
    let first = at(0);
    if flags & FNE_CHECK_START != 0 && !eval_isnamec1(c_int::from(first)) && first != b'{' {
        return found;
    }

    let incl_br = flags & FNE_INCL_BR != 0;
    let mut mb_nest = 0;
    let mut br_nest = 0;
    let mut p = 0;
    loop {
        // The byte under the cursor, read once per turn. The two
        // string-skipping arms below leave `p` on the closing quote, which
        // is the byte this already holds, so the bracket and brace tests
        // further down may use it rather than reading again.
        let c = match text.get(p) {
            Some(&c) if c != END => c,
            _ => break,
        };
        // Only a `.` under `FNE_INCL_BR` looks ahead.
        let dict_key = || c == b'.' && eval_isdictc(c_int::from(at(p + 1)));
        let in_name = eval_isnamec(c_int::from(c))
            || c == b'{'
            || (incl_br && (c == b'[' || dict_key()))
            || mb_nest != 0
            || br_nest != 0;
        if !in_name {
            break;
        }

        if c == b'\'' || c == b'"' {
            // A string inside `[...]`; a double-quoted one's escapes are
            // stepped over.
            p += 1;
            while at(p) != END && at(p) != c {
                if c == b'"' && at(p) == b'\\' && at(p + 1) != END {
                    p += 1;
                }
                p += char_len_at(text, p);
            }
            if at(p) == END {
                break;
            }
        } else if br_nest == 0 && mb_nest == 0 && c == b':' {
            // A `:` ends the name unless it is the namespace one — or
            // unless a `}` came just before it, which is a curly-braces
            // name that produced the scope letter itself.
            let not_after_brace = p > 1 && text[p - 1] != b'}';
            let scoped = !has_char(namespace_char, c_int::from(text[0]));
            if not_after_brace || (p == 1 && scoped) {
                break;
            }
        }

        if mb_nest == 0 {
            if c == b'[' {
                br_nest += 1;
            } else if c == b']' {
                br_nest -= 1;
            }
        }
        if br_nest == 0 {
            if c == b'{' {
                mb_nest += 1;
                if found.brace_open.is_none() {
                    found.brace_open = Some(p);
                }
            } else if c == b'}' {
                mb_nest -= 1;
                if mb_nest == 0 && found.brace_close.is_none() {
                    found.brace_close = Some(p);
                }
            }
        }
        p += char_len_at(text, p);
    }
    found.end = p;
    found
}

/// The name `name` spells once the `{expr}` from `open` to `close` is
/// evaluated and put in its place, or `None` when there is no closing brace
/// or the expression failed. The result is re-scanned, so nested curly
/// braces expand too.
pub(crate) fn expanded_name(name: &[u8], open: usize, close: Option<usize>) -> Option<XString> {
    let close = close?;
    let value = eval_to_string(&name[open + 1..close], false, false)?;
    let mut expanded = Vec::with_capacity(open + value.len() + name.len() - close);
    expanded.extend_from_slice(&name[..open]);
    expanded.extend_from_slice(&value);
    expanded.extend_from_slice(&name[close + 1..]);

    // The expansion may itself hold curly braces.
    let inner = name_end(&expanded, 0);
    match inner.brace_open {
        Some(open) => expanded_name(&expanded[..inner.end], open, inner.brace_close),
        None => Some(XString::from_bytes(&expanded)),
    }
}

/// An ASCII letter, tested on the code point rather than on a byte: the
/// callers pass a `c_char` widened to `c_int`, so a multibyte lead byte
/// arrives negative and must not match.
#[inline(always)]
fn is_alpha(c: c_int) -> bool {
    (c >= b'A' as c_int && c <= b'Z' as c_int) || (c >= b'a' as c_int && c <= b'z' as c_int)
}

/// May this character be part of a variable name?
pub fn eval_isnamec(c: c_int) -> bool {
    is_alpha(c)
        || ascii_isdigit(c)
        || c == b'_' as c_int
        || c == b':' as c_int
        || c == AUTOLOAD_CHAR
}

/// May this character *start* a variable name?
pub fn eval_isnamec1(c: c_int) -> bool {
    is_alpha(c) || c == b'_' as c_int
}

/// May this character be part of a `.key` subscript? Unlike a variable
/// name, no `:` and no `#`.
pub fn eval_isdictc(c: c_int) -> bool {
    is_alpha(c) || ascii_isdigit(c) || c == b'_' as c_int
}

/// Is this partial the one `v:lua` stands for? An identity test: the
/// handle's address against the one `v:lua` holds.
pub(crate) fn is_luafunc(partial: Option<&PartialRef>) -> bool {
    partial.is_some_and(|partial| is_lua_partial(partial.as_ptr().addr()))
}

/// Is this typval `v:lua`?
pub(crate) fn tv_is_luafunc(tv: &TypVal) -> bool {
    is_luafunc(tv.partial_shared())
}

/// The end of the `v:lua.` function name `text` starts with, which may hold
/// `.`, `-` and `'` as well as the usual name characters.
pub(crate) fn luafunc_name_end(text: &[u8]) -> usize {
    text.iter()
        .take_while(|&&b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'\''))
        .count()
}

/// The length of the `v:lua.` function name `text` starts with, or zero
/// when what follows it is not a `(` (with `paren`) or the end of `text`.
pub(crate) fn check_luafunc_name(text: &[u8], paren: bool) -> usize {
    let end = luafunc_name_end(text);
    let want = if paren { b'(' } else { END };
    if byte_at(text, end) == want { end } else { 0 }
}

/// An option name after its `&` or `+`, as [`option_var_end`] finds it.
#[derive(Debug)]
pub(crate) struct OptionVarName {
    /// Where the name proper starts: after the sigil and any `g:`/`l:`.
    pub(crate) start: usize,
    /// The offset of the first byte after it, `None` when there is no name.
    pub(crate) end: Option<usize>,
    /// Which option, `kOptInvalid` for none or a terminal option.
    pub(crate) index: OptIndex,
    /// Which scope the `g:`/`l:` asked for.
    pub(crate) flags: OptionSetFlags,
}

/// The option name in `text`, which starts on the `&` or `+`.
pub(crate) fn option_var_end(text: &[u8]) -> OptionVarName {
    let at = |i: usize| byte_at(text, i);
    let scope = at(1);
    let (start, flags) = match scope {
        b'g' | b'l' if at(2) == b':' => (
            3,
            if scope == b'g' {
                OptionSetFlags::GLOBAL
            } else {
                OptionSetFlags::LOCAL
            },
        ),
        _ => (1, OptionSetFlags::NONE),
    };
    let (index, len) = option_end(text.get(start..).unwrap_or_default());
    OptionVarName {
        start,
        end: len.map(|len| start + len),
        index,
        flags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_end_steps_over_braces_and_subscripts() {
        assert_eq!(name_end(b"abc def", 0).end, 3);
        let curly = name_end(b"a{b}c rest", 0);
        assert_eq!(
            (curly.end, curly.brace_open, curly.brace_close),
            (5, Some(1), Some(3))
        );
        assert_eq!(name_end(b"d['k']x y", FNE_INCL_BR).end, 7);
        assert_eq!(name_end(b"d.key+1", FNE_INCL_BR).end, 5);
        assert_eq!(name_end(b"d.key+1", 0).end, 1);
        let open = name_end(b"a{b", 0);
        assert_eq!(
            (open.end, open.brace_open, open.brace_close),
            (3, Some(1), None)
        );
        assert_eq!(name_end(b"1x", FNE_CHECK_START), NameEnd::default());
    }

    #[test]
    fn a_colon_ends_a_name_unless_it_is_a_scope() {
        assert_eq!(name_end(b"g:x:y", 0).end, 3);
        assert_eq!(name_end(b"q:x", 0).end, 1);
        assert_eq!(id_len(b"g:abc"), 5);
        assert_eq!(id_len(b"ab:c"), 2);
        assert_eq!(id_len(b"x"), 1);
        assert_eq!(id_len(b""), 0);
    }

    #[test]
    fn the_lua_scanners_stop_at_the_end() {
        assert_eq!(luafunc_name_end(b"a.b-c'd(x"), 7);
        assert_eq!(check_luafunc_name(b"f.g(", true), 3);
        assert_eq!(check_luafunc_name(b"f.g", false), 3);
        assert_eq!(check_luafunc_name(b"f.g ", false), 0);
        // `env_name_len` reads 'isident', which a lib test has not set up.
        assert_eq!(env_name_len(b""), 0);
    }
}
