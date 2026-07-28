//! Resolving the left-hand side of an assignment, and performing it.
//!
//! `get_lval` walks a name and its subscripts down to the container and key
//! that `set_var_lval` will write through.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn to_name_end(
    mut arg: *const c_char,
    mut use_namespace: bool,
) -> *const c_char {
    if !eval_isnamec1(*arg as c_int) {
        return arg;
    }
    let mut p: *const c_char = ::core::ptr::null::<c_char>();
    p = arg.offset(1 as c_int as isize);
    while *p as c_int != NUL && eval_isnamec(*p as c_int) as c_int != 0 {
        if *p as c_int == ':' as c_int
            && (p != arg.offset(1 as c_int as isize)
                || !use_namespace
                || vim_strchr(b"bgstvw\0".as_ptr() as *const c_char, *arg as c_int).is_null())
        {
            break;
        }
        p = p.offset(utfc_ptr2len(p as *mut c_char) as isize);
    }
    return p;
}

pub(crate) unsafe extern "C" fn get_lval_dict_item(
    mut lp: *mut lval_T,
    mut name: *mut c_char,
    mut key: *mut c_char,
    mut len: c_int,
    mut key_end: *mut *mut c_char,
    mut var1: *mut typval_T,
    mut flags: c_int,
    mut unlet: bool,
    mut rettv: *mut typval_T,
) -> glv_status_T {
    let mut quiet: bool = flags & GLV_QUIET as c_int != 0;
    let mut p: *mut c_char = *key_end;
    if len == -1 as c_int {
        key = tv_get_string(var1) as *mut c_char;
    }
    (*lp).ll_list = ::core::ptr::null_mut::<list_T>();
    if (*(*lp).ll_tv).vval.v_dict.is_null() {
        (*(*lp).ll_tv).vval.v_dict = tv_dict_alloc();
        (*(*(*lp).ll_tv).vval.v_dict).dv_refcount += 1;
    }
    (*lp).ll_dict = (*(*lp).ll_tv).vval.v_dict;
    (*lp).ll_di = tv_dict_find((*lp).ll_dict, key, len as ptrdiff_t);
    if !rettv.is_null() && (*(*lp).ll_dict).dv_scope as c_uint != 0 as c_uint {
        let mut prevval: c_char = 0;
        if len != -1 as c_int {
            prevval = *key.offset(len as isize);
            *key.offset(len as isize) = NUL as c_char;
        } else {
            prevval = 0 as c_char;
        }
        let mut wrong: bool = (*(*lp).ll_dict).dv_scope as c_uint
            == VAR_DEF_SCOPE as c_int as c_uint
            && tv_is_func(*rettv) as c_int != 0
            && var_wrong_func_name(key, (*lp).ll_di.is_null()) as c_int != 0
            || !valid_varname(key);
        if len != -1 as c_int {
            *key.offset(len as isize) = prevval;
        }
        if wrong {
            return GLV_FAIL;
        }
    }
    if !(*lp).ll_di.is_null()
        && tv_is_luafunc(&raw mut (*(*lp).ll_di).di_tv) as c_int != 0
        && len == -1 as c_int
        && rettv.is_null()
    {
        semsg(
            &raw const e_illvar as *const c_char,
            b"v:['lua']\0".as_ptr() as *const c_char,
        );
        return GLV_FAIL;
    }
    if (*lp).ll_di.is_null() {
        if (*lp).ll_dict == get_vimvar_dict()
            || &raw mut (*(*lp).ll_dict).dv_hashtab == get_funccal_args_ht()
        {
            semsg(gettext(&raw const e_illvar as *const c_char), name);
            return GLV_FAIL;
        }
        if *p as c_int == '[' as c_int || *p as c_int == '.' as c_int || unlet as c_int != 0 {
            if !quiet {
                semsg(gettext(&raw const e_dictkey as *const c_char), key);
            }
            return GLV_FAIL;
        }
        if len == -1 as c_int {
            (*lp).ll_newkey = xstrdup(key);
        } else {
            (*lp).ll_newkey = xmemdupz(key as *const c_void, len as size_t) as *mut c_char;
        }
        *key_end = p;
        return GLV_STOP;
    } else if flags & GLV_READ_ONLY as c_int == 0
        && (var_check_ro(
            (*(*lp).ll_di).di_flags as c_int,
            name,
            p.offset_from(name) as size_t,
        ) as c_int
            != 0
            || var_check_lock(
                (*(*lp).ll_di).di_flags as c_int,
                name,
                p.offset_from(name) as size_t,
            ) as c_int
                != 0)
    {
        return GLV_FAIL;
    }
    (*lp).ll_tv = &raw mut (*(*lp).ll_di).di_tv;
    return GLV_OK;
}

pub(crate) unsafe extern "C" fn get_lval_blob(
    mut lp: *mut lval_T,
    mut var1: *mut typval_T,
    mut var2: *mut typval_T,
    mut empty1: bool,
    mut quiet: bool,
) -> c_int {
    let bloblen: c_int = tv_blob_len((*(*lp).ll_tv).vval.v_blob);
    if empty1 {
        (*lp).ll_n1 = 0 as c_int;
    } else {
        (*lp).ll_n1 = tv_get_number(var1) as c_int;
    }
    if tv_blob_check_index(bloblen, (*lp).ll_n1 as varnumber_T, quiet) == FAIL {
        return FAIL;
    }
    if (*lp).ll_range as c_int != 0 && !(*lp).ll_empty2 {
        (*lp).ll_n2 = tv_get_number(var2) as c_int;
        if tv_blob_check_range(
            bloblen,
            (*lp).ll_n1 as varnumber_T,
            (*lp).ll_n2 as varnumber_T,
            quiet,
        ) == FAIL
        {
            return FAIL;
        }
    }
    (*lp).ll_blob = (*(*lp).ll_tv).vval.v_blob;
    (*lp).ll_tv = ::core::ptr::null_mut::<typval_T>();
    return OK;
}

pub(crate) unsafe extern "C" fn get_lval_list(
    mut lp: *mut lval_T,
    mut var1: *mut typval_T,
    mut var2: *mut typval_T,
    mut empty1: bool,
    mut _flags: c_int,
    mut quiet: bool,
) -> c_int {
    if empty1 {
        (*lp).ll_n1 = 0 as c_int;
    } else {
        (*lp).ll_n1 = tv_get_number(var1) as c_int;
    }
    (*lp).ll_dict = ::core::ptr::null_mut::<dict_T>();
    (*lp).ll_list = (*(*lp).ll_tv).vval.v_list;
    (*lp).ll_li = tv_list_check_range_index_one((*lp).ll_list, &raw mut (*lp).ll_n1, quiet);
    if (*lp).ll_li.is_null() {
        return FAIL;
    }
    if (*lp).ll_range as c_int != 0 && !(*lp).ll_empty2 {
        (*lp).ll_n2 = tv_get_number(var2) as c_int;
        if tv_list_check_range_index_two(
            (*lp).ll_list,
            &raw mut (*lp).ll_n1,
            (*lp).ll_li,
            &raw mut (*lp).ll_n2,
            quiet,
        ) == FAIL
        {
            return FAIL;
        }
    }
    (*lp).ll_tv = &raw mut (*(*lp).ll_li).li_tv;
    return OK;
}

pub(crate) unsafe extern "C" fn get_lval_subscript(
    mut lp: *mut lval_T,
    mut p: *mut c_char,
    mut name: *mut c_char,
    mut rettv: *mut typval_T,
    mut _ht: *mut hashtab_T,
    mut _v: *mut dictitem_T,
    mut unlet: bool,
    mut flags: c_int,
) -> *mut c_char {
    let mut quiet: bool = flags & GLV_QUIET as c_int != 0;
    let mut var1: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    var1.v_type = VAR_UNKNOWN;
    let mut var2: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    var2.v_type = VAR_UNKNOWN;
    let mut empty1: bool = false_0 != 0;
    let mut rc: c_int = FAIL;
    '_done: {
        while *p as c_int == '[' as c_int
            || *p as c_int == '.' as c_int
                && *p.offset(1 as c_int as isize) as c_int != '=' as c_int
                && *p.offset(1 as c_int as isize) as c_int != '.' as c_int
        {
            if *p as c_int == '.' as c_int
                && (*(*lp).ll_tv).v_type as c_uint != VAR_DICT as c_int as c_uint
            {
                if !quiet {
                    semsg(
                        gettext(
                            (e_dot_can_only_be_used_on_dictionary_str.ptr() as *const _)
                                as *const c_char,
                        ),
                        name,
                    );
                }
                return ::core::ptr::null_mut::<c_char>();
            }
            if (*(*lp).ll_tv).v_type as c_uint != VAR_LIST as c_int as c_uint
                && (*(*lp).ll_tv).v_type as c_uint != VAR_DICT as c_int as c_uint
                && (*(*lp).ll_tv).v_type as c_uint != VAR_BLOB as c_int as c_uint
            {
                if !quiet {
                    emsg(gettext(
                        b"E689: Can only index a List, Dictionary or Blob\0".as_ptr()
                            as *const c_char,
                    ));
                }
                return ::core::ptr::null_mut::<c_char>();
            }
            if (*(*lp).ll_tv).v_type as c_uint == VAR_LIST as c_int as c_uint
                && (*(*lp).ll_tv).vval.v_list.is_null()
            {
                tv_list_alloc_ret((*lp).ll_tv, kListLenUnknown as c_int as ptrdiff_t);
            } else if (*(*lp).ll_tv).v_type as c_uint == VAR_BLOB as c_int as c_uint
                && (*(*lp).ll_tv).vval.v_blob.is_null()
            {
                tv_blob_alloc_ret((*lp).ll_tv);
            }
            if (*lp).ll_range {
                if !quiet {
                    emsg(gettext(
                        b"E708: [:] must come last\0".as_ptr() as *const c_char
                    ));
                }
                break '_done;
            } else {
                let mut len: c_int = -1 as c_int;
                let mut key: *mut c_char = ::core::ptr::null_mut::<c_char>();
                if *p as c_int == '.' as c_int {
                    key = p.offset(1 as c_int as isize);
                    len = 0 as c_int;
                    while *key.offset(len as isize) as c_uint >= 'A' as c_uint
                        && *key.offset(len as isize) as c_uint <= 'Z' as c_uint
                        || *key.offset(len as isize) as c_uint >= 'a' as c_uint
                            && *key.offset(len as isize) as c_uint <= 'z' as c_uint
                        || ascii_isdigit(*key.offset(len as isize) as c_int) as c_int != 0
                        || *key.offset(len as isize) as c_int == '_' as c_int
                    {
                        len += 1;
                    }
                    if len == 0 as c_int {
                        if !quiet {
                            emsg(gettext(
                                b"E713: Cannot use empty key after .\0".as_ptr() as *const c_char
                            ));
                        }
                        return ::core::ptr::null_mut::<c_char>();
                    }
                    p = key.offset(len as isize);
                } else {
                    p = skipwhite(p.offset(1 as c_int as isize));
                    if *p as c_int == ':' as c_int {
                        empty1 = true_0 != 0;
                    } else {
                        empty1 = false_0 != 0;
                        if eval1(&raw mut p, &raw mut var1, EVALARG_EVALUATE.ptr()) == FAIL {
                            break '_done;
                        }
                        if !tv_check_str(&raw mut var1) {
                            break '_done;
                        }
                        p = skipwhite(p);
                    }
                    if *p as c_int == ':' as c_int {
                        if (*(*lp).ll_tv).v_type as c_uint == VAR_DICT as c_int as c_uint {
                            if !quiet {
                                emsg(gettext(
                                    (e_cannot_slice_dictionary.ptr() as *const _) as *const c_char,
                                ));
                            }
                            break '_done;
                        } else if !rettv.is_null()
                            && !((*rettv).v_type as c_uint == VAR_LIST as c_int as c_uint
                                && !(*rettv).vval.v_list.is_null())
                            && !((*rettv).v_type as c_uint == VAR_BLOB as c_int as c_uint
                                && !(*rettv).vval.v_blob.is_null())
                        {
                            if !quiet {
                                emsg(gettext(
                                    b"E709: [:] requires a List or Blob value\0".as_ptr()
                                        as *const c_char,
                                ));
                            }
                            break '_done;
                        } else {
                            p = skipwhite(p.offset(1 as c_int as isize));
                            if *p as c_int == ']' as c_int {
                                (*lp).ll_empty2 = true_0 != 0;
                            } else {
                                (*lp).ll_empty2 = false_0 != 0;
                                if eval1(&raw mut p, &raw mut var2, EVALARG_EVALUATE.ptr()) == FAIL
                                {
                                    break '_done;
                                }
                                if !tv_check_str(&raw mut var2) {
                                    break '_done;
                                }
                            }
                            (*lp).ll_range = true_0 != 0;
                        }
                    } else {
                        (*lp).ll_range = false_0 != 0;
                    }
                    if *p as c_int != ']' as c_int {
                        if !quiet {
                            emsg(gettext(e_missbrac.get()));
                        }
                        break '_done;
                    } else {
                        p = p.offset(1);
                    }
                }
                if (*(*lp).ll_tv).v_type as c_uint == VAR_DICT as c_int as c_uint {
                    let mut glv_status: glv_status_T = get_lval_dict_item(
                        lp,
                        name,
                        key,
                        len,
                        &raw mut p,
                        &raw mut var1,
                        flags,
                        unlet,
                        rettv,
                    );
                    if glv_status as c_uint == GLV_FAIL as c_int as c_uint {
                        break '_done;
                    }
                    if glv_status as c_uint == GLV_STOP as c_int as c_uint {
                        break;
                    }
                } else if (*(*lp).ll_tv).v_type as c_uint == VAR_BLOB as c_int as c_uint {
                    if get_lval_blob(lp, &raw mut var1, &raw mut var2, empty1, quiet) == FAIL {
                        break '_done;
                    } else {
                        break;
                    }
                } else if get_lval_list(lp, &raw mut var1, &raw mut var2, empty1, flags, quiet)
                    == FAIL
                {
                    break '_done;
                }
                tv_clear(&raw mut var1);
                tv_clear(&raw mut var2);
                var1.v_type = VAR_UNKNOWN;
                var2.v_type = VAR_UNKNOWN;
            }
        }
        rc = OK;
    }
    tv_clear(&raw mut var1);
    tv_clear(&raw mut var2);
    return if rc == OK {
        p
    } else {
        ::core::ptr::null_mut::<c_char>()
    };
}

pub unsafe extern "C" fn get_lval(
    name: *mut c_char,
    rettv: *mut typval_T,
    lp: *mut lval_T,
    unlet: bool,
    skip: bool,
    flags: c_int,
    fne_flags: c_int,
) -> *mut c_char {
    let mut quiet: c_int = flags & GLV_QUIET as c_int;
    memset(
        lp as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<lval_T>(),
    );
    if skip {
        (*lp).ll_name = name;
        return find_name_end(
            name,
            ::core::ptr::null_mut::<*const c_char>(),
            ::core::ptr::null_mut::<*const c_char>(),
            FNE_INCL_BR | fne_flags,
        ) as *mut c_char;
    }
    let mut expr_start: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut expr_end: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut p: *mut c_char = find_name_end(
        name,
        &raw mut expr_start as *mut *const c_char,
        &raw mut expr_end as *mut *const c_char,
        fne_flags,
    ) as *mut c_char;
    if !expr_start.is_null() {
        if unlet as c_int != 0
            && !ascii_iswhite(*p as c_int)
            && ends_excmd(*p as c_int) == 0
            && *p as c_int != '[' as c_int
            && *p as c_int != '.' as c_int
        {
            semsg(gettext(&raw const e_trailing_arg as *const c_char), p);
            return ::core::ptr::null_mut::<c_char>();
        }
        (*lp).ll_exp_name = make_expanded_name(name, expr_start, expr_end, p);
        (*lp).ll_name = (*lp).ll_exp_name;
        if (*lp).ll_exp_name.is_null() {
            if !aborting() && quiet == 0 {
                emsg_severe.set(true_0 != 0);
                semsg(gettext(&raw const e_invarg2 as *const c_char), name);
                return ::core::ptr::null_mut::<c_char>();
            }
            (*lp).ll_name_len = 0 as size_t;
        } else {
            (*lp).ll_name_len = strlen((*lp).ll_name);
        }
    } else {
        (*lp).ll_name = name;
        (*lp).ll_name_len = p.offset_from((*lp).ll_name) as size_t;
    }
    if *p as c_int != '[' as c_int && *p as c_int != '.' as c_int || (*lp).ll_name.is_null() {
        return p;
    }
    let mut ht: *mut hashtab_T = ::core::ptr::null_mut::<hashtab_T>();
    let mut v: *mut dictitem_T = find_var(
        (*lp).ll_name,
        (*lp).ll_name_len,
        if flags & GLV_READ_ONLY as c_int != 0 {
            ::core::ptr::null_mut::<*mut hashtab_T>()
        } else {
            &raw mut ht
        },
        flags & GLV_NO_AUTOLOAD as c_int,
    );
    if v.is_null() && quiet == 0 {
        semsg(
            gettext(b"E121: Undefined variable: %.*s\0".as_ptr() as *const c_char),
            (*lp).ll_name_len as c_int,
            (*lp).ll_name,
        );
    }
    if v.is_null() {
        return ::core::ptr::null_mut::<c_char>();
    }
    (*lp).ll_tv = &raw mut (*v).di_tv;
    if tv_is_luafunc((*lp).ll_tv) {
        return p;
    }
    p = get_lval_subscript(lp, p, name, rettv, ht, v, unlet, flags);
    if p.is_null() {
        return ::core::ptr::null_mut::<c_char>();
    }
    (*lp).ll_name_len = p.offset_from((*lp).ll_name) as size_t;
    return p;
}

pub unsafe extern "C" fn clear_lval(mut lp: *mut lval_T) {
    xfree((*lp).ll_exp_name as *mut c_void);
    xfree((*lp).ll_newkey as *mut c_void);
}

pub unsafe extern "C" fn set_var_lval(
    mut lp: *mut lval_T,
    mut endp: *mut c_char,
    mut rettv: *mut typval_T,
    mut copy: bool,
    is_const: bool,
    mut op: *const c_char,
) {
    let mut cc: c_int = 0;
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    if (*lp).ll_tv.is_null() {
        cc = *endp as uint8_t as c_int;
        *endp = NUL as c_char;
        if !(*lp).ll_blob.is_null() {
            if !op.is_null() && *op as c_int != '=' as c_int {
                semsg(gettext(&raw const e_letwrong as *const c_char), op);
                return;
            }
            if value_check_lock(
                (*(*lp).ll_blob).bv_lock,
                (*lp).ll_name,
                TV_CSTRING as size_t,
            ) {
                return;
            }
            if (*lp).ll_range as c_int != 0
                && (*rettv).v_type as c_uint == VAR_BLOB as c_int as c_uint
            {
                if (*lp).ll_empty2 {
                    (*lp).ll_n2 = tv_blob_len((*lp).ll_blob) - 1 as c_int;
                }
                if tv_blob_set_range(
                    (*lp).ll_blob,
                    (*lp).ll_n1 as varnumber_T,
                    (*lp).ll_n2 as varnumber_T,
                    rettv,
                ) == FAIL
                {
                    return;
                }
            } else {
                let mut error: bool = false_0 != 0;
                let val: varnumber_T = tv_get_number_chk(rettv, &raw mut error);
                if !error {
                    if val < 0 as varnumber_T || val > 255 as varnumber_T {
                        semsg(
                            gettext(&raw const e_invalid_value_for_blob_nr as *const c_char),
                            val,
                        );
                    } else {
                        tv_blob_set_append((*lp).ll_blob, (*lp).ll_n1, val as uint8_t);
                    }
                }
            }
        } else if !op.is_null() && *op as c_int != '=' as c_int {
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if is_const {
                emsg(gettext(&raw const e_cannot_mod as *const c_char));
                *endp = cc as c_char;
                return;
            }
            di = ::core::ptr::null_mut::<dictitem_T>();
            if eval_variable(
                (*lp).ll_name,
                (*lp).ll_name_len as c_int,
                &raw mut tv,
                &raw mut di,
                true_0 != 0,
                false_0 != 0,
            ) == OK
            {
                if (di.is_null()
                    || !var_check_ro((*di).di_flags as c_int, (*lp).ll_name, TV_CSTRING as size_t)
                        && !tv_check_lock(
                            &raw mut (*di).di_tv,
                            (*lp).ll_name,
                            TV_CSTRING as size_t,
                        ))
                    && eexe_mod_op(&raw mut tv, rettv, op) == OK
                {
                    set_var((*lp).ll_name, (*lp).ll_name_len, &raw mut tv, false_0 != 0);
                }
                tv_clear(&raw mut tv);
            }
        } else {
            set_var_const((*lp).ll_name, (*lp).ll_name_len, rettv, copy, is_const);
        }
        *endp = cc as c_char;
    } else if !value_check_lock(
        (if (*lp).ll_newkey.is_null() {
            (*(*lp).ll_tv).v_lock as c_uint
        } else {
            (*(*(*lp).ll_tv).vval.v_dict).dv_lock as c_uint
        }) as VarLockStatus,
        (*lp).ll_name,
        TV_CSTRING as size_t,
    ) {
        if (*lp).ll_range {
            if is_const {
                emsg(gettext(
                    b"E996: Cannot lock a range\0".as_ptr() as *const c_char
                ));
                return;
            }
            tv_list_assign_range(
                (*lp).ll_list,
                (*rettv).vval.v_list,
                (*lp).ll_n1,
                (*lp).ll_n2,
                (*lp).ll_empty2,
                op,
                (*lp).ll_name,
            );
        } else {
            let mut oldtv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            let mut dict: *mut dict_T = (*lp).ll_dict;
            let mut watched: bool = tv_dict_is_watched(dict);
            if is_const {
                emsg(gettext(
                    b"E996: Cannot lock a list or dict\0".as_ptr() as *const c_char
                ));
                return;
            }
            '_notify: {
                if !(*lp).ll_newkey.is_null() {
                    if !op.is_null() && *op as c_int != '=' as c_int {
                        semsg(
                            gettext(&raw const e_dictkey as *const c_char),
                            (*lp).ll_newkey,
                        );
                        return;
                    }
                    if tv_dict_wrong_func_name((*(*lp).ll_tv).vval.v_dict, rettv, (*lp).ll_newkey)
                        != 0
                    {
                        return;
                    }
                    di = tv_dict_item_alloc((*lp).ll_newkey);
                    if tv_dict_add((*(*lp).ll_tv).vval.v_dict, di) == FAIL {
                        xfree(di as *mut c_void);
                        return;
                    }
                    (*lp).ll_tv = &raw mut (*di).di_tv;
                } else {
                    if watched {
                        tv_copy((*lp).ll_tv, &raw mut oldtv);
                    }
                    if !op.is_null() && *op as c_int != '=' as c_int {
                        eexe_mod_op((*lp).ll_tv, rettv, op);
                        break '_notify;
                    } else {
                        tv_clear((*lp).ll_tv);
                    }
                }
                if copy {
                    tv_copy(rettv, (*lp).ll_tv);
                } else {
                    *(*lp).ll_tv = *rettv;
                    (*(*lp).ll_tv).v_lock = VAR_UNLOCKED;
                    tv_init(rettv);
                }
            }
            if watched {
                if oldtv.v_type as c_uint == VAR_UNKNOWN as c_int as c_uint {
                    '_c2rust_label: {
                        if !(*lp).ll_newkey.is_null() {
                        } else {
                            __assert_fail(
                                b"lp->ll_newkey != NULL\0".as_ptr()
                                    as *const c_char,
                                b"src/nvim/eval.rs\0"
                                    .as_ptr() as *const c_char,
                                1418 as c_uint,
                                b"void set_var_lval(lval_T *, char *, typval_T *, _Bool, const _Bool, const char *)\0"
                                    .as_ptr() as *const c_char,
                            );
                        }
                    };
                    tv_dict_watcher_notify(
                        dict,
                        (*lp).ll_newkey,
                        (*lp).ll_tv,
                        ::core::ptr::null_mut::<typval_T>(),
                    );
                } else {
                    let mut di_: *mut dictitem_T = (*lp).ll_di;
                    '_c2rust_label_0: {
                        if !(&raw mut (*di_).di_key as *mut c_char).is_null() {
                        } else {
                            __assert_fail(
                                b"di_->di_key != NULL\0".as_ptr()
                                    as *const c_char,
                                b"src/nvim/eval.rs\0"
                                    .as_ptr() as *const c_char,
                                1422 as c_uint,
                                b"void set_var_lval(lval_T *, char *, typval_T *, _Bool, const _Bool, const char *)\0"
                                    .as_ptr() as *const c_char,
                            );
                        }
                    };
                    tv_dict_watcher_notify(
                        dict,
                        &raw mut (*di_).di_key as *mut c_char,
                        (*lp).ll_tv,
                        &raw mut oldtv,
                    );
                    tv_clear(&raw mut oldtv);
                }
            }
        }
    }
}
