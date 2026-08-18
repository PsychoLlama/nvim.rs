//! The runtime path: finding files under it and running Lua.
//!
//! `nvim_get_runtime_file` walks `'runtimepath'` for a pattern through
//! `find_runtime_cb`, and `nvim__get_runtime` is the internal spelling that
//! also takes the ordering flags.  `nvim_exec_lua` is here rather than with
//! the Lua bridge because it is the runtime's entry point from the API.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::kvec::InitVec;

pub unsafe extern "C" fn nvim_exec_lua(
    mut code: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        return nlua_exec(
            code,
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetObject,
            arena,
            err,
        );
    }
}

pub unsafe extern "C" fn nvim__exec_lua_fast(
    mut code: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        return nvim_exec_lua(code, args, arena, err);
    }
}

pub unsafe extern "C" fn nvim_strwidth(mut text: String_0, mut err: *mut Error) -> Integer {
    unsafe {
        if !(text.size <= 2147483647 as ::core::ffi::c_int as size_t) {
            api_err_invalid(
                err,
                c"text length".as_ptr(),
                c"(too long)".as_ptr(),
                0 as int64_t,
                true,
            );
            return 0 as Integer;
        }
        return mb_string2cells(text.data) as Integer;
    }
}

pub unsafe extern "C" fn nvim_list_runtime_paths(
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
        return nvim_get_runtime_file(NULL_STRING, true, arena, err);
    }
}

pub unsafe extern "C" fn nvim__runtime_inspect(mut arena: *mut Arena) -> Array {
    unsafe {
        return runtime_inspect(arena);
    }
}

pub unsafe extern "C" fn nvim_get_runtime_file(
    mut name: String_0,
    mut all: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
        let mut cookie: RuntimeCookie = RuntimeCookie {
            rv: ArrayBuilder {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
                init_array: [Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                }; 16],
            },
            arena: arena,
        };
        cookie.rv.capacity = ::core::mem::size_of::<[Object; 16]>()
            .wrapping_div(::core::mem::size_of::<Object>())
            .wrapping_div(
                (::core::mem::size_of::<[Object; 16]>()
                    .wrapping_rem(::core::mem::size_of::<Object>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as size_t;
        cookie.rv.size = 0 as size_t;
        cookie.rv.items = &raw mut cookie.rv.init_array as *mut Object;
        let mut flags: ::core::ffi::c_int = DIP_DIRFILE as ::core::ffi::c_int
            | (if all as ::core::ffi::c_int != 0 {
                DIP_ALL as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            });
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        do_in_runtimepath(
            (if name.size != 0 {
                name.data as *const ::core::ffi::c_char
            } else {
                c"".as_ptr()
            }) as *mut ::core::ffi::c_char,
            flags,
            Some(
                find_runtime_cb
                    as unsafe fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            &raw mut cookie as *mut ::core::ffi::c_void,
        );
        try_leave(&raw mut tstate, err);
        return arena_take_arraybuilder(arena, &raw mut cookie.rv);
    }
}

unsafe fn find_runtime_cb(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut c: *mut ::core::ffi::c_void,
) -> bool {
    unsafe {
        let mut cookie: *mut RuntimeCookie = c as *mut RuntimeCookie;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_fnames {
            // `kv_push`, whose growth step c2rust expanded inline.
            InitVec::new(
                &mut (*cookie).rv.size,
                &mut (*cookie).rv.capacity,
                &mut (*cookie).rv.items,
                &mut (*cookie).rv.init_array,
            )
            .push(Object::string(arena_string(
                (*cookie).arena,
                cstr_as_string(*fnames.offset(i as isize)),
            )));
            if !all {
                return true;
            }
            i += 1;
        }
        return num_fnames > 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn nvim__get_lib_dir() -> String_0 {
    unsafe {
        return cstr_as_string(get_lib_dir());
    }
}

pub unsafe extern "C" fn nvim__get_runtime(
    mut pat: Array,
    mut all: Boolean,
    mut opts: *mut KeyDict_runtime,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
        if !(!(*opts).do_source || nlua_is_deferred_safe() as ::core::ffi::c_int != 0) {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"'do_source' used in fast callback".as_ptr(),
            );
        }
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
        let mut res: Array = runtime_get_named((*opts).is_lua, pat, all, arena);
        if (*opts).do_source {
            let mut i: size_t = 0 as size_t;
            while i < res.size {
                let mut name: String_0 = (*res.items.add(i)).data.string;
                do_source(
                    name.data,
                    false,
                    DOSO_NONE as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                );
                i = i.wrapping_add(1);
            }
        }
        return res;
    }
}

pub unsafe extern "C" fn nvim_set_current_dir(mut dir: String_0, mut err: *mut Error) {
    unsafe {
        if !(dir.size < 4096 as size_t) {
            api_err_invalid(
                err,
                c"directory name".as_ptr(),
                c"(too long)".as_ptr(),
                0 as int64_t,
                true,
            );
            return;
        }
        let mut string: [::core::ffi::c_char; 4096] = [0; 4096];
        memcpy(
            &raw mut string as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            dir.data as *const ::core::ffi::c_void,
            dir.size,
        );
        string[dir.size] = NUL as ::core::ffi::c_char;
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        changedir_func(&raw mut string as *mut ::core::ffi::c_char, kCdScopeGlobal);
        try_leave(&raw mut tstate, err);
    }
}
