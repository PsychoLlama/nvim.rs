//! Error format parsing and entry validation for quickfix lists.
//!
//! This module handles parsing error format lines and validating quickfix entries.
//! It implements the core parsing logic from `qf_parse_line()` and related functions.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::cast_lossless)]

use std::ffi::{c_char, c_int, c_void};

/// Line number type (matches `linenr_T` in Neovim)
type LinenrT = i32;

/// Opaque handle to `qfline_T` (quickfix entry)
type QfLinePtr = *mut crate::ffi_types::QfLineRaw;

// =============================================================================
// External C accessor functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Entry accessors
}

// =============================================================================
// Entry Validation Types
// =============================================================================

/// Result of validating a quickfix entry
#[repr(C)]
#[derive(Default)]
pub struct QfEntryValidation {
    /// Is the entry valid (has file/line/col)?
    pub valid: bool,
    /// Does the entry have a filename?
    pub has_filename: bool,
    /// Does the entry have a valid line number?
    pub has_lnum: bool,
    /// Does the entry have a valid column?
    pub has_col: bool,
    /// Does the entry have text?
    pub has_text: bool,
    /// Is the entry an error type?
    pub is_error: bool,
    /// Is the entry a warning type?
    pub is_warning: bool,
}

// =============================================================================
// Parsing Result Types
// =============================================================================

/// Result of parsing an error format line
#[repr(C)]
pub struct QfParseResult {
    /// Status code (`QF_OK`, `QF_FAIL`, etc.)
    pub status: c_int,
    /// Index of the matched format prefix
    pub prefix_idx: c_int,
    /// Whether this is a continuation line
    pub is_continuation: bool,
    /// Whether to ignore this line
    pub ignore: bool,
}

impl Default for QfParseResult {
    fn default() -> Self {
        Self {
            status: QF_OK,
            prefix_idx: 0,
            is_continuation: false,
            ignore: false,
        }
    }
}

// =============================================================================
// Entry Validation Functions
// =============================================================================

extern "C" {
    fn xfree(ptr: *mut ::std::ffi::c_void);
    fn xstrdup(s: *const ::std::ffi::c_char) -> *mut ::std::ffi::c_char;
}

/// Validate a quickfix entry and return detailed validation info.
///
/// This function checks various properties of a quickfix entry to determine
/// its validity and characteristics.
///
/// # Safety
///
/// - `qfp` may be null (returns default validation with all false)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_validate_entry(qfp: QfLinePtr) -> QfEntryValidation {
    if qfp.is_null() {
        return QfEntryValidation::default();
    }

    let lnum = (*qfp).qf_lnum;
    let col = (*qfp).qf_col;
    let fnum = (*qfp).qf_fnum;
    let type_char = (*qfp).qf_type;
    let text = (*qfp).qf_text;
    let valid = (*qfp).qf_valid != 0;

    let has_lnum = lnum > 0;
    let has_col = col > 0;
    let has_filename = fnum > 0;
    let has_text = !text.is_null() && unsafe { *text != 0 };

    // Check error/warning types - convert to u8 for safe comparison
    #[allow(clippy::cast_sign_loss)]
    let type_byte = type_char as u8;
    let type_upper = if type_byte.is_ascii_lowercase() {
        type_byte.to_ascii_uppercase()
    } else {
        type_byte
    };

    let is_error = type_upper == b'E';
    let is_warning = type_upper == b'W';

    QfEntryValidation {
        valid,
        has_filename,
        has_lnum,
        has_col,
        has_text,
        is_error,
        is_warning,
    }
}

/// Check if an entry has all required fields for a complete error.
///
/// A complete error has: filename, line number, and either text or valid flag.
///
/// # Safety
///
/// - `qfp` may be null (returns false)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_is_complete(qfp: QfLinePtr) -> bool {
    if qfp.is_null() {
        return false;
    }

    let validation = rs_qf_validate_entry(qfp);
    validation.has_filename && validation.has_lnum && (validation.has_text || validation.valid)
}

/// Check if an entry represents a diagnostic (error, warning, info, note).
///
/// # Safety
///
/// - `qfp` may be null (returns false)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_is_diagnostic(qfp: QfLinePtr) -> bool {
    if qfp.is_null() {
        return false;
    }

    let type_char = (*qfp).qf_type;
    #[allow(clippy::cast_sign_loss)]
    let type_byte = type_char as u8;
    let type_upper = if type_byte.is_ascii_lowercase() {
        type_byte.to_ascii_uppercase()
    } else {
        type_byte
    };

    matches!(type_upper, b'E' | b'W' | b'I' | b'N')
}

/// Get the severity level of an entry (0=none, 1=note, 2=info, 3=warning, 4=error).
///
/// Higher numbers indicate more severe issues.
///
/// # Safety
///
/// - `qfp` may be null (returns 0)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_severity(qfp: QfLinePtr) -> c_int {
    if qfp.is_null() {
        return 0;
    }

    let type_char = (*qfp).qf_type;
    #[allow(clippy::cast_sign_loss)]
    let type_byte = type_char as u8;
    let type_upper = if type_byte.is_ascii_lowercase() {
        type_byte.to_ascii_uppercase()
    } else {
        type_byte
    };

    match type_upper {
        b'E' => 4, // Error
        b'W' => 3, // Warning
        b'I' => 2, // Info
        b'N' => 1, // Note
        _ => 0,    // Unknown/none
    }
}

// =============================================================================
// Error Format Pattern Matching (supplements existing lib.rs functions)
// =============================================================================

/// Check if an errorformat prefix is a file prefix.
///
/// File prefixes are 'O', 'P', 'Q'.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_is_file_prefix(prefix: c_char) -> bool {
    matches!(prefix as u8, b'O' | b'P' | b'Q')
}

/// Check if an errorformat prefix is a directory prefix.
///
/// Directory prefixes are 'D' and 'X'.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_is_dir_prefix(prefix: c_char) -> bool {
    matches!(prefix as u8, b'D' | b'X')
}

// =============================================================================
// Line Classification
// =============================================================================

/// Line classification result
#[repr(C)]
#[derive(Default)]
pub struct QfLineClass {
    /// Looks like a compiler error/warning line
    pub looks_like_error: bool,
    /// Has filename:line:col pattern
    pub has_location: bool,
    /// Appears to be a continuation line
    pub is_continuation: bool,
    /// Is blank or whitespace only
    pub is_blank: bool,
    /// Contains common error keywords
    pub has_error_keyword: bool,
    /// Contains common warning keywords
    pub has_warning_keyword: bool,
}

/// Classify a line to determine its likely purpose.
///
/// This is used for heuristic line classification when no format matches.
///
/// # Safety
///
/// - `linebuf` may be null (returns default classification)
/// - If non-null, must point to a valid buffer of at least `linelen` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_qf_classify_line(
    linebuf: *const c_char,
    linelen: usize,
) -> QfLineClass {
    if linebuf.is_null() || linelen == 0 {
        return QfLineClass::default();
    }

    let bytes = std::slice::from_raw_parts(linebuf.cast::<u8>(), linelen);

    // Check for blank line
    let is_blank = bytes
        .iter()
        .all(|&b| b == b' ' || b == b'\t' || b == b'\n' || b == b'\r');
    if is_blank {
        return QfLineClass {
            is_blank: true,
            ..Default::default()
        };
    }

    // Check for continuation (starts with whitespace)
    let is_continuation = bytes.first().is_some_and(|&b| b == b' ' || b == b'\t');

    // Try to convert to string for pattern matching
    let Ok(s) = std::str::from_utf8(bytes) else {
        return QfLineClass {
            is_continuation,
            ..Default::default()
        };
    };

    let line_lower = s.to_lowercase();

    // Check for error/warning keywords
    let has_error_keyword = line_lower.contains("error")
        || line_lower.contains("fatal")
        || line_lower.contains("failed");

    let has_warning_keyword = line_lower.contains("warning")
        || line_lower.contains("caution")
        || line_lower.contains("note:");

    // Check for location pattern (file:line or file:line:col)
    let has_location = s.contains(':')
        && s.split(':')
            .nth(1)
            .is_some_and(|part| part.chars().take(10).all(|c| c.is_ascii_digit()));

    let looks_like_error = has_error_keyword || has_warning_keyword || has_location;

    QfLineClass {
        looks_like_error,
        has_location,
        is_continuation,
        is_blank,
        has_error_keyword,
        has_warning_keyword,
    }
}

// =============================================================================
// Entry Span Validation
// =============================================================================

/// Check if an entry's line range is valid (end >= start or no end).
///
/// # Safety
///
/// - `qfp` may be null (returns true - null entries are vacuously valid)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_has_valid_range(qfp: QfLinePtr) -> bool {
    if qfp.is_null() {
        return true;
    }

    let lnum = (*qfp).qf_lnum;
    let end_lnum = (*qfp).qf_end_lnum;

    // No end line means valid (single line entry)
    if end_lnum == 0 {
        return true;
    }

    // End must be >= start
    end_lnum >= lnum
}

/// Check if an entry's column range is valid (end >= start or no end).
///
/// # Safety
///
/// - `qfp` may be null (returns true - null entries are vacuously valid)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_has_valid_col_range(qfp: QfLinePtr) -> bool {
    if qfp.is_null() {
        return true;
    }

    let col = (*qfp).qf_col;
    let end_col = (*qfp).qf_end_col;
    let lnum = (*qfp).qf_lnum;
    let end_lnum = (*qfp).qf_end_lnum;

    // No end column means valid
    if end_col == 0 {
        return true;
    }

    // If multi-line, column comparison doesn't apply
    if end_lnum > 0 && end_lnum != lnum {
        return true;
    }

    // End must be >= start on the same line
    end_col >= col
}

/// Get the length (in lines) of an entry's span.
///
/// Returns 1 for single-line entries.
///
/// # Safety
///
/// - `qfp` may be null (returns 0)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_line_span(qfp: QfLinePtr) -> c_int {
    if qfp.is_null() {
        return 0;
    }

    let lnum = (*qfp).qf_lnum;
    let end_lnum = (*qfp).qf_end_lnum;

    if lnum <= 0 {
        return 0;
    }

    if end_lnum <= 0 || end_lnum < lnum {
        return 1;
    }

    end_lnum - lnum + 1
}

// =============================================================================
// Parse Type Detection
// =============================================================================

/// Detect the type of entry from text content heuristics.
///
/// This is used when the errorformat doesn't specify a type.
///
/// # Safety
///
/// - `text` may be null (returns 0)
/// - If non-null, must point to a valid C string
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_qf_detect_type_from_text(text: *const c_char) -> c_char {
    if text.is_null() {
        return 0;
    }

    let Ok(c_str) = std::ffi::CStr::from_ptr(text).to_str() else {
        return 0;
    };

    let lower = c_str.to_lowercase();

    // Check for error patterns first (highest priority)
    if lower.contains("error:") || lower.contains("error[") || lower.contains(": error") {
        return b'E' as c_char;
    }

    // Then warning patterns
    if lower.contains("warning:") || lower.contains("warning[") || lower.contains(": warning") {
        return b'W' as c_char;
    }

    // Info patterns
    if lower.contains("info:") || lower.contains("info[") || lower.contains(": info") {
        return b'I' as c_char;
    }

    // Note patterns
    if lower.contains("note:") || lower.contains("note[") || lower.contains(": note") {
        return b'N' as c_char;
    }

    // No type detected
    0
}

// =============================================================================
// Module Entry Parsing
// =============================================================================

/// Check if an entry has a module name set.
///
/// # Safety
///
/// - `qfp` may be null (returns false)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_has_module(qfp: QfLinePtr) -> bool {
    if qfp.is_null() {
        return false;
    }

    let module = (*qfp).qf_module;
    !module.is_null() && *module != 0
}

/// Check if an entry has an error number set.
///
/// # Safety
///
/// - `qfp` may be null (returns false)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_has_nr(qfp: QfLinePtr) -> bool {
    if qfp.is_null() {
        return false;
    }

    (*qfp).qf_nr != 0
}

// =============================================================================
// Quickfix Fields Structure (qffields_T equivalent)
// =============================================================================

/// Maximum buffer size for filename and pattern (matches CMDBUFFSIZE)
pub const CMDBUFFSIZE: usize = 256;

/// Quickfix fields - equivalent to C's qffields_T.
///
/// This structure holds parsed values from an errorformat line.
#[repr(C)]
pub struct QfFields {
    /// Buffer number (from %b)
    pub bnr: c_int,
    /// Line number (from %l)
    pub lnum: LinenrT,
    /// End line number (from %e)
    pub end_lnum: LinenrT,
    /// Column number (from %c, %p, or %v)
    pub col: c_int,
    /// End column number (from %k)
    pub end_col: c_int,
    /// Error number (from %n)
    pub enr: c_int,
    /// Error type character (from %t or prefix E/W/I/N)
    pub type_char: c_char,
    /// Whether entry is valid
    pub valid: bool,
    /// Whether column is visual column (from %v or %p)
    pub use_viscol: bool,
}

impl Default for QfFields {
    fn default() -> Self {
        Self {
            bnr: 0,
            lnum: 0,
            end_lnum: 0,
            col: 0,
            end_col: 0,
            enr: 0,
            type_char: 0,
            valid: false,
            use_viscol: false,
        }
    }
}

impl QfFields {
    /// Create new empty fields.
    pub const fn new() -> Self {
        Self {
            bnr: 0,
            lnum: 0,
            end_lnum: 0,
            col: 0,
            end_col: 0,
            enr: 0,
            type_char: 0,
            valid: false,
            use_viscol: false,
        }
    }

    /// Reset all fields to default values.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Check if this has a valid line number.
    pub const fn has_lnum(&self) -> bool {
        self.lnum > 0
    }

    /// Check if this has a valid column.
    pub const fn has_col(&self) -> bool {
        self.col > 0
    }

    /// Check if this has a line range.
    pub const fn has_line_range(&self) -> bool {
        self.end_lnum > 0
    }

    /// Check if this has a column range.
    pub const fn has_col_range(&self) -> bool {
        self.end_col > 0
    }

    /// Check if this has a buffer number.
    pub const fn has_buffer(&self) -> bool {
        self.bnr > 0
    }

    /// Check if this has an error number.
    pub const fn has_error_number(&self) -> bool {
        self.enr != 0
    }

    /// Get the error type.
    #[allow(clippy::cast_sign_loss)]
    pub fn error_type(&self) -> super::errorformat::QfErrorType {
        super::errorformat::QfErrorType::from_char(self.type_char as u8)
    }
}

// =============================================================================
// Format Specifier Parsing Results
// =============================================================================

/// Result codes for format parsing (matches C QF_OK, QF_FAIL, etc.)
pub const QF_OK: c_int = 0;
pub const QF_FAIL: c_int = -1;
pub const QF_NOMEM: c_int = -2;
pub const QF_MULTISCAN: c_int = -3;
pub const QF_END_OF_INPUT: c_int = -4;
pub const QF_IGNORE_LINE: c_int = -5;

/// Format specifier indices (matches C FMT_PATTERN_* constants)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FmtPatternIdx {
    /// %f - filename
    File = 0,
    /// %b - buffer number
    Buffer = 1,
    /// %n - error number
    Number = 2,
    /// %l - line number
    Line = 3,
    /// %e - end line number
    EndLine = 4,
    /// %c - column number
    Column = 5,
    /// %k - end column number
    EndColumn = 6,
    /// %t - error type
    Type = 7,
    /// %m - error message
    Message = 8,
    /// %r - rest of line
    Rest = 9,
    /// %p - pointer line (^)
    Pointer = 10,
    /// %v - virtual column
    VirtualColumn = 11,
    /// %s - search pattern
    Search = 12,
    /// %o - module name
    Module = 13,
}

impl FmtPatternIdx {
    /// Get the format character for this pattern.
    pub const fn to_char(self) -> u8 {
        match self {
            Self::File => b'f',
            Self::Buffer => b'b',
            Self::Number => b'n',
            Self::Line => b'l',
            Self::EndLine => b'e',
            Self::Column => b'c',
            Self::EndColumn => b'k',
            Self::Type => b't',
            Self::Message => b'm',
            Self::Rest => b'r',
            Self::Pointer => b'p',
            Self::VirtualColumn => b'v',
            Self::Search => b's',
            Self::Module => b'o',
        }
    }

    /// Get the pattern index from a format character.
    pub const fn from_char(c: u8) -> Option<Self> {
        match c {
            b'f' | b'F' => Some(Self::File),
            b'b' => Some(Self::Buffer),
            b'n' => Some(Self::Number),
            b'l' => Some(Self::Line),
            b'e' => Some(Self::EndLine),
            b'c' => Some(Self::Column),
            b'k' => Some(Self::EndColumn),
            b't' => Some(Self::Type),
            b'm' => Some(Self::Message),
            b'r' => Some(Self::Rest),
            b'p' => Some(Self::Pointer),
            b'v' => Some(Self::VirtualColumn),
            b's' => Some(Self::Search),
            b'o' => Some(Self::Module),
            _ => None,
        }
    }
}

// =============================================================================
// Format Specifier Parsing Functions
// =============================================================================

/// Parse a numeric value from a string slice.
///
/// Returns the parsed value or 0 if parsing fails.
fn parse_number(s: &[u8]) -> i64 {
    // Skip leading whitespace
    let s = s.iter().copied().skip_while(|&c| c == b' ' || c == b'\t');

    let mut result: i64 = 0;
    let mut negative = false;
    let mut started = false;

    for c in s {
        if c == b'-' && !started {
            negative = true;
            started = true;
        } else if c.is_ascii_digit() {
            started = true;
            result = result.saturating_mul(10).saturating_add((c - b'0') as i64);
        } else if started {
            break;
        }
    }

    if negative {
        result = -result;
    }
    result
}

/// Parse buffer number from a matched string (%b format).
///
/// Returns QF_OK if successful, QF_FAIL otherwise.
#[no_mangle]
pub extern "C" fn rs_qf_parse_fmt_b(
    match_start: *const c_char,
    match_len: usize,
    fields: &mut QfFields,
) -> c_int {
    if match_start.is_null() || match_len == 0 {
        return QF_FAIL;
    }

    let bytes = unsafe { std::slice::from_raw_parts(match_start.cast::<u8>(), match_len) };
    let bnr = parse_number(bytes) as c_int;

    if bnr <= 0 {
        return QF_FAIL;
    }

    fields.bnr = bnr;
    QF_OK
}

/// Parse error number from a matched string (%n format).
///
/// Returns QF_OK if successful, QF_FAIL otherwise.
#[no_mangle]
pub extern "C" fn rs_qf_parse_fmt_n(
    match_start: *const c_char,
    match_len: usize,
    fields: &mut QfFields,
) -> c_int {
    if match_start.is_null() || match_len == 0 {
        return QF_FAIL;
    }

    let bytes = unsafe { std::slice::from_raw_parts(match_start.cast::<u8>(), match_len) };
    fields.enr = parse_number(bytes) as c_int;
    QF_OK
}

/// Parse line number from a matched string (%l format).
///
/// Returns QF_OK if successful, QF_FAIL otherwise.
#[no_mangle]
pub extern "C" fn rs_qf_parse_fmt_l(
    match_start: *const c_char,
    match_len: usize,
    fields: &mut QfFields,
) -> c_int {
    if match_start.is_null() || match_len == 0 {
        return QF_FAIL;
    }

    let bytes = unsafe { std::slice::from_raw_parts(match_start.cast::<u8>(), match_len) };
    fields.lnum = parse_number(bytes) as LinenrT;
    QF_OK
}

/// Parse end line number from a matched string (%e format).
///
/// Returns QF_OK if successful, QF_FAIL otherwise.
#[no_mangle]
pub extern "C" fn rs_qf_parse_fmt_e(
    match_start: *const c_char,
    match_len: usize,
    fields: &mut QfFields,
) -> c_int {
    if match_start.is_null() || match_len == 0 {
        return QF_FAIL;
    }

    let bytes = unsafe { std::slice::from_raw_parts(match_start.cast::<u8>(), match_len) };
    fields.end_lnum = parse_number(bytes) as LinenrT;
    QF_OK
}

/// Parse column number from a matched string (%c format).
///
/// Returns QF_OK if successful, QF_FAIL otherwise.
#[no_mangle]
pub extern "C" fn rs_qf_parse_fmt_c(
    match_start: *const c_char,
    match_len: usize,
    fields: &mut QfFields,
) -> c_int {
    if match_start.is_null() || match_len == 0 {
        return QF_FAIL;
    }

    let bytes = unsafe { std::slice::from_raw_parts(match_start.cast::<u8>(), match_len) };
    fields.col = parse_number(bytes) as c_int;
    QF_OK
}

/// Parse end column number from a matched string (%k format).
///
/// Returns QF_OK if successful, QF_FAIL otherwise.
#[no_mangle]
pub extern "C" fn rs_qf_parse_fmt_k(
    match_start: *const c_char,
    match_len: usize,
    fields: &mut QfFields,
) -> c_int {
    if match_start.is_null() || match_len == 0 {
        return QF_FAIL;
    }

    let bytes = unsafe { std::slice::from_raw_parts(match_start.cast::<u8>(), match_len) };
    fields.end_col = parse_number(bytes) as c_int;
    QF_OK
}

/// Parse error type from a matched string (%t format).
///
/// Returns QF_OK if successful, QF_FAIL otherwise.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub extern "C" fn rs_qf_parse_fmt_t(
    match_start: *const c_char,
    match_len: usize,
    fields: &mut QfFields,
) -> c_int {
    if match_start.is_null() || match_len == 0 {
        return QF_FAIL;
    }

    let first_char = unsafe { *match_start.cast::<u8>() };
    fields.type_char = first_char as c_char;
    QF_OK
}

/// Parse virtual column from a matched string (%v format).
///
/// Returns QF_OK if successful, QF_FAIL otherwise.
#[no_mangle]
pub extern "C" fn rs_qf_parse_fmt_v(
    match_start: *const c_char,
    match_len: usize,
    fields: &mut QfFields,
) -> c_int {
    if match_start.is_null() || match_len == 0 {
        return QF_FAIL;
    }

    let bytes = unsafe { std::slice::from_raw_parts(match_start.cast::<u8>(), match_len) };
    fields.col = parse_number(bytes) as c_int;
    fields.use_viscol = true;
    QF_OK
}

/// Parse pointer line from a matched string (%p format).
///
/// The pointer line contains characters where tabs count as 8 columns.
///
/// Returns QF_OK if successful, QF_FAIL otherwise.
#[no_mangle]
pub extern "C" fn rs_qf_parse_fmt_p(
    match_start: *const c_char,
    match_len: usize,
    fields: &mut QfFields,
) -> c_int {
    if match_start.is_null() || match_len == 0 {
        return QF_FAIL;
    }

    let bytes = unsafe { std::slice::from_raw_parts(match_start.cast::<u8>(), match_len) };

    let mut col: c_int = 0;
    for &byte in bytes {
        col += 1;
        if byte == b'\t' {
            col += 7;
            col -= col % 8;
        }
    }
    col += 1; // 1-based column

    fields.col = col;
    fields.use_viscol = true;
    QF_OK
}

// =============================================================================
// Prefix Type Checking
// =============================================================================

/// Check if a prefix character indicates an error type.
///
/// Error-type prefixes are: E, W, I, N
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_is_error_prefix(prefix: c_char) -> bool {
    matches!(prefix as u8, b'E' | b'W' | b'I' | b'N')
}

/// Check if a prefix character indicates a continuation line.
///
/// Continuation prefixes are: C, Z
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_is_continuation_prefix(prefix: c_char) -> bool {
    matches!(prefix as u8, b'C' | b'Z')
}

/// Check if a prefix character indicates a multiline pattern.
///
/// Multiline prefixes are: A (start), C (continuation), Z (end)
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_is_multiline_prefix(prefix: c_char) -> bool {
    matches!(prefix as u8, b'A' | b'C' | b'Z')
}

/// Check if a prefix character indicates a global format.
///
/// Global prefixes are: G
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_is_global_prefix(prefix: c_char) -> bool {
    (prefix as u8) == b'G'
}

// Note: rs_efm_prefix_to_type is defined in lib.rs

// =============================================================================
// Multiline State Tracking
// =============================================================================

/// Multiline parsing state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct QfMultilineState {
    /// Currently in a multiline sequence
    pub in_multiline: bool,
    /// Should ignore continuation lines
    pub ignore: bool,
    /// Last prefix character
    pub last_prefix: c_char,
}

impl QfMultilineState {
    /// Create a new multiline state.
    pub const fn new() -> Self {
        Self {
            in_multiline: false,
            ignore: false,
            last_prefix: 0,
        }
    }

    /// Update state based on a matched prefix.
    #[allow(clippy::cast_sign_loss)]
    pub fn update(&mut self, prefix: c_char) {
        match prefix as u8 {
            b'A' => {
                self.in_multiline = true;
                self.ignore = false;
            }
            b'C' => {
                // Continue multiline
            }
            b'Z' => {
                self.in_multiline = false;
            }
            _ => {
                if !self.in_multiline {
                    self.ignore = false;
                }
            }
        }
        self.last_prefix = prefix;
    }

    /// Reset the multiline state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Create a new multiline state.
#[no_mangle]
pub extern "C" fn rs_qf_multiline_state_new() -> QfMultilineState {
    QfMultilineState::new()
}

/// Update multiline state with a prefix.
#[no_mangle]
pub extern "C" fn rs_qf_multiline_state_update(state: &mut QfMultilineState, prefix: c_char) {
    state.update(prefix);
}

/// Reset multiline state.
#[no_mangle]
pub extern "C" fn rs_qf_multiline_state_reset(state: &mut QfMultilineState) {
    state.reset();
}

// =============================================================================
// FFI Exports for QfFields
// =============================================================================

/// Create a new QfFields structure with default values.
#[no_mangle]
pub extern "C" fn rs_qf_fields_new() -> QfFields {
    QfFields::new()
}

/// Reset a QfFields structure to default values.
#[no_mangle]
pub extern "C" fn rs_qf_fields_reset(fields: &mut QfFields) {
    fields.reset();
}

/// Check if fields has a valid line number.
#[no_mangle]
pub extern "C" fn rs_qf_fields_has_lnum(fields: &QfFields) -> bool {
    fields.has_lnum()
}

/// Check if fields has a valid column.
#[no_mangle]
pub extern "C" fn rs_qf_fields_has_col(fields: &QfFields) -> bool {
    fields.has_col()
}

/// Check if fields has a line range.
#[no_mangle]
pub extern "C" fn rs_qf_fields_has_line_range(fields: &QfFields) -> bool {
    fields.has_line_range()
}

/// Check if fields has a column range.
#[no_mangle]
pub extern "C" fn rs_qf_fields_has_col_range(fields: &QfFields) -> bool {
    fields.has_col_range()
}

/// Check if fields has a buffer number.
#[no_mangle]
pub extern "C" fn rs_qf_fields_has_buffer(fields: &QfFields) -> bool {
    fields.has_buffer()
}

/// Check if fields has an error number.
#[no_mangle]
pub extern "C" fn rs_qf_fields_has_error_number(fields: &QfFields) -> bool {
    fields.has_error_number()
}

// =============================================================================
// Errorformat Pattern Conversion
// =============================================================================

/// Maximum number of % patterns recognized (matches C's FMT_PATTERNS)
pub const FMT_PATTERNS: usize = 14;

/// Index of %m (message) pattern
pub const FMT_PATTERN_M: usize = 8;

/// Index of %r (rest) pattern
pub const FMT_PATTERN_R: usize = 9;

/// Format pattern definitions - maps format character to regex pattern.
///
/// This matches the C `fmt_pat` array exactly.
pub const FMT_PAT: [(u8, &str); FMT_PATTERNS] = [
    (b'f', r".+"),       // 0: filename (only used when at end)
    (b'b', r"\d+"),      // 1: buffer number
    (b'n', r"\d+"),      // 2: error number
    (b'l', r"\d+"),      // 3: line number
    (b'e', r"\d+"),      // 4: end line number
    (b'c', r"\d+"),      // 5: column number
    (b'k', r"\d+"),      // 6: end column number
    (b't', r"."),        // 7: error type
    (b'm', r".+"),       // 8: message
    (b'r', r".*"),       // 9: rest of line
    (b'p', r"[-\t .]*"), // 10: pointer line
    (b'v', r"\d+"),      // 11: virtual column
    (b's', r".+"),       // 12: search pattern
    (b'o', r".+"),       // 13: module name
];

/// Result of analyzing an errorformat prefix
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct EfmPrefixResult {
    /// The prefix character (D/X/A/E/W/I/N/C/Z/G/O/P/Q)
    pub prefix: c_char,
    /// Optional flags (+/-)
    pub flags: c_char,
    /// Status: 0 = success, -1 = error
    pub status: c_int,
}

/// Result of converting an errorformat pattern to regex
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EfmPatResult {
    /// Number of bytes written to output
    pub bytes_written: usize,
    /// The round counter (pattern index)
    pub round: c_int,
    /// Status: 0 = success, -1 = error
    pub status: c_int,
}

impl Default for EfmPatResult {
    fn default() -> Self {
        Self {
            bytes_written: 0,
            round: 0,
            status: QF_FAIL,
        }
    }
}

/// Find the format pattern index for a given character.
///
/// Returns the index (0-13) or -1 if not found.
#[no_mangle]
pub extern "C" fn rs_efm_find_pattern_idx(c: c_char) -> c_int {
    #[allow(clippy::cast_sign_loss)]
    let c = c as u8;
    for (idx, (conv_char, _)) in FMT_PAT.iter().enumerate() {
        if *conv_char == c {
            return idx as c_int;
        }
    }
    -1
}

/// Get the regex pattern for a format pattern index.
///
/// Returns a pointer to a static string, or null if index is out of bounds.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_get_pattern(idx: c_int) -> *const c_char {
    if idx < 0 || idx as usize >= FMT_PATTERNS {
        return std::ptr::null();
    }
    FMT_PAT[idx as usize].1.as_ptr().cast()
}

/// Get the length of the regex pattern for a format pattern index.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_get_pattern_len(idx: c_int) -> usize {
    if idx < 0 || idx as usize >= FMT_PATTERNS {
        return 0;
    }
    FMT_PAT[idx as usize].1.len()
}

/// Analyze an errorformat prefix character.
///
/// Valid prefixes are: D, X, A, E, W, I, N, C, Z, G, O, P, Q
/// Optional flags before the prefix: +, -
///
/// # Safety
///
/// - `efmp` must be a valid pointer to a null-terminated string
/// - The caller must check `status` in the result (-1 indicates error)
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_efm_analyze_prefix(
    efmp: *const c_char,
    efmp_len: usize,
) -> EfmPrefixResult {
    if efmp.is_null() || efmp_len == 0 {
        return EfmPrefixResult {
            status: QF_FAIL,
            ..Default::default()
        };
    }

    let bytes = std::slice::from_raw_parts(efmp.cast::<u8>(), efmp_len);
    let mut idx = 0;
    let mut flags: c_char = 0;

    // Check for optional flags (+ or -)
    if idx < bytes.len() && (bytes[idx] == b'+' || bytes[idx] == b'-') {
        flags = bytes[idx] as c_char;
        idx += 1;
    }

    // Check for valid prefix character
    let prefix = if idx < bytes.len() {
        let c = bytes[idx];
        if matches!(
            c,
            b'D' | b'X'
                | b'A'
                | b'E'
                | b'W'
                | b'I'
                | b'N'
                | b'C'
                | b'Z'
                | b'G'
                | b'O'
                | b'P'
                | b'Q'
        ) {
            c as c_char
        } else {
            // Invalid prefix character
            return EfmPrefixResult {
                status: QF_FAIL,
                ..Default::default()
            };
        }
    } else {
        return EfmPrefixResult {
            status: QF_FAIL,
            ..Default::default()
        };
    };

    EfmPrefixResult {
        prefix,
        flags,
        status: QF_OK,
    }
}

/// Check if a pattern index is valid for the given prefix.
///
/// Returns true if valid, false otherwise.
///
/// Certain patterns are not allowed with certain prefixes:
/// - Patterns 1-8 (buffer through message) are not allowed with D/X/O/P/Q prefixes
/// - Pattern 9 (%r - rest) is only allowed with O/P/Q prefixes
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_pattern_valid_for_prefix(idx: c_int, prefix: c_char) -> bool {
    if idx < 0 || idx as usize >= FMT_PATTERNS {
        return false;
    }
    let idx = idx as usize;
    let prefix = prefix as u8;

    // Patterns 1-8 are not allowed with D/X/O/P/Q prefixes
    if idx > 0 && idx < FMT_PATTERN_R && matches!(prefix, b'D' | b'X' | b'O' | b'P' | b'Q') {
        return false;
    }

    // Pattern 9 (%r) is only allowed with O/P/Q prefixes
    if idx == FMT_PATTERN_R && !matches!(prefix, b'O' | b'P' | b'Q') {
        return false;
    }

    true
}

/// Convert a scanf-like format pattern to a regex pattern.
///
/// Handles patterns like %*[^:] (character class) or %*\D (regex escape).
///
/// # Arguments
///
/// * `efmp` - Pointer to format string starting after the '*'
/// * `efmp_len` - Length of remaining format string
/// * `out` - Output buffer for regex pattern
/// * `out_size` - Size of output buffer
///
/// # Returns
///
/// Number of bytes consumed from input, or -1 on error.
/// Output is written to `out` buffer, null-terminated.
///
/// # Safety
///
/// - `efmp` must be a valid pointer
/// - `out` must be a valid pointer to a buffer of at least `out_size` bytes
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_efm_scanf_to_regpat(
    efmp: *const c_char,
    efmp_len: usize,
    out: *mut c_char,
    out_size: usize,
) -> c_int {
    if efmp.is_null() || out.is_null() || efmp_len == 0 || out_size < 4 {
        return -1;
    }

    let input = std::slice::from_raw_parts(efmp.cast::<u8>(), efmp_len);
    let output = std::slice::from_raw_parts_mut(out.cast::<u8>(), out_size);

    let mut out_idx = 0;
    let mut in_idx = 0;

    match input.first() {
        Some(&b'[') => {
            // Character class: %*[^a-z0-9] -> [^a-z0-9]\+
            output[out_idx] = b'[';
            out_idx += 1;
            in_idx += 1;

            // Check for negation
            if in_idx < input.len() && input[in_idx] == b'^' {
                if out_idx >= out_size - 1 {
                    return -1;
                }
                output[out_idx] = b'^';
                out_idx += 1;
                in_idx += 1;
            }

            // Copy first character (could be ']')
            if in_idx < input.len() {
                if out_idx >= out_size - 1 {
                    return -1;
                }
                output[out_idx] = input[in_idx];
                out_idx += 1;
                in_idx += 1;
            }

            // Copy until closing ']'
            while in_idx < input.len() && input[in_idx] != b']' {
                if out_idx >= out_size - 1 {
                    return -1;
                }
                output[out_idx] = input[in_idx];
                out_idx += 1;
                in_idx += 1;
            }

            // Check for missing ']'
            if in_idx >= input.len() {
                return -1; // Missing ]
            }

            // Copy closing ']'
            if out_idx >= out_size - 1 {
                return -1;
            }
            output[out_idx] = b']';
            out_idx += 1;
            in_idx += 1;

            // Add \+ for one or more matches
            if out_idx >= out_size - 2 {
                return -1;
            }
            output[out_idx] = b'\\';
            out_idx += 1;
            output[out_idx] = b'+';
            out_idx += 1;
        }
        Some(&b'\\') => {
            // Regex escape: %*\D -> \D\+
            if in_idx + 1 >= input.len() {
                return -1;
            }
            if out_idx >= out_size - 4 {
                return -1;
            }
            output[out_idx] = b'\\';
            out_idx += 1;
            in_idx += 1;
            output[out_idx] = input[in_idx];
            out_idx += 1;
            in_idx += 1;

            // Add \+ for one or more matches
            output[out_idx] = b'\\';
            out_idx += 1;
            output[out_idx] = b'+';
            out_idx += 1;
        }
        _ => {
            // Unsupported format
            return -1;
        }
    }

    // Null-terminate
    if out_idx < out_size {
        output[out_idx] = 0;
    }

    in_idx as c_int
}

/// Convert an errorformat pattern specifier to regex.
///
/// This handles patterns like %f, %l, %c, etc. and converts them to
/// regex capture groups like \([^:]*\) or \(\d\+\).
///
/// # Arguments
///
/// * `efmpat` - The format character (f, l, c, etc.)
/// * `next_char` - The character following the format specifier (for %f handling)
/// * `addr` - Array tracking which patterns have been used (FMT_PATTERNS size)
/// * `idx` - Index of this pattern in the addr array
/// * `round` - Current round counter
/// * `prefix` - The format prefix character
/// * `out` - Output buffer for regex pattern
/// * `out_size` - Size of output buffer
///
/// # Returns
///
/// Result struct with bytes_written, updated round, and status.
///
/// # Safety
///
/// - All pointers must be valid
/// - `addr` must point to an array of at least FMT_PATTERNS bytes
/// - `out` must point to a buffer of at least `out_size` bytes
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_efmpat_to_regpat(
    efmpat: c_char,
    next_char: c_char,
    addr: *mut c_char,
    idx: c_int,
    round: c_int,
    prefix: c_char,
    out: *mut c_char,
    out_size: usize,
) -> EfmPatResult {
    if addr.is_null() || out.is_null() || out_size < 16 {
        return EfmPatResult::default();
    }

    let idx_usize = idx as usize;
    if idx < 0 || idx_usize >= FMT_PATTERNS {
        return EfmPatResult::default();
    }

    let addr_slice = std::slice::from_raw_parts_mut(addr.cast::<u8>(), FMT_PATTERNS);
    let output = std::slice::from_raw_parts_mut(out.cast::<u8>(), out_size);

    // Check if pattern already used
    if addr_slice[idx_usize] != 0 {
        // E372: Too many %X in format string
        return EfmPatResult::default();
    }

    // Check if pattern is valid for this prefix
    if !rs_efm_pattern_valid_for_prefix(idx, prefix) {
        // E373: Unexpected %X in format string
        return EfmPatResult::default();
    }

    // Mark pattern as used
    let new_round = round + 1;
    addr_slice[idx_usize] = new_round as u8;

    let mut out_idx = 0;

    // Start capture group: \(
    output[out_idx] = b'\\';
    out_idx += 1;
    output[out_idx] = b'(';
    out_idx += 1;

    // Handle %f specially - file names may contain spaces
    let efmpat_byte = efmpat as u8;
    let next_byte = next_char as u8;

    if efmpat_byte == b'f' && next_byte != 0 {
        if next_byte != b'\\' && next_byte != b'%' {
            // File name followed by regular character: use non-greedy match
            // ".{-1,}x" - match as few chars as possible before the delimiter
            let pattern = b".\\{-1,}";
            if out_idx + pattern.len() >= out_size {
                return EfmPatResult::default();
            }
            output[out_idx..out_idx + pattern.len()].copy_from_slice(pattern);
            out_idx += pattern.len();
        } else {
            // File name followed by '\' or '%': use greedy match
            let pattern = b"\\f\\+";
            if out_idx + pattern.len() >= out_size {
                return EfmPatResult::default();
            }
            output[out_idx..out_idx + pattern.len()].copy_from_slice(pattern);
            out_idx += pattern.len();
        }
    } else {
        // Use the standard pattern for this format specifier
        let pattern = FMT_PAT[idx_usize].1.as_bytes();
        if out_idx + pattern.len() >= out_size {
            return EfmPatResult::default();
        }
        output[out_idx..out_idx + pattern.len()].copy_from_slice(pattern);
        out_idx += pattern.len();
    }

    // End capture group: \)
    if out_idx + 2 >= out_size {
        return EfmPatResult::default();
    }
    output[out_idx] = b'\\';
    out_idx += 1;
    output[out_idx] = b')';
    out_idx += 1;

    // Null-terminate
    if out_idx < out_size {
        output[out_idx] = 0;
    }

    EfmPatResult {
        bytes_written: out_idx,
        round: new_round,
        status: QF_OK,
    }
}

/// Check if a character is a regex magic character that needs escaping.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_is_regex_magic(c: c_char) -> bool {
    matches!(c as u8, b'.' | b'*' | b'^' | b'$' | b'~' | b'[')
}

/// Check if a character is a format magic character that should be copied directly.
///
/// These are characters that appear after % in the format string and
/// should be copied directly to the regex pattern.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_efm_is_format_magic(c: c_char) -> bool {
    matches!(c as u8, b'%' | b'\\' | b'.' | b'^' | b'$' | b'~' | b'[')
}

// =============================================================================
// Full Errorformat to Regex Conversion
// =============================================================================

/// Result of the full errorformat-to-regex conversion
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EfmToRegpatResult {
    /// Number of bytes written to output
    pub bytes_written: usize,
    /// Prefix character (if found)
    pub prefix: c_char,
    /// Flags character (if found)
    pub flags: c_char,
    /// Whether %> (conthere) was found
    pub conthere: bool,
    /// Status: 0 = success, -1 = error
    pub status: c_int,
    /// Error code for specific error messages (E372-E377)
    pub error_code: c_int,
    /// Character that caused the error (for E372, E373, E376, E377)
    pub error_char: c_char,
}

impl Default for EfmToRegpatResult {
    fn default() -> Self {
        Self {
            bytes_written: 0,
            prefix: 0,
            flags: 0,
            conthere: false,
            status: QF_FAIL,
            error_code: 0,
            error_char: 0,
        }
    }
}

/// Convert an 'errorformat' string to a regular expression pattern.
///
/// This is the main entry point for errorformat parsing. It takes an
/// errorformat string like `%f:%l:%c: %m` and converts it to a regex
/// pattern like `^\(.+\):\(\d+\):\(\d+\): \(.+\)$`.
///
/// # Arguments
///
/// * `efm` - The errorformat string
/// * `efm_len` - Length of the errorformat string
/// * `addr` - Array tracking which patterns have been used (FMT_PATTERNS size)
/// * `out` - Output buffer for regex pattern
/// * `out_size` - Size of output buffer
///
/// # Returns
///
/// Result struct with bytes_written, prefix/flags info, conthere flag, and status.
///
/// # Safety
///
/// - All pointers must be valid
/// - `addr` must point to an array of at least FMT_PATTERNS bytes
/// - `out` must point to a buffer of at least `out_size` bytes
#[no_mangle]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_efm_to_regpat(
    efm: *const c_char,
    efm_len: usize,
    addr: *mut c_char,
    out: *mut c_char,
    out_size: usize,
) -> EfmToRegpatResult {
    if efm.is_null() || addr.is_null() || out.is_null() || efm_len == 0 || out_size < 16 {
        return EfmToRegpatResult::default();
    }

    let input = std::slice::from_raw_parts(efm.cast::<u8>(), efm_len);
    let addr_slice = std::slice::from_raw_parts_mut(addr.cast::<u8>(), FMT_PATTERNS);
    let output = std::slice::from_raw_parts_mut(out.cast::<u8>(), out_size);

    let mut out_idx = 0;
    let mut in_idx = 0;
    let mut round = 0;
    let mut prefix: c_char = 0;
    let mut flags: c_char = 0;
    let mut conthere = false;

    // Start regex with ^
    output[out_idx] = b'^';
    out_idx += 1;

    while in_idx < input.len() {
        if input[in_idx] == b'%' {
            in_idx += 1;
            if in_idx >= input.len() {
                break;
            }

            let c = input[in_idx];

            // Find pattern index for this character
            let pattern_idx = rs_efm_find_pattern_idx(c as c_char);

            if pattern_idx >= 0 {
                // Known format specifier (%f, %l, %c, etc.)
                let next_char = if in_idx + 1 < input.len() {
                    input[in_idx + 1] as c_char
                } else {
                    0
                };

                // Check for buffer overflow
                if out_idx + 32 >= out_size {
                    return EfmToRegpatResult::default();
                }

                let result = rs_efmpat_to_regpat(
                    c as c_char,
                    next_char,
                    addr.cast(),
                    pattern_idx,
                    round,
                    prefix,
                    output.as_mut_ptr().add(out_idx).cast(),
                    out_size - out_idx,
                );

                if result.status != QF_OK {
                    // Check which error - E372 if already used, E373 otherwise
                    let error_code = if addr_slice[pattern_idx as usize] != 0 {
                        372 // E372: Too many %X
                    } else {
                        373 // E373: Unexpected %X
                    };
                    return EfmToRegpatResult {
                        error_code,
                        error_char: c as c_char,
                        ..Default::default()
                    };
                }

                out_idx += result.bytes_written;
                round = result.round;
                in_idx += 1;
            } else if c == b'*' {
                // Scanf-like format: %*[...] or %*\X
                in_idx += 1;
                if in_idx >= input.len() {
                    return EfmToRegpatResult {
                        error_code: 375,
                        error_char: c as c_char,
                        ..Default::default()
                    };
                }

                if out_idx + 32 >= out_size {
                    return EfmToRegpatResult::default();
                }

                let consumed = rs_efm_scanf_to_regpat(
                    input.as_ptr().add(in_idx).cast(),
                    input.len() - in_idx,
                    output.as_mut_ptr().add(out_idx).cast(),
                    out_size - out_idx,
                );

                if consumed < 0 {
                    // E374 or E375 depending on the input
                    let err_code = if in_idx < input.len() && input[in_idx] == b'[' {
                        374 // Missing ]
                    } else {
                        375 // Unsupported
                    };
                    return EfmToRegpatResult {
                        error_code: err_code,
                        error_char: input[in_idx] as c_char,
                        ..Default::default()
                    };
                }

                // Find the actual end of output (strlen)
                let mut written = 0;
                while out_idx + written < out_size && output[out_idx + written] != 0 {
                    written += 1;
                }
                out_idx += written;
                in_idx += consumed as usize;
            } else if matches!(c, b'%' | b'\\' | b'.' | b'^' | b'$' | b'~' | b'[') {
                // Regexp magic characters after %
                if out_idx >= out_size - 1 {
                    return EfmToRegpatResult::default();
                }
                output[out_idx] = c;
                out_idx += 1;
                in_idx += 1;
            } else if c == b'#' {
                // %# becomes * (any number of matches)
                if out_idx >= out_size - 1 {
                    return EfmToRegpatResult::default();
                }
                output[out_idx] = b'*';
                out_idx += 1;
                in_idx += 1;
            } else if c == b'>' {
                // %> sets conthere flag
                conthere = true;
                in_idx += 1;
            } else if in_idx == 1 {
                // Prefix is only allowed at the beginning (after the first %)
                let prefix_result =
                    rs_efm_analyze_prefix(input.as_ptr().add(in_idx).cast(), input.len() - in_idx);

                if prefix_result.status != QF_OK {
                    // E376: Invalid prefix
                    return EfmToRegpatResult {
                        error_code: 376,
                        error_char: c as c_char,
                        ..Default::default()
                    };
                }

                prefix = prefix_result.prefix;
                flags = prefix_result.flags;
                in_idx += if flags != 0 { 2 } else { 1 };
            } else {
                // E377: Invalid %X in format string
                return EfmToRegpatResult {
                    error_code: 377,
                    error_char: c as c_char,
                    ..Default::default()
                };
            }
        } else {
            // Normal character
            let c = input[in_idx];

            if c == b'\\' && in_idx + 1 < input.len() {
                // Escape sequence - skip the backslash and copy next char
                in_idx += 1;
                if out_idx >= out_size - 1 {
                    return EfmToRegpatResult::default();
                }
                output[out_idx] = input[in_idx];
                out_idx += 1;
            } else if matches!(c, b'.' | b'*' | b'^' | b'$' | b'~' | b'[') {
                // Escape regex metacharacters
                if out_idx >= out_size - 2 {
                    return EfmToRegpatResult::default();
                }
                output[out_idx] = b'\\';
                out_idx += 1;
                output[out_idx] = c;
                out_idx += 1;
            } else if c != 0 {
                // Regular character
                if out_idx >= out_size - 1 {
                    return EfmToRegpatResult::default();
                }
                output[out_idx] = c;
                out_idx += 1;
            }
            in_idx += 1;
        }
    }

    // End regex with $
    if out_idx >= out_size - 2 {
        return EfmToRegpatResult::default();
    }
    output[out_idx] = b'$';
    out_idx += 1;
    output[out_idx] = 0; // Null-terminate

    EfmToRegpatResult {
        bytes_written: out_idx,
        prefix,
        flags,
        conthere,
        status: QF_OK,
        error_code: 0,
        error_char: 0,
    }
}

// =============================================================================
// Phase Q2: Line Parsing Helpers
// =============================================================================

/// Get the error type character for a prefix.
///
/// Prefixes E, W, I, N set the error type. Others return 0.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_qf_parse_prefix_type(prefix: c_char) -> c_char {
    match prefix as u8 {
        b'E' | b'W' | b'I' | b'N' => prefix,
        _ => 0,
    }
}

/// Check if a line should be skipped based on flags.
///
/// Lines with '-' flag should be excluded.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_qf_should_skip_line(flags: c_char) -> bool {
    (flags as u8) == b'-'
}

/// Check if a prefix is for continuation (C or Z).
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_qf_is_continuation(prefix: c_char) -> bool {
    matches!(prefix as u8, b'C' | b'Z')
}

/// Check if a prefix starts a multiline sequence (A, E, W, I, N).
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_qf_starts_multiline(prefix: c_char) -> bool {
    matches!(prefix as u8, b'A' | b'E' | b'W' | b'I' | b'N')
}

/// Check if a prefix is for directory handling (D or X).
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_qf_is_dir_handler(prefix: c_char) -> bool {
    matches!(prefix as u8, b'D' | b'X')
}

/// Check if a prefix is for file handling (O, P, or Q).
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_qf_is_file_handler(prefix: c_char) -> bool {
    matches!(prefix as u8, b'O' | b'P' | b'Q')
}

/// Validate entry fields for adding to quickfix list.
///
/// This validates the numeric fields in QfFields.
/// Note: Cannot check filename/message as those are stored separately in C's qffields_T.
#[no_mangle]
pub extern "C" fn rs_qf_validate_fields(fields: &QfFields) -> QfFieldsValidation {
    let has_lnum = fields.lnum > 0;
    let has_col = fields.col > 0;
    let has_end_lnum = fields.end_lnum > 0;
    let has_end_col = fields.end_col > 0;

    QfFieldsValidation {
        has_lnum,
        has_col,
        has_end_lnum,
        has_end_col,
        has_buffer: fields.bnr > 0,
        has_error_number: fields.enr != 0,
    }
}

/// Result of validating QfFields (numeric fields only).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct QfFieldsValidation {
    /// Has a valid line number
    pub has_lnum: bool,
    /// Has a valid column
    pub has_col: bool,
    /// Has end line number
    pub has_end_lnum: bool,
    /// Has end column
    pub has_end_col: bool,
    /// Has buffer number
    pub has_buffer: bool,
    /// Has error number
    pub has_error_number: bool,
}

/// Check if entry is considered printable (valid type character).
///
/// Returns true if the type character is printable or 0/1.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_qf_type_is_printable(type_char: c_char) -> bool {
    let c = type_char as u8;
    // Type 0 or 1 are valid (1 is legacy)
    // Otherwise must be printable ASCII
    c == 0 || c == 1 || (0x20..0x7F).contains(&c)
}

/// Normalize type character for storage.
///
/// If type is 1 or non-printable, returns 0.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub extern "C" fn rs_qf_normalize_type(type_char: c_char) -> c_char {
    let c = type_char as u8;
    if c == 1 || (c > 0 && !(0x20..0x7F).contains(&c)) {
        0
    } else {
        type_char
    }
}

/// Parse match state - tracks parsing progress through format patterns.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct QfParseState {
    /// Current format pattern index
    pub fmt_idx: c_int,
    /// Whether match was found
    pub matched: bool,
    /// The prefix of matched format
    pub prefix: c_char,
    /// The flags of matched format
    pub flags: c_char,
    /// Whether to continue from this format (%>)
    pub conthere: bool,
}

impl QfParseState {
    /// Create a new parse state.
    pub const fn new() -> Self {
        Self {
            fmt_idx: 0,
            matched: false,
            prefix: 0,
            flags: 0,
            conthere: false,
        }
    }
}

/// Create new parse state.
#[no_mangle]
pub extern "C" fn rs_qf_parse_state_new() -> QfParseState {
    QfParseState::new()
}

/// Reset parse state.
#[no_mangle]
pub extern "C" fn rs_qf_parse_state_reset(state: &mut QfParseState) {
    *state = QfParseState::new();
}

/// Update parse state after a match.
#[no_mangle]
pub extern "C" fn rs_qf_parse_state_set_match(
    state: &mut QfParseState,
    fmt_idx: c_int,
    prefix: c_char,
    flags: c_char,
    conthere: bool,
) {
    state.fmt_idx = fmt_idx;
    state.matched = true;
    state.prefix = prefix;
    state.flags = flags;
    state.conthere = conthere;
}

// =============================================================================
// Phase Q1: Errorformat Buffer Size and Part Length
// =============================================================================

/// Compute the size of the buffer needed to convert an 'errorformat' pattern
/// into a regular expression pattern.
///
/// This is a conservative estimate that accounts for:
/// - Pattern prefixes/suffixes (^ and $)
/// - Capture groups for each format specifier
/// - Worst-case expansion of %f pattern
///
/// # Safety
///
/// - `efm` may be null (returns 0)
/// - If non-null, must point to a buffer of at least `efm_len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_efm_regpat_bufsz(efm: *const c_char, efm_len: usize) -> usize {
    if efm.is_null() || efm_len == 0 {
        return 0;
    }

    // Base size: each format specifier can expand, plus overhead
    // - FMT_PATTERNS * 3 for capture group markers \( and \)
    // - efm_len * 4 for worst-case character expansion
    // - Sum of all pattern lengths
    // - Extra buffer for %f expansion (can become 12 chars longer on Windows)
    let mut sz = (FMT_PATTERNS * 3) + (efm_len << 2);

    // Add length of each pattern
    for (_, pattern) in &FMT_PAT {
        sz += pattern.len();
    }

    // %f can become significantly longer depending on platform
    // Windows: 12 chars longer (for drive letter matching)
    // Other: 2 chars longer
    #[cfg(windows)]
    {
        sz += 12;
    }
    #[cfg(not(windows))]
    {
        sz += 2;
    }

    sz
}

/// Return the length of an 'errorformat' option part (separated by ",").
///
/// Handles escaped commas (\\,) as part of the format.
///
/// # Safety
///
/// - `efm` may be null (returns 0)
/// - If non-null, must point to a buffer of at least `efm_max_len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_efm_option_part_len(efm: *const c_char, efm_max_len: usize) -> c_int {
    if efm.is_null() || efm_max_len == 0 {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(efm.cast::<u8>(), efm_max_len);
    let mut len = 0;

    while len < bytes.len() && bytes[len] != 0 && bytes[len] != b',' {
        // Check for escaped character
        if bytes[len] == b'\\' && len + 1 < bytes.len() && bytes[len + 1] != 0 {
            len += 2; // Skip both backslash and next character
        } else {
            len += 1;
        }
    }

    len as c_int
}

// =============================================================================
// Phase 5: qf_parse_match and qf_parse_line migration
//
// These functions use C-compatible status codes (QF_FAIL=0, QF_OK=1, etc.)
// to match the C enum in quickfix_shim.c.
// =============================================================================

/// C-compatible QF status codes (match the C enum in quickfix_shim.c)
const C_QF_FAIL: c_int = 0;
const C_QF_OK: c_int = 1;
// QF_END_OF_INPUT = 2 (not used here)
const C_QF_NOMEM: c_int = 3;
const C_QF_IGNORE_LINE: c_int = 4;
const C_QF_MULTISCAN: c_int = 5;

/// FMT_PATTERNS constant (14 format specifiers)
const C_FMT_PATTERNS: usize = 14;
/// Index of %m (message) pattern
const C_FMT_PATTERN_M: usize = 8;
/// Index of %r (rest) pattern
const C_FMT_PATTERN_R: usize = 9;

/// Handle for an `EfmPattern` node (opaque pointer to `crate::reader::EfmPattern`)
type EfmHandle = *mut c_void;

// C accessor functions needed for parse_match implementation
extern "C" {
    // regmatch_T indexed submatch accessors
    fn nvim_qf_regmatch_startp(rm: *const c_void, idx: c_int) -> *const c_char;
    fn nvim_qf_regmatch_endp(rm: *const c_void, idx: c_int) -> *const c_char;

    // Environment / filesystem helpers
    fn expand_env(src: *const c_char, dst: *mut c_char, dstlen: c_int);
    fn os_path_exists(path: *const c_char) -> bool;
    fn buflist_findnr(bnr: c_int) -> *mut c_void;

    // regmatch_T lifecycle for vim_regexec
    fn nvim_qf_regmatch_create_ic(prog: *mut c_void) -> *mut c_void;
    fn nvim_qf_regmatch_extract_prog(rm: *mut c_void) -> *mut c_void;
    fn nvim_qf_vim_regexec(rm: *mut c_void, line: *const c_char) -> bool;

    // qf_list_T accessors
    fn nvim_qf_get_last(qfl: *const c_void) -> *mut crate::ffi_types::QfLineRaw;

    // qf_list_T multi-scan state
    fn nvim_qf_get_multiline(qfl: *const c_void) -> bool;
    fn nvim_qf_get_multiignore(qfl: *const c_void) -> bool;
    fn nvim_qf_set_multiline(qfl: *mut c_void, multiline: bool);
    fn nvim_qf_set_multiignore(qfl: *mut c_void, multiignore: bool);
    fn nvim_qf_get_multiscan(qfl: *const c_void) -> bool;
    fn nvim_qf_set_multiscan(qfl: *mut c_void, multiscan: bool);

    // qfline_T set accessors (get accessors already declared at top of file)
    // nvim_qfline_replace_text is an alias for nvim_qfline_set_text

    // misc
    fn line_breakcheck();
    fn vim_isprintc(c: c_int) -> bool;
    fn nvim_qf_get_directory(qfl: *const c_void) -> *const c_char;
    fn nvim_qf_get_currfile(qfl: *const c_void) -> *const c_char;
    fn nvim_qf_set_directory(qfl: *mut c_void, dir: *mut c_char);
    fn nvim_qf_set_currfile(qfl: *mut c_void, file: *mut c_char);
    static IObuff: *mut c_char;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn emsg(msg: *const std::ffi::c_char) -> bool;
    // (nvim_qf_emsg_missing_dir deleted: use emsg directly)

    // Dir/file stack helpers (already exist in Rust, but need C side for push/pop)
    fn rs_qf_push_dir(qfl: *mut c_void, dirbuf: *mut c_char, is_file_stack: bool) -> *const c_char;
    fn rs_qf_pop_dir(qfl: *mut c_void, is_file_stack: bool) -> *const c_char;
}

// =============================================================================
// Inline accessors for EfmPattern and QfAllFields (Phase 9 Phase 2)
// =============================================================================
//
// These replace the deleted C accessor functions nvim_efm_get_* and nvim_qf_fields_*.
// The `fields` and `fmt_ptr` parameters are `*mut c_void` pointing to Rust-owned
// EfmPattern / QfAllFields structs.

use crate::reader::{EfmPattern, QfAllFields};

#[inline]
unsafe fn efm_get_prefix(efm: EfmHandle) -> c_char {
    (*efm.cast::<EfmPattern>()).prefix
}

#[inline]
unsafe fn efm_get_flags(efm: EfmHandle) -> c_char {
    (*efm.cast::<EfmPattern>()).flags
}

#[inline]
unsafe fn efm_get_conthere(efm: EfmHandle) -> c_int {
    (*efm.cast::<EfmPattern>()).conthere
}

#[inline]
unsafe fn efm_get_addr(efm: EfmHandle, idx: c_int) -> c_char {
    #[allow(clippy::cast_sign_loss)]
    (*efm.cast::<EfmPattern>()).addr[idx as usize]
}

#[inline]
unsafe fn efm_get_next(efm: EfmHandle) -> EfmHandle {
    (*efm.cast::<EfmPattern>()).next.cast()
}

#[inline]
unsafe fn efm_get_prog(efm: EfmHandle) -> *mut c_void {
    (*efm.cast::<EfmPattern>()).prog
}

#[inline]
unsafe fn efm_set_prog(efm: EfmHandle, prog: *mut c_void) {
    (*efm.cast::<EfmPattern>()).prog = prog;
}

#[inline]
unsafe fn fields_get_namebuf(fields: *mut c_void) -> *mut c_char {
    (*fields.cast::<QfAllFields>()).namebuf
}

#[inline]
unsafe fn fields_set_bnr(fields: *mut c_void, bnr: c_int) {
    (*fields.cast::<QfAllFields>()).bnr = bnr;
}

#[inline]
unsafe fn fields_get_module(fields: *mut c_void) -> *mut c_char {
    (*fields.cast::<QfAllFields>()).module
}

#[inline]
unsafe fn fields_get_errmsg(fields: *mut c_void) -> *mut c_char {
    (*fields.cast::<QfAllFields>()).errmsg
}

#[inline]
unsafe fn fields_set_errmsg(fields: *mut c_void, msg: *const c_char, len: usize) {
    (*fields.cast::<QfAllFields>()).set_errmsg(msg, len);
}

#[inline]
unsafe fn fields_get_lnum(fields: *const c_void) -> i32 {
    (*fields.cast::<QfAllFields>()).lnum
}

#[inline]
unsafe fn fields_set_lnum(fields: *mut c_void, lnum: i32) {
    (*fields.cast::<QfAllFields>()).lnum = lnum;
}

#[inline]
unsafe fn fields_get_end_lnum(fields: *const c_void) -> i32 {
    (*fields.cast::<QfAllFields>()).end_lnum
}

#[inline]
unsafe fn fields_set_end_lnum(fields: *mut c_void, end_lnum: i32) {
    (*fields.cast::<QfAllFields>()).end_lnum = end_lnum;
}

#[inline]
unsafe fn fields_get_col(fields: *const c_void) -> c_int {
    (*fields.cast::<QfAllFields>()).col
}

#[inline]
unsafe fn fields_set_col(fields: *mut c_void, col: c_int) {
    (*fields.cast::<QfAllFields>()).col = col;
}

#[inline]
unsafe fn fields_get_end_col(fields: *const c_void) -> c_int {
    (*fields.cast::<QfAllFields>()).end_col
}

#[inline]
unsafe fn fields_set_end_col(fields: *mut c_void, end_col: c_int) {
    (*fields.cast::<QfAllFields>()).end_col = end_col;
}

#[inline]
unsafe fn fields_get_use_viscol(fields: *const c_void) -> bool {
    (*fields.cast::<QfAllFields>()).use_viscol
}

#[inline]
unsafe fn fields_set_use_viscol(fields: *mut c_void, use_viscol: bool) {
    (*fields.cast::<QfAllFields>()).use_viscol = use_viscol;
}

#[inline]
unsafe fn fields_get_pattern(fields: *mut c_void) -> *mut c_char {
    (*fields.cast::<QfAllFields>()).pattern
}

#[inline]
unsafe fn fields_get_enr(fields: *const c_void) -> c_int {
    (*fields.cast::<QfAllFields>()).enr
}

#[inline]
unsafe fn fields_set_enr(fields: *mut c_void, enr: c_int) {
    (*fields.cast::<QfAllFields>()).enr = enr;
}

#[inline]
unsafe fn fields_get_type(fields: *const c_void) -> c_char {
    (*fields.cast::<QfAllFields>()).type_char
}

#[inline]
unsafe fn fields_set_type(fields: *mut c_void, type_char: c_char) {
    (*fields.cast::<QfAllFields>()).type_char = type_char;
}

#[inline]
unsafe fn fields_set_valid(fields: *mut c_void, valid: bool) {
    (*fields.cast::<QfAllFields>()).valid = valid;
}

// CMDBUFFSIZE: matches C's CMDBUFFSIZE (1024) from os_defs.h

/// Helper: get the length of a matched submatch (end - start).
/// Returns None if start or end is null.
unsafe fn submatch_len(rm: *const c_void, idx: c_int) -> Option<usize> {
    let start = nvim_qf_regmatch_startp(rm, idx);
    let end = nvim_qf_regmatch_endp(rm, idx);
    if start.is_null() || end.is_null() {
        return None;
    }
    // Safety: both pointers are valid, end >= start (regex invariant)
    let len = end.offset_from(start);
    if len < 0 {
        return None;
    }
    #[allow(clippy::cast_sign_loss)]
    Some(len as usize)
}

/// Helper: get submatch as a byte slice.
unsafe fn submatch_bytes<'a>(rm: *const c_void, idx: c_int) -> Option<&'a [u8]> {
    let start = nvim_qf_regmatch_startp(rm, idx);
    let len = submatch_len(rm, idx)?;
    Some(std::slice::from_raw_parts(start.cast::<u8>(), len))
}

/// Parse filename from submatch into fields->namebuf (%f).
/// Expands env vars, checks file existence for file-handler prefixes.
///
/// Uses a local buffer copy to avoid mutating the match pointers.
unsafe fn parse_fmt_f_impl(
    rm: *const c_void,
    midx: c_int,
    fields: *mut c_void,
    prefix: c_char,
) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    let end = nvim_qf_regmatch_endp(rm, midx);
    if start.is_null() || end.is_null() {
        return C_QF_FAIL;
    }
    let len = submatch_len(rm, midx).unwrap_or(0);
    if len == 0 {
        return C_QF_FAIL;
    }

    // Copy submatch to a local buffer (avoids in-place NUL-termination)
    let copy_len = len.min(CMDBUFFSIZE);
    let mut tmp: Vec<u8> = Vec::with_capacity(copy_len + 1);
    tmp.extend_from_slice(std::slice::from_raw_parts(start.cast::<u8>(), copy_len));
    tmp.push(0); // NUL-terminate

    let namebuf = fields_get_namebuf(fields);
    if namebuf.is_null() {
        return C_QF_FAIL;
    }
    // expand_env(tmp, namebuf, CMDBUFFSIZE)
    expand_env(tmp.as_ptr().cast(), namebuf, CMDBUFFSIZE as c_int);

    // For file-handler prefixes (O, P, Q) the file must exist
    if rs_qf_is_file_handler(prefix) && !os_path_exists(namebuf) {
        return C_QF_FAIL;
    }
    C_QF_OK
}

/// Parse buffer number from submatch into fields->bnr (%b).
unsafe fn parse_fmt_b_impl(rm: *const c_void, midx: c_int, fields: *mut c_void) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    if start.is_null() {
        return C_QF_FAIL;
    }
    // atol equivalent
    let bytes = submatch_bytes(rm, midx).unwrap_or(&[]);
    let bnr = parse_number(bytes) as c_int;
    if bnr <= 0 || buflist_findnr(bnr).is_null() {
        return C_QF_FAIL;
    }
    fields_set_bnr(fields, bnr);
    C_QF_OK
}

/// Parse error number from submatch into fields->enr (%n).
unsafe fn parse_fmt_n_impl(rm: *const c_void, midx: c_int, fields: *mut c_void) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    if start.is_null() {
        return C_QF_FAIL;
    }
    let bytes = submatch_bytes(rm, midx).unwrap_or(&[]);
    fields_set_enr(fields, parse_number(bytes) as c_int);
    C_QF_OK
}

/// Parse line number from submatch into fields->lnum (%l).
unsafe fn parse_fmt_l_impl(rm: *const c_void, midx: c_int, fields: *mut c_void) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    if start.is_null() {
        return C_QF_FAIL;
    }
    let bytes = submatch_bytes(rm, midx).unwrap_or(&[]);
    fields_set_lnum(fields, parse_number(bytes) as i32);
    C_QF_OK
}

/// Parse end line number from submatch into fields->end_lnum (%e).
unsafe fn parse_fmt_e_impl(rm: *const c_void, midx: c_int, fields: *mut c_void) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    if start.is_null() {
        return C_QF_FAIL;
    }
    let bytes = submatch_bytes(rm, midx).unwrap_or(&[]);
    fields_set_end_lnum(fields, parse_number(bytes) as i32);
    C_QF_OK
}

/// Parse column number from submatch into fields->col (%c).
unsafe fn parse_fmt_c_impl(rm: *const c_void, midx: c_int, fields: *mut c_void) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    if start.is_null() {
        return C_QF_FAIL;
    }
    let bytes = submatch_bytes(rm, midx).unwrap_or(&[]);
    fields_set_col(fields, parse_number(bytes) as c_int);
    C_QF_OK
}

/// Parse end column number from submatch into fields->end_col (%k).
unsafe fn parse_fmt_k_impl(rm: *const c_void, midx: c_int, fields: *mut c_void) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    if start.is_null() {
        return C_QF_FAIL;
    }
    let bytes = submatch_bytes(rm, midx).unwrap_or(&[]);
    fields_set_end_col(fields, parse_number(bytes) as c_int);
    C_QF_OK
}

/// Parse type character from submatch into fields->type (%t).
unsafe fn parse_fmt_t_impl(rm: *const c_void, midx: c_int, fields: *mut c_void) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    if start.is_null() {
        return C_QF_FAIL;
    }
    #[allow(clippy::cast_possible_wrap)]
    let first = *start.cast::<u8>() as c_char;
    fields_set_type(fields, first);
    C_QF_OK
}

/// Copy entire line as error message (for %+ and nomatch lines).
unsafe fn copy_nonerror_line_impl(
    linebuf: *const c_char,
    linelen: usize,
    fields: *mut c_void,
) -> c_int {
    if linebuf.is_null() {
        return C_QF_FAIL;
    }
    fields_set_errmsg(fields, linebuf, linelen);
    C_QF_OK
}

/// Parse error message from submatch into fields->errmsg (%m).
unsafe fn parse_fmt_m_impl(rm: *const c_void, midx: c_int, fields: *mut c_void) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    let end = nvim_qf_regmatch_endp(rm, midx);
    if start.is_null() || end.is_null() {
        return C_QF_FAIL;
    }
    let len = submatch_len(rm, midx).unwrap_or(0);
    fields_set_errmsg(fields, start, len);
    C_QF_OK
}

/// Extract rest-of-line pointer via submatch (%r). Sets *tail.
unsafe fn parse_fmt_r_impl(rm: *const c_void, midx: c_int, tail: *mut *mut c_char) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    if start.is_null() {
        return C_QF_FAIL;
    }
    if !tail.is_null() {
        *tail = start.cast_mut();
    }
    C_QF_OK
}

/// Parse pointer column (tab-aware) from submatch into fields->col (%p).
unsafe fn parse_fmt_p_impl(rm: *const c_void, midx: c_int, fields: *mut c_void) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    let end = nvim_qf_regmatch_endp(rm, midx);
    if start.is_null() || end.is_null() {
        return C_QF_FAIL;
    }
    let len = submatch_len(rm, midx).unwrap_or(0);
    let bytes = std::slice::from_raw_parts(start.cast::<u8>(), len);
    let mut col: c_int = 0;
    for &b in bytes {
        col += 1;
        if b == b'\t' {
            col += 7;
            col -= col % 8;
        }
    }
    col += 1; // 1-based
    fields_set_col(fields, col);
    fields_set_use_viscol(fields, true);
    C_QF_OK
}

/// Parse virtual column from submatch into fields->col (%v).
unsafe fn parse_fmt_v_impl(rm: *const c_void, midx: c_int, fields: *mut c_void) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    if start.is_null() {
        return C_QF_FAIL;
    }
    let bytes = submatch_bytes(rm, midx).unwrap_or(&[]);
    fields_set_col(fields, parse_number(bytes) as c_int);
    fields_set_use_viscol(fields, true);
    C_QF_OK
}

/// Parse search pattern from submatch into fields->pattern (%s).
unsafe fn parse_fmt_s_impl(rm: *const c_void, midx: c_int, fields: *mut c_void) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    let end = nvim_qf_regmatch_endp(rm, midx);
    if start.is_null() || end.is_null() {
        return C_QF_FAIL;
    }
    let len = submatch_len(rm, midx).unwrap_or(0);
    let len = len.min(CMDBUFFSIZE - 5);
    let pattern_ptr = fields_get_pattern(fields);
    if pattern_ptr.is_null() {
        return C_QF_FAIL;
    }
    // Write "^\V<match>\$\0" into pattern buffer
    let pattern_buf = std::slice::from_raw_parts_mut(pattern_ptr.cast::<u8>(), CMDBUFFSIZE + 1);
    pattern_buf[0] = b'^';
    pattern_buf[1] = b'\\';
    pattern_buf[2] = b'V';
    // Copy match bytes
    let src = std::slice::from_raw_parts(start.cast::<u8>(), len);
    pattern_buf[3..3 + len].copy_from_slice(src);
    pattern_buf[3 + len] = b'\\';
    pattern_buf[3 + len + 1] = b'$';
    pattern_buf[3 + len + 2] = 0;
    C_QF_OK
}

/// Parse module name from submatch, appending to fields->module (%o).
unsafe fn parse_fmt_o_impl(rm: *const c_void, midx: c_int, fields: *mut c_void) -> c_int {
    let start = nvim_qf_regmatch_startp(rm, midx);
    let end = nvim_qf_regmatch_endp(rm, midx);
    if start.is_null() || end.is_null() {
        return C_QF_FAIL;
    }
    let module_ptr = fields_get_module(fields);
    if module_ptr.is_null() {
        return C_QF_FAIL;
    }
    let len = submatch_len(rm, midx).unwrap_or(0);
    // Compute current module length
    let cur_len = libc_strlen(module_ptr.cast::<u8>());
    let dsize = (cur_len + len + 1).min(CMDBUFFSIZE);
    // Append src to module_ptr (dst has CMDBUFFSIZE+1 bytes allocated)
    let src = std::slice::from_raw_parts(start.cast::<u8>(), len);
    let module_buf = std::slice::from_raw_parts_mut(module_ptr.cast::<u8>(), CMDBUFFSIZE + 1);
    let avail = dsize.saturating_sub(cur_len + 1); // bytes available for appending
    let copy_len = len.min(avail);
    module_buf[cur_len..cur_len + copy_len].copy_from_slice(&src[..copy_len]);
    module_buf[cur_len + copy_len] = 0;
    C_QF_OK
}

/// strlen for raw bytes pointer (null-terminated).
fn libc_strlen(p: *const u8) -> usize {
    let mut len = 0;
    unsafe {
        while *p.add(len) != 0 {
            len += 1;
        }
    }
    len
}

/// Implement qf_parse_match: dispatch format extraction across all 14 patterns.
///
/// Takes opaque handles for regmatch (rm), efm format pointer (fmt_ptr),
/// and qffields_T (fields). Sets *tail for %r matches.
///
/// Returns C QF_ status codes.
///
/// # Safety
/// - All pointers must be valid
/// - `rm` must be a valid regmatch_T pointer (from nvim_qf_regmatch_create_ic)
/// - `fmt_ptr` must be a valid efm_T pointer
/// - `fields` must be a valid qffields_T pointer
/// - `linebuf` must point to a valid buffer of `linelen` bytes
/// - `tail` may be null; if non-null must be a valid `*mut *mut c_char`
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rs_qf_parse_match(
    linebuf: *const c_char,
    linelen: usize,
    fmt_ptr: EfmHandle,
    rm: *const c_void,
    fields: *mut c_void,
    qf_multiline: bool,
    qf_multiscan: bool,
    tail: *mut *mut c_char,
) -> c_int {
    let idx = efm_get_prefix(fmt_ptr);

    if rs_qf_is_continuation(idx) && !qf_multiline {
        return C_QF_FAIL;
    }
    fields_set_type(fields, rs_qf_parse_prefix_type(idx));

    for i in 0..C_FMT_PATTERNS {
        #[allow(clippy::cast_possible_wrap)]
        let midx = efm_get_addr(fmt_ptr, i as c_int) as c_int;

        let status = if i == 0 && midx > 0 {
            // %f - filename
            parse_fmt_f_impl(rm, midx, fields, idx)
        } else if i == C_FMT_PATTERN_M {
            let flags = efm_get_flags(fmt_ptr);
            #[allow(clippy::cast_sign_loss)]
            if (flags as u8) == b'+' && !qf_multiscan {
                // %+ - copy whole line
                copy_nonerror_line_impl(linebuf, linelen, fields)
            } else if midx > 0 {
                // %m - message
                parse_fmt_m_impl(rm, midx, fields)
            } else {
                C_QF_OK
            }
        } else if i == C_FMT_PATTERN_R && midx > 0 {
            // %r - rest of line
            parse_fmt_r_impl(rm, midx, tail)
        } else if midx > 0 {
            // other patterns
            match i {
                1 => parse_fmt_b_impl(rm, midx, fields),
                2 => parse_fmt_n_impl(rm, midx, fields),
                3 => parse_fmt_l_impl(rm, midx, fields),
                4 => parse_fmt_e_impl(rm, midx, fields),
                5 => parse_fmt_c_impl(rm, midx, fields),
                6 => parse_fmt_k_impl(rm, midx, fields),
                7 => parse_fmt_t_impl(rm, midx, fields),
                10 => parse_fmt_p_impl(rm, midx, fields),
                11 => parse_fmt_v_impl(rm, midx, fields),
                12 => parse_fmt_s_impl(rm, midx, fields),
                13 => parse_fmt_o_impl(rm, midx, fields),
                _ => C_QF_OK,
            }
        } else {
            C_QF_OK
        };

        if status != C_QF_OK {
            return status;
        }
    }

    C_QF_OK
}

// =============================================================================
// Phase 5 Phase 2: qf_parse_get_fields, qf_parse_dir_pfx, qf_parse_file_pfx,
//                  qf_parse_line_nomatch, qf_parse_multiline_pfx, qf_parse_line
// =============================================================================

/// Implement qf_parse_get_fields: init fields, run regex, call parse_match.
///
/// Creates a regmatch_T, runs vim_regexec, calls parse_match if matched,
/// then frees the regmatch_T and writes prog back to efm_T.
///
/// # Safety
/// All pointers must be valid.
unsafe fn qf_parse_get_fields_rs(
    linebuf: *const c_char,
    linelen: usize,
    fmt_ptr: EfmHandle,
    fields: *mut c_void,
    qf_multiline: bool,
    qf_multiscan: bool,
    tail: *mut *mut c_char,
) -> c_int {
    let prefix = efm_get_prefix(fmt_ptr);
    if qf_multiscan && !rs_qf_is_file_handler(prefix) {
        return C_QF_FAIL;
    }

    // Initialize fields for this line
    let namebuf = fields_get_namebuf(fields);
    if !namebuf.is_null() {
        *namebuf = 0;
    }
    fields_set_bnr(fields, 0);
    let module_ptr = fields_get_module(fields);
    if !module_ptr.is_null() {
        *module_ptr = 0;
    }
    let pattern_ptr = fields_get_pattern(fields);
    if !pattern_ptr.is_null() {
        *pattern_ptr = 0;
    }
    if !qf_multiscan {
        let errmsg_ptr = fields_get_errmsg(fields);
        if !errmsg_ptr.is_null() {
            *errmsg_ptr = 0;
        }
    }
    fields_set_lnum(fields, 0);
    fields_set_end_lnum(fields, 0);
    fields_set_col(fields, 0);
    fields_set_end_col(fields, 0);
    fields_set_use_viscol(fields, false);
    fields_set_enr(fields, -1);
    fields_set_type(fields, 0);
    if !tail.is_null() {
        *tail = std::ptr::null_mut();
    }

    let prog = efm_get_prog(fmt_ptr);
    // Allocate a regmatch_T with rm_ic=true and the given prog
    let rm = nvim_qf_regmatch_create_ic(prog);
    let matched = nvim_qf_vim_regexec(rm, linebuf);
    // The regmatch_T has match info AND potentially updated prog.
    // We call parse_match while rm is still live (before extract_prog frees it).
    let status = if matched {
        rs_qf_parse_match(
            linebuf,
            linelen,
            fmt_ptr,
            rm,
            fields,
            qf_multiline,
            qf_multiscan,
            tail,
        )
    } else {
        C_QF_FAIL
    };
    // Extract prog back (frees rm)
    let updated_prog = nvim_qf_regmatch_extract_prog(rm);
    efm_set_prog(fmt_ptr, updated_prog);

    status
}

/// Implement qf_parse_dir_pfx: handle %D/%X directory enter/leave prefixes.
///
/// # Safety
/// All pointers must be valid.
unsafe fn qf_parse_dir_pfx_rs(idx: c_char, fields: *mut c_void, qfl: *mut c_void) -> c_int {
    #[allow(clippy::cast_sign_loss)]
    match idx as u8 {
        b'D' => {
            // enter directory
            let namebuf = fields_get_namebuf(fields);
            if namebuf.is_null() || *namebuf == 0 {
                emsg(c"E379: Missing or empty directory name".as_ptr());
                return C_QF_FAIL;
            }
            let dir = rs_qf_push_dir(qfl, namebuf, false);
            if dir.is_null() {
                return C_QF_FAIL;
            }
            nvim_qf_set_directory(qfl, dir.cast_mut());
        }
        b'X' => {
            // leave directory
            let dir = rs_qf_pop_dir(qfl, false);
            nvim_qf_set_directory(qfl, dir.cast_mut());
        }
        _ => {}
    }
    C_QF_OK
}

/// Implement qf_parse_file_pfx: handle %O/%P/%Q file push/pop prefixes.
///
/// # Safety
/// All pointers must be valid.
unsafe fn qf_parse_file_pfx_rs(
    idx: c_char,
    fields: *mut c_void,
    qfl: *mut c_void,
    tail: *mut c_char,
) -> c_int {
    fields_set_valid(fields, false);
    let namebuf = fields_get_namebuf(fields);
    #[allow(clippy::cast_sign_loss)]
    let namebuf_empty = namebuf.is_null() || *namebuf == 0;
    let name_exists = !namebuf_empty && os_path_exists(namebuf);

    if namebuf_empty || name_exists {
        #[allow(clippy::cast_sign_loss)]
        if !namebuf_empty && (idx as u8) == b'P' {
            let file = rs_qf_push_dir(qfl, namebuf, true);
            nvim_qf_set_currfile(qfl, file.cast_mut());
        } else {
            #[allow(clippy::cast_sign_loss)]
            if (idx as u8) == b'Q' {
                let file = rs_qf_pop_dir(qfl, true);
                nvim_qf_set_currfile(qfl, file.cast_mut());
            }
        }
        // Clear namebuf
        if !namebuf.is_null() {
            *namebuf = 0;
        }
        if !tail.is_null() && unsafe { *tail != 0 } {
            // STRMOVE(IObuff, skipwhite(tail))
            let skipped = skipwhite(tail);
            let iobuff = IObuff;
            if !iobuff.is_null() && !skipped.is_null() {
                // STRMOVE(dst, src) = memmove(dst, src, strlen(src) + 1)
                libc::memmove(
                    iobuff.cast(),
                    skipped.cast(),
                    libc::strlen(skipped.cast_const()) + 1,
                );
            }
            nvim_qf_set_multiscan(qfl, true);
            return C_QF_MULTISCAN;
        }
    }
    C_QF_OK
}

/// Implement qf_parse_line_nomatch: handle lines that matched no format.
///
/// # Safety
/// All pointers must be valid.
unsafe fn qf_parse_line_nomatch_rs(
    linebuf: *const c_char,
    linelen: usize,
    fields: *mut c_void,
) -> c_int {
    let namebuf = fields_get_namebuf(fields);
    if !namebuf.is_null() {
        *namebuf = 0;
    }
    fields_set_lnum(fields, 0);
    fields_set_valid(fields, false);
    copy_nonerror_line_impl(linebuf, linelen, fields)
}

/// Implement qf_parse_multiline_pfx: handle %C/%Z continuation/end-of-multiline.
///
/// # Safety
/// All pointers must be valid.
unsafe fn qf_parse_multiline_pfx_rs(idx: c_char, qfl: *mut c_void, fields: *mut c_void) -> c_int {
    if !nvim_qf_get_multiignore(qfl) {
        let qfprev = nvim_qf_get_last(qfl);
        if qfprev.is_null() {
            return C_QF_FAIL;
        }
        let qfprev_const = qfprev;

        let errmsg = fields_get_errmsg(fields);
        if !errmsg.is_null() && *errmsg != 0 {
            // Inlined nvim_qfline_append_text (Phase 14):
            // Append errmsg to current qf_text with a newline separator.
            let new_text = std::ffi::CStr::from_ptr(errmsg);
            let new_bytes = new_text.to_bytes();
            let current = (*qfprev).qf_text;
            let combined: std::ffi::CString = if current.is_null() {
                // No existing text; just use the new message directly.
                std::ffi::CString::from_vec_unchecked(new_bytes.to_vec())
            } else {
                let old_bytes = std::ffi::CStr::from_ptr(current).to_bytes();
                let mut buf = Vec::with_capacity(old_bytes.len() + 1 + new_bytes.len() + 1);
                buf.extend_from_slice(old_bytes);
                buf.push(b'\n');
                buf.extend_from_slice(new_bytes);
                std::ffi::CString::from_vec_unchecked(buf)
            };
            {
                xfree((*qfprev).qf_text.cast());
                (*qfprev).qf_text = xstrdup(combined.as_ptr());
            };
        }

        // Update nr if not set
        if (*qfprev_const).qf_nr == -1 {
            (*qfprev).qf_nr = fields_get_enr(fields);
        }

        // Update type if not set and new type is printable
        let new_type = fields_get_type(fields);
        #[allow(clippy::cast_possible_wrap)]
        if vim_isprintc(new_type as c_int) && (*qfprev_const).qf_type == 0 {
            (*qfprev).qf_type = new_type;
        }

        // Update lnum if not set
        if (*qfprev_const).qf_lnum == 0 {
            (*qfprev).qf_lnum = fields_get_lnum(fields);
        }
        // Update end_lnum if not set
        if (*qfprev_const).qf_end_lnum == 0 {
            (*qfprev).qf_end_lnum = fields_get_end_lnum(fields);
        }
        // Update col and viscol if not set
        if (*qfprev_const).qf_col == 0 {
            (*qfprev).qf_col = fields_get_col(fields);
            #[allow(clippy::cast_possible_wrap)]
            let viscol = fields_get_use_viscol(fields) as c_char;
            (*qfprev).qf_viscol = viscol;
        }
        // Update end_col if not set
        if (*qfprev_const).qf_end_col == 0 {
            (*qfprev).qf_end_col = fields_get_end_col(fields);
        }
        // Update fnum if not set
        if (*qfprev_const).qf_fnum == 0 {
            let f = &*fields.cast::<crate::reader::QfAllFields>();
            // Equivalent to C: qf_get_fnum(qfl, qfl->qf_directory,
            //   *namebuf || qfl->qf_directory ? namebuf : qfl->qf_currfile && valid ? qfl->qf_currfile : NULL)
            let dir = nvim_qf_get_directory(qfl.cast_const());
            let currfile = nvim_qf_get_currfile(qfl.cast_const());
            let namebuf_empty = f.namebuf.is_null() || *f.namebuf == 0;
            let fname: *mut c_char = if !namebuf_empty || !dir.is_null() {
                f.namebuf
            } else if !currfile.is_null() && f.valid {
                currfile.cast_mut()
            } else {
                std::ptr::null_mut()
            };
            let fnum = crate::rs_qf_get_fnum(qfl, dir.cast_mut(), fname);
            (*qfprev).qf_fnum = fnum;
        }
    }

    #[allow(clippy::cast_sign_loss)]
    if (idx as u8) == b'Z' {
        nvim_qf_set_multiline(qfl, false);
        nvim_qf_set_multiignore(qfl, false);
    }
    line_breakcheck();

    C_QF_IGNORE_LINE
}

/// Static to hold fmt_start across calls (equivalent to C's `static efm_T *fmt_start`).
/// Reset when free_efm_list is called (via rs_qf_reset_fmt_start).
static mut FMT_START: EfmHandle = std::ptr::null_mut();

/// Reset the fmt_start static (called from free_efm_list in C).
///
/// # Safety
///
/// Must only be called from the main Neovim thread (same thread that owns the quickfix state).
#[no_mangle]
pub unsafe extern "C" fn rs_qf_reset_fmt_start() {
    FMT_START = std::ptr::null_mut();
}

/// Implement qf_parse_line: top-level line parser.
///
/// Iterates efm patterns, dispatches to prefix handlers.
/// Returns C QF_ status codes.
///
/// # Safety
/// - `qfl` must be a valid qf_list_T pointer
/// - `linebuf` must point to a buffer of `linelen` bytes
/// - `fmt_first` must be a valid efm_T pointer (or null for no patterns)
/// - `fields` must be a valid qffields_T pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parse_line(
    qfl: *mut c_void,
    linebuf: *mut c_char,
    linelen: usize,
    fmt_first: EfmHandle,
    fields: *mut c_void,
) -> c_int {
    let mut tail: *mut c_char = std::ptr::null_mut();
    let mut idx: c_char = 0;

    'restofline: loop {
        let mut fmt_ptr = if FMT_START.is_null() {
            fmt_first
        } else {
            let start = FMT_START;
            FMT_START = std::ptr::null_mut();
            start
        };

        // Mark fields as valid at start of each attempt
        fields_set_valid(fields, true);

        // Try each format pattern; initial value C_QF_FAIL used only if fmt_ptr is null
        #[allow(unused_assignments)]
        let mut status = C_QF_FAIL;
        while !fmt_ptr.is_null() {
            idx = efm_get_prefix(fmt_ptr);
            status = qf_parse_get_fields_rs(
                linebuf,
                linelen,
                fmt_ptr,
                fields,
                nvim_qf_get_multiline(qfl),
                nvim_qf_get_multiscan(qfl),
                &raw mut tail,
            );
            if status == C_QF_NOMEM {
                return status;
            }
            if status == C_QF_OK {
                break;
            }
            fmt_ptr = efm_get_next(fmt_ptr);
        }
        nvim_qf_set_multiscan(qfl, false);

        #[allow(clippy::cast_sign_loss)]
        if fmt_ptr.is_null() || (idx as u8) == b'D' || (idx as u8) == b'X' {
            if !fmt_ptr.is_null() {
                // 'D' and 'X' directory specifiers
                let dir_status = qf_parse_dir_pfx_rs(idx, fields, qfl);
                if dir_status != C_QF_OK {
                    return dir_status;
                }
            }
            let nomatch_status = qf_parse_line_nomatch_rs(linebuf, linelen, fields);
            if nomatch_status != C_QF_OK {
                return nomatch_status;
            }
            if fmt_ptr.is_null() {
                nvim_qf_set_multiline(qfl, false);
                nvim_qf_set_multiignore(qfl, false);
            }
        } else {
            // Honor %> item
            if efm_get_conthere(fmt_ptr) != 0 {
                FMT_START = fmt_ptr;
            }

            if rs_qf_starts_multiline(idx) {
                nvim_qf_set_multiline(qfl, true);
                nvim_qf_set_multiignore(qfl, false);
            } else if rs_qf_is_continuation(idx) {
                let ml_status = qf_parse_multiline_pfx_rs(idx, qfl, fields);
                if ml_status != C_QF_OK {
                    return ml_status;
                }
            } else if rs_qf_is_file_handler(idx) {
                let file_status = qf_parse_file_pfx_rs(idx, fields, qfl, tail);
                if file_status == C_QF_MULTISCAN {
                    // goto restofline
                    continue 'restofline;
                }
            }
            if rs_qf_should_skip_line(efm_get_flags(fmt_ptr)) {
                if nvim_qf_get_multiline(qfl) {
                    nvim_qf_set_multiignore(qfl, true);
                }
                return C_QF_IGNORE_LINE;
            }
        }

        return C_QF_OK;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_is_file_prefix() {
        assert!(rs_efm_is_file_prefix(b'O' as c_char));
        assert!(rs_efm_is_file_prefix(b'P' as c_char));
        assert!(rs_efm_is_file_prefix(b'Q' as c_char));
        assert!(!rs_efm_is_file_prefix(b'E' as c_char));
        assert!(!rs_efm_is_file_prefix(b'D' as c_char));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_is_dir_prefix() {
        assert!(rs_efm_is_dir_prefix(b'D' as c_char));
        assert!(rs_efm_is_dir_prefix(b'X' as c_char));
        assert!(!rs_efm_is_dir_prefix(b'E' as c_char));
        assert!(!rs_efm_is_dir_prefix(b'O' as c_char));
    }

    #[test]
    fn test_null_validate_entry() {
        unsafe {
            let validation = rs_qf_validate_entry(std::ptr::null_mut());
            assert!(!validation.valid);
            assert!(!validation.has_filename);
            assert!(!validation.has_lnum);
        }
    }

    #[test]
    fn test_null_entry_is_complete() {
        unsafe {
            assert!(!rs_qf_entry_is_complete(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_null_entry_is_diagnostic() {
        unsafe {
            assert!(!rs_qf_entry_is_diagnostic(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_null_entry_severity() {
        unsafe {
            assert_eq!(rs_qf_entry_severity(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_classify_line() {
        unsafe {
            let class = rs_qf_classify_line(std::ptr::null(), 0);
            assert!(!class.looks_like_error);
            // Null input returns default, which has is_blank = false
            assert!(!class.is_blank);
        }
    }

    #[test]
    fn test_classify_blank_line() {
        unsafe {
            let blank = b"   \t  \n";
            let class = rs_qf_classify_line(blank.as_ptr().cast(), blank.len());
            assert!(class.is_blank);
        }
    }

    #[test]
    fn test_classify_error_line() {
        unsafe {
            let error_line = b"error: something went wrong";
            let class = rs_qf_classify_line(error_line.as_ptr().cast(), error_line.len());
            assert!(class.looks_like_error);
            assert!(class.has_error_keyword);
            assert!(!class.has_warning_keyword);
        }
    }

    #[test]
    fn test_classify_warning_line() {
        unsafe {
            let warning_line = b"warning: this might be a problem";
            let class = rs_qf_classify_line(warning_line.as_ptr().cast(), warning_line.len());
            assert!(class.looks_like_error);
            assert!(class.has_warning_keyword);
        }
    }

    #[test]
    fn test_classify_location_line() {
        unsafe {
            let loc_line = b"file.c:123:45: message";
            let class = rs_qf_classify_line(loc_line.as_ptr().cast(), loc_line.len());
            assert!(class.has_location);
            assert!(class.looks_like_error);
        }
    }

    #[test]
    fn test_null_entry_has_valid_range() {
        unsafe {
            assert!(rs_qf_entry_has_valid_range(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_null_entry_line_span() {
        unsafe {
            assert_eq!(rs_qf_entry_line_span(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_detect_type_from_text() {
        unsafe {
            assert_eq!(rs_qf_detect_type_from_text(std::ptr::null()), 0);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_detect_type_error() {
        unsafe {
            let text = b"error: something failed\0";
            assert_eq!(
                rs_qf_detect_type_from_text(text.as_ptr().cast()),
                b'E' as c_char
            );
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_detect_type_warning() {
        unsafe {
            let text = b"warning: deprecated function\0";
            assert_eq!(
                rs_qf_detect_type_from_text(text.as_ptr().cast()),
                b'W' as c_char
            );
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_detect_type_info() {
        unsafe {
            let text = b"info: additional context\0";
            assert_eq!(
                rs_qf_detect_type_from_text(text.as_ptr().cast()),
                b'I' as c_char
            );
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_detect_type_note() {
        unsafe {
            let text = b"note: see previous definition\0";
            assert_eq!(
                rs_qf_detect_type_from_text(text.as_ptr().cast()),
                b'N' as c_char
            );
        }
    }

    #[test]
    fn test_null_entry_has_module() {
        unsafe {
            assert!(!rs_qf_entry_has_module(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_null_entry_has_nr() {
        unsafe {
            assert!(!rs_qf_entry_has_nr(std::ptr::null_mut()));
        }
    }

    // ==========================================================================
    // QfFields Tests
    // ==========================================================================

    #[test]
    fn test_qf_fields_default() {
        let fields = QfFields::default();
        assert_eq!(fields.bnr, 0);
        assert_eq!(fields.lnum, 0);
        assert_eq!(fields.col, 0);
        assert!(!fields.valid);
        assert!(!fields.use_viscol);
    }

    #[test]
    fn test_qf_fields_has_methods() {
        let mut fields = QfFields::new();
        assert!(!fields.has_lnum());
        assert!(!fields.has_col());
        assert!(!fields.has_buffer());

        fields.lnum = 10;
        fields.col = 5;
        fields.bnr = 1;

        assert!(fields.has_lnum());
        assert!(fields.has_col());
        assert!(fields.has_buffer());
    }

    #[test]
    fn test_qf_fields_reset() {
        let mut fields = QfFields::new();
        fields.lnum = 100;
        fields.col = 50;
        fields.valid = true;

        fields.reset();

        assert_eq!(fields.lnum, 0);
        assert_eq!(fields.col, 0);
        assert!(!fields.valid);
    }

    // ==========================================================================
    // Format Specifier Parsing Tests
    // ==========================================================================

    #[test]
    fn test_parse_fmt_l() {
        let mut fields = QfFields::new();
        let line_str = b"123";
        let result = rs_qf_parse_fmt_l(line_str.as_ptr().cast(), line_str.len(), &mut fields);
        assert_eq!(result, QF_OK);
        assert_eq!(fields.lnum, 123);
    }

    #[test]
    fn test_parse_fmt_l_with_whitespace() {
        let mut fields = QfFields::new();
        let line_str = b"  456";
        let result = rs_qf_parse_fmt_l(line_str.as_ptr().cast(), line_str.len(), &mut fields);
        assert_eq!(result, QF_OK);
        assert_eq!(fields.lnum, 456);
    }

    #[test]
    fn test_parse_fmt_c() {
        let mut fields = QfFields::new();
        let col_str = b"42";
        let result = rs_qf_parse_fmt_c(col_str.as_ptr().cast(), col_str.len(), &mut fields);
        assert_eq!(result, QF_OK);
        assert_eq!(fields.col, 42);
    }

    #[test]
    fn test_parse_fmt_e() {
        let mut fields = QfFields::new();
        let end_str = b"200";
        let result = rs_qf_parse_fmt_e(end_str.as_ptr().cast(), end_str.len(), &mut fields);
        assert_eq!(result, QF_OK);
        assert_eq!(fields.end_lnum, 200);
    }

    #[test]
    fn test_parse_fmt_k() {
        let mut fields = QfFields::new();
        let end_col_str = b"75";
        let result = rs_qf_parse_fmt_k(end_col_str.as_ptr().cast(), end_col_str.len(), &mut fields);
        assert_eq!(result, QF_OK);
        assert_eq!(fields.end_col, 75);
    }

    #[test]
    fn test_parse_fmt_n() {
        let mut fields = QfFields::new();
        let nr_str = b"501";
        let result = rs_qf_parse_fmt_n(nr_str.as_ptr().cast(), nr_str.len(), &mut fields);
        assert_eq!(result, QF_OK);
        assert_eq!(fields.enr, 501);
    }

    #[test]
    fn test_parse_fmt_b() {
        let mut fields = QfFields::new();
        let buf_str = b"5";
        let result = rs_qf_parse_fmt_b(buf_str.as_ptr().cast(), buf_str.len(), &mut fields);
        assert_eq!(result, QF_OK);
        assert_eq!(fields.bnr, 5);
    }

    #[test]
    fn test_parse_fmt_b_invalid() {
        let mut fields = QfFields::new();
        let buf_str = b"0";
        let result = rs_qf_parse_fmt_b(buf_str.as_ptr().cast(), buf_str.len(), &mut fields);
        assert_eq!(result, QF_FAIL);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_parse_fmt_t() {
        let mut fields = QfFields::new();
        let type_str = b"E";
        let result = rs_qf_parse_fmt_t(type_str.as_ptr().cast(), type_str.len(), &mut fields);
        assert_eq!(result, QF_OK);
        assert_eq!(fields.type_char, b'E' as c_char);
    }

    #[test]
    fn test_parse_fmt_v() {
        let mut fields = QfFields::new();
        let vcol_str = b"15";
        let result = rs_qf_parse_fmt_v(vcol_str.as_ptr().cast(), vcol_str.len(), &mut fields);
        assert_eq!(result, QF_OK);
        assert_eq!(fields.col, 15);
        assert!(fields.use_viscol);
    }

    #[test]
    fn test_parse_fmt_p() {
        let mut fields = QfFields::new();
        // Simulate "----^" pointer (5 chars including ^)
        let pointer_str = b"----^";
        let result = rs_qf_parse_fmt_p(pointer_str.as_ptr().cast(), pointer_str.len(), &mut fields);
        assert_eq!(result, QF_OK);
        assert_eq!(fields.col, 6); // 5 + 1 for 1-based
        assert!(fields.use_viscol);
    }

    #[test]
    fn test_parse_fmt_p_with_tab() {
        let mut fields = QfFields::new();
        // Tab counts as multiple columns
        let pointer_str = b"\t^";
        let result = rs_qf_parse_fmt_p(pointer_str.as_ptr().cast(), pointer_str.len(), &mut fields);
        assert_eq!(result, QF_OK);
        // Tab = 1 + 7 = 8, then ^ = 9, then +1 = 10 for 1-based... actually let's trace:
        // col = 0
        // for \t: col += 1 = 1, col += 7 = 8, col -= 8 % 8 = 8 - 0 = 8
        // for ^: col += 1 = 9
        // col += 1 = 10 for 1-based
        assert_eq!(fields.col, 10);
        assert!(fields.use_viscol);
    }

    // ==========================================================================
    // Prefix Tests
    // ==========================================================================

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_is_error_prefix() {
        assert!(rs_efm_is_error_prefix(b'E' as c_char));
        assert!(rs_efm_is_error_prefix(b'W' as c_char));
        assert!(rs_efm_is_error_prefix(b'I' as c_char));
        assert!(rs_efm_is_error_prefix(b'N' as c_char));
        assert!(!rs_efm_is_error_prefix(b'A' as c_char));
        assert!(!rs_efm_is_error_prefix(b'C' as c_char));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_is_continuation_prefix() {
        assert!(rs_efm_is_continuation_prefix(b'C' as c_char));
        assert!(rs_efm_is_continuation_prefix(b'Z' as c_char));
        assert!(!rs_efm_is_continuation_prefix(b'A' as c_char));
        assert!(!rs_efm_is_continuation_prefix(b'E' as c_char));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_is_multiline_prefix() {
        assert!(rs_efm_is_multiline_prefix(b'A' as c_char));
        assert!(rs_efm_is_multiline_prefix(b'C' as c_char));
        assert!(rs_efm_is_multiline_prefix(b'Z' as c_char));
        assert!(!rs_efm_is_multiline_prefix(b'E' as c_char));
        assert!(!rs_efm_is_multiline_prefix(b'G' as c_char));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_is_global_prefix() {
        assert!(rs_efm_is_global_prefix(b'G' as c_char));
        assert!(!rs_efm_is_global_prefix(b'E' as c_char));
        assert!(!rs_efm_is_global_prefix(b'A' as c_char));
    }

    // Note: rs_efm_prefix_to_type tests are in lib.rs

    // ==========================================================================
    // Multiline State Tests
    // ==========================================================================

    #[test]
    fn test_multiline_state_new() {
        let state = QfMultilineState::new();
        assert!(!state.in_multiline);
        assert!(!state.ignore);
        assert_eq!(state.last_prefix, 0);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_multiline_state_update() {
        let mut state = QfMultilineState::new();

        // Start multiline with 'A'
        state.update(b'A' as c_char);
        assert!(state.in_multiline);
        assert!(!state.ignore);

        // Continue with 'C'
        state.update(b'C' as c_char);
        assert!(state.in_multiline);

        // End with 'Z'
        state.update(b'Z' as c_char);
        assert!(!state.in_multiline);
    }

    #[test]
    fn test_multiline_state_reset() {
        let mut state = QfMultilineState::new();
        state.in_multiline = true;
        state.ignore = true;

        state.reset();

        assert!(!state.in_multiline);
        assert!(!state.ignore);
    }

    // ==========================================================================
    // FmtPatternIdx Tests
    // ==========================================================================

    #[test]
    fn test_fmt_pattern_idx_to_char() {
        assert_eq!(FmtPatternIdx::File.to_char(), b'f');
        assert_eq!(FmtPatternIdx::Line.to_char(), b'l');
        assert_eq!(FmtPatternIdx::Column.to_char(), b'c');
        assert_eq!(FmtPatternIdx::Message.to_char(), b'm');
        assert_eq!(FmtPatternIdx::Type.to_char(), b't');
    }

    #[test]
    fn test_fmt_pattern_idx_from_char() {
        assert_eq!(FmtPatternIdx::from_char(b'f'), Some(FmtPatternIdx::File));
        assert_eq!(FmtPatternIdx::from_char(b'l'), Some(FmtPatternIdx::Line));
        assert_eq!(FmtPatternIdx::from_char(b'c'), Some(FmtPatternIdx::Column));
        assert_eq!(FmtPatternIdx::from_char(b'm'), Some(FmtPatternIdx::Message));
        assert_eq!(FmtPatternIdx::from_char(b't'), Some(FmtPatternIdx::Type));
        assert_eq!(FmtPatternIdx::from_char(b'x'), None);
    }

    // ==========================================================================
    // Null Pointer Tests for New Functions
    // ==========================================================================

    #[test]
    fn test_parse_fmt_null() {
        let mut fields = QfFields::new();
        assert_eq!(rs_qf_parse_fmt_l(std::ptr::null(), 0, &mut fields), QF_FAIL);
        assert_eq!(rs_qf_parse_fmt_c(std::ptr::null(), 0, &mut fields), QF_FAIL);
        assert_eq!(rs_qf_parse_fmt_e(std::ptr::null(), 0, &mut fields), QF_FAIL);
        assert_eq!(rs_qf_parse_fmt_k(std::ptr::null(), 0, &mut fields), QF_FAIL);
        assert_eq!(rs_qf_parse_fmt_n(std::ptr::null(), 0, &mut fields), QF_FAIL);
        assert_eq!(rs_qf_parse_fmt_b(std::ptr::null(), 0, &mut fields), QF_FAIL);
        assert_eq!(rs_qf_parse_fmt_t(std::ptr::null(), 0, &mut fields), QF_FAIL);
        assert_eq!(rs_qf_parse_fmt_v(std::ptr::null(), 0, &mut fields), QF_FAIL);
        assert_eq!(rs_qf_parse_fmt_p(std::ptr::null(), 0, &mut fields), QF_FAIL);
    }

    // ==========================================================================
    // Errorformat Pattern Conversion Tests
    // ==========================================================================

    #[test]
    fn test_efm_find_pattern_idx() {
        assert_eq!(rs_efm_find_pattern_idx(b'f' as c_char), 0);
        assert_eq!(rs_efm_find_pattern_idx(b'l' as c_char), 3);
        assert_eq!(rs_efm_find_pattern_idx(b'c' as c_char), 5);
        assert_eq!(rs_efm_find_pattern_idx(b'm' as c_char), 8);
        assert_eq!(rs_efm_find_pattern_idx(b'x' as c_char), -1);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_analyze_prefix_simple() {
        unsafe {
            let prefix = b"E";
            let result = rs_efm_analyze_prefix(prefix.as_ptr().cast(), prefix.len());
            assert_eq!(result.status, QF_OK);
            assert_eq!(result.prefix, b'E' as c_char);
            assert_eq!(result.flags, 0);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_analyze_prefix_with_flag() {
        unsafe {
            let prefix = b"-E";
            let result = rs_efm_analyze_prefix(prefix.as_ptr().cast(), prefix.len());
            assert_eq!(result.status, QF_OK);
            assert_eq!(result.prefix, b'E' as c_char);
            assert_eq!(result.flags, b'-' as c_char);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_analyze_prefix_all_types() {
        unsafe {
            for &prefix_char in b"DXAEWINCZGOPQ" {
                let prefix = [prefix_char];
                let result = rs_efm_analyze_prefix(prefix.as_ptr().cast(), prefix.len());
                assert_eq!(result.status, QF_OK);
                assert_eq!(result.prefix, prefix_char as c_char);
            }
        }
    }

    #[test]
    fn test_efm_analyze_prefix_invalid() {
        unsafe {
            let prefix = b"x";
            let result = rs_efm_analyze_prefix(prefix.as_ptr().cast(), prefix.len());
            assert_eq!(result.status, QF_FAIL);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_pattern_valid_for_prefix() {
        // %l (line number, idx=3) should be valid with E prefix
        assert!(rs_efm_pattern_valid_for_prefix(3, b'E' as c_char));
        // %l should NOT be valid with D prefix
        assert!(!rs_efm_pattern_valid_for_prefix(3, b'D' as c_char));
        // %r (rest, idx=9) should only be valid with O/P/Q
        assert!(rs_efm_pattern_valid_for_prefix(9, b'O' as c_char));
        assert!(!rs_efm_pattern_valid_for_prefix(9, b'E' as c_char));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_scanf_to_regpat_char_class() {
        unsafe {
            let input = b"[^:]";
            let mut output = [0u8; 32];
            let consumed = rs_efm_scanf_to_regpat(
                input.as_ptr().cast(),
                input.len(),
                output.as_mut_ptr().cast(),
                output.len(),
            );
            assert!(consumed > 0);
            // Should produce "[^:]\+"
            let result = std::ffi::CStr::from_ptr(output.as_ptr().cast());
            assert_eq!(result.to_str().unwrap(), "[^:]\\+");
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efm_scanf_to_regpat_escape() {
        unsafe {
            let input = b"\\D";
            let mut output = [0u8; 32];
            let consumed = rs_efm_scanf_to_regpat(
                input.as_ptr().cast(),
                input.len(),
                output.as_mut_ptr().cast(),
                output.len(),
            );
            assert!(consumed > 0);
            // Should produce "\D\+"
            let result = std::ffi::CStr::from_ptr(output.as_ptr().cast());
            assert_eq!(result.to_str().unwrap(), "\\D\\+");
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efmpat_to_regpat_line() {
        unsafe {
            let mut addr = [0u8; FMT_PATTERNS];
            let mut output = [0u8; 64];
            let result = rs_efmpat_to_regpat(
                b'l' as c_char, // line number
                b':' as c_char, // next char
                addr.as_mut_ptr().cast(),
                3, // idx for line number
                0, // round
                0, // no prefix
                output.as_mut_ptr().cast(),
                output.len(),
            );
            assert_eq!(result.status, QF_OK);
            assert_eq!(result.round, 1);
            // Should produce \(\d\+\)
            let result_str = std::ffi::CStr::from_ptr(output.as_ptr().cast());
            assert_eq!(result_str.to_str().unwrap(), "\\(\\d+\\)");
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_efmpat_to_regpat_duplicate() {
        unsafe {
            let mut addr = [0u8; FMT_PATTERNS];
            let mut output = [0u8; 64];

            // First call should succeed
            let result1 = rs_efmpat_to_regpat(
                b'l' as c_char,
                0,
                addr.as_mut_ptr().cast(),
                3,
                0,
                0,
                output.as_mut_ptr().cast(),
                output.len(),
            );
            assert_eq!(result1.status, QF_OK);

            // Second call with same pattern should fail
            let result2 = rs_efmpat_to_regpat(
                b'l' as c_char,
                0,
                addr.as_mut_ptr().cast(),
                3,
                1,
                0,
                output.as_mut_ptr().cast(),
                output.len(),
            );
            assert_eq!(result2.status, QF_FAIL);
        }
    }

    #[test]
    fn test_efm_is_regex_magic() {
        assert!(rs_efm_is_regex_magic(b'.' as c_char));
        assert!(rs_efm_is_regex_magic(b'*' as c_char));
        assert!(rs_efm_is_regex_magic(b'^' as c_char));
        assert!(rs_efm_is_regex_magic(b'$' as c_char));
        assert!(rs_efm_is_regex_magic(b'[' as c_char));
        assert!(!rs_efm_is_regex_magic(b'a' as c_char));
        assert!(!rs_efm_is_regex_magic(b'z' as c_char));
    }

    #[test]
    fn test_efm_is_format_magic() {
        assert!(rs_efm_is_format_magic(b'%' as c_char));
        assert!(rs_efm_is_format_magic(b'\\' as c_char));
        assert!(rs_efm_is_format_magic(b'.' as c_char));
        assert!(!rs_efm_is_format_magic(b'a' as c_char));
        assert!(!rs_efm_is_format_magic(b'l' as c_char));
    }

    // ==========================================================================
    // Full Errorformat to Regex Conversion Tests
    // ==========================================================================

    #[test]
    fn test_efm_to_regpat_simple() {
        unsafe {
            // Simple format: %f:%l: %m
            let efm = b"%f:%l: %m";
            let mut addr = [0u8; FMT_PATTERNS];
            let mut output = [0u8; 256];

            let result = rs_efm_to_regpat(
                efm.as_ptr().cast(),
                efm.len(),
                addr.as_mut_ptr().cast(),
                output.as_mut_ptr().cast(),
                output.len(),
            );

            assert_eq!(result.status, QF_OK);
            assert!(result.bytes_written > 0);

            // Check that output starts with ^ and ends with $
            assert_eq!(output[0], b'^');
            assert_eq!(output[result.bytes_written - 1], b'$');

            // Check that filename pattern was used
            assert_ne!(addr[0], 0); // %f
            assert_ne!(addr[3], 0); // %l
            assert_ne!(addr[8], 0); // %m
        }
    }

    #[test]
    fn test_efm_to_regpat_with_prefix() {
        unsafe {
            // Format with error prefix: %E%f:%l: %m
            let efm = b"%E%f:%l: %m";
            let mut addr = [0u8; FMT_PATTERNS];
            let mut output = [0u8; 256];

            let result = rs_efm_to_regpat(
                efm.as_ptr().cast(),
                efm.len(),
                addr.as_mut_ptr().cast(),
                output.as_mut_ptr().cast(),
                output.len(),
            );

            assert_eq!(result.status, QF_OK);
            assert_eq!(result.prefix, b'E' as c_char);
        }
    }

    #[test]
    fn test_efm_to_regpat_conthere() {
        unsafe {
            // Format with conthere: %>%f:%l: %m
            let efm = b"%>%f:%l: %m";
            let mut addr = [0u8; FMT_PATTERNS];
            let mut output = [0u8; 256];

            let result = rs_efm_to_regpat(
                efm.as_ptr().cast(),
                efm.len(),
                addr.as_mut_ptr().cast(),
                output.as_mut_ptr().cast(),
                output.len(),
            );

            assert_eq!(result.status, QF_OK);
            assert!(result.conthere);
        }
    }

    #[test]
    fn test_efm_to_regpat_escape_metachar() {
        unsafe {
            // Format with regex metacharacter that needs escaping: file.c:%l
            let efm = b"file.c:%l";
            let mut addr = [0u8; FMT_PATTERNS];
            let mut output = [0u8; 256];

            let result = rs_efm_to_regpat(
                efm.as_ptr().cast(),
                efm.len(),
                addr.as_mut_ptr().cast(),
                output.as_mut_ptr().cast(),
                output.len(),
            );

            assert_eq!(result.status, QF_OK);

            // The '.' should be escaped to '\.'
            let result_str = std::ffi::CStr::from_ptr(output.as_ptr().cast());
            let s = result_str.to_str().unwrap();
            assert!(s.contains("\\.c"), "Expected \\. in: {s}");
        }
    }

    #[test]
    fn test_efm_to_regpat_duplicate_pattern() {
        unsafe {
            // Format with duplicate pattern: %l:%l
            let efm = b"%l:%l";
            let mut addr = [0u8; FMT_PATTERNS];
            let mut output = [0u8; 256];

            let result = rs_efm_to_regpat(
                efm.as_ptr().cast(),
                efm.len(),
                addr.as_mut_ptr().cast(),
                output.as_mut_ptr().cast(),
                output.len(),
            );

            assert_eq!(result.status, QF_FAIL);
            assert_eq!(result.error_code, 372); // E372: Too many %l
        }
    }

    #[test]
    fn test_efm_to_regpat_invalid_pattern() {
        unsafe {
            // Format with invalid pattern in middle: %f:%x:%l
            let efm = b"%f:%x:%l";
            let mut addr = [0u8; FMT_PATTERNS];
            let mut output = [0u8; 256];

            let result = rs_efm_to_regpat(
                efm.as_ptr().cast(),
                efm.len(),
                addr.as_mut_ptr().cast(),
                output.as_mut_ptr().cast(),
                output.len(),
            );

            assert_eq!(result.status, QF_FAIL);
            assert_eq!(result.error_code, 377); // E377: Invalid %x
        }
    }

    #[test]
    fn test_efm_to_regpat_scanf() {
        unsafe {
            // Format with scanf pattern: %f:%*[^:]:%l
            let efm = b"%f:%*[^:]:%l";
            let mut addr = [0u8; FMT_PATTERNS];
            let mut output = [0u8; 256];

            let result = rs_efm_to_regpat(
                efm.as_ptr().cast(),
                efm.len(),
                addr.as_mut_ptr().cast(),
                output.as_mut_ptr().cast(),
                output.len(),
            );

            assert_eq!(result.status, QF_OK);

            // Check output contains the scanf pattern converted to regex
            let result_str = std::ffi::CStr::from_ptr(output.as_ptr().cast());
            let s = result_str.to_str().unwrap();
            assert!(s.contains("[^:]\\+"), "Expected [^:]\\+ in: {s}");
        }
    }

    // ==========================================================================
    // Phase Q1: Additional Errorformat Helper Function Tests
    // ==========================================================================

    #[test]
    fn test_efm_regpat_bufsz() {
        unsafe {
            // Simple format: %f:%l: %m
            let efm = b"%f:%l: %m";
            let size = rs_efm_regpat_bufsz(efm.as_ptr().cast(), efm.len());
            // Should be large enough for the regex pattern
            assert!(size > efm.len());
            assert!(size >= 100); // Minimum reasonable size
        }
    }

    #[test]
    fn test_efm_regpat_bufsz_null() {
        unsafe {
            let size = rs_efm_regpat_bufsz(std::ptr::null(), 0);
            assert_eq!(size, 0);
        }
    }

    #[test]
    fn test_efm_option_part_len() {
        unsafe {
            // Simple format ending with comma
            let efm = b"%f:%l: %m,";
            let len = rs_efm_option_part_len(efm.as_ptr().cast(), efm.len());
            assert_eq!(len, 9); // "%f:%l: %m" without the comma
        }
    }

    #[test]
    fn test_efm_option_part_len_no_comma() {
        unsafe {
            // Format without comma at end
            let efm = b"%f:%l: %m";
            let len = rs_efm_option_part_len(efm.as_ptr().cast(), efm.len());
            assert_eq!(len, 9);
        }
    }

    #[test]
    fn test_efm_option_part_len_escaped_comma() {
        unsafe {
            // Format with escaped comma
            let efm = b"%f:%l\\,: %m,next";
            let len = rs_efm_option_part_len(efm.as_ptr().cast(), efm.len());
            // Should include the escaped comma as part of the format
            assert_eq!(len, 11); // "%f:%l\\,: %m"
        }
    }

    #[test]
    fn test_efm_option_part_len_null() {
        unsafe {
            let len = rs_efm_option_part_len(std::ptr::null(), 0);
            assert_eq!(len, 0);
        }
    }

    // ==========================================================================
    // Phase Q2: Line Parsing Helper Tests
    // ==========================================================================

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_parse_prefix_type() {
        assert_eq!(rs_qf_parse_prefix_type(b'E' as c_char), b'E' as c_char);
        assert_eq!(rs_qf_parse_prefix_type(b'W' as c_char), b'W' as c_char);
        assert_eq!(rs_qf_parse_prefix_type(b'I' as c_char), b'I' as c_char);
        assert_eq!(rs_qf_parse_prefix_type(b'N' as c_char), b'N' as c_char);
        assert_eq!(rs_qf_parse_prefix_type(b'A' as c_char), 0);
        assert_eq!(rs_qf_parse_prefix_type(b'C' as c_char), 0);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_should_skip_line() {
        assert!(rs_qf_should_skip_line(b'-' as c_char));
        assert!(!rs_qf_should_skip_line(b'+' as c_char));
        assert!(!rs_qf_should_skip_line(0));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_is_continuation() {
        assert!(rs_qf_is_continuation(b'C' as c_char));
        assert!(rs_qf_is_continuation(b'Z' as c_char));
        assert!(!rs_qf_is_continuation(b'A' as c_char));
        assert!(!rs_qf_is_continuation(b'E' as c_char));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_starts_multiline() {
        assert!(rs_qf_starts_multiline(b'A' as c_char));
        assert!(rs_qf_starts_multiline(b'E' as c_char));
        assert!(rs_qf_starts_multiline(b'W' as c_char));
        assert!(rs_qf_starts_multiline(b'I' as c_char));
        assert!(rs_qf_starts_multiline(b'N' as c_char));
        assert!(!rs_qf_starts_multiline(b'C' as c_char));
        assert!(!rs_qf_starts_multiline(b'Z' as c_char));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_is_dir_handler() {
        assert!(rs_qf_is_dir_handler(b'D' as c_char));
        assert!(rs_qf_is_dir_handler(b'X' as c_char));
        assert!(!rs_qf_is_dir_handler(b'O' as c_char));
        assert!(!rs_qf_is_dir_handler(b'E' as c_char));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_is_file_handler() {
        assert!(rs_qf_is_file_handler(b'O' as c_char));
        assert!(rs_qf_is_file_handler(b'P' as c_char));
        assert!(rs_qf_is_file_handler(b'Q' as c_char));
        assert!(!rs_qf_is_file_handler(b'D' as c_char));
        assert!(!rs_qf_is_file_handler(b'E' as c_char));
    }

    #[test]
    fn test_validate_fields() {
        let mut fields = QfFields::new();
        let result = rs_qf_validate_fields(&fields);
        assert!(!result.has_lnum);
        assert!(!result.has_col);
        assert!(!result.has_buffer);

        fields.lnum = 10;
        let result = rs_qf_validate_fields(&fields);
        assert!(result.has_lnum);
        assert!(!result.has_col);

        fields.col = 5;
        let result = rs_qf_validate_fields(&fields);
        assert!(result.has_lnum);
        assert!(result.has_col);

        fields.bnr = 1;
        let result = rs_qf_validate_fields(&fields);
        assert!(result.has_buffer);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_type_is_printable() {
        assert!(rs_qf_type_is_printable(0));
        assert!(rs_qf_type_is_printable(1));
        assert!(rs_qf_type_is_printable(b'E' as c_char));
        assert!(rs_qf_type_is_printable(b'W' as c_char));
        assert!(rs_qf_type_is_printable(b' ' as c_char));
        assert!(!rs_qf_type_is_printable(0x7F_i8)); // DEL
        assert!(!rs_qf_type_is_printable(0x1F_i8)); // Control char
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_normalize_type() {
        assert_eq!(rs_qf_normalize_type(b'E' as c_char), b'E' as c_char);
        assert_eq!(rs_qf_normalize_type(b'W' as c_char), b'W' as c_char);
        assert_eq!(rs_qf_normalize_type(0), 0);
        assert_eq!(rs_qf_normalize_type(1), 0); // Legacy type becomes 0
        assert_eq!(rs_qf_normalize_type(0x7F_i8), 0); // DEL becomes 0
    }

    #[test]
    fn test_parse_state() {
        let mut state = rs_qf_parse_state_new();
        assert!(!state.matched);
        assert_eq!(state.fmt_idx, 0);
        assert_eq!(state.prefix, 0);

        rs_qf_parse_state_set_match(&mut state, 5, b'E' as c_char, b'-' as c_char, true);
        assert!(state.matched);
        assert_eq!(state.fmt_idx, 5);
        assert_eq!(state.prefix, b'E' as c_char);
        assert_eq!(state.flags, b'-' as c_char);
        assert!(state.conthere);

        rs_qf_parse_state_reset(&mut state);
        assert!(!state.matched);
        assert_eq!(state.fmt_idx, 0);
    }
}
