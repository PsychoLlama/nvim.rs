//! Registering the whole `vim._ts_*` surface on the main state.
//!
//! [`nlua_treesitter_init`] builds the six metatables and sets the module
//! functions on the table on top. The two `*_language_version` functions
//! report the ABI range this build accepts.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/treesitter/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::cursor::{
    Match, QUERY_CURSOR_METHODS, QUERY_MATCH_METHODS, QueryCursor, create_query_cursor,
};
use super::lang::{add_language, has_language, inspect_language, remove_language};
use super::node::NODE_METHODS;
use super::parser::{PARSER_METHODS, Parser, create_parser};
use super::query::{QUERY_METHODS, parse_query};
use super::tree::{TREE_METHODS, Tree};
use super::{
    Node, Query, TREE_SITTER_LANGUAGE_VERSION, TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
    register_class, ts_set_allocator,
};
use crate::lua::stack::{Raise, Stack};
use crate::lua_functions;
use crate::memory::{xcalloc, xfree, xmalloc, xrealloc};

lua_functions! {
    /// The module functions, set on the `vim` table.
    enum TreesitterFunctions in TREESITTER_FUNCTIONS {
        0 => c"_create_ts_parser": create_parser,
        1 => c"_create_ts_querycursor": create_query_cursor,
        2 => c"_ts_add_language_from_object": add_language,
        3 => c"_ts_has_language": has_language,
        4 => c"_ts_remove_language": remove_language,
        5 => c"_ts_inspect_language": inspect_language,
        6 => c"_ts_parse_query": parse_query,
        7 => c"_ts_get_language_version": language_version,
        8 => c"_ts_get_minimum_language_version": minimum_language_version,
    }
}

/// `vim._ts_get_language_version()`.
fn language_version(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.push_number(f64::from(TREE_SITTER_LANGUAGE_VERSION));
    Ok(1)
}

/// `vim._ts_get_minimum_language_version()`.
fn minimum_language_version(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.push_number(f64::from(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION));
    Ok(1)
}

/// Register the metatables, route tree-sitter's allocations through the
/// editor's allocator, and set the module functions on the table on top.
pub fn nlua_treesitter_init(stack: &mut Stack) {
    register_class::<Parser>(stack, PARSER_METHODS);
    register_class::<Tree>(stack, TREE_METHODS);
    register_class::<Node>(stack, NODE_METHODS);
    register_class::<Query>(stack, QUERY_METHODS);
    register_class::<QueryCursor>(stack, QUERY_CURSOR_METHODS);
    register_class::<Match>(stack, QUERY_MATCH_METHODS);
    // SAFETY: four functions with the C allocator's contracts, which
    // tree-sitter calls from here on.
    unsafe {
        ts_set_allocator(Some(xmalloc), Some(xcalloc), Some(xrealloc), Some(xfree));
    }
    stack.register_functions(TREESITTER_FUNCTIONS);
}
