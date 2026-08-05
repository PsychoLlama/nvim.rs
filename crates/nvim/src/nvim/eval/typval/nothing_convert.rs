//! The `nothing` sink's per-value switch, as `typval_encode.c.h` expands it.
//!
//! One function, and it is a macro expansion rather than code: the same
//! algorithm the msgpack, json, echo, string, lua and object sinks each
//! carry their own copy of.  Left exactly as transpiled — collapsing the
//! seven copies onto one walker is a later slice's job.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn _typval_encode_nothing_convert_one_value(
    ignored: *const ::core::ffi::c_void,
    mpstack: *mut MPConvStack,
    _cur_mpsv: *mut MPConvStackVal,
    tv: *mut typval_T,
    copyID: ::core::ffi::c_int,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        '_typval_encode_stop_converting_one_item: {
            match (*tv).v_type as ::core::ffi::c_uint {
                2 => {
                    xfree((*tv).vval.v_string as *mut ::core::ffi::c_void);
                    (*tv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    (*tv).v_lock = VAR_UNLOCKED;
                }
                1 => {
                    (*tv).vval.v_number = 0 as varnumber_T;
                    (*tv).v_lock = VAR_UNLOCKED;
                }
                6 => {
                    (*tv).vval.v_float = 0 as ::core::ffi::c_int as float_T;
                    (*tv).v_lock = VAR_UNLOCKED;
                }
                10 => {
                    tv_blob_unref((*tv).vval.v_blob);
                    (*tv).vval.v_blob = ::core::ptr::null_mut::<blob_T>();
                    (*tv).v_lock = VAR_UNLOCKED;
                }
                3 => {
                    if _nothing_conv_func_start(tv, (*tv).vval.v_string) != NOTDONE {
                        return OK;
                    }
                    _nothing_conv_func_end(tv, copyID);
                }
                9 => {
                    let pt: *mut partial_T = (*tv).vval.v_partial;
                    let fun: *mut ::core::ffi::c_char = if pt.is_null() {
                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                    } else {
                        partial_name(pt)
                    };
                    let _prefix: *const ::core::ffi::c_char = if !fun.is_null()
                        && !pt.is_null()
                        && (*pt).pt_name.is_null()
                        && (*fun.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'A' as ::core::ffi::c_uint
                            && *fun.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'Z' as ::core::ffi::c_uint)
                    {
                        b"g:\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    };
                    if _nothing_conv_func_start(tv, fun) != NOTDONE {
                        return OK;
                    }
                    if (*mpstack).size == (*mpstack).capacity {
                        (*mpstack).capacity = if (*mpstack).capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*mpstack).capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*mpstack).items = (if (*mpstack).capacity
                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                (*mpstack).items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                        as *mut ::core::ffi::c_void,
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        } else {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                memcpy(
                                    xmalloc(
                                        (*mpstack)
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    ),
                                    (*mpstack).items as *const ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            } else {
                                xrealloc(
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        }) as *mut MPConvStackVal;
                    } else {
                    };
                    let c2rust_fresh3 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh3 as isize) = MPConvStackVal {
                        type_0: kMPConvPartial,
                        tv: tv,
                        saved_copyID: copyID - 1 as ::core::ffi::c_int,
                        data: C2Rust_Unnamed_18 {
                            p: C2Rust_Unnamed_20 {
                                stage: kMPConvPartialArgs,
                                pt: (*tv).vval.v_partial,
                            },
                        },
                    };
                }
                4 => {
                    if (*tv).vval.v_list.is_null()
                        || tv_list_len((*tv).vval.v_list) == 0 as ::core::ffi::c_int
                    {
                        tv_list_unref((*tv).vval.v_list);
                        (*tv).vval.v_list = ::core::ptr::null_mut::<list_T>();
                        (*tv).v_lock = VAR_UNLOCKED;
                    } else {
                        let saved_copyID: ::core::ffi::c_int = tv_list_copyid((*tv).vval.v_list);
                        let te_csr_ret: ::core::ffi::c_int =
                            _typval_encode_nothing_check_self_reference(
                                ignored,
                                (*tv).vval.v_list as *mut ::core::ffi::c_void,
                                &raw mut (*(*tv).vval.v_list).lv_copyID,
                                mpstack,
                                copyID,
                                kMPConvList,
                                objname,
                            );
                        if te_csr_ret != NOTDONE {
                            return te_csr_ret;
                        }
                        '_c2rust_label: {
                            if saved_copyID != copyID {
                            } else {
                                __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/typval.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                383 as ::core::ffi::c_uint,
                                b"int _typval_encode_nothing_convert_one_value(const void *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                            }
                        };
                        if (*mpstack).size == (*mpstack).capacity {
                            (*mpstack).capacity = if (*mpstack).capacity << 1 as ::core::ffi::c_int
                                > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                            .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ) {
                                (*mpstack).capacity << 1 as ::core::ffi::c_int
                            } else {
                                ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                            .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as size_t,
                                    )
                            };
                            (*mpstack).items = (if (*mpstack).capacity
                                == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                            .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ) {
                                if (*mpstack).items
                                    == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                {
                                    (*mpstack).items as *mut ::core::ffi::c_void
                                } else {
                                    _memcpy_free(
                                        &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                            as *mut ::core::ffi::c_void,
                                        (*mpstack).items as *mut ::core::ffi::c_void,
                                        (*mpstack)
                                            .size
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    )
                                }
                            } else {
                                if (*mpstack).items
                                    == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                {
                                    memcpy(
                                        xmalloc(
                                            (*mpstack).capacity.wrapping_mul(
                                                ::core::mem::size_of::<MPConvStackVal>(),
                                            ),
                                        ),
                                        (*mpstack).items as *const ::core::ffi::c_void,
                                        (*mpstack)
                                            .size
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    )
                                } else {
                                    xrealloc(
                                        (*mpstack).items as *mut ::core::ffi::c_void,
                                        (*mpstack)
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    )
                                }
                            })
                                as *mut MPConvStackVal;
                        } else {
                        };
                        let c2rust_fresh4 = (*mpstack).size;
                        (*mpstack).size = (*mpstack).size.wrapping_add(1);
                        *(*mpstack).items.offset(c2rust_fresh4 as isize) = MPConvStackVal {
                            type_0: kMPConvList,
                            tv: tv,
                            saved_copyID: saved_copyID,
                            data: C2Rust_Unnamed_18 {
                                l: C2Rust_Unnamed_21 {
                                    list: (*tv).vval.v_list,
                                    li: tv_list_first((*tv).vval.v_list),
                                },
                            },
                        };
                        let _ = _nothing_conv_real_list_after_start(
                            tv,
                            (*mpstack).items.offset(
                                (*mpstack)
                                    .size
                                    .wrapping_sub(0 as size_t)
                                    .wrapping_sub(1 as size_t)
                                    as isize,
                            ),
                        ) != NOTDONE;
                    }
                }
                7 => match (*tv).vval.v_bool as ::core::ffi::c_uint {
                    1 | 0 => {
                        (*tv).vval.v_bool = kBoolVarFalse;
                        (*tv).v_lock = VAR_UNLOCKED;
                    }
                    _ => {}
                },
                8 => match (*tv).vval.v_special as ::core::ffi::c_uint {
                    0 => {
                        (*tv).vval.v_special = kSpecialVarNull;
                        (*tv).v_lock = VAR_UNLOCKED;
                    }
                    _ => {}
                },
                5 => {
                    if (*tv).vval.v_dict.is_null()
                        || (*(*tv).vval.v_dict).dv_hashtab.ht_used == 0 as size_t
                    {
                        '_c2rust_label_0: {
                            if &raw mut (*tv).vval.v_dict as *mut ::core::ffi::c_void
                                != (_typval_encode_nothing_nodict_var.ptr() as *const _)
                                    as *mut ::core::ffi::c_void
                            {
                            } else {
                                __assert_fail(
                                b"(void *)&(tv->vval.v_dict) != (void *)&TYPVAL_ENCODE_NODICT_VAR\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/eval/typval.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                416 as ::core::ffi::c_uint,
                                b"int _typval_encode_nothing_convert_one_value(const void *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                            }
                        };
                        _nothing_conv_empty_dict(tv, &raw mut (*tv).vval.v_dict);
                    } else {
                        let mut type_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                        let mut val_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                        's_771: {
                            if TYPVAL_ENCODE_ALLOW_SPECIALS != 0
                                && (*(*tv).vval.v_dict).dv_hashtab.ht_used == 2 as size_t
                                && {
                                    type_di = tv_dict_find(
                                        (*tv).vval.v_dict,
                                        b"_TYPE\0".as_ptr() as *const ::core::ffi::c_char,
                                        ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                                            .wrapping_sub(1 as usize)
                                            as ptrdiff_t,
                                    );
                                    !type_di.is_null()
                                }
                                && (*type_di).di_tv.v_type as ::core::ffi::c_uint
                                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                && {
                                    val_di = tv_dict_find(
                                        (*tv).vval.v_dict,
                                        b"_VAL\0".as_ptr() as *const ::core::ffi::c_char,
                                        ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                            .wrapping_sub(1 as usize)
                                            as ptrdiff_t,
                                    );
                                    !val_di.is_null()
                                }
                            {
                                let mut i: size_t = 0;
                                i = 0 as size_t;
                                while i < ::core::mem::size_of::<[*const list_T; 8]>()
                                    .wrapping_div(::core::mem::size_of::<*const list_T>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[*const list_T; 8]>()
                                            .wrapping_rem(::core::mem::size_of::<*const list_T>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    )
                                {
                                    if (*type_di).di_tv.vval.v_list
                                        == (*eval_msgpack_type_lists.ptr())[i as usize]
                                            as *mut list_T
                                    {
                                        break;
                                    }
                                    i = i.wrapping_add(1);
                                }
                                if i != ::core::mem::size_of::<[*const list_T; 8]>()
                                    .wrapping_div(::core::mem::size_of::<*const list_T>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[*const list_T; 8]>()
                                            .wrapping_rem(::core::mem::size_of::<*const list_T>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    )
                                {
                                    match i as MessagePackType as ::core::ffi::c_uint {
                                        0 => {
                                            (*tv).vval.v_special = kSpecialVarNull;
                                            (*tv).v_lock = VAR_UNLOCKED;
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                        1 => {
                                            if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                                == VAR_NUMBER as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*tv).vval.v_bool = kBoolVarFalse;
                                                (*tv).v_lock = VAR_UNLOCKED;
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                        2 => {
                                            let mut val_list: *const list_T =
                                                ::core::ptr::null::<list_T>();
                                            let mut sign: varnumber_T = 0;
                                            let mut highest_bits: varnumber_T = 0;
                                            let mut high_bits: varnumber_T = 0;
                                            let mut low_bits: varnumber_T = 0;
                                            if !((*val_di).di_tv.v_type as ::core::ffi::c_uint
                                                != VAR_LIST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                || {
                                                    val_list = (*val_di).di_tv.vval.v_list;
                                                    tv_list_len(val_list) != 4 as ::core::ffi::c_int
                                                })
                                            {
                                                let sign_li: *const listitem_T =
                                                    tv_list_first(val_list);
                                                if !((*sign_li).li_tv.v_type as ::core::ffi::c_uint
                                                    != VAR_NUMBER as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                    || {
                                                        sign = (*sign_li).li_tv.vval.v_number;
                                                        sign == 0 as varnumber_T
                                                    })
                                                {
                                                    let highest_bits_li: *const listitem_T =
                                                        (*sign_li).li_next;
                                                    if !((*highest_bits_li).li_tv.v_type
                                                        as ::core::ffi::c_uint
                                                        != VAR_NUMBER as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                        || {
                                                            highest_bits = (*highest_bits_li)
                                                                .li_tv
                                                                .vval
                                                                .v_number;
                                                            highest_bits < 0 as varnumber_T
                                                        })
                                                    {
                                                        let high_bits_li: *const listitem_T =
                                                            (*highest_bits_li).li_next;
                                                        if !((*high_bits_li).li_tv.v_type
                                                            as ::core::ffi::c_uint
                                                            != VAR_NUMBER as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                            || {
                                                                high_bits = (*high_bits_li)
                                                                    .li_tv
                                                                    .vval
                                                                    .v_number;
                                                                high_bits < 0 as varnumber_T
                                                            })
                                                        {
                                                            let low_bits_li: *const listitem_T =
                                                                tv_list_last(val_list);
                                                            if !((*low_bits_li).li_tv.v_type
                                                                as ::core::ffi::c_uint
                                                                != VAR_NUMBER as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                || {
                                                                    low_bits = (*low_bits_li)
                                                                        .li_tv
                                                                        .vval
                                                                        .v_number;
                                                                    low_bits < 0 as varnumber_T
                                                                })
                                                            {
                                                                let number: uint64_t = (highest_bits
                                                                    as uint64_t)
                                                                    << 62 as ::core::ffi::c_int
                                                                    | (high_bits as uint64_t)
                                                                        << 31 as ::core::ffi::c_int
                                                                    | low_bits as uint64_t;
                                                                if sign <= 0 as varnumber_T {
                                                                    let _ = number.wrapping_neg();
                                                                    (*tv).vval.v_number =
                                                                        0 as varnumber_T;
                                                                    (*tv).v_lock = VAR_UNLOCKED;
                                                                }
                                                                break '_typval_encode_stop_converting_one_item;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        3 => {
                                            if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                                == VAR_FLOAT as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                (*tv).vval.v_float =
                                                    0 as ::core::ffi::c_int as float_T;
                                                (*tv).v_lock = VAR_UNLOCKED;
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                        4 => {
                                            if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                                == VAR_LIST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                let mut len: size_t = 0;
                                                let mut buf: *mut ::core::ffi::c_char =
                                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                                if encode_vim_list_to_buf(
                                                    (*val_di).di_tv.vval.v_list,
                                                    &raw mut len,
                                                    &raw mut buf,
                                                ) {
                                                    xfree(buf as *mut ::core::ffi::c_void);
                                                    break '_typval_encode_stop_converting_one_item;
                                                }
                                            }
                                        }
                                        5 => {
                                            if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                                == VAR_LIST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                let saved_copyID_0: ::core::ffi::c_int =
                                                    tv_list_copyid((*val_di).di_tv.vval.v_list);
                                                let te_csr_ret_0: ::core::ffi::c_int =
                                                    _typval_encode_nothing_check_self_reference(
                                                        ignored,
                                                        (*val_di).di_tv.vval.v_list
                                                            as *mut ::core::ffi::c_void,
                                                        &raw mut (*(*val_di).di_tv.vval.v_list)
                                                            .lv_copyID,
                                                        mpstack,
                                                        copyID,
                                                        kMPConvList,
                                                        objname,
                                                    );
                                                if te_csr_ret_0 != NOTDONE {
                                                    return te_csr_ret_0;
                                                }
                                                '_c2rust_label_1: {
                                                    if saved_copyID_0 != copyID
                                                        && saved_copyID_0
                                                            != copyID - 1 as ::core::ffi::c_int
                                                    {
                                                    } else {
                                                        __assert_fail(
                                                        b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        b"src/nvim/eval/typval.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        532 as ::core::ffi::c_uint,
                                                        b"int _typval_encode_nothing_convert_one_value(const void *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                    }
                                                };
                                                if (*mpstack).size == (*mpstack).capacity {
                                                    (*mpstack).capacity =
                                                        if (*mpstack).capacity
                                                            << 1 as ::core::ffi::c_int
                                                            > ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            )
                                                        {
                                                            (*mpstack).capacity
                                                                << 1 as ::core::ffi::c_int
                                                        } else {
                                                            ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as size_t,
                                                            )
                                                        };
                                                    (*mpstack).items =
                                                        (if (*mpstack).capacity
                                                            == ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            )
                                                        {
                                                            if (*mpstack).items
                                                                == &raw mut (*mpstack).init_array
                                                                    as *mut MPConvStackVal
                                                            {
                                                                (*mpstack).items
                                                                    as *mut ::core::ffi::c_void
                                                            } else {
                                                                _memcpy_free(
                                                                    &raw mut (*mpstack).init_array
                                                                        as *mut MPConvStackVal
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack).items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack).size.wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            MPConvStackVal,
                                                                        >(
                                                                        ),
                                                                    ),
                                                                )
                                                            }
                                                        } else {
                                                            if (*mpstack).items
                                                                == &raw mut (*mpstack).init_array
                                                                    as *mut MPConvStackVal
                                                            {
                                                                memcpy(
                                                            xmalloc(
                                                                (*mpstack).capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            (*mpstack).items
                                                                as *const ::core::ffi::c_void,
                                                            (*mpstack).size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                            } else {
                                                                xrealloc(
                                                                    (*mpstack).items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack)
                                                                        .capacity
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<
                                                                                MPConvStackVal,
                                                                            >(
                                                                            ),
                                                                        ),
                                                                )
                                                            }
                                                        })
                                                            as *mut MPConvStackVal;
                                                } else {
                                                };
                                                let c2rust_fresh5 = (*mpstack).size;
                                                (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                                *(*mpstack).items.offset(c2rust_fresh5 as isize) =
                                                    MPConvStackVal {
                                                        type_0: kMPConvList,
                                                        tv: tv,
                                                        saved_copyID: saved_copyID_0,
                                                        data: C2Rust_Unnamed_18 {
                                                            l: C2Rust_Unnamed_21 {
                                                                list: (*val_di).di_tv.vval.v_list,
                                                                li: tv_list_first(
                                                                    (*val_di).di_tv.vval.v_list,
                                                                ),
                                                            },
                                                        },
                                                    };
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                        6 => {
                                            if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                                == VAR_LIST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                let val_list_0: *mut list_T =
                                                    (*val_di).di_tv.vval.v_list;
                                                if val_list_0.is_null()
                                                    || tv_list_len(val_list_0)
                                                        == 0 as ::core::ffi::c_int
                                                {
                                                    '_c2rust_label_2: {
                                                        if (_typval_encode_nothing_nodict_var.ptr()
                                                            as *const _)
                                                            as *mut ::core::ffi::c_void
                                                            != (_typval_encode_nothing_nodict_var
                                                                .ptr()
                                                                as *const _)
                                                                as *mut ::core::ffi::c_void
                                                        {
                                                        } else {
                                                            __assert_fail(
                                                            b"(void *)&(_typval_encode_nothing_nodict_var) != (void *)&TYPVAL_ENCODE_NODICT_VAR\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            b"src/nvim/eval/typval.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            552 as ::core::ffi::c_uint,
                                                            b"int _typval_encode_nothing_convert_one_value(const void *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                        }
                                                    };
                                                    _nothing_conv_empty_dict(
                                                        tv,
                                                        (_typval_encode_nothing_nodict_var.ptr()
                                                            as *const _)
                                                            as *mut *mut dict_T,
                                                    );
                                                    break '_typval_encode_stop_converting_one_item;
                                                } else {
                                                    let l_: *const list_T = val_list_0;
                                                    's_689: {
                                                        if !l_.is_null() {
                                                            let mut li: *const listitem_T =
                                                                (*l_).lv_first;
                                                            loop {
                                                                if li.is_null() {
                                                                    break 's_689;
                                                                }
                                                                if (*li).li_tv.v_type
                                                                    as ::core::ffi::c_uint
                                                                    != VAR_LIST
                                                                        as ::core::ffi::c_int
                                                                        as ::core::ffi::c_uint
                                                                    || tv_list_len(
                                                                        (*li).li_tv.vval.v_list,
                                                                    ) != 2 as ::core::ffi::c_int
                                                                {
                                                                    break 's_771;
                                                                }
                                                                li = (*li).li_next;
                                                            }
                                                        }
                                                    }
                                                    let saved_copyID_1: ::core::ffi::c_int =
                                                        tv_list_copyid((*val_di).di_tv.vval.v_list);
                                                    let te_csr_ret_1: ::core::ffi::c_int =
                                                        _typval_encode_nothing_check_self_reference(
                                                            ignored,
                                                            val_list_0 as *mut ::core::ffi::c_void,
                                                            &raw mut (*val_list_0).lv_copyID,
                                                            mpstack,
                                                            copyID,
                                                            kMPConvPairs,
                                                            objname,
                                                        );
                                                    if te_csr_ret_1 != NOTDONE {
                                                        return te_csr_ret_1;
                                                    }
                                                    '_c2rust_label_3: {
                                                        if saved_copyID_1 != copyID
                                                            && saved_copyID_1
                                                                != copyID - 1 as ::core::ffi::c_int
                                                        {
                                                        } else {
                                                            __assert_fail(
                                                            b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            b"src/nvim/eval/typval.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            566 as ::core::ffi::c_uint,
                                                            b"int _typval_encode_nothing_convert_one_value(const void *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                        }
                                                    };
                                                    if (*mpstack).size == (*mpstack).capacity {
                                                        (*mpstack).capacity = if (*mpstack).capacity
                                                            << 1 as ::core::ffi::c_int
                                                            > ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            ) {
                                                            (*mpstack).capacity
                                                                << 1 as ::core::ffi::c_int
                                                        } else {
                                                            ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as size_t,
                                                            )
                                                        };
                                                        (*mpstack).items = (if (*mpstack).capacity
                                                            == ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            ) {
                                                            if (*mpstack).items
                                                                == &raw mut (*mpstack).init_array
                                                                    as *mut MPConvStackVal
                                                            {
                                                                (*mpstack).items
                                                                    as *mut ::core::ffi::c_void
                                                            } else {
                                                                _memcpy_free(
                                                                    &raw mut (*mpstack).init_array
                                                                        as *mut MPConvStackVal
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack).items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack).size.wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            MPConvStackVal,
                                                                        >(
                                                                        ),
                                                                    ),
                                                                )
                                                            }
                                                        } else {
                                                            if (*mpstack).items
                                                                == &raw mut (*mpstack).init_array
                                                                    as *mut MPConvStackVal
                                                            {
                                                                memcpy(
                                                                xmalloc(
                                                                    (*mpstack)
                                                                        .capacity
                                                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                                                ),
                                                                (*mpstack).items as *const ::core::ffi::c_void,
                                                                (*mpstack)
                                                                    .size
                                                                    .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                                            )
                                                            } else {
                                                                xrealloc(
                                                                    (*mpstack).items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack)
                                                                        .capacity
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<
                                                                                MPConvStackVal,
                                                                            >(
                                                                            ),
                                                                        ),
                                                                )
                                                            }
                                                        })
                                                            as *mut MPConvStackVal;
                                                    } else {
                                                    };
                                                    let c2rust_fresh6 = (*mpstack).size;
                                                    (*mpstack).size =
                                                        (*mpstack).size.wrapping_add(1);
                                                    *(*mpstack)
                                                        .items
                                                        .offset(c2rust_fresh6 as isize) =
                                                        MPConvStackVal {
                                                            type_0: kMPConvPairs,
                                                            tv: tv,
                                                            saved_copyID: saved_copyID_1,
                                                            data: C2Rust_Unnamed_18 {
                                                                l: C2Rust_Unnamed_21 {
                                                                    list: val_list_0,
                                                                    li: tv_list_first(val_list_0),
                                                                },
                                                            },
                                                        };
                                                    break '_typval_encode_stop_converting_one_item;
                                                }
                                            }
                                        }
                                        7 => {
                                            let mut val_list_1: *const list_T =
                                                ::core::ptr::null::<list_T>();
                                            let mut type_0: varnumber_T = 0;
                                            if !((*val_di).di_tv.v_type as ::core::ffi::c_uint
                                                != VAR_LIST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                || {
                                                    val_list_1 = (*val_di).di_tv.vval.v_list;
                                                    tv_list_len(val_list_1)
                                                        != 2 as ::core::ffi::c_int
                                                }
                                                || (*tv_list_first(val_list_1)).li_tv.v_type
                                                    as ::core::ffi::c_uint
                                                    != VAR_NUMBER as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                || {
                                                    type_0 = (*tv_list_first(val_list_1))
                                                        .li_tv
                                                        .vval
                                                        .v_number;
                                                    type_0 > INT8_MAX as varnumber_T
                                                }
                                                || type_0 < INT8_MIN as varnumber_T
                                                || (*tv_list_last(val_list_1)).li_tv.v_type
                                                    as ::core::ffi::c_uint
                                                    != VAR_LIST as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint)
                                            {
                                                let mut len_0: size_t = 0;
                                                let mut buf_0: *mut ::core::ffi::c_char =
                                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                                if encode_vim_list_to_buf(
                                                    (*tv_list_last(val_list_1)).li_tv.vval.v_list,
                                                    &raw mut len_0,
                                                    &raw mut buf_0,
                                                ) {
                                                    xfree(buf_0 as *mut ::core::ffi::c_void);
                                                    break '_typval_encode_stop_converting_one_item;
                                                }
                                            }
                                        }
                                        _ => {
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                }
                            }
                        }
                        let saved_copyID_2: ::core::ffi::c_int = (*(*tv).vval.v_dict).dv_copyID;
                        let te_csr_ret_2: ::core::ffi::c_int =
                            _typval_encode_nothing_check_self_reference(
                                ignored,
                                (*tv).vval.v_dict as *mut ::core::ffi::c_void,
                                &raw mut (*(*tv).vval.v_dict).dv_copyID,
                                mpstack,
                                copyID,
                                kMPConvDict,
                                objname,
                            );
                        if te_csr_ret_2 != NOTDONE {
                            return te_csr_ret_2;
                        }
                        '_c2rust_label_4: {
                            if saved_copyID_2 != copyID {
                            } else {
                                __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/typval.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                614 as ::core::ffi::c_uint,
                                b"int _typval_encode_nothing_convert_one_value(const void *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                            }
                        };
                        if (*mpstack).size == (*mpstack).capacity {
                            (*mpstack).capacity = if (*mpstack).capacity << 1 as ::core::ffi::c_int
                                > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                            .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ) {
                                (*mpstack).capacity << 1 as ::core::ffi::c_int
                            } else {
                                ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                            .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as size_t,
                                    )
                            };
                            (*mpstack).items = (if (*mpstack).capacity
                                == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                            .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ) {
                                if (*mpstack).items
                                    == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                {
                                    (*mpstack).items as *mut ::core::ffi::c_void
                                } else {
                                    _memcpy_free(
                                        &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                            as *mut ::core::ffi::c_void,
                                        (*mpstack).items as *mut ::core::ffi::c_void,
                                        (*mpstack)
                                            .size
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    )
                                }
                            } else {
                                if (*mpstack).items
                                    == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                {
                                    memcpy(
                                        xmalloc(
                                            (*mpstack).capacity.wrapping_mul(
                                                ::core::mem::size_of::<MPConvStackVal>(),
                                            ),
                                        ),
                                        (*mpstack).items as *const ::core::ffi::c_void,
                                        (*mpstack)
                                            .size
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    )
                                } else {
                                    xrealloc(
                                        (*mpstack).items as *mut ::core::ffi::c_void,
                                        (*mpstack)
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    )
                                }
                            })
                                as *mut MPConvStackVal;
                        } else {
                        };
                        let c2rust_fresh7 = (*mpstack).size;
                        (*mpstack).size = (*mpstack).size.wrapping_add(1);
                        *(*mpstack).items.offset(c2rust_fresh7 as isize) = MPConvStackVal {
                            type_0: kMPConvDict,
                            tv: tv,
                            saved_copyID: saved_copyID_2,
                            data: C2Rust_Unnamed_18 {
                                d: C2Rust_Unnamed_22 {
                                    dict: (*tv).vval.v_dict,
                                    dictp: &raw mut (*tv).vval.v_dict,
                                    hi: (*(*tv).vval.v_dict).dv_hashtab.ht_array,
                                    todo: (*(*tv).vval.v_dict).dv_hashtab.ht_used,
                                },
                            },
                        };
                        let _ = _nothing_conv_real_dict_after_start(
                            tv,
                            &raw mut (*tv).vval.v_dict,
                            (_typval_encode_nothing_nodict_var.ptr() as *const _)
                                as *mut ::core::ffi::c_void,
                            (*mpstack).items.offset(
                                (*mpstack)
                                    .size
                                    .wrapping_sub(0 as size_t)
                                    .wrapping_sub(1 as size_t)
                                    as isize,
                            ),
                        ) != NOTDONE;
                    }
                }
                0 => {
                    internal_error(b"_typval_encode_nothing_convert_one_value()\0".as_ptr()
                        as *const ::core::ffi::c_char);
                    return FAIL;
                }
                _ => {}
            }
        }
        return OK;
    }
}
