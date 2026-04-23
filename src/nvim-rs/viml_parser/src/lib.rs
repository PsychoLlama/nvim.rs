//! VimL Expression Parser Utilities
//!
//! This crate provides utilities for parsing VimL expressions, including:
//! - Token classification and scanning
//! - Number literal parsing (decimal, hex, octal, binary, float)
//! - String literal parsing and escape handling
//! - Expression operator precedence

#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::approx_constant)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::use_self)]

pub mod ast;
pub mod expr_types;
pub mod lexer;
pub mod literal;
pub mod repr;
pub mod string_tables;
pub mod token;

// Re-export common types
pub use literal::{NumberBase, ParsedNumber};
pub use token::{Token, TokenKind};
