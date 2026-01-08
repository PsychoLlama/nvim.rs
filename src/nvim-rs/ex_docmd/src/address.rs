//! Address and range parsing types for Ex commands.
//!
//! This module defines types for command address/range parsing,
//! such as `1,5`, `%`, `'a,'b`, etc.

use std::ffi::c_int;

// =============================================================================
// Address type enum
// =============================================================================

/// Type of address for an Ex command.
///
/// Determines how the address/range is interpreted for the command.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddrType {
    /// Buffer line numbers (default for most commands)
    Lines = 0,
    /// Window number
    Windows = 1,
    /// Argument number
    Arguments = 2,
    /// Buffer number of loaded buffer
    LoadedBuffers = 3,
    /// Buffer number (any buffer)
    Buffers = 4,
    /// Tab page number
    Tabs = 5,
    /// Tab page that only uses relative addressing
    TabsRelative = 6,
    /// Quickfix list valid entry number
    QuickfixValid = 7,
    /// Quickfix list entry number
    Quickfix = 8,
    /// Positive count or zero, defaults to 1
    Unsigned = 9,
    /// Something else, use line number for '$', '%', etc.
    Other = 10,
    /// No range used
    None = 11,
}

impl AddrType {
    /// Convert from C integer value.
    #[inline]
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Lines),
            1 => Some(Self::Windows),
            2 => Some(Self::Arguments),
            3 => Some(Self::LoadedBuffers),
            4 => Some(Self::Buffers),
            5 => Some(Self::Tabs),
            6 => Some(Self::TabsRelative),
            7 => Some(Self::QuickfixValid),
            8 => Some(Self::Quickfix),
            9 => Some(Self::Unsigned),
            10 => Some(Self::Other),
            11 => Some(Self::None),
            _ => Option::None,
        }
    }

    /// Convert to C integer value.
    #[inline]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this address type uses line numbers.
    #[inline]
    pub const fn uses_line_numbers(self) -> bool {
        matches!(self, Self::Lines | Self::Other)
    }

    /// Check if this address type uses buffer numbers.
    #[inline]
    pub const fn uses_buffer_numbers(self) -> bool {
        matches!(self, Self::LoadedBuffers | Self::Buffers)
    }

    /// Check if this address type uses window numbers.
    #[inline]
    pub const fn uses_window_numbers(self) -> bool {
        matches!(self, Self::Windows)
    }

    /// Check if this address type uses tab numbers.
    #[inline]
    pub const fn uses_tab_numbers(self) -> bool {
        matches!(self, Self::Tabs | Self::TabsRelative)
    }

    /// Check if this address type uses quickfix entries.
    #[inline]
    pub const fn uses_quickfix(self) -> bool {
        matches!(self, Self::Quickfix | Self::QuickfixValid)
    }
}

// =============================================================================
// FFI functions
// =============================================================================

/// Convert C address type integer to Rust enum.
///
/// Returns -1 if the value is invalid.
#[no_mangle]
pub extern "C" fn rs_addr_type_from_int(val: c_int) -> c_int {
    match AddrType::from_c_int(val) {
        Some(t) => t.to_c_int(),
        Option::None => -1,
    }
}

/// Check if address type uses line numbers.
#[no_mangle]
pub extern "C" fn rs_addr_type_uses_lines(val: c_int) -> c_int {
    match AddrType::from_c_int(val) {
        Some(t) => c_int::from(t.uses_line_numbers()),
        Option::None => 0,
    }
}

/// Check if address type uses buffer numbers.
#[no_mangle]
pub extern "C" fn rs_addr_type_uses_buffers(val: c_int) -> c_int {
    match AddrType::from_c_int(val) {
        Some(t) => c_int::from(t.uses_buffer_numbers()),
        Option::None => 0,
    }
}

/// Check if address type uses window numbers.
#[no_mangle]
pub extern "C" fn rs_addr_type_uses_windows(val: c_int) -> c_int {
    match AddrType::from_c_int(val) {
        Some(t) => c_int::from(t.uses_window_numbers()),
        Option::None => 0,
    }
}

/// Check if address type uses tab numbers.
#[no_mangle]
pub extern "C" fn rs_addr_type_uses_tabs(val: c_int) -> c_int {
    match AddrType::from_c_int(val) {
        Some(t) => c_int::from(t.uses_tab_numbers()),
        Option::None => 0,
    }
}

/// Check if address type uses quickfix entries.
#[no_mangle]
pub extern "C" fn rs_addr_type_uses_quickfix(val: c_int) -> c_int {
    match AddrType::from_c_int(val) {
        Some(t) => c_int::from(t.uses_quickfix()),
        Option::None => 0,
    }
}

// =============================================================================
// C-compatible constants
// =============================================================================

/// ADDR_LINES - buffer line numbers
pub const ADDR_LINES: c_int = 0;
/// ADDR_WINDOWS - window number
pub const ADDR_WINDOWS: c_int = 1;
/// ADDR_ARGUMENTS - argument number
pub const ADDR_ARGUMENTS: c_int = 2;
/// ADDR_LOADED_BUFFERS - loaded buffer number
pub const ADDR_LOADED_BUFFERS: c_int = 3;
/// ADDR_BUFFERS - buffer number
pub const ADDR_BUFFERS: c_int = 4;
/// ADDR_TABS - tab page number
pub const ADDR_TABS: c_int = 5;
/// ADDR_TABS_RELATIVE - relative tab page
pub const ADDR_TABS_RELATIVE: c_int = 6;
/// ADDR_QUICKFIX_VALID - valid quickfix entry
pub const ADDR_QUICKFIX_VALID: c_int = 7;
/// ADDR_QUICKFIX - quickfix entry
pub const ADDR_QUICKFIX: c_int = 8;
/// ADDR_UNSIGNED - unsigned count
pub const ADDR_UNSIGNED: c_int = 9;
/// ADDR_OTHER - other address type
pub const ADDR_OTHER: c_int = 10;
/// ADDR_NONE - no address
pub const ADDR_NONE: c_int = 11;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addr_type_from_c_int() {
        assert_eq!(AddrType::from_c_int(0), Some(AddrType::Lines));
        assert_eq!(AddrType::from_c_int(1), Some(AddrType::Windows));
        assert_eq!(AddrType::from_c_int(11), Some(AddrType::None));
        assert_eq!(AddrType::from_c_int(12), Option::None);
        assert_eq!(AddrType::from_c_int(-1), Option::None);
    }

    #[test]
    fn test_addr_type_to_c_int() {
        assert_eq!(AddrType::Lines.to_c_int(), 0);
        assert_eq!(AddrType::Windows.to_c_int(), 1);
        assert_eq!(AddrType::None.to_c_int(), 11);
    }

    #[test]
    fn test_uses_line_numbers() {
        assert!(AddrType::Lines.uses_line_numbers());
        assert!(AddrType::Other.uses_line_numbers());
        assert!(!AddrType::Windows.uses_line_numbers());
        assert!(!AddrType::Buffers.uses_line_numbers());
    }

    #[test]
    fn test_uses_buffer_numbers() {
        assert!(AddrType::Buffers.uses_buffer_numbers());
        assert!(AddrType::LoadedBuffers.uses_buffer_numbers());
        assert!(!AddrType::Lines.uses_buffer_numbers());
    }

    #[test]
    fn test_uses_tab_numbers() {
        assert!(AddrType::Tabs.uses_tab_numbers());
        assert!(AddrType::TabsRelative.uses_tab_numbers());
        assert!(!AddrType::Lines.uses_tab_numbers());
    }

    #[test]
    fn test_uses_quickfix() {
        assert!(AddrType::Quickfix.uses_quickfix());
        assert!(AddrType::QuickfixValid.uses_quickfix());
        assert!(!AddrType::Lines.uses_quickfix());
    }

    #[test]
    fn test_ffi_addr_type_from_int() {
        assert_eq!(rs_addr_type_from_int(0), 0);
        assert_eq!(rs_addr_type_from_int(11), 11);
        assert_eq!(rs_addr_type_from_int(99), -1);
    }

    #[test]
    fn test_ffi_uses_functions() {
        assert_eq!(rs_addr_type_uses_lines(ADDR_LINES), 1);
        assert_eq!(rs_addr_type_uses_lines(ADDR_WINDOWS), 0);

        assert_eq!(rs_addr_type_uses_buffers(ADDR_BUFFERS), 1);
        assert_eq!(rs_addr_type_uses_buffers(ADDR_LINES), 0);

        assert_eq!(rs_addr_type_uses_windows(ADDR_WINDOWS), 1);
        assert_eq!(rs_addr_type_uses_windows(ADDR_LINES), 0);

        assert_eq!(rs_addr_type_uses_tabs(ADDR_TABS), 1);
        assert_eq!(rs_addr_type_uses_tabs(ADDR_LINES), 0);

        assert_eq!(rs_addr_type_uses_quickfix(ADDR_QUICKFIX), 1);
        assert_eq!(rs_addr_type_uses_quickfix(ADDR_LINES), 0);
    }
}
