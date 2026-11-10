//! Decoding a single- or double-quoted string literal into the AST node that
//! holds it, and logging the highlighting of every escape it contains.
//!
//! The decoders work on the literal's body as a byte slice and build the
//! node's `Vec`; the C wrote straight into an `xmalloc`ed buffer through a
//! `char *` cursor, after a first pass that measured how big it had to be.
//! That measuring pass survives for the double-quoted form alone, and only
//! because the C makes a *decision* on it: an estimate of zero means the node
//! gets no buffer at all, whatever the decoder would go on to produce.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::keycodes::{FSK_IN_STRING, trans_special_into};
use core::ffi::{CStr, c_int};

use super::*;
use crate::mbyte::MB_MAXCHAR;
use crate::mbyte::{cluster_len, encode_char};

#[inline(always)]
pub(super) fn shifted_pos(pos: ParserPosition, shift: size_t) -> ParserPosition {
    ParserPosition {
        line: pos.line,
        col: pos.col.wrapping_add(shift),
    }
}

#[inline(always)]
pub(super) fn recol_pos(pos: ParserPosition, new_col: size_t) -> ParserPosition {
    ParserPosition {
        line: pos.line,
        col: new_col,
    }
}

/// Which stretch of a quoted string a highlight group names.
#[derive(Clone, Copy, Debug)]
enum Part {
    /// The quotes themselves.
    Quote,
    /// The stretches between the escapes.
    Body,
    /// An escape the decoder understood.
    Escape,
    /// An escape the decoder did not understand.
    UnknownEscape,
}

/// The highlight group naming `part` of a string in double (`is_double`) or
/// single quotes, `is_invalid` when the string is not well formed.
fn group(part: Part, is_double: bool, is_invalid: bool) -> &'static CStr {
    // By part, then: invalid double, double, invalid single, single. A
    // single-quoted string's only escape is `''`, hence its `Quote` names.
    const GROUPS: [[&CStr; 4]; 4] = [
        [
            c"NvimInvalidDoubleQuote",
            c"NvimDoubleQuote",
            c"NvimInvalidSingleQuote",
            c"NvimSingleQuote",
        ],
        [
            c"NvimInvalidDoubleQuotedBody",
            c"NvimDoubleQuotedBody",
            c"NvimInvalidSingleQuotedBody",
            c"NvimSingleQuotedBody",
        ],
        [
            c"NvimInvalidDoubleQuotedEscape",
            c"NvimDoubleQuotedEscape",
            c"NvimInvalidSingleQuotedQuote",
            c"NvimSingleQuotedQuote",
        ],
        [
            c"NvimInvalidDoubleQuotedUnknownEscape",
            c"NvimDoubleQuotedUnknownEscape",
            c"NvimInvalidSingleQuotedUnknownEscape",
            c"NvimSingleQuotedUnknownEscape",
        ],
    ];
    GROUPS[part as usize][2 * usize::from(!is_double) + usize::from(!is_invalid)]
}

/// Append the whole character at the start of `rest` — composing marks
/// included — and answer how many bytes it took. This is `mb_copy_char`.
///
/// The C measures with `utfc_ptr2len`, which walks to the line's NUL and so
/// may look past the literal's closing quote; `rest` is therefore the rest of
/// the *line*, not of the literal. Nothing is appended for a NUL, exactly as
/// in the C, and the caller's outer scan is what makes progress from there.
fn copy_one_char(out: &mut Vec<uint8_t>, rest: &[uint8_t]) -> size_t {
    // `cluster_len` reads a NUL as an ordinary one-byte character, because a
    // slice's end is its own; the C stops at one, so that case is spelled out
    // here rather than left to the measurement.
    if rest.first().is_none_or(|&byte| byte == 0) {
        return 0;
    }
    let len = cluster_len(rest);
    out.extend_from_slice(&rest[..len]);
    len
}

/// Append the UTF-8 encoding of `code`.
fn append_char(out: &mut Vec<uint8_t>, code: c_int) {
    let mut buf = [0u8; MB_MAXCHAR];
    let len = encode_char(code, &mut buf);
    out.extend_from_slice(&buf[..len]);
}

/// The bytes a single-quoted literal stands for: its body with every doubled
/// `''` collapsed to one, plus the shift log the highlighter needs.
///
/// The lexer only closes such a literal at a quote that is *not* doubled, so
/// every quote the body contains is the first of a pair.
fn decode_single(body: &[uint8_t], col: size_t, colors: bool) -> (Vec<uint8_t>, Vec<StringShift>) {
    let mut out = Vec::with_capacity(body.len());
    let mut shifts = Vec::new();
    let mut i = 0;
    while i < body.len() {
        let Some(offset) = body[i..].iter().position(|&byte| byte == b'\'') else {
            out.extend_from_slice(&body[i..]);
            break;
        };
        let quote = i + offset;
        out.extend_from_slice(&body[i..=quote]);
        if colors {
            shifts.push(StringShift {
                start: col + 1 + quote,
                orig_len: 2,
                act_len: 1,
                escape_not_known: false,
            });
        }
        i = quote + 2;
    }
    (out, shifts)
}

/// How many bytes the C reserves for a double-quoted literal's value.
///
/// It is an over-estimate in three places, all deliberate: a `\<…>` reserves
/// five bytes beyond the escape's own length because computing the real one
/// would mean resolving the key twice, a `\u…` counts one digit short, and an
/// escape ending a run of octal digits is stepped over. The decoder's output
/// is what the node finally reports; this answer only decides whether there
/// is a buffer at all.
fn reserve_for_double(body: &[uint8_t]) -> size_t {
    let mut size = body.len();
    let mut i = 0;
    while i < body.len() {
        if body[i] == b'\\' && i + 1 < body.len() {
            i += 1;
            if i + 1 == body.len() {
                size = size.wrapping_sub(1);
                break;
            }
            match body[i] {
                // A `\<x>` occupies at least four bytes and produces up to
                // nine (six for the character and three for a modifier).
                b'<' => size = size.wrapping_add(5),
                // Hexadecimal: always one byte out, at least three in.
                b'x' | b'X' => {
                    size = size.wrapping_sub(1);
                    if body[i + 1].is_ascii_hexdigit() {
                        size = size.wrapping_sub(1);
                        if i + 2 < body.len() && body[i + 2].is_ascii_hexdigit() {
                            size = size.wrapping_sub(1);
                        }
                    }
                }
                // Unicode: a one-digit `\u` escape stands for one byte, two
                // fewer than it occupies; a four-digit one for three bytes,
                // three fewer; the widest `\U` for six bytes, four fewer.
                b'u' | b'U' => {
                    let esc_start = i;
                    let mut digits: size_t = if body[i] == b'u' { 4 } else { 8 };
                    let mut code: c_int = 0;
                    i += 1;
                    while i + 1 < body.len() && digits != 0 && body[i + 1].is_ascii_hexdigit() {
                        digits -= 1;
                        i += 1;
                        code = (code << 4) + hex2nr(c_int::from(body[i]));
                    }
                    // `esc_start - 1` is the backslash and `i` the byte after
                    // the last one consumed, so the escape occupies
                    // `i - (esc_start - 1)` bytes and stands for
                    // `utf_char2len` of them.
                    let occupied = (i - (esc_start - 1)).cast_signed();
                    size =
                        size.wrapping_sub((occupied - utf_char2len(code) as isize).cast_unsigned());
                    i -= 1;
                }
                // Octal: always one byte out, at least two in.
                b'0'..=b'7' => {
                    size = size.wrapping_sub(1);
                    i += 1;
                    if (b'0'..=b'7').contains(&body[i]) {
                        size = size.wrapping_sub(1);
                        i += 1;
                        if i < body.len() && (b'0'..=b'7').contains(&body[i]) {
                            size = size.wrapping_sub(1);
                            i += 1;
                        }
                    }
                }
                _ => size = size.wrapping_sub(1),
            }
        }
        i += 1;
    }
    size
}

/// The bytes a double-quoted literal stands for, plus the shift log.
///
/// `tail` is the line from the byte after the opening quote onwards, of which
/// the first `body_len` bytes are the literal's body; the two escapes that
/// fall back on `mb_copy_char` measure their character against the whole
/// line, as the C does.
fn decode_double(
    tail: &[uint8_t],
    body_len: size_t,
    col: size_t,
    colors: bool,
) -> (Vec<uint8_t>, Vec<StringShift>) {
    let body = &tail[..body_len];
    let mut out = Vec::with_capacity(body_len);
    let mut shifts = Vec::new();
    let mut i = 0;
    while i < body.len() {
        let Some(offset) = body[i..].iter().position(|&byte| byte == b'\\') else {
            out.extend_from_slice(&body[i..]);
            break;
        };
        let escape = i + offset;
        out.extend_from_slice(&body[i..escape]);
        i = escape + 1;
        if i == body.len() {
            out.push(b'\\');
            break;
        }
        let produced_from = out.len();
        let mut unknown = false;
        match body[i] {
            b'b' => {
                out.push(0x08);
                i += 1;
            }
            b'e' => {
                out.push(0x1b);
                i += 1;
            }
            b'f' => {
                out.push(0x0c);
                i += 1;
            }
            b'n' => {
                out.push(b'\n');
                i += 1;
            }
            b'r' => {
                out.push(b'\r');
                i += 1;
            }
            b't' => {
                out.push(b'\t');
                i += 1;
            }
            b'"' => {
                out.push(b'"');
                i += 1;
            }
            b'\\' => {
                out.push(b'\\');
                i += 1;
            }
            b'x' | b'X' | b'u' | b'U' => {
                if i + 1 < body.len() && body[i + 1].is_ascii_hexdigit() {
                    let is_hex = body[i] == b'x' || body[i] == b'X';
                    let mut digits: size_t = match body[i] {
                        b'x' | b'X' => 2,
                        b'u' => 4,
                        _ => 8,
                    };
                    let mut code: c_int = 0;
                    while i + 1 < body.len() && digits != 0 && body[i + 1].is_ascii_hexdigit() {
                        digits -= 1;
                        i += 1;
                        code = (code << 4) + hex2nr(c_int::from(body[i]));
                    }
                    i += 1;
                    if is_hex {
                        out.push(u8::try_from(code).expect("two hex digits"));
                    } else {
                        append_char(&mut out, code);
                    }
                } else {
                    unknown = true;
                    out.push(body[i]);
                    i += 1;
                }
            }
            // Octal: `\1`, `\12`, `\123`. The accumulator is a byte, so a
            // fourth digit's worth of value simply falls off the top.
            b'0'..=b'7' => {
                let mut byte = body[i] - b'0';
                i += 1;
                if i < body.len() && (b'0'..=b'7').contains(&body[i]) {
                    byte = (byte << 3) + (body[i] - b'0');
                    i += 1;
                    if i < body.len() && (b'0'..=b'7').contains(&body[i]) {
                        byte = (byte << 3) + (body[i] - b'0');
                        i += 1;
                    }
                }
                out.push(byte);
            }
            // A special key, e.g. `\<C-W>`.
            b'<' => {
                let mut flags = FSK_KEYCODE | FSK_IN_STRING;
                if body.get(i + 1) != Some(&b'*') {
                    flags |= FSK_SIMPLIFY;
                }
                match trans_special_into(&body[i..], flags, false, &mut out) {
                    Ok(Some(consumed)) => i += consumed,
                    Ok(None) | Err(_) => {
                        unknown = true;
                        i += copy_one_char(&mut out, &tail[i..]);
                    }
                }
            }
            _ => {
                unknown = true;
                i += copy_one_char(&mut out, &tail[i..]);
            }
        }
        if colors {
            shifts.push(StringShift {
                start: col + 1 + escape,
                orig_len: i - escape,
                act_len: out.len() - produced_from,
                escape_not_known: unknown,
            });
        }
    }
    (out, shifts)
}

/// Decode the literal `token` covers, and record how every byte of it is
/// highlighted.
///
/// `token` must be a string token `pstate` produced.
pub(super) fn parse_quoted_string(
    pstate: &mut ParserState<'_>,
    token: LexExprToken,
    is_invalid: bool,
) -> ExprNodeStr {
    let closed = size_t::from(token.string_is_closed());
    let line = pstate
        .line(token.start.line)
        .expect("a token comes from a line of the input");
    let colors = pstate.highlight_count().is_some();
    let mut highlight = |start: ParserPosition, len: size_t, group: &'static CStr| {
        pstate.highlight(start, len, group);
    };

    let is_double = token.type_0 == EXPR_LEX_DOUBLE_QUOTED_STRING;
    // Everything between the quotes: the token less its opening quote and,
    // when the lexer found one, its closing quote.
    let body_len = token.len - closed - 1;
    let tail = &line[token.start.col + 1..];

    highlight(token.start, 1, group(Part::Quote, is_double, is_invalid));

    // `capacity` is what the C allocates and zero means it allocates nothing:
    // for a single-quoted literal that is exactly the decoded length, for a
    // double-quoted one the reserve measured up front, which the decision to
    // skip the decode entirely also rests on.
    let (value, shifts, capacity) = if is_double {
        let reserved = reserve_for_double(&tail[..body_len]);
        if reserved == 0 {
            (Vec::new(), Vec::new(), 0)
        } else {
            let (value, shifts) = decode_double(tail, body_len, token.start.col, colors);
            (value, shifts, reserved)
        }
    } else {
        let (value, shifts) = decode_single(&tail[..body_len], token.start.col, colors);
        let capacity = value.len();
        (value, shifts, capacity)
    };
    let literal = ExprNodeStr {
        value: (capacity != 0).then_some(value),
    };

    if colors {
        let body = group(Part::Body, is_double, is_invalid);
        let escape = group(Part::Escape, is_double, is_invalid);
        let unknown_escape = group(Part::UnknownEscape, is_double, is_invalid);
        let mut next_col = token.start.col + 1;
        for shift in &shifts {
            if shift.start > next_col {
                highlight(
                    recol_pos(token.start, next_col),
                    shift.start - next_col,
                    body,
                );
            }
            highlight(
                recol_pos(token.start, shift.start),
                shift.orig_len,
                if shift.escape_not_known {
                    unknown_escape
                } else {
                    escape
                },
            );
            next_col = shift.start + shift.orig_len;
        }
        if next_col - token.start.col < token.len - closed {
            highlight(
                recol_pos(token.start, next_col),
                token.start.col + token.len - closed - next_col,
                body,
            );
        }
    }

    if closed != 0 {
        highlight(
            shifted_pos(token.start, token.len - 1),
            1,
            group(Part::Quote, is_double, is_invalid),
        );
    }
    literal
}
