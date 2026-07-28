//! Re-checking options after something outside `:set` changed them.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn did_set_title() {
    if starting.get() != NO_SCREEN {
        maketitle();
    }
}

pub unsafe extern "C" fn set_options_bin(
    mut oldval: c_int,
    mut newval: c_int,
    mut opt_flags: c_int,
) {
    if newval != 0 {
        if oldval == 0 {
            if opt_flags & OPT_GLOBAL as c_int == 0 {
                (*curbuf.get()).b_p_tw_nobin = (*curbuf.get()).b_p_tw;
                (*curbuf.get()).b_p_wm_nobin = (*curbuf.get()).b_p_wm;
                (*curbuf.get()).b_p_ml_nobin = (*curbuf.get()).b_p_ml;
                (*curbuf.get()).b_p_et_nobin = (*curbuf.get()).b_p_et;
            }
            if opt_flags & OPT_LOCAL as c_int == 0 {
                p_tw_nobin.set(p_tw.get());
                p_wm_nobin.set(p_wm.get());
                p_ml_nobin.set(p_ml.get());
                p_et_nobin.set(p_et.get());
            }
        }
        if opt_flags & OPT_GLOBAL as c_int == 0 {
            (*curbuf.get()).b_p_tw = 0 as OptInt;
            (*curbuf.get()).b_p_wm = 0 as OptInt;
            (*curbuf.get()).b_p_ml = 0 as c_int;
            (*curbuf.get()).b_p_et = 0 as c_int;
        }
        if opt_flags & OPT_LOCAL as c_int == 0 {
            p_tw.set(0 as OptInt);
            p_wm.set(0 as OptInt);
            p_ml.set(false_0);
            p_et.set(false_0);
            p_bin.set(true_0);
        }
    } else if oldval != 0 {
        if opt_flags & OPT_GLOBAL as c_int == 0 {
            (*curbuf.get()).b_p_tw = (*curbuf.get()).b_p_tw_nobin;
            (*curbuf.get()).b_p_wm = (*curbuf.get()).b_p_wm_nobin;
            (*curbuf.get()).b_p_ml = (*curbuf.get()).b_p_ml_nobin;
            (*curbuf.get()).b_p_et = (*curbuf.get()).b_p_et_nobin;
        }
        if opt_flags & OPT_LOCAL as c_int == 0 {
            p_tw.set(p_tw_nobin.get());
            p_wm.set(p_wm_nobin.get());
            p_ml.set(p_ml_nobin.get());
            p_et.set(p_et_nobin.get());
        }
    }
    didset_options_sctx(opt_flags, p_bin_dep_opts.ptr() as *mut c_int);
}

pub(crate) unsafe extern "C" fn didset_options() {
    init_chartab();
    didset_string_options();
    spell_check_msm();
    spell_check_sps();
    compile_cap_prog((*curwin.get()).w_s);
    did_set_spell_option();
    did_set_cedit(::core::ptr::null_mut::<optset_T>());
    did_set_breakat(::core::ptr::null_mut::<optset_T>());
    didset_window_options(curwin.get(), true_0 != 0);
}

pub(crate) unsafe extern "C" fn didset_options2() {
    highlight_changed();
    set_chars_option(
        curwin.get(),
        (*curwin.get()).w_onebuf_opt.wo_fcs,
        kFillchars,
        true_0 != 0,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
    );
    set_chars_option(
        curwin.get(),
        (*curwin.get()).w_onebuf_opt.wo_lcs,
        kListchars,
        true_0 != 0,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
    );
    check_opt_wim();
    xfree((*curbuf.get()).b_p_vsts_array as *mut c_void);
    tabstop_set(
        (*curbuf.get()).b_p_vsts,
        &raw mut (*curbuf.get()).b_p_vsts_array,
    );
    xfree((*curbuf.get()).b_p_vts_array as *mut c_void);
    tabstop_set(
        (*curbuf.get()).b_p_vts,
        &raw mut (*curbuf.get()).b_p_vts_array,
    );
}

pub unsafe extern "C" fn check_options() {
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        if option_has_type(opt_idx, kOptValTypeString) as c_int != 0
            && !(*options.ptr())[opt_idx as usize].var.is_null()
        {
            check_string_option(get_varp(
                (options.ptr() as *mut vimoption_T).offset(opt_idx as isize),
            ) as *mut *mut c_char);
        }
        opt_idx += 1;
    }
}

pub unsafe extern "C" fn was_set_insecurely(
    wp: *mut win_T,
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
) -> c_int {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                1855 as c_uint,
                b"int was_set_insecurely(win_T *const, OptIndex, int)\0".as_ptr() as *const c_char,
            );
        }
    };
    let mut flagp: *mut uint32_t = insecure_flag(wp, opt_idx, opt_flags);
    return (*flagp & kOptFlagInsecure as c_int as uint32_t != 0 as uint32_t) as c_int;
}

pub unsafe extern "C" fn insecure_flag(
    wp: *mut win_T,
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
) -> *mut uint32_t {
    if opt_flags & OPT_LOCAL as c_int != 0 {
        '_c2rust_label: {
            if !wp.is_null() {
            } else {
                __assert_fail(
                    b"wp != NULL\0".as_ptr() as *const c_char,
                    b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                    1868 as c_uint,
                    b"uint32_t *insecure_flag(win_T *const, OptIndex, int)\0".as_ptr()
                        as *const c_char,
                );
            }
        };
        match opt_idx as c_int {
            367 => return &raw mut (*wp).w_onebuf_opt.wo_wrap_flags,
            294 => return &raw mut (*wp).w_onebuf_opt.wo_stl_flags,
            355 => return &raw mut (*wp).w_onebuf_opt.wo_wbr_flags,
            104 => return &raw mut (*wp).w_onebuf_opt.wo_fde_flags,
            113 => return &raw mut (*wp).w_onebuf_opt.wo_fdt_flags,
            148 => return &raw mut (*(*wp).w_buffer).b_p_inde_flags,
            114 => return &raw mut (*(*wp).w_buffer).b_p_fex_flags,
            146 => return &raw mut (*(*wp).w_buffer).b_p_inex_flags,
            _ => {}
        }
    } else {
        match opt_idx as c_int {
            367 => return &raw mut (*wp).w_allbuf_opt.wo_wrap_flags,
            104 => return &raw mut (*wp).w_allbuf_opt.wo_fde_flags,
            113 => return &raw mut (*wp).w_allbuf_opt.wo_fdt_flags,
            _ => {}
        }
    }
    return &raw mut (*(options.ptr() as *mut vimoption_T).offset(opt_idx as isize)).flags;
}

pub unsafe extern "C" fn redraw_titles() {
    need_maketitle.set(true_0 != 0);
    redraw_tabline.set(true_0 != 0);
}

pub unsafe extern "C" fn valid_name(mut val: *const c_char, mut allowed: *const c_char) -> bool {
    let mut s: *const c_char = val;
    while *s as c_int != NUL {
        if !(*s as c_uint >= 'A' as c_uint && *s as c_uint <= 'Z' as c_uint
            || *s as c_uint >= 'a' as c_uint && *s as c_uint <= 'z' as c_uint
            || ascii_isdigit(*s as c_int) as c_int != 0)
            && vim_strchr(allowed, *s as uint8_t as c_int).is_null()
        {
            return false_0 != 0;
        }
        s = s.offset(1);
    }
    return true_0 != 0;
}

pub unsafe extern "C" fn check_blending(mut wp: *mut win_T) {
    (*wp).w_grid_alloc.blending = (*wp).w_onebuf_opt.wo_winbl > 0 as OptInt
        || (*wp).w_floating as c_int != 0 && (*wp).w_config.shadow as c_int != 0;
}

pub unsafe extern "C" fn parse_winhl_opt(mut winhl: *const c_char, mut wp: *mut win_T) -> bool {
    let mut p: *const c_char = empty_string_option.ptr() as *mut c_char;
    if !winhl.is_null() {
        p = winhl;
    } else if !wp.is_null() {
        p = (*wp).w_onebuf_opt.wo_winhl;
    }
    if *p == 0 {
        if !wp.is_null() && (*wp).w_ns_hl_winhl > 0 as c_int && (*wp).w_ns_hl == (*wp).w_ns_hl_winhl
        {
            (*wp).w_ns_hl = 0 as c_int;
            (*wp).w_hl_needs_update = true_0;
        }
        return true_0 != 0;
    }
    let mut ns_hl: c_int = 0 as c_int;
    if !wp.is_null() {
        if (*wp).w_ns_hl_winhl == 0 as c_int {
            (*wp).w_ns_hl_winhl = nvim_create_namespace(NULL_STRING) as c_int;
        } else {
            let mut dp: *mut DecorProvider =
                get_decor_provider((*wp).w_ns_hl_winhl as NS, true_0 != 0);
            (*dp).hl_valid += 1;
        }
        ns_hl = (*wp).w_ns_hl_winhl;
        if (*wp).w_ns_hl <= 0 as c_int {
            (*wp).w_ns_hl = (*wp).w_ns_hl_winhl;
        }
    }
    while *p != 0 {
        let mut colon: *const c_char = strchr(p, ':' as c_int);
        if colon.is_null() {
            return false_0 != 0;
        }
        let mut nlen: size_t = colon.offset_from(p) as size_t;
        let mut hi: *const c_char = colon.offset(1 as c_int as isize);
        let mut commap: *const c_char = xstrchrnul(hi, ',' as c_char);
        let mut len: size_t = commap.offset_from(hi) as size_t;
        let mut hl_id: c_int = if len != 0 {
            syn_check_group(hi, len)
        } else {
            -1 as c_int
        };
        if hl_id == 0 as c_int {
            return false_0 != 0;
        }
        let mut hl_id_link: c_int = if nlen != 0 {
            syn_check_group(p, nlen)
        } else {
            0 as c_int
        };
        if hl_id_link == 0 as c_int {
            return false_0 != 0;
        }
        if !wp.is_null() {
            let mut attrs: HlAttrs = HLATTRS_INIT;
            attrs.rgb_ae_attr = (attrs.rgb_ae_attr as c_int | HL_GLOBAL as c_int) as int32_t;
            ns_hl_def(
                ns_hl as NS,
                hl_id_link,
                attrs,
                hl_id,
                ::core::ptr::null_mut::<KeyDict_highlight>(),
            );
        }
        p = if *commap as c_int != 0 {
            commap.offset(1 as c_int as isize)
        } else {
            b"\0".as_ptr() as *const c_char
        };
    }
    if !wp.is_null() {
        (*wp).w_hl_needs_update = true_0;
    }
    return true_0 != 0;
}

pub unsafe extern "C" fn check_redraw_for(
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut flags: uint32_t,
) {
    let mut all: bool =
        flags & kOptFlagRedrAll as c_int as uint32_t == kOptFlagRedrAll as c_int as uint32_t;
    if flags & kOptFlagRedrStat as c_int as uint32_t != 0 || all as c_int != 0 {
        status_redraw_all();
    }
    if flags & kOptFlagRedrTabl as c_int as uint32_t != 0 || all as c_int != 0 {
        redraw_tabline.set(true_0 != 0);
    }
    if flags & kOptFlagRedrBuf as c_int as uint32_t != 0
        || flags & kOptFlagRedrWin as c_int as uint32_t != 0
        || all as c_int != 0
    {
        if flags & kOptFlagHLOnly as c_int as uint32_t != 0 {
            redraw_later(win, UPD_NOT_VALID as c_int);
        } else {
            changed_window_setting(win);
        }
    }
    if flags & kOptFlagRedrBuf as c_int as uint32_t != 0 {
        redraw_buf_later(buf, UPD_NOT_VALID as c_int);
    }
    if all {
        redraw_all_later(UPD_NOT_VALID as c_int);
    }
}

pub unsafe extern "C" fn check_redraw(mut flags: uint32_t) {
    check_redraw_for(curbuf.get(), curwin.get(), flags);
}
