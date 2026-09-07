//! The parser's optional log callback.
//!
//! `parser_set_logger` installs a [`TSLogger`] whose payload is a Lua
//! reference plus the [`TSLuaLoggerOpts`] flags saying which of lex/parse to
//! report; `logger_cb` is what tree-sitter calls, and `logger_gc` releases
//! the reference when the parser dies.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/treesitter/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use super::*;

/// # Safety
///
/// `logger` must be a `TSLogger` this module installed: its `payload` is read
/// as a `TSLuaLoggerOpts` and freed, so a logger from anywhere else -- or one
/// already collected -- is a use-after-free.
pub(crate) unsafe fn logger_gc(logger: TSLogger) {
    unsafe {
        if logger.log.is_none() {
            return;
        }
        let opts: *mut TSLuaLoggerOpts = logger.payload as *mut TSLuaLoggerOpts;
        luaL_unref(
            (*opts).lstate,
            LUA_REGISTRYINDEX,
            (*opts).cb as ::core::ffi::c_int,
        );
        xfree(opts as *mut ::core::ffi::c_void);
    }
}

/// # Safety
///
/// `payload` must point at the `TSLuaLoggerOpts` this logger was installed
/// with and `s` at a NUL-terminated message; tree-sitter's callback contract,
/// which hands back exactly what it was given.
unsafe extern "C" fn logger_cb(
    payload: *mut ::core::ffi::c_void,
    logtype: TSLogType,
    s: *const ::core::ffi::c_char,
) {
    unsafe {
        let opts: *mut TSLuaLoggerOpts = payload as *mut TSLuaLoggerOpts;
        if !(*opts).lex
            && logtype as ::core::ffi::c_uint
                == TSLogTypeLex as ::core::ffi::c_int as ::core::ffi::c_uint
            || !(*opts).parse
                && logtype as ::core::ffi::c_uint
                    == TSLogTypeParse as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return;
        }
        let lstate: *mut lua_State = (*opts).lstate;
        lua_rawgeti(lstate, LUA_REGISTRYINDEX, (*opts).cb as ::core::ffi::c_int);
        lua_pushstring(
            lstate,
            if logtype as ::core::ffi::c_uint
                == TSLogTypeParse as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                c"parse".as_ptr()
            } else {
                c"lex".as_ptr()
            },
        );
        lua_pushstring(lstate, s);
        if lua_pcall(
            lstate,
            2 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        ) != 0
        {
            luaL_error(lstate, c"treesitter logger callback failed".as_ptr());
        }
    }
}

/// # Safety
///
/// `L` must point at the Lua state this call runs on.
pub(crate) unsafe extern "C-unwind" fn parser_set_logger(L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
        luaL_argcheck(
            L,
            lua_type(L, 2 as ::core::ffi::c_int) == 1 as ::core::ffi::c_int,
            2 as ::core::ffi::c_int,
            c"boolean expected".as_ptr(),
        );
        luaL_argcheck(
            L,
            lua_type(L, 3 as ::core::ffi::c_int) == 1 as ::core::ffi::c_int,
            3 as ::core::ffi::c_int,
            c"boolean expected".as_ptr(),
        );
        luaL_argcheck(
            L,
            lua_type(L, 4 as ::core::ffi::c_int) == 6 as ::core::ffi::c_int,
            4 as ::core::ffi::c_int,
            c"function expected".as_ptr(),
        );
        let opts: *mut TSLuaLoggerOpts =
            xmalloc(::core::mem::size_of::<TSLuaLoggerOpts>()) as *mut TSLuaLoggerOpts;
        lua_pushvalue(L, 4 as ::core::ffi::c_int);
        let ref_0: LuaRef = luaL_ref(L, LUA_REGISTRYINDEX);
        *opts = TSLuaLoggerOpts {
            cb: ref_0,
            lstate: L,
            lex: lua_toboolean(L, 2 as ::core::ffi::c_int) != 0,
            parse: lua_toboolean(L, 3 as ::core::ffi::c_int) != 0,
        };
        let logger: TSLogger = TSLogger {
            payload: opts as *mut ::core::ffi::c_void,
            log: Some(
                logger_cb
                    as unsafe extern "C" fn(
                        *mut ::core::ffi::c_void,
                        TSLogType,
                        *const ::core::ffi::c_char,
                    ) -> (),
            ),
        };
        ts_parser_set_logger(p, logger);
        0 as ::core::ffi::c_int
    }
}

/// # Safety
///
/// `L` must point at the Lua state this call runs on.
pub(crate) unsafe extern "C-unwind" fn parser_get_logger(L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
        let logger: TSLogger = ts_parser_logger(p);
        if logger.log.is_some() {
            let opts: *mut TSLuaLoggerOpts = logger.payload as *mut TSLuaLoggerOpts;
            lua_rawgeti(L, LUA_REGISTRYINDEX, (*opts).cb as ::core::ffi::c_int);
        } else {
            lua_pushnil(L);
        }
        1 as ::core::ffi::c_int
    }
}
