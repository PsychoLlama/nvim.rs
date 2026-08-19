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
//! Every object here shares one shape: outside Visual mode it fills in the
//! pending operator's `oap`, and inside it *extends* the selection instead --
//! which is why each of the five carries a retry or an `extend` path, and why
//! `'selection'` being exclusive is adjusted for on the way in and undone on
//! the way out.
//!
//! This parent keeps no functions, only the vocabulary the children share.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_uint, c_void};

use crate::types::MotionType;

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

pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
/// `findmatchlimit`'s "search forward" flag.
pub const FM_FORWARD: c_uint = 2;
pub const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();
/// 'cpoptions' `J`: a sentence ends at `.` `!` `?` followed by *two* spaces.
pub const CPO_ENDOFSENT: c_int = 'J' as c_int;
/// 'cpoptions' `M`: a `\` before a bracket does not make it escaped.
pub const CPO_MATCHBSL: c_int = 'M' as c_int;
