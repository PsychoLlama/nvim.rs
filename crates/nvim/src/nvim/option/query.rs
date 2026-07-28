//! The accessors the rest of the editor reads options through.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_equalprg() -> *mut c_char {
    if *(*curbuf.get()).b_p_ep as c_int == NUL {
        return p_ep.get();
    }
    return (*curbuf.get()).b_p_ep;
}

pub unsafe extern "C" fn get_findfunc() -> *mut c_char {
    if *(*curbuf.get()).b_p_ffu as c_int == NUL {
        return p_ffu.get();
    }
    return (*curbuf.get()).b_p_ffu;
}

pub unsafe extern "C" fn shortmess(mut x: c_int) -> bool {
    return !(*p_shm.ptr()).is_null()
        && (!vim_strchr(p_shm.get(), x).is_null()
            || !vim_strchr(p_shm.get(), 'a' as c_int).is_null() && {
                let mut c2rust_lvalue: [c_char; 5] = [
                    SHM_RO as c_int as c_char,
                    SHM_MOD as c_int as c_char,
                    SHM_LINES as c_int as c_char,
                    SHM_WRI as c_int as c_char,
                    0 as c_char,
                ];
                !vim_strchr(&raw mut c2rust_lvalue as *mut c_char, x).is_null()
            });
}

pub unsafe extern "C" fn vimrc_found(mut fname: *mut c_char, mut envname: *mut c_char) {
    if !fname.is_null() && !envname.is_null() {
        let mut p: *mut c_char = vim_getenv(envname);
        if p.is_null() {
            p = FullName_save(fname, false_0 != 0);
            if !p.is_null() {
                os_setenv(envname, p, 1 as c_int);
                xfree(p as *mut c_void);
            }
        } else {
            xfree(p as *mut c_void);
        }
    }
}

pub unsafe extern "C" fn option_was_set(mut opt_idx: OptIndex) -> bool {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                6204 as c_uint,
                b"_Bool option_was_set(OptIndex)\0".as_ptr() as *const c_char,
            );
        }
    };
    return (*options.ptr())[opt_idx as usize].flags & kOptFlagWasSet as c_int as uint32_t != 0;
}

pub unsafe extern "C" fn reset_option_was_set(mut opt_idx: OptIndex) {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                6213 as c_uint,
                b"void reset_option_was_set(OptIndex)\0".as_ptr() as *const c_char,
            );
        }
    };
    (*options.ptr())[opt_idx as usize].flags = ((*options.ptr())[opt_idx as usize].flags as c_uint
        & !(kOptFlagWasSet as c_int as c_uint))
        as uint32_t;
}

pub unsafe extern "C" fn fill_culopt_flags(mut val: *mut c_char, mut wp: *mut win_T) -> c_int {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut culopt_flags_new: uint8_t = 0 as uint8_t;
    if val.is_null() {
        p = (*wp).w_onebuf_opt.wo_culopt;
    } else {
        p = val;
    }
    while *p as c_int != NUL {
        if strncmp(p, b"line\0".as_ptr() as *const c_char, 4 as size_t) == 0 as c_int {
            p = p.offset(4 as c_int as isize);
            culopt_flags_new = (culopt_flags_new as c_int | kOptCuloptFlagLine as c_int) as uint8_t;
        } else if strncmp(p, b"both\0".as_ptr() as *const c_char, 4 as size_t) == 0 as c_int {
            p = p.offset(4 as c_int as isize);
            culopt_flags_new = (culopt_flags_new as c_int
                | (kOptCuloptFlagLine as c_int | kOptCuloptFlagNumber as c_int))
                as uint8_t;
        } else if strncmp(p, b"number\0".as_ptr() as *const c_char, 6 as size_t) == 0 as c_int {
            p = p.offset(6 as c_int as isize);
            culopt_flags_new =
                (culopt_flags_new as c_int | kOptCuloptFlagNumber as c_int) as uint8_t;
        } else if strncmp(p, b"screenline\0".as_ptr() as *const c_char, 10 as size_t) == 0 as c_int
        {
            p = p.offset(10 as c_int as isize);
            culopt_flags_new =
                (culopt_flags_new as c_int | kOptCuloptFlagScreenline as c_int) as uint8_t;
        }
        if *p as c_int != ',' as c_int && *p as c_int != NUL {
            return FAIL;
        }
        if *p as c_int == ',' as c_int {
            p = p.offset(1);
        }
    }
    if culopt_flags_new as c_int & kOptCuloptFlagLine as c_int != 0
        && culopt_flags_new as c_int & kOptCuloptFlagScreenline as c_int != 0
    {
        return FAIL;
    }
    (*wp).w_p_culopt_flags = culopt_flags_new;
    return OK;
}

pub unsafe extern "C" fn magic_isset() -> bool {
    match magic_overruled.get() as c_uint {
        1 => return true_0 != 0,
        2 => return false_0 != 0,
        0 | _ => {}
    }
    return p_magic.get() != 0;
}

pub unsafe extern "C" fn option_set_callback_func(
    mut optval: *mut c_char,
    mut optcb: *mut Callback,
) -> c_int {
    if optval.is_null() || *optval as c_int == NUL {
        callback_free(optcb);
        return OK;
    }
    let mut tv: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
    if *optval as c_int == '{' as c_int
        || strncmp(
            optval,
            b"function(\0".as_ptr() as *const c_char,
            9 as size_t,
        ) == 0 as c_int
        || strncmp(optval, b"funcref(\0".as_ptr() as *const c_char, 8 as size_t) == 0 as c_int
    {
        tv = eval_expr(optval, ::core::ptr::null_mut::<exarg_T>());
        if tv.is_null() {
            return FAIL;
        }
    } else {
        tv = xcalloc(1 as size_t, ::core::mem::size_of::<typval_T>()) as *mut typval_T;
        (*tv).v_type = VAR_STRING;
        (*tv).vval.v_string = xstrdup(optval);
    }
    let mut cb: Callback = Callback {
        data: Callback_data {
            funcref: ::core::ptr::null_mut::<c_char>(),
        },
        type_0: kCallbackNone,
    };
    if !callback_from_typval(&raw mut cb, tv)
        || cb.type_0 as c_uint == kCallbackNone as c_int as c_uint
    {
        tv_free(tv);
        return FAIL;
    }
    callback_free(optcb);
    *optcb = cb;
    tv_free(tv);
    return OK;
}

pub unsafe extern "C" fn can_bs(mut what: c_int) -> bool {
    if what == BS_START && bt_prompt(curbuf.get()) as c_int != 0 {
        return false_0 != 0;
    }
    if *p_bs.get() as c_int == '2' as c_int {
        return what != BS_NOSTOP;
    }
    return !vim_strchr(p_bs.get(), what).is_null();
}

pub unsafe extern "C" fn get_bkc_flags(mut buf: *mut buf_T) -> c_uint {
    return if (*buf).b_bkc_flags != 0 {
        (*buf).b_bkc_flags
    } else {
        bkc_flags.get()
    };
}

pub unsafe extern "C" fn get_flp_value(mut buf: *mut buf_T) -> *mut c_char {
    if (*buf).b_p_flp.is_null() || *(*buf).b_p_flp as c_int == NUL {
        return p_flp.get();
    }
    return (*buf).b_p_flp;
}

pub unsafe extern "C" fn get_ve_flags(mut wp: *mut win_T) -> c_uint {
    return (if (*wp).w_onebuf_opt.wo_ve_flags != 0 {
        (*wp).w_onebuf_opt.wo_ve_flags
    } else {
        ve_flags.get()
    }) & !((kOptVeFlagNone as c_int | kOptVeFlagNoneU as c_int) as c_uint);
}

pub unsafe extern "C" fn get_showbreak_value(win: *mut win_T) -> *mut c_char {
    if (*win).w_onebuf_opt.wo_sbr.is_null() || *(*win).w_onebuf_opt.wo_sbr as c_int == NUL {
        return p_sbr.get();
    }
    if strcmp(
        (*win).w_onebuf_opt.wo_sbr,
        b"NONE\0".as_ptr() as *const c_char,
    ) == 0 as c_int
    {
        return empty_string_option.ptr() as *mut c_char;
    }
    return (*win).w_onebuf_opt.wo_sbr;
}

pub unsafe extern "C" fn get_fileformat(mut buf: *const buf_T) -> c_int {
    let mut c: c_int = *(*buf).b_p_ff as c_uchar as c_int;
    if (*buf).b_p_bin != 0 || c == 'u' as c_int {
        return EOL_UNIX;
    }
    if c == 'm' as c_int {
        return EOL_MAC;
    }
    return EOL_DOS;
}

pub unsafe extern "C" fn get_fileformat_force(
    mut buf: *const buf_T,
    mut eap: *const exarg_T,
) -> c_int {
    let mut c: c_int = 0;
    if !eap.is_null() && (*eap).force_ff != 0 as c_int {
        c = (*eap).force_ff;
    } else {
        if if !eap.is_null() && (*eap).force_bin != 0 as c_int {
            ((*eap).force_bin == FORCE_BIN) as c_int
        } else {
            (*buf).b_p_bin
        } != 0
        {
            return EOL_UNIX;
        }
        c = *(*buf).b_p_ff as c_uchar as c_int;
    }
    if c == 'u' as c_int {
        return EOL_UNIX;
    }
    if c == 'm' as c_int {
        return EOL_MAC;
    }
    return EOL_DOS;
}

pub unsafe extern "C" fn default_fileformat() -> c_int {
    match *p_ffs.get() as c_int {
        109 => return EOL_MAC,
        100 => return EOL_DOS,
        _ => {}
    }
    return EOL_UNIX;
}

pub unsafe extern "C" fn set_fileformat(mut eol_style: c_int, mut opt_flags: c_int) {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    match eol_style {
        EOL_UNIX => {
            p = b"unix\0".as_ptr() as *const c_char as *mut c_char;
        }
        EOL_MAC => {
            p = b"mac\0".as_ptr() as *const c_char as *mut c_char;
        }
        EOL_DOS => {
            p = b"dos\0".as_ptr() as *const c_char as *mut c_char;
        }
        _ => {}
    }
    if !p.is_null() {
        set_option_direct(
            kOptFileformat,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(p),
                },
            },
            opt_flags,
            0 as scid_T,
        );
    }
    redraw_buf_status_later(curbuf.get());
    redraw_tabline.set(true_0 != 0);
    need_maketitle.set(true_0 != 0);
}

pub unsafe extern "C" fn skip_to_option_part(mut p: *const c_char) -> *mut c_char {
    if *p as c_int == ',' as c_int {
        p = p.offset(1);
    }
    while *p as c_int == ' ' as c_int {
        p = p.offset(1);
    }
    return p as *mut c_char;
}

pub unsafe extern "C" fn copy_option_part(
    mut option: *mut *mut c_char,
    mut buf: *mut c_char,
    mut maxlen: size_t,
    mut sep_chars: *mut c_char,
) -> size_t {
    let mut len: size_t = 0 as size_t;
    let mut p: *mut c_char = *option;
    if *p as c_int == '.' as c_int {
        let c2rust_fresh7 = p;
        p = p.offset(1);
        let c2rust_fresh8 = len;
        len = len.wrapping_add(1);
        *buf.offset(c2rust_fresh8 as isize) = *c2rust_fresh7;
    }
    while *p as c_int != NUL && vim_strchr(sep_chars, *p as uint8_t as c_int).is_null() {
        if *p.offset(0 as c_int as isize) as c_int == '\\' as c_int
            && !vim_strchr(
                sep_chars,
                *p.offset(1 as c_int as isize) as uint8_t as c_int,
            )
            .is_null()
        {
            p = p.offset(1);
        }
        if len < maxlen.wrapping_sub(1 as size_t) {
            let c2rust_fresh9 = len;
            len = len.wrapping_add(1);
            *buf.offset(c2rust_fresh9 as isize) = *p;
        }
        p = p.offset(1);
    }
    *buf.offset(len as isize) = NUL as c_char;
    if *p as c_int != NUL && *p as c_int != ',' as c_int {
        p = p.offset(1);
    }
    p = skip_to_option_part(p);
    *option = p;
    return len;
}

pub unsafe extern "C" fn csh_like_shell() -> c_int {
    return !strstr(path_tail(p_sh.get()), b"csh\0".as_ptr() as *const c_char).is_null() as c_int;
}

pub unsafe extern "C" fn fish_like_shell() -> bool {
    return !strstr(path_tail(p_sh.get()), b"fish\0".as_ptr() as *const c_char).is_null();
}

pub unsafe extern "C" fn get_winbuf_options(bufopt: c_int) -> *mut dict_T {
    let d: *mut dict_T = tv_dict_alloc();
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        let mut opt: *mut vimoption_T =
            (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
        if bufopt != 0 && option_has_scope(opt_idx, kOptScopeBuf) as c_int != 0
            || bufopt == 0 && option_has_scope(opt_idx, kOptScopeWin) as c_int != 0
        {
            let mut varp: *mut c_void = get_varp(opt);
            if !varp.is_null() {
                let mut opt_tv: typval_T =
                    optval_as_tv(optval_from_varp(opt_idx, varp), true_0 != 0);
                tv_dict_add_tv(d, (*opt).fullname, strlen((*opt).fullname), &raw mut opt_tv);
            }
        }
        opt_idx += 1;
    }
    return d;
}

pub unsafe extern "C" fn get_scrolloff_value(mut wp: *mut win_T) -> int64_t {
    if State.get() & MODE_TERMINAL as c_int != 0 && !(*(*wp).w_buffer).terminal.is_null() {
        return 0 as int64_t;
    }
    return if (*wp).w_onebuf_opt.wo_so < 0 as OptInt {
        p_so.get() as int64_t
    } else {
        (*wp).w_onebuf_opt.wo_so as int64_t
    };
}

pub unsafe extern "C" fn get_sidescrolloff_value(mut wp: *mut win_T) -> int64_t {
    return if (*wp).w_onebuf_opt.wo_siso < 0 as OptInt {
        p_siso.get() as int64_t
    } else {
        (*wp).w_onebuf_opt.wo_siso as int64_t
    };
}
