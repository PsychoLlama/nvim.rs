//! Serialisation: the `msgpack*()` and `json_*()` families.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_json_decode(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !encode_vim_list_to_buf(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list,
            &raw mut len,
            &raw mut tofree,
        ) {
            emsg(gettext(
                b"E474: Failed to convert list to string\0".as_ptr() as *const ::core::ffi::c_char,
            ));
            return;
        }
        s = tofree;
        if s.is_null() {
            '_c2rust_label: {
                if len == 0 as size_t {
                } else {
                    __assert_fail(
                        b"len == 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3859 as ::core::ffi::c_uint,
                        b"void f_json_decode(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            s = b"\0".as_ptr() as *const ::core::ffi::c_char;
        }
    } else {
        s = tv_get_string_buf_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut numbuf as *mut ::core::ffi::c_char,
        );
        if !s.is_null() {
            len = strlen(s);
        } else {
            return;
        }
    }
    if json_decode_string(s, len, rettv) == FAIL {
        semsg(
            gettext(b"E474: Failed to parse %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            len as ::core::ffi::c_int,
            s,
        );
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
    '_c2rust_label_0: {
        if (*rettv).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"rettv->v_type != VAR_UNKNOWN\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3875 as ::core::ffi::c_uint,
                b"void f_json_decode(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    xfree(tofree as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn f_json_encode(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = encode_tv2json(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<size_t>(),
    );
}
pub unsafe extern "C" fn f_msgpackdump(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_listarg as *const ::core::ffi::c_char),
            b"msgpackdump()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let list: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let mut packer: PackerBuffer = packer_string_buffer();
    let msg: *const ::core::ffi::c_char =
        gettext(b"msgpackdump() argument, index %i\0".as_ptr() as *const ::core::ffi::c_char);
    let mut msgbuf: [::core::ffi::c_char; 189] = [0; 189];
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *mut list_T = list;
    if !l_.is_null() {
        let mut li: *mut listitem_T = (*l_).lv_first;
        while !li.is_null() {
            vim_snprintf(
                &raw mut msgbuf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 189]>(),
                msg,
                idx,
            );
            idx += 1;
            if encode_vim_to_msgpack(
                &raw mut packer,
                &raw mut (*li).li_tv,
                &raw mut msgbuf as *mut ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                break;
            }
            li = (*li).li_next;
        }
    }
    let mut data: String_0 = packer_take_string(&mut packer);
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && strequal(
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
            b"B\0".as_ptr() as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int
            != 0
    {
        let mut b: *mut blob_T = tv_blob_alloc_ret(rettv);
        (*b).bv_ga.ga_data = data.data as *mut ::core::ffi::c_void;
        (*b).bv_ga.ga_len = data.size as ::core::ffi::c_int;
        (*b).bv_ga.ga_maxlen = packer.endptr.offset_from(packer.startptr) as ::core::ffi::c_int;
    } else {
        encode_list_write(
            tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t)
                as *mut ::core::ffi::c_void,
            data.data,
            data.size,
        );
        api_free_string(data);
    };
}
unsafe extern "C" fn emsg_mpack_error(mut status: ::core::ffi::c_int) {
    match status {
        2 => {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"Failed to parse msgpack string\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        1 => {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"Incomplete msgpack string\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        3 => {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"object was too deep to unpack\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        _ => {}
    };
}
unsafe extern "C" fn msgpackparse_unpack_list(list: *const list_T, ret_list: *mut list_T) {
    if tv_list_len(list) == 0 as ::core::ffi::c_int {
        return;
    }
    if (*tv_list_first(list)).li_tv.v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"List item is not a string\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut lrstate: ListReaderState = encode_init_lrstate(list);
    let mut buf: *mut ::core::ffi::c_char = alloc_block() as *mut ::core::ffi::c_char;
    let mut buf_size: size_t = 0 as size_t;
    let mut cur_item: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
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
                data: C2Rust_Unnamed_14 {
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
                data: C2Rust_Unnamed_14 {
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
    parser.data.p = &raw mut cur_item as *mut ::core::ffi::c_void;
    let mut status: ::core::ffi::c_int = MPACK_OK as ::core::ffi::c_int;
    '_end: {
        loop {
            let mut read_bytes: size_t = 0;
            let rlret: ::core::ffi::c_int = encode_read_from_list(
                &raw mut lrstate,
                buf.offset(buf_size as isize),
                (ARENA_BLOCK_SIZE as size_t).wrapping_sub(buf_size),
                &raw mut read_bytes,
            );
            if rlret == FAIL {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    b"List item is not a string\0".as_ptr() as *const ::core::ffi::c_char,
                );
                break '_end;
            } else {
                buf_size = buf_size.wrapping_add(read_bytes);
                let mut ptr: *const ::core::ffi::c_char = buf;
                while buf_size != 0 {
                    status = mpack_parse_typval(&raw mut parser, &raw mut ptr, &raw mut buf_size);
                    if status != MPACK_OK as ::core::ffi::c_int {
                        break;
                    }
                    tv_list_append_owned_tv(ret_list, cur_item);
                    cur_item.v_type = VAR_UNKNOWN;
                }
                if rlret == OK {
                    break;
                }
                if status == MPACK_EOF as ::core::ffi::c_int {
                    if buf_size != 0 && ptr > buf as *const ::core::ffi::c_char {
                        memmove(
                            buf as *mut ::core::ffi::c_void,
                            ptr as *const ::core::ffi::c_void,
                            buf_size,
                        );
                    }
                } else if status != MPACK_OK as ::core::ffi::c_int {
                    break;
                }
            }
        }
        if status != MPACK_OK as ::core::ffi::c_int {
            typval_parser_error_free(&raw mut parser);
            emsg_mpack_error(status);
        }
    }
    free_block(buf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn msgpackparse_unpack_blob(blob: *const blob_T, ret_list: *mut list_T) {
    let len: ::core::ffi::c_int = tv_blob_len(blob);
    if len == 0 as ::core::ffi::c_int {
        return;
    }
    let mut data: *const ::core::ffi::c_char = (*blob).bv_ga.ga_data as *const ::core::ffi::c_char;
    let mut remaining: size_t = len as size_t;
    while remaining != 0 {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut status: ::core::ffi::c_int =
            unpack_typval(&raw mut data, &raw mut remaining, &raw mut tv);
        if status != MPACK_OK as ::core::ffi::c_int {
            emsg_mpack_error(status);
            return;
        }
        tv_list_append_owned_tv(ret_list, tv);
    }
}
pub unsafe extern "C" fn f_msgpackparse(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_listblobarg as *const ::core::ffi::c_char),
            b"msgpackparse()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let ret_list: *mut list_T =
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        msgpackparse_unpack_list(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list,
            ret_list,
        );
    } else {
        msgpackparse_unpack_blob(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_blob,
            ret_list,
        );
    };
}
