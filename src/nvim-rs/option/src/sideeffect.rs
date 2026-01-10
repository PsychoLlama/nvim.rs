//! Option side effect dispatch helpers
//!
//! This module provides helpers for managing option change side effects,
//! including redraw triggers, validation callbacks, and change notifications.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

use crate::{OptFlags, OptScope, OptValType};

// =============================================================================
// Side Effect Categories
// =============================================================================

/// Categories of side effects that options can trigger.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SideEffectCategory {
    /// No side effects
    #[default]
    None = 0,
    /// Redraw required
    Redraw = 1,
    /// Recompute buffer text
    Recompute = 2,
    /// Notify UI
    NotifyUi = 3,
    /// Execute callback
    Callback = 4,
    /// Trigger autocommand
    Autocommand = 5,
}

impl SideEffectCategory {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Redraw,
            2 => Self::Recompute,
            3 => Self::NotifyUi,
            4 => Self::Callback,
            5 => Self::Autocommand,
            _ => Self::None,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Redraw Flags
// =============================================================================

/// Redraw flags derived from option flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RedrawFlags {
    /// Redraw status line
    pub statusline: bool,
    /// Redraw tabline
    pub tabline: bool,
    /// Redraw current window
    pub current_win: bool,
    /// Redraw current buffer
    pub current_buf: bool,
    /// Redraw all windows
    pub all_windows: bool,
    /// Clear and redraw
    pub clear: bool,
    /// Update cursor position
    pub cursor: bool,
}

impl RedrawFlags {
    /// Create from OptFlags.
    #[must_use]
    pub const fn from_opt_flags(flags: OptFlags) -> Self {
        Self {
            statusline: flags.contains(OptFlags::REDR_STAT),
            tabline: flags.contains(OptFlags::REDR_TABL),
            current_win: flags.contains(OptFlags::REDR_WIN),
            current_buf: flags.contains(OptFlags::REDR_BUF),
            all_windows: flags.contains(OptFlags::REDR_ALL),
            clear: flags.contains(OptFlags::REDR_CLEAR),
            cursor: flags.contains(OptFlags::CURSWANT),
        }
    }

    /// Check if any redraw is needed.
    #[must_use]
    pub const fn needs_redraw(&self) -> bool {
        self.statusline
            || self.tabline
            || self.current_win
            || self.current_buf
            || self.all_windows
            || self.clear
    }

    /// Check if only highlight changes.
    #[must_use]
    pub const fn highlight_only(&self) -> bool {
        !self.needs_redraw() && !self.cursor
    }

    /// Get redraw level (0=none, 1=status, 2=win, 3=all, 4=clear).
    #[must_use]
    pub const fn level(&self) -> c_int {
        if self.clear {
            4
        } else if self.all_windows {
            3
        } else if self.current_win || self.current_buf {
            2
        } else if self.statusline || self.tabline {
            1
        } else {
            0
        }
    }
}

// =============================================================================
// Side Effect Dispatch
// =============================================================================

/// Result of side effect dispatch.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DispatchResult {
    /// Whether dispatch succeeded
    pub success: bool,
    /// Error code if failed
    pub error_code: c_int,
    /// Whether redraw was scheduled
    pub redraw_scheduled: bool,
    /// Whether UI was notified
    pub ui_notified: bool,
}

impl DispatchResult {
    /// Create a success result.
    #[must_use]
    pub const fn ok() -> Self {
        Self {
            success: true,
            error_code: 0,
            redraw_scheduled: false,
            ui_notified: false,
        }
    }

    /// Create a success result with redraw.
    #[must_use]
    pub const fn with_redraw() -> Self {
        Self {
            success: true,
            error_code: 0,
            redraw_scheduled: true,
            ui_notified: false,
        }
    }

    /// Create a success result with UI notification.
    #[must_use]
    pub const fn with_ui() -> Self {
        Self {
            success: true,
            error_code: 0,
            redraw_scheduled: false,
            ui_notified: true,
        }
    }

    /// Create a failure result.
    #[must_use]
    pub const fn fail(code: c_int) -> Self {
        Self {
            success: false,
            error_code: code,
            redraw_scheduled: false,
            ui_notified: false,
        }
    }
}

// =============================================================================
// Change Context
// =============================================================================

/// Context for an option change.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ChangeContext {
    /// The scope where change occurred
    pub scope: c_int,
    /// Whether change is from :set (vs :setlocal)
    pub is_set_cmd: bool,
    /// Whether change is from modeline
    pub is_modeline: bool,
    /// Whether change is from script
    pub is_script: bool,
    /// Whether this is initial setup
    pub is_init: bool,
}

impl ChangeContext {
    /// Create a new change context.
    #[must_use]
    pub const fn new(scope: OptScope) -> Self {
        Self {
            scope: scope as c_int,
            is_set_cmd: false,
            is_modeline: false,
            is_script: false,
            is_init: false,
        }
    }

    /// Create a context for :set command.
    #[must_use]
    pub const fn from_set(scope: OptScope) -> Self {
        Self {
            scope: scope as c_int,
            is_set_cmd: true,
            is_modeline: false,
            is_script: false,
            is_init: false,
        }
    }

    /// Create a context for modeline.
    #[must_use]
    pub const fn from_modeline() -> Self {
        Self {
            scope: OptScope::Buf as c_int,
            is_set_cmd: false,
            is_modeline: true,
            is_script: false,
            is_init: false,
        }
    }

    /// Create a context for initialization.
    #[must_use]
    pub const fn init() -> Self {
        Self {
            scope: OptScope::Global as c_int,
            is_set_cmd: false,
            is_modeline: false,
            is_script: false,
            is_init: true,
        }
    }

    /// Get the scope.
    #[must_use]
    pub const fn get_scope(&self) -> OptScope {
        match self.scope {
            1 => OptScope::Win,
            2 => OptScope::Buf,
            _ => OptScope::Global,
        }
    }

    /// Check if side effects should be suppressed.
    #[must_use]
    pub const fn suppress_effects(&self) -> bool {
        self.is_init
    }
}

// =============================================================================
// Side Effect Flags
// =============================================================================

/// Flags controlling which side effects to execute.
pub mod effect_flags {
    use std::ffi::c_int;

    /// Execute redraw side effects
    pub const EFF_REDRAW: c_int = 0x01;
    /// Execute callback side effects
    pub const EFF_CALLBACK: c_int = 0x02;
    /// Execute UI notification
    pub const EFF_UI: c_int = 0x04;
    /// Execute autocommand
    pub const EFF_AUTOCMD: c_int = 0x08;
    /// Execute all side effects
    pub const EFF_ALL: c_int = EFF_REDRAW | EFF_CALLBACK | EFF_UI | EFF_AUTOCMD;
    /// Execute no side effects
    pub const EFF_NONE: c_int = 0;
}

/// Check if effect flags include a specific effect.
#[must_use]
#[inline]
pub const fn has_effect(flags: c_int, effect: c_int) -> bool {
    (flags & effect) != 0
}

// =============================================================================
// Change Event
// =============================================================================

/// Event describing an option change.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ChangeEvent {
    /// Option index (or -1 for unknown)
    pub opt_idx: c_int,
    /// Option type
    pub opt_type: c_int,
    /// Scope of change
    pub scope: c_int,
    /// Old value was nil/unset
    pub old_was_nil: bool,
    /// New value is nil/unset
    pub new_is_nil: bool,
    /// Whether value actually changed
    pub value_changed: bool,
}

impl ChangeEvent {
    /// Create a new change event.
    #[must_use]
    pub const fn new(opt_idx: c_int, opt_type: OptValType, scope: OptScope) -> Self {
        Self {
            opt_idx,
            opt_type: opt_type as c_int,
            scope: scope as c_int,
            old_was_nil: false,
            new_is_nil: false,
            value_changed: true,
        }
    }

    /// Check if this is a meaningful change.
    #[must_use]
    pub const fn is_meaningful(&self) -> bool {
        self.value_changed && !self.new_is_nil
    }

    /// Get the option type.
    #[must_use]
    pub const fn get_type(&self) -> OptValType {
        match self.opt_type {
            0 => OptValType::Boolean,
            1 => OptValType::Number,
            2 => OptValType::String,
            _ => OptValType::Nil,
        }
    }
}

// =============================================================================
// Callback Registry Types
// =============================================================================

/// Types of option change callbacks.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackType {
    /// Called before change (can reject)
    PreChange = 0,
    /// Called after change
    PostChange = 1,
    /// Called to validate new value
    Validate = 2,
    /// Called to expand value
    Expand = 3,
}

impl CallbackType {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::PreChange,
            1 => Self::PostChange,
            2 => Self::Validate,
            _ => Self::Expand,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

/// State for tracking registered callbacks.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CallbackState {
    /// Number of pre-change callbacks
    pub pre_count: c_int,
    /// Number of post-change callbacks
    pub post_count: c_int,
    /// Number of validate callbacks
    pub validate_count: c_int,
    /// Whether callbacks are enabled
    pub enabled: bool,
}

impl CallbackState {
    /// Create a new callback state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pre_count: 0,
            post_count: 0,
            validate_count: 0,
            enabled: true,
        }
    }

    /// Check if there are any callbacks.
    #[must_use]
    pub const fn has_callbacks(&self) -> bool {
        self.pre_count > 0 || self.post_count > 0 || self.validate_count > 0
    }

    /// Check if callbacks are active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.enabled && self.has_callbacks()
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get redraw level from option flags.
#[unsafe(no_mangle)]
pub extern "C" fn rs_redraw_level(flags: u32) -> c_int {
    RedrawFlags::from_opt_flags(OptFlags(flags)).level()
}

/// Check if redraw is needed for option flags.
#[unsafe(no_mangle)]
pub extern "C" fn rs_needs_redraw(flags: u32) -> c_int {
    c_int::from(RedrawFlags::from_opt_flags(OptFlags(flags)).needs_redraw())
}

/// Get side effect category from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_side_effect_category(value: c_int) -> c_int {
    SideEffectCategory::from_raw(value).to_raw()
}

/// Get callback type from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_callback_type(value: c_int) -> c_int {
    CallbackType::from_raw(value).to_raw()
}

/// Check if effect flags include a specific effect.
#[unsafe(no_mangle)]
pub extern "C" fn rs_has_effect(flags: c_int, effect: c_int) -> c_int {
    c_int::from(has_effect(flags, effect))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_effect_category() {
        assert_eq!(SideEffectCategory::from_raw(0), SideEffectCategory::None);
        assert_eq!(SideEffectCategory::from_raw(1), SideEffectCategory::Redraw);
        assert_eq!(
            SideEffectCategory::from_raw(5),
            SideEffectCategory::Autocommand
        );
        assert_eq!(SideEffectCategory::from_raw(99), SideEffectCategory::None);
    }

    #[test]
    fn test_redraw_flags() {
        // Test with REDR_STAT
        let flags = OptFlags::REDR_STAT;
        let redraw = RedrawFlags::from_opt_flags(flags);
        assert!(redraw.statusline);
        assert!(!redraw.tabline);
        assert!(redraw.needs_redraw());
        assert_eq!(redraw.level(), 1);

        // Test with REDR_ALL
        let flags = OptFlags::REDR_ALL;
        let redraw = RedrawFlags::from_opt_flags(flags);
        assert!(redraw.all_windows);
        assert_eq!(redraw.level(), 3);

        // Test with REDR_CLEAR
        let flags = OptFlags::REDR_CLEAR;
        let redraw = RedrawFlags::from_opt_flags(flags);
        assert!(redraw.clear);
        assert_eq!(redraw.level(), 4);
    }

    #[test]
    fn test_dispatch_result() {
        let ok = DispatchResult::ok();
        assert!(ok.success);
        assert_eq!(ok.error_code, 0);

        let with_redraw = DispatchResult::with_redraw();
        assert!(with_redraw.redraw_scheduled);

        let fail = DispatchResult::fail(42);
        assert!(!fail.success);
        assert_eq!(fail.error_code, 42);
    }

    #[test]
    fn test_change_context() {
        let ctx = ChangeContext::new(OptScope::Win);
        assert_eq!(ctx.get_scope(), OptScope::Win);
        assert!(!ctx.suppress_effects());

        let modeline = ChangeContext::from_modeline();
        assert!(modeline.is_modeline);

        let init = ChangeContext::init();
        assert!(init.is_init);
        assert!(init.suppress_effects());
    }

    #[test]
    fn test_effect_flags() {
        assert!(has_effect(effect_flags::EFF_ALL, effect_flags::EFF_REDRAW));
        assert!(has_effect(effect_flags::EFF_ALL, effect_flags::EFF_UI));
        assert!(!has_effect(effect_flags::EFF_REDRAW, effect_flags::EFF_UI));
        assert!(!has_effect(effect_flags::EFF_NONE, effect_flags::EFF_REDRAW));
    }

    #[test]
    fn test_change_event() {
        let event = ChangeEvent::new(5, OptValType::Boolean, OptScope::Global);
        assert_eq!(event.opt_idx, 5);
        assert_eq!(event.get_type(), OptValType::Boolean);
        assert!(event.is_meaningful());

        let mut event2 = event;
        event2.new_is_nil = true;
        assert!(!event2.is_meaningful());
    }

    #[test]
    fn test_callback_type() {
        assert_eq!(CallbackType::from_raw(0), CallbackType::PreChange);
        assert_eq!(CallbackType::from_raw(1), CallbackType::PostChange);
        assert_eq!(CallbackType::from_raw(2), CallbackType::Validate);
    }

    #[test]
    fn test_callback_state() {
        let mut state = CallbackState::new();
        assert!(!state.has_callbacks());
        assert!(!state.is_active());

        state.post_count = 1;
        assert!(state.has_callbacks());
        assert!(state.is_active());

        state.enabled = false;
        assert!(!state.is_active());
    }
}
