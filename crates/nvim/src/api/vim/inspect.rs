//! The internal `nvim__*` inspection surface.
//!
//! None of this is published in the api metadata: the four `nvim__id*`
//! functions exist to exercise the msgpack round trip, `nvim__stats` and
//! `nvim__inspect_cell` report internals the test suite asserts on, and
//! `nvim_get_proc`/`nvim_get_proc_children` are the process-tree queries
//! the job-control tests use.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{NIL, Reported, array_add, dict_put};
use crate::api::private::validate::err_bad_number;
use crate::api_error;
use crate::cstr;
use crate::grid::default_grid_ref;
use crate::log::logmsg;
use crate::popupmenu::pum_grid_ref;

/// `NULL` where a message names no offending string value.

pub unsafe fn nvim__id(obj: Object, arena: *mut Arena) -> Object {
    unsafe { copy_object(obj, arena) }
}

pub unsafe fn nvim__id_array(arr: Array, arena: *mut Arena) -> Array {
    unsafe { copy_array(arr, arena) }
}

pub unsafe fn nvim__id_dict(dct: Dict, arena: *mut Arena) -> Dict {
    unsafe { copy_dict(dct, arena) }
}

pub unsafe fn nvim__id_float(flt: Float) -> Float {
    flt
}

/// The counters the test suite asserts on: syncs, skipped log lines, live
/// Lua references, redraws and arena allocations.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim__stats(arena: *mut Arena) -> Dict {
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
    let mut rv: Dict = arena_dict(arena, entries.len());
    for (key, value) in entries {
        // SAFETY: `rv` is the dict the arena just sized for these six keys.
        unsafe { dict_put(&mut rv, key, value) };
    }
    rv
}

pub unsafe fn nvim_get_proc_children(pid: Integer, arena: *mut Arena) -> Result<Array, Error> {
    let mut error = Error::none();
    let mut rv: ::core::ffi::c_int = 0;
    let mut rvobj: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
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
            let mut a: Array = Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
            let mut a__items: [Object; 1] = [NIL; 1];
            a.capacity = 1 as size_t;
            a.items = &raw mut a__items as *mut Object;
            unsafe { array_add(&mut a, Object::integer(pid)) };
            let code = String_0::from_cstr(c"return vim._os_proc_children(...)");
            let name = ::core::ptr::null::<::core::ffi::c_char>();
            // SAFETY: `a` is the one-slot block above, `arena` is the
            // caller's and `error` this frame's own slot.
            let o = unsafe { nlua_exec(code, name, a, kRetObject, arena, &mut error) };
            if o.type_0 as ::core::ffi::c_uint
                == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                rvobj = unsafe { o.data.array };
            } else if !(error.kind() as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
            {
                error = api_error!(
                    kErrorTypeException,
                    "Failed to get process children. pid={pid} error={rv}"
                );
            }
        } else {
            rvobj = arena_array(arena, children.len() as size_t);
            for pid in children {
                unsafe { array_add(&mut rvobj, Object::integer(pid as Integer)) };
            }
        }
    }
    rvobj.reported(error)
}

pub unsafe fn nvim_get_proc(pid: Integer, arena: *mut Arena) -> Result<Object, Error> {
    let mut error = Error::none();
    let mut rvobj: Object = NIL;
    if !(pid > 0 as Integer && pid <= 2147483647 as Integer) {
        let name = c"pid".as_ptr();
        // SAFETY: `error` is this frame's own slot and `name` a literal.
        error = err_bad_number(unsafe { cstr::at(name) }, pid);
        return NIL.reported(error);
    }
    let mut a: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut a__items: [Object; 1] = [NIL; 1];
    a.capacity = 1 as size_t;
    a.items = &raw mut a__items as *mut Object;
    if a.size == a.capacity {
        a.capacity = if a.capacity != 0 {
            a.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        let (items, bytes) = (
            a.items.cast::<::core::ffi::c_void>(),
            ::core::mem::size_of::<Object>().wrapping_mul(a.capacity),
        );
        // SAFETY: `a.items` is this frame's own one-slot array, which
        // `xrealloc` copies out of and does not free.
        a.items = unsafe { xrealloc(items, bytes) }.cast::<Object>();
    };
    unsafe { array_add(&mut a, Object::integer(pid)) };
    let code = String_0::from_cstr(c"return vim._os_proc_info(...)");
    let name = ::core::ptr::null::<::core::ffi::c_char>();
    // SAFETY: `a` is the one-slot block above, `arena` is the caller's and
    // `error` this frame's own slot.
    let o = unsafe { nlua_exec(code, name, a, kRetObject, arena, &mut error) };
    if o.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        && unsafe { o.data.array }.size == 0 as size_t
    {
        return NIL.reported(error);
    } else if o.type_0 as ::core::ffi::c_uint
        == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        rvobj = o;
    } else if !(error.kind() as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        error = api_error!(kErrorTypeException, "Failed to get process info. pid={pid}");
    }
    rvobj.reported(error)
}

pub unsafe fn nvim__inspect_cell(
    grid: Integer,
    row: Integer,
    col: Integer,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = Error::none();
    let mut ret: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut g: GridRef = default_grid_ref();
    if grid == pum_grid_ref().handle as Integer {
        g = pum_grid_ref();
    } else if grid > 1 as Integer {
        let mut wp: *mut win_T = unsafe { get_win_by_grid_handle(grid as handle_T) };
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
    ret = arena_array(arena, 3 as size_t);
    let off: size_t = g.cell_offset(row as ::core::ffi::c_int, col as ::core::ffi::c_int);
    let mut sc_buf: *mut ::core::ffi::c_char =
        unsafe { arena_alloc(arena, MAX_SCHAR_SIZE as size_t, false) } as *mut ::core::ffi::c_char;
    unsafe { schar_get(sc_buf, g.char_at(off)) };
    unsafe { array_add(&mut ret, Object::string(cstr_as_string(sc_buf))) };
    let mut attr: ::core::ffi::c_int = g.attr_at(off) as ::core::ffi::c_int;
    // SAFETY: `arena` and `error` are this frame's own.
    let hl = unsafe { Object::dict(hl_get_attr_by_id(attr as Integer, true, arena, &mut error)) };
    // SAFETY: `ret` has room for the three items the arena sized it for.
    unsafe { array_add(&mut ret, hl) };
    if !unsafe { highlight_use_hlstate() } {
        unsafe { array_add(&mut ret, Object::array(hl_inspect(attr, arena))) };
    }
    ret.reported(error)
}

pub unsafe fn nvim__screenshot(path: String_0) {
    ui_call_screenshot(path);
}

pub unsafe fn nvim__invalidate_glyph_cache() {
    unsafe { schar_cache_clear() };
    must_redraw.set(UPD_CLEAR);
}

pub unsafe fn nvim__unpack(str: String_0, arena: *mut Arena) -> Result<Object, Error> {
    let mut error = Error::none();
    unsafe { unpack(str.data(), str.len(), arena, &mut error).reported(error) }
}
