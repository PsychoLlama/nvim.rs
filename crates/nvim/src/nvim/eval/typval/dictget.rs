//! Reading values back out of a `dict_T`.
//!
//! [`tv_dict_find`] is the hashtable lookup every getter goes through, and
//! the `tv_dict_get_*` family coerces what it finds to one type, answering a
//! caller-supplied default when the key is absent or the wrong kind.
//! [`tv_dict_to_env`] builds the `environ`-shaped array a job's environment
//! is passed as.  The `*2items` half and [`f_items`] / [`f_keys`] /
//! [`f_values`] are the builtins that turn a container into a list.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn tv_blob2items(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    unsafe {
        let mut blob: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob;
        tv_list_alloc_ret(rettv, tv_blob_len(blob) as ptrdiff_t);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < tv_blob_len(blob) {
            let mut l2: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
            tv_list_append_list((*rettv).vval.v_list, l2);
            tv_list_append_number(l2, i as varnumber_T);
            tv_list_append_number(l2, tv_blob_get(blob, i) as varnumber_T);
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn tv_dict2items(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    unsafe {
        tv_dict2list(argvars, rettv, kDict2ListItems);
    }
}

pub(crate) unsafe extern "C" fn tv_list2items(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    unsafe {
        let mut l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        tv_list_alloc_ret(rettv, tv_list_len(l) as ptrdiff_t);
        if l.is_null() {
            return;
        }
        let mut idx: varnumber_T = 0 as varnumber_T;
        let l_: *mut list_T = l;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let mut l2: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
                tv_list_append_list((*rettv).vval.v_list, l2);
                tv_list_append_number(l2, idx);
                tv_list_append_tv(l2, &raw mut (*li).li_tv);
                idx += 1;
                li = (*li).li_next;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn tv_string2items(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    unsafe {
        let mut p: *const ::core::ffi::c_char = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string;
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        if p.is_null() {
            return;
        }
        let mut idx: varnumber_T = 0 as varnumber_T;
        while *p as ::core::ffi::c_int != NUL {
            let mut len: ::core::ffi::c_int = utfc_ptr2len(p);
            if len == 0 as ::core::ffi::c_int {
                break;
            }
            let mut l2: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
            tv_list_append_list((*rettv).vval.v_list, l2);
            tv_list_append_number(l2, idx);
            tv_list_append_string(l2, p, len as ssize_t);
            p = p.offset(len as isize);
            idx += 1;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_find(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    len: ptrdiff_t,
) -> *mut dictitem_T {
    unsafe {
        if d.is_null() {
            return ::core::ptr::null_mut::<dictitem_T>();
        }
        let hi: *mut hashitem_T = if len < 0 as ptrdiff_t {
            hash_find(&raw const (*d).dv_hashtab, key)
        } else {
            hash_find_len(&raw const (*d).dv_hashtab, key, len as size_t)
        };
        if (*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            return ::core::ptr::null_mut::<dictitem_T>();
        }
        return (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
    }
}

pub unsafe extern "C" fn tv_dict_has_key(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
) -> bool {
    unsafe {
        return !tv_dict_find(d, key, -1 as ptrdiff_t).is_null();
    }
}

pub unsafe extern "C" fn tv_dict_get_tv(
    mut d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        let di: *mut dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
        if di.is_null() {
            return FAIL;
        }
        tv_copy(&raw mut (*di).di_tv, rettv);
        return OK;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_get_number(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
) -> varnumber_T {
    unsafe {
        return tv_dict_get_number_def(d, key, 0 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn tv_dict_get_number_def(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    def: ::core::ffi::c_int,
) -> varnumber_T {
    unsafe {
        let di: *mut dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
        if di.is_null() {
            return def as varnumber_T;
        }
        return tv_get_number(&raw mut (*di).di_tv);
    }
}

pub unsafe extern "C" fn tv_dict_get_bool(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    def: ::core::ffi::c_int,
) -> varnumber_T {
    unsafe {
        let di: *mut dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
        if di.is_null() {
            return def as varnumber_T;
        }
        return tv_get_bool(&raw mut (*di).di_tv);
    }
}

pub unsafe extern "C" fn tv_dict_to_env(mut denv: *mut dict_T) -> *mut *mut ::core::ffi::c_char {
    unsafe {
        let mut env_size: size_t = tv_dict_len(denv) as size_t;
        let mut i: size_t = 0 as size_t;
        let mut env: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        env = xmalloc(
            env_size
                .wrapping_add(1 as size_t)
                .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
        ) as *mut *mut ::core::ffi::c_char;
        let varhi_ht_: *mut hashtab_T = &raw mut (*denv).dv_hashtab;
        let mut varhi_todo_: size_t = (*varhi_ht_).ht_used;
        let mut varhi_: *mut hashitem_T = (*varhi_ht_).ht_array;
        while varhi_todo_ != 0 {
            if !((*varhi_).hi_key.is_null()
                || (*varhi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                varhi_todo_ = varhi_todo_.wrapping_sub(1);
                let var: *mut dictitem_T = (*varhi_)
                    .hi_key
                    .offset(-(17 as ::core::ffi::c_ulong as isize))
                    as *mut dictitem_T;
                let mut str: *const ::core::ffi::c_char = tv_get_string(&raw mut (*var).di_tv);
                '_c2rust_label: {
                    if !str.is_null() {
                    } else {
                        __assert_fail(
                            b"str\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            2339 as ::core::ffi::c_uint,
                            b"char **tv_dict_to_env(dict_T *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                let mut len: size_t = strlen(&raw mut (*var).di_key as *mut ::core::ffi::c_char)
                    .wrapping_add(strlen(str))
                    .wrapping_add(strlen(b"=\0".as_ptr() as *const ::core::ffi::c_char))
                    .wrapping_add(1 as size_t);
                *env.offset(i as isize) = xmalloc(len) as *mut ::core::ffi::c_char;
                snprintf(
                    *env.offset(i as isize),
                    len,
                    b"%s=%s\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut (*var).di_key as *mut ::core::ffi::c_char,
                    str,
                );
                i = i.wrapping_add(1);
            }
            varhi_ = varhi_.offset(1);
        }
        *env.offset(env_size as isize) = ::core::ptr::null_mut::<::core::ffi::c_char>();
        return env;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_get_string(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    save: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        static numbuf: GlobalCell<[::core::ffi::c_char; 65]> = GlobalCell::new([0; 65]);
        let s: *const ::core::ffi::c_char =
            tv_dict_get_string_buf(d, key, numbuf.ptr() as *mut ::core::ffi::c_char);
        if save as ::core::ffi::c_int != 0 && !s.is_null() {
            return xstrdup(s);
        }
        return s as *mut ::core::ffi::c_char;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_get_string_buf(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    numbuf: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let di: *const dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
        if di.is_null() {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        return tv_get_string_buf(&raw const (*di).di_tv, numbuf);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_get_string_buf_chk(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    key_len: ptrdiff_t,
    numbuf: *mut ::core::ffi::c_char,
    def: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let di: *const dictitem_T = tv_dict_find(d, key, key_len);
        if di.is_null() {
            return def;
        }
        return tv_get_string_buf_chk(&raw const (*di).di_tv, numbuf);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_get_callback(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: ptrdiff_t,
    result: *mut Callback,
) -> bool {
    unsafe {
        (*result).type_0 = kCallbackNone;
        let di: *mut dictitem_T = tv_dict_find(d, key, key_len);
        if di.is_null() {
            return true_0 != 0;
        }
        if !tv_is_func((*di).di_tv)
            && (*di).di_tv.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(
                b"E6000: Argument is not a function or function name\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return false_0 != 0;
        }
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv_copy(&raw mut (*di).di_tv, &raw mut tv);
        set_selfdict(&raw mut tv, d);
        let res: bool = callback_from_typval(result, &raw mut tv);
        tv_clear(&raw mut tv);
        return res;
    }
}

pub unsafe extern "C" fn tv_dict_wrong_func_name(
    mut d: *mut dict_T,
    mut tv: *mut typval_T,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return ((d == get_globvar_dict() || &raw mut (*d).dv_hashtab == get_funccal_local_ht())
            && tv_is_func(*tv) as ::core::ffi::c_int != 0
            && var_wrong_func_name(name, true_0 != 0) as ::core::ffi::c_int != 0)
            as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn tv_dict2list(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    what: DictListType,
) {
    unsafe {
        if tv_check_for_dict_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
            tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
            return;
        }
        let mut d: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        tv_list_alloc_ret(rettv, tv_dict_len(d) as ptrdiff_t);
        if d.is_null() {
            return;
        }
        let dihi_ht_: *mut hashtab_T = &raw mut (*d).dv_hashtab;
        let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
        let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
        while dihi_todo_ != 0 {
            if !((*dihi_).hi_key.is_null()
                || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                dihi_todo_ = dihi_todo_.wrapping_sub(1);
                let di: *mut dictitem_T = (*dihi_)
                    .hi_key
                    .offset(-(17 as ::core::ffi::c_ulong as isize))
                    as *mut dictitem_T;
                let mut tv_item: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                match what as ::core::ffi::c_uint {
                    0 => {
                        tv_item.v_type = VAR_STRING;
                        tv_item.vval.v_string =
                            xstrdup(&raw mut (*di).di_key as *mut ::core::ffi::c_char);
                    }
                    1 => {
                        tv_copy(&raw mut (*di).di_tv, &raw mut tv_item);
                    }
                    2 => {
                        let sub_l: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
                        tv_item.v_type = VAR_LIST;
                        tv_item.vval.v_list = sub_l;
                        tv_list_ref(sub_l);
                        tv_list_append_string(
                            sub_l,
                            &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                            -1 as ssize_t,
                        );
                        tv_list_append_tv(sub_l, &raw mut (*di).di_tv);
                    }
                    _ => {}
                }
                tv_list_append_owned_tv((*rettv).vval.v_list, tv_item);
            }
            dihi_ = dihi_.offset(1);
        }
    }
}

pub unsafe extern "C" fn f_items(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_string2items(argvars, rettv);
        } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_list2items(argvars, rettv);
        } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_blob2items(argvars, rettv);
        } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_dict2items(argvars, rettv);
        } else {
            semsg(
                gettext(
                    (e_list_dict_blob_or_string_required_for_argument_nr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                1 as ::core::ffi::c_int,
            );
        };
    }
}

pub unsafe extern "C" fn f_keys(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_dict2list(argvars, rettv, kDict2ListKeys);
    }
}

pub unsafe extern "C" fn f_values(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_dict2list(argvars, rettv, kDict2ListValues);
    }
}

pub unsafe extern "C" fn f_has_key(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if tv_check_for_dict_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
            return;
        }
        if (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict
            .is_null()
        {
            return;
        }
        (*rettv).vval.v_number = !tv_dict_find(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
            -1 as ptrdiff_t,
        )
        .is_null() as ::core::ffi::c_int as varnumber_T;
    }
}
