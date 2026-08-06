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
        let c2rust_fresh20 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh20 as isize) = key_value_pair {
            key: cstr_as_string(b"fsync\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*g_stats.ptr()).fsync,
                },
            },
        };
        let c2rust_fresh21 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh21 as isize) = key_value_pair {
            key: cstr_as_string(b"log_skip\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*g_stats.ptr()).log_skip as Integer,
                },
            },
        };
        let c2rust_fresh22 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh22 as isize) = key_value_pair {
            key: cstr_as_string(b"lua_refcount\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: nlua_get_global_ref_count() as Integer,
                },
            },
        };
        let c2rust_fresh23 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh23 as isize) = key_value_pair {
            key: cstr_as_string(b"redraw\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*g_stats.ptr()).redraw,
                },
            },
        };
        let c2rust_fresh24 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh24 as isize) = key_value_pair {
            key: cstr_as_string(b"arena_alloc_count\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: arena_alloc_count.get() as Integer,
                },
            },
        };
        let c2rust_fresh25 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh25 as isize) = key_value_pair {
            key: cstr_as_string(b"ts_query_parse_count\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: tslua_query_parse_count.get() as Integer,
                },
            },
        };
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
                b"pid\0".as_ptr() as *const ::core::ffi::c_char,
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
                    b"nvim_get_proc_children\0".as_ptr() as *const ::core::ffi::c_char,
                    1924 as ::core::ffi::c_int,
                    true_0 != 0,
                    b"fallback to vim._os_proc_children()\0".as_ptr() as *const ::core::ffi::c_char,
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
                let c2rust_fresh26 = a.size;
                a.size = a.size.wrapping_add(1);
                *a.items.offset(c2rust_fresh26 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed { integer: pid },
                };
                let mut o: Object = nlua_exec(
                    String_0 {
                        data: b"return vim._os_proc_children(...)\0".as_ptr()
                            as *const ::core::ffi::c_char
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
                        b"Failed to get process children. pid=%ld error=%d\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        pid,
                        rv,
                    );
                }
            } else {
                rvobj = arena_array(arena, children.len() as size_t);
                for pid in children {
                    let c2rust_fresh27 = rvobj.size;
                    rvobj.size = rvobj.size.wrapping_add(1);
                    *rvobj.items.offset(c2rust_fresh27 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: pid as Integer,
                        },
                    };
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
                b"pid\0".as_ptr() as *const ::core::ffi::c_char,
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
        let c2rust_fresh28 = a.size;
        a.size = a.size.wrapping_add(1);
        *a.items.offset(c2rust_fresh28 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: pid },
        };
        let mut o: Object = nlua_exec(
            String_0 {
                data: b"return vim._os_proc_info(...)\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
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
                b"Failed to get process info. pid=%ld\0".as_ptr() as *const ::core::ffi::c_char,
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
                    b"grid handle\0".as_ptr() as *const ::core::ffi::c_char,
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
        let mut off: size_t =
            (*(*g).line_offset.offset(row as size_t as isize)).wrapping_add(col as size_t);
        let mut sc_buf: *mut ::core::ffi::c_char =
            arena_alloc(arena, MAX_SCHAR_SIZE as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
        schar_get(sc_buf, *(*g).chars.offset(off as isize));
        let c2rust_fresh29 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh29 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(sc_buf),
            },
        };
        let mut attr: ::core::ffi::c_int = *(*g).attrs.offset(off as isize) as ::core::ffi::c_int;
        let c2rust_fresh30 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh30 as isize) = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed {
                dict: hl_get_attr_by_id(attr as Integer, true, arena, err),
            },
        };
        if !highlight_use_hlstate() {
            let c2rust_fresh31 = ret.size;
            ret.size = ret.size.wrapping_add(1);
            *ret.items.offset(c2rust_fresh31 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed {
                    array: hl_inspect(attr, arena),
                },
            };
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
