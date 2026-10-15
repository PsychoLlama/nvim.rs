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
    let mut rv = Array::EMPTY;
    let flags = RuntimeOpts::DIRFILE | RuntimeOpts::ALL.when(all);
    api_try(|| {
        // The name is this call's own, which no script the walk runs can
        // reach.
        let _ = do_in_runtimepath_visit(name.as_cstr(), flags, &mut |fnames, all| {
            for fname in fnames {
                rv.push(Object::string(String_0::from_cstr(fname)));
                if !all {
                    return true;
                }
            }
            !fnames.is_empty()
        });
    })?;
    Ok(rv)
}

// `nvim__get_lib_dir` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__get_lib_dir() -> String_0 {
    get_lib_dir().map_or(String_0::NULL, |dir| String_0::from_cstr(dir.as_cstr()))
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
            // Sourcing a file frees nothing the array holds.
            do_source(name.as_cstr(), false, none, None);
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
    // SAFETY: `dir` names `len` readable bytes, copied here.
    let bytes = unsafe { core::slice::from_raw_parts(dir.data().cast::<u8>(), dir.len()) };
    // Cut at an interior NUL, as upstream's `char *` copy is.
    let bytes = &bytes[..bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())];
    let dir = crate::cstr::owned(bytes);
    api_try(|| {
        change_dir(&dir, kCdScopeGlobal);
    })?;
    ().reported(error)
}
