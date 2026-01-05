//! Text object selection and navigation for Neovim
//!
//! This crate provides Rust implementations of text object functions
//! from `src/nvim/textobject.c`. It handles text object selection (aw, iw, as, is,
//! ap, ip, a", i", a{, i{, etc.) and word motions (w, W, b, B, e, E).

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

/// Direction: move forward
pub const FORWARD: c_int = 1;

/// Direction: move backward
pub const BACKWARD: c_int = -1;

/// Function succeeded
pub const OK: c_int = 1;

/// Function failed
pub const FAIL: c_int = 0;

/// NUL character
pub const NUL: c_int = 0;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a window (win_T*).
pub type WinHandle = *mut std::ffi::c_void;

/// Opaque handle to a buffer (buf_T*).
pub type BufHandle = *mut std::ffi::c_void;

/// Opaque handle to operator arguments (oparg_T*).
pub type OapHandle = *mut std::ffi::c_void;

/// Opaque handle to a position (pos_T*).
pub type PosHandle = *mut std::ffi::c_void;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(NUL, 0);
    }
}
