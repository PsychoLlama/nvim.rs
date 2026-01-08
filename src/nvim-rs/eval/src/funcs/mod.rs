//! VimL built-in functions (migrated from `src/nvim/eval/funcs.c`)
//!
//! This module provides Rust implementations of VimL built-in functions
//! like `abs()`, `sin()`, `split()`, `join()`, `map()`, `filter()`, etc.
//!
//! ## Module Structure
//!
//! Functions are organized by domain:
//! - `math`: Pure math functions (abs, sin, cos, sqrt, etc.)
//! - `bitwise`: Bitwise operations (and, or, xor, invert)
//! - `types`: Type inspection and conversion (type, typename, etc.)
//! - `string`: String manipulation (split, join, tolower, etc.)
//! - `list`: List operations (add, insert, remove, etc.)
//! - `dict`: Dictionary operations (keys, values, items, etc.)
//!
//! ## FFI Pattern
//!
//! Each function follows the VimL function signature:
//! ```ignore
//! extern "C" fn rs_f_<name>(
//!     argvars: *const typval_T,
//!     rettv: *mut typval_T,
//! )
//! ```
//!
//! The C wrapper in funcs.c calls these Rust implementations directly.

pub mod bitwise;
mod dispatch;
pub mod math;
pub mod random;
pub mod types;

pub use dispatch::*;
