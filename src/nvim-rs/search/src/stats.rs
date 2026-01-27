//! Search statistics types
//!
//! This module provides types and functions for managing search match
//! statistics, like the [N/M] display shown after searching.

use std::ffi::c_int;

// =============================================================================
// Search Statistics Types
// =============================================================================

/// Search statistics for displaying match counts.
///
/// This corresponds to `searchstat_T` in C.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SearchStat {
    /// Current match number (1-indexed)
    pub cur: c_int,
    /// Total count of matches
    pub cnt: c_int,
    /// Whether current match is exact (cursor on match)
    pub exact_match: bool,
    /// Incomplete status: 0=complete, 1=timed out, 2=max count exceeded
    pub incomplete: c_int,
    /// Max count used for last search
    pub last_maxcount: c_int,
}

impl SearchStat {
    /// Create a new empty search stat.
    pub const fn new() -> Self {
        Self {
            cur: 0,
            cnt: 0,
            exact_match: false,
            incomplete: 0,
            last_maxcount: 0,
        }
    }

    /// Check if statistics are valid (have been computed).
    pub const fn is_valid(&self) -> bool {
        self.cnt > 0 || self.incomplete != 0
    }

    /// Check if the search completed fully.
    pub const fn is_complete(&self) -> bool {
        self.incomplete == 0
    }

    /// Check if the search timed out.
    pub const fn timed_out(&self) -> bool {
        self.incomplete == 1
    }

    /// Check if the search exceeded max count.
    pub const fn exceeded_max(&self) -> bool {
        self.incomplete == 2
    }

    /// Check if we're on the first match.
    pub const fn is_first(&self) -> bool {
        self.cur == 1
    }

    /// Check if we're on the last match.
    pub const fn is_last(&self) -> bool {
        self.cur == self.cnt && self.is_complete()
    }

    /// Check if cursor is exactly on a match.
    pub const fn is_exact(&self) -> bool {
        self.exact_match
    }

    /// Get the display string components.
    ///
    /// Returns (current, total, show_total) where:
    /// - current: the current match number
    /// - total: the total count (may be approximate)
    /// - show_total: whether to show total (false if incomplete)
    pub fn display_info(&self) -> (c_int, c_int, bool) {
        if self.incomplete == 0 {
            (self.cur, self.cnt, true)
        } else if self.incomplete == 2 {
            // Max count exceeded - show ">N"
            (self.cur, self.cnt, true)
        } else {
            // Timed out - don't show total
            (self.cur, 0, false)
        }
    }
}

// =============================================================================
// Search Statistics State
// =============================================================================

/// State for incremental search statistics tracking.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SearchStatState {
    /// Current statistics
    pub stat: SearchStat,
    /// Last position where stats were computed
    pub last_lnum: i32,
    /// Last column where stats were computed
    pub last_col: i32,
    /// Buffer changedtick when stats were computed
    pub changedtick: c_int,
    /// Whether recomputation is needed
    pub needs_recompute: bool,
}

impl Default for SearchStatState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchStatState {
    /// Create a new search statistics state.
    pub const fn new() -> Self {
        Self {
            stat: SearchStat::new(),
            last_lnum: 0,
            last_col: 0,
            changedtick: 0,
            needs_recompute: true,
        }
    }

    /// Mark that recomputation is needed.
    pub fn invalidate(&mut self) {
        self.needs_recompute = true;
    }

    /// Update the position where stats were computed.
    pub fn set_position(&mut self, lnum: i32, col: i32) {
        self.last_lnum = lnum;
        self.last_col = col;
    }

    /// Update the changedtick.
    pub fn set_changedtick(&mut self, tick: c_int) {
        self.changedtick = tick;
    }

    /// Check if stats need recomputation based on position change.
    pub fn position_changed(&self, lnum: i32, col: i32) -> bool {
        self.last_lnum != lnum || self.last_col != col
    }

    /// Check if stats need recomputation based on buffer change.
    pub fn buffer_changed(&self, tick: c_int) -> bool {
        self.changedtick != tick
    }

    /// Mark stats as computed.
    pub fn mark_computed(&mut self) {
        self.needs_recompute = false;
    }
}

// =============================================================================
// Search Statistics Formatting
// =============================================================================

/// Formatting options for search statistics display.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SearchStatFormat {
    /// Show "TOP" when at first match after wrap
    pub show_top: bool,
    /// Show "BOT" when at last match after wrap
    pub show_bot: bool,
    /// Max count for display (0 = no limit)
    pub max_count: c_int,
    /// Timeout in milliseconds (0 = no timeout)
    pub timeout_ms: c_int,
}

impl Default for SearchStatFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchStatFormat {
    /// Create default format options.
    pub const fn new() -> Self {
        Self {
            show_top: true,
            show_bot: true,
            max_count: 99,
            timeout_ms: 20,
        }
    }

    /// Create format with specified max count.
    pub const fn with_max_count(max_count: c_int) -> Self {
        Self {
            max_count,
            ..Self::new()
        }
    }
}

/// Result of formatting search statistics.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FormattedSearchStat {
    /// Whether to show any statistics
    pub show: bool,
    /// Whether to show "TOP" indicator
    pub at_top: bool,
    /// Whether to show "BOT" indicator
    pub at_bot: bool,
    /// Current match number
    pub current: c_int,
    /// Total matches (0 if not showing)
    pub total: c_int,
    /// Whether total is approximate (exceeded max)
    pub approximate: bool,
}

impl Default for FormattedSearchStat {
    fn default() -> Self {
        Self::hidden()
    }
}

impl FormattedSearchStat {
    /// Create a hidden (not shown) result.
    pub const fn hidden() -> Self {
        Self {
            show: false,
            at_top: false,
            at_bot: false,
            current: 0,
            total: 0,
            approximate: false,
        }
    }

    /// Create a result showing current/total.
    pub const fn showing(current: c_int, total: c_int) -> Self {
        Self {
            show: true,
            at_top: false,
            at_bot: false,
            current,
            total,
            approximate: false,
        }
    }

    /// Mark as at top.
    pub fn mark_top(mut self) -> Self {
        self.at_top = true;
        self
    }

    /// Mark as at bottom.
    pub fn mark_bot(mut self) -> Self {
        self.at_bot = true;
        self
    }

    /// Mark total as approximate.
    pub fn mark_approximate(mut self) -> Self {
        self.approximate = true;
        self
    }
}

/// Format search statistics for display.
pub fn format_search_stat(stat: &SearchStat, format: &SearchStatFormat) -> FormattedSearchStat {
    if !stat.is_valid() {
        return FormattedSearchStat::hidden();
    }

    let mut result = FormattedSearchStat::showing(stat.cur, stat.cnt);

    // Check for TOP/BOT indicators
    if format.show_top && stat.is_first() {
        result = result.mark_top();
    }
    if format.show_bot && stat.is_last() {
        result = result.mark_bot();
    }

    // Check for approximate count
    if stat.exceeded_max() {
        result = result.mark_approximate();
    }

    // Hide total if timed out
    if stat.timed_out() {
        result.total = 0;
    }

    result
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Create new SearchStat.
#[no_mangle]
pub extern "C" fn rs_search_stat_new() -> SearchStat {
    SearchStat::new()
}

/// FFI: Check if SearchStat is valid.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_is_valid(stat: *const SearchStat) -> c_int {
    if stat.is_null() {
        return 0;
    }
    c_int::from((*stat).is_valid())
}

/// FFI: Check if search completed.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_is_complete(stat: *const SearchStat) -> c_int {
    if stat.is_null() {
        return 0;
    }
    c_int::from((*stat).is_complete())
}

/// FFI: Check if search timed out.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_timed_out(stat: *const SearchStat) -> c_int {
    if stat.is_null() {
        return 0;
    }
    c_int::from((*stat).timed_out())
}

/// FFI: Check if search exceeded max count.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_exceeded_max(stat: *const SearchStat) -> c_int {
    if stat.is_null() {
        return 0;
    }
    c_int::from((*stat).exceeded_max())
}

/// FFI: Check if at first match.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_is_first(stat: *const SearchStat) -> c_int {
    if stat.is_null() {
        return 0;
    }
    c_int::from((*stat).is_first())
}

/// FFI: Check if at last match.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_is_last(stat: *const SearchStat) -> c_int {
    if stat.is_null() {
        return 0;
    }
    c_int::from((*stat).is_last())
}

/// FFI: Check if cursor is exactly on match.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_is_exact(stat: *const SearchStat) -> c_int {
    if stat.is_null() {
        return 0;
    }
    c_int::from((*stat).is_exact())
}

/// FFI: Get current match number.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_get_cur(stat: *const SearchStat) -> c_int {
    if stat.is_null() {
        return 0;
    }
    (*stat).cur
}

/// FFI: Get total match count.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_get_cnt(stat: *const SearchStat) -> c_int {
    if stat.is_null() {
        return 0;
    }
    (*stat).cnt
}

/// FFI: Set current match number.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_set_cur(stat: *mut SearchStat, cur: c_int) {
    if !stat.is_null() {
        (*stat).cur = cur;
    }
}

/// FFI: Set total match count.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_set_cnt(stat: *mut SearchStat, cnt: c_int) {
    if !stat.is_null() {
        (*stat).cnt = cnt;
    }
}

/// FFI: Set exact match flag.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_set_exact(stat: *mut SearchStat, exact: c_int) {
    if !stat.is_null() {
        (*stat).exact_match = exact != 0;
    }
}

/// FFI: Set incomplete status.
///
/// # Safety
/// The caller must ensure `stat` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_set_incomplete(stat: *mut SearchStat, incomplete: c_int) {
    if !stat.is_null() {
        (*stat).incomplete = incomplete;
    }
}

/// FFI: Create default SearchStatFormat.
#[no_mangle]
pub extern "C" fn rs_search_stat_format_new() -> SearchStatFormat {
    SearchStatFormat::new()
}

/// FFI: Create SearchStatFormat with max count.
#[no_mangle]
pub extern "C" fn rs_search_stat_format_with_max(max_count: c_int) -> SearchStatFormat {
    SearchStatFormat::with_max_count(max_count)
}

/// FFI: Create new SearchStatState.
#[no_mangle]
pub extern "C" fn rs_search_stat_state_new() -> SearchStatState {
    SearchStatState::new()
}

/// FFI: Invalidate SearchStatState.
///
/// # Safety
/// The caller must ensure `state` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_state_invalidate(state: *mut SearchStatState) {
    if !state.is_null() {
        (*state).invalidate();
    }
}

/// FFI: Check if SearchStatState needs recompute.
///
/// # Safety
/// The caller must ensure `state` points to valid memory if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_search_stat_state_needs_recompute(
    state: *const SearchStatState,
) -> c_int {
    if state.is_null() {
        return 1;
    }
    c_int::from((*state).needs_recompute)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_stat_new() {
        let stat = SearchStat::new();
        assert_eq!(stat.cur, 0);
        assert_eq!(stat.cnt, 0);
        assert!(!stat.exact_match);
        assert_eq!(stat.incomplete, 0);
    }

    #[test]
    fn test_search_stat_valid() {
        let mut stat = SearchStat::new();
        assert!(!stat.is_valid());

        stat.cnt = 5;
        assert!(stat.is_valid());

        stat.cnt = 0;
        stat.incomplete = 1;
        assert!(stat.is_valid());
    }

    #[test]
    fn test_search_stat_incomplete() {
        let mut stat = SearchStat::new();
        stat.cnt = 10;

        stat.incomplete = 0;
        assert!(stat.is_complete());
        assert!(!stat.timed_out());
        assert!(!stat.exceeded_max());

        stat.incomplete = 1;
        assert!(!stat.is_complete());
        assert!(stat.timed_out());
        assert!(!stat.exceeded_max());

        stat.incomplete = 2;
        assert!(!stat.is_complete());
        assert!(!stat.timed_out());
        assert!(stat.exceeded_max());
    }

    #[test]
    fn test_search_stat_first_last() {
        let mut stat = SearchStat::new();
        stat.cnt = 10;
        stat.cur = 1;

        assert!(stat.is_first());
        assert!(!stat.is_last());

        stat.cur = 10;
        assert!(!stat.is_first());
        assert!(stat.is_last());

        // Not last if incomplete
        stat.incomplete = 1;
        assert!(!stat.is_last());
    }

    #[test]
    fn test_format_search_stat() {
        let mut stat = SearchStat::new();
        stat.cur = 3;
        stat.cnt = 10;

        let format = SearchStatFormat::new();
        let result = format_search_stat(&stat, &format);

        assert!(result.show);
        assert!(!result.at_top);
        assert!(!result.at_bot);
        assert_eq!(result.current, 3);
        assert_eq!(result.total, 10);
        assert!(!result.approximate);
    }

    #[test]
    fn test_format_search_stat_at_first() {
        let mut stat = SearchStat::new();
        stat.cur = 1;
        stat.cnt = 10;

        let format = SearchStatFormat::new();
        let result = format_search_stat(&stat, &format);

        assert!(result.at_top);
        assert!(!result.at_bot);
    }

    #[test]
    fn test_format_search_stat_at_last() {
        let mut stat = SearchStat::new();
        stat.cur = 10;
        stat.cnt = 10;

        let format = SearchStatFormat::new();
        let result = format_search_stat(&stat, &format);

        assert!(!result.at_top);
        assert!(result.at_bot);
    }

    #[test]
    fn test_format_search_stat_exceeded() {
        let mut stat = SearchStat::new();
        stat.cur = 50;
        stat.cnt = 99;
        stat.incomplete = 2;

        let format = SearchStatFormat::new();
        let result = format_search_stat(&stat, &format);

        assert!(result.show);
        assert!(result.approximate);
    }

    #[test]
    fn test_search_stat_state() {
        let mut state = SearchStatState::new();
        assert!(state.needs_recompute);

        state.mark_computed();
        assert!(!state.needs_recompute);

        state.invalidate();
        assert!(state.needs_recompute);
    }

    #[test]
    fn test_search_stat_state_position() {
        let mut state = SearchStatState::new();
        state.set_position(10, 5);

        assert!(!state.position_changed(10, 5));
        assert!(state.position_changed(10, 6));
        assert!(state.position_changed(11, 5));
    }
}
