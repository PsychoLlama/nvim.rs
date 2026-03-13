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
//! - `path`: File and path operations (fnamemodify, glob, etc.)
//! - `display`: UI and display functions (screenrow, winwidth, etc.)
//! - `channel`: Channel and job functions
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
pub mod buffer;
pub mod channel;
pub mod collections;
pub mod cursor;
pub mod dict;
mod dispatch;
pub mod display;
pub mod list;
pub mod math;
pub mod misc;
pub mod path;
pub mod random;
pub mod search;
pub mod string;
pub mod system;
pub mod text;
pub mod timer;
pub mod types;
pub mod window;

pub use channel::*;
pub use collections::*;
pub use dict::*;
pub use dispatch::*;
pub use display::*;
pub use list::*;
pub use path::*;
pub use string::*;
