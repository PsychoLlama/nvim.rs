//! Collection data structures for Neovim
//!
//! This module provides C-compatible implementations of:
//! - Growing arrays (garray) - dynamic arrays that grow as needed
//! - Hash tables (hashtab) - string-keyed hash tables
//!
//! These are designed to be compatible with nvim's existing C code.

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::doc_markdown)] // Technical identifiers like garray_T are intentional
#![allow(clippy::ptr_as_ptr)] // Common pattern in FFI code
#![allow(clippy::borrow_as_ptr)] // Common pattern in FFI tests
#![allow(clippy::manual_div_ceil)] // More explicit for documentation

pub mod garray;
pub mod hashtab;
