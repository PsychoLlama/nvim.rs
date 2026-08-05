//! The `nothing` encoder sink: the deep free `tv_clear` uses.
//!
//! Upstream instantiates `typval_encode.c.h` a seventh time here, with every
//! conversion hook doing nothing but releasing what it is handed, so that
//! [`super::value::tv_clear`] can free a container that references itself
//! without recursing.  This half is the driver and the hooks; the generated
//! per-value switch is [`super::nothing_convert`].

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[inline(always)]
pub(crate) unsafe extern "C" fn _nothing_conv_func_start(
    tv: *mut typval_T,
    fun: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        (*tv).v_lock = VAR_UNLOCKED;
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let pt_: *mut partial_T = (*tv).vval.v_partial;
            if !pt_.is_null() && (*pt_).pt_refcount > 1 as ::core::ffi::c_int {
                (*pt_).pt_refcount -= 1;
                (*tv).vval.v_partial = ::core::ptr::null_mut::<partial_T>();
                return OK;
            }
        } else {
            func_unref(fun);
            if fun != tv_empty_string.get() as *mut ::core::ffi::c_char {
                xfree(fun as *mut ::core::ffi::c_void);
            }
            (*tv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return NOTDONE;
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn _nothing_conv_func_end(
    tv: *mut typval_T,
    copyID: ::core::ffi::c_int,
) {
    unsafe {
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let pt: *mut partial_T = (*tv).vval.v_partial;
            if pt.is_null() {
                return;
            }
            '_c2rust_label: {
                if (*pt).pt_dict.is_null() || (*(*pt).pt_dict).dv_copyID == copyID {
                } else {
                    __assert_fail(
                        b"pt->pt_dict == NULL || pt->pt_dict->dv_copyID == copyID\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3488 as ::core::ffi::c_uint,
                        b"void _nothing_conv_func_end(typval_T *const, const int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            (*pt).pt_dict = ::core::ptr::null_mut::<dict_T>();
            (*pt).pt_argc = 0 as ::core::ffi::c_int;
            '_c2rust_label_0: {
                if (*pt).pt_refcount <= 1 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"pt->pt_refcount <= 1\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3492 as ::core::ffi::c_uint,
                        b"void _nothing_conv_func_end(typval_T *const, const int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            partial_unref(pt);
            (*tv).vval.v_partial = ::core::ptr::null_mut::<partial_T>();
            '_c2rust_label_1: {
                if (*tv).v_lock as ::core::ffi::c_uint
                    == VAR_UNLOCKED as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                } else {
                    __assert_fail(
                        b"tv->v_lock == VAR_UNLOCKED\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3495 as ::core::ffi::c_uint,
                        b"void _nothing_conv_func_end(typval_T *const, const int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
        }
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn _nothing_conv_empty_dict(
    tv: *mut typval_T,
    dictp: *mut *mut dict_T,
) {
    unsafe {
        tv_dict_unref(*dictp);
        *dictp = ::core::ptr::null_mut::<dict_T>();
        if !tv.is_null() {
            (*tv).v_lock = VAR_UNLOCKED;
        }
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn _nothing_conv_real_list_after_start(
    tv: *mut typval_T,
    mpsv: *mut MPConvStackVal,
) -> ::core::ffi::c_int {
    unsafe {
        '_c2rust_label: {
            if !tv.is_null() {
            } else {
                __assert_fail(
                b"tv != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3526 as ::core::ffi::c_uint,
                b"int _nothing_conv_real_list_after_start(typval_T *const, MPConvStackVal *const)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
            }
        };
        (*tv).v_lock = VAR_UNLOCKED;
        if (*(*tv).vval.v_list).lv_refcount > 1 as ::core::ffi::c_int {
            (*(*tv).vval.v_list).lv_refcount -= 1;
            (*tv).vval.v_list = ::core::ptr::null_mut::<list_T>();
            (*mpsv).data.l.li = ::core::ptr::null_mut::<listitem_T>();
            return OK;
        }
        return NOTDONE;
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn _nothing_conv_list_end(tv: *mut typval_T) {
    unsafe {
        if tv.is_null() {
            return;
        }
        '_c2rust_label: {
            if (*tv).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"tv->v_type == VAR_LIST\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3553 as ::core::ffi::c_uint,
                    b"void _nothing_conv_list_end(typval_T *const)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let list: *mut list_T = (*tv).vval.v_list;
        tv_list_unref(list);
        (*tv).vval.v_list = ::core::ptr::null_mut::<list_T>();
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn _nothing_conv_real_dict_after_start(
    tv: *mut typval_T,
    dictp: *mut *mut dict_T,
    nodictvar: *const ::core::ffi::c_void,
    mpsv: *mut MPConvStackVal,
) -> ::core::ffi::c_int {
    unsafe {
        if !tv.is_null() {
            (*tv).v_lock = VAR_UNLOCKED;
        }
        if dictp as *const ::core::ffi::c_void != nodictvar
            && (**dictp).dv_refcount > 1 as ::core::ffi::c_int
        {
            (**dictp).dv_refcount -= 1;
            *dictp = ::core::ptr::null_mut::<dict_T>();
            (*mpsv).data.d.todo = 0 as size_t;
            return OK;
        }
        return NOTDONE;
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn _nothing_conv_dict_end(
    _tv: *mut typval_T,
    dictp: *mut *mut dict_T,
    nodictvar: *const ::core::ffi::c_void,
) {
    unsafe {
        if dictp as *const ::core::ffi::c_void != nodictvar {
            tv_dict_unref(*dictp);
            *dictp = ::core::ptr::null_mut::<dict_T>();
        }
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn _typval_encode_nothing_check_self_reference(
    _ignored: *const ::core::ffi::c_void,
    _val: *mut ::core::ffi::c_void,
    val_copyID: *mut ::core::ffi::c_int,
    _mpstack: *const MPConvStack,
    copyID: ::core::ffi::c_int,
    _conv_type: MPConvStackValType,
    _objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if *val_copyID == copyID {
            return OK;
        }
        *val_copyID = copyID;
        return NOTDONE;
    }
}

pub(crate) unsafe extern "C" fn encode_vim_to_nothing(
    ignored: *const ::core::ffi::c_void,
    top_tv: *mut typval_T,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let copyID: ::core::ffi::c_int = get_copyID();
        let mut mpstack: MPConvStack = MPConvStack {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<MPConvStackVal>(),
            init_array: [MPConvStackVal {
                type_0: kMPConvDict,
                tv: ::core::ptr::null_mut::<typval_T>(),
                saved_copyID: 0,
                data: C2Rust_Unnamed_18 {
                    d: C2Rust_Unnamed_22 {
                        dict: ::core::ptr::null_mut::<dict_T>(),
                        dictp: ::core::ptr::null_mut::<*mut dict_T>(),
                        hi: ::core::ptr::null_mut::<hashitem_T>(),
                        todo: 0,
                    },
                },
            }; 8],
        };
        mpstack.capacity = ::core::mem::size_of::<[MPConvStackVal; 8]>()
            .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
            .wrapping_div(
                (::core::mem::size_of::<[MPConvStackVal; 8]>()
                    .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as size_t;
        mpstack.size = 0 as size_t;
        mpstack.items = &raw mut mpstack.init_array as *mut MPConvStackVal;
        '_encode_vim_to__error_ret: {
            if _typval_encode_nothing_convert_one_value(
                ignored,
                &raw mut mpstack,
                ::core::ptr::null_mut::<MPConvStackVal>(),
                top_tv,
                copyID,
                objname,
            ) != FAIL
            {
                while mpstack.size != 0 {
                    let mut cur_mpsv: *mut MPConvStackVal = mpstack.items.offset(
                        mpstack
                            .size
                            .wrapping_sub(0 as size_t)
                            .wrapping_sub(1 as size_t) as isize,
                    );
                    let mut tv: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
                    match (*cur_mpsv).type_0 as ::core::ffi::c_uint {
                        0 => {
                            if (*cur_mpsv).data.d.todo == 0 {
                                mpstack.size = mpstack.size.wrapping_sub(1);
                                (*(*cur_mpsv).data.d.dict).dv_copyID = (*cur_mpsv).saved_copyID;
                                _nothing_conv_dict_end(
                                    (*cur_mpsv).tv,
                                    (*cur_mpsv).data.d.dictp,
                                    (_typval_encode_nothing_nodict_var.ptr() as *const _)
                                        as *mut ::core::ffi::c_void,
                                );
                                continue;
                            } else {
                                let _ = (*cur_mpsv).data.d.todo
                                    != (*(*cur_mpsv).data.d.dict).dv_hashtab.ht_used;
                                while (*(*cur_mpsv).data.d.hi).hi_key.is_null()
                                    || (*(*cur_mpsv).data.d.hi).hi_key
                                        == &raw const hash_removed as *mut ::core::ffi::c_char
                                {
                                    (*cur_mpsv).data.d.hi = (*cur_mpsv).data.d.hi.offset(1);
                                }
                                let di: *mut dictitem_T = (*(*cur_mpsv).data.d.hi)
                                    .hi_key
                                    .offset(-(17 as ::core::ffi::c_ulong as isize))
                                    as *mut dictitem_T;
                                (*cur_mpsv).data.d.todo = (*cur_mpsv).data.d.todo.wrapping_sub(1);
                                (*cur_mpsv).data.d.hi = (*cur_mpsv).data.d.hi.offset(1);
                                tv = &raw mut (*di).di_tv;
                            }
                        }
                        1 => {
                            if (*cur_mpsv).data.l.li.is_null() {
                                mpstack.size = mpstack.size.wrapping_sub(1);
                                tv_list_set_copyid(
                                    (*cur_mpsv).data.l.list,
                                    (*cur_mpsv).saved_copyID,
                                );
                                _nothing_conv_list_end((*cur_mpsv).tv);
                                continue;
                            } else {
                                let _ =
                                    (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list);
                                tv = &raw mut (*(*cur_mpsv).data.l.li).li_tv;
                                (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                            }
                        }
                        2 => {
                            if (*cur_mpsv).data.l.li.is_null() {
                                mpstack.size = mpstack.size.wrapping_sub(1);
                                tv_list_set_copyid(
                                    (*cur_mpsv).data.l.list,
                                    (*cur_mpsv).saved_copyID,
                                );
                                _nothing_conv_dict_end(
                                    (*cur_mpsv).tv,
                                    (_typval_encode_nothing_nodict_var.ptr() as *const _)
                                        as *mut *mut dict_T,
                                    (_typval_encode_nothing_nodict_var.ptr() as *const _)
                                        as *mut ::core::ffi::c_void,
                                );
                                continue;
                            } else {
                                let _ =
                                    (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list);
                                let kv_pair: *const list_T =
                                    (*(*cur_mpsv).data.l.li).li_tv.vval.v_list;
                                if _typval_encode_nothing_convert_one_value(
                                    ignored,
                                    &raw mut mpstack,
                                    cur_mpsv,
                                    &raw mut (*tv_list_first(kv_pair)).li_tv,
                                    copyID,
                                    objname,
                                ) == FAIL
                                {
                                    break '_encode_vim_to__error_ret;
                                }
                                tv = &raw mut (*tv_list_last(kv_pair)).li_tv;
                                (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                            }
                        }
                        3 => {
                            let pt: *mut partial_T = (*cur_mpsv).data.p.pt;
                            tv = (*cur_mpsv).tv;
                            match (*cur_mpsv).data.p.stage as ::core::ffi::c_uint {
                                0 => {
                                    (*cur_mpsv).data.p.stage = kMPConvPartialSelf;
                                    if !pt.is_null() && (*pt).pt_argc > 0 as ::core::ffi::c_int {
                                        if mpstack.size == mpstack.capacity {
                                            mpstack.capacity =
                                                if mpstack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        )
                                                {
                                                    mpstack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                            mpstack.items =
                                                (if mpstack.capacity
                                                    == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        )
                                                {
                                                    if mpstack.items
                                                        == &raw mut mpstack.init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        mpstack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut mpstack.init_array
                                                                as *mut MPConvStackVal
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.items
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if mpstack.items
                                                        == &raw mut mpstack.init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        memcpy(
                                                            xmalloc(mpstack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            )),
                                                            mpstack.items
                                                                as *const ::core::ffi::c_void,
                                                            mpstack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            mpstack.items
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.capacity.wrapping_mul(
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
                                        let c2rust_fresh0 = mpstack.size;
                                        mpstack.size = mpstack.size.wrapping_add(1);
                                        *mpstack.items.offset(c2rust_fresh0 as isize) =
                                            MPConvStackVal {
                                                type_0: kMPConvPartialList,
                                                tv: ::core::ptr::null_mut::<typval_T>(),
                                                saved_copyID: copyID - 1 as ::core::ffi::c_int,
                                                data: C2Rust_Unnamed_18 {
                                                    a: C2Rust_Unnamed_19 {
                                                        arg: (*pt).pt_argv,
                                                        argv: (*pt).pt_argv,
                                                        todo: (*pt).pt_argc as size_t,
                                                    },
                                                },
                                            };
                                    }
                                    continue;
                                }
                                1 => {
                                    (*cur_mpsv).data.p.stage = kMPConvPartialEnd;
                                    let dict: *mut dict_T = if pt.is_null() {
                                        ::core::ptr::null_mut::<dict_T>()
                                    } else {
                                        (*pt).pt_dict
                                    };
                                    if dict.is_null() {
                                        continue;
                                    }
                                    if (*dict).dv_hashtab.ht_used == 0 as size_t {
                                        '_c2rust_label: {
                                            if &raw mut (*pt).pt_dict as *mut ::core::ffi::c_void
                                                != (_typval_encode_nothing_nodict_var.ptr()
                                                    as *const _)
                                                    as *mut ::core::ffi::c_void
                                            {
                                            } else {
                                                __assert_fail(
                                                b"(void *)&(pt->pt_dict) != (void *)&TYPVAL_ENCODE_NODICT_VAR\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                b"src/nvim/eval/typval.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                772 as ::core::ffi::c_uint,
                                                b"int encode_vim_to_nothing(const void *const, typval_T *const, const char *const)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                            }
                                        };
                                        _nothing_conv_empty_dict(
                                            ::core::ptr::null_mut::<typval_T>(),
                                            &raw mut (*pt).pt_dict,
                                        );
                                        continue;
                                    } else {
                                        let saved_copyID: ::core::ffi::c_int = (*dict).dv_copyID;
                                        let te_csr_ret: ::core::ffi::c_int =
                                            _typval_encode_nothing_check_self_reference(
                                                ignored,
                                                dict as *mut ::core::ffi::c_void,
                                                &raw mut (*dict).dv_copyID,
                                                &raw mut mpstack,
                                                copyID,
                                                kMPConvDict,
                                                objname,
                                            );
                                        if te_csr_ret != NOTDONE {
                                            if te_csr_ret == FAIL {
                                                break '_encode_vim_to__error_ret;
                                            } else {
                                                continue;
                                            }
                                        } else {
                                            '_c2rust_label_0: {
                                                if saved_copyID != copyID
                                                    && saved_copyID
                                                        != copyID - 1 as ::core::ffi::c_int
                                                {
                                                } else {
                                                    __assert_fail(
                                                    b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    b"src/nvim/eval/typval.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    789 as ::core::ffi::c_uint,
                                                    b"int encode_vim_to_nothing(const void *const, typval_T *const, const char *const)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                                }
                                            };
                                            if mpstack.size == mpstack.capacity {
                                                mpstack.capacity = if mpstack.capacity
                                                    << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                    mpstack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                                mpstack.items = (if mpstack.capacity
                                                    == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                    if mpstack.items
                                                        == &raw mut mpstack.init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        mpstack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut mpstack.init_array
                                                                as *mut MPConvStackVal
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.items
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if mpstack.items
                                                        == &raw mut mpstack.init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        memcpy(
                                                            xmalloc(mpstack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            )),
                                                            mpstack.items
                                                                as *const ::core::ffi::c_void,
                                                            mpstack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            mpstack.items
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.capacity.wrapping_mul(
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
                                            let c2rust_fresh1 = mpstack.size;
                                            mpstack.size = mpstack.size.wrapping_add(1);
                                            *mpstack.items.offset(c2rust_fresh1 as isize) =
                                                MPConvStackVal {
                                                    type_0: kMPConvDict,
                                                    tv: ::core::ptr::null_mut::<typval_T>(),
                                                    saved_copyID: saved_copyID,
                                                    data: C2Rust_Unnamed_18 {
                                                        d: C2Rust_Unnamed_22 {
                                                            dict: dict,
                                                            dictp: &raw mut (*pt).pt_dict,
                                                            hi: (*dict).dv_hashtab.ht_array,
                                                            todo: (*dict).dv_hashtab.ht_used,
                                                        },
                                                    },
                                                };
                                            if _nothing_conv_real_dict_after_start(
                                                ::core::ptr::null_mut::<typval_T>(),
                                                &raw mut (*pt).pt_dict,
                                                (_typval_encode_nothing_nodict_var.ptr()
                                                    as *const _)
                                                    as *mut ::core::ffi::c_void,
                                                mpstack.items.offset(
                                                    mpstack
                                                        .size
                                                        .wrapping_sub(0 as size_t)
                                                        .wrapping_sub(1 as size_t)
                                                        as isize,
                                                ),
                                            ) != NOTDONE
                                            {
                                                continue;
                                            } else {
                                                continue;
                                            }
                                        }
                                    }
                                }
                                2 => {
                                    _nothing_conv_func_end(tv, copyID);
                                    mpstack.size = mpstack.size.wrapping_sub(1);
                                    continue;
                                }
                                _ => {
                                    continue;
                                }
                            }
                        }
                        4 => {
                            if (*cur_mpsv).data.a.todo == 0 {
                                mpstack.size = mpstack.size.wrapping_sub(1);
                                _nothing_conv_list_end(::core::ptr::null_mut::<typval_T>());
                                continue;
                            } else {
                                let _ = (*cur_mpsv).data.a.argv != (*cur_mpsv).data.a.arg;
                                let c2rust_fresh2 = (*cur_mpsv).data.a.arg;
                                (*cur_mpsv).data.a.arg = (*cur_mpsv).data.a.arg.offset(1);
                                tv = c2rust_fresh2;
                                (*cur_mpsv).data.a.todo = (*cur_mpsv).data.a.todo.wrapping_sub(1);
                            }
                        }
                        _ => {}
                    }
                    '_c2rust_label_1: {
                        if !tv.is_null() {
                        } else {
                            __assert_fail(
                            b"tv != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/eval/typval.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            829 as ::core::ffi::c_uint,
                            b"int encode_vim_to_nothing(const void *const, typval_T *const, const char *const)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                        }
                    };
                    if _typval_encode_nothing_convert_one_value(
                        ignored,
                        &raw mut mpstack,
                        cur_mpsv,
                        tv,
                        copyID,
                        objname,
                    ) == FAIL
                    {
                        break '_encode_vim_to__error_ret;
                    }
                }
                if mpstack.items != &raw mut mpstack.init_array as *mut MPConvStackVal {
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut mpstack.items as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL_0;
                    let _ = *ptr_;
                }
                return OK;
            }
        }
        if mpstack.items != &raw mut mpstack.init_array as *mut MPConvStackVal {
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut mpstack.items as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL_0;
            let _ = *ptr__0;
        }
        return FAIL;
    }
}
