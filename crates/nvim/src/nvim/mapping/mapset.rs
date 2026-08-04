//! Setting a mapping from a dict: `mapset()` and `nvim_set_keymap`.
//!
//! Both take an already-built description rather than a command line —
//! [`f_mapset`] a `maparg()` dict, [`modify_keymap`] an API keyset — and both
//! end in [`buf_do_map`] or [`map_add`].

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn f_mapset(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if check_secure() {
            return;
        }
        let mut which: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let mut is_abbr: ::core::ffi::c_int = 0;
        let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        let dict_only: bool = (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type
            as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint;
        if dict_only {
            d = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_dict;
            which = tv_dict_get_string(
                d,
                b"mode\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            is_abbr = tv_dict_get_bool(
                d,
                b"abbr\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int;
            if which.is_null() || is_abbr < 0 as ::core::ffi::c_int {
                emsg(gettext(
                    (e_entries_missing_in_mapset_dict_argument.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ));
                return;
            }
        } else {
            which = tv_get_string_buf_chk(
                argvars.offset(0 as ::core::ffi::c_int as isize),
                &raw mut buf as *mut ::core::ffi::c_char,
            );
            if which.is_null() {
                return;
            }
            is_abbr =
                tv_get_bool(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
            if tv_check_for_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
                return;
            }
            d = (*argvars.offset(2 as ::core::ffi::c_int as isize))
                .vval
                .v_dict;
        }
        let mode: ::core::ffi::c_int = get_map_mode_string(which, is_abbr != 0);
        if mode == 0 as ::core::ffi::c_int {
            semsg(
                gettext(
                    (e_illegal_map_mode_string_str.ptr() as *const _) as *const ::core::ffi::c_char,
                ),
                which,
            );
            return;
        }
        let mut lhs: *mut ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"lhs\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        let mut lhsraw: *mut ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"lhsraw\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        let mut lhsrawalt: *mut ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"lhsrawalt\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        let mut orig_rhs: *mut ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"rhs\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        let mut rhs_lua: LuaRef = LUA_NOREF;
        let mut callback_di: *mut dictitem_T = tv_dict_find(
            d,
            b"callback\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !callback_di.is_null() {
            if (*callback_di).di_tv.v_type as ::core::ffi::c_uint
                == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut fp: *mut ufunc_T = find_func((*callback_di).di_tv.vval.v_string);
                if !fp.is_null() && (*fp).uf_flags & FC_LUAREF != 0 {
                    rhs_lua = api_new_luaref((*fp).uf_luaref);
                    orig_rhs =
                        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                }
            }
        }
        if lhs.is_null() || lhsraw.is_null() || orig_rhs.is_null() {
            emsg(gettext(
                (e_entries_missing_in_mapset_dict_argument.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ));
            api_free_luaref(rhs_lua);
            return;
        }
        let mut noremap: ::core::ffi::c_int =
            if tv_dict_get_number(d, b"noremap\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as varnumber_T
            {
                REMAP_NONE as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
        if tv_dict_get_number(d, b"script\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T
        {
            noremap = REMAP_SCRIPT as ::core::ffi::c_int;
        }
        let mut args: MapArguments = map_arguments {
            buffer: false,
            expr: tv_dict_get_number(d, b"expr\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as varnumber_T,
            noremap: false,
            nowait: tv_dict_get_number(d, b"nowait\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as varnumber_T,
            script: false,
            silent: tv_dict_get_number(d, b"silent\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as varnumber_T,
            unique: false,
            replace_keycodes: tv_dict_get_number(
                d,
                b"replace_keycodes\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0 as varnumber_T,
            lhs: [0; 51],
            lhs_len: 0,
            alt_lhs: [0; 51],
            alt_lhs_len: 0,
            rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            rhs_len: 0,
            rhs_lua: 0,
            rhs_is_noop: false,
            orig_rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            orig_rhs_len: 0,
            desc: tv_dict_get_string(
                d,
                b"desc\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ),
        };
        let mut sid: scid_T =
            tv_dict_get_number(d, b"sid\0".as_ptr() as *const ::core::ffi::c_char) as scid_T;
        let mut lnum: linenr_T =
            tv_dict_get_number(d, b"lnum\0".as_ptr() as *const ::core::ffi::c_char) as linenr_T;
        let mut buffer: bool =
            tv_dict_get_number(d, b"buffer\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as varnumber_T;
        set_maparg_rhs(
            orig_rhs,
            strlen(orig_rhs),
            rhs_lua,
            sid,
            p_cpo.get(),
            &raw mut args,
        );
        let mut map_table: *mut *mut mapblock_T = if buffer as ::core::ffi::c_int != 0 {
            &raw mut (*curbuf.get()).b_maphash as *mut *mut mapblock_T
        } else {
            MAPHASH.ptr() as *mut *mut mapblock_T
        };
        let mut abbr_table: *mut *mut mapblock_T = if buffer as ::core::ffi::c_int != 0 {
            &raw mut (*curbuf.get()).b_first_abbr
        } else {
            FIRST_ABBR.ptr()
        };
        let mut unmap_args: MapArguments = MAP_ARGUMENTS_INIT;
        set_maparg_lhs_rhs(
            lhs,
            strlen(lhs),
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            0 as size_t,
            LUA_NOREF,
            p_cpo.get(),
            &raw mut unmap_args,
        );
        unmap_args.buffer = buffer;
        buf_do_map(
            MAPTYPE_UNMAP_LHS as ::core::ffi::c_int,
            &raw mut unmap_args,
            mode,
            is_abbr != 0,
            curbuf.get(),
        );
        xfree(unmap_args.rhs as *mut ::core::ffi::c_void);
        xfree(unmap_args.orig_rhs as *mut ::core::ffi::c_void);
        let mut mp_result: [*mut mapblock_T; 2] = [
            ::core::ptr::null_mut::<mapblock_T>(),
            ::core::ptr::null_mut::<mapblock_T>(),
        ];
        mp_result[0 as ::core::ffi::c_int as usize] = map_add(
            curbuf.get(),
            map_table,
            abbr_table,
            lhsraw,
            &raw mut args,
            noremap,
            mode,
            is_abbr != 0,
            sid,
            lnum,
            false_0 != 0,
        );
        if !lhsrawalt.is_null() {
            mp_result[1 as ::core::ffi::c_int as usize] = map_add(
                curbuf.get(),
                map_table,
                abbr_table,
                lhsrawalt,
                &raw mut args,
                noremap,
                mode,
                is_abbr != 0,
                sid,
                lnum,
                true_0 != 0,
            );
        }
        if !mp_result[0 as ::core::ffi::c_int as usize].is_null()
            && !mp_result[1 as ::core::ffi::c_int as usize].is_null()
        {
            (*mp_result[0 as ::core::ffi::c_int as usize]).m_alt =
                mp_result[1 as ::core::ffi::c_int as usize];
            (*mp_result[1 as ::core::ffi::c_int as usize]).m_alt =
                mp_result[0 as ::core::ffi::c_int as usize];
        }
    }
}

pub unsafe extern "C" fn modify_keymap(
    mut channel_id: uint64_t,
    mut buffer: Buffer,
    mut is_unmap: bool,
    mut mode: String_0,
    mut lhs: String_0,
    mut rhs: String_0,
    mut opts: *mut KeyDict_keymap,
    mut err: *mut Error,
) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut forceit: bool = false;
        let mut mode_val: ::core::ffi::c_int = 0;
        let mut is_abbrev: bool = false;
        let mut is_noremap: bool = false;
        let mut maptype_val: ::core::ffi::c_int = 0;
        let mut lua_funcref: LuaRef = LUA_NOREF;
        let mut global: bool = buffer == -1 as ::core::ffi::c_int;
        if global {
            buffer = 0 as ::core::ffi::c_int as Buffer;
        }
        let mut target_buf: *mut buf_T = find_buffer_by_handle(buffer, err);
        if target_buf.is_null() {
            return;
        }
        let save_current_sctx: sctx_T = api_set_sctx(channel_id);
        let mut parsed_args: MapArguments = MAP_ARGUMENTS_INIT;
        if !opts.is_null() {
            parsed_args.nowait = (*opts).nowait as bool;
            parsed_args.noremap = (*opts).noremap as bool;
            parsed_args.silent = (*opts).silent as bool;
            parsed_args.script = (*opts).script as bool;
            parsed_args.expr = (*opts).expr as bool;
            parsed_args.unique = (*opts).unique as bool;
            parsed_args.replace_keycodes = (*opts).replace_keycodes as bool;
            if (*opts).is_set__keymap_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_keymap__callback
                != 0 as ::core::ffi::c_ulonglong
            {
                lua_funcref = (*opts).callback;
                (*opts).callback = LUA_NOREF as LuaRef;
            }
            if (*opts).is_set__keymap_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_keymap__desc
                != 0 as ::core::ffi::c_ulonglong
            {
                parsed_args.desc = string_to_cstr((*opts).desc);
            }
        }
        parsed_args.buffer = !global;
        '_fail_and_free: {
            if parsed_args.replace_keycodes as ::core::ffi::c_int != 0 && !parsed_args.expr {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"\"replace_keycodes\" requires \"expr\"\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            } else if !set_maparg_lhs_rhs(
                lhs.data,
                lhs.size,
                rhs.data,
                rhs.size,
                lua_funcref,
                p_cpo.get(),
                &raw mut parsed_args,
            ) {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"LHS exceeds maximum map length: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    lhs.data,
                );
            } else if parsed_args.lhs_len > MAXMAPLEN as ::core::ffi::c_int as size_t
                || parsed_args.alt_lhs_len > MAXMAPLEN as ::core::ffi::c_int as size_t
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"LHS exceeds maximum map length: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    lhs.data,
                );
            } else {
                p = (if mode.size > 0 as size_t {
                    mode.data as *const ::core::ffi::c_char
                } else {
                    b"m\0".as_ptr() as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char;
                forceit = *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int;
                mode_val = get_map_mode(&raw mut p, forceit);
                if forceit {
                    '_c2rust_label: {
                        if p == mode.data {
                        } else {
                            __assert_fail(
                            b"p == mode.data\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/mapping.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            2794 as ::core::ffi::c_uint,
                            b"void modify_keymap(uint64_t, Buffer, _Bool, String, String, String, KeyDict_keymap *, Error *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                        }
                    };
                    p = p.offset(1);
                }
                is_abbrev = mode_val & (MODE_INSERT | MODE_CMDLINE) != 0 as ::core::ffi::c_int
                    && *p as ::core::ffi::c_int == 'a' as ::core::ffi::c_int;
                if is_abbrev {
                    p = p.offset(1);
                }
                if mode.size > 0 as size_t && p.offset_from(mode.data) as size_t != mode.size {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Invalid mode shortname: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
                        mode.data,
                    );
                } else if parsed_args.lhs_len == 0 as size_t {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Invalid (empty) LHS\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                } else {
                    is_noremap = parsed_args.noremap;
                    '_c2rust_label_0: {
                        if !(is_unmap as ::core::ffi::c_int != 0
                            && is_noremap as ::core::ffi::c_int != 0)
                        {
                        } else {
                            __assert_fail(
                            b"!(is_unmap && is_noremap)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/mapping.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            2812 as ::core::ffi::c_uint,
                            b"void modify_keymap(uint64_t, Buffer, _Bool, String, String, String, KeyDict_keymap *, Error *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                        }
                    };
                    if !is_unmap
                        && lua_funcref == LUA_NOREF
                        && (parsed_args.rhs_len == 0 as size_t && !parsed_args.rhs_is_noop)
                    {
                        if rhs.size == 0 as size_t {
                            parsed_args.rhs_is_noop = true_0 != 0;
                        } else {
                            abort();
                        }
                    } else if is_unmap as ::core::ffi::c_int != 0
                        && (parsed_args.rhs_len != 0 || parsed_args.rhs_lua != LUA_NOREF)
                    {
                        if parsed_args.rhs_len != 0 {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Gave nonempty RHS in unmap command: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                parsed_args.rhs,
                            );
                        } else {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Gave nonempty RHS for unmap\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                        break '_fail_and_free;
                    }
                    maptype_val = MAPTYPE_MAP as ::core::ffi::c_int;
                    if is_unmap {
                        maptype_val = MAPTYPE_UNMAP as ::core::ffi::c_int;
                    } else if is_noremap {
                        maptype_val = MAPTYPE_NOREMAP as ::core::ffi::c_int;
                    }
                    match buf_do_map(
                        maptype_val,
                        &raw mut parsed_args,
                        mode_val,
                        is_abbrev,
                        target_buf,
                    ) {
                        0 => {}
                        1 => {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                &raw const e_invarg as *const ::core::ffi::c_char,
                                0 as ::core::ffi::c_int,
                            );
                        }
                        2 => {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                &raw const e_nomap as *const ::core::ffi::c_char,
                                0 as ::core::ffi::c_int,
                            );
                        }
                        5 => {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                if is_abbrev as ::core::ffi::c_int != 0 {
                                    (e_abbreviation_already_exists_for_str.ptr() as *const _)
                                        as *const ::core::ffi::c_char
                                } else {
                                    (e_mapping_already_exists_for_str.ptr() as *const _)
                                        as *const ::core::ffi::c_char
                                },
                                lhs.data,
                            );
                        }
                        6 => {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                if is_abbrev as ::core::ffi::c_int != 0 {
                                    (e_global_abbreviation_already_exists_for_str.ptr() as *const _)
                                        as *const ::core::ffi::c_char
                                } else {
                                    (e_global_mapping_already_exists_for_str.ptr() as *const _)
                                        as *const ::core::ffi::c_char
                                },
                                lhs.data,
                            );
                        }
                        _ => {
                            '_c2rust_label_1: {
                                if false {
                                } else {
                                    __assert_fail(
                                    b"false && \"Unrecognized return code!\"\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/mapping.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2860 as ::core::ffi::c_uint,
                                    b"void modify_keymap(uint64_t, Buffer, _Bool, String, String, String, KeyDict_keymap *, Error *)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                                }
                            };
                        }
                    }
                }
            }
        }
        current_sctx.set(save_current_sctx);
        if parsed_args.rhs_lua != LUA_NOREF {
            api_free_luaref(parsed_args.rhs_lua);
            parsed_args.rhs_lua = LUA_NOREF as LuaRef;
        }
        xfree(parsed_args.rhs as *mut ::core::ffi::c_void);
        xfree(parsed_args.orig_rhs as *mut ::core::ffi::c_void);
        xfree(parsed_args.desc as *mut ::core::ffi::c_void);
    }
}
