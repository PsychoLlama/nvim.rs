//! Turning a context into a match array.
//!
//! [`ExpandFromContext`] is the dispatcher: file-like contexts go to
//! `expand_wildcards`, everything else to a generator, and the answer is
//! sorted, deduplicated and escaped.  [`ExpandGeneric`] is the generic
//! generator loop every `get_*_name` callback is driven by, and
//! [`map_wildopts_to_ewflags`] translates `'wildoptions'` into `EW_*`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn map_wildopts_to_ewflags(
    mut options: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = EW_DIR;
    if options & WILD_LIST_NOTFOUND != 0 {
        flags |= EW_NOTFOUND;
    }
    if options & WILD_ADD_SLASH != 0 {
        flags |= EW_ADDSLASH;
    }
    if options & WILD_KEEP_ALL != 0 {
        flags |= EW_KEEPALL;
    }
    if options & WILD_SILENT != 0 {
        flags |= EW_SILENT;
    }
    if options & WILD_NOERROR != 0 {
        flags |= EW_NOERROR;
    }
    if options & WILD_ALLLINKS != 0 {
        flags |= EW_ALLLINKS;
    }
    return flags;
}

pub(crate) unsafe extern "C" fn ExpandFromContext(
    mut xp: *mut expand_T,
    mut pat: *mut ::core::ffi::c_char,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false_0 != 0,
        };
        let mut ret: ::core::ffi::c_int = 0;
        let mut flags: ::core::ffi::c_int = map_wildopts_to_ewflags(options);
        let fuzzy: bool = cmdline_fuzzy_complete(pat) as ::core::ffi::c_int != 0
            && cmdline_fuzzy_completion_supported(xp) as ::core::ffi::c_int != 0;
        if (*xp).xp_context == EXPAND_FILES
            || (*xp).xp_context == EXPAND_DIRECTORIES
            || (*xp).xp_context == EXPAND_FILES_IN_PATH
            || (*xp).xp_context == EXPAND_FINDFUNC
            || (*xp).xp_context == EXPAND_DIRS_IN_CDPATH
        {
            return expand_files_and_dirs(xp, pat, matches, numMatches, flags, options);
        }
        *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        *numMatches = 0 as ::core::ffi::c_int;
        if (*xp).xp_context == EXPAND_HELP {
            if find_help_tags(
                if *pat as ::core::ffi::c_int == NUL {
                    b"help\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    pat as *const ::core::ffi::c_char
                },
                numMatches,
                matches,
                false_0 != 0,
            ) == OK
            {
                cleanup_help_tags(*numMatches, *matches);
                return OK;
            }
            return FAIL;
        }
        if (*xp).xp_context == EXPAND_SHELLCMD {
            expand_shellcmd(pat, matches, numMatches, flags);
            return OK;
        }
        if (*xp).xp_context == EXPAND_OLD_SETTING {
            return ExpandOldSetting(numMatches, matches);
        }
        if (*xp).xp_context == EXPAND_BUFFERS {
            return ExpandBufnames(pat, numMatches, matches, options);
        }
        if (*xp).xp_context == EXPAND_DIFF_BUFFERS {
            return ExpandBufnames(pat, numMatches, matches, options | BUF_DIFF_FILTER);
        }
        if (*xp).xp_context == EXPAND_TAGS || (*xp).xp_context == EXPAND_TAGS_LISTFILES {
            return expand_tags((*xp).xp_context == EXPAND_TAGS, pat, numMatches, matches);
        }
        if (*xp).xp_context == EXPAND_COLORS {
            let mut directories: [*mut ::core::ffi::c_char; 2] = [
                b"colors\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return ExpandRTDir(
                pat,
                DIP_START as ::core::ffi::c_int + DIP_OPT as ::core::ffi::c_int,
                numMatches,
                matches,
                &raw mut directories as *mut *mut ::core::ffi::c_char,
            );
        }
        if (*xp).xp_context == EXPAND_COMPILER {
            let mut directories_0: [*mut ::core::ffi::c_char; 2] = [
                b"compiler\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return ExpandRTDir(
                pat,
                0 as ::core::ffi::c_int,
                numMatches,
                matches,
                &raw mut directories_0 as *mut *mut ::core::ffi::c_char,
            );
        }
        if (*xp).xp_context == EXPAND_OWNSYNTAX {
            let mut directories_1: [*mut ::core::ffi::c_char; 2] = [
                b"syntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return ExpandRTDir(
                pat,
                0 as ::core::ffi::c_int,
                numMatches,
                matches,
                &raw mut directories_1 as *mut *mut ::core::ffi::c_char,
            );
        }
        if (*xp).xp_context == EXPAND_FILETYPE {
            let mut directories_2: [*mut ::core::ffi::c_char; 4] = [
                b"syntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"indent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"ftplugin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return ExpandRTDir(
                pat,
                0 as ::core::ffi::c_int,
                numMatches,
                matches,
                &raw mut directories_2 as *mut *mut ::core::ffi::c_char,
            );
        }
        if (*xp).xp_context == EXPAND_KEYMAP {
            let mut directories_3: [*mut ::core::ffi::c_char; 2] = [
                b"keymap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return ExpandRTDir(
                pat,
                0 as ::core::ffi::c_int,
                numMatches,
                matches,
                &raw mut directories_3 as *mut *mut ::core::ffi::c_char,
            );
        }
        if (*xp).xp_context == EXPAND_USER_LIST {
            return ExpandUserList(xp, matches, numMatches);
        }
        if (*xp).xp_context == EXPAND_USER_LUA {
            return ExpandUserLua(xp, numMatches, matches);
        }
        if (*xp).xp_context == EXPAND_PACKADD {
            return ExpandPackAddDir(pat, numMatches, matches);
        }
        if (*xp).xp_context == EXPAND_RUNTIME {
            return expand_runtime_cmd(pat, numMatches, matches);
        }
        if (*xp).xp_context == EXPAND_PATTERN_IN_BUF {
            return expand_pattern_in_buf(pat, (*xp).xp_search_dir, matches, numMatches);
        }
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*xp).xp_context == EXPAND_USER_FUNC
            && strncmp(
                pat,
                b"^s:\0".as_ptr() as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            let len: size_t = strlen(pat).wrapping_add(20 as size_t);
            tofree = xmalloc(len) as *mut ::core::ffi::c_char;
            snprintf(
                tofree,
                len,
                b"^<SNR>\\d\\+_%s\0".as_ptr() as *const ::core::ffi::c_char,
                pat.offset(3 as ::core::ffi::c_int as isize),
            );
            pat = tofree;
        }
        if (*xp).xp_context == EXPAND_LUA {
            return nlua_expand_get_matches(numMatches, matches);
        }
        if !fuzzy {
            regmatch.regprog = vim_regcomp(
                pat,
                if magic_isset() as ::core::ffi::c_int != 0 {
                    RE_MAGIC
                } else {
                    0 as ::core::ffi::c_int
                },
            );
            if regmatch.regprog.is_null() {
                xfree(tofree as *mut ::core::ffi::c_void);
                return FAIL;
            }
            regmatch.rm_ic = ignorecase(pat) != 0;
        }
        if (*xp).xp_context == EXPAND_SETTINGS || (*xp).xp_context == EXPAND_BOOL_SETTINGS {
            ret = ExpandSettings(xp, &raw mut regmatch, pat, numMatches, matches, fuzzy);
        } else if (*xp).xp_context == EXPAND_STRING_SETTING {
            ret = ExpandStringSetting(xp, &raw mut regmatch, numMatches, matches);
        } else if (*xp).xp_context == EXPAND_SETTING_SUBTRACT {
            ret = ExpandSettingSubtract(xp, &raw mut regmatch, numMatches, matches);
        } else if (*xp).xp_context == EXPAND_MAPPINGS {
            ret = ExpandMappings(pat, &raw mut regmatch, numMatches, matches);
        } else if (*xp).xp_context == EXPAND_ARGOPT {
            ret = expand_argopt(pat, xp, &raw mut regmatch, matches, numMatches);
        } else if (*xp).xp_context == EXPAND_USER_DEFINED {
            ret = ExpandUserDefined(pat, xp, &raw mut regmatch, matches, numMatches);
        } else {
            ret = ExpandOther(pat, xp, &raw mut regmatch, matches, numMatches);
        }
        if !fuzzy {
            vim_regfree(regmatch.regprog);
        }
        xfree(tofree as *mut ::core::ffi::c_void);
        return ret;
    }
}

pub unsafe extern "C" fn ExpandGeneric(
    pat: *const ::core::ffi::c_char,
    mut xp: *mut expand_T,
    mut regmatch: *mut regmatch_T,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut func: CompleteListItemGetter,
    mut escaped: bool,
) {
    unsafe {
        let fuzzy: bool = cmdline_fuzzy_complete(pat);
        *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        *numMatches = 0 as ::core::ffi::c_int;
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
                30 as ::core::ffi::c_int,
            );
        } else {
            ga_init(
                &raw mut ga,
                ::core::mem::size_of::<fuzmatch_str_T>() as ::core::ffi::c_int,
                30 as ::core::ffi::c_int,
            );
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            let mut str: *mut ::core::ffi::c_char = Some(func.expect("non-null function pointer"))
                .expect("non-null function pointer")(
                xp, i
            );
            if str.is_null() {
                break;
            }
            if *str as ::core::ffi::c_int != NUL {
                let mut match_0: bool = false;
                let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if *(*xp).xp_pattern.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != NUL
                {
                    if !fuzzy {
                        match_0 = vim_regexec(regmatch, str, 0 as colnr_T);
                    } else {
                        score = fuzzy_match_str(str, pat);
                        match_0 = score != FUZZY_SCORE_NONE as ::core::ffi::c_int;
                    }
                } else {
                    match_0 = true_0 != 0;
                }
                if match_0 {
                    if escaped {
                        str = vim_strsave_escaped(
                            str,
                            b" \t\\.\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    } else {
                        str = xstrdup(str);
                    }
                    if fuzzy {
                        ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                        *(ga.ga_data as *mut fuzmatch_str_T).offset(ga.ga_len as isize) =
                            fuzmatch_str_T {
                                idx: ga.ga_len,
                                str: str,
                                score: score,
                            };
                        ga.ga_len += 1;
                    } else {
                        ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                        *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) =
                            str;
                        ga.ga_len += 1;
                    }
                    if func.is_some_and(|f| {
                        ::core::ptr::fn_addr_eq(
                            f,
                            get_menu_names
                                as unsafe extern "C" fn(
                                    *mut expand_T,
                                    ::core::ffi::c_int,
                                )
                                    -> *mut ::core::ffi::c_char,
                        )
                    }) {
                        str = str.offset(strlen(str).wrapping_sub(1 as size_t) as isize);
                        if *str as ::core::ffi::c_int == '\u{1}' as ::core::ffi::c_int {
                            *str = '.' as ::core::ffi::c_char;
                        }
                    }
                }
            }
            i += 1;
        }
        if ga.ga_len == 0 as ::core::ffi::c_int {
            return;
        }
        let sort_matches: bool = !fuzzy
            && (*xp).xp_context != EXPAND_MENUNAMES
            && (*xp).xp_context != EXPAND_STRING_SETTING
            && (*xp).xp_context != EXPAND_MENUS
            && (*xp).xp_context != EXPAND_SCRIPTNAMES
            && (*xp).xp_context != EXPAND_ARGOPT;
        let funcsort: bool = (*xp).xp_context == EXPAND_EXPRESSION
            || (*xp).xp_context == EXPAND_FUNCTIONS
            || (*xp).xp_context == EXPAND_USER_FUNC;
        if sort_matches {
            if funcsort {
                qsort(
                    ga.ga_data,
                    ga.ga_len as size_t,
                    ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
                    Some(
                        sort_func_compare
                            as unsafe extern "C" fn(
                                *const ::core::ffi::c_void,
                                *const ::core::ffi::c_void,
                            )
                                -> ::core::ffi::c_int,
                    ),
                );
            } else {
                sort_strings(ga.ga_data as *mut *mut ::core::ffi::c_char, ga.ga_len);
            }
        }
        if !fuzzy {
            *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
            *numMatches = ga.ga_len;
        } else {
            fuzzymatches_to_strmatches(
                ga.ga_data as *mut fuzmatch_str_T,
                matches,
                ga.ga_len,
                funcsort,
            );
            *numMatches = ga.ga_len;
        }
        reset_expand_highlight();
    }
}
