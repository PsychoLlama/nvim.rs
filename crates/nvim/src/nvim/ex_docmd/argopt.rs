//! Arguments that are not file names: `++opt=value`, `+cmd`, the
//! tab page argument, and the file the command will write to.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn getargcmd(mut argp: *mut *mut c_char) -> *mut c_char {
    let mut arg: *mut c_char = *argp;
    let mut command: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if *arg as c_int == '+' as c_int {
        arg = arg.offset(1);
        if ascii_isspace(*arg as c_int) as c_int != 0 || *arg as c_int == NUL {
            command = dollar_command.ptr() as *mut c_char;
        } else {
            command = arg;
            arg = skip_cmd_arg(command, true_0 != 0);
            if *arg as c_int != NUL {
                let c2rust_fresh26 = arg;
                arg = arg.offset(1);
                *c2rust_fresh26 = NUL as c_char;
            }
        }
        arg = skipwhite(arg);
        *argp = arg;
    }
    return command;
}

pub unsafe extern "C" fn get_bad_opt(mut p: *const c_char, mut eap: *mut exarg_T) -> c_int {
    if strcasecmp(
        p as *mut c_char,
        b"keep\0".as_ptr() as *const c_char as *mut c_char,
    ) == 0 as c_int
    {
        (*eap).bad_char = BAD_KEEP;
    } else if strcasecmp(
        p as *mut c_char,
        b"drop\0".as_ptr() as *const c_char as *mut c_char,
    ) == 0 as c_int
    {
        (*eap).bad_char = BAD_DROP;
    } else if (*utf8len_tab.ptr())[*p as uint8_t as usize] as c_int == 1 as c_int
        && *p.offset(1 as c_int as isize) as c_int == NUL
    {
        (*eap).bad_char = *p as uint8_t as c_int;
    } else {
        return FAIL;
    }
    return OK;
}

pub(crate) unsafe extern "C" fn get_bad_name(
    mut _xp: *mut expand_T,
    mut idx: c_int,
) -> *mut c_char {
    static p_bad_values: GlobalCell<[*mut c_char; 3]> = GlobalCell::new([
        b"?\0".as_ptr() as *const c_char as *mut c_char,
        b"keep\0".as_ptr() as *const c_char as *mut c_char,
        b"drop\0".as_ptr() as *const c_char as *mut c_char,
    ]);
    if idx
        < ::core::mem::size_of::<[*mut c_char; 3]>()
            .wrapping_div(::core::mem::size_of::<*mut c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut c_char; 3]>()
                    .wrapping_rem(::core::mem::size_of::<*mut c_char>())
                    == 0) as c_int as usize,
            ) as c_int
    {
        return (*p_bad_values.ptr())[idx as usize] as *mut c_char;
    }
    return ::core::ptr::null_mut::<c_char>();
}

pub unsafe extern "C" fn getargopt(mut eap: *mut exarg_T) -> c_int {
    let mut arg: *mut c_char = (*eap).arg.offset(2 as c_int as isize);
    let mut pp: *mut c_int = ::core::ptr::null_mut::<c_int>();
    let mut bad_char_idx: c_int = 0;
    if strncmp(arg, b"bin\0".as_ptr() as *const c_char, 3 as size_t) == 0 as c_int
        || strncmp(arg, b"nobin\0".as_ptr() as *const c_char, 5 as size_t) == 0 as c_int
    {
        if *arg as c_int == 'n' as c_int {
            arg = arg.offset(2 as c_int as isize);
            (*eap).force_bin = FORCE_NOBIN;
        } else {
            (*eap).force_bin = FORCE_BIN;
        }
        if !checkforcmd(
            &raw mut arg,
            b"binary\0".as_ptr() as *const c_char,
            3 as c_int,
        ) {
            return FAIL;
        }
        (*eap).arg = skipwhite(arg);
        return OK;
    }
    if strncmp(arg, b"edit\0".as_ptr() as *const c_char, 4 as size_t) == 0 as c_int
        && !(*arg.offset(4 as c_int as isize) as c_uint >= 'A' as c_uint
            && *arg.offset(4 as c_int as isize) as c_uint <= 'Z' as c_uint
            || *arg.offset(4 as c_int as isize) as c_uint >= 'a' as c_uint
                && *arg.offset(4 as c_int as isize) as c_uint <= 'z' as c_uint)
    {
        (*eap).read_edit = true_0;
        (*eap).arg = skipwhite(arg.offset(4 as c_int as isize));
        return OK;
    }
    if *arg.offset(0 as c_int as isize) as c_int == 'p' as c_int
        && !(*arg.offset(1 as c_int as isize) as c_uint >= 'A' as c_uint
            && *arg.offset(1 as c_int as isize) as c_uint <= 'Z' as c_uint
            || *arg.offset(1 as c_int as isize) as c_uint >= 'a' as c_uint
                && *arg.offset(1 as c_int as isize) as c_uint <= 'z' as c_uint)
    {
        (*eap).mkdir_p = true_0;
        (*eap).arg = skipwhite(arg.offset(1 as c_int as isize));
        return OK;
    }
    if strncmp(arg, b"ff\0".as_ptr() as *const c_char, 2 as size_t) == 0 as c_int {
        arg = arg.offset(2 as c_int as isize);
        pp = &raw mut (*eap).force_ff;
    } else if strncmp(arg, b"fileformat\0".as_ptr() as *const c_char, 10 as size_t) == 0 as c_int {
        arg = arg.offset(10 as c_int as isize);
        pp = &raw mut (*eap).force_ff;
    } else if strncmp(arg, b"enc\0".as_ptr() as *const c_char, 3 as size_t) == 0 as c_int {
        if strncmp(arg, b"encoding\0".as_ptr() as *const c_char, 8 as size_t) == 0 as c_int {
            arg = arg.offset(8 as c_int as isize);
        } else {
            arg = arg.offset(3 as c_int as isize);
        }
        pp = &raw mut (*eap).force_enc;
    } else if strncmp(arg, b"bad\0".as_ptr() as *const c_char, 3 as size_t) == 0 as c_int {
        arg = arg.offset(3 as c_int as isize);
        pp = &raw mut bad_char_idx;
    }
    if pp.is_null() || *arg as c_int != '=' as c_int {
        return FAIL;
    }
    arg = arg.offset(1);
    *pp = arg.offset_from((*eap).cmd) as c_int;
    arg = skip_cmd_arg(arg, false_0 != 0);
    (*eap).arg = skipwhite(arg);
    *arg = NUL as c_char;
    if pp == &raw mut (*eap).force_ff {
        if check_ff_value((*eap).cmd.offset((*eap).force_ff as isize)) == FAIL {
            return FAIL;
        }
        (*eap).force_ff = *(*eap).cmd.offset((*eap).force_ff as isize) as uint8_t as c_int;
    } else if pp == &raw mut (*eap).force_enc {
        let mut p: *mut c_char = (*eap).cmd.offset((*eap).force_enc as isize);
        while *p as c_int != NUL {
            *p = (if (*p as c_int) < 'A' as c_int || *p as c_int > 'Z' as c_int {
                *p as c_int
            } else {
                *p as c_int + ('a' as c_int - 'A' as c_int)
            }) as c_char;
            p = p.offset(1);
        }
    } else if get_bad_opt((*eap).cmd.offset(bad_char_idx as isize), eap) == FAIL {
        return FAIL;
    }
    return OK;
}

pub(crate) unsafe extern "C" fn get_argopt_name(
    mut _xp: *mut expand_T,
    mut idx: c_int,
) -> *mut c_char {
    static p_opt_values: GlobalCell<[*mut c_char; 7]> = GlobalCell::new([
        b"fileformat=\0".as_ptr() as *const c_char as *mut c_char,
        b"encoding=\0".as_ptr() as *const c_char as *mut c_char,
        b"binary\0".as_ptr() as *const c_char as *mut c_char,
        b"nobinary\0".as_ptr() as *const c_char as *mut c_char,
        b"bad=\0".as_ptr() as *const c_char as *mut c_char,
        b"edit\0".as_ptr() as *const c_char as *mut c_char,
        b"p\0".as_ptr() as *const c_char as *mut c_char,
    ]);
    if idx
        < ::core::mem::size_of::<[*mut c_char; 7]>()
            .wrapping_div(::core::mem::size_of::<*mut c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut c_char; 7]>()
                    .wrapping_rem(::core::mem::size_of::<*mut c_char>())
                    == 0) as c_int as usize,
            ) as c_int
    {
        return (*p_opt_values.ptr())[idx as usize] as *mut c_char;
    }
    return ::core::ptr::null_mut::<c_char>();
}

pub unsafe extern "C" fn expand_argopt(
    mut pat: *mut c_char,
    mut xp: *mut expand_T,
    mut rmp: *mut regmatch_T,
    mut matches: *mut *mut *mut c_char,
    mut numMatches: *mut c_int,
) -> c_int {
    if (*xp).xp_pattern > (*xp).xp_line
        && *(*xp).xp_pattern.offset(-(1 as c_int as isize)) as c_int == '=' as c_int
    {
        let mut cb: CompleteListItemGetter = None;
        let mut name_end: *mut c_char = (*xp).xp_pattern.offset(-(1 as c_int as isize));
        if name_end.offset_from((*xp).xp_line) >= 2 as isize
            && strncmp(
                name_end.offset(-(2 as c_int as isize)),
                b"ff\0".as_ptr() as *const c_char,
                2 as size_t,
            ) == 0 as c_int
        {
            cb = Some(
                get_fileformat_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char,
            ) as CompleteListItemGetter;
        } else if name_end.offset_from((*xp).xp_line) >= 10 as isize
            && strncmp(
                name_end.offset(-(10 as c_int as isize)),
                b"fileformat\0".as_ptr() as *const c_char,
                10 as size_t,
            ) == 0 as c_int
        {
            cb = Some(
                get_fileformat_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char,
            ) as CompleteListItemGetter;
        } else if name_end.offset_from((*xp).xp_line) >= 3 as isize
            && strncmp(
                name_end.offset(-(3 as c_int as isize)),
                b"enc\0".as_ptr() as *const c_char,
                3 as size_t,
            ) == 0 as c_int
        {
            cb = Some(
                get_encoding_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char,
            ) as CompleteListItemGetter;
        } else if name_end.offset_from((*xp).xp_line) >= 8 as isize
            && strncmp(
                name_end.offset(-(8 as c_int as isize)),
                b"encoding\0".as_ptr() as *const c_char,
                8 as size_t,
            ) == 0 as c_int
        {
            cb = Some(
                get_encoding_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char,
            ) as CompleteListItemGetter;
        } else if name_end.offset_from((*xp).xp_line) >= 3 as isize
            && strncmp(
                name_end.offset(-(3 as c_int as isize)),
                b"bad\0".as_ptr() as *const c_char,
                3 as size_t,
            ) == 0 as c_int
        {
            cb = Some(get_bad_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char)
                as CompleteListItemGetter;
        }
        if cb.is_some() {
            ExpandGeneric(pat, xp, rmp, matches, numMatches, cb, false_0 != 0);
            return OK;
        }
        return FAIL;
    }
    if (*xp).xp_pattern_len == 2 as size_t
        && strncmp(
            (*xp).xp_pattern,
            b"ff\0".as_ptr() as *const c_char,
            (*xp).xp_pattern_len,
        ) == 0 as c_int
    {
        *matches = xmalloc(::core::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
        *numMatches = 1 as c_int;
        *(*matches).offset(0 as c_int as isize) =
            xstrdup(b"fileformat=\0".as_ptr() as *const c_char);
        return OK;
    }
    ExpandGeneric(
        pat,
        xp,
        rmp,
        matches,
        numMatches,
        Some(get_argopt_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char),
        false_0 != 0,
    );
    return OK;
}

pub(crate) unsafe extern "C" fn get_tabpage_arg(mut eap: *mut exarg_T) -> c_int {
    let mut tab_number: c_int = 0 as c_int;
    let mut unaccept_arg0: c_int = if (*eap).cmdidx as c_int == CMD_tabmove as c_int {
        0 as c_int
    } else {
        1 as c_int
    };
    '_theend: {
        if !(*eap).arg.is_null() && *(*eap).arg as c_int != NUL {
            let mut p: *mut c_char = (*eap).arg;
            let mut relative: c_int = 0 as c_int;
            if *p as c_int == '-' as c_int {
                relative = -1 as c_int;
                p = p.offset(1);
            } else if *p as c_int == '+' as c_int {
                relative = 1 as c_int;
                p = p.offset(1);
            }
            let mut p_save: *mut c_char = p;
            tab_number = getdigits(&raw mut p, false_0 != 0, tab_number as intmax_t) as c_int;
            if relative == 0 as c_int {
                if strcmp(p, b"$\0".as_ptr() as *const c_char) == 0 as c_int {
                    tab_number = current_tab_nr(::core::ptr::null_mut::<tabpage_T>());
                } else if strcmp(p, b"#\0".as_ptr() as *const c_char) == 0 as c_int {
                    if valid_tabpage(lastused_tabpage.get()) {
                        tab_number = tabpage_index(lastused_tabpage.get());
                    } else {
                        (*eap).errmsg =
                            ex_errmsg(&raw const e_invargval as *const c_char, (*eap).arg);
                        tab_number = 0 as c_int;
                        break '_theend;
                    }
                } else if p == p_save
                    || *p_save as c_int == '-' as c_int
                    || *p as c_int != NUL
                    || tab_number > current_tab_nr(::core::ptr::null_mut::<tabpage_T>())
                {
                    (*eap).errmsg = ex_errmsg(&raw const e_invarg2 as *const c_char, (*eap).arg);
                    break '_theend;
                }
            } else {
                if *p_save as c_int == NUL {
                    tab_number = 1 as c_int;
                } else if p == p_save
                    || *p_save as c_int == '-' as c_int
                    || *p as c_int != NUL
                    || tab_number == 0 as c_int
                {
                    (*eap).errmsg = ex_errmsg(&raw const e_invarg2 as *const c_char, (*eap).arg);
                    break '_theend;
                }
                tab_number = tab_number * relative + tabpage_index(curtab.get());
                if unaccept_arg0 == 0 && relative == -1 as c_int {
                    tab_number -= 1;
                }
            }
            if tab_number < unaccept_arg0
                || tab_number > current_tab_nr(::core::ptr::null_mut::<tabpage_T>())
            {
                (*eap).errmsg = ex_errmsg(&raw const e_invarg2 as *const c_char, (*eap).arg);
            }
        } else if (*eap).addr_count > 0 as c_int {
            if unaccept_arg0 != 0 && (*eap).line2 == 0 as linenr_T {
                (*eap).errmsg = gettext(&raw const e_invrange as *const c_char);
                tab_number = 0 as c_int;
            } else {
                tab_number = (*eap).line2 as c_int;
                if unaccept_arg0 == 0 {
                    let mut cmdp: *mut c_char = (*eap).cmd;
                    loop {
                        cmdp = cmdp.offset(-1);
                        if !(cmdp > *(*eap).cmdlinep
                            && (ascii_iswhite(*cmdp as c_int) as c_int != 0
                                || ascii_isdigit(*cmdp as c_int) as c_int != 0))
                        {
                            break;
                        }
                    }
                    if *cmdp as c_int == '-' as c_int {
                        tab_number -= 1;
                        if tab_number < unaccept_arg0 {
                            (*eap).errmsg = gettext(&raw const e_invrange as *const c_char);
                        }
                    }
                }
            }
        } else {
            match (*eap).cmdidx as c_int {
                461 => {
                    tab_number = tabpage_index(curtab.get()) + 1 as c_int;
                    if tab_number > current_tab_nr(::core::ptr::null_mut::<tabpage_T>()) {
                        tab_number = 1 as c_int;
                    }
                }
                459 => {
                    tab_number = current_tab_nr(::core::ptr::null_mut::<tabpage_T>());
                }
                _ => {
                    tab_number = tabpage_index(curtab.get());
                }
            }
        }
    }
    return tab_number;
}

pub(crate) unsafe extern "C" fn check_more(mut message: bool, mut forceit: bool) -> c_int {
    let mut n: c_int =
        (*(*curwin.get()).w_alist).al_ga.ga_len - (*curwin.get()).w_arg_idx - 1 as c_int;
    if !forceit
        && only_one_window() as c_int != 0
        && (*(*curwin.get()).w_alist).al_ga.ga_len > 1 as c_int
        && !arg_had_last.get()
        && n > 0 as c_int
        && quitmore.get() == 0 as c_int
    {
        if message {
            if (p_confirm.get() != 0 || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as c_int != 0)
                && !(*curbuf.get()).b_fname.is_null()
            {
                let mut buff: [c_char; 1000] = [0; 1000];
                vim_snprintf(
                    &raw mut buff as *mut c_char,
                    DIALOG_MSG_SIZE as c_int as size_t,
                    ngettext(
                        b"%d more file to edit.  Quit anyway?\0".as_ptr() as *const c_char,
                        b"%d more files to edit.  Quit anyway?\0".as_ptr() as *const c_char,
                        n as c_ulong,
                    ),
                    n,
                );
                if vim_dialog_yesno(
                    VIM_QUESTION as c_int,
                    ::core::ptr::null_mut::<c_char>(),
                    &raw mut buff as *mut c_char,
                    1 as c_int,
                ) == VIM_YES as c_int
                {
                    return OK;
                }
                return FAIL;
            }
            semsg(
                ngettext(
                    b"E173: %d more file to edit\0".as_ptr() as *const c_char,
                    b"E173: %d more files to edit\0".as_ptr() as *const c_char,
                    n as c_ulong,
                ),
                n,
            );
            quitmore.set(2 as c_int);
        }
        return FAIL;
    }
    return OK;
}

pub unsafe extern "C" fn vim_mkdir_emsg(name: *const c_char, prot: c_int) -> c_int {
    let mut ret: c_int = 0;
    ret = os_mkdir(name, prot as int32_t);
    if ret != 0 as c_int {
        semsg(
            gettext(&raw const e_mkdir as *const c_char),
            name,
            uv_strerror(ret),
        );
        return FAIL;
    }
    return OK;
}

pub unsafe extern "C" fn open_exfile(
    mut fname: *mut c_char,
    mut forceit: c_int,
    mut mode: *mut c_char,
) -> *mut FILE {
    if os_isdir(fname) {
        semsg(gettext(&raw const e_isadir2 as *const c_char), fname);
        return ::core::ptr::null_mut::<FILE>();
    }
    if forceit == 0 && *mode as c_int != 'a' as c_int && os_path_exists(fname) as c_int != 0 {
        semsg(
            gettext(b"E189: \"%s\" exists (add ! to override)\0".as_ptr() as *const c_char),
            fname,
        );
        return ::core::ptr::null_mut::<FILE>();
    }
    let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    fd = os_fopen(fname, mode);
    if fd.is_null() {
        semsg(
            gettext(b"E190: Cannot open \"%s\" for writing\0".as_ptr() as *const c_char),
            fname,
        );
    }
    return fd;
}

pub unsafe extern "C" fn dialog_msg(
    mut buff: *mut c_char,
    mut format: *mut c_char,
    mut fname: *mut c_char,
) {
    if fname.is_null() {
        fname = gettext(b"Untitled\0".as_ptr() as *const c_char);
    }
    vim_snprintf(buff, DIALOG_MSG_SIZE as c_int as size_t, format, fname);
}
