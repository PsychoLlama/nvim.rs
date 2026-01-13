//! Option :set command processing
//!
//! This module provides Rust FFI implementations for the `:set`, `:setlocal`,
//! and `:setglobal` command processing infrastructure.

use std::ffi::{c_char, c_int, c_uint};
use std::ptr;

use crate::{OptInt, OptScope, FAIL, OK};

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // State accessors
    fn nvim_get_p_verbose() -> OptInt;
}

// =============================================================================
// Set Command Flags
// =============================================================================

/// Flags for :set command processing.
pub mod set_flags {
    use std::ffi::c_int;

    /// OPT_LOCAL - set local value
    pub const OPT_LOCAL: c_int = 0x02;
    /// OPT_GLOBAL - set global value
    pub const OPT_GLOBAL: c_int = 0x01;
    /// OPT_MODELINE - set from modeline
    pub const OPT_MODELINE: c_int = 0x04;
    /// OPT_WINONLY - only set window-local options
    pub const OPT_WINONLY: c_int = 0x08;
    /// OPT_NOWIN - don't set window-local options
    pub const OPT_NOWIN: c_int = 0x10;
    /// OPT_ONECOLUMN - show options in one column
    pub const OPT_ONECOLUMN: c_int = 0x20;
    /// OPT_NO_REDRAW - don't redraw screen
    pub const OPT_NO_REDRAW: c_int = 0x40;
    /// OPT_SKIPRTP - don't change runtimepath
    pub const OPT_SKIPRTP: c_int = 0x80;
}

// =============================================================================
// Set Command Type
// =============================================================================

/// Type of :set command.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetCommandType {
    /// :set - set both scopes
    #[default]
    Set = 0,
    /// :setlocal - set local scope only
    SetLocal = 1,
    /// :setglobal - set global scope only
    SetGlobal = 2,
}

impl SetCommandType {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::SetLocal,
            2 => Self::SetGlobal,
            _ => Self::Set,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Get the scope flags for this command type.
    #[must_use]
    pub const fn to_opt_flags(self) -> c_int {
        match self {
            Self::Set => 0,
            Self::SetLocal => set_flags::OPT_LOCAL,
            Self::SetGlobal => set_flags::OPT_GLOBAL,
        }
    }

    /// Check if this is :setlocal.
    #[must_use]
    pub const fn is_local(self) -> bool {
        matches!(self, Self::SetLocal)
    }

    /// Check if this is :setglobal.
    #[must_use]
    pub const fn is_global(self) -> bool {
        matches!(self, Self::SetGlobal)
    }
}

/// FFI: Create SetCommandType from flags.
#[no_mangle]
pub extern "C" fn rs_set_command_type_from_flags(flags: c_int) -> c_int {
    if (flags & set_flags::OPT_LOCAL) != 0 {
        SetCommandType::SetLocal.to_c_int()
    } else if (flags & set_flags::OPT_GLOBAL) != 0 {
        SetCommandType::SetGlobal.to_c_int()
    } else {
        SetCommandType::Set.to_c_int()
    }
}

/// FFI: Get opt flags from command type.
#[no_mangle]
pub extern "C" fn rs_set_command_type_to_flags(cmd_type: c_int) -> c_int {
    SetCommandType::from_c_int(cmd_type).to_opt_flags()
}

// =============================================================================
// Set Argument Parsing
// =============================================================================

/// Argument type in :set command.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetArgType {
    /// Show value (no assignment)
    #[default]
    Show = 0,
    /// Query value with ?
    Query = 1,
    /// Set to value
    Assign = 2,
    /// Reset to default with &
    Reset = 3,
    /// Toggle (invert) with inv or !
    Toggle = 4,
    /// Append with +=
    Append = 5,
    /// Prepend with ^=
    Prepend = 6,
    /// Remove with -=
    Remove = 7,
}

impl SetArgType {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Query,
            2 => Self::Assign,
            3 => Self::Reset,
            4 => Self::Toggle,
            5 => Self::Append,
            6 => Self::Prepend,
            7 => Self::Remove,
            _ => Self::Show,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this modifies the value.
    #[must_use]
    pub const fn modifies_value(self) -> bool {
        !matches!(self, Self::Show | Self::Query)
    }

    /// Check if this is a compound assignment (+=, ^=, -=).
    #[must_use]
    pub const fn is_compound(self) -> bool {
        matches!(self, Self::Append | Self::Prepend | Self::Remove)
    }
}

/// FFI: Check if arg type modifies value.
#[no_mangle]
pub extern "C" fn rs_set_arg_modifies(arg_type: c_int) -> c_int {
    c_int::from(SetArgType::from_c_int(arg_type).modifies_value())
}

/// FFI: Check if arg type is compound assignment.
#[no_mangle]
pub extern "C" fn rs_set_arg_is_compound(arg_type: c_int) -> c_int {
    c_int::from(SetArgType::from_c_int(arg_type).is_compound())
}

// =============================================================================
// Set Command Result
// =============================================================================

/// Result of a :set command operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SetCommandResult {
    /// Error message (NULL if success)
    pub errmsg: *const c_char,
    /// Return code (OK or FAIL)
    pub retval: c_int,
    /// Number of options processed
    pub count: c_int,
    /// Whether any option was changed
    pub changed: c_int,
}

impl Default for SetCommandResult {
    fn default() -> Self {
        Self {
            errmsg: ptr::null(),
            retval: OK,
            count: 0,
            changed: 0,
        }
    }
}

/// FFI: Create a success result.
#[no_mangle]
pub extern "C" fn rs_set_result_success(count: c_int, changed: c_int) -> SetCommandResult {
    SetCommandResult {
        errmsg: ptr::null(),
        retval: OK,
        count,
        changed,
    }
}

/// FFI: Create a failure result.
#[no_mangle]
pub extern "C" fn rs_set_result_fail(errmsg: *const c_char) -> SetCommandResult {
    SetCommandResult {
        errmsg,
        retval: FAIL,
        count: 0,
        changed: 0,
    }
}

/// FFI: Check if SetCommandResult is success.
///
/// # Safety
/// `result` must be a valid pointer to a `SetCommandResult` or null.
#[no_mangle]
pub unsafe extern "C" fn rs_setcmd_result_is_ok(result: *const SetCommandResult) -> c_int {
    if result.is_null() {
        return 0;
    }
    c_int::from((*result).retval == OK)
}

// =============================================================================
// Show Options Processing
// =============================================================================

/// Mode for showing options.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShowOptionsMode {
    /// Show current value
    #[default]
    Value = 0,
    /// Show all options
    All = 1,
    /// Show options that differ from default
    Changed = 2,
    /// Show terminal options
    Terminal = 3,
}

impl ShowOptionsMode {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::All,
            2 => Self::Changed,
            3 => Self::Terminal,
            _ => Self::Value,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

/// FFI: Check if showing all options.
#[no_mangle]
pub extern "C" fn rs_show_mode_is_all(mode: c_int) -> c_int {
    c_int::from(ShowOptionsMode::from_c_int(mode) == ShowOptionsMode::All)
}

/// FFI: Check if showing changed options.
#[no_mangle]
pub extern "C" fn rs_show_mode_is_changed(mode: c_int) -> c_int {
    c_int::from(ShowOptionsMode::from_c_int(mode) == ShowOptionsMode::Changed)
}

/// FFI: Check if showing terminal options.
#[no_mangle]
pub extern "C" fn rs_show_mode_is_terminal(mode: c_int) -> c_int {
    c_int::from(ShowOptionsMode::from_c_int(mode) == ShowOptionsMode::Terminal)
}

// =============================================================================
// Silent Mode Checking
// =============================================================================

// Note: rs_is_emsg_silent is defined in the message crate

/// Check if verbose mode is enabled for :set commands.
#[no_mangle]
pub unsafe extern "C" fn rs_setcmd_is_verbose() -> c_int {
    c_int::from(nvim_get_p_verbose() > 0)
}

/// Get verbose level for :set commands.
#[no_mangle]
pub unsafe extern "C" fn rs_setcmd_get_verbose_level() -> OptInt {
    nvim_get_p_verbose()
}

// =============================================================================
// Option Scope Resolution
// =============================================================================

/// Resolve effective scope for an option based on command type and option support.
///
/// # Arguments
/// * `cmd_type` - The :set command type (0=set, 1=setlocal, 2=setglobal)
/// * `opt_scope_support` - Bitmask of scopes the option supports
///
/// # Returns
/// The effective scope to use.
#[no_mangle]
pub extern "C" fn rs_resolve_effective_scope(cmd_type: c_int, opt_scope_support: c_uint) -> c_int {
    let cmd = SetCommandType::from_c_int(cmd_type);

    // For :setglobal, always use global scope
    if cmd.is_global() {
        return OptScope::Global as c_int;
    }

    // For :setlocal, prefer window > buffer > global
    if cmd.is_local() {
        if (opt_scope_support & 0x04) != 0 {
            // SCOPE_WINDOW
            return OptScope::Win as c_int;
        }
        if (opt_scope_support & 0x02) != 0 {
            // SCOPE_BUFFER
            return OptScope::Buf as c_int;
        }
    }

    // Default to global
    OptScope::Global as c_int
}

/// Check if an option should be set at a given scope.
///
/// # Arguments
/// * `cmd_type` - The :set command type
/// * `scope` - The scope to check
/// * `opt_scope_support` - Bitmask of scopes the option supports
///
/// # Returns
/// 1 if the option should be set at this scope, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_should_set_at_scope(
    cmd_type: c_int,
    scope: c_int,
    opt_scope_support: c_uint,
) -> c_int {
    let cmd = SetCommandType::from_c_int(cmd_type);

    // For :setglobal, only set global
    if cmd.is_global() {
        return c_int::from(scope == OptScope::Global as c_int);
    }

    // For :setlocal, only set local
    if cmd.is_local() {
        return c_int::from(scope != OptScope::Global as c_int);
    }

    // For :set, set both if the option supports both
    // Check if the requested scope is supported
    let scope_bit = match scope {
        0 => 0x01, // Global
        1 => 0x02, // Buffer
        2 => 0x04, // Window
        _ => return 0,
    };

    c_int::from((opt_scope_support & scope_bit) != 0)
}

// =============================================================================
// Option Copy Infrastructure
// =============================================================================

/// Direction for copying option values.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CopyOptDirection {
    /// Copy from global to local (for new window/buffer)
    #[default]
    GlobalToLocal = 0,
    /// Copy from local to global
    LocalToGlobal = 1,
}

impl CopyOptDirection {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::LocalToGlobal,
            _ => Self::GlobalToLocal,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

/// FFI: Get copy direction for new window initialization.
#[no_mangle]
pub extern "C" fn rs_copy_direction_for_new_win() -> c_int {
    CopyOptDirection::GlobalToLocal.to_c_int()
}

/// FFI: Get copy direction for :setglobal from local.
#[no_mangle]
pub extern "C" fn rs_copy_direction_local_to_global() -> c_int {
    CopyOptDirection::LocalToGlobal.to_c_int()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_command_type() {
        assert_eq!(SetCommandType::Set.to_c_int(), 0);
        assert_eq!(SetCommandType::SetLocal.to_c_int(), 1);
        assert_eq!(SetCommandType::SetGlobal.to_c_int(), 2);

        assert_eq!(SetCommandType::from_c_int(0), SetCommandType::Set);
        assert_eq!(SetCommandType::from_c_int(1), SetCommandType::SetLocal);
        assert_eq!(SetCommandType::from_c_int(2), SetCommandType::SetGlobal);
    }

    #[test]
    fn test_set_command_type_flags() {
        assert_eq!(SetCommandType::Set.to_opt_flags(), 0);
        assert_eq!(
            SetCommandType::SetLocal.to_opt_flags(),
            set_flags::OPT_LOCAL
        );
        assert_eq!(
            SetCommandType::SetGlobal.to_opt_flags(),
            set_flags::OPT_GLOBAL
        );

        assert!(SetCommandType::SetLocal.is_local());
        assert!(!SetCommandType::SetLocal.is_global());
        assert!(SetCommandType::SetGlobal.is_global());
        assert!(!SetCommandType::SetGlobal.is_local());
    }

    #[test]
    fn test_set_arg_type() {
        assert!(!SetArgType::Show.modifies_value());
        assert!(!SetArgType::Query.modifies_value());
        assert!(SetArgType::Assign.modifies_value());
        assert!(SetArgType::Reset.modifies_value());
        assert!(SetArgType::Toggle.modifies_value());

        assert!(!SetArgType::Show.is_compound());
        assert!(SetArgType::Append.is_compound());
        assert!(SetArgType::Prepend.is_compound());
        assert!(SetArgType::Remove.is_compound());
    }

    #[test]
    fn test_show_options_mode() {
        assert_eq!(ShowOptionsMode::from_c_int(0), ShowOptionsMode::Value);
        assert_eq!(ShowOptionsMode::from_c_int(1), ShowOptionsMode::All);
        assert_eq!(ShowOptionsMode::from_c_int(2), ShowOptionsMode::Changed);
        assert_eq!(ShowOptionsMode::from_c_int(3), ShowOptionsMode::Terminal);

        assert_eq!(rs_show_mode_is_all(1), 1);
        assert_eq!(rs_show_mode_is_all(0), 0);
        assert_eq!(rs_show_mode_is_changed(2), 1);
        assert_eq!(rs_show_mode_is_terminal(3), 1);
    }

    #[test]
    fn test_set_result() {
        let success = rs_set_result_success(5, 1);
        assert!(success.errmsg.is_null());
        assert_eq!(success.retval, OK);
        assert_eq!(success.count, 5);
        assert_eq!(success.changed, 1);

        let errmsg = c"test error".as_ptr();
        let fail = rs_set_result_fail(errmsg);
        assert_eq!(fail.errmsg, errmsg);
        assert_eq!(fail.retval, FAIL);
    }

    #[test]
    fn test_copy_direction() {
        assert_eq!(rs_copy_direction_for_new_win(), 0);
        assert_eq!(rs_copy_direction_local_to_global(), 1);
    }

    #[test]
    fn test_resolve_effective_scope() {
        // :setglobal always returns global
        assert_eq!(
            rs_resolve_effective_scope(2, 0x07),
            OptScope::Global as c_int
        );

        // :setlocal with window support returns window
        assert_eq!(rs_resolve_effective_scope(1, 0x04), OptScope::Win as c_int);

        // :setlocal with buffer support returns buffer
        assert_eq!(rs_resolve_effective_scope(1, 0x02), OptScope::Buf as c_int);

        // :set returns global by default
        assert_eq!(
            rs_resolve_effective_scope(0, 0x01),
            OptScope::Global as c_int
        );
    }
}
