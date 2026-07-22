use crate::src::nvim::eval::typval::{
    tv_dict_find, tv_list_append_allocated_string, tv_list_idx_of_item,
};
use crate::src::nvim::eval::vars::eval_msgpack_type_lists;
use crate::src::nvim::eval_1::{get_copyID, partial_name};
use crate::src::nvim::garray::{ga_append, ga_clear, ga_concat, ga_concat_len, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::main::IObuff;
use crate::src::nvim::mbyte::{utf_char2len, utf_printable, utf_ptr2char, utf_ptr2len};
use crate::src::nvim::memory::{memchrsub, memcnt, xfree, xmalloc, xmemdupz, xmemscan, xrealloc};
use crate::src::nvim::message::{emsg, internal_error, semsg};
use crate::src::nvim::msgpack_rpc::packer::{
    mpack_bin, mpack_check_buffer, mpack_ext, mpack_float8, mpack_integer, mpack_str, mpack_uint64,
};
use crate::src::nvim::os::libc::{__assert_fail, abort, gettext, memcpy, strlen};
use crate::src::nvim::strings::{vim_snprintf, vim_snprintf_safelen};
pub use crate::src::nvim::types::{
    blob_T, blobvar_S, dict_T, dictitem_T, dictvar_S, float_T, funccall_S,
    funccall_S_fc_fixvar as C2Rust_Unnamed, funccall_T, garray_T, hash_T, hashitem_T, hashtab_T,
    int32_t, int64_t, int8_t, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S,
    listwatch_T, packer_buffer_t, partial_S, partial_T, proftime_T, ptrdiff_t, queue, scid_T,
    sctx_T, size_t, typval_T, typval_vval_union, ufunc_S, ufunc_T, uint32_t, uint64_t, uint8_t,
    varnumber_T, BoolVarValue, Integer, ListReaderState, LuaRef, MPConvPartialStage, MPConvStack,
    MPConvStackVal, MPConvStackValType, MPConvStackVal_data as C2Rust_Unnamed_0,
    MPConvStackVal_data_a as C2Rust_Unnamed_1, MPConvStackVal_data_d as C2Rust_Unnamed_4,
    MPConvStackVal_data_l as C2Rust_Unnamed_3, MPConvStackVal_data_p as C2Rust_Unnamed_2,
    MessagePackType, PackerBuffer, PackerBufferFlush, ScopeDictDictItem, ScopeType,
    SpecialVarValue, String_0, VarLockStatus, VarType, QUEUE,
};
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_BLOB: VarType = 10;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const kMPExt: MessagePackType = 7;
pub const kMPMap: MessagePackType = 6;
pub const kMPArray: MessagePackType = 5;
pub const kMPString: MessagePackType = 4;
pub const kMPFloat: MessagePackType = 3;
pub const kMPInteger: MessagePackType = 2;
pub const kMPBoolean: MessagePackType = 1;
pub const kMPNil: MessagePackType = 0;
pub const kMPConvPartialEnd: MPConvPartialStage = 2;
pub const kMPConvPartialSelf: MPConvPartialStage = 1;
pub const kMPConvPartialArgs: MPConvPartialStage = 0;
pub const kMPConvPartialList: MPConvStackValType = 4;
pub const kMPConvPartial: MPConvStackValType = 3;
pub const kMPConvPairs: MPConvStackValType = 2;
pub const kMPConvList: MPConvStackValType = 1;
pub const kMPConvDict: MPConvStackValType = 0;
pub const INT8_MIN: ::core::ffi::c_int = -128 as ::core::ffi::c_int;
pub const INT8_MAX: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline(always)]
unsafe extern "C" fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    memcpy(dest, src, size);
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw const src as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    return dest;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BS: ::core::ffi::c_int = 8;
pub const TAB: ::core::ffi::c_int = 9;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const FF: ::core::ffi::c_int = 12;
pub const CAR: ::core::ffi::c_int = 13;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub static _typval_encode_msgpack_nodict_var: GlobalCell<*const dict_T> =
    GlobalCell::new(::core::ptr::null::<dict_T>());
#[inline(always)]
unsafe extern "C" fn _typval_encode_msgpack_check_self_reference(
    _packer: *mut PackerBuffer,
    _val: *mut ::core::ffi::c_void,
    val_copyID: *mut ::core::ffi::c_int,
    mpstack: *const MPConvStack,
    copyID: ::core::ffi::c_int,
    _conv_type: MPConvStackValType,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *val_copyID == copyID {
        return conv_error(
            gettext(
                b"E5005: Unable to dump %s: container references itself in %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            mpstack,
            objname,
        );
    }
    *val_copyID = copyID;
    return NOTDONE;
}
unsafe extern "C" fn _typval_encode_msgpack_convert_one_value(
    packer: *mut PackerBuffer,
    mpstack: *mut MPConvStack,
    _cur_mpsv: *mut MPConvStackVal,
    tv: *mut typval_T,
    copyID: ::core::ffi::c_int,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    mpack_check_buffer(packer);
    '_typval_encode_stop_converting_one_item: {
        match (*tv).v_type as ::core::ffi::c_uint {
            2 => {
                mpack_bin(
                    String_0 {
                        data: (*tv).vval.v_string,
                        size: tv_strlen(tv),
                    },
                    packer,
                );
            }
            1 => {
                mpack_integer(&raw mut (*packer).ptr, (*tv).vval.v_number);
            }
            6 => {
                mpack_float8(&raw mut (*packer).ptr, (*tv).vval.v_float);
            }
            10 => {
                mpack_bin(
                    String_0 {
                        data: (if !(*tv).vval.v_blob.is_null() {
                            (*(*tv).vval.v_blob).bv_ga.ga_data
                        } else {
                            ::core::ptr::null_mut::<::core::ffi::c_void>()
                        }) as *mut ::core::ffi::c_char,
                        size: tv_blob_len((*tv).vval.v_blob) as size_t,
                    },
                    packer,
                );
            }
            3 => {
                return conv_error(
                    gettext(
                        b"E5004: Error while dumping %s, %s: attempt to dump function reference\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    mpstack,
                    objname,
                );
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
                return conv_error(
                    gettext(
                        b"E5004: Error while dumping %s, %s: attempt to dump function reference\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    mpstack,
                    objname,
                );
            }
            4 => {
                if (*tv).vval.v_list.is_null()
                    || tv_list_len((*tv).vval.v_list) == 0 as ::core::ffi::c_int
                {
                    mpack_array(&raw mut (*packer).ptr, 0 as uint32_t);
                } else {
                    let saved_copyID: ::core::ffi::c_int = tv_list_copyid((*tv).vval.v_list);
                    let te_csr_ret: ::core::ffi::c_int =
                        _typval_encode_msgpack_check_self_reference(
                            packer,
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
                    mpack_array(
                        &raw mut (*packer).ptr,
                        tv_list_len((*tv).vval.v_list) as uint32_t,
                    );
                    '_c2rust_label: {
                        if saved_copyID != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/encode.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                383 as ::core::ffi::c_uint,
                                b"int _typval_encode_msgpack_convert_one_value(PackerBuffer *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
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
                    let c2rust_fresh4 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh4 as isize) = MPConvStackVal {
                        type_0: kMPConvList,
                        tv: tv,
                        saved_copyID: saved_copyID,
                        data: C2Rust_Unnamed_0 {
                            l: C2Rust_Unnamed_3 {
                                list: (*tv).vval.v_list,
                                li: tv_list_first((*tv).vval.v_list),
                            },
                        },
                    };
                }
            }
            7 => match (*tv).vval.v_bool as ::core::ffi::c_uint {
                1 | 0 => {
                    mpack_bool(
                        &raw mut (*packer).ptr,
                        ((*tv).vval.v_bool as u64 != 0) as ::core::ffi::c_int
                            == kBoolVarTrue as ::core::ffi::c_int,
                    );
                }
                _ => {}
            },
            8 => match (*tv).vval.v_special as ::core::ffi::c_uint {
                0 => {
                    let c2rust_fresh5 = (*packer).ptr;
                    (*packer).ptr = (*packer).ptr.offset(1);
                    *c2rust_fresh5 = 0xc0 as ::core::ffi::c_int as ::core::ffi::c_char;
                }
                _ => {}
            },
            5 => {
                if (*tv).vval.v_dict.is_null()
                    || (*(*tv).vval.v_dict).dv_hashtab.ht_used == 0 as size_t
                {
                    mpack_map(&raw mut (*packer).ptr, 0 as uint32_t);
                } else {
                    let mut type_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    let mut val_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    's_564: {
                        if TYPVAL_ENCODE_ALLOW_SPECIALS_0 != 0
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
                                    == (*eval_msgpack_type_lists.ptr())[i as usize] as *mut list_T
                                {
                                    break;
                                }
                                i = i.wrapping_add(1);
                            }
                            mpack_check_buffer(packer);
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
                                        let c2rust_fresh6 = (*packer).ptr;
                                        (*packer).ptr = (*packer).ptr.offset(1);
                                        *c2rust_fresh6 =
                                            0xc0 as ::core::ffi::c_int as ::core::ffi::c_char;
                                        break '_typval_encode_stop_converting_one_item;
                                    }
                                    1 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_NUMBER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            mpack_bool(
                                                &raw mut (*packer).ptr,
                                                (*val_di).di_tv.vval.v_number != 0,
                                            );
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
                                                        highest_bits =
                                                            (*highest_bits_li).li_tv.vval.v_number;
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
                                                            high_bits =
                                                                (*high_bits_li).li_tv.vval.v_number;
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
                                                            if sign > 0 as varnumber_T {
                                                                mpack_uint64(
                                                                    &raw mut (*packer).ptr,
                                                                    number,
                                                                );
                                                            } else {
                                                                mpack_integer(
                                                                    &raw mut (*packer).ptr,
                                                                    number.wrapping_neg()
                                                                        as Integer,
                                                                );
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
                                            mpack_float8(
                                                &raw mut (*packer).ptr,
                                                (*val_di).di_tv.vval.v_float,
                                            );
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    4 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let mut len: size_t = 0;
                                            let mut buf: *mut ::core::ffi::c_char =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            if encode_vim_list_to_buf(
                                                (*val_di).di_tv.vval.v_list,
                                                &raw mut len,
                                                &raw mut buf,
                                            ) {
                                                mpack_str(
                                                    String_0 {
                                                        data: buf,
                                                        size: len,
                                                    },
                                                    packer,
                                                );
                                                xfree(buf as *mut ::core::ffi::c_void);
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    5 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let saved_copyID_0: ::core::ffi::c_int =
                                                tv_list_copyid((*val_di).di_tv.vval.v_list);
                                            let te_csr_ret_0: ::core::ffi::c_int =
                                                _typval_encode_msgpack_check_self_reference(
                                                    packer,
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
                                            mpack_array(
                                                &raw mut (*packer).ptr,
                                                tv_list_len((*val_di).di_tv.vval.v_list)
                                                    as uint32_t,
                                            );
                                            '_c2rust_label_0: {
                                                if saved_copyID_0 != copyID
                                                    && saved_copyID_0
                                                        != copyID - 1 as ::core::ffi::c_int
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        b"src/nvim/eval/encode.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        532 as ::core::ffi::c_uint,
                                                        b"int _typval_encode_msgpack_convert_one_value(PackerBuffer *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            if (*mpstack).size == (*mpstack).capacity {
                                                (*mpstack).capacity = if (*mpstack).capacity
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
                                                    (*mpstack).capacity << 1 as ::core::ffi::c_int
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
                                                (*mpstack).items = (if (*mpstack).capacity
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
                                                    if (*mpstack).items
                                                        == &raw mut (*mpstack).init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        (*mpstack).items as *mut ::core::ffi::c_void
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
                                                            (*mpstack).capacity.wrapping_mul(
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
                                            let c2rust_fresh7 = (*mpstack).size;
                                            (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                            *(*mpstack).items.offset(c2rust_fresh7 as isize) =
                                                MPConvStackVal {
                                                    type_0: kMPConvList,
                                                    tv: tv,
                                                    saved_copyID: saved_copyID_0,
                                                    data: C2Rust_Unnamed_0 {
                                                        l: C2Rust_Unnamed_3 {
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
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let val_list_0: *mut list_T =
                                                (*val_di).di_tv.vval.v_list;
                                            if val_list_0.is_null()
                                                || tv_list_len(val_list_0)
                                                    == 0 as ::core::ffi::c_int
                                            {
                                                mpack_map(&raw mut (*packer).ptr, 0 as uint32_t);
                                                break '_typval_encode_stop_converting_one_item;
                                            } else {
                                                let l_: *const list_T = val_list_0;
                                                's_479: {
                                                    if !l_.is_null() {
                                                        let mut li: *const listitem_T =
                                                            (*l_).lv_first;
                                                        loop {
                                                            if li.is_null() {
                                                                break 's_479;
                                                            }
                                                            if (*li).li_tv.v_type
                                                                as ::core::ffi::c_uint
                                                                != VAR_LIST as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                || tv_list_len(
                                                                    (*li).li_tv.vval.v_list,
                                                                ) != 2 as ::core::ffi::c_int
                                                            {
                                                                break 's_564;
                                                            }
                                                            li = (*li).li_next;
                                                        }
                                                    }
                                                }
                                                let saved_copyID_1: ::core::ffi::c_int =
                                                    tv_list_copyid((*val_di).di_tv.vval.v_list);
                                                let te_csr_ret_1: ::core::ffi::c_int =
                                                    _typval_encode_msgpack_check_self_reference(
                                                        packer,
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
                                                mpack_map(
                                                    &raw mut (*packer).ptr,
                                                    tv_list_len(val_list_0) as uint32_t,
                                                );
                                                '_c2rust_label_1: {
                                                    if saved_copyID_1 != copyID
                                                        && saved_copyID_1
                                                            != copyID - 1 as ::core::ffi::c_int
                                                    {
                                                    } else {
                                                        __assert_fail(
                                                            b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            b"src/nvim/eval/encode.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            566 as ::core::ffi::c_uint,
                                                            b"int _typval_encode_msgpack_convert_one_value(PackerBuffer *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
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
                                                let c2rust_fresh8 = (*mpstack).size;
                                                (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                                *(*mpstack).items.offset(c2rust_fresh8 as isize) =
                                                    MPConvStackVal {
                                                        type_0: kMPConvPairs,
                                                        tv: tv,
                                                        saved_copyID: saved_copyID_1,
                                                        data: C2Rust_Unnamed_0 {
                                                            l: C2Rust_Unnamed_3 {
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
                                                tv_list_len(val_list_1) != 2 as ::core::ffi::c_int
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
                                                mpack_ext(buf_0, len_0, type_0 as int8_t, packer);
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
                        _typval_encode_msgpack_check_self_reference(
                            packer,
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
                    mpack_map(
                        &raw mut (*packer).ptr,
                        (*(*tv).vval.v_dict).dv_hashtab.ht_used as uint32_t,
                    );
                    '_c2rust_label_2: {
                        if saved_copyID_2 != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/encode.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                614 as ::core::ffi::c_uint,
                                b"int _typval_encode_msgpack_convert_one_value(PackerBuffer *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
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
                    let c2rust_fresh9 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh9 as isize) = MPConvStackVal {
                        type_0: kMPConvDict,
                        tv: tv,
                        saved_copyID: saved_copyID_2,
                        data: C2Rust_Unnamed_0 {
                            d: C2Rust_Unnamed_4 {
                                dict: (*tv).vval.v_dict,
                                dictp: &raw mut (*tv).vval.v_dict,
                                hi: (*(*tv).vval.v_dict).dv_hashtab.ht_array,
                                todo: (*(*tv).vval.v_dict).dv_hashtab.ht_used,
                            },
                        },
                    };
                }
            }
            0 => {
                internal_error(b"_typval_encode_msgpack_convert_one_value()\0".as_ptr()
                    as *const ::core::ffi::c_char);
                return FAIL;
            }
            _ => {}
        }
    }
    return OK;
}
pub unsafe extern "C" fn encode_vim_to_msgpack(
    packer: *mut PackerBuffer,
    top_tv: *mut typval_T,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let copyID: ::core::ffi::c_int = get_copyID();
    let mut mpstack: MPConvStack = MPConvStack {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<MPConvStackVal>(),
        init_array: [MPConvStackVal {
            type_0: kMPConvDict,
            tv: ::core::ptr::null_mut::<typval_T>(),
            saved_copyID: 0,
            data: C2Rust_Unnamed_0 {
                d: C2Rust_Unnamed_4 {
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
        if _typval_encode_msgpack_convert_one_value(
            packer,
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
                            mpack_str(
                                String_0 {
                                    data: (&raw mut (*di).di_key as *mut ::core::ffi::c_char)
                                        .offset(0 as ::core::ffi::c_int as isize),
                                    size: strlen(
                                        (&raw mut (*di).di_key as *mut ::core::ffi::c_char)
                                            .offset(0 as ::core::ffi::c_int as isize),
                                    ),
                                },
                                packer,
                            );
                            tv = &raw mut (*di).di_tv;
                        }
                    }
                    1 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            continue;
                        } else {
                            let _ = (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list);
                            tv = &raw mut (*(*cur_mpsv).data.l.li).li_tv;
                            (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                        }
                    }
                    2 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            continue;
                        } else {
                            let _ = (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list);
                            let kv_pair: *const list_T = (*(*cur_mpsv).data.l.li).li_tv.vval.v_list;
                            if _typval_encode_msgpack_convert_one_value(
                                packer,
                                &raw mut mpstack,
                                cur_mpsv,
                                &raw mut (*(tv_list_first
                                    as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                    kv_pair,
                                ))
                                .li_tv,
                                copyID,
                                objname,
                            ) == FAIL
                            {
                                break '_encode_vim_to__error_ret;
                            }
                            tv = &raw mut (*(tv_list_last
                                as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                kv_pair,
                            ))
                            .li_tv;
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
                                    mpack_array(&raw mut (*packer).ptr, (*pt).pt_argc as uint32_t);
                                    if mpstack.size == mpstack.capacity {
                                        mpstack.capacity = if mpstack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            mpstack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as size_t,
                                                )
                                        };
                                        mpstack.items = (if mpstack.capacity
                                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
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
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
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
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    )),
                                                    mpstack.items as *const ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            } else {
                                                xrealloc(
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
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
                                            data: C2Rust_Unnamed_0 {
                                                a: C2Rust_Unnamed_1 {
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
                                    mpack_map(&raw mut (*packer).ptr, 0 as uint32_t);
                                    continue;
                                } else {
                                    let saved_copyID: ::core::ffi::c_int = (*dict).dv_copyID;
                                    let te_csr_ret: ::core::ffi::c_int =
                                        _typval_encode_msgpack_check_self_reference(
                                            packer,
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
                                        mpack_map(
                                            &raw mut (*packer).ptr,
                                            (*dict).dv_hashtab.ht_used as uint32_t,
                                        );
                                        '_c2rust_label: {
                                            if saved_copyID != copyID
                                                && saved_copyID != copyID - 1 as ::core::ffi::c_int
                                            {
                                            } else {
                                                __assert_fail(
                                                    b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    b"src/nvim/eval/encode.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    789 as ::core::ffi::c_uint,
                                                    b"int encode_vim_to_msgpack(PackerBuffer *const, typval_T *const, const char *const)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
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
                                        let c2rust_fresh1 = mpstack.size;
                                        mpstack.size = mpstack.size.wrapping_add(1);
                                        *mpstack.items.offset(c2rust_fresh1 as isize) =
                                            MPConvStackVal {
                                                type_0: kMPConvDict,
                                                tv: ::core::ptr::null_mut::<typval_T>(),
                                                saved_copyID: saved_copyID,
                                                data: C2Rust_Unnamed_0 {
                                                    d: C2Rust_Unnamed_4 {
                                                        dict: dict,
                                                        dictp: &raw mut (*pt).pt_dict,
                                                        hi: (*dict).dv_hashtab.ht_array,
                                                        todo: (*dict).dv_hashtab.ht_used,
                                                    },
                                                },
                                            };
                                        continue;
                                    }
                                }
                            }
                            2 => {
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
                '_c2rust_label_0: {
                    if !tv.is_null() {
                    } else {
                        __assert_fail(
                            b"tv != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/eval/encode.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            829 as ::core::ffi::c_uint,
                            b"int encode_vim_to_msgpack(PackerBuffer *const, typval_T *const, const char *const)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if _typval_encode_msgpack_convert_one_value(
                    packer,
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
pub static _typval_encode_echo_nodict_var: GlobalCell<*const dict_T> =
    GlobalCell::new(::core::ptr::null::<dict_T>());
#[inline(always)]
unsafe extern "C" fn _typval_encode_echo_check_self_reference(
    gap: *mut garray_T,
    val: *mut ::core::ffi::c_void,
    val_copyID: *mut ::core::ffi::c_int,
    mpstack: *const MPConvStack,
    copyID: ::core::ffi::c_int,
    conv_type: MPConvStackValType,
    _objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *val_copyID == copyID {
        let mut ebuf: [::core::ffi::c_char; 72] = [0; 72];
        let mut backref: size_t = 0 as size_t;
        while backref < (*mpstack).size {
            let mpval: MPConvStackVal = *(*mpstack).items.offset(backref as isize);
            if mpval.type_0 as ::core::ffi::c_uint == conv_type as ::core::ffi::c_uint {
                if conv_type as ::core::ffi::c_uint
                    == kMPConvDict as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if mpval.data.d.dict as *mut ::core::ffi::c_void == val {
                        break;
                    }
                } else if conv_type as ::core::ffi::c_uint
                    == kMPConvList as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if mpval.data.l.list as *mut ::core::ffi::c_void == val {
                        break;
                    }
                }
            }
            backref = backref.wrapping_add(1);
        }
        if conv_type as ::core::ffi::c_uint
            == kMPConvDict as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            vim_snprintf(
                &raw mut ebuf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 72]>()
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[::core::ffi::c_char; 72]>()
                            .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                            == 0) as ::core::ffi::c_int as size_t,
                    ),
                b"{...@%zu}\0".as_ptr() as *const ::core::ffi::c_char,
                backref,
            );
        } else {
            vim_snprintf(
                &raw mut ebuf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 72]>()
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[::core::ffi::c_char; 72]>()
                            .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                            == 0) as ::core::ffi::c_int as size_t,
                    ),
                b"[...@%zu]\0".as_ptr() as *const ::core::ffi::c_char,
                backref,
            );
        }
        ga_concat(
            gap,
            (&raw mut ebuf as *mut ::core::ffi::c_char).offset(0 as ::core::ffi::c_int as isize),
        );
        return OK;
    }
    *val_copyID = copyID;
    return NOTDONE;
}
unsafe extern "C" fn _typval_encode_echo_convert_one_value(
    gap: *mut garray_T,
    mpstack: *mut MPConvStack,
    _cur_mpsv: *mut MPConvStackVal,
    tv: *mut typval_T,
    copyID: ::core::ffi::c_int,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    '_typval_encode_stop_converting_one_item: {
        match (*tv).v_type as ::core::ffi::c_uint {
            2 => {
                let buf_: *const ::core::ffi::c_char = (*tv).vval.v_string;
                if buf_.is_null() {
                    ga_concat_len(
                        gap,
                        b"''\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    let len_: size_t = tv_strlen(tv);
                    ga_grow(
                        gap,
                        (2 as size_t).wrapping_add(len_).wrapping_add(memcnt(
                            buf_ as *const ::core::ffi::c_void,
                            '\'' as ::core::ffi::c_char,
                            len_,
                        )) as ::core::ffi::c_int,
                    );
                    ga_append(gap, '\'' as uint8_t);
                    let mut i_: size_t = 0 as size_t;
                    while i_ < len_ {
                        if *buf_.offset(i_ as isize) as ::core::ffi::c_int
                            == '\'' as ::core::ffi::c_int
                        {
                            ga_append(gap, '\'' as uint8_t);
                        }
                        ga_append(gap, *buf_.offset(i_ as isize) as uint8_t);
                        i_ = i_.wrapping_add(1);
                    }
                    ga_append(gap, '\'' as uint8_t);
                }
            }
            1 => {
                let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
                let mut numbuflen: size_t = vim_snprintf_safelen(
                    &raw mut numbuf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                        .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                        .wrapping_div(
                            (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                == 0) as ::core::ffi::c_int as size_t,
                        ),
                    b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                    (*tv).vval.v_number,
                );
                ga_concat_len(gap, &raw mut numbuf as *mut ::core::ffi::c_char, numbuflen);
            }
            6 => {
                let flt_: float_T = (*tv).vval.v_float;
                match flt_.classify() {
                    ::core::num::FpCategory::Nan => {
                        ga_concat_len(
                            gap,
                            b"str2float('nan')\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 17]>()
                                .wrapping_sub(1 as size_t),
                        );
                    }
                    ::core::num::FpCategory::Infinite => {
                        if flt_ < 0 as ::core::ffi::c_int as float_T {
                            ga_append(gap, '-' as uint8_t);
                        }
                        ga_concat_len(
                            gap,
                            b"str2float('inf')\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 17]>()
                                .wrapping_sub(1 as size_t),
                        );
                    }
                    _ => {
                        let mut numbuf_0: [::core::ffi::c_char; 65] = [0; 65];
                        let mut numbuflen_0: size_t = vim_snprintf_safelen(
                            &raw mut numbuf_0 as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                ),
                            b"%g\0".as_ptr() as *const ::core::ffi::c_char,
                            flt_,
                        );
                        ga_concat_len(
                            gap,
                            &raw mut numbuf_0 as *mut ::core::ffi::c_char,
                            numbuflen_0,
                        );
                    }
                }
            }
            10 => {
                let blob_: *const blob_T = (*tv).vval.v_blob;
                let len__0: ::core::ffi::c_int = tv_blob_len((*tv).vval.v_blob);
                if len__0 == 0 as ::core::ffi::c_int {
                    ga_concat_len(
                        gap,
                        b"0z\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    ga_grow(
                        gap,
                        2 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int * len__0
                            + (len__0 - 1 as ::core::ffi::c_int) / 4 as ::core::ffi::c_int,
                    );
                    ga_concat_len(
                        gap,
                        b"0z\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                    let mut numbuf_1: [::core::ffi::c_char; 65] = [0; 65];
                    let mut i__0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i__0 < len__0 {
                        if i__0 > 0 as ::core::ffi::c_int
                            && i__0 & 3 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                        {
                            ga_append(gap, '.' as uint8_t);
                        }
                        let mut numbuflen_1: size_t = vim_snprintf_safelen(
                            &raw mut numbuf_1 as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                ),
                            b"%02X\0".as_ptr() as *const ::core::ffi::c_char,
                            tv_blob_get(blob_, i__0) as ::core::ffi::c_int,
                        );
                        ga_concat_len(
                            gap,
                            &raw mut numbuf_1 as *mut ::core::ffi::c_char,
                            numbuflen_1,
                        );
                        i__0 += 1;
                    }
                }
            }
            3 => {
                let fun_: *const ::core::ffi::c_char = (*tv).vval.v_string;
                if fun_.is_null() {
                    internal_error(
                        b"string(): NULL function name\0".as_ptr() as *const ::core::ffi::c_char
                    );
                    ga_concat_len(
                        gap,
                        b"function(NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 14]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    let prefix_: *const ::core::ffi::c_char =
                        b"\0".as_ptr() as *const ::core::ffi::c_char;
                    ga_concat_len(
                        gap,
                        b"function(\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                            .wrapping_sub(1 as size_t),
                    );
                    let name_off: ::core::ffi::c_int = (*gap).ga_len;
                    ga_concat(gap, prefix_);
                    let buf__0: *const ::core::ffi::c_char = fun_;
                    if buf__0.is_null() {
                        ga_concat_len(
                            gap,
                            b"''\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        );
                    } else {
                        let len__1: size_t = strlen(fun_);
                        ga_grow(
                            gap,
                            (2 as size_t).wrapping_add(len__1).wrapping_add(memcnt(
                                buf__0 as *const ::core::ffi::c_void,
                                '\'' as ::core::ffi::c_char,
                                len__1,
                            )) as ::core::ffi::c_int,
                        );
                        ga_append(gap, '\'' as uint8_t);
                        let mut i__1: size_t = 0 as size_t;
                        while i__1 < len__1 {
                            if *buf__0.offset(i__1 as isize) as ::core::ffi::c_int
                                == '\'' as ::core::ffi::c_int
                            {
                                ga_append(gap, '\'' as uint8_t);
                            }
                            ga_append(gap, *buf__0.offset(i__1 as isize) as uint8_t);
                            i__1 = i__1.wrapping_add(1);
                        }
                        ga_append(gap, '\'' as uint8_t);
                    }
                    *((*gap).ga_data as *mut ::core::ffi::c_char).offset(name_off as isize) =
                        '\'' as ::core::ffi::c_char;
                    memcpy(
                        ((*gap).ga_data as *mut ::core::ffi::c_char)
                            .offset(name_off as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *mut ::core::ffi::c_void,
                        prefix_ as *const ::core::ffi::c_void,
                        strlen(prefix_),
                    );
                }
                if 0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                    ga_concat_len(
                        gap,
                        b", \0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                }
                if -1 as ::core::ffi::c_int as ptrdiff_t != -1 as ptrdiff_t {
                    ga_concat_len(
                        gap,
                        b", \0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                }
                ga_append(gap, ')' as uint8_t);
            }
            9 => {
                let pt: *mut partial_T = (*tv).vval.v_partial;
                let fun: *mut ::core::ffi::c_char = if pt.is_null() {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    partial_name(pt)
                };
                let prefix: *const ::core::ffi::c_char = if !fun.is_null()
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
                let fun__0: *const ::core::ffi::c_char = fun;
                if fun__0.is_null() {
                    internal_error(
                        b"string(): NULL function name\0".as_ptr() as *const ::core::ffi::c_char
                    );
                    ga_concat_len(
                        gap,
                        b"function(NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 14]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    let prefix__0: *const ::core::ffi::c_char = prefix;
                    ga_concat_len(
                        gap,
                        b"function(\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                            .wrapping_sub(1 as size_t),
                    );
                    let name_off_0: ::core::ffi::c_int = (*gap).ga_len;
                    ga_concat(gap, prefix__0);
                    let buf__1: *const ::core::ffi::c_char = fun__0;
                    if buf__1.is_null() {
                        ga_concat_len(
                            gap,
                            b"''\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        );
                    } else {
                        let len__2: size_t = strlen(fun__0);
                        ga_grow(
                            gap,
                            (2 as size_t).wrapping_add(len__2).wrapping_add(memcnt(
                                buf__1 as *const ::core::ffi::c_void,
                                '\'' as ::core::ffi::c_char,
                                len__2,
                            )) as ::core::ffi::c_int,
                        );
                        ga_append(gap, '\'' as uint8_t);
                        let mut i__2: size_t = 0 as size_t;
                        while i__2 < len__2 {
                            if *buf__1.offset(i__2 as isize) as ::core::ffi::c_int
                                == '\'' as ::core::ffi::c_int
                            {
                                ga_append(gap, '\'' as uint8_t);
                            }
                            ga_append(gap, *buf__1.offset(i__2 as isize) as uint8_t);
                            i__2 = i__2.wrapping_add(1);
                        }
                        ga_append(gap, '\'' as uint8_t);
                    }
                    *((*gap).ga_data as *mut ::core::ffi::c_char).offset(name_off_0 as isize) =
                        '\'' as ::core::ffi::c_char;
                    memcpy(
                        ((*gap).ga_data as *mut ::core::ffi::c_char)
                            .offset(name_off_0 as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *mut ::core::ffi::c_void,
                        prefix__0 as *const ::core::ffi::c_void,
                        strlen(prefix__0),
                    );
                }
                if (*mpstack).size == (*mpstack).capacity {
                    (*mpstack).capacity = if (*mpstack).capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                            .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                            .wrapping_div(
                                (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        (*mpstack).capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[MPConvStackVal; 8]>()
                            .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                            .wrapping_div(
                                (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    (*mpstack).items = (if (*mpstack).capacity
                        == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                            .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                            .wrapping_div(
                                (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if (*mpstack).items == &raw mut (*mpstack).init_array as *mut MPConvStackVal
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
                        if (*mpstack).items == &raw mut (*mpstack).init_array as *mut MPConvStackVal
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
                let c2rust_fresh22 = (*mpstack).size;
                (*mpstack).size = (*mpstack).size.wrapping_add(1);
                *(*mpstack).items.offset(c2rust_fresh22 as isize) = MPConvStackVal {
                    type_0: kMPConvPartial,
                    tv: tv,
                    saved_copyID: copyID - 1 as ::core::ffi::c_int,
                    data: C2Rust_Unnamed_0 {
                        p: C2Rust_Unnamed_2 {
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
                    ga_concat_len(
                        gap,
                        b"[]\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    let saved_copyID: ::core::ffi::c_int = tv_list_copyid((*tv).vval.v_list);
                    let te_csr_ret: ::core::ffi::c_int = _typval_encode_echo_check_self_reference(
                        gap,
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
                    ga_append(gap, '[' as uint8_t);
                    '_c2rust_label: {
                        if saved_copyID != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/encode.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                383 as ::core::ffi::c_uint,
                                b"int _typval_encode_echo_convert_one_value(garray_T *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
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
                    let c2rust_fresh23 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh23 as isize) = MPConvStackVal {
                        type_0: kMPConvList,
                        tv: tv,
                        saved_copyID: saved_copyID,
                        data: C2Rust_Unnamed_0 {
                            l: C2Rust_Unnamed_3 {
                                list: (*tv).vval.v_list,
                                li: tv_list_first((*tv).vval.v_list),
                            },
                        },
                    };
                }
            }
            7 => match (*tv).vval.v_bool as ::core::ffi::c_uint {
                1 | 0 => {
                    if (*tv).vval.v_bool as ::core::ffi::c_uint
                        == kBoolVarTrue as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        ga_concat_len(
                            gap,
                            b"v:true\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                .wrapping_sub(1 as size_t),
                        );
                    } else {
                        ga_concat_len(
                            gap,
                            b"v:false\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                                .wrapping_sub(1 as size_t),
                        );
                    }
                }
                _ => {}
            },
            8 => match (*tv).vval.v_special as ::core::ffi::c_uint {
                0 => {
                    ga_concat_len(
                        gap,
                        b"v:null\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                            .wrapping_sub(1 as size_t),
                    );
                }
                _ => {}
            },
            5 => {
                if (*tv).vval.v_dict.is_null()
                    || (*(*tv).vval.v_dict).dv_hashtab.ht_used == 0 as size_t
                {
                    ga_concat_len(
                        gap,
                        b"{}\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    let mut type_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    let mut val_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    's_1204: {
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
                                    == (*eval_msgpack_type_lists.ptr())[i as usize] as *mut list_T
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
                                        ga_concat_len(
                                            gap,
                                            b"v:null\0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                                .wrapping_sub(1 as size_t),
                                        );
                                        break '_typval_encode_stop_converting_one_item;
                                    }
                                    1 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_NUMBER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            if (*val_di).di_tv.vval.v_number != 0 {
                                                ga_concat_len(
                                                    gap,
                                                    b"v:true\0".as_ptr() as *const ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                                        .wrapping_sub(1 as size_t),
                                                );
                                            } else {
                                                ga_concat_len(
                                                    gap,
                                                    b"v:false\0".as_ptr() as *const ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                                                        .wrapping_sub(1 as size_t),
                                                );
                                            }
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
                                                        highest_bits =
                                                            (*highest_bits_li).li_tv.vval.v_number;
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
                                                            high_bits =
                                                                (*high_bits_li).li_tv.vval.v_number;
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
                                                                let mut numbuf_2: [::core::ffi::c_char; 65] = [0; 65];
                                                                let mut numbuflen_2: size_t = vim_snprintf_safelen(
                                                                    &raw mut numbuf_2 as *mut ::core::ffi::c_char,
                                                                    ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                                                        .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                                                        .wrapping_div(
                                                                            (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                                                                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                                                                == 0) as ::core::ffi::c_int as size_t,
                                                                        ),
                                                                    b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                                                                    number.wrapping_neg() as int64_t,
                                                                );
                                                                ga_concat_len(
                                                                    gap,
                                                                    &raw mut numbuf_2
                                                                        as *mut ::core::ffi::c_char,
                                                                    numbuflen_2,
                                                                );
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
                                            let flt__0: float_T = (*val_di).di_tv.vval.v_float;
                                            match flt__0.classify() {
                                                ::core::num::FpCategory::Nan => {
                                                    ga_concat_len(
                                                        gap,
                                                        b"str2float('nan')\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 17],
                                                        >(
                                                        )
                                                        .wrapping_sub(1 as size_t),
                                                    );
                                                }
                                                ::core::num::FpCategory::Infinite => {
                                                    if flt__0 < 0 as ::core::ffi::c_int as float_T {
                                                        ga_append(gap, '-' as uint8_t);
                                                    }
                                                    ga_concat_len(
                                                        gap,
                                                        b"str2float('inf')\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 17],
                                                        >(
                                                        )
                                                        .wrapping_sub(1 as size_t),
                                                    );
                                                }
                                                _ => {
                                                    let mut numbuf_3: [::core::ffi::c_char; 65] =
                                                        [0; 65];
                                                    let mut numbuflen_3: size_t =
                                                        vim_snprintf_safelen(
                                                            &raw mut numbuf_3
                                                                as *mut ::core::ffi::c_char,
                                                            ::core::mem::size_of::<
                                                                [::core::ffi::c_char; 65],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                ::core::ffi::c_char,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [::core::ffi::c_char; 65],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        ::core::ffi::c_char,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as size_t,
                                                            ),
                                                            b"%g\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                            flt__0,
                                                        );
                                                    ga_concat_len(
                                                        gap,
                                                        &raw mut numbuf_3
                                                            as *mut ::core::ffi::c_char,
                                                        numbuflen_3,
                                                    );
                                                }
                                            }
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    4 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let mut len: size_t = 0;
                                            let mut buf: *mut ::core::ffi::c_char =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            if encode_vim_list_to_buf(
                                                (*val_di).di_tv.vval.v_list,
                                                &raw mut len,
                                                &raw mut buf,
                                            ) {
                                                let buf__2: *const ::core::ffi::c_char = buf;
                                                if buf__2.is_null() {
                                                    ga_concat_len(
                                                        gap,
                                                        b"''\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 3],
                                                        >(
                                                        )
                                                        .wrapping_sub(1 as size_t),
                                                    );
                                                } else {
                                                    let len__3: size_t = len;
                                                    ga_grow(
                                                        gap,
                                                        (2 as size_t)
                                                            .wrapping_add(len__3)
                                                            .wrapping_add(memcnt(
                                                                buf__2
                                                                    as *const ::core::ffi::c_void,
                                                                '\'' as ::core::ffi::c_char,
                                                                len__3,
                                                            ))
                                                            as ::core::ffi::c_int,
                                                    );
                                                    ga_append(gap, '\'' as uint8_t);
                                                    let mut i__3: size_t = 0 as size_t;
                                                    while i__3 < len__3 {
                                                        if *buf__2.offset(i__3 as isize)
                                                            as ::core::ffi::c_int
                                                            == '\'' as ::core::ffi::c_int
                                                        {
                                                            ga_append(gap, '\'' as uint8_t);
                                                        }
                                                        ga_append(
                                                            gap,
                                                            *buf__2.offset(i__3 as isize)
                                                                as uint8_t,
                                                        );
                                                        i__3 = i__3.wrapping_add(1);
                                                    }
                                                    ga_append(gap, '\'' as uint8_t);
                                                }
                                                xfree(buf as *mut ::core::ffi::c_void);
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    5 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let saved_copyID_0: ::core::ffi::c_int =
                                                tv_list_copyid((*val_di).di_tv.vval.v_list);
                                            let te_csr_ret_0: ::core::ffi::c_int =
                                                _typval_encode_echo_check_self_reference(
                                                    gap,
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
                                            ga_append(gap, '[' as uint8_t);
                                            '_c2rust_label_0: {
                                                if saved_copyID_0 != copyID
                                                    && saved_copyID_0
                                                        != copyID - 1 as ::core::ffi::c_int
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        b"src/nvim/eval/encode.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        532 as ::core::ffi::c_uint,
                                                        b"int _typval_encode_echo_convert_one_value(garray_T *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            if (*mpstack).size == (*mpstack).capacity {
                                                (*mpstack).capacity = if (*mpstack).capacity
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
                                                    (*mpstack).capacity << 1 as ::core::ffi::c_int
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
                                                (*mpstack).items = (if (*mpstack).capacity
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
                                                    if (*mpstack).items
                                                        == &raw mut (*mpstack).init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        (*mpstack).items as *mut ::core::ffi::c_void
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
                                                            (*mpstack).capacity.wrapping_mul(
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
                                            let c2rust_fresh24 = (*mpstack).size;
                                            (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                            *(*mpstack).items.offset(c2rust_fresh24 as isize) =
                                                MPConvStackVal {
                                                    type_0: kMPConvList,
                                                    tv: tv,
                                                    saved_copyID: saved_copyID_0,
                                                    data: C2Rust_Unnamed_0 {
                                                        l: C2Rust_Unnamed_3 {
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
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let val_list_0: *mut list_T =
                                                (*val_di).di_tv.vval.v_list;
                                            if val_list_0.is_null()
                                                || tv_list_len(val_list_0)
                                                    == 0 as ::core::ffi::c_int
                                            {
                                                ga_concat_len(
                                                    gap,
                                                    b"{}\0".as_ptr() as *const ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                        .wrapping_sub(1 as size_t),
                                                );
                                                break '_typval_encode_stop_converting_one_item;
                                            } else {
                                                let l_: *const list_T = val_list_0;
                                                's_1122: {
                                                    if !l_.is_null() {
                                                        let mut li: *const listitem_T =
                                                            (*l_).lv_first;
                                                        loop {
                                                            if li.is_null() {
                                                                break 's_1122;
                                                            }
                                                            if (*li).li_tv.v_type
                                                                as ::core::ffi::c_uint
                                                                != VAR_LIST as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                || tv_list_len(
                                                                    (*li).li_tv.vval.v_list,
                                                                ) != 2 as ::core::ffi::c_int
                                                            {
                                                                break 's_1204;
                                                            }
                                                            li = (*li).li_next;
                                                        }
                                                    }
                                                }
                                                let saved_copyID_1: ::core::ffi::c_int =
                                                    tv_list_copyid((*val_di).di_tv.vval.v_list);
                                                let te_csr_ret_1: ::core::ffi::c_int =
                                                    _typval_encode_echo_check_self_reference(
                                                        gap,
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
                                                ga_append(gap, '{' as uint8_t);
                                                '_c2rust_label_1: {
                                                    if saved_copyID_1 != copyID
                                                        && saved_copyID_1
                                                            != copyID - 1 as ::core::ffi::c_int
                                                    {
                                                    } else {
                                                        __assert_fail(
                                                            b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            b"src/nvim/eval/encode.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            566 as ::core::ffi::c_uint,
                                                            b"int _typval_encode_echo_convert_one_value(garray_T *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
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
                                                let c2rust_fresh25 = (*mpstack).size;
                                                (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                                *(*mpstack).items.offset(c2rust_fresh25 as isize) =
                                                    MPConvStackVal {
                                                        type_0: kMPConvPairs,
                                                        tv: tv,
                                                        saved_copyID: saved_copyID_1,
                                                        data: C2Rust_Unnamed_0 {
                                                            l: C2Rust_Unnamed_3 {
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
                                                tv_list_len(val_list_1) != 2 as ::core::ffi::c_int
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
                    let te_csr_ret_2: ::core::ffi::c_int = _typval_encode_echo_check_self_reference(
                        gap,
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
                    ga_append(gap, '{' as uint8_t);
                    '_c2rust_label_2: {
                        if saved_copyID_2 != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/encode.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                614 as ::core::ffi::c_uint,
                                b"int _typval_encode_echo_convert_one_value(garray_T *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
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
                    let c2rust_fresh26 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh26 as isize) = MPConvStackVal {
                        type_0: kMPConvDict,
                        tv: tv,
                        saved_copyID: saved_copyID_2,
                        data: C2Rust_Unnamed_0 {
                            d: C2Rust_Unnamed_4 {
                                dict: (*tv).vval.v_dict,
                                dictp: &raw mut (*tv).vval.v_dict,
                                hi: (*(*tv).vval.v_dict).dv_hashtab.ht_array,
                                todo: (*(*tv).vval.v_dict).dv_hashtab.ht_used,
                            },
                        },
                    };
                }
            }
            0 => {
                internal_error(b"_typval_encode_echo_convert_one_value()\0".as_ptr()
                    as *const ::core::ffi::c_char);
                return FAIL;
            }
            _ => {}
        }
    }
    return OK;
}
pub unsafe extern "C" fn encode_vim_to_echo(
    gap: *mut garray_T,
    top_tv: *mut typval_T,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let copyID: ::core::ffi::c_int = get_copyID();
    let mut mpstack: MPConvStack = MPConvStack {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<MPConvStackVal>(),
        init_array: [MPConvStackVal {
            type_0: kMPConvDict,
            tv: ::core::ptr::null_mut::<typval_T>(),
            saved_copyID: 0,
            data: C2Rust_Unnamed_0 {
                d: C2Rust_Unnamed_4 {
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
        if _typval_encode_echo_convert_one_value(
            gap,
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
                            ga_append(gap, '}' as uint8_t);
                            continue;
                        } else {
                            if (*cur_mpsv).data.d.todo
                                != (*(*cur_mpsv).data.d.dict).dv_hashtab.ht_used
                            {
                                ga_concat_len(
                                    gap,
                                    b", \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            }
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
                            let buf_: *const ::core::ffi::c_char = (&raw mut (*di).di_key
                                as *mut ::core::ffi::c_char)
                                .offset(0 as ::core::ffi::c_int as isize);
                            if buf_.is_null() {
                                ga_concat_len(
                                    gap,
                                    b"''\0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            } else {
                                let len_: size_t = strlen(
                                    (&raw mut (*di).di_key as *mut ::core::ffi::c_char)
                                        .offset(0 as ::core::ffi::c_int as isize),
                                );
                                ga_grow(
                                    gap,
                                    (2 as size_t).wrapping_add(len_).wrapping_add(memcnt(
                                        buf_ as *const ::core::ffi::c_void,
                                        '\'' as ::core::ffi::c_char,
                                        len_,
                                    )) as ::core::ffi::c_int,
                                );
                                ga_append(gap, '\'' as uint8_t);
                                let mut i_: size_t = 0 as size_t;
                                while i_ < len_ {
                                    if *buf_.offset(i_ as isize) as ::core::ffi::c_int
                                        == '\'' as ::core::ffi::c_int
                                    {
                                        ga_append(gap, '\'' as uint8_t);
                                    }
                                    ga_append(gap, *buf_.offset(i_ as isize) as uint8_t);
                                    i_ = i_.wrapping_add(1);
                                }
                                ga_append(gap, '\'' as uint8_t);
                            }
                            ga_concat_len(
                                gap,
                                b": \0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            );
                            tv = &raw mut (*di).di_tv;
                        }
                    }
                    1 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            ga_append(gap, ']' as uint8_t);
                            continue;
                        } else {
                            if (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list) {
                                ga_concat_len(
                                    gap,
                                    b", \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            }
                            tv = &raw mut (*(*cur_mpsv).data.l.li).li_tv;
                            (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                        }
                    }
                    2 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            ga_append(gap, '}' as uint8_t);
                            continue;
                        } else {
                            if (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list) {
                                ga_concat_len(
                                    gap,
                                    b", \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            }
                            let kv_pair: *const list_T = (*(*cur_mpsv).data.l.li).li_tv.vval.v_list;
                            if _typval_encode_echo_convert_one_value(
                                gap,
                                &raw mut mpstack,
                                cur_mpsv,
                                &raw mut (*(tv_list_first
                                    as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                    kv_pair,
                                ))
                                .li_tv,
                                copyID,
                                objname,
                            ) == FAIL
                            {
                                break '_encode_vim_to__error_ret;
                            }
                            ga_concat_len(
                                gap,
                                b": \0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            );
                            tv = &raw mut (*(tv_list_last
                                as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                kv_pair,
                            ))
                            .li_tv;
                            (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                        }
                    }
                    3 => {
                        let pt: *mut partial_T = (*cur_mpsv).data.p.pt;
                        tv = (*cur_mpsv).tv;
                        match (*cur_mpsv).data.p.stage as ::core::ffi::c_uint {
                            0 => {
                                if (if pt.is_null() {
                                    0 as ::core::ffi::c_int
                                } else {
                                    (*pt).pt_argc
                                }) != 0 as ::core::ffi::c_int
                                {
                                    ga_concat_len(
                                        gap,
                                        b", \0".as_ptr() as *const ::core::ffi::c_char,
                                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                            .wrapping_sub(1 as size_t),
                                    );
                                }
                                (*cur_mpsv).data.p.stage = kMPConvPartialSelf;
                                if !pt.is_null() && (*pt).pt_argc > 0 as ::core::ffi::c_int {
                                    ga_append(gap, '[' as uint8_t);
                                    if mpstack.size == mpstack.capacity {
                                        mpstack.capacity = if mpstack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            mpstack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as size_t,
                                                )
                                        };
                                        mpstack.items = (if mpstack.capacity
                                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
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
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
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
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    )),
                                                    mpstack.items as *const ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            } else {
                                                xrealloc(
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            }
                                        })
                                            as *mut MPConvStackVal;
                                    } else {
                                    };
                                    let c2rust_fresh19 = mpstack.size;
                                    mpstack.size = mpstack.size.wrapping_add(1);
                                    *mpstack.items.offset(c2rust_fresh19 as isize) =
                                        MPConvStackVal {
                                            type_0: kMPConvPartialList,
                                            tv: ::core::ptr::null_mut::<typval_T>(),
                                            saved_copyID: copyID - 1 as ::core::ffi::c_int,
                                            data: C2Rust_Unnamed_0 {
                                                a: C2Rust_Unnamed_1 {
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
                                if !dict.is_null() {
                                    if (*dict).dv_hashtab.ht_used as ptrdiff_t != -1 as ptrdiff_t {
                                        ga_concat_len(
                                            gap,
                                            b", \0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                .wrapping_sub(1 as size_t),
                                        );
                                    }
                                    if (*dict).dv_hashtab.ht_used == 0 as size_t {
                                        ga_concat_len(
                                            gap,
                                            b"{}\0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                .wrapping_sub(1 as size_t),
                                        );
                                        continue;
                                    } else {
                                        let saved_copyID: ::core::ffi::c_int = (*dict).dv_copyID;
                                        let te_csr_ret: ::core::ffi::c_int =
                                            _typval_encode_echo_check_self_reference(
                                                gap,
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
                                            ga_append(gap, '{' as uint8_t);
                                            '_c2rust_label: {
                                                if saved_copyID != copyID
                                                    && saved_copyID
                                                        != copyID - 1 as ::core::ffi::c_int
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        b"src/nvim/eval/encode.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        789 as ::core::ffi::c_uint,
                                                        b"int encode_vim_to_echo(garray_T *const, typval_T *const, const char *const)\0"
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
                                            let c2rust_fresh20 = mpstack.size;
                                            mpstack.size = mpstack.size.wrapping_add(1);
                                            *mpstack.items.offset(c2rust_fresh20 as isize) =
                                                MPConvStackVal {
                                                    type_0: kMPConvDict,
                                                    tv: ::core::ptr::null_mut::<typval_T>(),
                                                    saved_copyID: saved_copyID,
                                                    data: C2Rust_Unnamed_0 {
                                                        d: C2Rust_Unnamed_4 {
                                                            dict: dict,
                                                            dictp: &raw mut (*pt).pt_dict,
                                                            hi: (*dict).dv_hashtab.ht_array,
                                                            todo: (*dict).dv_hashtab.ht_used,
                                                        },
                                                    },
                                                };
                                            continue;
                                        }
                                    }
                                } else {
                                    if -1 as ::core::ffi::c_int as ptrdiff_t != -1 as ptrdiff_t {
                                        ga_concat_len(
                                            gap,
                                            b", \0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                .wrapping_sub(1 as size_t),
                                        );
                                    }
                                    continue;
                                }
                            }
                            2 => {
                                ga_append(gap, ')' as uint8_t);
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
                            ga_append(gap, ']' as uint8_t);
                            continue;
                        } else {
                            if (*cur_mpsv).data.a.argv != (*cur_mpsv).data.a.arg {
                                ga_concat_len(
                                    gap,
                                    b", \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            }
                            let c2rust_fresh21 = (*cur_mpsv).data.a.arg;
                            (*cur_mpsv).data.a.arg = (*cur_mpsv).data.a.arg.offset(1);
                            tv = c2rust_fresh21;
                            (*cur_mpsv).data.a.todo = (*cur_mpsv).data.a.todo.wrapping_sub(1);
                        }
                    }
                    _ => {}
                }
                '_c2rust_label_0: {
                    if !tv.is_null() {
                    } else {
                        __assert_fail(
                            b"tv != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/eval/encode.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            829 as ::core::ffi::c_uint,
                            b"int encode_vim_to_echo(garray_T *const, typval_T *const, const char *const)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if _typval_encode_echo_convert_one_value(
                    gap,
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
pub static _typval_encode_string_nodict_var: GlobalCell<*const dict_T> =
    GlobalCell::new(::core::ptr::null::<dict_T>());
#[inline(always)]
unsafe extern "C" fn _typval_encode_string_check_self_reference(
    gap: *mut garray_T,
    val: *mut ::core::ffi::c_void,
    val_copyID: *mut ::core::ffi::c_int,
    mpstack: *const MPConvStack,
    copyID: ::core::ffi::c_int,
    conv_type: MPConvStackValType,
    _objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *val_copyID == copyID {
        if !did_echo_string_emsg.get() {
            did_echo_string_emsg.set(true_0 != 0);
            emsg(gettext(
                b"E724: unable to correctly dump variable with self-referencing container\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            ));
        }
        let mut ebuf: [::core::ffi::c_char; 72] = [0; 72];
        let mut backref: size_t = 0 as size_t;
        while backref < (*mpstack).size {
            let mpval: MPConvStackVal = *(*mpstack).items.offset(backref as isize);
            if mpval.type_0 as ::core::ffi::c_uint == conv_type as ::core::ffi::c_uint {
                if conv_type as ::core::ffi::c_uint
                    == kMPConvDict as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if mpval.data.d.dict as *mut ::core::ffi::c_void == val {
                        break;
                    }
                } else if conv_type as ::core::ffi::c_uint
                    == kMPConvList as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if mpval.data.l.list as *mut ::core::ffi::c_void == val {
                        break;
                    }
                }
            }
            backref = backref.wrapping_add(1);
        }
        vim_snprintf(
            &raw mut ebuf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 72]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 72]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as size_t,
                ),
            b"{E724@%zu}\0".as_ptr() as *const ::core::ffi::c_char,
            backref,
        );
        ga_concat(
            gap,
            (&raw mut ebuf as *mut ::core::ffi::c_char).offset(0 as ::core::ffi::c_int as isize),
        );
        return OK;
    }
    *val_copyID = copyID;
    return NOTDONE;
}
unsafe extern "C" fn _typval_encode_string_convert_one_value(
    gap: *mut garray_T,
    mpstack: *mut MPConvStack,
    _cur_mpsv: *mut MPConvStackVal,
    tv: *mut typval_T,
    copyID: ::core::ffi::c_int,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    '_typval_encode_stop_converting_one_item: {
        match (*tv).v_type as ::core::ffi::c_uint {
            2 => {
                let buf_: *const ::core::ffi::c_char = (*tv).vval.v_string;
                if buf_.is_null() {
                    ga_concat_len(
                        gap,
                        b"''\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    let len_: size_t = tv_strlen(tv);
                    ga_grow(
                        gap,
                        (2 as size_t).wrapping_add(len_).wrapping_add(memcnt(
                            buf_ as *const ::core::ffi::c_void,
                            '\'' as ::core::ffi::c_char,
                            len_,
                        )) as ::core::ffi::c_int,
                    );
                    ga_append(gap, '\'' as uint8_t);
                    let mut i_: size_t = 0 as size_t;
                    while i_ < len_ {
                        if *buf_.offset(i_ as isize) as ::core::ffi::c_int
                            == '\'' as ::core::ffi::c_int
                        {
                            ga_append(gap, '\'' as uint8_t);
                        }
                        ga_append(gap, *buf_.offset(i_ as isize) as uint8_t);
                        i_ = i_.wrapping_add(1);
                    }
                    ga_append(gap, '\'' as uint8_t);
                }
            }
            1 => {
                let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
                let mut numbuflen: size_t = vim_snprintf_safelen(
                    &raw mut numbuf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                        .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                        .wrapping_div(
                            (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                == 0) as ::core::ffi::c_int as size_t,
                        ),
                    b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                    (*tv).vval.v_number,
                );
                ga_concat_len(gap, &raw mut numbuf as *mut ::core::ffi::c_char, numbuflen);
            }
            6 => {
                let flt_: float_T = (*tv).vval.v_float;
                match flt_.classify() {
                    ::core::num::FpCategory::Nan => {
                        ga_concat_len(
                            gap,
                            b"str2float('nan')\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 17]>()
                                .wrapping_sub(1 as size_t),
                        );
                    }
                    ::core::num::FpCategory::Infinite => {
                        if flt_ < 0 as ::core::ffi::c_int as float_T {
                            ga_append(gap, '-' as uint8_t);
                        }
                        ga_concat_len(
                            gap,
                            b"str2float('inf')\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 17]>()
                                .wrapping_sub(1 as size_t),
                        );
                    }
                    _ => {
                        let mut numbuf_0: [::core::ffi::c_char; 65] = [0; 65];
                        let mut numbuflen_0: size_t = vim_snprintf_safelen(
                            &raw mut numbuf_0 as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                ),
                            b"%g\0".as_ptr() as *const ::core::ffi::c_char,
                            flt_,
                        );
                        ga_concat_len(
                            gap,
                            &raw mut numbuf_0 as *mut ::core::ffi::c_char,
                            numbuflen_0,
                        );
                    }
                }
            }
            10 => {
                let blob_: *const blob_T = (*tv).vval.v_blob;
                let len__0: ::core::ffi::c_int = tv_blob_len((*tv).vval.v_blob);
                if len__0 == 0 as ::core::ffi::c_int {
                    ga_concat_len(
                        gap,
                        b"0z\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    ga_grow(
                        gap,
                        2 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int * len__0
                            + (len__0 - 1 as ::core::ffi::c_int) / 4 as ::core::ffi::c_int,
                    );
                    ga_concat_len(
                        gap,
                        b"0z\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                    let mut numbuf_1: [::core::ffi::c_char; 65] = [0; 65];
                    let mut i__0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i__0 < len__0 {
                        if i__0 > 0 as ::core::ffi::c_int
                            && i__0 & 3 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                        {
                            ga_append(gap, '.' as uint8_t);
                        }
                        let mut numbuflen_1: size_t = vim_snprintf_safelen(
                            &raw mut numbuf_1 as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                ),
                            b"%02X\0".as_ptr() as *const ::core::ffi::c_char,
                            tv_blob_get(blob_, i__0) as ::core::ffi::c_int,
                        );
                        ga_concat_len(
                            gap,
                            &raw mut numbuf_1 as *mut ::core::ffi::c_char,
                            numbuflen_1,
                        );
                        i__0 += 1;
                    }
                }
            }
            3 => {
                let fun_: *const ::core::ffi::c_char = (*tv).vval.v_string;
                if fun_.is_null() {
                    internal_error(
                        b"string(): NULL function name\0".as_ptr() as *const ::core::ffi::c_char
                    );
                    ga_concat_len(
                        gap,
                        b"function(NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 14]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    let prefix_: *const ::core::ffi::c_char =
                        b"\0".as_ptr() as *const ::core::ffi::c_char;
                    ga_concat_len(
                        gap,
                        b"function(\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                            .wrapping_sub(1 as size_t),
                    );
                    let name_off: ::core::ffi::c_int = (*gap).ga_len;
                    ga_concat(gap, prefix_);
                    let buf__0: *const ::core::ffi::c_char = fun_;
                    if buf__0.is_null() {
                        ga_concat_len(
                            gap,
                            b"''\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        );
                    } else {
                        let len__1: size_t = strlen(fun_);
                        ga_grow(
                            gap,
                            (2 as size_t).wrapping_add(len__1).wrapping_add(memcnt(
                                buf__0 as *const ::core::ffi::c_void,
                                '\'' as ::core::ffi::c_char,
                                len__1,
                            )) as ::core::ffi::c_int,
                        );
                        ga_append(gap, '\'' as uint8_t);
                        let mut i__1: size_t = 0 as size_t;
                        while i__1 < len__1 {
                            if *buf__0.offset(i__1 as isize) as ::core::ffi::c_int
                                == '\'' as ::core::ffi::c_int
                            {
                                ga_append(gap, '\'' as uint8_t);
                            }
                            ga_append(gap, *buf__0.offset(i__1 as isize) as uint8_t);
                            i__1 = i__1.wrapping_add(1);
                        }
                        ga_append(gap, '\'' as uint8_t);
                    }
                    *((*gap).ga_data as *mut ::core::ffi::c_char).offset(name_off as isize) =
                        '\'' as ::core::ffi::c_char;
                    memcpy(
                        ((*gap).ga_data as *mut ::core::ffi::c_char)
                            .offset(name_off as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *mut ::core::ffi::c_void,
                        prefix_ as *const ::core::ffi::c_void,
                        strlen(prefix_),
                    );
                }
                if 0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                    ga_concat_len(
                        gap,
                        b", \0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                }
                if -1 as ::core::ffi::c_int as ptrdiff_t != -1 as ptrdiff_t {
                    ga_concat_len(
                        gap,
                        b", \0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                }
                ga_append(gap, ')' as uint8_t);
            }
            9 => {
                let pt: *mut partial_T = (*tv).vval.v_partial;
                let fun: *mut ::core::ffi::c_char = if pt.is_null() {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    partial_name(pt)
                };
                let prefix: *const ::core::ffi::c_char = if !fun.is_null()
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
                let fun__0: *const ::core::ffi::c_char = fun;
                if fun__0.is_null() {
                    internal_error(
                        b"string(): NULL function name\0".as_ptr() as *const ::core::ffi::c_char
                    );
                    ga_concat_len(
                        gap,
                        b"function(NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 14]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    let prefix__0: *const ::core::ffi::c_char = prefix;
                    ga_concat_len(
                        gap,
                        b"function(\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                            .wrapping_sub(1 as size_t),
                    );
                    let name_off_0: ::core::ffi::c_int = (*gap).ga_len;
                    ga_concat(gap, prefix__0);
                    let buf__1: *const ::core::ffi::c_char = fun__0;
                    if buf__1.is_null() {
                        ga_concat_len(
                            gap,
                            b"''\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        );
                    } else {
                        let len__2: size_t = strlen(fun__0);
                        ga_grow(
                            gap,
                            (2 as size_t).wrapping_add(len__2).wrapping_add(memcnt(
                                buf__1 as *const ::core::ffi::c_void,
                                '\'' as ::core::ffi::c_char,
                                len__2,
                            )) as ::core::ffi::c_int,
                        );
                        ga_append(gap, '\'' as uint8_t);
                        let mut i__2: size_t = 0 as size_t;
                        while i__2 < len__2 {
                            if *buf__1.offset(i__2 as isize) as ::core::ffi::c_int
                                == '\'' as ::core::ffi::c_int
                            {
                                ga_append(gap, '\'' as uint8_t);
                            }
                            ga_append(gap, *buf__1.offset(i__2 as isize) as uint8_t);
                            i__2 = i__2.wrapping_add(1);
                        }
                        ga_append(gap, '\'' as uint8_t);
                    }
                    *((*gap).ga_data as *mut ::core::ffi::c_char).offset(name_off_0 as isize) =
                        '\'' as ::core::ffi::c_char;
                    memcpy(
                        ((*gap).ga_data as *mut ::core::ffi::c_char)
                            .offset(name_off_0 as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *mut ::core::ffi::c_void,
                        prefix__0 as *const ::core::ffi::c_void,
                        strlen(prefix__0),
                    );
                }
                if (*mpstack).size == (*mpstack).capacity {
                    (*mpstack).capacity = if (*mpstack).capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                            .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                            .wrapping_div(
                                (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        (*mpstack).capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[MPConvStackVal; 8]>()
                            .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                            .wrapping_div(
                                (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    (*mpstack).items = (if (*mpstack).capacity
                        == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                            .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                            .wrapping_div(
                                (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if (*mpstack).items == &raw mut (*mpstack).init_array as *mut MPConvStackVal
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
                        if (*mpstack).items == &raw mut (*mpstack).init_array as *mut MPConvStackVal
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
                let c2rust_fresh33 = (*mpstack).size;
                (*mpstack).size = (*mpstack).size.wrapping_add(1);
                *(*mpstack).items.offset(c2rust_fresh33 as isize) = MPConvStackVal {
                    type_0: kMPConvPartial,
                    tv: tv,
                    saved_copyID: copyID - 1 as ::core::ffi::c_int,
                    data: C2Rust_Unnamed_0 {
                        p: C2Rust_Unnamed_2 {
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
                    ga_concat_len(
                        gap,
                        b"[]\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    let saved_copyID: ::core::ffi::c_int = tv_list_copyid((*tv).vval.v_list);
                    let te_csr_ret: ::core::ffi::c_int = _typval_encode_string_check_self_reference(
                        gap,
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
                    ga_append(gap, '[' as uint8_t);
                    '_c2rust_label: {
                        if saved_copyID != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/encode.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                383 as ::core::ffi::c_uint,
                                b"int _typval_encode_string_convert_one_value(garray_T *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
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
                    let c2rust_fresh34 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh34 as isize) = MPConvStackVal {
                        type_0: kMPConvList,
                        tv: tv,
                        saved_copyID: saved_copyID,
                        data: C2Rust_Unnamed_0 {
                            l: C2Rust_Unnamed_3 {
                                list: (*tv).vval.v_list,
                                li: tv_list_first((*tv).vval.v_list),
                            },
                        },
                    };
                }
            }
            7 => match (*tv).vval.v_bool as ::core::ffi::c_uint {
                1 | 0 => {
                    if (*tv).vval.v_bool as ::core::ffi::c_uint
                        == kBoolVarTrue as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        ga_concat_len(
                            gap,
                            b"v:true\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                .wrapping_sub(1 as size_t),
                        );
                    } else {
                        ga_concat_len(
                            gap,
                            b"v:false\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                                .wrapping_sub(1 as size_t),
                        );
                    }
                }
                _ => {}
            },
            8 => match (*tv).vval.v_special as ::core::ffi::c_uint {
                0 => {
                    ga_concat_len(
                        gap,
                        b"v:null\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                            .wrapping_sub(1 as size_t),
                    );
                }
                _ => {}
            },
            5 => {
                if (*tv).vval.v_dict.is_null()
                    || (*(*tv).vval.v_dict).dv_hashtab.ht_used == 0 as size_t
                {
                    ga_concat_len(
                        gap,
                        b"{}\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    let mut type_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    let mut val_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    's_1204: {
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
                                    == (*eval_msgpack_type_lists.ptr())[i as usize] as *mut list_T
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
                                        ga_concat_len(
                                            gap,
                                            b"v:null\0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                                .wrapping_sub(1 as size_t),
                                        );
                                        break '_typval_encode_stop_converting_one_item;
                                    }
                                    1 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_NUMBER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            if (*val_di).di_tv.vval.v_number != 0 {
                                                ga_concat_len(
                                                    gap,
                                                    b"v:true\0".as_ptr() as *const ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                                        .wrapping_sub(1 as size_t),
                                                );
                                            } else {
                                                ga_concat_len(
                                                    gap,
                                                    b"v:false\0".as_ptr() as *const ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                                                        .wrapping_sub(1 as size_t),
                                                );
                                            }
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
                                                        highest_bits =
                                                            (*highest_bits_li).li_tv.vval.v_number;
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
                                                            high_bits =
                                                                (*high_bits_li).li_tv.vval.v_number;
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
                                                                let mut numbuf_2: [::core::ffi::c_char; 65] = [0; 65];
                                                                let mut numbuflen_2: size_t = vim_snprintf_safelen(
                                                                    &raw mut numbuf_2 as *mut ::core::ffi::c_char,
                                                                    ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                                                        .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                                                        .wrapping_div(
                                                                            (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                                                                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                                                                == 0) as ::core::ffi::c_int as size_t,
                                                                        ),
                                                                    b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                                                                    number.wrapping_neg() as int64_t,
                                                                );
                                                                ga_concat_len(
                                                                    gap,
                                                                    &raw mut numbuf_2
                                                                        as *mut ::core::ffi::c_char,
                                                                    numbuflen_2,
                                                                );
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
                                            let flt__0: float_T = (*val_di).di_tv.vval.v_float;
                                            match flt__0.classify() {
                                                ::core::num::FpCategory::Nan => {
                                                    ga_concat_len(
                                                        gap,
                                                        b"str2float('nan')\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 17],
                                                        >(
                                                        )
                                                        .wrapping_sub(1 as size_t),
                                                    );
                                                }
                                                ::core::num::FpCategory::Infinite => {
                                                    if flt__0 < 0 as ::core::ffi::c_int as float_T {
                                                        ga_append(gap, '-' as uint8_t);
                                                    }
                                                    ga_concat_len(
                                                        gap,
                                                        b"str2float('inf')\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 17],
                                                        >(
                                                        )
                                                        .wrapping_sub(1 as size_t),
                                                    );
                                                }
                                                _ => {
                                                    let mut numbuf_3: [::core::ffi::c_char; 65] =
                                                        [0; 65];
                                                    let mut numbuflen_3: size_t =
                                                        vim_snprintf_safelen(
                                                            &raw mut numbuf_3
                                                                as *mut ::core::ffi::c_char,
                                                            ::core::mem::size_of::<
                                                                [::core::ffi::c_char; 65],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                ::core::ffi::c_char,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [::core::ffi::c_char; 65],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        ::core::ffi::c_char,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as size_t,
                                                            ),
                                                            b"%g\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                            flt__0,
                                                        );
                                                    ga_concat_len(
                                                        gap,
                                                        &raw mut numbuf_3
                                                            as *mut ::core::ffi::c_char,
                                                        numbuflen_3,
                                                    );
                                                }
                                            }
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    4 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let mut len: size_t = 0;
                                            let mut buf: *mut ::core::ffi::c_char =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            if encode_vim_list_to_buf(
                                                (*val_di).di_tv.vval.v_list,
                                                &raw mut len,
                                                &raw mut buf,
                                            ) {
                                                let buf__2: *const ::core::ffi::c_char = buf;
                                                if buf__2.is_null() {
                                                    ga_concat_len(
                                                        gap,
                                                        b"''\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 3],
                                                        >(
                                                        )
                                                        .wrapping_sub(1 as size_t),
                                                    );
                                                } else {
                                                    let len__3: size_t = len;
                                                    ga_grow(
                                                        gap,
                                                        (2 as size_t)
                                                            .wrapping_add(len__3)
                                                            .wrapping_add(memcnt(
                                                                buf__2
                                                                    as *const ::core::ffi::c_void,
                                                                '\'' as ::core::ffi::c_char,
                                                                len__3,
                                                            ))
                                                            as ::core::ffi::c_int,
                                                    );
                                                    ga_append(gap, '\'' as uint8_t);
                                                    let mut i__3: size_t = 0 as size_t;
                                                    while i__3 < len__3 {
                                                        if *buf__2.offset(i__3 as isize)
                                                            as ::core::ffi::c_int
                                                            == '\'' as ::core::ffi::c_int
                                                        {
                                                            ga_append(gap, '\'' as uint8_t);
                                                        }
                                                        ga_append(
                                                            gap,
                                                            *buf__2.offset(i__3 as isize)
                                                                as uint8_t,
                                                        );
                                                        i__3 = i__3.wrapping_add(1);
                                                    }
                                                    ga_append(gap, '\'' as uint8_t);
                                                }
                                                xfree(buf as *mut ::core::ffi::c_void);
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    5 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let saved_copyID_0: ::core::ffi::c_int =
                                                tv_list_copyid((*val_di).di_tv.vval.v_list);
                                            let te_csr_ret_0: ::core::ffi::c_int =
                                                _typval_encode_string_check_self_reference(
                                                    gap,
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
                                            ga_append(gap, '[' as uint8_t);
                                            '_c2rust_label_0: {
                                                if saved_copyID_0 != copyID
                                                    && saved_copyID_0
                                                        != copyID - 1 as ::core::ffi::c_int
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        b"src/nvim/eval/encode.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        532 as ::core::ffi::c_uint,
                                                        b"int _typval_encode_string_convert_one_value(garray_T *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            if (*mpstack).size == (*mpstack).capacity {
                                                (*mpstack).capacity = if (*mpstack).capacity
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
                                                    (*mpstack).capacity << 1 as ::core::ffi::c_int
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
                                                (*mpstack).items = (if (*mpstack).capacity
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
                                                    if (*mpstack).items
                                                        == &raw mut (*mpstack).init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        (*mpstack).items as *mut ::core::ffi::c_void
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
                                                            (*mpstack).capacity.wrapping_mul(
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
                                            let c2rust_fresh35 = (*mpstack).size;
                                            (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                            *(*mpstack).items.offset(c2rust_fresh35 as isize) =
                                                MPConvStackVal {
                                                    type_0: kMPConvList,
                                                    tv: tv,
                                                    saved_copyID: saved_copyID_0,
                                                    data: C2Rust_Unnamed_0 {
                                                        l: C2Rust_Unnamed_3 {
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
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let val_list_0: *mut list_T =
                                                (*val_di).di_tv.vval.v_list;
                                            if val_list_0.is_null()
                                                || tv_list_len(val_list_0)
                                                    == 0 as ::core::ffi::c_int
                                            {
                                                ga_concat_len(
                                                    gap,
                                                    b"{}\0".as_ptr() as *const ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                        .wrapping_sub(1 as size_t),
                                                );
                                                break '_typval_encode_stop_converting_one_item;
                                            } else {
                                                let l_: *const list_T = val_list_0;
                                                's_1122: {
                                                    if !l_.is_null() {
                                                        let mut li: *const listitem_T =
                                                            (*l_).lv_first;
                                                        loop {
                                                            if li.is_null() {
                                                                break 's_1122;
                                                            }
                                                            if (*li).li_tv.v_type
                                                                as ::core::ffi::c_uint
                                                                != VAR_LIST as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                || tv_list_len(
                                                                    (*li).li_tv.vval.v_list,
                                                                ) != 2 as ::core::ffi::c_int
                                                            {
                                                                break 's_1204;
                                                            }
                                                            li = (*li).li_next;
                                                        }
                                                    }
                                                }
                                                let saved_copyID_1: ::core::ffi::c_int =
                                                    tv_list_copyid((*val_di).di_tv.vval.v_list);
                                                let te_csr_ret_1: ::core::ffi::c_int =
                                                    _typval_encode_string_check_self_reference(
                                                        gap,
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
                                                ga_append(gap, '{' as uint8_t);
                                                '_c2rust_label_1: {
                                                    if saved_copyID_1 != copyID
                                                        && saved_copyID_1
                                                            != copyID - 1 as ::core::ffi::c_int
                                                    {
                                                    } else {
                                                        __assert_fail(
                                                            b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            b"src/nvim/eval/encode.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            566 as ::core::ffi::c_uint,
                                                            b"int _typval_encode_string_convert_one_value(garray_T *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
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
                                                let c2rust_fresh36 = (*mpstack).size;
                                                (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                                *(*mpstack).items.offset(c2rust_fresh36 as isize) =
                                                    MPConvStackVal {
                                                        type_0: kMPConvPairs,
                                                        tv: tv,
                                                        saved_copyID: saved_copyID_1,
                                                        data: C2Rust_Unnamed_0 {
                                                            l: C2Rust_Unnamed_3 {
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
                                                tv_list_len(val_list_1) != 2 as ::core::ffi::c_int
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
                        _typval_encode_string_check_self_reference(
                            gap,
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
                    ga_append(gap, '{' as uint8_t);
                    '_c2rust_label_2: {
                        if saved_copyID_2 != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/encode.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                614 as ::core::ffi::c_uint,
                                b"int _typval_encode_string_convert_one_value(garray_T *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
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
                    let c2rust_fresh37 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh37 as isize) = MPConvStackVal {
                        type_0: kMPConvDict,
                        tv: tv,
                        saved_copyID: saved_copyID_2,
                        data: C2Rust_Unnamed_0 {
                            d: C2Rust_Unnamed_4 {
                                dict: (*tv).vval.v_dict,
                                dictp: &raw mut (*tv).vval.v_dict,
                                hi: (*(*tv).vval.v_dict).dv_hashtab.ht_array,
                                todo: (*(*tv).vval.v_dict).dv_hashtab.ht_used,
                            },
                        },
                    };
                }
            }
            0 => {
                internal_error(b"_typval_encode_string_convert_one_value()\0".as_ptr()
                    as *const ::core::ffi::c_char);
                return FAIL;
            }
            _ => {}
        }
    }
    return OK;
}
unsafe extern "C" fn encode_vim_to_string(
    gap: *mut garray_T,
    top_tv: *mut typval_T,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let copyID: ::core::ffi::c_int = get_copyID();
    let mut mpstack: MPConvStack = MPConvStack {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<MPConvStackVal>(),
        init_array: [MPConvStackVal {
            type_0: kMPConvDict,
            tv: ::core::ptr::null_mut::<typval_T>(),
            saved_copyID: 0,
            data: C2Rust_Unnamed_0 {
                d: C2Rust_Unnamed_4 {
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
        if _typval_encode_string_convert_one_value(
            gap,
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
                            ga_append(gap, '}' as uint8_t);
                            continue;
                        } else {
                            if (*cur_mpsv).data.d.todo
                                != (*(*cur_mpsv).data.d.dict).dv_hashtab.ht_used
                            {
                                ga_concat_len(
                                    gap,
                                    b", \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            }
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
                            let buf_: *const ::core::ffi::c_char = (&raw mut (*di).di_key
                                as *mut ::core::ffi::c_char)
                                .offset(0 as ::core::ffi::c_int as isize);
                            if buf_.is_null() {
                                ga_concat_len(
                                    gap,
                                    b"''\0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            } else {
                                let len_: size_t = strlen(
                                    (&raw mut (*di).di_key as *mut ::core::ffi::c_char)
                                        .offset(0 as ::core::ffi::c_int as isize),
                                );
                                ga_grow(
                                    gap,
                                    (2 as size_t).wrapping_add(len_).wrapping_add(memcnt(
                                        buf_ as *const ::core::ffi::c_void,
                                        '\'' as ::core::ffi::c_char,
                                        len_,
                                    )) as ::core::ffi::c_int,
                                );
                                ga_append(gap, '\'' as uint8_t);
                                let mut i_: size_t = 0 as size_t;
                                while i_ < len_ {
                                    if *buf_.offset(i_ as isize) as ::core::ffi::c_int
                                        == '\'' as ::core::ffi::c_int
                                    {
                                        ga_append(gap, '\'' as uint8_t);
                                    }
                                    ga_append(gap, *buf_.offset(i_ as isize) as uint8_t);
                                    i_ = i_.wrapping_add(1);
                                }
                                ga_append(gap, '\'' as uint8_t);
                            }
                            ga_concat_len(
                                gap,
                                b": \0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            );
                            tv = &raw mut (*di).di_tv;
                        }
                    }
                    1 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            ga_append(gap, ']' as uint8_t);
                            continue;
                        } else {
                            if (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list) {
                                ga_concat_len(
                                    gap,
                                    b", \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            }
                            tv = &raw mut (*(*cur_mpsv).data.l.li).li_tv;
                            (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                        }
                    }
                    2 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            ga_append(gap, '}' as uint8_t);
                            continue;
                        } else {
                            if (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list) {
                                ga_concat_len(
                                    gap,
                                    b", \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            }
                            let kv_pair: *const list_T = (*(*cur_mpsv).data.l.li).li_tv.vval.v_list;
                            if _typval_encode_string_convert_one_value(
                                gap,
                                &raw mut mpstack,
                                cur_mpsv,
                                &raw mut (*(tv_list_first
                                    as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                    kv_pair,
                                ))
                                .li_tv,
                                copyID,
                                objname,
                            ) == FAIL
                            {
                                break '_encode_vim_to__error_ret;
                            }
                            ga_concat_len(
                                gap,
                                b": \0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            );
                            tv = &raw mut (*(tv_list_last
                                as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                kv_pair,
                            ))
                            .li_tv;
                            (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                        }
                    }
                    3 => {
                        let pt: *mut partial_T = (*cur_mpsv).data.p.pt;
                        tv = (*cur_mpsv).tv;
                        match (*cur_mpsv).data.p.stage as ::core::ffi::c_uint {
                            0 => {
                                if (if pt.is_null() {
                                    0 as ::core::ffi::c_int
                                } else {
                                    (*pt).pt_argc
                                }) != 0 as ::core::ffi::c_int
                                {
                                    ga_concat_len(
                                        gap,
                                        b", \0".as_ptr() as *const ::core::ffi::c_char,
                                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                            .wrapping_sub(1 as size_t),
                                    );
                                }
                                (*cur_mpsv).data.p.stage = kMPConvPartialSelf;
                                if !pt.is_null() && (*pt).pt_argc > 0 as ::core::ffi::c_int {
                                    ga_append(gap, '[' as uint8_t);
                                    if mpstack.size == mpstack.capacity {
                                        mpstack.capacity = if mpstack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            mpstack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as size_t,
                                                )
                                        };
                                        mpstack.items = (if mpstack.capacity
                                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
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
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
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
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    )),
                                                    mpstack.items as *const ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            } else {
                                                xrealloc(
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            }
                                        })
                                            as *mut MPConvStackVal;
                                    } else {
                                    };
                                    let c2rust_fresh30 = mpstack.size;
                                    mpstack.size = mpstack.size.wrapping_add(1);
                                    *mpstack.items.offset(c2rust_fresh30 as isize) =
                                        MPConvStackVal {
                                            type_0: kMPConvPartialList,
                                            tv: ::core::ptr::null_mut::<typval_T>(),
                                            saved_copyID: copyID - 1 as ::core::ffi::c_int,
                                            data: C2Rust_Unnamed_0 {
                                                a: C2Rust_Unnamed_1 {
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
                                if !dict.is_null() {
                                    if (*dict).dv_hashtab.ht_used as ptrdiff_t != -1 as ptrdiff_t {
                                        ga_concat_len(
                                            gap,
                                            b", \0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                .wrapping_sub(1 as size_t),
                                        );
                                    }
                                    if (*dict).dv_hashtab.ht_used == 0 as size_t {
                                        ga_concat_len(
                                            gap,
                                            b"{}\0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                .wrapping_sub(1 as size_t),
                                        );
                                        continue;
                                    } else {
                                        let saved_copyID: ::core::ffi::c_int = (*dict).dv_copyID;
                                        let te_csr_ret: ::core::ffi::c_int =
                                            _typval_encode_string_check_self_reference(
                                                gap,
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
                                            ga_append(gap, '{' as uint8_t);
                                            '_c2rust_label: {
                                                if saved_copyID != copyID
                                                    && saved_copyID
                                                        != copyID - 1 as ::core::ffi::c_int
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        b"src/nvim/eval/encode.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        789 as ::core::ffi::c_uint,
                                                        b"int encode_vim_to_string(garray_T *const, typval_T *const, const char *const)\0"
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
                                            let c2rust_fresh31 = mpstack.size;
                                            mpstack.size = mpstack.size.wrapping_add(1);
                                            *mpstack.items.offset(c2rust_fresh31 as isize) =
                                                MPConvStackVal {
                                                    type_0: kMPConvDict,
                                                    tv: ::core::ptr::null_mut::<typval_T>(),
                                                    saved_copyID: saved_copyID,
                                                    data: C2Rust_Unnamed_0 {
                                                        d: C2Rust_Unnamed_4 {
                                                            dict: dict,
                                                            dictp: &raw mut (*pt).pt_dict,
                                                            hi: (*dict).dv_hashtab.ht_array,
                                                            todo: (*dict).dv_hashtab.ht_used,
                                                        },
                                                    },
                                                };
                                            continue;
                                        }
                                    }
                                } else {
                                    if -1 as ::core::ffi::c_int as ptrdiff_t != -1 as ptrdiff_t {
                                        ga_concat_len(
                                            gap,
                                            b", \0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                .wrapping_sub(1 as size_t),
                                        );
                                    }
                                    continue;
                                }
                            }
                            2 => {
                                ga_append(gap, ')' as uint8_t);
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
                            ga_append(gap, ']' as uint8_t);
                            continue;
                        } else {
                            if (*cur_mpsv).data.a.argv != (*cur_mpsv).data.a.arg {
                                ga_concat_len(
                                    gap,
                                    b", \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            }
                            let c2rust_fresh32 = (*cur_mpsv).data.a.arg;
                            (*cur_mpsv).data.a.arg = (*cur_mpsv).data.a.arg.offset(1);
                            tv = c2rust_fresh32;
                            (*cur_mpsv).data.a.todo = (*cur_mpsv).data.a.todo.wrapping_sub(1);
                        }
                    }
                    _ => {}
                }
                '_c2rust_label_0: {
                    if !tv.is_null() {
                    } else {
                        __assert_fail(
                            b"tv != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/eval/encode.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            829 as ::core::ffi::c_uint,
                            b"int encode_vim_to_string(garray_T *const, typval_T *const, const char *const)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if _typval_encode_string_convert_one_value(
                    gap,
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
pub static _typval_encode_json_nodict_var: GlobalCell<*const dict_T> =
    GlobalCell::new(::core::ptr::null::<dict_T>());
#[inline(always)]
unsafe extern "C" fn _typval_encode_json_check_self_reference(
    _gap: *mut garray_T,
    _val: *mut ::core::ffi::c_void,
    val_copyID: *mut ::core::ffi::c_int,
    _mpstack: *const MPConvStack,
    copyID: ::core::ffi::c_int,
    _conv_type: MPConvStackValType,
    _objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *val_copyID == copyID {
        if !did_echo_string_emsg.get() {
            did_echo_string_emsg.set(true_0 != 0);
            emsg(gettext(
                b"E724: unable to correctly dump variable with self-referencing container\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            ));
        }
        return OK;
    }
    *val_copyID = copyID;
    return NOTDONE;
}
unsafe extern "C" fn _typval_encode_json_convert_one_value(
    gap: *mut garray_T,
    mpstack: *mut MPConvStack,
    _cur_mpsv: *mut MPConvStackVal,
    tv: *mut typval_T,
    copyID: ::core::ffi::c_int,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    '_typval_encode_stop_converting_one_item: {
        match (*tv).v_type as ::core::ffi::c_uint {
            2 => {
                if convert_to_json_string(gap, (*tv).vval.v_string, tv_strlen(tv)) != OK {
                    return FAIL;
                }
            }
            1 => {
                let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
                let mut numbuflen: size_t = vim_snprintf_safelen(
                    &raw mut numbuf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                        .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                        .wrapping_div(
                            (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                == 0) as ::core::ffi::c_int as size_t,
                        ),
                    b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                    (*tv).vval.v_number,
                );
                ga_concat_len(gap, &raw mut numbuf as *mut ::core::ffi::c_char, numbuflen);
            }
            6 => {
                let flt_: float_T = (*tv).vval.v_float;
                match flt_.classify() {
                    ::core::num::FpCategory::Nan => {
                        emsg(gettext(
                            b"E474: Unable to represent NaN value in JSON\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ));
                        return FAIL;
                    }
                    ::core::num::FpCategory::Infinite => {
                        emsg(gettext(
                            b"E474: Unable to represent infinity in JSON\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ));
                        return FAIL;
                    }
                    _ => {
                        let mut numbuf_0: [::core::ffi::c_char; 65] = [0; 65];
                        let mut numbuflen_0: size_t = vim_snprintf_safelen(
                            &raw mut numbuf_0 as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                ),
                            b"%g\0".as_ptr() as *const ::core::ffi::c_char,
                            flt_,
                        );
                        ga_concat_len(
                            gap,
                            &raw mut numbuf_0 as *mut ::core::ffi::c_char,
                            numbuflen_0,
                        );
                    }
                }
            }
            10 => {
                let blob_: *const blob_T = (*tv).vval.v_blob;
                let len_: ::core::ffi::c_int = tv_blob_len((*tv).vval.v_blob);
                if len_ == 0 as ::core::ffi::c_int {
                    ga_concat_len(
                        gap,
                        b"[]\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    ga_append(gap, '[' as uint8_t);
                    let mut numbuf_1: [::core::ffi::c_char; 65] = [0; 65];
                    let mut i_: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i_ < len_ {
                        if i_ > 0 as ::core::ffi::c_int {
                            ga_concat_len(
                                gap,
                                b", \0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            );
                        }
                        let mut numbuflen_1: size_t = vim_snprintf_safelen(
                            &raw mut numbuf_1 as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                ),
                            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                            tv_blob_get(blob_, i_) as ::core::ffi::c_int,
                        );
                        ga_concat_len(
                            gap,
                            &raw mut numbuf_1 as *mut ::core::ffi::c_char,
                            numbuflen_1,
                        );
                        i_ += 1;
                    }
                    ga_append(gap, ']' as uint8_t);
                }
            }
            3 => {
                return conv_error(
                    gettext(
                        b"E474: Error while dumping %s, %s: attempt to dump function reference\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    mpstack,
                    objname,
                );
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
                return conv_error(
                    gettext(
                        b"E474: Error while dumping %s, %s: attempt to dump function reference\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    mpstack,
                    objname,
                );
            }
            4 => {
                if (*tv).vval.v_list.is_null()
                    || tv_list_len((*tv).vval.v_list) == 0 as ::core::ffi::c_int
                {
                    ga_concat_len(
                        gap,
                        b"[]\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    let saved_copyID: ::core::ffi::c_int = tv_list_copyid((*tv).vval.v_list);
                    let te_csr_ret: ::core::ffi::c_int = _typval_encode_json_check_self_reference(
                        gap,
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
                    ga_append(gap, '[' as uint8_t);
                    '_c2rust_label: {
                        if saved_copyID != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/encode.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                383 as ::core::ffi::c_uint,
                                b"int _typval_encode_json_convert_one_value(garray_T *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
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
                    let c2rust_fresh46 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh46 as isize) = MPConvStackVal {
                        type_0: kMPConvList,
                        tv: tv,
                        saved_copyID: saved_copyID,
                        data: C2Rust_Unnamed_0 {
                            l: C2Rust_Unnamed_3 {
                                list: (*tv).vval.v_list,
                                li: tv_list_first((*tv).vval.v_list),
                            },
                        },
                    };
                }
            }
            7 => match (*tv).vval.v_bool as ::core::ffi::c_uint {
                1 | 0 => {
                    if (*tv).vval.v_bool as ::core::ffi::c_uint
                        == kBoolVarTrue as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        ga_concat_len(
                            gap,
                            b"true\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                .wrapping_sub(1 as size_t),
                        );
                    } else {
                        ga_concat_len(
                            gap,
                            b"false\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                                .wrapping_sub(1 as size_t),
                        );
                    }
                }
                _ => {}
            },
            8 => match (*tv).vval.v_special as ::core::ffi::c_uint {
                0 => {
                    ga_concat_len(
                        gap,
                        b"null\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                            .wrapping_sub(1 as size_t),
                    );
                }
                _ => {}
            },
            5 => {
                if (*tv).vval.v_dict.is_null()
                    || (*(*tv).vval.v_dict).dv_hashtab.ht_used == 0 as size_t
                {
                    ga_concat_len(
                        gap,
                        b"{}\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    let mut type_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    let mut val_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    's_883: {
                        if TYPVAL_ENCODE_ALLOW_SPECIALS_1 != 0
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
                                    == (*eval_msgpack_type_lists.ptr())[i as usize] as *mut list_T
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
                                        ga_concat_len(
                                            gap,
                                            b"null\0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                                .wrapping_sub(1 as size_t),
                                        );
                                        break '_typval_encode_stop_converting_one_item;
                                    }
                                    1 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_NUMBER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            if (*val_di).di_tv.vval.v_number != 0 {
                                                ga_concat_len(
                                                    gap,
                                                    b"true\0".as_ptr() as *const ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                                        .wrapping_sub(1 as size_t),
                                                );
                                            } else {
                                                ga_concat_len(
                                                    gap,
                                                    b"false\0".as_ptr() as *const ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                                                        .wrapping_sub(1 as size_t),
                                                );
                                            }
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
                                                        highest_bits =
                                                            (*highest_bits_li).li_tv.vval.v_number;
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
                                                            high_bits =
                                                                (*high_bits_li).li_tv.vval.v_number;
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
                                                            if sign > 0 as varnumber_T {
                                                                let mut numbuf_2: [::core::ffi::c_char; 65] = [0; 65];
                                                                let mut numbuflen_2: size_t = vim_snprintf_safelen(
                                                                    &raw mut numbuf_2 as *mut ::core::ffi::c_char,
                                                                    ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                                                        .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                                                        .wrapping_div(
                                                                            (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                                                                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                                                                == 0) as ::core::ffi::c_int as size_t,
                                                                        ),
                                                                    b"%lu\0".as_ptr() as *const ::core::ffi::c_char,
                                                                    number,
                                                                );
                                                                ga_concat_len(
                                                                    gap,
                                                                    &raw mut numbuf_2
                                                                        as *mut ::core::ffi::c_char,
                                                                    numbuflen_2,
                                                                );
                                                            } else {
                                                                let mut numbuf_3: [::core::ffi::c_char; 65] = [0; 65];
                                                                let mut numbuflen_3: size_t = vim_snprintf_safelen(
                                                                    &raw mut numbuf_3 as *mut ::core::ffi::c_char,
                                                                    ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                                                        .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                                                        .wrapping_div(
                                                                            (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                                                                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                                                                == 0) as ::core::ffi::c_int as size_t,
                                                                        ),
                                                                    b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                                                                    number.wrapping_neg() as int64_t,
                                                                );
                                                                ga_concat_len(
                                                                    gap,
                                                                    &raw mut numbuf_3
                                                                        as *mut ::core::ffi::c_char,
                                                                    numbuflen_3,
                                                                );
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
                                            let flt__0: float_T = (*val_di).di_tv.vval.v_float;
                                            match flt__0.classify() {
                                                ::core::num::FpCategory::Nan => {
                                                    emsg(
                                                        gettext(
                                                            b"E474: Unable to represent NaN value in JSON\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                    );
                                                    return FAIL;
                                                }
                                                ::core::num::FpCategory::Infinite => {
                                                    emsg(
                                                        gettext(
                                                            b"E474: Unable to represent infinity in JSON\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                    );
                                                    return FAIL;
                                                }
                                                _ => {
                                                    let mut numbuf_4: [::core::ffi::c_char; 65] =
                                                        [0; 65];
                                                    let mut numbuflen_4: size_t =
                                                        vim_snprintf_safelen(
                                                            &raw mut numbuf_4
                                                                as *mut ::core::ffi::c_char,
                                                            ::core::mem::size_of::<
                                                                [::core::ffi::c_char; 65],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                ::core::ffi::c_char,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [::core::ffi::c_char; 65],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        ::core::ffi::c_char,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as size_t,
                                                            ),
                                                            b"%g\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                            flt__0,
                                                        );
                                                    ga_concat_len(
                                                        gap,
                                                        &raw mut numbuf_4
                                                            as *mut ::core::ffi::c_char,
                                                        numbuflen_4,
                                                    );
                                                }
                                            }
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    4 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let mut len: size_t = 0;
                                            let mut buf: *mut ::core::ffi::c_char =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            if encode_vim_list_to_buf(
                                                (*val_di).di_tv.vval.v_list,
                                                &raw mut len,
                                                &raw mut buf,
                                            ) {
                                                if convert_to_json_string(gap, buf, len) != OK {
                                                    return FAIL;
                                                }
                                                xfree(buf as *mut ::core::ffi::c_void);
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    5 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let saved_copyID_0: ::core::ffi::c_int =
                                                tv_list_copyid((*val_di).di_tv.vval.v_list);
                                            let te_csr_ret_0: ::core::ffi::c_int =
                                                _typval_encode_json_check_self_reference(
                                                    gap,
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
                                            ga_append(gap, '[' as uint8_t);
                                            '_c2rust_label_0: {
                                                if saved_copyID_0 != copyID
                                                    && saved_copyID_0
                                                        != copyID - 1 as ::core::ffi::c_int
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        b"src/nvim/eval/encode.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        532 as ::core::ffi::c_uint,
                                                        b"int _typval_encode_json_convert_one_value(garray_T *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            if (*mpstack).size == (*mpstack).capacity {
                                                (*mpstack).capacity = if (*mpstack).capacity
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
                                                    (*mpstack).capacity << 1 as ::core::ffi::c_int
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
                                                (*mpstack).items = (if (*mpstack).capacity
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
                                                    if (*mpstack).items
                                                        == &raw mut (*mpstack).init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        (*mpstack).items as *mut ::core::ffi::c_void
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
                                                            (*mpstack).capacity.wrapping_mul(
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
                                            let c2rust_fresh47 = (*mpstack).size;
                                            (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                            *(*mpstack).items.offset(c2rust_fresh47 as isize) =
                                                MPConvStackVal {
                                                    type_0: kMPConvList,
                                                    tv: tv,
                                                    saved_copyID: saved_copyID_0,
                                                    data: C2Rust_Unnamed_0 {
                                                        l: C2Rust_Unnamed_3 {
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
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let val_list_0: *mut list_T =
                                                (*val_di).di_tv.vval.v_list;
                                            if val_list_0.is_null()
                                                || tv_list_len(val_list_0)
                                                    == 0 as ::core::ffi::c_int
                                            {
                                                ga_concat_len(
                                                    gap,
                                                    b"{}\0".as_ptr() as *const ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                        .wrapping_sub(1 as size_t),
                                                );
                                                break '_typval_encode_stop_converting_one_item;
                                            } else {
                                                let l_: *const list_T = val_list_0;
                                                's_787: {
                                                    if !l_.is_null() {
                                                        let mut li: *const listitem_T =
                                                            (*l_).lv_first;
                                                        loop {
                                                            if li.is_null() {
                                                                break 's_787;
                                                            }
                                                            if (*li).li_tv.v_type
                                                                as ::core::ffi::c_uint
                                                                != VAR_LIST as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                || tv_list_len(
                                                                    (*li).li_tv.vval.v_list,
                                                                ) != 2 as ::core::ffi::c_int
                                                            {
                                                                break 's_883;
                                                            }
                                                            li = (*li).li_next;
                                                        }
                                                    }
                                                }
                                                let saved_copyID_1: ::core::ffi::c_int =
                                                    tv_list_copyid((*val_di).di_tv.vval.v_list);
                                                let te_csr_ret_1: ::core::ffi::c_int =
                                                    _typval_encode_json_check_self_reference(
                                                        gap,
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
                                                ga_append(gap, '{' as uint8_t);
                                                '_c2rust_label_1: {
                                                    if saved_copyID_1 != copyID
                                                        && saved_copyID_1
                                                            != copyID - 1 as ::core::ffi::c_int
                                                    {
                                                    } else {
                                                        __assert_fail(
                                                            b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            b"src/nvim/eval/encode.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            566 as ::core::ffi::c_uint,
                                                            b"int _typval_encode_json_convert_one_value(garray_T *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
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
                                                let c2rust_fresh48 = (*mpstack).size;
                                                (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                                *(*mpstack).items.offset(c2rust_fresh48 as isize) =
                                                    MPConvStackVal {
                                                        type_0: kMPConvPairs,
                                                        tv: tv,
                                                        saved_copyID: saved_copyID_1,
                                                        data: C2Rust_Unnamed_0 {
                                                            l: C2Rust_Unnamed_3 {
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
                                                tv_list_len(val_list_1) != 2 as ::core::ffi::c_int
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
                                                emsg(gettext(
                                                    b"E474: Unable to convert EXT string to JSON\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ));
                                                return FAIL;
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
                    let te_csr_ret_2: ::core::ffi::c_int = _typval_encode_json_check_self_reference(
                        gap,
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
                    ga_append(gap, '{' as uint8_t);
                    '_c2rust_label_2: {
                        if saved_copyID_2 != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/encode.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                614 as ::core::ffi::c_uint,
                                b"int _typval_encode_json_convert_one_value(garray_T *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
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
                    let c2rust_fresh49 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh49 as isize) = MPConvStackVal {
                        type_0: kMPConvDict,
                        tv: tv,
                        saved_copyID: saved_copyID_2,
                        data: C2Rust_Unnamed_0 {
                            d: C2Rust_Unnamed_4 {
                                dict: (*tv).vval.v_dict,
                                dictp: &raw mut (*tv).vval.v_dict,
                                hi: (*(*tv).vval.v_dict).dv_hashtab.ht_array,
                                todo: (*(*tv).vval.v_dict).dv_hashtab.ht_used,
                            },
                        },
                    };
                }
            }
            0 => {
                internal_error(b"_typval_encode_json_convert_one_value()\0".as_ptr()
                    as *const ::core::ffi::c_char);
                return FAIL;
            }
            _ => {}
        }
    }
    return OK;
}
unsafe extern "C" fn encode_vim_to_json(
    gap: *mut garray_T,
    top_tv: *mut typval_T,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let copyID: ::core::ffi::c_int = get_copyID();
    let mut mpstack: MPConvStack = MPConvStack {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<MPConvStackVal>(),
        init_array: [MPConvStackVal {
            type_0: kMPConvDict,
            tv: ::core::ptr::null_mut::<typval_T>(),
            saved_copyID: 0,
            data: C2Rust_Unnamed_0 {
                d: C2Rust_Unnamed_4 {
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
        if _typval_encode_json_convert_one_value(
            gap,
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
                            ga_append(gap, '}' as uint8_t);
                            continue;
                        } else {
                            if (*cur_mpsv).data.d.todo
                                != (*(*cur_mpsv).data.d.dict).dv_hashtab.ht_used
                            {
                                ga_concat_len(
                                    gap,
                                    b", \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            }
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
                            if convert_to_json_string(
                                gap,
                                (&raw mut (*di).di_key as *mut ::core::ffi::c_char)
                                    .offset(0 as ::core::ffi::c_int as isize),
                                strlen(
                                    (&raw mut (*di).di_key as *mut ::core::ffi::c_char)
                                        .offset(0 as ::core::ffi::c_int as isize),
                                ),
                            ) != OK
                            {
                                return FAIL;
                            }
                            ga_concat_len(
                                gap,
                                b": \0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                    .wrapping_sub(1 as size_t),
                            );
                            tv = &raw mut (*di).di_tv;
                        }
                    }
                    1 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            ga_append(gap, ']' as uint8_t);
                            continue;
                        } else {
                            if (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list) {
                                ga_concat_len(
                                    gap,
                                    b", \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            }
                            tv = &raw mut (*(*cur_mpsv).data.l.li).li_tv;
                            (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                        }
                    }
                    2 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            ga_append(gap, '}' as uint8_t);
                            continue;
                        } else {
                            if (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list) {
                                ga_concat_len(
                                    gap,
                                    b", \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            }
                            let kv_pair: *const list_T = (*(*cur_mpsv).data.l.li).li_tv.vval.v_list;
                            if !encode_check_json_key(
                                &raw mut (*(tv_list_first
                                    as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                    kv_pair,
                                ))
                                .li_tv,
                            ) {
                                emsg(gettext(
                                    b"E474: Invalid key in special dictionary\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ));
                                break '_encode_vim_to__error_ret;
                            } else {
                                if _typval_encode_json_convert_one_value(
                                    gap,
                                    &raw mut mpstack,
                                    cur_mpsv,
                                    &raw mut (*(tv_list_first
                                        as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                        kv_pair,
                                    ))
                                    .li_tv,
                                    copyID,
                                    objname,
                                ) == FAIL
                                {
                                    break '_encode_vim_to__error_ret;
                                }
                                ga_concat_len(
                                    gap,
                                    b": \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                                tv = &raw mut (*(tv_list_last
                                    as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                    kv_pair,
                                ))
                                .li_tv;
                                (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                            }
                        }
                    }
                    3 => {
                        let pt: *mut partial_T = (*cur_mpsv).data.p.pt;
                        tv = (*cur_mpsv).tv;
                        match (*cur_mpsv).data.p.stage as ::core::ffi::c_uint {
                            0 => {
                                if (if pt.is_null() {
                                    0 as ::core::ffi::c_int
                                } else {
                                    (*pt).pt_argc
                                }) != 0 as ::core::ffi::c_int
                                {
                                    ga_concat_len(
                                        gap,
                                        b", \0".as_ptr() as *const ::core::ffi::c_char,
                                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                            .wrapping_sub(1 as size_t),
                                    );
                                }
                                (*cur_mpsv).data.p.stage = kMPConvPartialSelf;
                                if !pt.is_null() && (*pt).pt_argc > 0 as ::core::ffi::c_int {
                                    ga_append(gap, '[' as uint8_t);
                                    if mpstack.size == mpstack.capacity {
                                        mpstack.capacity = if mpstack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            mpstack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as size_t,
                                                )
                                        };
                                        mpstack.items = (if mpstack.capacity
                                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
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
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
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
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    )),
                                                    mpstack.items as *const ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            } else {
                                                xrealloc(
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            }
                                        })
                                            as *mut MPConvStackVal;
                                    } else {
                                    };
                                    let c2rust_fresh42 = mpstack.size;
                                    mpstack.size = mpstack.size.wrapping_add(1);
                                    *mpstack.items.offset(c2rust_fresh42 as isize) =
                                        MPConvStackVal {
                                            type_0: kMPConvPartialList,
                                            tv: ::core::ptr::null_mut::<typval_T>(),
                                            saved_copyID: copyID - 1 as ::core::ffi::c_int,
                                            data: C2Rust_Unnamed_0 {
                                                a: C2Rust_Unnamed_1 {
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
                                if !dict.is_null() {
                                    if (*dict).dv_hashtab.ht_used as ptrdiff_t != -1 as ptrdiff_t {
                                        ga_concat_len(
                                            gap,
                                            b", \0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                .wrapping_sub(1 as size_t),
                                        );
                                    }
                                    if (*dict).dv_hashtab.ht_used == 0 as size_t {
                                        ga_concat_len(
                                            gap,
                                            b"{}\0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                .wrapping_sub(1 as size_t),
                                        );
                                        continue;
                                    } else {
                                        let saved_copyID: ::core::ffi::c_int = (*dict).dv_copyID;
                                        let te_csr_ret: ::core::ffi::c_int =
                                            _typval_encode_json_check_self_reference(
                                                gap,
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
                                            ga_append(gap, '{' as uint8_t);
                                            '_c2rust_label: {
                                                if saved_copyID != copyID
                                                    && saved_copyID
                                                        != copyID - 1 as ::core::ffi::c_int
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        b"src/nvim/eval/encode.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        789 as ::core::ffi::c_uint,
                                                        b"int encode_vim_to_json(garray_T *const, typval_T *const, const char *const)\0"
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
                                            let c2rust_fresh43 = mpstack.size;
                                            mpstack.size = mpstack.size.wrapping_add(1);
                                            *mpstack.items.offset(c2rust_fresh43 as isize) =
                                                MPConvStackVal {
                                                    type_0: kMPConvDict,
                                                    tv: ::core::ptr::null_mut::<typval_T>(),
                                                    saved_copyID: saved_copyID,
                                                    data: C2Rust_Unnamed_0 {
                                                        d: C2Rust_Unnamed_4 {
                                                            dict: dict,
                                                            dictp: &raw mut (*pt).pt_dict,
                                                            hi: (*dict).dv_hashtab.ht_array,
                                                            todo: (*dict).dv_hashtab.ht_used,
                                                        },
                                                    },
                                                };
                                            continue;
                                        }
                                    }
                                } else {
                                    if -1 as ::core::ffi::c_int as ptrdiff_t != -1 as ptrdiff_t {
                                        ga_concat_len(
                                            gap,
                                            b", \0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                .wrapping_sub(1 as size_t),
                                        );
                                    }
                                    continue;
                                }
                            }
                            2 => {
                                ga_append(gap, ')' as uint8_t);
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
                            ga_append(gap, ']' as uint8_t);
                            continue;
                        } else {
                            if (*cur_mpsv).data.a.argv != (*cur_mpsv).data.a.arg {
                                ga_concat_len(
                                    gap,
                                    b", \0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                        .wrapping_sub(1 as size_t),
                                );
                            }
                            let c2rust_fresh44 = (*cur_mpsv).data.a.arg;
                            (*cur_mpsv).data.a.arg = (*cur_mpsv).data.a.arg.offset(1);
                            tv = c2rust_fresh44;
                            (*cur_mpsv).data.a.todo = (*cur_mpsv).data.a.todo.wrapping_sub(1);
                        }
                    }
                    _ => {}
                }
                '_c2rust_label_0: {
                    if !tv.is_null() {
                    } else {
                        __assert_fail(
                            b"tv != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/eval/encode.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            829 as ::core::ffi::c_uint,
                            b"int encode_vim_to_json(garray_T *const, typval_T *const, const char *const)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if _typval_encode_json_convert_one_value(
                    gap,
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
pub const SURROGATE_HI_START: ::core::ffi::c_int = 0xd800 as ::core::ffi::c_int;
pub const SURROGATE_HI_END: ::core::ffi::c_int = 0xdbff as ::core::ffi::c_int;
pub const SURROGATE_LO_START: ::core::ffi::c_int = 0xdc00 as ::core::ffi::c_int;
pub const SURROGATE_LO_END: ::core::ffi::c_int = 0xdfff as ::core::ffi::c_int;
pub const SURROGATE_FIRST_CHAR: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub static encode_bool_var_names: GlobalCell<[*const ::core::ffi::c_char; 2]> = GlobalCell::new([
    b"v:false\0".as_ptr() as *const ::core::ffi::c_char,
    b"v:true\0".as_ptr() as *const ::core::ffi::c_char,
]);
pub static encode_special_var_names: GlobalCell<[*const ::core::ffi::c_char; 1]> =
    GlobalCell::new([b"v:null\0".as_ptr() as *const ::core::ffi::c_char]);
#[no_mangle]
pub unsafe extern "C" fn encode_list_write(
    data: *mut ::core::ffi::c_void,
    buf: *const ::core::ffi::c_char,
    len: size_t,
) {
    if len == 0 as size_t {
        return;
    }
    let list: *mut list_T = data as *mut list_T;
    let end: *const ::core::ffi::c_char = buf.offset(len as isize);
    let mut line_end: *const ::core::ffi::c_char = buf;
    let mut li: *mut listitem_T = tv_list_last(list);
    if !li.is_null() {
        line_end = xmemscan(
            buf as *const ::core::ffi::c_void,
            NL as ::core::ffi::c_char,
            len,
        ) as *const ::core::ffi::c_char;
        if line_end != buf {
            let line_length: size_t = line_end.offset_from(buf) as size_t;
            let mut str: *mut ::core::ffi::c_char = (*li).li_tv.vval.v_string;
            let li_len: size_t = if str.is_null() {
                0 as size_t
            } else {
                strlen(str)
            };
            (*li).li_tv.vval.v_string = xrealloc(
                str as *mut ::core::ffi::c_void,
                li_len.wrapping_add(line_length).wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            str = (*li).li_tv.vval.v_string.offset(li_len as isize);
            memcpy(
                str as *mut ::core::ffi::c_void,
                buf as *const ::core::ffi::c_void,
                line_length,
            );
            *str.offset(line_length as isize) = 0 as ::core::ffi::c_char;
            memchrsub(
                str as *mut ::core::ffi::c_void,
                NUL as ::core::ffi::c_char,
                NL as ::core::ffi::c_char,
                line_length,
            );
        }
        line_end = line_end.offset(1);
    }
    while line_end < end {
        let mut line_start: *const ::core::ffi::c_char = line_end;
        line_end = xmemscan(
            line_start as *const ::core::ffi::c_void,
            NL as ::core::ffi::c_char,
            end.offset_from(line_start) as size_t,
        ) as *const ::core::ffi::c_char;
        let mut str_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if line_end != line_start {
            let line_length_0: size_t = line_end.offset_from(line_start) as size_t;
            str_0 = xmemdupz(line_start as *const ::core::ffi::c_void, line_length_0)
                as *mut ::core::ffi::c_char;
            memchrsub(
                str_0 as *mut ::core::ffi::c_void,
                NUL as ::core::ffi::c_char,
                NL as ::core::ffi::c_char,
                line_length_0,
            );
        }
        tv_list_append_allocated_string(list, str_0);
        line_end = line_end.offset(1);
    }
    if line_end == end {
        tv_list_append_allocated_string(list, ::core::ptr::null_mut::<::core::ffi::c_char>());
    }
}
static did_echo_string_emsg: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
unsafe extern "C" fn conv_error(
    msg: *const ::core::ffi::c_char,
    mpstack: *const MPConvStack,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut msg_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut msg_ga,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    let key_msg: *const ::core::ffi::c_char =
        gettext(b"key %s\0".as_ptr() as *const ::core::ffi::c_char);
    let key_pair_msg: *const ::core::ffi::c_char =
        gettext(b"key %s at index %i from special map\0".as_ptr() as *const ::core::ffi::c_char);
    let idx_msg: *const ::core::ffi::c_char =
        gettext(b"index %i\0".as_ptr() as *const ::core::ffi::c_char);
    let partial_arg_msg: *const ::core::ffi::c_char =
        gettext(b"partial\0".as_ptr() as *const ::core::ffi::c_char);
    let partial_arg_i_msg: *const ::core::ffi::c_char =
        gettext(b"argument %i\0".as_ptr() as *const ::core::ffi::c_char);
    let partial_self_msg: *const ::core::ffi::c_char =
        gettext(b"partial self dictionary\0".as_ptr() as *const ::core::ffi::c_char);
    let mut i: size_t = 0 as size_t;
    while i < (*mpstack).size {
        if i != 0 as size_t {
            ga_concat_len(
                &raw mut msg_ga,
                b", \0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        let mut v: MPConvStackVal = *(*mpstack).items.offset(i as isize);
        match v.type_0 as ::core::ffi::c_uint {
            0 => {
                let mut key_tv: typval_T = typval_T {
                    v_type: VAR_STRING,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_string: (*if v.data.d.hi.is_null() {
                            (*v.data.d.dict).dv_hashtab.ht_array
                        } else {
                            v.data.d.hi.offset(-(1 as ::core::ffi::c_int as isize))
                        })
                        .hi_key,
                    },
                };
                let key: *mut ::core::ffi::c_char =
                    encode_tv2string(&raw mut key_tv, ::core::ptr::null_mut::<size_t>());
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    key_msg,
                    key,
                );
                xfree(key as *mut ::core::ffi::c_void);
                ga_concat(&raw mut msg_ga, IObuff.ptr() as *mut ::core::ffi::c_char);
            }
            2 | 1 => {
                let idx: ::core::ffi::c_int = if v.data.l.li == tv_list_first(v.data.l.list) {
                    0 as ::core::ffi::c_int
                } else if v.data.l.li.is_null() {
                    tv_list_len(v.data.l.list) - 1 as ::core::ffi::c_int
                } else {
                    tv_list_idx_of_item(v.data.l.list, (*v.data.l.li).li_prev)
                };
                let li: *const listitem_T = if v.data.l.li.is_null() {
                    tv_list_last(v.data.l.list)
                } else {
                    (*v.data.l.li).li_prev
                };
                if v.type_0 as ::core::ffi::c_uint
                    == kMPConvList as ::core::ffi::c_int as ::core::ffi::c_uint
                    || li.is_null()
                    || (*li).li_tv.v_type as ::core::ffi::c_uint
                        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                        && tv_list_len((*li).li_tv.vval.v_list) <= 0 as ::core::ffi::c_int
                {
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        idx_msg,
                        idx,
                    );
                    ga_concat(&raw mut msg_ga, IObuff.ptr() as *mut ::core::ffi::c_char);
                } else {
                    '_c2rust_label: {
                        if !li.is_null() {
                        } else {
                            __assert_fail(
                                b"li != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/eval/encode.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                165 as ::core::ffi::c_uint,
                                b"int conv_error(const char *const, const MPConvStack *const, const char *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    let first_item: *mut listitem_T = tv_list_first((*li).li_tv.vval.v_list);
                    '_c2rust_label_0: {
                        if !first_item.is_null() {
                        } else {
                            __assert_fail(
                                b"first_item != NULL\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/encode.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                168 as ::core::ffi::c_uint,
                                b"int conv_error(const char *const, const MPConvStack *const, const char *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    let mut key_tv_0: typval_T = (*first_item).li_tv;
                    let key_0: *mut ::core::ffi::c_char =
                        encode_tv2echo(&raw mut key_tv_0, ::core::ptr::null_mut::<size_t>());
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        key_pair_msg,
                        key_0,
                        idx,
                    );
                    xfree(key_0 as *mut ::core::ffi::c_void);
                    ga_concat(&raw mut msg_ga, IObuff.ptr() as *mut ::core::ffi::c_char);
                }
            }
            3 => match v.data.p.stage as ::core::ffi::c_uint {
                0 => {
                    abort();
                }
                1 => {
                    ga_concat(&raw mut msg_ga, partial_arg_msg);
                }
                2 => {
                    ga_concat(&raw mut msg_ga, partial_self_msg);
                }
                _ => {}
            },
            4 => {
                let idx_0: ::core::ffi::c_int = v.data.a.arg.offset_from(v.data.a.argv)
                    as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int;
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    partial_arg_i_msg,
                    idx_0,
                );
                ga_concat(&raw mut msg_ga, IObuff.ptr() as *mut ::core::ffi::c_char);
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    semsg(
        msg,
        gettext(objname),
        if (*mpstack).size == 0 as size_t {
            gettext(b"itself\0".as_ptr() as *const ::core::ffi::c_char)
        } else {
            msg_ga.ga_data as *mut ::core::ffi::c_char
        },
    );
    ga_clear(&raw mut msg_ga);
    return FAIL;
}
pub unsafe extern "C" fn encode_vim_list_to_buf(
    list: *const list_T,
    ret_len: *mut size_t,
    ret_buf: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut len: size_t = 0 as size_t;
    let l_: *const list_T = list;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return false;
            }
            len = len.wrapping_add(1);
            if !(*li).li_tv.vval.v_string.is_null() {
                len = len.wrapping_add(strlen((*li).li_tv.vval.v_string));
            }
            li = (*li).li_next;
        }
    }
    if len != 0 {
        len = len.wrapping_sub(1);
    }
    *ret_len = len;
    if len == 0 as size_t {
        *ret_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
        return true_0 != 0;
    }
    let mut lrstate: ListReaderState = encode_init_lrstate(list);
    let buf: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    let mut read_bytes: size_t = 0;
    if encode_read_from_list(&raw mut lrstate, buf, len, &raw mut read_bytes) != OK {
        abort();
    }
    '_c2rust_label: {
        if len == read_bytes {
        } else {
            __assert_fail(
                b"len == read_bytes\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/encode.rs\0".as_ptr() as *const ::core::ffi::c_char,
                240 as ::core::ffi::c_uint,
                b"_Bool encode_vim_list_to_buf(const list_T *const, size_t *const, char **const)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    *ret_buf = buf;
    return true_0 != 0;
}
pub unsafe extern "C" fn encode_read_from_list(
    state: *mut ListReaderState,
    buf: *mut ::core::ffi::c_char,
    nbuf: size_t,
    read_bytes: *mut size_t,
) -> ::core::ffi::c_int {
    let buf_end: *mut ::core::ffi::c_char = buf.offset(nbuf as isize);
    let mut p: *mut ::core::ffi::c_char = buf;
    while p < buf_end {
        '_c2rust_label: {
            if (*state).li_length == 0 as size_t || !(*(*state).li).li_tv.vval.v_string.is_null() {
            } else {
                __assert_fail(
                    b"state->li_length == 0 || TV_LIST_ITEM_TV(state->li)->vval.v_string != NULL\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/encode.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    265 as ::core::ffi::c_uint,
                    b"int encode_read_from_list(ListReaderState *const, char *const, const size_t, size_t *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut i: size_t = (*state).offset;
        while i < (*state).li_length && p < buf_end {
            '_c2rust_label_0: {
                if !(*(*state).li).li_tv.vval.v_string.is_null() {
                } else {
                    __assert_fail(
                        b"TV_LIST_ITEM_TV(state->li)->vval.v_string != NULL\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/eval/encode.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        267 as ::core::ffi::c_uint,
                        b"int encode_read_from_list(ListReaderState *const, char *const, const size_t, size_t *const)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let c2rust_fresh27 = (*state).offset;
            (*state).offset = (*state).offset.wrapping_add(1);
            let ch: ::core::ffi::c_char = *(*(*state).li)
                .li_tv
                .vval
                .v_string
                .offset(c2rust_fresh27 as isize);
            let c2rust_fresh28 = p;
            p = p.offset(1);
            *c2rust_fresh28 =
                (if ch as ::core::ffi::c_int == NL as ::core::ffi::c_char as ::core::ffi::c_int {
                    NUL as ::core::ffi::c_char as ::core::ffi::c_int
                } else {
                    ch as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
            i = i.wrapping_add(1);
        }
        if p < buf_end {
            (*state).li = (*(*state).li).li_next;
            if (*state).li.is_null() {
                *read_bytes = p.offset_from(buf) as size_t;
                return OK;
            }
            let c2rust_fresh29 = p;
            p = p.offset(1);
            *c2rust_fresh29 = NL as ::core::ffi::c_char;
            if (*(*state).li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *read_bytes = p.offset_from(buf) as size_t;
                return FAIL;
            }
            (*state).offset = 0 as size_t;
            (*state).li_length = if (*(*state).li).li_tv.vval.v_string.is_null() {
                0 as size_t
            } else {
                strlen((*(*state).li).li_tv.vval.v_string)
            };
        }
    }
    *read_bytes = nbuf;
    return if (*state).offset < (*state).li_length || !(*(*state).li).li_next.is_null() {
        NOTDONE
    } else {
        OK
    };
}
pub const TYPVAL_ENCODE_ALLOW_SPECIALS: ::core::ffi::c_int = false_0;
pub const TYPVAL_ENCODE_ALLOW_SPECIALS_1: ::core::ffi::c_int = true_0;
static escapes: GlobalCell<[[::core::ffi::c_char; 3]; 93]> = GlobalCell::new(unsafe {
    [
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\b\0"),
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\t\0"),
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\n\0"),
        [0; 3],
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\f\0"),
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\r\0"),
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\\"\0"),
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\\\\0"),
    ]
});
static xdigits: GlobalCell<[::core::ffi::c_char; 17]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"0123456789ABCDEF\0")
});
#[inline(always)]
unsafe extern "C" fn convert_to_json_string(
    gap: *mut garray_T,
    buf: *const ::core::ffi::c_char,
    len: size_t,
) -> ::core::ffi::c_int {
    let mut utf_buf: *const ::core::ffi::c_char = buf;
    if utf_buf.is_null() {
        ga_concat_len(
            gap,
            b"\"\"\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
        );
    } else {
        let mut utf_len: size_t = len;
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut str_len: size_t = 0 as size_t;
        let mut i: size_t = 0 as size_t;
        while i < utf_len {
            let ch: ::core::ffi::c_int = utf_ptr2char(utf_buf.offset(i as isize));
            let shift: size_t = if ch == 0 as ::core::ffi::c_int {
                1 as size_t
            } else {
                utf_ptr2len(utf_buf.offset(i as isize)) as size_t
            };
            '_c2rust_label: {
                if shift > 0 as size_t {
                } else {
                    __assert_fail(
                        b"shift > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/encode.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        643 as ::core::ffi::c_uint,
                        b"int convert_to_json_string(garray_T *const, const char *const, const size_t)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            i = i.wrapping_add(shift);
            match ch {
                BS | TAB | NL | FF | CAR | 34 | 92 => {
                    str_len = str_len.wrapping_add(2 as size_t);
                }
                _ => {
                    if ch > 0x7f as ::core::ffi::c_int && shift == 1 as size_t {
                        semsg(
                            gettext(
                                b"E474: String \"%.*s\" contains byte that does not start any UTF-8 character\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            utf_len.wrapping_sub(i.wrapping_sub(shift))
                                as ::core::ffi::c_int,
                            utf_buf.offset(i as isize).offset(-(shift as isize)),
                        );
                        xfree(tofree as *mut ::core::ffi::c_void);
                        return FAIL;
                    } else if SURROGATE_HI_START <= ch && ch <= SURROGATE_HI_END
                        || SURROGATE_LO_START <= ch && ch <= SURROGATE_LO_END
                    {
                        semsg(
                            gettext(
                                b"E474: UTF-8 string contains code point which belongs to a surrogate pair: %.*s\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            utf_len.wrapping_sub(i.wrapping_sub(shift))
                                as ::core::ffi::c_int,
                            utf_buf.offset(i as isize).offset(-(shift as isize)),
                        );
                        xfree(tofree as *mut ::core::ffi::c_void);
                        return FAIL;
                    } else if ch >= 0x20 as ::core::ffi::c_int
                        && utf_printable(ch) as ::core::ffi::c_int != 0
                    {
                        str_len = str_len.wrapping_add(shift);
                    } else {
                        str_len = (str_len as ::core::ffi::c_ulong).wrapping_add(
                            ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                .wrapping_sub(1 as usize)
                                .wrapping_mul(
                                    (1 as ::core::ffi::c_int
                                        + (ch >= SURROGATE_FIRST_CHAR) as ::core::ffi::c_int)
                                        as usize,
                                ) as ::core::ffi::c_ulong,
                        ) as size_t;
                    }
                }
            }
        }
        ga_append(gap, '"' as uint8_t);
        ga_grow(gap, str_len as ::core::ffi::c_int);
        let mut i_0: size_t = 0 as size_t;
        while i_0 < utf_len {
            let ch_0: ::core::ffi::c_int = utf_ptr2char(utf_buf.offset(i_0 as isize));
            let shift_0: size_t = if ch_0 == 0 as ::core::ffi::c_int {
                1 as size_t
            } else {
                utf_char2len(ch_0) as size_t
            };
            '_c2rust_label_0: {
                if shift_0 > 0 as size_t {
                } else {
                    __assert_fail(
                        b"shift > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/encode.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        683 as ::core::ffi::c_uint,
                        b"int convert_to_json_string(garray_T *const, const char *const, const size_t)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            '_c2rust_label_1: {
                if ch_0 == 0 as ::core::ffi::c_int
                    || shift_0 == utf_ptr2len(utf_buf.offset(i_0 as isize)) as size_t
                {
                } else {
                    __assert_fail(
                        b"ch == 0 || shift == ((size_t)utf_ptr2len(utf_buf + i))\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/encode.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        685 as ::core::ffi::c_uint,
                        b"int convert_to_json_string(garray_T *const, const char *const, const size_t)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            match ch_0 {
                BS | TAB | NL | FF | CAR | 34 | 92 => {
                    ga_concat_len(
                        gap,
                        &raw const *((escapes.ptr() as *const _) as *const [::core::ffi::c_char; 3])
                            .offset(ch_0 as isize)
                            as *const ::core::ffi::c_char,
                        2 as size_t,
                    );
                }
                _ => {
                    if ch_0 >= 0x20 as ::core::ffi::c_int
                        && utf_printable(ch_0) as ::core::ffi::c_int != 0
                    {
                        ga_concat_len(gap, utf_buf.offset(i_0 as isize), shift_0);
                    } else if ch_0 < SURROGATE_FIRST_CHAR {
                        let c2rust_lvalue: [::core::ffi::c_char; 6] = [
                            '\\' as ::core::ffi::c_char,
                            'u' as ::core::ffi::c_char,
                            (*xdigits.ptr())[(ch_0
                                >> 4 as ::core::ffi::c_int * 3 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(ch_0
                                >> 4 as ::core::ffi::c_int * 2 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(ch_0
                                >> 4 as ::core::ffi::c_int * 1 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(ch_0
                                >> 4 as ::core::ffi::c_int * 0 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                        ];
                        ga_concat_len(
                            gap,
                            &raw const c2rust_lvalue as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                .wrapping_sub(1 as size_t),
                        );
                    } else {
                        let tmp: ::core::ffi::c_int = ch_0 - SURROGATE_FIRST_CHAR;
                        let hi: ::core::ffi::c_int = SURROGATE_HI_START
                            + (tmp >> 10 as ::core::ffi::c_int
                                & ((1 as ::core::ffi::c_int) << 10 as ::core::ffi::c_int)
                                    - 1 as ::core::ffi::c_int);
                        let lo: ::core::ffi::c_int = SURROGATE_LO_END
                            + (tmp >> 0 as ::core::ffi::c_int
                                & ((1 as ::core::ffi::c_int) << 10 as ::core::ffi::c_int)
                                    - 1 as ::core::ffi::c_int);
                        let c2rust_lvalue_0: [::core::ffi::c_char; 12] = [
                            '\\' as ::core::ffi::c_char,
                            'u' as ::core::ffi::c_char,
                            (*xdigits.ptr())[(hi
                                >> 4 as ::core::ffi::c_int * 3 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(hi
                                >> 4 as ::core::ffi::c_int * 2 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(hi
                                >> 4 as ::core::ffi::c_int * 1 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(hi
                                >> 4 as ::core::ffi::c_int * 0 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            '\\' as ::core::ffi::c_char,
                            'u' as ::core::ffi::c_char,
                            (*xdigits.ptr())[(lo
                                >> 4 as ::core::ffi::c_int * 3 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(lo
                                >> 4 as ::core::ffi::c_int * 2 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(lo
                                >> 4 as ::core::ffi::c_int * 1 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(lo
                                >> 4 as ::core::ffi::c_int * 0 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                        ];
                        ga_concat_len(
                            gap,
                            &raw const c2rust_lvalue_0 as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                .wrapping_sub(1 as size_t)
                                .wrapping_mul(2 as size_t),
                        );
                    }
                }
            }
            i_0 = i_0.wrapping_add(shift_0);
        }
        ga_append(gap, '"' as uint8_t);
        xfree(tofree as *mut ::core::ffi::c_void);
    }
    return OK;
}
pub unsafe extern "C" fn encode_check_json_key(tv: *const typval_T) -> bool {
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return true_0 != 0;
    }
    if (*tv).v_type as ::core::ffi::c_uint != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return false_0 != 0;
    }
    let spdict: *const dict_T = (*tv).vval.v_dict;
    if (*spdict).dv_hashtab.ht_used != 2 as size_t {
        return false_0 != 0;
    }
    let mut type_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
    let mut val_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
    type_di = tv_dict_find(
        spdict,
        b"_TYPE\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if type_di.is_null()
        || (*type_di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*type_di).di_tv.vval.v_list
            != (*eval_msgpack_type_lists.ptr())[kMPString as ::core::ffi::c_int as usize]
                as *mut list_T
        || {
            val_di = tv_dict_find(
                spdict,
                b"_VAL\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            val_di.is_null()
        }
        || (*val_di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return false_0 != 0;
    }
    if (*val_di).di_tv.vval.v_list.is_null() {
        return true_0 != 0;
    }
    let l_: *const list_T = (*val_di).di_tv.vval.v_list;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return false;
            }
            li = (*li).li_next;
        }
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn encode_tv2string(
    mut tv: *mut typval_T,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    let evs_ret: ::core::ffi::c_int = encode_vim_to_string(
        &raw mut ga,
        tv,
        b"encode_tv2string() argument\0".as_ptr() as *const ::core::ffi::c_char,
    );
    '_c2rust_label: {
        if evs_ret == 1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"evs_ret == OK\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/encode.rs\0".as_ptr() as *const ::core::ffi::c_char,
                877 as ::core::ffi::c_uint,
                b"char *encode_tv2string(typval_T *, size_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    did_echo_string_emsg.set(false_0 != 0);
    if !len.is_null() {
        *len = ga.ga_len as size_t;
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    return ga.ga_data as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn encode_tv2echo(
    mut tv: *mut typval_T,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*tv).v_type as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !(*tv).vval.v_string.is_null() {
            ga_concat(&raw mut ga, (*tv).vval.v_string);
        }
    } else {
        let eve_ret: ::core::ffi::c_int = encode_vim_to_echo(
            &raw mut ga,
            tv,
            b":echo argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
        '_c2rust_label: {
            if eve_ret == 1 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"eve_ret == OK\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/encode.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    905 as ::core::ffi::c_uint,
                    b"char *encode_tv2echo(typval_T *, size_t *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
    }
    if !len.is_null() {
        *len = ga.ga_len as size_t;
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    return ga.ga_data as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn encode_tv2json(
    mut tv: *mut typval_T,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    let evj_ret: ::core::ffi::c_int = encode_vim_to_json(
        &raw mut ga,
        tv,
        b"encode_tv2json() argument\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if evj_ret == 0 {
        ga_clear(&raw mut ga);
    }
    did_echo_string_emsg.set(false_0 != 0);
    if !len.is_null() {
        *len = ga.ga_len as size_t;
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    return ga.ga_data as *mut ::core::ffi::c_char;
}
pub const TYPVAL_ENCODE_ALLOW_SPECIALS_0: ::core::ffi::c_int = true_0;
pub unsafe extern "C" fn encode_init_lrstate(list: *const list_T) -> ListReaderState {
    return ListReaderState {
        list: list,
        li: tv_list_first(list),
        offset: 0 as size_t,
        li_length: if (*tv_list_first(list)).li_tv.vval.v_string.is_null() {
            0 as size_t
        } else {
            strlen((*tv_list_first(list)).li_tv.vval.v_string)
        },
    };
}
#[inline]
unsafe extern "C" fn tv_list_set_copyid(l: *mut list_T, copyid: ::core::ffi::c_int) {
    (*l).lv_copyID = copyid;
}
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
#[inline]
unsafe extern "C" fn tv_list_copyid(l: *const list_T) -> ::core::ffi::c_int {
    return (*l).lv_copyID;
}
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
#[inline]
unsafe extern "C" fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_last;
}
#[inline]
unsafe extern "C" fn tv_blob_len(b: *const blob_T) -> ::core::ffi::c_int {
    if b.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*b).bv_ga.ga_len;
}
#[inline(always)]
unsafe extern "C" fn tv_blob_get(b: *const blob_T, mut idx: ::core::ffi::c_int) -> uint8_t {
    return *((*b).bv_ga.ga_data as *mut uint8_t).offset(idx as isize);
}
#[inline(always)]
unsafe extern "C" fn tv_strlen(tv: *const typval_T) -> size_t {
    '_c2rust_label: {
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"tv->v_type == VAR_STRING\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/encode.rs\0".as_ptr() as *const ::core::ffi::c_char,
                77 as ::core::ffi::c_uint,
                b"size_t tv_strlen(const typval_T *const)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return if (*tv).vval.v_string.is_null() {
        0 as size_t
    } else {
        strlen((*tv).vval.v_string)
    };
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mpack_w2(mut b: *mut *mut ::core::ffi::c_char, mut v: uint32_t) {
    let c2rust_fresh17 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh17 = (v >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh18 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh18 = (v & 0xff as uint32_t) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_w4(mut b: *mut *mut ::core::ffi::c_char, mut v: uint32_t) {
    let c2rust_fresh13 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh13 = (v >> 24 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh14 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh14 = (v >> 16 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh15 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh15 = (v >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh16 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh16 = (v & 0xff as uint32_t) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_bool(mut buf: *mut *mut ::core::ffi::c_char, mut val: bool) {
    let c2rust_fresh41 = *buf;
    *buf = (*buf).offset(1);
    *c2rust_fresh41 = (0xc2 as ::core::ffi::c_int
        | (if val as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_array(mut buf: *mut *mut ::core::ffi::c_char, mut len: uint32_t) {
    if len < 0x10 as uint32_t {
        let c2rust_fresh38 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh38 = (0x90 as uint32_t | len) as ::core::ffi::c_char;
    } else if len < 0x10000 as uint32_t {
        let c2rust_fresh39 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh39 = 0xdc as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, len);
    } else {
        let c2rust_fresh40 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh40 = 0xdd as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, len);
    };
}
#[inline]
unsafe extern "C" fn mpack_map(mut buf: *mut *mut ::core::ffi::c_char, mut len: uint32_t) {
    if len < 0x10 as uint32_t {
        let c2rust_fresh10 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh10 = (0x80 as uint32_t | len) as ::core::ffi::c_char;
    } else if len < 0x10000 as uint32_t {
        let c2rust_fresh11 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh11 = 0xde as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, len);
    } else {
        let c2rust_fresh12 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh12 = 0xdf as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, len);
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
