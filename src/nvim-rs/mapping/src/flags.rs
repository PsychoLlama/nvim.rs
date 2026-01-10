//! Mapping flags and attributes helpers
//!
//! This module provides helpers for working with mapping flags,
//! including noremap, silent, expr, and other mapping attributes.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

// =============================================================================
// Mapping Flags
// =============================================================================

/// Flags for mapping attributes.
pub mod map_flags {
    use std::ffi::c_int;

    /// Mapping is noremap (don't remap RHS)
    pub const MAP_NOREMAP: c_int = 0x01;
    /// Mapping is silent (no command echo)
    pub const MAP_SILENT: c_int = 0x02;
    /// Mapping is an expression (evaluate RHS)
    pub const MAP_EXPR: c_int = 0x04;
    /// Mapping has nowait flag (don't wait for more chars)
    pub const MAP_NOWAIT: c_int = 0x08;
    /// Mapping uses Lua callback
    pub const MAP_LUA: c_int = 0x10;
    /// Mapping should replace keycodes in expr result
    pub const MAP_REPLACE_KEYCODES: c_int = 0x20;
    /// Mapping is a special (internal) mapping
    pub const MAP_SPECIAL: c_int = 0x40;
    /// Mapping was simplified (key sequences)
    pub const MAP_SIMPLIFIED: c_int = 0x80;
    /// Mapping is script-local
    pub const MAP_SCRIPT: c_int = 0x100;
    /// Mapping is unique (don't allow override)
    pub const MAP_UNIQUE: c_int = 0x200;
}

/// Check if mapping flags have a specific flag set.
#[must_use]
#[inline]
pub const fn has_map_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set a mapping flag.
#[must_use]
#[inline]
pub const fn set_map_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear a mapping flag.
#[must_use]
#[inline]
pub const fn clear_map_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

// =============================================================================
// Noremap Values
// =============================================================================

/// Noremap type for mappings.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NoremapType {
    /// Normal mapping (allows remapping)
    #[default]
    Remap = 0,
    /// No remapping
    Noremap = 1,
    /// Script-local no remapping
    ScriptNoremap = 2,
    /// Buffer-local no remapping
    BufNoremap = 3,
}

impl NoremapType {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::Remap,
            1 => Self::Noremap,
            2 => Self::ScriptNoremap,
            _ => Self::BufNoremap,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this is any form of noremap.
    #[must_use]
    pub const fn is_noremap(&self) -> bool {
        !matches!(self, Self::Remap)
    }
}

// =============================================================================
// Mapping Attributes
// =============================================================================

/// Complete attributes for a mapping.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MapAttrs {
    /// Combined flags
    pub flags: c_int,
    /// Noremap type
    pub noremap: c_int,
    /// Script ID (for script-local mappings)
    pub script_id: c_int,
    /// Lua reference (or LUA_NOREF)
    pub luaref: c_int,
}

impl MapAttrs {
    /// Create new mapping attributes.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            flags: 0,
            noremap: 0,
            script_id: 0,
            luaref: -1, // LUA_NOREF
        }
    }

    /// Check if mapping is silent.
    #[must_use]
    pub const fn is_silent(&self) -> bool {
        has_map_flag(self.flags, map_flags::MAP_SILENT)
    }

    /// Check if mapping is an expression.
    #[must_use]
    pub const fn is_expr(&self) -> bool {
        has_map_flag(self.flags, map_flags::MAP_EXPR)
    }

    /// Check if mapping has nowait.
    #[must_use]
    pub const fn is_nowait(&self) -> bool {
        has_map_flag(self.flags, map_flags::MAP_NOWAIT)
    }

    /// Check if mapping uses Lua callback.
    #[must_use]
    pub const fn is_lua(&self) -> bool {
        has_map_flag(self.flags, map_flags::MAP_LUA)
    }

    /// Check if mapping should replace keycodes.
    #[must_use]
    pub const fn replace_keycodes(&self) -> bool {
        has_map_flag(self.flags, map_flags::MAP_REPLACE_KEYCODES)
    }

    /// Check if mapping is script-local.
    #[must_use]
    pub const fn is_script(&self) -> bool {
        has_map_flag(self.flags, map_flags::MAP_SCRIPT)
    }

    /// Check if mapping is unique.
    #[must_use]
    pub const fn is_unique(&self) -> bool {
        has_map_flag(self.flags, map_flags::MAP_UNIQUE)
    }

    /// Check if mapping is noremap.
    #[must_use]
    pub const fn is_noremap(&self) -> bool {
        self.noremap != 0
    }

    /// Set silent flag.
    pub fn set_silent(&mut self, silent: bool) {
        if silent {
            self.flags = set_map_flag(self.flags, map_flags::MAP_SILENT);
        } else {
            self.flags = clear_map_flag(self.flags, map_flags::MAP_SILENT);
        }
    }

    /// Set expr flag.
    pub fn set_expr(&mut self, expr: bool) {
        if expr {
            self.flags = set_map_flag(self.flags, map_flags::MAP_EXPR);
        } else {
            self.flags = clear_map_flag(self.flags, map_flags::MAP_EXPR);
        }
    }

    /// Set nowait flag.
    pub fn set_nowait(&mut self, nowait: bool) {
        if nowait {
            self.flags = set_map_flag(self.flags, map_flags::MAP_NOWAIT);
        } else {
            self.flags = clear_map_flag(self.flags, map_flags::MAP_NOWAIT);
        }
    }
}

// =============================================================================
// Argument Parsing Flags
// =============================================================================

/// Flags for map command argument parsing.
pub mod arg_flags {
    use std::ffi::c_int;

    /// <buffer> argument present
    pub const ARG_BUFFER: c_int = 0x01;
    /// <nowait> argument present
    pub const ARG_NOWAIT: c_int = 0x02;
    /// <silent> argument present
    pub const ARG_SILENT: c_int = 0x04;
    /// <special> argument present (deprecated)
    pub const ARG_SPECIAL: c_int = 0x08;
    /// <script> argument present
    pub const ARG_SCRIPT: c_int = 0x10;
    /// <expr> argument present
    pub const ARG_EXPR: c_int = 0x20;
    /// <unique> argument present
    pub const ARG_UNIQUE: c_int = 0x40;
}

/// State for parsing map command arguments.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MapArgState {
    /// Parsed argument flags
    pub arg_flags: c_int,
    /// Current position in command
    pub pos: usize,
    /// Whether we're still in argument section
    pub in_args: bool,
}

impl MapArgState {
    /// Create a new argument parse state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            arg_flags: 0,
            pos: 0,
            in_args: true,
        }
    }

    /// Check if <buffer> was specified.
    #[must_use]
    pub const fn has_buffer(&self) -> bool {
        has_map_flag(self.arg_flags, arg_flags::ARG_BUFFER)
    }

    /// Check if <nowait> was specified.
    #[must_use]
    pub const fn has_nowait(&self) -> bool {
        has_map_flag(self.arg_flags, arg_flags::ARG_NOWAIT)
    }

    /// Check if <silent> was specified.
    #[must_use]
    pub const fn has_silent(&self) -> bool {
        has_map_flag(self.arg_flags, arg_flags::ARG_SILENT)
    }

    /// Check if <expr> was specified.
    #[must_use]
    pub const fn has_expr(&self) -> bool {
        has_map_flag(self.arg_flags, arg_flags::ARG_EXPR)
    }

    /// Check if <unique> was specified.
    #[must_use]
    pub const fn has_unique(&self) -> bool {
        has_map_flag(self.arg_flags, arg_flags::ARG_UNIQUE)
    }

    /// Convert to MapAttrs.
    #[must_use]
    pub const fn to_map_attrs(&self, noremap: c_int) -> MapAttrs {
        let mut flags = 0;
        if self.has_silent() {
            flags |= map_flags::MAP_SILENT;
        }
        if self.has_expr() {
            flags |= map_flags::MAP_EXPR;
        }
        if self.has_nowait() {
            flags |= map_flags::MAP_NOWAIT;
        }
        if self.has_unique() {
            flags |= map_flags::MAP_UNIQUE;
        }
        MapAttrs {
            flags,
            noremap,
            script_id: 0,
            luaref: -1,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if mapping flags have a specific flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_map_has_flag(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_map_flag(flags, flag))
}

/// Set a mapping flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_map_set_flag(flags: c_int, flag: c_int) -> c_int {
    set_map_flag(flags, flag)
}

/// Clear a mapping flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_map_clear_flag(flags: c_int, flag: c_int) -> c_int {
    clear_map_flag(flags, flag)
}

/// Get noremap type from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_noremap_type(value: c_int) -> c_int {
    NoremapType::from_raw(value).to_raw()
}

/// Check if noremap value indicates any form of noremap.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_noremap(value: c_int) -> c_int {
    c_int::from(NoremapType::from_raw(value).is_noremap())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_flags() {
        let flags = 0;
        assert!(!has_map_flag(flags, map_flags::MAP_SILENT));

        let flags = set_map_flag(flags, map_flags::MAP_SILENT);
        assert!(has_map_flag(flags, map_flags::MAP_SILENT));

        let flags = set_map_flag(flags, map_flags::MAP_EXPR);
        assert!(has_map_flag(flags, map_flags::MAP_SILENT));
        assert!(has_map_flag(flags, map_flags::MAP_EXPR));

        let flags = clear_map_flag(flags, map_flags::MAP_SILENT);
        assert!(!has_map_flag(flags, map_flags::MAP_SILENT));
        assert!(has_map_flag(flags, map_flags::MAP_EXPR));
    }

    #[test]
    fn test_noremap_type() {
        assert_eq!(NoremapType::from_raw(0), NoremapType::Remap);
        assert_eq!(NoremapType::from_raw(1), NoremapType::Noremap);
        assert_eq!(NoremapType::from_raw(2), NoremapType::ScriptNoremap);
        assert_eq!(NoremapType::from_raw(99), NoremapType::BufNoremap);

        assert!(!NoremapType::Remap.is_noremap());
        assert!(NoremapType::Noremap.is_noremap());
        assert!(NoremapType::ScriptNoremap.is_noremap());
    }

    #[test]
    fn test_map_attrs() {
        let mut attrs = MapAttrs::new();
        assert!(!attrs.is_silent());
        assert!(!attrs.is_expr());
        assert!(!attrs.is_nowait());

        attrs.set_silent(true);
        assert!(attrs.is_silent());

        attrs.set_expr(true);
        assert!(attrs.is_expr());

        attrs.set_nowait(true);
        assert!(attrs.is_nowait());

        attrs.set_silent(false);
        assert!(!attrs.is_silent());
        assert!(attrs.is_expr());
    }

    #[test]
    fn test_map_arg_state() {
        let mut state = MapArgState::new();
        assert!(!state.has_buffer());
        assert!(!state.has_silent());

        state.arg_flags |= arg_flags::ARG_BUFFER;
        assert!(state.has_buffer());

        state.arg_flags |= arg_flags::ARG_SILENT | arg_flags::ARG_EXPR;
        assert!(state.has_silent());
        assert!(state.has_expr());

        let attrs = state.to_map_attrs(1);
        assert!(attrs.is_silent());
        assert!(attrs.is_expr());
        assert!(attrs.is_noremap());
    }
}
