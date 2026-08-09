//! The two JSON values with a syntax of their own: strings and numbers.
//!
//! Both start at the cursor, scan forward, report their own errors and hand
//! the finished value to [`Decoder::finish_value`] themselves — so like the
//! scanner that calls them they may come back having *rewound* the cursor,
//! with [`Decoder::next_map_special`] set.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int};

use super::super::decode_string;
use super::stack::Decoder;
use super::{BS, CAR, FF, NL, NUL, TAB};
use crate::src::nvim::ascii::{ascii_isdigit, ascii_isxdigit};
use crate::src::nvim::charset::vim_str2nr;
use crate::src::nvim::eval::string2float;
use crate::src::nvim::mbyte::{utf_char2bytes, utf_char2len, utf_ptr2char, utf_ptr2len};
use crate::src::nvim::memory::xmalloc;
use crate::src::nvim::os::libc::{abort, gettext};
use crate::src::nvim::types::{
    VAR_FLOAT, VAR_NUMBER, VAR_STRING, VAR_UNLOCKED, typval_T, typval_vval_union, uvarnumber_T,
    varnumber_T,
};

const E474_UNFINISHED_ESCAPE: &CStr = c"E474: Unfinished escape sequence: %.*s";
const E474_UNFINISHED_UNICODE: &CStr = c"E474: Unfinished unicode escape sequence: %.*s";
const E474_FOUR_HEX_DIGITS: &CStr = c"E474: Expected four hex digits after \\u: %.*s";
const E474_UNKNOWN_ESCAPE: &CStr = c"E474: Unknown escape sequence: %.*s";
const E474_CONTROL_CHARS: &CStr =
    c"E474: ASCII control characters cannot be present inside string: %.*s";
const E474_ONLY_UTF8: &CStr = c"E474: Only UTF-8 strings allowed: %.*s";
const E474_ABOVE_10FFFF: &CStr =
    c"E474: Only UTF-8 code points up to U+10FFFF are allowed to appear unescaped: %.*s";
const E474_STRING_END: &CStr = c"E474: Expected string end: %.*s";
const E474_LEADING_ZEROES: &CStr = c"E474: Leading zeroes are not allowed: %.*s";
const E474_AFTER_MINUS: &CStr = c"E474: Missing number after minus sign: %.*s";
const E474_AFTER_DOT: &CStr = c"E474: Missing number after decimal dot: %.*s";
const E474_MISSING_EXPONENT: &CStr = c"E474: Missing exponent: %.*s";
const E685_FLOAT: &CStr = c"E685: internal error: while converting number \"%.*s\" to float \
                            string2float consumed %zu bytes in place of %zu";
const E685_INTEGER: &CStr = c"E685: internal error: while converting number \"%.*s\" to integer \
                              vim_str2nr consumed %i bytes in place of %zu";

/// `vim_str2nr` flags: read hexadecimal, and read it whatever the prefix says.
const STR2NR_HEX: c_int = 4;
const STR2NR_FORCE: c_int = 128;

/// The UTF-16 surrogate range, which JSON's `\u` escapes use to spell a code
/// point above U+FFFF as two escapes.
const SURROGATE_HI_START: uvarnumber_T = 0xd800;
const SURROGATE_HI_END: uvarnumber_T = 0xdbff;
const SURROGATE_LO_START: uvarnumber_T = 0xdc00;
const SURROGATE_LO_END: uvarnumber_T = 0xdfff;
const SURROGATE_FIRST_CHAR: c_int = 0x10000;

/// Scan a double-quoted JSON string and store it.
///
/// `at` points at the opening `"` and comes back on the closing one — or
/// rewound, if storing the value restarted the container.  The string is
/// measured in one pass and decoded in a second, so the output buffer is
/// allocated exactly once and handed to the typval, which then owns it.
///
/// # Safety
/// `at` indexes `dec.buf` and points at a `"`.
pub(crate) unsafe fn parse_json_string(dec: &mut Decoder, at: &mut usize) -> bool {
    unsafe {
        let buf = dec.buf;
        let e = buf.len();
        let s = *at + 1;

        // Pass one: validate, and total up what the decoded bytes will need.
        // A `\u` escape is charged three bytes, which is the most one below
        // U+10000 can take — a surrogate pair spends both escapes' budget on
        // the four bytes the pair really needs.
        let mut p = s;
        let mut len: usize = 0;
        while p < e && buf[p] != b'"' {
            if buf[p] == b'\\' {
                p += 1;
                if p == e {
                    dec.emsg_rest(E474_UNFINISHED_ESCAPE, 0);
                    return fail(at, p);
                }
                match buf[p] {
                    b'u' => {
                        if p + 4 >= e {
                            dec.emsg_rest(E474_UNFINISHED_UNICODE, 0);
                            return fail(at, p);
                        }
                        if !buf[p + 1..p + 5]
                            .iter()
                            .all(|&c| ascii_isxdigit(c_int::from(c)))
                        {
                            dec.emsg_rest(E474_FOUR_HEX_DIGITS, p - 1);
                            return fail(at, p);
                        }
                        len += 3;
                        p += 5;
                    }
                    b'\\' | b'/' | b'"' | b't' | b'b' | b'n' | b'r' | b'f' => {
                        len += 1;
                        p += 1;
                    }
                    _ => {
                        dec.emsg_rest(E474_UNKNOWN_ESCAPE, p - 1);
                        return fail(at, p);
                    }
                }
                continue;
            }

            // unescaped = %x20-21 / %x23-5B / %x5D-10FFFF
            let byte = buf[p];
            if byte < 0x20 {
                dec.emsg_rest(E474_CONTROL_CHARS, p);
                return fail(at, p);
            }
            let ch = utf_ptr2char(buf.as_ptr().add(p) as *const c_char);
            // Every code point above U+007F is two or more bytes, so it can
            // never equal the byte it starts with — except that
            // `utf_ptr2char({0xFF, 0})` answers 0xFF even though 0xFF starts
            // no sequence at all.  U+00C3 is the one real exception, spelled
            // 0xC3 0x83.
            if ch >= 0x80
                && c_int::from(byte) == ch
                && !(ch == 0xc3 && p + 1 < e && buf[p + 1] == 0x83)
            {
                dec.emsg_rest(E474_ONLY_UTF8, p);
                return fail(at, p);
            }
            if ch > 0x10ffff {
                dec.emsg_rest(E474_ABOVE_10FFFF, p);
                return fail(at, p);
            }
            let ch_len = utf_char2len(ch) as usize;
            debug_assert!(
                ch_len
                    == if ch != 0 {
                        utf_ptr2len(buf.as_ptr().add(p) as *const c_char)
                    } else {
                        1
                    } as usize
            );
            len += ch_len;
            p += ch_len;
        }
        // `p > e` is reachable only if `utf_ptr2char` read a lead byte whose
        // continuation bytes ran past the document; upstream reads the byte
        // there anyway.
        if p >= e || buf[p] != b'"' {
            dec.emsg_rest(E474_STRING_END, 0);
            return fail(at, p);
        }

        // Pass two: write the decoded bytes.  `out` is handed to the typval
        // below, which frees it with the value.
        let out = xmalloc(len + 1) as *mut u8;
        let mut w: usize = 0;
        // A `\uD800`-range escape is held back until the next escape says
        // whether it is the first half of a surrogate pair.
        let mut fst_in_pair: c_int = 0;
        // `PUT_FST_IN_PAIR`: emit a held-back high surrogate as the lone code
        // point it turned out to be.
        let flush = |w: &mut usize, fst: &mut c_int| {
            if *fst != 0 {
                *w += utf_char2bytes(*fst, out.add(*w) as *mut c_char) as usize;
                *fst = 0;
            }
        };

        let mut t = s;
        while t < p {
            if buf[t] != b'\\' || buf[t + 1] != b'u' {
                flush(&mut w, &mut fst_in_pair);
            }
            if buf[t] != b'\\' {
                *out.add(w) = buf[t];
                w += 1;
                t += 1;
                continue;
            }
            t += 1;
            if buf[t] == b'u' {
                let hex = [
                    buf[t + 1] as c_char,
                    buf[t + 2] as c_char,
                    buf[t + 3] as c_char,
                    buf[t + 4] as c_char,
                ];
                t += 4;
                let mut ch: uvarnumber_T = 0;
                vim_str2nr(
                    hex.as_ptr(),
                    ::core::ptr::null_mut(),
                    ::core::ptr::null_mut(),
                    STR2NR_HEX | STR2NR_FORCE,
                    ::core::ptr::null_mut(),
                    &raw mut ch,
                    4,
                    true,
                    ::core::ptr::null_mut(),
                );
                if (SURROGATE_HI_START..=SURROGATE_HI_END).contains(&ch) {
                    flush(&mut w, &mut fst_in_pair);
                    fst_in_pair = ch as c_int;
                } else if (SURROGATE_LO_START..=SURROGATE_LO_END).contains(&ch) && fst_in_pair != 0
                {
                    let full_char = (ch - SURROGATE_LO_START) as c_int
                        + ((fst_in_pair - SURROGATE_HI_START as c_int) << 10)
                        + SURROGATE_FIRST_CHAR;
                    w += utf_char2bytes(full_char, out.add(w) as *mut c_char) as usize;
                    fst_in_pair = 0;
                } else {
                    flush(&mut w, &mut fst_in_pair);
                    w += utf_char2bytes(ch as c_int, out.add(w) as *mut c_char) as usize;
                }
            } else {
                *out.add(w) = match buf[t] {
                    b'\\' => b'\\',
                    b'/' => b'/',
                    b'"' => b'"',
                    b't' => TAB,
                    b'b' => BS,
                    b'n' => NL,
                    b'r' => CAR,
                    b'f' => FF,
                    // Pass one accepted no other escape.
                    _ => abort(),
                };
                w += 1;
            }
            t += 1;
        }
        flush(&mut w, &mut fst_in_pair);
        *out.add(w) = NUL;

        let obj = decode_string(out as *const c_char, w, false, true);
        // A string carrying an embedded NUL came back as a blob wrapped in a
        // special dictionary, which can be a dictionary value but not a key.
        let is_special_string = obj.v_type != VAR_STRING;
        let value = dec.value(obj, is_special_string);
        let ok = dec.finish_value(value, &mut p);
        *at = p;
        ok
    }
}

/// Scan a JSON number — `-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?` — and store it.
///
/// `at` points at the leading digit or minus sign and comes back on the last
/// character of the number, so that the scanner's own `+= 1` lands past it.
///
/// # Safety
/// `at` indexes `dec.buf` and points at `-` or a digit.
pub(crate) unsafe fn parse_json_number(dec: &mut Decoder, at: &mut usize) -> bool {
    unsafe {
        let buf = dec.buf;
        let e = buf.len();
        let s = *at;
        let mut p = s;

        let mut fracs: Option<usize> = None;
        let mut exps: Option<usize> = None;
        let mut exps_s: Option<usize> = None;

        if buf[p] == b'-' {
            p += 1;
        }
        let ints = p;
        // Everything below is "scan as far as the grammar allows"; running
        // out of input is not an error here, it is what the checks below are
        // for.
        'scan: {
            if p >= e {
                break 'scan;
            }
            while p < e && ascii_isdigit(c_int::from(buf[p])) {
                p += 1;
            }
            if p != ints + 1 && buf[ints] == b'0' {
                dec.emsg_rest(E474_LEADING_ZEROES, s);
                return fail(at, p);
            }
            if p >= e || p == ints {
                break 'scan;
            }
            if buf[p] == b'.' {
                p += 1;
                fracs = Some(p);
                while p < e && ascii_isdigit(c_int::from(buf[p])) {
                    p += 1;
                }
                if p >= e || Some(p) == fracs {
                    break 'scan;
                }
            }
            if buf[p] == b'e' || buf[p] == b'E' {
                p += 1;
                exps_s = Some(p);
                if p < e && (buf[p] == b'-' || buf[p] == b'+') {
                    p += 1;
                }
                exps = Some(p);
                while p < e && ascii_isdigit(c_int::from(buf[p])) {
                    p += 1;
                }
            }
        }

        if p == ints {
            dec.emsg_rest(E474_AFTER_MINUS, s);
            return fail(at, p);
        }
        if Some(p) == fracs || (fracs.is_some() && exps_s == fracs.map(|f| f + 1)) {
            dec.emsg_rest(E474_AFTER_DOT, s);
            return fail(at, p);
        }
        if Some(p) == exps {
            dec.emsg_rest(E474_MISSING_EXPONENT, s);
            return fail(at, p);
        }

        let text = buf.as_ptr().add(s) as *const c_char;
        let want = p - s;
        let mut tv = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if fracs.is_some() || exps.is_some() {
            let got = string2float(text, &raw mut tv.vval.v_float);
            if want != got {
                semsg_c!(gettext(E685_FLOAT.as_ptr()), want as c_int, text, got, want);
            }
            tv.v_type = VAR_FLOAT;
        } else {
            let mut nr: varnumber_T = 0;
            let mut got: c_int = 0;
            vim_str2nr(
                text,
                ::core::ptr::null_mut(),
                &raw mut got,
                0,
                &raw mut nr,
                ::core::ptr::null_mut(),
                want as c_int,
                true,
                ::core::ptr::null_mut(),
            );
            if want as c_int != got {
                semsg_c!(
                    gettext(E685_INTEGER.as_ptr()),
                    want as c_int,
                    text,
                    got,
                    want,
                );
            }
            tv.vval.v_number = nr;
        }

        let value = dec.value(tv, false);
        if !dec.finish_value(value, &mut p) {
            *at = p;
            return false;
        }
        if !dec.next_map_special {
            // The scanner's loop advances past the last character itself.
            p -= 1;
        }
        *at = p;
        true
    }
}

/// The `parse_json_*_fail` tail: publish the cursor and answer failure.
///
/// Both scanners write `*pp = p` on the way out whichever exit they take, so
/// that the caller's own error reporting sees where the scan stopped.
fn fail(at: &mut usize, p: usize) -> bool {
    *at = p;
    false
}
