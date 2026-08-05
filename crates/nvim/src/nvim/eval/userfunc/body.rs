//! Reading a function body, one line at a time.
//!
//! `get_function_body` is `:function`'s line loop: it tracks `:if`/`:while`/
//! `:for`/`:try` nesting so that the matching `:endfunction` is the right
//! one, keeps continuation lines and comments verbatim, honours a
//! here-document inside the body, and refuses to nest more than
//! MAX_FUNC_NESTING definitions deep.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub const MAX_FUNC_NESTING: ::core::ffi::c_int = 50 as ::core::ffi::c_int;

pub(crate) unsafe extern "C" fn get_function_body(
    mut eap: *mut exarg_T,
    mut newlines: *mut garray_T,
    mut line_arg_in: *mut ::core::ffi::c_char,
    mut line_to_free: *mut *mut ::core::ffi::c_char,
    mut show_block: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut saved_wait_return: bool = need_wait_return.get();
        let mut line_arg: *mut ::core::ffi::c_char = line_arg_in;
        let mut indent: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
        let mut nesting: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut skip_until: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut ret: ::core::ffi::c_int = FAIL;
        let mut is_heredoc: bool = false_0 != 0;
        let mut heredoc_trimmed: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut heredoc_trimmedlen: size_t = 0 as size_t;
        let mut do_concat: bool = true_0 != 0;
        '_theend: {
            loop {
                if KeyTyped.get() {
                    msg_scroll.set(true_0);
                    saved_wait_return = false_0 != 0;
                }
                need_wait_return.set(false_0 != 0);
                let mut theline: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut p: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut arg: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                if !line_arg.is_null() {
                    theline = line_arg;
                    p = vim_strchr(theline, '\n' as ::core::ffi::c_int);
                    if p.is_null() {
                        line_arg = line_arg.offset(strlen(line_arg) as isize);
                    } else {
                        *p = NUL as ::core::ffi::c_char;
                        line_arg = p.offset(1 as ::core::ffi::c_int as isize);
                    }
                } else {
                    xfree(*line_to_free as *mut ::core::ffi::c_void);
                    if (*eap).ea_getline.is_none() {
                        theline = getcmdline(
                            ':' as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                            indent,
                            do_concat,
                        );
                    } else {
                        theline = (*eap).ea_getline.expect("non-null function pointer")(
                            ':' as ::core::ffi::c_int,
                            (*eap).cookie,
                            indent,
                            do_concat,
                        );
                    }
                    *line_to_free = theline;
                }
                if KeyTyped.get() {
                    lines_left.set(Rows.get() - 1 as ::core::ffi::c_int);
                }
                if theline.is_null() {
                    if !skip_until.is_null() {
                        semsg(
                            gettext(E_MISSING_HEREDOC_END_MARKER_STR.as_ptr()),
                            skip_until,
                        );
                    } else {
                        emsg(gettext(
                            b"E126: Missing :endfunction\0".as_ptr() as *const ::core::ffi::c_char
                        ));
                    }
                    break '_theend;
                } else {
                    if show_block {
                        '_c2rust_label: {
                            if indent >= 0 as ::core::ffi::c_int {
                            } else {
                                __assert_fail(
                                b"indent >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/eval/userfunc.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                2419 as ::core::ffi::c_uint,
                                b"int get_function_body(exarg_T *, garray_T *, char *, char **, _Bool)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                            }
                        };
                        ui_ext_cmdline_block_append(indent as size_t, theline);
                    }
                    let mut sourcing_lnum_off: linenr_T =
                        get_sourced_lnum((*eap).ea_getline, (*eap).cookie);
                    if (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum
                        < sourcing_lnum_off
                    {
                        sourcing_lnum_off -= (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_lnum;
                    } else {
                        sourcing_lnum_off = 0 as ::core::ffi::c_int as linenr_T;
                    }
                    if !skip_until.is_null() {
                        if heredoc_trimmed.is_null()
                            || is_heredoc as ::core::ffi::c_int != 0
                                && skipwhite(theline) == theline
                            || strncmp(theline, heredoc_trimmed, heredoc_trimmedlen)
                                == 0 as ::core::ffi::c_int
                        {
                            if heredoc_trimmed.is_null() {
                                p = theline;
                            } else if is_heredoc {
                                p = if skipwhite(theline) == theline {
                                    theline
                                } else {
                                    theline.offset(heredoc_trimmedlen as isize)
                                };
                            } else {
                                p = theline.offset(heredoc_trimmedlen as isize);
                            }
                            if strcmp(p, skip_until) == 0 as ::core::ffi::c_int {
                                let mut ptr_: *mut *mut ::core::ffi::c_void =
                                    &raw mut skip_until as *mut *mut ::core::ffi::c_void;
                                xfree(*ptr_);
                                *ptr_ = NULL;
                                let _ = *ptr_;
                                let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                    &raw mut heredoc_trimmed as *mut *mut ::core::ffi::c_void;
                                xfree(*ptr__0);
                                *ptr__0 = NULL;
                                let _ = *ptr__0;
                                heredoc_trimmedlen = 0 as size_t;
                                do_concat = true_0 != 0;
                                is_heredoc = false_0 != 0;
                            }
                        }
                    } else {
                        p = theline;
                        while ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                            || *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                        {
                            p = p.offset(1);
                        }
                        if checkforcmd(
                            &raw mut p,
                            b"endfunction\0".as_ptr() as *const ::core::ffi::c_char,
                            4 as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                            != 0
                            && {
                                let c2rust_fresh12 = nesting;
                                nesting = nesting - 1;
                                c2rust_fresh12 == 0 as ::core::ffi::c_int
                            }
                        {
                            if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                                p = p.offset(1);
                            }
                            let mut nextcmd: *mut ::core::ffi::c_char =
                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                            if *p as ::core::ffi::c_int == '|' as ::core::ffi::c_int {
                                nextcmd = p.offset(1 as ::core::ffi::c_int as isize);
                            } else if !line_arg.is_null()
                                && *skipwhite(line_arg) as ::core::ffi::c_int != NUL
                            {
                                nextcmd = line_arg;
                            } else if *p as ::core::ffi::c_int != NUL
                                && *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int
                                && p_verbose.get() > 0 as OptInt
                            {
                                swmsg(
                                    true_0 != 0,
                                    gettext(b"W22: Text found after :endfunction: %s\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    p,
                                );
                            }
                            if !nextcmd.is_null() {
                                (*eap).nextcmd = nextcmd;
                                if !(*line_to_free).is_null() {
                                    xfree(*(*eap).cmdlinep as *mut ::core::ffi::c_void);
                                    *(*eap).cmdlinep = *line_to_free;
                                    *line_to_free = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                }
                            }
                            break;
                        } else {
                            if indent > 2 as ::core::ffi::c_int
                                && strncmp(
                                    p,
                                    b"end\0".as_ptr() as *const ::core::ffi::c_char,
                                    3 as size_t,
                                ) == 0 as ::core::ffi::c_int
                            {
                                indent -= 2 as ::core::ffi::c_int;
                            } else if strncmp(
                                p,
                                b"if\0".as_ptr() as *const ::core::ffi::c_char,
                                2 as size_t,
                            ) == 0 as ::core::ffi::c_int
                                || strncmp(
                                    p,
                                    b"wh\0".as_ptr() as *const ::core::ffi::c_char,
                                    2 as size_t,
                                ) == 0 as ::core::ffi::c_int
                                || strncmp(
                                    p,
                                    b"for\0".as_ptr() as *const ::core::ffi::c_char,
                                    3 as size_t,
                                ) == 0 as ::core::ffi::c_int
                                || strncmp(
                                    p,
                                    b"try\0".as_ptr() as *const ::core::ffi::c_char,
                                    3 as size_t,
                                ) == 0 as ::core::ffi::c_int
                            {
                                indent += 2 as ::core::ffi::c_int;
                            }
                            if checkforcmd(
                                &raw mut p,
                                b"function\0".as_ptr() as *const ::core::ffi::c_char,
                                2 as ::core::ffi::c_int,
                            ) {
                                if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                                    p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
                                }
                                p = p.offset(eval_fname_script(p) as isize);
                                xfree(trans_function_name(
                                    &raw mut p,
                                    true_0 != 0,
                                    0 as ::core::ffi::c_int,
                                    ::core::ptr::null_mut::<funcdict_T>(),
                                    ::core::ptr::null_mut::<*mut partial_T>(),
                                )
                                    as *mut ::core::ffi::c_void);
                                if *skipwhite(p) as ::core::ffi::c_int == '(' as ::core::ffi::c_int
                                {
                                    if nesting == MAX_FUNC_NESTING - 1 as ::core::ffi::c_int {
                                        emsg(gettext(E_FUNCTION_NESTING_TOO_DEEP.as_ptr()));
                                    } else {
                                        nesting += 1;
                                        indent += 2 as ::core::ffi::c_int;
                                    }
                                }
                            }
                            p = skip_range(p, ::core::ptr::null_mut::<::core::ffi::c_int>());
                            let tp: *mut ::core::ffi::c_char = p;
                            if (checkforcmd(
                                &raw mut p,
                                b"append\0".as_ptr() as *const ::core::ffi::c_char,
                                1 as ::core::ffi::c_int,
                            ) as ::core::ffi::c_int
                                != 0
                                || checkforcmd(
                                    &raw mut p,
                                    b"change\0".as_ptr() as *const ::core::ffi::c_char,
                                    1 as ::core::ffi::c_int,
                                ) as ::core::ffi::c_int
                                    != 0
                                || checkforcmd(
                                    &raw mut p,
                                    b"insert\0".as_ptr() as *const ::core::ffi::c_char,
                                    1 as ::core::ffi::c_int,
                                ) as ::core::ffi::c_int
                                    != 0)
                                && (*p as ::core::ffi::c_int == '!' as ::core::ffi::c_int
                                    || *p as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                                    || ascii_iswhite_nl_or_nul(*p as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0)
                            {
                                skip_until = xmemdupz(
                                    b".\0".as_ptr() as *const ::core::ffi::c_char
                                        as *const ::core::ffi::c_void,
                                    1 as size_t,
                                )
                                    as *mut ::core::ffi::c_char;
                            } else {
                                p = tp;
                            }
                            arg = skipwhite(skiptowhite(p));
                            if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '<' as ::core::ffi::c_int
                                && *arg.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '<' as ::core::ffi::c_int
                                && (*p.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'p' as ::core::ffi::c_int
                                    && *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'y' as ::core::ffi::c_int
                                    && (!(*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'A' as ::core::ffi::c_uint
                                        && *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'Z' as ::core::ffi::c_uint
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'a' as ::core::ffi::c_uint
                                            && *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'z' as ::core::ffi::c_uint
                                        || ascii_isdigit(
                                            *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int,
                                        )
                                            as ::core::ffi::c_int
                                            != 0)
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 't' as ::core::ffi::c_int
                                        || (*p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '3' as ::core::ffi::c_int
                                            || *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == 'x' as ::core::ffi::c_int)
                                            && !(*p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                >= 'A' as ::core::ffi::c_uint
                                                && *p.offset(3 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_uint
                                                    <= 'Z' as ::core::ffi::c_uint
                                                || *p.offset(3 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_uint
                                                    >= 'a' as ::core::ffi::c_uint
                                                    && *p.offset(3 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_uint
                                                        <= 'z' as ::core::ffi::c_uint))
                                    || *p.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'p' as ::core::ffi::c_int
                                        && *p.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'e' as ::core::ffi::c_int
                                        && (!(*p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'A' as ::core::ffi::c_uint
                                            && *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'Z' as ::core::ffi::c_uint
                                            || *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                >= 'a' as ::core::ffi::c_uint
                                                && *p.offset(2 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_uint
                                                    <= 'z' as ::core::ffi::c_uint)
                                            || *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == 'r' as ::core::ffi::c_int)
                                    || *p.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 't' as ::core::ffi::c_int
                                        && *p.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'c' as ::core::ffi::c_int
                                        && (!(*p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'A' as ::core::ffi::c_uint
                                            && *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'Z' as ::core::ffi::c_uint
                                            || *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                >= 'a' as ::core::ffi::c_uint
                                                && *p.offset(2 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_uint
                                                    <= 'z' as ::core::ffi::c_uint)
                                            || *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == 'l' as ::core::ffi::c_int)
                                    || *p.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'l' as ::core::ffi::c_int
                                        && *p.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'u' as ::core::ffi::c_int
                                        && *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'a' as ::core::ffi::c_int
                                        && !(*p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'A' as ::core::ffi::c_uint
                                            && *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'Z' as ::core::ffi::c_uint
                                            || *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                >= 'a' as ::core::ffi::c_uint
                                                && *p.offset(3 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_uint
                                                    <= 'z' as ::core::ffi::c_uint)
                                    || *p.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'r' as ::core::ffi::c_int
                                        && *p.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'u' as ::core::ffi::c_int
                                        && *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'b' as ::core::ffi::c_int
                                        && (!(*p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'A' as ::core::ffi::c_uint
                                            && *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'Z' as ::core::ffi::c_uint
                                            || *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                >= 'a' as ::core::ffi::c_uint
                                                && *p.offset(3 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_uint
                                                    <= 'z' as ::core::ffi::c_uint)
                                            || *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == 'y' as ::core::ffi::c_int)
                                    || *p.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'm' as ::core::ffi::c_int
                                        && *p.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'z' as ::core::ffi::c_int
                                        && (!(*p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'A' as ::core::ffi::c_uint
                                            && *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'Z' as ::core::ffi::c_uint
                                            || *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                >= 'a' as ::core::ffi::c_uint
                                                && *p.offset(2 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_uint
                                                    <= 'z' as ::core::ffi::c_uint)
                                            || *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == 's' as ::core::ffi::c_int))
                            {
                                p = skipwhite(arg.offset(2 as ::core::ffi::c_int as isize));
                                if strncmp(
                                    p,
                                    b"trim\0".as_ptr() as *const ::core::ffi::c_char,
                                    4 as size_t,
                                ) == 0 as ::core::ffi::c_int
                                    && (*p.offset(4 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == NUL
                                        || ascii_iswhite(
                                            *p.offset(4 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int,
                                        )
                                            as ::core::ffi::c_int
                                            != 0)
                                {
                                    p = skipwhite(p.offset(4 as ::core::ffi::c_int as isize));
                                    heredoc_trimmedlen =
                                        skipwhite(theline).offset_from(theline) as size_t;
                                    heredoc_trimmed = xmemdupz(
                                        theline as *const ::core::ffi::c_void,
                                        heredoc_trimmedlen,
                                    )
                                        as *mut ::core::ffi::c_char;
                                }
                                if *p as ::core::ffi::c_int == NUL {
                                    skip_until = xmemdupz(
                                        b".\0".as_ptr() as *const ::core::ffi::c_char
                                            as *const ::core::ffi::c_void,
                                        1 as size_t,
                                    )
                                        as *mut ::core::ffi::c_char;
                                } else {
                                    skip_until = xmemdupz(
                                        p as *const ::core::ffi::c_void,
                                        skiptowhite(p).offset_from(p) as size_t,
                                    )
                                        as *mut ::core::ffi::c_char;
                                }
                                do_concat = false_0 != 0;
                                is_heredoc = true_0 != 0;
                            }
                            if !is_heredoc {
                                arg = p;
                                if checkforcmd(
                                    &raw mut arg,
                                    b"let\0".as_ptr() as *const ::core::ffi::c_char,
                                    2 as ::core::ffi::c_int,
                                ) as ::core::ffi::c_int
                                    != 0
                                    || checkforcmd(
                                        &raw mut p,
                                        b"const\0".as_ptr() as *const ::core::ffi::c_char,
                                        5 as ::core::ffi::c_int,
                                    ) as ::core::ffi::c_int
                                        != 0
                                {
                                    let mut var_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    let mut semicolon: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    arg = skip_var_list(
                                        arg,
                                        &raw mut var_count,
                                        &raw mut semicolon,
                                        true_0 != 0,
                                    )
                                        as *mut ::core::ffi::c_char;
                                    if !arg.is_null() {
                                        arg = skipwhite(arg);
                                    }
                                    if !arg.is_null()
                                        && strncmp(
                                            arg,
                                            b"=<<\0".as_ptr() as *const ::core::ffi::c_char,
                                            3 as size_t,
                                        ) == 0 as ::core::ffi::c_int
                                    {
                                        p = skipwhite(arg.offset(3 as ::core::ffi::c_int as isize));
                                        let mut has_trim: bool = false_0 != 0;
                                        loop {
                                            if strncmp(
                                                p,
                                                b"trim\0".as_ptr() as *const ::core::ffi::c_char,
                                                4 as size_t,
                                            ) == 0 as ::core::ffi::c_int
                                                && (*p.offset(4 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == NUL
                                                    || ascii_iswhite(
                                                        *p.offset(4 as ::core::ffi::c_int as isize)
                                                            as ::core::ffi::c_int,
                                                    )
                                                        as ::core::ffi::c_int
                                                        != 0)
                                            {
                                                p = skipwhite(
                                                    p.offset(4 as ::core::ffi::c_int as isize),
                                                );
                                                has_trim = true_0 != 0;
                                            } else {
                                                if !(strncmp(
                                                    p,
                                                    b"eval\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    4 as size_t,
                                                ) == 0 as ::core::ffi::c_int
                                                    && (*p.offset(4 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        == NUL
                                                        || ascii_iswhite(*p.offset(
                                                            4 as ::core::ffi::c_int as isize,
                                                        )
                                                            as ::core::ffi::c_int)
                                                            as ::core::ffi::c_int
                                                            != 0))
                                                {
                                                    break;
                                                }
                                                p = skipwhite(
                                                    p.offset(4 as ::core::ffi::c_int as isize),
                                                );
                                            }
                                        }
                                        if has_trim {
                                            heredoc_trimmedlen =
                                                skipwhite(theline).offset_from(theline) as size_t;
                                            heredoc_trimmed = xmemdupz(
                                                theline as *const ::core::ffi::c_void,
                                                heredoc_trimmedlen,
                                            )
                                                as *mut ::core::ffi::c_char;
                                        }
                                        let mut ptr__1: *mut *mut ::core::ffi::c_void =
                                            &raw mut skip_until as *mut *mut ::core::ffi::c_void;
                                        xfree(*ptr__1);
                                        *ptr__1 = NULL;
                                        let _ = *ptr__1;
                                        skip_until = xmemdupz(
                                            p as *const ::core::ffi::c_void,
                                            skiptowhite(p).offset_from(p) as size_t,
                                        )
                                            as *mut ::core::ffi::c_char;
                                        do_concat = false_0 != 0;
                                        is_heredoc = true_0 != 0;
                                    }
                                }
                            }
                        }
                    }
                    ga_grow(
                        newlines,
                        1 as ::core::ffi::c_int + sourcing_lnum_off as ::core::ffi::c_int,
                    );
                    p = xstrdup(theline);
                    let c2rust_fresh13 = (*newlines).ga_len;
                    (*newlines).ga_len = (*newlines).ga_len + 1;
                    let c2rust_lvalue_ptr = &raw mut *((*newlines).ga_data
                        as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh13 as isize);
                    *c2rust_lvalue_ptr = p;
                    loop {
                        let c2rust_fresh14 = sourcing_lnum_off;
                        sourcing_lnum_off = sourcing_lnum_off - 1;
                        if c2rust_fresh14 <= 0 as linenr_T {
                            break;
                        }
                        let c2rust_fresh15 = (*newlines).ga_len;
                        (*newlines).ga_len = (*newlines).ga_len + 1;
                        let c2rust_lvalue_ptr_0 = &raw mut *((*newlines).ga_data
                            as *mut *mut ::core::ffi::c_char)
                            .offset(c2rust_fresh15 as isize);
                        *c2rust_lvalue_ptr_0 = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                    if !line_arg.is_null() && *line_arg as ::core::ffi::c_int == NUL {
                        line_arg = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                }
            }
            if did_emsg.get() == 0 {
                ret = OK;
            }
        }
        xfree(skip_until as *mut ::core::ffi::c_void);
        xfree(heredoc_trimmed as *mut ::core::ffi::c_void);
        need_wait_return.set(
            need_wait_return.get() as ::core::ffi::c_int | saved_wait_return as ::core::ffi::c_int
                != 0,
        );
        return ret;
    }
}
