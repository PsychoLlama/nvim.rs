//! The internal `nvim__*` inspection surface.
//!
//! None of this is published in the api metadata: the four `nvim__id*`
//! functions exist to exercise the msgpack round trip, `nvim__stats` and
//! `nvim__inspect_cell` report internals the test suite asserts on, and
//! `nvim_get_proc`/`nvim_get_proc_children` are the process-tree queries
//! the job-control tests use.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api::private::validate::err_bad_number;
use crate::api_error;
use crate::cstr;
use crate::grid::default_grid_ref;
use crate::log::logmsg;
use crate::popupmenu::pum_grid_ref;
use crate::winlayer::Win;
use core::ptr;

/// # Safety
///
/// `obj` must be a well-formed API object the caller owns for the call.
/// `arena` must point at a live arena, which the memory this answers with is
/// taken from and must outlive.
// `nvim__id` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub unsafe fn nvim__id(obj: Object) -> Object {
    obj.clone()
}

/// # Safety
///
/// `arr` must be a well-formed API array, its `size` elements initialized.
/// `arena` must point at a live arena, which the memory this answers with is
/// taken from and must outlive.
// `nvim__id_array` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub unsafe fn nvim__id_array(arr: Array) -> Array {
    arr.clone()
}

/// # Safety
///
/// `dct` must be a well-formed API dictionary, its `size` entries
/// initialized. `arena` must point at a live arena, which the memory this
/// answers with is taken from and must outlive.
// `nvim__id_dict` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub unsafe fn nvim__id_dict(dct: ApiDict) -> ApiDict {
    dct.clone()
}

// `nvim__id_float` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__id_float(flt: Float) -> Float {
    flt
}

/// The counters the test suite asserts on: syncs, skipped log lines, live
/// Lua references, redraws and arena allocations.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
// `nvim__stats` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub unsafe fn nvim__stats() -> ApiDict {
    let stats = g_stats.get();
    // SAFETY: the Lua state exists from startup to exit.
    let lua_refcount = unsafe { nlua_get_global_ref_count() };
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
        // SAFETY: `rv` is the dict the arena just sized for these six keys.
        rv.insert(String_0::from_cstr(key), value);
    }
    rv
}

/// # Safety
///
/// `arena` must point at a live arena, which the memory this answers with is
/// taken from and must outlive.
pub unsafe fn nvim_get_proc_children(pid: Integer, arena: *mut Arena) -> Result<Array, Error> {
    let mut error = Error::none();
    let mut rv: ::core::ffi::c_int = 0;
    let mut rvobj: Array = Array::EMPTY;
    let mut children: Vec<::core::ffi::c_int> = Vec::new();
    if !(pid > 0 as Integer && pid <= 2147483647 as Integer) {
        let name = c"pid".as_ptr();
        // SAFETY: `error` is this frame's own slot and `name` a literal.
        error = err_bad_number(unsafe { cstr::at(name) }, pid);
    } else {
        match os_proc_children(pid as ::core::ffi::c_int) {
            Some(pids) => children = pids,
            // Only "could not inspect" is reachable on this platform.
            None => rv = 2 as ::core::ffi::c_int,
        }
        if rv == 2 as ::core::ffi::c_int {
            logmsg!(
                LOGLVL_DBG,
                c"nvim_get_proc_children",
                1924,
                "fallback to vim._os_proc_children()"
            );
            let mut a: Array = Array::with_capacity(1);
            a.push(Object::integer(pid));
            let code = String_0::from_cstr(c"return vim._os_proc_children(...)");
            let name = ::core::ptr::null::<::core::ffi::c_char>();
            // SAFETY: `a` is the one-slot block above, `arena` is the
            // caller's and `error` this frame's own slot.
            let o = match unsafe { nlua_exec(&code, name, a, kRetObject, arena) } {
                Ok(value) => value,
                Err(e) => {
                    error = e;
                    Object::Nil
                }
            };
            if let Some(array) = o.into_array() {
                rvobj = array;
            } else if !(error.kind() as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
            {
                error = api_error!(
                    kErrorTypeException,
                    "Failed to get process children. pid={pid} error={rv}"
                );
            }
        } else {
            rvobj = Array::with_capacity(children.len() as size_t);
            for pid in children {
                rvobj.push(Object::integer(pid as Integer));
            }
        }
    }
    rvobj.reported(error)
}

/// # Safety
///
/// `arena` must point at a live arena, which the memory this answers with is
/// taken from and must outlive.
pub unsafe fn nvim_get_proc(pid: Integer, arena: *mut Arena) -> Result<Object, Error> {
    let mut error = Error::none();
    let mut rvobj: Object = Object::Nil;
    if !(pid > 0 as Integer && pid <= 2147483647 as Integer) {
        let name = c"pid".as_ptr();
        // SAFETY: `error` is this frame's own slot and `name` a literal.
        error = err_bad_number(unsafe { cstr::at(name) }, pid);
        return Object::Nil.reported(error);
    }
    let mut a: Array = Array::with_capacity(1);
    a.push(Object::integer(pid));
    let code = String_0::from_cstr(c"return vim._os_proc_info(...)");
    let name = ::core::ptr::null::<::core::ffi::c_char>();
    // SAFETY: `a` is the one-slot block above, `arena` is the caller's and
    // `error` this frame's own slot.
    let o = match unsafe { nlua_exec(&code, name, a, kRetObject, arena) } {
        Ok(value) => value,
        Err(e) => {
            error = e;
            Object::Nil
        }
    };
    if o.as_array().is_some_and(|array| array.len() == 0 as size_t) {
        return Object::Nil.reported(error);
    } else if matches!(o, Object::Dict(_)) {
        rvobj = o;
    } else if !(error.kind() as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        error = api_error!(kErrorTypeException, "Failed to get process info. pid={pid}");
    }
    rvobj.reported(error)
}

/// # Safety
///
/// `arena` must point at a live arena, which the memory this answers with is
/// taken from and must outlive.
// `nvim__inspect_cell` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub unsafe fn nvim__inspect_cell(
    grid: Integer,
    row: Integer,
    col: Integer,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = Error::none();
    let mut ret: Array = Array::EMPTY;
    let mut g: GridRef = default_grid_ref();
    if grid == pum_grid_ref().handle as Integer {
        g = pum_grid_ref();
    } else if grid > 1 as Integer {
        let wp: *mut Window =
            get_win_by_grid_handle(grid as Handle).map_or(ptr::null_mut(), Win::raw);
        if !(!wp.is_null() && unsafe { (*wp).w_grid_alloc.is_allocated() }) {
            let name = c"grid handle".as_ptr();
            // SAFETY: `error` is this frame's own slot and `name` a literal.
            error = err_bad_number(unsafe { cstr::at(name) }, grid);
            return ret.reported(error);
        }
        g = unsafe { GridRef::new(&raw mut (*wp).w_grid_alloc) };
    }
    if row < 0 as Integer
        || row >= g.rows as Integer
        || col < 0 as Integer
        || col >= g.cols as Integer
    {
        return ret.reported(error);
    }
    ret = Array::with_capacity(3 as size_t);
    let off: size_t = g.cell_offset(row as ::core::ffi::c_int, col as ::core::ffi::c_int);
    let sc_buf: *mut ::core::ffi::c_char =
        unsafe { arena_alloc(arena, MAX_SCHAR_SIZE as size_t, false) } as *mut ::core::ffi::c_char;
    unsafe { schar_get(sc_buf, g.char_at(off)) };
    // SAFETY: `sc_buf` is the NUL-terminated cell buffer filled above.
    ret.push(Object::string(unsafe { cstr_to_string(sc_buf) }));
    let attr: ::core::ffi::c_int = g.attr_at(off) as ::core::ffi::c_int;
    // SAFETY: `arena` is this frame's own.
    let hl = Object::dict(unsafe { hl_get_attr_by_id(attr as Integer, true) }?);
    // SAFETY: `ret` has room for the three items the arena sized it for.
    ret.push(hl);
    if !unsafe { highlight_use_hlstate() } {
        // SAFETY: `attr` is a resolved attribute id.
        ret.push(Object::array(unsafe { hl_inspect(attr) }));
    }
    ret.reported(error)
}

// `nvim__screenshot` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__screenshot(path: String_0) {
    ui_call_screenshot(path);
}

// `nvim__invalidate_glyph_cache` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__invalidate_glyph_cache() {
    unsafe { schar_cache_clear() };
    must_redraw.set(UPD_CLEAR);
}

/// # Safety
///
/// `str` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `arena` must point at a live arena, which the memory this
/// answers with is taken from and must outlive.
// `nvim__unpack` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub unsafe fn nvim__unpack(str: String_0) -> Result<Object, Error> {
    // SAFETY: the caller's string names its own bytes.
    unsafe { unpack(str.data(), str.len()) }
}
