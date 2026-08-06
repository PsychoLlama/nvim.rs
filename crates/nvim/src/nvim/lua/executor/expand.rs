//! Command-line completion over Lua names.
//!
//! `nlua_expand_pat` hands the pattern to `vim._expand_pat` and stashes the
//! results in [`expand_result_array`], which `nlua_expand_get_matches` then
//! drains -- the two-step shape exists because the caller wants the matches
//! after the Lua state has been unwound.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

static expand_result_array: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);

pub unsafe extern "C-unwind" fn nlua_expand_pat(mut xp: *mut expand_T) {
    unsafe {
        let mut completions: Array = Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let lstate: *mut lua_State = global_lstate.get();
        let mut status: ::core::ffi::c_int = FAIL;
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"vim\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_getfield(
            lstate,
            -1 as ::core::ffi::c_int,
            b"_expand_pat\0".as_ptr() as *const ::core::ffi::c_char,
        );
        luaL_checktype(lstate, -1 as ::core::ffi::c_int, LUA_TFUNCTION);
        let mut pat: *const ::core::ffi::c_char = (*xp).xp_pattern;
        '_c2rust_label: {
            if (*xp).xp_line.offset((*xp).xp_col as isize) >= pat as *mut ::core::ffi::c_char {
            } else {
                __assert_fail(
                    b"xp->xp_line + xp->xp_col >= pat\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1971 as ::core::ffi::c_uint,
                    b"void nlua_expand_pat(expand_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut patlen: ptrdiff_t = (*xp).xp_line.offset((*xp).xp_col as isize).offset_from(pat);
        lua_pushlstring(lstate, pat, patlen as size_t);
        if nlua_pcall(lstate, 1 as ::core::ffi::c_int, 2 as ::core::ffi::c_int)
            != 0 as ::core::ffi::c_int
        {
            nlua_error(
                lstate,
                gettext(b"vim._expand_pat: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            return;
        }
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut arena: Arena = ARENA_EMPTY;
        let mut prefix_len: ptrdiff_t =
            nlua_pop_Integer(lstate, &raw mut arena, &raw mut err) as ptrdiff_t;
        if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
            || prefix_len > patlen)
        {
            completions = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
            '_cleanup_array: {
                if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                    ga_clear(expand_result_array.ptr());
                    ga_init(
                        expand_result_array.ptr(),
                        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                        80 as ::core::ffi::c_int,
                    );
                    let mut i: size_t = 0 as size_t;
                    while i < completions.size {
                        let mut v: Object = *completions.items.offset(i as isize);
                        if v.type_0 as ::core::ffi::c_uint
                            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            break '_cleanup_array;
                        }
                        ga_grow(expand_result_array.ptr(), 1 as ::core::ffi::c_int);
                        *((*expand_result_array.ptr()).ga_data as *mut *mut ::core::ffi::c_char)
                            .offset((*expand_result_array.ptr()).ga_len as isize) =
                            string_to_cstr(v.data.string);
                        (*expand_result_array.ptr()).ga_len += 1;
                        i = i.wrapping_add(1);
                    }
                    (*xp).xp_pattern = (*xp).xp_pattern.offset(prefix_len as isize);
                    status = OK;
                }
            }
            arena_mem_free(arena_finish(&raw mut arena));
        }
        if status == FAIL {
            ga_clear(expand_result_array.ptr());
        }
    }
}

pub unsafe extern "C-unwind" fn nlua_expand_get_matches(
    mut num_results: *mut ::core::ffi::c_int,
    mut results: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        *results = (*expand_result_array.ptr()).ga_data as *mut *mut ::core::ffi::c_char;
        *num_results = (*expand_result_array.ptr()).ga_len;
        expand_result_array.set(GA_EMPTY_INIT_VALUE);
        return (*num_results > 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
}
