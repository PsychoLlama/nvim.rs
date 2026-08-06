//! Loading a parser library and registering it under a language name.
//!
//! `add_language` is the one entry point: it `dlopen`s the library (or takes
//! a wasm module), calls its `tree_sitter_<lang>` constructor, checks the
//! ABI version against `TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION` and
//! stores the `TSLanguage *` in [`langs`].  `tslua_inspect_lang` renders a
//! loaded language's symbols, fields and metadata back to Lua.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C-unwind" fn tslua_has_language(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut lang_name: *const ::core::ffi::c_char = luaL_checklstring(
            L,
            1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
        lua_pushboolean(
            L,
            set_has_cstr_t(&raw mut (*langs.ptr()).set, lang_name as cstr_t) as ::core::ffi::c_int,
        );
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn tslua_add_language_from_object(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        return add_language(L, false_0 != 0);
    }
}

unsafe extern "C-unwind" fn load_language_from_object(
    mut L: *mut lua_State,
    mut path: *const ::core::ffi::c_char,
    mut lang_name: *const ::core::ffi::c_char,
    mut symbol: *const ::core::ffi::c_char,
) -> *const TSLanguage {
    unsafe {
        let mut lib: uv_lib_t = uv_lib_t {
            handle: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        if uv_dlopen(path, &raw mut lib) != 0 {
            xstrlcpy(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                uv_dlerror(&raw mut lib),
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
            );
            uv_dlclose(&raw mut lib);
            luaL_error(
                L,
                c"Failed to load parser for language '%s': uv_dlopen: %s".as_ptr(),
                lang_name,
                IObuff.ptr() as *mut ::core::ffi::c_char,
            );
        }
        let mut symbol_buf: [::core::ffi::c_char; 128] = [0; 128];
        snprintf(
            &raw mut symbol_buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>(),
            c"tree_sitter_%s".as_ptr(),
            symbol,
        );
        let mut lang_parser: Option<unsafe extern "C" fn() -> *mut TSLanguage> = None;
        if uv_dlsym(
            &raw mut lib,
            &raw mut symbol_buf as *mut ::core::ffi::c_char,
            &raw mut lang_parser as *mut *mut ::core::ffi::c_void,
        ) != 0
        {
            xstrlcpy(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                uv_dlerror(&raw mut lib),
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
            );
            uv_dlclose(&raw mut lib);
            luaL_error(
                L,
                c"Failed to load parser: uv_dlsym: %s".as_ptr(),
                IObuff.ptr() as *mut ::core::ffi::c_char,
            );
        }
        let mut lang: *mut TSLanguage = lang_parser.expect("non-null function pointer")();
        if lang.is_null() {
            uv_dlclose(&raw mut lib);
            luaL_error(
                L,
                c"Failed to load parser %s: internal error".as_ptr(),
                path,
            );
        }
        return lang;
    }
}

unsafe extern "C-unwind" fn load_language_from_wasm(
    mut L: *mut lua_State,
    mut _path: *const ::core::ffi::c_char,
    mut _lang_name: *const ::core::ffi::c_char,
) -> *const TSLanguage {
    unsafe {
        luaL_error(L, c"Not supported".as_ptr());
        return ::core::ptr::null::<TSLanguage>();
    }
}

unsafe extern "C-unwind" fn add_language(
    mut L: *mut lua_State,
    mut is_wasm: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut path: *const ::core::ffi::c_char = luaL_checklstring(
            L,
            1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
        let mut lang_name: *const ::core::ffi::c_char = luaL_checklstring(
            L,
            2 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
        let mut symbol_name: *const ::core::ffi::c_char = lang_name;
        if !is_wasm
            && lua_gettop(L) >= 3 as ::core::ffi::c_int
            && !(lua_type(L, 3 as ::core::ffi::c_int) == LUA_TNIL)
        {
            symbol_name = luaL_checklstring(
                L,
                3 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            );
        }
        if set_has_cstr_t(&raw mut (*langs.ptr()).set, lang_name as cstr_t) {
            lua_pushboolean(L, true_0);
            return 1 as ::core::ffi::c_int;
        }
        let mut lang: *const TSLanguage = if is_wasm as ::core::ffi::c_int != 0 {
            load_language_from_wasm(L, path, lang_name)
        } else {
            load_language_from_object(L, path, lang_name, symbol_name)
        };
        let mut lang_version: uint32_t = ts_language_abi_version(lang);
        if lang_version < TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION as uint32_t
            || lang_version > TREE_SITTER_LANGUAGE_VERSION as uint32_t
        {
            return luaL_error(
                L,
                c"ABI version mismatch for %s: supported between %d and %d, found %d".as_ptr(),
                path,
                TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
                TREE_SITTER_LANGUAGE_VERSION,
                lang_version,
            );
        }
        map_put_cstr_t_ptr_t(
            langs.ptr(),
            xstrdup(lang_name) as cstr_t,
            lang as *mut TSLanguage as ptr_t,
        );
        lua_pushboolean(L, true_0);
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn tslua_remove_lang(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut lang_name: *const ::core::ffi::c_char = luaL_checklstring(
            L,
            1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
        let mut present: bool = set_has_cstr_t(&raw mut (*langs.ptr()).set, lang_name as cstr_t);
        if present {
            let mut key: cstr_t = ::core::ptr::null::<::core::ffi::c_char>();
            map_del_cstr_t_ptr_t(langs.ptr(), lang_name as cstr_t, &raw mut key);
            xfree(key as *mut ::core::ffi::c_void);
        }
        lua_pushboolean(L, present as ::core::ffi::c_int);
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn lang_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSLanguage {
    unsafe {
        let mut lang_name: *const ::core::ffi::c_char =
            luaL_checklstring(L, index, ::core::ptr::null_mut::<size_t>());
        let mut lang: *mut TSLanguage =
            map_get_cstr_t_ptr_t(langs.ptr(), lang_name as cstr_t) as *mut TSLanguage;
        if lang.is_null() {
            luaL_error(L, c"no such language: %s".as_ptr(), lang_name);
        }
        return lang;
    }
}

pub(crate) unsafe extern "C-unwind" fn tslua_inspect_lang(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut lang: *mut TSLanguage = lang_check(L, 1 as ::core::ffi::c_int);
        lua_createtable(L, 0 as ::core::ffi::c_int, 2 as ::core::ffi::c_int);
        let mut nsymbols: uint32_t = ts_language_symbol_count(lang);
        '_c2rust_label: {
            if nsymbols < 2147483647 as uint32_t {
            } else {
                __assert_fail(
                    c"nsymbols < INT_MAX".as_ptr(),
                    c"src/nvim/lua/treesitter.rs".as_ptr(),
                    276 as ::core::ffi::c_uint,
                    c"int tslua_inspect_lang(lua_State *)".as_ptr(),
                );
            }
        };
        lua_createtable(
            L,
            nsymbols.wrapping_sub(1 as uint32_t) as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
        let mut i: uint32_t = 0 as uint32_t;
        while i < nsymbols {
            let mut t: TSSymbolType = ts_language_symbol_type(lang, i as TSSymbol);
            if t as ::core::ffi::c_uint
                != TSSymbolTypeAuxiliary as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut name: *const ::core::ffi::c_char =
                    ts_language_symbol_name(lang, i as TSSymbol);
                let mut named: bool = t as ::core::ffi::c_uint
                    != TSSymbolTypeAnonymous as ::core::ffi::c_int as ::core::ffi::c_uint;
                lua_pushboolean(L, named as ::core::ffi::c_int);
                if !named {
                    let mut buf: [::core::ffi::c_char; 256] = [0; 256];
                    snprintf(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
                        c"\"%s\"".as_ptr(),
                        name,
                    );
                    lua_setfield(
                        L,
                        -2 as ::core::ffi::c_int,
                        &raw mut buf as *mut ::core::ffi::c_char,
                    );
                } else {
                    lua_setfield(L, -2 as ::core::ffi::c_int, name);
                }
            }
            i = i.wrapping_add(1);
        }
        lua_setfield(L, -2 as ::core::ffi::c_int, c"symbols".as_ptr());
        let mut nfields: uint32_t = ts_language_field_count(lang);
        lua_createtable(L, nfields as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        let mut i_0: uint32_t = 1 as uint32_t;
        while i_0 <= nfields {
            lua_pushstring(L, ts_language_field_name_for_id(lang, i_0 as TSFieldId));
            lua_rawseti(L, -2 as ::core::ffi::c_int, i_0 as ::core::ffi::c_int);
            i_0 = i_0.wrapping_add(1);
        }
        lua_setfield(L, -2 as ::core::ffi::c_int, c"fields".as_ptr());
        lua_pushboolean(L, ts_language_is_wasm(lang) as ::core::ffi::c_int);
        lua_setfield(L, -2 as ::core::ffi::c_int, c"_wasm".as_ptr());
        lua_pushinteger(L, ts_language_abi_version(lang) as lua_Integer);
        lua_setfield(L, -2 as ::core::ffi::c_int, c"abi_version".as_ptr());
        let mut meta: *const TSLanguageMetadata = ts_language_metadata(lang);
        if !meta.is_null() {
            lua_createtable(L, 0 as ::core::ffi::c_int, 3 as ::core::ffi::c_int);
            lua_pushinteger(L, (*meta).major_version as lua_Integer);
            lua_setfield(L, -2 as ::core::ffi::c_int, c"major_version".as_ptr());
            lua_pushinteger(L, (*meta).minor_version as lua_Integer);
            lua_setfield(L, -2 as ::core::ffi::c_int, c"minor_version".as_ptr());
            lua_pushinteger(L, (*meta).patch_version as lua_Integer);
            lua_setfield(L, -2 as ::core::ffi::c_int, c"patch_version".as_ptr());
            lua_setfield(L, -2 as ::core::ffi::c_int, c"metadata".as_ptr());
        }
        lua_pushinteger(L, ts_language_state_count(lang) as lua_Integer);
        lua_setfield(L, -2 as ::core::ffi::c_int, c"state_count".as_ptr());
        let mut nsupertypes: uint32_t = 0;
        let mut supertypes: *const TSSymbol = ts_language_supertypes(lang, &raw mut nsupertypes);
        lua_createtable(
            L,
            0 as ::core::ffi::c_int,
            nsupertypes as ::core::ffi::c_int,
        );
        let mut i_1: uint32_t = 0 as uint32_t;
        while i_1 < nsupertypes {
            let supertype: TSSymbol = *supertypes.add(i_1 as usize);
            let mut nsubtypes: uint32_t = 0;
            let mut subtypes: *const TSSymbol =
                ts_language_subtypes(lang, supertype, &raw mut nsubtypes);
            lua_createtable(L, nsubtypes as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            let mut j: uint32_t = 1 as uint32_t;
            while j <= nsubtypes {
                lua_pushstring(L, ts_language_symbol_name(lang, *subtypes.add(j as usize)));
                lua_rawseti(L, -2 as ::core::ffi::c_int, j as ::core::ffi::c_int);
                j = j.wrapping_add(1);
            }
            lua_setfield(
                L,
                -2 as ::core::ffi::c_int,
                ts_language_symbol_name(lang, supertype),
            );
            i_1 = i_1.wrapping_add(1);
        }
        lua_setfield(L, -2 as ::core::ffi::c_int, c"supertypes".as_ptr());
        return 1 as ::core::ffi::c_int;
    }
}
