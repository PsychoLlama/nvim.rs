//! `:global` and `:vglobal` command implementation.
//!
//! The `:global` (`:g`) command executes an Ex command on all lines matching
//! a pattern. The `:vglobal` (`:v`) command executes on non-matching lines.
//!
//! ## Usage
//! - `:g/pattern/cmd` - Execute cmd on lines matching pattern
//! - `:g!/pattern/cmd` - Execute cmd on lines NOT matching pattern (same as :v)
//! - `:v/pattern/cmd` - Execute cmd on lines NOT matching pattern
//! - `:{range}g/pattern/cmd` - Limit to range
//!
//! ## Implementation Notes
//!
//! The global command works in two phases:
//! 1. Mark all matching (or non-matching) lines
//! 2. Execute the command on each marked line
//!
//! This two-phase approach is necessary because executing commands can
//! change line numbers, so we can't simply iterate through lines.

use std::ffi::{c_char, c_int};

use crate::range::{LineNr, LineRange};
use crate::ExArgHandle;

extern crate libc;

extern "C" {
    // global_exe FFI
    fn setpcmark();
    fn nvim_excmds_set_msg_didout(val: c_int);
    fn nvim_excmds_set_sub_nsubs(val: c_int);
    fn nvim_excmds_set_sub_nlines(val: c_int);
    fn nvim_excmds_set_global_need_beginline(val: c_int);
    fn nvim_excmds_set_global_busy(val: c_int);
    fn nvim_excmds_global_busy() -> c_int;
    fn nvim_curbuf_get_b_ml_ml_line_count() -> c_int;
    fn nvim_excmds_got_int() -> c_int;
    fn nvim_excmds_ml_firstmarked() -> c_int;
    fn nvim_curwin_set_cursor_lnum(lnum: c_int);
    fn nvim_curwin_set_cursor_col(col: c_int);
    fn nvim_excmds_do_cmdline_global(cmd: *const c_char);
    fn os_breakcheck();
    fn nvim_excmds_get_global_need_beginline() -> c_int;
    fn beginline(flags: c_int);
    fn nvim_excmds_check_cursor_curwin();
    fn nvim_excmds_changed_line_abv_curs();
    fn nvim_excmds_get_msg_col() -> c_int;
    fn nvim_excmds_get_msg_scrolled() -> c_int;
    fn nvim_excmds_get_curbuf_identity() -> *mut std::ffi::c_void;
    fn msgmore(n: c_int);

    // ex_global FFI
    fn nvim_exarg_get_cmd(eap: *const ExArgHandle) -> *const c_char;
    fn nvim_excmds_get_arg_mut(eap: *mut ExArgHandle) -> *mut c_char;
    fn nvim_exarg_get_line1(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_get_line2(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_get_forceit(eap: *const ExArgHandle) -> c_int;
    fn nvim_excmds_search_regcomp_multi(
        pat: *const c_char,
        patlen: usize,
        used_pat_out: *mut *const c_char,
        which_pat: c_int,
    ) -> *mut std::ffi::c_void;
    fn nvim_excmds_vim_regexec_multi(regmatch: *mut std::ffi::c_void, lnum: c_int) -> c_int;
    fn nvim_excmds_vim_regfree_multi(regmatch: *mut std::ffi::c_void);
    fn nvim_excmds_regmmatch_regprog_null(regmatch: *mut std::ffi::c_void) -> c_int;
    fn nvim_excmds_skip_regexp_ex_global(
        eap: *mut ExArgHandle,
        pat: *mut c_char,
        delim: c_int,
    ) -> *mut c_char;
    fn nvim_excmds_ml_setmarked(lnum: c_int);
    fn nvim_excmds_ml_clearmarked();
    fn nvim_excmds_line_breakcheck();
    fn nvim_excmds_emsg_by_id(id: c_int);
    fn nvim_excmds_emsg_with_arg(id: c_int, arg: *const c_char);
    fn nvim_excmds_curwin_cursor_lnum() -> c_int;
    fn nvim_excmds_curwin_set_col_zero();
    fn rs_check_regexp_delim(c: c_int) -> c_int;
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
}

/// BL_WHITE | BL_FIX flags for beginline()
const BL_WHITE: c_int = 1;
const BL_FIX: c_int = 4;

// Forward declaration - rs_do_sub_msg is in substitute.rs in this crate (exported as do_sub_msg)
extern "C" {
    #[link_name = "do_sub_msg"]
    fn rs_do_sub_msg(count_only: bool) -> bool;
}

/// Type of global command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum GlobalType {
    /// Normal global: match lines (:g)
    #[default]
    Global = 0,
    /// Inverse global: non-matching lines (:v or :g!)
    VGlobal = 1,
}

impl GlobalType {
    /// Create from whether the command has a bang.
    #[inline]
    #[must_use]
    pub const fn from_bang(has_bang: bool) -> Self {
        if has_bang {
            GlobalType::VGlobal
        } else {
            GlobalType::Global
        }
    }

    /// Check if this matches non-matching lines (inverted).
    #[inline]
    #[must_use]
    pub const fn is_inverted(&self) -> bool {
        matches!(self, GlobalType::VGlobal)
    }

    /// Convert from C integer.
    #[inline]
    #[must_use]
    pub fn from_c(value: c_int) -> Self {
        if value == 0 {
            GlobalType::Global
        } else {
            GlobalType::VGlobal
        }
    }

    /// Convert to C integer.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        self as c_int
    }
}

/// Options for the global command.
#[derive(Debug, Clone, Default)]
pub struct GlobalOptions {
    /// Range of lines to search.
    pub range: LineRange,
    /// Type of global command (normal or inverted).
    pub global_type: GlobalType,
    /// Pattern to match (empty = use last pattern).
    pub pattern: String,
    /// Command to execute on matching lines.
    pub command: String,
}

impl GlobalOptions {
    /// Create options for a `:global` command.
    #[must_use]
    pub fn global(range: LineRange, pattern: String, command: String) -> Self {
        Self {
            range,
            global_type: GlobalType::Global,
            pattern,
            command,
        }
    }

    /// Create options for a `:vglobal` command.
    #[must_use]
    pub fn vglobal(range: LineRange, pattern: String, command: String) -> Self {
        Self {
            range,
            global_type: GlobalType::VGlobal,
            pattern,
            command,
        }
    }
}

/// State tracking for global command execution.
#[derive(Debug, Clone, Default)]
pub struct GlobalState {
    /// Whether we're currently executing a global command.
    pub busy: bool,
    /// Number of lines processed.
    pub lines_processed: i32,
    /// Number of lines where command was executed.
    pub lines_executed: i32,
}

impl GlobalState {
    /// Create a new state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            busy: false,
            lines_processed: 0,
            lines_executed: 0,
        }
    }

    /// Start global command execution.
    pub fn start(&mut self) {
        self.busy = true;
        self.lines_processed = 0;
        self.lines_executed = 0;
    }

    /// Record a line being processed.
    pub fn process_line(&mut self) {
        self.lines_processed += 1;
    }

    /// Record a command execution.
    pub fn execute_line(&mut self) {
        self.lines_executed += 1;
    }

    /// Finish global command execution.
    pub fn finish(&mut self) {
        self.busy = false;
    }

    /// Check if currently executing a global command.
    #[inline]
    #[must_use]
    pub const fn is_busy(&self) -> bool {
        self.busy
    }
}

/// Result of the global command.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GlobalResult {
    /// Number of matching lines found.
    pub matches: i32,
    /// Number of commands executed.
    pub executed: i32,
    /// Whether the operation was interrupted.
    pub interrupted: bool,
}

impl GlobalResult {
    /// Create a new empty result.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            matches: 0,
            executed: 0,
            interrupted: false,
        }
    }

    /// Check if any matches were found.
    #[inline]
    #[must_use]
    pub const fn has_matches(&self) -> bool {
        self.matches > 0
    }

    /// Record a match.
    #[inline]
    pub fn add_match(&mut self) {
        self.matches += 1;
    }

    /// Record a command execution.
    #[inline]
    pub fn add_execution(&mut self) {
        self.executed += 1;
    }

    /// Mark as interrupted.
    #[inline]
    pub fn set_interrupted(&mut self) {
        self.interrupted = true;
    }
}

/// Error type for global command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalError {
    /// Invalid pattern.
    InvalidPattern(String),
    /// No previous pattern.
    NoPreviousPattern,
    /// Invalid delimiter.
    InvalidDelimiter(char),
    /// Nested global command.
    NestedGlobal,
    /// Operation was interrupted.
    Interrupted,
    /// Invalid range.
    InvalidRange,
}

impl std::fmt::Display for GlobalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlobalError::InvalidPattern(msg) => write!(f, "invalid pattern: {msg}"),
            GlobalError::NoPreviousPattern => write!(f, "no previous pattern"),
            GlobalError::InvalidDelimiter(c) => write!(f, "invalid delimiter: {c}"),
            GlobalError::NestedGlobal => write!(f, "cannot nest global commands"),
            GlobalError::Interrupted => write!(f, "interrupted"),
            GlobalError::InvalidRange => write!(f, "invalid range"),
        }
    }
}

impl std::error::Error for GlobalError {}

/// A marked line for global command execution.
///
/// During the marking phase, we record the line number and position
/// of each matching line. The position is used for cursor placement
/// when executing the command.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkedLine {
    /// Line number (1-based).
    pub lnum: LineNr,
    /// Column position for cursor (0-based).
    pub col: i32,
}

impl MarkedLine {
    /// Create a new marked line.
    #[must_use]
    pub const fn new(lnum: LineNr, col: i32) -> Self {
        Self { lnum, col }
    }

    /// Create a marked line at the beginning of the line.
    #[must_use]
    pub const fn at_start(lnum: LineNr) -> Self {
        Self { lnum, col: 0 }
    }
}

/// Collection of marked lines for batch processing.
#[derive(Debug, Clone, Default)]
pub struct MarkedLines {
    lines: Vec<MarkedLine>,
}

impl MarkedLines {
    /// Create a new empty collection.
    #[must_use]
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    /// Create with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            lines: Vec::with_capacity(capacity),
        }
    }

    /// Add a marked line.
    pub fn push(&mut self, line: MarkedLine) {
        self.lines.push(line);
    }

    /// Get the number of marked lines.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Clear all marked lines.
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Iterate over marked lines.
    pub fn iter(&self) -> impl Iterator<Item = &MarkedLine> {
        self.lines.iter()
    }

    /// Iterate in reverse order (for processing from bottom to top).
    ///
    /// Processing from bottom to top can be useful when the command
    /// modifies line count, as it won't affect earlier line numbers.
    pub fn iter_rev(&self) -> impl Iterator<Item = &MarkedLine> {
        self.lines.iter().rev()
    }
}

impl IntoIterator for MarkedLines {
    type Item = MarkedLine;
    type IntoIter = std::vec::IntoIter<MarkedLine>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Execute `cmd` on lines marked with ml_setmarked(). Replaces C `global_exe` + `global_exe_one`.
///
/// # Safety
/// `cmd` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_global_exe(cmd: *const c_char) {
    // Set current position only once for a global command.
    // If global_busy is set, setpcmark() will not do anything.
    // If there is an error, global_busy will be incremented.
    setpcmark();

    // When the command writes a message, don't overwrite the command.
    nvim_excmds_set_msg_didout(1);

    nvim_excmds_set_sub_nsubs(0);
    nvim_excmds_set_sub_nlines(0);
    nvim_excmds_set_global_need_beginline(0);
    nvim_excmds_set_global_busy(1);

    let old_lcount = nvim_curbuf_get_b_ml_ml_line_count();
    let old_buf = nvim_excmds_get_curbuf_identity();

    while nvim_excmds_got_int() == 0 {
        let lnum = nvim_excmds_ml_firstmarked();
        if lnum == 0 || nvim_excmds_global_busy() != 1 {
            break;
        }
        // global_exe_one: set cursor position and execute command
        nvim_curwin_set_cursor_lnum(lnum);
        nvim_curwin_set_cursor_col(0);
        nvim_excmds_do_cmdline_global(cmd);
        os_breakcheck();
    }

    nvim_excmds_set_global_busy(0);
    if nvim_excmds_get_global_need_beginline() != 0 {
        beginline(BL_WHITE | BL_FIX);
    } else {
        nvim_excmds_check_cursor_curwin();
    }

    // The cursor may not have moved in the text but a change in a previous
    // line may move it on the screen.
    nvim_excmds_changed_line_abv_curs();

    // If it looks like no message was written, allow overwriting the command
    // with the report for number of changes.
    if nvim_excmds_get_msg_col() == 0 && nvim_excmds_get_msg_scrolled() == 0 {
        nvim_excmds_set_msg_didout(0);
    }

    // If substitutes done, report number of substitutes; otherwise report
    // number of extra or deleted lines.
    // Don't report extra or deleted lines in the edge case where the buffer
    // we are in after execution is different from the buffer we started in.
    let sub_reported = rs_do_sub_msg(false);
    if !sub_reported && nvim_excmds_get_curbuf_identity() == old_buf {
        let new_lcount = nvim_curbuf_get_b_ml_ml_line_count();
        msgmore(new_lcount - old_lcount);
    }
}

/// Create a global type from bang flag.
///
/// Returns 0 for normal global, 1 for vglobal.
pub extern "C" fn rs_global_type_from_bang(has_bang: c_int) -> c_int {
    GlobalType::from_bang(has_bang != 0).to_c()
}

/// Check if a global type is inverted.
pub extern "C" fn rs_global_type_is_inverted(global_type: c_int) -> c_int {
    c_int::from(GlobalType::from_c(global_type).is_inverted())
}

/// FFI export: Get GlobalType::Global constant
pub extern "C" fn rs_global_type_global() -> c_int {
    GlobalType::Global.to_c()
}

/// FFI export: Get GlobalType::VGlobal constant
pub extern "C" fn rs_global_type_vglobal() -> c_int {
    GlobalType::VGlobal.to_c()
}

/// FFI export: Create GlobalResult
pub extern "C" fn rs_global_result_new() -> GlobalResult {
    GlobalResult::new()
}

/// FFI export: Check if GlobalResult has matches
pub extern "C" fn rs_global_result_has_matches(matches: c_int) -> c_int {
    c_int::from(matches > 0)
}

/// FFI export: Create MarkedLine at start
pub extern "C" fn rs_marked_line_at_start(lnum: LineNr) -> MarkedLine {
    MarkedLine::at_start(lnum)
}

/// FFI export: Create MarkedLine with column
pub extern "C" fn rs_marked_line_new(lnum: LineNr, col: c_int) -> MarkedLine {
    MarkedLine::new(lnum, col)
}

/// FFI export: Get GlobalError::InvalidPattern code
pub extern "C" fn rs_global_error_invalid_pattern() -> c_int {
    0
}

/// FFI export: Get GlobalError::NoPreviousPattern code
pub extern "C" fn rs_global_error_no_previous_pattern() -> c_int {
    1
}

/// FFI export: Get GlobalError::InvalidDelimiter code
pub extern "C" fn rs_global_error_invalid_delimiter() -> c_int {
    2
}

/// FFI export: Get GlobalError::NestedGlobal code
pub extern "C" fn rs_global_error_nested() -> c_int {
    3
}

/// FFI export: Get GlobalError::Interrupted code
pub extern "C" fn rs_global_error_interrupted() -> c_int {
    4
}

/// FFI export: Get GlobalError::InvalidRange code
pub extern "C" fn rs_global_error_invalid_range() -> c_int {
    5
}

/// FFI export: Initialize GlobalState
pub extern "C" fn rs_global_state_is_busy(busy: c_int) -> c_int {
    c_int::from(busy != 0)
}

/// Implement `:global`/`:vglobal` command. Replaces C `ex_global`.
///
/// Two-phase approach:
/// 1. Mark all matching (or non-matching) lines with ml_setmarked()
/// 2. Execute the command on each marked line via global_exe (rs_global_exe)
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[export_name = "ex_global"]
pub unsafe extern "C" fn rs_ex_global(eap: *mut ExArgHandle) {
    // Constants matching C values
    const RE_LAST: c_int = 2;
    const RE_SEARCH: c_int = 0;
    const RE_SUBST: c_int = 1;
    const FAIL: c_int = 0;

    let line1 = nvim_exarg_get_line1(eap);
    let line2 = nvim_exarg_get_line2(eap);
    let forceit = nvim_exarg_get_forceit(eap);
    let ml_line_count = nvim_curbuf_get_b_ml_ml_line_count();

    // When nesting the command works on one line. This allows for
    // ":g/found/v/notfound/command".
    if nvim_excmds_global_busy() != 0 && (line1 != 1 || line2 != ml_line_count) {
        // will increment global_busy to break out of the loop
        nvim_excmds_emsg_by_id(2); // E147: Cannot do :global recursive with a range
        return;
    }

    // Determine type: 'g' (global) or 'v' (vglobal)
    // forceit means ":global!" => treat as 'v'
    let cmd_ptr = nvim_exarg_get_cmd(eap);
    let type_char: u8 = if forceit != 0 { b'v' } else { *cmd_ptr as u8 };

    // cmd starts at eap->arg (mutable for skip_regexp_ex)
    let mut cmd = nvim_excmds_get_arg_mut(eap);
    let mut which_pat: c_int = RE_LAST;
    let pat: *const c_char;
    let patlen: usize;

    // Undocumented vi feature: "\/" "\?" use previous search, "\&" uses previous substitute.
    if *cmd == b'\\' as c_char {
        cmd = cmd.add(1);
        // Check that next char is one of /?&
        let ok = vim_strchr(c"/?&".as_ptr(), *cmd as c_int);
        if ok.is_null() {
            nvim_excmds_emsg_by_id(4); // e_backslash
            return;
        }
        if *cmd == b'&' as c_char {
            which_pat = RE_SUBST;
        } else {
            which_pat = RE_SEARCH;
        }
        cmd = cmd.add(1);
        pat = c"".as_ptr();
        patlen = 0;
    } else if *cmd == 0 {
        nvim_excmds_emsg_by_id(3); // E148: Regular expression missing from global
        return;
    } else if rs_check_regexp_delim(*cmd as c_int) == FAIL {
        return;
    } else {
        let delim = *cmd as c_int;
        cmd = cmd.add(1); // skip delimiter
        pat = cmd as *const c_char; // remember start of pattern
        cmd = nvim_excmds_skip_regexp_ex_global(eap, cmd, delim);
        if *cmd as c_int == delim {
            // end delimiter found - replace with NUL
            *cmd = 0;
            cmd = cmd.add(1);
        }
        patlen = libc::strlen(pat);
    }

    // Compile the pattern
    let mut used_pat: *const c_char = std::ptr::null();
    let regmatch = nvim_excmds_search_regcomp_multi(pat, patlen, &mut used_pat, which_pat);
    if regmatch.is_null() {
        nvim_excmds_emsg_by_id(5); // e_invcmd
        return;
    }

    if nvim_excmds_global_busy() != 0 {
        // Nested global: work on the current line only.
        let lnum = nvim_excmds_curwin_cursor_lnum();
        let matched = nvim_excmds_vim_regexec_multi(regmatch, lnum);
        if (type_char == b'g' && matched != 0) || (type_char == b'v' && matched == 0) {
            nvim_excmds_curwin_set_col_zero();
            nvim_excmds_do_cmdline_global(cmd);
        }
    } else {
        let mut ndone: c_int = 0;
        // Pass 1: mark all (not) matching lines
        let mut lnum = nvim_exarg_get_line1(eap);
        let end_lnum = nvim_exarg_get_line2(eap);
        while lnum <= end_lnum && nvim_excmds_got_int() == 0 {
            let matched = nvim_excmds_vim_regexec_multi(regmatch, lnum);
            if nvim_excmds_regmmatch_regprog_null(regmatch) != 0 {
                break; // re-compiling regprog failed
            }
            if (type_char == b'g' && matched != 0) || (type_char == b'v' && matched == 0) {
                nvim_excmds_ml_setmarked(lnum);
                ndone += 1;
            }
            nvim_excmds_line_breakcheck();
            lnum += 1;
        }

        // Pass 2: execute the command for each marked line
        if nvim_excmds_got_int() != 0 {
            nvim_excmds_emsg_by_id(6); // e_interr (msg)
        } else if ndone == 0 {
            if type_char == b'v' {
                nvim_excmds_emsg_with_arg(2, used_pat); // smsg_pattern_found_every
            } else {
                nvim_excmds_emsg_with_arg(1, used_pat); // smsg_pattern_not_found
            }
        } else {
            rs_global_exe(cmd);
        }
        nvim_excmds_ml_clearmarked();
    }

    nvim_excmds_vim_regfree_multi(regmatch);
}

/// FFI export: Check if lines processed count is valid
pub extern "C" fn rs_global_has_processed_lines(lines_processed: c_int) -> c_int {
    c_int::from(lines_processed > 0)
}

/// FFI export: Check if lines executed count is valid
pub extern "C" fn rs_global_has_executed_lines(lines_executed: c_int) -> c_int {
    c_int::from(lines_executed > 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_type() {
        assert_eq!(GlobalType::Global.to_c(), 0);
        assert_eq!(GlobalType::VGlobal.to_c(), 1);

        assert_eq!(GlobalType::from_c(0), GlobalType::Global);
        assert_eq!(GlobalType::from_c(1), GlobalType::VGlobal);
        assert_eq!(GlobalType::from_c(99), GlobalType::VGlobal);
    }

    #[test]
    fn test_global_type_from_bang() {
        assert_eq!(GlobalType::from_bang(false), GlobalType::Global);
        assert_eq!(GlobalType::from_bang(true), GlobalType::VGlobal);
    }

    #[test]
    fn test_global_type_is_inverted() {
        assert!(!GlobalType::Global.is_inverted());
        assert!(GlobalType::VGlobal.is_inverted());
    }

    #[test]
    fn test_global_options() {
        let range = LineRange::whole_buffer(100);
        let opts = GlobalOptions::global(range, "pattern".to_string(), "delete".to_string());
        assert_eq!(opts.global_type, GlobalType::Global);
        assert_eq!(opts.pattern, "pattern");
        assert_eq!(opts.command, "delete");

        let opts = GlobalOptions::vglobal(range, "pattern".to_string(), "delete".to_string());
        assert_eq!(opts.global_type, GlobalType::VGlobal);
    }

    #[test]
    fn test_global_state() {
        let mut state = GlobalState::new();
        assert!(!state.is_busy());

        state.start();
        assert!(state.is_busy());

        state.process_line();
        state.process_line();
        assert_eq!(state.lines_processed, 2);

        state.execute_line();
        assert_eq!(state.lines_executed, 1);

        state.finish();
        assert!(!state.is_busy());
    }

    #[test]
    fn test_global_result() {
        let mut result = GlobalResult::new();
        assert!(!result.has_matches());

        result.add_match();
        result.add_match();
        assert!(result.has_matches());
        assert_eq!(result.matches, 2);

        result.add_execution();
        assert_eq!(result.executed, 1);

        result.set_interrupted();
        assert!(result.interrupted);
    }

    #[test]
    fn test_global_error_display() {
        let err = GlobalError::InvalidPattern("bad regex".to_string());
        assert_eq!(format!("{err}"), "invalid pattern: bad regex");

        let err = GlobalError::NoPreviousPattern;
        assert_eq!(format!("{err}"), "no previous pattern");

        let err = GlobalError::NestedGlobal;
        assert_eq!(format!("{err}"), "cannot nest global commands");
    }

    #[test]
    fn test_marked_line() {
        let line = MarkedLine::new(10, 5);
        assert_eq!(line.lnum, 10);
        assert_eq!(line.col, 5);

        let line = MarkedLine::at_start(20);
        assert_eq!(line.lnum, 20);
        assert_eq!(line.col, 0);
    }

    #[test]
    fn test_marked_lines() {
        let mut lines = MarkedLines::new();
        assert!(lines.is_empty());

        lines.push(MarkedLine::at_start(10));
        lines.push(MarkedLine::at_start(20));
        lines.push(MarkedLine::at_start(30));

        assert_eq!(lines.len(), 3);
        assert!(!lines.is_empty());

        let lnums: Vec<_> = lines.iter().map(|l| l.lnum).collect();
        assert_eq!(lnums, vec![10, 20, 30]);

        let lnums_rev: Vec<_> = lines.iter_rev().map(|l| l.lnum).collect();
        assert_eq!(lnums_rev, vec![30, 20, 10]);

        lines.clear();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_marked_lines_into_iter() {
        let mut lines = MarkedLines::new();
        lines.push(MarkedLine::at_start(10));
        lines.push(MarkedLine::at_start(20));

        let lnums: Vec<_> = lines.into_iter().map(|l| l.lnum).collect();
        assert_eq!(lnums, vec![10, 20]);
    }

    #[test]
    fn test_rs_global_type_from_bang() {
        assert_eq!(rs_global_type_from_bang(0), 0);
        assert_eq!(rs_global_type_from_bang(1), 1);
    }

    #[test]
    fn test_rs_global_type_is_inverted() {
        assert_eq!(rs_global_type_is_inverted(0), 0);
        assert_eq!(rs_global_type_is_inverted(1), 1);
    }
}
