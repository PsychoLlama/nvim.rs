//! The internal `nvim__*` inspection surface.
//!
//! None of this is published in the api metadata: the four `nvim__id*`
//! functions exist to exercise the msgpack round trip, `nvim__stats` and
//! `nvim__inspect_cell` report internals the test suite asserts on, and
//! `nvim_get_proc`/`nvim_get_proc_children` are the process-tree queries
//! the job-control tests use.

#![forbid(unsafe_code)]

use super::*;
use crate::api::private::validate::err_bad_number;
use crate::api_error;
use crate::grid::default_grid_ref;
use crate::log::logmsg;
use crate::popupmenu::pum_grid_ref;

// `nvim__id` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__id(obj: Object) -> Object {
    obj.clone()
}

// `nvim__id_array` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__id_array(arr: Array) -> Array {
    arr.clone()
}

// `nvim__id_dict` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__id_dict(dct: ApiDict) -> ApiDict {
    dct.clone()
}

// `nvim__id_float` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__id_float(flt: Float) -> Float {
    flt
}

/// The counters the test suite asserts on: syncs, skipped log lines, live
/// Lua references, redraws and arena allocations.
// `nvim__stats` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__stats() -> ApiDict {
    let stats = g_stats.get();
    let lua_refcount = nlua_get_global_ref_count();
    let entries = [
        (c"fsync", Object::integer(stats.fsync)),
        (c"log_skip", Object::integer(stats.log_skip as Integer)),
        (c"lua_refcount", Object::integer(lua_refcount as Integer)),
        (c"redraw", Object::integer(stats.redraw)),
        (
            c"arena_alloc_count",
            Object::integer(arena_alloc_count.get() as Integer),
        ),
        (
            c"ts_query_parse_count",
            Object::integer(tslua_query_parse_count.get() as Integer),
        ),
    ];
    let mut rv: ApiDict = ApiDict::with_capacity(entries.len());
    for (key, value) in entries {
        rv.insert(key, value);
    }
    rv
}

/// The pids of `pid`'s child processes.
pub fn nvim_get_proc_children(pid: Integer) -> Result<Array, Error> {
    let Some(pid) = ::core::ffi::c_int::try_from(pid)
        .ok()
        .filter(|&pid| pid > 0)
    else {
        return Err(err_bad_number(c"pid", pid));
    };
    // Only "could not inspect" (2) is reachable on this platform.
    if let Some(children) = os_proc_children(pid) {
        return Ok(children
            .into_iter()
            .map(|pid| Object::integer(pid.into()))
            .collect());
    }
    logmsg!(
        LOGLVL_DBG,
        c"nvim_get_proc_children",
        1924,
        "fallback to vim._os_proc_children()"
    );
    let mut a: Array = Array::with_capacity(1);
    a.push(Object::integer(pid.into()));
    let o = nlua_exec(b"return vim._os_proc_children(...)", None, a, kRetObject)?;
    o.into_array().ok_or_else(|| {
        api_error!(
            kErrorTypeException,
            "Failed to get process children. pid={pid} error=2"
        )
    })
}

/// What the OS says about process `pid`, or nil when there is none.
pub fn nvim_get_proc(pid: Integer) -> Result<Object, Error> {
    if !(pid > 0 && pid <= Integer::from(::core::ffi::c_int::MAX)) {
        return Err(err_bad_number(c"pid", pid));
    }
    let mut a: Array = Array::with_capacity(1);
    a.push(Object::integer(pid));
    let o = nlua_exec(b"return vim._os_proc_info(...)", None, a, kRetObject)?;
    if o.as_array().is_some_and(|array| array.is_empty()) {
        return Ok(Object::Nil);
    }
    if !matches!(o, Object::Dict(_)) {
        return Err(api_error!(
            kErrorTypeException,
            "Failed to get process info. pid={pid}"
        ));
    }
    Ok(o)
}

/// The text, attributes and highlight of one screen cell.
// `nvim__inspect_cell` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__inspect_cell(grid: Integer, row: Integer, col: Integer) -> Result<Array, Error> {
    let mut ret: Array = Array::EMPTY;
    let mut g: GridRef = default_grid_ref();
    if grid == pum_grid_ref().handle as Integer {
        g = pum_grid_ref();
    } else if grid > 1 as Integer {
        let wp = get_win_by_grid_handle(grid as Handle).filter(|wp| wp.w_grid_alloc.is_allocated());
        let Some(wp) = wp else {
            return Err(err_bad_number(c"grid handle", grid));
        };
        g = wp.grid_alloc();
    }
    if row < 0 as Integer
        || row >= g.rows as Integer
        || col < 0 as Integer
        || col >= g.cols as Integer
    {
        return Ok(ret);
    }
    ret = Array::with_capacity(3 as size_t);
    let off: size_t = g.cell_offset(row as ::core::ffi::c_int, col as ::core::ffi::c_int);
    let mut sc_buf: SCharBuf = [0; SCHAR_BUF_SIZE];
    ret.push(Object::string(String_0::from_bytes(schar_text(
        g.char_at(off),
        &mut sc_buf,
    ))));
    let attr: ::core::ffi::c_int = g.attr_at(off) as ::core::ffi::c_int;
    let hl = Object::dict(hl_get_attr_by_id(attr as Integer, true)?);
    ret.push(hl);
    if !highlight_use_hlstate() {
        ret.push(Object::array(hl_inspect(attr)));
    }
    Ok(ret)
}

// `nvim__screenshot` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__screenshot(path: String_0) {
    ui_call_screenshot(path);
}

// `nvim__invalidate_glyph_cache` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__invalidate_glyph_cache() {
    schar_cache_clear();
    must_redraw.set(UPD_CLEAR);
}

// `nvim__unpack` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__unpack(str: String_0) -> Result<Object, Error> {
    unpack(str.as_bytes())
}
