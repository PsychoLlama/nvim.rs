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
type QfLineHandle = *const c_void;

// =============================================================================
// External C accessor functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Entry accessors
    fn nvim_qfline_get_lnum(qfp: QfLineHandle) -> LinenrT;
    fn nvim_qfline_get_col(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_valid(qfp: QfLineHandle) -> bool;
    fn nvim_qfline_get_type(qfp: QfLineHandle) -> c_char;
    fn nvim_qfline_get_fnum(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_end_lnum(qfp: QfLineHandle) -> LinenrT;
    fn nvim_qfline_get_end_col(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_text(qfp: QfLineHandle) -> *const c_char;
    fn nvim_qfline_get_nr(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_module(qfp: QfLineHandle) -> *const c_char;
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
pub unsafe extern "C" fn rs_qf_validate_entry(qfp: QfLineHandle) -> QfEntryValidation {
    if qfp.is_null() {
        return QfEntryValidation::default();
    }

    let lnum = nvim_qfline_get_lnum(qfp);
    let col = nvim_qfline_get_col(qfp);
    let fnum = nvim_qfline_get_fnum(qfp);
    let type_char = nvim_qfline_get_type(qfp);
    let text = nvim_qfline_get_text(qfp);
    let valid = nvim_qfline_get_valid(qfp);

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
pub unsafe extern "C" fn rs_qf_entry_is_complete(qfp: QfLineHandle) -> bool {
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
pub unsafe extern "C" fn rs_qf_entry_is_diagnostic(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return false;
    }

    let type_char = nvim_qfline_get_type(qfp);
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
pub unsafe extern "C" fn rs_qf_entry_severity(qfp: QfLineHandle) -> c_int {
    if qfp.is_null() {
        return 0;
    }

    let type_char = nvim_qfline_get_type(qfp);
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
pub unsafe extern "C" fn rs_qf_entry_has_valid_range(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return true;
    }

    let lnum = nvim_qfline_get_lnum(qfp);
    let end_lnum = nvim_qfline_get_end_lnum(qfp);

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
pub unsafe extern "C" fn rs_qf_entry_has_valid_col_range(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return true;
    }

    let col = nvim_qfline_get_col(qfp);
    let end_col = nvim_qfline_get_end_col(qfp);
    let lnum = nvim_qfline_get_lnum(qfp);
    let end_lnum = nvim_qfline_get_end_lnum(qfp);

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
pub unsafe extern "C" fn rs_qf_entry_line_span(qfp: QfLineHandle) -> c_int {
    if qfp.is_null() {
        return 0;
    }

    let lnum = nvim_qfline_get_lnum(qfp);
    let end_lnum = nvim_qfline_get_end_lnum(qfp);

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
pub unsafe extern "C" fn rs_qf_entry_has_module(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return false;
    }

    let module = nvim_qfline_get_module(qfp);
    !module.is_null() && *module != 0
}

/// Check if an entry has an error number set.
///
/// # Safety
///
/// - `qfp` may be null (returns false)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_has_nr(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return false;
    }

    nvim_qfline_get_nr(qfp) != 0
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
            let validation = rs_qf_validate_entry(std::ptr::null());
            assert!(!validation.valid);
            assert!(!validation.has_filename);
            assert!(!validation.has_lnum);
        }
    }

    #[test]
    fn test_null_entry_is_complete() {
        unsafe {
            assert!(!rs_qf_entry_is_complete(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_entry_is_diagnostic() {
        unsafe {
            assert!(!rs_qf_entry_is_diagnostic(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_entry_severity() {
        unsafe {
            assert_eq!(rs_qf_entry_severity(std::ptr::null()), 0);
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
            assert!(rs_qf_entry_has_valid_range(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_entry_line_span() {
        unsafe {
            assert_eq!(rs_qf_entry_line_span(std::ptr::null()), 0);
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
            assert!(!rs_qf_entry_has_module(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_entry_has_nr() {
        unsafe {
            assert!(!rs_qf_entry_has_nr(std::ptr::null()));
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
}
