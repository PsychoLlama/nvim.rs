//! User command argument parsing
//!
//! This module provides Rust implementations for parsing user command
//! arguments, including tokenization, quoting, and escape handling.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::manual_is_ascii_check)]

use std::ffi::{c_char, c_int};

use crate::complete::{
    ADDR_TYPE_COMPLETE, COMMAND_COMPLETE, EXPAND_BUFFERS, EXPAND_DIRECTORIES, EXPAND_FILES,
    EXPAND_SHELLCMDLINE, EXPAND_USER_DEFINED, EXPAND_USER_LIST,
};
use crate::define::{
    EX_BANG, EX_BUFNAME, EX_COUNT, EX_DFLALL, EX_EXTRA, EX_KEEPSCRIPT, EX_NEEDARG, EX_NOSPC,
    EX_RANGE, EX_REGSTR, EX_TRLBAR, EX_XFILE, EX_ZEROR, UC_BUFFER,
};
use crate::AddrType;

// =============================================================================
// Parse State
// =============================================================================

/// State of argument parsing
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseState {
    /// Normal parsing
    Normal = 0,
    /// Inside single quotes
    SingleQuote = 1,
    /// Inside double quotes
    DoubleQuote = 2,
    /// After backslash escape
    Escape = 3,
    /// Inside bar-separated command
    BarSeparated = 4,
}

impl ParseState {
    /// Check if in a quoted context
    pub const fn is_quoted(self) -> bool {
        matches!(self, Self::SingleQuote | Self::DoubleQuote)
    }

    /// Check if in escape context
    pub const fn is_escape(self) -> bool {
        matches!(self, Self::Escape)
    }

    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::SingleQuote),
            2 => Some(Self::DoubleQuote),
            3 => Some(Self::Escape),
            4 => Some(Self::BarSeparated),
            _ => None,
        }
    }
}

impl Default for ParseState {
    fn default() -> Self {
        Self::Normal
    }
}

// =============================================================================
// Token Type
// =============================================================================

/// Type of parsed token
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Regular word
    Word = 0,
    /// Whitespace
    Whitespace = 1,
    /// Single-quoted string
    SingleQuoted = 2,
    /// Double-quoted string
    DoubleQuoted = 3,
    /// Command separator (|)
    Bar = 4,
    /// End of input
    End = 5,
    /// Error token
    Error = 6,
}

impl TokenType {
    /// Check if this is a string token
    pub const fn is_string(self) -> bool {
        matches!(self, Self::Word | Self::SingleQuoted | Self::DoubleQuoted)
    }

    /// Check if this is a separator
    pub const fn is_separator(self) -> bool {
        matches!(self, Self::Whitespace | Self::Bar | Self::End)
    }

    /// Check if this is an error
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }
}

impl Default for TokenType {
    fn default() -> Self {
        Self::End
    }
}

// =============================================================================
// Token
// =============================================================================

/// A parsed token with position information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Token {
    /// Token type
    pub ttype: TokenType,
    /// Start position in input (byte offset)
    pub start: c_int,
    /// End position in input (byte offset, exclusive)
    pub end: c_int,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            ttype: TokenType::End,
            start: 0,
            end: 0,
        }
    }
}

impl Token {
    /// Create a new token
    pub const fn new(ttype: TokenType, start: c_int, end: c_int) -> Self {
        Self { ttype, start, end }
    }

    /// Get token length
    pub const fn len(&self) -> c_int {
        self.end - self.start
    }

    /// Check if token is empty
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Check if this is a valid token
    pub const fn is_valid(&self) -> bool {
        self.start >= 0 && self.end >= self.start && !self.ttype.is_error()
    }
}

// =============================================================================
// Parse Result
// =============================================================================

/// Result of parsing an argument list
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ParseResult {
    /// Number of arguments found
    pub argc: c_int,
    /// Position of parse error (-1 if no error)
    pub error_pos: c_int,
    /// Whether parsing completed successfully
    pub success: bool,
    /// Whether there are more commands after bar
    pub has_bar: bool,
}

impl Default for ParseResult {
    fn default() -> Self {
        Self {
            argc: 0,
            error_pos: -1,
            success: true,
            has_bar: false,
        }
    }
}

impl ParseResult {
    /// Create a successful result
    pub const fn success(argc: c_int) -> Self {
        Self {
            argc,
            error_pos: -1,
            success: true,
            has_bar: false,
        }
    }

    /// Create an error result
    pub const fn error(pos: c_int) -> Self {
        Self {
            argc: 0,
            error_pos: pos,
            success: false,
            has_bar: false,
        }
    }

    /// Check if parse was successful
    pub const fn is_ok(&self) -> bool {
        self.success
    }

    /// Check if there was an error
    pub const fn is_err(&self) -> bool {
        !self.success
    }
}

// =============================================================================
// Character Classification
// =============================================================================

/// Check if character is whitespace for argument parsing
pub const fn is_arg_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Check if character starts a quote
pub const fn is_quote_start(c: u8) -> bool {
    c == b'\'' || c == b'"'
}

/// Check if character is escape
pub const fn is_escape_char(c: u8) -> bool {
    c == b'\\'
}

/// Check if character is command separator
pub const fn is_bar(c: u8) -> bool {
    c == b'|'
}

/// Check if character is special and needs escaping
pub const fn needs_escape(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\\' | b'"' | b'\'' | b'|' | b'<' | b'>')
}

/// Get the escaped form of a character (None if no escape needed)
pub const fn escape_char(c: u8) -> Option<u8> {
    match c {
        b'n' => Some(b'\n'),
        b'r' => Some(b'\r'),
        b't' => Some(b'\t'),
        b'e' => Some(0x1b), // ESC
        b'b' => Some(0x08), // Backspace
        _ => None,
    }
}

// =============================================================================
// Argument Counting
// =============================================================================

/// Count arguments in a string (simple whitespace split)
pub fn count_args(s: &[u8]) -> c_int {
    if s.is_empty() {
        return 0;
    }

    let mut count = 0;
    let mut in_word = false;
    let mut state = ParseState::Normal;

    for &c in s {
        match state {
            ParseState::Normal => {
                if is_arg_whitespace(c) {
                    in_word = false;
                } else if c == b'\'' {
                    if !in_word {
                        count += 1;
                    }
                    in_word = true;
                    state = ParseState::SingleQuote;
                } else if c == b'"' {
                    if !in_word {
                        count += 1;
                    }
                    in_word = true;
                    state = ParseState::DoubleQuote;
                } else if c == b'\\' {
                    if !in_word {
                        count += 1;
                        in_word = true;
                    }
                    state = ParseState::Escape;
                } else {
                    if !in_word {
                        count += 1;
                        in_word = true;
                    }
                }
            }
            ParseState::SingleQuote => {
                if c == b'\'' {
                    state = ParseState::Normal;
                }
            }
            ParseState::DoubleQuote => {
                if c == b'"' {
                    state = ParseState::Normal;
                } else if c == b'\\' {
                    state = ParseState::Escape;
                }
            }
            ParseState::Escape => {
                state = if matches!(state, ParseState::DoubleQuote) {
                    ParseState::DoubleQuote
                } else {
                    ParseState::Normal
                };
            }
            ParseState::BarSeparated => {
                // Stop counting at bar
                break;
            }
        }
    }

    count
}

// =============================================================================
// Quote Validation
// =============================================================================

/// Check if quotes are balanced in a string
pub fn quotes_balanced(s: &[u8]) -> bool {
    let mut state = ParseState::Normal;

    for &c in s {
        match state {
            ParseState::Normal => {
                if c == b'\'' {
                    state = ParseState::SingleQuote;
                } else if c == b'"' {
                    state = ParseState::DoubleQuote;
                } else if c == b'\\' {
                    state = ParseState::Escape;
                }
            }
            ParseState::SingleQuote => {
                if c == b'\'' {
                    state = ParseState::Normal;
                }
            }
            ParseState::DoubleQuote => {
                if c == b'"' {
                    state = ParseState::Normal;
                } else if c == b'\\' {
                    state = ParseState::Escape;
                }
            }
            ParseState::Escape => {
                state = ParseState::Normal;
            }
            ParseState::BarSeparated => {}
        }
    }

    matches!(state, ParseState::Normal)
}

// =============================================================================
// C FFI Accessor Declarations (Phase 2)
// =============================================================================

extern "C" {
    /// Call emsg() in C with a translatable message
    fn nvim_uc_emsg(msg: *const c_char);
    /// Call semsg() in C with one format argument
    fn nvim_uc_semsg_1(fmt: *const c_char, arg1: *const c_char);
    /// Call getdigits_int() in C — parses digits from *pp, advances *pp
    fn nvim_uc_getdigits_int(pp: *mut *mut c_char, maxlen: c_int, def: c_int) -> c_int;
    /// Call xstrnsave() in C — allocates a copy of the string
    fn nvim_uc_xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
}

// =============================================================================
// Helpers
// =============================================================================

/// Case-insensitive comparison of a byte slice against a literal for a given
/// length. Equivalent to `STRNICMP(attr, name, len) == 0`.
fn strnicmp_eq(a: &[u8], b: &[u8], len: usize) -> bool {
    if a.len() < len || b.len() < len {
        return false;
    }
    a[..len].eq_ignore_ascii_case(&b[..len])
}

// =============================================================================
// Pure Functions — uc_nargs_upper_bound, uc_split_args_iter
// =============================================================================

/// Count the upper bound on the number of arguments in a string.
///
/// This counts whitespace-separated words (where whitespace is space or tab).
/// It is an upper bound because escape sequences are not processed.
pub fn uc_nargs_upper_bound(arg: &[u8]) -> usize {
    let mut was_white = true;
    let mut nargs = 0usize;
    for &c in arg {
        let is_white = is_arg_whitespace(c);
        if was_white && !is_white {
            nargs += 1;
        }
        was_white = is_white;
    }
    nargs
}

/// Iterate over whitespace-separated arguments, processing backslash escapes.
///
/// On each call, extracts the next argument starting at position `*end`,
/// writing the unescaped content into `buf` and the length into `*len`.
/// Updates `*end` to point past the argument.
///
/// Returns `true` when iteration is complete (no more arguments),
/// `false` when there are more arguments to process.
///
/// Escape handling: `\\` → `\`, `\ ` → ` `, `\<tab>` → `<tab>`.
pub fn uc_split_args_iter(arg: &[u8], end: &mut usize, buf: &mut [u8]) -> (bool, usize) {
    let arglen = arg.len();
    if arglen == 0 {
        return (true, 0);
    }

    let mut pos = *end;
    // Skip leading whitespace
    while pos < arglen && is_arg_whitespace(arg[pos]) {
        pos += 1;
    }

    let mut l = 0usize;
    // Process characters up to arglen-1 (look-ahead for escape and whitespace)
    while pos < arglen.saturating_sub(1) {
        if arg[pos] == b'\\' && (arg[pos + 1] == b'\\' || is_arg_whitespace(arg[pos + 1])) {
            pos += 1;
        }
        buf[l] = arg[pos];
        l += 1;
        if is_arg_whitespace(arg[pos + 1]) {
            *end = pos + 1;
            return (false, l);
        }
        pos += 1;
    }

    // Handle the last character
    if pos < arglen && !is_arg_whitespace(arg[pos]) {
        buf[l] = arg[pos];
        l += 1;
    }

    (true, l)
}

// =============================================================================
// FFI-dependent: parse_addr_type_arg
// =============================================================================

/// Parse an address type argument value.
///
/// Searches the `ADDR_TYPE_COMPLETE` table for a match against
/// `value[0..vallen]`. On success, writes the matched address type to
/// `*addr_type_arg` and returns OK (1). On failure, calls `semsg` and
/// returns FAIL (0).
///
/// # Safety
/// `value` must point to a valid byte sequence of at least `vallen` bytes.
pub unsafe fn parse_addr_type_arg_impl(
    value: *const c_char,
    vallen: c_int,
    addr_type_arg: &mut c_int,
) -> c_int {
    let vallen_usize = vallen as usize;
    let value_slice = unsafe { std::slice::from_raw_parts(value.cast::<u8>(), vallen_usize) };

    for entry in ADDR_TYPE_COMPLETE {
        if entry.expand == AddrType::None {
            break;
        }
        // entry.name is NUL-terminated, get length without NUL
        let name = &entry.name[..entry.name.len() - 1];
        if name.len() == vallen_usize && value_slice == name {
            *addr_type_arg = entry.expand.as_raw();
            return 1; // OK
        }
    }

    // Not found — emit error
    // Build a NUL-terminated copy of the value for the error message.
    // In C, the code modifies the string in-place, but we use a stack buffer.
    let mut err_buf = [0u8; 256];
    let copy_len = vallen_usize.min(err_buf.len() - 1);
    err_buf[..copy_len].copy_from_slice(&value_slice[..copy_len]);
    // Truncate at first whitespace (matching C behavior)
    for (i, byte) in err_buf[..copy_len].iter_mut().enumerate() {
        if is_arg_whitespace(*byte) || *byte == 0 {
            err_buf[i] = 0;
            break;
        }
    }
    err_buf[copy_len] = 0;
    unsafe {
        nvim_uc_semsg_1(
            c"E180: Invalid address type value: %s".as_ptr(),
            err_buf.as_ptr().cast::<c_char>(),
        );
    }
    0 // FAIL
}

// =============================================================================
// FFI-dependent: parse_compl_arg
// =============================================================================

/// Parse a completion argument "value[vallen]".
///
/// The detected completion goes in `*complp`, argument type bits in `*argt`.
/// When there is an argument (for custom/customlist completion), it's copied
/// to allocated memory and stored in `*compl_arg`.
///
/// Returns OK (1) on success, FAIL (0) on error.
///
/// # Safety
/// `value` must point to a valid byte sequence of at least `vallen` bytes.
/// `complp`, `argt`, `compl_arg` must be valid pointers.
pub unsafe fn parse_compl_arg_impl(
    value: *const c_char,
    vallen: c_int,
    complp: &mut c_int,
    argt: &mut u32,
    compl_arg: &mut *mut c_char,
) -> c_int {
    let vallen_usize = vallen as usize;
    let value_slice = unsafe { std::slice::from_raw_parts(value.cast::<u8>(), vallen_usize) };

    // Look for any argument part — the part after any ','
    let comma_pos = value_slice.iter().position(|&c| c == b',');
    let arg_ptr: *const c_char =
        comma_pos.map_or(std::ptr::null(), |i| unsafe { value.add(i + 1) });
    let arglen: usize = comma_pos.map_or(0, |i| vallen_usize - i - 1);
    let valend = comma_pos.unwrap_or(vallen_usize);

    // Search the command_complete table
    let mut found = false;
    for (i, slot) in COMMAND_COMPLETE.iter().enumerate() {
        let Some(entry) = slot else {
            continue;
        };
        // entry is NUL-terminated, get length without NUL
        let name = &entry[..entry.len() - 1];
        if name.len() == valend && &value_slice[..valend] == name {
            *complp = i as c_int;
            if i as c_int == EXPAND_BUFFERS {
                *argt |= EX_BUFNAME;
            } else if i as c_int == EXPAND_DIRECTORIES
                || i as c_int == EXPAND_FILES
                || i as c_int == EXPAND_SHELLCMDLINE
            {
                *argt |= EX_XFILE;
            }
            found = true;
            break;
        }
    }

    if !found {
        unsafe {
            nvim_uc_semsg_1(c"E180: Invalid complete value: %s".as_ptr(), value);
        }
        return 0; // FAIL
    }

    if *complp != EXPAND_USER_DEFINED && *complp != EXPAND_USER_LIST && !arg_ptr.is_null() {
        unsafe {
            nvim_uc_emsg(c"E468: Completion argument only allowed for custom completion".as_ptr());
        }
        return 0; // FAIL
    }

    if (*complp == EXPAND_USER_DEFINED || *complp == EXPAND_USER_LIST) && arg_ptr.is_null() {
        unsafe {
            nvim_uc_emsg(c"E467: Custom completion requires a function argument".as_ptr());
        }
        return 0; // FAIL
    }

    if !arg_ptr.is_null() {
        *compl_arg = unsafe { nvim_uc_xstrnsave(arg_ptr, arglen) };
    }

    1 // OK
}

// =============================================================================
// FFI-dependent: uc_scan_attr
// =============================================================================

/// Scan a single `-attr` or `-attr=value` attribute for `:command` definition.
///
/// Reads `attr[0..len]` and updates the output parameters accordingly.
/// Returns OK (1) on success, FAIL (0) on error (after emitting an error
/// message via the C FFI).
///
/// # Safety
/// `attr` must point to a valid byte sequence of at least `len` bytes.
/// All output pointers must be valid.
pub unsafe fn uc_scan_attr_impl(
    attr: *mut c_char,
    len: usize,
    argt: &mut u32,
    def: &mut c_int,
    flags: &mut c_int,
    complp: &mut c_int,
    compl_arg: &mut *mut c_char,
    addr_type_arg: &mut c_int,
) -> c_int {
    if len == 0 {
        unsafe { nvim_uc_emsg(c"E175: No attribute specified".as_ptr()) };
        return 0; // FAIL
    }

    let attr_slice = unsafe { std::slice::from_raw_parts(attr.cast::<u8>(), len) };

    // Simple attributes (no arguments)
    if strnicmp_eq(attr_slice, b"bang", len) {
        *argt |= EX_BANG;
    } else if strnicmp_eq(attr_slice, b"buffer", len) {
        *flags |= UC_BUFFER;
    } else if strnicmp_eq(attr_slice, b"register", len) {
        *argt |= EX_REGSTR;
    } else if strnicmp_eq(attr_slice, b"keepscript", len) {
        *argt |= EX_KEEPSCRIPT;
    } else if strnicmp_eq(attr_slice, b"bar", len) {
        *argt |= EX_TRLBAR;
    } else {
        // Complex attributes: split at '='
        let eq_pos = attr_slice.iter().position(|&c| c == b'=');
        let attrlen = eq_pos.unwrap_or(len);

        let val_ptr: *mut c_char =
            eq_pos.map_or(std::ptr::null_mut(), |i| unsafe { attr.add(i + 1) });
        let vallen = eq_pos.map_or(0, |i| len - i - 1);

        if strnicmp_eq(attr_slice, b"nargs", attrlen) {
            if vallen == 1 {
                let val_byte = unsafe { *val_ptr.cast::<u8>() };
                match val_byte {
                    b'0' => { /* default, do nothing */ }
                    b'1' => *argt |= EX_EXTRA | EX_NOSPC | EX_NEEDARG,
                    b'*' => *argt |= EX_EXTRA,
                    b'?' => *argt |= EX_EXTRA | EX_NOSPC,
                    b'+' => *argt |= EX_EXTRA | EX_NEEDARG,
                    _ => {
                        unsafe { nvim_uc_emsg(c"E176: Invalid number of arguments".as_ptr()) };
                        return 0;
                    }
                }
            } else {
                unsafe { nvim_uc_emsg(c"E176: Invalid number of arguments".as_ptr()) };
                return 0;
            }
        } else if strnicmp_eq(attr_slice, b"range", attrlen) {
            *argt |= EX_RANGE;
            if vallen == 1 && unsafe { *val_ptr.cast::<u8>() } == b'%' {
                *argt |= EX_DFLALL;
            } else if !val_ptr.is_null() {
                if *def >= 0 {
                    unsafe { nvim_uc_emsg(c"E177: Count cannot be specified twice".as_ptr()) };
                    return 0;
                }
                let mut p = val_ptr;
                *def = unsafe { nvim_uc_getdigits_int(&mut p, 1, 0) };
                *argt |= EX_ZEROR;

                // Check that all characters were consumed and value is non-empty
                let consumed = unsafe { p.offset_from(val_ptr) } as usize;
                if consumed != vallen || vallen == 0 {
                    unsafe {
                        nvim_uc_emsg(c"E178: Invalid default value for count".as_ptr());
                    }
                    return 0;
                }
            }
            // default for -range is using buffer lines
            if *addr_type_arg == AddrType::None.as_raw() {
                *addr_type_arg = AddrType::Lines.as_raw();
            }
        } else if strnicmp_eq(attr_slice, b"count", attrlen) {
            *argt |= EX_COUNT | EX_ZEROR | EX_RANGE;
            // default for -count is using any number
            if *addr_type_arg == AddrType::None.as_raw() {
                *addr_type_arg = AddrType::Other.as_raw();
            }

            if !val_ptr.is_null() {
                if *def >= 0 {
                    unsafe { nvim_uc_emsg(c"E177: Count cannot be specified twice".as_ptr()) };
                    return 0;
                }
                let mut p = val_ptr;
                *def = unsafe { nvim_uc_getdigits_int(&mut p, 1, 0) };

                let consumed = unsafe { p.offset_from(val_ptr) } as usize;
                if consumed != vallen {
                    unsafe {
                        nvim_uc_emsg(c"E178: Invalid default value for count".as_ptr());
                    }
                    return 0;
                }
            }

            if *def < 0 {
                *def = 0;
            }
        } else if strnicmp_eq(attr_slice, b"complete", attrlen) {
            if val_ptr.is_null() {
                unsafe {
                    nvim_uc_semsg_1(
                        c"E179: Argument required for %s".as_ptr(),
                        c"-complete".as_ptr(),
                    );
                }
                return 0;
            }
            let rc =
                unsafe { parse_compl_arg_impl(val_ptr, vallen as c_int, complp, argt, compl_arg) };
            if rc == 0 {
                return 0;
            }
        } else if strnicmp_eq(attr_slice, b"addr", attrlen) {
            *argt |= EX_RANGE;
            if val_ptr.is_null() {
                unsafe {
                    nvim_uc_semsg_1(
                        c"E179: Argument required for %s".as_ptr(),
                        c"-addr".as_ptr(),
                    );
                }
                return 0;
            }
            let rc = unsafe { parse_addr_type_arg_impl(val_ptr, vallen as c_int, addr_type_arg) };
            if rc == 0 {
                return 0;
            }
            if *addr_type_arg != AddrType::Lines.as_raw() {
                *argt |= EX_ZEROR;
            }
        } else {
            // Unknown attribute — emit error
            // We need to NUL-terminate the attribute name temporarily for the
            // error message, matching the C behavior of temporarily writing NUL.
            let saved = unsafe { *attr.add(len) };
            unsafe { *attr.add(len) = 0 };
            unsafe {
                nvim_uc_semsg_1(c"E181: Invalid attribute: %s".as_ptr(), attr);
            }
            unsafe { *attr.add(len) = saved };
            return 0;
        }
    }

    1 // OK
}

// =============================================================================
// FFI Exports — Phase 1
// =============================================================================

/// FFI export: Check if character is whitespace
#[no_mangle]
pub extern "C" fn rs_usercmd_is_arg_whitespace(c: u8) -> c_int {
    c_int::from(is_arg_whitespace(c))
}

/// FFI export: Check if character is quote start
#[no_mangle]
pub extern "C" fn rs_usercmd_is_quote_start(c: u8) -> c_int {
    c_int::from(is_quote_start(c))
}

/// FFI export: Check if character needs escaping
#[no_mangle]
pub extern "C" fn rs_usercmd_needs_escape(c: u8) -> c_int {
    c_int::from(needs_escape(c))
}

/// FFI export: Check if parse state is quoted
#[no_mangle]
pub extern "C" fn rs_usercmd_parse_state_is_quoted(state: c_int) -> c_int {
    ParseState::from_raw(state).map_or(0, |s| c_int::from(s.is_quoted()))
}

/// FFI export: Check if token type is string
#[no_mangle]
pub extern "C" fn rs_usercmd_token_is_string(ttype: c_int) -> c_int {
    let ttype = match ttype {
        0 => TokenType::Word,
        2 => TokenType::SingleQuoted,
        3 => TokenType::DoubleQuoted,
        _ => TokenType::End,
    };
    c_int::from(ttype.is_string())
}

/// FFI export: Create a token
#[no_mangle]
pub extern "C" fn rs_usercmd_token_new(ttype: c_int, start: c_int, end: c_int) -> Token {
    let token_type = match ttype {
        0 => TokenType::Word,
        1 => TokenType::Whitespace,
        2 => TokenType::SingleQuoted,
        3 => TokenType::DoubleQuoted,
        4 => TokenType::Bar,
        5 => TokenType::End,
        _ => TokenType::Error,
    };
    Token::new(token_type, start, end)
}

/// FFI export: Get token length
#[no_mangle]
pub extern "C" fn rs_usercmd_token_len(token: *const Token) -> c_int {
    if token.is_null() {
        return 0;
    }
    unsafe { (*token).len() }
}

/// FFI export: Check if token is valid
#[no_mangle]
pub extern "C" fn rs_usercmd_token_is_valid(token: *const Token) -> c_int {
    if token.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*token).is_valid() })
}

/// FFI export: Create successful parse result
#[no_mangle]
pub extern "C" fn rs_usercmd_parse_result_success(argc: c_int) -> ParseResult {
    ParseResult::success(argc)
}

/// FFI export: Create error parse result
#[no_mangle]
pub extern "C" fn rs_usercmd_parse_result_error(pos: c_int) -> ParseResult {
    ParseResult::error(pos)
}

// =============================================================================
// FFI Exports — Phase 2
// =============================================================================

/// FFI export: Count upper bound of arguments in a string.
///
/// `arg` is a pointer to the argument string, `arglen` is the byte length.
#[no_mangle]
pub unsafe extern "C" fn rs_uc_nargs_upper_bound(arg: *const c_char, arglen: usize) -> usize {
    if arg.is_null() || arglen == 0 {
        return 0;
    }
    let slice = unsafe { std::slice::from_raw_parts(arg.cast::<u8>(), arglen) };
    uc_nargs_upper_bound(slice)
}

/// FFI export: Iterate over split arguments.
///
/// Returns 1 when iteration is complete (no more args), 0 when there are more.
/// Writes the unescaped argument into `buf` and its length into `*len`.
/// `*end` tracks the iteration position across calls.
#[no_mangle]
pub unsafe extern "C" fn rs_uc_split_args_iter(
    arg: *const c_char,
    arglen: usize,
    end: *mut usize,
    buf: *mut c_char,
    len: *mut usize,
) -> c_int {
    if arg.is_null() || arglen == 0 {
        if !len.is_null() {
            unsafe { *len = 0 };
        }
        return 1; // done
    }
    let arg_slice = unsafe { std::slice::from_raw_parts(arg.cast::<u8>(), arglen) };
    let end_ref = unsafe { &mut *end };

    // Use a large stack buffer to receive the output, then copy to caller's buf
    let mut local_buf = [0u8; 4096];
    let (done, out_len) = uc_split_args_iter(arg_slice, end_ref, &mut local_buf);

    // Copy to caller's buffer
    if !buf.is_null() && out_len > 0 {
        unsafe {
            std::ptr::copy_nonoverlapping(local_buf.as_ptr(), buf.cast::<u8>(), out_len);
        }
    }
    if !len.is_null() {
        unsafe { *len = out_len };
    }

    c_int::from(done)
}

/// FFI export: Parse address type argument.
///
/// Returns OK (1) on success, FAIL (0) on error.
#[export_name = "parse_addr_type_arg"]
pub unsafe extern "C" fn rs_parse_addr_type_arg(
    value: *const c_char,
    vallen: c_int,
    addr_type_arg: *mut c_int,
) -> c_int {
    if value.is_null() || addr_type_arg.is_null() {
        return 0;
    }
    unsafe { parse_addr_type_arg_impl(value, vallen, &mut *addr_type_arg) }
}

/// FFI export: Parse completion argument.
///
/// Returns OK (1) on success, FAIL (0) on error.
#[export_name = "parse_compl_arg"]
pub unsafe extern "C" fn rs_parse_compl_arg(
    value: *const c_char,
    vallen: c_int,
    complp: *mut c_int,
    argt: *mut u32,
    compl_arg: *mut *mut c_char,
) -> c_int {
    if value.is_null() || complp.is_null() || argt.is_null() || compl_arg.is_null() {
        return 0;
    }
    unsafe { parse_compl_arg_impl(value, vallen, &mut *complp, &mut *argt, &mut *compl_arg) }
}

/// FFI export: Scan a single attribute for `:command` definition.
///
/// Returns OK (1) on success, FAIL (0) on error.
#[export_name = "uc_scan_attr"]
pub unsafe extern "C" fn rs_uc_scan_attr(
    attr: *mut c_char,
    len: usize,
    argt: *mut u32,
    def: *mut c_int,
    flags: *mut c_int,
    complp: *mut c_int,
    compl_arg: *mut *mut c_char,
    addr_type_arg: *mut c_int,
) -> c_int {
    if attr.is_null()
        || argt.is_null()
        || def.is_null()
        || flags.is_null()
        || complp.is_null()
        || compl_arg.is_null()
        || addr_type_arg.is_null()
    {
        return 0;
    }
    unsafe {
        uc_scan_attr_impl(
            attr,
            len,
            &mut *argt,
            &mut *def,
            &mut *flags,
            &mut *complp,
            &mut *compl_arg,
            &mut *addr_type_arg,
        )
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_state() {
        assert!(!ParseState::Normal.is_quoted());
        assert!(ParseState::SingleQuote.is_quoted());
        assert!(ParseState::DoubleQuote.is_quoted());
        assert!(ParseState::Escape.is_escape());

        assert_eq!(ParseState::from_raw(0), Some(ParseState::Normal));
        assert_eq!(ParseState::from_raw(100), None);
    }

    #[test]
    fn test_token_type() {
        assert!(TokenType::Word.is_string());
        assert!(TokenType::SingleQuoted.is_string());
        assert!(!TokenType::Whitespace.is_string());

        assert!(TokenType::Whitespace.is_separator());
        assert!(TokenType::Bar.is_separator());
        assert!(!TokenType::Word.is_separator());
    }

    #[test]
    fn test_token() {
        let token = Token::new(TokenType::Word, 0, 5);
        assert_eq!(token.len(), 5);
        assert!(!token.is_empty());
        assert!(token.is_valid());

        let empty = Token::new(TokenType::Word, 5, 5);
        assert!(empty.is_empty());

        let error = Token::new(TokenType::Error, 0, 1);
        assert!(!error.is_valid());
    }

    #[test]
    fn test_parse_result() {
        let success = ParseResult::success(3);
        assert!(success.is_ok());
        assert!(!success.is_err());
        assert_eq!(success.argc, 3);

        let error = ParseResult::error(10);
        assert!(!error.is_ok());
        assert!(error.is_err());
        assert_eq!(error.error_pos, 10);
    }

    #[test]
    fn test_char_classification() {
        assert!(is_arg_whitespace(b' '));
        assert!(is_arg_whitespace(b'\t'));
        assert!(!is_arg_whitespace(b'a'));

        assert!(is_quote_start(b'\''));
        assert!(is_quote_start(b'"'));
        assert!(!is_quote_start(b'a'));

        assert!(is_escape_char(b'\\'));
        assert!(!is_escape_char(b'/'));

        assert!(is_bar(b'|'));
        assert!(!is_bar(b'-'));
    }

    #[test]
    fn test_needs_escape() {
        assert!(needs_escape(b' '));
        assert!(needs_escape(b'\\'));
        assert!(needs_escape(b'"'));
        assert!(!needs_escape(b'a'));
    }

    #[test]
    fn test_escape_char() {
        assert_eq!(escape_char(b'n'), Some(b'\n'));
        assert_eq!(escape_char(b't'), Some(b'\t'));
        assert_eq!(escape_char(b'a'), None);
    }

    #[test]
    fn test_count_args() {
        assert_eq!(count_args(b""), 0);
        assert_eq!(count_args(b"one"), 1);
        assert_eq!(count_args(b"one two"), 2);
        assert_eq!(count_args(b"one two three"), 3);
        assert_eq!(count_args(b"  one  two  "), 2);
        assert_eq!(count_args(b"'one two'"), 1);
        assert_eq!(count_args(b"\"one two\""), 1);
    }

    #[test]
    fn test_quotes_balanced() {
        assert!(quotes_balanced(b"no quotes"));
        assert!(quotes_balanced(b"'single'"));
        assert!(quotes_balanced(b"\"double\""));
        assert!(quotes_balanced(b"'a' \"b\""));
        assert!(!quotes_balanced(b"'unbalanced"));
        assert!(!quotes_balanced(b"\"unbalanced"));
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_usercmd_is_arg_whitespace(b' '), 1);
        assert_eq!(rs_usercmd_is_arg_whitespace(b'a'), 0);

        assert_eq!(rs_usercmd_is_quote_start(b'"'), 1);
        assert_eq!(rs_usercmd_is_quote_start(b'a'), 0);

        assert_eq!(rs_usercmd_needs_escape(b' '), 1);
        assert_eq!(rs_usercmd_needs_escape(b'a'), 0);

        assert_eq!(rs_usercmd_parse_state_is_quoted(1), 1);
        assert_eq!(rs_usercmd_parse_state_is_quoted(0), 0);

        let token = rs_usercmd_token_new(0, 0, 5);
        assert_eq!(rs_usercmd_token_len(&token), 5);
    }

    // =========================================================================
    // uc_nargs_upper_bound tests
    // =========================================================================

    #[test]
    fn test_nargs_upper_bound_empty() {
        assert_eq!(uc_nargs_upper_bound(b""), 0);
    }

    #[test]
    fn test_nargs_upper_bound_single() {
        assert_eq!(uc_nargs_upper_bound(b"hello"), 1);
    }

    #[test]
    fn test_nargs_upper_bound_two() {
        assert_eq!(uc_nargs_upper_bound(b"hello world"), 2);
    }

    #[test]
    fn test_nargs_upper_bound_three() {
        assert_eq!(uc_nargs_upper_bound(b"a b c"), 3);
    }

    #[test]
    fn test_nargs_upper_bound_leading_whitespace() {
        assert_eq!(uc_nargs_upper_bound(b"  hello"), 1);
    }

    #[test]
    fn test_nargs_upper_bound_trailing_whitespace() {
        assert_eq!(uc_nargs_upper_bound(b"hello  "), 1);
    }

    #[test]
    fn test_nargs_upper_bound_multiple_spaces() {
        assert_eq!(uc_nargs_upper_bound(b"a   b   c"), 3);
    }

    #[test]
    fn test_nargs_upper_bound_tabs() {
        assert_eq!(uc_nargs_upper_bound(b"a\tb\tc"), 3);
    }

    #[test]
    fn test_nargs_upper_bound_mixed_whitespace() {
        assert_eq!(uc_nargs_upper_bound(b"a \t b"), 2);
    }

    #[test]
    fn test_nargs_upper_bound_only_whitespace() {
        assert_eq!(uc_nargs_upper_bound(b"   "), 0);
    }

    // =========================================================================
    // uc_split_args_iter tests
    // =========================================================================

    #[test]
    fn test_split_args_iter_empty() {
        let mut end = 0usize;
        let mut buf = [0u8; 64];
        let (done, len) = uc_split_args_iter(b"", &mut end, &mut buf);
        assert!(done);
        assert_eq!(len, 0);
    }

    #[test]
    fn test_split_args_iter_single_word() {
        let arg = b"hello";
        let mut end = 0usize;
        let mut buf = [0u8; 64];
        let (done, len) = uc_split_args_iter(arg, &mut end, &mut buf);
        assert!(done);
        assert_eq!(len, 5);
        assert_eq!(&buf[..5], b"hello");
    }

    #[test]
    fn test_split_args_iter_two_words() {
        let arg = b"hello world";
        let mut end = 0usize;
        let mut buf = [0u8; 64];

        // First iteration: "hello"
        let (done, len) = uc_split_args_iter(arg, &mut end, &mut buf);
        assert!(!done);
        assert_eq!(len, 5);
        assert_eq!(&buf[..5], b"hello");

        // Second iteration: "world"
        let (done, len) = uc_split_args_iter(arg, &mut end, &mut buf);
        assert!(done);
        assert_eq!(len, 5);
        assert_eq!(&buf[..5], b"world");
    }

    #[test]
    fn test_split_args_iter_three_words() {
        let arg = b"a bb ccc";
        let mut end = 0usize;
        let mut buf = [0u8; 64];

        let (done, len) = uc_split_args_iter(arg, &mut end, &mut buf);
        assert!(!done);
        assert_eq!(len, 1);
        assert_eq!(&buf[..1], b"a");

        let (done, len) = uc_split_args_iter(arg, &mut end, &mut buf);
        assert!(!done);
        assert_eq!(len, 2);
        assert_eq!(&buf[..2], b"bb");

        let (done, len) = uc_split_args_iter(arg, &mut end, &mut buf);
        assert!(done);
        assert_eq!(len, 3);
        assert_eq!(&buf[..3], b"ccc");
    }

    #[test]
    fn test_split_args_iter_escaped_backslash() {
        // "a\\ b" → arg1="a\", arg2="b"
        let arg = b"a\\\\ b";
        let mut end = 0usize;
        let mut buf = [0u8; 64];

        let (done, len) = uc_split_args_iter(arg, &mut end, &mut buf);
        assert!(!done);
        assert_eq!(len, 2);
        assert_eq!(&buf[..2], b"a\\");

        let (done, len) = uc_split_args_iter(arg, &mut end, &mut buf);
        assert!(done);
        assert_eq!(len, 1);
        assert_eq!(&buf[..1], b"b");
    }

    #[test]
    fn test_split_args_iter_escaped_space() {
        // "a\ b c" → arg1="a b", arg2="c"
        let arg = b"a\\ b c";
        let mut end = 0usize;
        let mut buf = [0u8; 64];

        // The escaped space joins the words
        let (done, len) = uc_split_args_iter(arg, &mut end, &mut buf);
        assert!(!done);
        assert_eq!(&buf[..len], b"a b");

        let (done, len) = uc_split_args_iter(arg, &mut end, &mut buf);
        assert!(done);
        assert_eq!(&buf[..len], b"c");
    }

    #[test]
    fn test_split_args_iter_leading_whitespace() {
        let arg = b"  hello";
        let mut end = 0usize;
        let mut buf = [0u8; 64];

        let (done, len) = uc_split_args_iter(arg, &mut end, &mut buf);
        assert!(done);
        assert_eq!(len, 5);
        assert_eq!(&buf[..5], b"hello");
    }

    // =========================================================================
    // strnicmp_eq tests
    // =========================================================================

    #[test]
    fn test_strnicmp_eq_basic() {
        assert!(strnicmp_eq(b"BANG", b"bang", 4));
        assert!(strnicmp_eq(b"Bang", b"bang", 4));
        assert!(strnicmp_eq(b"bang", b"bang", 4));
        assert!(!strnicmp_eq(b"ban", b"bang", 4));
        assert!(!strnicmp_eq(b"bong", b"bang", 4));
    }

    #[test]
    fn test_strnicmp_eq_partial() {
        // Only compare first 3 bytes
        assert!(strnicmp_eq(b"BAN", b"bang", 3));
        assert!(strnicmp_eq(b"BANG_EXTRA", b"bang", 4));
    }

    #[test]
    fn test_strnicmp_eq_short_input() {
        assert!(!strnicmp_eq(b"ba", b"bang", 4));
    }
}
