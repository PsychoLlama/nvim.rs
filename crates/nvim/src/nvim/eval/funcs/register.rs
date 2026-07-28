//! Registers: `getreg()`, `setreg()`, `getreginfo()` and the
//! recording state.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

unsafe extern "C" fn getreg_get_regname(mut argvars: *mut typval_T) -> ::core::ffi::c_int {
    let mut strregname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        strregname = tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        if strregname.is_null() {
            return 0 as ::core::ffi::c_int;
        }
    } else {
        strregname = get_vim_var_str(VV_REG);
    }
    return if *strregname as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        '"' as ::core::ffi::c_int
    } else {
        *strregname as uint8_t as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn f_getreg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut arg2: ::core::ffi::c_int = false_0;
    let mut return_list: bool = false_0 != 0;
    let mut regname: ::core::ffi::c_int = getreg_get_regname(argvars);
    if regname == 0 as ::core::ffi::c_int {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error: bool = false_0 != 0;
        arg2 = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if !error
            && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return_list = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) != 0;
        }
        if error {
            return;
        }
    }
    if return_list {
        (*rettv).v_type = VAR_LIST;
        (*rettv).vval.v_list = get_reg_contents(
            regname,
            (if arg2 != 0 {
                kGRegExprSrc as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | kGRegList as ::core::ffi::c_int,
        ) as *mut list_T;
        if (*rettv).vval.v_list.is_null() {
            (*rettv).vval.v_list = tv_list_alloc(0 as ptrdiff_t);
        }
        tv_list_ref((*rettv).vval.v_list);
    } else {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = get_reg_contents(
            regname,
            if arg2 != 0 {
                kGRegExprSrc as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        ) as *mut ::core::ffi::c_char;
    };
}
pub unsafe extern "C" fn f_getregtype(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut regname: ::core::ffi::c_int = getreg_get_regname(argvars);
    if regname == 0 as ::core::ffi::c_int {
        return;
    }
    let mut reglen: colnr_T = 0 as colnr_T;
    let mut buf: [::core::ffi::c_char; 67] = [0; 67];
    let mut reg_type: MotionType = get_reg_type(regname, &raw mut reglen);
    format_reg_type(
        reg_type,
        reglen,
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 67]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_char; 67]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
    );
    (*rettv).vval.v_string = xstrdup(&raw mut buf as *mut ::core::ffi::c_char);
}
pub unsafe extern "C" fn f_getreginfo(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut regname: ::core::ffi::c_int = getreg_get_regname(argvars);
    if regname == 0 as ::core::ffi::c_int {
        return;
    }
    if regname == '@' as ::core::ffi::c_int {
        regname = '"' as ::core::ffi::c_int;
    }
    tv_dict_alloc_ret(rettv);
    let dict: *mut dict_T = (*rettv).vval.v_dict;
    let list: *mut list_T = get_reg_contents(
        regname,
        kGRegExprSrc as ::core::ffi::c_int | kGRegList as ::core::ffi::c_int,
    ) as *mut list_T;
    if list.is_null() {
        return;
    }
    tv_dict_add_list(
        dict,
        b"regcontents\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
        list,
    );
    let mut buf: [::core::ffi::c_char; 67] = [0; 67];
    buf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    let mut reglen: colnr_T = 0 as colnr_T;
    match get_reg_type(regname, &raw mut reglen) as ::core::ffi::c_int {
        1 => {
            buf[0 as ::core::ffi::c_int as usize] = 'V' as ::core::ffi::c_char;
        }
        0 => {
            buf[0 as ::core::ffi::c_int as usize] = 'v' as ::core::ffi::c_char;
        }
        2 => {
            vim_snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 67]>(),
                b"%c%d\0".as_ptr() as *const ::core::ffi::c_char,
                Ctrl_V,
                reglen as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            );
        }
        -1 => {
            abort();
        }
        _ => {}
    }
    tv_dict_add_str(
        dict,
        b"regtype\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    buf[0 as ::core::ffi::c_int as usize] =
        get_register_name(get_unname_register()) as ::core::ffi::c_char;
    buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    if regname == '"' as ::core::ffi::c_int {
        tv_dict_add_str(
            dict,
            b"points_to\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
    } else {
        tv_dict_add_bool(
            dict,
            b"isunnamed\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            (if regname == buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int {
                kBoolVarTrue as ::core::ffi::c_int
            } else {
                kBoolVarFalse as ::core::ffi::c_int
            }) as BoolVarValue,
        );
    };
}
unsafe extern "C" fn return_register(mut regname: ::core::ffi::c_int, mut rettv: *mut typval_T) {
    let mut buf: [::core::ffi::c_char; 2] =
        [regname as ::core::ffi::c_char, 0 as ::core::ffi::c_char];
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xstrdup(&raw mut buf as *mut ::core::ffi::c_char);
}
pub unsafe extern "C" fn f_reg_executing(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    return_register(reg_executing.get(), rettv);
}
pub unsafe extern "C" fn f_reg_recording(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    return_register(reg_recording.get(), rettv);
}
pub unsafe extern "C" fn f_reg_recorded(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    return_register(reg_recorded.get(), rettv);
}
unsafe extern "C" fn get_yank_type(
    pp: *mut *mut ::core::ffi::c_char,
    yank_type: *mut MotionType,
    block_len: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut stropt: *mut ::core::ffi::c_char = *pp;
    match *stropt as ::core::ffi::c_int {
        118 | 99 => {
            *yank_type = kMTCharWise;
        }
        86 | 108 => {
            *yank_type = kMTLineWise;
        }
        98 | Ctrl_V => {
            *yank_type = kMTBlockWise;
            if ascii_isdigit(*stropt.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            {
                stropt = stropt.offset(1);
                *block_len = getdigits_int(&raw mut stropt, false_0 != 0, 0 as ::core::ffi::c_int)
                    - 1 as ::core::ffi::c_int;
                stropt = stropt.offset(-1);
            }
        }
        _ => return FAIL,
    }
    *pp = stropt;
    return OK;
}
pub unsafe extern "C" fn f_setreg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut append: bool = false_0 != 0;
    let mut block_len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut yank_type: MotionType = kMTUnknown;
    (*rettv).vval.v_number = 1 as varnumber_T;
    let strregname: *const ::core::ffi::c_char = tv_get_string_chk(argvars);
    if strregname.is_null() {
        return;
    }
    let mut regname: ::core::ffi::c_char = *strregname;
    if regname as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || regname as ::core::ffi::c_int == '@' as ::core::ffi::c_int
    {
        regname = '"' as ::core::ffi::c_char;
    }
    let mut regcontents: *const typval_T = ::core::ptr::null::<typval_T>();
    let mut pointreg: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let d: *mut dict_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if tv_dict_len(d) == 0 as ::core::ffi::c_long {
            let mut lstval: [*mut ::core::ffi::c_char; 2] = [
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            write_reg_contents_lst(
                regname as ::core::ffi::c_int,
                &raw mut lstval as *mut *mut ::core::ffi::c_char,
                false_0 != 0,
                kMTUnknown,
                -1 as colnr_T,
            );
            return;
        }
        let di: *mut dictitem_T = tv_dict_find(
            d,
            b"regcontents\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            regcontents = &raw mut (*di).di_tv;
        }
        let mut stropt: *const ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"regtype\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if !stropt.is_null() {
            let ret: ::core::ffi::c_int = get_yank_type(
                &raw mut stropt as *mut *mut ::core::ffi::c_char,
                &raw mut yank_type,
                &raw mut block_len,
            );
            if ret == FAIL || {
                stropt = stropt.offset(1);
                *stropt as ::core::ffi::c_int != NUL
            } {
                semsg(
                    gettext(&raw const e_invargval as *const ::core::ffi::c_char),
                    b"value\0".as_ptr() as *const ::core::ffi::c_char,
                );
                return;
            }
        }
        if regname as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
            stropt = tv_dict_get_string(
                d,
                b"points_to\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            if !stropt.is_null() {
                pointreg = *stropt;
                regname = pointreg;
            }
        } else if tv_dict_get_number(d, b"isunnamed\0".as_ptr() as *const ::core::ffi::c_char) != 0
        {
            pointreg = regname;
        }
    } else {
        regcontents = argvars.offset(1 as ::core::ffi::c_int as isize);
    }
    let mut set_unnamed: bool = false_0 != 0;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if yank_type as ::core::ffi::c_int != kMTUnknown as ::core::ffi::c_int {
            semsg(
                gettext(&raw const e_toomanyarg as *const ::core::ffi::c_char),
                b"setreg\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        let mut stropt_0: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(2 as ::core::ffi::c_int as isize));
        if stropt_0.is_null() {
            return;
        }
        while *stropt_0 as ::core::ffi::c_int != NUL {
            match *stropt_0 as ::core::ffi::c_int {
                97 | 65 => {
                    append = true_0 != 0;
                }
                117 | 34 => {
                    set_unnamed = true_0 != 0;
                }
                _ => {
                    get_yank_type(
                        &raw mut stropt_0 as *mut *mut ::core::ffi::c_char,
                        &raw mut yank_type,
                        &raw mut block_len,
                    );
                }
            }
            stropt_0 = stropt_0.offset(1);
        }
    }
    if !regcontents.is_null()
        && (*regcontents).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let ll: *mut list_T = (*regcontents).vval.v_list;
        let len: ::core::ffi::c_int = tv_list_len(ll);
        let mut lstval_0: *mut *mut ::core::ffi::c_char = xmalloc(
            ::core::mem::size_of::<*mut ::core::ffi::c_char>().wrapping_mul(
                (len as size_t)
                    .wrapping_add(1 as size_t)
                    .wrapping_mul(2 as size_t),
            ),
        )
            as *mut *mut ::core::ffi::c_char;
        let mut curval: *mut *const ::core::ffi::c_char =
            lstval_0 as *mut *const ::core::ffi::c_char;
        let mut allocval: *mut *mut ::core::ffi::c_char = lstval_0
            .offset(len as isize)
            .offset(2 as ::core::ffi::c_int as isize);
        let mut curallocval: *mut *mut ::core::ffi::c_char = allocval;
        let l_: *const list_T = ll;
        '_free_lstval: {
            's_313: {
                if !l_.is_null() {
                    let mut li: *const listitem_T = (*l_).lv_first;
                    loop {
                        if li.is_null() {
                            break 's_313;
                        }
                        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
                        *curval = tv_get_string_buf_chk(
                            &raw const (*li).li_tv,
                            &raw mut buf as *mut ::core::ffi::c_char,
                        );
                        if (*curval).is_null() {
                            break '_free_lstval;
                        }
                        if *curval
                            == &raw mut buf as *mut ::core::ffi::c_char
                                as *const ::core::ffi::c_char
                        {
                            *curallocval = xstrdup(*curval);
                            *curval = *curallocval;
                            curallocval = curallocval.offset(1);
                        }
                        curval = curval.offset(1);
                        li = (*li).li_next;
                    }
                }
            }
            let c2rust_fresh9 = curval;
            curval = curval.offset(1);
            let c2rust_lvalue_ptr = &raw mut *c2rust_fresh9;
            *c2rust_lvalue_ptr = ::core::ptr::null::<::core::ffi::c_char>();
            write_reg_contents_lst(
                regname as ::core::ffi::c_int,
                lstval_0,
                append,
                yank_type,
                block_len,
            );
        }
        while curallocval > allocval {
            curallocval = curallocval.offset(-1);
            xfree(*curallocval as *mut ::core::ffi::c_void);
        }
        xfree(lstval_0 as *mut ::core::ffi::c_void);
    } else if !regcontents.is_null() {
        let strval: *const ::core::ffi::c_char = tv_get_string_chk(regcontents);
        if strval.is_null() {
            return;
        }
        write_reg_contents_ex(
            regname as ::core::ffi::c_int,
            strval,
            strlen(strval) as ssize_t,
            append,
            yank_type,
            block_len,
        );
    }
    if pointreg as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        get_yank_register(
            pointreg as ::core::ffi::c_int,
            YREG_YANK as ::core::ffi::c_int,
        );
    }
    (*rettv).vval.v_number = 0 as varnumber_T;
    if set_unnamed {
        op_reg_set_previous(regname);
    }
}
