//! OPENLINE flags for the open_line() function.
//!
//! These flags control the behavior of opening a new line (Enter/o/O commands).

use std::ffi::c_int;

/// Flags for open_line() function.
///
/// These correspond to the OPENLINE_* constants in change.h.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OpenlineFlags(c_int);

impl OpenlineFlags {
    /// Delete spaces after cursor.
    pub const DELSPACES: Self = Self(0x01);
    /// Format comments.
    pub const DO_COM: Self = Self(0x02);
    /// Keep trailing spaces.
    pub const KEEPTRAIL: Self = Self(0x04);
    /// Fix mark positions.
    pub const MARKFIX: Self = Self(0x08);
    /// Format comments with list/2nd line indent.
    pub const COM_LIST: Self = Self(0x10);
    /// Formatting long comment.
    pub const FORMAT: Self = Self(0x20);
    /// Use second_line_indent without indent logic.
    pub const FORCE_INDENT: Self = Self(0x40);

    /// Create flags from raw integer value.
    #[inline]
    pub const fn from_raw(val: c_int) -> Self {
        Self(val)
    }

    /// Get the raw integer value.
    #[inline]
    pub const fn as_raw(self) -> c_int {
        self.0
    }

    /// Check if this flag is set.
    #[inline]
    pub const fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }

    /// Combine flags using bitwise OR.
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for OpenlineFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl std::ops::BitAnd for OpenlineFlags {
    type Output = bool;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.contains(rhs)
    }
}

// =============================================================================
// FFI Flag Accessors
// =============================================================================

/// Get OPENLINE_DELSPACES flag value.
#[no_mangle]
pub extern "C" fn rs_openline_delspaces() -> c_int {
    OpenlineFlags::DELSPACES.as_raw()
}

/// Get OPENLINE_DO_COM flag value.
#[no_mangle]
pub extern "C" fn rs_openline_do_com() -> c_int {
    OpenlineFlags::DO_COM.as_raw()
}

/// Get OPENLINE_KEEPTRAIL flag value.
#[no_mangle]
pub extern "C" fn rs_openline_keeptrail() -> c_int {
    OpenlineFlags::KEEPTRAIL.as_raw()
}

/// Get OPENLINE_MARKFIX flag value.
#[no_mangle]
pub extern "C" fn rs_openline_markfix() -> c_int {
    OpenlineFlags::MARKFIX.as_raw()
}

/// Get OPENLINE_COM_LIST flag value.
#[no_mangle]
pub extern "C" fn rs_openline_com_list() -> c_int {
    OpenlineFlags::COM_LIST.as_raw()
}

/// Get OPENLINE_FORMAT flag value.
#[no_mangle]
pub extern "C" fn rs_openline_format() -> c_int {
    OpenlineFlags::FORMAT.as_raw()
}

/// Get OPENLINE_FORCE_INDENT flag value.
#[no_mangle]
pub extern "C" fn rs_openline_force_indent() -> c_int {
    OpenlineFlags::FORCE_INDENT.as_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_values() {
        assert_eq!(OpenlineFlags::DELSPACES.as_raw(), 0x01);
        assert_eq!(OpenlineFlags::DO_COM.as_raw(), 0x02);
        assert_eq!(OpenlineFlags::KEEPTRAIL.as_raw(), 0x04);
        assert_eq!(OpenlineFlags::MARKFIX.as_raw(), 0x08);
        assert_eq!(OpenlineFlags::COM_LIST.as_raw(), 0x10);
        assert_eq!(OpenlineFlags::FORMAT.as_raw(), 0x20);
        assert_eq!(OpenlineFlags::FORCE_INDENT.as_raw(), 0x40);
    }

    #[test]
    fn test_flag_contains() {
        let flags = OpenlineFlags::DELSPACES | OpenlineFlags::DO_COM;
        assert!(flags.contains(OpenlineFlags::DELSPACES));
        assert!(flags.contains(OpenlineFlags::DO_COM));
        assert!(!flags.contains(OpenlineFlags::KEEPTRAIL));
    }

    #[test]
    fn test_flag_union() {
        let a = OpenlineFlags::DELSPACES;
        let b = OpenlineFlags::MARKFIX;
        let combined = a.union(b);
        assert_eq!(combined.as_raw(), 0x01 | 0x08);
    }
}
