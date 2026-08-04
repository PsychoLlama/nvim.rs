//! `:set nowildmenu` output: the match list printed as messages.
//!
//! [`showmatches`] lays the matches out in columns and prints them with
//! [`showmatches_oneline`]; [`expand_showtail`] decides whether a file match
//! is shown as its tail alone.  [`addstar`] is here because it is the other
//! half of the same question — what the pattern looked like before the
//! matches were found.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn showmatches_oneline(
    mut xp: *mut expand_T,
    mut matches: *mut *mut ::core::ffi::c_char,
    mut numMatches: ::core::ffi::c_int,
    mut lines: ::core::ffi::c_int,
    mut linenr: ::core::ffi::c_int,
    mut maxlen: ::core::ffi::c_int,
    mut showtail: bool,
) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut lastlen: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
        let mut j: ::core::ffi::c_int = linenr;
        while j < numMatches {
            if (*xp).xp_context == EXPAND_TAGS_LISTFILES as ::core::ffi::c_int {
                msg_outtrans(*matches.offset(j as isize), HLF_D, false_0 != 0);
                p = (*matches.offset(j as isize))
                    .offset(strlen(*matches.offset(j as isize)) as isize)
                    .offset(1 as ::core::ffi::c_int as isize);
                msg_advance(maxlen + 1 as ::core::ffi::c_int);
                msg_puts(p);
                msg_advance(maxlen + 3 as ::core::ffi::c_int);
                msg_outtrans_long(p.offset(2 as ::core::ffi::c_int as isize), HLF_D);
                break;
            } else {
                let mut i: ::core::ffi::c_int = maxlen - lastlen;
                loop {
                    i -= 1;
                    if i < 0 as ::core::ffi::c_int {
                        break;
                    }
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
                let mut isdir: bool = false;
                if (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int
                    || (*xp).xp_context == EXPAND_SHELLCMD as ::core::ffi::c_int
                    || (*xp).xp_context == EXPAND_BUFFERS as ::core::ffi::c_int
                {
                    if (*xp).xp_numfiles != -1 as ::core::ffi::c_int {
                        let mut exp_path: *mut ::core::ffi::c_char =
                            expand_env_save_opt(*matches.offset(j as isize), true_0 != 0);
                        let mut path: *mut ::core::ffi::c_char = if !exp_path.is_null() {
                            exp_path
                        } else {
                            *matches.offset(j as isize)
                        };
                        let mut halved_slash: *mut ::core::ffi::c_char = backslash_halve_save(path);
                        isdir = os_isdir(halved_slash);
                        xfree(exp_path as *mut ::core::ffi::c_void);
                        if halved_slash != path {
                            xfree(halved_slash as *mut ::core::ffi::c_void);
                        }
                    } else {
                        isdir = os_isdir(*matches.offset(j as isize));
                    }
                    if showtail {
                        p = if showtail as ::core::ffi::c_int != 0 {
                            showmatches_gettail(*matches.offset(j as isize), false_0 != 0)
                        } else {
                            *matches.offset(j as isize)
                        };
                    } else {
                        home_replace(
                            ::core::ptr::null::<buf_T>(),
                            *matches.offset(j as isize),
                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                            MAXPATHL as size_t,
                            true_0 != 0,
                        );
                        p = NameBuff.ptr() as *mut ::core::ffi::c_char;
                    }
                } else {
                    isdir = false_0 != 0;
                    p = if showtail as ::core::ffi::c_int != 0 {
                        showmatches_gettail(*matches.offset(j as isize), false_0 != 0)
                    } else {
                        *matches.offset(j as isize)
                    };
                }
                lastlen = msg_outtrans(
                    p,
                    if isdir as ::core::ffi::c_int != 0 {
                        HLF_D
                    } else {
                        0 as ::core::ffi::c_int
                    },
                    false_0 != 0,
                );
                j += lines;
            }
        }
        if msg_col.get() > 0 as ::core::ffi::c_int {
            msg_clr_eos();
            msg_putchar('\n' as ::core::ffi::c_int);
        }
    }
}

pub unsafe extern "C" fn showmatches(
    mut xp: *mut expand_T,
    mut display_wildmenu: bool,
    mut display_list: bool,
    mut noselect: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let ccline: *mut CmdlineInfo = get_cmdline_info();
        let mut numMatches: ::core::ffi::c_int = 0;
        let mut matches: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut maxlen: ::core::ffi::c_int = 0;
        let mut lines: ::core::ffi::c_int = 0;
        let mut columns: ::core::ffi::c_int = 0;
        let mut showtail: bool = false;
        if (*xp).xp_numfiles == -1 as ::core::ffi::c_int {
            set_expand_context(xp);
            if (*xp).xp_context == EXPAND_LUA as ::core::ffi::c_int {
                nlua_expand_pat(xp);
            }
            let mut retval: ::core::ffi::c_int = expand_cmdline(
                xp,
                (*ccline).cmdbuff,
                (*ccline).cmdpos,
                &raw mut numMatches,
                &raw mut matches,
            );
            if retval != EXPAND_OK as ::core::ffi::c_int {
                return retval;
            }
            showtail = expand_showtail(xp);
        } else {
            numMatches = (*xp).xp_numfiles;
            matches = (*xp).xp_files;
            showtail = cmd_showtail.get();
        }
        if cmdline_compl_use_pum(display_wildmenu as ::core::ffi::c_int != 0 && !display_list) {
            cmdline_pum_create(ccline, xp, matches, numMatches, showtail, noselect);
            compl_selected.set(if noselect as ::core::ffi::c_int != 0 {
                -1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            });
            pum_clear();
            cmdline_pum_display(true_0 != 0);
            return EXPAND_OK as ::core::ffi::c_int;
        }
        if display_list {
            msg_didany.set(false_0 != 0);
            msg_start();
            if !ui_has(kUIMessages) {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            ui_flush();
            cmdline_row.set(msg_row.get());
            msg_didany.set(false_0 != 0);
            msg_ext_set_kind(b"wildlist\0".as_ptr() as *const ::core::ffi::c_char);
            msg_start();
        }
        if got_int.get() {
            got_int.set(false_0 != 0);
        } else if display_wildmenu as ::core::ffi::c_int != 0 && !display_list {
            redraw_wildmenu(
                xp,
                numMatches,
                matches,
                if noselect as ::core::ffi::c_int != 0 {
                    -1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                },
                showtail,
            );
        } else if display_list {
            maxlen = 0 as ::core::ffi::c_int;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < numMatches {
                let mut len: ::core::ffi::c_int = 0;
                if !showtail
                    && ((*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int
                        || (*xp).xp_context == EXPAND_SHELLCMD as ::core::ffi::c_int
                        || (*xp).xp_context == EXPAND_BUFFERS as ::core::ffi::c_int)
                {
                    home_replace(
                        ::core::ptr::null::<buf_T>(),
                        *matches.offset(i as isize),
                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                        MAXPATHL as size_t,
                        true_0 != 0,
                    );
                    len = vim_strsize(NameBuff.ptr() as *mut ::core::ffi::c_char);
                } else {
                    len = vim_strsize(if showtail as ::core::ffi::c_int != 0 {
                        showmatches_gettail(*matches.offset(i as isize), false_0 != 0)
                    } else {
                        *matches.offset(i as isize)
                    });
                }
                maxlen = if maxlen > len { maxlen } else { len };
                i += 1;
            }
            if (*xp).xp_context == EXPAND_TAGS_LISTFILES as ::core::ffi::c_int {
                lines = numMatches;
            } else {
                maxlen += 2 as ::core::ffi::c_int;
                columns = (Columns.get() + 2 as ::core::ffi::c_int) / maxlen;
                if columns < 1 as ::core::ffi::c_int {
                    columns = 1 as ::core::ffi::c_int;
                }
                lines = (numMatches + columns - 1 as ::core::ffi::c_int) / columns;
            }
            if (*xp).xp_context == EXPAND_TAGS_LISTFILES as ::core::ffi::c_int {
                msg_puts_hl(
                    gettext(b"tagname\0".as_ptr() as *const ::core::ffi::c_char),
                    HLF_T,
                    false_0 != 0,
                );
                msg_clr_eos();
                msg_advance(maxlen - 3 as ::core::ffi::c_int);
                msg_puts_hl(
                    gettext(b" kind file\n\0".as_ptr() as *const ::core::ffi::c_char),
                    HLF_T,
                    false_0 != 0,
                );
            }
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < lines {
                showmatches_oneline(xp, matches, numMatches, lines, i_0, maxlen, showtail);
                if got_int.get() {
                    got_int.set(false_0 != 0);
                    break;
                } else {
                    i_0 += 1;
                }
            }
            cmdline_row.set(msg_row.get());
        }
        if (*xp).xp_numfiles == -1 as ::core::ffi::c_int {
            FreeWild(numMatches, matches);
        }
        return EXPAND_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn showmatches_gettail(
    mut s: *mut ::core::ffi::c_char,
    mut eager: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut t: *mut ::core::ffi::c_char = s;
        let mut had_sep: bool = false_0 != 0;
        let mut p: *mut ::core::ffi::c_char = s;
        while *p as ::core::ffi::c_int != NUL {
            if vim_ispathsep(*p as ::core::ffi::c_int) {
                if eager {
                    t = p.offset(1 as ::core::ffi::c_int as isize);
                } else {
                    had_sep = true_0 != 0;
                }
            } else if had_sep {
                t = p;
                had_sep = false_0 != 0;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        return t;
    }
}

pub(crate) unsafe extern "C" fn expand_showtail(mut xp: *mut expand_T) -> bool {
    unsafe {
        if (*xp).xp_context != EXPAND_FILES as ::core::ffi::c_int
            && (*xp).xp_context != EXPAND_SHELLCMD as ::core::ffi::c_int
            && (*xp).xp_context != EXPAND_DIRECTORIES as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
        let mut end: *mut ::core::ffi::c_char = path_tail((*xp).xp_pattern);
        if end == (*xp).xp_pattern {
            return false_0 != 0;
        }
        let mut s: *mut ::core::ffi::c_char = (*xp).xp_pattern;
        while s < end {
            if rem_backslash(s) {
                s = s.offset(1);
            } else if !vim_strchr(
                b"*?[\0".as_ptr() as *const ::core::ffi::c_char,
                *s as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
            {
                return false_0 != 0;
            }
            s = s.offset(1);
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn addstar(
    mut fname: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut context: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if context != EXPAND_FILES as ::core::ffi::c_int
            && context != EXPAND_FILES_IN_PATH as ::core::ffi::c_int
            && context != EXPAND_SHELLCMD as ::core::ffi::c_int
            && context != EXPAND_DIRECTORIES as ::core::ffi::c_int
            && context != EXPAND_DIRS_IN_CDPATH as ::core::ffi::c_int
        {
            if context == EXPAND_FINDFUNC as ::core::ffi::c_int
                || context == EXPAND_HELP as ::core::ffi::c_int
                || context == EXPAND_COLORS as ::core::ffi::c_int
                || context == EXPAND_COMPILER as ::core::ffi::c_int
                || context == EXPAND_OWNSYNTAX as ::core::ffi::c_int
                || context == EXPAND_FILETYPE as ::core::ffi::c_int
                || context == EXPAND_KEYMAP as ::core::ffi::c_int
                || context == EXPAND_PACKADD as ::core::ffi::c_int
                || context == EXPAND_RUNTIME as ::core::ffi::c_int
                || (context == EXPAND_TAGS_LISTFILES as ::core::ffi::c_int
                    || context == EXPAND_TAGS as ::core::ffi::c_int)
                    && *fname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                || context == EXPAND_CHECKHEALTH as ::core::ffi::c_int
                || context == EXPAND_LSP as ::core::ffi::c_int
                || context == EXPAND_LUA as ::core::ffi::c_int
            {
                retval = xstrnsave(fname, len);
            } else {
                let mut new_len: size_t = len.wrapping_add(2 as size_t);
                let mut i: size_t = 0 as size_t;
                while i < len {
                    if *fname.offset(i as isize) as ::core::ffi::c_int == '*' as ::core::ffi::c_int
                        || *fname.offset(i as isize) as ::core::ffi::c_int
                            == '~' as ::core::ffi::c_int
                    {
                        new_len = new_len.wrapping_add(1);
                    }
                    if context == EXPAND_BUFFERS as ::core::ffi::c_int
                        && *fname.offset(i as isize) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                    {
                        new_len = new_len.wrapping_add(1);
                    }
                    if (context == EXPAND_USER_DEFINED as ::core::ffi::c_int
                        || context == EXPAND_USER_LIST as ::core::ffi::c_int)
                        && *fname.offset(i as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                    {
                        new_len = new_len.wrapping_add(1);
                    }
                    i = i.wrapping_add(1);
                }
                retval = xmalloc(new_len) as *mut ::core::ffi::c_char;
                *retval.offset(0 as ::core::ffi::c_int as isize) = '^' as ::core::ffi::c_char;
                let mut j: size_t = 1 as size_t;
                let mut i_0: size_t = 0 as size_t;
                while i_0 < len {
                    if context != EXPAND_USER_DEFINED as ::core::ffi::c_int
                        && context != EXPAND_USER_LIST as ::core::ffi::c_int
                        && *fname.offset(i_0 as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                        && {
                            i_0 = i_0.wrapping_add(1);
                            i_0 == len
                        }
                    {
                        break;
                    }
                    's_82: {
                        match *fname.offset(i_0 as isize) as ::core::ffi::c_int {
                            42 => {
                                let c2rust_fresh6 = j;
                                j = j.wrapping_add(1);
                                *retval.offset(c2rust_fresh6 as isize) = '.' as ::core::ffi::c_char;
                            }
                            126 => {
                                let c2rust_fresh7 = j;
                                j = j.wrapping_add(1);
                                *retval.offset(c2rust_fresh7 as isize) =
                                    '\\' as ::core::ffi::c_char;
                            }
                            63 => {
                                *retval.offset(j as isize) = '.' as ::core::ffi::c_char;
                                break 's_82;
                            }
                            46 => {
                                if context == EXPAND_BUFFERS as ::core::ffi::c_int {
                                    let c2rust_fresh8 = j;
                                    j = j.wrapping_add(1);
                                    *retval.offset(c2rust_fresh8 as isize) =
                                        '\\' as ::core::ffi::c_char;
                                }
                            }
                            92 => {
                                if context == EXPAND_USER_DEFINED as ::core::ffi::c_int
                                    || context == EXPAND_USER_LIST as ::core::ffi::c_int
                                {
                                    let c2rust_fresh9 = j;
                                    j = j.wrapping_add(1);
                                    *retval.offset(c2rust_fresh9 as isize) =
                                        '\\' as ::core::ffi::c_char;
                                }
                            }
                            _ => {}
                        }
                        *retval.offset(j as isize) = *fname.offset(i_0 as isize);
                    }
                    i_0 = i_0.wrapping_add(1);
                    j = j.wrapping_add(1);
                }
                *retval.offset(j as isize) = NUL as ::core::ffi::c_char;
            }
        } else {
            retval = xmalloc(len.wrapping_add(4 as size_t)) as *mut ::core::ffi::c_char;
            xmemcpyz(
                retval as *mut ::core::ffi::c_void,
                fname as *const ::core::ffi::c_void,
                len,
            );
            let mut tail: *mut ::core::ffi::c_char = path_tail(retval);
            let mut ends_in_star: ::core::ffi::c_int = (len > 0 as size_t
                && *retval.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    == '*' as ::core::ffi::c_int)
                as ::core::ffi::c_int;
            let mut k: ssize_t = len as ssize_t - 2 as ssize_t;
            while k >= 0 as ssize_t {
                if *retval.offset(k as isize) as ::core::ffi::c_int != '\\' as ::core::ffi::c_int {
                    break;
                }
                ends_in_star = (ends_in_star == 0) as ::core::ffi::c_int;
                k -= 1;
            }
            if (*retval as ::core::ffi::c_int != '~' as ::core::ffi::c_int || tail != retval)
                && ends_in_star == 0
                && vim_strchr(tail, '$' as ::core::ffi::c_int).is_null()
                && vim_strchr(retval, '`' as ::core::ffi::c_int).is_null()
            {
                let c2rust_fresh10 = len;
                len = len.wrapping_add(1);
                *retval.offset(c2rust_fresh10 as isize) = '*' as ::core::ffi::c_char;
            } else if len > 0 as size_t
                && *retval.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    == '$' as ::core::ffi::c_int
            {
                len = len.wrapping_sub(1);
            }
            *retval.offset(len as isize) = NUL as ::core::ffi::c_char;
        }
        return retval;
    }
}
