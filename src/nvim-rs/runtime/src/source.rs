//! Script sourcing operations
//!
//! This module handles sourcing Vim and Lua scripts.

use std::ffi::{c_char, c_int, c_void};

use crate::constants::CSTP_FINISH;
use crate::{doso, LinenrT, ScidT};

// =============================================================================
// C FFI for Phase 2 functions
// =============================================================================

extern "C" {
    // source_cookie_T field accessors (defined in runtime.c)
    fn nvim_rt_cookie_get_breakpoint_ptr(cookie: *mut c_void) -> *mut LinenrT;
    fn nvim_rt_cookie_get_dbg_tick_ptr(cookie: *mut c_void) -> *mut c_int;
    fn nvim_rt_cookie_get_level(cookie: *mut c_void) -> c_int;
    fn nvim_rt_cookie_get_finished(cookie: *mut c_void) -> bool;
    fn nvim_rt_cookie_set_finished(cookie: *mut c_void, val: bool);

    // exarg_T accessors (defined in runtime_ffi.c)
    fn nvim_rt_exarg_is_sourcing(eap: *mut c_void) -> bool;
    fn nvim_rt_exarg_get_source_cookie(eap: *mut c_void) -> *mut c_void;
    fn nvim_rt_getline_is_sourcing(fgetline: *mut c_void, cookie: *mut c_void) -> bool;
    fn nvim_rt_getline_get_source_cookie(fgetline: *mut c_void, cookie: *mut c_void)
        -> *mut c_void;
    fn nvim_rt_exarg_get_cstack(eap: *mut c_void) -> *mut c_void;

    // Encoding functions
    #[link_name = "enc_canonize"]
    fn nvim_rt_enc_canonize(enc: *mut c_char) -> *mut c_char;
    #[link_name = "convert_setup"]
    fn nvim_rt_convert_setup(vcp: *mut c_void, from: *mut c_char, to: *const c_char) -> c_int;
    fn nvim_rt_cookie_get_conv(cookie: *mut c_void) -> *mut c_void;
    fn nvim_rt_get_p_enc() -> *const c_char;

    // Conditional cleanup
    fn nvim_rt_cleanup_conditionals(
        cstack: *mut c_void,
        searched_cond: c_int,
        inclusive: c_int,
    ) -> c_int;
    fn nvim_rt_cstack_set_pending(cstack: *mut c_void, idx: c_int, val: c_int);
    fn nvim_rt_report_make_pending_finish();

    // Error messages
    fn nvim_rt_emsg_scriptencoding_outside();
    fn nvim_rt_emsg_finish_outside();

    // exarg_T helpers
    fn nvim_rt_exarg_get_arg(eap: *mut c_void) -> *mut c_char;

    // Memory management
    fn xfree(ptr: *mut c_void);
}

// =============================================================================
// Phase 2: source cookie / finish / scriptencoding functions
// =============================================================================

/// Returns a pointer to the breakpoint field of the source cookie.
///
/// # Safety
/// cookie must be a valid source_cookie_T pointer.
#[unsafe(export_name = "source_breakpoint")]
pub unsafe extern "C" fn rs_source_breakpoint(cookie: *mut c_void) -> *mut LinenrT {
    nvim_rt_cookie_get_breakpoint_ptr(cookie)
}

/// Returns a pointer to the dbg_tick field of the source cookie.
///
/// # Safety
/// cookie must be a valid source_cookie_T pointer.
#[unsafe(export_name = "source_dbg_tick")]
pub unsafe extern "C" fn rs_source_dbg_tick(cookie: *mut c_void) -> *mut c_int {
    nvim_rt_cookie_get_dbg_tick_ptr(cookie)
}

/// Returns the nesting level from the source cookie.
///
/// # Safety
/// cookie must be a valid source_cookie_T pointer.
#[unsafe(export_name = "source_level")]
pub unsafe extern "C" fn rs_source_level(cookie: *mut c_void) -> c_int {
    nvim_rt_cookie_get_level(cookie)
}

/// Returns true if the current getline function is getsourceline (i.e., we are sourcing).
///
/// # Safety
/// eap must be a valid exarg_T pointer.
#[unsafe(export_name = "sourcing_a_script")]
pub unsafe extern "C" fn rs_sourcing_a_script(eap: *mut c_void) -> c_int {
    c_int::from(nvim_rt_exarg_is_sourcing(eap))
}

/// `:scriptencoding` command: set encoding conversion for a sourced script.
///
/// # Safety
/// eap must be a valid exarg_T pointer.
#[unsafe(export_name = "ex_scriptencoding")]
pub unsafe extern "C" fn rs_ex_scriptencoding(eap: *mut c_void) {
    if !nvim_rt_exarg_is_sourcing(eap) {
        nvim_rt_emsg_scriptencoding_outside();
        return;
    }

    let arg = nvim_rt_exarg_get_arg(eap);
    let name = if !arg.is_null() && *arg != 0 {
        nvim_rt_enc_canonize(arg)
    } else {
        arg
    };

    let sp = nvim_rt_exarg_get_source_cookie(eap);
    let vcp = nvim_rt_cookie_get_conv(sp);
    let p_enc = nvim_rt_get_p_enc();
    nvim_rt_convert_setup(vcp, name, p_enc);

    if !name.is_null() && name != arg {
        xfree(name.cast::<c_void>());
    }
}

/// `:finish` command: mark a sourced file as finished.
///
/// # Safety
/// eap must be a valid exarg_T pointer.
#[unsafe(export_name = "ex_finish")]
pub unsafe extern "C" fn rs_ex_finish(eap: *mut c_void) {
    if nvim_rt_exarg_is_sourcing(eap) {
        rs_do_finish(eap, false);
    } else {
        nvim_rt_emsg_finish_outside();
    }
}

/// Mark a sourced file as finished; handle pending `:finish` in try/finally.
///
/// # Safety
/// eap must be a valid exarg_T pointer.
#[unsafe(export_name = "do_finish")]
pub unsafe extern "C" fn rs_do_finish(eap: *mut c_void, reanimate: bool) {
    if reanimate {
        let sp = nvim_rt_exarg_get_source_cookie(eap);
        nvim_rt_cookie_set_finished(sp, false);
    }

    let cstack = nvim_rt_exarg_get_cstack(eap);
    let idx = nvim_rt_cleanup_conditionals(cstack, 0, 1);
    if idx >= 0 {
        nvim_rt_cstack_set_pending(cstack, idx, CSTP_FINISH);
        nvim_rt_report_make_pending_finish();
    } else {
        let sp = nvim_rt_exarg_get_source_cookie(eap);
        nvim_rt_cookie_set_finished(sp, true);
    }
}

/// Returns true when a sourced file had the `:finish` command.
///
/// # Safety
/// fgetline and cookie must be valid pointers.
#[unsafe(export_name = "source_finished")]
pub unsafe extern "C" fn rs_source_finished(fgetline: *mut c_void, cookie: *mut c_void) -> bool {
    if !nvim_rt_getline_is_sourcing(fgetline, cookie) {
        return false;
    }
    let sp = nvim_rt_getline_get_source_cookie(fgetline, cookie);
    nvim_rt_cookie_get_finished(sp)
}

// =============================================================================
// Source Flags
// =============================================================================

/// Check if sourcing a vimrc file.
pub fn rs_sourcing_vimrc(flags: c_int) -> bool {
    flags == doso::VIMRC
}

/// Check if this is a regular source (not vimrc).
pub fn rs_sourcing_regular(flags: c_int) -> bool {
    flags == doso::NONE
}

// =============================================================================
// Source State
// =============================================================================

/// Source file state for tracking line-by-line execution.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SourceState {
    /// Current line number being executed
    pub current_lnum: LinenrT,
    /// Total lines in the file
    pub total_lines: LinenrT,
    /// Script ID assigned to this source
    pub script_id: ScidT,
    /// Whether we're in breakpoint debug mode
    pub breakpoint: bool,
    /// Whether this is a Lua file
    pub is_lua: bool,
}

impl Default for SourceState {
    fn default() -> Self {
        Self {
            current_lnum: 1,
            total_lines: 0,
            script_id: 0,
            breakpoint: false,
            is_lua: false,
        }
    }
}

/// Create default source state.
pub fn rs_source_state_default() -> SourceState {
    SourceState::default()
}

/// Initialize source state for a script.
pub fn rs_source_state_init(script_id: ScidT, total_lines: LinenrT, is_lua: bool) -> SourceState {
    SourceState {
        current_lnum: 1,
        total_lines,
        script_id,
        breakpoint: false,
        is_lua,
    }
}

/// Advance to the next line.
pub fn rs_source_state_next_line(state: &mut SourceState) {
    state.current_lnum += 1;
}

/// Check if we're at the end of the file.
pub fn rs_source_state_at_end(state: &SourceState) -> bool {
    state.current_lnum > state.total_lines
}

/// Get progress percentage (0-100).
pub fn rs_source_state_progress(state: &SourceState) -> c_int {
    if state.total_lines <= 0 {
        return 100;
    }
    ((state.current_lnum * 100) / state.total_lines) as c_int
}

// =============================================================================
// Source Result
// =============================================================================

/// Result codes from do_source()
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceResult {
    /// Script sourced successfully
    Ok = 0,
    /// File not found
    FileNotFound = 1,
    /// Permission denied
    PermissionDenied = 2,
    /// Read error
    ReadError = 3,
    /// Script aborted (user interrupt)
    Aborted = 4,
    /// Error in script
    ScriptError = 5,
}

impl SourceResult {
    /// Convert to integer
    pub const fn as_int(self) -> c_int {
        self as c_int
    }

    /// Create from integer
    pub const fn from_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Ok),
            1 => Some(Self::FileNotFound),
            2 => Some(Self::PermissionDenied),
            3 => Some(Self::ReadError),
            4 => Some(Self::Aborted),
            5 => Some(Self::ScriptError),
            _ => None,
        }
    }

    /// Check if result is success
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }
}

/// Check if source result indicates success.
pub fn rs_source_result_ok(result: c_int) -> bool {
    result == SourceResult::Ok as c_int
}

/// Check if source result indicates file not found.
pub fn rs_source_result_not_found(result: c_int) -> bool {
    result == SourceResult::FileNotFound as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_flags() {
        assert!(!rs_sourcing_vimrc(doso::NONE));
        assert!(rs_sourcing_vimrc(doso::VIMRC));
        assert!(rs_sourcing_regular(doso::NONE));
        assert!(!rs_sourcing_regular(doso::VIMRC));
    }

    #[test]
    fn test_source_state() {
        let state = rs_source_state_init(1, 100, false);
        assert_eq!(state.current_lnum, 1);
        assert_eq!(state.total_lines, 100);
        assert_eq!(state.script_id, 1);
        assert!(!state.is_lua);
        assert!(!rs_source_state_at_end(&state));
    }

    #[test]
    fn test_source_state_progress() {
        let mut state = rs_source_state_init(1, 100, false);
        assert_eq!(rs_source_state_progress(&state), 1);

        state.current_lnum = 50;
        assert_eq!(rs_source_state_progress(&state), 50);

        state.current_lnum = 100;
        assert_eq!(rs_source_state_progress(&state), 100);

        // Empty file
        let empty = rs_source_state_init(1, 0, false);
        assert_eq!(rs_source_state_progress(&empty), 100);
    }

    #[test]
    fn test_source_result() {
        assert!(rs_source_result_ok(SourceResult::Ok as c_int));
        assert!(!rs_source_result_ok(SourceResult::FileNotFound as c_int));
        assert!(rs_source_result_not_found(
            SourceResult::FileNotFound as c_int
        ));
    }
}
