//! The `ccline.cmdbuff` allocation, and pasting into it.
//!
//! [`realloc_cmdbuff`] is the one every caller has to be careful of: it moves
//! the buffer, so nothing may hold a pointer into it across a call.
//! [`cmdline_paste`] and [`ccheck_abbr`] are the two writers that go through
//! the register and abbreviation machinery, and the `*_fnameescape` helpers
//! escape a file name on its way in.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn getexline(
    mut c: ::core::ffi::c_int,
    mut _cookie: *mut ::core::ffi::c_void,
    mut indent: ::core::ffi::c_int,
    mut do_concat: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if exec_from_reg.get() as ::core::ffi::c_int != 0 && vpeekc() == ':' as ::core::ffi::c_int {
            vgetc();
        }
        return getcmdline(c, 1 as ::core::ffi::c_int, indent, do_concat);
    }
}

pub unsafe extern "C" fn cmdline_overstrike() -> bool {
    unsafe {
        return (*ccline.ptr()).overstrike != 0;
    }
}

pub unsafe extern "C" fn cmdline_at_end() -> bool {
    unsafe {
        return (*ccline.ptr()).cmdpos >= (*ccline.ptr()).cmdlen;
    }
}

pub(crate) unsafe extern "C" fn dealloc_cmdbuff() {
    unsafe {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*ccline.ptr()).cmdbuff as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        (*ccline.ptr()).cmdbufflen = 0 as ::core::ffi::c_int;
        (*ccline.ptr()).cmdlen = (*ccline.ptr()).cmdbufflen;
    }
}

pub(crate) unsafe extern "C" fn alloc_cmdbuff(mut len: ::core::ffi::c_int) {
    unsafe {
        if len < 80 as ::core::ffi::c_int {
            len = 100 as ::core::ffi::c_int;
        } else {
            len += 20 as ::core::ffi::c_int;
        }
        (*ccline.ptr()).cmdbuff = xmalloc(len as size_t) as *mut ::core::ffi::c_char;
        (*ccline.ptr()).cmdbufflen = len;
    }
}

pub unsafe extern "C" fn realloc_cmdbuff(mut len: ::core::ffi::c_int) {
    unsafe {
        if len < (*ccline.ptr()).cmdbufflen {
            return;
        }
        let mut p: *mut ::core::ffi::c_char = (*ccline.ptr()).cmdbuff;
        alloc_cmdbuff(len);
        memmove(
            (*ccline.ptr()).cmdbuff as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            (*ccline.ptr()).cmdlen as size_t,
        );
        *(*ccline.ptr())
            .cmdbuff
            .offset((*ccline.ptr()).cmdlen as isize) = NUL as ::core::ffi::c_char;
        if !(*ccline.ptr()).xpc.is_null()
            && !(*(*ccline.ptr()).xpc).xp_pattern.is_null()
            && (*(*ccline.ptr()).xpc).xp_context != EXPAND_NOTHING as ::core::ffi::c_int
            && (*(*ccline.ptr()).xpc).xp_context != EXPAND_UNSUCCESSFUL as ::core::ffi::c_int
        {
            let mut i: ::core::ffi::c_int =
                (*(*ccline.ptr()).xpc).xp_pattern.offset_from(p) as ::core::ffi::c_int;
            if i >= 0 as ::core::ffi::c_int && i <= (*ccline.ptr()).cmdlen {
                (*(*ccline.ptr()).xpc).xp_pattern = (*ccline.ptr()).cmdbuff.offset(i as isize);
            }
        }
        xfree(p as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn save_cmdline(mut ccp: *mut CmdlineInfo) {
    unsafe {
        *ccp = ccline.get();
        memset(
            ccline.ptr() as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<CmdlineInfo>(),
        );
        (*ccline.ptr()).prev_ccline = ccp;
        (*ccline.ptr()).cmdbuff = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn restore_cmdline(mut ccp: *mut CmdlineInfo) {
    unsafe {
        ccline.set(*ccp);
    }
}

pub(crate) unsafe extern "C" fn cmdline_paste(
    mut regname: ::core::ffi::c_int,
    mut literally: bool,
    mut remcr: bool,
) -> bool {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut allocated: bool = false;
        if regname != Ctrl_F
            && regname != Ctrl_P
            && regname != Ctrl_W
            && regname != Ctrl_A
            && regname != Ctrl_L
            && !valid_yank_reg(regname, false_0 != 0)
        {
            return FAIL != 0;
        }
        line_breakcheck();
        if got_int.get() {
            return FAIL != 0;
        }
        (*textlock.ptr()) += 1;
        let i: bool = get_spec_reg(regname, &raw mut arg, &raw mut allocated, true_0 != 0);
        (*textlock.ptr()) -= 1;
        if i {
            if arg.is_null() {
                return FAIL != 0;
            }
            let mut p: *mut ::core::ffi::c_char = arg;
            if p_is.get() != 0 && regname == Ctrl_W {
                let mut w: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut len: ::core::ffi::c_int = 0;
                w = (*ccline.ptr())
                    .cmdbuff
                    .offset((*ccline.ptr()).cmdpos as isize);
                while w > (*ccline.ptr()).cmdbuff {
                    len = utf_head_off(
                        (*ccline.ptr()).cmdbuff,
                        w.offset(-(1 as ::core::ffi::c_int as isize)),
                    ) + 1 as ::core::ffi::c_int;
                    if !vim_iswordc(utf_ptr2char(w.offset(-(len as isize)))) {
                        break;
                    }
                    w = w.offset(-(len as isize));
                }
                len = (*ccline.ptr())
                    .cmdbuff
                    .offset((*ccline.ptr()).cmdpos as isize)
                    .offset_from(w) as ::core::ffi::c_int;
                if if p_ic.get() != 0 {
                    (strncasecmp(w, arg, len as size_t) == 0 as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                } else {
                    (strncmp(w, arg, len as size_t) == 0 as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                } != 0
                {
                    p = p.offset(len as isize);
                }
            }
            cmdline_paste_str(p, literally);
            if allocated {
                xfree(arg as *mut ::core::ffi::c_void);
            }
            return OK != 0;
        }
        return cmdline_paste_reg(regname, literally, remcr);
    }
}

pub unsafe extern "C" fn cmdline_paste_str(mut s: *const ::core::ffi::c_char, mut literally: bool) {
    unsafe {
        if literally {
            put_on_cmdline(s, -1 as ::core::ffi::c_int, true_0 != 0);
        } else {
            while *s as ::core::ffi::c_int != NUL {
                let mut cv: ::core::ffi::c_int = *s as uint8_t as ::core::ffi::c_int;
                if cv == Ctrl_V
                    && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
                {
                    s = s.offset(1);
                }
                let mut c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
                if cv == Ctrl_V
                    || c == ESC
                    || c == Ctrl_C
                    || c == CAR
                    || c == NL
                    || c == Ctrl_L
                    || c == Ctrl_BSL && *s as ::core::ffi::c_int == Ctrl_N
                {
                    stuffcharReadbuff(Ctrl_V);
                }
                stuffcharReadbuff(c);
            }
        };
    }
}

pub(crate) unsafe extern "C" fn ccheck_abbr(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        let mut spos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if p_paste.get() != 0 || no_abbr.get() as ::core::ffi::c_int != 0 {
            return false_0;
        }
        while spos < (*ccline.ptr()).cmdlen
            && ascii_iswhite(*(*ccline.ptr()).cmdbuff.offset(spos as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            spos += 1;
        }
        if (*ccline.ptr()).cmdlen - spos > 5 as ::core::ffi::c_int
            && *(*ccline.ptr()).cmdbuff.offset(spos as isize) as ::core::ffi::c_int
                == '\'' as ::core::ffi::c_int
            && *(*ccline.ptr())
                .cmdbuff
                .offset((spos + 2 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                == ',' as ::core::ffi::c_int
            && *(*ccline.ptr())
                .cmdbuff
                .offset((spos + 3 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                == '\'' as ::core::ffi::c_int
        {
            spos += 5 as ::core::ffi::c_int;
        } else {
            spos = 0 as ::core::ffi::c_int;
        }
        return check_abbr(c, (*ccline.ptr()).cmdbuff, (*ccline.ptr()).cmdpos, spos)
            as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn vim_strsave_fnameescape(
    fname: *const ::core::ffi::c_char,
    what: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = vim_strsave_escaped(
            fname,
            if what == VSE_SHELL {
                SHELL_ESC_CHARS.as_ptr()
            } else if what == VSE_BUFFER {
                BUFFER_ESC_CHARS.as_ptr()
            } else {
                PATH_ESC_CHARS.as_ptr()
            },
        );
        if what == VSE_SHELL && csh_like_shell() {
            let mut s: *mut ::core::ffi::c_char =
                vim_strsave_escaped(p, b"!\0".as_ptr() as *const ::core::ffi::c_char);
            xfree(p as *mut ::core::ffi::c_void);
            p = s;
        }
        if *p as ::core::ffi::c_int == '>' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            escape_fname(&raw mut p);
        }
        return p;
    }
}

pub unsafe extern "C" fn escape_fname(mut pp: *mut *mut ::core::ffi::c_char) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char =
            xmalloc(strlen(*pp).wrapping_add(2 as size_t)) as *mut ::core::ffi::c_char;
        *p.offset(0 as ::core::ffi::c_int as isize) = '\\' as ::core::ffi::c_char;
        strcpy(p.offset(1 as ::core::ffi::c_int as isize), *pp);
        xfree(*pp as *mut ::core::ffi::c_void);
        *pp = p;
    }
}

pub unsafe extern "C" fn tilde_replace(
    mut orig_pat: *mut ::core::ffi::c_char,
    mut num_files: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
) {
    unsafe {
        if *orig_pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '~' as ::core::ffi::c_int
            && vim_ispathsep(
                *orig_pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            ) as ::core::ffi::c_int
                != 0
        {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < num_files {
                let mut p: *mut ::core::ffi::c_char =
                    home_replace_save(::core::ptr::null_mut::<buf_T>(), *files.offset(i as isize));
                xfree(*files.offset(i as isize) as *mut ::core::ffi::c_void);
                *files.offset(i as isize) = p;
                i += 1;
            }
        }
    }
}
