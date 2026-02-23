//! `:substitute` command implementation.
//!
//! The `:substitute` (`:s`) command performs search and replace operations
//! on buffer text using regular expressions.
//!
//! ## Usage
//! - `:s/pattern/replacement/` - Substitute on current line
//! - `:%s/pattern/replacement/g` - Substitute all occurrences in buffer
//! - `:{range}s/pattern/replacement/flags` - Substitute in range with flags
//!
//! ## Flags
//! - `g` - Global: replace all occurrences on each line
//! - `c` - Confirm: ask for confirmation for each replacement
//! - `i` - Ignore case
//! - `I` - Don't ignore case (match case)
//! - `n` - Count only, don't substitute
//! - `e` - Don't report errors if no match
//! - `p` - Print the last line with a substitution
//! - `l` - Like 'p' but list the line
//! - `#` - Like 'p' but show line number
//!
//! ## Implementation Notes
//!
//! This module provides type definitions and flag parsing. The actual
//! regex matching and text replacement is performed by Neovim's core
//! substitution engine.

use std::ffi::{c_char, c_int};

use crate::range::LineNr;
use crate::ExArgHandle;
use crate::SubIgnoreType;

// =============================================================================
// Phase 3: sub_parse_flags migration constants
// =============================================================================

/// Pattern type: use last used regexp (RE_LAST from search.h)
const RE_LAST: c_int = 2;

/// Pattern type: RE_SUBST (save as substitute pattern)
const RE_SUBST: c_int = 1;

/// EXFLAG constants (must match C defines)
const EXFLAG_LIST: c_int = 0x01;
const EXFLAG_NR: c_int = 0x02;
const EXFLAG_PRINT: c_int = 0x04;

extern "C" {
    /// Get the current value of p_gd (gdefault option).
    fn nvim_option_get_gd() -> c_int;

    // sub_joining_lines FFI
    fn nvim_exarg_get_skip(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_get_line1(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_get_line2(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_set_flags(eap: *mut ExArgHandle, flags: c_int);
    fn nvim_curwin_set_cursor_lnum(lnum: c_int);
    fn nvim_curbuf_get_b_ml_ml_line_count() -> c_int;
    fn nvim_excmds_do_join(count: c_int) -> c_int;
    fn nvim_excmds_set_sub_nsubs(val: c_int);
    fn nvim_excmds_set_sub_nlines(val: c_int);
    fn nvim_excmds_ex_may_print(eap: *mut ExArgHandle);
    fn nvim_excmds_save_re_pat(idx: c_int, pat: *const c_char, patlen: usize, magic: c_int);
    fn nvim_excmds_add_to_hist_search(pat: *const c_char, patlen: usize);
    fn rs_magic_isset() -> c_int;

    // do_sub_msg FFI -- control flow in Rust, formatting/messaging in C
    /// Return sub_nsubs global.
    fn nvim_excmds_get_sub_nsubs() -> c_int;
    /// Return sub_nlines global.
    fn nvim_excmds_get_sub_nlines() -> c_int;
    /// Return p_report option value.
    fn nvim_excmds_p_report() -> i64;
    /// Return KeyTyped global.
    fn nvim_excmds_get_KeyTyped() -> c_int;
    /// Return messaging() result (1 = messaging on, 0 = off).
    fn nvim_excmds_messaging() -> c_int;
    /// Return got_int global.
    fn nvim_excmds_got_int() -> c_int;
    /// Format and display the substitution count message (NGETTEXT in C).
    /// Returns true if message was displayed.
    fn nvim_excmds_format_sub_msg(count_only: c_int) -> c_int;
    /// emsg(_(e_interr)) wrapper.
    fn nvim_excmds_emsg_interr();

    // sub_grow_buf FFI
    fn xcalloc(count: usize, size: usize) -> *mut std::ffi::c_void;
    fn xrealloc(ptr: *mut std::ffi::c_void, size: usize) -> *mut std::ffi::c_void;
}

/// C-compatible layout for subflags_T.
///
/// Must match the C struct exactly:
/// ```c
/// typedef struct {
///   bool do_all;
///   bool do_ask;
///   bool do_count;
///   bool do_error;
///   bool do_print;
///   bool do_list;
///   bool do_number;
///   SubIgnoreType do_ic;  // int enum, aligned to 4 bytes
/// } subflags_T;
/// ```
#[repr(C)]
pub struct CSubFlags {
    pub do_all: bool,
    pub do_ask: bool,
    pub do_count: bool,
    pub do_error: bool,
    pub do_print: bool,
    pub do_list: bool,
    pub do_number: bool,
    pub do_ic: c_int, // SubIgnoreType as int
}

/// Flags for the `:substitute` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubFlags {
    /// Do multiple substitutions per line (g flag).
    pub do_all: bool,
    /// Ask for confirmation (c flag).
    pub do_ask: bool,
    /// Count only, don't substitute (n flag).
    pub do_count: bool,
    /// If false, ignore errors when no match (e flag).
    pub do_error: bool,
    /// Print last line with subs (p flag).
    pub do_print: bool,
    /// List last line with subs (l flag).
    pub do_list: bool,
    /// List last line with line number (# flag).
    pub do_number: bool,
    /// Case sensitivity mode.
    pub do_ic: SubIgnoreType,
}

impl SubFlags {
    /// Create flags with default values (honor options, errors enabled).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            do_all: false,
            do_ask: false,
            do_count: false,
            do_error: true,
            do_print: false,
            do_list: false,
            do_number: false,
            do_ic: SubIgnoreType::HonorOptions,
        }
    }

    /// Create flags for a simple global substitute.
    #[must_use]
    pub const fn global() -> Self {
        Self {
            do_all: true,
            do_error: true,
            do_ask: false,
            do_count: false,
            do_print: false,
            do_list: false,
            do_number: false,
            do_ic: SubIgnoreType::HonorOptions,
        }
    }

    /// Create flags for count-only mode.
    #[must_use]
    pub const fn count_only() -> Self {
        Self {
            do_all: true,
            do_count: true,
            do_error: true,
            do_ask: false,
            do_print: false,
            do_list: false,
            do_number: false,
            do_ic: SubIgnoreType::HonorOptions,
        }
    }

    /// Parse flags from a flag string.
    ///
    /// # Arguments
    /// * `flags` - A string containing flag characters (e.g., "gc" for global + confirm)
    ///
    /// # Returns
    /// The parsed flags, or an error if an invalid flag is found.
    pub fn parse(flags: &str) -> Result<Self, SubstituteError> {
        let mut result = Self::new();

        for c in flags.chars() {
            match c {
                'g' => result.do_all = true,
                'c' => result.do_ask = true,
                'n' => result.do_count = true,
                'e' => result.do_error = false,
                'p' => result.do_print = true,
                'l' => result.do_list = true,
                '#' => result.do_number = true,
                'i' => result.do_ic = SubIgnoreType::IgnoreCase,
                'I' => result.do_ic = SubIgnoreType::MatchCase,
                'r' => { /* use last search pattern - handled elsewhere */ }
                ' ' | '\t' => { /* skip whitespace */ }
                _ => return Err(SubstituteError::InvalidFlag(c)),
            }
        }

        Ok(result)
    }

    /// Check if this is a counting-only operation.
    #[inline]
    #[must_use]
    pub const fn is_count_only(&self) -> bool {
        self.do_count
    }

    /// Check if confirmation is required.
    #[inline]
    #[must_use]
    pub const fn needs_confirm(&self) -> bool {
        self.do_ask
    }
}

/// Result of a substitution operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubResult {
    /// Number of substitutions made.
    pub count: i32,
    /// Number of lines changed.
    pub lines: i32,
    /// Whether the operation was interrupted.
    pub interrupted: bool,
}

impl SubResult {
    /// Create a new empty result.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            count: 0,
            lines: 0,
            interrupted: false,
        }
    }

    /// Check if any substitutions were made.
    #[inline]
    #[must_use]
    pub const fn has_matches(&self) -> bool {
        self.count > 0
    }

    /// Add a match to the result.
    #[inline]
    pub fn add_match(&mut self) {
        self.count += 1;
    }

    /// Record a changed line.
    #[inline]
    pub fn add_line(&mut self) {
        self.lines += 1;
    }

    /// Mark as interrupted.
    #[inline]
    pub fn set_interrupted(&mut self) {
        self.interrupted = true;
    }
}

/// Statistics for a substitution operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubStats {
    /// Total number of substitutions across all operations.
    pub total_subs: i32,
    /// Total number of lines changed across all operations.
    pub total_lines: i32,
}

impl SubStats {
    /// Create new statistics.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_subs: 0,
            total_lines: 0,
        }
    }

    /// Add a result to the statistics.
    pub fn add_result(&mut self, result: &SubResult) {
        self.total_subs += result.count;
        self.total_lines += result.lines;
    }

    /// Reset the statistics.
    pub fn reset(&mut self) {
        self.total_subs = 0;
        self.total_lines = 0;
    }
}

/// Error type for substitution operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstituteError {
    /// Invalid flag character.
    InvalidFlag(char),
    /// Invalid delimiter (alphanumeric).
    InvalidDelimiter(char),
    /// Empty pattern with no previous pattern.
    NoPreviousPattern,
    /// Empty replacement with no previous replacement.
    NoPreviousReplacement,
    /// Invalid regular expression.
    InvalidRegex(String),
    /// Invalid range.
    InvalidRange,
    /// Zero count given.
    ZeroCount,
    /// Operation was interrupted.
    Interrupted,
}

impl std::fmt::Display for SubstituteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubstituteError::InvalidFlag(c) => write!(f, "invalid flag: {c}"),
            SubstituteError::InvalidDelimiter(c) => {
                write!(f, "regular expressions can't be delimited by letters: {c}")
            }
            SubstituteError::NoPreviousPattern => write!(f, "no previous pattern"),
            SubstituteError::NoPreviousReplacement => write!(f, "no previous substitute command"),
            SubstituteError::InvalidRegex(msg) => write!(f, "invalid regex: {msg}"),
            SubstituteError::InvalidRange => write!(f, "invalid range"),
            SubstituteError::ZeroCount => write!(f, "zero count"),
            SubstituteError::Interrupted => write!(f, "interrupted"),
        }
    }
}

impl std::error::Error for SubstituteError {}

/// Check if a character is a valid delimiter.
///
/// Delimiters cannot be alphanumeric characters.
#[inline]
#[must_use]
pub fn is_valid_delimiter(c: char) -> bool {
    !c.is_alphanumeric()
}

/// Position within a match result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MatchPosition {
    /// Line number (1-based).
    pub lnum: LineNr,
    /// Column offset (0-based).
    pub col: i32,
}

impl MatchPosition {
    /// Create a new match position.
    #[must_use]
    pub const fn new(lnum: LineNr, col: i32) -> Self {
        Self { lnum, col }
    }
}

/// A range of matched text for preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MatchRange {
    /// Start position.
    pub start: MatchPosition,
    /// End position.
    pub end: MatchPosition,
}

impl MatchRange {
    /// Create a new match range.
    #[must_use]
    pub const fn new(start: MatchPosition, end: MatchPosition) -> Self {
        Self { start, end }
    }

    /// Check if this is a single-line match.
    #[inline]
    #[must_use]
    pub const fn is_single_line(&self) -> bool {
        self.start.lnum == self.end.lnum
    }

    /// Get the number of lines spanned by this match.
    #[inline]
    #[must_use]
    pub const fn line_span(&self) -> LineNr {
        self.end.lnum - self.start.lnum + 1
    }
}

// =============================================================================
// Phase 1: sub_joining_lines + sub_grow_buf
// =============================================================================

/// Recognize `:%s/\n//` and turn it into a join command, which is much
/// more efficient. Replaces the C `sub_joining_lines` static function.
///
/// Returns 1 (true) if the substitute can be replaced with a join, 0 (false) otherwise.
///
/// # Safety
/// All pointer arguments must be non-null and point to valid C strings.
/// `eap` must be a valid exarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_sub_joining_lines(
    eap: *mut ExArgHandle,
    pat: *const c_char,
    patlen: usize,
    sub: *const c_char,
    cmd: *const c_char,
    save: c_int,
    keeppatterns: c_int,
) -> c_int {
    use std::ffi::CStr;

    // pat must be non-null and equal to "\\n", sub must be NUL, and
    // cmd must be NUL or a single one of 'g', 'l', 'p', '#'.
    if pat.is_null() {
        return 0;
    }

    let pat_bytes = CStr::from_ptr(pat).to_bytes();
    if pat_bytes != b"\\n" {
        return 0;
    }

    // *sub == NUL
    if *sub != 0 {
        return 0;
    }

    // cmd must be NUL or a single recognized flag
    let cmd0 = *cmd as u8;
    let cmd1 = if cmd0 != 0 { *cmd.add(1) as u8 } else { 0 };
    if cmd0 != 0 && (cmd1 != 0 || !matches!(cmd0, b'g' | b'l' | b'p' | b'#')) {
        return 0;
    }

    // Pattern matches - this is a join-optimization candidate
    if nvim_exarg_get_skip(eap) != 0 {
        return 1; // skip mode: pretend we handled it
    }

    let line1 = nvim_exarg_get_line1(eap);
    let line2 = nvim_exarg_get_line2(eap);

    nvim_curwin_set_cursor_lnum(line1);

    // Set eap->flags based on cmd character
    match cmd0 {
        b'l' => nvim_exarg_set_flags(eap, EXFLAG_LIST),
        b'#' => nvim_exarg_set_flags(eap, EXFLAG_NR),
        b'p' => nvim_exarg_set_flags(eap, EXFLAG_PRINT),
        _ => {}
    }

    let ml_line_count = nvim_curbuf_get_b_ml_ml_line_count();
    let joined_lines_count = (line2 - line1 + 1) + if line2 < ml_line_count { 1 } else { 0 };

    if joined_lines_count > 1 {
        nvim_excmds_do_join(joined_lines_count);
        nvim_excmds_set_sub_nsubs(joined_lines_count - 1);
        nvim_excmds_set_sub_nlines(1);
        rs_do_sub_msg(false);
        nvim_excmds_ex_may_print(eap);
    }

    if save != 0 {
        if keeppatterns == 0 {
            nvim_excmds_save_re_pat(RE_SUBST, pat, patlen, rs_magic_isset());
        }
        nvim_excmds_add_to_hist_search(pat, patlen);
    }

    1 // true: substitute was handled as join
}

/// Allocate or grow the replacement text buffer for :substitute.
/// Replaces the C `sub_grow_buf` static function.
///
/// - If `*new_start` is null: allocates a new buffer of size `needed_len + 50`,
///   zero-initialized, and returns pointer to the start.
/// - Otherwise: if the existing buffer is too small, reallocates it.
///   Returns pointer to the current end of the string in the buffer.
///
/// # Safety
/// `new_start` and `new_start_len` must be non-null valid pointers.
/// `*new_start` must be null or point to an xmalloc-allocated buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_sub_grow_buf(
    new_start: *mut *mut c_char,
    new_start_len: *mut c_int,
    needed_len: c_int,
) -> *mut c_char {
    if (*new_start).is_null() {
        // Initial allocation: needed_len + 50 extra bytes, zero-initialized.
        let alloc_len = (needed_len + 50) as usize;
        *new_start_len = needed_len + 50;
        let ptr = xcalloc(1, alloc_len) as *mut c_char;
        *ptr = 0; // ensure NUL-terminated
        *new_start = ptr;
        ptr
    } else {
        // Find current length of string in the buffer (strlen).
        let current_ptr = *new_start;
        let mut len = 0usize;
        while *current_ptr.add(len) != 0 {
            len += 1;
        }
        let needed = needed_len + len as c_int;
        if needed > *new_start_len {
            let prev_len = *new_start_len as usize;
            *new_start_len = needed + 50;
            let new_alloc_len = *new_start_len as usize;
            *new_start =
                xrealloc(*new_start as *mut std::ffi::c_void, new_alloc_len) as *mut c_char;
            // Zero out the newly added region
            let added = new_alloc_len - prev_len;
            std::ptr::write_bytes((*new_start).add(prev_len), 0, added);
        }
        (*new_start).add(len)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Give message for number of substitutions.
///
/// Replaces the C `do_sub_msg` function. Contains the control-flow logic;
/// delegates NGETTEXT formatting to `nvim_excmds_format_sub_msg` which keeps
/// internationalization in C.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_do_sub_msg(count_only: bool) -> bool {
    let sub_nsubs = nvim_excmds_get_sub_nsubs();
    let sub_nlines = nvim_excmds_get_sub_nlines();
    let p_report = nvim_excmds_p_report();
    let key_typed = nvim_excmds_get_KeyTyped() != 0;
    let messaging = nvim_excmds_messaging() != 0;
    let got_int = nvim_excmds_got_int() != 0;

    let threshold_met = (sub_nsubs as i64 > p_report
        && (key_typed || sub_nlines > 1 || p_report < 1))
        || count_only;

    if threshold_met && messaging {
        nvim_excmds_format_sub_msg(c_int::from(count_only));
        return true;
    }
    if got_int {
        nvim_excmds_emsg_interr();
        return true;
    }
    false
}

/// Parse :substitute flags from a C command string. Replaces the C
/// `sub_parse_flags` function.
///
/// - If `*cmd == '&'`, advances cmd and keeps existing flags.
/// - Otherwise resets flags: do_all = p_gd, do_ask = false, do_error = true, etc.
/// - Loops through chars: toggles g/c, sets other flags, breaks on unrecognized.
/// - If do_count, sets do_ask = false.
/// - Returns pointer past consumed flags.
///
/// # Safety
/// `cmd` must be non-null and point to a valid null-terminated C string.
/// `subflags` must be non-null and point to a valid CSubFlags struct.
/// `which_pat` must be non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_sub_parse_flags(
    cmd: *mut c_char,
    subflags: *mut CSubFlags,
    which_pat: *mut c_int,
) -> *mut c_char {
    let p_gd = nvim_option_get_gd() != 0;

    // Find trailing options. When '&' is used, keep old options.
    let mut p = cmd;
    if *p == b'&' as c_char {
        p = p.add(1);
    } else {
        (*subflags).do_all = p_gd;
        (*subflags).do_ask = false;
        (*subflags).do_error = true;
        (*subflags).do_print = false;
        (*subflags).do_list = false;
        (*subflags).do_count = false;
        (*subflags).do_number = false;
        (*subflags).do_ic = 0; // kSubHonorOptions
    }

    loop {
        let c = *p as u8;
        if c == b'g' {
            (*subflags).do_all = !(*subflags).do_all;
        } else if c == b'c' {
            (*subflags).do_ask = !(*subflags).do_ask;
        } else if c == b'n' {
            (*subflags).do_count = true;
        } else if c == b'e' {
            (*subflags).do_error = !(*subflags).do_error;
        } else if c == b'r' {
            // use last used regexp
            *which_pat = RE_LAST;
        } else if c == b'p' {
            (*subflags).do_print = true;
        } else if c == b'#' {
            (*subflags).do_print = true;
            (*subflags).do_number = true;
        } else if c == b'l' {
            (*subflags).do_print = true;
            (*subflags).do_list = true;
        } else if c == b'i' {
            // ignore case
            (*subflags).do_ic = 1; // kSubIgnoreCase
        } else if c == b'I' {
            // don't ignore case
            (*subflags).do_ic = 2; // kSubMatchCase
        } else {
            break;
        }
        p = p.add(1);
    }

    if (*subflags).do_count {
        (*subflags).do_ask = false;
    }

    p
}

/// Parse substitute flags from a string.
///
/// Returns a bitmask of flags:
/// - bit 0: do_all (g)
/// - bit 1: do_ask (c)
/// - bit 2: do_count (n)
/// - bit 3: do_error (inverted: set if errors should be reported)
/// - bit 4: do_print (p)
/// - bit 5: do_list (l)
/// - bit 6: do_number (#)
/// - bits 7-8: do_ic (0=honor, 1=ignore, 2=match)
///
/// Returns -1 on error.
///
/// # Safety
/// The `flags` pointer must be null or point to a valid null-terminated C string.
pub unsafe extern "C" fn rs_parse_sub_flags(flags: *const std::ffi::c_char) -> c_int {
    if flags.is_null() {
        return 0; // No flags = default
    }

    let flags_str = match std::ffi::CStr::from_ptr(flags).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match SubFlags::parse(flags_str) {
        Ok(f) => {
            let mut result: c_int = 0;
            if f.do_all {
                result |= 1 << 0;
            }
            if f.do_ask {
                result |= 1 << 1;
            }
            if f.do_count {
                result |= 1 << 2;
            }
            if f.do_error {
                result |= 1 << 3;
            }
            if f.do_print {
                result |= 1 << 4;
            }
            if f.do_list {
                result |= 1 << 5;
            }
            if f.do_number {
                result |= 1 << 6;
            }
            result |= (f.do_ic.to_c() & 0x3) << 7;
            result
        }
        Err(_) => -1,
    }
}

/// Check if a delimiter character is valid.
///
/// Returns 1 if valid, 0 if invalid (alphanumeric).
pub extern "C" fn rs_is_valid_delimiter(c: c_int) -> c_int {
    if !(0..=127).contains(&c) {
        return 0;
    }
    c_int::from(is_valid_delimiter(char::from(c as u8)))
}

/// OK/FAIL constants matching C definitions
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Check if a character is a valid regexp delimiter.
///
/// Returns OK (1) if valid, FAIL (0) if the character is alphabetic.
/// Emits an error message on failure.
///
/// # Safety
/// Calls C emsg function.
#[no_mangle]
pub unsafe extern "C" fn rs_check_regexp_delim(c: c_int) -> c_int {
    use crate::emsg;

    // isalpha(c) in C -- check if ASCII alphabetic
    if (c as u8 as char).is_ascii_alphabetic() {
        emsg(c"E146: Regular expressions can't be delimited by letters".as_ptr());
        return FAIL;
    }
    OK
}

/// Skip over the "sub" part in :s/pat/sub/ where `delimiter` is the
/// separating character. Replaces the end delimiter with NUL.
///
/// Returns a pointer past the end delimiter (or to end of string).
///
/// # Safety
/// `start` must point to a valid, writable null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_substitute(
    start: *mut std::ffi::c_char,
    delimiter: c_int,
) -> *mut std::ffi::c_char {
    use crate::utfc_ptr2len;

    let mut p = start;
    while *p != 0 {
        if *p as c_int == delimiter {
            // end delimiter found -- replace it with NUL
            *p = 0;
            p = p.add(1);
            break;
        }
        if *p == b'\\' as i8 && *p.add(1) != 0 {
            // skip escaped characters
            p = p.add(1);
        }
        // MB_PTR_ADV(p)
        let len = utfc_ptr2len(p) as usize;
        p = p.add(len);
    }
    p
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_flags_new() {
        let flags = SubFlags::new();
        assert!(!flags.do_all);
        assert!(!flags.do_ask);
        assert!(!flags.do_count);
        assert!(flags.do_error); // Default is to report errors
        assert!(!flags.do_print);
        assert!(!flags.do_list);
        assert!(!flags.do_number);
        assert_eq!(flags.do_ic, SubIgnoreType::HonorOptions);
    }

    #[test]
    fn test_sub_flags_global() {
        let flags = SubFlags::global();
        assert!(flags.do_all);
        assert!(!flags.do_ask);
    }

    #[test]
    fn test_sub_flags_count_only() {
        let flags = SubFlags::count_only();
        assert!(flags.do_all);
        assert!(flags.do_count);
        assert!(flags.is_count_only());
    }

    #[test]
    fn test_sub_flags_parse() {
        // Empty string
        let flags = SubFlags::parse("").unwrap();
        assert!(!flags.do_all);

        // Global flag
        let flags = SubFlags::parse("g").unwrap();
        assert!(flags.do_all);

        // Multiple flags
        let flags = SubFlags::parse("gc").unwrap();
        assert!(flags.do_all);
        assert!(flags.do_ask);
        assert!(flags.needs_confirm());

        // Case flags
        let flags = SubFlags::parse("gi").unwrap();
        assert!(flags.do_all);
        assert_eq!(flags.do_ic, SubIgnoreType::IgnoreCase);

        let flags = SubFlags::parse("gI").unwrap();
        assert!(flags.do_all);
        assert_eq!(flags.do_ic, SubIgnoreType::MatchCase);

        // Error suppression
        let flags = SubFlags::parse("e").unwrap();
        assert!(!flags.do_error);

        // Print flags
        let flags = SubFlags::parse("p").unwrap();
        assert!(flags.do_print);

        let flags = SubFlags::parse("l").unwrap();
        assert!(flags.do_list);

        let flags = SubFlags::parse("#").unwrap();
        assert!(flags.do_number);
    }

    #[test]
    fn test_sub_flags_parse_invalid() {
        // Invalid flag character
        let result = SubFlags::parse("gx");
        assert!(matches!(result, Err(SubstituteError::InvalidFlag('x'))));
    }

    #[test]
    fn test_sub_result() {
        let mut result = SubResult::new();
        assert!(!result.has_matches());

        result.add_match();
        assert!(result.has_matches());
        assert_eq!(result.count, 1);

        result.add_line();
        assert_eq!(result.lines, 1);

        result.set_interrupted();
        assert!(result.interrupted);
    }

    #[test]
    fn test_sub_stats() {
        let mut stats = SubStats::new();
        assert_eq!(stats.total_subs, 0);

        let result = SubResult {
            count: 5,
            lines: 3,
            interrupted: false,
        };
        stats.add_result(&result);
        assert_eq!(stats.total_subs, 5);
        assert_eq!(stats.total_lines, 3);

        stats.reset();
        assert_eq!(stats.total_subs, 0);
        assert_eq!(stats.total_lines, 0);
    }

    #[test]
    fn test_is_valid_delimiter() {
        // Valid delimiters
        assert!(is_valid_delimiter('/'));
        assert!(is_valid_delimiter('#'));
        assert!(is_valid_delimiter('@'));
        assert!(is_valid_delimiter('!'));
        assert!(is_valid_delimiter(':'));

        // Invalid delimiters (alphanumeric)
        assert!(!is_valid_delimiter('a'));
        assert!(!is_valid_delimiter('Z'));
        assert!(!is_valid_delimiter('0'));
        assert!(!is_valid_delimiter('9'));
    }

    #[test]
    fn test_match_position() {
        let pos = MatchPosition::new(10, 5);
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);
    }

    #[test]
    fn test_match_range() {
        let start = MatchPosition::new(10, 0);
        let end = MatchPosition::new(10, 5);
        let range = MatchRange::new(start, end);

        assert!(range.is_single_line());
        assert_eq!(range.line_span(), 1);

        // Multi-line range
        let end = MatchPosition::new(12, 5);
        let range = MatchRange::new(start, end);

        assert!(!range.is_single_line());
        assert_eq!(range.line_span(), 3);
    }

    #[test]
    fn test_substitute_error_display() {
        let err = SubstituteError::InvalidFlag('x');
        assert_eq!(format!("{err}"), "invalid flag: x");

        let err = SubstituteError::InvalidDelimiter('a');
        assert!(format!("{err}").contains("delimited by letters"));

        let err = SubstituteError::NoPreviousPattern;
        assert_eq!(format!("{err}"), "no previous pattern");

        let err = SubstituteError::ZeroCount;
        assert_eq!(format!("{err}"), "zero count");
    }

    #[test]
    fn test_rs_is_valid_delimiter() {
        assert_eq!(rs_is_valid_delimiter(b'/' as c_int), 1);
        assert_eq!(rs_is_valid_delimiter(b'#' as c_int), 1);
        assert_eq!(rs_is_valid_delimiter(b'a' as c_int), 0);
        assert_eq!(rs_is_valid_delimiter(b'0' as c_int), 0);
    }

    #[test]
    fn test_rs_parse_sub_flags() {
        use std::ffi::CString;

        let flags = CString::new("g").unwrap();
        let result = unsafe { rs_parse_sub_flags(flags.as_ptr()) };
        assert!(result >= 0);
        assert_eq!(result & 1, 1); // do_all

        let flags = CString::new("gc").unwrap();
        let result = unsafe { rs_parse_sub_flags(flags.as_ptr()) };
        assert!(result >= 0);
        assert_eq!(result & 1, 1); // do_all
        assert_eq!(result & 2, 2); // do_ask

        // Invalid flag
        let flags = CString::new("gx").unwrap();
        let result = unsafe { rs_parse_sub_flags(flags.as_ptr()) };
        assert_eq!(result, -1);
    }
}
