//! Reporting mappings to Vimscript and to the API.
//!
//! [`mapblock_fill_dict`] renders one [`mapblock_T`] as the twenty-key dict
//! that `maparg()`, `maplist()` and `nvim_get_keymap` all answer with;
//! [`get_maparg`] backs `maparg()`/`mapcheck()` and [`keymap_array`] backs
//! `nvim_get_keymap`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn f_hasmapto(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut mode: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let name: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut abbr: bool = false_0 != 0;
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            mode = b"nvo\0".as_ptr() as *const ::core::ffi::c_char;
        } else {
            mode = tv_get_string_buf(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut buf as *mut ::core::ffi::c_char,
            );
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                abbr = tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) != 0;
            }
        }
        (*rettv).vval.v_number = map_to_exists(name, mode, abbr) as varnumber_T;
    }
}

pub(crate) unsafe extern "C" fn mapblock_fill_dict(
    mp: *const mapblock_T,
    mut lhsrawalt: *const ::core::ffi::c_char,
    buffer_value: ::core::ffi::c_int,
    abbr: bool,
    compatible: bool,
    mut arena: *mut Arena,
) -> Dict {
    unsafe {
        let mut dict: Dict = arena_dict(arena, 20 as size_t);
        let lhs: *mut ::core::ffi::c_char =
            str2special_arena((*mp).m_keys, compatible, !compatible, arena);
        let mapmode: *mut ::core::ffi::c_char =
            arena_alloc(arena, 7 as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
        mapmode.copy_from_nonoverlapping(map_mode_to_chars((*mp).m_mode).as_ptr(), 7);
        let mut noremap_value: ::core::ffi::c_int = 0;
        if compatible {
            noremap_value = ((*mp).m_noremap != 0) as ::core::ffi::c_int;
        } else {
            noremap_value = if (*mp).m_noremap == REMAP_SCRIPT as ::core::ffi::c_int {
                2 as ::core::ffi::c_int
            } else {
                ((*mp).m_noremap != 0) as ::core::ffi::c_int
            };
        }
        if (*mp).m_luaref != LUA_NOREF {
            let c2rust_fresh21 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh21 as isize) = key_value_pair {
                key: cstr_as_string(b"callback\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeLuaRef,
                    data: C2Rust_Unnamed {
                        luaref: api_new_luaref((*mp).m_luaref),
                    },
                },
            };
        } else {
            let mut rhs: String_0 = cstr_as_string(if compatible as ::core::ffi::c_int != 0 {
                (*mp).m_orig_str
            } else {
                str2special_arena((*mp).m_str, false_0 != 0, true_0 != 0, arena)
            });
            let c2rust_fresh22 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh22 as isize) = key_value_pair {
                key: cstr_as_string(b"rhs\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed { string: rhs },
                },
            };
        }
        if !(*mp).m_desc.is_null() {
            let c2rust_fresh23 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh23 as isize) = key_value_pair {
                key: cstr_as_string(b"desc\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string((*mp).m_desc),
                    },
                },
            };
        }
        let c2rust_fresh24 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh24 as isize) = key_value_pair {
            key: cstr_as_string(b"lhs\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(lhs),
                },
            },
        };
        let c2rust_fresh25 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh25 as isize) = key_value_pair {
            key: cstr_as_string(b"lhsraw\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*mp).m_keys),
                },
            },
        };
        if !lhsrawalt.is_null() {
            let c2rust_fresh26 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh26 as isize) = key_value_pair {
                key: cstr_as_string(b"lhsrawalt\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(lhsrawalt),
                    },
                },
            };
        }
        let c2rust_fresh27 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh27 as isize) = key_value_pair {
            key: cstr_as_string(b"noremap\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: noremap_value as Integer,
                },
            },
        };
        let c2rust_fresh28 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh28 as isize) = key_value_pair {
            key: cstr_as_string(b"script\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (if (*mp).m_noremap == REMAP_SCRIPT as ::core::ffi::c_int {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as Integer,
                },
            },
        };
        let c2rust_fresh29 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh29 as isize) = key_value_pair {
            key: cstr_as_string(b"expr\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (if (*mp).m_expr as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as Integer,
                },
            },
        };
        let c2rust_fresh30 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh30 as isize) = key_value_pair {
            key: cstr_as_string(b"silent\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (if (*mp).m_silent as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as Integer,
                },
            },
        };
        let c2rust_fresh31 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh31 as isize) = key_value_pair {
            key: cstr_as_string(b"sid\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*mp).m_script_ctx.sc_sid as Integer,
                },
            },
        };
        let c2rust_fresh32 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh32 as isize) = key_value_pair {
            key: cstr_as_string(b"scriptversion\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: 1 as Integer,
                },
            },
        };
        let c2rust_fresh33 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh33 as isize) = key_value_pair {
            key: cstr_as_string(b"lnum\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*mp).m_script_ctx.sc_lnum as Integer,
                },
            },
        };
        let c2rust_fresh34 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh34 as isize) = key_value_pair {
            key: cstr_as_string(b"buffer\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: buffer_value as Integer,
                },
            },
        };
        if !compatible {
            let c2rust_fresh35 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh35 as isize) = key_value_pair {
                key: cstr_as_string(b"buf\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: buffer_value as Integer,
                    },
                },
            };
        }
        let c2rust_fresh36 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh36 as isize) = key_value_pair {
            key: cstr_as_string(b"nowait\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (if (*mp).m_nowait as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as Integer,
                },
            },
        };
        let c2rust_fresh37 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh37 as isize) = key_value_pair {
            key: cstr_as_string(b"replace_keycodes\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (if (*mp).m_replace_keycodes as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as Integer,
                },
            },
        };
        let c2rust_fresh38 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh38 as isize) = key_value_pair {
            key: cstr_as_string(b"mode\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(mapmode),
                },
            },
        };
        let c2rust_fresh39 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh39 as isize) = key_value_pair {
            key: cstr_as_string(b"abbr\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (if abbr as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as Integer,
                },
            },
        };
        let c2rust_fresh40 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh40 as isize) = key_value_pair {
            key: cstr_as_string(b"mode_bits\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*mp).m_mode as Integer,
                },
            },
        };
        return dict;
    }
}

pub(crate) unsafe extern "C" fn get_maparg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut exact: ::core::ffi::c_int,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut keys: *mut ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_char;
        if *keys as ::core::ffi::c_int == NUL {
            return;
        }
        let mut which: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let mut abbr: bool = false_0 != 0;
        let mut get_dict: bool = false_0 != 0;
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            which = tv_get_string_buf_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut buf as *mut ::core::ffi::c_char,
            );
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                abbr = tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) != 0;
                if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    get_dict = tv_get_number(argvars.offset(3 as ::core::ffi::c_int as isize)) != 0;
                }
            }
        } else {
            which = b"\0".as_ptr() as *const ::core::ffi::c_char;
        }
        if which.is_null() {
            return;
        }
        let mut keys_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut alt_keys_buf: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut did_simplify: bool = false_0 != 0;
        let flags: ::core::ffi::c_int =
            REPTERM_FROM_PART as ::core::ffi::c_int | REPTERM_DO_LT as ::core::ffi::c_int;
        let mode: ::core::ffi::c_int =
            get_map_mode(&raw mut which as *mut *mut ::core::ffi::c_char, false);
        let mut keys_simplified: *mut ::core::ffi::c_char = replace_termcodes(
            keys,
            strlen(keys),
            &raw mut keys_buf,
            0 as scid_T,
            flags,
            &raw mut did_simplify,
            p_cpo.get(),
        );
        let mut found = check_map(keys_simplified, mode, exact != 0, false, abbr);
        if did_simplify {
            replace_termcodes(
                keys,
                strlen(keys),
                &raw mut alt_keys_buf,
                0 as scid_T,
                flags | REPTERM_NO_SIMPLIFY as ::core::ffi::c_int,
                ::core::ptr::null_mut::<bool>(),
                p_cpo.get(),
            );
            // When the lhs is being simplified the not-simplified keys are
            // preferred, like in do_map(). Upstream leaves the previous `mp`
            // in place when this second look-up fails, but then both `rhs`
            // and `rhs_lua` are cleared, so every reader of `mp` is behind a
            // test that has already failed -- dropping the whole match is the
            // same answer.
            found = check_map(alt_keys_buf, mode, exact != 0, false, abbr);
        }
        let mp = found.as_ref().map_or(::core::ptr::null_mut(), |f| f.mp);
        let buffer_local = ::core::ffi::c_int::from(found.as_ref().is_some_and(|f| f.local));
        let rhs_lua = found.as_ref().map_or(LUA_NOREF, |f| f.rhs_lua);
        let rhs = found.as_ref().map_or(::core::ptr::null_mut(), |f| f.rhs);
        if !get_dict {
            if !rhs.is_null() {
                if *rhs as ::core::ffi::c_int == NUL {
                    (*rettv).vval.v_string =
                        xstrdup(b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char);
                } else {
                    (*rettv).vval.v_string = str2special_save(rhs, false_0 != 0, false_0 != 0);
                }
            } else if rhs_lua != LUA_NOREF {
                (*rettv).vval.v_string =
                    nlua_funcref_str((*mp).m_luaref, ::core::ptr::null_mut::<Arena>());
            }
        } else if !mp.is_null() && (!rhs.is_null() || rhs_lua != LUA_NOREF) {
            let mut arena: Arena = ARENA_EMPTY;
            let mut dict: Dict = mapblock_fill_dict(
                mp,
                if did_simplify as ::core::ffi::c_int != 0 {
                    keys_simplified
                } else {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                },
                buffer_local,
                abbr,
                true_0 != 0,
                &raw mut arena,
            );
            let mut c2rust_lvalue: Object = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: dict },
            };
            object_to_vim_take_luaref(
                &raw mut c2rust_lvalue,
                rettv,
                true_0 != 0,
                ::core::ptr::null_mut::<Error>(),
            );
            arena_mem_free(arena_finish(&raw mut arena));
        } else {
            tv_dict_alloc_ret(rettv);
        }
        xfree(keys_buf as *mut ::core::ffi::c_void);
        xfree(alt_keys_buf as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn f_maplist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let flags: ::core::ffi::c_int =
            REPTERM_FROM_PART as ::core::ffi::c_int | REPTERM_DO_LT as ::core::ffi::c_int;
        let abbr: bool = (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type
            as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && tv_get_bool(argvars.offset(0 as ::core::ffi::c_int as isize)) != 0;
        tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
        let mut buffer_local: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while buffer_local <= 1 as ::core::ffi::c_int {
            let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while hash < 256 as ::core::ffi::c_int {
                let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
                if abbr {
                    if hash > 0 as ::core::ffi::c_int {
                        break;
                    }
                    if buffer_local != 0 {
                        mp = (*curbuf.get()).b_first_abbr;
                    } else {
                        mp = FIRST_ABBR.get();
                    }
                } else if buffer_local != 0 {
                    mp = (*curbuf.get()).b_maphash[hash as usize] as *mut mapblock_T;
                } else {
                    mp = (*MAPHASH.ptr())[hash as usize] as *mut mapblock_T;
                }
                while !mp.is_null() {
                    if (*mp).m_simplified == 0 {
                        let mut keys_buf: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        let mut did_simplify: bool = false_0 != 0;
                        let mut arena: Arena = ARENA_EMPTY;
                        let mut lhs: *mut ::core::ffi::c_char = str2special_arena(
                            (*mp).m_keys,
                            true_0 != 0,
                            false_0 != 0,
                            &raw mut arena,
                        );
                        replace_termcodes(
                            lhs,
                            strlen(lhs),
                            &raw mut keys_buf,
                            0 as scid_T,
                            flags,
                            &raw mut did_simplify,
                            p_cpo.get(),
                        );
                        let mut dict: Dict = mapblock_fill_dict(
                            mp,
                            if did_simplify as ::core::ffi::c_int != 0 {
                                keys_buf
                            } else {
                                ::core::ptr::null_mut::<::core::ffi::c_char>()
                            },
                            buffer_local,
                            abbr,
                            true_0 != 0,
                            &raw mut arena,
                        );
                        let mut d: typval_T = typval_T {
                            v_type: VAR_UNKNOWN,
                            v_lock: VAR_UNLOCKED,
                            vval: typval_vval_union { v_number: 0 },
                        };
                        let mut c2rust_lvalue: Object = object {
                            type_0: kObjectTypeDict,
                            data: C2Rust_Unnamed { dict: dict },
                        };
                        object_to_vim_take_luaref(
                            &raw mut c2rust_lvalue,
                            &raw mut d,
                            true_0 != 0,
                            ::core::ptr::null_mut::<Error>(),
                        );
                        '_c2rust_label: {
                            if d.v_type as ::core::ffi::c_uint
                                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                            } else {
                                __assert_fail(
                                    b"d.v_type == VAR_DICT\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/mapping.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                    2431 as ::core::ffi::c_uint,
                                    b"void f_maplist(typval_T *, typval_T *, EvalFuncData)\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        tv_list_append_dict((*rettv).vval.v_list, d.vval.v_dict);
                        arena_mem_free(arena_finish(&raw mut arena));
                        xfree(keys_buf as *mut ::core::ffi::c_void);
                    }
                    mp = (*mp).m_next;
                }
                hash += 1;
            }
            buffer_local += 1;
        }
    }
}

pub unsafe extern "C" fn f_maparg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        get_maparg(argvars, rettv, true_0);
    }
}

pub unsafe extern "C" fn f_mapcheck(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        get_maparg(argvars, rettv, false_0);
    }
}

pub unsafe extern "C" fn keymap_array(
    mut mode: String_0,
    mut buf: *mut buf_T,
    mut arena: *mut Arena,
) -> Array {
    unsafe {
        let mut mappings: ArrayBuilder = ArrayBuilder {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
            init_array: [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 16],
        };
        mappings.capacity = ::core::mem::size_of::<[Object; 16]>()
            .wrapping_div(::core::mem::size_of::<Object>())
            .wrapping_div(
                (::core::mem::size_of::<[Object; 16]>()
                    .wrapping_rem(::core::mem::size_of::<Object>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as size_t;
        mappings.size = 0 as size_t;
        mappings.items = &raw mut mappings.init_array as *mut Object;
        let mut p: *mut ::core::ffi::c_char = (if mode.size > 0 as size_t {
            mode.data as *const ::core::ffi::c_char
        } else {
            b"m\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char;
        let mut forceit: bool = *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int;
        let mut int_mode: ::core::ffi::c_int = get_map_mode(&raw mut p, forceit);
        if forceit {
            '_c2rust_label: {
                if p == mode.data {
                } else {
                    __assert_fail(
                        b"p == mode.data\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/mapping.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2888 as ::core::ffi::c_uint,
                        b"Array keymap_array(String, buf_T *, Arena *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            p = p.offset(1);
        }
        let mut is_abbrev: bool = int_mode & (MODE_INSERT | MODE_CMDLINE)
            != 0 as ::core::ffi::c_int
            && *p as ::core::ffi::c_int == 'a' as ::core::ffi::c_int;
        let mut buffer_value: ::core::ffi::c_int = if buf.is_null() {
            0 as ::core::ffi::c_int
        } else {
            (*buf).handle as ::core::ffi::c_int
        };
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i
            < (if is_abbrev as ::core::ffi::c_int != 0 {
                1 as ::core::ffi::c_int
            } else {
                MAX_MAPHASH as ::core::ffi::c_int
            })
        {
            let mut current_maphash: *const mapblock_T = if is_abbrev as ::core::ffi::c_int != 0 {
                if !buf.is_null() {
                    (*buf).b_first_abbr
                } else {
                    FIRST_ABBR.get()
                }
            } else if !buf.is_null() {
                (*buf).b_maphash[i as usize] as *mut mapblock_T
            } else {
                (*MAPHASH.ptr())[i as usize] as *mut mapblock_T
            };
            while !current_maphash.is_null() {
                if (*current_maphash).m_simplified == 0 {
                    if int_mode & (*current_maphash).m_mode != 0 {
                        if mappings.size == mappings.capacity {
                            mappings.capacity = if mappings.capacity << 1 as ::core::ffi::c_int
                                > ::core::mem::size_of::<[Object; 16]>()
                                    .wrapping_div(::core::mem::size_of::<Object>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[Object; 16]>()
                                            .wrapping_rem(::core::mem::size_of::<Object>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ) {
                                mappings.capacity << 1 as ::core::ffi::c_int
                            } else {
                                ::core::mem::size_of::<[Object; 16]>()
                                    .wrapping_div(::core::mem::size_of::<Object>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[Object; 16]>()
                                            .wrapping_rem(::core::mem::size_of::<Object>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as size_t,
                                    )
                            };
                            mappings.items = (if mappings.capacity
                                == ::core::mem::size_of::<[Object; 16]>()
                                    .wrapping_div(::core::mem::size_of::<Object>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[Object; 16]>()
                                            .wrapping_rem(::core::mem::size_of::<Object>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ) {
                                if mappings.items == &raw mut mappings.init_array as *mut Object {
                                    mappings.items as *mut ::core::ffi::c_void
                                } else {
                                    _memcpy_free(
                                        &raw mut mappings.init_array as *mut Object
                                            as *mut ::core::ffi::c_void,
                                        mappings.items as *mut ::core::ffi::c_void,
                                        mappings
                                            .size
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    )
                                }
                            } else {
                                if mappings.items == &raw mut mappings.init_array as *mut Object {
                                    memcpy(
                                        xmalloc(
                                            mappings
                                                .capacity
                                                .wrapping_mul(::core::mem::size_of::<Object>()),
                                        ),
                                        mappings.items as *const ::core::ffi::c_void,
                                        mappings
                                            .size
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    )
                                } else {
                                    xrealloc(
                                        mappings.items as *mut ::core::ffi::c_void,
                                        mappings
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    )
                                }
                            }) as *mut Object;
                        } else {
                        };
                        let c2rust_fresh41 = mappings.size;
                        mappings.size = mappings.size.wrapping_add(1);
                        *mappings.items.offset(c2rust_fresh41 as isize) = object {
                            type_0: kObjectTypeDict,
                            data: C2Rust_Unnamed {
                                dict: mapblock_fill_dict(
                                    current_maphash,
                                    if !(*current_maphash).m_alt.is_null() {
                                        (*(*current_maphash).m_alt).m_keys
                                    } else {
                                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                                    },
                                    buffer_value,
                                    is_abbrev,
                                    false,
                                    arena,
                                ),
                            },
                        };
                    }
                }
                current_maphash = (*current_maphash).m_next;
            }
            i += 1;
        }
        return arena_take_arraybuilder(arena, &raw mut mappings);
    }
}
