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

use std::ffi::{c_char, c_int};

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
// Constants matching C definitions
// =============================================================================

/// STR2NR_BIN: allow binary numbers.
const STR2NR_BIN: c_int = 1 << 0;
/// STR2NR_OCT: allow octal numbers.
const STR2NR_OCT: c_int = 1 << 1;
/// STR2NR_HEX: allow hexadecimal numbers.
const STR2NR_HEX: c_int = 1 << 2;
/// STR2NR_FORCE: force particular base.
const STR2NR_FORCE: c_int = 1 << 7;

/// MAXLNUM: maximum (invalid) line number.
const MAXLNUM: c_int = 0x7fff_ffff;

/// RE_MAGIC: 'magic' option for regex.
const RE_MAGIC: c_int = 1;

/// kExtmarkNOOP: extmarks shouldn't be moved.
const KEXTMARK_NOOP: c_int = 0;
/// kExtmarkUndo: operation should be reversible.
const KEXTMARK_UNDO: c_int = 1;

/// BL_WHITE | BL_FIX for beginline().
const BL_WHITE_FIX: c_int = 1 | 4;

/// FAIL constant from vim_defs.h
const FAIL: c_int = 0;

// =============================================================================
// Internal sort helpers
// =============================================================================

/// Compare two C strings for sorting, respecting locale and case flags.
unsafe fn string_compare(
    s1: *const c_char,
    s2: *const c_char,
    use_locale: bool,
    ignore_case: bool,
) -> i32 {
    if use_locale {
        libc::strcoll(s1, s2) as i32
    } else if ignore_case {
        // Case-insensitive comparison
        let mut p1 = s1 as *const u8;
        let mut p2 = s2 as *const u8;
        loop {
            let c1 = (*p1).to_ascii_uppercase();
            let c2 = (*p2).to_ascii_uppercase();
            if c1 != c2 {
                return i32::from(c1) - i32::from(c2);
            }
            if c1 == 0 {
                return 0;
            }
            p1 = p1.add(1);
            p2 = p2.add(1);
        }
    } else {
        libc::strcmp(s1, s2) as i32
    }
}

/// Internal sort entry used during sorting.
#[derive(Clone, Copy)]
struct InternalSortEntry {
    lnum: c_int,
    key: InternalSortKey,
}

#[derive(Clone, Copy)]
enum InternalSortKey {
    Text { start_col: c_int, end_col: c_int },
    Integer { value: i64, is_number: bool },
    Float { value: f64 },
}

// =============================================================================
// FFI Exports
// =============================================================================

/// `:sort` command implementation.
///
/// # Safety
/// `eap` must be a valid pointer to an exarg_T.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_sort(eap: *mut crate::ExArgHandle) {
    use crate::{
        beginline, changed_lines, check_nextcmd, fast_breakcheck, last_search_pat, ml_append,
        ml_delete, ml_get, ml_get_len, msgmore, nvim_curwin_set_cursor_lnum, nvim_exarg_get_arg,
        nvim_exarg_get_forceit, nvim_exarg_get_line1, nvim_exarg_get_line2, nvim_exarg_set_nextcmd,
        nvim_excmds_emsg_interr, nvim_excmds_emsg_invarg, nvim_excmds_emsg_noprevre,
        nvim_excmds_extmark_splice, nvim_excmds_mark_adjust, nvim_excmds_regcomp,
        nvim_excmds_regfree, nvim_excmds_regmatch_endp0, nvim_excmds_regmatch_set_ic,
        nvim_excmds_regmatch_startp0, nvim_excmds_semsg_invarg2, nvim_excmds_str2nr,
        nvim_get_curbuf, skip_regexp_err, skiptobin, skiptodigit, skiptohex, u_save, vim_regexec,
    };

    let line1 = nvim_exarg_get_line1(eap);
    let line2 = nvim_exarg_get_line2(eap);
    let count = (line2 - line1 + 1) as usize;

    // Sorting one line is really quick!
    if count <= 1 {
        return;
    }

    if u_save(line1 - 1, line2 + 1) == FAIL {
        return;
    }

    let mut regmatch: *mut crate::RegmatchHandle = std::ptr::null_mut();
    let mut sort_ic = false;
    let mut sort_lc = false;
    let mut sort_rx = false;
    let mut sort_nr = false;
    let mut sort_flt = false;
    let mut unique = false;
    let mut sort_what: c_int = 0;
    let mut format_found: usize = 0;
    let mut change_occurred = false;

    // Parse flags from eap->arg
    let mut p = nvim_exarg_get_arg(eap) as *mut c_char;
    loop {
        let c = *p;
        if c == 0 {
            break;
        }
        if c == b' ' as c_char || c == b'\t' as c_char {
            // skip whitespace
        } else if c == b'i' as c_char {
            sort_ic = true;
        } else if c == b'l' as c_char {
            sort_lc = true;
        } else if c == b'r' as c_char {
            sort_rx = true;
        } else if c == b'n' as c_char {
            sort_nr = true;
            format_found += 1;
        } else if c == b'f' as c_char {
            sort_flt = true;
            format_found += 1;
        } else if c == b'b' as c_char {
            sort_what = STR2NR_BIN + STR2NR_FORCE;
            format_found += 1;
        } else if c == b'o' as c_char {
            sort_what = STR2NR_OCT + STR2NR_FORCE;
            format_found += 1;
        } else if c == b'x' as c_char {
            sort_what = STR2NR_HEX + STR2NR_FORCE;
            format_found += 1;
        } else if c == b'u' as c_char {
            unique = true;
        } else if c == b'"' as c_char {
            // comment start
            break;
        } else if !check_nextcmd(p).is_null() {
            nvim_exarg_set_nextcmd(eap, check_nextcmd(p));
            break;
        } else if !((c as u8).is_ascii_alphabetic()) && regmatch.is_null() {
            let s = skip_regexp_err(p.add(1), c as c_int, 1);
            if s.is_null() {
                return; // goto sortend equivalent -- but nothing to clean up yet
            }
            *s = 0; // NUL-terminate the pattern

            // Use last search pattern if sort pattern is empty.
            if std::ptr::eq(s, p.add(1)) {
                let last_pat = last_search_pat();
                if last_pat.is_null() {
                    nvim_excmds_emsg_noprevre();
                    return;
                }
                regmatch = nvim_excmds_regcomp(last_pat, RE_MAGIC);
            } else {
                regmatch = nvim_excmds_regcomp(p.add(1), RE_MAGIC);
            }
            if regmatch.is_null() {
                return;
            }
            p = s; // continue after the regexp
            nvim_excmds_regmatch_set_ic(regmatch, crate::p_ic);
        } else {
            nvim_excmds_semsg_invarg2(p);
            nvim_excmds_regfree(regmatch);
            return;
        }
        p = p.add(1);
    }

    // Can only have one of 'n', 'f', 'b', 'o' and 'x'.
    if format_found > 1 {
        nvim_excmds_emsg_invarg();
        nvim_excmds_regfree(regmatch);
        return;
    }

    // From here on "sort_nr" is used as a flag for any integer number sorting.
    if sort_what != 0 {
        sort_nr = true;
    }

    // Build sort entries for each line.
    let mut nrs: Vec<InternalSortEntry> = Vec::with_capacity(count);
    let mut maxlen: c_int = 0;

    for lnum in line1..=line2 {
        let s = ml_get(lnum);
        let len = ml_get_len(lnum);
        if len > maxlen {
            maxlen = len;
        }

        let mut start_col: c_int = 0;
        let mut end_col: c_int = len;
        if !regmatch.is_null() && vim_regexec(regmatch, s, 0) {
            if sort_rx {
                start_col = nvim_excmds_regmatch_startp0(regmatch).offset_from(s) as c_int;
                end_col = nvim_excmds_regmatch_endp0(regmatch).offset_from(s) as c_int;
            } else {
                start_col = nvim_excmds_regmatch_endp0(regmatch).offset_from(s) as c_int;
            }
        } else if !regmatch.is_null() {
            end_col = 0;
        }

        let key = if sort_nr || sort_flt {
            // Temporarily NUL-terminate at end_col
            let s2 = s.add(end_col as usize) as *mut c_char;
            let saved_c = *s2;
            *s2 = 0;

            let key_p = s.add(start_col as usize) as *mut c_char;
            let key = if sort_nr {
                let skip_s = if sort_what & STR2NR_HEX != 0 {
                    skiptohex(key_p)
                } else if sort_what & STR2NR_BIN != 0 {
                    skiptobin(key_p)
                } else {
                    skiptodigit(key_p)
                };

                // Include preceding negative sign
                let skip_s = if skip_s > key_p && *skip_s.sub(1) == b'-' as c_char {
                    skip_s.sub(1) as *mut c_char
                } else {
                    skip_s
                };

                if *skip_s == 0 {
                    // line without number should sort before any number
                    InternalSortKey::Integer {
                        value: 0,
                        is_number: false,
                    }
                } else {
                    let mut val: i64 = 0;
                    nvim_excmds_str2nr(skip_s, sort_what, &mut val);
                    InternalSortKey::Integer {
                        value: val,
                        is_number: true,
                    }
                }
            } else {
                // Float sort
                let ws = crate::skipwhite(key_p);
                let ws = if *ws == b'+' as c_char {
                    crate::skipwhite(ws.add(1))
                } else {
                    ws
                };

                if *ws == 0 {
                    InternalSortKey::Float { value: f64::MIN }
                } else {
                    InternalSortKey::Float {
                        value: libc::strtod(ws, std::ptr::null_mut()),
                    }
                }
            };

            *s2 = saved_c;
            key
        } else {
            InternalSortKey::Text { start_col, end_col }
        };

        nrs.push(InternalSortEntry { lnum, key });

        if !regmatch.is_null() {
            fast_breakcheck();
        }
        if crate::got_int {
            nvim_excmds_regfree(regmatch);
            nvim_excmds_emsg_interr();
            return;
        }
    }

    // Allocate sort buffers for text comparison.
    let buf_size = (maxlen as usize) + 1;
    let mut sortbuf1: Vec<u8> = vec![0u8; buf_size];
    let mut sortbuf2: Vec<u8> = vec![0u8; buf_size];

    // Sort the array. Use stable sort for deterministic tiebreaker.
    let mut sort_abort = false;
    nrs.sort_by(|a, b| {
        if sort_abort {
            return std::cmp::Ordering::Equal;
        }
        fast_breakcheck();
        if crate::got_int {
            sort_abort = true;
            return std::cmp::Ordering::Equal;
        }

        let result = match (&a.key, &b.key) {
            (
                InternalSortKey::Integer {
                    value: v1,
                    is_number: n1,
                },
                InternalSortKey::Integer {
                    value: v2,
                    is_number: n2,
                },
            ) => {
                if n1 != n2 {
                    // Lines with numbers sort after lines without
                    n1.cmp(n2)
                } else {
                    v1.cmp(v2)
                }
            }
            (InternalSortKey::Float { value: v1 }, InternalSortKey::Float { value: v2 }) => {
                v1.partial_cmp(v2).unwrap_or(std::cmp::Ordering::Equal)
            }
            (
                InternalSortKey::Text {
                    start_col: sc1,
                    end_col: ec1,
                },
                InternalSortKey::Text {
                    start_col: sc2,
                    end_col: ec2,
                },
            ) => {
                // Copy line substrings into sortbufs to avoid invalidation
                let s1_ptr = ml_get(a.lnum);
                let len1 = (*ec1 - *sc1) as usize;
                std::ptr::copy_nonoverlapping(
                    s1_ptr.add(*sc1 as usize) as *const u8,
                    sortbuf1.as_mut_ptr(),
                    len1,
                );
                sortbuf1[len1] = 0;

                let s2_ptr = ml_get(b.lnum);
                let len2 = (*ec2 - *sc2) as usize;
                std::ptr::copy_nonoverlapping(
                    s2_ptr.add(*sc2 as usize) as *const u8,
                    sortbuf2.as_mut_ptr(),
                    len2,
                );
                sortbuf2[len2] = 0;

                let cmp = string_compare(
                    sortbuf1.as_ptr() as *const c_char,
                    sortbuf2.as_ptr() as *const c_char,
                    sort_lc,
                    sort_ic,
                );
                cmp.cmp(&0)
            }
            _ => std::cmp::Ordering::Equal,
        };

        if result == std::cmp::Ordering::Equal {
            // Preserve original line order (stable tiebreaker).
            a.lnum.cmp(&b.lnum)
        } else {
            result
        }
    });

    if sort_abort {
        nvim_excmds_regfree(regmatch);
        nvim_excmds_emsg_interr();
        return;
    }

    // Insert the lines in sorted order below the last one.
    let mut old_count: i64 = 0;
    let mut new_count: i64 = 0;
    let mut lnum = line2;
    let forceit = nvim_exarg_get_forceit(eap) != 0;
    let mut i: usize = 0;

    // We need a separate sortbuf for unique comparison (reuse sortbuf1)
    while i < count {
        let idx = if forceit { count - i - 1 } else { i };
        let get_lnum = nrs[idx].lnum;

        // Detect if buffer order actually changed.
        if get_lnum + (count as c_int - 1) != lnum {
            change_occurred = true;
        }

        let s = ml_get(get_lnum);
        let bytelen = ml_get_len(get_lnum) + 1; // include EOL in bytelen
        old_count += i64::from(bytelen);

        // For unique: compare with previous line (stored in sortbuf1)
        let should_insert = if !unique || i == 0 {
            true
        } else {
            string_compare(s, sortbuf1.as_ptr() as *const c_char, sort_lc, sort_ic) != 0
        };

        if should_insert {
            // Copy the line into sortbuf1 (needed for unique check and because
            // ml_append can invalidate the pointer).
            let slen = libc::strlen(s);
            std::ptr::copy_nonoverlapping(s as *const u8, sortbuf1.as_mut_ptr(), slen + 1);
            if ml_append(lnum, sortbuf1.as_ptr() as *const c_char, 0, 0) == FAIL {
                break;
            }
            lnum += 1;
            new_count += i64::from(bytelen);
        }

        fast_breakcheck();
        if crate::got_int {
            nvim_excmds_regfree(regmatch);
            nvim_excmds_emsg_interr();
            return;
        }
        i += 1;
    }

    // Delete the original lines if appending worked.
    if i == count {
        for _ in 0..count {
            ml_delete(line1);
        }
    } else {
        // count = 0 equivalent: skip adjustment
        nvim_excmds_regfree(regmatch);
        return;
    }

    // Adjust marks for deleted (or added) lines.
    let deleted = count as c_int - (lnum - line2);
    if deleted > 0 {
        nvim_excmds_mark_adjust(line2 - deleted, line2, MAXLNUM, -deleted, KEXTMARK_NOOP);
        msgmore(-deleted);
    } else if deleted < 0 {
        nvim_excmds_mark_adjust(line2, MAXLNUM, -deleted, 0, KEXTMARK_NOOP);
    }

    if change_occurred || deleted != 0 {
        nvim_excmds_extmark_splice(
            line1 - 1,
            0,
            count as c_int,
            0,
            old_count,
            lnum - line2,
            0,
            new_count,
            KEXTMARK_UNDO,
        );
        changed_lines(nvim_get_curbuf(), line1, 0, line2 + 1, -deleted, 1);
    }

    nvim_curwin_set_cursor_lnum(line1);
    beginline(BL_WHITE_FIX);

    // Cleanup
    nvim_excmds_regfree(regmatch);
    if crate::got_int {
        nvim_excmds_emsg_interr();
    }
}

/// `:uniq` command implementation.
///
/// Removes duplicate adjacent lines, with optional regex-based comparison
/// and three modes: default (remove adjacent duplicates), keep-only-unique (u),
/// and keep-only-not-unique (!).
///
/// # Safety
/// `eap` must be a valid pointer to an exarg_T.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_uniq(eap: *mut crate::ExArgHandle) {
    use crate::{
        beginline, changed_lines, check_nextcmd, fast_breakcheck, last_search_pat, ml_delete,
        ml_get, ml_get_len, msgmore, nvim_curwin_set_cursor_lnum, nvim_exarg_get_arg,
        nvim_exarg_get_forceit, nvim_exarg_get_line1, nvim_exarg_get_line2,
        nvim_exarg_is_nextcmd_null, nvim_exarg_set_nextcmd, nvim_excmds_emsg_interr,
        nvim_excmds_emsg_noprevre, nvim_excmds_mark_adjust, nvim_excmds_regcomp,
        nvim_excmds_regfree, nvim_excmds_regmatch_endp0, nvim_excmds_regmatch_set_ic,
        nvim_excmds_regmatch_startp0, nvim_excmds_semsg_invarg2, nvim_get_curbuf, skip_regexp_err,
        u_save, vim_regexec,
    };

    let line1 = nvim_exarg_get_line1(eap);
    let line2 = nvim_exarg_get_line2(eap);
    let mut count: c_int = line2 - line1 + 1;
    let keep_only_not_unique = nvim_exarg_get_forceit(eap) != 0;
    let mut keep_only_unique = false;
    let mut deleted: c_int = 0;

    // Uniq one line is really quick!
    if count <= 1 {
        return;
    }

    if u_save(line1 - 1, line2 + 1) == FAIL {
        return;
    }

    let mut regmatch: *mut crate::RegmatchHandle = std::ptr::null_mut();
    let mut sort_ic = false;
    let mut sort_lc = false;
    let mut sort_rx = false;
    let mut change_occurred = false;

    // Parse flags from eap->arg
    let mut p = nvim_exarg_get_arg(eap) as *mut c_char;
    loop {
        let c = *p;
        if c == 0 {
            break;
        }
        if c == b' ' as c_char || c == b'\t' as c_char {
            // skip whitespace
        } else if c == b'i' as c_char {
            sort_ic = true;
        } else if c == b'l' as c_char {
            sort_lc = true;
        } else if c == b'r' as c_char {
            sort_rx = true;
        } else if c == b'u' as c_char {
            // 'u' is only valid when '!' is not given.
            if !keep_only_not_unique {
                keep_only_unique = true;
            }
        } else if c == b'"' as c_char {
            // comment start
            break;
        } else if nvim_exarg_is_nextcmd_null(eap) != 0 && !check_nextcmd(p).is_null() {
            nvim_exarg_set_nextcmd(eap, check_nextcmd(p));
            break;
        } else if !((c as u8).is_ascii_alphabetic()) && regmatch.is_null() {
            let s = skip_regexp_err(p.add(1), c as c_int, 1);
            if s.is_null() {
                return;
            }
            *s = 0;

            if std::ptr::eq(s, p.add(1)) {
                let last_pat = last_search_pat();
                if last_pat.is_null() {
                    nvim_excmds_emsg_noprevre();
                    return;
                }
                regmatch = nvim_excmds_regcomp(last_pat, RE_MAGIC);
            } else {
                regmatch = nvim_excmds_regcomp(p.add(1), RE_MAGIC);
            }
            if regmatch.is_null() {
                return;
            }
            p = s;
            nvim_excmds_regmatch_set_ic(regmatch, crate::p_ic);
        } else {
            nvim_excmds_semsg_invarg2(p);
            nvim_excmds_regfree(regmatch);
            return;
        }
        p = p.add(1);
    }

    // Find the length of the longest line.
    let mut maxlen: c_int = 0;
    for lnum in line1..=line2 {
        let len = ml_get_len(lnum);
        if len > maxlen {
            maxlen = len;
        }
        if crate::got_int {
            nvim_excmds_regfree(regmatch);
            nvim_excmds_emsg_interr();
            return;
        }
    }

    // Allocate a buffer that can hold the longest line.
    let buf_size = (maxlen as usize) + 1;
    let mut sortbuf1: Vec<u8> = vec![0u8; buf_size];

    // Delete lines according to options.
    let mut match_continue = false;
    let mut next_is_unmatch = false;
    let mut done_lnum: c_int = line1 - 1;
    let mut i: c_int = 0;

    while i < count {
        let get_lnum = line1 + i;

        let s = ml_get(get_lnum) as *mut c_char;
        let len = ml_get_len(get_lnum);

        let mut start_col: c_int = 0;
        let mut end_col: c_int = len;
        if !regmatch.is_null() && vim_regexec(regmatch, s, 0) {
            if sort_rx {
                start_col = nvim_excmds_regmatch_startp0(regmatch).offset_from(s) as c_int;
                end_col = nvim_excmds_regmatch_endp0(regmatch).offset_from(s) as c_int;
            } else {
                start_col = nvim_excmds_regmatch_endp0(regmatch).offset_from(s) as c_int;
            }
        } else if !regmatch.is_null() {
            end_col = 0;
        }

        let save_c: c_char = if end_col > 0 {
            let c = *s.add(end_col as usize);
            *s.add(end_col as usize) = 0;
            c
        } else {
            0
        };

        let mut is_match = if i > 0 {
            string_compare(
                s.add(start_col as usize),
                sortbuf1.as_ptr() as *const c_char,
                sort_lc,
                sort_ic,
            ) == 0
        } else {
            false
        };

        let mut delete_lnum: c_int = 0;
        if next_is_unmatch {
            is_match = false;
            next_is_unmatch = false;
        }

        if !keep_only_unique && !keep_only_not_unique {
            // Default mode: remove adjacent duplicates
            if is_match {
                delete_lnum = get_lnum;
            } else {
                let slen = libc::strlen(s.add(start_col as usize));
                std::ptr::copy_nonoverlapping(
                    s.add(start_col as usize) as *const u8,
                    sortbuf1.as_mut_ptr(),
                    slen + 1,
                );
            }
        } else if keep_only_not_unique {
            // Keep only non-unique lines (! mode)
            if is_match {
                done_lnum = get_lnum - 1;
                delete_lnum = get_lnum;
                match_continue = true;
            } else {
                if i > 0 && !match_continue && get_lnum - 1 > done_lnum {
                    delete_lnum = get_lnum - 1;
                    next_is_unmatch = true;
                } else if i >= count - 1 {
                    delete_lnum = get_lnum;
                }
                match_continue = false;
                let slen = libc::strlen(s.add(start_col as usize));
                std::ptr::copy_nonoverlapping(
                    s.add(start_col as usize) as *const u8,
                    sortbuf1.as_mut_ptr(),
                    slen + 1,
                );
            }
        } else {
            // keep_only_unique mode (u flag)
            if is_match {
                if !match_continue {
                    delete_lnum = get_lnum - 1;
                } else {
                    delete_lnum = get_lnum;
                }
                match_continue = true;
            } else {
                if i == 0 && match_continue {
                    delete_lnum = get_lnum;
                }
                match_continue = false;
                let slen = libc::strlen(s.add(start_col as usize));
                std::ptr::copy_nonoverlapping(
                    s.add(start_col as usize) as *const u8,
                    sortbuf1.as_mut_ptr(),
                    slen + 1,
                );
            }
        }

        if end_col > 0 {
            *s.add(end_col as usize) = save_c;
        }

        if delete_lnum > 0 {
            ml_delete(delete_lnum);
            i -= get_lnum - delete_lnum + 1;
            count -= 1;
            deleted += 1;
            change_occurred = true;
        }

        fast_breakcheck();
        if crate::got_int {
            nvim_excmds_regfree(regmatch);
            nvim_excmds_emsg_interr();
            return;
        }
        i += 1;
    }

    // Adjust marks for deleted lines and prepare for displaying.
    let etype = if change_occurred {
        KEXTMARK_UNDO
    } else {
        KEXTMARK_NOOP
    };
    nvim_excmds_mark_adjust(line2 - deleted, line2, MAXLNUM, -deleted, etype);
    msgmore(-deleted);

    if change_occurred {
        changed_lines(nvim_get_curbuf(), line1, 0, line2 + 1, -deleted, 1);
    }

    nvim_curwin_set_cursor_lnum(line1);
    beginline(BL_WHITE_FIX);

    // Cleanup
    nvim_excmds_regfree(regmatch);
    if crate::got_int {
        nvim_excmds_emsg_interr();
    }
}

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
