//! Operands that are written out: numbers, the three string forms,
//! `&option` and `$ENV`.
//!
//! The two quoted forms are each parsed twice — once to find the closing
//! quote and report a missing one, once to fill the result — and the two
//! passes must agree on where the string ends. The double-quoted measuring
//! pass also counts how much longer the result may be than its source
//! (`extra`), which is what the "used more space than allocated" check
//! still compares against.

#![forbid(unsafe_code)]

use crate::ascii::ascii_isxdigit;
use crate::charset::{Str2NrBases, hex2nr, str2nr_in};
use crate::cstr::byte_at;
use crate::eval::typval::{tv_blob_alloc, tv_blob_set_ret};
use crate::eval::vars::{eval_one_expr_in_text, optval_as_tv};
use crate::eval::{Cursor, env_name_len, option_var_end};
use crate::keycodes::{
    FSK_IN_STRING, FSK_KEYCODE, FSK_SIMPLIFY, special_key_at, trans_special_into,
};
use crate::mbyte::{cluster_len, encode_char};
use crate::memory::XString;
use crate::memory::handoff::owned_cstr;
use crate::message::{emsg, iemsg};
use crate::message_fmt::msg_bytes;
use crate::option::{get_option_value, get_tty_option, is_option_hidden, is_tty_option};
use crate::options::kOptInvalid;
use crate::os::cshim::gettext;
use crate::os::env::{expand_env_save_opt_of, vim_getenv_owned};
use crate::semsg;
use crate::types::{Failed, Float, MB_MAXCHAR, NUL, OptVal, TypVal};
use core::ffi::c_int;
use core::ptr::null_mut;

/// The NUL byte, as the walks below compare it.
const END: u8 = NUL as u8;

/// `&option`, `&l:option`, `&g:option` or `+option`, with the cursor on the
/// `&` or the `+`. Leaves it after the option name.
///
/// A `None` result means "only say whether this names an option"; that is
/// `has("+option")`, which is also the only caller `working` is true for.
pub(crate) fn eval_option(
    cursor: &mut Cursor<'_>,
    result: Option<&mut TypVal>,
    evaluate: bool,
) -> Result<(), Failed> {
    let working = cursor.byte() == b'+'; // has("+option")
    let text = cursor.rest();
    let option = option_var_end(text);
    let Some(end) = option.end else {
        if result.is_some() {
            let name = msg_bytes(text);
            semsg!("E112: Option name missing: {name}");
        }
        return Err(Failed);
    };
    if !evaluate {
        cursor.bump(end);
        return Ok(());
    }

    // The name alone, without the sigil or the scope: what the lookup and
    // the message want.
    let name = &text[option.start..end];
    let is_tty_opt = is_tty_option(name);
    let ret = if option.index == kOptInvalid && !is_tty_opt {
        // Only report it when the result is going to be used.
        if result.is_some() {
            let name = msg_bytes(name);
            semsg!("E113: Unknown option: {name}");
        }
        Err(Failed)
    } else if let Some(result) = result {
        let value: OptVal = if is_tty_opt {
            get_tty_option(name)
        } else {
            get_option_value(option.index, option.flags)
        };
        debug_assert!(!value.is_nil());
        // The slot has never held a value, so the old bytes are not released.
        result.overwrite(optval_as_tv(value, true));
        Ok(())
    } else if working && !is_tty_opt && is_option_hidden(option.index) {
        Err(Failed)
    } else {
        Ok(())
    };
    cursor.bump(end);
    ret
}

/// How many decimal digits `text` starts with.
fn digits(text: &[u8]) -> usize {
    text.iter().take_while(|b| b.is_ascii_digit()).count()
}

/// A Number, a Float or a `0z` Blob literal, with the cursor on the first
/// digit. `want_string` suppresses the Float reading, so that `1.2` in a
/// context that wants a string is the Number 1 followed by `.2`.
pub(crate) fn eval_number(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
    want_string: bool,
) -> Result<(), Failed> {
    let text = cursor.rest();
    let at = |i: usize| byte_at(text, i);
    let mut p = 1 + digits(&text[1..]);

    // A Float is accepted only for the exact `1.2`, `1.2e3` shapes: a
    // digit either side of the dot, and nothing alphabetic or a second
    // dot after what was read.
    let mut get_float = false;
    if !want_string && at(p) == b'.' && at(p + 1).is_ascii_digit() {
        get_float = true;
        p += 2;
        p += digits(&text[p..]);
        if matches!(at(p), b'e' | b'E') {
            p += 1;
            if matches!(at(p), b'-' | b'+') {
                p += 1;
            }
            if at(p).is_ascii_digit() {
                p += 1 + digits(&text[p + 1..]);
            } else {
                get_float = false;
            }
        }
        if at(p).is_ascii_alphabetic() || at(p) == b'.' {
            get_float = false;
        }
    }

    if get_float {
        // The shape just read is the whole of what `strtod` would take: it
        // is followed by neither a digit, a dot nor a letter.
        cursor.bump(p);
        if evaluate {
            result.write_float(decimal_float(&text[..p]));
        }
    } else if at(0) == b'0' && matches!(at(1), b'z' | b'Z') {
        // The handle owns the allocation for the length of the walk: every
        // way out of it below drops what it holds.
        let mut held = evaluate.then(tv_blob_alloc);
        let mut bp = 2;
        while ascii_isxdigit(c_int::from(at(bp))) {
            if !ascii_isxdigit(c_int::from(at(bp + 1))) {
                if held.is_some() {
                    let odd = c"E973: Blob literal should have an even number of hex characters";
                    emsg(gettext(odd));
                }
                return Err(Failed);
            }
            if let Some(blob) = held.as_mut() {
                let pair = (hex2nr(c_int::from(at(bp))) << 4) + hex2nr(c_int::from(at(bp + 1)));
                blob.push(u8::try_from(pair).unwrap_or_default());
            }
            // A dot may separate byte pairs: `0z00.11.22`.
            if at(bp + 2) == b'.' && ascii_isxdigit(c_int::from(at(bp + 3))) {
                bp += 1;
            }
            bp += 2;
        }
        if held.is_some() {
            tv_blob_set_ret(result, held);
        }
        cursor.bump(bp);
    } else {
        let number = str2nr_in(text, Str2NrBases::ALL, true);
        if number.len == 0 {
            if evaluate {
                let text = msg_bytes(text);
                semsg!("E15: Invalid expression: \"{text}\"");
            }
            return Err(Failed);
        }
        cursor.bump(number.len);
        if evaluate {
            result.write_number(number.value);
        }
    }
    Ok(())
}

/// The value of a decimal Float literal `eval_number` has already shaped:
/// digits, a dot, digits, and an optional exponent. Correctly rounded, as
/// `strtod` is.
fn decimal_float(text: &[u8]) -> Float {
    core::str::from_utf8(text)
        .ok()
        .and_then(|digits| digits.parse::<Float>().ok())
        .unwrap_or_default()
}

/// A double-quoted string, with the cursor on the quote — or, when
/// `interpolate` is set, on the first character of a `$"..."` piece, which
/// ends at the closing quote or at a single `{`.
pub(crate) fn eval_string(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
    interpolate: bool,
) -> Result<(), Failed> {
    let mut out = Vec::new();
    string_body(cursor, evaluate.then_some(&mut out), interpolate)?;
    if evaluate {
        result.write_string(owned_cstr(out));
    }
    Ok(())
}

/// The two passes of [`eval_string`]: find the end and check the text, then
/// — when `out` is given — append what it stands for to `out`. The cursor
/// is left after the closing quote, or on the `{` that ends a piece.
fn string_body(
    cursor: &mut Cursor<'_>,
    out: Option<&mut Vec<u8>>,
    interpolate: bool,
) -> Result<(), Failed> {
    let text = cursor.rest();
    let at = |i: usize| byte_at(text, i);
    let off = usize::from(!interpolate);
    // How much longer the result is than the text it is read from. The
    // 1 an interpolated piece starts with is the terminator it writes;
    // a doubled brace gives a byte back. It may go negative; the sum with
    // the source length is what is used, and stays positive.
    let mut extra: isize = isize::from(interpolate);

    // Find the end of the string, skipping backslashed characters.
    let mut p = off;
    while at(p) != END && at(p) != b'"' {
        if at(p) == b'\\' && at(p + 1) != END {
            p += 1;
            if at(p) == b'<' {
                // A `\<x>` form is at least 4 characters and produces up
                // to 9 (6 for the character, 3 for a modifier): reserve
                // five extra.
                extra += 5;
                // Skip to the `>` so a `{` inside is not read as the
                // start of an interpolated expression.
                if let Some((_, _, used)) = special_key_at(&text[p..], key_flags(at(p + 1)), None) {
                    p += used - 1; // leave `p` on the `>`
                }
            }
        } else if interpolate && (at(p) == b'{' || at(p) == b'}') {
            if at(p) == b'{' && at(p + 1) != b'{' {
                break; // start of an expression
            }
            p += 1;
            if text[p - 1] == b'}' && at(p) != b'}' {
                let text = msg_bytes(text);
                semsg!("E1278: Stray '}}' without a matching '{{': {text}");
                return Err(Failed);
            }
            extra -= 1; // `{{` becomes `{`, `}}` becomes `}`
        }
        p += cluster_len(&text[p..]);
    }

    if at(p) != b'"' && !(interpolate && at(p) == b'{') {
        let text = msg_bytes(text);
        semsg!("E114: Missing quote: {text}");
        return Err(Failed);
    }
    let Some(out) = out else {
        cursor.bump(p + off);
        return Ok(());
    };

    // Copy the string, resolving the escapes. `room` is what the measuring
    // pass sized it at, terminator included.
    let room = p.saturating_add_signed(extra);
    let base = out.len();
    out.reserve(room);
    let mut p = off;
    while at(p) != END && at(p) != b'"' {
        if at(p) != b'\\' {
            if interpolate && (at(p) == b'{' || at(p) == b'}') {
                if at(p) == b'{' && at(p + 1) != b'{' {
                    break; // start of an expression
                }
                p += 1; // reduce `{{` to `{` and `}}` to `}`
            }
            p = copy_char(text, p, out);
            continue;
        }

        p += 1;
        // Every arm that handles the escape itself leaves `handled` set;
        // the rest — including `\<` that did not name a key — fall
        // through to copying the character after the backslash.
        let mut handled = true;
        match at(p) {
            b'b' | b'e' | b'f' | b'n' | b'r' | b't' => {
                out.push(match at(p) {
                    b'b' => 0x08, // BS
                    b'e' => 0x1b, // ESC
                    b'f' => 0x0c, // FF
                    b'n' => b'\n',
                    b'r' => b'\r',
                    _ => b'\t',
                });
                p += 1;
            }
            // hex `\x1`/`\x12`, Unicode `#`/`\U0001f600`. With no
            // hex digit after it the letter itself is copied, by the
            // next pass of the loop rather than here.
            b'X' | b'x' | b'u' | b'U' => {
                if ascii_isxdigit(c_int::from(at(p + 1))) {
                    let byte = at(p).eq_ignore_ascii_case(&b'x');
                    let mut n = if byte {
                        2
                    } else if at(p) == b'u' {
                        4
                    } else {
                        8
                    };
                    let mut nr: c_int = 0;
                    while n > 0 && ascii_isxdigit(c_int::from(at(p + 1))) {
                        n -= 1;
                        p += 1;
                        nr = (nr << 4) + hex2nr(c_int::from(at(p)));
                    }
                    p += 1;
                    // `\u` stores the character in the current encoding;
                    // `\x` stores the byte.
                    if byte {
                        out.push(nr.to_le_bytes()[0]);
                    } else {
                        let mut encoded = [0u8; MB_MAXCHAR];
                        let len = encode_char(nr, &mut encoded);
                        out.extend_from_slice(&encoded[..len]);
                    }
                }
            }
            // octal `\1`, `\12`, `\123`, kept to its low byte
            b'0'..=b'7' => {
                let mut value = u32::from(at(p) - b'0');
                p += 1;
                for _ in 0..2 {
                    if !(b'0'..=b'7').contains(&at(p)) {
                        break;
                    }
                    value = (value << 3) + u32::from(at(p) - b'0');
                    p += 1;
                }
                out.push(value.to_le_bytes()[0]);
            }
            // a special key, e.g. `\<C-W>`
            b'<' => match trans_special_into(&text[p..], key_flags(at(p + 1)), false, out) {
                Some(used) => {
                    p += used;
                    if out.len() - base >= room {
                        iemsg(c"eval_string() used more space than allocated");
                    }
                }
                None => handled = false,
            },
            _ => handled = false,
        }
        if !handled {
            p = copy_char(text, p, out);
        }
    }

    if at(p) == b'"' && !interpolate {
        p += 1;
    }
    cursor.bump(p);
    Ok(())
}

/// The `find_special_key` flags for `\<...>` in a string: a key code, in a
/// string, and simplified unless the name opens `<*`.
fn key_flags(after_lt: u8) -> c_int {
    let flags = FSK_KEYCODE | FSK_IN_STRING;
    if after_lt == b'*' {
        flags
    } else {
        flags | FSK_SIMPLIFY
    }
}

/// Copy the character (with its composing marks) at `text[p]` to `out` and
/// answer the offset after it — nothing at the end of `text`.
fn copy_char(text: &[u8], p: usize, out: &mut Vec<u8>) -> usize {
    let rest = text.get(p..).unwrap_or_default();
    let len = cluster_len(rest);
    out.extend_from_slice(&rest[..len]);
    p + len
}

/// A single-quoted string, in which the only escape is a doubled quote —
/// or, when `interpolate` is set, a `$'...'` piece, which also reduces a
/// doubled brace and stops at a single `{`.
pub(crate) fn eval_lit_string(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
    interpolate: bool,
) -> Result<(), Failed> {
    let mut out = Vec::new();
    lit_string_body(cursor, evaluate.then_some(&mut out), interpolate)?;
    if evaluate {
        result.write_string(owned_cstr(out));
    }
    Ok(())
}

/// The two passes of [`eval_lit_string`], as [`string_body`] is for
/// [`eval_string`].
fn lit_string_body(
    cursor: &mut Cursor<'_>,
    out: Option<&mut Vec<u8>>,
    interpolate: bool,
) -> Result<(), Failed> {
    let text = cursor.rest();
    let at = |i: usize| byte_at(text, i);
    let off = usize::from(!interpolate);

    // Find the end of the string, skipping `''`.
    let mut p = off;
    while at(p) != END {
        if at(p) == b'\'' {
            if at(p + 1) != b'\'' {
                break;
            }
            p += 1;
        } else if interpolate {
            if at(p) == b'{' {
                if at(p + 1) != b'{' {
                    break; // start of an expression
                }
                p += 1;
            } else if at(p) == b'}' {
                p += 1;
                if at(p) != b'}' {
                    let text = msg_bytes(text);
                    semsg!("E1278: Stray '}}' without a matching '{{': {text}");
                    return Err(Failed);
                }
            }
        }
        p += cluster_len(&text[p..]);
    }

    if at(p) != b'\'' && !(interpolate && at(p) == b'{') {
        let text = msg_bytes(text);
        semsg!("E115: Missing quote: {text}");
        return Err(Failed);
    }
    let Some(out) = out else {
        cursor.bump(p + off);
        return Ok(());
    };

    out.reserve(p);
    let mut p = off;
    while at(p) != END {
        if at(p) == b'\'' {
            if at(p + 1) != b'\'' {
                break;
            }
            p += 1;
        } else if interpolate && (at(p) == b'{' || at(p) == b'}') {
            if at(p) == b'{' && at(p + 1) != b'{' {
                break; // start of an expression
            }
            p += 1;
        }
        p = copy_char(text, p, out);
    }
    cursor.bump(p + off);
    Ok(())
}

/// `$"..."` or `$'...'`, with the cursor on the `$`: alternating literal
/// pieces and `{expr}` substitutions, joined into one String.
///
/// Answers `Ok` even for a piece that failed — upstream's; `result` then
/// holds whatever was assembled before the error, which may be null.
pub(crate) fn eval_interp_string(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<(), Failed> {
    let mut text = Vec::<u8>::new();

    // The cursor is on the `$`; move it to the first string character.
    cursor.bump(1);
    let quote = cursor.byte();
    cursor.bump(1);

    let ret = loop {
        // The piece up to the matching quote or to a single `{`; the
        // cursor is left on whichever it was.
        let base = text.len();
        let out = evaluate.then_some(&mut text);
        let piece = if quote == b'"' {
            string_body(cursor, out, true)
        } else {
            lit_string_body(cursor, out, true)
        };
        if piece.is_err() {
            break piece;
        }
        // A piece joins the result as a C string would: up to a NUL that
        // `\x00` put in it.
        if let Some(nul) = text[base..].iter().position(|&byte| byte == END) {
            text.truncate(base + nul);
        }
        if cursor.byte() != b'{' {
            // Found the terminating quote.
            cursor.bump(1);
            break Ok(());
        }
        match eval_one_expr_in_text(cursor.rest(), &mut text, evaluate) {
            Some(used) => cursor.bump(used),
            None => break Err(Failed),
        }
    };

    // A skipped run, or an error before the first piece, answers null; an
    // evaluated run answers its text even when it is empty.
    result.write_string(if !text.is_empty() || (ret.is_ok() && evaluate) {
        owned_cstr(text)
    } else {
        null_mut()
    });
    Ok(())
}

/// `$NAME`, with the cursor on the `$`.
pub(crate) fn eval_env_var(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<(), Failed> {
    cursor.bump(1);
    let name = &cursor.rest()[..env_name_len(cursor.rest())];
    cursor.bump(name.len());
    if !evaluate {
        return Ok(());
    }
    if name.is_empty() {
        return Err(Failed);
    }

    let value = vim_getenv_owned(XString::from_bytes(name).as_cstr());
    let value = value.filter(|value| !value.is_empty());
    let value = value.or_else(|| {
        // Not in the environment: let `expand_env` have it, which knows
        // the names nvim answers itself. A result that still starts with
        // `$` is the name coming back unexpanded.
        let mut spelled = XString::with_capacity(name.len() + 1);
        spelled.push_byte(b'$');
        spelled.push_bytes(name);
        let expanded = expand_env_save_opt_of(spelled.as_cstr(), false);
        (expanded.first() != Some(&b'$')).then_some(expanded)
    });
    result.write_string(value.map_or(null_mut(), XString::into_raw));
    Ok(())
}
