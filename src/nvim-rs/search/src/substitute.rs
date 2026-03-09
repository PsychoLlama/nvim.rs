//! Substitution helpers
//!
//! This module provides functions for substitution command helpers,
//! including flag parsing, pattern index management, and count tracking.

use std::ffi::c_int;

use crate::state;

// =============================================================================
// C External Functions
// =============================================================================

extern "C" {
    fn nvim_set_last_idx(idx: c_int);
}

// =============================================================================
// Pattern Index Constants
// =============================================================================

/// Pattern index for search pattern
pub const RE_SEARCH: c_int = 0;
/// Pattern index for substitute pattern
pub const RE_SUBST: c_int = 1;
/// Pattern index for last used pattern
pub const RE_LAST: c_int = 2;
/// Pattern index for both search and substitute
pub const RE_BOTH: c_int = 3;

// =============================================================================
// Substitution Flag Constants
// =============================================================================

/// Substitution case handling: honor options
pub const SUB_HONOR_OPTIONS: c_int = 0;
/// Substitution case handling: ignore case
pub const SUB_IGNORE_CASE: c_int = 1;
/// Substitution case handling: match case
pub const SUB_MATCH_CASE: c_int = 2;

// =============================================================================
// Substitution Flags
// =============================================================================

/// Flags for the :substitute command.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SubstituteFlags {
    /// Replace all matches on a line (g flag)
    pub do_all: bool,
    /// Ask for confirmation (c flag)
    pub do_ask: bool,
    /// Show error for no matches (default true)
    pub do_error: bool,
    /// Print substituted line (p flag)
    pub do_print: bool,
    /// Print with line numbers (# flag)
    pub do_number: bool,
    /// Print in list format (l flag)
    pub do_list: bool,
    /// Count matches only (n flag)
    pub do_count: bool,
    /// Case handling: SUB_HONOR_OPTIONS, SUB_IGNORE_CASE, or SUB_MATCH_CASE
    pub do_ic: c_int,
}

impl Default for SubstituteFlags {
    fn default() -> Self {
        Self::new()
    }
}

impl SubstituteFlags {
    /// Create new flags with default values.
    pub const fn new() -> Self {
        Self {
            do_all: false,
            do_ask: false,
            do_error: true,
            do_print: false,
            do_number: false,
            do_list: false,
            do_count: false,
            do_ic: SUB_HONOR_OPTIONS,
        }
    }

    /// Create flags from the 'gdefault' option value.
    pub fn with_gdefault(gdefault: bool) -> Self {
        Self {
            do_all: gdefault,
            ..Self::new()
        }
    }

    /// Reset flags to defaults, optionally using 'gdefault'.
    pub fn reset(&mut self, gdefault: bool) {
        self.do_all = gdefault;
        self.do_ask = false;
        self.do_error = true;
        self.do_print = false;
        self.do_number = false;
        self.do_list = false;
        self.do_count = false;
        self.do_ic = SUB_HONOR_OPTIONS;
    }

    /// Toggle the 'global' flag.
    pub fn toggle_all(&mut self) {
        self.do_all = !self.do_all;
    }

    /// Toggle the 'confirm' flag.
    pub fn toggle_ask(&mut self) {
        self.do_ask = !self.do_ask;
    }

    /// Toggle the 'error' flag.
    pub fn toggle_error(&mut self) {
        self.do_error = !self.do_error;
    }

    /// Set print flag.
    pub fn set_print(&mut self) {
        self.do_print = true;
    }

    /// Set print with numbers.
    pub fn set_number(&mut self) {
        self.do_print = true;
        self.do_number = true;
    }

    /// Set print in list format.
    pub fn set_list(&mut self) {
        self.do_print = true;
        self.do_list = true;
    }

    /// Set count-only mode.
    pub fn set_count(&mut self) {
        self.do_count = true;
    }

    /// Set ignore case.
    pub fn set_ignore_case(&mut self) {
        self.do_ic = SUB_IGNORE_CASE;
    }

    /// Set match case.
    pub fn set_match_case(&mut self) {
        self.do_ic = SUB_MATCH_CASE;
    }

    /// After parsing, if counting, disable ask.
    pub fn finalize(&mut self) {
        if self.do_count {
            self.do_ask = false;
        }
    }

    /// Check if any output flags are set.
    pub fn has_output(&self) -> bool {
        self.do_print || self.do_number || self.do_list
    }

    /// Check if case should be ignored.
    pub fn ignores_case(&self, p_ic: bool, p_scs: bool, pattern_has_upper: bool) -> bool {
        match self.do_ic {
            SUB_IGNORE_CASE => true,
            SUB_MATCH_CASE => false,
            _ => {
                // Honor options: use p_ic, but check smartcase
                if p_ic {
                    if p_scs && pattern_has_upper {
                        false // smartcase: pattern has uppercase, match case
                    } else {
                        true // ignorecase is on
                    }
                } else {
                    false // ignorecase is off
                }
            }
        }
    }
}

// =============================================================================
// Pattern Index Management
// =============================================================================

/// Check if the substitute pattern is the last used.
#[inline]
pub fn substitute_was_last_used() -> bool {
    state::get_last_idx() == RE_SUBST
}

/// Check if the search pattern is the last used.
#[inline]
pub fn search_was_last_used() -> bool {
    state::get_last_idx() == RE_SEARCH
}

/// Set the substitute pattern as the last used.
#[inline]
pub fn set_substitute_as_last() {
    // SAFETY: Setting global variable through accessor
    unsafe { nvim_set_last_idx(RE_SUBST) }
}

/// Set the search pattern as the last used.
#[inline]
pub fn set_search_as_last() {
    // SAFETY: Setting global variable through accessor
    unsafe { nvim_set_last_idx(RE_SEARCH) }
}

/// Set which pattern is last used by boolean.
#[inline]
pub fn set_last_used_pattern(is_substitute: bool) {
    if is_substitute {
        set_substitute_as_last();
    } else {
        set_search_as_last();
    }
}

/// Get the pattern index to use when pat_use is RE_LAST.
#[inline]
pub fn resolve_pattern_index(pat_use: c_int) -> c_int {
    if pat_use == RE_LAST {
        state::get_last_idx()
    } else {
        pat_use
    }
}

// =============================================================================
// Substitution Count
// =============================================================================

/// Substitution count state for tracking matches.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SubstitutionCount {
    /// Total number of substitutions
    pub nsubs: c_int,
    /// Total number of lines changed
    pub nlines: c_int,
}

impl SubstitutionCount {
    /// Create a new zero count.
    pub const fn new() -> Self {
        Self {
            nsubs: 0,
            nlines: 0,
        }
    }

    /// Reset the count to zero.
    pub fn reset(&mut self) {
        self.nsubs = 0;
        self.nlines = 0;
    }

    /// Add a substitution.
    pub fn add_sub(&mut self) {
        self.nsubs += 1;
    }

    /// Add a line change.
    pub fn add_line(&mut self) {
        self.nlines += 1;
    }

    /// Add multiple substitutions (e.g., for join lines).
    pub fn add_subs(&mut self, count: c_int) {
        self.nsubs += count;
    }

    /// Check if any substitutions were made.
    pub fn has_subs(&self) -> bool {
        self.nsubs > 0
    }

    /// Check if any lines were changed.
    pub fn has_lines(&self) -> bool {
        self.nlines > 0
    }

    /// Combine counts (e.g., after a range operation).
    pub fn combine(&mut self, other: &SubstitutionCount) {
        self.nsubs += other.nsubs;
        self.nlines += other.nlines;
    }
}

// =============================================================================
// Flag Character Parsing
// =============================================================================

/// Parse a single substitution flag character.
///
/// Returns true if the character was a valid flag.
#[inline]
pub fn parse_flag_char(c: u8, flags: &mut SubstituteFlags) -> bool {
    match c {
        b'g' => {
            flags.toggle_all();
            true
        }
        b'c' => {
            flags.toggle_ask();
            true
        }
        b'n' => {
            flags.set_count();
            true
        }
        b'e' => {
            flags.toggle_error();
            true
        }
        b'r' => {
            // 'r' flag is handled by caller (use previous pattern)
            true
        }
        b'p' => {
            flags.set_print();
            true
        }
        b'#' => {
            flags.set_number();
            true
        }
        b'l' => {
            flags.set_list();
            true
        }
        b'i' | b'I' => {
            if c == b'i' {
                flags.set_ignore_case();
            } else {
                flags.set_match_case();
            }
            true
        }
        _ => false,
    }
}

/// Check if a character is a valid substitute flag.
#[inline]
pub fn is_subst_flag(c: u8) -> bool {
    matches!(
        c,
        b'g' | b'c' | b'n' | b'e' | b'r' | b'p' | b'#' | b'l' | b'i' | b'I'
    )
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Create new substitute flags.
#[no_mangle]
pub extern "C" fn rs_subst_flags_new() -> SubstituteFlags {
    SubstituteFlags::new()
}

/// FFI: Create substitute flags with gdefault.
#[no_mangle]
pub extern "C" fn rs_subst_flags_with_gdefault(gdefault: c_int) -> SubstituteFlags {
    SubstituteFlags::with_gdefault(gdefault != 0)
}

/// FFI: Reset substitute flags.
///
/// # Safety
///
/// The caller must ensure `flags` points to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_subst_flags_reset(flags: *mut SubstituteFlags, gdefault: c_int) {
    if !flags.is_null() {
        (*flags).reset(gdefault != 0);
    }
}

/// FFI: Finalize substitute flags after parsing.
///
/// # Safety
///
/// The caller must ensure `flags` points to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_subst_flags_finalize(flags: *mut SubstituteFlags) {
    if !flags.is_null() {
        (*flags).finalize();
    }
}

/// FFI: Parse a single flag character.
///
/// # Safety
///
/// The caller must ensure `flags` points to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_subst_parse_flag_char(c: c_int, flags: *mut SubstituteFlags) -> c_int {
    if flags.is_null() {
        return 0;
    }
    c_int::from(parse_flag_char(c as u8, &mut *flags))
}

/// FFI: Check if character is a substitute flag.
#[no_mangle]
pub extern "C" fn rs_is_subst_flag(c: c_int) -> c_int {
    c_int::from(is_subst_flag(c as u8))
}

/// FFI: Check if substitute was last used.
#[no_mangle]
pub extern "C" fn rs_substitute_was_last_used() -> c_int {
    c_int::from(substitute_was_last_used())
}

/// FFI: Set substitute as last used.
#[no_mangle]
pub extern "C" fn rs_set_substitute_as_last() {
    set_substitute_as_last();
}

/// FFI: Set search as last used.
#[no_mangle]
pub extern "C" fn rs_set_search_as_last() {
    set_search_as_last();
}

/// FFI: Set last used pattern.
#[no_mangle]
pub extern "C" fn rs_set_last_used_pattern(is_substitute: c_int) {
    set_last_used_pattern(is_substitute != 0);
}

/// C ABI export for `set_last_used_pattern`, taking `bool` to match C callers.
#[unsafe(export_name = "set_last_used_pattern")]
pub extern "C" fn set_last_used_pattern_export(is_substitute: bool) {
    set_last_used_pattern(is_substitute);
}

/// FFI: Resolve pattern index for substitute.
#[no_mangle]
pub extern "C" fn rs_subst_resolve_pattern_index(pat_use: c_int) -> c_int {
    resolve_pattern_index(pat_use)
}

/// FFI: Create new substitution count.
#[no_mangle]
pub extern "C" fn rs_subst_count_new() -> SubstitutionCount {
    SubstitutionCount::new()
}

/// FFI: Reset substitution count.
///
/// # Safety
///
/// The caller must ensure `count` points to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_subst_count_reset(count: *mut SubstitutionCount) {
    if !count.is_null() {
        (*count).reset();
    }
}

/// FFI: Add a substitution to count.
///
/// # Safety
///
/// The caller must ensure `count` points to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_subst_count_add_sub(count: *mut SubstitutionCount) {
    if !count.is_null() {
        (*count).add_sub();
    }
}

/// FFI: Add a line to count.
///
/// # Safety
///
/// The caller must ensure `count` points to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_subst_count_add_line(count: *mut SubstitutionCount) {
    if !count.is_null() {
        (*count).add_line();
    }
}

/// FFI: Check if flags should ignore case.
///
/// # Safety
///
/// The caller must ensure `flags` points to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_subst_flags_ignores_case(
    flags: *const SubstituteFlags,
    p_ic: c_int,
    p_scs: c_int,
    has_upper: c_int,
) -> c_int {
    if flags.is_null() {
        return 0;
    }
    c_int::from((*flags).ignores_case(p_ic != 0, p_scs != 0, has_upper != 0))
}

/// FFI: Get RE_SEARCH constant.
#[no_mangle]
pub extern "C" fn rs_re_search_const() -> c_int {
    RE_SEARCH
}

/// FFI: Get RE_SUBST constant.
#[no_mangle]
pub extern "C" fn rs_re_subst_const() -> c_int {
    RE_SUBST
}

/// FFI: Get RE_LAST constant.
#[no_mangle]
pub extern "C" fn rs_re_last_const() -> c_int {
    RE_LAST
}

/// FFI: Get RE_BOTH constant.
#[no_mangle]
pub extern "C" fn rs_re_both_const() -> c_int {
    RE_BOTH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_flags_new() {
        let flags = SubstituteFlags::new();
        assert!(!flags.do_all);
        assert!(!flags.do_ask);
        assert!(flags.do_error);
        assert!(!flags.do_print);
        assert!(!flags.do_count);
        assert_eq!(flags.do_ic, SUB_HONOR_OPTIONS);
    }

    #[test]
    fn test_substitute_flags_with_gdefault() {
        let flags = SubstituteFlags::with_gdefault(true);
        assert!(flags.do_all);

        let flags = SubstituteFlags::with_gdefault(false);
        assert!(!flags.do_all);
    }

    #[test]
    fn test_substitute_flags_toggle() {
        let mut flags = SubstituteFlags::new();

        flags.toggle_all();
        assert!(flags.do_all);
        flags.toggle_all();
        assert!(!flags.do_all);

        flags.toggle_ask();
        assert!(flags.do_ask);

        flags.toggle_error();
        assert!(!flags.do_error);
    }

    #[test]
    fn test_substitute_flags_set() {
        let mut flags = SubstituteFlags::new();

        flags.set_print();
        assert!(flags.do_print);

        flags.set_number();
        assert!(flags.do_print);
        assert!(flags.do_number);

        flags.set_list();
        assert!(flags.do_list);

        flags.set_count();
        assert!(flags.do_count);
    }

    #[test]
    fn test_substitute_flags_case() {
        let mut flags = SubstituteFlags::new();

        flags.set_ignore_case();
        assert_eq!(flags.do_ic, SUB_IGNORE_CASE);

        flags.set_match_case();
        assert_eq!(flags.do_ic, SUB_MATCH_CASE);
    }

    #[test]
    fn test_substitute_flags_finalize() {
        let mut flags = SubstituteFlags::new();
        flags.do_ask = true;
        flags.do_count = true;

        flags.finalize();
        assert!(!flags.do_ask); // Disabled because counting
    }

    #[test]
    fn test_substitute_flags_ignores_case() {
        let mut flags = SubstituteFlags::new();

        // SUB_IGNORE_CASE always ignores
        flags.set_ignore_case();
        assert!(flags.ignores_case(false, false, false));
        assert!(flags.ignores_case(false, true, true));

        // SUB_MATCH_CASE never ignores
        flags.set_match_case();
        assert!(!flags.ignores_case(true, false, false));
        assert!(!flags.ignores_case(true, true, false));

        // SUB_HONOR_OPTIONS depends on p_ic and smartcase
        flags.do_ic = SUB_HONOR_OPTIONS;

        // p_ic = false -> don't ignore
        assert!(!flags.ignores_case(false, false, false));

        // p_ic = true, no smartcase -> ignore
        assert!(flags.ignores_case(true, false, false));

        // p_ic = true, smartcase, no uppercase -> ignore
        assert!(flags.ignores_case(true, true, false));

        // p_ic = true, smartcase, has uppercase -> don't ignore
        assert!(!flags.ignores_case(true, true, true));
    }

    #[test]
    fn test_parse_flag_char() {
        let mut flags = SubstituteFlags::new();

        assert!(parse_flag_char(b'g', &mut flags));
        assert!(flags.do_all);

        assert!(parse_flag_char(b'c', &mut flags));
        assert!(flags.do_ask);

        assert!(parse_flag_char(b'n', &mut flags));
        assert!(flags.do_count);

        assert!(parse_flag_char(b'p', &mut flags));
        assert!(flags.do_print);

        assert!(parse_flag_char(b'i', &mut flags));
        assert_eq!(flags.do_ic, SUB_IGNORE_CASE);

        assert!(parse_flag_char(b'I', &mut flags));
        assert_eq!(flags.do_ic, SUB_MATCH_CASE);

        // Invalid flag
        assert!(!parse_flag_char(b'x', &mut flags));
    }

    #[test]
    fn test_is_subst_flag() {
        assert!(is_subst_flag(b'g'));
        assert!(is_subst_flag(b'c'));
        assert!(is_subst_flag(b'n'));
        assert!(is_subst_flag(b'e'));
        assert!(is_subst_flag(b'r'));
        assert!(is_subst_flag(b'p'));
        assert!(is_subst_flag(b'#'));
        assert!(is_subst_flag(b'l'));
        assert!(is_subst_flag(b'i'));
        assert!(is_subst_flag(b'I'));

        assert!(!is_subst_flag(b'x'));
        assert!(!is_subst_flag(b'a'));
        assert!(!is_subst_flag(b' '));
    }

    #[test]
    fn test_substitution_count() {
        let mut count = SubstitutionCount::new();
        assert_eq!(count.nsubs, 0);
        assert_eq!(count.nlines, 0);
        assert!(!count.has_subs());
        assert!(!count.has_lines());

        count.add_sub();
        assert_eq!(count.nsubs, 1);
        assert!(count.has_subs());

        count.add_line();
        assert_eq!(count.nlines, 1);
        assert!(count.has_lines());

        count.add_subs(5);
        assert_eq!(count.nsubs, 6);

        let mut other = SubstitutionCount::new();
        other.nsubs = 10;
        other.nlines = 3;
        count.combine(&other);
        assert_eq!(count.nsubs, 16);
        assert_eq!(count.nlines, 4);

        count.reset();
        assert_eq!(count.nsubs, 0);
        assert_eq!(count.nlines, 0);
    }

    #[test]
    fn test_pattern_constants() {
        assert_eq!(RE_SEARCH, 0);
        assert_eq!(RE_SUBST, 1);
        assert_eq!(RE_LAST, 2);
        assert_eq!(RE_BOTH, 3);
    }

    #[test]
    fn test_case_constants() {
        assert_eq!(SUB_HONOR_OPTIONS, 0);
        assert_eq!(SUB_IGNORE_CASE, 1);
        assert_eq!(SUB_MATCH_CASE, 2);
    }
}
