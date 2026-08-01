//! Expanding a list of patterns, the way a command line means them.
//!
//! [`gen_expand_wildcards`] is the entry point for `:edit`, `expand()` and
//! command-line completion: it expands environment variables, hands
//! backticked patterns to the shell ([`expand_backtick`]), searches `'path'`
//! when the caller asked for that ([`expand_path_option`]), and otherwise
//! falls through to the file-system walk in [`glob`](super::glob). The `EW_*`
//! flags say which of those apply.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn expand_path_option(
    mut curdir: *mut ::core::ffi::c_char,
    mut path_option: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
) {
    unsafe {
        let mut buf: *mut ::core::ffi::c_char =
            xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        let mut curdirlen: size_t = 0 as size_t;
        while *path_option as ::core::ffi::c_int != NUL {
            let mut buflen: size_t = copy_option_part(
                &raw mut path_option,
                buf,
                MAXPATHL as size_t,
                b" ,\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if !vim_strchr(buf, '`' as ::core::ffi::c_int).is_null() {
                continue;
            }
            if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && (*buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || vim_ispathsep(
                        *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0)
            {
                if (*curbuf.get()).b_ffname.is_null() {
                    continue;
                }
                let mut p: *mut ::core::ffi::c_char = path_tail((*curbuf.get()).b_ffname);
                let mut plen: size_t = p.offset_from((*curbuf.get()).b_ffname) as size_t;
                if plen.wrapping_add(strlen(buf)) >= MAXPATHL as size_t {
                    continue;
                }
                if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                    *buf.offset(plen as isize) = NUL as ::core::ffi::c_char;
                } else {
                    memmove(
                        buf.offset(plen as isize) as *mut ::core::ffi::c_void,
                        buf.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                        buflen.wrapping_sub(2 as size_t).wrapping_add(1 as size_t),
                    );
                }
                memmove(
                    buf as *mut ::core::ffi::c_void,
                    (*curbuf.get()).b_ffname as *const ::core::ffi::c_void,
                    plen,
                );
                buflen = simplify_filename(buf);
            } else if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                strcpy(buf, curdir);
                if curdirlen == 0 as size_t {
                    curdirlen = strlen(curdir);
                }
                buflen = curdirlen;
            } else {
                if path_with_url(buf) != 0 {
                    continue;
                }
                if !path_is_absolute(buf) {
                    if curdirlen == 0 as size_t {
                        curdirlen = strlen(curdir);
                    }
                    if curdirlen.wrapping_add(buflen).wrapping_add(3 as size_t) > MAXPATHL as size_t
                    {
                        continue;
                    }
                    memmove(
                        buf.offset(curdirlen as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *mut ::core::ffi::c_void,
                        buf as *const ::core::ffi::c_void,
                        buflen.wrapping_add(1 as size_t),
                    );
                    strcpy(buf, curdir);
                    *buf.offset(curdirlen as isize) = PATHSEP as ::core::ffi::c_char;
                    buflen = simplify_filename(buf);
                }
            }
            ga_grow(gap, 1 as ::core::ffi::c_int);
            *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset((*gap).ga_len as isize) =
                xmemdupz(buf as *const ::core::ffi::c_void, buflen) as *mut ::core::ffi::c_char;
            (*gap).ga_len += 1;
        }
        xfree(buf as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn expand_in_path(
    gap: *mut garray_T,
    pattern: *mut ::core::ffi::c_char,
    flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut path_ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut path_option: *mut ::core::ffi::c_char =
            if *(*curbuf.get()).b_p_path as ::core::ffi::c_int == NUL {
                p_path.get()
            } else {
                (*curbuf.get()).b_p_path
            };
        let curdir: *mut ::core::ffi::c_char =
            xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        os_dirname(curdir, MAXPATHL as size_t);
        ga_init(
            &raw mut path_ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
        if flags & EW_CDPATH as ::core::ffi::c_int != 0 {
            expand_path_option(curdir, p_cdpath.get(), &raw mut path_ga);
        } else {
            expand_path_option(curdir, path_option, &raw mut path_ga);
        }
        xfree(curdir as *mut ::core::ffi::c_void);
        if path_ga.ga_len <= 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        let paths: *mut ::core::ffi::c_char = ga_concat_strings(
            &raw mut path_ga,
            b",\0".as_ptr() as *const ::core::ffi::c_char,
        );
        ga_clear_strings(&raw mut path_ga);
        let mut glob_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if flags & EW_ICASE as ::core::ffi::c_int != 0 {
            glob_flags |= WILD_ICASE as ::core::ffi::c_int;
        }
        if flags & EW_ADDSLASH as ::core::ffi::c_int != 0 {
            glob_flags |= WILD_ADD_SLASH as ::core::ffi::c_int;
        }
        globpath(
            paths,
            pattern,
            gap,
            glob_flags,
            flags & EW_CDPATH as ::core::ffi::c_int != 0,
        );
        xfree(paths as *mut ::core::ffi::c_void);
        return (*gap).ga_len;
    }
}

pub(crate) unsafe extern "C" fn has_env_var(mut p: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        while *p != 0 {
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                p = p.offset(1);
            } else if !vim_strchr(
                b"$\0".as_ptr() as *const ::core::ffi::c_char,
                *p as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
            {
                return true_0 != 0;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn gen_expand_wildcards(
    mut num_pat: ::core::ffi::c_int,
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        let mut add_pat: ::core::ffi::c_int = 0;
        let mut did_expand_in_path: bool = false_0 != 0;
        let mut path_option: *mut ::core::ffi::c_char =
            if *(*curbuf.get()).b_p_path as ::core::ffi::c_int == NUL {
                p_path.get()
            } else {
                (*curbuf.get()).b_p_path
            };
        if recursive.get() {
            return os_expand_wildcards(num_pat, pat, num_file, file, flags);
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_pat {
            if has_special_wildchar(*pat.offset(i as isize), flags) as ::core::ffi::c_int != 0
                && !(vim_backtick(*pat.offset(i as isize)) as ::core::ffi::c_int != 0
                    && *(*pat.offset(i as isize)).offset(1 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int)
            {
                return os_expand_wildcards(num_pat, pat, num_file, file, flags);
            }
            i += 1;
        }
        recursive.set(true_0 != 0);
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            30 as ::core::ffi::c_int,
        );
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < num_pat && !got_int.get() {
            add_pat = -1 as ::core::ffi::c_int;
            p = *pat.offset(i_0 as isize);
            if vim_backtick(p) {
                add_pat = expand_backtick(&raw mut ga, p, flags);
                if add_pat == -1 as ::core::ffi::c_int {
                    recursive.set(false_0 != 0);
                    ga_clear_strings(&raw mut ga);
                    *num_file = 0 as ::core::ffi::c_int;
                    *file = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
                    return FAIL;
                }
            } else {
                if has_env_var(p) as ::core::ffi::c_int != 0
                    && flags & EW_NOTENV as ::core::ffi::c_int == 0
                    || *p as ::core::ffi::c_int == '~' as ::core::ffi::c_int
                {
                    p = expand_env_save_opt(p, true_0 != 0);
                    if p.is_null() {
                        p = *pat.offset(i_0 as isize);
                    } else if has_env_var(p) as ::core::ffi::c_int != 0
                        || *p as ::core::ffi::c_int == '~' as ::core::ffi::c_int
                    {
                        xfree(p as *mut ::core::ffi::c_void);
                        ga_clear_strings(&raw mut ga);
                        i_0 = os_expand_wildcards(
                            num_pat,
                            pat,
                            num_file,
                            file,
                            flags | EW_KEEPDOLLAR as ::core::ffi::c_int,
                        );
                        recursive.set(false_0 != 0);
                        return i_0;
                    }
                }
                if path_has_exp_wildcard(p) as ::core::ffi::c_int != 0
                    || flags & EW_ICASE as ::core::ffi::c_int != 0
                {
                    if flags & (EW_PATH as ::core::ffi::c_int | EW_CDPATH as ::core::ffi::c_int)
                        != 0
                        && !path_is_absolute(p)
                        && !(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                            && (vim_ispathsep(
                                *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            ) as ::core::ffi::c_int
                                != 0
                                || *p.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '.' as ::core::ffi::c_int
                                    && vim_ispathsep(*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0))
                    {
                        recursive.set(false_0 != 0);
                        add_pat = expand_in_path(&raw mut ga, p, flags);
                        recursive.set(true_0 != 0);
                        did_expand_in_path = true_0 != 0;
                    } else {
                        recursive.set(false_0 != 0);
                        let mut tmp_add_pat: size_t = path_expand(&raw mut ga, p, flags);
                        recursive.set(true_0 != 0);
                        '_c2rust_label: {
                            if tmp_add_pat <= 2147483647 as ::core::ffi::c_int as size_t {
                            } else {
                                __assert_fail(
                                b"tmp_add_pat <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                1375 as ::core::ffi::c_uint,
                                b"int gen_expand_wildcards(int, char **, int *, char ***, int)\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            }
                        };
                        add_pat = tmp_add_pat as ::core::ffi::c_int;
                    }
                }
            }
            if add_pat == -1 as ::core::ffi::c_int
                || add_pat == 0 as ::core::ffi::c_int
                    && flags & EW_NOTFOUND as ::core::ffi::c_int != 0
            {
                let mut t: *mut ::core::ffi::c_char = backslash_halve_save(p);
                if flags & EW_NOTFOUND as ::core::ffi::c_int != 0 {
                    addfile(
                        &raw mut ga,
                        t,
                        flags | EW_DIR as ::core::ffi::c_int | EW_FILE as ::core::ffi::c_int,
                    );
                } else {
                    addfile(&raw mut ga, t, flags);
                }
                if t != p {
                    xfree(t as *mut ::core::ffi::c_void);
                }
            }
            if did_expand_in_path as ::core::ffi::c_int != 0
                && !(ga.ga_len <= 0 as ::core::ffi::c_int)
                && flags & (EW_PATH as ::core::ffi::c_int | EW_CDPATH as ::core::ffi::c_int) != 0
            {
                recursive.set(false_0 != 0);
                uniquefy_paths(&raw mut ga, p, path_option);
                recursive.set(true_0 != 0);
            }
            if p != *pat.offset(i_0 as isize) {
                xfree(p as *mut ::core::ffi::c_void);
            }
            i_0 += 1;
        }
        *num_file = ga.ga_len;
        *file = (if !ga.ga_data.is_null() {
            ga.ga_data
        } else {
            NULL
        }) as *mut *mut ::core::ffi::c_char;
        recursive.set(false_0 != 0);
        return if flags & EW_EMPTYOK as ::core::ffi::c_int != 0 || !ga.ga_data.is_null() {
            OK
        } else {
            FAIL
        };
    }
}

pub unsafe extern "C" fn FreeWild(
    mut count: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
) {
    unsafe {
        if count <= 0 as ::core::ffi::c_int || files.is_null() {
            return;
        }
        loop {
            let c2rust_fresh7 = count;
            count = count - 1;
            if c2rust_fresh7 == 0 {
                break;
            }
            xfree(*files.offset(count as isize) as *mut ::core::ffi::c_void);
        }
        xfree(files as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn vim_backtick(mut p: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        return *p as ::core::ffi::c_int == '`' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *p
                .offset(strlen(p) as isize)
                .offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                == '`' as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn expand_backtick(
    mut gap: *mut garray_T,
    mut pat: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut buffer: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cmd: *mut ::core::ffi::c_char = xmemdupz(
            pat.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            strlen(pat).wrapping_sub(2 as size_t),
        ) as *mut ::core::ffi::c_char;
        if *cmd as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
            buffer = eval_to_string(
                cmd.offset(1 as ::core::ffi::c_int as isize),
                true_0 != 0,
                false_0 != 0,
            );
        } else {
            buffer = get_cmd_output(
                cmd,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                if flags & EW_SILENT as ::core::ffi::c_int != 0 {
                    kShellOptSilent as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                },
                ::core::ptr::null_mut::<size_t>(),
            );
        }
        xfree(cmd as *mut ::core::ffi::c_void);
        if buffer.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        cmd = buffer;
        while *cmd as ::core::ffi::c_int != NUL {
            cmd = skipwhite(cmd);
            p = cmd;
            while *p as ::core::ffi::c_int != NUL
                && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            if p > cmd {
                let mut i: ::core::ffi::c_char = *p;
                *p = NUL as ::core::ffi::c_char;
                addfile(gap, cmd, flags);
                *p = i;
                cnt += 1;
            }
            cmd = p;
            while *cmd as ::core::ffi::c_int != NUL
                && (*cmd as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
                    || *cmd as ::core::ffi::c_int == '\n' as ::core::ffi::c_int)
            {
                cmd = cmd.offset(1);
            }
        }
        xfree(buffer as *mut ::core::ffi::c_void);
        return cnt;
    }
}

pub unsafe extern "C" fn expand_wildcards_eval(
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ret: ::core::ffi::c_int = FAIL;
        let mut eval_pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut exp_pat: *mut ::core::ffi::c_char = *pat;
        let mut ignored_msg: *const ::core::ffi::c_char =
            ::core::ptr::null::<::core::ffi::c_char>();
        let mut usedlen: size_t = 0;
        let is_cur_alt_file: bool = *exp_pat as ::core::ffi::c_int == '%' as ::core::ffi::c_int
            || *exp_pat as ::core::ffi::c_int == '#' as ::core::ffi::c_int;
        let mut star_follows: bool = false_0 != 0;
        if is_cur_alt_file as ::core::ffi::c_int != 0
            || *exp_pat as ::core::ffi::c_int == '<' as ::core::ffi::c_int
        {
            (*emsg_off.ptr()) += 1;
            eval_pat = eval_vars(
                exp_pat,
                exp_pat,
                &raw mut usedlen,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut ignored_msg,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                true_0 != 0,
            );
            (*emsg_off.ptr()) -= 1;
            if !eval_pat.is_null() {
                star_follows = strcmp(
                    exp_pat.offset(usedlen as isize),
                    b"*\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int;
                exp_pat = concat_str(eval_pat, exp_pat.offset(usedlen as isize));
            }
        }
        if !exp_pat.is_null() {
            ret = expand_wildcards(
                1 as ::core::ffi::c_int,
                &raw mut exp_pat,
                num_file,
                file,
                flags,
            );
        }
        if !eval_pat.is_null() {
            if *num_file == 0 as ::core::ffi::c_int
                && is_cur_alt_file as ::core::ffi::c_int != 0
                && star_follows as ::core::ffi::c_int != 0
            {
                *file = xmalloc(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    as *mut *mut ::core::ffi::c_char;
                **file = eval_pat;
                eval_pat = ::core::ptr::null_mut::<::core::ffi::c_char>();
                *num_file = 1 as ::core::ffi::c_int;
                ret = OK;
            }
            xfree(exp_pat as *mut ::core::ffi::c_void);
            xfree(eval_pat as *mut ::core::ffi::c_void);
        }
        return ret;
    }
}

pub unsafe extern "C" fn expand_wildcards(
    mut num_pat: ::core::ffi::c_int,
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_files: *mut ::core::ffi::c_int,
    mut files: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int =
            gen_expand_wildcards(num_pat, pat, num_files, files, flags);
        if flags & EW_KEEPALL as ::core::ffi::c_int != 0 || retval == FAIL {
            return retval;
        }
        if *p_wig.get() != 0 {
            '_c2rust_label: {
                if *num_files == 0 as ::core::ffi::c_int || !(*files).is_null() {
                } else {
                    __assert_fail(
                        b"*num_files == 0 || *files != NULL\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2221 as ::core::ffi::c_uint,
                        b"int expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < *num_files {
                let mut ffname: *mut ::core::ffi::c_char =
                    FullName_save(*(*files).offset(i as isize), false_0 != 0);
                '_c2rust_label_0: {
                    if !(*(*files).offset(i as isize)).is_null() {
                    } else {
                        __assert_fail(
                            b"(*files)[i] != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            2224 as ::core::ffi::c_uint,
                            b"int expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                '_c2rust_label_1: {
                    if !ffname.is_null() {
                    } else {
                        __assert_fail(
                            b"ffname != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            2225 as ::core::ffi::c_uint,
                            b"int expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                if match_file_list(p_wig.get(), *(*files).offset(i as isize), ffname) {
                    xfree(*(*files).offset(i as isize) as *mut ::core::ffi::c_void);
                    let mut j: ::core::ffi::c_int = i;
                    while (j + 1 as ::core::ffi::c_int) < *num_files {
                        *(*files).offset(j as isize) =
                            *(*files).offset((j + 1 as ::core::ffi::c_int) as isize);
                        j += 1;
                    }
                    *num_files -= 1;
                    i -= 1;
                }
                xfree(ffname as *mut ::core::ffi::c_void);
                i += 1;
            }
        }
        '_c2rust_label_2: {
            if *num_files == 0 as ::core::ffi::c_int || !(*files).is_null() {
            } else {
                __assert_fail(
                    b"*num_files == 0 || *files != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2241 as ::core::ffi::c_uint,
                    b"int expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if *num_files > 1 as ::core::ffi::c_int && !got_int.get() {
            let mut non_suf_match: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < *num_files {
                if !match_suffix(*(*files).offset(i_0 as isize)) {
                    let mut p: *mut ::core::ffi::c_char = *(*files).offset(i_0 as isize);
                    let mut j_0: ::core::ffi::c_int = i_0;
                    while j_0 > non_suf_match {
                        *(*files).offset(j_0 as isize) =
                            *(*files).offset((j_0 - 1 as ::core::ffi::c_int) as isize);
                        j_0 -= 1;
                    }
                    let c2rust_fresh8 = non_suf_match;
                    non_suf_match = non_suf_match + 1;
                    let c2rust_lvalue_ptr = &raw mut *(*files).offset(c2rust_fresh8 as isize);
                    *c2rust_lvalue_ptr = p;
                }
                i_0 += 1;
            }
        }
        if *num_files == 0 as ::core::ffi::c_int {
            let mut ptr_: *mut *mut ::core::ffi::c_void = files as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            return FAIL;
        }
        return retval;
    }
}
