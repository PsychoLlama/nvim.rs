// VimL quoted string parser — Rust replacement for parse_quoted_string().
//
// Handles both single-quoted and double-quoted strings including:
// - escape sequence processing for double-quoted strings
// - highlighting via C accessor functions
// - size calculation before allocation

#![allow(clippy::too_many_lines)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::many_single_char_names)]
#![allow(dead_code)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::len_zero)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::if_not_else)]

use std::ffi::{c_char, c_int, c_void};

use crate::expr_types::{
    AstNodeDataStr, ExprASTNode, LexExprToken, LexExprTokenType, ParserPosition,
};
use crate::lexer::ParserState;

// ---------------------------------------------------------------------------
// Extern "C" functions needed for string parsing
// ---------------------------------------------------------------------------

extern "C" {
    fn nvim_viml_parser_highlight(
        pstate: *mut ParserState,
        start: ParserPosition,
        len: usize,
        group: *const c_char,
    );
    fn nvim_parser_get_line_data(
        pstate: *const ParserState,
        line_idx: usize,
        data_out: *mut *const c_char,
        size_out: *mut usize,
    );
    fn nvim_parser_get_colors(pstate: *const ParserState) -> *mut c_void;

    /// hex2nr: convert hex character to number (0-15).
    fn hex2nr(c: c_char) -> c_int;

    /// utf_char2len: get UTF-8 length of a codepoint.
    fn utf_char2len(c: c_int) -> c_int;

    /// utf_char2bytes: encode a codepoint to UTF-8 bytes, returns byte count.
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;

    /// trans_special: translate a special key escape sequence.
    fn trans_special(
        srcp: *mut *const c_char,
        src_len: usize,
        dst: *mut c_char,
        flags: c_int,
        key_code: bool,
        modifiers: *mut c_int,
    ) -> usize;

    /// mb_copy_char: copy one multibyte character from *src to *dst, advancing both.
    fn mb_copy_char(src: *mut *const c_char, dst: *mut *mut c_char);

    /// xmalloc: allocate memory (nvim's allocator).
    fn xmalloc(size: usize) -> *mut c_void;

    /// xmallocz: allocate size+1 bytes, NUL-terminate.
    fn xmallocz(size: usize) -> *mut c_void;
}

// FSK_* constants matching keycodes.h
const FSK_KEYCODE: c_int = 0x01;
const FSK_SIMPLIFY: c_int = 0x02;
const FSK_IN_STRING: c_int = 0x08;

// ASCII escape codes matching nvim's ascii_defs.h
const BS: u8 = 0x08;
const ESC: u8 = 0x1b;
const FF: u8 = 0x0c;
const NL: u8 = 0x0a;
const CAR: u8 = 0x0d;
const TAB: u8 = 0x09;

// ---------------------------------------------------------------------------
// StringShift — maps original string positions to processed positions
// ---------------------------------------------------------------------------

struct StringShift {
    start: usize,
    orig_len: usize,
    act_len: usize,
    escape_not_known: bool,
}

// ---------------------------------------------------------------------------
// HL helper
// ---------------------------------------------------------------------------

/// Construct a "NvimXxx" or "NvimInvalidXxx" highlight group name.
/// Returns a `*const c_char` to a static string literal.
macro_rules! hl_group {
    ($is_invalid:expr, $suffix:literal) => {
        if $is_invalid {
            concat!("NvimInvalid", $suffix, "\0")
                .as_ptr()
                .cast::<c_char>()
        } else {
            concat!("Nvim", $suffix, "\0").as_ptr().cast::<c_char>()
        }
    };
}

#[inline]
fn ascii_isxdigit(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

// ---------------------------------------------------------------------------
// parse_quoted_string
// ---------------------------------------------------------------------------

/// Parse and highlight a single- or double-quoted string token.
///
/// Fills `node.data.str_` with the parsed string value.
///
/// # Safety
/// All pointers must be valid. `node` must outlive this call.
pub unsafe fn parse_quoted_string(
    pstate: *mut ParserState,
    node: *mut ExprASTNode,
    token: LexExprToken,
    is_invalid: bool,
) {
    // Get the raw line data for this token's line.
    let mut line_data: *const c_char = std::ptr::null();
    let mut line_size: usize = 0;
    unsafe {
        nvim_parser_get_line_data(
            pstate.cast_const(),
            token.start.line,
            &raw mut line_data,
            &raw mut line_size,
        );
    }
    let has_colors = !unsafe { nvim_parser_get_colors(pstate.cast_const()) }.is_null();

    // s = start of token text, e = end (excluding closing quote if closed)
    let s: *const u8 = unsafe { line_data.add(token.start.col) }.cast::<u8>();
    let closed = unsafe { token.data.str_.closed };
    let is_double = token.typ == LexExprTokenType::DoubleQuotedString;
    let token_len = token.len;

    // `e` = one past last char we process (before closing quote, if any)
    let e: *const u8 = unsafe { s.add(token_len - if closed { 1 } else { 0 }) };

    let mut size: usize = token_len - if closed { 1 } else { 0 } - 1; // exclude opening quote

    let mut shifts: Vec<StringShift> = Vec::with_capacity(16);

    if !is_double {
        // Single-quoted string: only '' escape.
        unsafe {
            nvim_viml_parser_highlight(
                pstate,
                token.start,
                1,
                hl_group!(is_invalid, "SingleQuote"),
            );
        }
        let mut p: *const u8 = unsafe { s.add(1) };
        while p < e {
            // Find next single-quote pair.
            let chunk_e_opt = unsafe { memchr(p, b'\'', e.offset_from(p) as usize) };
            let chunk_e = match chunk_e_opt {
                None => break,
                Some(ptr) => ptr,
            };
            size -= 1;
            p = unsafe { chunk_e.add(2) };
            if has_colors {
                shifts.push(StringShift {
                    start: token.start.col + unsafe { chunk_e.offset_from(s) } as usize,
                    orig_len: 2,
                    act_len: 1,
                    escape_not_known: false,
                });
            }
        }

        // Build value string.
        let str_data: AstNodeDataStr = if size == 0 {
            AstNodeDataStr {
                value: std::ptr::null_mut(),
                size: 0,
            }
        } else {
            let v_raw = unsafe { xmallocz(size) };
            let mut v_p: *mut u8 = v_raw.cast::<u8>();
            let mut p2: *const u8 = unsafe { s.add(1) };
            while p2 < e {
                let chunk_e_opt = unsafe { memchr(p2, b'\'', e.offset_from(p2) as usize) };
                match chunk_e_opt {
                    None => {
                        let remaining = unsafe { e.offset_from(p2) } as usize;
                        unsafe { std::ptr::copy_nonoverlapping(p2, v_p, remaining) };
                        break;
                    }
                    Some(chunk_e) => {
                        let n = unsafe { chunk_e.offset_from(p2) } as usize;
                        unsafe { std::ptr::copy_nonoverlapping(p2, v_p, n) };
                        unsafe { v_p = v_p.add(n + 1) };
                        unsafe { *v_p.sub(1) = b'\'' };
                        p2 = unsafe { chunk_e.add(2) };
                    }
                }
            }
            AstNodeDataStr {
                value: v_raw.cast::<c_char>(),
                size,
            }
        };
        unsafe { (*node).data.str_ = str_data };
    } else {
        // Double-quoted string: handle escape sequences.
        unsafe {
            nvim_viml_parser_highlight(
                pstate,
                token.start,
                1,
                hl_group!(is_invalid, "DoubleQuote"),
            );
        }

        // First pass: compute size.
        let mut p: *const u8 = unsafe { s.add(1) };
        while p < e {
            if unsafe { *p } == b'\\' && unsafe { p.add(1) } < e {
                p = unsafe { p.add(1) };
                if unsafe { p.add(1) } == e {
                    size -= 1;
                    break;
                }
                match unsafe { *p } {
                    b'<' => {
                        size += 5;
                    }
                    b'x' | b'X' => {
                        size -= 1;
                        if p < e && ascii_isxdigit(unsafe { *p.add(1) }) {
                            size -= 1;
                            if unsafe { p.add(2) } < e && ascii_isxdigit(unsafe { *p.add(2) }) {
                                size -= 1;
                            }
                        }
                    }
                    b'u' | b'U' => {
                        let esc_start = p;
                        let n_max: usize = if unsafe { *p } == b'u' { 4 } else { 8 };
                        let mut nr: c_int = 0;
                        let mut n = n_max;
                        p = unsafe { p.add(1) };
                        while unsafe { p.add(1) } < e
                            && n > 0
                            && ascii_isxdigit(unsafe { *p.add(1) })
                        {
                            p = unsafe { p.add(1) };
                            n -= 1;
                            nr = (nr << 4) + unsafe { hex2nr(*p.cast::<c_char>()) };
                        }
                        let char_len = unsafe { utf_char2len(nr) } as usize;
                        let esc_total = unsafe { p.offset_from(esc_start.sub(1)) } as usize;
                        // size -= (esc_total - char_len)
                        if esc_total > char_len {
                            size -= esc_total - char_len;
                        } else {
                            size += char_len - esc_total;
                        }
                        p = unsafe { p.sub(1) };
                    }
                    b'0'..=b'7' => {
                        size -= 1;
                        p = unsafe { p.add(1) };
                        if unsafe { *p } >= b'0' && unsafe { *p } <= b'7' {
                            size -= 1;
                            p = unsafe { p.add(1) };
                            if p < e && unsafe { *p } >= b'0' && unsafe { *p } <= b'7' {
                                size -= 1;
                                p = unsafe { p.add(1) };
                            }
                        }
                        continue;
                    }
                    _ => {
                        size -= 1;
                    }
                }
            }
            p = unsafe { p.add(1) };
        }

        // Build value string.
        let str_data: AstNodeDataStr = if size == 0 {
            AstNodeDataStr {
                value: std::ptr::null_mut(),
                size: 0,
            }
        } else {
            let v_raw = unsafe { xmalloc(size) };
            let mut v_p: *mut u8 = v_raw.cast::<u8>();
            let mut p2: *const u8 = unsafe { s.add(1) };
            while p2 < e {
                let chunk_e_opt = unsafe { memchr(p2, b'\\', e.offset_from(p2) as usize) };
                match chunk_e_opt {
                    None => {
                        let n = unsafe { e.offset_from(p2) } as usize;
                        unsafe { std::ptr::copy_nonoverlapping(p2, v_p, n) };
                        unsafe { v_p = v_p.add(n) };
                        break;
                    }
                    Some(chunk_e) => {
                        let n = unsafe { chunk_e.offset_from(p2) } as usize;
                        unsafe { std::ptr::copy_nonoverlapping(p2, v_p, n) };
                        unsafe { v_p = v_p.add(n) };
                        p2 = unsafe { chunk_e.add(1) };
                        if p2 == e {
                            unsafe { *v_p = b'\\' };
                            unsafe { v_p = v_p.add(1) };
                            break;
                        }
                        let mut is_unknown = false;
                        let v_p_start = v_p;
                        match unsafe { *p2 } {
                            b'b' => {
                                unsafe { *v_p = BS };
                                unsafe { v_p = v_p.add(1) };
                                p2 = unsafe { p2.add(1) };
                            }
                            b'e' => {
                                unsafe { *v_p = ESC };
                                unsafe { v_p = v_p.add(1) };
                                p2 = unsafe { p2.add(1) };
                            }
                            b'f' => {
                                unsafe { *v_p = FF };
                                unsafe { v_p = v_p.add(1) };
                                p2 = unsafe { p2.add(1) };
                            }
                            b'n' => {
                                unsafe { *v_p = NL };
                                unsafe { v_p = v_p.add(1) };
                                p2 = unsafe { p2.add(1) };
                            }
                            b'r' => {
                                unsafe { *v_p = CAR };
                                unsafe { v_p = v_p.add(1) };
                                p2 = unsafe { p2.add(1) };
                            }
                            b't' => {
                                unsafe { *v_p = TAB };
                                unsafe { v_p = v_p.add(1) };
                                p2 = unsafe { p2.add(1) };
                            }
                            b'"' => {
                                unsafe { *v_p = b'"' };
                                unsafe { v_p = v_p.add(1) };
                                p2 = unsafe { p2.add(1) };
                            }
                            b'\\' => {
                                unsafe { *v_p = b'\\' };
                                unsafe { v_p = v_p.add(1) };
                                p2 = unsafe { p2.add(1) };
                            }
                            b'X' | b'x' | b'u' | b'U' => {
                                if unsafe { p2.add(1) } < e && ascii_isxdigit(unsafe { *p2.add(1) })
                                {
                                    let n_max: usize =
                                        if unsafe { *p2 } == b'x' || unsafe { *p2 } == b'X' {
                                            2
                                        } else if unsafe { *p2 } == b'u' {
                                            4
                                        } else {
                                            8
                                        };
                                    let is_hex = unsafe { *p2 } == b'x' || unsafe { *p2 } == b'X';
                                    let mut n = n_max;
                                    let mut nr: c_int = 0;
                                    while unsafe { p2.add(1) } < e
                                        && n > 0
                                        && ascii_isxdigit(unsafe { *p2.add(1) })
                                    {
                                        p2 = unsafe { p2.add(1) };
                                        n -= 1;
                                        nr = (nr << 4) + unsafe { hex2nr(*p2.cast::<c_char>()) };
                                    }
                                    p2 = unsafe { p2.add(1) };
                                    if is_hex {
                                        unsafe { *v_p = nr as u8 };
                                        unsafe { v_p = v_p.add(1) };
                                    } else {
                                        let bytes_written =
                                            unsafe { utf_char2bytes(nr, v_p.cast::<c_char>()) };
                                        unsafe { v_p = v_p.add(bytes_written as usize) };
                                    }
                                } else {
                                    is_unknown = true;
                                    let b = unsafe { *p2 };
                                    unsafe { *v_p = b };
                                    unsafe { v_p = v_p.add(1) };
                                    p2 = unsafe { p2.add(1) };
                                }
                            }
                            b'0'..=b'7' => {
                                let mut ch = unsafe { *p2 } - b'0';
                                p2 = unsafe { p2.add(1) };
                                if p2 < e && unsafe { *p2 } >= b'0' && unsafe { *p2 } <= b'7' {
                                    ch = (ch << 3) + unsafe { *p2 } - b'0';
                                    p2 = unsafe { p2.add(1) };
                                    if p2 < e && unsafe { *p2 } >= b'0' && unsafe { *p2 } <= b'7' {
                                        ch = (ch << 3) + unsafe { *p2 } - b'0';
                                        p2 = unsafe { p2.add(1) };
                                    }
                                }
                                unsafe { *v_p = ch };
                                unsafe { v_p = v_p.add(1) };
                            }
                            b'<' => {
                                let mut flags_val = FSK_KEYCODE | FSK_IN_STRING;
                                if unsafe { *p2.add(1) } != b'*' {
                                    flags_val |= FSK_SIMPLIFY;
                                }
                                let mut srcp = p2.cast::<c_char>();
                                let remaining = unsafe { e.offset_from(p2) } as usize;
                                let special_len = unsafe {
                                    trans_special(
                                        &raw mut srcp,
                                        remaining,
                                        v_p.cast::<c_char>(),
                                        flags_val,
                                        false,
                                        std::ptr::null_mut(),
                                    )
                                };
                                if special_len != 0 {
                                    unsafe { v_p = v_p.add(special_len) };
                                    p2 = srcp.cast::<u8>();
                                } else {
                                    is_unknown = true;
                                    let mut srcp2 = p2.cast::<c_char>();
                                    let mut dstp = v_p.cast::<c_char>();
                                    unsafe { mb_copy_char(&raw mut srcp2, &raw mut dstp) };
                                    p2 = srcp2.cast::<u8>();
                                    v_p = dstp.cast::<u8>();
                                }
                            }
                            _ => {
                                is_unknown = true;
                                let mut srcp = p2.cast::<c_char>();
                                let mut dstp = v_p.cast::<c_char>();
                                unsafe { mb_copy_char(&raw mut srcp, &raw mut dstp) };
                                p2 = srcp.cast::<u8>();
                                v_p = dstp.cast::<u8>();
                            }
                        }
                        if has_colors {
                            let act = unsafe { v_p.offset_from(v_p_start) } as usize;
                            shifts.push(StringShift {
                                start: token.start.col + unsafe { chunk_e.offset_from(s) } as usize,
                                orig_len: unsafe { p2.offset_from(chunk_e) } as usize,
                                act_len: act,
                                escape_not_known: is_unknown,
                            });
                        }
                    }
                }
            }
            let actual_size = unsafe { v_p.offset_from(v_raw.cast::<u8>()) } as usize;
            AstNodeDataStr {
                value: v_raw.cast::<c_char>(),
                size: actual_size,
            }
        };
        unsafe { (*node).data.str_ = str_data };
    }

    // Emit highlighting if colors are enabled.
    if has_colors {
        let mut next_col = token.start.col + 1;
        let body_str = if is_double {
            hl_group!(is_invalid, "DoubleQuotedBody")
        } else {
            hl_group!(is_invalid, "SingleQuotedBody")
        };
        let esc_str = if is_double {
            hl_group!(is_invalid, "DoubleQuotedEscape")
        } else {
            hl_group!(is_invalid, "SingleQuotedQuote")
        };
        let ukn_esc_str = if is_double {
            hl_group!(is_invalid, "DoubleQuotedUnknownEscape")
        } else {
            hl_group!(is_invalid, "SingleQuotedUnknownEscape")
        };

        for shift in &shifts {
            if shift.start > next_col {
                let pos = ParserPosition {
                    line: token.start.line,
                    col: next_col,
                };
                unsafe {
                    nvim_viml_parser_highlight(pstate, pos, shift.start - next_col, body_str)
                };
            }
            let pos = ParserPosition {
                line: token.start.line,
                col: shift.start,
            };
            let group = if shift.escape_not_known {
                ukn_esc_str
            } else {
                esc_str
            };
            unsafe { nvim_viml_parser_highlight(pstate, pos, shift.orig_len, group) };
            next_col = shift.start + shift.orig_len;
        }

        let end_col = token.start.col + token_len - if closed { 1 } else { 0 };
        if next_col - token.start.col < token_len - if closed { 1 } else { 0 } {
            let pos = ParserPosition {
                line: token.start.line,
                col: next_col,
            };
            let body_len = end_col - next_col;
            if body_len > 0 {
                unsafe { nvim_viml_parser_highlight(pstate, pos, body_len, body_str) };
            }
        }
    }

    if closed {
        let close_hl = if is_double {
            hl_group!(is_invalid, "DoubleQuote")
        } else {
            hl_group!(is_invalid, "SingleQuote")
        };
        let pos = ParserPosition {
            line: token.start.line,
            col: token.start.col + token_len - 1,
        };
        unsafe { nvim_viml_parser_highlight(pstate, pos, 1, close_hl) };
    }
}

// ---------------------------------------------------------------------------
// memchr helper (avoid pulling in libc crate)
// ---------------------------------------------------------------------------

/// Search for byte `needle` in slice starting at `haystack`, length `len`.
/// Returns a pointer to the first occurrence, or None.
unsafe fn memchr(haystack: *const u8, needle: u8, len: usize) -> Option<*const u8> {
    for i in 0..len {
        let p = unsafe { haystack.add(i) };
        if unsafe { *p } == needle {
            return Some(p);
        }
    }
    None
}
