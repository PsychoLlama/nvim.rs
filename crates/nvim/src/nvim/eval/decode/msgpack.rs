//! msgpack to `typval_T`: the two parser callbacks and their entry points.
//!
//! `mpack_parse()` drives the byte stream and calls `typval_parse_enter` as
//! each node opens and `typval_parse_exit` as it closes; strings, maps and ext
//! values are the ones that need the exit hook, because they are only complete
//! once their data chunks have arrived.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn positive_integer_to_special_typval(
    mut rettv: *mut typval_T,
    mut val: uint64_t,
) {
    unsafe {
        if val <= VARNUMBER_MAX as uint64_t {
            *rettv = typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union {
                    v_number: val as varnumber_T,
                },
            };
        } else {
            let list: *mut list_T = tv_list_alloc(4 as ptrdiff_t);
            tv_list_ref(list);
            create_special_dict(
                rettv,
                kMPInteger,
                typval_T {
                    v_type: VAR_LIST,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_list: list },
                },
            );
            tv_list_append_number(list, 1 as varnumber_T);
            tv_list_append_number(
                list,
                (val >> 62 as ::core::ffi::c_int & 0x3 as uint64_t) as varnumber_T,
            );
            tv_list_append_number(
                list,
                (val >> 31 as ::core::ffi::c_int & 0x7fffffff as uint64_t) as varnumber_T,
            );
            tv_list_append_number(list, (val & 0x7fffffff as uint64_t) as varnumber_T);
        };
    }
}

unsafe extern "C-unwind" fn typval_parse_enter(
    mut parser: *mut mpack_parser_t,
    mut node: *mut mpack_node_t,
) {
    unsafe {
        let mut result: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
        let mut parent: *mut mpack_node_t = if (*node.offset(-(1 as ::core::ffi::c_int as isize)))
            .pos
            == -1 as ::core::ffi::c_int as size_t
        {
            ::core::ptr::null_mut::<mpack_node_t>()
        } else {
            node.offset(-(1 as ::core::ffi::c_int as isize))
        };
        if !parent.is_null() {
            match (*parent).tok.type_0 as ::core::ffi::c_uint {
                7 => {
                    let mut list: *mut list_T =
                        (*parent).data[1 as ::core::ffi::c_int as usize].p as *mut list_T;
                    result = tv_list_append_owned_tv(
                        list,
                        typval_T {
                            v_type: VAR_UNKNOWN,
                            v_lock: VAR_UNLOCKED,
                            vval: typval_vval_union { v_number: 0 },
                        },
                    );
                }
                8 => {
                    let mut items: *mut [typval_T; 2] =
                        (*parent).data[1 as ::core::ffi::c_int as usize].p as *mut [typval_T; 2];
                    result = (&raw mut *items.offset((*parent).pos as isize) as *mut typval_T)
                        .offset((*parent).key_visited as isize);
                }
                10 | 9 | 11 => {
                    '_c2rust_label: {
                        if (*node).tok.type_0 as ::core::ffi::c_uint
                            == MPACK_TOKEN_CHUNK as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                        } else {
                            __assert_fail(
                                b"node->tok.type == MPACK_TOKEN_CHUNK\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/decode.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                932 as ::core::ffi::c_uint,
                                b"void typval_parse_enter(mpack_parser_t *, mpack_node_t *)\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                }
                _ => {
                    abort();
                }
            }
        } else {
            result = (*parser).data.p as *mut typval_T;
        }
        (*node).data[0 as ::core::ffi::c_int as usize].p = result as *mut ::core::ffi::c_void;
        (*node).data[1 as ::core::ffi::c_int as usize].p = NULL;
        match (*node).tok.type_0 as ::core::ffi::c_uint {
            1 => {
                *result = typval_T {
                    v_type: VAR_SPECIAL,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_special: kSpecialVarNull,
                    },
                };
            }
            2 => {
                *result = typval_T {
                    v_type: VAR_BOOL,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_bool: (if mpack_unpack_boolean((*node).tok) as ::core::ffi::c_int != 0 {
                            kBoolVarTrue as ::core::ffi::c_int
                        } else {
                            kBoolVarFalse as ::core::ffi::c_int
                        }) as BoolVarValue,
                    },
                };
            }
            4 => {
                *result = typval_T {
                    v_type: VAR_NUMBER,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_number: mpack_unpack_sint((*node).tok) as varnumber_T,
                    },
                };
            }
            3 => {
                positive_integer_to_special_typval(
                    result,
                    mpack_unpack_uint((*node).tok) as uint64_t,
                );
            }
            5 => {
                *result = typval_T {
                    v_type: VAR_FLOAT,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_float: mpack_unpack_float_fast((*node).tok),
                    },
                };
            }
            9 | 10 | 11 => {
                (*node).data[1 as ::core::ffi::c_int as usize].p =
                    xmallocz((*node).tok.length as size_t);
            }
            6 => {
                let mut data: *mut ::core::ffi::c_char =
                    (*parent).data[1 as ::core::ffi::c_int as usize].p as *mut ::core::ffi::c_char;
                memcpy(
                    data.offset((*parent).pos as isize) as *mut ::core::ffi::c_void,
                    (*node).tok.data.chunk_ptr as *const ::core::ffi::c_void,
                    (*node).tok.length as size_t,
                );
            }
            7 => {
                let list_0: *mut list_T = tv_list_alloc((*node).tok.length as ptrdiff_t);
                tv_list_ref(list_0);
                *result = typval_T {
                    v_type: VAR_LIST,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_list: list_0 },
                };
                (*node).data[1 as ::core::ffi::c_int as usize].p =
                    list_0 as *mut ::core::ffi::c_void;
            }
            8 => {
                (*node).data[1 as ::core::ffi::c_int as usize].p = xmallocz(
                    ((*node).tok.length.wrapping_mul(2 as mpack_uint32_t) as size_t)
                        .wrapping_mul(::core::mem::size_of::<typval_T>()),
                );
            }
            _ => {}
        };
    }
}

pub unsafe extern "C" fn typval_parser_error_free(mut parser: *mut mpack_parser_t) {
    unsafe {
        let mut i: uint32_t = 0 as uint32_t;
        while i < (*parser).size as uint32_t {
            let mut node: *mut mpack_node_t =
                (&raw mut (*parser).items as *mut mpack_node_t).offset(i as isize);
            match (*node).tok.type_0 as ::core::ffi::c_uint {
                9 | 10 | 11 | 8 => {
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut (*(&raw mut (*node).data as *mut mpack_data_t)
                            .offset(1 as ::core::ffi::c_int as isize))
                        .p;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    let _ = *ptr_;
                }
                _ => {}
            }
            i = i.wrapping_add(1);
        }
    }
}

unsafe extern "C-unwind" fn typval_parse_exit(
    mut _parser: *mut mpack_parser_t,
    mut node: *mut mpack_node_t,
) {
    unsafe {
        let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        let mut result: *mut typval_T =
            (*node).data[0 as ::core::ffi::c_int as usize].p as *mut typval_T;
        's_308: {
            match (*node).tok.type_0 as ::core::ffi::c_uint {
                9 | 10 => {
                    *result = decode_string(
                        (*node).data[1 as ::core::ffi::c_int as usize].p
                            as *const ::core::ffi::c_char,
                        (*node).tok.length as size_t,
                        false_0 != 0,
                        true_0 != 0,
                    );
                    (*node).data[1 as ::core::ffi::c_int as usize].p = NULL;
                }
                11 => {
                    let list: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
                    tv_list_ref(list);
                    tv_list_append_number(list, (*node).tok.data.ext_type as varnumber_T);
                    let ext_val_list: *mut list_T =
                        tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
                    tv_list_append_list(list, ext_val_list);
                    create_special_dict(
                        result,
                        kMPExt,
                        typval_T {
                            v_type: VAR_LIST,
                            v_lock: VAR_UNLOCKED,
                            vval: typval_vval_union { v_list: list },
                        },
                    );
                    encode_list_write(
                        ext_val_list as *mut ::core::ffi::c_void,
                        (*node).data[1 as ::core::ffi::c_int as usize].p
                            as *const ::core::ffi::c_char,
                        (*node).tok.length as size_t,
                    );
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut (*(&raw mut (*node).data as *mut mpack_data_t)
                            .offset(1 as ::core::ffi::c_int as isize))
                        .p;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    let _ = *ptr_;
                }
                8 => {
                    let mut items: *mut [typval_T; 2] =
                        (*node).data[1 as ::core::ffi::c_int as usize].p as *mut [typval_T; 2];
                    let mut i: size_t = 0 as size_t;
                    's_251: {
                        while i < (*node).tok.length as size_t {
                            let mut key: *mut typval_T = (&raw mut *items.offset(i as isize)
                                as *mut typval_T)
                                .offset(0 as ::core::ffi::c_int as isize);
                            if (*key).v_type as ::core::ffi::c_uint
                                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                                || (*key).vval.v_string.is_null()
                                || *(*key)
                                    .vval
                                    .v_string
                                    .offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == NUL
                            {
                                break 's_251;
                            }
                            i = i.wrapping_add(1);
                        }
                        dict = tv_dict_alloc();
                        (*dict).dv_refcount += 1;
                        *result = typval_T {
                            v_type: VAR_DICT,
                            v_lock: VAR_UNLOCKED,
                            vval: typval_vval_union { v_dict: dict },
                        };
                        let mut i_0: size_t = 0 as size_t;
                        while i_0 < (*node).tok.length as size_t {
                            let mut key_0: *mut ::core::ffi::c_char = (*items.offset(i_0 as isize))
                                [0 as ::core::ffi::c_int as usize]
                                .vval
                                .v_string;
                            let mut keylen: size_t = strlen(key_0);
                            let di: *mut dictitem_T =
                                xmallocz((17 as size_t).wrapping_add(keylen)) as *mut dictitem_T;
                            memcpy(
                                (&raw mut (*di).di_key as *mut ::core::ffi::c_char)
                                    .offset(0 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_void,
                                key_0 as *const ::core::ffi::c_void,
                                keylen,
                            );
                            (*di).di_tv.v_type = VAR_UNKNOWN;
                            if tv_dict_add(dict, di) == FAIL {
                                let dhi_ht_: *mut hashtab_T = &raw mut (*dict).dv_hashtab;
                                let mut dhi_todo_: size_t = (*dhi_ht_).ht_used;
                                let mut dhi_: *mut hashitem_T = (*dhi_ht_).ht_array;
                                while dhi_todo_ != 0 {
                                    if !((*dhi_).hi_key.is_null()
                                        || (*dhi_).hi_key
                                            == &raw const hash_removed as *mut ::core::ffi::c_char)
                                    {
                                        dhi_todo_ = dhi_todo_.wrapping_sub(1);
                                        let d: *mut dictitem_T = (*dhi_)
                                            .hi_key
                                            .offset(-(17 as ::core::ffi::c_ulong as isize))
                                            as *mut dictitem_T;
                                        (*d).di_tv.v_type = VAR_SPECIAL;
                                        (*d).di_tv.vval.v_special = kSpecialVarNull;
                                    }
                                    dhi_ = dhi_.offset(1);
                                }
                                tv_clear(result);
                                xfree(di as *mut ::core::ffi::c_void);
                                break 's_251;
                            } else {
                                (*di).di_tv =
                                    (*items.offset(i_0 as isize))[1 as ::core::ffi::c_int as usize];
                                i_0 = i_0.wrapping_add(1);
                            }
                        }
                        let mut i_1: size_t = 0 as size_t;
                        while i_1 < (*node).tok.length as size_t {
                            xfree(
                                (*items.offset(i_1 as isize))[0 as ::core::ffi::c_int as usize]
                                    .vval
                                    .v_string
                                    as *mut ::core::ffi::c_void,
                            );
                            i_1 = i_1.wrapping_add(1);
                        }
                        let mut ptr__0: *mut *mut ::core::ffi::c_void =
                            &raw mut (*(&raw mut (*node).data as *mut mpack_data_t)
                                .offset(1 as ::core::ffi::c_int as isize))
                            .p;
                        xfree(*ptr__0);
                        *ptr__0 = NULL;
                        let _ = *ptr__0;
                        break 's_308;
                    }
                    let list_0: *mut list_T =
                        decode_create_map_special_dict(result, (*node).tok.length as ptrdiff_t);
                    let mut i_2: size_t = 0 as size_t;
                    while i_2 < (*node).tok.length as size_t {
                        let kv_pair: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
                        tv_list_append_list(list_0, kv_pair);
                        tv_list_append_owned_tv(
                            kv_pair,
                            (*items.offset(i_2 as isize))[0 as ::core::ffi::c_int as usize],
                        );
                        tv_list_append_owned_tv(
                            kv_pair,
                            (*items.offset(i_2 as isize))[1 as ::core::ffi::c_int as usize],
                        );
                        i_2 = i_2.wrapping_add(1);
                    }
                    let mut ptr__1: *mut *mut ::core::ffi::c_void =
                        &raw mut (*(&raw mut (*node).data as *mut mpack_data_t)
                            .offset(1 as ::core::ffi::c_int as isize))
                        .p;
                    xfree(*ptr__1);
                    *ptr__1 = NULL;
                    let _ = *ptr__1;
                }
                _ => {}
            }
        };
    }
}

pub unsafe extern "C" fn mpack_parse_typval(
    mut parser: *mut mpack_parser_t,
    mut data: *mut *const ::core::ffi::c_char,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    unsafe {
        return mpack_parse(
            parser,
            data,
            size,
            Some(
                typval_parse_enter
                    as unsafe extern "C-unwind" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
            ),
            Some(
                typval_parse_exit
                    as unsafe extern "C-unwind" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
            ),
        );
    }
}

pub unsafe extern "C" fn unpack_typval(
    mut data: *mut *const ::core::ffi::c_char,
    mut size: *mut size_t,
    mut ret: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        (*ret).v_type = VAR_UNKNOWN;
        let mut parser: mpack_parser_t = mpack_parser_t {
            data: mpack_data_t {
                p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
            size: 0,
            capacity: 0,
            status: 0,
            exiting: 0,
            tokbuf: mpack_tokbuf_t {
                pending: [0; 9],
                pending_tok: mpack_token_t {
                    type_0: 0 as mpack_token_type_t,
                    length: 0,
                    data: C2Rust_Unnamed_0 {
                        value: mpack_value_t { lo: 0, hi: 0 },
                    },
                },
                ppos: 0,
                plen: 0,
                passthrough: 0,
            },
            items: [mpack_node_t {
                tok: mpack_token_t {
                    type_0: 0 as mpack_token_type_t,
                    length: 0,
                    data: C2Rust_Unnamed_0 {
                        value: mpack_value_t { lo: 0, hi: 0 },
                    },
                },
                pos: 0,
                key_visited: 0,
                data: [mpack_data_t {
                    p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                }; 2],
            }; 33],
        };
        mpack_parser_init(&raw mut parser, 0 as mpack_uint32_t);
        parser.data.p = ret as *mut ::core::ffi::c_void;
        let mut status: ::core::ffi::c_int = mpack_parse_typval(&raw mut parser, data, size);
        if status != MPACK_OK as ::core::ffi::c_int {
            typval_parser_error_free(&raw mut parser);
            tv_clear(ret);
        }
        return status;
    }
}
