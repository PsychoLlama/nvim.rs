//! Lua values that Vimscript can call, and asking whether it can.
//!
//! [`nlua_register_table_as_callable`] gives a table with a `__call`
//! metamethod a `LuaRef` so it can be stored as a Funcref,
//! [`nlua_funcref_str`] renders one back to the `<Lua N: file:line>` form a
//! listing shows, and [`nlua_func_exists`] answers `exists('v:lua.…')`.
//! [`nlua_execute_on_key`] is the `vim.on_key()` callback, called for every
//! key the editor consumes.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use super::{get_global_lstate, kRetNilBool, lua_Debug, lua_getinfo, nlua_error, nlua_exec};
use crate::eval::userfunc::register_luafunc;
use crate::getchar::state::{got_int, mod_mask};
use crate::global_cell::GlobalCell;
use crate::keycodes::{special_key_bytes, unescape_ks_in};
use crate::lua::executor::{nlua_pushref, nlua_ref_global};
use crate::lua::ffi::{
    LUA_NOREF, LUA_TBOOLEAN, LUA_TFUNCTION, lua_checkstack, lua_getfield, lua_getglobal,
    lua_getmetatable, lua_gettop, lua_pcall, lua_pop, lua_pushlstring, lua_toboolean, lua_type,
    luaL_checktype,
};
use crate::memory::xfree;
use crate::os::cshim::gettext;
use crate::os::env::home_replace_save;

use crate::types::{Array, LuaRef, Object, String_0, TypVal, VAR_DICT, VAR_LIST};

/// An all-zero [`lua_Debug`], which `lua_getinfo` fills.
const LUA_DEBUG_INIT: lua_Debug = lua_Debug {
    event: 0,
    name: ptr::null(),
    namewhat: ptr::null(),
    what: ptr::null(),
    source: ptr::null(),
    currentline: 0,
    nups: 0,
    linedefined: 0,
    lastlinedefined: 0,
    short_src: [0; 60],
    i_ci: 0,
};

/// The Lua table `arg` came from, if it came from one at all.
fn lua_table_ref(arg: &TypVal) -> LuaRef {
    unsafe {
        match (*arg).v_type() {
            VAR_DICT => (*(*arg).dict_or_null()).lua_table_ref,
            VAR_LIST => (*(*arg).list_or_null()).lua_table_ref,
            _ => LUA_NOREF,
        }
    }
}

/// Whether this dictionary or list is a *view* of a Lua table rather than a
/// Vimscript value of its own.
pub fn nlua_is_table_from_lua(arg: &TypVal) -> bool {
    lua_table_ref(arg) != LUA_NOREF
}

/// If `arg` is a Lua table with a `__call` metamethod, register that
/// metamethod as a Vimscript function and answer its name, allocated for
/// the caller to free; otherwise null.
///
/// Every exit leaves the Lua stack exactly as it found it.
///
/// # Safety
/// `arg` must be a live typval and the main state must exist.
pub unsafe fn nlua_register_table_as_callable(arg: &TypVal) -> *mut c_char {
    unsafe {
        let table_ref = lua_table_ref(arg);
        if table_ref == LUA_NOREF {
            return ptr::null_mut();
        }

        let lstate = get_global_lstate();
        let top = lua_gettop(lstate);
        nlua_pushref(lstate, table_ref); // [table]
        if lua_getmetatable(lstate, -1) == 0 {
            lua_pop(lstate, 1);
            debug_assert!(top == lua_gettop(lstate));
            return ptr::null_mut();
        }
        // [table, mt]
        lua_getfield(lstate, -1, c"__call".as_ptr()); // [table, mt, __call]
        if lua_type(lstate, -1) != LUA_TFUNCTION {
            lua_pop(lstate, 3);
            debug_assert!(top == lua_gettop(lstate));
            return ptr::null_mut();
        }
        lua_pop(lstate, 2); // [table]

        // The reference is on the *table*, not on `__call`: calling the
        // table is what runs the metamethod, with the table itself as the
        // first argument.
        let func = nlua_ref_global(lstate, -1);
        let name = register_luafunc(func);
        lua_pop(lstate, 1);
        debug_assert!(top == lua_gettop(lstate));
        name.into_raw()
    }
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
    let (buf, buf_len) = (key.as_bytes(), key.as_bytes().len());
    let up_to_nul = |bytes: &[u8]| bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    typed.truncate(up_to_nul(typed));
    let unescaped = unescape_ks_in(typed);
    typed.truncate(unescaped);
    let typed = &typed[..up_to_nul(typed)];
    // SAFETY: the main state, which exists while keys are read; both strings
    // are pushed as copies before anything runs.
    let discard = unsafe {
        let lstate = get_global_lstate();
        let top = lua_gettop(lstate);

        lua_getglobal(lstate, c"vim".as_ptr());
        lua_getfield(lstate, -1, c"_on_key".as_ptr());
        luaL_checktype(lstate, -1, LUA_TFUNCTION);
        lua_pushlstring(lstate, buf.as_ptr().cast(), buf_len);
        lua_pushlstring(lstate, typed.as_ptr().cast(), typed.len());

        let save_got_int = got_int.get();
        got_int.set(false);
        let mut discard = false;
        if lua_pcall(lstate, 2, 1, 0) != 0 {
            nlua_error(lstate, gettext(c"vim.on_key() callbacks: %.*s").as_ptr());
        } else {
            if lua_type(lstate, -1) == LUA_TBOOLEAN {
                discard = lua_toboolean(lstate, -1) != 0;
            }
            lua_pop(lstate, 1);
        }
        got_int.set(got_int.get() || save_got_int);
        lua_pop(lstate, 1);
        debug_assert!(top == lua_gettop(lstate));
        discard
    };
    RECURSIVE.set(false);
    discard
}

/// [`nlua_register_table_as_callable`], answering the name owned.
pub(crate) fn register_table_as_callable(arg: &TypVal) -> Option<crate::memory::ThinCString> {
    // SAFETY: a live value, and the main state, which lives as long as the
    // editor; the name is an allocation of its own, adopted here.
    unsafe { crate::memory::ThinCString::from_raw(nlua_register_table_as_callable(arg)) }
}

/// [`nlua_funcref_str`], as an owned string.
pub(crate) fn funcref_description(reference: LuaRef) -> Option<crate::memory::ThinCString> {
    // SAFETY: the main state lives as long as the editor; the rendering is a
    // block of its own, which the answer takes over.
    unsafe { crate::memory::ThinCString::from_raw(nlua_funcref_str(reference)) }
}

/// How a `LuaRef` renders in a listing: `<Lua N: file:line>` when the
/// reference is a function defined in a file, `<Lua N>` otherwise.
///
/// The answer is a block of its own, which the caller frees.
///
/// # Safety
/// The main state must exist.
pub unsafe fn nlua_funcref_str(ref_0: LuaRef) -> *mut c_char {
    unsafe {
        let lstate = get_global_lstate();
        if lua_checkstack(lstate, 1) != 0 {
            nlua_pushref(lstate, ref_0);
            if lua_type(lstate, -1) != LUA_TFUNCTION {
                lua_pop(lstate, 1);
            } else {
                // `>S` consumes the function from the stack and fills in the
                // source it was defined in.
                let mut ar = LUA_DEBUG_INIT;
                if lua_getinfo(lstate, c">S".as_ptr(), &raw mut ar) != 0
                    && *ar.source == b'@'.cast_signed()
                    && ar.linedefined >= 0
                {
                    let src = home_replace_save(None, ar.source.add(1));
                    let str: String_0 = crate::printf_string!(
                        c"<Lua %d: %s:%d>".as_ptr(),
                        ref_0,
                        src,
                        ar.linedefined
                    );
                    xfree(src.cast::<c_void>());
                    // The caller frees the rendering, so the string gives
                    // its block up rather than releasing it here.
                    return str.into_raw();
                }
            }
        }
        crate::printf_string!(c"<Lua %d>".as_ptr(), ref_0).into_raw()
    }
}

/// `exists('v:lua.f')`: whether the Lua expression names a function.
pub fn nlua_func_exists(lua_funcname: &CStr) -> bool {
    // The name is evaluated as an expression rather than looked up, so
    // `v:lua.pkg.fn` works.
    let expr = [b"return ".as_slice(), lua_funcname.to_bytes()].concat();
    let args = Array::from(vec![Object::string(String_0::from_bytes(&expr))]);
    // SAFETY: a null `chunkname`, which the contract allows.
    let result = unsafe {
        nlua_exec(
            &String_0::from_cstr(c"return type(loadstring(...)()) == 'function'"),
            ptr::null::<c_char>(),
            args,
            kRetNilBool,
        )
    };
    result.is_ok_and(|value| value.as_boolean() == Some(true))
}
