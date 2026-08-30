//! The runtime path: finding files under it and running Lua.
//!
//! `nvim_get_runtime_file` walks `'runtimepath'` for a pattern through
//! `find_runtime_cb`, and `nvim__get_runtime` is the internal spelling that
//! also takes the ordering flags.  `nvim_exec_lua` is here rather than with
//! the Lua bridge because it is the runtime's entry point from the API.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported, api_try};
use crate::api::private::validate::err_invalid_ptr;
use crate::kvec::InitVec;
use crate::types::NUL;
use crate::winlayer::Live;
use core::ffi::CStr;

pub unsafe fn nvim_exec_lua(
    code: String_0,
    args: Array,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let name = ::core::ptr::null::<::core::ffi::c_char>();
    // SAFETY: `code` and `args` are the caller's, `arena` is the caller's
    // own and `error` is this frame's slot.
    unsafe { nlua_exec(code, name, args, kRetObject, arena, &mut error) }.reported(error)
}

pub unsafe fn nvim__exec_lua_fast(
    code: String_0,
    args: Array,
    arena: *mut Arena,
) -> Result<Object, Error> {
    unsafe { nvim_exec_lua(code, args, arena) }
}

pub unsafe fn nvim_strwidth(text: String_0) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    if text.len() > ::core::ffi::c_int::MAX as size_t {
        too_long(&mut error, c"text length");
        return (0 as Integer).reported(error);
    }
    (unsafe { mb_string2cells(text.data()) } as Integer).reported(error)
}

pub unsafe fn nvim_list_runtime_paths(arena: *mut Arena) -> Result<Array, Error> {
    unsafe { nvim_get_runtime_file(String_0::NULL, true, arena) }
}

pub unsafe fn nvim__runtime_inspect(arena: *mut Arena) -> Array {
    unsafe { runtime_inspect(arena) }
}

pub unsafe fn nvim_get_runtime_file(
    name: String_0,
    all: Boolean,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    let mut cookie: RuntimeCookie = RuntimeCookie {
        rv: ArrayBuilder {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
            init_array: [NIL; 16],
        },
        arena,
    };
    cookie.rv.capacity = ::core::mem::size_of::<[Object; 16]>()
        .wrapping_div(::core::mem::size_of::<Object>())
        .wrapping_div(
            (::core::mem::size_of::<[Object; 16]>().wrapping_rem(::core::mem::size_of::<Object>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    cookie.rv.size = 0 as size_t;
    cookie.rv.items = &raw mut cookie.rv.init_array as *mut Object;
    let flags = RuntimeOpts::DIRFILE | RuntimeOpts::ALL.when(all);
    let pat = if name.is_empty() {
        c"".as_ptr().cast_mut()
    } else {
        name.data()
    };
    let found = Some(
        find_runtime_cb
            as unsafe fn(
                ::core::ffi::c_int,
                *mut *mut ::core::ffi::c_char,
                bool,
                *mut ::core::ffi::c_void,
            ) -> bool,
    );
    api_try(&mut error, |_| {
        let cookie = (&raw mut cookie).cast::<::core::ffi::c_void>();
        // SAFETY: `pat` is NUL-terminated and `cookie` is this frame's own,
        // live for the whole walk.
        unsafe { do_in_runtimepath(pat, flags, found, cookie) };
    });
    // SAFETY: `arena` is the caller's and `cookie.rv` this frame's own.
    unsafe { arena_take_arraybuilder(arena, &raw mut cookie.rv) }.reported(error)
}

unsafe fn find_runtime_cb(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut c: *mut ::core::ffi::c_void,
) -> bool {
    let mut cookie: *mut RuntimeCookie = c as *mut RuntimeCookie;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        // SAFETY: `fnames` names `num_fnames` C strings, and `cookie` is the
        // `RuntimeCookie` this walk was started with -- the copy the arena
        // takes is what outlives it.
        let name = unsafe {
            let found = cstr_as_string(*fnames.offset(i as isize));
            Object::string(arena_string((*cookie).arena, found))
        };
        // SAFETY: as above. The borrow ends with the push.
        let rv = unsafe { &mut (*cookie).rv };
        // `kv_push`, whose growth step c2rust expanded inline.
        InitVec::new(
            &mut rv.size,
            &mut rv.capacity,
            &mut rv.items,
            &mut rv.init_array,
        )
        .push(name);
        if !all {
            return true;
        }
        i += 1;
    }
    num_fnames > 0 as ::core::ffi::c_int
}

pub unsafe fn nvim__get_lib_dir() -> String_0 {
    unsafe { cstr_as_string(get_lib_dir()) }
}

pub unsafe fn nvim__get_runtime(
    pat: Array,
    all: Boolean,
    opts: *mut KeyDict_runtime,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    // SAFETY: the caller's keyset, live for the whole call.
    let opts = unsafe { Live::new(opts) };
    // SAFETY: the Lua state exists from startup to exit.
    let deferred_safe = unsafe { nlua_is_deferred_safe() };
    if opts.do_source && !deferred_safe {
        error = Error::validation(c"'do_source' used in fast callback");
        return Array::EMPTY.reported(error);
    }
    // SAFETY: `pat` is the caller's array and `arena` its own.
    let res: Array = unsafe { runtime_get_named(opts.is_lua, pat, all, arena) };
    if opts.do_source {
        for i in 0..res.size {
            // SAFETY: `res` is the array `runtime_get_named` just built, of
            // `size` Strings; sourcing one may free nothing it holds.
            unsafe {
                let name = (*res.items.add(i)).data.string;
                let none = DOSO_NONE as ::core::ffi::c_int;
                do_source(name.data(), false, none, ::core::ptr::null_mut());
            }
        }
    }
    res.reported(error)
}

/// "Invalid `name`: '(too long)'", the one message this file shares.
fn too_long(err: &mut Error, name: &CStr) {
    let too_long = c"(too long)".as_ptr();
    // SAFETY: `err` is the caller's own slot, and both strings are literals.
    *err = unsafe { err_invalid_ptr(name.as_ptr(), too_long, 0, true) };
}

pub unsafe fn nvim_set_current_dir(dir: String_0) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    if dir.len() >= 4096 as size_t {
        too_long(&mut error, c"directory name");
        return ().reported(error);
    }
    let mut string: [::core::ffi::c_char; 4096] = [0; 4096];
    unsafe {
        memcpy(
            &raw mut string as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            dir.data() as *const ::core::ffi::c_void,
            dir.len(),
        )
    };
    string[dir.len()] = NUL as ::core::ffi::c_char;
    api_try(&mut error, |_| {
        let dir = (&raw mut string).cast::<::core::ffi::c_char>();
        // SAFETY: `dir` is this frame's own NUL-terminated copy.
        unsafe { changedir_func(dir, kCdScopeGlobal) };
    });
    ().reported(error)
}
