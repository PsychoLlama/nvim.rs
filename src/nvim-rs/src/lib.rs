//! Rust components for Neovim
//!
//! This crate provides Rust implementations of Neovim functionality,
//! designed to be called from C code via FFI during the migration period.
//!
//! The crate re-exports all FFI functions from sub-crates so they are
//! available in the single `libnvim_rs.a` static library.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)] // FFI functions need unsafe but docs come later
#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::wildcard_imports)] // We want to re-export everything

// Re-export all FFI functions from sub-crates
// This ensures they're included in the static library
pub use nvim_math::*;
pub use nvim_mbyte::*;
pub use nvim_path::*;
pub use nvim_strings::*;

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
