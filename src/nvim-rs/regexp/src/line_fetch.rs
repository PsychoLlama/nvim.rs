//! Buffer line fetching for multi-line regex matching.
//!
//! This module provides infrastructure for fetching lines from Neovim buffers
//! during multi-line regex execution. It includes:
//!
//! - Line fetching callbacks for buffer access
//! - Line caching for performance
//! - Multi-line boundary handling
//!
//! # Key Concepts
//!
//! When matching patterns across multiple lines in a buffer, we need to:
//! 1. Fetch line content from the buffer on demand
//! 2. Handle line boundaries (when pattern spans lines)
//! 3. Track position as (line number, column) pairs
//!
//! The C code uses `reg_getline()` to fetch lines from `rex.reg_buf`.
//! We provide Rust equivalents that integrate with the execution state.

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::exec_state::{ColNr, LineNr};
use crate::BufHandle;

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    /// Get line content from buffer (ml_get_buf).
    fn ml_get_buf(buf: BufHandle, lnum: LineNr) -> *mut c_char;

    /// Get line length from buffer (ml_get_buf_len).
    fn ml_get_buf_len(buf: BufHandle, lnum: LineNr) -> ColNr;

    // Rex buffer accessor
    fn nvim_rex_get_reg_buf() -> BufHandle;
    fn nvim_rex_get_reg_firstlnum() -> LineNr;
    fn nvim_rex_get_reg_maxline() -> LineNr;
}

// =============================================================================
// Line Fetch Flags
// =============================================================================

/// Flags for line fetching operations (matches C reg_getline_flags_T).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineFlags(pub c_int);

impl LineFlags {
    /// Fetch line content.
    pub const LINE: Self = Self(0x01);
    /// Fetch line length.
    pub const LENGTH: Self = Self(0x02);
    /// Use submatch context (rsm instead of rex).
    pub const SUBMATCH: Self = Self(0x04);

    /// Check if a flag is set.
    #[inline]
    pub fn has(self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }

    /// Combine flags.
    #[inline]
    pub fn with(self, flag: Self) -> Self {
        Self(self.0 | flag.0)
    }
}

impl std::ops::BitOr for LineFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

// =============================================================================
// Line Cache
// =============================================================================

/// Cache for recently fetched lines.
///
/// Caches both line content and length to avoid repeated FFI calls
/// when the same line is accessed multiple times during matching.
#[derive(Debug)]
pub struct LineCache {
    /// Cached line number (absolute).
    lnum: LineNr,
    /// Cached line content pointer.
    line: *const u8,
    /// Cached line length.
    length: ColNr,
    /// Whether cache is valid.
    valid: bool,
}

impl Default for LineCache {
    fn default() -> Self {
        Self::new()
    }
}

impl LineCache {
    /// Create a new empty cache.
    pub const fn new() -> Self {
        Self {
            lnum: 0,
            line: ptr::null(),
            length: 0,
            valid: false,
        }
    }

    /// Invalidate the cache.
    #[inline]
    pub fn invalidate(&mut self) {
        self.valid = false;
        self.line = ptr::null();
    }

    /// Check if cache is valid for a specific line.
    #[inline]
    pub fn is_valid_for(&self, lnum: LineNr) -> bool {
        self.valid && self.lnum == lnum
    }

    /// Update the cache.
    #[inline]
    pub fn update(&mut self, lnum: LineNr, line: *const u8, length: ColNr) {
        self.lnum = lnum;
        self.line = line;
        self.length = length;
        self.valid = true;
    }

    /// Get cached line if valid.
    #[inline]
    pub fn get_line(&self, lnum: LineNr) -> Option<*const u8> {
        if self.is_valid_for(lnum) {
            Some(self.line)
        } else {
            None
        }
    }

    /// Get cached length if valid.
    #[inline]
    pub fn get_length(&self, lnum: LineNr) -> Option<ColNr> {
        if self.is_valid_for(lnum) {
            Some(self.length)
        } else {
            None
        }
    }
}

// =============================================================================
// Line Fetcher
// =============================================================================

/// Line fetcher for multi-line regex matching.
///
/// This structure manages fetching lines from a buffer during regex execution.
/// It includes caching to improve performance when lines are accessed repeatedly.
pub struct LineFetcher {
    /// Buffer to fetch lines from.
    buf: BufHandle,
    /// First line number (1-based, absolute).
    first_lnum: LineNr,
    /// Maximum relative line number (0 = last line of buffer).
    max_line: LineNr,
    /// Line cache.
    cache: LineCache,
}

impl LineFetcher {
    /// Create a new line fetcher.
    pub fn new(buf: BufHandle, first_lnum: LineNr, max_line: LineNr) -> Self {
        Self {
            buf,
            first_lnum,
            max_line,
            cache: LineCache::new(),
        }
    }

    /// Create a line fetcher from current rex state.
    ///
    /// # Safety
    /// Rex must be properly initialized.
    pub unsafe fn from_rex() -> Self {
        Self::new(
            nvim_rex_get_reg_buf(),
            nvim_rex_get_reg_firstlnum(),
            nvim_rex_get_reg_maxline(),
        )
    }

    /// Get a line by relative line number.
    ///
    /// The line number is relative to `first_lnum`.
    /// Returns NULL if line is out of range.
    ///
    /// # Safety
    /// Buffer must be valid.
    pub unsafe fn get_line(&mut self, rel_lnum: LineNr) -> *const u8 {
        let abs_lnum = self.first_lnum + rel_lnum;

        // Check bounds
        if abs_lnum < 1 {
            return ptr::null();
        }
        if rel_lnum > self.max_line {
            // Past end - return empty string for "\n" match at end
            return c"".as_ptr().cast();
        }

        // Check cache
        if let Some(line) = self.cache.get_line(abs_lnum) {
            return line;
        }

        // Fetch from buffer
        let line = ml_get_buf(self.buf, abs_lnum) as *const u8;
        let length = ml_get_buf_len(self.buf, abs_lnum);
        self.cache.update(abs_lnum, line, length);

        line
    }

    /// Get line length by relative line number.
    ///
    /// # Safety
    /// Buffer must be valid.
    pub unsafe fn get_line_len(&mut self, rel_lnum: LineNr) -> ColNr {
        let abs_lnum = self.first_lnum + rel_lnum;

        // Check bounds
        if abs_lnum < 1 || rel_lnum > self.max_line {
            return 0;
        }

        // Check cache
        if let Some(len) = self.cache.get_length(abs_lnum) {
            return len;
        }

        // Fetch from buffer
        let line = ml_get_buf(self.buf, abs_lnum) as *const u8;
        let length = ml_get_buf_len(self.buf, abs_lnum);
        self.cache.update(abs_lnum, line, length);

        length
    }

    /// Invalidate the cache (e.g., when buffer changes).
    pub fn invalidate_cache(&mut self) {
        self.cache.invalidate();
    }

    /// Get the first line number (absolute).
    #[inline]
    pub fn first_lnum(&self) -> LineNr {
        self.first_lnum
    }

    /// Get the maximum relative line number.
    #[inline]
    pub fn max_line(&self) -> LineNr {
        self.max_line
    }
}

// =============================================================================
// Global Line Fetch Functions
// =============================================================================

/// Get line content from rex buffer.
///
/// This is the Rust equivalent of C's `reg_getline()`.
/// Line number is relative to `rex.reg_firstlnum`.
///
/// # Safety
/// Rex must be properly initialized.
pub unsafe fn reg_getline(rel_lnum: LineNr) -> *const u8 {
    let first_lnum = nvim_rex_get_reg_firstlnum();
    let max_line = nvim_rex_get_reg_maxline();
    let abs_lnum = first_lnum + rel_lnum;

    // Handle lookbehind (negative relative line)
    if abs_lnum < 1 {
        return ptr::null();
    }

    // Past last line - return empty for "\n" at end
    if rel_lnum > max_line {
        return c"".as_ptr().cast();
    }

    let buf = nvim_rex_get_reg_buf();
    ml_get_buf(buf, abs_lnum) as *const u8
}

/// Get line length from rex buffer.
///
/// This is the Rust equivalent of C's `reg_getline_len()`.
///
/// # Safety
/// Rex must be properly initialized.
pub unsafe fn reg_getline_len(rel_lnum: LineNr) -> ColNr {
    let first_lnum = nvim_rex_get_reg_firstlnum();
    let max_line = nvim_rex_get_reg_maxline();
    let abs_lnum = first_lnum + rel_lnum;

    // Handle bounds
    if abs_lnum < 1 || rel_lnum > max_line {
        return 0;
    }

    let buf = nvim_rex_get_reg_buf();
    ml_get_buf_len(buf, abs_lnum)
}

// =============================================================================
// Multi-line Position Helpers
// =============================================================================

/// Position in a multi-line context.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LinePos {
    /// Line number (relative to first_lnum).
    pub lnum: LineNr,
    /// Column (byte offset).
    pub col: ColNr,
}

impl LinePos {
    /// Create a new line position.
    pub const fn new(lnum: LineNr, col: ColNr) -> Self {
        Self { lnum, col }
    }

    /// Check if position is valid (non-negative line).
    pub fn is_valid(&self) -> bool {
        self.lnum >= 0
    }

    /// Create an invalid position.
    pub const fn invalid() -> Self {
        Self { lnum: -1, col: 0 }
    }
}

/// Compare two line positions.
///
/// Returns:
/// - Negative if a < b
/// - Zero if a == b
/// - Positive if a > b
pub fn compare_pos(a: &LinePos, b: &LinePos) -> c_int {
    if a.lnum != b.lnum {
        a.lnum - b.lnum
    } else {
        a.col - b.col
    }
}

/// Check if position a is before position b.
#[inline]
pub fn pos_before(a: &LinePos, b: &LinePos) -> bool {
    compare_pos(a, b) < 0
}

/// Check if position a is after position b.
#[inline]
pub fn pos_after(a: &LinePos, b: &LinePos) -> bool {
    compare_pos(a, b) > 0
}

// =============================================================================
// Multi-line Range
// =============================================================================

/// A range spanning multiple lines.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LineRange {
    /// Start position.
    pub start: LinePos,
    /// End position.
    pub end: LinePos,
}

impl LineRange {
    /// Create a new line range.
    pub const fn new(start: LinePos, end: LinePos) -> Self {
        Self { start, end }
    }

    /// Check if range is valid.
    pub fn is_valid(&self) -> bool {
        self.start.is_valid() && self.end.is_valid() && !pos_after(&self.start, &self.end)
    }

    /// Check if range is empty.
    pub fn is_empty(&self) -> bool {
        self.start.lnum == self.end.lnum && self.start.col == self.end.col
    }

    /// Check if range spans multiple lines.
    pub fn is_multiline(&self) -> bool {
        self.start.lnum != self.end.lnum
    }

    /// Check if a position is within this range.
    pub fn contains(&self, pos: &LinePos) -> bool {
        !pos_before(pos, &self.start) && pos_before(pos, &self.end)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create a new line fetcher.
///
/// # Safety
/// `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_line_fetcher_new(
    buf: BufHandle,
    first_lnum: LineNr,
    max_line: LineNr,
) -> *mut LineFetcher {
    Box::into_raw(Box::new(LineFetcher::new(buf, first_lnum, max_line)))
}

/// Create a line fetcher from current rex state.
///
/// # Safety
/// Rex must be properly initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_line_fetcher_from_rex() -> *mut LineFetcher {
    Box::into_raw(Box::new(LineFetcher::from_rex()))
}

/// Free a line fetcher.
///
/// # Safety
/// `fetcher` must be a valid pointer from `rs_line_fetcher_new*`.
#[no_mangle]
pub unsafe extern "C" fn rs_line_fetcher_free(fetcher: *mut LineFetcher) {
    if !fetcher.is_null() {
        drop(Box::from_raw(fetcher));
    }
}

/// Get line content.
///
/// # Safety
/// `fetcher` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_line_fetcher_get_line(
    fetcher: *mut LineFetcher,
    rel_lnum: LineNr,
) -> *const u8 {
    if fetcher.is_null() {
        return ptr::null();
    }
    (*fetcher).get_line(rel_lnum)
}

/// Get line length.
///
/// # Safety
/// `fetcher` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_line_fetcher_get_len(
    fetcher: *mut LineFetcher,
    rel_lnum: LineNr,
) -> ColNr {
    if fetcher.is_null() {
        return 0;
    }
    (*fetcher).get_line_len(rel_lnum)
}

/// Invalidate the fetcher's cache.
///
/// # Safety
/// `fetcher` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_line_fetcher_invalidate(fetcher: *mut LineFetcher) {
    if !fetcher.is_null() {
        (*fetcher).invalidate_cache();
    }
}

/// Get line from rex buffer (global function).
///
/// # Safety
/// Rex must be properly initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_getline(rel_lnum: LineNr) -> *const u8 {
    reg_getline(rel_lnum)
}

/// Get line length from rex buffer (global function).
///
/// # Safety
/// Rex must be properly initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_getline_len(rel_lnum: LineNr) -> ColNr {
    reg_getline_len(rel_lnum)
}

/// Create a line cache.
#[no_mangle]
pub extern "C" fn rs_line_cache_new() -> *mut LineCache {
    Box::into_raw(Box::new(LineCache::new()))
}

/// Free a line cache.
///
/// # Safety
/// `cache` must be a valid pointer from `rs_line_cache_new`.
#[no_mangle]
pub unsafe extern "C" fn rs_line_cache_free(cache: *mut LineCache) {
    if !cache.is_null() {
        drop(Box::from_raw(cache));
    }
}

/// Invalidate a line cache.
///
/// # Safety
/// `cache` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_line_cache_invalidate(cache: *mut LineCache) {
    if !cache.is_null() {
        (*cache).invalidate();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_flags() {
        let flags = LineFlags::LINE | LineFlags::LENGTH;
        assert!(flags.has(LineFlags::LINE));
        assert!(flags.has(LineFlags::LENGTH));
        assert!(!flags.has(LineFlags::SUBMATCH));

        let with_submatch = flags.with(LineFlags::SUBMATCH);
        assert!(with_submatch.has(LineFlags::SUBMATCH));
    }

    #[test]
    fn test_line_cache() {
        let mut cache = LineCache::new();

        assert!(!cache.valid);
        assert!(!cache.is_valid_for(5));
        assert!(cache.get_line(5).is_none());

        let line_data = [b'a', b'b', b'c', 0];
        cache.update(5, line_data.as_ptr(), 3);

        assert!(cache.valid);
        assert!(cache.is_valid_for(5));
        assert!(!cache.is_valid_for(6));
        assert_eq!(cache.get_line(5), Some(line_data.as_ptr()));
        assert_eq!(cache.get_length(5), Some(3));

        cache.invalidate();
        assert!(!cache.valid);
        assert!(cache.get_line(5).is_none());
    }

    #[test]
    fn test_line_pos() {
        let pos = LinePos::new(3, 10);
        assert_eq!(pos.lnum, 3);
        assert_eq!(pos.col, 10);
        assert!(pos.is_valid());

        let invalid = LinePos::invalid();
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_compare_pos() {
        let a = LinePos::new(1, 5);
        let b = LinePos::new(1, 10);
        let c = LinePos::new(2, 0);

        assert!(pos_before(&a, &b));
        assert!(pos_before(&b, &c));
        assert!(pos_before(&a, &c));

        assert!(pos_after(&b, &a));
        assert!(pos_after(&c, &b));
        assert!(pos_after(&c, &a));

        assert_eq!(compare_pos(&a, &a), 0);
    }

    #[test]
    fn test_line_range() {
        let start = LinePos::new(1, 5);
        let end = LinePos::new(2, 10);
        let range = LineRange::new(start, end);

        assert!(range.is_valid());
        assert!(range.is_multiline());
        assert!(!range.is_empty());

        let mid = LinePos::new(1, 8);
        assert!(range.contains(&mid));

        let before = LinePos::new(1, 0);
        assert!(!range.contains(&before));
    }

    #[test]
    fn test_line_range_empty() {
        let pos = LinePos::new(1, 5);
        let range = LineRange::new(pos, pos);

        assert!(range.is_empty());
        assert!(!range.is_multiline());
    }

    #[test]
    fn test_line_range_single_line() {
        let start = LinePos::new(1, 0);
        let end = LinePos::new(1, 10);
        let range = LineRange::new(start, end);

        assert!(!range.is_multiline());
        assert!(!range.is_empty());
    }
}
