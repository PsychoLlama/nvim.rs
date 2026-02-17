//! Syntax item management.
//!
//! This module provides utilities for managing syntax items including
//! adding, removing, and clearing items from syntax definitions.

use std::ffi::c_int;

// =============================================================================
// Syntax item types
// =============================================================================

/// Type of syntax item.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynItemType {
    /// Invalid/unknown item type.
    Invalid = 0,
    /// A syntax match item.
    Match = 1,
    /// A syntax region item.
    Region = 2,
    /// A syntax keyword item.
    Keyword = 3,
}

impl SynItemType {
    /// Convert from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Match,
            2 => Self::Region,
            3 => Self::Keyword,
            _ => Self::Invalid,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if item type uses patterns.
    #[must_use]
    pub const fn uses_pattern(self) -> bool {
        matches!(self, Self::Match | Self::Region)
    }

    /// Check if item type is valid.
    #[must_use]
    pub const fn is_valid(self) -> bool {
        !matches!(self, Self::Invalid)
    }
}

// =============================================================================
// Item flags
// =============================================================================

/// Flags for syntax item behavior.
pub mod item_flags {
    use std::ffi::c_int;

    /// Item is contained (only matches inside another).
    pub const CONTAINED: c_int = 0x001;
    /// Item can contain other items.
    pub const CONTAINER: c_int = 0x002;
    /// Item is transparent (doesn't affect highlighting).
    pub const TRANSPARENT: c_int = 0x004;
    /// Item creates a fold.
    pub const FOLD: c_int = 0x008;
    /// Item affects spell checking.
    pub const SPELL: c_int = 0x010;
    /// Item is concealed.
    pub const CONCEAL: c_int = 0x020;
    /// Item requires display mode.
    pub const DISPLAY: c_int = 0x040;
    /// Item only matches on displayed text.
    pub const ONELINE: c_int = 0x080;
    /// Don't extend past end of line.
    pub const EXCLUDENL: c_int = 0x100;
    /// Keep end of parent region.
    pub const KEEPEND: c_int = 0x200;
    /// Extend to end of pattern.
    pub const EXTEND: c_int = 0x400;
}

/// Check if item flag is set.
#[inline]
pub const fn has_item_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set item flag.
#[inline]
pub const fn set_item_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear item flag.
#[inline]
pub const fn clear_item_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

// =============================================================================
// Item info
// =============================================================================

/// Information about a syntax item.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SynItemInfo {
    /// Item type.
    pub item_type: c_int,
    /// Item flags.
    pub flags: c_int,
    /// Syntax ID.
    pub syn_id: c_int,
    /// Highlight attribute ID.
    pub attr_id: c_int,
    /// Group ID.
    pub group_id: c_int,
    /// Priority.
    pub priority: c_int,
    /// Conceal character.
    pub conceal_char: c_int,
}

impl SynItemInfo {
    /// Create new item info.
    #[must_use]
    pub const fn new(item_type: SynItemType, syn_id: c_int) -> Self {
        Self {
            item_type: item_type.to_c_int(),
            flags: 0,
            syn_id,
            attr_id: 0,
            group_id: 0,
            priority: 0,
            conceal_char: 0,
        }
    }

    /// Get item type.
    #[must_use]
    pub const fn get_type(&self) -> SynItemType {
        SynItemType::from_c_int(self.item_type)
    }

    /// Check if item is contained.
    #[must_use]
    pub const fn is_contained(&self) -> bool {
        has_item_flag(self.flags, item_flags::CONTAINED)
    }

    /// Check if item is a container.
    #[must_use]
    pub const fn is_container(&self) -> bool {
        has_item_flag(self.flags, item_flags::CONTAINER)
    }

    /// Check if item is transparent.
    #[must_use]
    pub const fn is_transparent(&self) -> bool {
        has_item_flag(self.flags, item_flags::TRANSPARENT)
    }

    /// Check if item creates fold.
    #[must_use]
    pub const fn creates_fold(&self) -> bool {
        has_item_flag(self.flags, item_flags::FOLD)
    }

    /// Check if item is concealed.
    #[must_use]
    pub const fn is_concealed(&self) -> bool {
        has_item_flag(self.flags, item_flags::CONCEAL)
    }

    /// Set contained flag.
    pub fn set_contained(&mut self, contained: bool) {
        if contained {
            self.flags = set_item_flag(self.flags, item_flags::CONTAINED);
        } else {
            self.flags = clear_item_flag(self.flags, item_flags::CONTAINED);
        }
    }

    /// Set container flag.
    pub fn set_container(&mut self, container: bool) {
        if container {
            self.flags = set_item_flag(self.flags, item_flags::CONTAINER);
        } else {
            self.flags = clear_item_flag(self.flags, item_flags::CONTAINER);
        }
    }
}

// =============================================================================
// Item operations
// =============================================================================

/// Result of an item operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ItemOpResult {
    /// Whether operation succeeded.
    pub success: bool,
    /// ID of affected item (-1 if none).
    pub item_id: c_int,
    /// Error code (0 if success).
    pub error: c_int,
}

impl ItemOpResult {
    /// Create success result.
    #[must_use]
    pub const fn ok(item_id: c_int) -> Self {
        Self {
            success: true,
            item_id,
            error: 0,
        }
    }

    /// Create error result.
    #[must_use]
    pub const fn err(error: c_int) -> Self {
        Self {
            success: false,
            item_id: -1,
            error,
        }
    }
}

impl Default for ItemOpResult {
    fn default() -> Self {
        Self::err(0)
    }
}

// =============================================================================
// FFI exports
// =============================================================================

/// Get item type from integer.
#[no_mangle]
pub extern "C" fn rs_syn_item_type_from_int(val: c_int) -> c_int {
    SynItemType::from_c_int(val).to_c_int()
}

/// Check if item type uses patterns.
#[no_mangle]
pub extern "C" fn rs_syn_item_type_uses_pattern(item_type: c_int) -> c_int {
    c_int::from(SynItemType::from_c_int(item_type).uses_pattern())
}

/// Check if item type is valid.
#[no_mangle]
pub extern "C" fn rs_syn_item_type_is_valid(item_type: c_int) -> c_int {
    c_int::from(SynItemType::from_c_int(item_type).is_valid())
}

/// Check item flag.
#[no_mangle]
pub extern "C" fn rs_syn_item_has_flag(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_item_flag(flags, flag))
}

/// Set item flag.
#[no_mangle]
pub extern "C" fn rs_syn_item_set_flag(flags: c_int, flag: c_int) -> c_int {
    set_item_flag(flags, flag)
}

/// Clear item flag.
#[no_mangle]
pub extern "C" fn rs_syn_item_clear_flag(flags: c_int, flag: c_int) -> c_int {
    clear_item_flag(flags, flag)
}

/// Create new item info.
#[no_mangle]
pub extern "C" fn rs_syn_item_info_new(item_type: c_int, syn_id: c_int) -> SynItemInfo {
    SynItemInfo::new(SynItemType::from_c_int(item_type), syn_id)
}
/// Create success operation result.
#[no_mangle]
pub extern "C" fn rs_syn_item_op_result_ok(item_id: c_int) -> ItemOpResult {
    ItemOpResult::ok(item_id)
}

/// Create error operation result.
#[no_mangle]
pub extern "C" fn rs_syn_item_op_result_err(error: c_int) -> ItemOpResult {
    ItemOpResult::err(error)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syn_item_type() {
        assert_eq!(SynItemType::Match.to_c_int(), 1);
        assert_eq!(SynItemType::from_c_int(1), SynItemType::Match);
        assert_eq!(SynItemType::from_c_int(0), SynItemType::Invalid);

        assert!(SynItemType::Match.uses_pattern());
        assert!(SynItemType::Region.uses_pattern());
        assert!(!SynItemType::Keyword.uses_pattern());

        assert!(SynItemType::Match.is_valid());
        assert!(!SynItemType::Invalid.is_valid());
    }

    #[test]
    fn test_item_flags() {
        let flags = 0;
        assert!(!has_item_flag(flags, item_flags::CONTAINED));

        let flags = set_item_flag(flags, item_flags::CONTAINED);
        assert!(has_item_flag(flags, item_flags::CONTAINED));
        assert!(!has_item_flag(flags, item_flags::FOLD));

        let flags = set_item_flag(flags, item_flags::FOLD);
        assert!(has_item_flag(flags, item_flags::CONTAINED));
        assert!(has_item_flag(flags, item_flags::FOLD));

        let flags = clear_item_flag(flags, item_flags::CONTAINED);
        assert!(!has_item_flag(flags, item_flags::CONTAINED));
        assert!(has_item_flag(flags, item_flags::FOLD));
    }

    #[test]
    fn test_item_info() {
        let mut info = SynItemInfo::new(SynItemType::Match, 10);
        assert_eq!(info.get_type(), SynItemType::Match);
        assert_eq!(info.syn_id, 10);
        assert!(!info.is_contained());

        info.set_contained(true);
        assert!(info.is_contained());

        info.set_contained(false);
        assert!(!info.is_contained());
    }

    #[test]
    fn test_item_op_result() {
        let ok = ItemOpResult::ok(5);
        assert!(ok.success);
        assert_eq!(ok.item_id, 5);
        assert_eq!(ok.error, 0);

        let err = ItemOpResult::err(42);
        assert!(!err.success);
        assert_eq!(err.item_id, -1);
        assert_eq!(err.error, 42);
    }
}
