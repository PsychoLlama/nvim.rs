//! The internal `nvim__*` inspection surface.
//!
//! None of this is published in the api metadata: the four `nvim__id*`
//! functions exist to exercise the msgpack round trip, `nvim__stats` and
//! `nvim__inspect_cell` report internals the test suite asserts on, and
//! `nvim_get_proc`/`nvim_get_proc_children` are the process-tree queries
//! the job-control tests use.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::validate::err_bad_number;
use crate::api::private::validate::handle_arg;
use crate::api_error;
use crate::grid::default_grid_ref;
use crate::log::logmsg;
use crate::narrow::{len_as_integer, number_as_int};
use crate::popupmenu::pum_grid_ref;
use crate::types::DictBuilder;

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
        (c"log_skip", Object::integer(Integer::from(stats.log_skip))),
        (
            c"lua_refcount",
            Object::integer(Integer::from(lua_refcount)),
        ),
        (c"redraw", Object::integer(stats.redraw)),
        (
            c"arena_alloc_count",
            Object::integer(len_as_integer(arena_alloc_count.get())),
        ),
        (
            c"ts_query_parse_count",
            Object::integer(tslua_query_parse_count.get().cast_signed()),
        ),
    ];
    let mut rv = DictBuilder::with_capacity(entries.len());
    for (key, value) in entries {
        rv.insert(key, value);
    }
    rv.into()
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
    let a = vec![Object::integer(pid.into())];
    let o = nlua_exec(
        b"return vim._os_proc_children(...)",
        None,
        a.into(),
        kRetObject,
    )?;
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
    let a = vec![Object::integer(pid)];
    let o = nlua_exec(b"return vim._os_proc_info(...)", None, a.into(), kRetObject)?;
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
    let mut ret = Vec::new();
    let mut g: GridRef = default_grid_ref();
    if grid == Integer::from(pum_grid_ref().handle) {
        g = pum_grid_ref();
    } else if grid > 1 as Integer {
        let wp = get_win_by_grid_handle(handle_arg(c"grid handle", grid)?)
            .filter(|wp| wp.w_grid_alloc.is_allocated());
        let Some(wp) = wp else {
            return Err(err_bad_number(c"grid handle", grid));
        };
        g = wp.grid_alloc();
    }
    if row < 0 as Integer
        || row >= Integer::from(g.rows)
        || col < 0 as Integer
        || col >= Integer::from(g.cols)
    {
        return Ok(ret.into());
    }
    ret = Vec::with_capacity(3 as size_t);
    let off: size_t = g.cell_offset(number_as_int(row), number_as_int(col));
    let mut sc_buf: SCharBuf = [0; SCHAR_BUF_SIZE];
    ret.push(Object::string(ApiString::from_bytes(schar_text(
        g.char_at(off),
        &mut sc_buf,
    ))));
    let attr: ::core::ffi::c_int = g.attr_at(off) as ::core::ffi::c_int;
    let hl = Object::dict(hl_get_attr_by_id(Integer::from(attr), true)?);
    ret.push(hl);
    if !highlight_use_hlstate() {
        ret.push(Object::array(hl_inspect(attr)));
    }
    Ok(ret.into())
}

// `nvim__screenshot` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__screenshot(path: ApiString) {
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
pub fn nvim__unpack(str: ApiString) -> Result<Object, Error> {
    unpack(str.as_bytes())
}
