//! Option scope resolution helpers
//!
//! This module provides helpers for resolving option scopes,
//! determining effective values, and managing scope inheritance.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::match_same_arms)]

use std::ffi::c_int;

use crate::OptScope;

// =============================================================================
// Scope Priority
// =============================================================================

/// Priority level for option scopes (higher = more specific).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopePriority {
    /// Global scope (lowest priority)
    Global = 0,
    /// Buffer-local scope
    Buffer = 1,
    /// Window-local scope (highest priority)
    Window = 2,
}

impl ScopePriority {
    /// Convert from OptScope.
    #[must_use]
    pub const fn from_scope(scope: OptScope) -> Self {
        match scope {
            OptScope::Global => Self::Global,
            OptScope::Buf => Self::Buffer,
            OptScope::Win => Self::Window,
        }
    }

    /// Convert to OptScope.
    #[must_use]
    pub const fn to_scope(self) -> OptScope {
        match self {
            Self::Global => OptScope::Global,
            Self::Buffer => OptScope::Buf,
            Self::Window => OptScope::Win,
        }
    }

    /// Check if this scope is more specific than another.
    #[must_use]
    pub const fn is_more_specific(self, other: Self) -> bool {
        (self as i32) > (other as i32)
    }
}

// =============================================================================
// Scope Support Flags
// =============================================================================

/// Flags indicating which scopes an option supports.
pub mod scope_flags {
    use std::ffi::c_int;

    /// Option supports global scope
    pub const SCOPE_GLOBAL: c_int = 0x01;
    /// Option supports buffer-local scope
    pub const SCOPE_BUFFER: c_int = 0x02;
    /// Option supports window-local scope
    pub const SCOPE_WINDOW: c_int = 0x04;
    /// Option is global-only
    pub const SCOPE_GLOBAL_ONLY: c_int = SCOPE_GLOBAL;
    /// Option is buffer-local
    pub const SCOPE_BUFFER_LOCAL: c_int = SCOPE_GLOBAL | SCOPE_BUFFER;
    /// Option is window-local
    pub const SCOPE_WINDOW_LOCAL: c_int = SCOPE_GLOBAL | SCOPE_WINDOW;
}

/// Check if scope flags support a specific scope.
#[must_use]
#[inline]
pub const fn supports_scope(flags: c_int, scope: OptScope) -> bool {
    let scope_bit = match scope {
        OptScope::Global => scope_flags::SCOPE_GLOBAL,
        OptScope::Buf => scope_flags::SCOPE_BUFFER,
        OptScope::Win => scope_flags::SCOPE_WINDOW,
    };
    (flags & scope_bit) != 0
}

/// Get the most specific scope supported by an option.
#[must_use]
pub const fn most_specific_scope(flags: c_int) -> OptScope {
    if (flags & scope_flags::SCOPE_WINDOW) != 0 {
        OptScope::Win
    } else if (flags & scope_flags::SCOPE_BUFFER) != 0 {
        OptScope::Buf
    } else {
        OptScope::Global
    }
}

// =============================================================================
// Scope Resolution State
// =============================================================================

/// State for resolving an option's effective value.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ScopeResolution {
    /// The scope from which the value came
    pub resolved_scope: c_int,
    /// Whether a local value was used
    pub used_local: bool,
    /// Whether the value was inherited from global
    pub inherited: bool,
    /// Whether the option is set at this scope
    pub is_set: bool,
}

impl ScopeResolution {
    /// Create a global resolution.
    #[must_use]
    pub const fn global() -> Self {
        Self {
            resolved_scope: OptScope::Global as c_int,
            used_local: false,
            inherited: false,
            is_set: true,
        }
    }

    /// Create a local resolution.
    #[must_use]
    pub const fn local(scope: OptScope) -> Self {
        Self {
            resolved_scope: scope as c_int,
            used_local: true,
            inherited: false,
            is_set: true,
        }
    }

    /// Create an inherited resolution.
    #[must_use]
    pub const fn inherited(from_scope: OptScope) -> Self {
        Self {
            resolved_scope: from_scope as c_int,
            used_local: false,
            inherited: true,
            is_set: false,
        }
    }

    /// Get the resolved scope.
    #[must_use]
    pub const fn get_scope(&self) -> OptScope {
        match self.resolved_scope {
            1 => OptScope::Win,
            2 => OptScope::Buf,
            _ => OptScope::Global,
        }
    }
}

// =============================================================================
// Scope-Specific Value State
// =============================================================================

/// Tracks whether an option has values set at different scopes.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ScopeValueState {
    /// Whether global value is set
    pub global_set: bool,
    /// Whether buffer-local value is set
    pub buffer_set: bool,
    /// Whether window-local value is set
    pub window_set: bool,
}

impl ScopeValueState {
    /// Create a new state with no values set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            global_set: false,
            buffer_set: false,
            window_set: false,
        }
    }

    /// Check if any value is set.
    #[must_use]
    pub const fn any_set(&self) -> bool {
        self.global_set || self.buffer_set || self.window_set
    }

    /// Check if a specific scope is set.
    #[must_use]
    pub const fn is_set(&self, scope: OptScope) -> bool {
        match scope {
            OptScope::Global => self.global_set,
            OptScope::Buf => self.buffer_set,
            OptScope::Win => self.window_set,
        }
    }

    /// Mark a scope as set.
    pub fn set(&mut self, scope: OptScope) {
        match scope {
            OptScope::Global => self.global_set = true,
            OptScope::Buf => self.buffer_set = true,
            OptScope::Win => self.window_set = true,
        }
    }

    /// Clear a scope.
    pub fn clear(&mut self, scope: OptScope) {
        match scope {
            OptScope::Global => self.global_set = false,
            OptScope::Buf => self.buffer_set = false,
            OptScope::Win => self.window_set = false,
        }
    }

    /// Get the most specific set scope.
    #[must_use]
    pub const fn most_specific_set(&self) -> Option<OptScope> {
        if self.window_set {
            Some(OptScope::Win)
        } else if self.buffer_set {
            Some(OptScope::Buf)
        } else if self.global_set {
            Some(OptScope::Global)
        } else {
            None
        }
    }
}

// =============================================================================
// Scope Copy Direction
// =============================================================================

/// Direction for copying option values between scopes.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyDirection {
    /// Copy from global to local
    GlobalToLocal = 0,
    /// Copy from local to global
    LocalToGlobal = 1,
    /// Copy from buffer to window
    BufToWin = 2,
    /// Copy from window to buffer
    WinToBuf = 3,
}

impl CopyDirection {
    /// Get the source scope.
    #[must_use]
    pub const fn source(&self) -> OptScope {
        match self {
            Self::GlobalToLocal => OptScope::Global,
            Self::LocalToGlobal => OptScope::Win, // or Buf, depends on option
            Self::BufToWin => OptScope::Buf,
            Self::WinToBuf => OptScope::Win,
        }
    }

    /// Get the destination scope.
    #[must_use]
    pub const fn dest(&self) -> OptScope {
        match self {
            Self::GlobalToLocal => OptScope::Win, // or Buf, depends on option
            Self::LocalToGlobal => OptScope::Global,
            Self::BufToWin => OptScope::Win,
            Self::WinToBuf => OptScope::Buf,
        }
    }
}

// =============================================================================
// Effective Scope Calculator
// =============================================================================

/// Result of calculating effective scope.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EffectiveScope {
    /// The effective scope to use
    pub scope: c_int,
    /// Whether to use setlocal behavior
    pub use_local: bool,
    /// Whether global should also be updated
    pub update_global: bool,
}

impl EffectiveScope {
    /// Calculate effective scope for getting a value.
    #[must_use]
    pub const fn for_get(supported: c_int, requested: OptScope) -> Self {
        // If requested scope is supported, use it
        if supports_scope(supported, requested) {
            Self {
                scope: requested as c_int,
                use_local: !matches!(requested, OptScope::Global),
                update_global: false,
            }
        } else {
            // Fall back to global
            Self {
                scope: OptScope::Global as c_int,
                use_local: false,
                update_global: false,
            }
        }
    }

    /// Calculate effective scope for setting a value.
    #[must_use]
    pub const fn for_set(supported: c_int, requested: OptScope, is_setlocal: bool) -> Self {
        if is_setlocal {
            // :setlocal - only set local value
            let scope = if supports_scope(supported, requested) {
                requested
            } else {
                OptScope::Global
            };
            Self {
                scope: scope as c_int,
                use_local: true,
                update_global: false,
            }
        } else {
            // :set - set both local and global for local options
            let scope = most_specific_scope(supported);
            Self {
                scope: scope as c_int,
                use_local: !matches!(scope, OptScope::Global),
                update_global: true,
            }
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if scope flags support a specific scope.
#[unsafe(no_mangle)]
pub extern "C" fn rs_supports_scope(flags: c_int, scope: c_int) -> c_int {
    let opt_scope = match scope {
        1 => OptScope::Win,
        2 => OptScope::Buf,
        _ => OptScope::Global,
    };
    c_int::from(supports_scope(flags, opt_scope))
}

/// Get the most specific scope supported.
#[unsafe(no_mangle)]
pub extern "C" fn rs_most_specific_scope(flags: c_int) -> c_int {
    most_specific_scope(flags) as c_int
}

/// Get scope priority.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_priority(scope: c_int) -> c_int {
    let opt_scope = match scope {
        1 => OptScope::Win,
        2 => OptScope::Buf,
        _ => OptScope::Global,
    };
    ScopePriority::from_scope(opt_scope) as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_priority() {
        assert!(ScopePriority::Window.is_more_specific(ScopePriority::Buffer));
        assert!(ScopePriority::Buffer.is_more_specific(ScopePriority::Global));
        assert!(!ScopePriority::Global.is_more_specific(ScopePriority::Window));
    }

    #[test]
    fn test_scope_flags() {
        let flags = scope_flags::SCOPE_BUFFER_LOCAL;
        assert!(supports_scope(flags, OptScope::Global));
        assert!(supports_scope(flags, OptScope::Buf));
        assert!(!supports_scope(flags, OptScope::Win));

        let flags = scope_flags::SCOPE_WINDOW_LOCAL;
        assert!(supports_scope(flags, OptScope::Global));
        assert!(!supports_scope(flags, OptScope::Buf));
        assert!(supports_scope(flags, OptScope::Win));
    }

    #[test]
    fn test_most_specific_scope() {
        assert_eq!(
            most_specific_scope(scope_flags::SCOPE_GLOBAL_ONLY),
            OptScope::Global
        );
        assert_eq!(
            most_specific_scope(scope_flags::SCOPE_BUFFER_LOCAL),
            OptScope::Buf
        );
        assert_eq!(
            most_specific_scope(scope_flags::SCOPE_WINDOW_LOCAL),
            OptScope::Win
        );
    }

    #[test]
    fn test_scope_resolution() {
        let global = ScopeResolution::global();
        assert!(!global.used_local);
        assert!(!global.inherited);
        assert!(global.is_set);

        let local = ScopeResolution::local(OptScope::Win);
        assert!(local.used_local);
        assert!(!local.inherited);

        let inherited = ScopeResolution::inherited(OptScope::Global);
        assert!(!inherited.used_local);
        assert!(inherited.inherited);
        assert!(!inherited.is_set);
    }

    #[test]
    fn test_scope_value_state() {
        let mut state = ScopeValueState::new();
        assert!(!state.any_set());

        state.set(OptScope::Global);
        assert!(state.is_set(OptScope::Global));
        assert!(!state.is_set(OptScope::Win));

        state.set(OptScope::Win);
        assert_eq!(state.most_specific_set(), Some(OptScope::Win));

        state.clear(OptScope::Win);
        assert_eq!(state.most_specific_set(), Some(OptScope::Global));
    }

    #[test]
    fn test_copy_direction() {
        assert_eq!(CopyDirection::GlobalToLocal.source(), OptScope::Global);
        assert_eq!(CopyDirection::LocalToGlobal.dest(), OptScope::Global);
        assert_eq!(CopyDirection::BufToWin.source(), OptScope::Buf);
        assert_eq!(CopyDirection::BufToWin.dest(), OptScope::Win);
    }

    #[test]
    fn test_effective_scope() {
        // Global-only option
        let eff = EffectiveScope::for_get(scope_flags::SCOPE_GLOBAL_ONLY, OptScope::Win);
        assert_eq!(eff.scope, OptScope::Global as c_int);

        // Buffer-local option, requesting buffer
        let eff = EffectiveScope::for_get(scope_flags::SCOPE_BUFFER_LOCAL, OptScope::Buf);
        assert_eq!(eff.scope, OptScope::Buf as c_int);
        assert!(eff.use_local);

        // :set on buffer-local option (updates both)
        let eff = EffectiveScope::for_set(scope_flags::SCOPE_BUFFER_LOCAL, OptScope::Buf, false);
        assert!(eff.update_global);

        // :setlocal on buffer-local option (local only)
        let eff = EffectiveScope::for_set(scope_flags::SCOPE_BUFFER_LOCAL, OptScope::Buf, true);
        assert!(!eff.update_global);
    }
}
