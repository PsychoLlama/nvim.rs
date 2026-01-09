//! User command execution handling
//!
//! This module provides Rust implementations for user command execution,
//! including execution context, modifiers, and result handling.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_int;

/// Line number type
type LinenrT = i32;

// =============================================================================
// Execution Modifiers
// =============================================================================

/// Command modifier flags
pub const MOD_SILENT: u32 = 0x0001;
pub const MOD_VERTICAL: u32 = 0x0002;
pub const MOD_HORIZONTAL: u32 = 0x0004;
pub const MOD_TOPLEFT: u32 = 0x0008;
pub const MOD_BOTRIGHT: u32 = 0x0010;
pub const MOD_LEFTABOVE: u32 = 0x0020;
pub const MOD_RIGHTBELOW: u32 = 0x0040;
pub const MOD_TAB: u32 = 0x0080;
pub const MOD_CONFIRM: u32 = 0x0100;
pub const MOD_KEEPALT: u32 = 0x0200;
pub const MOD_KEEPJUMPS: u32 = 0x0400;
pub const MOD_KEEPMARKS: u32 = 0x0800;
pub const MOD_KEEPPATTERNS: u32 = 0x1000;
pub const MOD_LOCKMARKS: u32 = 0x2000;
pub const MOD_NOAUTOCMD: u32 = 0x4000;
pub const MOD_NOSWAPFILE: u32 = 0x8000;
pub const MOD_HIDE: u32 = 0x10000;
pub const MOD_BROWSE: u32 = 0x20000;

/// Command modifiers wrapper
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CmdModifiers {
    flags: u32,
    /// Tab page number for :tab
    tab: c_int,
    /// Count before modifier
    count: c_int,
}

impl CmdModifiers {
    /// Create with no modifiers
    pub const fn none() -> Self {
        Self {
            flags: 0,
            tab: 0,
            count: 0,
        }
    }

    /// Create from raw flags
    pub const fn from_raw(flags: u32) -> Self {
        Self {
            flags,
            tab: 0,
            count: 0,
        }
    }

    /// Get raw flags
    pub const fn as_raw(self) -> u32 {
        self.flags
    }

    /// Check if silent
    pub const fn is_silent(self) -> bool {
        (self.flags & MOD_SILENT) != 0
    }

    /// Check if vertical split
    pub const fn is_vertical(self) -> bool {
        (self.flags & MOD_VERTICAL) != 0
    }

    /// Check if horizontal split
    pub const fn is_horizontal(self) -> bool {
        (self.flags & MOD_HORIZONTAL) != 0
    }

    /// Check if top-left position
    pub const fn is_topleft(self) -> bool {
        (self.flags & MOD_TOPLEFT) != 0
    }

    /// Check if bottom-right position
    pub const fn is_botright(self) -> bool {
        (self.flags & MOD_BOTRIGHT) != 0
    }

    /// Check if new tab
    pub const fn is_tab(self) -> bool {
        (self.flags & MOD_TAB) != 0
    }

    /// Check if confirm mode
    pub const fn is_confirm(self) -> bool {
        (self.flags & MOD_CONFIRM) != 0
    }

    /// Check if keepalt
    pub const fn is_keepalt(self) -> bool {
        (self.flags & MOD_KEEPALT) != 0
    }

    /// Check if keepjumps
    pub const fn is_keepjumps(self) -> bool {
        (self.flags & MOD_KEEPJUMPS) != 0
    }

    /// Check if noautocmd
    pub const fn is_noautocmd(self) -> bool {
        (self.flags & MOD_NOAUTOCMD) != 0
    }

    /// Check if browse mode
    pub const fn is_browse(self) -> bool {
        (self.flags & MOD_BROWSE) != 0
    }

    /// Get tab number
    pub const fn tab_number(self) -> c_int {
        self.tab
    }

    /// Set silent flag
    pub fn set_silent(&mut self, value: bool) {
        if value {
            self.flags |= MOD_SILENT;
        } else {
            self.flags &= !MOD_SILENT;
        }
    }

    /// Set vertical flag
    pub fn set_vertical(&mut self, value: bool) {
        if value {
            self.flags |= MOD_VERTICAL;
        } else {
            self.flags &= !MOD_VERTICAL;
        }
    }
}

// =============================================================================
// Execution Context
// =============================================================================

/// Context for user command execution
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExecContext {
    /// Command modifiers
    pub modifiers: CmdModifiers,
    /// First line of range
    pub line1: LinenrT,
    /// Last line of range
    pub line2: LinenrT,
    /// Whether range was given
    pub range_given: bool,
    /// Whether bang (!) was used
    pub bang: bool,
    /// Count value (-1 if not given)
    pub count: c_int,
    /// Register name (0 if not given)
    pub reg: u8,
}

impl Default for ExecContext {
    fn default() -> Self {
        Self {
            modifiers: CmdModifiers::none(),
            line1: 1,
            line2: 1,
            range_given: false,
            bang: false,
            count: -1,
            reg: 0,
        }
    }
}

impl ExecContext {
    /// Create a new execution context
    pub const fn new() -> Self {
        Self {
            modifiers: CmdModifiers::none(),
            line1: 1,
            line2: 1,
            range_given: false,
            bang: false,
            count: -1,
            reg: 0,
        }
    }

    /// Check if a range was given
    pub const fn has_range(&self) -> bool {
        self.range_given
    }

    /// Check if a count was given
    pub const fn has_count(&self) -> bool {
        self.count >= 0
    }

    /// Check if a register was given
    pub const fn has_register(&self) -> bool {
        self.reg != 0
    }

    /// Get the number of lines in the range
    pub const fn line_count(&self) -> LinenrT {
        if self.range_given {
            self.line2 - self.line1 + 1
        } else {
            1
        }
    }

    /// Check if this is a single line range
    pub const fn is_single_line(&self) -> bool {
        self.line1 == self.line2
    }
}

// =============================================================================
// Execution Result
// =============================================================================

/// Result of command execution
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecResult {
    /// Command executed successfully
    Success = 0,
    /// Command failed
    Failure = 1,
    /// Command was interrupted
    Interrupted = 2,
    /// Command not found
    NotFound = 3,
    /// Invalid arguments
    InvalidArgs = 4,
    /// Permission denied
    Permission = 5,
    /// Range error
    RangeError = 6,
    /// Command is disabled
    Disabled = 7,
}

impl ExecResult {
    /// Check if execution was successful
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if execution failed
    pub const fn is_err(self) -> bool {
        !self.is_ok()
    }

    /// Convert to raw integer
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Create from raw integer
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::Success,
            1 => Self::Failure,
            2 => Self::Interrupted,
            3 => Self::NotFound,
            4 => Self::InvalidArgs,
            5 => Self::Permission,
            6 => Self::RangeError,
            7 => Self::Disabled,
            _ => Self::Failure,
        }
    }
}

// =============================================================================
// Execution State
// =============================================================================

/// State during command execution
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExecState {
    /// Whether currently executing
    pub executing: bool,
    /// Nesting level (for recursive commands)
    pub level: c_int,
    /// Whether output is being captured
    pub capturing: bool,
    /// Whether errors should be suppressed
    pub silent_errors: bool,
}

impl Default for ExecState {
    fn default() -> Self {
        Self {
            executing: false,
            level: 0,
            capturing: false,
            silent_errors: false,
        }
    }
}

impl ExecState {
    /// Check if at top level
    pub const fn is_top_level(&self) -> bool {
        self.level == 0
    }

    /// Check if nested
    pub const fn is_nested(&self) -> bool {
        self.level > 0
    }
}

// =============================================================================
// Special Values for <q-args>, <f-args>, etc.
// =============================================================================

/// Special argument expansion type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialArg {
    /// <args> - raw arguments
    Args = 0,
    /// <q-args> - quoted arguments
    QArgs = 1,
    /// <f-args> - function arguments (split)
    FArgs = 2,
    /// <bang> - bang (!)
    Bang = 3,
    /// <line1> - first line
    Line1 = 4,
    /// <line2> - last line
    Line2 = 5,
    /// <count> - count value
    Count = 6,
    /// <reg> - register
    Reg = 7,
    /// <mods> - modifiers
    Mods = 8,
    /// <lt> - literal <
    Lt = 9,
}

impl SpecialArg {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Args),
            1 => Some(Self::QArgs),
            2 => Some(Self::FArgs),
            3 => Some(Self::Bang),
            4 => Some(Self::Line1),
            5 => Some(Self::Line2),
            6 => Some(Self::Count),
            7 => Some(Self::Reg),
            8 => Some(Self::Mods),
            9 => Some(Self::Lt),
            _ => None,
        }
    }

    /// Get the placeholder name (without <>)
    pub const fn name(self) -> &'static str {
        match self {
            Self::Args => "args",
            Self::QArgs => "q-args",
            Self::FArgs => "f-args",
            Self::Bang => "bang",
            Self::Line1 => "line1",
            Self::Line2 => "line2",
            Self::Count => "count",
            Self::Reg => "reg",
            Self::Mods => "mods",
            Self::Lt => "lt",
        }
    }

    /// Check if this requires the execution context
    pub const fn needs_context(self) -> bool {
        !matches!(self, Self::Lt)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if modifiers is silent
#[no_mangle]
pub extern "C" fn rs_usercmd_mods_is_silent(flags: u32) -> c_int {
    c_int::from(CmdModifiers::from_raw(flags).is_silent())
}

/// FFI export: Check if modifiers is vertical
#[no_mangle]
pub extern "C" fn rs_usercmd_mods_is_vertical(flags: u32) -> c_int {
    c_int::from(CmdModifiers::from_raw(flags).is_vertical())
}

/// FFI export: Check if modifiers is tab
#[no_mangle]
pub extern "C" fn rs_usercmd_mods_is_tab(flags: u32) -> c_int {
    c_int::from(CmdModifiers::from_raw(flags).is_tab())
}

/// FFI export: Check if modifiers is noautocmd
#[no_mangle]
pub extern "C" fn rs_usercmd_mods_is_noautocmd(flags: u32) -> c_int {
    c_int::from(CmdModifiers::from_raw(flags).is_noautocmd())
}

/// FFI export: Create default execution context
#[no_mangle]
pub extern "C" fn rs_usercmd_exec_context_new() -> ExecContext {
    ExecContext::new()
}

/// FFI export: Get line count from context
#[no_mangle]
pub extern "C" fn rs_usercmd_exec_line_count(ctx: *const ExecContext) -> LinenrT {
    if ctx.is_null() {
        return 1;
    }
    unsafe { (*ctx).line_count() }
}

/// FFI export: Check if context has range
#[no_mangle]
pub extern "C" fn rs_usercmd_exec_has_range(ctx: *const ExecContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*ctx).has_range() })
}

/// FFI export: Check if result is ok
#[no_mangle]
pub extern "C" fn rs_usercmd_exec_result_is_ok(result: c_int) -> c_int {
    c_int::from(ExecResult::from_raw(result).is_ok())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_modifiers() {
        let mods = CmdModifiers::none();
        assert!(!mods.is_silent());
        assert!(!mods.is_vertical());

        let mods = CmdModifiers::from_raw(MOD_SILENT | MOD_VERTICAL);
        assert!(mods.is_silent());
        assert!(mods.is_vertical());
        assert!(!mods.is_tab());
    }

    #[test]
    fn test_cmd_modifiers_set() {
        let mut mods = CmdModifiers::none();
        mods.set_silent(true);
        assert!(mods.is_silent());

        mods.set_vertical(true);
        assert!(mods.is_vertical());

        mods.set_silent(false);
        assert!(!mods.is_silent());
    }

    #[test]
    fn test_exec_context() {
        let ctx = ExecContext::new();
        assert!(!ctx.has_range());
        assert!(!ctx.has_count());
        assert!(!ctx.has_register());
        assert_eq!(ctx.line_count(), 1);

        let ctx = ExecContext {
            line1: 10,
            line2: 20,
            range_given: true,
            ..Default::default()
        };
        assert!(ctx.has_range());
        assert_eq!(ctx.line_count(), 11);
        assert!(!ctx.is_single_line());
    }

    #[test]
    fn test_exec_result() {
        assert!(ExecResult::Success.is_ok());
        assert!(!ExecResult::Success.is_err());

        assert!(!ExecResult::Failure.is_ok());
        assert!(ExecResult::Failure.is_err());

        assert_eq!(ExecResult::from_raw(0), ExecResult::Success);
        assert_eq!(ExecResult::from_raw(1), ExecResult::Failure);
        assert_eq!(ExecResult::from_raw(100), ExecResult::Failure);
    }

    #[test]
    fn test_exec_state() {
        let state = ExecState::default();
        assert!(state.is_top_level());
        assert!(!state.is_nested());

        let nested = ExecState {
            level: 2,
            ..Default::default()
        };
        assert!(!nested.is_top_level());
        assert!(nested.is_nested());
    }

    #[test]
    fn test_special_arg() {
        assert_eq!(SpecialArg::from_raw(0), Some(SpecialArg::Args));
        assert_eq!(SpecialArg::from_raw(100), None);

        assert!(SpecialArg::Args.needs_context());
        assert!(!SpecialArg::Lt.needs_context());

        assert_eq!(SpecialArg::QArgs.name(), "q-args");
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_usercmd_mods_is_silent(MOD_SILENT), 1);
        assert_eq!(rs_usercmd_mods_is_silent(0), 0);

        assert_eq!(rs_usercmd_exec_result_is_ok(0), 1);
        assert_eq!(rs_usercmd_exec_result_is_ok(1), 0);
    }
}
