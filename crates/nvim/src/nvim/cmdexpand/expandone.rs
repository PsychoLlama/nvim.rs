//! The wildcard key: one `<Tab>` press, from key to command line.
//!
//! [`nextwild`] is what the command-line key loop calls; it isolates the word
//! under the cursor, hands it to [`ExpandOne`] and puts the answer back.
//! [`ExpandOne`] owns the match array across presses — [`ExpandOne_start`]
//! fills it, [`get_next_or_prev_match`] cycles it and [`find_longest_match`]
//! computes the `'wildmode'`=longest answer.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nextwild(
    mut xp: *mut expand_T,
    mut type_0: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
    mut escape: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let ccline: *mut CmdlineInfo = get_cmdline_info();
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut from_wildtrigger_func: bool =
            options & WILD_FUNC_TRIGGER as ::core::ffi::c_int != 0;
        let mut wild_navigate: bool = type_0 == WILD_NEXT as ::core::ffi::c_int
            || type_0 == WILD_PREV as ::core::ffi::c_int
            || type_0 == WILD_PAGEUP as ::core::ffi::c_int
            || type_0 == WILD_PAGEDOWN as ::core::ffi::c_int
            || type_0 == WILD_PUM_WANT as ::core::ffi::c_int;
        if (*xp).xp_numfiles == -1 as ::core::ffi::c_int {
            pre_incsearch_pos.set((*xp).xp_pre_incsearch_pos);
            if (*ccline).input_fn != 0
                && (*ccline).xp_context == EXPAND_COMMANDS as ::core::ffi::c_int
            {
                set_cmd_context(
                    xp,
                    (*ccline).cmdbuff,
                    (*ccline).cmdlen,
                    (*ccline).cmdpos,
                    false_0,
                );
            } else {
                may_expand_pattern
                    .set(options & WILD_MAY_EXPAND_PATTERN as ::core::ffi::c_int != 0);
                set_expand_context(xp);
                may_expand_pattern.set(false_0 != 0);
            }
            if (*xp).xp_context == EXPAND_LUA as ::core::ffi::c_int {
                nlua_expand_pat(xp);
            }
            cmd_showtail.set(expand_showtail(xp));
        }
        if (*xp).xp_context == EXPAND_UNSUCCESSFUL as ::core::ffi::c_int {
            beep_flush();
            return OK;
        }
        if (*xp).xp_context == EXPAND_NOTHING as ::core::ffi::c_int {
            return FAIL;
        }
        let mut i: ::core::ffi::c_int =
            (*xp).xp_pattern.offset_from((*ccline).cmdbuff) as ::core::ffi::c_int;
        '_c2rust_label: {
            if (*ccline).cmdpos >= i {
            } else {
                __assert_fail(
                    b"ccline->cmdpos >= i\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    288 as ::core::ffi::c_uint,
                    b"int nextwild(expand_T *, int, int, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        (*xp).xp_pattern_len = ((*ccline).cmdpos as size_t).wrapping_sub(i as size_t);
        if from_wildtrigger_func as ::core::ffi::c_int != 0
            && (*xp).xp_context == EXPAND_COMMANDS as ::core::ffi::c_int
            && (*xp).xp_pattern_len == 0 as size_t
        {
            return FAIL;
        }
        if !cmd_silent.get()
            && !from_wildtrigger_func
            && !wild_navigate
            && !(ui_has(kUICmdline) as ::core::ffi::c_int != 0
                || ui_has(kUIWildmenu) as ::core::ffi::c_int != 0)
        {
            msg_puts(b"...\0".as_ptr() as *const ::core::ffi::c_char);
            ui_flush();
        }
        if wild_navigate {
            p = ExpandOne(
                xp,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
                type_0,
            );
        } else {
            let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if cmdline_fuzzy_completion_supported(xp) as ::core::ffi::c_int != 0
                || (*xp).xp_context == EXPAND_PATTERN_IN_BUF as ::core::ffi::c_int
            {
                tmp = xstrnsave((*xp).xp_pattern, (*xp).xp_pattern_len);
            } else {
                tmp = addstar((*xp).xp_pattern, (*xp).xp_pattern_len, (*xp).xp_context);
            }
            let use_options: ::core::ffi::c_int = options
                | WILD_HOME_REPLACE as ::core::ffi::c_int
                | WILD_ADD_SLASH as ::core::ffi::c_int
                | WILD_SILENT as ::core::ffi::c_int
                | (if escape as ::core::ffi::c_int != 0 {
                    WILD_ESCAPE as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                | (if p_wic.get() != 0 {
                    WILD_ICASE as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                });
            p = ExpandOne(
                xp,
                tmp,
                xstrnsave((*ccline).cmdbuff.offset(i as isize), (*xp).xp_pattern_len),
                use_options,
                type_0,
            );
            xfree(tmp as *mut ::core::ffi::c_void);
            if !p.is_null() && type_0 == WILD_LONGEST as ::core::ffi::c_int {
                let mut j: ::core::ffi::c_int = 0;
                j = 0 as ::core::ffi::c_int;
                while (j as size_t) < (*xp).xp_pattern_len {
                    let mut c: ::core::ffi::c_char = *(*ccline).cmdbuff.offset((i + j) as isize);
                    if c as ::core::ffi::c_int == '*' as ::core::ffi::c_int
                        || c as ::core::ffi::c_int == '?' as ::core::ffi::c_int
                    {
                        break;
                    }
                    j += 1;
                }
                if (strlen(p) as ::core::ffi::c_int) < j {
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut p as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    let _ = *ptr_;
                }
            }
        }
        if !wild_navigate && !(*ccline).cmdbuff.is_null() {
            xfree(cmdline_orig.get() as *mut ::core::ffi::c_void);
            cmdline_orig.set(xstrnsave((*ccline).cmdbuff, (*ccline).cmdlen as size_t));
        }
        if !p.is_null() && !got_int.get() && options & WILD_NOSELECT as ::core::ffi::c_int == 0 {
            let mut plen: size_t = strlen(p);
            let mut difflen: ::core::ffi::c_int =
                plen as ::core::ffi::c_int - (*xp).xp_pattern_len as ::core::ffi::c_int;
            if (*ccline).cmdlen + difflen + 4 as ::core::ffi::c_int > (*ccline).cmdbufflen {
                realloc_cmdbuff((*ccline).cmdlen + difflen + 4 as ::core::ffi::c_int);
                (*xp).xp_pattern = (*ccline).cmdbuff.offset(i as isize);
            }
            '_c2rust_label_0: {
                if (*ccline).cmdpos <= (*ccline).cmdlen {
                } else {
                    __assert_fail(
                        b"ccline->cmdpos <= ccline->cmdlen\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        355 as ::core::ffi::c_uint,
                        b"int nextwild(expand_T *, int, int, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            memmove(
                (*ccline)
                    .cmdbuff
                    .offset(((*ccline).cmdpos + difflen) as isize)
                    as *mut ::core::ffi::c_void,
                (*ccline).cmdbuff.offset((*ccline).cmdpos as isize) as *const ::core::ffi::c_void,
                ((*ccline).cmdlen as size_t)
                    .wrapping_sub((*ccline).cmdpos as size_t)
                    .wrapping_add(1 as size_t),
            );
            memmove(
                (*ccline).cmdbuff.offset(i as isize) as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                plen,
            );
            (*ccline).cmdlen += difflen;
            (*ccline).cmdpos += difflen;
        }
        redrawcmd();
        cursorcmd();
        if (*xp).xp_context == EXPAND_MAPPINGS as ::core::ffi::c_int && p.is_null() {
            return FAIL;
        }
        if (*xp).xp_numfiles <= 0 as ::core::ffi::c_int && p.is_null() {
            beep_flush();
        } else if (*xp).xp_numfiles == 1 as ::core::ffi::c_int
            && options & WILD_NOSELECT as ::core::ffi::c_int == 0
            && !wild_navigate
        {
            ExpandOne(
                xp,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
                WILD_FREE as ::core::ffi::c_int,
            );
        }
        xfree(p as *mut ::core::ffi::c_void);
        return OK;
    }
}

pub(crate) unsafe extern "C" fn get_next_or_prev_match(
    mut mode: ::core::ffi::c_int,
    mut xp: *mut expand_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if (*xp).xp_numfiles <= 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut findex: ::core::ffi::c_int = (*xp).xp_selected;
        if mode == WILD_PREV as ::core::ffi::c_int {
            if findex == -1 as ::core::ffi::c_int {
                findex = (*xp).xp_numfiles;
            }
            findex -= 1;
        } else if mode == WILD_NEXT as ::core::ffi::c_int {
            findex += 1;
        } else if mode == WILD_PAGEUP as ::core::ffi::c_int
            || mode == WILD_PAGEDOWN as ::core::ffi::c_int
        {
            let mut ht: ::core::ffi::c_int = pum_get_height();
            if ht > 3 as ::core::ffi::c_int {
                ht -= 2 as ::core::ffi::c_int;
            }
            if mode == WILD_PAGEUP as ::core::ffi::c_int {
                if findex == 0 as ::core::ffi::c_int {
                    findex = -1 as ::core::ffi::c_int;
                } else if findex < 0 as ::core::ffi::c_int {
                    findex = (*xp).xp_numfiles - 1 as ::core::ffi::c_int;
                } else {
                    findex = if findex - ht > 0 as ::core::ffi::c_int {
                        findex - ht
                    } else {
                        0 as ::core::ffi::c_int
                    };
                }
            } else if findex == (*xp).xp_numfiles - 1 as ::core::ffi::c_int {
                findex = -1 as ::core::ffi::c_int;
            } else if findex < 0 as ::core::ffi::c_int {
                findex = 0 as ::core::ffi::c_int;
            } else {
                findex = if findex + ht < (*xp).xp_numfiles - 1 as ::core::ffi::c_int {
                    findex + ht
                } else {
                    (*xp).xp_numfiles - 1 as ::core::ffi::c_int
                };
            }
        } else {
            '_c2rust_label: {
                if (*pum_want.ptr()).active {
                } else {
                    __assert_fail(
                        b"pum_want.active\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        739 as ::core::ffi::c_uint,
                        b"char *get_next_or_prev_match(int, expand_T *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            findex = (*pum_want.ptr()).item;
        }
        if findex < 0 as ::core::ffi::c_int || findex >= (*xp).xp_numfiles {
            if !(*xp).xp_orig.is_null() {
                findex = -1 as ::core::ffi::c_int;
            } else {
                findex = if findex < 0 as ::core::ffi::c_int {
                    (*xp).xp_numfiles - 1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            }
        }
        if p_wmnu.get() != 0 {
            if !(*compl_match_array.ptr()).is_null() {
                compl_selected.set(findex);
                cmdline_pum_display(false_0 != 0);
            } else if cmdline_compl_use_pum(true_0 != 0) {
                cmdline_pum_create(
                    get_cmdline_info(),
                    xp,
                    (*xp).xp_files,
                    (*xp).xp_numfiles,
                    cmd_showtail.get(),
                    false_0 != 0,
                );
                compl_selected.set(findex);
                pum_clear();
                cmdline_pum_display(true_0 != 0);
            } else {
                redraw_wildmenu(
                    xp,
                    (*xp).xp_numfiles,
                    (*xp).xp_files,
                    findex,
                    cmd_showtail.get(),
                );
            }
        }
        (*xp).xp_selected = findex;
        return xstrdup(if findex == -1 as ::core::ffi::c_int {
            (*xp).xp_orig
        } else {
            *(*xp).xp_files.offset(findex as isize)
        });
    }
}

pub(crate) unsafe extern "C" fn ExpandOne_start(
    mut mode: ::core::ffi::c_int,
    mut xp: *mut expand_T,
    mut str: *mut ::core::ffi::c_char,
    mut options: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut non_suf_match: ::core::ffi::c_int = 0;
        let mut ss: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if ExpandFromContext(
            xp,
            str,
            &raw mut (*xp).xp_files,
            &raw mut (*xp).xp_numfiles,
            options,
        ) != FAIL
        {
            if (*xp).xp_numfiles == 0 as ::core::ffi::c_int {
                if options & WILD_SILENT as ::core::ffi::c_int == 0 {
                    semsg(
                        gettext(&raw const e_nomatch2 as *const ::core::ffi::c_char),
                        str,
                    );
                }
            } else {
                ExpandEscape(xp, str, (*xp).xp_numfiles, (*xp).xp_files, options);
                if mode != WILD_ALL as ::core::ffi::c_int
                    && mode != WILD_ALL_KEEP as ::core::ffi::c_int
                    && mode != WILD_LONGEST as ::core::ffi::c_int
                {
                    if (*xp).xp_numfiles != 0 {
                        non_suf_match = (*xp).xp_numfiles;
                    } else {
                        non_suf_match = 1 as ::core::ffi::c_int;
                    }
                    if ((*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int
                        || (*xp).xp_context == EXPAND_DIRECTORIES as ::core::ffi::c_int)
                        && (*xp).xp_numfiles > 1 as ::core::ffi::c_int
                    {
                        non_suf_match = 0 as ::core::ffi::c_int;
                        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while i < 2 as ::core::ffi::c_int {
                            if match_suffix(*(*xp).xp_files.offset(i as isize)) {
                                non_suf_match += 1;
                            }
                            i += 1;
                        }
                    }
                    if non_suf_match != 1 as ::core::ffi::c_int {
                        if options & WILD_SILENT as ::core::ffi::c_int == 0 {
                            emsg(gettext(&raw const e_toomany as *const ::core::ffi::c_char));
                        } else if options & WILD_NO_BEEP as ::core::ffi::c_int == 0 {
                            beep_flush();
                        }
                    }
                    if !(non_suf_match != 1 as ::core::ffi::c_int
                        && mode == WILD_EXPAND_FREE as ::core::ffi::c_int)
                    {
                        ss = xstrdup(*(*xp).xp_files.offset(0 as ::core::ffi::c_int as isize));
                    }
                }
            }
        }
        return ss;
    }
}

pub(crate) unsafe extern "C" fn find_longest_match(
    mut xp: *mut expand_T,
    mut options: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut len: size_t = 0 as size_t;
        let mut mb_len: size_t = 0;
        while *(*(*xp).xp_files.offset(0 as ::core::ffi::c_int as isize)).offset(len as isize) != 0
        {
            mb_len = utfc_ptr2len(
                (*(*xp).xp_files.offset(0 as ::core::ffi::c_int as isize)).offset(len as isize),
            ) as size_t;
            let mut c0: ::core::ffi::c_int = utf_ptr2char(
                (*(*xp).xp_files.offset(0 as ::core::ffi::c_int as isize)).offset(len as isize),
            );
            let mut i: ::core::ffi::c_int = 0;
            i = 1 as ::core::ffi::c_int;
            while i < (*xp).xp_numfiles {
                let mut ci: ::core::ffi::c_int =
                    utf_ptr2char((*(*xp).xp_files.offset(i as isize)).offset(len as isize));
                if p_fic.get() != 0
                    && ((*xp).xp_context == EXPAND_DIRECTORIES as ::core::ffi::c_int
                        || (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int
                        || (*xp).xp_context == EXPAND_SHELLCMD as ::core::ffi::c_int
                        || (*xp).xp_context == EXPAND_BUFFERS as ::core::ffi::c_int)
                {
                    if mb_tolower(c0) != mb_tolower(ci) {
                        break;
                    }
                } else if c0 != ci {
                    break;
                }
                i += 1;
            }
            if i < (*xp).xp_numfiles {
                if options & WILD_NO_BEEP as ::core::ffi::c_int == 0 {
                    vim_beep(kOptBoFlagWildmode as ::core::ffi::c_int as ::core::ffi::c_uint);
                }
                break;
            } else {
                len = len.wrapping_add(mb_len);
            }
        }
        return xmemdupz(
            *(*xp).xp_files.offset(0 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            len,
        ) as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn ExpandOne(
    mut xp: *mut expand_T,
    mut str: *mut ::core::ffi::c_char,
    mut orig: *mut ::core::ffi::c_char,
    mut options: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut ss: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut orig_saved: bool = false_0 != 0;
        if mode == WILD_NEXT as ::core::ffi::c_int
            || mode == WILD_PREV as ::core::ffi::c_int
            || mode == WILD_PAGEUP as ::core::ffi::c_int
            || mode == WILD_PAGEDOWN as ::core::ffi::c_int
            || mode == WILD_PUM_WANT as ::core::ffi::c_int
        {
            return get_next_or_prev_match(mode, xp);
        }
        if mode == WILD_CANCEL as ::core::ffi::c_int {
            ss = xstrdup(if !(*xp).xp_orig.is_null() {
                (*xp).xp_orig as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            });
        } else if mode == WILD_APPLY as ::core::ffi::c_int {
            ss = xstrdup(if (*xp).xp_selected == -1 as ::core::ffi::c_int {
                if !(*xp).xp_orig.is_null() {
                    (*xp).xp_orig as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                }
            } else {
                *(*xp).xp_files.offset((*xp).xp_selected as isize) as *const ::core::ffi::c_char
            });
        }
        if (*xp).xp_numfiles != -1 as ::core::ffi::c_int
            && mode != WILD_ALL as ::core::ffi::c_int
            && mode != WILD_LONGEST as ::core::ffi::c_int
        {
            FreeWild((*xp).xp_numfiles, (*xp).xp_files);
            (*xp).xp_numfiles = -1 as ::core::ffi::c_int;
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*xp).xp_orig as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            if !(*compl_match_array.ptr()).is_null() {
                cmdline_pum_remove(false_0 != 0);
            }
        }
        (*xp).xp_selected = if options & WILD_NOSELECT as ::core::ffi::c_int != 0 {
            -1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        if mode == WILD_FREE as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if (*xp).xp_numfiles == -1 as ::core::ffi::c_int
            && mode != WILD_APPLY as ::core::ffi::c_int
            && mode != WILD_CANCEL as ::core::ffi::c_int
        {
            xfree((*xp).xp_orig as *mut ::core::ffi::c_void);
            (*xp).xp_orig = orig;
            orig_saved = true_0 != 0;
            ss = ExpandOne_start(mode, xp, str, options);
        }
        if mode == WILD_LONGEST as ::core::ffi::c_int && (*xp).xp_numfiles > 0 as ::core::ffi::c_int
        {
            ss = find_longest_match(xp, options);
            (*xp).xp_selected = -1 as ::core::ffi::c_int;
        }
        if mode == WILD_ALL as ::core::ffi::c_int
            && (*xp).xp_numfiles > 0 as ::core::ffi::c_int
            && !got_int.get()
        {
            let mut ss_size: size_t = 0 as size_t;
            let mut prefix: *mut ::core::ffi::c_char =
                b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            let mut suffix: *mut ::core::ffi::c_char =
                (if options & WILD_USE_NL as ::core::ffi::c_int != 0 {
                    b"\n\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b" \0".as_ptr() as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char;
            let n: ::core::ffi::c_int = (*xp).xp_numfiles - 1 as ::core::ffi::c_int;
            if (*xp).xp_prefix as ::core::ffi::c_uint
                == XP_PREFIX_NO as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                prefix = b"no\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                ss_size = ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                    .wrapping_sub(1 as usize)
                    .wrapping_mul(n as usize) as size_t;
            } else if (*xp).xp_prefix as ::core::ffi::c_uint
                == XP_PREFIX_INV as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                prefix =
                    b"inv\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                ss_size = ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                    .wrapping_sub(1 as usize)
                    .wrapping_mul(n as usize) as size_t;
            }
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*xp).xp_numfiles {
                ss_size = ss_size.wrapping_add(
                    strlen(*(*xp).xp_files.offset(i as isize)).wrapping_add(1 as size_t),
                );
                i += 1;
            }
            ss_size = ss_size.wrapping_add(1);
            ss = xmalloc(ss_size) as *mut ::core::ffi::c_char;
            *ss = NUL as ::core::ffi::c_char;
            let mut ssp: *mut ::core::ffi::c_char = ss;
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < (*xp).xp_numfiles {
                if i_0 > 0 as ::core::ffi::c_int {
                    ssp = xstpcpy(ssp, prefix);
                }
                ssp = xstpcpy(ssp, *(*xp).xp_files.offset(i_0 as isize));
                if i_0 < n {
                    ssp = xstpcpy(ssp, suffix);
                }
                '_c2rust_label: {
                    if ssp < ss.offset(ss_size as isize) {
                    } else {
                        __assert_fail(
                            b"ssp < ss + ss_size\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            995 as ::core::ffi::c_uint,
                            b"char *ExpandOne(expand_T *, char *, char *, int, int)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                i_0 += 1;
            }
        }
        if mode == WILD_EXPAND_FREE as ::core::ffi::c_int || mode == WILD_ALL as ::core::ffi::c_int
        {
            ExpandCleanup(xp);
        }
        if !orig_saved {
            xfree(orig as *mut ::core::ffi::c_void);
        }
        return ss;
    }
}

pub unsafe extern "C" fn ExpandInit(mut xp: *mut expand_T) {
    unsafe {
        memset(
            xp as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<expand_T>(),
        );
        (*xp).xp_backslash = XP_BS_NONE as ::core::ffi::c_int;
        (*xp).xp_prefix = XP_PREFIX_NONE;
        (*xp).xp_numfiles = -1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn ExpandCleanup(mut xp: *mut expand_T) {
    unsafe {
        if (*xp).xp_numfiles >= 0 as ::core::ffi::c_int {
            FreeWild((*xp).xp_numfiles, (*xp).xp_files);
            (*xp).xp_numfiles = -1 as ::core::ffi::c_int;
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*xp).xp_orig as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
}

pub unsafe extern "C" fn clear_cmdline_orig() {
    unsafe {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            cmdline_orig.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
}
