//! `:let` with no value: printing variables rather than setting them.
//!
//! `list_arg_vars` resolves each argument (including a bare scope name) and
//! `list_one_var_a` does the printing, padding the name to column 22 and
//! prefixing the value with `#`, `*`, `[` or `{` by type.  That layout is a
//! contract: it is what a user sees.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn list_vim_vars(mut first: *mut ::core::ffi::c_int) {
    unsafe {
        list_hashtable_vars(
            &raw mut (*vimvardict.ptr()).dv_hashtab,
            b"v:\0".as_ptr() as *const ::core::ffi::c_char,
            false_0,
            first,
        );
    }
}

pub(crate) unsafe extern "C" fn list_script_vars(mut first: *mut ::core::ffi::c_int) {
    unsafe {
        if (*current_sctx.ptr()).sc_sid > 0 as ::core::ffi::c_int
            && (*current_sctx.ptr()).sc_sid <= (*script_items.ptr()).ga_len
        {
            list_hashtable_vars(
                &raw mut (*(**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T).offset(
                    ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        as isize,
                ))
                .sn_vars)
                    .sv_dict
                    .dv_hashtab,
                b"s:\0".as_ptr() as *const ::core::ffi::c_char,
                false_0,
                first,
            );
        }
    }
}

pub unsafe extern "C" fn list_hashtable_vars(
    mut ht: *mut hashtab_T,
    mut prefix: *const ::core::ffi::c_char,
    mut empty: ::core::ffi::c_int,
    mut first: *mut ::core::ffi::c_int,
) {
    unsafe {
        let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
        let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        let mut todo: ::core::ffi::c_int = 0;
        todo = (*ht).ht_used as ::core::ffi::c_int;
        hi = (*ht).ht_array;
        while todo > 0 as ::core::ffi::c_int && !got_int.get() {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                todo -= 1;
                di = (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
                let mut buf: [::core::ffi::c_char; 1025] = [0; 1025];
                xstrlcpy(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    prefix,
                    IOSIZE as size_t,
                );
                xstrlcat(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                );
                if !message_filtered(&raw mut buf as *mut ::core::ffi::c_char) {
                    if empty != 0
                        || (*di).di_tv.v_type as ::core::ffi::c_uint
                            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                        || !(*di).di_tv.vval.v_string.is_null()
                    {
                        list_one_var(di, prefix, first);
                    }
                }
            }
            hi = hi.offset(1);
        }
    }
}

pub(crate) unsafe extern "C" fn list_glob_vars(mut first: *mut ::core::ffi::c_int) {
    unsafe {
        list_hashtable_vars(
            &raw mut (*globvardict.ptr()).dv_hashtab,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            true_0,
            first,
        );
    }
}

pub(crate) unsafe extern "C" fn list_buf_vars(mut first: *mut ::core::ffi::c_int) {
    unsafe {
        list_hashtable_vars(
            &raw mut (*(*curbuf.get()).b_vars).dv_hashtab,
            b"b:\0".as_ptr() as *const ::core::ffi::c_char,
            true_0,
            first,
        );
    }
}

pub(crate) unsafe extern "C" fn list_win_vars(mut first: *mut ::core::ffi::c_int) {
    unsafe {
        list_hashtable_vars(
            &raw mut (*(*curwin.get()).w_vars).dv_hashtab,
            b"w:\0".as_ptr() as *const ::core::ffi::c_char,
            true_0,
            first,
        );
    }
}

pub(crate) unsafe extern "C" fn list_tab_vars(mut first: *mut ::core::ffi::c_int) {
    unsafe {
        list_hashtable_vars(
            &raw mut (*(*curtab.get()).tp_vars).dv_hashtab,
            b"t:\0".as_ptr() as *const ::core::ffi::c_char,
            true_0,
            first,
        );
    }
}

pub(crate) unsafe extern "C" fn list_arg_vars(
    mut eap: *mut exarg_T,
    mut arg: *const ::core::ffi::c_char,
    mut first: *mut ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut error: bool = false_0 != 0;
        let mut len: ::core::ffi::c_int = 0;
        let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut name_start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        while ends_excmd(*arg as ::core::ffi::c_int) == 0 && !got_int.get() {
            if error as ::core::ffi::c_int != 0 || (*eap).skip != 0 {
                arg = find_name_end(
                    arg,
                    ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                    FNE_INCL_BR | FNE_CHECK_START,
                );
                if !ascii_iswhite(*arg as ::core::ffi::c_int)
                    && ends_excmd(*arg as ::core::ffi::c_int) == 0
                {
                    emsg_severe.set(true_0 != 0);
                    semsg(
                        gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                        arg,
                    );
                    break;
                }
            } else {
                name = arg;
                name_start = name;
                let mut tofree: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                len = get_name_len(&raw mut arg, &raw mut tofree, true_0 != 0, true_0 != 0);
                if len <= 0 as ::core::ffi::c_int {
                    if len < 0 as ::core::ffi::c_int && !aborting() {
                        emsg_severe.set(true_0 != 0);
                        semsg(
                            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                            arg,
                        );
                        break;
                    } else {
                        error = true_0 != 0;
                    }
                } else {
                    if !tofree.is_null() {
                        name = tofree;
                    }
                    if eval_variable(
                        name,
                        len,
                        &raw mut tv,
                        ::core::ptr::null_mut::<*mut dictitem_T>(),
                        true_0 != 0,
                        false_0 != 0,
                    ) == FAIL
                    {
                        error = true_0 != 0;
                    } else {
                        let arg_subsc: *const ::core::ffi::c_char = arg;
                        if handle_subscript(
                            &raw mut arg,
                            &raw mut tv,
                            EVALARG_EVALUATE.ptr(),
                            true_0 != 0,
                        ) == FAIL
                        {
                            error = true_0 != 0;
                        } else {
                            if arg == arg_subsc
                                && len == 2 as ::core::ffi::c_int
                                && *name.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ':' as ::core::ffi::c_int
                            {
                                match *name as ::core::ffi::c_int {
                                    103 => {
                                        list_glob_vars(first);
                                    }
                                    98 => {
                                        list_buf_vars(first);
                                    }
                                    119 => {
                                        list_win_vars(first);
                                    }
                                    116 => {
                                        list_tab_vars(first);
                                    }
                                    118 => {
                                        list_vim_vars(first);
                                    }
                                    115 => {
                                        list_script_vars(first);
                                    }
                                    108 => {
                                        list_func_vars(first);
                                    }
                                    _ => {
                                        semsg(
                                            gettext(
                                                b"E738: Can't list variables for %s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            name,
                                        );
                                    }
                                }
                            } else {
                                let s: *mut ::core::ffi::c_char =
                                    encode_tv2echo(&raw mut tv, ::core::ptr::null_mut::<size_t>());
                                let used_name: *const ::core::ffi::c_char =
                                    if arg == arg_subsc { name } else { name_start };
                                '_c2rust_label: {
                                    if !used_name.is_null() {
                                    } else {
                                        __assert_fail(
                                        b"used_name != NULL\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/eval/vars.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        1266 as ::core::ffi::c_uint,
                                        b"const char *list_arg_vars(exarg_T *, const char *, int *)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                    }
                                };
                                let name_size: ptrdiff_t =
                                    if used_name == tofree as *const ::core::ffi::c_char {
                                        strlen(used_name) as ptrdiff_t
                                    } else {
                                        arg.offset_from(used_name)
                                    };
                                list_one_var_a(
                                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                                    used_name,
                                    name_size,
                                    tv.v_type,
                                    if s.is_null() {
                                        b"\0".as_ptr() as *const ::core::ffi::c_char
                                    } else {
                                        s as *const ::core::ffi::c_char
                                    },
                                    first,
                                );
                                xfree(s as *mut ::core::ffi::c_void);
                            }
                            tv_clear(&raw mut tv);
                        }
                    }
                }
                xfree(tofree as *mut ::core::ffi::c_void);
            }
            arg = skipwhite(arg);
        }
        return arg;
    }
}

unsafe extern "C" fn list_one_var(
    mut v: *mut dictitem_T,
    mut prefix: *const ::core::ffi::c_char,
    mut first: *mut ::core::ffi::c_int,
) {
    unsafe {
        let s: *mut ::core::ffi::c_char =
            encode_tv2echo(&raw mut (*v).di_tv, ::core::ptr::null_mut::<size_t>());
        list_one_var_a(
            prefix,
            &raw mut (*v).di_key as *mut ::core::ffi::c_char,
            strlen(&raw mut (*v).di_key as *mut ::core::ffi::c_char) as ptrdiff_t,
            (*v).di_tv.v_type,
            if s.is_null() {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                s as *const ::core::ffi::c_char
            },
            first,
        );
        xfree(s as *mut ::core::ffi::c_void);
    }
}

unsafe extern "C" fn list_one_var_a(
    mut prefix: *const ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
    name_len: ptrdiff_t,
    type_0: VarType,
    mut string: *const ::core::ffi::c_char,
    mut first: *mut ::core::ffi::c_int,
) {
    unsafe {
        if *first != 0 {
            msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
            msg_start();
        } else {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        if *prefix as ::core::ffi::c_int != NUL {
            msg_puts(prefix);
        }
        if !name.is_null() {
            msg_puts_len(name, name_len, 0 as ::core::ffi::c_int, false_0 != 0);
        }
        msg_putchar(' ' as ::core::ffi::c_int);
        msg_advance(22 as ::core::ffi::c_int);
        if type_0 as ::core::ffi::c_uint == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            msg_putchar('#' as ::core::ffi::c_int);
        } else if type_0 as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            || type_0 as ::core::ffi::c_uint
                == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            msg_putchar('*' as ::core::ffi::c_int);
        } else if type_0 as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            msg_putchar('[' as ::core::ffi::c_int);
            if *string as ::core::ffi::c_int == '[' as ::core::ffi::c_int {
                string = string.offset(1);
            }
        } else if type_0 as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            msg_putchar('{' as ::core::ffi::c_int);
            if *string as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                string = string.offset(1);
            }
        } else {
            msg_putchar(' ' as ::core::ffi::c_int);
        }
        msg_outtrans(string, 0 as ::core::ffi::c_int, false_0 != 0);
        if type_0 as ::core::ffi::c_uint == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            || type_0 as ::core::ffi::c_uint
                == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            msg_puts(b"()\0".as_ptr() as *const ::core::ffi::c_char);
        }
        if *first != 0 {
            msg_clr_eos();
            *first = false_0;
        }
    }
}
