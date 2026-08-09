//! The execution stack -- what `<sfile>`, `<stack>` and every error message's
//! "line N of ..." prefix are read from.
//!
//! `exestack` is a stack of `estack_T` entries, one per nested thing being
//! executed: a sourced script, a user function, an autocommand.  `estack_push`
//! and `estack_pop` bracket each of them, and `estack_sfile` renders the stack
//! the three ways vimscript can ask for it -- `<sfile>` (the innermost name),
//! `<slnum>`-carrying `<stack>` (the whole chain, `..`-joined), and the
//! `ESTACK_SCRIPT` form that stops at the innermost *script*.  `stacktrace_*`
//! and `f_getstacktrace()` are the same data as a list of dicts.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn estack_init() {
    unsafe {
        ga_grow(exestack.ptr(), 10 as ::core::ffi::c_int);
        let mut entry: *mut estack_T =
            ((*exestack.ptr()).ga_data as *mut estack_T).offset((*exestack.ptr()).ga_len as isize);
        (*entry).es_type = ETYPE_TOP;
        (*entry).es_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*entry).es_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*entry).es_info.ufunc = ::core::ptr::null_mut::<ufunc_T>();
        (*exestack.ptr()).ga_len += 1;
    }
}

pub unsafe extern "C" fn estack_push(
    mut type_0: etype_T,
    mut name: *mut ::core::ffi::c_char,
    mut lnum: linenr_T,
) -> *mut estack_T {
    unsafe {
        ga_grow(exestack.ptr(), 1 as ::core::ffi::c_int);
        let mut entry: *mut estack_T =
            ((*exestack.ptr()).ga_data as *mut estack_T).offset((*exestack.ptr()).ga_len as isize);
        (*entry).es_type = type_0;
        (*entry).es_name = name;
        (*entry).es_lnum = lnum;
        (*entry).es_info.ufunc = ::core::ptr::null_mut::<ufunc_T>();
        (*exestack.ptr()).ga_len += 1;
        return entry;
    }
}

pub unsafe extern "C" fn estack_push_ufunc(mut ufunc: *mut ufunc_T, mut lnum: linenr_T) {
    unsafe {
        let mut entry: *mut estack_T = estack_push(
            ETYPE_UFUNC,
            if !(*ufunc).uf_name_exp.is_null() {
                (*ufunc).uf_name_exp
            } else {
                &raw mut (*ufunc).uf_name as *mut ::core::ffi::c_char
            },
            lnum,
        );
        if !entry.is_null() {
            (*entry).es_info.ufunc = ufunc;
        }
    }
}

pub unsafe extern "C" fn estack_pop() {
    unsafe {
        if (*exestack.ptr()).ga_len > 1 as ::core::ffi::c_int {
            (*exestack.ptr()).ga_len -= 1;
        }
    }
}

pub unsafe extern "C" fn estack_sfile(mut which: estack_arg_T) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut entry: *const estack_T = ((*exestack.ptr()).ga_data as *mut estack_T)
            .offset((*exestack.ptr()).ga_len as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        if which as ::core::ffi::c_uint == ESTACK_SFILE as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*entry).es_type as ::core::ffi::c_uint
                != ETYPE_UFUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return if !(*entry).es_name.is_null() {
                xstrdup((*entry).es_name)
            } else {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            };
        }
        if which as ::core::ffi::c_uint
            == ESTACK_SCRIPT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut idx: ::core::ffi::c_int = (*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int;
            while idx >= 0 as ::core::ffi::c_int {
                if (*entry).es_type as ::core::ffi::c_uint
                    == ETYPE_UFUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*entry).es_type as ::core::ffi::c_uint
                        == ETYPE_AUCMD as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let def_ctx: *const sctx_T = if (*entry).es_type as ::core::ffi::c_uint
                        == ETYPE_UFUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        &raw mut (*(*entry).es_info.ufunc).uf_script_ctx
                    } else {
                        &raw mut (*(*entry).es_info.aucmd).script_ctx
                    };
                    return if (*def_ctx).sc_sid > 0 as ::core::ffi::c_int {
                        xstrdup(
                            (**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T).offset(
                                ((*def_ctx).sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                    as isize,
                            ))
                            .sn_name,
                        )
                    } else {
                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                    };
                } else if (*entry).es_type as ::core::ffi::c_uint
                    == ETYPE_SCRIPT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    return xstrdup((*entry).es_name);
                }
                idx -= 1;
                entry = entry.offset(-1);
            }
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
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
            100 as ::core::ffi::c_int,
        );
        let mut last_type: etype_T = ETYPE_SCRIPT;
        let mut idx_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx_0 < (*exestack.ptr()).ga_len {
            entry = ((*exestack.ptr()).ga_data as *mut estack_T).offset(idx_0 as isize);
            if !(*entry).es_name.is_null() {
                let mut type_name: String_0 = String_0 {
                    data: c"".as_ptr() as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 1]>()
                        .wrapping_sub(1 as size_t),
                };
                let mut es_name: String_0 = cstr_as_string((*entry).es_name);
                if (*entry).es_type as ::core::ffi::c_uint != last_type as ::core::ffi::c_uint {
                    match (*entry).es_type as ::core::ffi::c_uint {
                        1 => {
                            type_name = String_0 {
                                data: c"script ".as_ptr() as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                                    .wrapping_sub(1 as size_t),
                            };
                        }
                        2 => {
                            type_name = String_0 {
                                data: c"function ".as_ptr() as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                                    .wrapping_sub(1 as size_t),
                            };
                        }
                        _ => {}
                    }
                    last_type = (*entry).es_type;
                }
                let mut lnum: linenr_T = if idx_0
                    == (*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int
                {
                    if which as ::core::ffi::c_uint
                        == ESTACK_STACK as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_lnum
                    } else {
                        0 as linenr_T
                    }
                } else {
                    (*entry).es_lnum
                };
                let mut len: size_t = es_name
                    .size
                    .wrapping_add(type_name.size)
                    .wrapping_add(26 as size_t);
                ga_grow(&raw mut ga, len as ::core::ffi::c_int);
                ga_concat_len(&raw mut ga, type_name.data, type_name.size);
                ga_concat_len(&raw mut ga, es_name.data, es_name.size);
                if lnum != 0 as linenr_T {
                    ga.ga_len += vim_snprintf_safelen(
                        (ga.ga_data as *mut ::core::ffi::c_char).offset(ga.ga_len as isize),
                        (ga.ga_maxlen - ga.ga_len) as size_t,
                        c"[%d]".as_ptr(),
                        lnum,
                    ) as ::core::ffi::c_int;
                }
                if idx_0 != (*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int {
                    ga_concat_len(
                        &raw mut ga,
                        c"..".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                }
            }
            idx_0 += 1;
        }
        if !ga.ga_data.is_null() {
            ga_append(&raw mut ga, NUL as uint8_t);
        }
        return ga.ga_data as *mut ::core::ffi::c_char;
    }
}

unsafe extern "C" fn stacktrace_push_item(
    l: *mut list_T,
    fp: *mut ufunc_T,
    event: *const ::core::ffi::c_char,
    lnum: linenr_T,
    filepath: *mut ::core::ffi::c_char,
) {
    unsafe {
        let d: *mut dict_T = tv_dict_alloc_lock(VAR_FIXED);
        let mut tv: typval_T = typval_T {
            v_type: VAR_DICT,
            v_lock: VAR_LOCKED,
            vval: typval_vval_union { v_dict: d },
        };
        if !fp.is_null() {
            tv_dict_add_func(
                d,
                c"funcref".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                fp,
            );
        }
        if !event.is_null() {
            tv_dict_add_str(
                d,
                c"event".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                event,
            );
        }
        tv_dict_add_nr(
            d,
            c"lnum".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            lnum as varnumber_T,
        );
        tv_dict_add_str(
            d,
            c"filepath".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            filepath,
        );
        tv_list_append_tv(l, &raw mut tv);
    }
}

pub unsafe extern "C" fn stacktrace_create() -> *mut list_T {
    unsafe {
        let l: *mut list_T = tv_list_alloc((*exestack.ptr()).ga_len as ptrdiff_t);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*exestack.ptr()).ga_len {
            let entry: *mut estack_T =
                ((*exestack.ptr()).ga_data as *mut estack_T).offset(i as isize);
            let mut lnum: linenr_T = (*entry).es_lnum;
            if (*entry).es_type as ::core::ffi::c_uint
                == ETYPE_SCRIPT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                stacktrace_push_item(
                    l,
                    ::core::ptr::null_mut::<ufunc_T>(),
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    lnum,
                    (*entry).es_name,
                );
            } else if (*entry).es_type as ::core::ffi::c_uint
                == ETYPE_UFUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let fp: *mut ufunc_T = (*entry).es_info.ufunc;
                let sctx: sctx_T = (*fp).uf_script_ctx;
                let mut filepath: *mut ::core::ffi::c_char =
                    (if sctx.sc_sid > 0 as ::core::ffi::c_int {
                        get_scriptname(sctx, ::core::ptr::null_mut::<bool>())
                            as *const ::core::ffi::c_char
                    } else {
                        c"".as_ptr()
                    }) as *mut ::core::ffi::c_char;
                lnum += sctx.sc_lnum;
                stacktrace_push_item(
                    l,
                    fp,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    lnum,
                    filepath,
                );
            } else if (*entry).es_type as ::core::ffi::c_uint
                == ETYPE_AUCMD as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let sctx_0: sctx_T = (*(*entry).es_info.aucmd).script_ctx;
                let mut filepath_0: *mut ::core::ffi::c_char =
                    (if sctx_0.sc_sid > 0 as ::core::ffi::c_int {
                        get_scriptname(sctx_0, ::core::ptr::null_mut::<bool>())
                            as *const ::core::ffi::c_char
                    } else {
                        c"".as_ptr()
                    }) as *mut ::core::ffi::c_char;
                lnum += sctx_0.sc_lnum;
                stacktrace_push_item(
                    l,
                    ::core::ptr::null_mut::<ufunc_T>(),
                    (*entry).es_name,
                    lnum,
                    filepath_0,
                );
            }
            i += 1;
        }
        return l;
    }
}

pub unsafe extern "C" fn f_getstacktrace(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_set_ret(rettv, stacktrace_create());
    }
}
