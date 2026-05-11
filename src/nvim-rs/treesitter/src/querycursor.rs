//! Lua metatable methods for TSQueryCursor userdata.
//!
//! Phase 3: querycursor_gc, querycursor_remove_match.
//! Phase 4: querycursor_next_match, querycursor_next_capture.

use std::ffi::{c_int, c_void};

use crate::querymatch::push_querymatch;
use crate::ts_sys::*;
use crate::types::{TSQueryMatch, TS_META_QUERYCURSOR};

// =============================================================================
// Internal helpers
// =============================================================================

/// Check and return TSQueryCursor pointer at stack index.
///
/// # Safety
/// `L` must be a valid Lua state.
#[inline]
unsafe fn querycursor_check(L: *mut c_void, index: c_int) -> *mut c_void {
    let ud = luaL_checkudata(L, index, TS_META_QUERYCURSOR.as_ptr()) as *mut *mut c_void;
    luaL_argcheck(
        L,
        c_int::from(!(*ud).is_null()),
        index,
        c"TSQueryCursor expected".as_ptr(),
    );
    *ud
}

// =============================================================================
// Phase 3 implementations
// =============================================================================

/// __gc: ts_query_cursor_delete
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_querycursor_gc")]
pub unsafe extern "C" fn querycursor_gc(L: *mut c_void) -> c_int {
    let cursor = querycursor_check(L, 1);
    ts_query_cursor_delete(cursor);
    0
}

/// remove_match: ts_query_cursor_remove_match
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_querycursor_remove_match")]
pub unsafe extern "C" fn querycursor_remove_match(L: *mut c_void) -> c_int {
    let cursor = querycursor_check(L, 1);
    let match_id = luaL_checkinteger(L, 2) as u32;
    ts_query_cursor_remove_match(cursor, match_id);
    0
}

// =============================================================================
// Phase 4: next_match, next_capture
// =============================================================================

/// next_match: advance cursor, push querymatch or return 0
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_querycursor_next_match")]
pub unsafe extern "C" fn querycursor_next_match(L: *mut c_void) -> c_int {
    let cursor = querycursor_check(L, 1);

    let mut match_: TSQueryMatch = std::mem::zeroed();
    if !ts_query_cursor_next_match(cursor, &raw mut match_) {
        return 0;
    }

    push_querymatch(L, &raw const match_, 1);
    1
}

/// next_capture: advance cursor, push capture_index + node + querymatch
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_querycursor_next_capture")]
pub unsafe extern "C" fn querycursor_next_capture(L: *mut c_void) -> c_int {
    let cursor = querycursor_check(L, 1);

    let mut match_: TSQueryMatch = std::mem::zeroed();
    let mut capture_index: u32 = 0;
    if !ts_query_cursor_next_capture(cursor, &raw mut match_, &raw mut capture_index) {
        return 0;
    }

    let capture = &*match_.captures.add(capture_index as usize);

    lua_pushinteger(L, i64::from(capture.index) + 1); // [index]
    crate::node::push_node(L, capture.node, 1); // [index, node]
    push_querymatch(L, &raw const match_, 1);

    3
}
