//! The runtime path: finding files under it and running Lua.
//!
//! `nvim_get_runtime_file` walks `'runtimepath'` for a pattern through
//! `find_runtime_cb`, and `nvim__get_runtime` is the internal spelling that
//! also takes the ordering flags.  `nvim_exec_lua` is here rather than with
//! the Lua bridge because it is the runtime's entry point from the API.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::{Reported, api_try};
use crate::api::private::validate::err_bad_value;
use crate::types::NUL;
use crate::winlayer::Live;
use core::ffi::CStr;

/// Run `code` as a Lua chunk with `args` as its `...`.
pub fn nvim_exec_lua(code: String_0, args: Array) -> Result<Object, Error> {
    let name = ::core::ptr::null::<::core::ffi::c_char>();
    // SAFETY: `code` and `args` are this call's own, and there is no chunk
    // name.
    unsafe { nlua_exec(&code, name, args, kRetObject) }
}

/// [`nvim_exec_lua`], marked fast for the RPC dispatcher.
// `nvim__exec_lua_fast` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__exec_lua_fast(code: String_0, args: Array) -> Result<Object, Error> {
    nvim_exec_lua(code, args)
}

pub fn nvim_strwidth(text: String_0) -> Result<Integer, Error> {
    if text.len() > ::core::ffi::c_int::MAX as size_t {
        return Err(too_long(c"text length"));
    }
    Ok(unsafe { mb_string2cells(text.data()) } as Integer)
}

/// Every directory of 'runtimepath'.
pub fn nvim_list_runtime_paths() -> Result<Array, Error> {
    nvim_get_runtime_file(String_0::NULL, true)
}

// `nvim__runtime_inspect` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__runtime_inspect() -> Array {
    runtime_inspect()
}

/// The files matching `name` under 'runtimepath': the first, or with `all`
/// every one.
pub fn nvim_get_runtime_file(name: String_0, all: Boolean) -> Result<Array, Error> {
    let mut cookie = RuntimeCookie { rv: Array::EMPTY };
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
    api_try(|| {
        let cookie = (&raw mut cookie).cast::<::core::ffi::c_void>();
        // SAFETY: `pat` is NUL-terminated and `cookie` is this frame's own,
        // live for the whole walk.
        let _ = unsafe { do_in_runtimepath(pat, flags, found, cookie) };
    })?;
    Ok(cookie.rv)
}

/// # Safety
///
/// `fnames` must point at `num_fnames` NUL-terminated names, and `c` at the
/// `CollectCookie` this callback was registered with -- `do_in_runtimepath`
/// passes back exactly what it was handed.
unsafe fn find_runtime_cb(
    num_fnames: ::core::ffi::c_int,
    fnames: *mut *mut ::core::ffi::c_char,
    all: bool,
    c: *mut ::core::ffi::c_void,
) -> bool {
    let cookie: *mut RuntimeCookie = c as *mut RuntimeCookie;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        // SAFETY: `fnames` names `num_fnames` C strings, and `cookie` is the
        // `RuntimeCookie` this walk was started with.
        let name = unsafe {
            let found = cstr_to_string(*fnames.offset(i as isize));
            Object::string(found.clone())
        };
        // SAFETY: as above. The borrow ends with the push.
        let rv = unsafe { &mut (*cookie).rv };
        // `kv_push`, whose growth step c2rust expanded inline.
        rv.push(name);
        if !all {
            return true;
        }
        i += 1;
    }
    num_fnames > 0 as ::core::ffi::c_int
}

// `nvim__get_lib_dir` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__get_lib_dir() -> String_0 {
    unsafe { cstr_to_string(get_lib_dir()) }
}

// `nvim__get_runtime` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__get_runtime(
    pat: Array,
    all: Boolean,
    opts: &mut KeyDict_runtime,
) -> Result<Array, Error> {
    let mut error = Error::none();
    // SAFETY: the caller's keyset, live for the whole call.
    let opts = unsafe { Live::new(opts) };
    let should_source = opts.do_source.unwrap_or(false);
    let is_lua = opts.is_lua.unwrap_or(false);
    let deferred_safe = nlua_is_deferred_safe();
    if should_source && !deferred_safe {
        error = Error::validation(c"'do_source' used in fast callback");
        return Array::EMPTY.reported(error);
    }
    let res: Array = runtime_get_named(is_lua, &pat, all);
    if should_source {
        for i in 0..res.len() {
            let name = &res[i]
                .as_string()
                .expect("`runtime_get_named` answers an array of Strings");
            let none = DOSO_NONE as ::core::ffi::c_int;
            // SAFETY: sourcing a file frees nothing the array holds.
            unsafe { do_source(name.data(), false, none, ::core::ptr::null_mut()) };
        }
    }
    res.reported(error)
}

/// "Invalid `name`: '(too long)'", the one message this file shares.
fn too_long(name: &CStr) -> Error {
    err_bad_value(name, c"(too long)")
}

pub fn nvim_set_current_dir(dir: String_0) -> Result<(), Error> {
    let error = Error::none();
    if dir.len() >= 4096 as size_t {
        return Err(too_long(c"directory name"));
    }
    let mut string: [::core::ffi::c_char; 4096] = [0; 4096];
    unsafe {
        (&raw mut string as *mut ::core::ffi::c_char)
            .cast::<u8>()
            .copy_from_nonoverlapping(dir.data().cast(), dir.len())
    };
    string[dir.len()] = NUL as ::core::ffi::c_char;
    api_try(|| {
        let dir = (&raw mut string).cast::<::core::ffi::c_char>();
        // SAFETY: `dir` is this frame's own NUL-terminated copy.
        unsafe { changedir_func(dir, kCdScopeGlobal) };
    })?;
    ().reported(error)
}
