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
    type FieldsHandle = *mut c_void;
    type EfmHandle = *mut c_void;

    const QF_OK: c_int = 1;
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

        // Globals
        fn nvim_get_got_int() -> c_int;
        fn nvim_set_got_int(val: c_int);
        fn nvim_line_breakcheck();

        // C-side helpers
        // nvim_qf_init_clear_last_bufname removed: use nvim_qf_clear_fnum_cache (Phase 16)
        fn nvim_qf_clear_fnum_cache();
        // nvim_qf_init_resolve_efm removed: logic inlined into rs_qf_init_ext (Phase 16)
        fn nvim_get_p_efm() -> *const c_char;
        fn nvim_buf_get_b_p_efm(buf: BufHandle) -> *const c_char;

        // Phase 16: rs_qf_init accessors
        fn nvim_get_ql_info() -> QfInfoHandleMut;
        fn rs_ll_get_or_alloc_list(wp: *mut c_void) -> QfInfoHandleMut;
        fn nvim_curbuf_ptr() -> BufHandle;

        // nvim_qf_init_finalize_list deleted: inlined below (Phase 14)
        fn nvim_qf_get_index(qfl: QfListHandle) -> c_int;
        fn nvim_qf_set_index(qfl: QfListHandleMut, idx: c_int);
        fn nvim_qf_set_ptr(qfl: QfListHandleMut, ptr: QfLineHandle);
        fn nvim_qf_get_start(qfl: QfListHandle) -> QfLineHandle;
        fn nvim_qf_set_nonevalid(qfl: QfListHandleMut, nonevalid: bool);
        fn nvim_qf_init_emsg_readerrf();
        fn nvim_qf_decrement_listcount(qi: QfInfoHandleMut);
        fn nvim_qf_set_curlist_idx(qi: QfInfoHandleMut, idx: c_int);

        // qf_list_T directory/currfile accessors
        fn nvim_qf_get_directory(qfl: *const c_void) -> *const c_char;
        fn nvim_qf_get_currfile(qfl: *const c_void) -> *const c_char;

        // rs_qf_parse_line (already in Rust parse.rs, callable via extern "C")
        fn rs_qf_parse_line(
            qfl: *mut c_void,
            linebuf: *mut c_char,
            linelen: usize,
            fmt_first: EfmHandle,
            fields: *mut c_void,
        ) -> c_int;

        // rs_qf_add_entry (already in Rust lib.rs, callable via extern "C")
        fn rs_qf_add_entry(
            qfl: QfListHandleMut,
            dir: *mut c_char,
            fname: *const c_char,
            module: *const c_char,
            bufnum: c_int,
            mesg: *const c_char,
            lnum: LinenrT,
            end_lnum: LinenrT,
            col: c_int,
            end_col: c_int,
            vis_col: c_char,
            pattern: *const c_char,
            nr: c_int,
            type_char: c_char,
            user_data: *const c_void,
            valid: c_char,
        ) -> c_int;
    }

    /// Process one line: read next line via Rust parser state, parse it, add entry.
    ///
    /// This replaces the C `nvim_qf_init_process_nextline` wrapper.
    ///
    /// # Safety
    ///
    /// All pointer parameters must be valid.
    #[allow(clippy::too_many_arguments)]
    unsafe fn process_nextline(
        qfl: QfListHandleMut,
        fmt_first: EfmHandle,
        state: *mut crate::reader::QfParserState,
        fields: FieldsHandle,
    ) -> c_int {
        // Get the next line from the source
        let status = (*state).get_nextline();
        if status != QF_OK {
            return status;
        }

        let linebuf = (*state).linebuf;
        let linelen = (*state).linelen;

        // Parse the line against errorformat patterns
        let parse_status = rs_qf_parse_line(qfl, linebuf, linelen, fmt_first, fields);
        if parse_status != QF_OK {
            return parse_status;
        }

        // Build the rs_qf_add_entry arguments using direct Rust struct field access
        let f = &*fields.cast::<crate::reader::QfAllFields>();
        let dir = nvim_qf_get_directory(qfl.cast_const()).cast_mut();
        let currfile = nvim_qf_get_currfile(qfl.cast_const());
        let valid = f.valid;

        // fname selection matches C logic:
        //   if (*namebuf || dir != NULL) use namebuf
        //   elif (currfile != NULL && valid) use currfile
        //   else use NULL
        let fname: *const c_char = if !f.namebuf.is_null() && (*f.namebuf != 0 || !dir.is_null()) {
            f.namebuf.cast_const()
        } else if !currfile.is_null() && valid {
            currfile
        } else {
            std::ptr::null()
        };

        rs_qf_add_entry(
            qfl,
            dir,
            fname,
            f.module.cast_const(),
            f.bnr,
            f.errmsg.cast_const(),
            f.lnum,
            f.end_lnum,
            f.col,
            f.end_col,
            f.use_viscol as c_char,
            f.pattern.cast_const(),
            f.enr,
            f.type_char,
            std::ptr::null(), // user_data: always null in Rust-managed path
            valid as c_char,
        )
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
        // nvim_qf_init_clear_last_bufname removed: use nvim_qf_clear_fnum_cache (Phase 16)
        nvim_qf_clear_fnum_cache();

        // Allocate fields in Rust (Phase 2)
        let fields = crate::reader::rs_qf_alloc_fields();

        // Setup parser state in Rust (replaces nvim_qf_init_setup_state)
        let state_ptr =
            crate::reader::rs_qf_parser_state_new(enc, efile, tv, buf, lnumfirst, lnumlast);
        let state: *mut crate::reader::QfParserState = state_ptr.cast();

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
                crate::rs_qf_new_list(qi, qf_title);
                qf_idx = nvim_qf_get_curlist_idx(qi);
                qfl = nvim_qf_get_list_at_mut(qi, qf_idx);
            } else {
                // Adding to existing list, use last entry.
                adding = true;
                qfl = nvim_qf_get_list_at_mut(qi, qf_idx);
                if !crate::rs_qf_list_empty(qfl) {
                    old_last = nvim_qf_get_last(qfl);
                }
            }

            // Resolve the effective errorformat.
            // Inlined from nvim_qf_init_resolve_efm (Phase 16):
            // If errorformat is the global p_efm, tv is NULL, and buf has its own efm, use buf's.
            let efm = {
                let p_efm = nvim_get_p_efm();
                if errorformat.cast_const() == p_efm && tv.is_null() && !buf.is_null() {
                    let b_p_efm = nvim_buf_get_b_p_efm(buf);
                    if !b_p_efm.is_null() && *b_p_efm != 0 {
                        b_p_efm.cast_mut()
                    } else {
                        errorformat
                    }
                } else {
                    errorformat
                }
            };

            // Update the cached efm parsing (Phase 2: Rust-owned cache)
            let fmt_first = crate::reader::rs_qf_init_update_efm_cache(efm);
            if fmt_first.is_null() {
                break 'init -1; // error2 path
            }

            // got_int is reset here, because it was probably set when killing
            // the ":make" command, but we still want to read the errorfile then.
            nvim_set_got_int(0);

            // Read the lines in the error file one by one.
            // Try to recognize one of the error formats in each line.
            while nvim_get_got_int() == 0 {
                // Use Rust parser state directly (replaces nvim_qf_init_process_nextline)
                let status = process_nextline(qfl, fmt_first, state, fields);
                if status == QF_END_OF_INPUT {
                    break;
                }
                if status == QF_FAIL {
                    break 'init -1; // error2 path
                }
                nvim_line_breakcheck();
            }

            // Check if file source had a read error (replaces nvim_qf_init_state_no_fd_error)
            if (*state).no_fd_error() {
                // Inlined nvim_qf_init_finalize_list (Phase 14):
                // Set qf_ptr/qf_index/qf_nonevalid based on whether valid entries exist.
                if nvim_qf_get_index(qfl) == 0 {
                    // no valid entry found
                    nvim_qf_set_ptr(qfl, nvim_qf_get_start(qfl));
                    nvim_qf_set_index(qfl, 1);
                    nvim_qf_set_nonevalid(qfl, true);
                } else {
                    nvim_qf_set_nonevalid(qfl, false);
                    // qf_ptr already set to the first valid entry by rs_qf_add_entry
                }
                nvim_qf_get_count(qfl) // success: return number of matches
            } else {
                nvim_qf_init_emsg_readerrf();
                -1 // error2 path
            }
        };

        // error2: free the new list on error if we weren't appending
        if retval == -1 && !adding && !qfl.is_null() {
            // Inline error cleanup: free the list, decrement listcount/curlist
            crate::rs_qf_free_list(qfl);
            nvim_qf_decrement_listcount(qi);
            let cur = nvim_qf_get_curlist_idx(qi);
            if cur > 0 {
                nvim_qf_set_curlist_idx(qi, cur - 1);
            }
        }

        // qf_init_end: always-run cleanup
        if qf_idx == nvim_qf_get_curlist_idx(qi) {
            nvim_qf_update_buffer(qi, old_last);
        }
        // Free Rust parser state and fields (Phase 2: both Rust-owned)
        crate::reader::rs_qf_parser_state_free(state_ptr);
        crate::reader::rs_qf_free_fields(fields);

        retval
    }

    /// Entry point for C `qf_init`: resolve qi and call `rs_qf_init_ext`.
    ///
    /// Replaces the C `qf_init` body. The C function becomes a thin wrapper.
    ///
    /// # Safety
    ///
    /// - `wp` may be NULL (uses global quickfix stack) or a valid `win_T *`
    /// - Other pointers follow the same contract as `rs_qf_init_ext`
    #[no_mangle]
    #[allow(clippy::too_many_arguments)]
    pub unsafe extern "C" fn rs_qf_init(
        wp: *mut c_void,
        efile: *const c_char,
        errorformat: *mut c_char,
        newlist: bool,
        qf_title: *const c_char,
        enc: *mut c_char,
    ) -> c_int {
        let qi = if wp.is_null() {
            nvim_get_ql_info()
        } else {
            rs_ll_get_or_alloc_list(wp)
        };
        assert!(!qi.is_null());
        let curlist = nvim_qf_get_curlist_idx(qi);
        let buf = nvim_curbuf_ptr();
        rs_qf_init_ext(
            qi,
            curlist,
            efile,
            buf,
            std::ptr::null_mut(),
            errorformat,
            newlist,
            0,
            0,
            qf_title,
            enc,
        )
    }
}

// =============================================================================
// Phase 16: qf_stack_get_bufnr migration
// =============================================================================

extern "C" {
    fn nvim_get_ql_info() -> *mut std::ffi::c_void;
    fn nvim_qf_get_bufnr(qi: *const std::ffi::c_void) -> std::ffi::c_int;
}

/// Returns the quickfix buffer number for the global quickfix stack.
///
/// Replaces C `qf_stack_get_bufnr` (Phase 16).
///
/// # Safety
///
/// `nvim_get_ql_info()` must return a non-null pointer.
///
/// # Panics
///
/// Panics if `nvim_get_ql_info()` returns null (indicates uninitialized quickfix stack).
#[export_name = "qf_stack_get_bufnr"]
pub unsafe extern "C" fn rs_qf_stack_get_bufnr() -> std::ffi::c_int {
    let qi = nvim_get_ql_info();
    assert!(!qi.is_null());
    nvim_qf_get_bufnr(qi)
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
