//! Which copy of a value a scope is looking at — the `varp` plumbing.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn is_option_hidden(mut opt_idx: OptIndex) -> bool {
    return opt_idx as c_int != kOptInvalid as c_int
        && (*options.ptr())[opt_idx as usize].immutable as c_int != 0
        && (*options.ptr())[opt_idx as usize].var
            == &raw mut (*(options.ptr() as *mut vimoption_T).offset(opt_idx as isize))
                .def_val
                .data as *mut c_void;
}

pub unsafe extern "C" fn option_has_type(mut opt_idx: OptIndex, mut type_0: OptValType) -> bool {
    return opt_idx as c_int != kOptInvalid as c_int
        && (*options.ptr())[opt_idx as usize].type_0 as c_int == type_0 as c_int;
}

pub unsafe extern "C" fn option_has_scope(mut opt_idx: OptIndex, mut scope: OptScope) -> bool {
    '_c2rust_label: {
        if scope as c_uint >= kOptScopeGlobal as c_int as c_uint
            && (scope as c_uint) < (kOptScopeBuf as c_int + 1 as c_int) as c_uint
        {
        } else {
            __assert_fail(
                b"scope >= kOptScopeGlobal && scope < kOptScopeSize\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                3512 as c_uint,
                b"_Bool option_has_scope(OptIndex, OptScope)\0".as_ptr() as *const c_char,
            );
        }
    };
    return (*get_option(opt_idx)).scope_flags as c_int & (1 as c_int) << scope as c_uint != 0;
}

#[inline]
pub(crate) unsafe extern "C" fn option_is_global_local(mut opt_idx: OptIndex) -> bool {
    return opt_idx as c_int != kOptInvalid as c_int
        && !is_power_of_two((*options.ptr())[opt_idx as usize].scope_flags as uint64_t);
}

#[inline]
pub(crate) unsafe extern "C" fn option_is_global_only(mut opt_idx: OptIndex) -> bool {
    return opt_idx as c_int != kOptInvalid as c_int
        && is_power_of_two((*options.ptr())[opt_idx as usize].scope_flags as uint64_t) as c_int
            != 0
        && option_has_scope(opt_idx, kOptScopeGlobal) as c_int != 0;
}

#[inline]
pub(crate) unsafe extern "C" fn option_is_window_local(mut opt_idx: OptIndex) -> bool {
    return opt_idx as c_int != kOptInvalid as c_int
        && is_power_of_two((*options.ptr())[opt_idx as usize].scope_flags as uint64_t) as c_int
            != 0
        && option_has_scope(opt_idx, kOptScopeWin) as c_int != 0;
}

pub unsafe extern "C" fn option_scope_idx(mut opt_idx: OptIndex, mut scope: OptScope) -> ssize_t {
    return (*options.ptr())[opt_idx as usize].scope_idx[scope as usize];
}

pub unsafe extern "C" fn get_varp_scope_from(
    mut p: *mut vimoption_T,
    mut opt_flags: c_int,
    mut buf: *mut buf_T,
    mut win: *mut win_T,
) -> *mut c_void {
    let mut opt_idx: OptIndex = get_opt_idx(p);
    if opt_flags & OPT_GLOBAL as c_int != 0 && !option_is_global_only(opt_idx) {
        if option_is_window_local(opt_idx) {
            return (get_varp_from(p, buf, win) as *mut c_char)
                .offset(::core::mem::size_of::<winopt_T>() as isize)
                as *mut c_void;
        }
        return (*p).var;
    }
    if opt_flags & OPT_LOCAL as c_int != 0 && option_is_global_local(opt_idx) as c_int != 0 {
        match opt_idx as c_int {
            117 => return &raw mut (*buf).b_p_fp as *mut c_void,
            118 => return &raw mut (*buf).b_p_fs as *mut c_void,
            99 => return &raw mut (*buf).b_p_ffu as *mut c_void,
            87 => return &raw mut (*buf).b_p_efm as *mut c_void,
            120 => return &raw mut (*buf).b_p_gefm as *mut c_void,
            121 => return &raw mut (*buf).b_p_gp as *mut c_void,
            180 => return &raw mut (*buf).b_p_mp as *mut c_void,
            84 => return &raw mut (*buf).b_p_ep as *mut c_void,
            160 => return &raw mut (*buf).b_p_kp as *mut c_void,
            217 => return &raw mut (*buf).b_p_path as *mut c_void,
            6 => return &raw mut (*buf).b_p_ac as *mut c_void,
            10 => return &raw mut (*buf).b_p_ar as *mut c_void,
            310 => return &raw mut (*buf).b_p_tags as *mut c_void,
            306 => return &raw mut (*buf).b_p_tc as *mut c_void,
            276 => {
                return &raw mut (*win).w_onebuf_opt.wo_siso as *mut c_void;
            }
            247 => return &raw mut (*win).w_onebuf_opt.wo_so as *mut c_void,
            67 => return &raw mut (*buf).b_p_def as *mut c_void,
            145 => return &raw mut (*buf).b_p_inc as *mut c_void,
            54 => return &raw mut (*buf).b_p_cot as *mut c_void,
            69 => return &raw mut (*buf).b_p_dict as *mut c_void,
            71 => return &raw mut (*buf).b_p_dia as *mut c_void,
            319 => return &raw mut (*buf).b_p_tsr as *mut c_void,
            320 => return &raw mut (*buf).b_p_tsrfu as *mut c_void,
            307 => return &raw mut (*buf).b_p_tfu as *mut c_void,
            268 => return &raw mut (*win).w_onebuf_opt.wo_sbr as *mut c_void,
            294 => return &raw mut (*win).w_onebuf_opt.wo_stl as *mut c_void,
            355 => return &raw mut (*win).w_onebuf_opt.wo_wbr as *mut c_void,
            333 => return &raw mut (*buf).b_p_ul as *mut c_void,
            173 => return &raw mut (*buf).b_p_lw as *mut c_void,
            16 => return &raw mut (*buf).b_p_bkc as *mut c_void,
            179 => return &raw mut (*buf).b_p_menc as *mut c_void,
            98 => return &raw mut (*win).w_onebuf_opt.wo_fcs as *mut c_void,
            175 => return &raw mut (*win).w_onebuf_opt.wo_lcs as *mut c_void,
            343 => return &raw mut (*win).w_onebuf_opt.wo_ve as *mut c_void,
            _ => {
                abort();
            }
        }
    }
    return get_varp_from(p, buf, win);
}

pub unsafe extern "C" fn get_varp_scope(
    mut p: *mut vimoption_T,
    mut opt_flags: c_int,
) -> *mut c_void {
    return get_varp_scope_from(p, opt_flags, curbuf.get(), curwin.get());
}

pub unsafe extern "C" fn get_option_varp_scope_from(
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
    mut buf: *mut buf_T,
    mut win: *mut win_T,
) -> *mut c_void {
    return get_varp_scope_from(
        (options.ptr() as *mut vimoption_T).offset(opt_idx as isize),
        opt_flags,
        buf,
        win,
    );
}

pub unsafe extern "C" fn get_varp_from(
    mut p: *mut vimoption_T,
    mut buf: *mut buf_T,
    mut win: *mut win_T,
) -> *mut c_void {
    let mut opt_idx: OptIndex = get_opt_idx(p);
    if is_option_hidden(opt_idx) as c_int != 0 || option_is_global_only(opt_idx) as c_int != 0 {
        return (*p).var;
    }
    match opt_idx as c_int {
        84 => {
            return if *(*buf).b_p_ep as c_int != NUL {
                &raw mut (*buf).b_p_ep as *mut c_void
            } else {
                (*p).var
            };
        }
        160 => {
            return if *(*buf).b_p_kp as c_int != NUL {
                &raw mut (*buf).b_p_kp as *mut c_void
            } else {
                (*p).var
            };
        }
        217 => {
            return if *(*buf).b_p_path as c_int != NUL {
                &raw mut (*buf).b_p_path as *mut c_void
            } else {
                (*p).var
            };
        }
        6 => {
            return if (*buf).b_p_ac >= 0 as c_int {
                &raw mut (*buf).b_p_ac as *mut c_void
            } else {
                (*p).var
            };
        }
        10 => {
            return if (*buf).b_p_ar >= 0 as c_int {
                &raw mut (*buf).b_p_ar as *mut c_void
            } else {
                (*p).var
            };
        }
        310 => {
            return if *(*buf).b_p_tags as c_int != NUL {
                &raw mut (*buf).b_p_tags as *mut c_void
            } else {
                (*p).var
            };
        }
        306 => {
            return if *(*buf).b_p_tc as c_int != NUL {
                &raw mut (*buf).b_p_tc as *mut c_void
            } else {
                (*p).var
            };
        }
        276 => {
            return if (*win).w_onebuf_opt.wo_siso >= 0 as OptInt {
                &raw mut (*win).w_onebuf_opt.wo_siso as *mut c_void
            } else {
                (*p).var
            };
        }
        247 => {
            return if (*win).w_onebuf_opt.wo_so >= 0 as OptInt {
                &raw mut (*win).w_onebuf_opt.wo_so as *mut c_void
            } else {
                (*p).var
            };
        }
        16 => {
            return if *(*buf).b_p_bkc as c_int != NUL {
                &raw mut (*buf).b_p_bkc as *mut c_void
            } else {
                (*p).var
            };
        }
        67 => {
            return if *(*buf).b_p_def as c_int != NUL {
                &raw mut (*buf).b_p_def as *mut c_void
            } else {
                (*p).var
            };
        }
        145 => {
            return if *(*buf).b_p_inc as c_int != NUL {
                &raw mut (*buf).b_p_inc as *mut c_void
            } else {
                (*p).var
            };
        }
        54 => {
            return if *(*buf).b_p_cot as c_int != NUL {
                &raw mut (*buf).b_p_cot as *mut c_void
            } else {
                (*p).var
            };
        }
        69 => {
            return if *(*buf).b_p_dict as c_int != NUL {
                &raw mut (*buf).b_p_dict as *mut c_void
            } else {
                (*p).var
            };
        }
        71 => {
            return if *(*buf).b_p_dia as c_int != NUL {
                &raw mut (*buf).b_p_dia as *mut c_void
            } else {
                (*p).var
            };
        }
        319 => {
            return if *(*buf).b_p_tsr as c_int != NUL {
                &raw mut (*buf).b_p_tsr as *mut c_void
            } else {
                (*p).var
            };
        }
        320 => {
            return if *(*buf).b_p_tsrfu as c_int != NUL {
                &raw mut (*buf).b_p_tsrfu as *mut c_void
            } else {
                (*p).var
            };
        }
        117 => {
            return if *(*buf).b_p_fp as c_int != NUL {
                &raw mut (*buf).b_p_fp as *mut c_void
            } else {
                (*p).var
            };
        }
        118 => {
            return if (*buf).b_p_fs >= 0 as c_int {
                &raw mut (*buf).b_p_fs as *mut c_void
            } else {
                (*p).var
            };
        }
        99 => {
            return if *(*buf).b_p_ffu as c_int != NUL {
                &raw mut (*buf).b_p_ffu as *mut c_void
            } else {
                (*p).var
            };
        }
        87 => {
            return if *(*buf).b_p_efm as c_int != NUL {
                &raw mut (*buf).b_p_efm as *mut c_void
            } else {
                (*p).var
            };
        }
        120 => {
            return if *(*buf).b_p_gefm as c_int != NUL {
                &raw mut (*buf).b_p_gefm as *mut c_void
            } else {
                (*p).var
            };
        }
        121 => {
            return if *(*buf).b_p_gp as c_int != NUL {
                &raw mut (*buf).b_p_gp as *mut c_void
            } else {
                (*p).var
            };
        }
        180 => {
            return if *(*buf).b_p_mp as c_int != NUL {
                &raw mut (*buf).b_p_mp as *mut c_void
            } else {
                (*p).var
            };
        }
        268 => {
            return if *(*win).w_onebuf_opt.wo_sbr as c_int != NUL {
                &raw mut (*win).w_onebuf_opt.wo_sbr as *mut c_void
            } else {
                (*p).var
            };
        }
        294 => {
            return if *(*win).w_onebuf_opt.wo_stl as c_int != NUL {
                &raw mut (*win).w_onebuf_opt.wo_stl as *mut c_void
            } else {
                (*p).var
            };
        }
        355 => {
            return if *(*win).w_onebuf_opt.wo_wbr as c_int != NUL {
                &raw mut (*win).w_onebuf_opt.wo_wbr as *mut c_void
            } else {
                (*p).var
            };
        }
        333 => {
            return if (*buf).b_p_ul != NO_LOCAL_UNDOLEVEL as OptInt {
                &raw mut (*buf).b_p_ul as *mut c_void
            } else {
                (*p).var
            };
        }
        173 => {
            return if *(*buf).b_p_lw as c_int != NUL {
                &raw mut (*buf).b_p_lw as *mut c_void
            } else {
                (*p).var
            };
        }
        179 => {
            return if *(*buf).b_p_menc as c_int != NUL {
                &raw mut (*buf).b_p_menc as *mut c_void
            } else {
                (*p).var
            };
        }
        98 => {
            return if *(*win).w_onebuf_opt.wo_fcs as c_int != NUL {
                &raw mut (*win).w_onebuf_opt.wo_fcs as *mut c_void
            } else {
                (*p).var
            };
        }
        175 => {
            return if *(*win).w_onebuf_opt.wo_lcs as c_int != NUL {
                &raw mut (*win).w_onebuf_opt.wo_lcs as *mut c_void
            } else {
                (*p).var
            };
        }
        343 => {
            return if *(*win).w_onebuf_opt.wo_ve as c_int != NUL {
                &raw mut (*win).w_onebuf_opt.wo_ve as *mut c_void
            } else {
                (*p).var
            };
        }
        3 => return &raw mut (*win).w_onebuf_opt.wo_arab as *mut c_void,
        174 => return &raw mut (*win).w_onebuf_opt.wo_list as *mut c_void,
        283 => return &raw mut (*win).w_onebuf_opt.wo_spell as *mut c_void,
        63 => return &raw mut (*win).w_onebuf_opt.wo_cuc as *mut c_void,
        64 => return &raw mut (*win).w_onebuf_opt.wo_cul as *mut c_void,
        65 => return &raw mut (*win).w_onebuf_opt.wo_culopt as *mut c_void,
        46 => return &raw mut (*win).w_onebuf_opt.wo_cc as *mut c_void,
        70 => return &raw mut (*win).w_onebuf_opt.wo_diff as *mut c_void,
        89 => return &raw mut (*win).w_onebuf_opt.wo_eiw as *mut c_void,
        102 => return &raw mut (*win).w_onebuf_opt.wo_fdc as *mut c_void,
        103 => return &raw mut (*win).w_onebuf_opt.wo_fen as *mut c_void,
        105 => return &raw mut (*win).w_onebuf_opt.wo_fdi as *mut c_void,
        106 => return &raw mut (*win).w_onebuf_opt.wo_fdl as *mut c_void,
        109 => return &raw mut (*win).w_onebuf_opt.wo_fdm as *mut c_void,
        110 => return &raw mut (*win).w_onebuf_opt.wo_fml as *mut c_void,
        111 => return &raw mut (*win).w_onebuf_opt.wo_fdn as *mut c_void,
        104 => return &raw mut (*win).w_onebuf_opt.wo_fde as *mut c_void,
        113 => return &raw mut (*win).w_onebuf_opt.wo_fdt as *mut c_void,
        108 => return &raw mut (*win).w_onebuf_opt.wo_fmr as *mut c_void,
        206 => return &raw mut (*win).w_onebuf_opt.wo_nu as *mut c_void,
        234 => return &raw mut (*win).w_onebuf_opt.wo_rnu as *mut c_void,
        207 => return &raw mut (*win).w_onebuf_opt.wo_nuw as *mut c_void,
        359 => return &raw mut (*win).w_onebuf_opt.wo_wfb as *mut c_void,
        360 => return &raw mut (*win).w_onebuf_opt.wo_wfh as *mut c_void,
        361 => return &raw mut (*win).w_onebuf_opt.wo_wfw as *mut c_void,
        220 => return &raw mut (*win).w_onebuf_opt.wo_pvw as *mut c_void,
        167 => return &raw mut (*win).w_onebuf_opt.wo_lhi as *mut c_void,
        238 => return &raw mut (*win).w_onebuf_opt.wo_rl as *mut c_void,
        239 => return &raw mut (*win).w_onebuf_opt.wo_rlc as *mut c_void,
        243 => return &raw mut (*win).w_onebuf_opt.wo_scr as *mut c_void,
        281 => return &raw mut (*win).w_onebuf_opt.wo_sms as *mut c_void,
        367 => return &raw mut (*win).w_onebuf_opt.wo_wrap as *mut c_void,
        168 => return &raw mut (*win).w_onebuf_opt.wo_lbr as *mut c_void,
        24 => return &raw mut (*win).w_onebuf_opt.wo_bri as *mut c_void,
        25 => return &raw mut (*win).w_onebuf_opt.wo_briopt as *mut c_void,
        245 => return &raw mut (*win).w_onebuf_opt.wo_scb as *mut c_void,
        62 => return &raw mut (*win).w_onebuf_opt.wo_crb as *mut c_void,
        57 => return &raw mut (*win).w_onebuf_opt.wo_cocu as *mut c_void,
        58 => return &raw mut (*win).w_onebuf_opt.wo_cole as *mut c_void,
        9 => return &raw mut (*buf).b_p_ai as *mut c_void,
        21 => return &raw mut (*buf).b_p_bin as *mut c_void,
        22 => return &raw mut (*buf).b_p_bomb as *mut c_void,
        27 => return &raw mut (*buf).b_p_bh as *mut c_void,
        29 => return &raw mut (*buf).b_p_bt as *mut c_void,
        28 => return &raw mut (*buf).b_p_bl as *mut c_void,
        30 => return &raw mut (*buf).b_p_busy as *mut c_void,
        35 => return &raw mut (*buf).b_p_channel as *mut c_void,
        60 => return &raw mut (*buf).b_p_ci as *mut c_void,
        38 => return &raw mut (*buf).b_p_cin as *mut c_void,
        39 => return &raw mut (*buf).b_p_cink as *mut c_void,
        40 => return &raw mut (*buf).b_p_cino as *mut c_void,
        41 => return &raw mut (*buf).b_p_cinsd as *mut c_void,
        42 => return &raw mut (*buf).b_p_cinw as *mut c_void,
        48 => return &raw mut (*buf).b_p_com as *mut c_void,
        49 => return &raw mut (*buf).b_p_cms as *mut c_void,
        51 => return &raw mut (*buf).b_p_cpt as *mut c_void,
        52 => return &raw mut (*buf).b_p_cfu as *mut c_void,
        208 => return &raw mut (*buf).b_p_ofu as *mut c_void,
        81 => return &raw mut (*buf).b_p_eof as *mut c_void,
        82 => return &raw mut (*buf).b_p_eol as *mut c_void,
        100 => return &raw mut (*buf).b_p_fixeol as *mut c_void,
        90 => return &raw mut (*buf).b_p_et as *mut c_void,
        92 => return &raw mut (*buf).b_p_fenc as *mut c_void,
        94 => return &raw mut (*buf).b_p_ff as *mut c_void,
        97 => return &raw mut (*buf).b_p_ft as *mut c_void,
        116 => return &raw mut (*buf).b_p_fo as *mut c_void,
        115 => return &raw mut (*buf).b_p_flp as *mut c_void,
        142 => return &raw mut (*buf).b_p_iminsert as *mut c_void,
        143 => return &raw mut (*buf).b_p_imsearch as *mut c_void,
        150 => return &raw mut (*buf).b_p_inf as *mut c_void,
        154 => return &raw mut (*buf).b_p_isk as *mut c_void,
        146 => return &raw mut (*buf).b_p_inex as *mut c_void,
        148 => return &raw mut (*buf).b_p_inde as *mut c_void,
        149 => return &raw mut (*buf).b_p_indk as *mut c_void,
        114 => return &raw mut (*buf).b_p_fex as *mut c_void,
        171 => return &raw mut (*buf).b_p_lisp as *mut c_void,
        172 => return &raw mut (*buf).b_p_lop as *mut c_void,
        191 => return &raw mut (*buf).b_p_ml as *mut c_void,
        181 => return &raw mut (*buf).b_p_mps as *mut c_void,
        194 => return &raw mut (*buf).b_p_ma as *mut c_void,
        195 => return &raw mut (*buf).b_changed as *mut c_void,
        205 => return &raw mut (*buf).b_p_nf as *mut c_void,
        218 => return &raw mut (*buf).b_p_pi as *mut c_void,
        229 => return &raw mut (*buf).b_p_qe as *mut c_void,
        230 => return &raw mut (*buf).b_p_ro as *mut c_void,
        244 => return &raw mut (*buf).b_p_scbk as *mut c_void,
        279 => return &raw mut (*buf).b_p_si as *mut c_void,
        282 => return &raw mut (*buf).b_p_sts as *mut c_void,
        296 => return &raw mut (*buf).b_p_sua as *mut c_void,
        297 => return &raw mut (*buf).b_p_swf as *mut c_void,
        299 => return &raw mut (*buf).b_p_smc as *mut c_void,
        300 => return &raw mut (*buf).b_p_syn as *mut c_void,
        284 => return &raw mut (*(*win).w_s).b_p_spc as *mut c_void,
        285 => return &raw mut (*(*win).w_s).b_p_spf as *mut c_void,
        286 => return &raw mut (*(*win).w_s).b_p_spl as *mut c_void,
        287 => return &raw mut (*(*win).w_s).b_p_spo as *mut c_void,
        266 => return &raw mut (*buf).b_p_sw as *mut c_void,
        307 => return &raw mut (*buf).b_p_tfu as *mut c_void,
        304 => return &raw mut (*buf).b_p_ts as *mut c_void,
        318 => return &raw mut (*buf).b_p_tw as *mut c_void,
        332 => return &raw mut (*buf).b_p_udf as *mut c_void,
        368 => return &raw mut (*buf).b_p_wm as *mut c_void,
        337 => return &raw mut (*buf).b_p_vsts as *mut c_void,
        338 => return &raw mut (*buf).b_p_vts as *mut c_void,
        158 => return &raw mut (*buf).b_p_keymap as *mut c_void,
        277 => return &raw mut (*win).w_onebuf_opt.wo_scl as *mut c_void,
        363 => return &raw mut (*win).w_onebuf_opt.wo_winhl as *mut c_void,
        356 => return &raw mut (*win).w_onebuf_opt.wo_winbl as *mut c_void,
        293 => return &raw mut (*win).w_onebuf_opt.wo_stc as *mut c_void,
        _ => {
            iemsg(gettext(b"E356: get_varp ERROR\0".as_ptr() as *const c_char));
        }
    }
    return &raw mut (*buf).b_p_wm as *mut c_void;
}

#[inline]
pub(crate) unsafe extern "C" fn get_opt_idx(mut opt: *mut vimoption_T) -> OptIndex {
    return opt.offset_from(options.ptr() as *mut vimoption_T) as OptIndex;
}

#[inline]
pub(crate) unsafe extern "C" fn get_varp(mut p: *mut vimoption_T) -> *mut c_void {
    return get_varp_from(p, curbuf.get(), curwin.get());
}
