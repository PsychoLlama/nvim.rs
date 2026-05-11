//! Tree-sitter Lua bindings migrated from src/nvim/lua/treesitter.c
//!
//! This crate provides Rust implementations of the TSNode, TSTree, TSQuery,
//! TSQueryCursor, and TSQueryMatch Lua metatable methods.

#![allow(clippy::missing_safety_doc)]
// All functions here are unsafe extern "C" callbacks. The lint
// unsafe_op_in_unsafe_fn would require unsafe{} blocks around every FFI call
// inside unsafe fns, which is overly verbose for leaf Lua callbacks.
// The workspace enables it as a warn but we suppress it for this crate.
#![allow(unsafe_op_in_unsafe_fn)]
// `L` is the conventional Lua state parameter name throughout the Lua C API.
// Renaming it would hurt readability for anyone familiar with the C API.
#![allow(non_snake_case)]
// Helper functions mirror the Lua C API naming convention (luaL_*, lua_*).
#![allow(clippy::used_underscore_binding)]
// TS* and Lua* identifiers are C API names — doc_markdown backtick lint is noise.
#![allow(clippy::doc_markdown)]
// Casts in FFI code (e.g. luaL_checkint returns i64, tree-sitter uses u32) are
// intentional; wrapping/truncation checks would obscure the logic.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
// Raw-pointer-to-raw-pointer casts without constness change are idiomatic FFI.
#![allow(clippy::ptr_as_ptr)]
// Wildcard imports from ts_sys bring in all Lua/TS FFI bindings — intentional.
#![allow(clippy::wildcard_imports)]
// is_multiple_of is nightly-only; modulo comparison is fine in stable.
#![allow(clippy::manual_is_multiple_of)]
// Single-use map_or: explicit if/else is clearer in unsafe FFI context.
#![allow(clippy::option_if_let_else)]
// query_err_string is a long but linear function with no natural split point.
#![allow(clippy::too_many_lines)]

pub mod node;
pub mod parser;
pub mod query;
pub mod querycursor;
pub mod querymatch;
pub mod tree;
pub mod ts_sys;
pub mod types;
