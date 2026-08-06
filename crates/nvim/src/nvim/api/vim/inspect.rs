//! The internal `nvim__*` inspection surface.
//!
//! None of this is published in the api metadata: the four `nvim__id*`
//! functions exist to exercise the msgpack round trip, `nvim__stats` and
//! `nvim__inspect_cell` report internals the test suite asserts on, and
//! `nvim_get_proc`/`nvim_get_proc_children` are the process-tree queries
//! the job-control tests use.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::{array_add, dict_put};

pub unsafe extern "C" fn nvim__id(mut obj: Object, mut arena: *mut Arena) -> Object {
    unsafe {
        return copy_object(obj, arena);
    }
}

pub unsafe extern "C" fn nvim__id_array(mut arr: Array, mut arena: *mut Arena) -> Array {
    unsafe {
        return copy_array(arr, arena);
    }
}

pub unsafe extern "C" fn nvim__id_dict(mut dct: Dict, mut arena: *mut Arena) -> Dict {
    unsafe {
        return copy_dict(dct, arena);
    }
}

pub unsafe extern "C" fn nvim__id_float(mut flt: Float) -> Float {
    return flt;
}

pub unsafe extern "C" fn nvim__stats(mut arena: *mut Arena) -> Dict {
    unsafe {
        let mut rv: Dict = arena_dict(arena, 6 as size_t);
        dict_put(&mut rv, c"fsync", Object::integer((*g_stats.ptr()).fsync));
        dict_put(
            &mut rv,
            c"log_skip",
            Object::integer((*g_stats.ptr()).log_skip as Integer),
        );
        dict_put(
            &mut rv,
            c"lua_refcount",
            Object::integer(nlua_get_global_ref_count() as Integer),
        );
        dict_put(&mut rv, c"redraw", Object::integer((*g_stats.ptr()).redraw));
        dict_put(
            &mut rv,
            c"arena_alloc_count",
            Object::integer(arena_alloc_count.get() as Integer),
        );
        dict_put(
            &mut rv,
            c"ts_query_parse_count",
            Object::integer(tslua_query_parse_count.get() as Integer),
        );
        return rv;
    }
}

pub unsafe extern "C" fn nvim_get_proc_children(
    mut pid: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
        let mut rv: ::core::ffi::c_int = 0;
        let mut rvobj: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut children: Vec<::core::ffi::c_int> = Vec::new();
        if !(pid > 0 as Integer && pid <= 2147483647 as Integer) {
            api_err_invalid(
                err,
                c"pid".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                pid as int64_t,
                false_0 != 0,
            );
        } else {
            match os_proc_children(pid as ::core::ffi::c_int) {
                Some(pids) => children = pids,
                // Only "could not inspect" is reachable on this platform.
                None => rv = 2 as ::core::ffi::c_int,
            }
            if rv == 2 as ::core::ffi::c_int {
                logmsg(
                    LOGLVL_DBG,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    c"nvim_get_proc_children".as_ptr(),
                    1924 as ::core::ffi::c_int,
                    true_0 != 0,
                    c"fallback to vim._os_proc_children()".as_ptr(),
                );
                let mut a: Array = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                let mut a__items: [Object; 1] = [Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                }; 1];
                a.capacity = 1 as size_t;
                a.items = &raw mut a__items as *mut Object;
                array_add(&mut a, Object::integer(pid));
                let mut o: Object = nlua_exec(
                    String_0 {
                        data: c"return vim._os_proc_children(...)".as_ptr()
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 34]>()
                            .wrapping_sub(1 as size_t),
                    },
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    a,
                    kRetObject,
                    arena,
                    err,
                );
                if o.type_0 as ::core::ffi::c_uint
                    == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    rvobj = o.data.array;
                } else if !((*err).type_0 as ::core::ffi::c_int
                    != kErrorTypeNone as ::core::ffi::c_int)
                {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        c"Failed to get process children. pid=%ld error=%d".as_ptr(),
                        pid,
                        rv,
                    );
                }
            } else {
                rvobj = arena_array(arena, children.len() as size_t);
                for pid in children {
                    array_add(&mut rvobj, Object::integer(pid as Integer));
                }
            }
        }
        return rvobj;
    }
}

pub unsafe extern "C" fn nvim_get_proc(
    mut pid: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut rvobj: Object = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        if !(pid > 0 as Integer && pid <= 2147483647 as Integer) {
            api_err_invalid(
                err,
                c"pid".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                pid as int64_t,
                false_0 != 0,
            );
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        let mut a: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut a__items: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 1];
        a.capacity = 1 as size_t;
        a.items = &raw mut a__items as *mut Object;
        if a.size == a.capacity {
            a.capacity = if a.capacity != 0 {
                a.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            a.items = xrealloc(
                a.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul(a.capacity),
            ) as *mut Object;
        } else {
        };
        array_add(&mut a, Object::integer(pid));
        let mut o: Object = nlua_exec(
            String_0 {
                data: c"return vim._os_proc_info(...)".as_ptr() as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 30]>().wrapping_sub(1 as size_t),
            },
            ::core::ptr::null::<::core::ffi::c_char>(),
            a,
            kRetObject,
            arena,
            err,
        );
        if o.type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            && o.data.array.size == 0 as size_t
        {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        } else if o.type_0 as ::core::ffi::c_uint
            == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            rvobj = o;
        } else if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
            api_set_error(
                err,
                kErrorTypeException,
                c"Failed to get process info. pid=%ld".as_ptr(),
                pid,
            );
        }
        return rvobj;
    }
}

pub unsafe extern "C" fn nvim__inspect_cell(
    mut grid: Integer,
    mut row: Integer,
    mut col: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
        let mut ret: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut g: *mut ScreenGrid = default_grid.ptr();
        if grid == (*pum_grid.ptr()).handle as Integer {
            g = pum_grid.ptr();
        } else if grid > 1 as Integer {
            let mut wp: *mut win_T = get_win_by_grid_handle(grid as handle_T);
            if !(!wp.is_null() && !(*wp).w_grid_alloc.chars.is_null()) {
                api_err_invalid(
                    err,
                    c"grid handle".as_ptr(),
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    grid as int64_t,
                    false_0 != 0,
                );
                return ret;
            }
            g = &raw mut (*wp).w_grid_alloc;
        }
        if row < 0 as Integer
            || row >= (*g).rows as Integer
            || col < 0 as Integer
            || col >= (*g).cols as Integer
        {
            return ret;
        }
        ret = arena_array(arena, 3 as size_t);
        let mut off: size_t = (*(*g).line_offset.add(row as size_t)) + col as size_t;
        let mut sc_buf: *mut ::core::ffi::c_char =
            arena_alloc(arena, MAX_SCHAR_SIZE as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
        schar_get(sc_buf, *(*g).chars.add(off));
        array_add(&mut ret, Object::string(cstr_as_string(sc_buf)));
        let mut attr: ::core::ffi::c_int = *(*g).attrs.add(off) as ::core::ffi::c_int;
        array_add(
            &mut ret,
            Object::dict(hl_get_attr_by_id(attr as Integer, true, arena, err)),
        );
        if !highlight_use_hlstate() {
            array_add(&mut ret, Object::array(hl_inspect(attr, arena)));
        }
        return ret;
    }
}

pub unsafe extern "C" fn nvim__screenshot(mut path: String_0) {
    ui_call_screenshot(path);
}

pub unsafe extern "C" fn nvim__invalidate_glyph_cache() {
    unsafe {
        schar_cache_clear();
        must_redraw.set(UPD_CLEAR);
    }
}

pub unsafe extern "C" fn nvim__unpack(
    mut str: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        return unpack(str.data, str.size, arena, err);
    }
}
