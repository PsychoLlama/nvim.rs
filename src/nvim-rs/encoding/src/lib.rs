//! Encoding utilities for Neovim
//!
//! This module provides:
//! - Base64 encoding/decoding
//! - SHA-256 hashing
//!
//! These are designed to be compatible with nvim's existing C code.

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::manual_is_multiple_of)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::single_match_else)]
#![allow(clippy::many_single_char_names)] // SHA-256 uses standard naming a-h
#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::format_push_string)]
#![allow(static_mut_refs)] // Needed for static hex buffer

pub mod base64;
pub mod sha256;
