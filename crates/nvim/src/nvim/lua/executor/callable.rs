//! Lua values that Vimscript can call, and asking whether it can.
//!
//! [`nlua_register_table_as_callable`] gives a table with a `__call`
//! metamethod a `LuaRef` so it can be stored as a Funcref,
//! [`nlua_funcref_str`] renders one back to the `<Lua N: file:line>` form a
//! listing shows, and [`nlua_func_exists`] answers `exists('v:lua.…')`.
//! [`nlua_execute_on_key`] is the `vim.on_key()` callback, called for every
//! key the editor consumes.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::{get_global_lstate, kRetNilBool, lua_Debug, lua_getinfo, nlua_error, nlua_exec};
use crate::src::nvim::api::private::helpers::{api_clear_error, cstr_as_string};
use crate::src::nvim::eval::userfunc::register_luafunc;
use crate::src::nvim::ex_getln::ERROR_INIT;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::keycodes::{special_to_buf, vim_unescape_ks};
use crate::src::nvim::lua::executor::{nlua_pushref, nlua_ref_global};
use crate::src::nvim::lua::ffi::{
    LUA_NOREF, LUA_TBOOLEAN, LUA_TFUNCTION, lua_checkstack, lua_getfield, lua_getglobal,
    lua_getmetatable, lua_gettop, lua_pcall, lua_pop, lua_pushlstring, lua_pushstring,
    lua_toboolean, lua_type, luaL_checktype,
};
use crate::src::nvim::main::{got_int, mod_mask};
use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::os::env::home_replace_save;
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::strings::{arena_printf, vim_snprintf};
use crate::src::nvim::types::builders::static_cstring;
use crate::src::nvim::types::{
    Arena, Array, LuaRef, Object, String_0, VAR_DICT, VAR_LIST, buf_T, kObjectTypeBoolean, size_t,
    typval_T,
};

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
///
/// # Safety
/// `arg` must be a live typval.
unsafe fn lua_table_ref(arg: *const typval_T) -> LuaRef {
    unsafe {
        match (*arg).v_type {
            VAR_DICT => (*(*arg).vval.v_dict).lua_table_ref,
            VAR_LIST => (*(*arg).vval.v_list).lua_table_ref,
            _ => LUA_NOREF,
        }
    }
}

/// Whether this dictionary or list is a *view* of a Lua table rather than a
/// Vimscript value of its own.
///
/// # Safety
/// `arg` must be a live typval.
pub unsafe extern "C-unwind" fn nlua_is_table_from_lua(arg: *const typval_T) -> bool {
    unsafe { lua_table_ref(arg) != LUA_NOREF }
}

/// If `arg` is a Lua table with a `__call` metamethod, register that
/// metamethod as a Vimscript function and answer its name; otherwise null.
///
/// Every exit leaves the Lua stack exactly as it found it.
///
/// # Safety
/// `arg` must be a live typval and the main state must exist.
pub unsafe extern "C-unwind" fn nlua_register_table_as_callable(
    arg: *const typval_T,
) -> *mut c_char {
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
        name
    }
}

/// `vim.on_key()`: hand every consumed key to the Lua callbacks, and answer
/// whether they want it discarded.
///
/// Not re-entrant — a callback that itself consumes a key is skipped — and
/// it swallows any interrupt a callback raised, restoring whatever `got_int`
/// was before.
///
/// # Safety
/// `typed_buf` must be a NUL-terminated buffer the caller owns, and the main
/// state must exist.
pub unsafe extern "C-unwind" fn nlua_execute_on_key(c: c_int, typed_buf: *mut c_char) -> bool {
    unsafe {
        static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);
        if RECURSIVE.get() {
            return false;
        }
        RECURSIVE.set(true);

        let mut buf = [0 as c_char; 67];
        let buf_len = special_to_buf(c, mod_mask.get(), false, buf.as_mut_ptr()) as size_t;
        vim_unescape_ks(typed_buf);

        let lstate = get_global_lstate();
        let top = lua_gettop(lstate);

        lua_getglobal(lstate, c"vim".as_ptr());
        lua_getfield(lstate, -1, c"_on_key".as_ptr());
        luaL_checktype(lstate, -1, LUA_TFUNCTION);
        lua_pushlstring(lstate, buf.as_ptr(), buf_len);
        lua_pushstring(lstate, typed_buf);

        let save_got_int = got_int.get();
        got_int.set(false);
        let mut discard = false;
        if lua_pcall(lstate, 2, 1, 0) != 0 {
            nlua_error(lstate, gettext(c"vim.on_key() callbacks: %.*s".as_ptr()));
        } else {
            if lua_type(lstate, -1) == LUA_TBOOLEAN {
                discard = lua_toboolean(lstate, -1) != 0;
            }
            lua_pop(lstate, 1);
        }
        got_int.set(got_int.get() || save_got_int);
        lua_pop(lstate, 1);
        debug_assert!(top == lua_gettop(lstate));

        RECURSIVE.set(false);
        discard
    }
}

/// How a `LuaRef` renders in a listing: `<Lua N: file:line>` when the
/// reference is a function defined in a file, `<Lua N>` otherwise.
///
/// # Safety
/// The main state must exist and `arena` be a live arena or null.
pub unsafe extern "C-unwind" fn nlua_funcref_str(ref_0: LuaRef, arena: *mut Arena) -> *mut c_char {
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
                    && *ar.source == b'@' as c_char
                    && ar.linedefined >= 0
                {
                    let src = home_replace_save(ptr::null_mut::<buf_T>(), ar.source.add(1));
                    let str: String_0 = arena_printf(
                        arena,
                        c"<Lua %d: %s:%d>".as_ptr(),
                        ref_0,
                        src,
                        ar.linedefined,
                    );
                    xfree(src.cast::<c_void>());
                    return str.data;
                }
            }
        }
        arena_printf(arena, c"<Lua %d>".as_ptr(), ref_0).data
    }
}

/// `exists('v:lua.f')`: whether the Lua expression names a function.
///
/// # Safety
/// `lua_funcname` must be a NUL-terminated Lua expression.
pub unsafe extern "C-unwind" fn nlua_func_exists(lua_funcname: *const c_char) -> bool {
    unsafe {
        // `return %s` plus the terminator: the name is evaluated as an
        // expression rather than looked up, so `v:lua.pkg.fn` works.
        let length = strlen(lua_funcname).wrapping_add(8);
        let str = xmalloc(length).cast::<c_char>();
        vim_snprintf(str, length, c"return %s".as_ptr(), lua_funcname);

        let mut args__items = [Object::string(cstr_as_string(str))];
        let args = Array {
            size: 1,
            capacity: 1,
            items: args__items.as_mut_ptr(),
        };

        let mut err = ERROR_INIT;
        let result = nlua_exec(
            static_cstring(c"return type(loadstring(...)()) == 'function'"),
            ptr::null::<c_char>(),
            args,
            kRetNilBool,
            ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        xfree(str.cast::<c_void>());
        api_clear_error(&raw mut err);
        result.type_0 == kObjectTypeBoolean && result.data.boolean
    }
}
