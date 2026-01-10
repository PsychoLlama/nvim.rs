//! Additional Ex command utilities for Neovim
//!
//! This module provides utilities for commands like `:argdo`, `:windo`, `:bufdo`,
//! `:tabdo`, and related Ex commands.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::manual_c_str_literals)]

use std::ffi::c_int;

mod autowrite;
mod bufcheck;
mod listdo;

pub use autowrite::*;
pub use bufcheck::*;
pub use listdo::*;

// =============================================================================
// Check Changed Flags
// =============================================================================

bitflags::bitflags! {
    /// Flags for check_changed function
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CcgdFlags: u32 {
        /// Do autowrite if buffer is modified
        const AW = 1;
        /// Add '!' to cmdline
        const FORCEIT = 2;
        /// Buffer is in multiple windows
        const MULTWIN = 4;
        /// Error message is for ex command
        const EXCMD = 8;
        /// May write to any buffer
        const ALLBUF = 16;
    }
}

/// Get the AW flag value
#[no_mangle]
pub extern "C" fn rs_ccgd_aw() -> c_int {
    CcgdFlags::AW.bits() as c_int
}

/// Get the FORCEIT flag value
#[no_mangle]
pub extern "C" fn rs_ccgd_forceit() -> c_int {
    CcgdFlags::FORCEIT.bits() as c_int
}

/// Get the MULTWIN flag value
#[no_mangle]
pub extern "C" fn rs_ccgd_multwin() -> c_int {
    CcgdFlags::MULTWIN.bits() as c_int
}

/// Get the EXCMD flag value
#[no_mangle]
pub extern "C" fn rs_ccgd_excmd() -> c_int {
    CcgdFlags::EXCMD.bits() as c_int
}

/// Get the ALLBUF flag value
#[no_mangle]
pub extern "C" fn rs_ccgd_allbuf() -> c_int {
    CcgdFlags::ALLBUF.bits() as c_int
}

// =============================================================================
// Dialog message size constant
// =============================================================================

/// Size of dialog message buffer
pub const DIALOG_MSG_SIZE: usize = 256;

/// Get dialog message buffer size
#[no_mangle]
pub extern "C" fn rs_dialog_msg_size() -> usize {
    DIALOG_MSG_SIZE
}

// =============================================================================
// Script Host Names
// =============================================================================

/// Script host names for ex commands
pub const SCRIPT_HOSTS: &[&str] = &["ruby", "python3", "perl"];

/// Get number of script hosts
#[no_mangle]
pub extern "C" fn rs_script_host_count() -> c_int {
    SCRIPT_HOSTS.len() as c_int
}

/// Check if name is a valid script host
#[no_mangle]
pub extern "C" fn rs_is_script_host(name: *const std::ffi::c_char, len: usize) -> bool {
    if name.is_null() || len == 0 {
        return false;
    }

    let slice = unsafe { std::slice::from_raw_parts(name.cast::<u8>(), len) };
    let Ok(name_str) = std::str::from_utf8(slice) else {
        return false;
    };

    SCRIPT_HOSTS.contains(&name_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ccgd_flags() {
        assert_eq!(rs_ccgd_aw(), 1);
        assert_eq!(rs_ccgd_forceit(), 2);
        assert_eq!(rs_ccgd_multwin(), 4);
        assert_eq!(rs_ccgd_excmd(), 8);
        assert_eq!(rs_ccgd_allbuf(), 16);
    }

    #[test]
    fn test_dialog_msg_size() {
        assert_eq!(rs_dialog_msg_size(), 256);
    }

    #[test]
    fn test_script_hosts() {
        assert_eq!(rs_script_host_count(), 3);
        assert!(rs_is_script_host(b"ruby\0".as_ptr().cast(), 4));
        assert!(rs_is_script_host(b"python3\0".as_ptr().cast(), 7));
        assert!(rs_is_script_host(b"perl\0".as_ptr().cast(), 4));
        assert!(!rs_is_script_host(b"lua\0".as_ptr().cast(), 3));
    }
}
