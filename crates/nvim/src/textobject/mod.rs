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
//! pending operator's `op`, and inside it *extends* the selection instead --
//! which is why each of the five carries a retry or an `extend` path, and why
//! `'selection'` being exclusive is adjusted for on the way in and undone on
//! the way out.
//!
//! This parent keeps no functions, only the vocabulary the children share.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::c_uint;

use crate::types::MotionType;

mod pair;
mod para;
mod quote;
mod sent;
mod word;

pub(crate) use self::pair::*;
pub use self::para::*;
pub(crate) use self::quote::*;
pub(crate) use self::sent::*;
pub(crate) use self::word::*;

/// A text object or motion that ran out of text: nothing to select, or no
/// further to move. Nothing is said about it; the caller beeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NoObject;

pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
/// `findmatchlimit`'s "search forward" flag.
pub const FM_FORWARD: c_uint = 2;
