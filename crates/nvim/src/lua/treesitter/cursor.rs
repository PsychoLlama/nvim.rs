//! The query cursor, and the match it hands back.
//!
//! `vim._create_ts_querycursor(node, query, opts)` makes a cursor over
//! (query, node) with the point range, start depth and match limit applied,
//! and its `next_capture` / `next_match` methods drive it. A match is
//! pushed as its own userdatum ([`Match`]) rather than copied, so the
//! capture list is only materialised if Lua asks for it.
//!
//! A cursor's environment is `{tree, query, cursor}`, and every node and
//! match it answers shares it: the tree the nodes point into, the query the
//! cursor walks and the cursor whose memory a match's captures live in all
//! outlive whatever still reads them. (Upstream gave the cursor the node's
//! environment, which keeps only the tree: a match read after its cursor
//! was collected read freed memory.)

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

use core::ffi::{CStr, c_int};
use core::ptr::NonNull;

use super::node::{check_node, push_node};
use super::query::check_query;
use super::*;
use crate::lua::stack::{Kind, Raise};
use crate::lua_functions;

/// A query cursor, as its userdatum owns it. Empty once collected.
#[derive(Default)]
pub(crate) struct QueryCursor(Option<NonNull<TSQueryCursor>>);

impl Drop for QueryCursor {
    fn drop(&mut self) {
        if let Some(cursor) = self.0.take() {
            // SAFETY: the cursor this userdatum owns, deleted once.
            unsafe { ts_query_cursor_delete(cursor.as_ptr()) };
        }
    }
}

impl Userdata for QueryCursor {
    const METATABLE: &'static CStr = c"treesitter_querycursor";
}

/// A match, as its userdatum holds it: its captures are the cursor's.
#[derive(Clone, Copy)]
pub(crate) struct Match(TSQueryMatch);

impl Default for Match {
    fn default() -> Match {
        Match(TSQueryMatch {
            id: 0,
            pattern_index: 0,
            capture_count: 0,
            captures: core::ptr::null(),
        })
    }
}

impl Userdata for Match {
    const METATABLE: &'static CStr = c"treesitter_querymatch";
}

lua_functions! {
    /// The methods of a query cursor.
    pub(crate) enum QueryCursorMethods in QUERY_CURSOR_METHODS {
        0 => c"remove_match": cursor_remove_match,
        1 => c"next_capture": cursor_next_capture,
        2 => c"next_match": cursor_next_match,
    }
}

lua_functions! {
    /// The methods of a match.
    pub(crate) enum QueryMatchMethods in QUERY_MATCH_METHODS {
        0 => c"info": match_info,
        1 => c"captures": match_captures,
    }
}

/// Field `name` of the options table in slot 3, as `luaL_checkinteger`
/// reads it (the field numbered -1 in its error); `None` for a nil one when
/// `optional`.
fn option(stack: &mut Stack, name: &CStr, optional: bool) -> Result<Option<u32>, Raise> {
    stack.get_field(At::arg(3), name);
    if optional && stack.kind(At::TOP) == Kind::Nil {
        stack.pop(1);
        return Ok(None);
    }
    // A refusal leaves the field on top, where the check's error finds it.
    let value = wrap_u32(stack.check_integer(At::TOP)?);
    stack.pop(1);
    Ok(Some(value))
}

/// `vim._create_ts_querycursor(node, query, opts)`.
pub(crate) fn create_query_cursor(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    let query = check_query(stack, At::arg(2))?;
    if stack.top().index() >= 3
        && stack.kind(At::arg(3)) != Kind::Nil
        && stack.kind(At::arg(3)) != Kind::Table
    {
        return Err(Raise::argument(3, "table expected"));
    }
    let mut point = [0; 4];
    for (field, name) in point
        .iter_mut()
        .zip([c"start_row", c"start_col", c"end_row", c"end_col"])
    {
        *field = option(stack, name, false)?.unwrap_or_default();
    }
    let max_start_depth = option(stack, c"max_start_depth", true)?;
    let match_limit = option(stack, c"match_limit", true)?;

    let [start_row, start_col, end_row, end_col] = point;
    // SAFETY: a fresh cursor, run over the query and the node the userdata
    // in slots 1 and 2 keep alive -- and the environment set below keeps
    // them alive for as long as the cursor.
    let cursor = unsafe {
        let cursor = ts_query_cursor_new();
        ts_query_cursor_set_point_range(
            cursor,
            TSPoint {
                row: start_row,
                column: start_col,
            },
            TSPoint {
                row: end_row,
                column: end_col,
            },
        );
        if let Some(depth) = max_start_depth {
            ts_query_cursor_set_max_start_depth(cursor, depth);
        }
        if let Some(limit) = match_limit {
            ts_query_cursor_set_match_limit(cursor, limit);
        }
        ts_query_cursor_exec(cursor, query, node);
        cursor
    };
    stack.new_userdata(QueryCursor(NonNull::new(cursor)));
    // {tree, query, cursor}: the tree from the node's environment.
    stack.new_table(3, 0);
    stack.push_environment(At::arg(1));
    stack.raw_get_index(At::TOP, 1);
    stack.raw_set_index(At::from_top(3), 1);
    stack.pop(1);
    stack.push_value(At::arg(2));
    stack.raw_set_index(At::from_top(2), 2);
    stack.push_value(At::from_top(2));
    stack.raw_set_index(At::from_top(2), 3);
    stack.set_environment(At::from_top(2));
    Ok(1)
}

/// `luaL_checkudata` for a cursor: its `TSQueryCursor *`, refusing a
/// collected one.
fn check_cursor(stack: &mut Stack, at: At) -> Result<*mut TSQueryCursor, Raise> {
    match stack.check_userdata::<QueryCursor>(at)?.0 {
        Some(cursor) => Ok(cursor.as_ptr()),
        None => Err(Raise::argument(at.index(), "TSQueryCursor expected")),
    }
}

fn cursor_remove_match(stack: &mut Stack) -> Result<c_int, Raise> {
    let cursor = check_cursor(stack, At::arg(1))?;
    let id = wrap_u32(stack.check_integer(At::arg(2))?);
    // SAFETY: the cursor the userdatum owns.
    unsafe { ts_query_cursor_remove_match(cursor, id) };
    Ok(0)
}

/// Push `found` as a match userdatum sharing the environment of the
/// userdatum at `owner`.
fn push_match(stack: &mut Stack, found: TSQueryMatch, owner: At) {
    stack.new_userdata(Match(found));
    stack.push_environment(owner);
    stack.set_environment(At::from_top(2));
}

/// `cursor:next_capture()`: the capture's one-based id, its node, and its
/// match; nothing at the end.
fn cursor_next_capture(stack: &mut Stack) -> Result<c_int, Raise> {
    let cursor = check_cursor(stack, At::arg(1))?;
    let mut found = Match::default().0;
    let mut capture_index: u32 = 0;
    // SAFETY: the cursor the userdatum owns; a match it answers points at
    // `capture_count` captures in the cursor's memory, which the cursor's
    // environment keeps alive for every userdatum that shares it.
    let capture = unsafe {
        if !ts_query_cursor_next_capture(cursor, &raw mut found, &raw mut capture_index) {
            return Ok(0);
        }
        *found.captures.add(capture_index as usize)
    };
    stack.push_integer(integer(capture.index) + 1);
    push_node(stack, capture.node, At::arg(1));
    push_match(stack, found, At::arg(1));
    Ok(3)
}

/// `cursor:next_match()`: the next match; nothing at the end.
fn cursor_next_match(stack: &mut Stack) -> Result<c_int, Raise> {
    let cursor = check_cursor(stack, At::arg(1))?;
    let mut found = Match::default().0;
    // SAFETY: the cursor the userdatum owns.
    if !unsafe { ts_query_cursor_next_match(cursor, &raw mut found) } {
        return Ok(0);
    }
    push_match(stack, found, At::arg(1));
    Ok(1)
}

/// `match:info()`: its id and its one-based pattern index.
fn match_info(stack: &mut Stack) -> Result<c_int, Raise> {
    let found = stack.check_userdata::<Match>(At::arg(1))?.0;
    stack.push_integer(integer(found.id));
    stack.push_integer(integer(u32::from(found.pattern_index)) + 1);
    Ok(2)
}

/// `match:captures()`: `{[capture id] = {node, ..}}`.
fn match_captures(stack: &mut Stack) -> Result<c_int, Raise> {
    let found = stack.check_userdata::<Match>(At::arg(1))?.0;
    stack.new_table(0, 0);
    let captures = if found.captures.is_null() {
        &[][..]
    } else {
        // SAFETY: the match's captures, in the memory of the cursor its
        // environment keeps alive.
        unsafe { core::slice::from_raw_parts(found.captures, usize::from(found.capture_count)) }
    };
    for capture in captures {
        let index = c_int::try_from(capture.index)
            .unwrap_or(c_int::MAX)
            .saturating_add(1);
        stack.raw_get_index(At::TOP, index);
        if stack.kind(At::TOP) == Kind::Nil {
            stack.pop(1);
            stack.new_table(0, 0);
        }
        push_node(stack, capture.node, At::arg(1));
        let next = c_int::try_from(stack.raw_len(At::from_top(2)))
            .unwrap_or(c_int::MAX)
            .saturating_add(1);
        stack.raw_set_index(At::from_top(2), next);
        stack.raw_set_index(At::from_top(2), index);
    }
    Ok(1)
}
