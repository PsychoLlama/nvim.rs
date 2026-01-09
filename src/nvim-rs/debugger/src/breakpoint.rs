//! Breakpoint management
//!
//! This module provides Rust implementations for breakpoint handling,
//! including creation, modification, and hit testing.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_int;

/// Line number type
type LinenrT = i32;

// =============================================================================
// Breakpoint Type
// =============================================================================

/// Type of breakpoint
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BreakpointType {
    /// Standard breakpoint
    #[default]
    Standard = 0,
    /// Conditional breakpoint
    Conditional = 1,
    /// Log point (doesn't stop)
    Logpoint = 2,
    /// Function breakpoint
    Function = 3,
    /// Data breakpoint (watchpoint)
    Data = 4,
    /// Exception breakpoint
    Exception = 5,
}

impl BreakpointType {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Standard),
            1 => Some(Self::Conditional),
            2 => Some(Self::Logpoint),
            3 => Some(Self::Function),
            4 => Some(Self::Data),
            5 => Some(Self::Exception),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this type stops execution
    pub const fn stops_execution(self) -> bool {
        !matches!(self, Self::Logpoint)
    }

    /// Check if this is location-based
    pub const fn is_location_based(self) -> bool {
        matches!(self, Self::Standard | Self::Conditional | Self::Logpoint)
    }
}

// =============================================================================
// Breakpoint Flags
// =============================================================================

/// Breakpoint flags
pub const BP_ENABLED: u32 = 0x0001;
pub const BP_VERIFIED: u32 = 0x0002;
pub const BP_TEMPORARY: u32 = 0x0004;
pub const BP_HIDDEN: u32 = 0x0008;
pub const BP_HIT: u32 = 0x0010;
pub const BP_PENDING: u32 = 0x0020;

/// Breakpoint flags wrapper
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BreakpointFlags {
    flags: u32,
}

impl BreakpointFlags {
    /// Create with no flags
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// Create from raw value
    pub const fn from_raw(flags: u32) -> Self {
        Self { flags }
    }

    /// Get raw value
    pub const fn as_raw(self) -> u32 {
        self.flags
    }

    /// Create default enabled breakpoint
    pub const fn enabled() -> Self {
        Self { flags: BP_ENABLED }
    }

    /// Check if enabled
    pub const fn is_enabled(self) -> bool {
        (self.flags & BP_ENABLED) != 0
    }

    /// Check if verified
    pub const fn is_verified(self) -> bool {
        (self.flags & BP_VERIFIED) != 0
    }

    /// Check if temporary
    pub const fn is_temporary(self) -> bool {
        (self.flags & BP_TEMPORARY) != 0
    }

    /// Check if hidden
    pub const fn is_hidden(self) -> bool {
        (self.flags & BP_HIDDEN) != 0
    }

    /// Check if hit
    pub const fn is_hit(self) -> bool {
        (self.flags & BP_HIT) != 0
    }

    /// Check if pending verification
    pub const fn is_pending(self) -> bool {
        (self.flags & BP_PENDING) != 0
    }

    /// Set enabled flag
    pub fn set_enabled(&mut self, value: bool) {
        if value {
            self.flags |= BP_ENABLED;
        } else {
            self.flags &= !BP_ENABLED;
        }
    }

    /// Set verified flag
    pub fn set_verified(&mut self, value: bool) {
        if value {
            self.flags |= BP_VERIFIED;
            self.flags &= !BP_PENDING;
        } else {
            self.flags &= !BP_VERIFIED;
        }
    }

    /// Set hit flag
    pub fn set_hit(&mut self, value: bool) {
        if value {
            self.flags |= BP_HIT;
        } else {
            self.flags &= !BP_HIT;
        }
    }
}

// =============================================================================
// Breakpoint Location
// =============================================================================

/// Breakpoint location information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BreakpointLocation {
    /// Line number (1-based)
    pub line: LinenrT,
    /// Column number (0 = any column)
    pub column: c_int,
    /// End line (for range breakpoints)
    pub end_line: LinenrT,
}

impl Default for BreakpointLocation {
    fn default() -> Self {
        Self {
            line: 0,
            column: 0,
            end_line: 0,
        }
    }
}

impl BreakpointLocation {
    /// Create a new location
    pub const fn new(line: LinenrT) -> Self {
        Self {
            line,
            column: 0,
            end_line: 0,
        }
    }

    /// Create with column
    pub const fn with_column(line: LinenrT, column: c_int) -> Self {
        Self {
            line,
            column,
            end_line: 0,
        }
    }

    /// Check if location is valid
    pub const fn is_valid(&self) -> bool {
        self.line > 0
    }

    /// Check if this is a range
    pub const fn is_range(&self) -> bool {
        self.end_line > 0 && self.end_line > self.line
    }

    /// Check if location matches a line
    pub const fn matches_line(&self, line: LinenrT) -> bool {
        if !self.is_valid() {
            return false;
        }
        if self.is_range() {
            line >= self.line && line <= self.end_line
        } else {
            line == self.line
        }
    }
}

// =============================================================================
// Breakpoint
// =============================================================================

/// Breakpoint definition
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Breakpoint {
    /// Unique ID
    pub id: c_int,
    /// Breakpoint type
    pub btype: BreakpointType,
    /// Flags
    pub flags: BreakpointFlags,
    /// Location
    pub location: BreakpointLocation,
    /// Hit count (number of times hit)
    pub hit_count: c_int,
    /// Hit condition (stop after N hits, 0 = always)
    pub hit_condition: c_int,
}

impl Default for Breakpoint {
    fn default() -> Self {
        Self {
            id: 0,
            btype: BreakpointType::Standard,
            flags: BreakpointFlags::enabled(),
            location: BreakpointLocation::default(),
            hit_count: 0,
            hit_condition: 0,
        }
    }
}

impl Breakpoint {
    /// Create a new breakpoint
    pub const fn new(id: c_int, line: LinenrT) -> Self {
        Self {
            id,
            btype: BreakpointType::Standard,
            flags: BreakpointFlags::enabled(),
            location: BreakpointLocation::new(line),
            hit_count: 0,
            hit_condition: 0,
        }
    }

    /// Check if breakpoint is active
    pub const fn is_active(&self) -> bool {
        self.flags.is_enabled() && self.location.is_valid()
    }

    /// Check if breakpoint should stop on hit
    pub fn should_stop(&self) -> bool {
        if !self.is_active() {
            return false;
        }
        if !self.btype.stops_execution() {
            return false;
        }
        if self.hit_condition > 0 && self.hit_count < self.hit_condition {
            return false;
        }
        true
    }

    /// Record a hit and return whether to stop
    pub fn record_hit(&mut self) -> bool {
        self.hit_count += 1;
        self.flags.set_hit(true);
        self.should_stop()
    }

    /// Reset hit tracking
    pub fn reset_hits(&mut self) {
        self.hit_count = 0;
        self.flags.set_hit(false);
    }
}

// =============================================================================
// Breakpoint Match Result
// =============================================================================

/// Result of breakpoint matching
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BreakpointMatchResult {
    /// Whether a breakpoint was found
    pub found: bool,
    /// ID of matching breakpoint
    pub breakpoint_id: c_int,
    /// Whether execution should stop
    pub should_stop: bool,
}

impl BreakpointMatchResult {
    /// Create a no-match result
    pub const fn no_match() -> Self {
        Self {
            found: false,
            breakpoint_id: -1,
            should_stop: false,
        }
    }

    /// Create a match result
    pub const fn matched(id: c_int, should_stop: bool) -> Self {
        Self {
            found: true,
            breakpoint_id: id,
            should_stop,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if breakpoint type is valid
#[no_mangle]
pub extern "C" fn rs_breakpoint_type_valid(btype: c_int) -> c_int {
    c_int::from(BreakpointType::from_raw(btype).is_some())
}

/// FFI export: Check if breakpoint type stops execution
#[no_mangle]
pub extern "C" fn rs_breakpoint_type_stops(btype: c_int) -> c_int {
    BreakpointType::from_raw(btype).map_or(0, |t| c_int::from(t.stops_execution()))
}

/// FFI export: Check if flags is enabled
#[no_mangle]
pub extern "C" fn rs_breakpoint_flags_is_enabled(flags: u32) -> c_int {
    c_int::from(BreakpointFlags::from_raw(flags).is_enabled())
}

/// FFI export: Check if flags is verified
#[no_mangle]
pub extern "C" fn rs_breakpoint_flags_is_verified(flags: u32) -> c_int {
    c_int::from(BreakpointFlags::from_raw(flags).is_verified())
}

/// FFI export: Check if location matches line
#[no_mangle]
pub extern "C" fn rs_breakpoint_location_matches(
    loc_line: LinenrT,
    loc_end: LinenrT,
    test_line: LinenrT,
) -> c_int {
    let loc = BreakpointLocation {
        line: loc_line,
        column: 0,
        end_line: loc_end,
    };
    c_int::from(loc.matches_line(test_line))
}

/// FFI export: Create a new breakpoint
#[no_mangle]
pub extern "C" fn rs_breakpoint_new(id: c_int, line: LinenrT) -> Breakpoint {
    Breakpoint::new(id, line)
}

/// FFI export: Check if breakpoint is active
#[no_mangle]
pub extern "C" fn rs_breakpoint_is_active(bp: *const Breakpoint) -> c_int {
    if bp.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*bp).is_active() })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_type() {
        assert_eq!(BreakpointType::from_raw(0), Some(BreakpointType::Standard));
        assert_eq!(BreakpointType::from_raw(100), None);

        assert!(BreakpointType::Standard.stops_execution());
        assert!(!BreakpointType::Logpoint.stops_execution());

        assert!(BreakpointType::Standard.is_location_based());
        assert!(!BreakpointType::Function.is_location_based());
    }

    #[test]
    fn test_breakpoint_flags() {
        let flags = BreakpointFlags::none();
        assert!(!flags.is_enabled());

        let flags = BreakpointFlags::enabled();
        assert!(flags.is_enabled());
        assert!(!flags.is_verified());

        let mut flags = BreakpointFlags::from_raw(BP_ENABLED | BP_PENDING);
        flags.set_verified(true);
        assert!(flags.is_verified());
        assert!(!flags.is_pending()); // Cleared by verify
    }

    #[test]
    fn test_breakpoint_location() {
        let loc = BreakpointLocation::new(10);
        assert!(loc.is_valid());
        assert!(!loc.is_range());
        assert!(loc.matches_line(10));
        assert!(!loc.matches_line(11));

        let range = BreakpointLocation {
            line: 10,
            column: 0,
            end_line: 20,
        };
        assert!(range.is_range());
        assert!(range.matches_line(10));
        assert!(range.matches_line(15));
        assert!(range.matches_line(20));
        assert!(!range.matches_line(21));
    }

    #[test]
    fn test_breakpoint() {
        let bp = Breakpoint::new(1, 10);
        assert!(bp.is_active());

        let mut bp = Breakpoint::new(2, 20);
        bp.hit_condition = 3;
        assert!(!bp.should_stop()); // Hit count = 0, condition = 3

        bp.record_hit();
        bp.record_hit();
        assert!(!bp.should_stop()); // Hit count = 2, still < 3

        bp.record_hit();
        assert!(bp.should_stop()); // Hit count = 3, meets condition
    }

    #[test]
    fn test_breakpoint_match_result() {
        let no_match = BreakpointMatchResult::no_match();
        assert!(!no_match.found);

        let matched = BreakpointMatchResult::matched(5, true);
        assert!(matched.found);
        assert_eq!(matched.breakpoint_id, 5);
        assert!(matched.should_stop);
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_breakpoint_type_valid(0), 1);
        assert_eq!(rs_breakpoint_type_valid(100), 0);

        assert_eq!(rs_breakpoint_type_stops(0), 1);
        assert_eq!(rs_breakpoint_type_stops(2), 0); // Logpoint

        assert_eq!(rs_breakpoint_flags_is_enabled(BP_ENABLED), 1);
        assert_eq!(rs_breakpoint_flags_is_enabled(0), 0);
    }
}
