//! Completion sources for command-line expansion
//!
//! This module provides specialized completion helpers for various
//! completion contexts like commands, files, options, buffers, etc.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

pub mod buffer;
pub mod command;
pub mod file;
pub mod help;
pub mod option;
pub mod usercmd;
