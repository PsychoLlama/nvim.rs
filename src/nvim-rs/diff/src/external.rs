//! External diff command utilities
//!
//! This module provides structures and helpers for running external diff
//! commands and parsing their output. It handles:
//! - Diff command flag construction
//! - Temp file management helpers
//! - Output format detection (normal vs unified)
//! - Shell command building

use std::ffi::{c_char, c_int};

// Line number type matching linenr_T (i32)
type LinenrT = i32;

// Result constants
const OK: c_int = 1;
const FAIL: c_int = 0;

// Diff flags (must match C #define values)
const DIFF_IBLANK: c_int = 0x002;
const DIFF_ICASE: c_int = 0x004;
const DIFF_IWHITE: c_int = 0x008;
const DIFF_IWHITEALL: c_int = 0x010;
const DIFF_IWHITEEOL: c_int = 0x020;
const DIFF_INTERNAL: c_int = 0x200;

// =============================================================================
// External Diff Flags
// =============================================================================

/// Diff command flag settings.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffCmdFlags {
    /// Use -a flag (treat as ASCII)
    pub use_ascii: bool,
    /// Use -b flag (ignore whitespace changes)
    pub ignore_whitespace: bool,
    /// Use -w flag (ignore all whitespace)
    pub ignore_all_whitespace: bool,
    /// Use -Z flag (ignore whitespace at EOL)
    pub ignore_whitespace_eol: bool,
    /// Use -B flag (ignore blank lines)
    pub ignore_blank: bool,
    /// Use -i flag (ignore case)
    pub ignore_case: bool,
}

/// Convert diff_flags to external command flags.
#[no_mangle]
pub const extern "C" fn rs_diff_flags_to_cmd(diff_flags: c_int) -> DiffCmdFlags {
    DiffCmdFlags {
        use_ascii: true, // default on unless -a doesn't work
        ignore_whitespace: (diff_flags & DIFF_IWHITE) != 0,
        ignore_all_whitespace: (diff_flags & DIFF_IWHITEALL) != 0,
        ignore_whitespace_eol: (diff_flags & DIFF_IWHITEEOL) != 0,
        ignore_blank: (diff_flags & DIFF_IBLANK) != 0,
        ignore_case: (diff_flags & DIFF_ICASE) != 0,
    }
}

/// Calculate the required buffer size for diff command string.
///
/// Returns the size needed for: "diff [flags] <orig> <new>"
/// Does not include shell redirection.
#[no_mangle]
pub const extern "C" fn rs_diff_cmd_size(
    flags: &DiffCmdFlags,
    orig_len: usize,
    new_len: usize,
) -> usize {
    let mut size: usize = 5; // "diff "

    if flags.use_ascii {
        size += 3; // "-a "
    }
    if flags.ignore_whitespace {
        size += 3; // "-b "
    }
    if flags.ignore_all_whitespace {
        size += 3; // "-w "
    }
    if flags.ignore_whitespace_eol {
        size += 3; // "-Z "
    }
    if flags.ignore_blank {
        size += 3; // "-B "
    }
    if flags.ignore_case {
        size += 3; // "-i "
    }

    size += orig_len + 1; // orig filename + space
    size += new_len; // new filename
    size += 1; // null terminator

    size
}

/// Check if using internal diff (xdiff) based on flags and diffexpr.
#[no_mangle]
pub const extern "C" fn rs_diff_use_internal(diff_flags: c_int, has_diffexpr: bool) -> bool {
    !has_diffexpr && (diff_flags & DIFF_INTERNAL) != 0
}

// =============================================================================
// Diff Output Format Detection
// =============================================================================

/// Diff output format types.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffOutputFormat {
    /// Unknown/undetected format
    #[default]
    Unknown = 0,
    /// Normal diff format (1c1, 2,3d1, etc.)
    Normal = 1,
    /// Unified diff format (@@ -1 +1 @@)
    Unified = 2,
    /// Context diff format (*** 1,3 ****)
    Context = 3,
}

/// Result of parsing a diff output line.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffLineParseResult {
    /// Detected format
    pub format: DiffOutputFormat,
    /// Whether this is a hunk header line
    pub is_hunk_header: bool,
    /// For normal format: original start line
    pub orig_start: LinenrT,
    /// For normal format: original end line (or start if single line)
    pub orig_end: LinenrT,
    /// For normal format: change type ('a', 'c', 'd')
    pub change_type: c_char,
    /// For normal format: new start line
    pub new_start: LinenrT,
    /// For normal format: new end line (or start if single line)
    pub new_end: LinenrT,
    /// Whether parsing succeeded
    pub valid: bool,
}

/// Check if a line looks like a normal diff hunk (e.g., "1c1", "2,3d1", "1a2,4")
fn is_normal_diff_line(line: &[u8]) -> bool {
    if line.is_empty() {
        return false;
    }

    // Must start with a digit
    if !line[0].is_ascii_digit() {
        return false;
    }

    // Find the change character (a, c, or d)
    let mut found_change = false;
    for &c in line {
        if c == b'a' || c == b'c' || c == b'd' {
            found_change = true;
            break;
        }
        if !c.is_ascii_digit() && c != b',' {
            return false;
        }
    }

    found_change
}

/// Check if a line looks like a unified diff hunk header (e.g., "@@ -1 +1 @@")
fn is_unified_diff_line(line: &[u8]) -> bool {
    if line.len() < 11 {
        return false;
    }
    line.starts_with(b"@@ -")
}

/// Check if a line looks like a context diff hunk header (e.g., "*** 1,3 ****")
fn is_context_diff_line(line: &[u8]) -> bool {
    if line.len() < 10 {
        return false;
    }
    line.starts_with(b"*** ") && line.contains(&b'*')
}

/// Parse a normal diff line (e.g., "1c1", "2,3d1", "1a2,4")
fn parse_normal_diff_line(line: &[u8]) -> DiffLineParseResult {
    let mut result = DiffLineParseResult {
        format: DiffOutputFormat::Normal,
        is_hunk_header: true,
        ..Default::default()
    };

    let mut pos = 0;
    let len = line.len();

    // Parse original start
    let (orig_start, new_pos) = parse_number(line, pos);
    if new_pos == pos {
        return result;
    }
    result.orig_start = orig_start;
    result.orig_end = orig_start;
    pos = new_pos;

    // Check for comma (range)
    if pos < len && line[pos] == b',' {
        pos += 1;
        let (orig_end, new_pos) = parse_number(line, pos);
        if new_pos == pos {
            return result;
        }
        result.orig_end = orig_end;
        pos = new_pos;
    }

    // Parse change type
    if pos >= len {
        return result;
    }
    let change = line[pos];
    if change != b'a' && change != b'c' && change != b'd' {
        return result;
    }
    #[allow(clippy::cast_possible_wrap)]
    {
        result.change_type = change as c_char;
    }
    pos += 1;

    // Parse new start
    let (new_start, new_pos) = parse_number(line, pos);
    if new_pos == pos {
        return result;
    }
    result.new_start = new_start;
    result.new_end = new_start;
    pos = new_pos;

    // Check for comma (range)
    if pos < len && line[pos] == b',' {
        pos += 1;
        let (new_end, _) = parse_number(line, pos);
        result.new_end = new_end;
    }

    result.valid = true;
    result
}

/// Parse a number from a byte slice starting at position.
/// Returns (number, new_position).
fn parse_number(line: &[u8], start: usize) -> (LinenrT, usize) {
    let mut pos = start;
    let mut num: LinenrT = 0;

    while pos < line.len() && line[pos].is_ascii_digit() {
        num = num
            .saturating_mul(10)
            .saturating_add(LinenrT::from(line[pos] - b'0'));
        pos += 1;
    }

    (num, pos)
}

/// Detect the format of a diff output line.
///
/// # Safety
/// `line` must be a valid pointer to `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_detect_format(line: *const u8, len: usize) -> DiffOutputFormat {
    if line.is_null() || len == 0 {
        return DiffOutputFormat::Unknown;
    }

    let bytes = std::slice::from_raw_parts(line, len);

    if is_unified_diff_line(bytes) {
        DiffOutputFormat::Unified
    } else if is_normal_diff_line(bytes) {
        DiffOutputFormat::Normal
    } else if is_context_diff_line(bytes) {
        DiffOutputFormat::Context
    } else {
        DiffOutputFormat::Unknown
    }
}

/// Parse a normal diff hunk line.
///
/// # Safety
/// `line` must be a valid pointer to `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_parse_normal_line(
    line: *const u8,
    len: usize,
) -> DiffLineParseResult {
    if line.is_null() || len == 0 {
        return DiffLineParseResult::default();
    }

    let bytes = std::slice::from_raw_parts(line, len);

    if !is_normal_diff_line(bytes) {
        return DiffLineParseResult::default();
    }

    parse_normal_diff_line(bytes)
}

/// Check if a line is a valid diff verification line.
///
/// For checking if external diff works, we look for either:
/// - "1c1" (normal diff with change)
/// - "@@ -1 +1 @@" (unified diff header)
///
/// # Safety
/// `line` must be a valid pointer to `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_is_valid_test_output(line: *const u8, len: usize) -> bool {
    if line.is_null() || len < 3 {
        return false;
    }

    let bytes = std::slice::from_raw_parts(line, len);

    // Check for "1c1" (normal diff)
    if len >= 3 && bytes.starts_with(b"1c1") {
        return true;
    }

    // Check for "@@ -1 +1 @@" (unified diff)
    if len >= 11 && bytes.starts_with(b"@@ -1 +1 @@") {
        return true;
    }

    false
}

// =============================================================================
// Temp File State
// =============================================================================

/// State for external diff temp files.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffTempState {
    /// Whether original file has been written
    pub orig_written: bool,
    /// Whether new file has been written
    pub new_written: bool,
    /// Whether diff output exists
    pub diff_exists: bool,
    /// Whether an I/O error occurred
    pub io_error: bool,
}

/// Update temp file state after writing original file.
#[no_mangle]
pub const extern "C" fn rs_diff_temp_mark_orig_written(state: &mut DiffTempState, success: bool) {
    if success {
        state.orig_written = true;
    } else {
        state.io_error = true;
    }
}

/// Update temp file state after writing new file.
#[no_mangle]
pub const extern "C" fn rs_diff_temp_mark_new_written(state: &mut DiffTempState, success: bool) {
    if success {
        state.new_written = true;
    } else {
        state.io_error = true;
    }
}

/// Update temp file state after diff command completes.
#[no_mangle]
pub const extern "C" fn rs_diff_temp_mark_diff_done(state: &mut DiffTempState, success: bool) {
    if success {
        state.diff_exists = true;
    } else {
        state.io_error = true;
    }
}

/// Check if temp state is ready for diff execution.
#[no_mangle]
pub const extern "C" fn rs_diff_temp_ready_for_diff(state: &DiffTempState) -> bool {
    state.orig_written && state.new_written && !state.io_error
}

/// Check if temp state has valid diff output.
#[no_mangle]
pub const extern "C" fn rs_diff_temp_has_output(state: &DiffTempState) -> bool {
    state.diff_exists && !state.io_error
}

// =============================================================================
// External Diff Verification
// =============================================================================

/// State for verifying external diff functionality.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffVerifyState {
    /// Current verification attempt (0 = with -a, 1 = without -a)
    pub attempt: c_int,
    /// Whether -a flag works
    pub a_works: c_int, // -1 = unknown, 0 = no, 1 = yes
    /// Whether verification succeeded
    pub verified: bool,
    /// Whether I/O error occurred
    pub io_error: bool,
}

/// Initialize verification state.
#[no_mangle]
pub const extern "C" fn rs_diff_verify_init() -> DiffVerifyState {
    DiffVerifyState {
        attempt: 0,
        a_works: -1, // unknown
        verified: false,
        io_error: false,
    }
}

/// Check if we should try again without -a flag.
#[no_mangle]
pub const extern "C" fn rs_diff_verify_should_retry(state: &DiffVerifyState) -> bool {
    // Only retry if first attempt failed and we haven't determined -a status
    state.attempt == 0 && !state.verified && state.a_works == -1
}

/// Update verification state after an attempt.
#[no_mangle]
pub const extern "C" fn rs_diff_verify_update(
    state: &mut DiffVerifyState,
    success: bool,
    io_error: bool,
) {
    if io_error {
        state.io_error = true;
        return;
    }

    if success {
        state.verified = true;
        if state.attempt == 0 {
            state.a_works = 1; // -a works
        }
    } else if state.attempt == 0 {
        // First attempt failed, will retry without -a
        state.a_works = 0;
        state.attempt = 1;
    }
}

/// Check if verification is complete (either succeeded or definitively failed).
#[no_mangle]
pub const extern "C" fn rs_diff_verify_done(state: &DiffVerifyState) -> bool {
    state.verified || state.io_error || state.attempt > 0
}

/// Get the result of verification.
#[no_mangle]
pub const extern "C" fn rs_diff_verify_result(state: &DiffVerifyState) -> c_int {
    if state.verified {
        OK
    } else {
        FAIL
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_flags_to_cmd() {
        let flags = rs_diff_flags_to_cmd(0);
        assert!(flags.use_ascii);
        assert!(!flags.ignore_whitespace);
        assert!(!flags.ignore_all_whitespace);

        let flags = rs_diff_flags_to_cmd(DIFF_IWHITE | DIFF_ICASE);
        assert!(flags.ignore_whitespace);
        assert!(flags.ignore_case);
        assert!(!flags.ignore_all_whitespace);
    }

    #[test]
    fn test_diff_cmd_size() {
        let flags = DiffCmdFlags::default();
        // "diff " (5) + filename lens + space + null
        assert_eq!(rs_diff_cmd_size(&flags, 10, 10), 5 + 10 + 1 + 10 + 1);

        let flags_with_options = DiffCmdFlags {
            use_ascii: true,
            ignore_whitespace: true,
            ..Default::default()
        };
        // "diff " (5) + "-a " (3) + "-b " (3) + filenames
        assert_eq!(
            rs_diff_cmd_size(&flags_with_options, 10, 10),
            5 + 3 + 3 + 10 + 1 + 10 + 1
        );
    }

    #[test]
    fn test_diff_use_internal() {
        assert!(rs_diff_use_internal(DIFF_INTERNAL, false));
        assert!(!rs_diff_use_internal(DIFF_INTERNAL, true));
        assert!(!rs_diff_use_internal(0, false));
    }

    #[test]
    fn test_diff_output_format_detection() {
        assert!(is_normal_diff_line(b"1c1"));
        assert!(is_normal_diff_line(b"2,3d1"));
        assert!(is_normal_diff_line(b"1a2,4"));
        assert!(!is_normal_diff_line(b"@@ -1 +1 @@"));
        assert!(!is_normal_diff_line(b""));

        assert!(is_unified_diff_line(b"@@ -1 +1 @@"));
        assert!(is_unified_diff_line(b"@@ -1,3 +1,5 @@"));
        assert!(!is_unified_diff_line(b"1c1"));

        assert!(is_context_diff_line(b"*** 1,3 ****"));
        assert!(!is_context_diff_line(b"1c1"));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_parse_normal_diff_line() {
        let result = parse_normal_diff_line(b"1c1");
        assert!(result.valid);
        assert_eq!(result.orig_start, 1);
        assert_eq!(result.orig_end, 1);
        assert_eq!(result.change_type, b'c' as c_char);
        assert_eq!(result.new_start, 1);
        assert_eq!(result.new_end, 1);

        let result = parse_normal_diff_line(b"2,5d1");
        assert!(result.valid);
        assert_eq!(result.orig_start, 2);
        assert_eq!(result.orig_end, 5);
        assert_eq!(result.change_type, b'd' as c_char);
        assert_eq!(result.new_start, 1);
        assert_eq!(result.new_end, 1);

        let result = parse_normal_diff_line(b"1a2,4");
        assert!(result.valid);
        assert_eq!(result.orig_start, 1);
        assert_eq!(result.orig_end, 1);
        assert_eq!(result.change_type, b'a' as c_char);
        assert_eq!(result.new_start, 2);
        assert_eq!(result.new_end, 4);
    }

    #[test]
    fn test_diff_temp_state() {
        let mut state = DiffTempState::default();
        assert!(!rs_diff_temp_ready_for_diff(&state));

        rs_diff_temp_mark_orig_written(&mut state, true);
        assert!(!rs_diff_temp_ready_for_diff(&state));

        rs_diff_temp_mark_new_written(&mut state, true);
        assert!(rs_diff_temp_ready_for_diff(&state));

        rs_diff_temp_mark_diff_done(&mut state, true);
        assert!(rs_diff_temp_has_output(&state));
    }

    #[test]
    fn test_diff_temp_state_error() {
        let mut state = DiffTempState::default();
        rs_diff_temp_mark_orig_written(&mut state, false);
        assert!(!rs_diff_temp_ready_for_diff(&state));
        assert!(state.io_error);
    }

    #[test]
    fn test_diff_verify_state() {
        let mut state = rs_diff_verify_init();
        assert!(!rs_diff_verify_done(&state));

        // First attempt succeeds
        rs_diff_verify_update(&mut state, true, false);
        assert!(rs_diff_verify_done(&state));
        assert_eq!(rs_diff_verify_result(&state), OK);
        assert_eq!(state.a_works, 1);
    }

    #[test]
    fn test_diff_verify_retry() {
        let mut state = rs_diff_verify_init();

        // First attempt fails
        rs_diff_verify_update(&mut state, false, false);
        assert!(!rs_diff_verify_done(&state));
        assert!(rs_diff_verify_should_retry(&state));
        assert_eq!(state.a_works, 0);

        // Second attempt succeeds
        rs_diff_verify_update(&mut state, true, false);
        assert!(rs_diff_verify_done(&state));
        assert_eq!(rs_diff_verify_result(&state), OK);
    }

    #[test]
    fn test_diff_verify_io_error() {
        let mut state = rs_diff_verify_init();
        rs_diff_verify_update(&mut state, false, true);
        assert!(rs_diff_verify_done(&state));
        assert_eq!(rs_diff_verify_result(&state), FAIL);
    }

    #[test]
    fn test_is_valid_test_output() {
        unsafe {
            let line1 = b"1c1";
            assert!(rs_diff_is_valid_test_output(line1.as_ptr(), line1.len()));

            let line2 = b"@@ -1 +1 @@";
            assert!(rs_diff_is_valid_test_output(line2.as_ptr(), line2.len()));

            let line3 = b"2c2";
            assert!(!rs_diff_is_valid_test_output(line3.as_ptr(), line3.len()));

            let line4 = b"random text";
            assert!(!rs_diff_is_valid_test_output(line4.as_ptr(), line4.len()));
        }
    }
}
