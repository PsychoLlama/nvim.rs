//! Lua metatable methods for TSQuery userdata.
//!
//! Phase 3: query_tostring, query_gc, query_disable_capture, query_disable_pattern.
//! Phase 4: query_inspect, query_err_string, query_err_to_string.

use std::ffi::{c_int, c_void};

use crate::ts_sys::*;
use crate::types::{TSQueryPredicateStepType, TS_META_QUERY};

// =============================================================================
// Internal helpers
// =============================================================================

/// Check and return TSQuery pointer at stack index.
///
/// # Safety
/// `L` must be a valid Lua state.
#[inline]
unsafe fn query_check(L: *mut c_void, index: c_int) -> *mut c_void {
    let ud = luaL_checkudata(L, index, TS_META_QUERY.as_ptr()) as *mut *mut c_void;
    luaL_argcheck(
        L,
        c_int::from(!(*ud).is_null()),
        index,
        c"TSQuery expected".as_ptr(),
    );
    *ud
}

// =============================================================================
// Phase 3 implementations
// =============================================================================

/// __tostring: "<query>"
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_query_tostring")]
pub unsafe extern "C" fn query_tostring(L: *mut c_void) -> c_int {
    lua_pushstring(L, c"<query>".as_ptr());
    1
}

/// __gc: ts_query_delete
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_query_gc")]
pub unsafe extern "C" fn query_gc(L: *mut c_void) -> c_int {
    let query = query_check(L, 1);
    ts_query_delete(query);
    0
}

/// disable_capture: ts_query_disable_capture
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_query_disable_capture")]
pub unsafe extern "C" fn query_disable_capture(L: *mut c_void) -> c_int {
    let query = query_check(L, 1);
    let mut name_len: usize = 0;
    let name = luaL_checklstring(L, 2, &raw mut name_len);
    ts_query_disable_capture(query, name, name_len as u32);
    0
}

/// disable_pattern: ts_query_disable_pattern
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_query_disable_pattern")]
pub unsafe extern "C" fn query_disable_pattern(L: *mut c_void) -> c_int {
    let query = query_check(L, 1);
    let pattern_index = luaL_checkinteger(L, 2) as u32;
    ts_query_disable_pattern(query, pattern_index - 1);
    0
}

// =============================================================================
// Phase 4: query_inspect
// =============================================================================

/// inspect: build TSQueryInfo table {patterns = ..., captures = ...}
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_query_inspect")]
pub unsafe extern "C" fn query_inspect(L: *mut c_void) -> c_int {
    let query = query_check(L, 1);

    lua_createtable(L, 0, 2); // [retval]

    let n_pat = ts_query_pattern_count(query);
    lua_createtable(L, n_pat as c_int, 1); // [retval, patterns]
    for i in 0..n_pat {
        let mut len: u32 = 0;
        let step = ts_query_predicates_for_pattern(query, i, &raw mut len);
        if len == 0 {
            continue;
        }
        lua_createtable(L, (len / 4) as c_int, 1); // [retval, patterns, pat]
        lua_createtable(L, 3, 0); // [retval, patterns, pat, pred]
        let mut nextpred: c_int = 1;
        let mut nextitem: c_int = 1;
        for k in 0..len {
            let s = &*step.add(k as usize);
            if s.type_ == TSQueryPredicateStepType::Done {
                lua_rawseti(L, -2, nextpred); // [retval, patterns, pat]
                nextpred += 1;
                lua_createtable(L, 3, 0); // [retval, patterns, pat, pred]
                nextitem = 1;
                continue;
            }
            if s.type_ == TSQueryPredicateStepType::String {
                let mut slen: u32 = 0;
                let str_ptr = ts_query_string_value_for_id(query, s.value_id, &raw mut slen);
                lua_pushlstring(L, str_ptr, slen as usize);
            } else if s.type_ == TSQueryPredicateStepType::Capture {
                lua_pushinteger(L, i64::from(s.value_id) + 1);
            } else {
                std::process::abort();
            }
            lua_rawseti(L, -2, nextitem); // [retval, patterns, pat, pred]
            nextitem += 1;
        }
        // last predicate should have ended with TypeDone, pop the trailing empty pred
        lua_pop(L, 1); // [retval, patterns, pat]
        lua_rawseti(L, -2, (i + 1) as c_int); // [retval, patterns]
    }
    lua_setfield(L, -2, c"patterns".as_ptr()); // [retval]

    let n_captures = ts_query_capture_count(query);
    lua_createtable(L, n_captures as c_int, 0); // [retval, captures]
    for i in 0..n_captures {
        let mut slen: u32 = 0;
        let str_ptr = ts_query_capture_name_for_id(query, i, &raw mut slen);
        lua_pushlstring(L, str_ptr, slen as usize);
        lua_rawseti(L, -2, (i + 1) as c_int);
    }
    lua_setfield(L, -2, c"captures".as_ptr()); // [retval]

    1
}

// =============================================================================
// Phase 4: query_err_string (pure string work, no macros)
// =============================================================================

/// Convert TSQueryError enum to a descriptive prefix string.
///
/// # Safety
/// Always safe — pure function.
const unsafe fn query_err_to_string(error_type: u32) -> *const std::ffi::c_char {
    // TSQueryError values: None=0, Syntax=1, NodeType=2, Field=3, Capture=4, Structure=5, Language=6
    match error_type {
        1 => c"Invalid syntax:\n".as_ptr(),
        2 => c"Invalid node type ".as_ptr(),
        3 => c"Invalid field name ".as_ptr(),
        4 => c"Invalid capture name ".as_ptr(),
        5 => c"Impossible pattern:\n".as_ptr(),
        _ => c"error".as_ptr(),
    }
}

/// Format a query error into `err_buf` (up to `errlen` bytes).
///
/// Exposed as `rs_ts_query_err_string` so `tslua_parse_query` in C can call it.
///
/// # Safety
/// `src` must be a valid C string. `err_buf` must be writable for `errlen` bytes.
#[unsafe(export_name = "rs_ts_query_err_string")]
pub unsafe extern "C" fn query_err_string(
    src: *const std::ffi::c_char,
    error_offset: c_int,
    error_type: u32,
    err_buf: *mut std::ffi::c_char,
    errlen: usize,
) {
    if errlen == 0 {
        return;
    }

    // Build a Rust slice view over the source
    let src_bytes: &[u8] = {
        let mut len = 0usize;
        while *src.add(len) != 0 {
            len += 1;
        }
        std::slice::from_raw_parts(src as *const u8, len)
    };

    let error_offset_usize = if error_offset >= 0 && (error_offset as usize) <= src_bytes.len() {
        error_offset as usize
    } else {
        src_bytes.len()
    };

    // Find row/column and the error line
    let mut line_start = 0usize;
    let mut row = 0u32;
    let mut error_line: Option<&[u8]> = None;

    let mut pos = 0usize;
    loop {
        let next_nl = src_bytes[pos..].iter().position(|&b| b == b'\n');
        let line_end = match next_nl {
            Some(rel) => pos + rel,
            None => src_bytes.len(),
        };
        if line_end > error_offset_usize {
            error_line = Some(&src_bytes[pos..line_end]);
            line_start = pos;
            break;
        }
        pos = line_end + 1;
        row += 1;
        if next_nl.is_none() {
            break;
        }
    }

    let column = error_offset_usize.saturating_sub(line_start) as u32;

    let type_msg_ptr = query_err_to_string(error_type);
    let type_msg = std::ffi::CStr::from_ptr(type_msg_ptr).to_bytes();

    // Write into err_buf using a Rust slice
    let out = std::slice::from_raw_parts_mut(err_buf as *mut u8, errlen);
    let mut written = 0usize;

    // Helper closure: write bytes, respect errlen-1 limit (leave room for NUL)
    macro_rules! write_bytes {
        ($bytes:expr) => {
            for &b in $bytes {
                if written + 1 >= errlen {
                    break;
                }
                out[written] = b;
                written += 1;
            }
        };
    }

    // "Query error at ROW:COL. TYPE_MSG"
    let prefix = format!("Query error at {}:{}. ", row + 1, column + 1);
    write_bytes!(prefix.as_bytes());
    write_bytes!(type_msg);

    // For types that report names (NodeType=2, Field=3, Capture=4)
    if error_type == 2 || error_type == 3 || error_type == 4 {
        let suffix = &src_bytes[error_offset_usize..];
        let is_anonymous =
            error_type == 2 && error_offset_usize > 0 && src_bytes[error_offset_usize - 1] == b'"';

        let suffix_len = if is_anonymous {
            let mut backslashes = 0u32;
            let mut len = 0usize;
            loop {
                if len >= suffix.len() {
                    break;
                }
                let c = suffix[len];
                if c == b'"' && backslashes % 2 == 0 {
                    break;
                }
                if c == b'\\' {
                    backslashes += 1;
                } else {
                    backslashes = 0;
                }
                len += 1;
            }
            len
        } else {
            let mut len = 0usize;
            while len < suffix.len() {
                let c = suffix[len];
                if c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'.' {
                    len += 1;
                } else {
                    break;
                }
            }
            len
        };

        // Write: "\"SUFFIX\":\n"
        write_bytes!(b"\"");
        write_bytes!(&suffix[..suffix_len.min(suffix.len())]);
        write_bytes!(b"\":\n");
    }

    // Write error line (if found)
    if let Some(line) = error_line {
        write_bytes!(line);
        write_bytes!(b"\n");
        // Write column indicator: spaces + "^" + "\n"
        for _ in 0..column {
            write_bytes!(b" ");
        }
        write_bytes!(b"^\n");
    } else {
        write_bytes!(b"Unexpected EOF\n");
    }

    // NUL-terminate
    if written < errlen {
        out[written] = 0;
    } else {
        out[errlen - 1] = 0;
    }
}
