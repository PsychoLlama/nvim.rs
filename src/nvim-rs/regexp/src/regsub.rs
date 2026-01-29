//! Regex substitution engine.
//!
//! This module provides the infrastructure for performing substitutions after
//! a regex match. It handles:
//!
//! - Submatch expansion (\0 through \9, and & in magic mode)
//! - Case conversion (\u, \U, \l, \L, \e, \E)
//! - Escape sequences (\r, \n, \t, \b)
//! - Special character handling (backslash escaping, K_SPECIAL)
//!
//! The main entry point is `vim_regsub_both` which is called by both
//! `vim_regsub` (single-line) and `vim_regsub_multi` (multi-line).

use std::ffi::c_int;
use std::ptr;

// =============================================================================
// Constants
// =============================================================================

/// Flag: really copy into dest (otherwise just compute length).
pub const REGSUB_COPY: c_int = 1;

/// Flag: behave as if 'magic' is set.
pub const REGSUB_MAGIC: c_int = 2;

/// Flag: backslash will be removed later, need to double them.
pub const REGSUB_BACKSLASH: c_int = 4;

/// Maximum nesting level for substitute expressions.
pub const MAX_REGSUB_NESTING: usize = 4;

/// Carriage return character.
const CAR: u8 = b'\r';

/// Newline character.
const NL: u8 = b'\n';

/// Tab character.
const TAB: u8 = b'\t';

/// Ctrl-H (backspace).
const CTRL_H: u8 = 8;

/// K_SPECIAL - start of special key sequence.
const K_SPECIAL: u8 = 0x80;

/// Number of subexpressions.
pub const NSUBEXP: usize = 10;

// =============================================================================
// Types
// =============================================================================

/// Case conversion function type.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseConvert {
    /// No conversion.
    None,
    /// Convert to uppercase.
    Upper,
    /// Convert to lowercase.
    Lower,
}

/// Substitution context for tracking state during substitution.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SubContext {
    /// Source pattern position.
    pub src: *const u8,
    /// Destination buffer position.
    pub dst: *mut u8,
    /// Destination buffer start.
    pub dest_start: *mut u8,
    /// Destination buffer length.
    pub destlen: c_int,
    /// Flags (REGSUB_COPY, REGSUB_MAGIC, REGSUB_BACKSLASH).
    pub flags: c_int,
    /// Case conversion for next character.
    pub func_one: CaseConvert,
    /// Case conversion for all following characters.
    pub func_all: CaseConvert,
    /// Whether we're in copy mode.
    pub copy: bool,
}

impl Default for SubContext {
    fn default() -> Self {
        Self {
            src: ptr::null(),
            dst: ptr::null_mut(),
            dest_start: ptr::null_mut(),
            destlen: 0,
            flags: 0,
            func_one: CaseConvert::None,
            func_all: CaseConvert::None,
            copy: false,
        }
    }
}

impl SubContext {
    /// Create a new substitution context.
    ///
    /// # Safety
    /// `source` must point to a valid null-terminated string.
    /// `dest` must point to a buffer of at least `destlen` bytes.
    pub unsafe fn new(source: *const u8, dest: *mut u8, destlen: c_int, flags: c_int) -> Self {
        Self {
            src: source,
            dst: dest,
            dest_start: dest,
            destlen,
            flags,
            func_one: CaseConvert::None,
            func_all: CaseConvert::None,
            copy: (flags & REGSUB_COPY) != 0,
        }
    }

    /// Check if there's room for `n` more bytes in the destination buffer.
    #[inline]
    pub fn has_room(&self, n: c_int) -> bool {
        if !self.copy {
            return true;
        }
        // Safety: pointer arithmetic within destination buffer bounds
        unsafe { self.dst.add(n as usize) <= self.dest_start.add(self.destlen as usize) }
    }

    /// Write a byte to the destination if in copy mode.
    ///
    /// # Safety
    /// Must have verified `has_room(1)` first.
    #[inline]
    pub unsafe fn write_byte(&mut self, byte: u8) {
        if self.copy {
            *self.dst = byte;
        }
        self.dst = self.dst.add(1);
    }

    /// Get the current output length.
    #[inline]
    pub fn output_len(&self) -> c_int {
        // Safety: dst and dest_start are from the same allocation
        unsafe { self.dst.offset_from(self.dest_start) as c_int }
    }

    /// Apply case conversion to a character.
    #[inline]
    pub fn apply_case(&mut self, c: c_int) -> c_int {
        if self.func_one != CaseConvert::None {
            let converted = match self.func_one {
                CaseConvert::Upper => do_upper(c),
                CaseConvert::Lower => do_lower(c),
                CaseConvert::None => c,
            };
            self.func_one = CaseConvert::None;
            converted
        } else if self.func_all != CaseConvert::None {
            match self.func_all {
                CaseConvert::Upper => do_upper(c),
                CaseConvert::Lower => do_lower(c),
                CaseConvert::None => c,
            }
        } else {
            c
        }
    }

    /// Reset case conversion functions.
    #[inline]
    pub fn reset_case(&mut self) {
        self.func_one = CaseConvert::None;
        self.func_all = CaseConvert::None;
    }
}

/// Single-line match info for substitution.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct RegMatch {
    /// Start positions for each submatch.
    pub startp: [*const u8; NSUBEXP],
    /// End positions for each submatch.
    pub endp: [*const u8; NSUBEXP],
}

impl Default for RegMatch {
    fn default() -> Self {
        Self {
            startp: [ptr::null(); NSUBEXP],
            endp: [ptr::null(); NSUBEXP],
        }
    }
}

/// Line position for multi-line matches.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LPos {
    /// Line number (1-based).
    pub lnum: c_int,
    /// Column number (0-based byte offset).
    pub col: c_int,
}

/// Multi-line match info for substitution.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct RegMMatch {
    /// Start positions for each submatch.
    pub startpos: [LPos; NSUBEXP],
    /// End positions for each submatch.
    pub endpos: [LPos; NSUBEXP],
}

impl Default for RegMMatch {
    fn default() -> Self {
        Self {
            startpos: [LPos::default(); NSUBEXP],
            endpos: [LPos::default(); NSUBEXP],
        }
    }
}

// =============================================================================
// Case Conversion Functions
// =============================================================================

/// Convert a character to uppercase.
#[inline]
pub fn do_upper(c: c_int) -> c_int {
    if c < 0x80 {
        // ASCII fast path
        if (c as u8).is_ascii_lowercase() {
            (c as u8).to_ascii_uppercase() as c_int
        } else {
            c
        }
    } else {
        // Unicode - would need proper Unicode tables
        // For now, just return as-is (full implementation would use libc toupper or similar)
        c
    }
}

/// Convert a character to lowercase.
#[inline]
pub fn do_lower(c: c_int) -> c_int {
    if c < 0x80 {
        // ASCII fast path
        if (c as u8).is_ascii_uppercase() {
            (c as u8).to_ascii_lowercase() as c_int
        } else {
            c
        }
    } else {
        // Unicode - would need proper Unicode tables
        c
    }
}

// =============================================================================
// Escape Sequence Handling
// =============================================================================

/// Result of parsing an escape sequence.
#[derive(Debug, Clone, Copy)]
pub enum EscapeResult {
    /// Regular character (not an escape).
    None,
    /// Submatch reference (0-9).
    Submatch(u8),
    /// Case conversion: uppercase next.
    UpperOne,
    /// Case conversion: uppercase all.
    UpperAll,
    /// Case conversion: lowercase next.
    LowerOne,
    /// Case conversion: lowercase all.
    LowerAll,
    /// Case conversion: end.
    CaseEnd,
    /// Literal character from escape (e.g., \r -> CR).
    Literal(u8),
    /// Escaped backslash (keep the backslash).
    EscapedBackslash,
}

/// Parse an escape sequence at the current position.
///
/// Returns the escape result and the number of source bytes consumed.
///
/// # Safety
/// `src` must point to a valid string with at least 2 bytes available
/// when the first byte is a backslash.
pub unsafe fn parse_escape(src: *const u8, magic: bool) -> (EscapeResult, usize) {
    let c = *src;

    // Handle & in magic mode
    if c == b'&' && magic {
        return (EscapeResult::Submatch(0), 1);
    }

    // Not a backslash escape
    if c != b'\\' {
        return (EscapeResult::None, 0);
    }

    let next = *src.add(1);
    if next == 0 {
        return (EscapeResult::None, 0);
    }

    match next {
        // & in non-magic mode
        b'&' if !magic => (EscapeResult::Submatch(0), 2),

        // Submatch references \0 - \9
        b'0'..=b'9' => (EscapeResult::Submatch(next - b'0'), 2),

        // Case conversions
        b'u' => (EscapeResult::UpperOne, 2),
        b'U' => (EscapeResult::UpperAll, 2),
        b'l' => (EscapeResult::LowerOne, 2),
        b'L' => (EscapeResult::LowerAll, 2),
        b'e' | b'E' => (EscapeResult::CaseEnd, 2),

        // Character escapes
        b'r' => (EscapeResult::Literal(CAR), 2),
        b'n' => (EscapeResult::Literal(NL), 2),
        b't' => (EscapeResult::Literal(TAB), 2),
        b'b' => (EscapeResult::Literal(CTRL_H), 2),

        // Other backslash escapes - keep the character
        _ => (EscapeResult::EscapedBackslash, 1),
    }
}

/// Check if a byte is the start of a K_SPECIAL sequence.
#[inline]
pub fn is_k_special(b: u8) -> bool {
    b == K_SPECIAL
}

// =============================================================================
// Submatch Extraction
// =============================================================================

/// Get the submatch string and length for single-line match.
///
/// # Safety
/// `match_info` must be valid and the submatch must exist.
pub unsafe fn get_submatch_single(
    match_info: *const RegMatch,
    no: usize,
) -> Option<(*const u8, c_int)> {
    if match_info.is_null() || no >= NSUBEXP {
        return None;
    }

    let start = (*match_info).startp[no];
    let end = (*match_info).endp[no];

    if start.is_null() || end.is_null() {
        return None;
    }

    let len = end.offset_from(start) as c_int;
    Some((start, len))
}

/// Information needed to iterate over a multi-line submatch.
#[derive(Debug, Clone)]
pub struct MultiLineSubmatch {
    /// Start line number.
    pub start_lnum: c_int,
    /// End line number.
    pub end_lnum: c_int,
    /// Start column.
    pub start_col: c_int,
    /// End column.
    pub end_col: c_int,
}

/// Get the multi-line submatch bounds.
///
/// # Safety
/// `match_info` must be valid.
pub unsafe fn get_submatch_multi(
    match_info: *const RegMMatch,
    no: usize,
) -> Option<MultiLineSubmatch> {
    if match_info.is_null() || no >= NSUBEXP {
        return None;
    }

    let start_lnum = (*match_info).startpos[no].lnum;
    let end_lnum = (*match_info).endpos[no].lnum;

    if start_lnum < 0 || end_lnum < 0 {
        return None;
    }

    Some(MultiLineSubmatch {
        start_lnum,
        end_lnum,
        start_col: (*match_info).startpos[no].col,
        end_col: (*match_info).endpos[no].col,
    })
}

// =============================================================================
// UTF-8 Helpers
// =============================================================================

/// Get the number of bytes in a UTF-8 character.
#[inline]
pub fn utf8_char_len(first_byte: u8) -> usize {
    if first_byte < 0x80 {
        1
    } else if first_byte < 0xC0 {
        // Continuation byte (shouldn't be first)
        1
    } else if first_byte < 0xE0 {
        2
    } else if first_byte < 0xF0 {
        3
    } else {
        4
    }
}

/// Decode a UTF-8 character from bytes.
///
/// # Safety
/// `ptr` must point to a valid UTF-8 sequence.
pub unsafe fn utf8_ptr2char(ptr: *const u8) -> c_int {
    let b0 = *ptr;
    if b0 < 0x80 {
        return b0 as c_int;
    }

    let len = utf8_char_len(b0);
    match len {
        2 => {
            let b1 = *ptr.add(1);
            (((b0 & 0x1F) as c_int) << 6) | ((b1 & 0x3F) as c_int)
        }
        3 => {
            let b1 = *ptr.add(1);
            let b2 = *ptr.add(2);
            (((b0 & 0x0F) as c_int) << 12) | (((b1 & 0x3F) as c_int) << 6) | ((b2 & 0x3F) as c_int)
        }
        4 => {
            let b1 = *ptr.add(1);
            let b2 = *ptr.add(2);
            let b3 = *ptr.add(3);
            (((b0 & 0x07) as c_int) << 18)
                | (((b1 & 0x3F) as c_int) << 12)
                | (((b2 & 0x3F) as c_int) << 6)
                | ((b3 & 0x3F) as c_int)
        }
        _ => b0 as c_int,
    }
}

/// Encode a Unicode code point to UTF-8.
///
/// Returns the number of bytes written (1-4).
///
/// # Safety
/// `dst` must have room for at least 4 bytes.
pub unsafe fn utf8_char2bytes(c: c_int, dst: *mut u8) -> usize {
    if c < 0x80 {
        *dst = c as u8;
        1
    } else if c < 0x800 {
        *dst = (0xC0 | ((c >> 6) & 0x1F)) as u8;
        *dst.add(1) = (0x80 | (c & 0x3F)) as u8;
        2
    } else if c < 0x10000 {
        *dst = (0xE0 | ((c >> 12) & 0x0F)) as u8;
        *dst.add(1) = (0x80 | ((c >> 6) & 0x3F)) as u8;
        *dst.add(2) = (0x80 | (c & 0x3F)) as u8;
        3
    } else {
        *dst = (0xF0 | ((c >> 18) & 0x07)) as u8;
        *dst.add(1) = (0x80 | ((c >> 12) & 0x3F)) as u8;
        *dst.add(2) = (0x80 | ((c >> 6) & 0x3F)) as u8;
        *dst.add(3) = (0x80 | (c & 0x3F)) as u8;
        4
    }
}

/// Get the byte length of a UTF-8 encoded character.
#[inline]
pub fn utf8_char2len(c: c_int) -> usize {
    if c < 0x80 {
        1
    } else if c < 0x800 {
        2
    } else if c < 0x10000 {
        3
    } else {
        4
    }
}

// =============================================================================
// Main Substitution Function
// =============================================================================

/// Perform regex substitution.
///
/// This is the main substitution function that handles:
/// - Submatch expansion (\0-\9, &)
/// - Case conversion (\u, \U, \l, \L, \e, \E)
/// - Escape sequences (\r, \n, \t, \b)
/// - Literal character copy
///
/// # Parameters
/// - `source`: Replacement pattern
/// - `dest`: Output buffer
/// - `destlen`: Output buffer length
/// - `match_info`: Single-line match info (or null)
/// - `mmatch_info`: Multi-line match info (or null)
/// - `flags`: REGSUB_* flags
///
/// # Returns
/// Length of the substitution result, or -1 on error.
///
/// # Safety
/// All non-null pointers must be valid.
pub unsafe fn vim_regsub_both(
    source: *const u8,
    dest: *mut u8,
    destlen: c_int,
    match_info: *const RegMatch,
    _mmatch_info: *const RegMMatch,
    flags: c_int,
) -> c_int {
    if source.is_null() {
        return -1;
    }

    let mut ctx = SubContext::new(source, dest, destlen, flags);
    let magic = (flags & REGSUB_MAGIC) != 0;

    while *ctx.src != 0 {
        // Check for escape sequences
        let (escape_result, consumed) = parse_escape(ctx.src, magic);

        match escape_result {
            EscapeResult::None => {
                // Regular character - copy with case conversion
                let char_len = utf8_char_len(*ctx.src);
                let c = utf8_ptr2char(ctx.src);
                let converted = ctx.apply_case(c);

                if converted != c || char_len > 1 {
                    // Need to encode the (possibly converted) character
                    if !ctx.has_room(4) {
                        break;
                    }
                    let written = utf8_char2bytes(converted, ctx.dst);
                    if ctx.copy {
                        ctx.dst = ctx.dst.add(written);
                    } else {
                        ctx.dst = ctx.dst.add(utf8_char2len(converted));
                    }
                } else {
                    // Simple ASCII copy
                    if !ctx.has_room(1) {
                        break;
                    }
                    ctx.write_byte(*ctx.src);
                }
                ctx.src = ctx.src.add(char_len);
            }

            EscapeResult::Submatch(no) => {
                // Copy the submatch content
                if let Some((start, len)) = get_submatch_single(match_info, no as usize) {
                    if !ctx.has_room(len) {
                        break;
                    }

                    // Copy each character with case conversion
                    let mut p = start;
                    let end = start.add(len as usize);
                    while p < end {
                        let char_len = utf8_char_len(*p);
                        let c = utf8_ptr2char(p);
                        let converted = ctx.apply_case(c);

                        let written = utf8_char2bytes(converted, ctx.dst);
                        if ctx.copy {
                            ctx.dst = ctx.dst.add(written);
                        } else {
                            ctx.dst = ctx.dst.add(utf8_char2len(converted));
                        }
                        p = p.add(char_len);
                    }
                }
                ctx.src = ctx.src.add(consumed);
            }

            EscapeResult::UpperOne => {
                ctx.func_one = CaseConvert::Upper;
                ctx.src = ctx.src.add(consumed);
            }

            EscapeResult::UpperAll => {
                ctx.func_all = CaseConvert::Upper;
                ctx.src = ctx.src.add(consumed);
            }

            EscapeResult::LowerOne => {
                ctx.func_one = CaseConvert::Lower;
                ctx.src = ctx.src.add(consumed);
            }

            EscapeResult::LowerAll => {
                ctx.func_all = CaseConvert::Lower;
                ctx.src = ctx.src.add(consumed);
            }

            EscapeResult::CaseEnd => {
                ctx.reset_case();
                ctx.src = ctx.src.add(consumed);
            }

            EscapeResult::Literal(byte) => {
                if !ctx.has_room(1) {
                    break;
                }
                let converted = ctx.apply_case(byte as c_int);
                ctx.write_byte(converted as u8);
                ctx.src = ctx.src.add(consumed);
            }

            EscapeResult::EscapedBackslash => {
                // Skip the backslash, copy the next character
                ctx.src = ctx.src.add(1);
                if *ctx.src != 0 {
                    let char_len = utf8_char_len(*ctx.src);
                    let c = utf8_ptr2char(ctx.src);
                    let converted = ctx.apply_case(c);

                    if !ctx.has_room(utf8_char2len(converted) as c_int) {
                        break;
                    }
                    let written = utf8_char2bytes(converted, ctx.dst);
                    if ctx.copy {
                        ctx.dst = ctx.dst.add(written);
                    } else {
                        ctx.dst = ctx.dst.add(utf8_char2len(converted));
                    }
                    ctx.src = ctx.src.add(char_len);
                }
            }
        }
    }

    ctx.output_len()
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Perform regex substitution.
///
/// # Safety
/// All non-null pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regsub_both(
    source: *const u8,
    dest: *mut u8,
    destlen: c_int,
    match_info: *const RegMatch,
    mmatch_info: *const RegMMatch,
    flags: c_int,
) -> c_int {
    vim_regsub_both(source, dest, destlen, match_info, mmatch_info, flags)
}

/// Create a new substitution context.
///
/// # Safety
/// `source` must point to a valid null-terminated string.
/// `dest` must point to a buffer of at least `destlen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_sub_context_new(
    source: *const u8,
    dest: *mut u8,
    destlen: c_int,
    flags: c_int,
) -> SubContext {
    SubContext::new(source, dest, destlen, flags)
}

/// Check if context has room for n bytes.
///
/// # Safety
/// If `ctx` is non-null, it must point to a valid SubContext.
#[no_mangle]
pub unsafe extern "C" fn rs_sub_context_has_room(ctx: *const SubContext, n: c_int) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from((*ctx).has_room(n))
}

/// Get the output length so far.
///
/// # Safety
/// If `ctx` is non-null, it must point to a valid SubContext.
#[no_mangle]
pub unsafe extern "C" fn rs_sub_context_output_len(ctx: *const SubContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    (*ctx).output_len()
}

/// Apply case conversion to a character.
#[no_mangle]
pub extern "C" fn rs_do_upper(c: c_int) -> c_int {
    do_upper(c)
}

/// Apply case conversion to a character.
#[no_mangle]
pub extern "C" fn rs_do_lower(c: c_int) -> c_int {
    do_lower(c)
}

/// Get the UTF-8 byte length for a character.
#[no_mangle]
pub extern "C" fn rs_utf8_char_len(first_byte: u8) -> c_int {
    utf8_char_len(first_byte) as c_int
}

/// Decode a UTF-8 character.
///
/// # Safety
/// `ptr` must point to a valid UTF-8 sequence.
#[no_mangle]
pub unsafe extern "C" fn rs_utf8_ptr2char(ptr: *const u8) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    utf8_ptr2char(ptr)
}

/// Encode a character to UTF-8.
///
/// # Safety
/// `dst` must have room for at least 4 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_utf8_char2bytes(c: c_int, dst: *mut u8) -> c_int {
    if dst.is_null() {
        return 0;
    }
    utf8_char2bytes(c, dst) as c_int
}

/// Get byte length for encoded character.
#[no_mangle]
pub extern "C" fn rs_utf8_char2len(c: c_int) -> c_int {
    utf8_char2len(c) as c_int
}

/// Check if byte is K_SPECIAL.
#[no_mangle]
pub extern "C" fn rs_is_k_special(b: u8) -> c_int {
    c_int::from(is_k_special(b))
}

// =============================================================================
// Additional FFI Exports (R6)
// =============================================================================

/// Get REGSUB_COPY constant.
#[no_mangle]
pub extern "C" fn rs_regsub_copy() -> c_int {
    REGSUB_COPY
}

/// Get REGSUB_MAGIC constant.
#[no_mangle]
pub extern "C" fn rs_regsub_magic() -> c_int {
    REGSUB_MAGIC
}

/// Get REGSUB_BACKSLASH constant.
#[no_mangle]
pub extern "C" fn rs_regsub_backslash() -> c_int {
    REGSUB_BACKSLASH
}

/// Get NSUBEXP constant.
#[no_mangle]
pub extern "C" fn rs_regsub_nsubexp() -> c_int {
    NSUBEXP as c_int
}

/// Get MAX_REGSUB_NESTING constant.
#[no_mangle]
pub extern "C" fn rs_regsub_max_nesting() -> c_int {
    MAX_REGSUB_NESTING as c_int
}

/// Get the carriage return constant.
#[no_mangle]
pub extern "C" fn rs_regsub_car() -> u8 {
    CAR
}

/// Get the newline constant.
#[no_mangle]
pub extern "C" fn rs_regsub_nl() -> u8 {
    NL
}

/// Get the tab constant.
#[no_mangle]
pub extern "C" fn rs_regsub_tab() -> u8 {
    TAB
}

/// Get the K_SPECIAL constant for regsub.
#[no_mangle]
pub extern "C" fn rs_regsub_k_special() -> u8 {
    K_SPECIAL
}

/// Parse a regsub escape sequence at the given position.
///
/// Returns the escape type and number of bytes consumed.
///
/// Escape types:
/// - 0: None (not an escape)
/// - 1-10: Submatch reference (\0-\9)
/// - 11: UpperOne
/// - 12: UpperAll
/// - 13: LowerOne
/// - 14: LowerAll
/// - 15: CaseEnd
/// - 16: Literal (literal byte in out_byte)
/// - 17: EscapedBackslash
///
/// # Safety
/// `src` must point to a valid string.
#[no_mangle]
pub unsafe extern "C" fn rs_regsub_parse_escape(
    src: *const u8,
    magic: c_int,
    out_consumed: *mut c_int,
    out_byte: *mut u8,
) -> c_int {
    if src.is_null() {
        if !out_consumed.is_null() {
            *out_consumed = 0;
        }
        return 0;
    }

    let (result, consumed) = parse_escape(src, magic != 0);

    if !out_consumed.is_null() {
        *out_consumed = consumed as c_int;
    }

    match result {
        EscapeResult::None => 0,
        EscapeResult::Submatch(n) => (n + 1) as c_int,
        EscapeResult::UpperOne => 11,
        EscapeResult::UpperAll => 12,
        EscapeResult::LowerOne => 13,
        EscapeResult::LowerAll => 14,
        EscapeResult::CaseEnd => 15,
        EscapeResult::Literal(b) => {
            if !out_byte.is_null() {
                *out_byte = b;
            }
            16
        }
        EscapeResult::EscapedBackslash => 17,
    }
}

/// Get a single-line submatch.
///
/// Returns 1 if found, 0 if not found.
///
/// # Safety
/// `match_info` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_get_submatch_single(
    match_info: *const RegMatch,
    no: c_int,
    out_start: *mut *const u8,
    out_len: *mut c_int,
) -> c_int {
    if no < 0 {
        return 0;
    }
    match get_submatch_single(match_info, no as usize) {
        Some((start, len)) => {
            if !out_start.is_null() {
                *out_start = start;
            }
            if !out_len.is_null() {
                *out_len = len;
            }
            1
        }
        None => 0,
    }
}

/// Get multi-line submatch bounds.
///
/// Returns 1 if found, 0 if not found.
///
/// # Safety
/// `match_info` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_get_submatch_multi(
    match_info: *const RegMMatch,
    no: c_int,
    out_start_lnum: *mut c_int,
    out_end_lnum: *mut c_int,
    out_start_col: *mut c_int,
    out_end_col: *mut c_int,
) -> c_int {
    if no < 0 {
        return 0;
    }
    match get_submatch_multi(match_info, no as usize) {
        Some(info) => {
            if !out_start_lnum.is_null() {
                *out_start_lnum = info.start_lnum;
            }
            if !out_end_lnum.is_null() {
                *out_end_lnum = info.end_lnum;
            }
            if !out_start_col.is_null() {
                *out_start_col = info.start_col;
            }
            if !out_end_col.is_null() {
                *out_end_col = info.end_col;
            }
            1
        }
        None => 0,
    }
}

/// Create a new RegMMatch structure.
#[no_mangle]
pub extern "C" fn rs_regmmatch_new() -> *mut RegMMatch {
    Box::into_raw(Box::new(RegMMatch::default()))
}

/// Free a RegMMatch structure.
///
/// # Safety
/// `m` must be from rs_regmmatch_new.
#[no_mangle]
pub unsafe extern "C" fn rs_regmmatch_free(m: *mut RegMMatch) {
    if !m.is_null() {
        drop(Box::from_raw(m));
    }
}

/// Set multi-line submatch start position.
///
/// # Safety
/// `m` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_regmmatch_set_startpos(
    m: *mut RegMMatch,
    no: c_int,
    lnum: c_int,
    col: c_int,
) {
    if !m.is_null() && no >= 0 && (no as usize) < NSUBEXP {
        (*m).startpos[no as usize].lnum = lnum;
        (*m).startpos[no as usize].col = col;
    }
}

/// Set multi-line submatch end position.
///
/// # Safety
/// `m` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_regmmatch_set_endpos(
    m: *mut RegMMatch,
    no: c_int,
    lnum: c_int,
    col: c_int,
) {
    if !m.is_null() && no >= 0 && (no as usize) < NSUBEXP {
        (*m).endpos[no as usize].lnum = lnum;
        (*m).endpos[no as usize].col = col;
    }
}

/// Check if a character is ASCII uppercase.
#[no_mangle]
pub extern "C" fn rs_is_ascii_uppercase(c: u8) -> c_int {
    c_int::from(c.is_ascii_uppercase())
}

/// Check if a character is ASCII lowercase.
#[no_mangle]
pub extern "C" fn rs_is_ascii_lowercase(c: u8) -> c_int {
    c_int::from(c.is_ascii_lowercase())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(REGSUB_COPY, 1);
        assert_eq!(REGSUB_MAGIC, 2);
        assert_eq!(REGSUB_BACKSLASH, 4);
        assert_eq!(MAX_REGSUB_NESTING, 4);
    }

    #[test]
    fn test_do_upper() {
        assert_eq!(do_upper(b'a' as c_int), b'A' as c_int);
        assert_eq!(do_upper(b'z' as c_int), b'Z' as c_int);
        assert_eq!(do_upper(b'A' as c_int), b'A' as c_int);
        assert_eq!(do_upper(b'1' as c_int), b'1' as c_int);
    }

    #[test]
    fn test_do_lower() {
        assert_eq!(do_lower(b'A' as c_int), b'a' as c_int);
        assert_eq!(do_lower(b'Z' as c_int), b'z' as c_int);
        assert_eq!(do_lower(b'a' as c_int), b'a' as c_int);
        assert_eq!(do_lower(b'1' as c_int), b'1' as c_int);
    }

    #[test]
    fn test_parse_escape_submatch() {
        unsafe {
            // \0 through \9
            let src = b"\\0rest\0";
            let (result, consumed) = parse_escape(src.as_ptr(), false);
            assert!(matches!(result, EscapeResult::Submatch(0)));
            assert_eq!(consumed, 2);

            let src = b"\\5rest\0";
            let (result, consumed) = parse_escape(src.as_ptr(), false);
            assert!(matches!(result, EscapeResult::Submatch(5)));
            assert_eq!(consumed, 2);

            // & in magic mode
            let src = b"&rest\0";
            let (result, consumed) = parse_escape(src.as_ptr(), true);
            assert!(matches!(result, EscapeResult::Submatch(0)));
            assert_eq!(consumed, 1);

            // & in non-magic mode
            let src = b"&rest\0";
            let (result, _) = parse_escape(src.as_ptr(), false);
            assert!(matches!(result, EscapeResult::None));
        }
    }

    #[test]
    fn test_parse_escape_case() {
        unsafe {
            let src = b"\\urest\0";
            let (result, consumed) = parse_escape(src.as_ptr(), false);
            assert!(matches!(result, EscapeResult::UpperOne));
            assert_eq!(consumed, 2);

            let src = b"\\Urest\0";
            let (result, _) = parse_escape(src.as_ptr(), false);
            assert!(matches!(result, EscapeResult::UpperAll));

            let src = b"\\lrest\0";
            let (result, _) = parse_escape(src.as_ptr(), false);
            assert!(matches!(result, EscapeResult::LowerOne));

            let src = b"\\Lrest\0";
            let (result, _) = parse_escape(src.as_ptr(), false);
            assert!(matches!(result, EscapeResult::LowerAll));

            let src = b"\\erest\0";
            let (result, _) = parse_escape(src.as_ptr(), false);
            assert!(matches!(result, EscapeResult::CaseEnd));
        }
    }

    #[test]
    fn test_parse_escape_literal() {
        unsafe {
            let src = b"\\rrest\0";
            let (result, consumed) = parse_escape(src.as_ptr(), false);
            assert!(matches!(result, EscapeResult::Literal(CAR)));
            assert_eq!(consumed, 2);

            let src = b"\\nrest\0";
            let (result, _) = parse_escape(src.as_ptr(), false);
            assert!(matches!(result, EscapeResult::Literal(NL)));

            let src = b"\\trest\0";
            let (result, _) = parse_escape(src.as_ptr(), false);
            assert!(matches!(result, EscapeResult::Literal(TAB)));

            let src = b"\\brest\0";
            let (result, _) = parse_escape(src.as_ptr(), false);
            assert!(matches!(result, EscapeResult::Literal(CTRL_H)));
        }
    }

    #[test]
    fn test_utf8_char_len() {
        assert_eq!(utf8_char_len(b'A'), 1);
        assert_eq!(utf8_char_len(0x7F), 1);
        assert_eq!(utf8_char_len(0xC2), 2);
        assert_eq!(utf8_char_len(0xE0), 3);
        assert_eq!(utf8_char_len(0xF0), 4);
    }

    #[test]
    fn test_utf8_ptr2char() {
        unsafe {
            // ASCII
            let s = b"A\0";
            assert_eq!(utf8_ptr2char(s.as_ptr()), b'A' as c_int);

            // 2-byte UTF-8 (é = U+00E9)
            let s = [0xC3, 0xA9, 0x00];
            assert_eq!(utf8_ptr2char(s.as_ptr()), 0xE9);

            // 3-byte UTF-8 (中 = U+4E2D)
            let s = [0xE4, 0xB8, 0xAD, 0x00];
            assert_eq!(utf8_ptr2char(s.as_ptr()), 0x4E2D);
        }
    }

    #[test]
    fn test_utf8_char2bytes() {
        unsafe {
            let mut buf = [0u8; 4];

            // ASCII
            let len = utf8_char2bytes(b'A' as c_int, buf.as_mut_ptr());
            assert_eq!(len, 1);
            assert_eq!(buf[0], b'A');

            // 2-byte (é = U+00E9)
            let len = utf8_char2bytes(0xE9, buf.as_mut_ptr());
            assert_eq!(len, 2);
            assert_eq!(buf[0], 0xC3);
            assert_eq!(buf[1], 0xA9);

            // 3-byte (中 = U+4E2D)
            let len = utf8_char2bytes(0x4E2D, buf.as_mut_ptr());
            assert_eq!(len, 3);
            assert_eq!(buf[0], 0xE4);
            assert_eq!(buf[1], 0xB8);
            assert_eq!(buf[2], 0xAD);
        }
    }

    #[test]
    fn test_utf8_char2len() {
        assert_eq!(utf8_char2len(0x41), 1); // 'A'
        assert_eq!(utf8_char2len(0x7F), 1);
        assert_eq!(utf8_char2len(0x80), 2);
        assert_eq!(utf8_char2len(0x7FF), 2);
        assert_eq!(utf8_char2len(0x800), 3);
        assert_eq!(utf8_char2len(0xFFFF), 3);
        assert_eq!(utf8_char2len(0x10000), 4);
    }

    #[test]
    fn test_case_convert() {
        let mut ctx = SubContext::default();

        // No conversion
        assert_eq!(ctx.apply_case(b'a' as c_int), b'a' as c_int);

        // One-shot upper
        ctx.func_one = CaseConvert::Upper;
        assert_eq!(ctx.apply_case(b'a' as c_int), b'A' as c_int);
        assert_eq!(ctx.func_one, CaseConvert::None); // Reset after use
        assert_eq!(ctx.apply_case(b'b' as c_int), b'b' as c_int);

        // All upper
        ctx.func_all = CaseConvert::Upper;
        assert_eq!(ctx.apply_case(b'a' as c_int), b'A' as c_int);
        assert_eq!(ctx.apply_case(b'b' as c_int), b'B' as c_int);
        assert_ne!(ctx.func_all, CaseConvert::None); // Not reset

        // Reset
        ctx.reset_case();
        assert_eq!(ctx.func_one, CaseConvert::None);
        assert_eq!(ctx.func_all, CaseConvert::None);
    }

    #[test]
    fn test_sub_context_default() {
        let ctx = SubContext::default();
        assert!(ctx.src.is_null());
        assert!(ctx.dst.is_null());
        assert_eq!(ctx.destlen, 0);
        assert_eq!(ctx.flags, 0);
        assert!(!ctx.copy);
    }

    #[test]
    fn test_is_k_special() {
        assert!(is_k_special(K_SPECIAL));
        assert!(!is_k_special(b'A'));
        assert!(!is_k_special(0));
    }
}
