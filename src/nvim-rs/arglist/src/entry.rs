//! Argument list entry types
//!
//! This module provides types for representing individual entries
//! in the argument list.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_int;

// =============================================================================
// Entry Flags
// =============================================================================

/// Argument entry flags
pub const ARG_EDITED: u32 = 0x0001;
pub const ARG_LOADED: u32 = 0x0002;
pub const ARG_READONLY: u32 = 0x0004;
pub const ARG_MODIFIED: u32 = 0x0008;
pub const ARG_WILDCARD: u32 = 0x0010;

/// Argument entry flags wrapper
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ArgEntryFlags {
    flags: u32,
}

impl ArgEntryFlags {
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

    /// Check if file has been edited
    pub const fn is_edited(self) -> bool {
        (self.flags & ARG_EDITED) != 0
    }

    /// Check if file is loaded in a buffer
    pub const fn is_loaded(self) -> bool {
        (self.flags & ARG_LOADED) != 0
    }

    /// Check if file is read-only
    pub const fn is_readonly(self) -> bool {
        (self.flags & ARG_READONLY) != 0
    }

    /// Check if file has been modified
    pub const fn is_modified(self) -> bool {
        (self.flags & ARG_MODIFIED) != 0
    }

    /// Check if this was from a wildcard expansion
    pub const fn is_wildcard(self) -> bool {
        (self.flags & ARG_WILDCARD) != 0
    }

    /// Set edited flag
    pub fn set_edited(&mut self, value: bool) {
        if value {
            self.flags |= ARG_EDITED;
        } else {
            self.flags &= !ARG_EDITED;
        }
    }

    /// Set loaded flag
    pub fn set_loaded(&mut self, value: bool) {
        if value {
            self.flags |= ARG_LOADED;
        } else {
            self.flags &= !ARG_LOADED;
        }
    }

    /// Set modified flag
    pub fn set_modified(&mut self, value: bool) {
        if value {
            self.flags |= ARG_MODIFIED;
        } else {
            self.flags &= !ARG_MODIFIED;
        }
    }
}

// =============================================================================
// Argument Entry
// =============================================================================

/// An entry in the argument list
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ArgEntry {
    /// Entry index (0-based)
    pub index: c_int,
    /// Entry flags
    pub flags: ArgEntryFlags,
    /// Associated buffer number (0 if none)
    pub buf_nr: c_int,
    /// File id (for uniqueness checking)
    pub file_id: u64,
}

impl Default for ArgEntry {
    fn default() -> Self {
        Self {
            index: -1,
            flags: ArgEntryFlags::none(),
            buf_nr: 0,
            file_id: 0,
        }
    }
}

impl ArgEntry {
    /// Create a new entry
    pub const fn new(index: c_int) -> Self {
        Self {
            index,
            flags: ArgEntryFlags::none(),
            buf_nr: 0,
            file_id: 0,
        }
    }

    /// Create with buffer number
    pub const fn with_buffer(index: c_int, buf_nr: c_int) -> Self {
        Self {
            index,
            flags: ArgEntryFlags { flags: ARG_LOADED },
            buf_nr,
            file_id: 0,
        }
    }

    /// Check if entry is valid
    pub const fn is_valid(&self) -> bool {
        self.index >= 0
    }

    /// Check if entry has a buffer
    pub const fn has_buffer(&self) -> bool {
        self.buf_nr > 0
    }

    /// Check if entry is the current argument
    pub const fn is_current(&self, current_idx: c_int) -> bool {
        self.index == current_idx
    }
}

// =============================================================================
// Entry Match Result
// =============================================================================

/// Result of searching for an entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ArgEntryMatch {
    /// Whether entry was found
    pub found: bool,
    /// Index of matching entry
    pub index: c_int,
    /// Whether it's an exact match
    pub exact: bool,
}

impl ArgEntryMatch {
    /// Create a not-found result
    pub const fn not_found() -> Self {
        Self {
            found: false,
            index: -1,
            exact: false,
        }
    }

    /// Create an exact match result
    pub const fn exact(index: c_int) -> Self {
        Self {
            found: true,
            index,
            exact: true,
        }
    }

    /// Create a partial match result
    pub const fn partial(index: c_int) -> Self {
        Self {
            found: true,
            index,
            exact: false,
        }
    }
}

impl Default for ArgEntryMatch {
    fn default() -> Self {
        Self::not_found()
    }
}

// =============================================================================
// Entry Info
// =============================================================================

/// Information about an argument entry for display
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ArgEntryInfo {
    /// Entry index (1-based for display)
    pub display_index: c_int,
    /// Total count
    pub total: c_int,
    /// Whether this is current
    pub is_current: bool,
    /// Flags
    pub flags: ArgEntryFlags,
}

impl Default for ArgEntryInfo {
    fn default() -> Self {
        Self {
            display_index: 0,
            total: 0,
            is_current: false,
            flags: ArgEntryFlags::none(),
        }
    }
}

impl ArgEntryInfo {
    /// Create display info for an entry
    pub const fn new(index: c_int, total: c_int, is_current: bool, flags: ArgEntryFlags) -> Self {
        Self {
            display_index: index + 1, // Convert to 1-based
            total,
            is_current,
            flags,
        }
    }

    /// Get display string like "(1 of 5)"
    pub fn position_string(&self) -> [u8; 32] {
        let mut buf = [0u8; 32];
        // Just store the numbers for C to format
        buf[0] = (self.display_index & 0xFF) as u8;
        buf[1] = ((self.display_index >> 8) & 0xFF) as u8;
        buf[2] = (self.total & 0xFF) as u8;
        buf[3] = ((self.total >> 8) & 0xFF) as u8;
        buf
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if flags is edited
#[no_mangle]
pub extern "C" fn rs_argentry_flags_is_edited(flags: u32) -> c_int {
    c_int::from(ArgEntryFlags::from_raw(flags).is_edited())
}

/// FFI export: Check if flags is loaded
#[no_mangle]
pub extern "C" fn rs_argentry_flags_is_loaded(flags: u32) -> c_int {
    c_int::from(ArgEntryFlags::from_raw(flags).is_loaded())
}

/// FFI export: Check if flags is modified
#[no_mangle]
pub extern "C" fn rs_argentry_flags_is_modified(flags: u32) -> c_int {
    c_int::from(ArgEntryFlags::from_raw(flags).is_modified())
}

/// FFI export: Create new entry
#[no_mangle]
pub extern "C" fn rs_argentry_new(index: c_int) -> ArgEntry {
    ArgEntry::new(index)
}

/// FFI export: Create entry with buffer
#[no_mangle]
pub extern "C" fn rs_argentry_with_buffer(index: c_int, buf_nr: c_int) -> ArgEntry {
    ArgEntry::with_buffer(index, buf_nr)
}

/// FFI export: Check if entry is valid
#[no_mangle]
pub extern "C" fn rs_argentry_is_valid(entry: *const ArgEntry) -> c_int {
    if entry.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*entry).is_valid() })
}

/// FFI export: Check if entry has buffer
#[no_mangle]
pub extern "C" fn rs_argentry_has_buffer(entry: *const ArgEntry) -> c_int {
    if entry.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*entry).has_buffer() })
}

/// FFI export: Create not-found match
#[no_mangle]
pub extern "C" fn rs_argentry_match_not_found() -> ArgEntryMatch {
    ArgEntryMatch::not_found()
}

/// FFI export: Create exact match
#[no_mangle]
pub extern "C" fn rs_argentry_match_exact(index: c_int) -> ArgEntryMatch {
    ArgEntryMatch::exact(index)
}

/// FFI export: Create entry info
#[no_mangle]
pub extern "C" fn rs_argentry_info_new(
    index: c_int,
    total: c_int,
    is_current: c_int,
    flags: u32,
) -> ArgEntryInfo {
    ArgEntryInfo::new(
        index,
        total,
        is_current != 0,
        ArgEntryFlags::from_raw(flags),
    )
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::borrow_as_ptr)]
mod tests {
    use super::*;

    #[test]
    fn test_argentry_flags() {
        let flags = ArgEntryFlags::none();
        assert!(!flags.is_edited());
        assert!(!flags.is_loaded());

        let flags = ArgEntryFlags::from_raw(ARG_EDITED | ARG_LOADED);
        assert!(flags.is_edited());
        assert!(flags.is_loaded());
        assert!(!flags.is_readonly());
    }

    #[test]
    fn test_argentry_flags_set() {
        let mut flags = ArgEntryFlags::none();
        flags.set_edited(true);
        assert!(flags.is_edited());

        flags.set_loaded(true);
        assert!(flags.is_loaded());

        flags.set_edited(false);
        assert!(!flags.is_edited());
    }

    #[test]
    fn test_argentry() {
        let entry = ArgEntry::new(0);
        assert!(entry.is_valid());
        assert!(!entry.has_buffer());

        let with_buf = ArgEntry::with_buffer(1, 5);
        assert!(with_buf.has_buffer());
        assert!(with_buf.flags.is_loaded());
    }

    #[test]
    fn test_argentry_match() {
        let not_found = ArgEntryMatch::not_found();
        assert!(!not_found.found);

        let exact = ArgEntryMatch::exact(5);
        assert!(exact.found);
        assert!(exact.exact);
        assert_eq!(exact.index, 5);

        let partial = ArgEntryMatch::partial(3);
        assert!(partial.found);
        assert!(!partial.exact);
    }

    #[test]
    fn test_argentry_info() {
        let info = ArgEntryInfo::new(0, 5, true, ArgEntryFlags::none());
        assert_eq!(info.display_index, 1); // 1-based
        assert!(info.is_current);
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_argentry_flags_is_edited(ARG_EDITED), 1);
        assert_eq!(rs_argentry_flags_is_edited(0), 0);

        let entry = rs_argentry_new(0);
        assert_eq!(rs_argentry_is_valid(&entry), 1);
    }
}
