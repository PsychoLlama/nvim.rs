//! OS abstraction layer for Neovim
//!
//! This module provides portable OS abstractions for:
//! - Environment variables
//! - Time operations
//! - Filesystem operations
//! - Process information
//!
//! These implementations are designed to work alongside nvim's existing
//! C code during the migration period.

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::similar_names)] // Common pattern in FFI code: name_cstr -> name_str
#![allow(clippy::manual_let_else)] // Match expressions are clearer for FFI error handling
#![allow(clippy::cast_possible_truncation)] // Intentional truncation in time functions
#![allow(clippy::cast_possible_wrap)] // Time values won't wrap in practice
#![allow(clippy::borrow_as_ptr)] // Clearer in libc calls
#![allow(clippy::option_if_let_else)] // Match is clearer for complex error handling

pub mod env;
pub mod fs;
pub mod mem;
pub mod proc;
pub mod time;

use std::ffi::c_int;

/// Return values matching nvim's OK/FAIL
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
