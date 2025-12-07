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
pub use nvim_arabic::*;
pub use nvim_ascii::*;
pub use nvim_buffer::*;
pub use nvim_charset::*;
pub use nvim_cmdhist::*;
pub use nvim_collections::garray::*;
pub use nvim_collections::hashtab::*;
// Note: rs_hash_hash, rs_hash_hash_len are in nvim_memutil (not re-exported from hashtab)
pub use nvim_encoding::base64::*;
pub use nvim_encoding::sha256::*;
pub use nvim_eval::*;
pub use nvim_ex_docmd::*;
pub use nvim_fileio::*;
pub use nvim_grid::*;
pub use nvim_help::*;
pub use nvim_indent::*;
pub use nvim_keycodes::*;
pub use nvim_mark::*;
pub use nvim_math::*;
pub use nvim_mbyte::*;
pub use nvim_memutil::*;
pub use nvim_menu::*;
pub use nvim_ops::*;
pub use nvim_os::env::*;
pub use nvim_os::fs::*;
pub use nvim_os::time::*;
pub use nvim_path::*;
pub use nvim_profile::*;
pub use nvim_register::*;
pub use nvim_spell::*;
pub use nvim_strings::*;
pub use nvim_utf8proc::*;
pub use nvim_version::*;
pub use nvim_window::*;

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
