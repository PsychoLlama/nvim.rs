//! Building and reshaping strings: escaping, formatting, splitting,
//! substituting and spelling.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_char2nr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !tv_check_num(argvars.offset(1 as ::core::ffi::c_int as isize)) {
            return;
        }
    }
    (*rettv).vval.v_number = utf_ptr2char(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    )) as varnumber_T;
}
pub unsafe extern "C" fn f_escape(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    (*rettv).vval.v_string = vim_strsave_escaped(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        tv_get_string_buf(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        ),
    );
    (*rettv).v_type = VAR_STRING;
}
pub unsafe extern "C" fn f_fnameescape(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_string = vim_strsave_fnameescape(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        VSE_NONE as ::core::ffi::c_int,
    );
    (*rettv).v_type = VAR_STRING;
}
pub unsafe extern "C" fn f_gettext(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_nonempty_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xstrdup(gettext(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string,
    ));
}
pub unsafe extern "C" fn f_keytrans(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string
            .is_null()
    {
        return;
    }
    let mut escaped: *mut ::core::ffi::c_char = vim_strsave_escape_ks(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string,
    );
    (*rettv).vval.v_string = str2special_save(escaped, true_0 != 0, true_0 != 0);
    xfree(escaped as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn f_nr2char(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !tv_check_num(argvars.offset(1 as ::core::ffi::c_int as isize)) {
            return;
        }
    }
    let mut error: bool = false_0 != 0;
    let num: varnumber_T = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut error,
    );
    if error {
        return;
    }
    if num < 0 as varnumber_T {
        emsg(gettext(
            b"E5070: Character number must not be less than zero\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    if num > INT_MAX as varnumber_T {
        semsg(
            gettext(
                b"E5071: Character number must not be greater than INT_MAX (%i)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            INT_MAX,
        );
        return;
    }
    let mut buf: [::core::ffi::c_char; 6] = [0; 6];
    let len: ::core::ffi::c_int = utf_char2bytes(
        num as ::core::ffi::c_int,
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xmemdupz(
        &raw mut buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
        len as size_t,
    ) as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn f_printf(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut saved_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(false_0);
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut fmt: *const ::core::ffi::c_char = tv_get_string_buf(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    let mut len: ::core::ffi::c_int = vim_vsnprintf_typval(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as size_t,
        fmt,
        (*dummy_ap.ptr()).clone(),
        argvars.offset(1 as ::core::ffi::c_int as isize),
    );
    if did_emsg.get() == 0 {
        let mut s: *mut ::core::ffi::c_char =
            xmalloc((len as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        (*rettv).vval.v_string = s;
        vim_vsnprintf_typval(
            s,
            (len as size_t).wrapping_add(1 as size_t),
            fmt,
            (*dummy_ap.ptr()).clone(),
            argvars.offset(1 as ::core::ffi::c_int as isize),
        );
    }
    (*did_emsg.ptr()) |= saved_did_emsg;
}
pub unsafe extern "C" fn f_repeat(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut n: varnumber_T = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize));
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_list_alloc_ret(
            rettv,
            (n > 0 as varnumber_T) as ::core::ffi::c_int as ptrdiff_t
                * n as ptrdiff_t
                * tv_list_len(
                    (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_list,
                ) as ptrdiff_t,
        );
        loop {
            let c2rust_fresh8 = n;
            n = n - 1;
            if c2rust_fresh8 <= 0 as varnumber_T {
                break;
            }
            tv_list_extend(
                (*rettv).vval.v_list,
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list,
                ::core::ptr::null_mut::<listitem_T>(),
            );
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_blob_alloc_ret(rettv);
        if (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob
            .is_null()
            || n <= 0 as varnumber_T
        {
            return;
        }
        let slen: ::core::ffi::c_int = (*(*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob)
            .bv_ga
            .ga_len;
        let len: ::core::ffi::c_int = (slen as varnumber_T * n) as ::core::ffi::c_int;
        if len <= 0 as ::core::ffi::c_int {
            return;
        }
        ga_grow(&raw mut (*(*rettv).vval.v_blob).bv_ga, len);
        (*(*rettv).vval.v_blob).bv_ga.ga_len = len;
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < slen {
            if tv_blob_get(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_blob,
                i,
            ) as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                break;
            }
            i += 1;
        }
        if i == slen {
            return;
        }
        i = 0 as ::core::ffi::c_int;
        while (i as varnumber_T) < n {
            tv_blob_set_range(
                (*rettv).vval.v_blob,
                (i * slen) as varnumber_T,
                ((i + 1 as ::core::ffi::c_int) * slen - 1 as ::core::ffi::c_int) as varnumber_T,
                argvars,
            );
            i += 1;
        }
    } else {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if n <= 0 as varnumber_T {
            return;
        }
        let p: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let slen_0: size_t = strlen(p);
        if slen_0 == 0 as size_t {
            return;
        }
        let len_0: size_t = slen_0.wrapping_mul(n as size_t);
        if len_0.wrapping_div(n as size_t) != slen_0 {
            return;
        }
        let r: *mut ::core::ffi::c_char = xmallocz(len_0) as *mut ::core::ffi::c_char;
        let mut i_0: varnumber_T = 0 as varnumber_T;
        while i_0 < n {
            memmove(
                r.offset((i_0 as size_t).wrapping_mul(slen_0) as isize) as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                slen_0,
            );
            i_0 += 1;
        }
        (*rettv).vval.v_string = r;
    };
}
pub unsafe extern "C" fn f_sha256(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    let hash = if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let blob: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob;
        let bytes: &[u8] = if !blob.is_null() && !(*blob).bv_ga.ga_data.is_null() {
            ::core::slice::from_raw_parts(
                (*blob).bv_ga.ga_data as *const u8,
                (*blob).bv_ga.ga_len as usize,
            )
        } else {
            &[]
        };
        hex_digest(bytes)
    } else {
        let p: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        hex_digest(::core::slice::from_raw_parts(p as *const u8, strlen(p)))
    };
    (*rettv).vval.v_string = xmemdupz(hash.as_ptr() as *const ::core::ffi::c_void, hash.len())
        as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn f_shellescape(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let do_special: bool = non_zero_arg(argvars.offset(1 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_string = vim_strsave_shellescape(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        do_special,
        do_special,
    );
    (*rettv).v_type = VAR_STRING;
}
pub unsafe extern "C" fn f_soundfold(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    let s: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_string = eval_soundfold(s);
}
pub unsafe extern "C" fn f_spellbadword(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let wo_spell_save: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_spell;
    if (*curwin.get()).w_onebuf_opt.wo_spell == 0 {
        parse_spelllang(curwin.get());
        (*curwin.get()).w_onebuf_opt.wo_spell = true_0;
    }
    if *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int == NUL {
        emsg(gettext(&raw const e_no_spell as *const ::core::ffi::c_char));
        (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
        return;
    }
    let mut word: *const ::core::ffi::c_char = b"\0".as_ptr() as *const ::core::ffi::c_char;
    let mut attr: hlf_T = HLF_COUNT;
    let mut len: size_t = 0 as size_t;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        len = spell_move_to(
            curwin.get(),
            FORWARD as ::core::ffi::c_int,
            SMT_ALL,
            true_0 != 0,
            &raw mut attr,
        );
        if len != 0 as size_t {
            word = get_cursor_pos_ptr();
            (*curwin.get()).w_set_curswant = true_0;
        }
    } else if *(*curbuf.get()).b_s.b_p_spl as ::core::ffi::c_int != NUL {
        let mut str: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut capcol: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if !str.is_null() {
            while *str as ::core::ffi::c_int != NUL {
                len = spell_check(
                    curwin.get(),
                    str as *mut ::core::ffi::c_char,
                    &raw mut attr,
                    &raw mut capcol,
                    false_0 != 0,
                );
                if attr as ::core::ffi::c_uint
                    != HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    word = str;
                    break;
                } else {
                    str = str.offset(len as isize);
                    capcol -= len as ::core::ffi::c_int;
                    len = 0 as size_t;
                }
            }
        }
    }
    (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
    '_c2rust_label: {
        if len <= 2147483647 as ::core::ffi::c_int as size_t {
        } else {
            __assert_fail(
                b"len <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                6973 as ::core::ffi::c_uint,
                b"void f_spellbadword(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    tv_list_append_string((*rettv).vval.v_list, word, len as ssize_t);
    match attr as ::core::ffi::c_uint {
        37 => {
            tv_list_append_string(
                (*rettv).vval.v_list,
                b"bad\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
        }
        39 => {
            tv_list_append_string(
                (*rettv).vval.v_list,
                b"rare\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
        }
        40 => {
            tv_list_append_string(
                (*rettv).vval.v_list,
                b"local\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
        }
        38 => {
            tv_list_append_string(
                (*rettv).vval.v_list,
                b"caps\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
        }
        _ => {
            tv_list_append_string(
                (*rettv).vval.v_list,
                ::core::ptr::null::<::core::ffi::c_char>(),
                -1 as ssize_t,
            );
        }
    };
}
pub unsafe extern "C" fn f_spellsuggest(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut ga: garray_T = GA_EMPTY_INIT_VALUE;
    let wo_spell_save: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_spell;
    if (*curwin.get()).w_onebuf_opt.wo_spell == 0 {
        parse_spelllang(curwin.get());
        (*curwin.get()).w_onebuf_opt.wo_spell = true_0;
    }
    if *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int == NUL {
        emsg(gettext(&raw const e_no_spell as *const ::core::ffi::c_char));
        (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
        return;
    }
    let mut maxcount: ::core::ffi::c_int = 0;
    let mut need_capital: bool = false_0 != 0;
    let str: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    '_f_spellsuggest_return: {
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut typeerr: bool = false_0 != 0;
            maxcount = tv_get_number_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut typeerr,
            ) as ::core::ffi::c_int;
            if maxcount <= 0 as ::core::ffi::c_int {
                break '_f_spellsuggest_return;
            } else if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type
                as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                need_capital = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut typeerr,
                ) != 0;
                if typeerr {
                    break '_f_spellsuggest_return;
                }
            }
        } else {
            maxcount = 25 as ::core::ffi::c_int;
        }
        spell_suggest_list(
            &raw mut ga,
            str as *mut ::core::ffi::c_char,
            maxcount,
            need_capital,
            false_0 != 0,
        );
    }
    tv_list_alloc_ret(rettv, ga.ga_len as ptrdiff_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < ga.ga_len {
        let p: *mut ::core::ffi::c_char =
            *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
        tv_list_append_allocated_string((*rettv).vval.v_list, p);
        i += 1;
    }
    ga_clear(&raw mut ga);
    (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
}
pub unsafe extern "C" fn f_split(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut col: colnr_T = 0 as colnr_T;
    let mut keepempty: bool = false_0 != 0;
    let mut typeerr: bool = false_0 != 0;
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    let mut str: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut pat: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        pat = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut patbuf as *mut ::core::ffi::c_char,
        );
        if pat.is_null() {
            typeerr = true_0 != 0;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            keepempty = tv_get_bool_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut typeerr,
            ) != 0;
        }
    }
    if pat.is_null() || *pat as ::core::ffi::c_int == NUL {
        pat = b"[\\x01- ]\\+\0".as_ptr() as *const ::core::ffi::c_char;
    }
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if !typeerr {
        regmatch = regmatch_T {
            regprog: vim_regcomp(pat, RE_MAGIC + RE_STRING),
            startp: [
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ],
            endp: [
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ],
            rm_matchcol: 0,
            rm_ic: false_0 != 0,
        };
        if !regmatch.regprog.is_null() {
            while *str as ::core::ffi::c_int != NUL || keepempty as ::core::ffi::c_int != 0 {
                let mut match_0: bool = false;
                if *str as ::core::ffi::c_int == NUL {
                    match_0 = false_0 != 0;
                } else {
                    match_0 = vim_regexec_nl(&raw mut regmatch, str, col);
                }
                let mut end: *const ::core::ffi::c_char =
                    ::core::ptr::null::<::core::ffi::c_char>();
                if match_0 {
                    end = regmatch.startp[0 as ::core::ffi::c_int as usize];
                } else {
                    end = str.offset(strlen(str) as isize);
                }
                if keepempty as ::core::ffi::c_int != 0
                    || end > str
                    || tv_list_len((*rettv).vval.v_list) > 0 as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int != NUL
                        && match_0 as ::core::ffi::c_int != 0
                        && end
                            < regmatch.endp[0 as ::core::ffi::c_int as usize]
                                as *const ::core::ffi::c_char
                {
                    tv_list_append_string(
                        (*rettv).vval.v_list,
                        str,
                        end.offset_from(str) as ssize_t,
                    );
                }
                if !match_0 {
                    break;
                }
                if regmatch.endp[0 as ::core::ffi::c_int as usize] > str as *mut ::core::ffi::c_char
                {
                    col = 0 as ::core::ffi::c_int as colnr_T;
                } else {
                    col = utfc_ptr2len(regmatch.endp[0 as ::core::ffi::c_int as usize]) as colnr_T;
                }
                str = regmatch.endp[0 as ::core::ffi::c_int as usize];
            }
            vim_regfree(regmatch.regprog);
        }
    }
    p_cpo.set(save_cpo);
}
pub unsafe extern "C" fn f_strftime(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut seconds: time_t = 0;
    (*rettv).v_type = VAR_STRING;
    let mut p: *mut ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        seconds = time(::core::ptr::null_mut::<time_t>());
    } else {
        seconds = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as time_t;
    }
    let mut curtime: tm = tm_zeroed();
    if !os_localtime_r(seconds, &mut curtime) {
        (*rettv).vval.v_string = xstrdup(gettext(
            b"(Invalid)\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    let mut conv: vimconv_T = vimconv_T {
        vc_type: 0,
        vc_factor: 0,
        vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        vc_fail: false,
    };
    conv.vc_type = CONV_NONE as ::core::ffi::c_int;
    let mut enc: *mut ::core::ffi::c_char = enc_locale();
    convert_setup(&raw mut conv, p_enc.get(), enc);
    if conv.vc_type != CONV_NONE as ::core::ffi::c_int {
        p = string_convert(&raw mut conv, p, ::core::ptr::null_mut::<size_t>());
    }
    let mut result_buf: [::core::ffi::c_char; 256] = [0; 256];
    if p.is_null()
        || strftime(
            &raw mut result_buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
            p,
            &raw mut curtime,
        ) == 0 as size_t
    {
        result_buf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    }
    if conv.vc_type != CONV_NONE as ::core::ffi::c_int {
        xfree(p as *mut ::core::ffi::c_void);
    }
    convert_setup(&raw mut conv, enc, p_enc.get());
    if conv.vc_type != CONV_NONE as ::core::ffi::c_int {
        (*rettv).vval.v_string = string_convert(
            &raw mut conv,
            &raw mut result_buf as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<size_t>(),
        );
    } else {
        (*rettv).vval.v_string = xstrdup(&raw mut result_buf as *mut ::core::ffi::c_char);
    }
    convert_setup(
        &raw mut conv,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    xfree(enc as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn f_strptime(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fmt_buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut str_buf: [::core::ffi::c_char; 65] = [0; 65];
    // strptime() is asked to determine DST itself.
    let mut tmval: tm = tm {
        tm_isdst: -1,
        ..tm_zeroed()
    };
    let mut fmt: *mut ::core::ffi::c_char = tv_get_string_buf(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut fmt_buf as *mut ::core::ffi::c_char,
    ) as *mut ::core::ffi::c_char;
    let mut str: *mut ::core::ffi::c_char = tv_get_string_buf(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut str_buf as *mut ::core::ffi::c_char,
    ) as *mut ::core::ffi::c_char;
    let mut conv: vimconv_T = vimconv_T {
        vc_type: CONV_NONE as ::core::ffi::c_int,
        vc_factor: 0,
        vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        vc_fail: false,
    };
    let mut enc: *mut ::core::ffi::c_char = enc_locale();
    convert_setup(&raw mut conv, p_enc.get(), enc);
    if conv.vc_type != CONV_NONE as ::core::ffi::c_int {
        fmt = string_convert(&raw mut conv, fmt, ::core::ptr::null_mut::<size_t>());
    }
    if fmt.is_null()
        || os_strptime(CStr::from_ptr(str), CStr::from_ptr(fmt), &mut tmval).is_null()
        || {
            (*rettv).vval.v_number = mktime(&raw mut tmval) as varnumber_T;
            (*rettv).vval.v_number == -1 as varnumber_T
        }
    {
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
    if conv.vc_type != CONV_NONE as ::core::ffi::c_int {
        xfree(fmt as *mut ::core::ffi::c_void);
    }
    convert_setup(
        &raw mut conv,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    xfree(enc as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn f_submatch(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut error: bool = false_0 != 0;
    let mut no: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut error,
    ) as ::core::ffi::c_int;
    if error {
        return;
    }
    if no < 0 as ::core::ffi::c_int || no >= NSUBEXP as ::core::ffi::c_int {
        semsg(
            gettext((e_invalid_submatch_number_nr.ptr() as *const _) as *const ::core::ffi::c_char),
            no,
        );
        return;
    }
    let mut retList: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        retList = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if error {
            return;
        }
    }
    if retList == 0 as ::core::ffi::c_int {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = reg_submatch(no);
    } else {
        (*rettv).v_type = VAR_LIST;
        (*rettv).vval.v_list = reg_submatch_list(no);
    };
}
pub unsafe extern "C" fn f_substitute(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut subbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut flagsbuf: [::core::ffi::c_char; 65] = [0; 65];
    let str: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let pat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut patbuf as *mut ::core::ffi::c_char,
    );
    let mut sub: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let flg: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(3 as ::core::ffi::c_int as isize),
        &raw mut flagsbuf as *mut ::core::ffi::c_char,
    );
    let mut expr: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
    if tv_is_func(*argvars.offset(2 as ::core::ffi::c_int as isize)) {
        expr = argvars.offset(2 as ::core::ffi::c_int as isize);
    } else {
        sub = tv_get_string_buf_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut subbuf as *mut ::core::ffi::c_char,
        );
    }
    (*rettv).v_type = VAR_STRING;
    if str.is_null() || pat.is_null() || sub.is_null() && expr.is_null() || flg.is_null() {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        (*rettv).vval.v_string = do_string_sub(
            str as *mut ::core::ffi::c_char,
            strlen(str),
            pat as *mut ::core::ffi::c_char,
            sub as *mut ::core::ffi::c_char,
            expr,
            flg as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<size_t>(),
        );
    };
}
