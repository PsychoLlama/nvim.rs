//! Expanding what a command's file argument stands for: `%`, `#`,
//! `<cfile>` and the rest of the `<…>` family, wildcards, and the backtick
//! form.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn replace_makeprg(
    mut eap: *mut exarg_T,
    mut arg: *mut c_char,
    mut cmdlinep: *mut *mut c_char,
) -> *mut c_char {
    let mut isgrep: bool = (*eap).cmdidx as c_int == CMD_grep as c_int
        || (*eap).cmdidx as c_int == CMD_lgrep as c_int
        || (*eap).cmdidx as c_int == CMD_grepadd as c_int
        || (*eap).cmdidx as c_int == CMD_lgrepadd as c_int;
    if ((*eap).cmdidx as c_int == CMD_make as c_int
        || (*eap).cmdidx as c_int == CMD_lmake as c_int
        || isgrep as c_int != 0)
        && grep_internal((*eap).cmdidx) == 0
    {
        let mut program: *const c_char = if isgrep as c_int != 0 {
            if *(*curbuf.get()).b_p_gp as c_int == NUL {
                p_gp.get()
            } else {
                (*curbuf.get()).b_p_gp
            }
        } else if *(*curbuf.get()).b_p_mp as c_int == NUL {
            p_mp.get()
        } else {
            (*curbuf.get()).b_p_mp
        };
        arg = skipwhite(arg);
        let mut new_cmdline: *mut c_char = ::core::ptr::null_mut::<c_char>();
        new_cmdline = strrep(program, b"$*\0".as_ptr() as *const c_char, arg);
        if new_cmdline.is_null() {
            new_cmdline = xmalloc(
                strlen(program)
                    .wrapping_add(strlen(arg))
                    .wrapping_add(2 as size_t),
            ) as *mut c_char;
            strcpy(new_cmdline, program as *mut c_char);
            strcat(new_cmdline, b" \0".as_ptr() as *const c_char);
            strcat(new_cmdline, arg);
        }
        msg_make(arg);
        xfree(*cmdlinep as *mut c_void);
        *cmdlinep = new_cmdline;
        arg = new_cmdline;
    }
    return arg;
}

pub unsafe extern "C" fn expand_filename(
    mut eap: *mut exarg_T,
    mut cmdlinep: *mut *mut c_char,
    mut errormsgp: *mut *const c_char,
) -> c_int {
    let mut p: *mut c_char = skip_grep_pat(eap);
    let mut has_wildcards: bool = path_has_wildcard(p);
    while *p as c_int != NUL {
        if *p.offset(0 as c_int as isize) as c_int == '`' as c_int
            && *p.offset(1 as c_int as isize) as c_int == '=' as c_int
        {
            p = p.offset(2 as c_int as isize);
            skip_expr(&raw mut p, ::core::ptr::null_mut::<evalarg_T>());
            if *p as c_int == '`' as c_int {
                p = p.offset(1);
            }
        } else if vim_strchr(b"%#<\0".as_ptr() as *const c_char, *p as uint8_t as c_int).is_null() {
            p = p.offset(1);
        } else {
            let mut srclen: size_t = 0;
            let mut escaped: c_int = 0;
            let mut repl: *mut c_char = eval_vars(
                p,
                (*eap).arg,
                &raw mut srclen,
                &raw mut (*eap).do_ecmd_lnum,
                errormsgp,
                &raw mut escaped,
                true_0 != 0,
            );
            if !(*errormsgp).is_null() {
                return FAIL;
            }
            if repl.is_null() {
                p = p.offset(srclen as isize);
            } else {
                if !vim_strchr(repl, '$' as c_int).is_null()
                    || !vim_strchr(repl, '~' as c_int).is_null()
                {
                    let mut l: *mut c_char = repl;
                    repl = expand_env_save(repl);
                    xfree(l as *mut c_void);
                }
                if (*eap).usefilter == 0
                    && escaped == 0
                    && (*eap).cmdidx as c_int != CMD_bang as c_int
                    && (*eap).cmdidx as c_int != CMD_grep as c_int
                    && (*eap).cmdidx as c_int != CMD_grepadd as c_int
                    && (*eap).cmdidx as c_int != CMD_lgrep as c_int
                    && (*eap).cmdidx as c_int != CMD_lgrepadd as c_int
                    && (*eap).cmdidx as c_int != CMD_lmake as c_int
                    && (*eap).cmdidx as c_int != CMD_make as c_int
                    && (*eap).cmdidx as c_int != CMD_terminal as c_int
                    && (*eap).argt & EX_NOSPC as uint32_t == 0
                {
                    let mut l_0: *mut c_char = ::core::ptr::null_mut::<c_char>();
                    l_0 = repl;
                    while *l_0 != 0 {
                        if !vim_strchr(escape_chars.get(), *l_0 as uint8_t as c_int).is_null() {
                            l_0 = vim_strsave_escaped(repl, escape_chars.get());
                            xfree(repl as *mut c_void);
                            repl = l_0;
                            break;
                        } else {
                            l_0 = l_0.offset(1);
                        }
                    }
                }
                if ((*eap).usefilter != 0
                    || (*eap).cmdidx as c_int == CMD_bang as c_int
                    || (*eap).cmdidx as c_int == CMD_terminal as c_int)
                    && !strpbrk(repl, b"!\0".as_ptr() as *const c_char).is_null()
                {
                    let mut l_1: *mut c_char =
                        vim_strsave_escaped(repl, b"!\0".as_ptr() as *const c_char);
                    xfree(repl as *mut c_void);
                    repl = l_1;
                }
                p = repl_cmdline(eap, p, srclen, repl, cmdlinep);
                xfree(repl as *mut c_void);
            }
        }
    }
    if (*eap).argt & EX_NOSPC as uint32_t != 0 && (*eap).usefilter == 0 {
        if has_wildcards {
            if !vim_strchr((*eap).arg, '$' as c_int).is_null()
                || !vim_strchr((*eap).arg, '~' as c_int).is_null()
            {
                expand_env_esc(
                    (*eap).arg,
                    NameBuff.ptr() as *mut c_char,
                    MAXPATHL,
                    true_0 != 0,
                    true_0 != 0,
                    ::core::ptr::null_mut::<c_char>(),
                );
                has_wildcards = path_has_wildcard(NameBuff.ptr() as *mut c_char);
                p = NameBuff.ptr() as *mut c_char;
            } else {
                p = ::core::ptr::null_mut::<c_char>();
            }
            if !p.is_null() {
                repl_cmdline(eap, (*eap).arg, strlen((*eap).arg), p, cmdlinep);
            }
        }
        if !has_wildcards {
            backslash_halve((*eap).arg);
        }
        if has_wildcards {
            let mut xpc: expand_T = expand_T {
                xp_pattern: ::core::ptr::null_mut::<c_char>(),
                xp_context: 0,
                xp_pattern_len: 0,
                xp_prefix: XP_PREFIX_NONE,
                xp_arg: ::core::ptr::null_mut::<c_char>(),
                xp_luaref: 0,
                xp_script_ctx: sctx_T {
                    sc_sid: 0,
                    sc_seq: 0,
                    sc_lnum: 0,
                    sc_chan: 0,
                },
                xp_backslash: 0,
                xp_shell: false,
                xp_numfiles: 0,
                xp_col: 0,
                xp_selected: 0,
                xp_orig: ::core::ptr::null_mut::<c_char>(),
                xp_files: ::core::ptr::null_mut::<*mut c_char>(),
                xp_line: ::core::ptr::null_mut::<c_char>(),
                xp_buf: [0; 256],
                xp_search_dir: kDirectionNotSet,
                xp_pre_incsearch_pos: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
            };
            let mut options: c_int =
                WILD_LIST_NOTFOUND as c_int | WILD_NOERROR as c_int | WILD_ADD_SLASH as c_int;
            ExpandInit(&raw mut xpc);
            xpc.xp_context = EXPAND_FILES as c_int;
            if p_wic.get() != 0 {
                options += WILD_ICASE as c_int;
            }
            p = ExpandOne(
                &raw mut xpc,
                (*eap).arg,
                ::core::ptr::null_mut::<c_char>(),
                options,
                WILD_EXPAND_FREE as c_int,
            );
            if p.is_null() {
                return FAIL;
            }
            repl_cmdline(eap, (*eap).arg, strlen((*eap).arg), p, cmdlinep);
            xfree(p as *mut c_void);
        }
    }
    return OK;
}

pub(crate) unsafe extern "C" fn repl_cmdline(
    mut eap: *mut exarg_T,
    mut src: *mut c_char,
    mut srclen: size_t,
    mut repl: *mut c_char,
    mut cmdlinep: *mut *mut c_char,
) -> *mut c_char {
    let mut len: size_t = strlen(repl);
    let mut i: size_t = (src.offset_from(*cmdlinep) as size_t)
        .wrapping_add(strlen(src.offset(srclen as isize)))
        .wrapping_add(len)
        .wrapping_add(3 as size_t);
    if !(*eap).nextcmd.is_null() {
        i = i.wrapping_add(strlen((*eap).nextcmd));
    }
    let mut new_cmdline: *mut c_char = xmalloc(i) as *mut c_char;
    let mut offset: size_t = src.offset_from(*cmdlinep) as size_t;
    i = offset;
    memmove(new_cmdline as *mut c_void, *cmdlinep as *const c_void, i);
    memmove(
        new_cmdline.offset(i as isize) as *mut c_void,
        repl as *const c_void,
        len,
    );
    i = i.wrapping_add(len);
    strcpy(new_cmdline.offset(i as isize), src.offset(srclen as isize));
    src = new_cmdline.offset(i as isize);
    if !(*eap).nextcmd.is_null() {
        i = strlen(new_cmdline).wrapping_add(1 as size_t);
        strcpy(new_cmdline.offset(i as isize), (*eap).nextcmd);
        (*eap).nextcmd = new_cmdline.offset(i as isize);
    }
    (*eap).cmd = new_cmdline.offset((*eap).cmd.offset_from(*cmdlinep) as isize);
    (*eap).arg = new_cmdline.offset((*eap).arg.offset_from(*cmdlinep) as isize);
    let mut j: size_t = 0 as size_t;
    while j < (*eap).argc {
        if offset >= (*(*eap).args.offset(j as isize)).offset_from(*cmdlinep) as size_t {
            *(*eap).args.offset(j as isize) = new_cmdline
                .offset((*(*eap).args.offset(j as isize)).offset_from(*cmdlinep) as isize);
        } else {
            *(*eap).args.offset(j as isize) = new_cmdline.offset(
                ((*(*eap).args.offset(j as isize)).offset_from(*cmdlinep)
                    + len.wrapping_sub(srclen) as isize) as isize,
            );
        }
        j = j.wrapping_add(1);
    }
    if !(*eap).do_ecmd_cmd.is_null() && (*eap).do_ecmd_cmd != dollar_command.ptr() as *mut c_char {
        (*eap).do_ecmd_cmd = new_cmdline.offset((*eap).do_ecmd_cmd.offset_from(*cmdlinep) as isize);
    }
    xfree(*cmdlinep as *mut c_void);
    *cmdlinep = new_cmdline;
    return src;
}

pub unsafe extern "C" fn find_cmdline_var(
    mut src: *const c_char,
    mut usedlen: *mut size_t,
) -> ssize_t {
    static spec_str: GlobalCell<[*mut c_char; 15]> = GlobalCell::new([
        b"%\0".as_ptr() as *const c_char as *mut c_char,
        b"#\0".as_ptr() as *const c_char as *mut c_char,
        b"<cword>\0".as_ptr() as *const c_char as *mut c_char,
        b"<cWORD>\0".as_ptr() as *const c_char as *mut c_char,
        b"<cexpr>\0".as_ptr() as *const c_char as *mut c_char,
        b"<cfile>\0".as_ptr() as *const c_char as *mut c_char,
        b"<sfile>\0".as_ptr() as *const c_char as *mut c_char,
        b"<slnum>\0".as_ptr() as *const c_char as *mut c_char,
        b"<stack>\0".as_ptr() as *const c_char as *mut c_char,
        b"<script>\0".as_ptr() as *const c_char as *mut c_char,
        b"<afile>\0".as_ptr() as *const c_char as *mut c_char,
        b"<abuf>\0".as_ptr() as *const c_char as *mut c_char,
        b"<amatch>\0".as_ptr() as *const c_char as *mut c_char,
        b"<sflnum>\0".as_ptr() as *const c_char as *mut c_char,
        b"<SID>\0".as_ptr() as *const c_char as *mut c_char,
    ]);
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[*mut c_char; 15]>()
        .wrapping_div(::core::mem::size_of::<*mut c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[*mut c_char; 15]>()
                .wrapping_rem(::core::mem::size_of::<*mut c_char>())
                == 0) as c_int as usize,
        )
    {
        let mut len: size_t = strlen((*spec_str.ptr())[i as usize] as *const c_char);
        if strncmp(src, (*spec_str.ptr())[i as usize] as *const c_char, len) == 0 as c_int {
            *usedlen = len;
            '_c2rust_label: {
                if i <= 9223372036854775807 as c_long as size_t {
                } else {
                    __assert_fail(
                        b"i <= SSIZE_MAX\0".as_ptr() as *const c_char,
                        b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                        7692 as c_uint,
                        b"ssize_t find_cmdline_var(const char *, size_t *)\0".as_ptr()
                            as *const c_char,
                    );
                }
            };
            return i as ssize_t;
        }
        i = i.wrapping_add(1);
    }
    return -1 as ssize_t;
}

pub unsafe extern "C" fn eval_vars(
    mut src: *mut c_char,
    mut srcstart: *const c_char,
    mut usedlen: *mut size_t,
    mut lnump: *mut linenr_T,
    mut errormsg: *mut *const c_char,
    mut escaped: *mut c_int,
    mut empty_is_error: bool,
) -> *mut c_char {
    let mut result: *mut c_char = b"\0".as_ptr() as *const c_char as *mut c_char;
    let mut resultbuf: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut resultlen: size_t = 0;
    let mut valid: c_int = VALID_HEAD as c_int | VALID_PATH as c_int;
    let mut tilde_file: bool = false_0 != 0;
    let mut skip_mod: bool = false_0 != 0;
    let mut strbuf: [c_char; 30] = [0; 30];
    *errormsg = ::core::ptr::null::<c_char>();
    if !escaped.is_null() {
        *escaped = false_0;
    }
    let mut spec_idx: ssize_t = find_cmdline_var(src, usedlen);
    if spec_idx < 0 as ssize_t {
        *usedlen = 1 as size_t;
        return ::core::ptr::null_mut::<c_char>();
    }
    if src > srcstart as *mut c_char && *src.offset(-1 as c_int as isize) as c_int == '\\' as c_int
    {
        *usedlen = 0 as size_t;
        memmove(
            src.offset(-(1 as c_int as isize)) as *mut c_void,
            src as *const c_void,
            strlen(src).wrapping_add(1 as size_t),
        );
        return ::core::ptr::null_mut::<c_char>();
    }
    if spec_idx == SPEC_CWORD as c_int as ssize_t
        || spec_idx == SPEC_CCWORD as c_int as ssize_t
        || spec_idx == SPEC_CEXPR as c_int as ssize_t
    {
        resultlen = find_ident_under_cursor(
            &raw mut result,
            if spec_idx == SPEC_CWORD as c_int as ssize_t {
                FIND_IDENT as c_int | FIND_STRING as c_int
            } else if spec_idx == SPEC_CEXPR as c_int as ssize_t {
                FIND_IDENT as c_int | FIND_STRING as c_int | FIND_EVAL as c_int
            } else {
                FIND_STRING as c_int
            },
            ::core::ptr::null_mut::<c_int>(),
        );
        if resultlen == 0 as size_t {
            *errormsg = b"\0".as_ptr() as *const c_char;
            return ::core::ptr::null_mut::<c_char>();
        }
    } else {
        let mut s: *mut c_char = ::core::ptr::null_mut::<c_char>();
        let mut i: c_int = 0;
        match spec_idx {
            0 => {
                if (*curbuf.get()).b_fname.is_null() {
                    result = b"\0".as_ptr() as *const c_char as *mut c_char;
                    valid = 0 as c_int;
                } else {
                    result = (*curbuf.get()).b_fname;
                    tilde_file = strcmp(result, b"~\0".as_ptr() as *const c_char) == 0 as c_int;
                }
            }
            1 => {
                if *src.offset(1 as c_int as isize) as c_int == '#' as c_int {
                    result = arg_all();
                    resultbuf = result;
                    *usedlen = 2 as size_t;
                    if !escaped.is_null() {
                        *escaped = true_0;
                    }
                    skip_mod = true_0 != 0;
                } else {
                    s = src.offset(1 as c_int as isize);
                    if *s as c_int == '<' as c_int {
                        s = s.offset(1);
                    }
                    i = getdigits_int(&raw mut s, false_0 != 0, 0 as c_int);
                    if s == src.offset(2 as c_int as isize)
                        && *src.offset(1 as c_int as isize) as c_int == '-' as c_int
                    {
                        s = s.offset(-1);
                    }
                    *usedlen = s.offset_from(src) as size_t;
                    if *src.offset(1 as c_int as isize) as c_int == '<' as c_int && i != 0 as c_int
                    {
                        if *usedlen < 2 as size_t {
                            *usedlen = 1 as size_t;
                            return ::core::ptr::null_mut::<c_char>();
                        }
                        result = tv_list_find_str(get_vim_var_list(VV_OLDFILES), i - 1 as c_int)
                            as *mut c_char;
                        if result.is_null() {
                            *errormsg = b"\0".as_ptr() as *const c_char;
                            return ::core::ptr::null_mut::<c_char>();
                        }
                    } else {
                        if i == 0 as c_int
                            && *src.offset(1 as c_int as isize) as c_int == '<' as c_int
                            && *usedlen > 1 as size_t
                        {
                            *usedlen = 1 as size_t;
                        }
                        let mut buf: *mut buf_T = buflist_findnr(i);
                        if buf.is_null() {
                            *errormsg = gettext(
                                b"E194: No alternate file name to substitute for '#'\0".as_ptr()
                                    as *const c_char,
                            );
                            return ::core::ptr::null_mut::<c_char>();
                        }
                        if !lnump.is_null() {
                            *lnump = ECMD_LAST as c_int as linenr_T;
                        }
                        if (*buf).b_fname.is_null() {
                            result = b"\0".as_ptr() as *const c_char as *mut c_char;
                            valid = 0 as c_int;
                        } else {
                            result = (*buf).b_fname;
                            tilde_file =
                                strcmp(result, b"~\0".as_ptr() as *const c_char) == 0 as c_int;
                        }
                    }
                }
            }
            5 => {
                result = file_name_at_cursor(
                    FNAME_MESS as c_int | FNAME_HYP as c_int,
                    1 as c_int,
                    ::core::ptr::null_mut::<linenr_T>(),
                );
                if result.is_null() {
                    *errormsg = b"\0".as_ptr() as *const c_char;
                    return ::core::ptr::null_mut::<c_char>();
                }
                resultbuf = result;
            }
            10 => {
                if !(*autocmd_fname.ptr()).is_null() && !autocmd_fname_full.get() {
                    autocmd_fname_full.set(true_0 != 0);
                    result = FullName_save(autocmd_fname.get(), false_0 != 0);
                    xstrlcpy(autocmd_fname.get(), result, MAXPATHL as size_t);
                    xfree(result as *mut c_void);
                }
                result = autocmd_fname.get();
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_autocommand_file_name_to_substitute_for_afile.ptr() as *const _)
                            as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                result = path_try_shorten_fname(result);
            }
            11 => {
                if autocmd_bufnr.get() <= 0 as c_int {
                    *errormsg = gettext(
                        (e_no_autocommand_buffer_number_to_substitute_for_abuf.ptr() as *const _)
                            as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                snprintf(
                    &raw mut strbuf as *mut c_char,
                    ::core::mem::size_of::<[c_char; 30]>(),
                    b"%d\0".as_ptr() as *const c_char,
                    autocmd_bufnr.get(),
                );
                result = &raw mut strbuf as *mut c_char;
            }
            12 => {
                result = autocmd_match.get();
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_autocommand_match_name_to_substitute_for_amatch.ptr() as *const _)
                            as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
            }
            6 => {
                result = estack_sfile(ESTACK_SFILE);
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_source_file_name_to_substitute_for_sfile.ptr() as *const _)
                            as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                resultbuf = result;
            }
            8 => {
                result = estack_sfile(ESTACK_STACK);
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_call_stack_to_substitute_for_stack.ptr() as *const _)
                            as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                resultbuf = result;
            }
            9 => {
                result = estack_sfile(ESTACK_SCRIPT);
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_script_file_name_to_substitute_for_script.ptr() as *const _)
                            as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                resultbuf = result;
            }
            7 => {
                if (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                .es_name
                .is_null()
                    || (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                    .es_lnum
                        == 0 as linenr_T
                {
                    *errormsg = gettext(
                        (e_no_line_number_to_use_for_slnum.ptr() as *const _) as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                snprintf(
                    &raw mut strbuf as *mut c_char,
                    ::core::mem::size_of::<[c_char; 30]>(),
                    b"%d\0".as_ptr() as *const c_char,
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                    .es_lnum,
                );
                result = &raw mut strbuf as *mut c_char;
            }
            13 => {
                if (*current_sctx.ptr()).sc_lnum
                    + (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                    .es_lnum
                    == 0 as linenr_T
                {
                    *errormsg = gettext(
                        (e_no_line_number_to_use_for_sflnum.ptr() as *const _) as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                snprintf(
                    &raw mut strbuf as *mut c_char,
                    ::core::mem::size_of::<[c_char; 30]>(),
                    b"%d\0".as_ptr() as *const c_char,
                    (*current_sctx.ptr()).sc_lnum
                        + (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                        .es_lnum,
                );
                result = &raw mut strbuf as *mut c_char;
            }
            14 => {
                if (*current_sctx.ptr()).sc_sid <= 0 as c_int {
                    *errormsg = gettext(&raw const e_usingsid as *const c_char);
                    return ::core::ptr::null_mut::<c_char>();
                }
                snprintf(
                    &raw mut strbuf as *mut c_char,
                    ::core::mem::size_of::<[c_char; 30]>(),
                    b"<SNR>%d_\0".as_ptr() as *const c_char,
                    (*current_sctx.ptr()).sc_sid,
                );
                result = &raw mut strbuf as *mut c_char;
            }
            _ => {
                *errormsg = b"\0".as_ptr() as *const c_char;
            }
        }
        resultlen = strlen(result);
        if *src.offset(*usedlen as isize) as c_int == '<' as c_int {
            *usedlen = (*usedlen).wrapping_add(1);
            let mut s_0: *mut c_char = ::core::ptr::null_mut::<c_char>();
            s_0 = strrchr(result, '.' as c_int);
            if !s_0.is_null() && s_0 >= path_tail(result) {
                resultlen = s_0.offset_from(result) as size_t;
            }
        } else if !skip_mod {
            valid |= modify_fname(
                src,
                tilde_file,
                usedlen,
                &raw mut result,
                &raw mut resultbuf,
                &raw mut resultlen,
            );
            if result.is_null() {
                *errormsg = b"\0".as_ptr() as *const c_char;
                return ::core::ptr::null_mut::<c_char>();
            }
        }
    }
    if resultlen == 0 as size_t || valid != VALID_HEAD as c_int + VALID_PATH as c_int {
        if empty_is_error {
            if valid != VALID_HEAD as c_int + VALID_PATH as c_int {
                *errormsg = gettext(
                    b"E499: Empty file name for '%' or '#', only works with \":p:h\"\0".as_ptr()
                        as *const c_char,
                );
            } else {
                *errormsg =
                    gettext(b"E500: Evaluates to an empty string\0".as_ptr() as *const c_char);
            }
        }
        result = ::core::ptr::null_mut::<c_char>();
    } else {
        result = xmemdupz(result as *const c_void, resultlen) as *mut c_char;
    }
    xfree(resultbuf as *mut c_void);
    return result;
}

pub unsafe extern "C" fn expand_sfile(mut arg: *mut c_char) -> *mut c_char {
    let mut result: *mut c_char = xstrdup(arg);
    let mut p: *mut c_char = result;
    while *p != 0 {
        if strncmp(p, b"<sfile>\0".as_ptr() as *const c_char, 7 as size_t) != 0 as c_int {
            p = p.offset(1);
        } else {
            let mut srclen: size_t = 0;
            let mut errormsg: *const c_char = ::core::ptr::null::<c_char>();
            let mut repl: *mut c_char = eval_vars(
                p,
                result,
                &raw mut srclen,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut errormsg,
                ::core::ptr::null_mut::<c_int>(),
                true_0 != 0,
            );
            if !errormsg.is_null() {
                if *errormsg != 0 {
                    emsg(errormsg);
                }
                xfree(result as *mut c_void);
                return ::core::ptr::null_mut::<c_char>();
            }
            if repl.is_null() {
                p = p.offset(srclen as isize);
            } else {
                let mut len: size_t = strlen(result)
                    .wrapping_sub(srclen)
                    .wrapping_add(strlen(repl))
                    .wrapping_add(1 as size_t);
                let mut newres: *mut c_char = xmalloc(len) as *mut c_char;
                memmove(
                    newres as *mut c_void,
                    result as *const c_void,
                    p.offset_from(result) as size_t,
                );
                strcpy(newres.offset(p.offset_from(result) as isize), repl);
                len = strlen(newres);
                strcat(newres, p.offset(srclen as isize));
                xfree(repl as *mut c_void);
                xfree(result as *mut c_void);
                result = newres;
                p = newres.offset(len as isize);
            }
        }
    }
    return result;
}
