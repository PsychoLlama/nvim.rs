//! What `nvim_get_option_info` reports.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_vimoption(
    mut name: String_0,
    mut opt_flags: c_int,
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut opt_idx: OptIndex = find_option_len(name.data, name.size);
    if !(opt_idx as c_int != kOptInvalid as c_int) {
        api_err_invalid(
            err,
            b"option (not found)\0".as_ptr() as *const c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    return vimoption2dict(
        (options.ptr() as *mut vimoption_T).offset(opt_idx as isize),
        opt_flags,
        buf,
        win,
        arena,
    );
}

pub unsafe extern "C" fn get_all_vimoptions(mut arena: *mut Arena) -> Dict {
    let mut retval: Dict = arena_dict(arena, kOptCount as size_t);
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        let mut opt_dict: Dict = vimoption2dict(
            (options.ptr() as *mut vimoption_T).offset(opt_idx as isize),
            OPT_GLOBAL as c_int,
            curbuf.get(),
            curwin.get(),
            arena,
        );
        let c2rust_fresh27 = retval.size;
        retval.size = retval.size.wrapping_add(1);
        *retval.items.offset(c2rust_fresh27 as isize) = key_value_pair {
            key: cstr_as_string((*options.ptr())[opt_idx as usize].fullname),
            value: object {
                type_0: kObjectTypeDict,
                data: object_data { dict: opt_dict },
            },
        };
        opt_idx += 1;
    }
    return retval;
}

pub(crate) unsafe extern "C" fn vimoption2dict(
    mut opt: *mut vimoption_T,
    mut opt_flags: c_int,
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut arena: *mut Arena,
) -> Dict {
    let mut opt_idx: OptIndex = get_opt_idx(opt);
    let mut dict: Dict = arena_dict(arena, 13 as size_t);
    let c2rust_fresh14 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh14 as isize) = key_value_pair {
        key: cstr_as_string(b"name\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeString,
            data: object_data {
                string: cstr_as_string((*opt).fullname),
            },
        },
    };
    let c2rust_fresh15 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh15 as isize) = key_value_pair {
        key: cstr_as_string(b"shortname\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeString,
            data: object_data {
                string: cstr_as_string((*opt).shortname),
            },
        },
    };
    let mut scope: *const c_char = ::core::ptr::null::<c_char>();
    if option_has_scope(opt_idx, kOptScopeBuf) {
        scope = b"buf\0".as_ptr() as *const c_char;
    } else if option_has_scope(opt_idx, kOptScopeWin) {
        scope = b"win\0".as_ptr() as *const c_char;
    } else {
        scope = b"global\0".as_ptr() as *const c_char;
    }
    let c2rust_fresh16 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh16 as isize) = key_value_pair {
        key: cstr_as_string(b"scope\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeString,
            data: object_data {
                string: cstr_as_string(scope),
            },
        },
    };
    let c2rust_fresh17 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh17 as isize) = key_value_pair {
        key: cstr_as_string(b"global_local\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: object_data {
                boolean: option_is_global_local(opt_idx),
            },
        },
    };
    let c2rust_fresh18 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh18 as isize) = key_value_pair {
        key: cstr_as_string(b"commalist\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: object_data {
                boolean: (*opt).flags & kOptFlagComma as c_int as uint32_t != 0,
            },
        },
    };
    let c2rust_fresh19 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh19 as isize) = key_value_pair {
        key: cstr_as_string(b"flaglist\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: object_data {
                boolean: (*opt).flags & kOptFlagFlagList as c_int as uint32_t != 0,
            },
        },
    };
    let c2rust_fresh20 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh20 as isize) = key_value_pair {
        key: cstr_as_string(b"was_set\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: object_data {
                boolean: (*opt).flags & kOptFlagWasSet as c_int as uint32_t != 0,
            },
        },
    };
    let mut script_ctx: sctx_T = sctx_T {
        sc_sid: 0 as scid_T,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };
    if opt_flags == OPT_GLOBAL as c_int {
        script_ctx = (*opt).script_ctx;
    } else {
        if option_has_scope(opt_idx, kOptScopeBuf) {
            script_ctx =
                (*buf).b_p_script_ctx[(*opt).scope_idx[kOptScopeBuf as c_int as usize] as usize];
        }
        if option_has_scope(opt_idx, kOptScopeWin) {
            script_ctx = (*win).w_onebuf_opt.wo_script_ctx
                [(*opt).scope_idx[kOptScopeWin as c_int as usize] as usize];
        }
        if opt_flags != OPT_LOCAL as c_int && script_ctx.sc_sid == 0 as c_int {
            script_ctx = (*opt).script_ctx;
        }
    }
    let c2rust_fresh21 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh21 as isize) = key_value_pair {
        key: cstr_as_string(b"last_set_sid\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: object_data {
                integer: script_ctx.sc_sid as Integer,
            },
        },
    };
    let c2rust_fresh22 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh22 as isize) = key_value_pair {
        key: cstr_as_string(b"last_set_linenr\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: object_data {
                integer: script_ctx.sc_lnum as Integer,
            },
        },
    };
    let c2rust_fresh23 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh23 as isize) = key_value_pair {
        key: cstr_as_string(b"last_set_chan\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: object_data {
                integer: script_ctx.sc_chan as int64_t,
            },
        },
    };
    let c2rust_fresh24 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh24 as isize) = key_value_pair {
        key: cstr_as_string(b"type\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeString,
            data: object_data {
                string: cstr_as_string(optval_type_get_name(option_get_type(get_opt_idx(opt)))),
            },
        },
    };
    let c2rust_fresh25 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh25 as isize) = key_value_pair {
        key: cstr_as_string(b"default\0".as_ptr() as *const c_char),
        value: optval_as_object((*opt).def_val),
    };
    let c2rust_fresh26 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh26 as isize) = key_value_pair {
        key: cstr_as_string(b"allows_duplicates\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: object_data {
                boolean: (*opt).flags & kOptFlagNoDup as c_int as uint32_t == 0,
            },
        },
    };
    return dict;
}
