use crate::src::nvim::eval::typval::{
    tv_clear, tv_get_number, tv_get_string, tv_get_string_buf, tv_list_extend,
};
use crate::src::nvim::eval_1::{grow_string_tv, num_divide, num_modulus};
use crate::src::nvim::garray::ga_grow;
use crate::src::nvim::main::e_letwrong;
use crate::src::nvim::message::semsg;
use crate::src::nvim::os::libc::{abort, memmove};
use crate::src::nvim::strings::{concat_str, vim_strchr};
pub use crate::src::nvim::types::{
    blob_T, blobvar_S, dict_T, dictvar_S, float_T, funccall_S,
    funccall_S_fc_fixvar as C2Rust_Unnamed, funccall_T, garray_T, hash_T, hashitem_T, hashtab_T,
    int32_t, int64_t, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S,
    listwatch_T, partial_S, partial_T, proftime_T, queue, scid_T, sctx_T, size_t, typval_T,
    typval_vval_union, ufunc_S, ufunc_T, uint64_t, uint8_t, varnumber_T, BoolVarValue, LuaRef,
    ScopeDictDictItem, ScopeType, SpecialVarValue, VarLockStatus, VarType, QUEUE,
};
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
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
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn tv_op_blob(
    mut tv1: *mut typval_T,
    mut tv2: *const typval_T,
    mut op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *op as ::core::ffi::c_int != '+' as ::core::ffi::c_int
        || (*tv2).v_type as ::core::ffi::c_uint
            != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    if (*tv2).vval.v_blob.is_null() {
        return OK;
    }
    if (*tv1).vval.v_blob.is_null() {
        (*tv1).vval.v_blob = (*tv2).vval.v_blob;
        (*(*tv1).vval.v_blob).bv_refcount += 1;
        return OK;
    }
    let b1: *mut blob_T = (*tv1).vval.v_blob;
    let b2: *mut blob_T = (*tv2).vval.v_blob;
    let len: ::core::ffi::c_int = tv_blob_len(b2);
    if len > 0 as ::core::ffi::c_int {
        ga_grow(&raw mut (*b1).bv_ga, len);
        memmove(
            ((*b1).bv_ga.ga_data as *mut uint8_t).offset((*b1).bv_ga.ga_len as isize)
                as *mut ::core::ffi::c_void,
            (*b2).bv_ga.ga_data as *mut uint8_t as *const ::core::ffi::c_void,
            len as size_t,
        );
        (*b1).bv_ga.ga_len += len;
    }
    return OK;
}
unsafe extern "C" fn tv_op_list(
    mut tv1: *mut typval_T,
    mut tv2: *const typval_T,
    mut op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *op as ::core::ffi::c_int != '+' as ::core::ffi::c_int
        || (*tv2).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    if (*tv2).vval.v_list.is_null() {
        return OK;
    }
    if (*tv1).vval.v_list.is_null() {
        (*tv1).vval.v_list = (*tv2).vval.v_list;
        tv_list_ref((*tv1).vval.v_list);
    } else {
        tv_list_extend(
            (*tv1).vval.v_list,
            (*tv2).vval.v_list,
            ::core::ptr::null_mut::<listitem_T>(),
        );
    }
    return OK;
}
unsafe extern "C" fn tv_op_number(
    mut tv1: *mut typval_T,
    mut tv2: *const typval_T,
    mut op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut n: varnumber_T = tv_get_number(tv1);
    if (*tv2).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut f: float_T = n as float_T;
        if *op as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
            return FAIL;
        }
        match *op as ::core::ffi::c_int {
            43 => {
                f += (*tv2).vval.v_float;
            }
            45 => {
                f -= (*tv2).vval.v_float;
            }
            42 => {
                f *= (*tv2).vval.v_float;
            }
            47 => {
                f /= (*tv2).vval.v_float;
            }
            _ => {}
        }
        tv_clear(tv1);
        (*tv1).v_type = VAR_FLOAT;
        (*tv1).vval.v_float = f;
    } else {
        match *op as ::core::ffi::c_int {
            43 => {
                n += tv_get_number(tv2);
            }
            45 => {
                n -= tv_get_number(tv2);
            }
            42 => {
                n *= tv_get_number(tv2);
            }
            47 => {
                n = num_divide(n, tv_get_number(tv2));
            }
            37 => {
                n = num_modulus(n, tv_get_number(tv2));
            }
            _ => {}
        }
        tv_clear(tv1);
        (*tv1).v_type = VAR_NUMBER;
        (*tv1).vval.v_number = n;
    }
    return OK;
}
unsafe extern "C" fn tv_op_string(
    mut tv1: *mut typval_T,
    mut tv2: *const typval_T,
    mut _op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*tv2).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut s2: *const ::core::ffi::c_char =
        tv_get_string_buf(tv2, &raw mut numbuf as *mut ::core::ffi::c_char);
    if grow_string_tv(tv1, s2) == OK {
        return OK;
    }
    let mut tvs: *const ::core::ffi::c_char = tv_get_string(tv1);
    let s: *mut ::core::ffi::c_char = concat_str(tvs, s2);
    tv_clear(tv1);
    (*tv1).v_type = VAR_STRING;
    (*tv1).vval.v_string = s;
    return OK;
}
unsafe extern "C" fn tv_op_nr_or_string(
    mut tv1: *mut typval_T,
    mut tv2: *const typval_T,
    mut op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*tv2).v_type as ::core::ffi::c_uint == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    if !vim_strchr(
        b"+-*/%\0".as_ptr() as *const ::core::ffi::c_char,
        *op as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
    {
        return tv_op_number(tv1, tv2, op);
    }
    return tv_op_string(tv1, tv2, op);
}
unsafe extern "C" fn tv_op_float(
    mut tv1: *mut typval_T,
    mut tv2: *const typval_T,
    mut op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *op as ::core::ffi::c_int == '%' as ::core::ffi::c_int
        || *op as ::core::ffi::c_int == '.' as ::core::ffi::c_int
        || (*tv2).v_type as ::core::ffi::c_uint
            != VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*tv2).v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*tv2).v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    let f: float_T = if (*tv2).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*tv2).vval.v_float
    } else {
        tv_get_number(tv2) as float_T
    };
    match *op as ::core::ffi::c_int {
        43 => {
            (*tv1).vval.v_float += f;
        }
        45 => {
            (*tv1).vval.v_float -= f;
        }
        42 => {
            (*tv1).vval.v_float *= f;
        }
        47 => {
            (*tv1).vval.v_float /= f;
        }
        _ => {}
    }
    return OK;
}
pub unsafe extern "C" fn eexe_mod_op(
    tv1: *mut typval_T,
    tv2: *const typval_T,
    op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*tv2).v_type as ::core::ffi::c_uint == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*tv2).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        || ((*tv2).v_type as ::core::ffi::c_uint
            == VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*tv2).v_type as ::core::ffi::c_uint
                == VAR_SPECIAL as ::core::ffi::c_int as ::core::ffi::c_uint)
            && *op as ::core::ffi::c_int == '.' as ::core::ffi::c_int
    {
        semsg(
            &raw const e_letwrong as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            op,
        );
        return FAIL;
    }
    let mut retval: ::core::ffi::c_int = FAIL;
    match (*tv1).v_type as ::core::ffi::c_uint {
        10 => {
            retval = tv_op_blob(tv1, tv2, op);
        }
        4 => {
            retval = tv_op_list(tv1, tv2, op);
        }
        1 | 2 => {
            retval = tv_op_nr_or_string(tv1, tv2, op);
        }
        6 => {
            retval = tv_op_float(tv1, tv2, op);
        }
        0 => {
            abort();
        }
        5 | 3 | 9 | 7 | 8 | _ => {}
    }
    if retval != OK {
        semsg(
            &raw const e_letwrong as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            op,
        );
    }
    return retval;
}
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
#[inline]
unsafe extern "C" fn tv_blob_len(b: *const blob_T) -> ::core::ffi::c_int {
    if b.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*b).bv_ga.ga_len;
}
