//! Lua values that Vimscript can call, and asking whether it can.
//!
//! [`register_table_as_callable`] gives a table with a `__call` metamethod a
//! `LuaRef` so it can be stored as a Funcref, [`funcref_description`]
//! renders one back to the `<Lua N: file:line>` form a listing shows, and
//! [`nlua_func_exists`] answers `exists('v:lua.…')`.
//! [`nlua_execute_on_key`] is the `vim.on_key()` callback, called for every
//! key the editor consumes.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};
use std::ffi::CString;

use super::{kRetNilBool, nlua_error, nlua_exec, with_main_stack};
use crate::eval::userfunc::register_luafunc;
use crate::getchar::state::{got_int, mod_mask};
use crate::global_cell::GlobalCell;
use crate::keycodes::{special_key_bytes, unescape_ks_in};
use crate::lua::ffi::LUA_NOREF;
use crate::lua::stack::{At, Kind};
use crate::memory::ThinCString;
use crate::os::env::home_replace_in;
use crate::types::{Array, LuaRef, Object, String_0, TypVal, VAR_DICT, VAR_LIST};

/// The Lua table `arg` came from, if it came from one at all.
fn lua_table_ref(arg: &TypVal) -> LuaRef {
    match arg.v_type() {
        VAR_DICT => arg.dict_ref().map_or(LUA_NOREF, |dict| dict.lua_table_ref),
        VAR_LIST => arg.list_ref().map_or(LUA_NOREF, |list| list.lua_table_ref),
        _ => LUA_NOREF,
    }
}

/// Whether this dictionary or list is a *view* of a Lua table rather than a
/// Vimscript value of its own.
pub fn nlua_is_table_from_lua(arg: &TypVal) -> bool {
    lua_table_ref(arg) != LUA_NOREF
}

/// If `arg` is a Lua table with a `__call` metamethod, register that
/// metamethod as a Vimscript function and answer its name.
///
/// Every exit leaves the Lua stack exactly as it found it.
pub(crate) fn register_table_as_callable(arg: &TypVal) -> Option<ThinCString> {
    let table_ref = lua_table_ref(arg);
    if table_ref == LUA_NOREF {
        return None;
    }

    with_main_stack(|stack| {
        let top = stack.top();
        stack.push_reference(table_ref); // [table]
        if !stack.get_metatable(At::TOP) {
            stack.pop(1);
            debug_assert!(top == stack.top());
            return None;
        }
        // [table, mt]
        stack.get_field(At::TOP, c"__call"); // [table, mt, __call]
        if stack.kind(At::TOP) != Kind::Function {
            stack.pop(3);
            debug_assert!(top == stack.top());
            return None;
        }
        stack.pop(2); // [table]

        // The reference is on the *table*, not on `__call`: calling the table is
        // what runs the metamethod, with the table itself as the first argument.
        let func = stack.reference(At::TOP);
        let name = register_luafunc(func);
        stack.pop(1);
        debug_assert!(top == stack.top());
        Some(name)
    })
}

/// `vim.on_key()`: hand every consumed key to the Lua callbacks, and answer
/// whether they want it discarded.
///
/// Not re-entrant — a callback that itself consumes a key is skipped — and
/// it swallows any interrupt a callback raised, restoring whatever `got_int`
/// was before.
///
/// `typed` is the bytes that were typed for the key, still escaped; they are
/// unescaped in place, and read up to the first NUL as upstream's string
/// is. The main state exists whenever a key is being read.
pub fn nlua_execute_on_key(c: c_int, typed: &mut Vec<u8>) -> bool {
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);
    if RECURSIVE.get() {
        return false;
    }
    RECURSIVE.set(true);

    let key = special_key_bytes(c, mod_mask.get(), false);
    let up_to_nul = |bytes: &[u8]| bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    typed.truncate(up_to_nul(typed));
    let unescaped = unescape_ks_in(typed);
    typed.truncate(unescaped);
    let typed = &typed[..up_to_nul(typed)];

    with_main_stack(|stack| {
        let top = stack.top();
        stack.get_global(c"vim");
        // Upstream checks the type and raises outside any protected call, which
        // takes the editor down; a callback that is not one is reported by the
        // call below instead.
        stack.get_field(At::TOP, c"_on_key");
        stack.push_bytes(key.as_bytes());
        stack.push_bytes(typed);

        let save_got_int = got_int.get();
        got_int.set(false);
        let mut discard = false;
        if stack.pcall(2, 1, None).is_err() {
            nlua_error(stack, c"vim.on_key() callbacks: %.*s");
        } else {
            if stack.kind(At::TOP) == Kind::Boolean {
                discard = stack.to_boolean(At::TOP);
            }
            stack.pop(1);
        }
        got_int.set(got_int.get() || save_got_int);
        stack.pop(1);
        debug_assert!(top == stack.top());

        RECURSIVE.set(false);
        discard
    })
}

/// How a `LuaRef` renders in a listing: `<Lua N: file:line>` when the
/// reference is a function defined in a file, `<Lua N>` otherwise.
pub(crate) fn funcref_description(reference: LuaRef) -> Option<ThinCString> {
    with_main_stack(|stack| {
        if stack.reserve(1) {
            stack.push_reference(reference);
            if stack.kind(At::TOP) != Kind::Function {
                stack.pop(1);
            } else if let Some((source, line)) = stack.pop_function_source()
                && let Some(path) = source.strip_prefix(b"@")
                && line >= 0
            {
                // `>S` consumed the function; a source name is a C string.
                let path = CString::new(path).expect("a chunk name holds no NUL");
                let path = home_replace_in(None, &path, path.as_bytes().len() + 3, true);
                let text = [
                    format!("<Lua {reference}: ").as_bytes(),
                    &path[..],
                    format!(":{line}>").as_bytes(),
                ]
                .concat();
                return Some(ThinCString::from_bytes(&text));
            }
        }
        Some(ThinCString::from_bytes(
            format!("<Lua {reference}>").as_bytes(),
        ))
    })
}

/// `exists('v:lua.f')`: whether the Lua expression names a function.
pub fn nlua_func_exists(lua_funcname: &CStr) -> bool {
    // The name is evaluated as an expression rather than looked up, so
    // `v:lua.pkg.fn` works.
    let expr = [b"return ".as_slice(), lua_funcname.to_bytes()].concat();
    let args = Array::from(vec![Object::string(String_0::from_bytes(&expr))]);
    let result = nlua_exec(
        b"return type(loadstring(...)()) == 'function'",
        None,
        args,
        kRetNilBool,
    );
    result.is_ok_and(|value| value.as_boolean() == Some(true))
}
