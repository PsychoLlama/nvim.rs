//! Package/plugin management
//!
//! This module handles loading plugins from 'packpath' directories.

use std::ffi::{c_char, c_int};

use crate::dip;

// =============================================================================
// Package Loading State
// =============================================================================

/// Package load status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageStatus {
    /// Package not yet loaded
    NotLoaded = 0,
    /// Package currently loading
    Loading = 1,
    /// Package loaded successfully
    Loaded = 2,
    /// Package load failed
    Failed = 3,
}

impl PackageStatus {
    /// Convert from integer
    pub const fn from_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::NotLoaded),
            1 => Some(Self::Loading),
            2 => Some(Self::Loaded),
            3 => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Check if a package status indicates it can be loaded.
#[no_mangle]
pub extern "C" fn rs_package_can_load(status: c_int) -> bool {
    status == PackageStatus::NotLoaded as c_int
}

/// Check if a package status indicates it's already loaded.
#[no_mangle]
pub extern "C" fn rs_package_is_loaded(status: c_int) -> bool {
    status == PackageStatus::Loaded as c_int
}

// =============================================================================
// Package Type
// =============================================================================

/// Package type (start vs opt)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageType {
    /// Loaded at startup (pack/*/start/*)
    Start = 0,
    /// Loaded on demand (pack/*/opt/*)
    Opt = 1,
}

impl PackageType {
    /// Get the corresponding DIP flag
    pub const fn to_dip_flag(self) -> c_int {
        match self {
            Self::Start => dip::START,
            Self::Opt => dip::OPT,
        }
    }
}

/// Get DIP flag for start packages.
#[no_mangle]
pub extern "C" fn rs_package_start_flag() -> c_int {
    PackageType::Start.to_dip_flag()
}

/// Get DIP flag for opt packages.
#[no_mangle]
pub extern "C" fn rs_package_opt_flag() -> c_int {
    PackageType::Opt.to_dip_flag()
}

// =============================================================================
// Package Name Handling
// =============================================================================

/// Check if a package name is valid.
///
/// Package names must not be empty, contain path separators, or contain wildcards.
///
/// # Safety
///
/// `name` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_package_name_valid(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }

    let first = *name as u8;
    if first == 0 {
        return false; // Empty name
    }

    let mut p = name;
    while *p != 0 {
        let c = *p as u8;
        // Reject path separators
        if c == b'/' || c == b'\\' {
            return false;
        }
        // Reject wildcards
        if c == b'*' || c == b'?' || c == b'[' {
            return false;
        }
        p = p.add(1);
    }

    true
}

// =============================================================================
// Plugin Directory Names
// =============================================================================

/// Well-known plugin subdirectories that should be searched.
pub const PLUGIN_DIRS: &[&[u8]] = &[
    b"plugin\0",
    b"autoload\0",
    b"colors\0",
    b"compiler\0",
    b"doc\0",
    b"ftdetect\0",
    b"ftplugin\0",
    b"indent\0",
    b"keymap\0",
    b"lang\0",
    b"syntax\0",
];

/// Number of plugin directories
pub const PLUGIN_DIR_COUNT: usize = PLUGIN_DIRS.len();

/// Get a plugin directory name by index.
///
/// Returns null if index is out of bounds.
#[no_mangle]
pub extern "C" fn rs_get_plugin_dir(idx: usize) -> *const c_char {
    if idx >= PLUGIN_DIR_COUNT {
        return std::ptr::null();
    }
    PLUGIN_DIRS[idx].as_ptr().cast::<c_char>()
}

/// Get the count of plugin directories.
#[no_mangle]
pub extern "C" fn rs_plugin_dir_count() -> usize {
    PLUGIN_DIR_COUNT
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_package_status() {
        assert!(rs_package_can_load(PackageStatus::NotLoaded as c_int));
        assert!(!rs_package_can_load(PackageStatus::Loading as c_int));
        assert!(!rs_package_can_load(PackageStatus::Loaded as c_int));

        assert!(rs_package_is_loaded(PackageStatus::Loaded as c_int));
        assert!(!rs_package_is_loaded(PackageStatus::NotLoaded as c_int));
    }

    #[test]
    fn test_package_type_flags() {
        assert_eq!(rs_package_start_flag(), dip::START);
        assert_eq!(rs_package_opt_flag(), dip::OPT);
    }

    #[test]
    fn test_package_name_valid() {
        unsafe {
            let valid = CString::new("vim-plugin").unwrap();
            assert!(rs_package_name_valid(valid.as_ptr()));

            let empty = CString::new("").unwrap();
            assert!(!rs_package_name_valid(empty.as_ptr()));

            let with_slash = CString::new("plugin/sub").unwrap();
            assert!(!rs_package_name_valid(with_slash.as_ptr()));

            let with_wild = CString::new("plugin*").unwrap();
            assert!(!rs_package_name_valid(with_wild.as_ptr()));

            assert!(!rs_package_name_valid(std::ptr::null()));
        }
    }

    #[test]
    fn test_plugin_dirs() {
        assert_eq!(rs_plugin_dir_count(), PLUGIN_DIR_COUNT);
        assert!(rs_plugin_dir_count() > 0);

        // First should be "plugin"
        let first = rs_get_plugin_dir(0);
        assert!(!first.is_null());

        // Out of bounds returns null
        let oob = rs_get_plugin_dir(1000);
        assert!(oob.is_null());
    }
}
