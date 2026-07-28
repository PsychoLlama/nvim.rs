//! Handing a new window or buffer its own copy of the option values.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn win_copy_options(mut wp_from: *mut win_T, mut wp_to: *mut win_T) {
    copy_winopt(
        &raw mut (*wp_from).w_onebuf_opt,
        &raw mut (*wp_to).w_onebuf_opt,
    );
    copy_winopt(
        &raw mut (*wp_from).w_allbuf_opt,
        &raw mut (*wp_to).w_allbuf_opt,
    );
    didset_window_options(wp_to, true_0 != 0);
}

pub(crate) unsafe extern "C" fn copy_option_val(mut val: *const c_char) -> *mut c_char {
    if val == empty_string_option.ptr() as *mut c_char as *const c_char {
        return empty_string_option.ptr() as *mut c_char;
    }
    return xstrdup(val);
}

pub unsafe extern "C" fn copy_winopt(mut from: *mut winopt_T, mut to: *mut winopt_T) {
    (*to).wo_arab = (*from).wo_arab;
    (*to).wo_list = (*from).wo_list;
    (*to).wo_lcs = copy_option_val((*from).wo_lcs);
    (*to).wo_fcs = copy_option_val((*from).wo_fcs);
    (*to).wo_nu = (*from).wo_nu;
    (*to).wo_rnu = (*from).wo_rnu;
    (*to).wo_ve = copy_option_val((*from).wo_ve);
    (*to).wo_ve_flags = (*from).wo_ve_flags;
    (*to).wo_nuw = (*from).wo_nuw;
    (*to).wo_rl = (*from).wo_rl;
    (*to).wo_rlc = copy_option_val((*from).wo_rlc);
    (*to).wo_sbr = copy_option_val((*from).wo_sbr);
    (*to).wo_stl = copy_option_val((*from).wo_stl);
    (*to).wo_wbr = copy_option_val((*from).wo_wbr);
    (*to).wo_wrap = (*from).wo_wrap;
    (*to).wo_wrap_save = (*from).wo_wrap_save;
    (*to).wo_lbr = (*from).wo_lbr;
    (*to).wo_bri = (*from).wo_bri;
    (*to).wo_briopt = copy_option_val((*from).wo_briopt);
    (*to).wo_scb = (*from).wo_scb;
    (*to).wo_scb_save = (*from).wo_scb_save;
    (*to).wo_sms = (*from).wo_sms;
    (*to).wo_crb = (*from).wo_crb;
    (*to).wo_crb_save = (*from).wo_crb_save;
    (*to).wo_siso = (*from).wo_siso;
    (*to).wo_so = (*from).wo_so;
    (*to).wo_spell = (*from).wo_spell;
    (*to).wo_cuc = (*from).wo_cuc;
    (*to).wo_cul = (*from).wo_cul;
    (*to).wo_culopt = copy_option_val((*from).wo_culopt);
    (*to).wo_cc = copy_option_val((*from).wo_cc);
    (*to).wo_diff = (*from).wo_diff;
    (*to).wo_diff_saved = (*from).wo_diff_saved;
    (*to).wo_eiw = copy_option_val((*from).wo_eiw);
    (*to).wo_cocu = copy_option_val((*from).wo_cocu);
    (*to).wo_cole = (*from).wo_cole;
    (*to).wo_fdc = copy_option_val((*from).wo_fdc);
    (*to).wo_fdc_save = if (*from).wo_diff_saved != 0 {
        xstrdup((*from).wo_fdc_save)
    } else {
        empty_string_option.ptr() as *mut c_char
    };
    (*to).wo_fen = (*from).wo_fen;
    (*to).wo_fen_save = (*from).wo_fen_save;
    (*to).wo_fdi = copy_option_val((*from).wo_fdi);
    (*to).wo_fml = (*from).wo_fml;
    (*to).wo_fdl = (*from).wo_fdl;
    (*to).wo_fdl_save = (*from).wo_fdl_save;
    (*to).wo_fdm = copy_option_val((*from).wo_fdm);
    (*to).wo_fdm_save = if (*from).wo_diff_saved != 0 {
        xstrdup((*from).wo_fdm_save)
    } else {
        empty_string_option.ptr() as *mut c_char
    };
    (*to).wo_fdn = (*from).wo_fdn;
    (*to).wo_fde = copy_option_val((*from).wo_fde);
    (*to).wo_fdt = copy_option_val((*from).wo_fdt);
    (*to).wo_fmr = copy_option_val((*from).wo_fmr);
    (*to).wo_scl = copy_option_val((*from).wo_scl);
    (*to).wo_lhi = (*from).wo_lhi;
    (*to).wo_winhl = copy_option_val((*from).wo_winhl);
    (*to).wo_winbl = (*from).wo_winbl;
    (*to).wo_stc = copy_option_val((*from).wo_stc);
    (*to).wo_wrap_flags = (*from).wo_wrap_flags;
    (*to).wo_stl_flags = (*from).wo_stl_flags;
    (*to).wo_wbr_flags = (*from).wo_wbr_flags;
    (*to).wo_fde_flags = (*from).wo_fde_flags;
    (*to).wo_fdt_flags = (*from).wo_fdt_flags;
    memmove(
        &raw mut (*to).wo_script_ctx as *mut sctx_T as *mut c_void,
        &raw mut (*from).wo_script_ctx as *mut sctx_T as *const c_void,
        ::core::mem::size_of::<[sctx_T; 51]>(),
    );
    check_winopt(to);
}

pub(crate) unsafe extern "C" fn check_win_options(mut win: *mut win_T) {
    check_winopt(&raw mut (*win).w_onebuf_opt);
    check_winopt(&raw mut (*win).w_allbuf_opt);
}

pub(crate) unsafe extern "C" fn check_winopt(mut wop: *mut winopt_T) {
    check_string_option(&raw mut (*wop).wo_fdc);
    check_string_option(&raw mut (*wop).wo_fdc_save);
    check_string_option(&raw mut (*wop).wo_fdi);
    check_string_option(&raw mut (*wop).wo_fdm);
    check_string_option(&raw mut (*wop).wo_fdm_save);
    check_string_option(&raw mut (*wop).wo_fde);
    check_string_option(&raw mut (*wop).wo_fdt);
    check_string_option(&raw mut (*wop).wo_fmr);
    check_string_option(&raw mut (*wop).wo_eiw);
    check_string_option(&raw mut (*wop).wo_scl);
    check_string_option(&raw mut (*wop).wo_rlc);
    check_string_option(&raw mut (*wop).wo_sbr);
    check_string_option(&raw mut (*wop).wo_stl);
    check_string_option(&raw mut (*wop).wo_culopt);
    check_string_option(&raw mut (*wop).wo_cc);
    check_string_option(&raw mut (*wop).wo_cocu);
    check_string_option(&raw mut (*wop).wo_briopt);
    check_string_option(&raw mut (*wop).wo_winhl);
    check_string_option(&raw mut (*wop).wo_lcs);
    check_string_option(&raw mut (*wop).wo_fcs);
    check_string_option(&raw mut (*wop).wo_ve);
    check_string_option(&raw mut (*wop).wo_wbr);
    check_string_option(&raw mut (*wop).wo_stc);
}

pub unsafe extern "C" fn clear_winopt(mut wop: *mut winopt_T) {
    clear_string_option(&raw mut (*wop).wo_fdc);
    clear_string_option(&raw mut (*wop).wo_fdc_save);
    clear_string_option(&raw mut (*wop).wo_fdi);
    clear_string_option(&raw mut (*wop).wo_fdm);
    clear_string_option(&raw mut (*wop).wo_fdm_save);
    clear_string_option(&raw mut (*wop).wo_fde);
    clear_string_option(&raw mut (*wop).wo_fdt);
    clear_string_option(&raw mut (*wop).wo_fmr);
    clear_string_option(&raw mut (*wop).wo_eiw);
    clear_string_option(&raw mut (*wop).wo_scl);
    clear_string_option(&raw mut (*wop).wo_rlc);
    clear_string_option(&raw mut (*wop).wo_sbr);
    clear_string_option(&raw mut (*wop).wo_stl);
    clear_string_option(&raw mut (*wop).wo_culopt);
    clear_string_option(&raw mut (*wop).wo_cc);
    clear_string_option(&raw mut (*wop).wo_cocu);
    clear_string_option(&raw mut (*wop).wo_briopt);
    clear_string_option(&raw mut (*wop).wo_winhl);
    clear_string_option(&raw mut (*wop).wo_lcs);
    clear_string_option(&raw mut (*wop).wo_fcs);
    clear_string_option(&raw mut (*wop).wo_ve);
    clear_string_option(&raw mut (*wop).wo_wbr);
    clear_string_option(&raw mut (*wop).wo_stc);
}

pub unsafe extern "C" fn didset_window_options(mut wp: *mut win_T, mut valid_cursor: bool) {
    if (*wp).w_onebuf_opt.wo_wrap != 0 {
        (*wp).w_leftcol = 0 as c_int as colnr_T;
    } else {
        (*wp).w_skipcol = 0 as c_int as colnr_T;
    }
    check_colorcolumn(::core::ptr::null_mut::<c_char>(), wp);
    briopt_check(::core::ptr::null_mut::<c_char>(), wp);
    fill_culopt_flags(::core::ptr::null_mut::<c_char>(), wp);
    set_chars_option(
        wp,
        (*wp).w_onebuf_opt.wo_fcs,
        kFillchars,
        true_0 != 0,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
    );
    set_chars_option(
        wp,
        (*wp).w_onebuf_opt.wo_lcs,
        kListchars,
        true_0 != 0,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
    );
    parse_winhl_opt(::core::ptr::null::<c_char>(), wp);
    check_blending(wp);
    set_winbar_win(wp, false_0 != 0, valid_cursor);
    check_signcolumn(::core::ptr::null_mut::<c_char>(), wp);
    (*wp).w_grid_alloc.blending = (*wp).w_onebuf_opt.wo_winbl > 0 as OptInt;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buf_copy_options(mut buf: *mut buf_T, mut flags: c_int) {
    let mut should_copy: bool = true_0 != 0;
    let mut save_p_isk: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut did_isk: bool = false_0 != 0;
    if !(*p_cpo.ptr()).is_null() {
        if (vim_strchr(p_cpo.get(), CPO_BUFOPTGLOB).is_null() || flags & BCO_ENTER as c_int == 0)
            && ((*buf).b_p_initialized as c_int != 0
                || flags & BCO_ENTER as c_int == 0
                    && !vim_strchr(p_cpo.get(), CPO_BUFOPT).is_null())
        {
            should_copy = false_0 != 0;
        }
        if should_copy as c_int != 0 || flags & BCO_ALWAYS as c_int != 0 {
            memset(
                &raw mut (*buf).b_p_script_ctx as *mut c_void,
                0 as c_int,
                ::core::mem::size_of::<[sctx_T; 92]>(),
            );
            let mut dont_do_help: bool = flags & BCO_NOHELP as c_int != 0
                && (*buf).b_help as c_int != 0
                || (*buf).b_p_initialized as c_int != 0;
            if dont_do_help {
                save_p_isk = (*buf).b_p_isk;
                (*buf).b_p_isk = ::core::ptr::null_mut::<c_char>();
            }
            if !(*buf).b_p_initialized {
                free_buf_options(buf, true_0 != 0);
                (*buf).b_p_ro = false_0;
                (*buf).b_p_fenc = xstrdup(p_fenc.get());
                match *p_ffs.get() as c_int {
                    109 => {
                        (*buf).b_p_ff = xstrdup(b"mac\0".as_ptr() as *const c_char);
                    }
                    100 => {
                        (*buf).b_p_ff = xstrdup(b"dos\0".as_ptr() as *const c_char);
                    }
                    117 => {
                        (*buf).b_p_ff = xstrdup(b"unix\0".as_ptr() as *const c_char);
                    }
                    _ => {
                        (*buf).b_p_ff = xstrdup(p_ff.get());
                    }
                }
                (*buf).b_p_bh = empty_string_option.ptr() as *mut c_char;
                (*buf).b_p_bt = empty_string_option.ptr() as *mut c_char;
            } else {
                free_buf_options(buf, false_0 != 0);
            }
            (*buf).b_p_ai = p_ai.get();
            (*buf).b_p_script_ctx[kBufOptAutoindent as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptAutoindent as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_ai_nopaste = p_ai_nopaste.get();
            (*buf).b_p_sw = p_sw.get();
            (*buf).b_p_script_ctx[kBufOptShiftwidth as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptShiftwidth as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_scbk = p_scbk.get();
            (*buf).b_p_script_ctx[kBufOptScrollback as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptScrollback as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_tw = p_tw.get();
            (*buf).b_p_script_ctx[kBufOptTextwidth as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptTextwidth as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_tw_nopaste = p_tw_nopaste.get();
            (*buf).b_p_tw_nobin = p_tw_nobin.get();
            (*buf).b_p_wm = p_wm.get();
            (*buf).b_p_script_ctx[kBufOptWrapmargin as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptWrapmargin as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_wm_nopaste = p_wm_nopaste.get();
            (*buf).b_p_wm_nobin = p_wm_nobin.get();
            (*buf).b_p_bin = p_bin.get();
            (*buf).b_p_script_ctx[kBufOptBinary as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptBinary as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_bomb = p_bomb.get();
            (*buf).b_p_script_ctx[kBufOptBomb as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex).offset(kBufOptBomb as c_int as isize)
                    as usize]
                .script_ctx;
            (*buf).b_p_et = p_et.get();
            (*buf).b_p_script_ctx[kBufOptExpandtab as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptExpandtab as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_fixeol = p_fixeol.get();
            (*buf).b_p_script_ctx[kBufOptFixendofline as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptFixendofline as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_et_nobin = p_et_nobin.get();
            (*buf).b_p_et_nopaste = p_et_nopaste.get();
            (*buf).b_p_ml = p_ml.get();
            (*buf).b_p_script_ctx[kBufOptModeline as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptModeline as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_ml_nobin = p_ml_nobin.get();
            (*buf).b_p_inf = p_inf.get();
            (*buf).b_p_script_ctx[kBufOptInfercase as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptInfercase as c_int as isize) as usize]
                .script_ctx;
            if (*cmdmod.ptr()).cmod_flags & CMOD_NOSWAPFILE as c_int != 0 {
                (*buf).b_p_swf = false_0;
            } else {
                (*buf).b_p_swf = p_swf.get();
                (*buf).b_p_script_ctx[kBufOptSwapfile as c_int as usize] = (*options.ptr())
                    [*(&raw const buf_opt_idx as *const OptIndex)
                        .offset(kBufOptSwapfile as c_int as isize) as usize]
                    .script_ctx;
            }
            (*buf).b_p_cpt = xstrdup(p_cpt.get());
            (*buf).b_p_script_ctx[kBufOptComplete as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptComplete as c_int as isize) as usize]
                .script_ctx;
            set_buflocal_cpt_callbacks(buf);
            (*buf).b_p_cfu = xstrdup(p_cfu.get());
            (*buf).b_p_script_ctx[kBufOptCompletefunc as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCompletefunc as c_int as isize) as usize]
                .script_ctx;
            set_buflocal_cfu_callback(buf);
            (*buf).b_p_ofu = xstrdup(p_ofu.get());
            (*buf).b_p_script_ctx[kBufOptOmnifunc as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptOmnifunc as c_int as isize) as usize]
                .script_ctx;
            set_buflocal_ofu_callback(buf);
            (*buf).b_p_tfu = xstrdup(p_tfu.get());
            (*buf).b_p_script_ctx[kBufOptTagfunc as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptTagfunc as c_int as isize) as usize]
                .script_ctx;
            set_buflocal_tfu_callback(buf);
            (*buf).b_p_sts = p_sts.get();
            (*buf).b_p_script_ctx[kBufOptSofttabstop as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSofttabstop as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_sts_nopaste = p_sts_nopaste.get();
            (*buf).b_p_vsts = xstrdup(p_vsts.get());
            (*buf).b_p_script_ctx[kBufOptVarsofttabstop as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptVarsofttabstop as c_int as isize) as usize]
                .script_ctx;
            if !(*p_vsts.ptr()).is_null()
                && p_vsts.get() != empty_string_option.ptr() as *mut c_char
            {
                tabstop_set(p_vsts.get(), &raw mut (*buf).b_p_vsts_array);
            } else {
                (*buf).b_p_vsts_array = ::core::ptr::null_mut::<colnr_T>();
            }
            (*buf).b_p_vsts_nopaste = if !(*p_vsts_nopaste.ptr()).is_null() {
                xstrdup(p_vsts_nopaste.get())
            } else {
                ::core::ptr::null_mut::<c_char>()
            };
            (*buf).b_p_com = xstrdup(p_com.get());
            (*buf).b_p_script_ctx[kBufOptComments as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptComments as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cms = xstrdup(p_cms.get());
            (*buf).b_p_script_ctx[kBufOptCommentstring as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCommentstring as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_fo = xstrdup(p_fo.get());
            (*buf).b_p_script_ctx[kBufOptFormatoptions as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptFormatoptions as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_flp = xstrdup(p_flp.get());
            (*buf).b_p_script_ctx[kBufOptFormatlistpat as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptFormatlistpat as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_nf = xstrdup(p_nf.get());
            (*buf).b_p_script_ctx[kBufOptNrformats as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptNrformats as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_mps = xstrdup(p_mps.get());
            (*buf).b_p_script_ctx[kBufOptMatchpairs as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptMatchpairs as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_si = p_si.get();
            (*buf).b_p_script_ctx[kBufOptSmartindent as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSmartindent as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_channel = 0 as OptInt;
            (*buf).b_p_ci = p_ci.get();
            (*buf).b_p_script_ctx[kBufOptCopyindent as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCopyindent as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cin = p_cin.get();
            (*buf).b_p_script_ctx[kBufOptCindent as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCindent as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cink = xstrdup(p_cink.get());
            (*buf).b_p_script_ctx[kBufOptCinkeys as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCinkeys as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cino = xstrdup(p_cino.get());
            (*buf).b_p_script_ctx[kBufOptCinoptions as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCinoptions as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cinsd = xstrdup(p_cinsd.get());
            (*buf).b_p_script_ctx[kBufOptCinscopedecls as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCinscopedecls as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_lop = xstrdup(p_lop.get());
            (*buf).b_p_script_ctx[kBufOptLispoptions as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptLispoptions as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_ft = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_pi = p_pi.get();
            (*buf).b_p_script_ctx[kBufOptPreserveindent as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptPreserveindent as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cinw = xstrdup(p_cinw.get());
            (*buf).b_p_script_ctx[kBufOptCinwords as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCinwords as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_lisp = p_lisp.get();
            (*buf).b_p_script_ctx[kBufOptLisp as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex).offset(kBufOptLisp as c_int as isize)
                    as usize]
                .script_ctx;
            (*buf).b_p_syn = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_smc = p_smc.get();
            (*buf).b_p_script_ctx[kBufOptSynmaxcol as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSynmaxcol as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_s.b_syn_isk = empty_string_option.ptr() as *mut c_char;
            (*buf).b_s.b_p_spc = xstrdup(p_spc.get());
            (*buf).b_p_script_ctx[kBufOptSpellcapcheck as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSpellcapcheck as c_int as isize) as usize]
                .script_ctx;
            compile_cap_prog(&raw mut (*buf).b_s);
            (*buf).b_s.b_p_spf = xstrdup(p_spf.get());
            (*buf).b_p_script_ctx[kBufOptSpellfile as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSpellfile as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_s.b_p_spl = xstrdup(p_spl.get());
            (*buf).b_p_script_ctx[kBufOptSpelllang as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSpelllang as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_s.b_p_spo = xstrdup(p_spo.get());
            (*buf).b_p_script_ctx[kBufOptSpelloptions as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSpelloptions as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_s.b_p_spo_flags = spo_flags.get();
            (*buf).b_p_inde = xstrdup(p_inde.get());
            (*buf).b_p_script_ctx[kBufOptIndentexpr as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptIndentexpr as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_indk = xstrdup(p_indk.get());
            (*buf).b_p_script_ctx[kBufOptIndentkeys as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptIndentkeys as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_fp = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_fex = xstrdup(p_fex.get());
            (*buf).b_p_script_ctx[kBufOptFormatexpr as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptFormatexpr as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_sua = xstrdup(p_sua.get());
            (*buf).b_p_script_ctx[kBufOptSuffixesadd as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSuffixesadd as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_keymap = xstrdup(p_keymap.get());
            (*buf).b_p_script_ctx[kBufOptKeymap as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptKeymap as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_kmap_state = ((*buf).b_kmap_state as c_int | KEYMAP_INIT) as int16_t;
            (*buf).b_p_iminsert = p_iminsert.get();
            (*buf).b_p_script_ctx[kBufOptIminsert as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptIminsert as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_imsearch = p_imsearch.get();
            (*buf).b_p_script_ctx[kBufOptImsearch as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptImsearch as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_ac = -1 as c_int;
            (*buf).b_p_ar = -1 as c_int;
            (*buf).b_p_fs = -1 as c_int;
            (*buf).b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;
            (*buf).b_p_bkc = empty_string_option.ptr() as *mut c_char;
            (*buf).b_bkc_flags = 0 as c_uint;
            (*buf).b_p_gefm = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_gp = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_mp = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_efm = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_ep = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_ffu = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_kp = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_path = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_tags = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_tc = empty_string_option.ptr() as *mut c_char;
            (*buf).b_tc_flags = 0 as c_uint;
            (*buf).b_p_def = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_inc = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_inex = xstrdup(p_inex.get());
            (*buf).b_p_script_ctx[kBufOptIncludeexpr as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptIncludeexpr as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cot = empty_string_option.ptr() as *mut c_char;
            (*buf).b_cot_flags = 0 as c_uint;
            (*buf).b_p_dict = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_dia = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_tsr = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_tsrfu = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_qe = xstrdup(p_qe.get());
            (*buf).b_p_script_ctx[kBufOptQuoteescape as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptQuoteescape as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_udf = p_udf.get();
            (*buf).b_p_script_ctx[kBufOptUndofile as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptUndofile as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_lw = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_menc = empty_string_option.ptr() as *mut c_char;
            if dont_do_help {
                (*buf).b_p_isk = save_p_isk;
                if !(*p_vts.ptr()).is_null()
                    && *p_vts.get() as c_int != NUL
                    && (*buf).b_p_vts_array.is_null()
                {
                    tabstop_set(p_vts.get(), &raw mut (*buf).b_p_vts_array);
                } else {
                    (*buf).b_p_vts_array = ::core::ptr::null_mut::<colnr_T>();
                }
            } else {
                (*buf).b_p_isk = xstrdup(p_isk.get());
                (*buf).b_p_script_ctx[kBufOptIskeyword as c_int as usize] = (*options.ptr())
                    [*(&raw const buf_opt_idx as *const OptIndex)
                        .offset(kBufOptIskeyword as c_int as isize) as usize]
                    .script_ctx;
                did_isk = true_0 != 0;
                (*buf).b_p_ts = p_ts.get();
                (*buf).b_p_script_ctx[kBufOptTabstop as c_int as usize] = (*options.ptr())
                    [*(&raw const buf_opt_idx as *const OptIndex)
                        .offset(kBufOptTabstop as c_int as isize) as usize]
                    .script_ctx;
                (*buf).b_p_vts = xstrdup(p_vts.get());
                (*buf).b_p_script_ctx[kBufOptVartabstop as c_int as usize] = (*options.ptr())
                    [*(&raw const buf_opt_idx as *const OptIndex)
                        .offset(kBufOptVartabstop as c_int as isize) as usize]
                    .script_ctx;
                if !(*p_vts.ptr()).is_null()
                    && *p_vts.get() as c_int != NUL
                    && (*buf).b_p_vts_array.is_null()
                {
                    tabstop_set(p_vts.get(), &raw mut (*buf).b_p_vts_array);
                } else {
                    (*buf).b_p_vts_array = ::core::ptr::null_mut::<colnr_T>();
                }
                (*buf).b_help = false_0 != 0;
                if *(*buf).b_p_bt.offset(0 as c_int as isize) as c_int == 'h' as c_int {
                    clear_string_option(&raw mut (*buf).b_p_bt);
                }
                (*buf).b_p_ma = p_ma.get();
                (*buf).b_p_script_ctx[kBufOptModifiable as c_int as usize] = (*options.ptr())
                    [*(&raw const buf_opt_idx as *const OptIndex)
                        .offset(kBufOptModifiable as c_int as isize) as usize]
                    .script_ctx;
            }
        }
        if should_copy {
            (*buf).b_p_initialized = true_0 != 0;
        }
    }
    check_buf_options(buf);
    if did_isk {
        buf_init_chartab(buf, false);
    }
}

pub unsafe extern "C" fn reset_modifiable() {
    (*curbuf.get()).b_p_ma = false_0;
    p_ma.set(false_0);
    change_option_default(
        kOptModifiable,
        OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData { boolean: kFalse },
        },
    );
}

pub unsafe extern "C" fn set_iminsert_global(mut buf: *mut buf_T) {
    p_iminsert.set((*buf).b_p_iminsert);
}

pub unsafe extern "C" fn set_imsearch_global(mut buf: *mut buf_T) {
    p_imsearch.set((*buf).b_p_imsearch);
}
