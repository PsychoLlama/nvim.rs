//! Lua metatable methods for TSQueryMatch userdata.
//!
//! Phase 3: querymatch_info.
//! Phase 4: querymatch_captures, push_querymatch helper.

use std::ffi::{c_int, c_void};
use std::mem::size_of;

use crate::ts_sys::*;
use crate::types::{TSQueryMatch, LUA_REGISTRYINDEX, TS_META_QUERYMATCH};

// =============================================================================
// Internal helpers
// =============================================================================

/// Push a TSQueryMatch userdata onto the stack, copying the fenv from `uindex`.
///
/// # Safety
/// `L` must be a valid Lua state. `match_` must be a valid pointer.
pub unsafe fn push_querymatch(L: *mut c_void, match_: *const TSQueryMatch, uindex: c_int) {
    let ud = lua_newuserdata(L, size_of::<TSQueryMatch>()) as *mut TSQueryMatch; // [udata]
    *ud = *match_;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERYMATCH.as_ptr()); // [udata, meta]
    lua_setmetatable(L, -2); // [udata]

    lua_getfenv(L, uindex); // [udata, reftable]
    lua_setfenv(L, -2); // [udata]
}

// =============================================================================
// Phase 3 implementations
// =============================================================================

/// info: push match id + pattern_index + 1
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_querymatch_info")]
pub unsafe extern "C" fn querymatch_info(L: *mut c_void) -> c_int {
    let match_ = luaL_checkudata(L, 1, TS_META_QUERYMATCH.as_ptr()) as *const TSQueryMatch;
    lua_pushinteger(L, i64::from((*match_).id));
    lua_pushinteger(L, i64::from((*match_).pattern_index) + 1);
    2
}

// =============================================================================
// Phase 4: captures
// =============================================================================

/// captures: build table of capture_index -> [nodes...]
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_querymatch_captures")]
pub unsafe extern "C" fn querymatch_captures(L: *mut c_void) -> c_int {
    let match_ = luaL_checkudata(L, 1, TS_META_QUERYMATCH.as_ptr()) as *const TSQueryMatch;
    lua_newtable(L); // [match, nodes, captures]
    for i in 0..(*match_).capture_count as usize {
        let capture = &*(*match_).captures.add(i);
        let index = i64::from(capture.index) + 1;

        lua_rawgeti(L, -1, index as c_int); // [match, node, captures]
        if lua_isnil(L, -1) != 0 {
            // [match, node, captures, nil]
            lua_pop(L, 1); // [match, node, captures]
            lua_newtable(L); // [match, node, captures, nodes]
        }
        crate::node::push_node(L, capture.node, 1); // [match, node, captures, nodes, node]
        let next_idx = lua_objlen(L, -2) as c_int + 1;
        lua_rawseti(L, -2, next_idx); // [match, node, captures, nodes]
        lua_rawseti(L, -2, index as c_int); // [match, node, captures]
    }
    1
}
