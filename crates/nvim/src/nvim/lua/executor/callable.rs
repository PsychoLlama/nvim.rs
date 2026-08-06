//! Lua values that Vimscript can call, and asking whether it can.
//!
//! `nlua_register_table_as_callable` gives a table with a `__call`
//! metamethod a `LuaRef` so it can be stored as a Funcref, `nlua_funcref_str`
//! renders one back to the `<Lua N: file:line>` form a listing shows, and
//! `nlua_func_exists` answers `exists('v:lua.…')`.  `nlua_execute_on_key` is
//! the `vim.on_key()` callback, called for every key the editor consumes.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C-unwind" fn nlua_is_table_from_lua(arg: *const typval_T) -> bool {
    unsafe {
        if (*arg).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return (*(*arg).vval.v_dict).lua_table_ref != LUA_NOREF;
        } else if (*arg).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return (*(*arg).vval.v_list).lua_table_ref != LUA_NOREF;
        } else {
            return false_0 != 0;
        };
    }
}

pub unsafe extern "C-unwind" fn nlua_register_table_as_callable(
    arg: *const typval_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut table_ref: LuaRef = LUA_NOREF;
        if (*arg).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            table_ref = (*(*arg).vval.v_dict).lua_table_ref;
        } else if (*arg).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            table_ref = (*(*arg).vval.v_list).lua_table_ref;
        }
        if table_ref == LUA_NOREF {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let lstate: *mut lua_State = global_lstate.get();
        let mut top: ::core::ffi::c_int = lua_gettop(lstate);
        nlua_pushref(lstate, table_ref);
        if lua_getmetatable(lstate, -1 as ::core::ffi::c_int) == 0 {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            '_c2rust_label: {
                if top == lua_gettop(lstate) {
                } else {
                    __assert_fail(
                        b"top == lua_gettop(lstate)\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2064 as ::core::ffi::c_uint,
                        b"char *nlua_register_table_as_callable(const typval_T *const)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        lua_getfield(
            lstate,
            -1 as ::core::ffi::c_int,
            b"__call\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if !(lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TFUNCTION) {
            lua_settop(lstate, -3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            '_c2rust_label_0: {
                if top == lua_gettop(lstate) {
                } else {
                    __assert_fail(
                        b"top == lua_gettop(lstate)\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2071 as ::core::ffi::c_uint,
                        b"char *nlua_register_table_as_callable(const typval_T *const)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        let mut func: LuaRef = nlua_ref_global(lstate, -1 as ::core::ffi::c_int);
        let mut name: *mut ::core::ffi::c_char = register_luafunc(func);
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        '_c2rust_label_1: {
            if top == lua_gettop(lstate) {
            } else {
                __assert_fail(
                    b"top == lua_gettop(lstate)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2081 as ::core::ffi::c_uint,
                    b"char *nlua_register_table_as_callable(const typval_T *const)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return name;
    }
}

pub unsafe extern "C-unwind" fn nlua_execute_on_key(
    mut c: ::core::ffi::c_int,
    mut typed_buf: *mut ::core::ffi::c_char,
) -> bool {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if recursive.get() {
            return false_0 != 0;
        }
        recursive.set(true_0 != 0);
        let mut buf: [::core::ffi::c_char; 67] = [0; 67];
        let mut buf_len: size_t = special_to_buf(
            c,
            mod_mask.get(),
            false_0 != 0,
            &raw mut buf as *mut ::core::ffi::c_char,
        ) as size_t;
        vim_unescape_ks(typed_buf);
        let lstate: *mut lua_State = global_lstate.get();
        let mut top: ::core::ffi::c_int = lua_gettop(lstate);
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"vim\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_getfield(
            lstate,
            -1 as ::core::ffi::c_int,
            b"_on_key\0".as_ptr() as *const ::core::ffi::c_char,
        );
        luaL_checktype(lstate, -1 as ::core::ffi::c_int, LUA_TFUNCTION);
        lua_pushlstring(lstate, &raw mut buf as *mut ::core::ffi::c_char, buf_len);
        lua_pushstring(lstate, typed_buf);
        let mut save_got_int: ::core::ffi::c_int = got_int.get() as ::core::ffi::c_int;
        got_int.set(false_0 != 0);
        let mut discard: bool = false_0 != 0;
        if lua_pcall(
            lstate,
            2 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        ) != 0
        {
            nlua_error(
                lstate,
                gettext(b"vim.on_key() callbacks: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
        } else {
            if lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TBOOLEAN {
                discard = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
            }
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
        got_int.set(got_int.get() as ::core::ffi::c_int | save_got_int != 0);
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        '_c2rust_label: {
            if top == lua_gettop(lstate) {
            } else {
                __assert_fail(
                    b"top == lua_gettop(lstate)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2138 as ::core::ffi::c_uint,
                    b"_Bool nlua_execute_on_key(int, char *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        recursive.set(false_0 != 0);
        return discard;
    }
}

pub unsafe extern "C-unwind" fn nlua_funcref_str(
    mut ref_0: LuaRef,
    mut arena: *mut Arena,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut ar: lua_Debug = lua_Debug {
            event: 0,
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            namewhat: ::core::ptr::null::<::core::ffi::c_char>(),
            what: ::core::ptr::null::<::core::ffi::c_char>(),
            source: ::core::ptr::null::<::core::ffi::c_char>(),
            currentline: 0,
            nups: 0,
            linedefined: 0,
            lastlinedefined: 0,
            short_src: [0; 60],
            i_ci: 0,
        };
        let lstate: *mut lua_State = global_lstate.get();
        if lua_checkstack(lstate, 1 as ::core::ffi::c_int) != 0 {
            nlua_pushref(lstate, ref_0);
            if !(lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TFUNCTION) {
                lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            } else {
                ar = lua_Debug {
                    event: 0,
                    name: ::core::ptr::null::<::core::ffi::c_char>(),
                    namewhat: ::core::ptr::null::<::core::ffi::c_char>(),
                    what: ::core::ptr::null::<::core::ffi::c_char>(),
                    source: ::core::ptr::null::<::core::ffi::c_char>(),
                    currentline: 0,
                    nups: 0,
                    linedefined: 0,
                    lastlinedefined: 0,
                    short_src: [0; 60],
                    i_ci: 0,
                };
                if lua_getinfo(
                    lstate,
                    b">S\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut ar,
                ) != 0
                    && *ar.source as ::core::ffi::c_int == '@' as ::core::ffi::c_int
                    && ar.linedefined >= 0 as ::core::ffi::c_int
                {
                    let mut src: *mut ::core::ffi::c_char = home_replace_save(
                        ::core::ptr::null_mut::<buf_T>(),
                        ar.source.offset(1 as ::core::ffi::c_int as isize),
                    );
                    let mut str: String_0 = arena_printf(
                        arena,
                        b"<Lua %d: %s:%d>\0".as_ptr() as *const ::core::ffi::c_char,
                        ref_0,
                        src,
                        ar.linedefined,
                    );
                    xfree(src as *mut ::core::ffi::c_void);
                    return str.data;
                }
            }
        }
        return arena_printf(
            arena,
            b"<Lua %d>\0".as_ptr() as *const ::core::ffi::c_char,
            ref_0,
        )
        .data;
    }
}

pub unsafe extern "C-unwind" fn nlua_func_exists(
    mut lua_funcname: *const ::core::ffi::c_char,
) -> bool {
    unsafe {
        let mut args: Array = ARRAY_DICT_INIT;
        let mut args__items: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_11 { boolean: false },
        }; 1];
        args.capacity = 1 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let mut length: size_t = strlen(lua_funcname).wrapping_add(8 as size_t);
        let mut str: *mut ::core::ffi::c_char = xmalloc(length) as *mut ::core::ffi::c_char;
        vim_snprintf(
            str,
            length,
            b"return %s\0".as_ptr() as *const ::core::ffi::c_char,
            lua_funcname,
        );
        let c2rust_fresh3 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh3 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_11 {
                string: cstr_as_string(str),
            },
        };
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut result: Object = nlua_exec(
            String_0 {
                data: b"return type(loadstring(...)()) == 'function'\0".as_ptr()
                    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 45]>().wrapping_sub(1 as size_t),
            },
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetNilBool,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        xfree(str as *mut ::core::ffi::c_void);
        api_clear_error(&raw mut err);
        return result.type_0 as ::core::ffi::c_uint
            == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            && result.data.boolean as ::core::ffi::c_int == true_0;
    }
}
// LuaJIT bytecode for the builtin `vim.*` modules, compiled by build.rs
// (src/gen/compile_lua_modules.lua) from `runtime/lua/vim/`. c2rust
// originally transpiled these as ~215k lines of array literals frozen from
// upstream's generated char blobs; now the sources next to the binary are
// the sources inside it. Each blob carries gen_char_blob.lua's trailing 0
// sentinel, which is why nlua_module_preloader loads `size - 1` bytes.
