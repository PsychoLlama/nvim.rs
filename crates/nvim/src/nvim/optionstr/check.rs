//! Vetting a string option's value, and the sweeps that re-vet every
//! buffer's copy of one.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn didset_string_options() {
    check_str_opt(kOptCasemap, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptBackupcopy, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptBelloff, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptCompleteopt, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptSessionoptions, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptViewoptions, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptFoldopen, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptDisplay, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptJumpoptions, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptRedrawdebug, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptTagcase, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptTermpastefilter, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptVirtualedit, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptSwitchbuf, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptTabclose, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptWildoptions, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptClipboard, ::core::ptr::null_mut::<*mut c_char>());
}

pub unsafe extern "C" fn illegal_char(
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
    mut c: c_int,
) -> *mut c_char {
    if errbuf.is_null() {
        return b"\0".as_ptr() as *const c_char as *mut c_char;
    }
    vim_snprintf(
        errbuf,
        errbuflen,
        gettext(b"E539: Illegal character <%s>\0".as_ptr() as *const c_char),
        transchar(c),
    );
    return errbuf;
}

pub(crate) unsafe extern "C" fn illegal_char_after_chr(
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
    mut c: c_int,
) -> *mut c_char {
    if errbuf.is_null() {
        return b"\0".as_ptr() as *const c_char as *mut c_char;
    }
    vim_snprintf(
        errbuf,
        errbuflen,
        gettext((e_illegal_character_after_chr.ptr() as *const _) as *const c_char),
        c,
    );
    return errbuf;
}

pub unsafe extern "C" fn check_buf_options(mut buf: *mut buf_T) {
    check_string_option(&raw mut (*buf).b_p_bh);
    check_string_option(&raw mut (*buf).b_p_bt);
    check_string_option(&raw mut (*buf).b_p_fenc);
    check_string_option(&raw mut (*buf).b_p_ff);
    check_string_option(&raw mut (*buf).b_p_def);
    check_string_option(&raw mut (*buf).b_p_inc);
    check_string_option(&raw mut (*buf).b_p_inex);
    check_string_option(&raw mut (*buf).b_p_inde);
    check_string_option(&raw mut (*buf).b_p_indk);
    check_string_option(&raw mut (*buf).b_p_fp);
    check_string_option(&raw mut (*buf).b_p_fex);
    check_string_option(&raw mut (*buf).b_p_kp);
    check_string_option(&raw mut (*buf).b_p_mps);
    check_string_option(&raw mut (*buf).b_p_fo);
    check_string_option(&raw mut (*buf).b_p_flp);
    check_string_option(&raw mut (*buf).b_p_isk);
    check_string_option(&raw mut (*buf).b_p_com);
    check_string_option(&raw mut (*buf).b_p_cms);
    check_string_option(&raw mut (*buf).b_p_nf);
    check_string_option(&raw mut (*buf).b_p_qe);
    check_string_option(&raw mut (*buf).b_p_syn);
    check_string_option(&raw mut (*buf).b_s.b_syn_isk);
    check_string_option(&raw mut (*buf).b_s.b_p_spc);
    check_string_option(&raw mut (*buf).b_s.b_p_spf);
    check_string_option(&raw mut (*buf).b_s.b_p_spl);
    check_string_option(&raw mut (*buf).b_s.b_p_spo);
    check_string_option(&raw mut (*buf).b_p_sua);
    check_string_option(&raw mut (*buf).b_p_cink);
    check_string_option(&raw mut (*buf).b_p_cino);
    parse_cino(buf);
    check_string_option(&raw mut (*buf).b_p_lop);
    check_string_option(&raw mut (*buf).b_p_ft);
    check_string_option(&raw mut (*buf).b_p_cinw);
    check_string_option(&raw mut (*buf).b_p_cinsd);
    check_string_option(&raw mut (*buf).b_p_cot);
    check_string_option(&raw mut (*buf).b_p_cpt);
    check_string_option(&raw mut (*buf).b_p_cfu);
    check_string_option(&raw mut (*buf).b_p_ofu);
    check_string_option(&raw mut (*buf).b_p_keymap);
    check_string_option(&raw mut (*buf).b_p_gefm);
    check_string_option(&raw mut (*buf).b_p_gp);
    check_string_option(&raw mut (*buf).b_p_mp);
    check_string_option(&raw mut (*buf).b_p_efm);
    check_string_option(&raw mut (*buf).b_p_ep);
    check_string_option(&raw mut (*buf).b_p_path);
    check_string_option(&raw mut (*buf).b_p_tags);
    check_string_option(&raw mut (*buf).b_p_ffu);
    check_string_option(&raw mut (*buf).b_p_tfu);
    check_string_option(&raw mut (*buf).b_p_tc);
    check_string_option(&raw mut (*buf).b_p_dict);
    check_string_option(&raw mut (*buf).b_p_dia);
    check_string_option(&raw mut (*buf).b_p_tsr);
    check_string_option(&raw mut (*buf).b_p_tsrfu);
    check_string_option(&raw mut (*buf).b_p_lw);
    check_string_option(&raw mut (*buf).b_p_bkc);
    check_string_option(&raw mut (*buf).b_p_menc);
    check_string_option(&raw mut (*buf).b_p_vsts);
    check_string_option(&raw mut (*buf).b_p_vts);
}

pub unsafe extern "C" fn free_string_option(mut p: *mut c_char) {
    if p != empty_string_option.ptr() as *mut c_char {
        xfree(p as *mut c_void);
    }
}

pub unsafe extern "C" fn clear_string_option(mut pp: *mut *mut c_char) {
    if *pp != empty_string_option.ptr() as *mut c_char {
        xfree(*pp as *mut c_void);
    }
    *pp = empty_string_option.ptr() as *mut c_char;
}

pub unsafe extern "C" fn check_string_option(mut pp: *mut *mut c_char) {
    if (*pp).is_null() {
        *pp = empty_string_option.ptr() as *mut c_char;
    }
}

pub(crate) unsafe extern "C" fn valid_filetype(mut val: *const c_char) -> bool {
    return valid_name(val, b".-_\0".as_ptr() as *const c_char);
}

pub unsafe extern "C" fn check_signcolumn(mut scl: *mut c_char, mut wp: *mut win_T) -> c_int {
    let mut val: *mut c_char = empty_string_option.ptr() as *mut c_char;
    if !scl.is_null() {
        val = scl;
    } else if !wp.is_null() {
        val = (*wp).w_onebuf_opt.wo_scl;
    }
    if *val as c_int == NUL {
        return FAIL;
    }
    if opt_strings_flags(
        val,
        opt_scl_values.ptr() as *mut *const c_char,
        ::core::ptr::null_mut::<c_uint>(),
        false_0 != 0,
    ) == OK
    {
        if wp.is_null() {
            return OK;
        }
        if strncmp(val, b"no\0".as_ptr() as *const c_char, 2 as size_t) == 0 {
            (*wp).w_maxscwidth = SCL_NO;
            (*wp).w_minscwidth = (*wp).w_maxscwidth;
        } else if strncmp(val, b"nu\0".as_ptr() as *const c_char, 2 as size_t) == 0
            && ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
        {
            (*wp).w_maxscwidth = SCL_NUM;
            (*wp).w_minscwidth = (*wp).w_maxscwidth;
        } else if strncmp(val, b"yes:\0".as_ptr() as *const c_char, 4 as size_t) == 0 {
            (*wp).w_maxscwidth = *val.offset(4 as c_int as isize) as c_int - '0' as c_int;
            (*wp).w_minscwidth = (*wp).w_maxscwidth;
        } else if *val as c_int == 'y' as c_int {
            (*wp).w_maxscwidth = 1 as c_int;
            (*wp).w_minscwidth = (*wp).w_maxscwidth;
        } else if strncmp(val, b"auto:\0".as_ptr() as *const c_char, 5 as size_t) == 0 {
            (*wp).w_minscwidth = 0 as c_int;
            (*wp).w_maxscwidth = *val.offset(5 as c_int as isize) as c_int - '0' as c_int;
        } else {
            (*wp).w_minscwidth = 0 as c_int;
            (*wp).w_maxscwidth = 1 as c_int;
        }
    } else {
        if strncmp(val, b"auto:\0".as_ptr() as *const c_char, 5 as size_t) != 0 as c_int
            || strlen(val) != 8 as size_t
            || !ascii_isdigit(*val.offset(5 as c_int as isize) as c_int)
            || *val.offset(6 as c_int as isize) as c_int != '-' as c_int
            || !ascii_isdigit(*val.offset(7 as c_int as isize) as c_int)
        {
            return FAIL;
        }
        let mut min: c_int = *val.offset(5 as c_int as isize) as c_int - '0' as c_int;
        let mut max: c_int = *val.offset(7 as c_int as isize) as c_int - '0' as c_int;
        if min < 1 as c_int || max < 2 as c_int || min > 8 as c_int || min >= max {
            return FAIL;
        }
        if wp.is_null() {
            return OK;
        }
        (*wp).w_minscwidth = min;
        (*wp).w_maxscwidth = max;
    }
    let mut scwidth: c_int = if (*wp).w_minscwidth <= 0 as c_int {
        0 as c_int
    } else if (*wp).w_maxscwidth < (*wp).w_scwidth {
        (*wp).w_maxscwidth
    } else {
        (*wp).w_scwidth
    };
    (*wp).w_scwidth = if (*wp).w_minscwidth > scwidth {
        (*wp).w_minscwidth
    } else {
        scwidth
    };
    return OK;
}

pub unsafe extern "C" fn check_stl_option(mut s: *mut c_char) -> *const c_char {
    let mut groupdepth: c_int = 0 as c_int;
    static errbuf: GlobalCell<[c_char; 80]> = GlobalCell::new([0; 80]);
    while *s != 0 {
        while *s as c_int != 0 && *s as c_int != '%' as c_int {
            s = s.offset(1);
        }
        if *s == 0 {
            break;
        }
        s = s.offset(1);
        if *s as c_int == '%' as c_int
            || *s as c_int == STL_TRUNCMARK as c_int
            || *s as c_int == STL_SEPARATE as c_int
        {
            s = s.offset(1);
        } else if *s as c_int == ')' as c_int {
            s = s.offset(1);
            groupdepth -= 1;
            if groupdepth < 0 as c_int {
                break;
            }
        } else {
            if *s as c_int == '-' as c_int {
                s = s.offset(1);
            }
            while ascii_isdigit(*s as c_int) {
                s = s.offset(1);
            }
            if *s as c_int == STL_USER_HL as c_int {
                continue;
            }
            if *s as c_int == '.' as c_int {
                s = s.offset(1);
                while *s as c_int != 0 && ascii_isdigit(*s as c_int) as c_int != 0 {
                    s = s.offset(1);
                }
            }
            if *s as c_int == '(' as c_int {
                groupdepth += 1;
            } else {
                let mut c2rust_lvalue: [c_char; 45] = [
                    STL_FILEPATH as c_int as c_char,
                    STL_FULLPATH as c_int as c_char,
                    STL_FILENAME as c_int as c_char,
                    STL_COLUMN as c_int as c_char,
                    STL_VIRTCOL as c_int as c_char,
                    STL_VIRTCOL_ALT as c_int as c_char,
                    STL_LINE as c_int as c_char,
                    STL_NUMLINES as c_int as c_char,
                    STL_BUFNO as c_int as c_char,
                    STL_KEYMAP as c_int as c_char,
                    STL_OFFSET as c_int as c_char,
                    STL_OFFSET_X as c_int as c_char,
                    STL_BYTEVAL as c_int as c_char,
                    STL_BYTEVAL_X as c_int as c_char,
                    STL_ROFLAG as c_int as c_char,
                    STL_ROFLAG_ALT as c_int as c_char,
                    STL_HELPFLAG as c_int as c_char,
                    STL_HELPFLAG_ALT as c_int as c_char,
                    STL_FILETYPE as c_int as c_char,
                    STL_FILETYPE_ALT as c_int as c_char,
                    STL_PREVIEWFLAG as c_int as c_char,
                    STL_PREVIEWFLAG_ALT as c_int as c_char,
                    STL_MODIFIED as c_int as c_char,
                    STL_MODIFIED_ALT as c_int as c_char,
                    STL_QUICKFIX as c_int as c_char,
                    STL_PERCENTAGE as c_int as c_char,
                    STL_ALTPERCENT as c_int as c_char,
                    STL_ARGLISTSTAT as c_int as c_char,
                    STL_PAGENUM as c_int as c_char,
                    STL_SHOWCMD as c_int as c_char,
                    STL_FOLDCOL as c_int as c_char,
                    STL_SIGNCOL as c_int as c_char,
                    STL_VIM_EXPR as c_int as c_char,
                    STL_SEPARATE as c_int as c_char,
                    STL_TRUNCMARK as c_int as c_char,
                    STL_USER_HL as c_int as c_char,
                    STL_HIGHLIGHT as c_int as c_char,
                    STL_HIGHLIGHT_COMB as c_int as c_char,
                    STL_TABPAGENR as c_int as c_char,
                    STL_TABCLOSENR as c_int as c_char,
                    STL_CLICK_FUNC as c_int as c_char,
                    STL_TABPAGENR as c_int as c_char,
                    STL_TABCLOSENR as c_int as c_char,
                    STL_CLICK_FUNC as c_int as c_char,
                    0 as c_char,
                ];
                if vim_strchr(
                    &raw mut c2rust_lvalue as *mut c_char,
                    *s as uint8_t as c_int,
                )
                .is_null()
                {
                    return illegal_char(
                        errbuf.ptr() as *mut c_char,
                        ::core::mem::size_of::<[c_char; 80]>(),
                        *s as uint8_t as c_int,
                    );
                }
                if *s as c_int == '{' as c_int {
                    s = s.offset(1);
                    let mut reevaluate: bool = *s as c_int == '%' as c_int;
                    if reevaluate as c_int != 0 && {
                        s = s.offset(1);
                        *s as c_int == '}' as c_int
                    } {
                        return illegal_char(
                            errbuf.ptr() as *mut c_char,
                            ::core::mem::size_of::<[c_char; 80]>(),
                            '}' as c_int,
                        );
                    }
                    while (*s as c_int != '}' as c_int
                        || reevaluate as c_int != 0
                            && *s.offset(-1 as c_int as isize) as c_int != '%' as c_int)
                        && *s as c_int != 0
                    {
                        s = s.offset(1);
                    }
                    if *s as c_int != '}' as c_int {
                        return (e_unclosed_expression_sequence.ptr() as *const _) as *const c_char;
                    }
                }
            }
        }
    }
    if groupdepth != 0 as c_int {
        return (e_unbalanced_groups.ptr() as *const _) as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn check_illegal_path_names(
    mut val: *mut c_char,
    mut flags: uint32_t,
) -> bool {
    return flags & kOptFlagNFname as c_int as uint32_t != 0
        && !strpbrk(
            val,
            if secure.get() != 0 {
                b"/\\*?[|;&<>\r\n\0".as_ptr() as *const c_char
            } else {
                b"/\\*?[<>\r\n\0".as_ptr() as *const c_char
            },
        )
        .is_null()
        || flags & kOptFlagNDname as c_int as uint32_t != 0
            && !strpbrk(val, b"*?[|;&<>\r\n\0".as_ptr() as *const c_char).is_null();
}

pub(crate) unsafe extern "C" fn check_str_opt(
    mut idx: OptIndex,
    mut varp: *mut *mut c_char,
) -> c_int {
    let mut opt: *mut vimoption_T = get_option(idx);
    if varp.is_null() {
        varp = (*opt).var as *mut *mut c_char;
    }
    let mut list: bool =
        (*opt).flags & (kOptFlagComma as c_int | kOptFlagOneComma as c_int) as uint32_t != 0;
    let mut values: *mut *const c_char = opt_values(idx, ::core::ptr::null_mut::<size_t>());
    return opt_strings_flags(*varp, values, (*opt).flags_var, list);
}
