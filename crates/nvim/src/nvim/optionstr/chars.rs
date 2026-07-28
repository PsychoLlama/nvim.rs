//! 'fillchars' and 'listchars': a field list parsed into the character
//! tables the screen draws from.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn did_set_global_chars_option(
    mut win: *mut win_T,
    mut val: *mut c_char,
    mut what: CharsOption,
    mut opt_flags: c_int,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    let mut local_ptr: *mut *mut c_char = if what as c_uint == kListchars as c_int as c_uint {
        &raw mut (*win).w_onebuf_opt.wo_lcs
    } else {
        &raw mut (*win).w_onebuf_opt.wo_fcs
    };
    errmsg = set_chars_option(
        win,
        val,
        what,
        **local_ptr as c_int == NUL || opt_flags & OPT_GLOBAL as c_int == 0,
        errbuf,
        errbuflen,
    );
    if !errmsg.is_null() {
        return errmsg;
    }
    if opt_flags & OPT_GLOBAL as c_int == 0 {
        clear_string_option(local_ptr);
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            let mut opt: *mut c_char = if what as c_uint == kListchars as c_int as c_uint {
                (*wp).w_onebuf_opt.wo_lcs
            } else {
                (*wp).w_onebuf_opt.wo_fcs
            };
            if *opt as c_int == NUL {
                set_chars_option(wp, opt, what, true_0 != 0, errbuf, errbuflen);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    redraw_all_later(UPD_NOT_VALID as c_int);
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_chars_option(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    if varp == p_lcs.ptr() {
        errmsg = did_set_global_chars_option(
            win,
            *varp,
            kListchars,
            (*args).os_flags,
            (*args).os_errbuf,
            (*args).os_errbuflen,
        );
    } else if varp == p_fcs.ptr() {
        errmsg = did_set_global_chars_option(
            win,
            *varp,
            kFillchars,
            (*args).os_flags,
            (*args).os_errbuf,
            (*args).os_errbuflen,
        );
    } else if varp == &raw mut (*win).w_onebuf_opt.wo_lcs {
        errmsg = set_chars_option(
            win,
            *varp,
            kListchars,
            true_0 != 0,
            (*args).os_errbuf,
            (*args).os_errbuflen,
        );
    } else if varp == &raw mut (*win).w_onebuf_opt.wo_fcs {
        errmsg = set_chars_option(
            win,
            *varp,
            kFillchars,
            true_0 != 0,
            (*args).os_errbuf,
            (*args).os_errbuflen,
        );
    }
    return errmsg;
}

pub(crate) unsafe extern "C" fn get_encoded_char_adv(mut p: *mut *const c_char) -> schar_T {
    let mut s: *const c_char = *p;
    if *s.offset(0 as c_int as isize) as c_int == '\\' as c_int
        && (*s.offset(1 as c_int as isize) as c_int == 'x' as c_int
            || *s.offset(1 as c_int as isize) as c_int == 'u' as c_int
            || *s.offset(1 as c_int as isize) as c_int == 'U' as c_int)
    {
        let mut num: int64_t = 0 as int64_t;
        let mut bytes: c_int = if *s.offset(1 as c_int as isize) as c_int == 'x' as c_int {
            1 as c_int
        } else if *s.offset(1 as c_int as isize) as c_int == 'u' as c_int {
            2 as c_int
        } else {
            4 as c_int
        };
        while bytes > 0 as c_int {
            *p = (*p).offset(2 as c_int as isize);
            let mut n: c_int = hexhex2nr(*p);
            if n < 0 as c_int {
                return 0 as schar_T;
            }
            num = num * 256 as int64_t + n as int64_t;
            bytes -= 1;
        }
        *p = (*p).offset(2 as c_int as isize);
        return if char2cells(num as c_int) > 1 as c_int {
            0 as schar_T
        } else {
            schar_from_char(num as c_int)
        };
    }
    let mut clen: c_int = utfc_ptr2len(s);
    let mut firstc: c_int = 0;
    let mut c: schar_T = utfc_ptr2schar(s, &raw mut firstc);
    *p = (*p).offset(clen as isize);
    return if clen == 1 as c_int && firstc > 127 as c_int || char2cells(firstc) > 1 as c_int {
        0 as schar_T
    } else {
        c
    };
}

pub(crate) unsafe extern "C" fn field_value_err(
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
    mut fmt: *const c_char,
    mut field: *const c_char,
) -> *mut c_char {
    if errbuf.is_null() {
        return b"\0".as_ptr() as *const c_char as *mut c_char;
    }
    vim_snprintf(errbuf, errbuflen, gettext(fmt), field);
    return errbuf;
}

pub unsafe extern "C" fn set_chars_option(
    mut wp: *mut win_T,
    mut value: *const c_char,
    mut what: CharsOption,
    mut apply: bool,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut last_multispace: *const c_char = ::core::ptr::null::<c_char>();
    let mut last_lmultispace: *const c_char = ::core::ptr::null::<c_char>();
    let mut multispace_len: c_int = 0 as c_int;
    let mut lead_multispace_len: c_int = 0 as c_int;
    let mut tab: *const chars_tab = ::core::ptr::null::<chars_tab>();
    let mut entries: c_int = 0;
    if what as c_uint == kListchars as c_int as c_uint {
        tab = (lcs_tab.ptr() as *const _) as *const chars_tab;
        entries = ::core::mem::size_of::<[chars_tab; 12]>()
            .wrapping_div(::core::mem::size_of::<chars_tab>())
            .wrapping_div(
                (::core::mem::size_of::<[chars_tab; 12]>()
                    .wrapping_rem(::core::mem::size_of::<chars_tab>())
                    == 0) as c_int as usize,
            ) as c_int;
        if *(*wp).w_onebuf_opt.wo_lcs.offset(0 as c_int as isize) as c_int == NUL {
            value = p_lcs.get();
        }
    } else {
        tab = (fcs_tab.ptr() as *const _) as *const chars_tab;
        entries = ::core::mem::size_of::<[chars_tab; 21]>()
            .wrapping_div(::core::mem::size_of::<chars_tab>())
            .wrapping_div(
                (::core::mem::size_of::<[chars_tab; 21]>()
                    .wrapping_rem(::core::mem::size_of::<chars_tab>())
                    == 0) as c_int as usize,
            ) as c_int;
        if *(*wp).w_onebuf_opt.wo_fcs.offset(0 as c_int as isize) as c_int == NUL {
            value = p_fcs.get();
        }
    }
    let mut round: c_int = 0 as c_int;
    while round
        <= (if apply as c_int != 0 {
            1 as c_int
        } else {
            0 as c_int
        })
    {
        let mut has_tab: bool = false_0 != 0;
        let mut has_leadtab: bool = false_0 != 0;
        if round > 0 as c_int {
            let mut i: c_int = 0 as c_int;
            while i < entries {
                if !(*tab.offset(i as isize)).cp.is_null() {
                    *(*tab.offset(i as isize)).cp = schar_from_str(
                        if !(*tab.offset(i as isize)).def.is_null()
                            && ptr2cells((*tab.offset(i as isize)).def) == 1 as c_int
                        {
                            (*tab.offset(i as isize)).def
                        } else {
                            (*tab.offset(i as isize)).fallback
                        },
                    );
                }
                i += 1;
            }
            if what as c_uint == kListchars as c_int as c_uint {
                (*lcs_chars.ptr()).tab1 = NUL as schar_T;
                (*lcs_chars.ptr()).tab3 = NUL as schar_T;
                (*lcs_chars.ptr()).leadtab1 = NUL as schar_T;
                (*lcs_chars.ptr()).leadtab3 = NUL as schar_T;
                if multispace_len > 0 as c_int {
                    (*lcs_chars.ptr()).multispace = xmalloc(
                        (multispace_len as size_t)
                            .wrapping_add(1 as size_t)
                            .wrapping_mul(::core::mem::size_of::<schar_T>()),
                    ) as *mut schar_T;
                    *(*lcs_chars.ptr())
                        .multispace
                        .offset(multispace_len as isize) = NUL as schar_T;
                } else {
                    (*lcs_chars.ptr()).multispace = ::core::ptr::null_mut::<schar_T>();
                }
                if lead_multispace_len > 0 as c_int {
                    (*lcs_chars.ptr()).leadmultispace = xmalloc(
                        (lead_multispace_len as size_t)
                            .wrapping_add(1 as size_t)
                            .wrapping_mul(::core::mem::size_of::<schar_T>()),
                    ) as *mut schar_T;
                    *(*lcs_chars.ptr())
                        .leadmultispace
                        .offset(lead_multispace_len as isize) = NUL as schar_T;
                } else {
                    (*lcs_chars.ptr()).leadmultispace = ::core::ptr::null_mut::<schar_T>();
                }
            }
        }
        let mut p: *const c_char = value;
        while *p != 0 {
            let mut i_0: c_int = 0;
            i_0 = 0 as c_int;
            while i_0 < entries {
                if !(strncmp(
                    p,
                    (*tab.offset(i_0 as isize)).name.data,
                    (*tab.offset(i_0 as isize)).name.size,
                ) == 0 as c_int
                    && *p.offset((*tab.offset(i_0 as isize)).name.size as isize) as c_int
                        == ':' as c_int)
                {
                    i_0 += 1;
                } else {
                    let mut s: *const c_char = p
                        .offset((*tab.offset(i_0 as isize)).name.size as isize)
                        .offset(1 as c_int as isize);
                    if what as c_uint == kListchars as c_int as c_uint
                        && strcmp(
                            (*tab.offset(i_0 as isize)).name.data,
                            b"multispace\0".as_ptr() as *const c_char,
                        ) == 0 as c_int
                    {
                        if round == 0 as c_int {
                            last_multispace = p;
                            multispace_len = 0 as c_int;
                            while *s as c_int != NUL && *s as c_int != ',' as c_int {
                                let mut c1: schar_T = get_encoded_char_adv(&raw mut s);
                                if c1 == 0 as schar_T {
                                    return field_value_err(
                                        errbuf,
                                        errbuflen,
                                        (e_wrong_character_width_for_field_str.ptr() as *const _)
                                            as *const c_char,
                                        (*tab.offset(i_0 as isize)).name.data,
                                    );
                                }
                                multispace_len += 1;
                            }
                            if multispace_len == 0 as c_int {
                                return field_value_err(
                                    errbuf,
                                    errbuflen,
                                    (e_wrong_number_of_characters_for_field_str.ptr() as *const _)
                                        as *const c_char,
                                    (*tab.offset(i_0 as isize)).name.data,
                                );
                            }
                        } else {
                            let mut multispace_pos: c_int = 0 as c_int;
                            while *s as c_int != NUL && *s as c_int != ',' as c_int {
                                let mut c1_0: schar_T = get_encoded_char_adv(&raw mut s);
                                if p == last_multispace {
                                    let c2rust_fresh2 = multispace_pos;
                                    multispace_pos = multispace_pos + 1;
                                    *(*lcs_chars.ptr()).multispace.offset(c2rust_fresh2 as isize) =
                                        c1_0;
                                }
                            }
                        }
                        p = s;
                        break;
                    } else if what as c_uint == kListchars as c_int as c_uint
                        && strcmp(
                            (*tab.offset(i_0 as isize)).name.data,
                            b"leadmultispace\0".as_ptr() as *const c_char,
                        ) == 0 as c_int
                    {
                        if round == 0 as c_int {
                            last_lmultispace = p;
                            lead_multispace_len = 0 as c_int;
                            while *s as c_int != NUL && *s as c_int != ',' as c_int {
                                let mut c1_1: schar_T = get_encoded_char_adv(&raw mut s);
                                if c1_1 == 0 as schar_T {
                                    return field_value_err(
                                        errbuf,
                                        errbuflen,
                                        (e_wrong_character_width_for_field_str.ptr() as *const _)
                                            as *const c_char,
                                        (*tab.offset(i_0 as isize)).name.data,
                                    );
                                }
                                lead_multispace_len += 1;
                            }
                            if lead_multispace_len == 0 as c_int {
                                return field_value_err(
                                    errbuf,
                                    errbuflen,
                                    (e_wrong_number_of_characters_for_field_str.ptr() as *const _)
                                        as *const c_char,
                                    (*tab.offset(i_0 as isize)).name.data,
                                );
                            }
                        } else {
                            let mut multispace_pos_0: c_int = 0 as c_int;
                            while *s as c_int != NUL && *s as c_int != ',' as c_int {
                                let mut c1_2: schar_T = get_encoded_char_adv(&raw mut s);
                                if p == last_lmultispace {
                                    let c2rust_fresh3 = multispace_pos_0;
                                    multispace_pos_0 = multispace_pos_0 + 1;
                                    *(*lcs_chars.ptr())
                                        .leadmultispace
                                        .offset(c2rust_fresh3 as isize) = c1_2;
                                }
                            }
                        }
                        p = s;
                        break;
                    } else {
                        if *s as c_int == NUL {
                            return field_value_err(
                                errbuf,
                                errbuflen,
                                (e_wrong_number_of_characters_for_field_str.ptr() as *const _)
                                    as *const c_char,
                                (*tab.offset(i_0 as isize)).name.data,
                            );
                        }
                        let mut c1_3: schar_T = get_encoded_char_adv(&raw mut s);
                        if c1_3 == 0 as schar_T {
                            return field_value_err(
                                errbuf,
                                errbuflen,
                                (e_wrong_character_width_for_field_str.ptr() as *const _)
                                    as *const c_char,
                                (*tab.offset(i_0 as isize)).name.data,
                            );
                        }
                        let mut c2: schar_T = 0 as schar_T;
                        let mut c3: schar_T = 0 as schar_T;
                        if (*tab.offset(i_0 as isize)).cp == &raw mut (*lcs_chars.ptr()).tab2
                            || (*tab.offset(i_0 as isize)).cp
                                == &raw mut (*lcs_chars.ptr()).leadtab2
                        {
                            if *s as c_int == NUL {
                                return field_value_err(
                                    errbuf,
                                    errbuflen,
                                    (e_wrong_number_of_characters_for_field_str.ptr() as *const _)
                                        as *const c_char,
                                    (*tab.offset(i_0 as isize)).name.data,
                                );
                            }
                            c2 = get_encoded_char_adv(&raw mut s);
                            if c2 == 0 as schar_T {
                                return field_value_err(
                                    errbuf,
                                    errbuflen,
                                    (e_wrong_character_width_for_field_str.ptr() as *const _)
                                        as *const c_char,
                                    (*tab.offset(i_0 as isize)).name.data,
                                );
                            }
                            if !(*s as c_int == ',' as c_int || *s as c_int == NUL) {
                                c3 = get_encoded_char_adv(&raw mut s);
                                if c3 == 0 as schar_T {
                                    return field_value_err(
                                        errbuf,
                                        errbuflen,
                                        (e_wrong_character_width_for_field_str.ptr() as *const _)
                                            as *const c_char,
                                        (*tab.offset(i_0 as isize)).name.data,
                                    );
                                }
                            }
                            if (*tab.offset(i_0 as isize)).cp == &raw mut (*lcs_chars.ptr()).tab2 {
                                has_tab = true_0 != 0;
                            } else {
                                has_leadtab = true_0 != 0;
                            }
                        }
                        if *s as c_int == ',' as c_int || *s as c_int == NUL {
                            if round > 0 as c_int {
                                if (*tab.offset(i_0 as isize)).cp
                                    == &raw mut (*lcs_chars.ptr()).tab2
                                {
                                    (*lcs_chars.ptr()).tab1 = c1_3;
                                    (*lcs_chars.ptr()).tab2 = c2;
                                    (*lcs_chars.ptr()).tab3 = c3;
                                } else if (*tab.offset(i_0 as isize)).cp
                                    == &raw mut (*lcs_chars.ptr()).leadtab2
                                {
                                    (*lcs_chars.ptr()).leadtab1 = c1_3;
                                    (*lcs_chars.ptr()).leadtab2 = c2;
                                    (*lcs_chars.ptr()).leadtab3 = c3;
                                } else if !(*tab.offset(i_0 as isize)).cp.is_null() {
                                    *(*tab.offset(i_0 as isize)).cp = c1_3;
                                }
                            }
                            p = s;
                            break;
                        } else {
                            return field_value_err(
                                errbuf,
                                errbuflen,
                                (e_wrong_number_of_characters_for_field_str.ptr() as *const _)
                                    as *const c_char,
                                (*tab.offset(i_0 as isize)).name.data,
                            );
                        }
                    }
                }
            }
            if i_0 == entries {
                return &raw const e_invarg as *const c_char;
            }
            if *p as c_int == ',' as c_int {
                p = p.offset(1);
            }
        }
        if what as c_uint == kListchars as c_int as c_uint && has_leadtab as c_int != 0 && !has_tab
        {
            return &raw const e_leadtab_requires_tab as *const c_char;
        }
        round += 1;
    }
    if apply {
        if what as c_uint == kListchars as c_int as c_uint {
            xfree((*wp).w_p_lcs_chars.multispace as *mut c_void);
            xfree((*wp).w_p_lcs_chars.leadmultispace as *mut c_void);
            (*wp).w_p_lcs_chars = lcs_chars.get();
        } else {
            (*wp).w_p_fcs_chars = fcs_chars.get();
        }
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn get_fillchars_name(mut _xp: *mut expand_T, mut idx: c_int) -> *mut c_char {
    if idx < 0 as c_int
        || idx
            >= ::core::mem::size_of::<[chars_tab; 21]>()
                .wrapping_div(::core::mem::size_of::<chars_tab>())
                .wrapping_div(
                    (::core::mem::size_of::<[chars_tab; 21]>()
                        .wrapping_rem(::core::mem::size_of::<chars_tab>())
                        == 0) as c_int as usize,
                ) as c_int
    {
        return ::core::ptr::null_mut::<c_char>();
    }
    return (*fcs_tab.ptr())[idx as usize].name.data;
}

pub unsafe extern "C" fn get_listchars_name(mut _xp: *mut expand_T, mut idx: c_int) -> *mut c_char {
    if idx < 0 as c_int
        || idx
            >= ::core::mem::size_of::<[chars_tab; 12]>()
                .wrapping_div(::core::mem::size_of::<chars_tab>())
                .wrapping_div(
                    (::core::mem::size_of::<[chars_tab; 12]>()
                        .wrapping_rem(::core::mem::size_of::<chars_tab>())
                        == 0) as c_int as usize,
                ) as c_int
    {
        return ::core::ptr::null_mut::<c_char>();
    }
    return (*lcs_tab.ptr())[idx as usize].name.data;
}

pub unsafe extern "C" fn check_chars_options() -> *const c_char {
    if !set_chars_option(
        curwin.get(),
        p_lcs.get(),
        kListchars,
        false_0 != 0,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
    )
    .is_null()
    {
        return (e_conflicts_with_value_of_listchars.ptr() as *const _) as *const c_char;
    }
    if !set_chars_option(
        curwin.get(),
        p_fcs.get(),
        kFillchars,
        false_0 != 0,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
    )
    .is_null()
    {
        return (e_conflicts_with_value_of_fillchars.ptr() as *const _) as *const c_char;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if !set_chars_option(
                wp,
                (*wp).w_onebuf_opt.wo_lcs,
                kListchars,
                true_0 != 0,
                ::core::ptr::null_mut::<c_char>(),
                0 as size_t,
            )
            .is_null()
            {
                return (e_conflicts_with_value_of_listchars.ptr() as *const _) as *const c_char;
            }
            if !set_chars_option(
                wp,
                (*wp).w_onebuf_opt.wo_fcs,
                kFillchars,
                true_0 != 0,
                ::core::ptr::null_mut::<c_char>(),
                0 as size_t,
            )
            .is_null()
            {
                return (e_conflicts_with_value_of_fillchars.ptr() as *const _) as *const c_char;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return ::core::ptr::null::<c_char>();
}
