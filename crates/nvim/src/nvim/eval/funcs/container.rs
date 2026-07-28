//! Reading a List, Dict or Blob: `get()`, `empty()`, `index()`,
//! `flatten()` and friends.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_copy(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    var_item_copy(
        ::core::ptr::null::<vimconv_T>(),
        argvars.offset(0 as ::core::ffi::c_int as isize),
        rettv,
        false_0 != 0,
        0 as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn f_deepcopy(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_opt_bool_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut noref: varnumber_T = 0 as varnumber_T;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        noref = tv_get_bool_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        );
    }
    var_item_copy(
        ::core::ptr::null::<vimconv_T>(),
        argvars.offset(0 as ::core::ffi::c_int as isize),
        rettv,
        true_0 != 0,
        if noref == 0 as varnumber_T {
            get_copyID()
        } else {
            0 as ::core::ffi::c_int
        },
    );
}
pub unsafe extern "C" fn f_empty(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut n: bool = true_0 != 0;
    match (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint {
        2 | 3 => {
            n = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_string
                .is_null()
                || *(*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_string as ::core::ffi::c_int
                    == NUL;
        }
        9 => {
            n = false_0 != 0;
        }
        1 => {
            n = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_number
                == 0 as varnumber_T;
        }
        6 => {
            n = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_float
                == 0.0f64;
        }
        4 => {
            n = tv_list_len(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list,
            ) == 0 as ::core::ffi::c_int;
        }
        5 => {
            n = tv_dict_len(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_dict,
            ) == 0 as ::core::ffi::c_long;
        }
        7 => {
            match (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_bool as ::core::ffi::c_uint
            {
                1 => {
                    n = false_0 != 0;
                }
                0 => {
                    n = true_0 != 0;
                }
                _ => {}
            }
        }
        8 => {
            n = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_special as ::core::ffi::c_uint
                == kSpecialVarNull as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        10 => {
            n = tv_blob_len(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_blob,
            ) == 0 as ::core::ffi::c_int;
        }
        0 => {
            internal_error(b"f_empty(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char);
        }
        _ => {}
    }
    (*rettv).vval.v_number = n as varnumber_T;
}
unsafe extern "C" fn flatten_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut make_copy: bool,
) {
    let mut error: bool = false_0 != 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_listarg as *const ::core::ffi::c_char),
            b"flatten()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut maxdepth: ::core::ffi::c_int = 0;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        maxdepth = 999999 as ::core::ffi::c_int;
    } else {
        maxdepth = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if error {
            return;
        }
        if maxdepth < 0 as ::core::ffi::c_int {
            emsg(gettext(
                b"E900: maxdepth must be non-negative number\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return;
        }
    }
    let mut list: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    (*rettv).v_type = VAR_LIST;
    (*rettv).vval.v_list = list;
    if list.is_null() {
        return;
    }
    if make_copy {
        list = tv_list_copy(
            ::core::ptr::null::<vimconv_T>(),
            list,
            false_0 != 0,
            get_copyID(),
        );
        (*rettv).vval.v_list = list;
        if list.is_null() {
            return;
        }
    } else {
        if value_check_lock(
            tv_list_locked(list),
            b"flatten() argument\0".as_ptr() as *const ::core::ffi::c_char,
            TV_TRANSLATE as size_t,
        ) {
            return;
        }
        tv_list_ref(list);
    }
    tv_list_flatten(
        list,
        ::core::ptr::null_mut::<listitem_T>(),
        tv_list_len(list) as int64_t,
        maxdepth as int64_t,
    );
}
pub unsafe extern "C" fn f_flatten(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    flatten_common(argvars, rettv, false_0 != 0);
}
pub unsafe extern "C" fn f_flattennew(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    flatten_common(argvars, rettv, true_0 != 0);
}
pub unsafe extern "C" fn f_get(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut tv: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
    let mut what_is_dict: bool = false_0 != 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error: bool = false_0 != 0;
        let mut idx: ::core::ffi::c_int = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if !error {
            (*rettv).v_type = VAR_NUMBER;
            if idx < 0 as ::core::ffi::c_int {
                idx = tv_blob_len(
                    (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_blob,
                ) + idx;
            }
            if idx < 0 as ::core::ffi::c_int
                || idx
                    >= tv_blob_len(
                        (*argvars.offset(0 as ::core::ffi::c_int as isize))
                            .vval
                            .v_blob,
                    )
            {
                (*rettv).vval.v_number = -1 as varnumber_T;
            } else {
                (*rettv).vval.v_number = tv_blob_get(
                    (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_blob,
                    idx,
                ) as varnumber_T;
                tv = rettv;
            }
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if !l.is_null() {
            let mut error_0: bool = false_0 != 0;
            let mut li: *mut listitem_T = tv_list_find(
                l,
                tv_get_number_chk(
                    argvars.offset(1 as ::core::ffi::c_int as isize),
                    &raw mut error_0,
                ) as ::core::ffi::c_int,
            );
            if !error_0 && !li.is_null() {
                tv = &raw mut (*li).li_tv;
            }
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut d: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if !d.is_null() {
            let mut di: *mut dictitem_T = tv_dict_find(
                d,
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                tv = &raw mut (*di).di_tv;
            }
        }
    } else if tv_is_func(*argvars.offset(0 as ::core::ffi::c_int as isize)) {
        let mut pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
        let mut fref_pt: partial_T = partial_T {
            pt_refcount: 0,
            pt_copyID: 0,
            pt_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            pt_func: ::core::ptr::null_mut::<ufunc_T>(),
            pt_auto: false,
            pt_argc: 0,
            pt_argv: ::core::ptr::null_mut::<typval_T>(),
            pt_dict: ::core::ptr::null_mut::<dict_T>(),
        };
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            pt = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_partial;
        } else {
            memset(
                &raw mut fref_pt as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<partial_T>(),
            );
            fref_pt.pt_name = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_string;
            pt = &raw mut fref_pt;
        }
        if !pt.is_null() {
            let what: *const ::core::ffi::c_char =
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
            if strcmp(what, b"func\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
                || strcmp(what, b"name\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
            {
                let mut name: *const ::core::ffi::c_char = partial_name(pt);
                (*rettv).v_type = (if *what as ::core::ffi::c_int == 'f' as ::core::ffi::c_int {
                    VAR_FUNC as ::core::ffi::c_int
                } else {
                    VAR_STRING as ::core::ffi::c_int
                }) as VarType;
                '_c2rust_label: {
                    if !name.is_null() {
                    } else {
                        __assert_fail(
                            b"name != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            1889 as ::core::ffi::c_uint,
                            b"void f_get(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                if (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    func_ref(name as *mut ::core::ffi::c_char);
                }
                if *what as ::core::ffi::c_int == 'n' as ::core::ffi::c_int
                    && (*pt).pt_name.is_null()
                    && !(*pt).pt_func.is_null()
                {
                    name = printable_func_name((*pt).pt_func);
                }
                (*rettv).vval.v_string = xstrdup(name);
            } else if strcmp(what, b"dict\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                what_is_dict = true_0 != 0;
                if !(*pt).pt_dict.is_null() {
                    tv_dict_set_ret(rettv, (*pt).pt_dict);
                }
            } else if strcmp(what, b"args\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                (*rettv).v_type = VAR_LIST;
                tv_list_alloc_ret(rettv, (*pt).pt_argc as ptrdiff_t);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*pt).pt_argc {
                    tv_list_append_tv((*rettv).vval.v_list, (*pt).pt_argv.offset(i as isize));
                    i += 1;
                }
            } else if strcmp(what, b"arity\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                let mut required: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut optional: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut varargs: bool = false_0 != 0;
                let mut name_0: *const ::core::ffi::c_char = partial_name(pt);
                get_func_arity(
                    name_0,
                    &raw mut required,
                    &raw mut optional,
                    &raw mut varargs,
                );
                (*rettv).v_type = VAR_DICT;
                tv_dict_alloc_ret(rettv);
                let mut dict: *mut dict_T = (*rettv).vval.v_dict;
                if (*pt).pt_argc >= required + optional {
                    optional = 0 as ::core::ffi::c_int;
                    required = optional;
                } else if (*pt).pt_argc > required {
                    optional -= (*pt).pt_argc - required;
                    required = 0 as ::core::ffi::c_int;
                } else {
                    required -= (*pt).pt_argc;
                }
                tv_dict_add_nr(
                    dict,
                    b"required\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                    required as varnumber_T,
                );
                tv_dict_add_nr(
                    dict,
                    b"optional\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                    optional as varnumber_T,
                );
                tv_dict_add_bool(
                    dict,
                    b"varargs\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    varargs as BoolVarValue,
                );
            } else {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    what,
                );
            }
            if !what_is_dict {
                return;
            }
        }
    } else {
        semsg(
            gettext(&raw const e_listdictblobarg as *const ::core::ffi::c_char),
            b"get()\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if tv.is_null() {
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_copy(argvars.offset(2 as ::core::ffi::c_int as isize), rettv);
        }
    } else {
        tv_copy(tv, rettv);
    };
}
pub unsafe extern "C" fn f_index(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ic: bool = false_0 != 0;
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error: bool = false_0 != 0;
        let mut start: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            start = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if error {
                return;
            }
        }
        let b: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob;
        if b.is_null() {
            return;
        }
        if start < 0 as ::core::ffi::c_int {
            start = tv_blob_len(b) + start;
            if start < 0 as ::core::ffi::c_int {
                start = 0 as ::core::ffi::c_int;
            }
        }
        idx = start;
        while idx < tv_blob_len(b) {
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            tv.v_type = VAR_NUMBER;
            tv.vval.v_number = tv_blob_get(b, idx) as varnumber_T;
            if tv_equal(
                &raw mut tv,
                argvars.offset(1 as ::core::ffi::c_int as isize),
                ic,
            ) {
                (*rettv).vval.v_number = idx as varnumber_T;
                return;
            }
            idx += 1;
        }
        return;
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(
            &raw const e_listblobreq as *const ::core::ffi::c_char,
        ));
        return;
    }
    let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if l.is_null() {
        return;
    }
    let mut item: *mut listitem_T = tv_list_first(l);
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error_0: bool = false_0 != 0;
        idx = tv_list_uidx(
            l,
            tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error_0,
            ) as ::core::ffi::c_int,
        );
        if error_0 as ::core::ffi::c_int != 0 || idx == -1 as ::core::ffi::c_int {
            item = ::core::ptr::null_mut::<listitem_T>();
        } else {
            item = tv_list_find(l, idx);
            '_c2rust_label: {
                if !item.is_null() {
                } else {
                    __assert_fail(
                        b"item != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2971 as ::core::ffi::c_uint,
                        b"void f_index(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
        }
        if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ic = tv_get_number_chk(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut error_0,
            ) != 0;
            if error_0 {
                item = ::core::ptr::null_mut::<listitem_T>();
            }
        }
    }
    while !item.is_null() {
        if tv_equal(
            &raw mut (*item).li_tv,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            ic,
        ) {
            (*rettv).vval.v_number = idx as varnumber_T;
            break;
        } else {
            item = (*item).li_next;
            idx += 1;
        }
    }
}
unsafe extern "C" fn indexof_eval_expr(mut expr: *mut typval_T) -> varnumber_T {
    let mut argv: [typval_T; 3] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 3];
    argv[0 as ::core::ffi::c_int as usize] = *get_vim_var_tv(VV_KEY);
    argv[1 as ::core::ffi::c_int as usize] = *get_vim_var_tv(VV_VAL);
    let mut newtv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    newtv.v_type = VAR_UNKNOWN;
    if eval_expr_typval(
        expr,
        false_0 != 0,
        &raw mut argv as *mut typval_T,
        2 as ::core::ffi::c_int,
        &raw mut newtv,
    ) == FAIL
    {
        return false_0 as varnumber_T;
    }
    let mut error: bool = false_0 != 0;
    let mut found: varnumber_T = tv_get_bool_chk(&raw mut newtv, &raw mut error);
    tv_clear(&raw mut newtv);
    return if error as ::core::ffi::c_int != 0 {
        false_0 as varnumber_T
    } else {
        found
    };
}
unsafe extern "C" fn indexof_blob(
    mut b: *mut blob_T,
    mut startidx: varnumber_T,
    mut expr: *mut typval_T,
) -> varnumber_T {
    if b.is_null() {
        return -1 as varnumber_T;
    }
    if startidx < 0 as varnumber_T {
        startidx = tv_blob_len(b) as varnumber_T + startidx;
        if startidx < 0 as varnumber_T {
            startidx = 0 as varnumber_T;
        }
    }
    set_vim_var_type(VV_KEY, VAR_NUMBER);
    set_vim_var_type(VV_VAL, VAR_NUMBER);
    let called_emsg_start: ::core::ffi::c_int = called_emsg.get();
    let mut idx: varnumber_T = startidx;
    while idx < tv_blob_len(b) as varnumber_T {
        set_vim_var_nr(VV_KEY, idx);
        set_vim_var_nr(
            VV_VAL,
            tv_blob_get(b, idx as ::core::ffi::c_int) as varnumber_T,
        );
        if indexof_eval_expr(expr) != 0 {
            return idx;
        }
        if called_emsg.get() != called_emsg_start {
            return -1 as varnumber_T;
        }
        idx += 1;
    }
    return -1 as varnumber_T;
}
unsafe extern "C" fn indexof_list(
    mut l: *mut list_T,
    mut startidx: varnumber_T,
    mut expr: *mut typval_T,
) -> varnumber_T {
    if l.is_null() {
        return -1 as varnumber_T;
    }
    let mut item: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
    let mut idx: varnumber_T = 0 as varnumber_T;
    if startidx == 0 as varnumber_T {
        item = tv_list_first(l);
    } else {
        idx = tv_list_uidx(l, startidx as ::core::ffi::c_int) as varnumber_T;
        if idx == -1 as varnumber_T {
            item = ::core::ptr::null_mut::<listitem_T>();
        } else {
            item = tv_list_find(l, idx as ::core::ffi::c_int);
            '_c2rust_label: {
                if !item.is_null() {
                } else {
                    __assert_fail(
                        b"item != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3068 as ::core::ffi::c_uint,
                        b"varnumber_T indexof_list(list_T *, varnumber_T, typval_T *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
        }
    }
    set_vim_var_type(VV_KEY, VAR_NUMBER);
    let called_emsg_start: ::core::ffi::c_int = called_emsg.get();
    while !item.is_null() {
        set_vim_var_nr(VV_KEY, idx);
        tv_copy(&raw mut (*item).li_tv, get_vim_var_tv(VV_VAL));
        let mut found: bool = indexof_eval_expr(expr) != 0;
        tv_clear(get_vim_var_tv(VV_VAL));
        if found {
            return idx;
        }
        if called_emsg.get() != called_emsg_start {
            return -1 as varnumber_T;
        }
        item = (*item).li_next;
        idx += 1;
    }
    return -1 as varnumber_T;
}
pub unsafe extern "C" fn f_indexof(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    if tv_check_for_list_or_blob_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_string_or_func_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && ((*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_string
            .is_null()
            || *(*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_string as ::core::ffi::c_int
                == NUL)
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_partial
                .is_null()
    {
        return;
    }
    let mut startidx: varnumber_T = 0 as varnumber_T;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        startidx = tv_dict_get_number_def(
            (*argvars.offset(2 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"startidx\0".as_ptr() as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    }
    let mut save_val: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut save_key: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    prepare_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
    prepare_vimvar(VV_KEY as ::core::ffi::c_int, &raw mut save_key);
    let save_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(false_0);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*rettv).vval.v_number = indexof_blob(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_blob,
            startidx,
            argvars.offset(1 as ::core::ffi::c_int as isize),
        );
    } else {
        (*rettv).vval.v_number = indexof_list(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list,
            startidx,
            argvars.offset(1 as ::core::ffi::c_int as isize),
        );
    }
    restore_vimvar(VV_KEY as ::core::ffi::c_int, &raw mut save_key);
    restore_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
    (*did_emsg.ptr()) |= save_did_emsg;
}
pub unsafe extern "C" fn f_len(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    match (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint {
        2 | 1 => {
            (*rettv).vval.v_number = strlen(tv_get_string(
                argvars.offset(0 as ::core::ffi::c_int as isize),
            )) as varnumber_T;
        }
        10 => {
            (*rettv).vval.v_number = tv_blob_len(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_blob,
            ) as varnumber_T;
        }
        4 => {
            (*rettv).vval.v_number = tv_list_len(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list,
            ) as varnumber_T;
        }
        5 => {
            (*rettv).vval.v_number = tv_dict_len(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_dict,
            ) as varnumber_T;
        }
        0 | 7 | 8 | 6 | 9 | 3 => {
            emsg(gettext(
                b"E701: Invalid type for len()\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        _ => {}
    };
}
pub unsafe extern "C" fn f_type(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut n: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint {
        1 => {
            n = VAR_TYPE_NUMBER as ::core::ffi::c_int;
        }
        2 => {
            n = VAR_TYPE_STRING as ::core::ffi::c_int;
        }
        9 | 3 => {
            n = VAR_TYPE_FUNC as ::core::ffi::c_int;
        }
        4 => {
            n = VAR_TYPE_LIST as ::core::ffi::c_int;
        }
        5 => {
            n = VAR_TYPE_DICT as ::core::ffi::c_int;
        }
        6 => {
            n = VAR_TYPE_FLOAT as ::core::ffi::c_int;
        }
        7 => {
            n = VAR_TYPE_BOOL as ::core::ffi::c_int;
        }
        8 => {
            n = VAR_TYPE_SPECIAL as ::core::ffi::c_int;
        }
        10 => {
            n = VAR_TYPE_BLOB as ::core::ffi::c_int;
        }
        0 => {
            internal_error(b"f_type(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char);
        }
        _ => {}
    }
    (*rettv).vval.v_number = n as varnumber_T;
}
