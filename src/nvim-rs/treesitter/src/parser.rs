//! Lua metatable methods for TSParser userdata (partial).
//!
//! Phase 3: parser_tostring, parser_reset (orphan leaf functions).
//! Deferred: parser_parse, parser_gc, parser_set_logger, parser_get_logger.

use std::ffi::{c_int, c_void};

use crate::ts_sys::*;
use crate::types::TS_META_PARSER;

extern "C" {
    fn ts_parser_reset(parser: *mut c_void);
}

/// Check and return TSParser pointer at stack index.
///
/// # Safety
/// `L` must be a valid Lua state.
#[inline]
unsafe fn parser_check(L: *mut c_void, index: c_int) -> *mut c_void {
    let ud = luaL_checkudata(L, index, TS_META_PARSER.as_ptr()) as *mut *mut c_void;
    luaL_argcheck(
        L,
        c_int::from(!(*ud).is_null()),
        index,
        c"TSParser expected".as_ptr(),
    );
    *ud
}

/// __tostring: "<parser>"
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_parser_tostring")]
pub unsafe extern "C" fn parser_tostring(L: *mut c_void) -> c_int {
    lua_pushstring(L, c"<parser>".as_ptr());
    1
}

/// reset: ts_parser_reset
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_parser_reset")]
pub unsafe extern "C" fn parser_reset(L: *mut c_void) -> c_int {
    let p = parser_check(L, 1);
    ts_parser_reset(p);
    0
}
