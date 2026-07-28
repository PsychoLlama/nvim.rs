//! The callbacks for options that decide how a buffer's text is read,
//! written and understood.
//!
//! They are `pub` only so the generated option table can name them.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn did_set_backspace(mut args: *mut optset_T) -> *const c_char {
    if ascii_isdigit(*p_bs.get() as c_int) {
        if *p_bs.get() as c_int != '2' as c_int {
            return &raw const e_invarg as *const c_char;
        }
        return ::core::ptr::null::<c_char>();
    }
    return did_set_str_generic(args);
}

pub unsafe extern "C" fn did_set_backupcopy(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut oldval: *const c_char = (*args).os_oldval.string.data;
    let mut opt_flags: c_int = (*args).os_flags;
    let mut bkc: *mut c_char = p_bkc.get();
    let mut flags: *mut c_uint = bkc_flags.ptr();
    if opt_flags & OPT_LOCAL as c_int != 0 {
        bkc = (*buf).b_p_bkc;
        flags = &raw mut (*buf).b_bkc_flags;
    } else if opt_flags & OPT_GLOBAL as c_int == 0 {
        (*buf).b_bkc_flags = 0 as c_uint;
    }
    if opt_flags & OPT_LOCAL as c_int != 0 && *bkc as c_int == NUL {
        *flags = 0 as c_uint;
    } else {
        if opt_strings_flags(
            bkc,
            opt_bkc_values.ptr() as *mut *const c_char,
            flags,
            true_0 != 0,
        ) != OK
        {
            return &raw const e_invarg as *const c_char;
        }
        if (*flags & kOptBkcFlagAuto as c_int as c_uint != 0 as c_uint) as c_int
            + (*flags & kOptBkcFlagYes as c_int as c_uint != 0 as c_uint) as c_int
            + (*flags & kOptBkcFlagNo as c_int as c_uint != 0 as c_uint) as c_int
            != 1 as c_int
        {
            opt_strings_flags(
                oldval,
                opt_bkc_values.ptr() as *mut *const c_char,
                flags,
                true_0 != 0,
            );
            return &raw const e_invarg as *const c_char;
        }
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_backupext_or_patchmode(mut _args: *mut optset_T) -> *const c_char {
    if strcmp(
        if *p_bex.get() as c_int == '.' as c_int {
            (*p_bex.ptr()).offset(1 as c_int as isize)
        } else {
            p_bex.get()
        },
        if *p_pm.get() as c_int == '.' as c_int {
            (*p_pm.ptr()).offset(1 as c_int as isize)
        } else {
            p_pm.get()
        },
    ) == 0 as c_int
    {
        return (e_backupext_and_patchmode_are_equal.ptr() as *const _) as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_bufhidden(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    return did_set_opt_flags(
        (*buf).b_p_bh,
        opt_bh_values.ptr() as *mut *const c_char,
        ::core::ptr::null_mut::<c_uint>(),
        false_0 != 0,
    );
}

pub unsafe extern "C" fn did_set_buftype(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if !(*buf).terminal.is_null()
        && *(*buf).b_p_bt.offset(0 as c_int as isize) as c_int != 't' as c_int
        || (*buf).terminal.is_null()
            && *(*buf).b_p_bt.offset(0 as c_int as isize) as c_int == 't' as c_int
        || opt_strings_flags(
            (*buf).b_p_bt,
            opt_bt_values.ptr() as *mut *const c_char,
            ::core::ptr::null_mut::<c_uint>(),
            false_0 != 0,
        ) != OK
    {
        return &raw const e_invarg as *const c_char;
    }
    if *(*buf).b_p_bt.offset(0 as c_int as isize) as c_int == 'p' as c_int {
        set_option_direct(
            kOptComments,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"\0".as_ptr() as *const c_char as *mut c_char,
                        size: ::core::mem::size_of::<[c_char; 1]>().wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as c_int,
            SID_NONE,
        );
        let mut next_prompt: pos_T = pos_T {
            lnum: (*buf).b_ml.ml_line_count,
            col: (*buf).b_prompt_start.mark.col,
            coladd: 0 as colnr_T,
        };
        let fmarkp___: *mut fmark_T = &raw mut (*buf).b_prompt_start;
        free_fmark(*fmarkp___);
        let fmarkp__: *mut fmark_T = fmarkp___;
        (*fmarkp__).mark = next_prompt;
        (*fmarkp__).fnum = 0 as c_int;
        (*fmarkp__).timestamp = os_time();
        (*fmarkp__).view = fmarkv_T {
            topline_offset: MAXLNUM as c_int as linenr_T,
            skipcol: 0 as colnr_T,
        };
        (*fmarkp__).additional_data = ::core::ptr::null_mut::<AdditionalData>();
    }
    if (*win).w_status_height != 0 || global_stl_height() != 0 {
        (*win).w_redr_status = true_0 != 0;
        redraw_later(win, UPD_VALID as c_int);
    }
    (*buf).b_help = *(*buf).b_p_bt.offset(0 as c_int as isize) as c_int == 'h' as c_int;
    redraw_titles();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_cinoptions(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    parse_cino(buf);
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_comments(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut errmsg: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut s: *mut c_char = *varp;
    while *s != 0 {
        while *s as c_int != 0 && *s as c_int != ':' as c_int {
            if vim_strchr(COM_ALL.as_ptr(), *s as uint8_t as c_int).is_null()
                && !ascii_isdigit(*s as c_int)
                && *s as c_int != '-' as c_int
            {
                errmsg = illegal_char(
                    (*args).os_errbuf,
                    (*args).os_errbuflen,
                    *s as uint8_t as c_int,
                );
                break;
            } else {
                s = s.offset(1);
            }
        }
        let c2rust_fresh4 = s;
        s = s.offset(1);
        if *c2rust_fresh4 as c_int == NUL {
            errmsg = b"E524: Missing colon\0".as_ptr() as *const c_char as *mut c_char;
        } else if *s as c_int == ',' as c_int || *s as c_int == NUL {
            errmsg = b"E525: Zero length string\0".as_ptr() as *const c_char as *mut c_char;
        }
        if !errmsg.is_null() {
            break;
        }
        while *s as c_int != 0 && *s as c_int != ',' as c_int {
            if *s as c_int == '\\' as c_int && *s.offset(1 as c_int as isize) as c_int != NUL {
                s = s.offset(1);
            }
            s = s.offset(1);
        }
        s = skip_to_option_part(s);
    }
    return errmsg;
}

pub unsafe extern "C" fn did_set_commentstring(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if **varp as c_int != NUL && strstr(*varp, b"%s\0".as_ptr() as *const c_char).is_null() {
        return b"E537: 'commentstring' must be empty or contain %s\0".as_ptr() as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_cpoptions(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return did_set_option_listflag(
        *varp,
        CPO_VI.as_ptr() as *mut c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}

pub unsafe extern "C" fn did_set_diffanchors(mut args: *mut optset_T) -> *const c_char {
    if diffanchors_changed((*args).os_flags & OPT_LOCAL as c_int != 0) == FAIL {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_diffopt(mut _args: *mut optset_T) -> *const c_char {
    return if diffopt_changed() == FAIL {
        &raw const e_invarg as *const c_char
    } else {
        ::core::ptr::null::<c_char>()
    };
}

pub unsafe extern "C" fn did_set_encoding(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut opt_flags: c_int = (*args).os_flags;
    let mut gvarp: *mut *mut c_char = get_option_varp_scope_from(
        (*args).os_idx,
        OPT_GLOBAL as c_int,
        buf,
        ::core::ptr::null_mut::<win_T>(),
    ) as *mut *mut c_char;
    if gvarp == p_fenc.ptr() {
        if (*buf).b_p_ma == 0 && opt_flags != OPT_GLOBAL as c_int {
            return &raw const e_modifiable as *const c_char;
        }
        if !vim_strchr(*varp, ',' as c_int).is_null() {
            return &raw const e_invarg as *const c_char;
        }
        redraw_titles();
        ml_setflags(buf);
    }
    let mut p: *mut c_char = enc_canonize(*varp);
    xfree(*varp as *mut c_void);
    *varp = p;
    if varp == p_enc.ptr() {
        if strcmp(p_enc.get(), b"utf-8\0".as_ptr() as *const c_char) != 0 as c_int {
            return &raw const e_unsupportedoption as *const c_char;
        }
        spell_reload();
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_eventignore(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if check_ei(*varp) == FAIL {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_fileformat(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut oldval: *const c_char = (*args).os_oldval.string.data;
    let mut opt_flags: c_int = (*args).os_flags;
    if (*buf).b_p_ma == 0 && opt_flags & OPT_GLOBAL as c_int == 0 {
        return &raw const e_modifiable as *const c_char;
    }
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    redraw_titles();
    ml_setflags(buf);
    if get_fileformat(buf) == EOL_MAC || *oldval as c_int == 'm' as c_int {
        redraw_buf_later(buf, UPD_NOT_VALID as c_int);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_filetype_or_syntax(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if !valid_filetype(*varp) {
        return &raw const e_invarg as *const c_char;
    }
    (*args).os_value_changed = strcmp((*args).os_oldval.string.data, *varp) != 0 as c_int;
    (*args).os_value_checked = true_0 != 0;
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_foldexpr(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    did_set_optexpr(args);
    if foldmethodIsExpr(win) {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_foldignore(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if foldmethodIsIndent(win) {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_foldmarker(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut p: *mut c_char = vim_strchr(*varp, ',' as c_int);
    if p.is_null() {
        return (e_comma_required.ptr() as *const _) as *const c_char;
    }
    if p == *varp || *p.offset(1 as c_int as isize) as c_int == NUL {
        return &raw const e_invarg as *const c_char;
    }
    if foldmethodIsMarker(win) {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_foldmethod(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    foldUpdateAll(win);
    if foldmethodIsDiff(win) {
        newFoldLevel();
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_formatoptions(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return did_set_option_listflag(
        *varp,
        FO_ALL.as_ptr() as *mut c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}

pub unsafe extern "C" fn did_set_iskeyword(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if varp == p_isk.ptr() {
        if check_isopt(*varp) == FAIL {
            return &raw const e_invarg as *const c_char;
        }
    } else {
        return did_set_isopt(args);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_isopt(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if !buf_init_chartab(buf, true) {
        (*args).os_restore_chartab = true_0 != 0;
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_keymap(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut opt_flags: c_int = (*args).os_flags;
    if !valid_filetype(*varp) {
        return &raw const e_invarg as *const c_char;
    }
    let mut secure_save: c_int = secure.get();
    secure.set(0 as c_int);
    let mut errmsg: *const c_char = keymap_init();
    secure.set(secure_save);
    (*args).os_value_checked = true_0 != 0;
    if errmsg.is_null() {
        if *(*buf).b_p_keymap as c_int != NUL {
            (*buf).b_p_iminsert = B_IMODE_LMAP as OptInt;
            if (*buf).b_p_imsearch != B_IMODE_USE_INSERT as OptInt {
                (*buf).b_p_imsearch = B_IMODE_LMAP as OptInt;
            }
        } else {
            if (*buf).b_p_iminsert == B_IMODE_LMAP as OptInt {
                (*buf).b_p_iminsert = B_IMODE_NONE as OptInt;
            }
            if (*buf).b_p_imsearch == B_IMODE_LMAP as OptInt {
                (*buf).b_p_imsearch = B_IMODE_USE_INSERT as OptInt;
            }
        }
        if opt_flags & OPT_LOCAL as c_int == 0 as c_int {
            set_iminsert_global(buf);
            set_imsearch_global(buf);
        }
        status_redraw_buf(buf);
    }
    return errmsg;
}

pub unsafe extern "C" fn did_set_lispoptions(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if **varp as c_int != NUL
        && strcmp(*varp, b"expr:0\0".as_ptr() as *const c_char) != 0 as c_int
        && strcmp(*varp, b"expr:1\0".as_ptr() as *const c_char) != 0 as c_int
    {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_matchpairs(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut p: *mut c_char = *varp;
    while *p as c_int != NUL {
        let mut x2: c_int = -1 as c_int;
        let mut x3: c_int = -1 as c_int;
        p = p.offset(utfc_ptr2len(p) as isize);
        if *p as c_int != NUL {
            let c2rust_fresh9 = p;
            p = p.offset(1);
            x2 = *c2rust_fresh9 as c_uchar as c_int;
        }
        if *p as c_int != NUL {
            x3 = utf_ptr2char(p);
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        if x2 != ':' as c_int
            || x3 == -1 as c_int
            || *p as c_int != NUL && *p as c_int != ',' as c_int
        {
            return &raw const e_invarg as *const c_char;
        }
        if *p as c_int == NUL {
            break;
        }
        p = p.offset(1);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_varsofttabstop(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if *(*varp).offset(0 as c_int as isize) == 0
        || *(*varp).offset(0 as c_int as isize) as c_int == '0' as c_int
            && *(*varp).offset(1 as c_int as isize) == 0
    {
        let mut ptr_: *mut *mut c_void = &raw mut (*buf).b_p_vsts_array as *mut *mut c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        return ::core::ptr::null::<c_char>();
    }
    let mut cp: *mut c_char = *varp;
    while *cp != 0 {
        if !ascii_isdigit(*cp as c_int) {
            if !(*cp as c_int == ',' as c_int
                && cp > *varp
                && *cp.offset(-(1 as c_int as isize)) as c_int != ',' as c_int)
            {
                return &raw const e_invarg as *const c_char;
            }
        }
        cp = cp.offset(1);
    }
    let mut oldarray: *mut colnr_T = (*buf).b_p_vsts_array;
    if tabstop_set(*varp, &raw mut (*buf).b_p_vsts_array) {
        xfree(oldarray as *mut c_void);
    } else {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_vartabstop(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if *(*varp).offset(0 as c_int as isize) == 0
        || *(*varp).offset(0 as c_int as isize) as c_int == '0' as c_int
            && *(*varp).offset(1 as c_int as isize) == 0
    {
        let mut ptr_: *mut *mut c_void = &raw mut (*buf).b_p_vts_array as *mut *mut c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        return ::core::ptr::null::<c_char>();
    }
    let mut cp: *mut c_char = *varp;
    while *cp != 0 {
        if !ascii_isdigit(*cp as c_int) {
            if !(*cp as c_int == ',' as c_int
                && cp > *varp
                && *cp.offset(-(1 as c_int as isize)) as c_int != ',' as c_int)
            {
                return &raw const e_invarg as *const c_char;
            }
        }
        cp = cp.offset(1);
    }
    let mut oldarray: *mut colnr_T = (*buf).b_p_vts_array;
    if tabstop_set(*varp, &raw mut (*buf).b_p_vts_array) {
        xfree(oldarray as *mut c_void);
        if foldmethodIsIndent(win) {
            foldUpdateAll(win);
        }
    } else {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
