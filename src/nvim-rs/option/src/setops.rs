//! Core set operations for options
//!
//! This module provides Rust implementations for the core option setting
//! operations including redraw flag checking, scope determination,
//! option change notification, and autocommand preparation infrastructure.

use std::ffi::{c_char, c_int, c_uint};

use crate::{OptScope, FAIL, OK};

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // State accessors
    fn nvim_get_secure() -> c_int;
    fn nvim_get_sandbox() -> c_int;
}

// =============================================================================
// Redraw Type Constants
// =============================================================================

/// Redraw type: not valid, clear and recompute
pub const UPD_NOT_VALID: c_int = 10;

/// Redraw type: clear screen
pub const UPD_CLEAR: c_int = 70;

// =============================================================================
// Option Flag Constants (for redraw checking)
// =============================================================================

/// Redraw tabline flag
pub const K_OPT_FLAG_REDR_TABL: c_uint = 1 << 6;

/// Redraw status lines flag
pub const K_OPT_FLAG_REDR_STAT: c_uint = 1 << 7;

/// Redraw current window flag
pub const K_OPT_FLAG_REDR_WIN: c_uint = 1 << 8;

/// Redraw current buffer flag
pub const K_OPT_FLAG_REDR_BUF: c_uint = 1 << 9;

/// Update curswant flag
pub const K_OPT_FLAG_CURSWANT: c_uint = 1 << 21;

/// Highlight only flag
pub const K_OPT_FLAG_HL_ONLY: c_uint = 1 << 23;

/// Combination of REDR_WIN and REDR_BUF
pub const K_OPT_FLAG_REDR_ALL: c_uint = K_OPT_FLAG_REDR_WIN | K_OPT_FLAG_REDR_BUF;

// =============================================================================
// Scope Constants
// =============================================================================

/// OPT_LOCAL flag
pub const OPT_LOCAL: c_int = 0x02;

/// OPT_GLOBAL flag
pub const OPT_GLOBAL: c_int = 0x01;

/// OPT_MODELINE flag
pub const OPT_MODELINE: c_int = 0x04;

/// MAXCOL constant (maximum column value)
pub const MAXCOL: c_int = 0x7fff_ffff;

// =============================================================================
// Scope Determination
// =============================================================================

/// Determine if scope is local only.
#[no_mangle]
pub extern "C" fn rs_scope_is_local(opt_flags: c_int) -> c_int {
    c_int::from((opt_flags & OPT_LOCAL) != 0)
}

/// Determine if scope is global only.
#[no_mangle]
pub extern "C" fn rs_scope_is_global(opt_flags: c_int) -> c_int {
    c_int::from((opt_flags & OPT_GLOBAL) != 0)
}

/// Determine if scope is both local and global (neither flag set).
#[no_mangle]
pub extern "C" fn rs_scope_is_both(opt_flags: c_int) -> c_int {
    c_int::from((opt_flags & (OPT_LOCAL | OPT_GLOBAL)) == 0)
}

/// Determine if this is from a modeline.
#[no_mangle]
pub extern "C" fn rs_is_modeline(opt_flags: c_int) -> c_int {
    c_int::from((opt_flags & OPT_MODELINE) != 0)
}

/// Convert scope flags to OptScope enum value.
#[no_mangle]
pub extern "C" fn rs_flags_to_scope(opt_flags: c_int) -> c_int {
    if (opt_flags & OPT_LOCAL) != 0 {
        if (opt_flags & OPT_GLOBAL) != 0 {
            OptScope::Global as c_int // Both = global
        } else {
            OptScope::Win as c_int // Local only could be win or buf
        }
    } else {
        OptScope::Global as c_int
    }
}

// =============================================================================
// Redraw Flag Checking
// =============================================================================

/// Result of redraw flag analysis.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RedrawFlags {
    /// Need to redraw tabline
    pub redraw_tabline: c_int,
    /// Need to redraw status lines
    pub redraw_status: c_int,
    /// Need to redraw current window
    pub redraw_window: c_int,
    /// Need to redraw current buffer
    pub redraw_buffer: c_int,
    /// Need to update curswant
    pub update_curswant: c_int,
    /// All flags combined for redraw_all
    pub redraw_all: c_int,
}

/// Analyze option flags to determine what needs to be redrawn.
#[no_mangle]
pub extern "C" fn rs_analyze_redraw_flags(flags: c_uint) -> RedrawFlags {
    let all = (flags & K_OPT_FLAG_REDR_ALL) == K_OPT_FLAG_REDR_ALL;
    let hl_only = (flags & K_OPT_FLAG_HL_ONLY) != 0;

    RedrawFlags {
        redraw_tabline: c_int::from((flags & K_OPT_FLAG_REDR_TABL) != 0 || all),
        redraw_status: c_int::from((flags & K_OPT_FLAG_REDR_STAT) != 0 || all),
        redraw_window: c_int::from((flags & K_OPT_FLAG_REDR_WIN) != 0 || all),
        redraw_buffer: c_int::from((flags & K_OPT_FLAG_REDR_BUF) != 0 || all),
        update_curswant: c_int::from(
            ((flags & (K_OPT_FLAG_CURSWANT | K_OPT_FLAG_REDR_ALL)) != 0) && !hl_only,
        ),
        redraw_all: c_int::from(all),
    }
}

// Note: check_redraw and post_set_processing remain in C (option.c) since they
// interact with many global variables and redraw functions. The Rust code provides
// the analysis via rs_analyze_redraw_flags() which C code can use to decide what
// actions to take.

// =============================================================================
// Option Change Context
// =============================================================================

/// Context for an option change operation.
///
/// This structure captures all the state needed to process an option change,
/// including old and new values, scope information, and callback state.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OptionChangeContext {
    /// Option index
    pub opt_idx: c_int,
    /// Option flags (OPT_LOCAL, OPT_GLOBAL, etc.)
    pub opt_flags: c_int,
    /// Script ID that set the option
    pub set_sid: c_int,
    /// Whether this is a direct set (no side effects)
    pub direct: c_int,
    /// Whether value was completely replaced
    pub value_replaced: c_int,
    /// Whether value was checked by callback
    pub value_checked: c_int,
    /// Whether value was changed by callback
    pub value_changed: c_int,
    /// Whether to restore chartab on error
    pub restore_chartab: c_int,
}

impl OptionChangeContext {
    /// Create a new option change context.
    #[must_use]
    pub const fn new(opt_idx: c_int, opt_flags: c_int) -> Self {
        Self {
            opt_idx,
            opt_flags,
            set_sid: 0,
            direct: 0,
            value_replaced: 0,
            value_checked: 0,
            value_changed: 0,
            restore_chartab: 0,
        }
    }
}

/// FFI: Create a new option change context.
#[no_mangle]
pub extern "C" fn rs_option_change_context_new(
    opt_idx: c_int,
    opt_flags: c_int,
) -> OptionChangeContext {
    OptionChangeContext::new(opt_idx, opt_flags)
}

/// FFI: Check if scope is local only.
#[no_mangle]
pub unsafe extern "C" fn rs_option_change_scope_local(ctx: *const OptionChangeContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    rs_scope_is_local((*ctx).opt_flags)
}

/// FFI: Check if scope is global only.
#[no_mangle]
pub unsafe extern "C" fn rs_option_change_scope_global(ctx: *const OptionChangeContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    rs_scope_is_global((*ctx).opt_flags)
}

/// FFI: Check if scope is both (neither local nor global flag set).
#[no_mangle]
pub unsafe extern "C" fn rs_option_change_scope_both(ctx: *const OptionChangeContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    rs_scope_is_both((*ctx).opt_flags)
}

// =============================================================================
// Secure Mode Checking
// =============================================================================

/// Check if option setting should be done in secure mode.
///
/// Returns 1 if secure mode should be used, 0 otherwise.
/// Secure mode is used when:
/// - Option is set from a modeline
/// - We're in sandbox mode
/// - Value was set with kOptFlagInsecure and not completely replaced
#[no_mangle]
pub unsafe extern "C" fn rs_should_use_secure_mode(
    opt_flags: c_int,
    insecure_flag: c_uint,
    value_replaced: c_int,
) -> c_int {
    if (opt_flags & OPT_MODELINE) != 0 {
        return 1;
    }
    if nvim_get_sandbox() != 0 {
        return 1;
    }
    // Use K_OPT_FLAG_INSECURE from module-level constant
    if value_replaced == 0 && (insecure_flag & K_OPT_FLAG_INSECURE) != 0 {
        return 1;
    }
    0
}

// =============================================================================
// Autocommand Preparation
// =============================================================================

/// Flags indicating which autocommands should be triggered.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct AutocmdFlags {
    /// Trigger OptionSet autocommand
    pub trigger_optionset: c_int,
    /// Trigger syntax autocommand
    pub trigger_syntax: c_int,
    /// Trigger filetype autocommand
    pub trigger_filetype: c_int,
    /// Trigger spelllang autocommand
    pub trigger_spelllang: c_int,
}

/// FFI: Create autocommand flags.
#[no_mangle]
pub extern "C" fn rs_autocmd_flags_new() -> AutocmdFlags {
    AutocmdFlags::default()
}

/// Determine which autocommands should be triggered after an option change.
///
/// # Arguments
/// * `opt_flags` - Option flags
/// * `direct` - Whether this is a direct set (no side effects)
/// * `value_changed` - Whether the value actually changed
/// * `starting` - The 'starting' global value (non-zero during startup)
///
/// # Returns
/// Flags indicating which autocommands to trigger.
#[no_mangle]
pub extern "C" fn rs_determine_autocmds(
    opt_flags: c_int,
    direct: c_int,
    value_changed: c_int,
    starting: c_int,
) -> AutocmdFlags {
    let mut flags = AutocmdFlags::default();

    // Don't trigger autocommands for direct sets
    if direct != 0 {
        return flags;
    }

    // Don't trigger OptionSet during startup
    if starting == 0 {
        flags.trigger_optionset = 1;
    }

    // Syntax and filetype autocommands are triggered based on value_changed
    // and whether we're in a modeline (handled in C code for now due to complexity)
    if value_changed != 0 || (opt_flags & OPT_MODELINE) == 0 {
        // These are set based on which option changed (varp comparison)
        // For now, we just indicate potential triggers
        flags.trigger_syntax = 0;
        flags.trigger_filetype = 0;
        flags.trigger_spelllang = 0;
    }

    flags
}

// =============================================================================
// Value Comparison Utilities
// =============================================================================
//
// Note: rs_bool_values_equal and rs_num_values_equal are defined in copy.rs
// to avoid symbol duplication. Import from crate::copy if needed locally.

/// Compare two string option values (by pointer only).
/// For deep comparison, use C-side strcmp.
#[no_mangle]
pub extern "C" fn rs_str_values_same_ptr(old: *const c_char, new: *const c_char) -> c_int {
    c_int::from(old == new)
}

// =============================================================================
// Set Result Processing
// =============================================================================

/// Result of a set operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SetOperationResult {
    /// Error message (NULL if success)
    pub errmsg: *const c_char,
    /// Whether the operation succeeded
    pub success: c_int,
    /// Whether value was actually changed
    pub value_changed: c_int,
    /// Whether side effects were processed
    pub side_effects_done: c_int,
}

impl Default for SetOperationResult {
    fn default() -> Self {
        Self {
            errmsg: std::ptr::null(),
            success: OK,
            value_changed: 0,
            side_effects_done: 0,
        }
    }
}

/// FFI: Create a successful set operation result.
#[no_mangle]
pub extern "C" fn rs_set_result_ok() -> SetOperationResult {
    SetOperationResult::default()
}

/// FFI: Create a failed set operation result.
#[no_mangle]
pub extern "C" fn rs_set_result_error(errmsg: *const c_char) -> SetOperationResult {
    SetOperationResult {
        errmsg,
        success: FAIL,
        value_changed: 0,
        side_effects_done: 0,
    }
}

/// FFI: Mark set operation result as value changed.
#[no_mangle]
pub unsafe extern "C" fn rs_set_result_mark_changed(result: *mut SetOperationResult) {
    if !result.is_null() {
        (*result).value_changed = 1;
    }
}

/// FFI: Mark set operation result as side effects done.
#[no_mangle]
pub unsafe extern "C" fn rs_set_result_mark_side_effects(result: *mut SetOperationResult) {
    if !result.is_null() {
        (*result).side_effects_done = 1;
    }
}

// =============================================================================
// WasSet Flag Management
// =============================================================================

/// kOptFlagWasSet constant
pub const K_OPT_FLAG_WAS_SET: c_uint = 1 << 3;

/// Check if the WasSet flag should be set.
///
/// Returns 1 if the flag should be set, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_should_set_was_set(errmsg: *const c_char) -> c_int {
    c_int::from(errmsg.is_null())
}

/// Compute new flags value with WasSet added.
#[no_mangle]
pub extern "C" fn rs_add_was_set_flag(flags: c_uint) -> c_uint {
    flags | K_OPT_FLAG_WAS_SET
}

// =============================================================================
// Insecure Flag Management
// =============================================================================

/// kOptFlagInsecure constant
pub const K_OPT_FLAG_INSECURE: c_uint = 1 << 18;

/// Determine how to update the insecure flag after setting an option.
///
/// # Arguments
/// * `value_checked` - Whether the value was checked by a callback
/// * `value_replaced` - Whether the value was completely replaced
/// * `opt_flags` - Option flags
///
/// # Returns
/// 0 = no change, 1 = add insecure flag, -1 = remove insecure flag
#[no_mangle]
pub unsafe extern "C" fn rs_compute_insecure_flag_change(
    value_checked: c_int,
    value_replaced: c_int,
    opt_flags: c_int,
) -> c_int {
    // If value wasn't checked and (secure or sandbox or modeline), add insecure flag
    if value_checked == 0
        && (nvim_get_secure() != 0 || nvim_get_sandbox() != 0 || (opt_flags & OPT_MODELINE) != 0)
    {
        return 1;
    }

    // If value was completely replaced, remove insecure flag
    if value_replaced != 0 {
        return -1;
    }

    0
}

/// Apply insecure flag change to flags.
#[no_mangle]
pub extern "C" fn rs_apply_insecure_flag_change(flags: c_uint, change: c_int) -> c_uint {
    match change {
        1 => flags | K_OPT_FLAG_INSECURE,
        -1 => flags & !K_OPT_FLAG_INSECURE,
        _ => flags,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::copy::{rs_bool_values_equal, rs_num_values_equal};

    #[test]
    fn test_scope_is_local() {
        assert_eq!(rs_scope_is_local(OPT_LOCAL), 1);
        assert_eq!(rs_scope_is_local(OPT_GLOBAL), 0);
        assert_eq!(rs_scope_is_local(OPT_LOCAL | OPT_GLOBAL), 1);
        assert_eq!(rs_scope_is_local(0), 0);
    }

    #[test]
    fn test_scope_is_global() {
        assert_eq!(rs_scope_is_global(OPT_GLOBAL), 1);
        assert_eq!(rs_scope_is_global(OPT_LOCAL), 0);
        assert_eq!(rs_scope_is_global(OPT_LOCAL | OPT_GLOBAL), 1);
        assert_eq!(rs_scope_is_global(0), 0);
    }

    #[test]
    fn test_scope_is_both() {
        assert_eq!(rs_scope_is_both(0), 1);
        assert_eq!(rs_scope_is_both(OPT_MODELINE), 1);
        assert_eq!(rs_scope_is_both(OPT_LOCAL), 0);
        assert_eq!(rs_scope_is_both(OPT_GLOBAL), 0);
        assert_eq!(rs_scope_is_both(OPT_LOCAL | OPT_GLOBAL), 0);
    }

    #[test]
    fn test_is_modeline() {
        assert_eq!(rs_is_modeline(OPT_MODELINE), 1);
        assert_eq!(rs_is_modeline(OPT_LOCAL | OPT_MODELINE), 1);
        assert_eq!(rs_is_modeline(OPT_LOCAL), 0);
        assert_eq!(rs_is_modeline(0), 0);
    }

    #[test]
    fn test_analyze_redraw_flags() {
        // Test individual flags
        let rf = rs_analyze_redraw_flags(K_OPT_FLAG_REDR_TABL);
        assert_eq!(rf.redraw_tabline, 1);
        assert_eq!(rf.redraw_status, 0);
        assert_eq!(rf.redraw_window, 0);
        assert_eq!(rf.redraw_buffer, 0);

        let rf = rs_analyze_redraw_flags(K_OPT_FLAG_REDR_STAT);
        assert_eq!(rf.redraw_status, 1);
        assert_eq!(rf.redraw_tabline, 0);

        // Test REDR_ALL
        let rf = rs_analyze_redraw_flags(K_OPT_FLAG_REDR_ALL);
        assert_eq!(rf.redraw_all, 1);
        assert_eq!(rf.redraw_window, 1);
        assert_eq!(rf.redraw_buffer, 1);
        assert_eq!(rf.redraw_tabline, 1);
        assert_eq!(rf.redraw_status, 1);

        // Test curswant with HL_ONLY
        let rf = rs_analyze_redraw_flags(K_OPT_FLAG_CURSWANT);
        assert_eq!(rf.update_curswant, 1);

        let rf = rs_analyze_redraw_flags(K_OPT_FLAG_CURSWANT | K_OPT_FLAG_HL_ONLY);
        assert_eq!(rf.update_curswant, 0);
    }

    #[test]
    fn test_option_change_context() {
        let ctx = OptionChangeContext::new(42, OPT_LOCAL);
        assert_eq!(ctx.opt_idx, 42);
        assert_eq!(ctx.opt_flags, OPT_LOCAL);
        assert_eq!(ctx.direct, 0);
        assert_eq!(ctx.value_replaced, 0);
    }

    #[test]
    fn test_bool_values_equal() {
        assert_eq!(rs_bool_values_equal(0, 0), 1);
        assert_eq!(rs_bool_values_equal(1, 1), 1);
        assert_eq!(rs_bool_values_equal(1, 2), 1); // Both truthy
        assert_eq!(rs_bool_values_equal(0, 1), 0);
        assert_eq!(rs_bool_values_equal(1, 0), 0);
    }

    #[test]
    fn test_num_values_equal() {
        assert_eq!(rs_num_values_equal(0, 0), 1);
        assert_eq!(rs_num_values_equal(42, 42), 1);
        assert_eq!(rs_num_values_equal(-1, -1), 1);
        assert_eq!(rs_num_values_equal(0, 1), 0);
        assert_eq!(rs_num_values_equal(42, 43), 0);
    }

    #[test]
    fn test_set_result() {
        let result = rs_set_result_ok();
        assert!(result.errmsg.is_null());
        assert_eq!(result.success, OK);
        assert_eq!(result.value_changed, 0);

        let errmsg = c"test error".as_ptr();
        let result = rs_set_result_error(errmsg);
        assert_eq!(result.errmsg, errmsg);
        assert_eq!(result.success, FAIL);
    }

    #[test]
    fn test_was_set_flag() {
        assert_eq!(rs_should_set_was_set(std::ptr::null()), 1);
        assert_eq!(rs_should_set_was_set(c"error".as_ptr()), 0);

        let flags: c_uint = 0;
        let new_flags = rs_add_was_set_flag(flags);
        assert_eq!(new_flags, K_OPT_FLAG_WAS_SET);

        let flags: c_uint = 0xFF;
        let new_flags = rs_add_was_set_flag(flags);
        assert_eq!(new_flags, 0xFF | K_OPT_FLAG_WAS_SET);
    }

    #[test]
    fn test_insecure_flag_change() {
        // Test apply changes
        let flags: c_uint = 0;

        // Add flag
        let new_flags = rs_apply_insecure_flag_change(flags, 1);
        assert_eq!(new_flags, K_OPT_FLAG_INSECURE);

        // Remove flag
        let flags_with_insecure = K_OPT_FLAG_INSECURE;
        let new_flags = rs_apply_insecure_flag_change(flags_with_insecure, -1);
        assert_eq!(new_flags, 0);

        // No change
        let flags: c_uint = 0x100;
        let new_flags = rs_apply_insecure_flag_change(flags, 0);
        assert_eq!(new_flags, 0x100);
    }

    #[test]
    fn test_str_values_same_ptr() {
        let s1 = c"test".as_ptr();
        let s3 = s1;

        // Same pointer
        assert_eq!(rs_str_values_same_ptr(s1, s3), 1);
        // Different pointers
        assert_eq!(rs_str_values_same_ptr(std::ptr::null(), s1), 0);
    }
}
