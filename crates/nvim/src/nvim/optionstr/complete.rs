//! The callbacks for the completion, spelling and tag options.
//!
//! They are `pub` only so the generated option table can name them.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn did_set_complete(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut buffer: [c_char; 512] = [0; 512];
    let mut char_before: uint8_t = NUL as uint8_t;
    let mut p: *mut c_char = *varp;
    while *p != 0 {
        memset(
            &raw mut buffer as *mut c_char as *mut c_void,
            0 as c_int,
            LSIZE as c_int as size_t,
        );
        let mut buf_ptr: *mut c_char = &raw mut buffer as *mut c_char;
        let mut escape: c_int = 0 as c_int;
        while *p as c_int != 0
            && (*p as c_int != ',' as c_int || escape != 0)
            && buf_ptr
                < (&raw mut buffer as *mut c_char)
                    .offset(LSIZE as c_int as isize)
                    .offset(-(1 as c_int as isize))
        {
            if *p as c_int == '\\' as c_int
                && *p.offset(1 as c_int as isize) as c_int == ',' as c_int
            {
                escape = 1 as c_int;
                p = p.offset(1);
            } else {
                escape = 0 as c_int;
                let c2rust_fresh5 = buf_ptr;
                buf_ptr = buf_ptr.offset(1);
                *c2rust_fresh5 = *p;
            }
            p = p.offset(1);
        }
        *buf_ptr = NUL as c_char;
        if vim_strchr(
            b".wbuksid]tUfFo\0".as_ptr() as *const c_char,
            *(&raw mut buffer as *mut c_char) as uint8_t as c_int,
        )
        .is_null()
        {
            return illegal_char(
                (*args).os_errbuf,
                (*args).os_errbuflen,
                *(&raw mut buffer as *mut c_char) as uint8_t as c_int,
            );
        }
        if vim_strchr(
            b"ksF\0".as_ptr() as *const c_char,
            *(&raw mut buffer as *mut c_char) as uint8_t as c_int,
        )
        .is_null()
            && *(&raw mut buffer as *mut c_char).offset(1 as c_int as isize) as c_int != NUL
            && *(&raw mut buffer as *mut c_char).offset(1 as c_int as isize) as c_int
                != '^' as c_int
        {
            char_before = *(&raw mut buffer as *mut c_char) as uint8_t;
        } else {
            let mut t: *mut c_char = ::core::ptr::null_mut::<c_char>();
            t = vim_strchr(&raw mut buffer as *mut c_char, '^' as c_int);
            if !t.is_null() {
                let c2rust_fresh6 = t;
                t = t.offset(1);
                *c2rust_fresh6 = NUL as c_char;
                if *t == 0 {
                    char_before = '^' as uint8_t;
                } else {
                    while *t != 0 {
                        if !ascii_isdigit(*t as c_int) {
                            char_before = '^' as uint8_t;
                            break;
                        } else {
                            t = t.offset(1);
                        }
                    }
                }
            }
        }
        if char_before as c_int != NUL {
            if !(*args).os_errbuf.is_null() {
                return illegal_char_after_chr(
                    (*args).os_errbuf,
                    (*args).os_errbuflen,
                    char_before as c_int,
                );
            }
            return ::core::ptr::null::<c_char>();
        }
        while *p as c_int == ',' as c_int || *p as c_int == ' ' as c_int {
            p = p.offset(1);
        }
    }
    if set_cpt_callbacks(args) != OK {
        return illegal_char_after_chr((*args).os_errbuf, (*args).os_errbuflen, 'F' as c_int);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_completeitemalign(mut _args: *mut optset_T) -> *const c_char {
    let mut p: *mut c_char = p_cia.get();
    let mut new_cia_flags: c_uint = 0 as c_uint;
    let mut seen: [bool; 3] = [false_0 != 0, false_0 != 0, false_0 != 0];
    let mut count: c_int = 0 as c_int;
    let mut buf: [c_char; 10] = [0; 10];
    while *p != 0 {
        copy_option_part(
            &raw mut p,
            &raw mut buf as *mut c_char,
            ::core::mem::size_of::<[c_char; 10]>(),
            b",\0".as_ptr() as *const c_char as *mut c_char,
        );
        if count >= 3 as c_int {
            return &raw const e_invarg as *const c_char;
        }
        if strequal(
            &raw mut buf as *mut c_char,
            b"abbr\0".as_ptr() as *const c_char,
        ) {
            if seen[CPT_ABBR as c_int as usize] {
                return &raw const e_invarg as *const c_char;
            }
            new_cia_flags = new_cia_flags
                .wrapping_mul(10 as c_uint)
                .wrapping_add(CPT_ABBR as c_int as c_uint);
            seen[CPT_ABBR as c_int as usize] = true_0 != 0;
            count += 1;
        } else if strequal(
            &raw mut buf as *mut c_char,
            b"kind\0".as_ptr() as *const c_char,
        ) {
            if seen[CPT_KIND as c_int as usize] {
                return &raw const e_invarg as *const c_char;
            }
            new_cia_flags = new_cia_flags
                .wrapping_mul(10 as c_uint)
                .wrapping_add(CPT_KIND as c_int as c_uint);
            seen[CPT_KIND as c_int as usize] = true_0 != 0;
            count += 1;
        } else if strequal(
            &raw mut buf as *mut c_char,
            b"menu\0".as_ptr() as *const c_char,
        ) {
            if seen[CPT_MENU as c_int as usize] {
                return &raw const e_invarg as *const c_char;
            }
            new_cia_flags = new_cia_flags
                .wrapping_mul(10 as c_uint)
                .wrapping_add(CPT_MENU as c_int as c_uint);
            seen[CPT_MENU as c_int as usize] = true_0 != 0;
            count += 1;
        } else {
            return &raw const e_invarg as *const c_char;
        }
    }
    if new_cia_flags == 0 as c_uint || count != 3 as c_int {
        return &raw const e_invarg as *const c_char;
    }
    cia_flags.set(new_cia_flags);
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_completeopt(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut cot: *mut c_char = p_cot.get();
    let mut flags: *mut c_uint = cot_flags.ptr();
    if (*args).os_flags & OPT_LOCAL as c_int != 0 {
        cot = (*buf).b_p_cot;
        flags = &raw mut (*buf).b_cot_flags;
    } else if (*args).os_flags & OPT_GLOBAL as c_int == 0 {
        (*buf).b_cot_flags = 0 as c_uint;
    }
    if opt_strings_flags(
        cot,
        opt_cot_values.ptr() as *mut *const c_char,
        flags,
        true_0 != 0,
    ) != OK
    {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_helpfile(mut _args: *mut optset_T) -> *const c_char {
    if didset_vim.get() {
        vim_unsetenv_ext(b"VIM\0".as_ptr() as *const c_char);
    }
    if didset_vimruntime.get() {
        vim_unsetenv_ext(b"VIMRUNTIME\0".as_ptr() as *const c_char);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_helplang(mut _args: *mut optset_T) -> *const c_char {
    let mut s: *mut c_char = p_hlg.get();
    while *s as c_int != NUL {
        if *s.offset(1 as c_int as isize) as c_int == NUL
            || (*s.offset(2 as c_int as isize) as c_int != ',' as c_int
                || *s.offset(3 as c_int as isize) as c_int == NUL)
                && *s.offset(2 as c_int as isize) as c_int != NUL
        {
            return &raw const e_invarg as *const c_char;
        }
        if *s.offset(2 as c_int as isize) as c_int == NUL {
            break;
        }
        s = s.offset(3 as c_int as isize);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_mkspellmem(mut _args: *mut optset_T) -> *const c_char {
    if spell_check_msm() != OK {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_optexpr(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut name: *mut c_char = get_scriptlocal_funcname(*varp);
    if !name.is_null() {
        free_string_option(*varp);
        *varp = name;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_spellcapcheck(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    return compile_cap_prog((*win).w_s);
}

pub unsafe extern "C" fn did_set_spellfile(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if !valid_spellfile(*varp) {
        return &raw const e_invarg as *const c_char;
    }
    return did_set_spell_option();
}

pub unsafe extern "C" fn did_set_spelllang(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if !valid_spelllang(*varp) {
        return &raw const e_invarg as *const c_char;
    }
    return did_set_spell_option();
}

pub unsafe extern "C" fn did_set_spelloptions(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut opt_flags: c_int = (*args).os_flags;
    let mut val: *const c_char = (*args).os_newval.string.data;
    if opt_flags & OPT_LOCAL as c_int == 0
        && opt_strings_flags(
            val,
            opt_spo_values.ptr() as *mut *const c_char,
            spo_flags.ptr(),
            true_0 != 0,
        ) != OK
    {
        return &raw const e_invarg as *const c_char;
    }
    if opt_flags & OPT_GLOBAL as c_int == 0
        && opt_strings_flags(
            val,
            opt_spo_values.ptr() as *mut *const c_char,
            &raw mut (*(*win).w_s).b_p_spo_flags,
            true_0 != 0,
        ) != OK
    {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_spellsuggest(mut _args: *mut optset_T) -> *const c_char {
    if spell_check_sps() != OK {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_tagcase(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut opt_flags: c_int = (*args).os_flags;
    let mut flags: *mut c_uint = ::core::ptr::null_mut::<c_uint>();
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if opt_flags & OPT_LOCAL as c_int != 0 {
        p = (*buf).b_p_tc;
        flags = &raw mut (*buf).b_tc_flags;
    } else {
        p = p_tc.get();
        flags = tc_flags.ptr();
    }
    if opt_flags & OPT_LOCAL as c_int != 0 && *p as c_int == NUL {
        *flags = 0 as c_uint;
    } else if opt_strings_flags(
        p,
        opt_tc_values.ptr() as *mut *const c_char,
        flags,
        false_0 != 0,
    ) != OK
    {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
