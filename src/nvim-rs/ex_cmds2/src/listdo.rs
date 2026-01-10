//! List-do command utilities (:argdo, :windo, :bufdo, :tabdo, etc.)
//!
//! This module provides utilities for commands that operate on lists of items.

use std::ffi::c_int;

// =============================================================================
// List-Do Command Types
// =============================================================================

/// Types of list-do commands
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListDoType {
    /// :argdo - operate on argument list
    Argdo = 0,
    /// :windo - operate on windows
    Windo = 1,
    /// :bufdo - operate on buffers
    Bufdo = 2,
    /// :tabdo - operate on tabs
    Tabdo = 3,
    /// :cdo - operate on quickfix entries
    Cdo = 4,
    /// :ldo - operate on location list entries
    Ldo = 5,
    /// :cfdo - operate on quickfix files
    Cfdo = 6,
    /// :lfdo - operate on location list files
    Lfdo = 7,
}

impl ListDoType {
    /// Create from integer value
    pub fn from_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Argdo),
            1 => Some(Self::Windo),
            2 => Some(Self::Bufdo),
            3 => Some(Self::Tabdo),
            4 => Some(Self::Cdo),
            5 => Some(Self::Ldo),
            6 => Some(Self::Cfdo),
            7 => Some(Self::Lfdo),
            _ => None,
        }
    }

    /// Check if this is a quickfix-related command
    pub fn is_quickfix(self) -> bool {
        matches!(self, Self::Cdo | Self::Ldo | Self::Cfdo | Self::Lfdo)
    }

    /// Check if this is a location-list command
    pub fn is_location_list(self) -> bool {
        matches!(self, Self::Ldo | Self::Lfdo)
    }

    /// Check if this operates on files rather than entries
    pub fn is_file_based(self) -> bool {
        matches!(self, Self::Cfdo | Self::Lfdo)
    }

    /// Check if this needs syntax autocommand handling
    pub fn needs_syntax_handling(self) -> bool {
        !matches!(self, Self::Windo | Self::Tabdo)
    }
}

/// Check if listdo type is quickfix-related
#[no_mangle]
pub extern "C" fn rs_listdo_is_quickfix(typ: c_int) -> bool {
    ListDoType::from_int(typ).is_some_and(|t| t.is_quickfix())
}

/// Check if listdo type is location-list
#[no_mangle]
pub extern "C" fn rs_listdo_is_location_list(typ: c_int) -> bool {
    ListDoType::from_int(typ).is_some_and(|t| t.is_location_list())
}

/// Check if listdo type is file-based
#[no_mangle]
pub extern "C" fn rs_listdo_is_file_based(typ: c_int) -> bool {
    ListDoType::from_int(typ).is_some_and(|t| t.is_file_based())
}

/// Check if listdo type needs syntax handling
#[no_mangle]
pub extern "C" fn rs_listdo_needs_syntax_handling(typ: c_int) -> bool {
    ListDoType::from_int(typ).is_some_and(|t| t.needs_syntax_handling())
}

// =============================================================================
// Listdo State
// =============================================================================

/// State for tracking listdo command progress
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ListDoState {
    /// Current index
    pub index: c_int,
    /// Start line/index
    pub start: c_int,
    /// End line/index
    pub end: c_int,
    /// Next buffer file number (for bufdo)
    pub next_fnum: c_int,
    /// Whether we're still processing
    pub busy: bool,
}

/// Initialize listdo state
#[no_mangle]
pub extern "C" fn rs_listdo_state_init(start: c_int, end: c_int) -> ListDoState {
    ListDoState {
        index: 0,
        start,
        end,
        next_fnum: -1,
        busy: false,
    }
}

/// Check if listdo should continue
#[no_mangle]
pub unsafe extern "C" fn rs_listdo_should_continue(state: *const ListDoState) -> bool {
    if state.is_null() {
        return false;
    }

    let state = &*state;
    state.busy && state.index <= state.end
}

/// Increment listdo index
#[no_mangle]
pub unsafe extern "C" fn rs_listdo_advance(state: *mut ListDoState) {
    if state.is_null() {
        return;
    }

    (*state).index += 1;
}

// =============================================================================
// Compiler Support
// =============================================================================

/// Common compiler script extensions
pub const COMPILER_EXTENSIONS: &[&str] = &["vim", "lua"];

/// Get number of compiler extensions
#[no_mangle]
pub extern "C" fn rs_compiler_ext_count() -> c_int {
    COMPILER_EXTENSIONS.len() as c_int
}

/// Check if extension is a compiler script extension
#[no_mangle]
pub extern "C" fn rs_is_compiler_ext(ext: *const std::ffi::c_char, len: usize) -> bool {
    if ext.is_null() || len == 0 {
        return false;
    }

    let slice = unsafe { std::slice::from_raw_parts(ext.cast::<u8>(), len) };
    let Ok(ext_str) = std::str::from_utf8(slice) else {
        return false;
    };

    COMPILER_EXTENSIONS.contains(&ext_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listdo_type() {
        // Quickfix checks
        assert!(!rs_listdo_is_quickfix(0)); // argdo
        assert!(!rs_listdo_is_quickfix(1)); // windo
        assert!(rs_listdo_is_quickfix(4)); // cdo
        assert!(rs_listdo_is_quickfix(5)); // ldo
        assert!(rs_listdo_is_quickfix(6)); // cfdo
        assert!(rs_listdo_is_quickfix(7)); // lfdo

        // Location list checks
        assert!(!rs_listdo_is_location_list(4)); // cdo
        assert!(rs_listdo_is_location_list(5)); // ldo
        assert!(!rs_listdo_is_location_list(6)); // cfdo
        assert!(rs_listdo_is_location_list(7)); // lfdo

        // File-based checks
        assert!(!rs_listdo_is_file_based(4)); // cdo
        assert!(!rs_listdo_is_file_based(5)); // ldo
        assert!(rs_listdo_is_file_based(6)); // cfdo
        assert!(rs_listdo_is_file_based(7)); // lfdo

        // Syntax handling checks
        assert!(rs_listdo_needs_syntax_handling(0)); // argdo
        assert!(!rs_listdo_needs_syntax_handling(1)); // windo
        assert!(rs_listdo_needs_syntax_handling(2)); // bufdo
        assert!(!rs_listdo_needs_syntax_handling(3)); // tabdo
    }

    #[test]
    fn test_listdo_state() {
        unsafe {
            let mut state = rs_listdo_state_init(1, 10);
            assert_eq!(state.index, 0);
            assert_eq!(state.start, 1);
            assert_eq!(state.end, 10);
            assert!(!state.busy);

            state.busy = true;
            assert!(rs_listdo_should_continue(&state));

            for _ in 0..11 {
                rs_listdo_advance(&mut state);
            }
            assert!(!rs_listdo_should_continue(&state));
        }
    }

    #[test]
    fn test_compiler_extensions() {
        assert_eq!(rs_compiler_ext_count(), 2);
        assert!(rs_is_compiler_ext(b"vim\0".as_ptr().cast(), 3));
        assert!(rs_is_compiler_ext(b"lua\0".as_ptr().cast(), 3));
        assert!(!rs_is_compiler_ext(b"py\0".as_ptr().cast(), 2));
    }
}
