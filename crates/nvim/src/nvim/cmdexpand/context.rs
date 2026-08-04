//! Classifying the command line: which completion applies where.
//!
//! [`set_expand_context`] is the entry point; [`set_cmd_index`] parses the
//! range and the command name, and the `set_context_in_*` helpers handle the
//! commands whose argument is not a plain file name.  The big per-command
//! switch is in [`super::cmdname`].

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn set_expand_context(mut xp: *mut expand_T) {
    unsafe {
        let ccline: *mut CmdlineInfo = get_cmdline_info();
        if ((*ccline).cmdfirstc == '/' as ::core::ffi::c_int
            || (*ccline).cmdfirstc == '?' as ::core::ffi::c_int)
            && may_expand_pattern.get() as ::core::ffi::c_int != 0
        {
            (*xp).xp_context = EXPAND_PATTERN_IN_BUF;
            (*xp).xp_search_dir = (if (*ccline).cmdfirstc == '/' as ::core::ffi::c_int {
                FORWARD as ::core::ffi::c_int
            } else {
                BACKWARD as ::core::ffi::c_int
            }) as Direction;
            (*xp).xp_pattern = (*ccline).cmdbuff;
            (*xp).xp_pattern_len = (*ccline).cmdpos as size_t;
            search_first_line.set(0 as ::core::ffi::c_int as linenr_T);
            return;
        }
        if (*ccline).cmdfirstc != ':' as ::core::ffi::c_int
            && (*ccline).cmdfirstc != '>' as ::core::ffi::c_int
            && (*ccline).cmdfirstc != '=' as ::core::ffi::c_int
            && (*ccline).input_fn == 0
        {
            (*xp).xp_context = EXPAND_NOTHING;
            return;
        }
        set_cmd_context(
            xp,
            (*ccline).cmdbuff,
            (*ccline).cmdlen,
            (*ccline).cmdpos,
            true_0,
        );
    }
}

pub(crate) unsafe extern "C" fn set_cmd_index(
    mut cmd: *const ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut xp: *mut expand_T,
    mut complp: *mut ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let fuzzy: bool = cmdline_fuzzy_complete(cmd);
        if !fuzzy
            && p_ic.get() == 0
            && (*cmd as ::core::ffi::c_int == 'k' as ::core::ffi::c_int
                && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != 'e' as ::core::ffi::c_int)
        {
            (*eap).cmdidx = CMD_k;
            p = cmd.offset(1 as ::core::ffi::c_int as isize);
        } else {
            p = cmd;
            while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                >= 'A' as ::core::ffi::c_uint
                && *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    <= 'Z' as ::core::ffi::c_uint
            {
                while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                    || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                        && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                    || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    || *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int
                {
                    p = p.offset(1);
                }
            }
            if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'p' as ::core::ffi::c_int
                && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'y' as ::core::ffi::c_int
                && p == cmd.offset(2 as ::core::ffi::c_int as isize)
                && *p as ::core::ffi::c_int == '3' as ::core::ffi::c_int
            {
                p = p.offset(1);
                while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                    || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                        && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                    || *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int
                {
                    p = p.offset(1);
                }
            }
            if p == cmd
                && !vim_strchr(
                    b"@*!=><&~#\0".as_ptr() as *const ::core::ffi::c_char,
                    *p as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
            {
                p = p.offset(1);
            }
            let mut len: size_t = p.offset_from(cmd) as size_t;
            if len == 0 as size_t {
                (*xp).xp_context = EXPAND_UNSUCCESSFUL;
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
            (*eap).cmdidx = excmd_get_cmdidx(cmd, len);
            if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                >= 'A' as ::core::ffi::c_int
                && *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    <= 'Z' as ::core::ffi::c_int
                || fuzzy as ::core::ffi::c_int != 0
                    && (*eap).cmdidx as ::core::ffi::c_int != CMD_bang as ::core::ffi::c_int
                    && *p as ::core::ffi::c_int != NUL
            {
                while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                    || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                        && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                    || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    || *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int
                {
                    p = p.offset(1);
                }
            }
        }
        if *p as ::core::ffi::c_int == NUL
            && (*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                >= 'A' as ::core::ffi::c_uint
                && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    <= 'Z' as ::core::ffi::c_uint
                || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    >= 'a' as ::core::ffi::c_uint
                    && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                        <= 'z' as ::core::ffi::c_uint
                || ascii_isdigit(*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
        {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int {
            if *cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int
                && !vim_strchr(
                    b"cgriI\0".as_ptr() as *const ::core::ffi::c_char,
                    *cmd.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
            {
                (*eap).cmdidx = CMD_substitute;
                p = cmd.offset(1 as ::core::ffi::c_int as isize);
            } else if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                >= 'A' as ::core::ffi::c_int
                && *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    <= 'Z' as ::core::ffi::c_int
            {
                (*eap).cmd = cmd as *mut ::core::ffi::c_char;
                p = find_ucmd(
                    eap,
                    p as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    xp,
                    complp,
                );
                if p.is_null() {
                    (*eap).cmdidx = CMD_SIZE;
                }
            }
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int {
            (*xp).xp_context = EXPAND_UNSUCCESSFUL;
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        return p;
    }
}

pub(crate) unsafe extern "C" fn set_context_for_wildcard_arg(
    mut eap: *mut exarg_T,
    mut arg: *const ::core::ffi::c_char,
    mut usefilter: bool,
    mut xp: *mut expand_T,
    mut complp: *mut ::core::ffi::c_int,
) {
    unsafe {
        let mut in_quote: bool = false_0 != 0;
        let mut bow: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut len: size_t = 0 as size_t;
        (*xp).xp_pattern = skipwhite(arg);
        let mut p: *const ::core::ffi::c_char = (*xp).xp_pattern;
        while *p as ::core::ffi::c_int != NUL {
            let mut c: ::core::ffi::c_int = utf_ptr2char(p);
            if c == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                p = p.offset(1);
            } else if c == '`' as ::core::ffi::c_int {
                if !in_quote {
                    (*xp).xp_pattern = p as *mut ::core::ffi::c_char;
                    bow = p.offset(1 as ::core::ffi::c_int as isize);
                }
                in_quote = !in_quote;
            } else if c == '|' as ::core::ffi::c_int
                || c == '\n' as ::core::ffi::c_int
                || c == '"' as ::core::ffi::c_int
                || ascii_iswhite(c) as ::core::ffi::c_int != 0
            {
                len = 0 as size_t;
                while *p as ::core::ffi::c_int != NUL {
                    c = utf_ptr2char(p);
                    if c == '`' as ::core::ffi::c_int
                        || vim_isfilec_or_wc(c) as ::core::ffi::c_int != 0
                    {
                        break;
                    }
                    len = utfc_ptr2len(p) as size_t;
                    p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
                }
                if in_quote {
                    bow = p;
                } else {
                    (*xp).xp_pattern = p as *mut ::core::ffi::c_char;
                }
                p = p.offset(-(len as isize));
            }
            p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
        }
        if !bow.is_null() && in_quote as ::core::ffi::c_int != 0 {
            (*xp).xp_pattern = bow as *mut ::core::ffi::c_char;
        }
        (*xp).xp_context = EXPAND_FILES;
        if usefilter as ::core::ffi::c_int != 0
            || !eap.is_null()
                && ((*eap).cmdidx as ::core::ffi::c_int == CMD_bang as ::core::ffi::c_int
                    || (*eap).cmdidx as ::core::ffi::c_int == CMD_terminal as ::core::ffi::c_int)
            || *complp == EXPAND_SHELLCMDLINE
        {
            (*xp).xp_shell = true_0 != 0;
            if (*xp).xp_pattern == skipwhite(arg) {
                (*xp).xp_context = EXPAND_SHELLCMD;
            }
        }
        if *(*xp).xp_pattern as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
            p = (*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize);
            while *p as ::core::ffi::c_int != NUL {
                if !vim_isIDc(*p as uint8_t as ::core::ffi::c_int) {
                    break;
                }
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int == NUL {
                (*xp).xp_context = EXPAND_ENV_VARS;
                (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                if *complp != EXPAND_USER_DEFINED && *complp != EXPAND_USER_LIST {
                    *complp = EXPAND_ENV_VARS;
                }
            }
        }
        if *(*xp).xp_pattern as ::core::ffi::c_int == '~' as ::core::ffi::c_int {
            p = (*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize);
            while *p as ::core::ffi::c_int != NUL
                && *p as ::core::ffi::c_int != '/' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            let user: *mut ::core::ffi::c_char = (*xp).xp_pattern.offset(1);
            if *p as ::core::ffi::c_int == NUL
                && p > user as *const ::core::ffi::c_char
                && match_user(CStr::from_ptr(user)) != UserMatch::None
            {
                (*xp).xp_context = EXPAND_USER;
                (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
            }
        }
    }
}

pub(crate) unsafe extern "C" fn set_context_in_argopt(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = vim_strchr(arg, '=' as ::core::ffi::c_int);
        if p.is_null() {
            (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        } else {
            (*xp).xp_pattern = p.offset(1 as ::core::ffi::c_int as isize);
        }
        (*xp).xp_context = EXPAND_ARGOPT;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn set_context_in_filter_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        if *arg as ::core::ffi::c_int != NUL {
            arg = skip_vimgrep_pat(
                arg as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
        }
        if arg.is_null() || *arg as ::core::ffi::c_int == NUL {
            (*xp).xp_context = EXPAND_NOTHING;
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        return skipwhite(arg);
    }
}

pub(crate) unsafe extern "C" fn set_context_in_match_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        if *arg as ::core::ffi::c_int == NUL || ends_excmd(*arg as ::core::ffi::c_int) == 0 {
            set_context_in_echohl_cmd(xp, arg);
            arg = skipwhite(skiptowhite(arg));
            if *arg as ::core::ffi::c_int != NUL {
                (*xp).xp_context = EXPAND_NOTHING;
                arg = skip_regexp(
                    (arg as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize),
                    *arg as uint8_t as ::core::ffi::c_int,
                    magic_isset() as ::core::ffi::c_int,
                );
            }
        }
        return find_nextcmd(arg);
    }
}

pub(crate) unsafe extern "C" fn find_cmd_after_global_cmd(
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let delim: ::core::ffi::c_int = *arg as uint8_t as ::core::ffi::c_int;
        if delim != 0 {
            arg = arg.offset(1);
        }
        while *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *arg.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                != delim
        {
            if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                arg = arg.offset(1);
            }
            arg = arg.offset(1);
        }
        if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
            return arg.offset(1 as ::core::ffi::c_int as isize);
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn find_cmd_after_substitute_cmd(
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let delim: ::core::ffi::c_int = *arg as uint8_t as ::core::ffi::c_int;
        if delim != 0 {
            arg = arg.offset(1);
            arg = skip_regexp(
                arg as *mut ::core::ffi::c_char,
                delim,
                magic_isset() as ::core::ffi::c_int,
            );
            if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                && *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == delim
            {
                arg = arg.offset(1);
                while *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                    && *arg.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int
                        != delim
                {
                    if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                        && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != NUL
                    {
                        arg = arg.offset(1);
                    }
                    arg = arg.offset(1);
                }
                if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                    arg = arg.offset(1);
                }
            }
        }
        while *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
            && strchr(
                b"|\"#\0".as_ptr() as *const ::core::ffi::c_char,
                *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            )
            .is_null()
        {
            arg = arg.offset(1);
        }
        if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
            return arg;
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn find_cmd_after_isearch_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        arg = skipwhite(skipdigits(arg));
        if *arg as ::core::ffi::c_int != '/' as ::core::ffi::c_int {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        arg = arg.offset(1);
        while *arg as ::core::ffi::c_int != 0
            && *arg as ::core::ffi::c_int != '/' as ::core::ffi::c_int
        {
            if *arg as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                arg = arg.offset(1);
            }
            arg = arg.offset(1);
        }
        if *arg != 0 {
            arg = skipwhite(arg.offset(1 as ::core::ffi::c_int as isize));
            if *arg as ::core::ffi::c_int == NUL
                || strchr(
                    b"|\"\n\0".as_ptr() as *const ::core::ffi::c_char,
                    *arg as ::core::ffi::c_int,
                )
                .is_null()
            {
                (*xp).xp_context = EXPAND_NOTHING;
            } else {
                return arg;
            }
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn set_context_in_unlet_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        loop {
            (*xp).xp_pattern = strchr(arg, ' ' as ::core::ffi::c_int);
            if (*xp).xp_pattern.is_null() {
                break;
            }
            arg = (*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize);
        }
        (*xp).xp_context = EXPAND_USER_VARS;
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        if *(*xp).xp_pattern as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
            (*xp).xp_context = EXPAND_ENV_VARS;
            (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn set_context_in_lang_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut p: *const ::core::ffi::c_char = skiptowhite(arg);
        if *p as ::core::ffi::c_int == NUL {
            (*xp).xp_context = EXPAND_LANGUAGE;
            (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        } else if strncmp(
            arg,
            b"messages\0".as_ptr() as *const ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                arg,
                b"ctype\0".as_ptr() as *const ::core::ffi::c_char,
                p.offset_from(arg) as size_t,
            ) == 0 as ::core::ffi::c_int
            || strncmp(
                arg,
                b"time\0".as_ptr() as *const ::core::ffi::c_char,
                p.offset_from(arg) as size_t,
            ) == 0 as ::core::ffi::c_int
            || strncmp(
                arg,
                b"collate\0".as_ptr() as *const ::core::ffi::c_char,
                p.offset_from(arg) as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            (*xp).xp_context = EXPAND_LOCALES;
            (*xp).xp_pattern = skipwhite(p);
        } else {
            (*xp).xp_context = EXPAND_NOTHING;
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn set_context_in_breakadd_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
    mut cmdidx: cmdidx_T,
) -> *const ::core::ffi::c_char {
    unsafe {
        (*xp).xp_context = EXPAND_BREAKPOINT;
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        if cmdidx as ::core::ffi::c_int == CMD_breakadd as ::core::ffi::c_int {
            breakpt_expand_what.set(EXP_BREAKPT_ADD);
        } else if cmdidx as ::core::ffi::c_int == CMD_breakdel as ::core::ffi::c_int {
            breakpt_expand_what.set(EXP_BREAKPT_DEL);
        } else {
            breakpt_expand_what.set(EXP_PROFDEL);
        }
        let mut p: *const ::core::ffi::c_char = skipwhite(arg);
        if *p as ::core::ffi::c_int == NUL {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        let mut subcmd_start: *const ::core::ffi::c_char = p;
        if strncmp(
            b"file \0".as_ptr() as *const ::core::ffi::c_char,
            p,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                b"func \0".as_ptr() as *const ::core::ffi::c_char,
                p,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(4 as ::core::ffi::c_int as isize);
            p = skipwhite(p);
            if ascii_isdigit(*p as ::core::ffi::c_int) {
                p = skipdigits(p);
                if *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int {
                    (*xp).xp_context = EXPAND_NOTHING;
                    return ::core::ptr::null::<::core::ffi::c_char>();
                }
                p = skipwhite(p);
            }
            if strncmp(
                b"file\0".as_ptr() as *const ::core::ffi::c_char,
                subcmd_start,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                (*xp).xp_context = EXPAND_FILES;
            } else {
                (*xp).xp_context = EXPAND_USER_FUNC;
            }
            (*xp).xp_pattern = p as *mut ::core::ffi::c_char;
        } else if strncmp(
            b"expr \0".as_ptr() as *const ::core::ffi::c_char,
            p,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            (*xp).xp_context = EXPAND_EXPRESSION;
            (*xp).xp_pattern = skipwhite(p.offset(5 as ::core::ffi::c_int as isize));
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn set_context_in_scriptnames_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        (*xp).xp_context = EXPAND_NOTHING;
        (*xp).xp_pattern = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = skipwhite(arg);
        if ascii_isdigit(*p as ::core::ffi::c_int) {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        (*xp).xp_context = EXPAND_SCRIPTNAMES;
        (*xp).xp_pattern = p;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn set_context_in_filetype_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        (*xp).xp_context = EXPAND_FILETYPECMD;
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        filetype_expand_what.set(EXP_FILETYPECMD_ALL);
        let mut p: *mut ::core::ffi::c_char = skipwhite(arg);
        if *p as ::core::ffi::c_int == NUL {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        let mut val: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            if strncmp(
                p,
                b"plugin\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                val |= EXPAND_FILETYPECMD_PLUGIN;
                p = skipwhite(p.offset(6 as ::core::ffi::c_int as isize));
            } else {
                if strncmp(
                    p,
                    b"indent\0".as_ptr() as *const ::core::ffi::c_char,
                    6 as size_t,
                ) != 0 as ::core::ffi::c_int
                {
                    break;
                }
                val |= EXPAND_FILETYPECMD_INDENT;
                p = skipwhite(p.offset(6 as ::core::ffi::c_int as isize));
            }
        }
        if val & EXPAND_FILETYPECMD_PLUGIN != 0 && val & EXPAND_FILETYPECMD_INDENT != 0 {
            filetype_expand_what.set(EXP_FILETYPECMD_ONOFF);
        } else if val & EXPAND_FILETYPECMD_PLUGIN != 0 {
            filetype_expand_what.set(EXP_FILETYPECMD_INDENT);
        } else if val & EXPAND_FILETYPECMD_INDENT != 0 {
            filetype_expand_what.set(EXP_FILETYPECMD_PLUGIN);
        }
        (*xp).xp_pattern = p;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn set_context_with_pattern(mut xp: *mut expand_T) {
    unsafe {
        let mut ccline: *mut CmdlineInfo = get_cmdline_info();
        (*emsg_off.ptr()) += 1;
        let mut skiplen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut dummy: ::core::ffi::c_int = 0;
        let mut patlen: ::core::ffi::c_int = 0;
        let mut retval: ::core::ffi::c_int = parse_pattern_and_range(
            pre_incsearch_pos.ptr(),
            &raw mut dummy,
            &raw mut skiplen,
            &raw mut patlen,
        ) as ::core::ffi::c_int;
        (*emsg_off.ptr()) -= 1;
        if retval == 0 || (*ccline).cmdpos <= skiplen || (*ccline).cmdpos > skiplen + patlen {
            return;
        }
        (*xp).xp_pattern = (*ccline).cmdbuff.offset(skiplen as isize);
        (*xp).xp_pattern_len = ((*ccline).cmdpos - skiplen) as size_t;
        (*xp).xp_context = EXPAND_PATTERN_IN_BUF;
        (*xp).xp_search_dir = FORWARD;
    }
}
