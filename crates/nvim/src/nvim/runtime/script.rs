//! What the editor remembers about scripts it has sourced, and how a sourced
//! script is read and left.
//!
//! `script_items` is the registry -- one entry per script ever sourced, with
//! its name, its script-local variables, its `<SID>` and its profiling
//! counters.  `:scriptnames`, `getscriptinfo()`, `get_scriptname` and
//! `find_script_by_name` are its readers, and `script_autoload` is the lookup
//! that turns `foo#bar()` into a `autoload/foo.vim` to source.
//!
//! `getsourceline` and `get_one_sourceline` are the reader `do_cmdline` pulls
//! from while a script runs -- the place `\` continuation lines are joined,
//! 'scriptencoding' conversion is applied and the debugger gets its per-line
//! hook.  `:finish` and `source_finished` are how a script stops early.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn script_is_lua(mut sid: scid_T) -> bool {
    unsafe {
        if sid == SID_LUA {
            return true_0 != 0;
        }
        if !(sid > 0 as ::core::ffi::c_int && sid <= (*script_items.ptr()).ga_len) {
            return false_0 != 0;
        }
        return (**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
            .offset((sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize))
        .sn_lua;
    }
}

pub unsafe extern "C" fn find_script_by_name(
    mut name: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        debug_assert!(
            (*script_items.ptr()).ga_len >= 0 as ::core::ffi::c_int,
            "script_items.ga_len >= 0"
        );
        let mut sid: ::core::ffi::c_int = (*script_items.ptr()).ga_len;
        while sid > 0 as ::core::ffi::c_int {
            let mut si: *mut scriptitem_T = *((*script_items.ptr()).ga_data
                as *mut *mut scriptitem_T)
                .offset((sid - 1 as ::core::ffi::c_int) as isize);
            if !(*si).sn_name.is_null()
                && path_fnamecmp((*si).sn_name, name) == 0 as ::core::ffi::c_int
            {
                return sid;
            }
            sid -= 1;
        }
        return -1 as ::core::ffi::c_int;
    }
}

pub unsafe fn ex_scriptnames(mut eap: *mut exarg_T) {
    unsafe {
        if (*eap).addr_count > 0 as ::core::ffi::c_int || *(*eap).arg as ::core::ffi::c_int != NUL {
            if (*eap).addr_count > 0 as ::core::ffi::c_int
                && !((*eap).line2 > 0 as linenr_T
                    && (*eap).line2 <= (*script_items.ptr()).ga_len as linenr_T)
            {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            } else {
                if (*eap).addr_count > 0 as ::core::ffi::c_int {
                    (*eap).arg = (**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
                        .offset(((*eap).line2 - 1 as linenr_T) as isize))
                    .sn_name;
                } else {
                    expand_env(
                        (*eap).arg,
                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                        MAXPATHL,
                    );
                    (*eap).arg = NameBuff.ptr() as *mut ::core::ffi::c_char;
                }
                do_exedit(eap, ::core::ptr::null_mut::<win_T>());
            }
            return;
        }
        msg_ext_set_kind(c"list_cmd".as_ptr());
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i <= (*script_items.ptr()).ga_len && !got_int.get() {
            if !(**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
                .offset((i - 1 as ::core::ffi::c_int) as isize))
            .sn_name
            .is_null()
            {
                home_replace(
                    ::core::ptr::null::<buf_T>(),
                    (**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
                        .offset((i - 1 as ::core::ffi::c_int) as isize))
                    .sn_name,
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    MAXPATHL as size_t,
                    true_0 != 0,
                );
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    c"%3d: %s".as_ptr(),
                    i,
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                );
                if !message_filtered(IObuff.ptr() as *mut ::core::ffi::c_char) {
                    if msg_col.get() > 0 as ::core::ffi::c_int {
                        msg_putchar('\n' as ::core::ffi::c_int);
                    }
                    msg_outtrans(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                    );
                    line_breakcheck();
                }
            }
            i += 1;
        }
    }
}

pub unsafe extern "C" fn get_scriptname(
    mut script_ctx: sctx_T,
    mut should_free: *mut bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if !should_free.is_null() {
            *should_free = false_0 != 0;
        }
        match script_ctx.sc_sid {
            SID_MODELINE => {
                return gettext(c"modeline".as_ptr());
            }
            SID_CMDARG => {
                return gettext(c"--cmd argument".as_ptr());
            }
            SID_CARG => {
                return gettext(c"-c argument".as_ptr());
            }
            SID_ENV => {
                return gettext(c"environment variable".as_ptr());
            }
            SID_ERROR => {
                return gettext(c"error handler".as_ptr());
            }
            SID_WINLAYOUT => {
                return gettext(c"changed window size".as_ptr());
            }
            SID_LUA => return gettext(c"Lua".as_ptr()),
            SID_API_CLIENT => {
                snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    gettext(c"API client (channel id %lu)".as_ptr()),
                    script_ctx.sc_chan,
                );
                return IObuff.ptr() as *mut ::core::ffi::c_char;
            }
            SID_STR => {
                return gettext(c"anonymous :source".as_ptr());
            }
            _ => {
                let sname: *mut ::core::ffi::c_char =
                    (**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T).offset(
                        (script_ctx.sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            as isize,
                    ))
                    .sn_name;
                if sname.is_null() {
                    snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        gettext(c"anonymous :source (script id %d)".as_ptr()),
                        script_ctx.sc_sid,
                    );
                    return IObuff.ptr() as *mut ::core::ffi::c_char;
                }
                if !should_free.is_null() {
                    *should_free = true_0 != 0;
                    return home_replace_save(::core::ptr::null_mut::<buf_T>(), sname);
                } else {
                    return sname;
                }
            }
        };
    }
}

pub unsafe extern "C" fn get_sourced_lnum(
    mut fgetline: LineGetter,
    mut cookie: *mut ::core::ffi::c_void,
) -> linenr_T {
    unsafe {
        return if fgetline.is_some_and(|f| {
            ::core::ptr::fn_addr_eq(
                f,
                getsourceline
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            )
        }) {
            (*(cookie as *mut source_cookie_T)).sourcing_lnum
        } else {
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum
        };
    }
}

unsafe extern "C" fn get_script_local_funcs(mut sid: scid_T) -> *mut list_T {
    unsafe {
        let functbl: *mut hashtab_T = func_tbl_get();
        let mut l: *mut list_T = tv_list_alloc((*functbl).ht_used as ptrdiff_t);
        let hiht_: *mut hashtab_T = functbl;
        let mut hitodo_: size_t = (*hiht_).ht_used;
        let mut hi: *mut hashitem_T = (*hiht_).ht_array;
        while hitodo_ != 0 {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                hitodo_ = hitodo_.wrapping_sub(1);
                let fp: *const ufunc_T =
                    (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
                if (*fp).uf_script_ctx.sc_sid == sid {
                    let name: *const ::core::ffi::c_char = if !(*fp).uf_name_exp.is_null() {
                        (*fp).uf_name_exp as *const ::core::ffi::c_char
                    } else {
                        &raw const (*fp).uf_name as *const ::core::ffi::c_char
                    };
                    tv_list_append_string(l, name, -1 as ssize_t);
                }
            }
            hi = hi.offset(1);
        }
        return l;
    }
}

pub unsafe extern "C" fn f_getscriptinfo(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_alloc_ret(rettv, (*script_items.ptr()).ga_len as ptrdiff_t);
        if tv_check_for_opt_dict_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
            return;
        }
        let mut l: *mut list_T = (*rettv).vval.v_list;
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: p_ic.get() != 0,
        };
        let mut filterpat: bool = false_0 != 0;
        let mut sid: varnumber_T = -1 as varnumber_T;
        let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut sid_di: *mut dictitem_T = tv_dict_find(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_dict,
                c"sid".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1_usize)
                    as ptrdiff_t,
            );
            if !sid_di.is_null() {
                let mut error: bool = false_0 != 0;
                sid = tv_get_number_chk(&raw mut (*sid_di).di_tv, &raw mut error);
                if error {
                    return;
                }
                if sid <= 0 as varnumber_T {
                    semsg_c!(
                        gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                        c"sid".as_ptr(),
                        tv_get_string(&raw mut (*sid_di).di_tv),
                    );
                    return;
                }
            } else {
                pat = tv_dict_get_string(
                    (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_dict,
                    c"name".as_ptr(),
                    true_0 != 0,
                );
                if !pat.is_null() {
                    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
                }
                if !regmatch.regprog.is_null() {
                    filterpat = true_0 != 0;
                }
            }
        }
        let mut i: varnumber_T = if sid > 0 as varnumber_T {
            sid
        } else {
            1 as varnumber_T
        };
        while (i == sid || sid <= 0 as varnumber_T)
            && i <= (*script_items.ptr()).ga_len as varnumber_T
        {
            let mut si: *mut scriptitem_T = *((*script_items.ptr()).ga_data
                as *mut *mut scriptitem_T)
                .offset((i - 1 as varnumber_T) as isize);
            if !(*si).sn_name.is_null() {
                if !(filterpat as ::core::ffi::c_int != 0
                    && !vim_regexec(&raw mut regmatch, (*si).sn_name, 0 as colnr_T))
                {
                    let mut d: *mut dict_T = tv_dict_alloc();
                    tv_list_append_dict(l, d);
                    tv_dict_add_str(
                        d,
                        c"name".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                            .wrapping_sub(1 as size_t),
                        (*si).sn_name,
                    );
                    tv_dict_add_nr(
                        d,
                        c"sid".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                            .wrapping_sub(1 as size_t),
                        i,
                    );
                    tv_dict_add_nr(
                        d,
                        c"version".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                            .wrapping_sub(1 as size_t),
                        1 as varnumber_T,
                    );
                    tv_dict_add_bool(
                        d,
                        c"autoload".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 9]>()
                            .wrapping_sub(1 as size_t),
                        kBoolVarFalse,
                    );
                    if sid > 0 as varnumber_T {
                        let mut var_dict: *mut dict_T = tv_dict_copy(
                            ::core::ptr::null::<vimconv_T>(),
                            &raw mut (*(*si).sn_vars).sv_dict,
                            true_0 != 0,
                            get_copyID(),
                        );
                        tv_dict_add_dict(
                            d,
                            c"variables".as_ptr(),
                            ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                                .wrapping_sub(1 as size_t),
                            var_dict,
                        );
                        tv_dict_add_list(
                            d,
                            c"functions".as_ptr(),
                            ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                                .wrapping_sub(1 as size_t),
                            get_script_local_funcs(sid as scid_T),
                        );
                    }
                }
            }
            i += 1;
        }
        vim_regfree(regmatch.regprog);
        xfree(pat as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn getsourceline(
    mut _c: ::core::ffi::c_int,
    mut cookie: *mut ::core::ffi::c_void,
    mut _indent: ::core::ffi::c_int,
    mut do_concat: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut sp: *mut source_cookie_T = cookie as *mut source_cookie_T;
        let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*sp).dbg_tick < debug_tick.get() && !(*sp).source_from_buf_or_str {
            (*sp).breakpoint = dbg_find_breakpoint(
                true_0 != 0,
                (*sp).fname,
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum,
            );
            (*sp).dbg_tick = debug_tick.get();
        }
        if do_profiling.get() == PROF_YES {
            script_line_end();
        }
        (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum = (*sp).sourcing_lnum + 1 as linenr_T;
        if (*sp).finished as ::core::ffi::c_int != 0
            || !(*sp).source_from_buf_or_str && (*sp).fp.is_null()
        {
            line = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else if (*sp).nextline.is_null() {
            line = get_one_sourceline(sp);
        } else {
            line = (*sp).nextline;
            (*sp).nextline = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*sp).sourcing_lnum += 1;
        }
        if !line.is_null() && do_profiling.get() == PROF_YES {
            script_line_start();
        }
        if !line.is_null()
            && do_concat as ::core::ffi::c_int != 0
            && vim_strchr(p_cpo.get(), CPO_CONCAT).is_null()
        {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*sp).sourcing_lnum -= 1;
            (*sp).nextline = get_one_sourceline(sp);
            if !(*sp).nextline.is_null() && {
                p = skipwhite((*sp).nextline);
                *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                        && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int
            } {
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
                    400 as ::core::ffi::c_int,
                );
                ga_concat(&raw mut ga, line);
                while !(*sp).nextline.is_null()
                    && concat_continued_line(
                        &raw mut ga,
                        400 as ::core::ffi::c_int,
                        (*sp).nextline,
                        strlen((*sp).nextline),
                    ) as ::core::ffi::c_int
                        != 0
                {
                    xfree((*sp).nextline as *mut ::core::ffi::c_void);
                    (*sp).nextline = get_one_sourceline(sp);
                }
                ga_append(&raw mut ga, NUL as uint8_t);
                xfree(line as *mut ::core::ffi::c_void);
                line = ga.ga_data as *mut ::core::ffi::c_char;
            }
        }
        if !line.is_null() && (*sp).conv.vc_type != CONV_NONE {
            let mut s: *mut ::core::ffi::c_char =
                string_convert(&raw mut (*sp).conv, line, ::core::ptr::null_mut::<size_t>());
            if !s.is_null() {
                xfree(line as *mut ::core::ffi::c_void);
                line = s;
            }
        }
        if !(*sp).source_from_buf_or_str
            && (*sp).breakpoint != 0 as linenr_T
            && (*sp).breakpoint
                <= (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum
        {
            dbg_breakpoint(
                (*sp).fname,
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum,
            );
            (*sp).breakpoint = dbg_find_breakpoint(
                true_0 != 0,
                (*sp).fname,
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum,
            );
            (*sp).dbg_tick = debug_tick.get();
        }
        return line;
    }
}

unsafe extern "C" fn get_one_sourceline(mut sp: *mut source_cookie_T) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut len: ::core::ffi::c_int = 0;
        let mut c: ::core::ffi::c_int = 0;
        let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut have_read: bool = false_0 != 0;
        ga_init(
            &raw mut ga,
            1 as ::core::ffi::c_int,
            250 as ::core::ffi::c_int,
        );
        (*sp).sourcing_lnum += 1;
        's_138: loop {
            ga_grow(&raw mut ga, 120 as ::core::ffi::c_int);
            if (*sp).source_from_buf_or_str {
                if (*sp).buf_lnum >= (*sp).buflines.ga_len {
                    break;
                }
                ga_concat(
                    &raw mut ga,
                    *((*sp).buflines.ga_data as *mut *mut ::core::ffi::c_char)
                        .offset((*sp).buf_lnum as isize),
                );
                (*sp).buf_lnum += 1;
                ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                buf = ga.ga_data as *mut ::core::ffi::c_char;
                let c2rust_fresh3 = ga.ga_len;
                ga.ga_len = ga.ga_len + 1;
                *buf.offset(c2rust_fresh3 as isize) = NUL as ::core::ffi::c_char;
                len = ga.ga_len;
            } else {
                buf = ga.ga_data as *mut ::core::ffi::c_char;
                loop {
                    *__errno_location() = 0 as ::core::ffi::c_int;
                    if !fgets(
                        buf.offset(ga.ga_len as isize),
                        ga.ga_maxlen - ga.ga_len,
                        (*sp).fp,
                    )
                    .is_null()
                    {
                        break;
                    }
                    if *__errno_location() != EINTR {
                        break 's_138;
                    }
                }
                len = ga.ga_len + strlen(buf.offset(ga.ga_len as isize)) as ::core::ffi::c_int;
            }
            have_read = true_0 != 0;
            ga.ga_len = len;
            if ga.ga_maxlen - ga.ga_len == 1 as ::core::ffi::c_int
                && *buf.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                    != '\n' as ::core::ffi::c_int
            {
                continue;
            }
            if len >= 1 as ::core::ffi::c_int
                && *buf.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                    == '\n' as ::core::ffi::c_int
            {
                c = len - 2 as ::core::ffi::c_int;
                while c >= 0 as ::core::ffi::c_int
                    && *buf.offset(c as isize) as ::core::ffi::c_int == Ctrl_V
                {
                    c -= 1;
                }
                if len & 1 as ::core::ffi::c_int != c & 1 as ::core::ffi::c_int {
                    (*sp).sourcing_lnum += 1;
                    continue;
                } else {
                    *buf.offset((len - 1 as ::core::ffi::c_int) as isize) =
                        NUL as ::core::ffi::c_char;
                }
            }
            line_breakcheck();
            break;
        }
        if have_read {
            return ga.ga_data as *mut ::core::ffi::c_char;
        }
        xfree(ga.ga_data);
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn sourcing_a_script(mut eap: *mut exarg_T) -> ::core::ffi::c_int {
    unsafe {
        return getline_equal(
            (*eap).ea_getline,
            (*eap).cookie,
            Some(
                getsourceline
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
        ) as ::core::ffi::c_int;
    }
}

pub unsafe fn ex_scriptencoding(mut eap: *mut exarg_T) {
    unsafe {
        if sourcing_a_script(eap) == 0 {
            emsg(gettext(
                c"E167: :scriptencoding used outside of a sourced file".as_ptr(),
            ));
            return;
        }
        let mut name: *mut ::core::ffi::c_char = if *(*eap).arg as ::core::ffi::c_int != NUL {
            enc_canonize((*eap).arg)
        } else {
            (*eap).arg
        };
        let mut sp: *mut source_cookie_T =
            getline_cookie((*eap).ea_getline, (*eap).cookie) as *mut source_cookie_T;
        convert_setup(&raw mut (*sp).conv, name, p_enc.get());
        if name != (*eap).arg {
            xfree(name as *mut ::core::ffi::c_void);
        }
    }
}

pub unsafe fn ex_finish(mut eap: *mut exarg_T) {
    unsafe {
        if sourcing_a_script(eap) != 0 {
            do_finish(eap, false_0 != 0);
        } else {
            emsg(gettext(
                c"E168: :finish used outside of a sourced file".as_ptr(),
            ));
        };
    }
}

pub unsafe extern "C" fn do_finish(mut eap: *mut exarg_T, mut reanimate: bool) {
    unsafe {
        if reanimate {
            (*(getline_cookie((*eap).ea_getline, (*eap).cookie) as *mut source_cookie_T))
                .finished = false_0 != 0;
        }
        let mut idx: ::core::ffi::c_int =
            cleanup_conditionals((*eap).cstack, 0 as ::core::ffi::c_int, true_0);
        if idx >= 0 as ::core::ffi::c_int {
            (*(*eap).cstack).cs_pending[idx as usize] = CSTP_FINISH as ::core::ffi::c_char;
            report_make_pending(CSTP_FINISH, NULL_0);
        } else {
            (*(getline_cookie((*eap).ea_getline, (*eap).cookie) as *mut source_cookie_T))
                .finished = true_0 != 0;
        };
    }
}

pub unsafe extern "C" fn source_finished(
    mut fgetline: LineGetter,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    unsafe {
        return getline_equal(
            fgetline,
            cookie,
            Some(
                getsourceline
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
        ) as ::core::ffi::c_int
            != 0
            && (*(getline_cookie(fgetline, cookie) as *mut source_cookie_T)).finished
                as ::core::ffi::c_int
                != 0;
    }
}

pub unsafe extern "C" fn autoload_name(
    name: *const ::core::ffi::c_char,
    name_len: size_t,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let scriptname: *mut ::core::ffi::c_char =
            xmalloc(name_len.wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 14]>()))
                as *mut ::core::ffi::c_char;
        memcpy(
            scriptname as *mut ::core::ffi::c_void,
            c"autoload/".as_ptr() as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        );
        memcpy(
            scriptname
                .add(::core::mem::size_of::<[::core::ffi::c_char; 10]>())
                .offset(-(1 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_void,
            name as *const ::core::ffi::c_void,
            name_len,
        );
        let mut auchar_idx: size_t = 0 as size_t;
        let mut i: size_t =
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t);
        while i
            .wrapping_sub(::core::mem::size_of::<[::core::ffi::c_char; 10]>())
            .wrapping_add(1 as size_t)
            < name_len
        {
            if *scriptname.add(i) as ::core::ffi::c_int == AUTOLOAD_CHAR {
                *scriptname.add(i) = '/' as ::core::ffi::c_char;
                auchar_idx = i;
            }
            i = i.wrapping_add(1);
        }
        memcpy(
            scriptname.add(auchar_idx) as *mut ::core::ffi::c_void,
            c".vim".as_ptr() as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>(),
        );
        return scriptname;
    }
}

pub unsafe extern "C" fn script_autoload(
    name: *const ::core::ffi::c_char,
    name_len: size_t,
    reload: bool,
) -> bool {
    unsafe {
        let mut p: *const ::core::ffi::c_char =
            memchr(name as *const ::core::ffi::c_void, AUTOLOAD_CHAR, name_len)
                as *const ::core::ffi::c_char;
        if p.is_null() || p == name {
            return false_0 != 0;
        }
        let mut ret: bool = false_0 != 0;
        let mut tofree: *mut ::core::ffi::c_char = autoload_name(name, name_len);
        let mut scriptname: *mut ::core::ffi::c_char = tofree;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*ga_loaded.ptr()).ga_len {
            if strcmp(
                (*((*ga_loaded.ptr()).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize))
                    .offset(9 as ::core::ffi::c_int as isize),
                scriptname.offset(9 as ::core::ffi::c_int as isize),
            ) == 0 as ::core::ffi::c_int
            {
                break;
            }
            i += 1;
        }
        if !reload && i < (*ga_loaded.ptr()).ga_len {
            ret = false_0 != 0;
        } else {
            if i == (*ga_loaded.ptr()).ga_len {
                ga_grow(ga_loaded.ptr(), 1 as ::core::ffi::c_int);
                *((*ga_loaded.ptr()).ga_data as *mut *mut ::core::ffi::c_char)
                    .offset((*ga_loaded.ptr()).ga_len as isize) = scriptname;
                (*ga_loaded.ptr()).ga_len += 1;
                tofree = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            let mut ret_sid: ::core::ffi::c_int = 0;
            if do_in_runtimepath(
                scriptname,
                DIP_START as ::core::ffi::c_int,
                Some(
                    source_callback
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut *mut ::core::ffi::c_char,
                            bool,
                            *mut ::core::ffi::c_void,
                        ) -> bool,
                ),
                &raw mut ret_sid as *mut ::core::ffi::c_void,
            ) == OK
            {
                ret = true_0 != 0;
            }
        }
        xfree(tofree as *mut ::core::ffi::c_void);
        return ret;
    }
}
