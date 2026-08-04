//! Escaping a match, and whether fuzzy matching applies.
//!
//! [`wildescape`] puts back whatever the shell, the command line or a `:set`
//! value would otherwise eat, once per match, and [`ExpandEscape`] runs it
//! over a whole match array.  [`cmdline_fuzzy_complete`] answers whether
//! `'wildoptions'` asked for fuzzy matching *and* the context supports it —
//! the contexts that expand paths or option values never do.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn cmdline_fuzzy_completion_supported(xp: *const expand_T) -> bool {
    unsafe {
        match (*xp).xp_context {
            5 | 28 | 29 | 3 | 56 | 2 | 37 | 36 | 59 | 58 | 8 | 55 | 63 | 7 | 52 | 53 | 38 | 44
            | 51 | 33 | 57 | 6 | 17 | 31 | 32 => return false_0 != 0,
            _ => {}
        }
        return wop_flags.get() & kOptWopFlagFuzzy as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0;
    }
}

pub unsafe extern "C" fn cmdline_fuzzy_complete(fuzzystr: *const ::core::ffi::c_char) -> bool {
    unsafe {
        return wop_flags.get() & kOptWopFlagFuzzy as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0
            && *fuzzystr as ::core::ffi::c_int != NUL;
    }
}

pub(crate) unsafe extern "C" fn sort_func_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p1: *mut ::core::ffi::c_char = *(s1 as *mut *mut ::core::ffi::c_char);
        let mut p2: *mut ::core::ffi::c_char = *(s2 as *mut *mut ::core::ffi::c_char);
        if *p1 as ::core::ffi::c_int != '<' as ::core::ffi::c_int
            && *p2 as ::core::ffi::c_int == '<' as ::core::ffi::c_int
        {
            return -1 as ::core::ffi::c_int;
        }
        if *p1 as ::core::ffi::c_int == '<' as ::core::ffi::c_int
            && *p2 as ::core::ffi::c_int != '<' as ::core::ffi::c_int
        {
            return 1 as ::core::ffi::c_int;
        }
        return strcmp(p1, p2);
    }
}

pub(crate) unsafe extern "C" fn wildescape(
    mut xp: *mut expand_T,
    mut str: *const ::core::ffi::c_char,
    mut numfiles: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let vse_what: ::core::ffi::c_int = if (*xp).xp_context == EXPAND_BUFFERS {
            VSE_BUFFER
        } else {
            VSE_NONE
        };
        if (*xp).xp_context == EXPAND_FILES
            || (*xp).xp_context == EXPAND_FILES_IN_PATH
            || (*xp).xp_context == EXPAND_SHELLCMD
            || (*xp).xp_context == EXPAND_BUFFERS
            || (*xp).xp_context == EXPAND_DIRECTORIES
            || (*xp).xp_context == EXPAND_DIRS_IN_CDPATH
        {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < numfiles {
                if (*xp).xp_backslash & XP_BS_THREE != 0 {
                    let mut pat: *mut ::core::ffi::c_char =
                        (if (*xp).xp_backslash & XP_BS_COMMA != 0 {
                            b" ,\0".as_ptr() as *const ::core::ffi::c_char
                        } else {
                            b" \0".as_ptr() as *const ::core::ffi::c_char
                        }) as *mut ::core::ffi::c_char;
                    p = vim_strsave_escaped(*files.offset(i as isize), pat);
                    xfree(*files.offset(i as isize) as *mut ::core::ffi::c_void);
                    *files.offset(i as isize) = p;
                } else if (*xp).xp_backslash & XP_BS_COMMA != 0 {
                    if !vim_strchr(*files.offset(i as isize), ',' as ::core::ffi::c_int).is_null() {
                        p = vim_strsave_escaped(
                            *files.offset(i as isize),
                            b",\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        xfree(*files.offset(i as isize) as *mut ::core::ffi::c_void);
                        *files.offset(i as isize) = p;
                    }
                }
                p = vim_strsave_fnameescape(
                    *files.offset(i as isize),
                    if (*xp).xp_shell as ::core::ffi::c_int != 0 {
                        VSE_SHELL
                    } else {
                        vse_what
                    },
                );
                xfree(*files.offset(i as isize) as *mut ::core::ffi::c_void);
                *files.offset(i as isize) = p;
                if *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '~' as ::core::ffi::c_int
                    && *(*files.offset(i as isize)).offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == '~' as ::core::ffi::c_int
                {
                    escape_fname(files.offset(i as isize));
                }
                i += 1;
            }
            (*xp).xp_backslash = XP_BS_NONE;
            if **files.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '+' as ::core::ffi::c_int
            {
                escape_fname(files.offset(0 as ::core::ffi::c_int as isize));
            }
        } else if (*xp).xp_context == EXPAND_TAGS {
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < numfiles {
                p = vim_strsave_escaped(
                    *files.offset(i_0 as isize),
                    b"\\|\"\0".as_ptr() as *const ::core::ffi::c_char,
                );
                xfree(*files.offset(i_0 as isize) as *mut ::core::ffi::c_void);
                *files.offset(i_0 as isize) = p;
                i_0 += 1;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn ExpandEscape(
    mut xp: *mut expand_T,
    mut str: *mut ::core::ffi::c_char,
    mut numfiles: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
    mut options: ::core::ffi::c_int,
) {
    unsafe {
        if options & WILD_HOME_REPLACE != 0 {
            tilde_replace(str, numfiles, files);
        }
        if options & WILD_ESCAPE != 0 {
            wildescape(xp, str, numfiles, files);
        }
    }
}
