//! Sourcing a script -- `:source`, `:runtime`'s callback, and `nvim_exec2`.
//!
//! `do_source_ext` is the whole of it: open the file (or take the buffer or
//! string the API handed over), find or create the script's `scriptitem_T`
//! and its script-local scope, push it on the execution stack, install the
//! line reader that handles `\`-continuations and 'scriptencoding'
//! conversion, run `do_cmdline` over it, and unwind all of that whatever
//! happens.  Everything else here is one of the entry points into it, or one
//! of the accessors `do_cmdline` calls back through to ask about the source
//! it is reading from.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn cmd_source(mut fname: *mut ::core::ffi::c_char, mut eap: *mut exarg_T) {
    unsafe {
        if *fname as ::core::ffi::c_int != NUL
            && !eap.is_null()
            && (*eap).addr_count > 0 as ::core::ffi::c_int
        {
            emsg(gettext(&raw const e_norange as *const ::core::ffi::c_char));
            return;
        }
        if !eap.is_null() && *fname as ::core::ffi::c_int == NUL {
            if (*eap).forceit != 0 {
                emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
            } else {
                cmd_source_buffer(eap, false_0 != 0);
            }
        } else if !eap.is_null() && (*eap).forceit != 0 {
            openscript(
                fname,
                global_busy.get() != 0
                    || listcmd_busy.get() as ::core::ffi::c_int != 0
                    || !(*eap).nextcmd.is_null()
                    || (*(*eap).cstack).cs_idx >= 0 as ::core::ffi::c_int,
            );
        } else if do_source(
            fname,
            false_0 != 0,
            DOSO_NONE,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ) == FAIL
        {
            semsg_c!(
                gettext(&raw const e_notopen as *const ::core::ffi::c_char),
                fname,
            );
        }
    }
}

pub unsafe fn ex_source(mut eap: *mut exarg_T) {
    unsafe {
        cmd_source((*eap).arg, eap);
    }
}

pub unsafe fn ex_options(mut _eap: *mut exarg_T) {
    unsafe {
        let mut buf: [::core::ffi::c_char; 500] = [0; 500];
        let mut multi_mods: bool = false;
        buf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        add_win_cmd_modifiers(
            &raw mut buf as *mut ::core::ffi::c_char,
            cmdmod.ptr(),
            &raw mut multi_mods,
        );
        os_setenv(
            c"OPTWIN_CMD".as_ptr(),
            &raw mut buf as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
        cmd_source(
            SYS_OPTWIN_FILE.as_ptr() as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<exarg_T>(),
        );
    }
}

pub unsafe extern "C" fn source_breakpoint(mut cookie: *mut ::core::ffi::c_void) -> *mut linenr_T {
    unsafe {
        return &raw mut (*(cookie as *mut source_cookie_T)).breakpoint;
    }
}

pub unsafe extern "C" fn source_dbg_tick(
    mut cookie: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_int {
    unsafe {
        return &raw mut (*(cookie as *mut source_cookie_T)).dbg_tick;
    }
}

pub unsafe extern "C" fn source_level(mut cookie: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    unsafe {
        return (*(cookie as *mut source_cookie_T)).level;
    }
}

unsafe extern "C" fn fopen_noinh_readbin(mut filename: *mut ::core::ffi::c_char) -> *mut FILE {
    unsafe {
        let mut fd_tmp: ::core::ffi::c_int = os_open(filename, O_RDONLY, 0 as ::core::ffi::c_int);
        if fd_tmp < 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<FILE>();
        }
        os_set_cloexec(fd_tmp);
        return fdopen(fd_tmp, READBIN.as_ptr());
    }
}

pub(crate) unsafe extern "C" fn concat_continued_line(
    ga: *mut garray_T,
    init_growsize: ::core::ffi::c_int,
    p: *const ::core::ffi::c_char,
    mut len: size_t,
) -> bool {
    unsafe {
        let line: *const ::core::ffi::c_char = skipwhite_len(p, len);
        len = len.wrapping_sub(line.offset_from(p) as size_t);
        if len >= 3 as size_t
            && strncmp(line, c"\"\\ ".as_ptr(), 3 as size_t) == 0 as ::core::ffi::c_int
        {
            return true_0 != 0;
        } else if len == 0 as size_t
            || *line.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '\\' as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
        if (*ga).ga_len > init_growsize {
            ga_set_growsize(
                ga,
                if (*ga).ga_len < 8000 as ::core::ffi::c_int {
                    (*ga).ga_len
                } else {
                    8000 as ::core::ffi::c_int
                },
            );
        }
        ga_concat_len(
            ga,
            line.offset(1 as ::core::ffi::c_int as isize),
            len.wrapping_sub(1 as size_t),
        );
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn new_script_item(
    name: *mut ::core::ffi::c_char,
    sid_out: *mut scid_T,
) -> *mut scriptitem_T {
    unsafe {
        static last_current_SID: GlobalCell<scid_T> = GlobalCell::new(0 as scid_T);
        (*last_current_SID.ptr()) += 1;
        let sid: scid_T = last_current_SID.get();
        if !sid_out.is_null() {
            *sid_out = sid;
        }
        ga_grow(
            script_items.ptr(),
            sid as ::core::ffi::c_int - (*script_items.ptr()).ga_len,
        );
        while (*script_items.ptr()).ga_len < sid {
            let mut si: *mut scriptitem_T =
                xcalloc(1 as size_t, ::core::mem::size_of::<scriptitem_T>()) as *mut scriptitem_T;
            (*script_items.ptr()).ga_len += 1;
            *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
                .offset(((*script_items.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize) = si;
            (*si).sn_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
            new_script_vars((*script_items.ptr()).ga_len as scid_T);
            (*si).sn_prof_on = false_0 != 0;
        }
        (**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
            .offset((sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize))
        .sn_name = name;
        return *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
            .offset((sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize);
    }
}

unsafe extern "C" fn do_source_buffer_init(
    mut sp: *mut source_cookie_T,
    mut eap: *const exarg_T,
    mut ex_lua: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if (*curbuf.ptr()).is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !(*curbuf.get()).b_ffname.is_null() {
            fname = xstrdup((*curbuf.get()).b_ffname);
        } else {
            if ex_lua {
                snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    c":{range}lua buffer=%d".as_ptr(),
                    (*curbuf.get()).handle,
                );
            } else {
                snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    c":source buffer=%d".as_ptr(),
                    (*curbuf.get()).handle,
                );
            }
            fname = xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
        }
        ga_init(
            &raw mut (*sp).buflines,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            100 as ::core::ffi::c_int,
        );
        let mut curr_lnum: linenr_T = (*eap).line1;
        while curr_lnum <= (*eap).line2 {
            ga_grow(&raw mut (*sp).buflines, 1 as ::core::ffi::c_int);
            *((*sp).buflines.ga_data as *mut *mut ::core::ffi::c_char)
                .offset((*sp).buflines.ga_len as isize) = xstrdup(ml_get(curr_lnum));
            (*sp).buflines.ga_len += 1;
            curr_lnum += 1;
        }
        (*sp).buf_lnum = 0 as ::core::ffi::c_int;
        (*sp).source_from_buf_or_str = true_0 != 0;
        (*sp).sourcing_lnum = (*eap).line1 - 1 as linenr_T;
        return fname;
    }
}

unsafe extern "C" fn do_source_str_init(
    mut sp: *mut source_cookie_T,
    mut str: *const ::core::ffi::c_char,
) {
    unsafe {
        ga_init(
            &raw mut (*sp).buflines,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            100 as ::core::ffi::c_int,
        );
        while *str as ::core::ffi::c_int != NUL {
            let mut eol: *const ::core::ffi::c_char = skip_to_newline(str);
            ga_grow(&raw mut (*sp).buflines, 1 as ::core::ffi::c_int);
            *((*sp).buflines.ga_data as *mut *mut ::core::ffi::c_char)
                .offset((*sp).buflines.ga_len as isize) = xmemdupz(
                str as *const ::core::ffi::c_void,
                eol.offset_from(str) as size_t,
            ) as *mut ::core::ffi::c_char;
            (*sp).buflines.ga_len += 1;
            str = eol.offset((*eol as ::core::ffi::c_int != NUL) as ::core::ffi::c_int as isize);
        }
        (*sp).buf_lnum = 0 as ::core::ffi::c_int;
        (*sp).source_from_buf_or_str = true_0 != 0;
    }
}

pub unsafe extern "C" fn cmd_source_buffer(eap: *const exarg_T, mut ex_lua: bool) {
    unsafe {
        do_source_ext(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            DOSO_NONE,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            eap,
            ex_lua,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
    }
}

pub unsafe extern "C" fn do_source_str(
    mut str: *const ::core::ffi::c_char,
    mut traceback_name: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let sourcing_name: *mut ::core::ffi::c_char = (*((*exestack.ptr()).ga_data
            as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name;
        let sourcing_lnum: linenr_T = (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum;
        let mut sname_buf: [::core::ffi::c_char; 256] = [0; 256];
        if !sourcing_name.is_null() {
            snprintf(
                &raw mut sname_buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
                c"%s called at %s:%d".as_ptr(),
                traceback_name,
                sourcing_name,
                sourcing_lnum,
            );
            traceback_name = &raw mut sname_buf as *mut ::core::ffi::c_char;
        }
        return do_source_ext(
            traceback_name,
            false_0 != 0,
            DOSO_NONE,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null::<exarg_T>(),
            false_0 != 0,
            str,
        );
    }
}

unsafe extern "C" fn do_source_ext(
    fname: *mut ::core::ffi::c_char,
    check_other: bool,
    is_vimrc: ::core::ffi::c_int,
    ret_sid: *mut ::core::ffi::c_int,
    eap: *const exarg_T,
    ex_lua: bool,
    str: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut sid: ::core::ffi::c_int = 0;
        let mut rel_time: proftime_T = 0;
        let mut start_time: proftime_T = 0;
        let mut l_time_fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
        let mut l_do_profiling: ::core::ffi::c_int = 0;
        let mut funccalp_entry: funccal_entry_T = funccal_entry_T {
            top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            next: ::core::ptr::null_mut::<funccal_entry_T>(),
        };
        let mut save_current_sctx: sctx_T = sctx_T {
            sc_sid: 0,
            sc_seq: 0,
            sc_lnum: 0,
            sc_chan: 0,
        };
        let mut ts_lua: bool = false;
        let mut cookie: source_cookie_T = source_cookie_T {
            fp: ::core::ptr::null_mut::<FILE>(),
            nextline: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            sourcing_lnum: 0,
            finished: false,
            source_from_buf_or_str: false,
            buf_lnum: 0,
            buflines: garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
            breakpoint: 0,
            fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            dbg_tick: 0,
            level: 0,
            conv: vimconv_T {
                vc_type: 0,
                vc_factor: 0,
                vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                vc_fail: false,
            },
        };
        let mut firstline: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
        let mut retval: ::core::ffi::c_int = FAIL;
        let mut save_debug_break_level: ::core::ffi::c_int = debug_break_level.get();
        let mut si: *mut scriptitem_T = ::core::ptr::null_mut::<scriptitem_T>();
        let mut wait_start: proftime_T = 0;
        let mut trigger_source_post: bool = false_0 != 0;
        memset(
            &raw mut cookie as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<source_cookie_T>(),
        );
        let mut fname_exp: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        '_theend: {
            if fname.is_null() {
                debug_assert!(str.is_null(), "str == NULL");
                fname_exp = do_source_buffer_init(&raw mut cookie, eap, ex_lua);
                if fname_exp.is_null() {
                    return FAIL;
                }
            } else if !str.is_null() {
                do_source_str_init(&raw mut cookie, str);
                fname_exp = xstrdup(fname);
            } else {
                let mut p: *mut ::core::ffi::c_char = expand_env_save(fname);
                if p.is_null() {
                    return retval;
                }
                fname_exp = fix_fname(p);
                xfree(p as *mut ::core::ffi::c_void);
                if fname_exp.is_null() {
                    return retval;
                }
                if os_isdir(fname_exp) {
                    smsg_c!(
                        0 as ::core::ffi::c_int,
                        gettext(c"Cannot source a directory: \"%s\"".as_ptr()),
                        fname,
                    );
                    break '_theend;
                }
            }
            sid = if !str.is_null() {
                SID_STR
            } else {
                find_script_by_name(fname_exp)
            };
            if sid > 0 as ::core::ffi::c_int && !ret_sid.is_null() {
                *ret_sid = sid;
                retval = OK;
            } else {
                if str.is_null() {
                    if has_autocmd(EVENT_SOURCECMD, fname_exp, ::core::ptr::null_mut::<buf_T>())
                        as ::core::ffi::c_int
                        != 0
                        && apply_autocmds(
                            EVENT_SOURCECMD,
                            fname_exp,
                            fname_exp,
                            false_0 != 0,
                            curbuf.get(),
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        retval = if aborting() as ::core::ffi::c_int != 0 {
                            FAIL
                        } else {
                            OK
                        };
                        if retval == OK {
                            apply_autocmds(
                                EVENT_SOURCEPOST,
                                fname_exp,
                                fname_exp,
                                false_0 != 0,
                                curbuf.get(),
                            );
                        }
                        break '_theend;
                    } else {
                        apply_autocmds(
                            EVENT_SOURCEPRE,
                            fname_exp,
                            fname_exp,
                            false_0 != 0,
                            curbuf.get(),
                        );
                    }
                }
                if !cookie.source_from_buf_or_str {
                    cookie.fp = fopen_noinh_readbin(fname_exp);
                }
                if cookie.fp.is_null() && check_other as ::core::ffi::c_int != 0 {
                    let mut p_0: *mut ::core::ffi::c_char = path_tail(fname_exp);
                    if (*p_0 as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                        || *p_0 as ::core::ffi::c_int == '_' as ::core::ffi::c_int)
                        && (strcasecmp(
                            p_0.offset(1 as ::core::ffi::c_int as isize),
                            c"nvimrc".as_ptr() as *mut ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                            || strcasecmp(
                                p_0.offset(1 as ::core::ffi::c_int as isize),
                                c"exrc".as_ptr() as *mut ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int)
                    {
                        *p_0 = (if *p_0 as ::core::ffi::c_int == '_' as ::core::ffi::c_int {
                            '.' as ::core::ffi::c_int
                        } else {
                            '_' as ::core::ffi::c_int
                        }) as ::core::ffi::c_char;
                        cookie.fp = fopen_noinh_readbin(fname_exp);
                    }
                }
                if cookie.fp.is_null() && !cookie.source_from_buf_or_str {
                    if p_verbose.get() > 1 as OptInt {
                        verbose_enter();
                        if (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_name
                        .is_null()
                        {
                            smsg_c!(
                                0 as ::core::ffi::c_int,
                                gettext(c"could not source \"%s\"".as_ptr()),
                                fname,
                            );
                        } else {
                            smsg_c!(
                                0 as ::core::ffi::c_int,
                                gettext(c"line %ld: could not source \"%s\"".as_ptr()),
                                (*((*exestack.ptr()).ga_data as *mut estack_T).offset(
                                    ((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize,
                                ))
                                .es_lnum as int64_t,
                                fname,
                            );
                        }
                        verbose_leave();
                    }
                } else {
                    if p_verbose.get() > 1 as OptInt {
                        verbose_enter();
                        if (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_name
                        .is_null()
                        {
                            smsg_c!(
                                0 as ::core::ffi::c_int,
                                gettext(c"sourcing \"%s\"".as_ptr()),
                                fname,
                            );
                        } else {
                            smsg_c!(
                                0 as ::core::ffi::c_int,
                                gettext(c"line %ld: sourcing \"%s\"".as_ptr()),
                                (*((*exestack.ptr()).ga_data as *mut estack_T).offset(
                                    ((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize,
                                ))
                                .es_lnum as int64_t,
                                fname,
                            );
                        }
                        verbose_leave();
                    }
                    if is_vimrc == DOSO_VIMRC {
                        vimrc_found(fname_exp, c"MYVIMRC".as_ptr() as *mut ::core::ffi::c_char);
                    }
                    cookie.breakpoint = dbg_find_breakpoint(true_0 != 0, fname_exp, 0 as linenr_T);
                    cookie.fname = fname_exp;
                    cookie.dbg_tick = debug_tick.get();
                    cookie.level = ex_nesting_level.get();
                    rel_time = 0;
                    start_time = 0;
                    l_time_fd = time_fd.get();
                    if !l_time_fd.is_null() {
                        (rel_time, start_time) = time_push();
                    }
                    l_do_profiling = do_profiling.get();
                    if l_do_profiling == PROF_YES {
                        wait_start = prof_child_enter();
                    }
                    funccalp_entry = funccal_entry_T {
                        top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        next: ::core::ptr::null_mut::<funccal_entry_T>(),
                    };
                    save_funccal(&raw mut funccalp_entry);
                    save_current_sctx = current_sctx.get();
                    (*last_current_SID_seq.ptr()) += 1;
                    (*current_sctx.ptr()).sc_seq = last_current_SID_seq.get();
                    if sid > 0 as ::core::ffi::c_int {
                        si = *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
                            .offset((sid - 1 as ::core::ffi::c_int) as isize);
                    } else if str.is_null() {
                        si = new_script_item(fname_exp, &raw mut sid);
                        (*si).sn_lua = path_with_extension(fname_exp, c"lua".as_ptr());
                        fname_exp = xstrdup((*si).sn_name);
                        if !ret_sid.is_null() {
                            *ret_sid = sid;
                        }
                    }
                    debug_assert!(
                        !si.is_null() as ::core::ffi::c_int == str.is_null() as ::core::ffi::c_int,
                        "(si != NULL) == (str == NULL)"
                    );
                    if str.is_null() || !script_is_lua((*current_sctx.ptr()).sc_sid) {
                        (*current_sctx.ptr()).sc_sid = sid as scid_T;
                        (*current_sctx.ptr()).sc_lnum = 0 as ::core::ffi::c_int as linenr_T;
                    }
                    estack_push(
                        ETYPE_SCRIPT,
                        if !si.is_null() {
                            (*si).sn_name
                        } else {
                            fname_exp
                        },
                        0 as linenr_T,
                    );
                    if l_do_profiling == PROF_YES && !si.is_null() {
                        let mut forceit: bool = false_0 != 0;
                        if !(*si).sn_prof_on
                            && has_profiling(true_0 != 0, (*si).sn_name, &raw mut forceit)
                                as ::core::ffi::c_int
                                != 0
                        {
                            profile_init(si);
                            (*si).sn_pr_force = forceit;
                        }
                        if (*si).sn_prof_on {
                            (*si).sn_pr_count += 1;
                            (*si).sn_pr_start = profile_start();
                            (*si).sn_pr_children = profile_zero();
                        }
                    }
                    cookie.conv.vc_type = CONV_NONE;
                    ts_lua = false_0 != 0;
                    if fname.is_null()
                        && !eap.is_null()
                        && !ex_lua
                        && !strequal((*curbuf.get()).b_p_ft, c"lua".as_ptr())
                        && !(!(*curbuf.get()).b_fname.is_null()
                            && path_with_extension((*curbuf.get()).b_fname, c"lua".as_ptr())
                                as ::core::ffi::c_int
                                != 0)
                    {
                        let mut args: Array = ARRAY_DICT_INIT;
                        let mut args__items: [Object; 3] = [Object {
                            type_0: kObjectTypeNil,
                            data: object_data { boolean: false },
                        }; 3];
                        args.capacity = 3 as size_t;
                        args.items = &raw mut args__items as *mut Object;
                        let c2rust_fresh0 = args.size;
                        args.size = args.size.wrapping_add(1);
                        *args.items.add(c2rust_fresh0) = object {
                            type_0: kObjectTypeInteger,
                            data: object_data {
                                integer: (*curbuf.get()).handle as Integer,
                            },
                        };
                        let c2rust_fresh1 = args.size;
                        args.size = args.size.wrapping_add(1);
                        *args.items.add(c2rust_fresh1) = object {
                            type_0: kObjectTypeInteger,
                            data: object_data {
                                integer: (*eap).line1 as Integer,
                            },
                        };
                        let c2rust_fresh2 = args.size;
                        args.size = args.size.wrapping_add(1);
                        *args.items.add(c2rust_fresh2) = object {
                            type_0: kObjectTypeInteger,
                            data: object_data {
                                integer: (*eap).line2 as Integer,
                            },
                        };
                        let mut err: Error = Error {
                            type_0: kErrorTypeNone,
                            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        };
                        let mut result: Object = nlua_exec(
                            String_0 {
                                data: c"return require('vim._core.util').source_is_lua(...)"
                                    .as_ptr()
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 52]>()
                                    .wrapping_sub(1 as size_t),
                            },
                            ::core::ptr::null::<::core::ffi::c_char>(),
                            args,
                            kRetNilBool,
                            ::core::ptr::null_mut::<Arena>(),
                            &raw mut err,
                        );
                        if !(err.type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int)
                            && (result.type_0 as ::core::ffi::c_uint
                                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                                && result.data.boolean as ::core::ffi::c_int == true_0)
                        {
                            ts_lua = true_0 != 0;
                        }
                        api_clear_error(&raw mut err);
                    }
                    if fname.is_null()
                        && (ex_lua as ::core::ffi::c_int != 0
                            || ts_lua as ::core::ffi::c_int != 0
                            || strequal((*curbuf.get()).b_p_ft, c"lua".as_ptr())
                                as ::core::ffi::c_int
                                != 0
                            || !(*curbuf.get()).b_fname.is_null()
                                && path_with_extension((*curbuf.get()).b_fname, c"lua".as_ptr())
                                    as ::core::ffi::c_int
                                    != 0)
                    {
                        nlua_exec_ga(&raw mut cookie.buflines, fname_exp);
                    } else if !si.is_null() && (*si).sn_lua as ::core::ffi::c_int != 0 {
                        nlua_exec_file(fname_exp);
                    } else {
                        firstline = getsourceline(
                            0 as ::core::ffi::c_int,
                            &raw mut cookie as *mut ::core::ffi::c_void,
                            0 as ::core::ffi::c_int,
                            true_0 != 0,
                        ) as *mut uint8_t;
                        if !firstline.is_null()
                            && strlen(firstline as *mut ::core::ffi::c_char) >= 3 as size_t
                            && *firstline.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == 0xef as ::core::ffi::c_int
                            && *firstline.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == 0xbb as ::core::ffi::c_int
                            && *firstline.offset(2 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == 0xbf as ::core::ffi::c_int
                        {
                            convert_setup(
                                &raw mut cookie.conv,
                                c"utf-8".as_ptr() as *mut ::core::ffi::c_char,
                                p_enc.get(),
                            );
                            let mut p_1: *mut ::core::ffi::c_char = string_convert(
                                &raw mut cookie.conv,
                                (firstline as *mut ::core::ffi::c_char)
                                    .offset(3 as ::core::ffi::c_int as isize),
                                ::core::ptr::null_mut::<size_t>(),
                            );
                            if p_1.is_null() {
                                p_1 = xstrdup(
                                    (firstline as *mut ::core::ffi::c_char)
                                        .offset(3 as ::core::ffi::c_int as isize),
                                );
                            }
                            xfree(firstline as *mut ::core::ffi::c_void);
                            firstline = p_1 as *mut uint8_t;
                        }
                        do_cmdline(
                            firstline as *mut ::core::ffi::c_char,
                            Some(
                                getsourceline
                                    as unsafe extern "C" fn(
                                        ::core::ffi::c_int,
                                        *mut ::core::ffi::c_void,
                                        ::core::ffi::c_int,
                                        bool,
                                    )
                                        -> *mut ::core::ffi::c_char,
                            ),
                            &raw mut cookie as *mut ::core::ffi::c_void,
                            DOCMD_VERBOSE | DOCMD_NOWAIT | DOCMD_REPEAT,
                        );
                    }
                    retval = OK;
                    if l_do_profiling == PROF_YES && !si.is_null() {
                        si = *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T).offset(
                            ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int) as isize,
                        );
                        if (*si).sn_prof_on {
                            (*si).sn_pr_start = profile_end((*si).sn_pr_start);
                            (*si).sn_pr_start = profile_sub_wait(wait_start, (*si).sn_pr_start);
                            (*si).sn_pr_total = profile_add((*si).sn_pr_total, (*si).sn_pr_start);
                            (*si).sn_pr_self = profile_self(
                                (*si).sn_pr_self,
                                (*si).sn_pr_start,
                                (*si).sn_pr_children,
                            );
                        }
                    }
                    if got_int.get() {
                        emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
                    }
                    estack_pop();
                    if p_verbose.get() > 1 as OptInt {
                        verbose_enter();
                        smsg_c!(
                            0 as ::core::ffi::c_int,
                            gettext(c"finished sourcing %s".as_ptr()),
                            fname,
                        );
                        if !(*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_name
                        .is_null()
                        {
                            smsg_c!(
                                0 as ::core::ffi::c_int,
                                gettext(c"continuing in %s".as_ptr()),
                                (*((*exestack.ptr()).ga_data as *mut estack_T).offset(
                                    ((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize,
                                ))
                                .es_name,
                            );
                        }
                        verbose_leave();
                    }
                    if !l_time_fd.is_null() {
                        vim_snprintf(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            IOSIZE as size_t,
                            c"sourcing %s".as_ptr(),
                            fname,
                        );
                        time_msg(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            &raw mut start_time,
                        );
                        time_pop(rel_time);
                    }
                    if !got_int.get() {
                        trigger_source_post = true_0 != 0;
                    }
                    if save_debug_break_level > ex_nesting_level.get()
                        && debug_break_level.get() == ex_nesting_level.get()
                    {
                        (*debug_break_level.ptr()) += 1;
                    }
                    current_sctx.set(save_current_sctx);
                    restore_funccal();
                    if l_do_profiling == PROF_YES {
                        prof_child_exit(wait_start);
                    }
                    if !cookie.fp.is_null() {
                        fclose(cookie.fp);
                    }
                    if cookie.source_from_buf_or_str {
                        ga_clear_strings(&raw mut cookie.buflines);
                    }
                    xfree(cookie.nextline as *mut ::core::ffi::c_void);
                    xfree(firstline as *mut ::core::ffi::c_void);
                    convert_setup(
                        &raw mut cookie.conv,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if str.is_null() && trigger_source_post as ::core::ffi::c_int != 0 {
                        apply_autocmds(
                            EVENT_SOURCEPOST,
                            fname_exp,
                            fname_exp,
                            false_0 != 0,
                            curbuf.get(),
                        );
                    }
                }
            }
        }
        xfree(fname_exp as *mut ::core::ffi::c_void);
        return retval;
    }
}

pub unsafe extern "C" fn do_source(
    mut fname: *mut ::core::ffi::c_char,
    mut check_other: bool,
    mut is_vimrc: ::core::ffi::c_int,
    mut ret_sid: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return do_source_ext(
            fname,
            check_other,
            is_vimrc,
            ret_sid,
            ::core::ptr::null::<exarg_T>(),
            false_0 != 0,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
    }
}
