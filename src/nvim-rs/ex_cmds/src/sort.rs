//! `:sort` command implementation.
//!
//! The `:sort` command sorts lines in the buffer.
//!
//! ## Usage
//! - `:{range}sort` - Sort lines alphabetically
//! - `:{range}sort!` - Sort in reverse order
//! - `:{range}sort i` - Case-insensitive sort
//! - `:{range}sort n` - Sort by decimal number
//! - `:{range}sort f` - Sort by floating-point number
//! - `:{range}sort x` - Sort by hexadecimal number
//! - `:{range}sort o` - Sort by octal number
//! - `:{range}sort b` - Sort by binary number
//! - `:{range}sort u` - Remove duplicate lines
//! - `:{range}sort /pattern/` - Sort by text matching pattern
//! - `:{range}sort r /pattern/` - Sort by the matched text itself
//! - `:{range}sort l` - Sort using locale (strcoll)
//!
//! ## Implementation Notes
//!
//! This module provides type definitions for sort operations.
//! The actual sorting is performed by the C implementation using qsort.

use std::ffi::c_int;

use crate::range::{LineNr, LineRange};

/// Numeric sort mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum NumericMode {
    /// No numeric sorting (text sort)
    #[default]
    None = 0,
    /// Sort by decimal number (n flag)
    Decimal = 1,
    /// Sort by floating-point number (f flag)
    Float = 2,
    /// Sort by hexadecimal number (x flag)
    Hex = 3,
    /// Sort by octal number (o flag)
    Octal = 4,
    /// Sort by binary number (b flag)
    Binary = 5,
}

impl NumericMode {
    /// Check if this is any kind of numeric sort.
    #[inline]
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        !matches!(self, NumericMode::None)
    }

    /// Check if this is integer-based numeric sort.
    #[inline]
    #[must_use]
    pub const fn is_integer(&self) -> bool {
        matches!(
            self,
            NumericMode::Decimal | NumericMode::Hex | NumericMode::Octal | NumericMode::Binary
        )
    }

    /// Check if this is floating-point sort.
    #[inline]
    #[must_use]
    pub const fn is_float(&self) -> bool {
        matches!(self, NumericMode::Float)
    }

    /// Convert from C integer.
    #[inline]
    #[must_use]
    pub fn from_c(value: c_int) -> Self {
        match value {
            0 => NumericMode::None,
            1 => NumericMode::Decimal,
            2 => NumericMode::Float,
            3 => NumericMode::Hex,
            4 => NumericMode::Octal,
            5 => NumericMode::Binary,
            _ => NumericMode::None,
        }
    }

    /// Convert to C integer.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        self as c_int
    }
}

/// Flags for the `:sort` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SortFlags {
    /// Ignore case (i flag).
    pub ignore_case: bool,
    /// Use locale-aware comparison (l flag).
    pub use_locale: bool,
    /// Reverse order (! modifier).
    pub reverse: bool,
    /// Remove duplicate lines (u flag).
    pub unique: bool,
    /// Sort by matched pattern text (r flag).
    pub match_text: bool,
    /// Numeric sort mode.
    pub numeric: NumericMode,
}

impl SortFlags {
    /// Create new flags with defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ignore_case: false,
            use_locale: false,
            reverse: false,
            unique: false,
            match_text: false,
            numeric: NumericMode::None,
        }
    }

    /// Parse flags from a flag string.
    ///
    /// # Returns
    /// The parsed flags, or an error if invalid.
    pub fn parse(flags: &str) -> Result<Self, SortError> {
        let mut result = Self::new();
        let mut numeric_count = 0;

        for c in flags.chars() {
            match c {
                'i' => result.ignore_case = true,
                'l' => result.use_locale = true,
                'u' => result.unique = true,
                'r' => result.match_text = true,
                'n' => {
                    result.numeric = NumericMode::Decimal;
                    numeric_count += 1;
                }
                'f' => {
                    result.numeric = NumericMode::Float;
                    numeric_count += 1;
                }
                'x' => {
                    result.numeric = NumericMode::Hex;
                    numeric_count += 1;
                }
                'o' => {
                    result.numeric = NumericMode::Octal;
                    numeric_count += 1;
                }
                'b' => {
                    result.numeric = NumericMode::Binary;
                    numeric_count += 1;
                }
                ' ' | '\t' => { /* skip whitespace */ }
                _ => return Err(SortError::InvalidFlag(c)),
            }
        }

        // Can only have one numeric mode
        if numeric_count > 1 {
            return Err(SortError::MultipleNumericModes);
        }

        Ok(result)
    }

    /// Check if this is a text-based sort.
    #[inline]
    #[must_use]
    pub const fn is_text_sort(&self) -> bool {
        !self.numeric.is_numeric()
    }
}

/// Options for the sort command.
#[derive(Debug, Clone, Default)]
pub struct SortOptions {
    /// Range of lines to sort.
    pub range: LineRange,
    /// Sort flags.
    pub flags: SortFlags,
    /// Pattern for pattern-based sorting (optional).
    pub pattern: Option<String>,
}

impl SortOptions {
    /// Create options for a simple sort.
    #[must_use]
    pub fn simple(range: LineRange) -> Self {
        Self {
            range,
            flags: SortFlags::new(),
            pattern: None,
        }
    }

    /// Create options for a reverse sort.
    #[must_use]
    pub fn reverse(range: LineRange) -> Self {
        Self {
            range,
            flags: SortFlags {
                reverse: true,
                ..SortFlags::new()
            },
            pattern: None,
        }
    }

    /// Create options for a numeric sort.
    #[must_use]
    pub fn numeric(range: LineRange) -> Self {
        Self {
            range,
            flags: SortFlags {
                numeric: NumericMode::Decimal,
                ..SortFlags::new()
            },
            pattern: None,
        }
    }

    /// Create options for unique sort.
    #[must_use]
    pub fn unique(range: LineRange) -> Self {
        Self {
            range,
            flags: SortFlags {
                unique: true,
                ..SortFlags::new()
            },
            pattern: None,
        }
    }
}

/// Result of a sort operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SortResult {
    /// Number of lines sorted.
    pub lines_sorted: i32,
    /// Number of duplicate lines removed (when unique flag set).
    pub duplicates_removed: i32,
    /// Whether the buffer was actually changed.
    pub changed: bool,
    /// Whether the operation was interrupted.
    pub interrupted: bool,
}

impl SortResult {
    /// Create a new empty result.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            lines_sorted: 0,
            duplicates_removed: 0,
            changed: false,
            interrupted: false,
        }
    }

    /// Mark the buffer as changed.
    pub fn set_changed(&mut self) {
        self.changed = true;
    }

    /// Set the number of sorted lines.
    pub fn set_lines_sorted(&mut self, count: i32) {
        self.lines_sorted = count;
    }

    /// Set the number of duplicates removed.
    pub fn set_duplicates_removed(&mut self, count: i32) {
        self.duplicates_removed = count;
    }

    /// Mark as interrupted.
    pub fn set_interrupted(&mut self) {
        self.interrupted = true;
    }
}

/// Error type for sort operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortError {
    /// Invalid flag character.
    InvalidFlag(char),
    /// Multiple numeric modes specified.
    MultipleNumericModes,
    /// Invalid pattern.
    InvalidPattern(String),
    /// No previous pattern.
    NoPreviousPattern,
    /// Invalid range.
    InvalidRange,
    /// Operation was interrupted.
    Interrupted,
}

impl std::fmt::Display for SortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortError::InvalidFlag(c) => write!(f, "invalid flag: {c}"),
            SortError::MultipleNumericModes => {
                write!(f, "can only have one of 'n', 'f', 'b', 'o', 'x'")
            }
            SortError::InvalidPattern(msg) => write!(f, "invalid pattern: {msg}"),
            SortError::NoPreviousPattern => write!(f, "no previous pattern"),
            SortError::InvalidRange => write!(f, "invalid range"),
            SortError::Interrupted => write!(f, "interrupted"),
        }
    }
}

impl std::error::Error for SortError {}

/// Information about a line for sorting.
///
/// This struct holds either the numeric value or string position
/// information needed for comparing lines during sort.
#[derive(Debug, Clone, Copy)]
pub enum SortKey {
    /// Text-based sort: start and end column positions.
    Text { start_col: i32, end_col: i32 },
    /// Integer sort.
    Integer { value: i64, is_number: bool },
    /// Float sort.
    Float { value: f64 },
}

impl SortKey {
    /// Create a text sort key.
    #[must_use]
    pub const fn text(start_col: i32, end_col: i32) -> Self {
        Self::Text { start_col, end_col }
    }

    /// Create an integer sort key.
    #[must_use]
    pub const fn integer(value: i64, is_number: bool) -> Self {
        Self::Integer { value, is_number }
    }

    /// Create a float sort key.
    #[must_use]
    pub const fn float(value: f64) -> Self {
        Self::Float { value }
    }

    /// Create a sort key for a line with no number (sorts before numbers).
    #[must_use]
    pub const fn no_number() -> Self {
        Self::Integer {
            value: 0,
            is_number: false,
        }
    }

    /// Create a sort key for an empty line in float sort.
    #[must_use]
    pub fn empty_float() -> Self {
        Self::Float { value: f64::MIN }
    }
}

/// A line entry for sorting.
#[derive(Debug, Clone, Copy)]
pub struct SortEntry {
    /// Line number (1-based).
    pub lnum: LineNr,
    /// Sort key for comparison.
    pub key: SortKey,
}

impl SortEntry {
    /// Create a new sort entry.
    #[must_use]
    pub const fn new(lnum: LineNr, key: SortKey) -> Self {
        Self { lnum, key }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Parse sort flags from a string.
///
/// Returns a bitmask:
/// - bit 0: ignore_case
/// - bit 1: use_locale
/// - bit 2: reverse
/// - bit 3: unique
/// - bit 4: match_text
/// - bits 5-7: numeric mode (0-5)
///
/// Returns -1 on error.
///
/// # Safety
/// The `flags` pointer must be null or point to a valid null-terminated C string.
pub unsafe extern "C" fn rs_parse_sort_flags(flags: *const std::ffi::c_char) -> c_int {
    if flags.is_null() {
        return 0; // No flags = default
    }

    let flags_str = match std::ffi::CStr::from_ptr(flags).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match SortFlags::parse(flags_str) {
        Ok(f) => {
            let mut result: c_int = 0;
            if f.ignore_case {
                result |= 1 << 0;
            }
            if f.use_locale {
                result |= 1 << 1;
            }
            if f.reverse {
                result |= 1 << 2;
            }
            if f.unique {
                result |= 1 << 3;
            }
            if f.match_text {
                result |= 1 << 4;
            }
            result |= (f.numeric.to_c() & 0x7) << 5;
            result
        }
        Err(_) => -1,
    }
}

/// Check if a numeric mode is integer-based.
pub extern "C" fn rs_sort_numeric_is_integer(mode: c_int) -> c_int {
    c_int::from(NumericMode::from_c(mode).is_integer())
}

/// Check if a numeric mode is float-based.
pub extern "C" fn rs_sort_numeric_is_float(mode: c_int) -> c_int {
    c_int::from(NumericMode::from_c(mode).is_float())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_mode() {
        assert!(!NumericMode::None.is_numeric());
        assert!(NumericMode::Decimal.is_numeric());
        assert!(NumericMode::Float.is_numeric());
        assert!(NumericMode::Hex.is_numeric());

        assert!(NumericMode::Decimal.is_integer());
        assert!(NumericMode::Hex.is_integer());
        assert!(!NumericMode::Float.is_integer());
        assert!(!NumericMode::None.is_integer());

        assert!(NumericMode::Float.is_float());
        assert!(!NumericMode::Decimal.is_float());
    }

    #[test]
    fn test_numeric_mode_from_c() {
        assert_eq!(NumericMode::from_c(0), NumericMode::None);
        assert_eq!(NumericMode::from_c(1), NumericMode::Decimal);
        assert_eq!(NumericMode::from_c(2), NumericMode::Float);
        assert_eq!(NumericMode::from_c(3), NumericMode::Hex);
        assert_eq!(NumericMode::from_c(4), NumericMode::Octal);
        assert_eq!(NumericMode::from_c(5), NumericMode::Binary);
        assert_eq!(NumericMode::from_c(99), NumericMode::None);
    }

    #[test]
    fn test_sort_flags_new() {
        let flags = SortFlags::new();
        assert!(!flags.ignore_case);
        assert!(!flags.use_locale);
        assert!(!flags.reverse);
        assert!(!flags.unique);
        assert!(!flags.match_text);
        assert_eq!(flags.numeric, NumericMode::None);
        assert!(flags.is_text_sort());
    }

    #[test]
    fn test_sort_flags_parse() {
        // Empty string
        let flags = SortFlags::parse("").unwrap();
        assert!(!flags.ignore_case);

        // Single flags
        let flags = SortFlags::parse("i").unwrap();
        assert!(flags.ignore_case);

        let flags = SortFlags::parse("u").unwrap();
        assert!(flags.unique);

        let flags = SortFlags::parse("n").unwrap();
        assert_eq!(flags.numeric, NumericMode::Decimal);

        // Multiple flags
        let flags = SortFlags::parse("iu").unwrap();
        assert!(flags.ignore_case);
        assert!(flags.unique);

        // Numeric modes
        let flags = SortFlags::parse("f").unwrap();
        assert_eq!(flags.numeric, NumericMode::Float);

        let flags = SortFlags::parse("x").unwrap();
        assert_eq!(flags.numeric, NumericMode::Hex);

        let flags = SortFlags::parse("o").unwrap();
        assert_eq!(flags.numeric, NumericMode::Octal);

        let flags = SortFlags::parse("b").unwrap();
        assert_eq!(flags.numeric, NumericMode::Binary);
    }

    #[test]
    fn test_sort_flags_parse_multiple_numeric() {
        // Can't have multiple numeric modes
        let result = SortFlags::parse("nf");
        assert!(matches!(result, Err(SortError::MultipleNumericModes)));

        let result = SortFlags::parse("xo");
        assert!(matches!(result, Err(SortError::MultipleNumericModes)));
    }

    #[test]
    fn test_sort_flags_parse_invalid() {
        let result = SortFlags::parse("z");
        assert!(matches!(result, Err(SortError::InvalidFlag('z'))));
    }

    #[test]
    fn test_sort_options() {
        let range = LineRange::new(1, 100);

        let opts = SortOptions::simple(range);
        assert!(!opts.flags.reverse);
        assert!(!opts.flags.unique);

        let opts = SortOptions::reverse(range);
        assert!(opts.flags.reverse);

        let opts = SortOptions::numeric(range);
        assert_eq!(opts.flags.numeric, NumericMode::Decimal);

        let opts = SortOptions::unique(range);
        assert!(opts.flags.unique);
    }

    #[test]
    fn test_sort_result() {
        let mut result = SortResult::new();
        assert!(!result.changed);
        assert_eq!(result.lines_sorted, 0);

        result.set_changed();
        assert!(result.changed);

        result.set_lines_sorted(100);
        assert_eq!(result.lines_sorted, 100);

        result.set_duplicates_removed(5);
        assert_eq!(result.duplicates_removed, 5);

        result.set_interrupted();
        assert!(result.interrupted);
    }

    #[test]
    fn test_sort_error_display() {
        let err = SortError::InvalidFlag('z');
        assert_eq!(format!("{err}"), "invalid flag: z");

        let err = SortError::MultipleNumericModes;
        assert!(format!("{err}").contains("can only have one"));

        let err = SortError::InvalidRange;
        assert_eq!(format!("{err}"), "invalid range");
    }

    #[test]
    fn test_sort_key() {
        let key = SortKey::text(0, 10);
        assert!(matches!(key, SortKey::Text { .. }));

        let key = SortKey::integer(42, true);
        assert!(matches!(
            key,
            SortKey::Integer {
                value: 42,
                is_number: true
            }
        ));

        let key = SortKey::no_number();
        assert!(matches!(
            key,
            SortKey::Integer {
                is_number: false,
                ..
            }
        ));

        let key = SortKey::float(3.5);
        assert!(matches!(key, SortKey::Float { .. }));

        let key = SortKey::empty_float();
        assert!(matches!(key, SortKey::Float { .. }));
    }

    #[test]
    fn test_sort_entry() {
        let entry = SortEntry::new(10, SortKey::text(0, 5));
        assert_eq!(entry.lnum, 10);
    }

    #[test]
    fn test_rs_parse_sort_flags() {
        use std::ffi::CString;

        let flags = CString::new("iu").unwrap();
        let result = unsafe { rs_parse_sort_flags(flags.as_ptr()) };
        assert!(result >= 0);
        assert_eq!(result & 1, 1); // ignore_case
        assert_eq!(result & 8, 8); // unique

        let flags = CString::new("n").unwrap();
        let result = unsafe { rs_parse_sort_flags(flags.as_ptr()) };
        assert!(result >= 0);
        let numeric = (result >> 5) & 0x7;
        assert_eq!(numeric, 1); // Decimal

        // Invalid - multiple numeric modes
        let flags = CString::new("nf").unwrap();
        let result = unsafe { rs_parse_sort_flags(flags.as_ptr()) };
        assert_eq!(result, -1);
    }

    #[test]
    fn test_rs_sort_numeric_is_integer() {
        assert_eq!(rs_sort_numeric_is_integer(0), 0); // None
        assert_eq!(rs_sort_numeric_is_integer(1), 1); // Decimal
        assert_eq!(rs_sort_numeric_is_integer(2), 0); // Float
        assert_eq!(rs_sort_numeric_is_integer(3), 1); // Hex
    }

    #[test]
    fn test_rs_sort_numeric_is_float() {
        assert_eq!(rs_sort_numeric_is_float(0), 0); // None
        assert_eq!(rs_sort_numeric_is_float(1), 0); // Decimal
        assert_eq!(rs_sort_numeric_is_float(2), 1); // Float
    }
}
