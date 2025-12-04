//! Rust components for Neovim
//!
//! This crate provides Rust implementations of Neovim functionality,
//! designed to be called from C code via FFI during the migration period.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)] // FFI functions need unsafe but docs come later
#![allow(unsafe_code)] // FFI requires unsafe

/// FFI-safe result type for operations that can fail
#[repr(C)]
pub struct NvimResult<T> {
    pub ok: bool,
    pub value: T,
}

/// Placeholder function to verify the build system works.
/// This will be removed once real functionality is added.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn nvim_rs_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(nvim_rs_version(), 1);
    }
}
