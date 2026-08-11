//! Decoding a single- or double-quoted string literal into the AST node that
//! holds it, and logging the highlighting of every escape it contains.
//!
//! The decoders work on the literal's body as a byte slice and build a `Vec`,
//! which is copied into the node's `xmalloc`ed buffer at the end; the C wrote
//! straight into that buffer through a `char *` cursor, after a first pass
//! that measured how big it had to be. That measuring pass survives for the
//! double-quoted form alone, and only because the C makes a *decision* on it:
//! an estimate of zero means the node gets no buffer at all, whatever the
//! decoder would go on to produce.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::{ptr, slice};

use super::*;
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::types::MB_MAXCHAR;

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

/// The group naming the quotes themselves.
fn quote_group(is_double: bool, is_invalid: bool) -> &'static CStr {
    match (is_double, is_invalid) {
        (true, true) => c"NvimInvalidDoubleQuote",
        (true, false) => c"NvimDoubleQuote",
        (false, true) => c"NvimInvalidSingleQuote",
        (false, false) => c"NvimSingleQuote",
    }
}

/// The group naming the stretches between the escapes.
fn body_group(is_double: bool, is_invalid: bool) -> &'static CStr {
    match (is_double, is_invalid) {
        (true, true) => c"NvimInvalidDoubleQuotedBody",
        (true, false) => c"NvimDoubleQuotedBody",
        (false, true) => c"NvimInvalidSingleQuotedBody",
        (false, false) => c"NvimSingleQuotedBody",
    }
}

/// The group naming an escape the decoder understood.
fn escape_group(is_double: bool, is_invalid: bool) -> &'static CStr {
    match (is_double, is_invalid) {
        (true, true) => c"NvimInvalidDoubleQuotedEscape",
        (true, false) => c"NvimDoubleQuotedEscape",
        (false, true) => c"NvimInvalidSingleQuotedQuote",
        (false, false) => c"NvimSingleQuotedQuote",
    }
}

/// The group naming an escape the decoder did not understand.
fn unknown_escape_group(is_double: bool, is_invalid: bool) -> &'static CStr {
    match (is_double, is_invalid) {
        (true, true) => c"NvimInvalidDoubleQuotedUnknownEscape",
        (true, false) => c"NvimDoubleQuotedUnknownEscape",
        (false, true) => c"NvimInvalidSingleQuotedUnknownEscape",
        (false, false) => c"NvimSingleQuotedUnknownEscape",
    }
}

/// Append the whole character at the start of `rest` — composing marks
/// included — and answer how many bytes it took. This is `mb_copy_char`.
///
/// The C measures with `utfc_ptr2len`, which walks to the line's NUL and so
/// may look past the literal's closing quote; `rest` is therefore the rest of
/// the *line*, not of the literal. Nothing is appended for a NUL, exactly as
/// in the C, and the caller's outer scan is what makes progress from there.
fn copy_one_char(out: &mut Vec<uint8_t>, rest: &[uint8_t]) -> size_t {
    // SAFETY: the parser's input lines are NUL-terminated, which is what
    // `utfc_ptr2len` needs to stop; the answer is clamped to `rest` so that a
    // character truncated by the end of the line cannot read past it.
    let len = unsafe { utfc_ptr2len(rest.as_ptr().cast::<c_char>()) };
    let len = (len as size_t).min(rest.len());
    out.extend_from_slice(&rest[..len]);
    len
}

/// Append the UTF-8 encoding of `code`.
fn append_char(out: &mut Vec<uint8_t>, code: c_int) {
    let mut buf = [0u8; MB_MAXCHAR];
    // SAFETY: `buf` is `MB_MAXCHAR` bytes, the most `utf_char2bytes` writes.
    let len = unsafe { utf_char2bytes(code, buf.as_mut_ptr().cast::<c_char>()) };
    out.extend_from_slice(&buf[..len as size_t]);
}

/// `trans_special` over a slice: how many bytes of `rest` the key name
/// occupied, the key's encoding, and how many bytes of it are meaningful.
/// Zero written means `rest` does not start with a name this understands.
fn special_key(rest: &[uint8_t], flags: c_int) -> (size_t, [uint8_t; 19], size_t) {
    // Room for one key's encoding, which is what `trans_special` asks of its
    // destination: three bytes of modifiers plus the character itself.
    let mut key = [0u8; 19];
    let start = rest.as_ptr();
    let mut cursor = start.cast::<c_char>();
    // SAFETY: `src_len` bounds the read to `rest`, and `key` is the room the
    // destination is documented to need.
    let written = unsafe {
        trans_special(
            &raw mut cursor,
            rest.len(),
            key.as_mut_ptr().cast::<c_char>(),
            flags,
            false,
            ptr::null_mut(),
        )
    };
    (cursor as size_t - start as size_t, key, written as size_t)
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
                // Unicode: `\uF` is one byte, two fewer than the escape;
                // `￿` three bytes, three fewer; `\U7FFFFFFF` six bytes,
                // four fewer.
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
                    let occupied = (i - (esc_start - 1)) as isize;
                    size = size.wrapping_sub((occupied - utf_char2len(code) as isize) as size_t);
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
                        out.push(code as uint8_t);
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
                let mut flags = FSK_KEYCODE as c_int | FSK_IN_STRING as c_int;
                if body.get(i + 1) != Some(&b'*') {
                    flags |= FSK_SIMPLIFY as c_int;
                }
                let (consumed, key, written) = special_key(&body[i..], flags);
                i += consumed;
                if written != 0 {
                    out.extend_from_slice(&key[..written]);
                } else {
                    unknown = true;
                    i += copy_one_char(&mut out, &tail[i..]);
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

/// Decode the literal `token` covers into `node`, and record how every byte
/// of it is highlighted.
///
/// # Safety
///
/// `pstate` must be the parser that produced `token`, and `node` a node of a
/// string type whose value slot is still unset.
pub(super) unsafe fn parse_quoted_string(
    pstate: *mut ParserState,
    node: *mut ExprASTNode,
    token: LexExprToken,
    is_invalid: bool,
) {
    // SAFETY: `token` came from this parser, so `start.line` indexes a line
    // the reader is still holding and `start.col + len` is inside it. Reading
    // `str.closed` is reading the variant the token's type names.
    let (line, closed, colors) = unsafe {
        let pline = *(*pstate).reader.lines.items.add(token.start.line);
        (
            slice::from_raw_parts(pline.data.cast::<uint8_t>(), pline.size),
            size_t::from(token.data.str.closed),
            !(*pstate).colors.is_null(),
        )
    };
    // SAFETY: `pstate` is the caller's, and every group name is a static
    // NUL-terminated string.
    let mut highlight = |start: ParserPosition, len: size_t, group: &'static CStr| unsafe {
        viml_parser_highlight(pstate, start, len, group.as_ptr());
    };

    let is_double = token.type_0 == kExprLexDoubleQuotedString;
    // Everything between the quotes: the token less its opening quote and,
    // when the lexer found one, its closing quote.
    let body_len = token.len - closed - 1;
    let tail = &line[token.start.col + 1..];

    highlight(token.start, 1, quote_group(is_double, is_invalid));

    // `capacity` is what the C allocates and zero means it allocates nothing:
    // for a single-quoted literal that is exactly the decoded length, for a
    // double-quoted one the reserve measured up front, which the decision to
    // skip the decode entirely also rests on. Taking the larger of the two
    // costs nothing and puts a floor under the reserve, whose over-estimates
    // are argued rather than proved.
    let (value, shifts, capacity) = if is_double {
        let reserved = reserve_for_double(&tail[..body_len]);
        if reserved == 0 {
            (Vec::new(), Vec::new(), 0)
        } else {
            let (value, shifts) = decode_double(tail, body_len, token.start.col, colors);
            let capacity = reserved.max(value.len());
            (value, shifts, capacity)
        }
    } else {
        let (value, shifts) = decode_single(&tail[..body_len], token.start.col, colors);
        let capacity = value.len();
        (value, shifts, capacity)
    };

    // The buffer comes from `xmalloc` because `viml_pexpr_free_ast` releases
    // it with `xfree`; the single-quoted form uses `xmallocz` so that its
    // value stays NUL-terminated, as the C did.
    let buffer = if capacity == 0 {
        ptr::null_mut::<c_char>()
    } else {
        // SAFETY: `capacity` is at least `value.len()`, so the copy fits.
        unsafe {
            let buffer = if is_double {
                xmalloc(capacity)
            } else {
                xmallocz(capacity)
            };
            buffer
                .cast::<uint8_t>()
                .copy_from_nonoverlapping(value.as_ptr(), value.len());
            buffer.cast::<c_char>()
        }
    };
    // SAFETY: `node` is the caller's, and its value slot is unset, so nothing
    // is leaked by writing it.
    unsafe {
        (*node).data.str.value = buffer;
        (*node).data.str.size = if buffer.is_null() { 0 } else { value.len() };
    }

    if colors {
        let body = body_group(is_double, is_invalid);
        let escape = escape_group(is_double, is_invalid);
        let unknown_escape = unknown_escape_group(is_double, is_invalid);
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
            quote_group(is_double, is_invalid),
        );
    }
}
