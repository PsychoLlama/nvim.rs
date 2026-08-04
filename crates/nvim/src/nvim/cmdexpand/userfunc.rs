//! Match sources that run user code or walk the file system.
//!
//! `'shellcmd'` completion ([`expand_shellcmd`]) walks `$PATH`;
//! [`globpath`] walks a comma-separated directory list; and the
//! `custom,`/`customlist,`/Lua completion functions of `:command` are called
//! through [`ExpandUserDefined`], [`ExpandUserList`] and [`ExpandUserLua`].

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn expand_shellcmd_onedir(
    mut pathed_pattern: *mut ::core::ffi::c_char,
    mut pathlen: size_t,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut ht: *mut hashtab_T,
    mut gap: *mut garray_T,
) {
    unsafe {
        if expand_wildcards(
            1 as ::core::ffi::c_int,
            &raw mut pathed_pattern,
            numMatches,
            matches,
            flags,
        ) != OK
        {
            return;
        }
        ga_grow(gap, *numMatches);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < *numMatches {
            let mut name: *mut ::core::ffi::c_char = *(*matches).offset(i as isize);
            let mut namelen: size_t = strlen(name);
            if namelen > pathlen {
                let mut hash: hash_T = hash_hash(name.offset(pathlen as isize));
                let mut hi: *mut hashitem_T = hash_lookup(
                    ht,
                    name.offset(pathlen as isize),
                    namelen.wrapping_sub(pathlen),
                    hash,
                );
                if (*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
                {
                    memmove(
                        name as *mut ::core::ffi::c_void,
                        name.offset(pathlen as isize) as *const ::core::ffi::c_void,
                        namelen.wrapping_sub(pathlen).wrapping_add(1 as size_t),
                    );
                    let c2rust_fresh2 = (*gap).ga_len;
                    (*gap).ga_len = (*gap).ga_len + 1;
                    let c2rust_lvalue_ptr = &raw mut *((*gap).ga_data
                        as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh2 as isize);
                    *c2rust_lvalue_ptr = name;
                    hash_add_item(ht, hi, name, hash);
                    name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            }
            xfree(name as *mut ::core::ffi::c_void);
            i += 1;
        }
        xfree(*matches as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn expand_shellcmd(
    mut filepat: *mut ::core::ffi::c_char,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut flagsarg: ::core::ffi::c_int,
) {
    unsafe {
        let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut buf: *mut ::core::ffi::c_char =
            xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        let mut flags: ::core::ffi::c_int = flagsarg;
        let mut did_curdir: bool = false_0 != 0;
        let mut patlen: size_t = strlen(filepat);
        let mut pat: *mut ::core::ffi::c_char =
            xmemdupz(filepat as *const ::core::ffi::c_void, patlen) as *mut ::core::ffi::c_char;
        let mut e: *mut ::core::ffi::c_char = pat.offset(patlen as isize);
        let mut s: *mut ::core::ffi::c_char = pat;
        while *s as ::core::ffi::c_int != NUL {
            if *s as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                let mut p: *mut ::core::ffi::c_char = s.offset(1 as ::core::ffi::c_int as isize);
                if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                    memmove(
                        s as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        (e.offset_from(p) as size_t).wrapping_add(1 as size_t),
                    );
                    e = e.offset(-1);
                }
            }
            s = s.offset(1);
        }
        patlen = e.offset_from(pat) as size_t;
        flags |= EW_FILE | EW_EXEC | EW_SHELLCMD;
        let mut mustfree: bool = false_0 != 0;
        if *pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && (vim_ispathsep(*pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                || *pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                    && vim_ispathsep(
                        *pat.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0)
        {
            path = b".\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            if !path_is_absolute(pat) {
                path = vim_getenv(b"PATH\0".as_ptr() as *const ::core::ffi::c_char);
            }
            if path.is_null() {
                path = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                mustfree = true_0 != 0;
            }
        }
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            10 as ::core::ffi::c_int,
        );
        let mut found_ht: hashtab_T = hashtab_T {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ::core::ptr::null_mut::<hashitem_T>(),
            ht_smallarray: [hashitem_T {
                hi_hash: 0,
                hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            }; 16],
        };
        hash_init(&raw mut found_ht);
        let mut s_0: *mut ::core::ffi::c_char = path;
        loop {
            let mut pathlen: size_t = 0;
            let mut seplen: size_t = 0;
            if *s_0 as ::core::ffi::c_int == NUL {
                if did_curdir {
                    break;
                }
                did_curdir = true_0 != 0;
                flags |= EW_DIR;
                e = s_0;
                pathlen = 0 as size_t;
                seplen = 0 as size_t;
            } else {
                e = vim_strchr(s_0, ENV_SEPCHAR);
                if e.is_null() {
                    e = s_0.offset(strlen(s_0) as isize);
                }
                pathlen = e.offset_from(s_0) as size_t;
                if strncmp(s_0, b".\0".as_ptr() as *const ::core::ffi::c_char, pathlen)
                    == 0 as ::core::ffi::c_int
                {
                    did_curdir = true_0 != 0;
                    flags |= EW_DIR;
                } else {
                    flags &= !(EW_DIR);
                }
                seplen = (if after_pathsep(s_0, e) == 0 {
                    ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1_usize)
                } else {
                    0_usize
                }) as size_t;
            }
            if pathlen
                .wrapping_add(seplen)
                .wrapping_add(patlen)
                .wrapping_add(1 as size_t)
                <= MAXPATHL as size_t
            {
                if pathlen > 0 as size_t {
                    xmemcpyz(
                        buf as *mut ::core::ffi::c_void,
                        s_0 as *const ::core::ffi::c_void,
                        pathlen,
                    );
                    if seplen > 0 as size_t {
                        xmemcpyz(
                            buf.offset(pathlen as isize) as *mut ::core::ffi::c_void,
                            b"/\0".as_ptr() as *const ::core::ffi::c_char
                                as *const ::core::ffi::c_void,
                            ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        );
                        pathlen = pathlen.wrapping_add(seplen);
                    }
                }
                xmemcpyz(
                    buf.offset(pathlen as isize) as *mut ::core::ffi::c_void,
                    pat as *const ::core::ffi::c_void,
                    patlen,
                );
                expand_shellcmd_onedir(
                    buf,
                    pathlen,
                    matches,
                    numMatches,
                    flags,
                    &raw mut found_ht,
                    &raw mut ga,
                );
            }
            if *e as ::core::ffi::c_int != NUL {
                e = e.offset(1);
            }
            s_0 = e;
        }
        *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
        *numMatches = ga.ga_len;
        xfree(buf as *mut ::core::ffi::c_void);
        xfree(pat as *mut ::core::ffi::c_void);
        if mustfree {
            xfree(path as *mut ::core::ffi::c_void);
        }
        hash_clear(&raw mut found_ht);
    }
}

pub(crate) unsafe extern "C" fn call_user_expand_func(
    mut user_expand_func: user_expand_func_T,
    mut xp: *mut expand_T,
) -> *mut ::core::ffi::c_void {
    unsafe {
        let ccline: *mut CmdlineInfo = get_cmdline_info();
        let mut keep: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
        let mut args: [typval_T; 4] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 4];
        let save_current_sctx: sctx_T = current_sctx.get();
        if (*xp).xp_arg.is_null()
            || *(*xp).xp_arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || (*xp).xp_line.is_null()
        {
            return NULL;
        }
        if !(*ccline).cmdbuff.is_null() {
            keep = *(*ccline).cmdbuff.offset((*ccline).cmdlen as isize);
            *(*ccline).cmdbuff.offset((*ccline).cmdlen as isize) = 0 as ::core::ffi::c_char;
        }
        let mut pat: *mut ::core::ffi::c_char = xstrnsave((*xp).xp_pattern, (*xp).xp_pattern_len);
        args[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        args[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        args[2 as ::core::ffi::c_int as usize].v_type = VAR_NUMBER;
        args[3 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
        args[0 as ::core::ffi::c_int as usize].vval.v_string = pat;
        args[1 as ::core::ffi::c_int as usize].vval.v_string = (*xp).xp_line;
        args[2 as ::core::ffi::c_int as usize].vval.v_number = (*xp).xp_col as varnumber_T;
        current_sctx.set((*xp).xp_script_ctx);
        let ret: *mut ::core::ffi::c_void = user_expand_func.expect("non-null function pointer")(
            (*xp).xp_arg,
            3 as ::core::ffi::c_int,
            &raw mut args as *mut typval_T,
        );
        current_sctx.set(save_current_sctx);
        if !(*ccline).cmdbuff.is_null() {
            *(*ccline).cmdbuff.offset((*ccline).cmdlen as isize) = keep;
        }
        xfree(pat as *mut ::core::ffi::c_void);
        return ret;
    }
}

pub(crate) unsafe extern "C" fn ExpandUserDefined(
    pat: *const ::core::ffi::c_char,
    mut xp: *mut expand_T,
    mut regmatch: *mut regmatch_T,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let fuzzy: bool = cmdline_fuzzy_complete(pat);
        *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        *numMatches = 0 as ::core::ffi::c_int;
        let retstr: *mut ::core::ffi::c_char = call_user_expand_func(
            Some(
                call_func_retstr
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_char,
                        ::core::ffi::c_int,
                        *mut typval_T,
                    ) -> *mut ::core::ffi::c_void,
            ),
            xp,
        ) as *mut ::core::ffi::c_char;
        if retstr.is_null() {
            return FAIL;
        }
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        if !fuzzy {
            ga_init(
                &raw mut ga,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                3 as ::core::ffi::c_int,
            );
        } else {
            ga_init(
                &raw mut ga,
                ::core::mem::size_of::<fuzmatch_str_T>() as ::core::ffi::c_int,
                3 as ::core::ffi::c_int,
            );
        }
        let mut s: *mut ::core::ffi::c_char = retstr;
        let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        while *s as ::core::ffi::c_int != NUL {
            e = vim_strchr(s, '\n' as ::core::ffi::c_int);
            if e.is_null() {
                e = s.offset(strlen(s) as isize);
            }
            let keep: ::core::ffi::c_char = *e;
            *e = NUL as ::core::ffi::c_char;
            let mut match_0: bool = false;
            let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if *(*xp).xp_pattern.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != NUL
            {
                if !fuzzy {
                    match_0 = vim_regexec(regmatch, s, 0 as colnr_T);
                } else {
                    score = fuzzy_match_str(s, pat);
                    match_0 = score != FUZZY_SCORE_NONE as ::core::ffi::c_int;
                }
            } else {
                match_0 = true_0 != 0;
            }
            *e = keep;
            if match_0 {
                let mut p: *mut ::core::ffi::c_char =
                    xmemdupz(s as *const ::core::ffi::c_void, e.offset_from(s) as size_t)
                        as *mut ::core::ffi::c_char;
                if !fuzzy {
                    ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                    *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) = p;
                    ga.ga_len += 1;
                } else {
                    ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                    *(ga.ga_data as *mut fuzmatch_str_T).offset(ga.ga_len as isize) =
                        fuzmatch_str_T {
                            idx: ga.ga_len,
                            str: p,
                            score: score,
                        };
                    ga.ga_len += 1;
                }
            }
            if *e as ::core::ffi::c_int != NUL {
                e = e.offset(1);
            }
            s = e;
        }
        xfree(retstr as *mut ::core::ffi::c_void);
        if ga.ga_len == 0 as ::core::ffi::c_int {
            return OK;
        }
        if !fuzzy {
            *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
            *numMatches = ga.ga_len;
        } else {
            fuzzymatches_to_strmatches(
                ga.ga_data as *mut fuzmatch_str_T,
                matches,
                ga.ga_len,
                false_0 != 0,
            );
            *numMatches = ga.ga_len;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn process_user_list(
    mut retlist: *mut list_T,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
) {
    unsafe {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            3 as ::core::ffi::c_int,
        );
        let l_: *const list_T = retlist;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                if !((*li).li_tv.v_type as ::core::ffi::c_uint
                    != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*li).li_tv.vval.v_string.is_null())
                {
                    let mut p: *mut ::core::ffi::c_char = xstrdup((*li).li_tv.vval.v_string);
                    ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                    *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) = p;
                    ga.ga_len += 1;
                }
                li = (*li).li_next;
            }
        }
        tv_list_unref(retlist);
        *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
        *numMatches = ga.ga_len;
    }
}

pub(crate) unsafe extern "C" fn ExpandUserList(
    mut xp: *mut expand_T,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        *numMatches = 0 as ::core::ffi::c_int;
        let retlist: *mut list_T = call_user_expand_func(
            Some(
                call_func_retlist
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_char,
                        ::core::ffi::c_int,
                        *mut typval_T,
                    ) -> *mut ::core::ffi::c_void,
            ),
            xp,
        ) as *mut list_T;
        if retlist.is_null() {
            return FAIL;
        }
        process_user_list(retlist, matches, numMatches);
        return OK;
    }
}

pub(crate) unsafe extern "C" fn ExpandUserLua(
    mut xp: *mut expand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        nlua_call_user_expand_func(xp, &raw mut rettv);
        if rettv.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_clear(&raw mut rettv);
            return FAIL;
        }
        let retlist: *mut list_T = rettv.vval.v_list;
        process_user_list(retlist, matches, numMatches);
        return OK;
    }
}

pub unsafe extern "C" fn globpath(
    mut path: *mut ::core::ffi::c_char,
    mut file: *mut ::core::ffi::c_char,
    mut ga: *mut garray_T,
    mut expand_options: ::core::ffi::c_int,
    mut dirs: bool,
) {
    unsafe {
        let mut buf: *mut ::core::ffi::c_char =
            xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        let mut xpc: expand_T = expand_T {
            xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            xp_context: 0,
            xp_pattern_len: 0,
            xp_prefix: XP_PREFIX_NONE,
            xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
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
            xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            xp_buf: [0; 256],
            xp_search_dir: kDirectionNotSet,
            xp_pre_incsearch_pos: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
        };
        ExpandInit(&raw mut xpc);
        xpc.xp_context = if dirs as ::core::ffi::c_int != 0 {
            EXPAND_DIRECTORIES
        } else {
            EXPAND_FILES
        };
        let mut filelen: size_t = strlen(file);
        while *path as ::core::ffi::c_int != NUL {
            let mut pathlen: size_t = copy_option_part(
                &raw mut path,
                buf,
                MAXPATHL as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            let mut seplen: size_t = if *buf as ::core::ffi::c_int != NUL
                && after_pathsep(buf, buf.offset(pathlen as isize)) == 0
            {
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t)
            } else {
                0 as size_t
            };
            if pathlen
                .wrapping_add(seplen)
                .wrapping_add(filelen)
                .wrapping_add(1 as size_t)
                <= MAXPATHL as size_t
            {
                if seplen > 0 as size_t {
                    xmemcpyz(
                        buf.offset(pathlen as isize) as *mut ::core::ffi::c_void,
                        b"/\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                            .wrapping_sub(1 as size_t),
                    );
                    pathlen = pathlen.wrapping_add(seplen);
                }
                xmemcpyz(
                    buf.offset(pathlen as isize) as *mut ::core::ffi::c_void,
                    file as *const ::core::ffi::c_void,
                    filelen,
                );
                let mut p: *mut *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
                let mut num_p: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                ExpandFromContext(
                    &raw mut xpc,
                    buf,
                    &raw mut p,
                    &raw mut num_p,
                    WILD_SILENT | expand_options,
                );
                if num_p > 0 as ::core::ffi::c_int {
                    escape_matches(
                        &raw mut xpc,
                        buf,
                        core::slice::from_raw_parts_mut(p, num_p as usize),
                        WILD_SILENT | expand_options,
                    );
                    ga_grow(ga, num_p);
                    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i < num_p {
                        *((*ga).ga_data as *mut *mut ::core::ffi::c_char)
                            .offset((*ga).ga_len as isize) = *p.offset(i as isize);
                        (*ga).ga_len += 1;
                        i += 1;
                    }
                    xfree(p as *mut ::core::ffi::c_void);
                }
            }
        }
        xfree(buf as *mut ::core::ffi::c_void);
    }
}
