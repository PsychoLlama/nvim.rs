//! Text objects and the motions that share their rules.
//!
//! One question per file -- "which region of the buffer does this object
//! name" -- with the motions that answer it by walking rather than by
//! selecting kept beside it, because they are the same rule read twice.
//!
//! | file | objects | motions |
//! | --- | --- | --- |
//! | `word` | `iw` `aw` `iW` `aW` | `w` `W` `b` `B` `e` `E` `ge` `gE` |
//! | `sent` | `is` `as` | `(` `)` |
//! | `para` | `ip` `ap` | `{` `}` `[[` `]]` `[]` `][` |
//! | `pair` | `i(` `a{` ... `it` `at` | -- |
//! | `quote` | `i"` `a'` ... | -- |
//!
//! This parent keeps no functions, only the vocabulary the children share.

mod pair;
mod para;
mod quote;
mod sent;
mod word;

pub use self::pair::*;
pub use self::para::*;
pub use self::quote::*;
pub use self::sent::*;
pub use self::word::*;

use crate::src::nvim::types::MotionType;

pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const FM_FORWARD: C2Rust_Unnamed_14 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CPO_ENDOFSENT: ::core::ffi::c_int = 'J' as ::core::ffi::c_int;
pub const CPO_MATCHBSL: ::core::ffi::c_int = 'M' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
