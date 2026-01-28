//! Tag jump orchestration for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations for tag jump preparation,
//! validation, and state management.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ptr_as_ptr)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

/// Return value indicating success
const OK: c_int = 1;
/// Return value indicating failure
const FAIL: c_int = 0;
/// Return value indicating file not found
const NOTAGFILE: c_int = 2;

/// Maximum line size for tag patterns
const LSIZE: usize = 512;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Memory functions
    fn xfree(ptr: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_void;

    // String functions
    fn strlen(s: *const c_char) -> usize;
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;

    // File functions
    fn nvim_tag_path_exists(path: *const c_char) -> bool;
    fn nvim_has_bufreadcmd(fname: *const c_char) -> bool;

    // Tag filename expansion
    fn nvim_expand_tag_fname(
        fname: *const c_char,
        tag_fname: *const c_char,
        expand: bool,
    ) -> *mut c_char;

    // Global state accessors
    fn nvim_get_postponed_split() -> c_int;
    fn nvim_set_postponed_split(val: c_int);
    fn nvim_get_g_do_tagpreview() -> c_int;
    fn nvim_set_g_do_tagpreview(val: c_int);

    // Window functions
    fn nvim_check_can_set_curbuf_forceit(forceit: c_int) -> bool;

    // Error reporting
    fn nvim_set_nofile_fname(fname: *const c_char);
}

// =============================================================================
// Jump state structure
// =============================================================================

/// State for a tag jump operation.
///
/// This structure holds all the intermediate state needed during a tag jump,
/// allowing the operation to be split into prepare/execute/cleanup phases.
#[repr(C)]
pub struct JumpTagState {
    /// Copy of the match line buffer
    pub lbuf: *mut c_char,
    /// Length of lbuf (excluding null terminator)
    pub lbuf_len: usize,
    /// Search pattern buffer
    pub pbuf: *mut c_char,
    /// End of pattern in pbuf
    pub pbuf_end: *mut c_char,
    /// Expanded file name (may be allocated)
    pub expanded_fname: *mut c_char,
    /// Original file name pointer (into lbuf)
    pub orig_fname: *const c_char,
    /// Tag file name
    pub tag_fname: *const c_char,
    /// Force flag
    pub forceit: c_int,
    /// Keep help flag
    pub keep_help: bool,
    /// Whether file exists
    pub file_exists: bool,
    /// Current state (for phased execution)
    pub phase: c_int,
    /// Result code
    pub result: c_int,
}

impl Default for JumpTagState {
    fn default() -> Self {
        Self {
            lbuf: ptr::null_mut(),
            lbuf_len: 0,
            pbuf: ptr::null_mut(),
            pbuf_end: ptr::null_mut(),
            expanded_fname: ptr::null_mut(),
            orig_fname: ptr::null(),
            tag_fname: ptr::null(),
            forceit: 0,
            keep_help: false,
            file_exists: false,
            phase: 0,
            result: 0,
        }
    }
}

/// Jump phases
pub mod phase {
    use std::ffi::c_int;

    /// Initial state
    pub const INIT: c_int = 0;
    /// Prepared (buffers allocated, parsed)
    pub const PREPARED: c_int = 1;
    /// File validated
    pub const VALIDATED: c_int = 2;
    /// Ready to execute
    pub const READY: c_int = 3;
    /// Execution complete
    pub const DONE: c_int = 4;
    /// Error state
    pub const ERROR: c_int = -1;
}

// =============================================================================
// State lifecycle
// =============================================================================

/// Create a new jump state.
#[no_mangle]
pub extern "C" fn rs_jump_tag_state_new() -> *mut JumpTagState {
    Box::into_raw(Box::new(JumpTagState::default()))
}

/// Free a jump state and all its resources.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_state_free(state: *mut JumpTagState) {
    if state.is_null() {
        return;
    }

    let state = &mut *state;
    rs_jump_tag_cleanup(state);
    drop(Box::from_raw(state));
}

/// Clean up resources in a jump state (but don't free the state itself).
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_cleanup(state: *mut JumpTagState) {
    if state.is_null() {
        return;
    }

    let state = &mut *state;

    if !state.lbuf.is_null() {
        xfree(state.lbuf as *mut c_void);
        state.lbuf = ptr::null_mut();
    }

    if !state.pbuf.is_null() {
        xfree(state.pbuf as *mut c_void);
        state.pbuf = ptr::null_mut();
    }

    if !state.expanded_fname.is_null() {
        xfree(state.expanded_fname as *mut c_void);
        state.expanded_fname = ptr::null_mut();
    }

    state.pbuf_end = ptr::null_mut();
    state.orig_fname = ptr::null();
    state.tag_fname = ptr::null();
    state.lbuf_len = 0;
    state.phase = phase::INIT;
}

// =============================================================================
// Preparation phase
// =============================================================================

/// Prepare for a tag jump by parsing the match line.
///
/// This allocates buffers and parses the match line into components.
///
/// # Arguments
///
/// * `state` - Jump state to initialize
/// * `lbuf_arg` - Match line buffer
/// * `forceit` - Force flag for buffer switching
/// * `keep_help` - Whether to keep help flag
///
/// # Returns
///
/// OK on success, FAIL on error.
#[no_mangle]
pub unsafe extern "C" fn rs_jumpto_tag_prepare(
    state: *mut JumpTagState,
    lbuf_arg: *const c_char,
    forceit: c_int,
    keep_help: bool,
) -> c_int {
    if state.is_null() || lbuf_arg.is_null() {
        return FAIL;
    }

    let state = &mut *state;

    // Check if we can set curbuf
    let postponed_split = nvim_get_postponed_split();
    if postponed_split == 0 && !nvim_check_can_set_curbuf_forceit(forceit) {
        state.phase = phase::ERROR;
        state.result = FAIL;
        return FAIL;
    }

    // Calculate the length of the match line
    let len = matching_line_len(lbuf_arg) + 1;

    // Allocate and copy the match line
    state.lbuf = xmalloc(len) as *mut c_char;
    if state.lbuf.is_null() {
        state.phase = phase::ERROR;
        state.result = FAIL;
        return FAIL;
    }
    memmove(state.lbuf as *mut c_void, lbuf_arg as *const c_void, len);
    state.lbuf_len = len - 1;

    // Allocate pattern buffer
    state.pbuf = xmalloc(LSIZE) as *mut c_char;
    if state.pbuf.is_null() {
        xfree(state.lbuf as *mut c_void);
        state.lbuf = ptr::null_mut();
        state.phase = phase::ERROR;
        state.result = FAIL;
        return FAIL;
    }

    state.forceit = forceit;
    state.keep_help = keep_help;
    state.phase = phase::PREPARED;

    OK
}

/// Calculate the length of a matching tag line.
///
/// The format is: `<mtt><tag_fname><NUL><NUL><lbuf>` for regular tags.
unsafe fn matching_line_len(lbuf: *const c_char) -> usize {
    let mut p = lbuf.add(1); // Skip mtt byte

    // Skip tag_fname and first NUL
    p = p.add(strlen(p) + 1);

    // Return total length including final part
    let offset = p.offset_from(lbuf) as usize;
    offset + strlen(p)
}

// =============================================================================
// Validation phase
// =============================================================================

/// Validate that the tag file exists.
///
/// # Arguments
///
/// * `state` - Jump state with prepared buffers
///
/// # Returns
///
/// OK if file exists, NOTAGFILE if not found, FAIL on error.
#[no_mangle]
pub unsafe extern "C" fn rs_jumpto_tag_validate(state: *mut JumpTagState) -> c_int {
    if state.is_null() {
        return FAIL;
    }

    let state = &mut *state;

    if state.phase != phase::PREPARED {
        return FAIL;
    }

    // At this point, the C code would have already expanded the filename
    // and stored it in expanded_fname. Check if it exists.
    if state.expanded_fname.is_null() {
        state.phase = phase::ERROR;
        state.result = FAIL;
        return FAIL;
    }

    // Check if file exists or has a BufReadCmd autocmd
    let exists =
        nvim_tag_path_exists(state.expanded_fname) || nvim_has_bufreadcmd(state.expanded_fname);

    if !exists {
        state.file_exists = false;
        state.result = NOTAGFILE;
        nvim_set_nofile_fname(state.expanded_fname);
        state.phase = phase::ERROR;
        return NOTAGFILE;
    }

    state.file_exists = true;
    state.phase = phase::VALIDATED;
    OK
}

// =============================================================================
// State accessors
// =============================================================================

/// Get the current phase of the jump state.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_phase(state: *const JumpTagState) -> c_int {
    if state.is_null() {
        return phase::ERROR;
    }
    (*state).phase
}

/// Set the current phase of the jump state.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_set_phase(state: *mut JumpTagState, new_phase: c_int) {
    if state.is_null() {
        return;
    }
    (*state).phase = new_phase;
}

/// Get the result code from the jump state.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_result(state: *const JumpTagState) -> c_int {
    if state.is_null() {
        return FAIL;
    }
    (*state).result
}

/// Set the result code in the jump state.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_set_result(state: *mut JumpTagState, result: c_int) {
    if state.is_null() {
        return;
    }
    (*state).result = result;
}

/// Get the match line buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_lbuf(state: *const JumpTagState) -> *mut c_char {
    if state.is_null() {
        return ptr::null_mut();
    }
    (*state).lbuf
}

/// Get the pattern buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_pbuf(state: *const JumpTagState) -> *mut c_char {
    if state.is_null() {
        return ptr::null_mut();
    }
    (*state).pbuf
}

/// Get the expanded file name.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_fname(state: *const JumpTagState) -> *mut c_char {
    if state.is_null() {
        return ptr::null_mut();
    }
    (*state).expanded_fname
}

/// Set the expanded file name.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_set_fname(state: *mut JumpTagState, fname: *mut c_char) {
    if state.is_null() {
        return;
    }
    (*state).expanded_fname = fname;
}

/// Get the force flag.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_forceit(state: *const JumpTagState) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).forceit
}

/// Get the keep_help flag.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_keep_help(state: *const JumpTagState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).keep_help
}

/// Check if the file was validated as existing.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_file_exists(state: *const JumpTagState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).file_exists
}

// =============================================================================
// Helper functions for tag file opening
// =============================================================================

/// Check if a tag file can be opened.
///
/// This validates that the file exists and can potentially be read.
///
/// # Arguments
///
/// * `fname` - File name to check
///
/// # Returns
///
/// true if the file can be opened, false otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_can_open_tag_file(fname: *const c_char) -> bool {
    if fname.is_null() {
        return false;
    }

    nvim_tag_path_exists(fname) || nvim_has_bufreadcmd(fname)
}

/// Expand a tag file name relative to the tag file that contains it.
///
/// This is a thin wrapper around the C expand_tag_fname function.
#[no_mangle]
pub unsafe extern "C" fn rs_expand_tag_fname(
    fname: *const c_char,
    tag_fname: *const c_char,
    expand: bool,
) -> *mut c_char {
    nvim_expand_tag_fname(fname, tag_fname, expand)
}

// =============================================================================
// Global state management
// =============================================================================

/// Get the postponed_split global.
#[no_mangle]
pub extern "C" fn rs_get_postponed_split() -> c_int {
    unsafe { nvim_get_postponed_split() }
}

/// Set the postponed_split global.
#[no_mangle]
pub unsafe extern "C" fn rs_set_postponed_split(val: c_int) {
    nvim_set_postponed_split(val);
}

/// Get the g_do_tagpreview global.
#[no_mangle]
pub extern "C" fn rs_get_g_do_tagpreview() -> c_int {
    unsafe { nvim_get_g_do_tagpreview() }
}

/// Set the g_do_tagpreview global.
#[no_mangle]
pub unsafe extern "C" fn rs_set_g_do_tagpreview(val: c_int) {
    nvim_set_g_do_tagpreview(val);
}

/// Reset g_do_tagpreview to 0 (cleanup after tag operation).
#[no_mangle]
pub unsafe extern "C" fn rs_reset_tagpreview() {
    nvim_set_g_do_tagpreview(0);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jump_state_default() {
        let state = JumpTagState::default();
        assert!(state.lbuf.is_null());
        assert!(state.pbuf.is_null());
        assert!(state.expanded_fname.is_null());
        assert_eq!(state.phase, phase::INIT);
        assert_eq!(state.result, 0);
    }

    #[test]
    fn test_jump_state_new() {
        let state = rs_jump_tag_state_new();
        assert!(!state.is_null());
        unsafe {
            assert!((*state).lbuf.is_null());
            assert_eq!((*state).phase, phase::INIT);
            // Note: Can't call rs_jump_tag_state_free without C runtime
        }
    }

    #[test]
    fn test_null_safety() {
        unsafe {
            // Cleanup with null
            rs_jump_tag_cleanup(ptr::null_mut());

            // Prepare with null state
            assert_eq!(
                rs_jumpto_tag_prepare(ptr::null_mut(), c"test".as_ptr(), 0, false),
                FAIL
            );

            // Prepare with null lbuf
            let state = rs_jump_tag_state_new();
            assert_eq!(rs_jumpto_tag_prepare(state, ptr::null(), 0, false), FAIL);

            // Validate with null
            assert_eq!(rs_jumpto_tag_validate(ptr::null_mut()), FAIL);

            // Accessors with null
            assert_eq!(rs_jump_tag_get_phase(ptr::null()), phase::ERROR);
            rs_jump_tag_set_phase(ptr::null_mut(), phase::DONE);
            assert_eq!(rs_jump_tag_get_result(ptr::null()), FAIL);
            rs_jump_tag_set_result(ptr::null_mut(), OK);
            assert!(rs_jump_tag_get_lbuf(ptr::null()).is_null());
            assert!(rs_jump_tag_get_pbuf(ptr::null()).is_null());
            assert!(rs_jump_tag_get_fname(ptr::null()).is_null());
            rs_jump_tag_set_fname(ptr::null_mut(), ptr::null_mut());
            assert_eq!(rs_jump_tag_get_forceit(ptr::null()), 0);
            assert!(!rs_jump_tag_get_keep_help(ptr::null()));
            assert!(!rs_jump_tag_file_exists(ptr::null()));
        }
    }

    #[test]
    fn test_can_open_tag_file_null() {
        unsafe {
            assert!(!rs_can_open_tag_file(ptr::null()));
        }
    }

    #[test]
    fn test_phase_constants() {
        assert_eq!(phase::INIT, 0);
        assert_eq!(phase::PREPARED, 1);
        assert_eq!(phase::VALIDATED, 2);
        assert_eq!(phase::READY, 3);
        assert_eq!(phase::DONE, 4);
        assert_eq!(phase::ERROR, -1);
    }

    #[test]
    fn test_result_constants() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(NOTAGFILE, 2);
    }
}
