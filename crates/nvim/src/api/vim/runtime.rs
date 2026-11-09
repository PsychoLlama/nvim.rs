//! The runtime path: finding files under it and running Lua.
//!
//! `nvim_get_runtime_file` walks `'runtimepath'` for a pattern through
//! `find_runtime_cb`, and `nvim__get_runtime` is the internal spelling that
//! also takes the ordering flags.  `nvim_exec_lua` is here rather than with
//! the Lua bridge because it is the runtime's entry point from the API.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::helpers::api_try;
use crate::api::private::validate::err_bad_value;
use crate::narrow::len_as_integer;
use core::ffi::CStr;

/// Run `code` as a Lua chunk with `args` as its `...`.
pub fn nvim_exec_lua(code: ApiString, args: Array) -> Result<Object, Error> {
    nlua_exec(code.as_bytes(), None, args, RET_OBJECT)
}

/// [`nvim_exec_lua`], marked fast for the RPC dispatcher.
// `nvim__exec_lua_fast` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__exec_lua_fast(code: ApiString, args: Array) -> Result<Object, Error> {
    nvim_exec_lua(code, args)
}

pub fn nvim_strwidth(text: ApiString) -> Result<Integer, Error> {
    if text.len() > ::core::ffi::c_int::MAX as size_t {
        return Err(too_long(c"text length"));
    }
    Ok(len_as_integer(c_string_cells(text.as_bytes())))
}

/// Every directory of 'runtimepath'.
pub fn nvim_list_runtime_paths() -> Result<Array, Error> {
    nvim_get_runtime_file(ApiString::NULL, true)
}

// `nvim__runtime_inspect` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__runtime_inspect() -> Array {
    runtime_inspect()
}

/// The files matching `name` under 'runtimepath': the first, or with `all`
/// every one.
pub fn nvim_get_runtime_file(name: ApiString, all: Boolean) -> Result<Array, Error> {
    let mut rv = Vec::new();
    let flags = RuntimeOpts::DIRFILE | RuntimeOpts::ALL.when(all);
    api_try(|| {
        // The name is this call's own, which no script the walk runs can
        // reach.
        do_in_runtimepath_visit(name.as_cstr(), flags, &mut |fnames, all| {
            for fname in fnames {
                rv.push(Object::string(ApiString::from_cstr(fname)));
                if !all {
                    return true;
                }
            }
            !fnames.is_empty()
        });
    })?;
    Ok(rv.into())
}

// `nvim__get_lib_dir` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__get_lib_dir() -> ApiString {
    get_lib_dir().map_or(ApiString::NULL, |dir| ApiString::from_cstr(dir.as_cstr()))
}

// `nvim__get_runtime` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__get_runtime(
    pat: Array,
    all: Boolean,
    opts: &mut KeyDict_runtime,
) -> Result<Array, Error> {
    let should_source = opts.do_source.unwrap_or(false);
    let is_lua = opts.is_lua.unwrap_or(false);
    let deferred_safe = nlua_is_deferred_safe();
    if should_source && !deferred_safe {
        return Err(Error::validation(c"'do_source' used in fast callback"));
    }
    let res: Array = runtime_get_named(is_lua, &pat, all);
    if should_source {
        for i in 0..res.len() {
            let name = &res[i]
                .as_string()
                .expect("`runtime_get_named` answers an array of Strings");
            // Sourcing a file frees nothing the array holds, and a file
            // that went away since it was found is skipped, as upstream.
            let _ = do_source(name.as_cstr());
        }
    }
    Ok(res)
}

/// "Invalid `name`: '(too long)'", the one message this file shares.
fn too_long(name: &CStr) -> Error {
    err_bad_value(name, c"(too long)")
}

pub fn nvim_set_current_dir(dir: ApiString) -> Result<(), Error> {
    if dir.len() >= 4096 as size_t {
        return Err(too_long(c"directory name"));
    }
    // Cut at an interior NUL, as upstream's `char *` copy is.
    let dir = crate::cstr::owned(dir.as_cstr().to_bytes());
    api_try(|| {
        let _ = change_dir(&dir, kCdScopeGlobal);
    })
}
