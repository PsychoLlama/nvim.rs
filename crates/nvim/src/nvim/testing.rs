use crate::src::nvim::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::src::nvim::eval::typval::{
    tv_check_for_float_or_nr_arg, tv_check_for_opt_number_arg, tv_check_for_opt_string_arg,
    tv_check_for_opt_string_or_list_arg, tv_check_for_string_or_number_arg, tv_clear,
    tv_dict_add_tv, tv_dict_alloc, tv_dict_find, tv_equal, tv_get_float, tv_get_number_chk,
    tv_get_string, tv_get_string_buf_chk, tv_get_string_chk,
};
use crate::src::nvim::eval::typval::{tv_list_first, tv_list_last, tv_list_len};
use crate::src::nvim::eval::vars::{
    assert_error, get_vim_var_nr, get_vim_var_str, get_vim_var_tv, set_vim_var_string,
};
use crate::src::nvim::eval::{garbage_collect, pattern_match};
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::garray::{ga_append, ga_clear, ga_concat, ga_concat_len, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::main::{
    IObuff, Rows, called_emsg, called_vim_beep, did_emsg, e_cant_read_file_str,
    emsg_assert_fails_context, emsg_assert_fails_lnum, emsg_assert_fails_msg, emsg_on_display,
    emsg_silent, got_int, in_assert_fails, lines_left, msg_col, need_wait_return, no_wait_return,
    suppress_errthrow, trylevel,
};
use crate::src::nvim::mbyte::{mb_cptr2char_adv, utf_ptr2char};
use crate::src::nvim::memory::{xfree, xstrdup, xstrlcpy};
use crate::src::nvim::message::{emsg, msg_reset_scroll};
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::libc::{fclose, fgetc, gettext, memmove, strcmp, strlen, strstr};
use crate::src::nvim::runtime::{estack_sfile, exestack};
use crate::src::nvim::strings::{vim_snprintf, vim_snprintf_safelen};
use crate::src::nvim::types::{
    BoolVarValue, EvalFuncData, FILE, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_LIST, VAR_NUMBER,
    VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, VV_ERRMSG, VV_EXCEPTION, VV_TESTING, dict_T, dictitem_T,
    estack_T, estack_arg_T, float_T, garray_T, hashitem_T, int64_t, kBoolVarFalse, kBoolVarTrue,
    linenr_T, list_T, ptrdiff_t, size_t, typval_T, typval_vval_union, uint8_t, varnumber_T,
};
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const NUMBUFLEN: C2Rust_Unnamed_1 = 65;
pub const ESTACK_NONE: estack_arg_T = 0;
pub type assert_type_T = ::core::ffi::c_uint;
pub const ASSERT_OTHER: assert_type_T = 5;
pub const ASSERT_FAILS: assert_type_T = 4;
pub const ASSERT_NOTMATCH: assert_type_T = 3;
pub const ASSERT_MATCH: assert_type_T = 2;
pub const ASSERT_NOTEQUAL: assert_type_T = 1;
pub const ASSERT_EQUAL: assert_type_T = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BS: ::core::ffi::c_int = 8;
pub const TAB: ::core::ffi::c_int = 9;
pub const NL: ::core::ffi::c_int = 10;
pub const FF: ::core::ffi::c_int = 12;
pub const CAR: ::core::ffi::c_int = 13;
pub const ESC: ::core::ffi::c_int = 27;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
static e_assert_fails_second_arg: GlobalCell<[::core::ffi::c_char; 90]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<
        [u8; 90],
        [::core::ffi::c_char; 90],
    >(
        *b"E856: \"assert_fails()\" second argument must be a string or a list with one or two strings\0",
    )
});
static e_assert_fails_fourth_argument: GlobalCell<[::core::ffi::c_char; 57]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 57], [::core::ffi::c_char; 57]>(
            *b"E1115: \"assert_fails()\" fourth argument must be a number\0",
        )
    });
static e_assert_fails_fifth_argument: GlobalCell<[::core::ffi::c_char; 56]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
            *b"E1116: \"assert_fails()\" fifth argument must be a string\0",
        )
    });
static e_calling_test_garbagecollect_now_while_v_testing_is_not_set: GlobalCell<
    [::core::ffi::c_char; 68],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 68], [::core::ffi::c_char; 68]>(
        *b"E1142: Calling test_garbagecollect_now() while v:testing is not set\0",
    )
});
unsafe extern "C" fn prepare_assert_error(mut gap: *mut garray_T) {
    let mut sname: *mut ::core::ffi::c_char = estack_sfile(ESTACK_NONE);
    ga_init(gap, 1 as ::core::ffi::c_int, 100 as ::core::ffi::c_int);
    if !sname.is_null() {
        ga_concat(gap, sname);
        if (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum
            > 0 as linenr_T
        {
            ga_concat(gap, c" ".as_ptr());
        }
    }
    if (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum
        > 0 as linenr_T
    {
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let mut buflen: size_t = vim_snprintf_safelen(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as size_t,
                ),
            c"line %ld".as_ptr(),
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum as int64_t,
        );
        ga_concat_len(gap, &raw mut buf as *mut ::core::ffi::c_char, buflen);
    }
    if !sname.is_null()
        || (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum
            > 0 as linenr_T
    {
        ga_concat_len(
            gap,
            c": ".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
        );
    }
    xfree(sname as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn ga_concat_esc(
    mut gap: *mut garray_T,
    mut p: *const ::core::ffi::c_char,
    mut clen: ::core::ffi::c_int,
) {
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    if clen > 1 as ::core::ffi::c_int {
        memmove(
            &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            clen as size_t,
        );
        buf[clen as usize] = NUL as ::core::ffi::c_char;
        ga_concat_len(
            gap,
            &raw mut buf as *mut ::core::ffi::c_char,
            clen as size_t,
        );
        return;
    }
    match *p as ::core::ffi::c_int {
        BS => {
            ga_concat_len(
                gap,
                c"\\b".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        ESC => {
            ga_concat_len(
                gap,
                c"\\e".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        FF => {
            ga_concat_len(
                gap,
                c"\\f".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        NL => {
            ga_concat_len(
                gap,
                c"\\n".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        TAB => {
            ga_concat_len(
                gap,
                c"\\t".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        CAR => {
            ga_concat_len(
                gap,
                c"\\r".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        92 => {
            ga_concat_len(
                gap,
                c"\\\\".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        _ => {
            if (*p as uint8_t as ::core::ffi::c_int) < ' ' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int
            {
                let mut buflen: size_t = vim_snprintf_safelen(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    NUMBUFLEN as ::core::ffi::c_int as size_t,
                    c"\\x%02x".as_ptr(),
                    *p as ::core::ffi::c_int,
                );
                ga_concat_len(gap, &raw mut buf as *mut ::core::ffi::c_char, buflen);
            } else {
                ga_append(gap, *p as uint8_t);
            }
        }
    };
}
unsafe extern "C" fn ga_concat_shorten_esc(
    mut gap: *mut garray_T,
    mut str: *const ::core::ffi::c_char,
) {
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    if str.is_null() {
        ga_concat_len(
            gap,
            c"NULL".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        );
        return;
    }
    let mut p: *const ::core::ffi::c_char = str;
    while *p as ::core::ffi::c_int != NUL {
        let mut same_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut s: *const ::core::ffi::c_char = p;
        let c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
        let clen: ::core::ffi::c_int = s.offset_from(p) as ::core::ffi::c_int;
        while *s as ::core::ffi::c_int != NUL && c == utf_ptr2char(s) {
            same_len += 1;
            s = s.offset(clen as isize);
        }
        if same_len > 20 as ::core::ffi::c_int {
            ga_concat_len(
                gap,
                c"\\[".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
            ga_concat_esc(gap, p, clen);
            ga_concat_len(
                gap,
                c" occurs ".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            );
            let mut buflen: size_t = vim_snprintf_safelen(
                &raw mut buf as *mut ::core::ffi::c_char,
                NUMBUFLEN as ::core::ffi::c_int as size_t,
                c"%d".as_ptr(),
                same_len,
            );
            ga_concat_len(gap, &raw mut buf as *mut ::core::ffi::c_char, buflen);
            ga_concat_len(
                gap,
                c" times]".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            );
            p = s;
        } else {
            ga_concat_esc(gap, p, clen);
            p = p.offset(clen as isize);
        }
    }
}
unsafe extern "C" fn fill_assert_error(
    mut gap: *mut garray_T,
    mut opt_msg_tv: *mut typval_T,
    mut exp_str: *const ::core::ffi::c_char,
    mut exp_tv_arg: *mut typval_T,
    mut got_tv_arg: *mut typval_T,
    mut atype: assert_type_T,
) {
    let mut exp_tv: *mut typval_T = exp_tv_arg;
    let mut got_tv: *mut typval_T = got_tv_arg;
    let mut did_copy: bool = false_0 != 0;
    let mut omitted: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*opt_msg_tv).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && !((*opt_msg_tv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && ((*opt_msg_tv).vval.v_string.is_null()
                || *(*opt_msg_tv).vval.v_string as ::core::ffi::c_int == NUL))
    {
        let mut tofree: *mut ::core::ffi::c_char =
            encode_tv2echo(opt_msg_tv, ::core::ptr::null_mut::<size_t>());
        ga_concat(gap, tofree);
        xfree(tofree as *mut ::core::ffi::c_void);
        ga_concat_len(
            gap,
            c": ".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
        );
    }
    if atype as ::core::ffi::c_uint == ASSERT_MATCH as ::core::ffi::c_int as ::core::ffi::c_uint
        || atype as ::core::ffi::c_uint
            == ASSERT_NOTMATCH as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ga_concat_len(
            gap,
            c"Pattern ".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        );
    } else if atype as ::core::ffi::c_uint
        == ASSERT_NOTEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ga_concat_len(
            gap,
            c"Expected not equal to ".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 23]>().wrapping_sub(1 as size_t),
        );
    } else {
        ga_concat_len(
            gap,
            c"Expected ".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        );
    }
    if exp_str.is_null() {
        if atype as ::core::ffi::c_uint
            != ASSERT_NOTEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*exp_tv).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*got_tv).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            && !(*exp_tv).vval.v_dict.is_null()
            && !(*got_tv).vval.v_dict.is_null()
        {
            let mut exp_d: *mut dict_T = (*exp_tv).vval.v_dict;
            let mut got_d: *mut dict_T = (*got_tv).vval.v_dict;
            did_copy = true_0 != 0;
            (*exp_tv).vval.v_dict = tv_dict_alloc();
            (*got_tv).vval.v_dict = tv_dict_alloc();
            let mut todo: ::core::ffi::c_int = (*exp_d).dv_hashtab.ht_used as ::core::ffi::c_int;
            let mut hi: *const hashitem_T = (*exp_d).dv_hashtab.ht_array;
            while todo > 0 as ::core::ffi::c_int {
                if !((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                {
                    let mut item2: *mut dictitem_T =
                        tv_dict_find(got_d, (*hi).hi_key, -1 as ptrdiff_t);
                    if item2.is_null()
                        || !tv_equal(
                            &raw mut (*((*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize))
                                as *mut dictitem_T))
                                .di_tv,
                            &raw mut (*item2).di_tv,
                            false_0 != 0,
                        )
                    {
                        let key_len: size_t = strlen((*hi).hi_key);
                        tv_dict_add_tv(
                            (*exp_tv).vval.v_dict,
                            (*hi).hi_key,
                            key_len,
                            &raw mut (*((*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize))
                                as *mut dictitem_T))
                                .di_tv,
                        );
                        if !item2.is_null() {
                            tv_dict_add_tv(
                                (*got_tv).vval.v_dict,
                                (*hi).hi_key,
                                key_len,
                                &raw mut (*item2).di_tv,
                            );
                        }
                    } else {
                        omitted += 1;
                    }
                    todo -= 1;
                }
                hi = hi.offset(1);
            }
            todo = (*got_d).dv_hashtab.ht_used as ::core::ffi::c_int;
            let mut hi_0: *const hashitem_T = (*got_d).dv_hashtab.ht_array;
            while todo > 0 as ::core::ffi::c_int {
                if !((*hi_0).hi_key.is_null()
                    || (*hi_0).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                {
                    let mut item2_0: *mut dictitem_T =
                        tv_dict_find(exp_d, (*hi_0).hi_key, -1 as ptrdiff_t);
                    if item2_0.is_null() {
                        let key_len_0: size_t = strlen((*hi_0).hi_key);
                        tv_dict_add_tv(
                            (*got_tv).vval.v_dict,
                            (*hi_0).hi_key,
                            key_len_0,
                            &raw mut (*((*hi_0)
                                .hi_key
                                .offset(-(17 as ::core::ffi::c_ulong as isize))
                                as *mut dictitem_T))
                                .di_tv,
                        );
                    }
                    todo -= 1;
                }
                hi_0 = hi_0.offset(1);
            }
        }
        let mut tofree_0: *mut ::core::ffi::c_char =
            encode_tv2string(exp_tv, ::core::ptr::null_mut::<size_t>());
        ga_concat_shorten_esc(gap, tofree_0);
        xfree(tofree_0 as *mut ::core::ffi::c_void);
    } else {
        if atype as ::core::ffi::c_uint == ASSERT_FAILS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ga_concat_len(
                gap,
                c"'".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            );
        }
        ga_concat_shorten_esc(gap, exp_str);
        if atype as ::core::ffi::c_uint == ASSERT_FAILS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ga_concat_len(
                gap,
                c"'".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            );
        }
    }
    if atype as ::core::ffi::c_uint != ASSERT_NOTEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if atype as ::core::ffi::c_uint == ASSERT_MATCH as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ga_concat_len(
                gap,
                c" does not match ".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
            );
        } else if atype as ::core::ffi::c_uint
            == ASSERT_NOTMATCH as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ga_concat_len(
                gap,
                c" does match ".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as size_t),
            );
        } else {
            ga_concat_len(
                gap,
                c" but got ".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            );
        }
        let mut tofree_1: *mut ::core::ffi::c_char =
            encode_tv2string(got_tv, ::core::ptr::null_mut::<size_t>());
        ga_concat_shorten_esc(gap, tofree_1);
        xfree(tofree_1 as *mut ::core::ffi::c_void);
        if omitted != 0 as ::core::ffi::c_int {
            let mut buf: [::core::ffi::c_char; 100] = [0; 100];
            let mut buflen: size_t = vim_snprintf_safelen(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
                c" - %d equal item%s omitted".as_ptr(),
                omitted,
                if omitted == 1 as ::core::ffi::c_int {
                    c"".as_ptr()
                } else {
                    c"s".as_ptr()
                },
            );
            ga_concat_len(gap, &raw mut buf as *mut ::core::ffi::c_char, buflen);
        }
    }
    if did_copy {
        tv_clear(exp_tv);
        tv_clear(got_tv);
    }
}
unsafe extern "C" fn assert_equal_common(
    mut argvars: *mut typval_T,
    mut atype: assert_type_T,
) -> ::core::ffi::c_int {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if tv_equal(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        argvars.offset(1 as ::core::ffi::c_int as isize),
        false_0 != 0,
    ) as ::core::ffi::c_int
        != (atype as ::core::ffi::c_uint
            == ASSERT_EQUAL as ::core::ffi::c_int as ::core::ffi::c_uint)
            as ::core::ffi::c_int
    {
        prepare_assert_error(&raw mut ga);
        fill_assert_error(
            &raw mut ga,
            argvars.offset(2 as ::core::ffi::c_int as isize),
            ::core::ptr::null::<::core::ffi::c_char>(),
            argvars.offset(0 as ::core::ffi::c_int as isize),
            argvars.offset(1 as ::core::ffi::c_int as isize),
            atype,
        );
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn assert_match_common(
    mut argvars: *mut typval_T,
    mut atype: assert_type_T,
) -> ::core::ffi::c_int {
    let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
    let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
    let pat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf1 as *mut ::core::ffi::c_char,
    );
    let text: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf2 as *mut ::core::ffi::c_char,
    );
    if !pat.is_null()
        && !text.is_null()
        && pattern_match(pat, text, false_0 != 0)
            != (atype as ::core::ffi::c_uint
                == ASSERT_MATCH as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        prepare_assert_error(&raw mut ga);
        fill_assert_error(
            &raw mut ga,
            argvars.offset(2 as ::core::ffi::c_int as isize),
            ::core::ptr::null::<::core::ffi::c_char>(),
            argvars.offset(0 as ::core::ffi::c_int as isize),
            argvars.offset(1 as ::core::ffi::c_int as isize),
            atype,
        );
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn assert_bool(
    mut argvars: *mut typval_T,
    mut is_true: bool,
) -> ::core::ffi::c_int {
    let mut error: bool = false_0 != 0;
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if ((*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) == 0 as varnumber_T) as ::core::ffi::c_int
            == is_true as ::core::ffi::c_int
        || error as ::core::ffi::c_int != 0)
        && ((*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_bool as ::core::ffi::c_uint
                != (if is_true as ::core::ffi::c_int != 0 {
                    kBoolVarTrue as ::core::ffi::c_int
                } else {
                    kBoolVarFalse as ::core::ffi::c_int
                }) as BoolVarValue as ::core::ffi::c_uint)
    {
        prepare_assert_error(&raw mut ga);
        fill_assert_error(
            &raw mut ga,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            if is_true as ::core::ffi::c_int != 0 {
                c"True".as_ptr()
            } else {
                c"False".as_ptr()
            },
            ::core::ptr::null_mut::<typval_T>(),
            argvars.offset(0 as ::core::ffi::c_int as isize),
            ASSERT_OTHER,
        );
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn assert_append_cmd_or_arg(
    mut gap: *mut garray_T,
    mut argvars: *mut typval_T,
    mut cmd: *const ::core::ffi::c_char,
) {
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let tofree: *mut ::core::ffi::c_char = encode_tv2echo(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<size_t>(),
        );
        ga_concat(gap, tofree);
        xfree(tofree as *mut ::core::ffi::c_void);
    } else {
        ga_concat(gap, cmd);
    };
}
unsafe extern "C" fn assert_beeps(
    mut argvars: *mut typval_T,
    mut no_beep: bool,
) -> ::core::ffi::c_int {
    let cmd: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    called_vim_beep.set(false_0 != 0);
    suppress_errthrow.set(true_0 != 0);
    emsg_silent.set(false_0);
    do_cmdline_cmd(cmd);
    if if no_beep as ::core::ffi::c_int != 0 {
        called_vim_beep.get() as ::core::ffi::c_int
    } else {
        !called_vim_beep.get() as ::core::ffi::c_int
    } != 0
    {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        prepare_assert_error(&raw mut ga);
        if no_beep {
            ga_concat_len(
                &raw mut ga,
                c"command did beep: ".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
            );
        } else {
            ga_concat_len(
                &raw mut ga,
                c"command did not beep: ".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 23]>().wrapping_sub(1 as size_t),
            );
        }
        ga_concat(&raw mut ga, cmd);
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        ret = 1 as ::core::ffi::c_int;
    }
    suppress_errthrow.set(false_0 != 0);
    emsg_on_display.set(false_0 != 0);
    return ret;
}
pub unsafe extern "C" fn f_assert_beeps(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_beeps(argvars, false_0 != 0) as varnumber_T;
}
pub unsafe extern "C" fn f_assert_nobeep(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_beeps(argvars, true_0 != 0) as varnumber_T;
}
pub unsafe extern "C" fn f_assert_equal(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_equal_common(argvars, ASSERT_EQUAL) as varnumber_T;
}
unsafe extern "C" fn assert_equalfile(mut argvars: *mut typval_T) -> ::core::ffi::c_int {
    let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
    let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
    let fname1: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf1 as *mut ::core::ffi::c_char,
    );
    let fname2: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf2 as *mut ::core::ffi::c_char,
    );
    if fname1.is_null() || fname2.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    (*IObuff.ptr())[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    let mut IObufflen: size_t = 0 as size_t;
    let fd1: *mut FILE = os_fopen(fname1, READBIN.as_ptr());
    let mut line1: [::core::ffi::c_char; 200] = [0; 200];
    let mut line2: [::core::ffi::c_char; 200] = [0; 200];
    let mut lineidx: ptrdiff_t = 0 as ptrdiff_t;
    if fd1.is_null() {
        IObufflen = vim_snprintf_safelen(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            &raw const e_cant_read_file_str as *const ::core::ffi::c_char,
            fname1,
        );
    } else {
        let fd2: *mut FILE = os_fopen(fname2, READBIN.as_ptr());
        if fd2.is_null() {
            fclose(fd1);
            IObufflen = vim_snprintf_safelen(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                &raw const e_cant_read_file_str as *const ::core::ffi::c_char,
                fname2,
            );
        } else {
            let mut linecount: int64_t = 1 as int64_t;
            let mut count: int64_t = 0 as int64_t;
            loop {
                let c1: ::core::ffi::c_int = fgetc(fd1);
                let c2: ::core::ffi::c_int = fgetc(fd2);
                if c1 == EOF {
                    if c2 != EOF {
                        IObufflen = xstrlcpy(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            c"first file is shorter".as_ptr(),
                            IOSIZE as size_t,
                        );
                    }
                    break;
                } else if c2 == EOF {
                    IObufflen = xstrlcpy(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        c"second file is shorter".as_ptr(),
                        IOSIZE as size_t,
                    );
                    break;
                } else {
                    line1[lineidx as usize] = c1 as ::core::ffi::c_char;
                    line2[lineidx as usize] = c2 as ::core::ffi::c_char;
                    lineidx += 1;
                    if c1 != c2 {
                        IObufflen = vim_snprintf_safelen(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            IOSIZE as size_t,
                            c"difference at byte %ld, line %ld".as_ptr(),
                            count,
                            linecount,
                        );
                        break;
                    } else {
                        if c1 == NL {
                            linecount += 1;
                            lineidx = 0 as ptrdiff_t;
                        } else if lineidx + 2 as ptrdiff_t
                            == ::core::mem::size_of::<[::core::ffi::c_char; 200]>() as ptrdiff_t
                        {
                            memmove(
                                &raw mut line1 as *mut ::core::ffi::c_char
                                    as *mut ::core::ffi::c_void,
                                (&raw mut line1 as *mut ::core::ffi::c_char)
                                    .offset(100 as ::core::ffi::c_int as isize)
                                    as *const ::core::ffi::c_void,
                                (lineidx - 100 as ptrdiff_t) as size_t,
                            );
                            memmove(
                                &raw mut line2 as *mut ::core::ffi::c_char
                                    as *mut ::core::ffi::c_void,
                                (&raw mut line2 as *mut ::core::ffi::c_char)
                                    .offset(100 as ::core::ffi::c_int as isize)
                                    as *const ::core::ffi::c_void,
                                (lineidx - 100 as ptrdiff_t) as size_t,
                            );
                            lineidx -= 100 as ptrdiff_t;
                        }
                        count += 1;
                    }
                }
            }
            fclose(fd1);
            fclose(fd2);
        }
    }
    if IObufflen > 0 as size_t {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        prepare_assert_error(&raw mut ga);
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let tofree: *mut ::core::ffi::c_char = encode_tv2echo(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<size_t>(),
            );
            ga_concat(&raw mut ga, tofree);
            xfree(tofree as *mut ::core::ffi::c_void);
            ga_concat_len(
                &raw mut ga,
                c": ".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        ga_concat_len(
            &raw mut ga,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IObufflen,
        );
        if lineidx > 0 as ptrdiff_t {
            line1[lineidx as usize] = NUL as ::core::ffi::c_char;
            line2[lineidx as usize] = NUL as ::core::ffi::c_char;
            ga_concat_len(
                &raw mut ga,
                c" after \"".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            );
            ga_concat_len(
                &raw mut ga,
                &raw mut line1 as *mut ::core::ffi::c_char,
                lineidx as size_t,
            );
            if strcmp(
                &raw mut line1 as *mut ::core::ffi::c_char,
                &raw mut line2 as *mut ::core::ffi::c_char,
            ) != 0 as ::core::ffi::c_int
            {
                ga_concat_len(
                    &raw mut ga,
                    c"\" vs \"".as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                );
                ga_concat_len(
                    &raw mut ga,
                    &raw mut line2 as *mut ::core::ffi::c_char,
                    lineidx as size_t,
                );
            }
            ga_concat_len(
                &raw mut ga,
                c"\"".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            );
        }
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn f_assert_equalfile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_equalfile(argvars) as varnumber_T;
}
pub unsafe extern "C" fn f_assert_notequal(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_equal_common(argvars, ASSERT_NOTEQUAL) as varnumber_T;
}
pub unsafe extern "C" fn f_assert_exception(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let error: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if *get_vim_var_str(VV_EXCEPTION) as ::core::ffi::c_int == NUL {
        prepare_assert_error(&raw mut ga);
        ga_concat_len(
            &raw mut ga,
            c"v:exception is not set".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 23]>().wrapping_sub(1 as size_t),
        );
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        (*rettv).vval.v_number = 1 as varnumber_T;
    } else if !error.is_null() && strstr(get_vim_var_str(VV_EXCEPTION), error).is_null() {
        prepare_assert_error(&raw mut ga);
        fill_assert_error(
            &raw mut ga,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            ::core::ptr::null::<::core::ffi::c_char>(),
            argvars.offset(0 as ::core::ffi::c_int as isize),
            get_vim_var_tv(VV_EXCEPTION),
            ASSERT_OTHER,
        );
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        (*rettv).vval.v_number = 1 as varnumber_T;
    }
}
pub unsafe extern "C" fn f_assert_fails(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let save_trylevel: ::core::ffi::c_int = trylevel.get();
    let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
    let mut wrong_arg_msg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if tv_check_for_string_or_number_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_string_or_list_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && ((*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && (tv_check_for_opt_number_arg(argvars, 3 as ::core::ffi::c_int) == FAIL
                    || (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type
                        as ::core::ffi::c_uint
                        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                        && tv_check_for_opt_string_arg(argvars, 4 as ::core::ffi::c_int) == FAIL))
    {
        return;
    }
    trylevel.set(0 as ::core::ffi::c_int);
    suppress_errthrow.set(true_0 != 0);
    in_assert_fails.set(true_0 != 0);
    (*no_wait_return.ptr()) += 1;
    let cmd: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    do_cmdline_cmd(cmd);
    trylevel.set(save_trylevel);
    suppress_errthrow.set(false_0 != 0);
    '_theend: {
        if called_emsg.get() == called_emsg_before {
            prepare_assert_error(&raw mut ga);
            ga_concat_len(
                &raw mut ga,
                c"command did not fail: ".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 23]>().wrapping_sub(1 as size_t),
            );
            assert_append_cmd_or_arg(&raw mut ga, argvars, cmd);
            assert_error(&raw mut ga);
            ga_clear(&raw mut ga);
            (*rettv).vval.v_number = 1 as varnumber_T;
        } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut buf: [::core::ffi::c_char; 65] = [0; 65];
            let mut expected: *const ::core::ffi::c_char =
                ::core::ptr::null::<::core::ffi::c_char>();
            let mut expected_str: *const ::core::ffi::c_char =
                ::core::ptr::null::<::core::ffi::c_char>();
            let mut error_found: bool = false_0 != 0;
            let mut error_found_index: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            let mut actual: *mut ::core::ffi::c_char = (if (*emsg_assert_fails_msg.ptr()).is_null()
            {
                c"[unknown]".as_ptr()
            } else {
                emsg_assert_fails_msg.get() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
            if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                expected = tv_get_string_buf_chk(
                    argvars.offset(1 as ::core::ffi::c_int as isize),
                    &raw mut buf as *mut ::core::ffi::c_char,
                );
                error_found = expected.is_null() || strstr(actual, expected).is_null();
            } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type
                as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let list: *const list_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list;
                if list.is_null()
                    || tv_list_len(list) < 1 as ::core::ffi::c_int
                    || tv_list_len(list) > 2 as ::core::ffi::c_int
                {
                    wrong_arg_msg =
                        (e_assert_fails_second_arg.ptr() as *const _) as *const ::core::ffi::c_char;
                    break '_theend;
                } else {
                    let mut tv: *const typval_T = &raw mut (*tv_list_first(list)).li_tv;
                    expected = tv_get_string_buf_chk(tv, &raw mut buf as *mut ::core::ffi::c_char);
                    if expected.is_null() {
                        break '_theend;
                    } else if !pattern_match(expected, actual, false_0 != 0) {
                        error_found = true_0 != 0;
                        expected_str = expected;
                    } else if tv_list_len(list) == 2 as ::core::ffi::c_int {
                        actual = xstrdup(get_vim_var_str(VV_ERRMSG));
                        tofree = actual;
                        tv = &raw mut (*tv_list_last(list)).li_tv;
                        expected =
                            tv_get_string_buf_chk(tv, &raw mut buf as *mut ::core::ffi::c_char);
                        if expected.is_null() {
                            break '_theend;
                        } else if !pattern_match(expected, actual, false_0 != 0) {
                            error_found = true_0 != 0;
                            expected_str = expected;
                        }
                    }
                }
            } else {
                wrong_arg_msg =
                    (e_assert_fails_second_arg.ptr() as *const _) as *const ::core::ffi::c_char;
                break '_theend;
            }
            if !error_found
                && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    wrong_arg_msg = (e_assert_fails_fourth_argument.ptr() as *const _)
                        as *const ::core::ffi::c_char;
                    break '_theend;
                } else {
                    if (*argvars.offset(3 as ::core::ffi::c_int as isize))
                        .vval
                        .v_number
                        >= 0 as varnumber_T
                        && (*argvars.offset(3 as ::core::ffi::c_int as isize))
                            .vval
                            .v_number
                            != emsg_assert_fails_lnum.get() as varnumber_T
                    {
                        error_found = true_0 != 0;
                        error_found_index = 3 as ::core::ffi::c_int;
                    }
                    if !error_found
                        && (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type
                            as ::core::ffi::c_uint
                            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type
                            as ::core::ffi::c_uint
                            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            wrong_arg_msg = (e_assert_fails_fifth_argument.ptr() as *const _)
                                as *const ::core::ffi::c_char;
                            break '_theend;
                        } else if !(*argvars.offset(4 as ::core::ffi::c_int as isize))
                            .vval
                            .v_string
                            .is_null()
                            && !pattern_match(
                                (*argvars.offset(4 as ::core::ffi::c_int as isize))
                                    .vval
                                    .v_string,
                                emsg_assert_fails_context.get(),
                                false_0 != 0,
                            )
                        {
                            error_found = true_0 != 0;
                            error_found_index = 4 as ::core::ffi::c_int;
                        }
                    }
                }
            }
            if error_found {
                let mut actual_tv: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                prepare_assert_error(&raw mut ga);
                if error_found_index == 3 as ::core::ffi::c_int {
                    actual_tv.v_type = VAR_NUMBER;
                    actual_tv.vval.v_number = emsg_assert_fails_lnum.get() as varnumber_T;
                } else if error_found_index == 4 as ::core::ffi::c_int {
                    actual_tv.v_type = VAR_STRING;
                    actual_tv.vval.v_string = emsg_assert_fails_context.get();
                } else {
                    actual_tv.v_type = VAR_STRING;
                    actual_tv.vval.v_string = actual;
                }
                fill_assert_error(
                    &raw mut ga,
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    expected_str,
                    argvars.offset(error_found_index as isize),
                    &raw mut actual_tv,
                    ASSERT_FAILS,
                );
                ga_concat_len(
                    &raw mut ga,
                    c": ".as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
                );
                assert_append_cmd_or_arg(&raw mut ga, argvars, cmd);
                assert_error(&raw mut ga);
                ga_clear(&raw mut ga);
                (*rettv).vval.v_number = 1 as varnumber_T;
            }
        }
    }
    trylevel.set(save_trylevel);
    suppress_errthrow.set(false_0 != 0);
    in_assert_fails.set(false_0 != 0);
    did_emsg.set(false_0);
    got_int.set(false_0 != 0);
    msg_col.set(0 as ::core::ffi::c_int);
    (*no_wait_return.ptr()) -= 1;
    need_wait_return.set(false_0 != 0);
    emsg_on_display.set(false_0 != 0);
    msg_reset_scroll();
    lines_left.set(Rows.get());
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        emsg_assert_fails_msg.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    xfree(tofree as *mut ::core::ffi::c_void);
    set_vim_var_string(
        VV_ERRMSG,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ptrdiff_t,
    );
    if !wrong_arg_msg.is_null() {
        emsg(gettext(wrong_arg_msg));
    }
}
pub unsafe extern "C" fn f_assert_false(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_bool(argvars, false_0 != 0) as varnumber_T;
}
unsafe extern "C" fn assert_inrange(mut argvars: *mut typval_T) -> ::core::ffi::c_int {
    let mut error: bool = false_0 != 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let flower: float_T = tv_get_float(argvars.offset(0 as ::core::ffi::c_int as isize));
        let fupper: float_T = tv_get_float(argvars.offset(1 as ::core::ffi::c_int as isize));
        let factual: float_T = tv_get_float(argvars.offset(2 as ::core::ffi::c_int as isize));
        if factual < flower || factual > fupper {
            let mut ga: garray_T = garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            };
            prepare_assert_error(&raw mut ga);
            let mut expected_str: [::core::ffi::c_char; 200] = [0; 200];
            vim_snprintf(
                &raw mut expected_str as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 200]>(),
                c"range %g - %g,".as_ptr(),
                flower,
                fupper,
            );
            fill_assert_error(
                &raw mut ga,
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut expected_str as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<typval_T>(),
                argvars.offset(2 as ::core::ffi::c_int as isize),
                ASSERT_OTHER,
            );
            assert_error(&raw mut ga);
            ga_clear(&raw mut ga);
            return 1 as ::core::ffi::c_int;
        }
    } else {
        let lower: varnumber_T = tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        let upper: varnumber_T = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        let actual: varnumber_T = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        if error {
            return 0 as ::core::ffi::c_int;
        }
        if actual < lower || actual > upper {
            let mut ga_0: garray_T = garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            };
            prepare_assert_error(&raw mut ga_0);
            let mut expected_str_0: [::core::ffi::c_char; 200] = [0; 200];
            vim_snprintf(
                &raw mut expected_str_0 as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 200]>(),
                c"range %ld - %ld,".as_ptr(),
                lower,
                upper,
            );
            fill_assert_error(
                &raw mut ga_0,
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut expected_str_0 as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<typval_T>(),
                argvars.offset(2 as ::core::ffi::c_int as isize),
                ASSERT_OTHER,
            );
            assert_error(&raw mut ga_0);
            ga_clear(&raw mut ga_0);
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn f_assert_inrange(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_float_or_nr_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_float_or_nr_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_float_or_nr_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_string_arg(argvars, 3 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    (*rettv).vval.v_number = assert_inrange(argvars) as varnumber_T;
}
pub unsafe extern "C" fn f_assert_match(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_match_common(argvars, ASSERT_MATCH) as varnumber_T;
}
pub unsafe extern "C" fn f_assert_notmatch(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_match_common(argvars, ASSERT_NOTMATCH) as varnumber_T;
}
pub unsafe extern "C" fn f_assert_report(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    prepare_assert_error(&raw mut ga);
    ga_concat(
        &raw mut ga,
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
    );
    assert_error(&raw mut ga);
    ga_clear(&raw mut ga);
    (*rettv).vval.v_number = 1 as varnumber_T;
}
pub unsafe extern "C" fn f_assert_true(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_bool(argvars, true_0 != 0) as varnumber_T;
}
pub unsafe extern "C" fn f_test_garbagecollect_now(
    mut _argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if get_vim_var_nr(VV_TESTING) == 0 {
        emsg(gettext(
            (e_calling_test_garbagecollect_now_while_v_testing_is_not_set.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ));
    } else {
        garbage_collect(true_0 != 0);
    };
}
pub unsafe extern "C" fn f_test_write_list_log(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let fname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if fname.is_null() {
        return;
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const READBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rb\0") };
