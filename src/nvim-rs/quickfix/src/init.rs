//! Quickfix initialization helpers.
//!
//! This module provides helpers for initializing quickfix parsing state.

#![allow(clippy::cast_lossless)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_char, c_int};

/// Input source type for quickfix initialization.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QfInputSource {
    /// No input source (invalid state)
    #[default]
    None,
    /// Reading from a file
    File,
    /// Reading from a buffer
    Buffer,
    /// Reading from a string
    String,
    /// Reading from a list
    List,
}

/// Result of validating input source.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct QfInputValidation {
    /// Whether the input source is valid
    pub valid: bool,
    /// The detected input source type
    pub source: QfInputSource,
    /// Whether the input is empty
    pub is_empty: bool,
}

/// Validate and classify input source for quickfix initialization.
///
/// Determines the input source type based on provided parameters:
/// - `has_efile`: Whether an error file path was provided
/// - `has_buffer`: Whether a buffer was provided
/// - `has_string`: Whether a string was provided
/// - `has_list`: Whether a list was provided
#[no_mangle]
pub extern "C" fn rs_qf_validate_input_source(
    has_efile: bool,
    has_buffer: bool,
    has_string: bool,
    has_list: bool,
) -> QfInputValidation {
    // Only one source should be active
    let source_count = has_efile as u8 + has_buffer as u8 + has_string as u8 + has_list as u8;

    if source_count == 0 {
        return QfInputValidation {
            valid: false,
            source: QfInputSource::None,
            is_empty: true,
        };
    }

    if source_count > 1 {
        // Multiple sources - ambiguous, but we prioritize in order
        // This matches C behavior where first valid source wins
    }

    let source = if has_efile {
        QfInputSource::File
    } else if has_buffer {
        QfInputSource::Buffer
    } else if has_string {
        QfInputSource::String
    } else if has_list {
        QfInputSource::List
    } else {
        QfInputSource::None
    };

    QfInputValidation {
        valid: source != QfInputSource::None,
        source,
        is_empty: false,
    }
}

/// Get the input source type.
#[no_mangle]
pub extern "C" fn rs_qf_input_source_type(validation: &QfInputValidation) -> c_int {
    validation.source as c_int
}

/// Check if input source is from a file.
#[no_mangle]
pub extern "C" fn rs_qf_input_is_file(source: QfInputSource) -> bool {
    source == QfInputSource::File
}

/// Check if input source is from a buffer.
#[no_mangle]
pub extern "C" fn rs_qf_input_is_buffer(source: QfInputSource) -> bool {
    source == QfInputSource::Buffer
}

/// Check if input source is from a string.
#[no_mangle]
pub extern "C" fn rs_qf_input_is_string(source: QfInputSource) -> bool {
    source == QfInputSource::String
}

/// Check if input source is from a list.
#[no_mangle]
pub extern "C" fn rs_qf_input_is_list(source: QfInputSource) -> bool {
    source == QfInputSource::List
}

// =============================================================================
// Line Reading State
// =============================================================================

/// State for reading lines during quickfix initialization.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct QfReadState {
    /// Current line number (1-based)
    pub line_nr: c_int,
    /// Total lines processed
    pub lines_processed: c_int,
    /// Lines that matched an errorformat
    pub lines_matched: c_int,
    /// Lines that didn't match (informational)
    pub lines_unmatched: c_int,
    /// Whether we've reached end of input
    pub at_end: bool,
    /// Whether an error occurred
    pub had_error: bool,
}

impl QfReadState {
    /// Create a new read state.
    pub const fn new() -> Self {
        Self {
            line_nr: 0,
            lines_processed: 0,
            lines_matched: 0,
            lines_unmatched: 0,
            at_end: false,
            had_error: false,
        }
    }

    /// Advance to next line.
    pub fn advance(&mut self) {
        self.line_nr += 1;
        self.lines_processed += 1;
    }

    /// Record a matched line.
    pub fn record_match(&mut self) {
        self.lines_matched += 1;
    }

    /// Record an unmatched line.
    pub fn record_nomatch(&mut self) {
        self.lines_unmatched += 1;
    }

    /// Mark as ended.
    pub fn mark_end(&mut self) {
        self.at_end = true;
    }

    /// Mark as errored.
    pub fn mark_error(&mut self) {
        self.had_error = true;
    }
}

/// Create a new read state.
#[no_mangle]
pub extern "C" fn rs_qf_read_state_new() -> QfReadState {
    QfReadState::new()
}

/// Advance read state to next line.
#[no_mangle]
pub extern "C" fn rs_qf_read_state_advance(state: &mut QfReadState) {
    state.advance();
}

/// Record a matched line in read state.
#[no_mangle]
pub extern "C" fn rs_qf_read_state_match(state: &mut QfReadState) {
    state.record_match();
}

/// Record an unmatched line in read state.
#[no_mangle]
pub extern "C" fn rs_qf_read_state_nomatch(state: &mut QfReadState) {
    state.record_nomatch();
}

/// Mark read state as ended.
#[no_mangle]
pub extern "C" fn rs_qf_read_state_end(state: &mut QfReadState) {
    state.mark_end();
}

/// Mark read state as errored.
#[no_mangle]
pub extern "C" fn rs_qf_read_state_error(state: &mut QfReadState) {
    state.mark_error();
}

/// Check if read state has more input.
#[no_mangle]
pub extern "C" fn rs_qf_read_state_has_more(state: &QfReadState) -> bool {
    !state.at_end && !state.had_error
}

// =============================================================================
// Initialization Options
// =============================================================================

/// Options for quickfix initialization.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct QfInitOptions {
    /// Whether to use the current quickfix list
    pub use_curlist: bool,
    /// Whether to append to existing entries
    pub append: bool,
    /// Whether to create a new list
    pub new_list: bool,
    /// Starting line number for buffer reading
    pub lnum_first: c_int,
    /// Ending line number for buffer reading
    pub lnum_last: c_int,
}

/// Create default init options.
#[no_mangle]
pub extern "C" fn rs_qf_init_options_new() -> QfInitOptions {
    QfInitOptions::default()
}

/// Parse action character into init options.
///
/// Action characters:
/// - ' ', 'a', or 'f': append to current list
/// - 'r': replace current list
/// - otherwise: create new list
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_qf_init_options_from_action(action: c_char) -> QfInitOptions {
    let action_byte = action as u8;
    match action_byte {
        b' ' | b'a' | b'f' => QfInitOptions {
            use_curlist: true,
            append: true,
            new_list: false,
            lnum_first: 0,
            lnum_last: 0,
        },
        b'r' => QfInitOptions {
            use_curlist: true,
            append: false,
            new_list: false,
            lnum_first: 0,
            lnum_last: 0,
        },
        _ => QfInitOptions {
            use_curlist: false,
            append: false,
            new_list: true,
            lnum_first: 0,
            lnum_last: 0,
        },
    }
}

/// Check if init options require appending.
#[no_mangle]
pub extern "C" fn rs_qf_init_should_append(options: &QfInitOptions) -> bool {
    options.append
}

/// Check if init options require new list.
#[no_mangle]
pub extern "C" fn rs_qf_init_should_create_list(options: &QfInitOptions) -> bool {
    options.new_list
}

// =============================================================================
// qf_init_ext migration
// =============================================================================

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
mod init_ext {
    use std::ffi::{c_char, c_int, c_void};

    type LinenrT = i32;

    /// Opaque handles — must match signatures in lib.rs
    type QfInfoHandle = *const c_void;
    type QfInfoHandleMut = *mut c_void;
    type QfListHandle = *const c_void;
    type QfListHandleMut = *mut c_void;
    type QfLineHandle = *const c_void;
    type BufHandle = *mut c_void;
    type TvHandle = *mut c_void;
    type StateHandle = *mut c_void;
    type FieldsHandle = *mut c_void;
    type EfmHandle = *mut c_void;

    const QF_END_OF_INPUT: c_int = 2;
    const QF_FAIL: c_int = 0;

    extern "C" {
        // Existing accessors (signatures must match lib.rs)
        fn nvim_qf_get_listcount(qi: QfInfoHandle) -> c_int;
        fn nvim_qf_get_curlist_idx(qi: QfInfoHandle) -> c_int;
        fn nvim_qf_get_list_at_mut(qi: QfInfoHandleMut, idx: c_int) -> QfListHandleMut;
        fn nvim_qf_get_count(qfl: QfListHandle) -> c_int;
        fn nvim_qf_get_last(qfl: QfListHandle) -> QfLineHandle;
        fn nvim_qf_update_buffer(qi: QfInfoHandleMut, old_last: QfLineHandle);

        // Existing Rust functions called via FFI
        fn rs_qf_new_list(qi: QfInfoHandleMut, title: *const c_char);
        fn rs_qf_list_empty(qfl: *const c_void) -> bool;

        // Globals
        fn nvim_get_got_int() -> c_int;
        fn nvim_set_got_int(val: c_int);
        fn nvim_line_breakcheck();

        // New Phase 4 accessors
        fn nvim_qf_init_alloc_fields() -> FieldsHandle;
        fn nvim_qf_init_free_fields(fields: FieldsHandle);
        fn nvim_qf_init_setup_state(
            enc: *mut c_char,
            efile: *const c_char,
            tv: TvHandle,
            buf: BufHandle,
            lnumfirst: LinenrT,
            lnumlast: LinenrT,
        ) -> StateHandle;
        fn nvim_qf_init_cleanup_state(state: StateHandle);
        fn nvim_qf_init_clear_last_bufname();
        fn nvim_qf_init_resolve_efm(
            errorformat: *mut c_char,
            tv: TvHandle,
            buf: BufHandle,
        ) -> *mut c_char;
        fn nvim_qf_init_update_efm_cache(efm: *mut c_char) -> EfmHandle;
        fn nvim_qf_init_process_nextline(
            qfl: QfListHandleMut,
            fmt_first: EfmHandle,
            state: StateHandle,
            fields: FieldsHandle,
        ) -> c_int;
        fn nvim_qf_init_state_no_fd_error(state: StateHandle) -> bool;
        fn nvim_qf_init_finalize_list(qfl: QfListHandleMut);
        fn nvim_qf_init_emsg_readerrf();
        fn nvim_qf_init_error_cleanup(qi: QfInfoHandleMut, qfl: QfListHandleMut);
    }

    /// Initialize quickfix list from error file/buffer/string/list.
    ///
    /// # Safety
    ///
    /// All pointer parameters must be valid or NULL as documented.
    #[no_mangle]
    #[allow(clippy::too_many_arguments)]
    pub unsafe extern "C" fn rs_qf_init_ext(
        qi: QfInfoHandleMut,
        mut qf_idx: c_int,
        efile: *const c_char,
        buf: BufHandle,
        tv: TvHandle,
        errorformat: *mut c_char,
        newlist: bool,
        lnumfirst: LinenrT,
        lnumlast: LinenrT,
        qf_title: *const c_char,
        enc: *mut c_char,
    ) -> c_int {
        // Do not use the cached buffer, it may have been wiped out.
        nvim_qf_init_clear_last_bufname();

        let fields = nvim_qf_init_alloc_fields();
        let state = nvim_qf_init_setup_state(enc, efile, tv, buf, lnumfirst, lnumlast);

        // Tracks whether we need to run the error2 cleanup path.
        let mut adding = false;
        let mut qfl: QfListHandleMut = std::ptr::null_mut();
        let mut old_last: QfLineHandle = std::ptr::null();

        // Main logic: labeled block replaces C goto error2/qf_init_end.
        let retval = 'init: {
            if state.is_null() {
                break 'init -1;
            }

            if newlist || qf_idx == nvim_qf_get_listcount(qi) {
                // Make place for a new list
                rs_qf_new_list(qi, qf_title);
                qf_idx = nvim_qf_get_curlist_idx(qi);
                qfl = nvim_qf_get_list_at_mut(qi, qf_idx);
            } else {
                // Adding to existing list, use last entry.
                adding = true;
                qfl = nvim_qf_get_list_at_mut(qi, qf_idx);
                if !rs_qf_list_empty(qfl) {
                    old_last = nvim_qf_get_last(qfl);
                }
            }

            // Resolve the effective errorformat.
            let efm = nvim_qf_init_resolve_efm(errorformat, tv, buf);

            // Update the cached efm parsing.
            let fmt_first = nvim_qf_init_update_efm_cache(efm);
            if fmt_first.is_null() {
                break 'init -1; // error2 path
            }

            // got_int is reset here, because it was probably set when killing
            // the ":make" command, but we still want to read the errorfile then.
            nvim_set_got_int(0);

            // Read the lines in the error file one by one.
            // Try to recognize one of the error formats in each line.
            while nvim_get_got_int() == 0 {
                let status = nvim_qf_init_process_nextline(qfl, fmt_first, state, fields);
                if status == QF_END_OF_INPUT {
                    break;
                }
                if status == QF_FAIL {
                    break 'init -1; // error2 path
                }
                nvim_line_breakcheck();
            }

            if nvim_qf_init_state_no_fd_error(state) {
                nvim_qf_init_finalize_list(qfl);
                nvim_qf_get_count(qfl) // success: return number of matches
            } else {
                nvim_qf_init_emsg_readerrf();
                -1 // error2 path
            }
        };

        // error2: free the new list on error if we weren't appending
        if retval == -1 && !adding && !qfl.is_null() {
            nvim_qf_init_error_cleanup(qi, qfl);
        }

        // qf_init_end: always-run cleanup
        if qf_idx == nvim_qf_get_curlist_idx(qi) {
            nvim_qf_update_buffer(qi, old_last);
        }
        nvim_qf_init_cleanup_state(state);
        nvim_qf_init_free_fields(fields);

        retval
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_input_source_file() {
        let result = rs_qf_validate_input_source(true, false, false, false);
        assert!(result.valid);
        assert_eq!(result.source, QfInputSource::File);
    }

    #[test]
    fn test_validate_input_source_buffer() {
        let result = rs_qf_validate_input_source(false, true, false, false);
        assert!(result.valid);
        assert_eq!(result.source, QfInputSource::Buffer);
    }

    #[test]
    fn test_validate_input_source_string() {
        let result = rs_qf_validate_input_source(false, false, true, false);
        assert!(result.valid);
        assert_eq!(result.source, QfInputSource::String);
    }

    #[test]
    fn test_validate_input_source_list() {
        let result = rs_qf_validate_input_source(false, false, false, true);
        assert!(result.valid);
        assert_eq!(result.source, QfInputSource::List);
    }

    #[test]
    fn test_validate_input_source_none() {
        let result = rs_qf_validate_input_source(false, false, false, false);
        assert!(!result.valid);
        assert!(result.is_empty);
        assert_eq!(result.source, QfInputSource::None);
    }

    #[test]
    fn test_input_source_checks() {
        assert!(rs_qf_input_is_file(QfInputSource::File));
        assert!(!rs_qf_input_is_file(QfInputSource::Buffer));
        assert!(rs_qf_input_is_buffer(QfInputSource::Buffer));
        assert!(rs_qf_input_is_string(QfInputSource::String));
        assert!(rs_qf_input_is_list(QfInputSource::List));
    }

    #[test]
    fn test_read_state() {
        let mut state = rs_qf_read_state_new();
        assert_eq!(state.line_nr, 0);
        assert!(rs_qf_read_state_has_more(&state));

        rs_qf_read_state_advance(&mut state);
        assert_eq!(state.line_nr, 1);
        assert_eq!(state.lines_processed, 1);

        rs_qf_read_state_match(&mut state);
        assert_eq!(state.lines_matched, 1);

        rs_qf_read_state_nomatch(&mut state);
        assert_eq!(state.lines_unmatched, 1);

        rs_qf_read_state_end(&mut state);
        assert!(!rs_qf_read_state_has_more(&state));
    }

    #[test]
    fn test_read_state_error() {
        let mut state = rs_qf_read_state_new();
        assert!(rs_qf_read_state_has_more(&state));

        rs_qf_read_state_error(&mut state);
        assert!(!rs_qf_read_state_has_more(&state));
        assert!(state.had_error);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_init_options_from_action() {
        let opts = rs_qf_init_options_from_action(b' ' as c_char);
        assert!(opts.use_curlist);
        assert!(opts.append);
        assert!(!opts.new_list);

        let opts = rs_qf_init_options_from_action(b'a' as c_char);
        assert!(opts.append);

        let opts = rs_qf_init_options_from_action(b'r' as c_char);
        assert!(opts.use_curlist);
        assert!(!opts.append);

        let opts = rs_qf_init_options_from_action(0);
        assert!(!opts.use_curlist);
        assert!(opts.new_list);
    }
}
