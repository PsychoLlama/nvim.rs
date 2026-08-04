//! Command-line history: `<Up>`, `<Down>` and the history commands.
//!
//! [`command_line_browse_history`] is the recall itself and
//! [`command_line_next_histidx`] the index walk it drives, matching the
//! typed prefix where `'wildoptions'` asks for it.  [`get_list_range`] parses
//! the `:history` and `:clist` style range arguments.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn command_line_next_histidx(
    mut s: *mut CommandLineState,
    mut next_match: bool,
) {
    unsafe {
        loop {
            if !next_match {
                if (*s).hiscnt == get_hislen() {
                    (*s).hiscnt = get_hisidx((*s).histype);
                } else if (*s).hiscnt == 0 as ::core::ffi::c_int
                    && get_hisidx((*s).histype) != get_hislen() - 1 as ::core::ffi::c_int
                {
                    (*s).hiscnt = get_hislen() - 1 as ::core::ffi::c_int;
                } else if (*s).hiscnt != get_hisidx((*s).histype) + 1 as ::core::ffi::c_int {
                    (*s).hiscnt -= 1;
                } else {
                    (*s).hiscnt = (*s).save_hiscnt;
                    break;
                }
            } else if (*s).hiscnt == get_hisidx((*s).histype) {
                (*s).hiscnt = get_hislen();
                break;
            } else {
                if (*s).hiscnt == get_hislen() {
                    break;
                }
                if (*s).hiscnt == get_hislen() - 1 as ::core::ffi::c_int {
                    (*s).hiscnt = 0 as ::core::ffi::c_int;
                } else {
                    (*s).hiscnt += 1;
                }
            }
            let entry = hist_entry_ref((*s).histype, (*s).hiscnt);
            match entry {
                None => {
                    (*s).hiscnt = (*s).save_hiscnt;
                    break;
                }
                Some(entry) => {
                    if (*s).c != K_UP && (*s).c != K_DOWN
                        || (*s).hiscnt == (*s).save_hiscnt
                        || strncmp(entry.text, (*s).lookfor, (*s).lookforlen as size_t)
                            == 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                }
            }
        }
    }
}

pub(crate) unsafe extern "C" fn command_line_browse_history(
    mut s: *mut CommandLineState,
) -> ::core::ffi::c_int {
    unsafe {
        if (*s).histype == HIST_INVALID as ::core::ffi::c_int
            || get_hislen() == 0 as ::core::ffi::c_int
            || (*s).firstc == NUL
        {
            return CMDLINE_NOT_CHANGED;
        }
        (*s).save_hiscnt = (*s).hiscnt;
        if (*s).lookfor.is_null() {
            (*s).lookfor = xstrnsave((*ccline.ptr()).cmdbuff, (*ccline.ptr()).cmdlen as size_t);
            *(*s).lookfor.offset((*ccline.ptr()).cmdpos as isize) = NUL as ::core::ffi::c_char;
            (*s).lookforlen = (*ccline.ptr()).cmdpos;
        }
        let mut next_match: bool = (*s).c == K_DOWN
            || (*s).c == K_S_DOWN
            || (*s).c == Ctrl_N
            || (*s).c == K_PAGEDOWN
            || (*s).c == K_KPAGEDOWN;
        command_line_next_histidx(s, next_match);
        if (*s).hiscnt != (*s).save_hiscnt {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut plen: ::core::ffi::c_int = 0;
            let mut old_firstc: ::core::ffi::c_int = 0;
            let mut hist_sep: ::core::ffi::c_int = NUL;
            dealloc_cmdbuff();
            (*s).xpc.xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
            if (*s).hiscnt == get_hislen() {
                p = (*s).lookfor;
                plen = (*s).lookforlen;
            } else {
                let entry =
                    hist_entry_ref((*s).histype, (*s).hiscnt).expect("browsed slot is occupied");
                p = entry.text as *mut ::core::ffi::c_char;
                plen = entry.len as ::core::ffi::c_int;
                hist_sep = entry.sep as ::core::ffi::c_int;
            }
            if (*s).histype == HIST_SEARCH as ::core::ffi::c_int && p != (*s).lookfor && {
                old_firstc = hist_sep;
                old_firstc != (*s).firstc
            } {
                let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i <= 1 as ::core::ffi::c_int {
                    len = 0 as ::core::ffi::c_int;
                    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while *p.offset(j as isize) as ::core::ffi::c_int != NUL {
                        if *p.offset(j as isize) as ::core::ffi::c_int == old_firstc
                            && (j == 0 as ::core::ffi::c_int
                                || *p.offset((j - 1 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                                    != '\\' as ::core::ffi::c_int)
                        {
                            if i > 0 as ::core::ffi::c_int {
                                *(*ccline.ptr()).cmdbuff.offset(len as isize) =
                                    (*s).firstc as ::core::ffi::c_char;
                            }
                        } else {
                            if *p.offset(j as isize) as ::core::ffi::c_int == (*s).firstc
                                && (j == 0 as ::core::ffi::c_int
                                    || *p.offset((j - 1 as ::core::ffi::c_int) as isize)
                                        as ::core::ffi::c_int
                                        != '\\' as ::core::ffi::c_int)
                            {
                                if i > 0 as ::core::ffi::c_int {
                                    *(*ccline.ptr()).cmdbuff.offset(len as isize) =
                                        '\\' as ::core::ffi::c_char;
                                }
                                len += 1;
                            }
                            if i > 0 as ::core::ffi::c_int {
                                *(*ccline.ptr()).cmdbuff.offset(len as isize) =
                                    *p.offset(j as isize);
                            }
                        }
                        len += 1;
                        j += 1;
                    }
                    if i == 0 as ::core::ffi::c_int {
                        alloc_cmdbuff(len);
                    }
                    i += 1;
                }
                *(*ccline.ptr()).cmdbuff.offset(len as isize) = NUL as ::core::ffi::c_char;
                (*ccline.ptr()).cmdlen = len;
                (*ccline.ptr()).cmdpos = (*ccline.ptr()).cmdlen;
            } else {
                alloc_cmdbuff(plen);
                strcpy((*ccline.ptr()).cmdbuff, p);
                (*ccline.ptr()).cmdlen = plen;
                (*ccline.ptr()).cmdpos = (*ccline.ptr()).cmdlen;
            }
            redrawcmd();
            return CMDLINE_CHANGED;
        }
        beep_flush();
        return CMDLINE_NOT_CHANGED;
    }
}

pub unsafe extern "C" fn get_list_range(
    mut str: *mut *mut ::core::ffi::c_char,
    mut num1: *mut ::core::ffi::c_int,
    mut num2: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut len: ::core::ffi::c_int = 0;
        let mut first: bool = false_0 != 0;
        let mut num: varnumber_T = 0;
        *str = skipwhite(*str);
        if **str as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            || ascii_isdigit(**str as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            vim_str2nr(
                *str,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                &raw mut len,
                0 as ::core::ffi::c_int,
                &raw mut num,
                ::core::ptr::null_mut::<uvarnumber_T>(),
                0 as ::core::ffi::c_int,
                false_0 != 0,
                ::core::ptr::null_mut::<bool>(),
            );
            *str = (*str).offset(len as isize);
            if num > INT_MAX as varnumber_T {
                return FAIL;
            }
            *num1 = num as ::core::ffi::c_int;
            first = true_0 != 0;
        }
        *str = skipwhite(*str);
        if **str as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            *str = skipwhite((*str).offset(1 as ::core::ffi::c_int as isize));
            vim_str2nr(
                *str,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                &raw mut len,
                0 as ::core::ffi::c_int,
                &raw mut num,
                ::core::ptr::null_mut::<uvarnumber_T>(),
                0 as ::core::ffi::c_int,
                false_0 != 0,
                ::core::ptr::null_mut::<bool>(),
            );
            if len > 0 as ::core::ffi::c_int {
                *str = skipwhite((*str).offset(len as isize));
                if num > INT_MAX as varnumber_T {
                    return FAIL;
                }
                *num2 = num as ::core::ffi::c_int;
            } else if !first {
                return FAIL;
            }
        } else if first {
            *num2 = *num1;
        }
        return OK;
    }
}
