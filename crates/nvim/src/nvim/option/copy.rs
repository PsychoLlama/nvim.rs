//! Handing a new window or buffer its own copy of the option values.
//!
//! A window's values are copied from the window it was split from; a
//! buffer's are copied from the global values. Both are field-by-field
//! rather than a struct assignment, because a string field has to be
//! duplicated and a few fields are deliberately *not* copied.
//!
//! The shared empty string every unset string option points at is never
//! duplicated — [`copy_option_val`] hands the same pointer back — so that
//! `free_string_option` can go on telling an owned value from an unset one
//! by its address.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

use crate::src::nvim::buffer::free_buf_options;
use crate::src::nvim::charset::buf_init_chartab;
use crate::src::nvim::indent::{briopt_check, tabstop_set};
use crate::src::nvim::insexpand::{
    set_buflocal_cfu_callback, set_buflocal_cpt_callbacks, set_buflocal_ofu_callback,
};
use crate::src::nvim::main::{
    cmdmod, curbuf, empty_string_option, p_ai, p_bin, p_bomb, p_cfu, p_ci, p_cin, p_cink, p_cino,
    p_cinsd, p_cinw, p_cms, p_com, p_cpo, p_cpt, p_et, p_fenc, p_fex, p_ff, p_ffs, p_fixeol, p_flp,
    p_fo, p_iminsert, p_imsearch, p_inde, p_indk, p_inex, p_inf, p_isk, p_keymap, p_lisp, p_lop,
    p_ma, p_ml, p_mps, p_nf, p_ofu, p_pi, p_qe, p_scbk, p_si, p_smc, p_spc, p_spf, p_spl, p_spo,
    p_sts, p_sua, p_sw, p_swf, p_tfu, p_ts, p_tw, p_udf, p_vsts, p_vts, p_wm, spo_flags,
};
use crate::src::nvim::memory::xstrdup;

use super::check::{p_et_nobin, p_ml_nobin, p_tw_nobin, p_wm_nobin};
use super::paste::{
    p_ai_nopaste, p_et_nopaste, p_sts_nopaste, p_tw_nopaste, p_vsts_nopaste, p_wm_nopaste,
};
use crate::src::nvim::options::{
    BufOptIndex, buf_opt_idx, kBufOptAutoindent, kBufOptBinary, kBufOptBomb, kBufOptCindent,
    kBufOptCinkeys, kBufOptCinoptions, kBufOptCinscopedecls, kBufOptCinwords, kBufOptComments,
    kBufOptCommentstring, kBufOptComplete, kBufOptCompletefunc, kBufOptCopyindent,
    kBufOptExpandtab, kBufOptFixendofline, kBufOptFormatexpr, kBufOptFormatlistpat,
    kBufOptFormatoptions, kBufOptIminsert, kBufOptImsearch, kBufOptIncludeexpr, kBufOptIndentexpr,
    kBufOptIndentkeys, kBufOptInfercase, kBufOptIskeyword, kBufOptKeymap, kBufOptLisp,
    kBufOptLispoptions, kBufOptMatchpairs, kBufOptModeline, kBufOptModifiable, kBufOptNrformats,
    kBufOptOmnifunc, kBufOptPreserveindent, kBufOptQuoteescape, kBufOptScrollback,
    kBufOptShiftwidth, kBufOptSmartindent, kBufOptSofttabstop, kBufOptSpellcapcheck,
    kBufOptSpellfile, kBufOptSpelllang, kBufOptSpelloptions, kBufOptSuffixesadd, kBufOptSwapfile,
    kBufOptSynmaxcol, kBufOptTabstop, kBufOptTagfunc, kBufOptTextwidth, kBufOptUndofile,
    kBufOptVarsofttabstop, kBufOptVartabstop, kBufOptWrapmargin, kOptModifiable, options,
};
use crate::src::nvim::optionstr::{
    check_buf_options, check_signcolumn, check_string_option, clear_string_option,
};
use crate::src::nvim::spell::compile_cap_prog;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::tag::set_buflocal_tfu_callback;
use crate::src::nvim::types::{
    OptInt, OptVal, OptValData, buf_T, colnr_T, int16_t, kFalse, win_T, winopt_T,
};
use crate::src::nvim::window::{check_colorcolumn, set_winbar_win};

use super::{
    BCO_ALWAYS, BCO_ENTER, BCO_NOHELP, CMOD_NOSWAPFILE, KEYMAP_INIT, NO_LOCAL_UNDOLEVEL, NUL,
    change_option_default, check_blending, fill_culopt_flags, kFillchars, kListchars,
    kOptValTypeBoolean, parse_winhl_opt, set_chars_option,
};

/// The two 'cpo' flags that decide when a buffer's options are copied:
/// 's' copies them the first time the buffer is entered, 'S' every time.
const CPO_BUFOPT: c_int = 's' as c_int;
const CPO_BUFOPTGLOB: c_int = 'S' as c_int;

/// The string every unset string option shares.
fn unset_string() -> *mut c_char {
    empty_string_option.ptr().cast::<c_char>()
}

/// Give a window's option values to a freshly split one.
///
/// # Safety
///
/// Both windows must be live, and `wp_to`'s option fields uninitialised or
/// already released.
pub unsafe fn win_copy_options(wp_from: *mut win_T, wp_to: *mut win_T) {
    // SAFETY: the caller's windows.
    unsafe {
        copy_winopt(
            &raw mut (*wp_from).w_onebuf_opt,
            &raw mut (*wp_to).w_onebuf_opt,
        );
        copy_winopt(
            &raw mut (*wp_from).w_allbuf_opt,
            &raw mut (*wp_to).w_allbuf_opt,
        );
        didset_window_options(wp_to, true);
    }
}

/// A copy of a string option's value, sharing the unset string rather than
/// duplicating it.
///
/// # Safety
///
/// `val` must be a string option's value.
pub(crate) unsafe fn copy_option_val(val: *const c_char) -> *mut c_char {
    if val == unset_string() {
        return unset_string();
    }
    // SAFETY: the caller's value is a NUL-terminated option value.
    unsafe { xstrdup(val) }
}

/// Copy one window's worth of option values.
///
/// # Safety
///
/// Both must point at `winopt_T`s, and `to`'s fields uninitialised or
/// already released.
pub unsafe fn copy_winopt(from: *mut winopt_T, to: *mut winopt_T) {
    // SAFETY: the caller's structures.
    unsafe {
        let f = &*from;
        let t = &mut *to;

        t.wo_arab = f.wo_arab;
        t.wo_list = f.wo_list;
        t.wo_lcs = copy_option_val(f.wo_lcs);
        t.wo_fcs = copy_option_val(f.wo_fcs);
        t.wo_nu = f.wo_nu;
        t.wo_rnu = f.wo_rnu;
        t.wo_ve = copy_option_val(f.wo_ve);
        t.wo_ve_flags = f.wo_ve_flags;
        t.wo_nuw = f.wo_nuw;
        t.wo_rl = f.wo_rl;
        t.wo_rlc = copy_option_val(f.wo_rlc);
        t.wo_sbr = copy_option_val(f.wo_sbr);
        t.wo_stl = copy_option_val(f.wo_stl);
        t.wo_wbr = copy_option_val(f.wo_wbr);
        t.wo_wrap = f.wo_wrap;
        t.wo_wrap_save = f.wo_wrap_save;
        t.wo_lbr = f.wo_lbr;
        t.wo_bri = f.wo_bri;
        t.wo_briopt = copy_option_val(f.wo_briopt);
        t.wo_scb = f.wo_scb;
        t.wo_scb_save = f.wo_scb_save;
        t.wo_sms = f.wo_sms;
        t.wo_crb = f.wo_crb;
        t.wo_crb_save = f.wo_crb_save;
        t.wo_siso = f.wo_siso;
        t.wo_so = f.wo_so;
        t.wo_spell = f.wo_spell;
        t.wo_cuc = f.wo_cuc;
        t.wo_cul = f.wo_cul;
        t.wo_culopt = copy_option_val(f.wo_culopt);
        t.wo_cc = copy_option_val(f.wo_cc);
        t.wo_diff = f.wo_diff;
        t.wo_diff_saved = f.wo_diff_saved;
        t.wo_eiw = copy_option_val(f.wo_eiw);
        t.wo_cocu = copy_option_val(f.wo_cocu);
        t.wo_cole = f.wo_cole;
        t.wo_fdc = copy_option_val(f.wo_fdc);
        // The four `_save` copies only hold anything while `:diffthis` is
        // in effect; otherwise they are the unset string, not a value to
        // duplicate.
        t.wo_fdc_save = if f.wo_diff_saved != 0 {
            xstrdup(f.wo_fdc_save)
        } else {
            unset_string()
        };
        t.wo_fen = f.wo_fen;
        t.wo_fen_save = f.wo_fen_save;
        t.wo_fdi = copy_option_val(f.wo_fdi);
        t.wo_fml = f.wo_fml;
        t.wo_fdl = f.wo_fdl;
        t.wo_fdl_save = f.wo_fdl_save;
        t.wo_fdm = copy_option_val(f.wo_fdm);
        t.wo_fdm_save = if f.wo_diff_saved != 0 {
            xstrdup(f.wo_fdm_save)
        } else {
            unset_string()
        };
        t.wo_fdn = f.wo_fdn;
        t.wo_fde = copy_option_val(f.wo_fde);
        t.wo_fdt = copy_option_val(f.wo_fdt);
        t.wo_fmr = copy_option_val(f.wo_fmr);
        t.wo_scl = copy_option_val(f.wo_scl);
        t.wo_lhi = f.wo_lhi;
        t.wo_winhl = copy_option_val(f.wo_winhl);
        t.wo_winbl = f.wo_winbl;
        t.wo_stc = copy_option_val(f.wo_stc);
        t.wo_wrap_flags = f.wo_wrap_flags;
        t.wo_stl_flags = f.wo_stl_flags;
        t.wo_wbr_flags = f.wo_wbr_flags;
        t.wo_fde_flags = f.wo_fde_flags;
        t.wo_fdt_flags = f.wo_fdt_flags;
        t.wo_script_ctx = f.wo_script_ctx;

        check_winopt(to);
    }
}

/// The window-local string options, as the address of each field.
///
/// # Safety
///
/// `wop` must point at a `winopt_T`.
unsafe fn winopt_strings(wop: *mut winopt_T) -> [*mut *mut c_char; 23] {
    // SAFETY: the caller's structure.
    unsafe {
        [
            &raw mut (*wop).wo_fdc,
            &raw mut (*wop).wo_fdc_save,
            &raw mut (*wop).wo_fdi,
            &raw mut (*wop).wo_fdm,
            &raw mut (*wop).wo_fdm_save,
            &raw mut (*wop).wo_fde,
            &raw mut (*wop).wo_fdt,
            &raw mut (*wop).wo_fmr,
            &raw mut (*wop).wo_eiw,
            &raw mut (*wop).wo_scl,
            &raw mut (*wop).wo_rlc,
            &raw mut (*wop).wo_sbr,
            &raw mut (*wop).wo_stl,
            &raw mut (*wop).wo_culopt,
            &raw mut (*wop).wo_cc,
            &raw mut (*wop).wo_cocu,
            &raw mut (*wop).wo_briopt,
            &raw mut (*wop).wo_winhl,
            &raw mut (*wop).wo_lcs,
            &raw mut (*wop).wo_fcs,
            &raw mut (*wop).wo_ve,
            &raw mut (*wop).wo_wbr,
            &raw mut (*wop).wo_stc,
        ]
    }
}

/// Give a window's two option sets any string values they are still missing.
///
/// # Safety
///
/// `win` must be a live window.
pub(crate) unsafe fn check_win_options(win: *mut win_T) {
    // SAFETY: the caller's window.
    unsafe {
        check_winopt(&raw mut (*win).w_onebuf_opt);
        check_winopt(&raw mut (*win).w_allbuf_opt);
    }
}

/// Replace any null string value with the shared unset string.
///
/// # Safety
///
/// `wop` must point at a `winopt_T`.
pub(crate) unsafe fn check_winopt(wop: *mut winopt_T) {
    // SAFETY: the caller's structure, and each address is one of its own
    // string fields.
    unsafe {
        for field in winopt_strings(wop) {
            check_string_option(field);
        }
    }
}

/// Release every string value a window's option set owns.
///
/// # Safety
///
/// `wop` must point at a `winopt_T` whose string values are its own.
pub unsafe fn clear_winopt(wop: *mut winopt_T) {
    // SAFETY: the caller's structure, and each address is one of its own
    // string fields.
    unsafe {
        for field in winopt_strings(wop) {
            clear_string_option(field);
        }
    }
}

/// Rebuild everything a window derives from its option values, after they
/// were copied or replaced wholesale.
///
/// `valid_cursor` says whether the window's cursor position can be trusted;
/// a window being created does not have one yet.
///
/// # Safety
///
/// `wp` must be a live window.
pub unsafe fn didset_window_options(wp: *mut win_T, valid_cursor: bool) {
    // SAFETY: the caller's window.
    unsafe {
        // 'wrap' and 'smoothscroll' scroll in different directions, and only
        // one of the two offsets can be non-zero.
        if (*wp).w_onebuf_opt.wo_wrap != 0 {
            (*wp).w_leftcol = 0 as colnr_T;
        } else {
            (*wp).w_skipcol = 0 as colnr_T;
        }
        check_colorcolumn(ptr::null_mut(), wp);
        briopt_check(ptr::null_mut(), wp);
        fill_culopt_flags(ptr::null_mut(), wp);
        set_chars_option(
            wp,
            (*wp).w_onebuf_opt.wo_fcs,
            kFillchars,
            true,
            ptr::null_mut(),
            0,
        );
        set_chars_option(
            wp,
            (*wp).w_onebuf_opt.wo_lcs,
            kListchars,
            true,
            ptr::null_mut(),
            0,
        );
        parse_winhl_opt(ptr::null(), wp);
        check_blending(wp);
        set_winbar_win(wp, false, valid_cursor);
        check_signcolumn(ptr::null_mut(), wp);
        (*wp).w_grid_alloc.blending = (*wp).w_onebuf_opt.wo_winbl > 0 as OptInt;
    }
}

/// Attribute a buffer-local option to whatever script set the global value
/// it was just copied from.
///
/// # Safety
///
/// `buf` must be a live buffer.
unsafe fn copy_sctx(buf: *mut buf_T, bv: BufOptIndex) {
    // SAFETY: the caller's buffer, and `buf_opt_idx` maps every buffer-local
    // row to a row of the option table.
    unsafe {
        let opt_idx = (*buf_opt_idx.ptr())[bv as usize];
        (*buf).b_p_script_ctx[bv as usize] = (*options.ptr())[opt_idx as usize].script_ctx;
    }
}

/// Copy the global option values into one buffer's local ones.
///
/// `flags` is `BCO_ENTER` when the buffer is about to be entered,
/// `BCO_ALWAYS` to copy regardless, `BCO_NOHELP` to leave a help buffer's
/// own settings alone.
///
/// Whether the copy happens at all is 'cpo' 's' and 'S':
///
/// | 'S' | `BCO_ENTER` | initialized | 's' | copy |
/// | --- | --- | --- | --- | --- |
/// | yes | yes | — | — | yes |
/// | yes | no | yes | — | no |
/// | no | — | yes | — | no |
/// | — | no | no | yes | no |
/// | — | no | no | no | yes |
/// | no | yes | no | — | yes |
///
/// # Safety
///
/// `buf` must be a live buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn buf_copy_options(buf: *mut buf_T, flags: c_int) {
    let mut did_isk = false;

    // SAFETY: the caller's buffer, and every global read here is an option
    // variable.
    unsafe {
        // Before the defaults exist there is nothing to copy: `main` makes
        // the first buffer that early.
        if p_cpo.get().is_null() {
            check_buf_options(buf);
            return;
        }

        let entering = flags & BCO_ENTER as c_int != 0;
        let keep_global = vim_strchr(p_cpo.get(), CPO_BUFOPTGLOB).is_null() || !entering;
        let keep_local =
            (*buf).b_p_initialized || (!entering && !vim_strchr(p_cpo.get(), CPO_BUFOPT).is_null());
        let should_copy = !(keep_global && keep_local);

        if should_copy || flags & BCO_ALWAYS as c_int != 0 {
            (*buf).b_p_script_ctx = core::mem::zeroed();

            // A help buffer keeps its own settings when it already has them
            // — jumping back to one with CTRL-T or CTRL-O must not reset it.
            let dont_do_help =
                (flags & BCO_NOHELP as c_int != 0 && (*buf).b_help) || (*buf).b_p_initialized;
            // 'iskeyword' is the one string that survives the free below.
            let save_p_isk = if dont_do_help {
                let saved = (*buf).b_p_isk;
                (*buf).b_p_isk = ptr::null_mut();
                saved
            } else {
                ptr::null_mut()
            };

            if (*buf).b_p_initialized {
                free_buf_options(buf, false);
            } else {
                free_buf_options(buf, true);
                (*buf).b_p_ro = 0;
                (*buf).b_p_fenc = xstrdup(p_fenc.get());
                // A new buffer takes the *first* of 'fileformats' rather
                // than 'fileformat', since nothing has been read yet.
                (*buf).b_p_ff = match *p_ffs.get() as u8 {
                    b'm' => xstrdup(c"mac".as_ptr()),
                    b'd' => xstrdup(c"dos".as_ptr()),
                    b'u' => xstrdup(c"unix".as_ptr()),
                    _ => xstrdup(p_ff.get()),
                };
                (*buf).b_p_bh = unset_string();
                (*buf).b_p_bt = unset_string();
            }

            (*buf).b_p_ai = p_ai.get();
            copy_sctx(buf, kBufOptAutoindent);
            (*buf).b_p_ai_nopaste = p_ai_nopaste.get();
            (*buf).b_p_sw = p_sw.get();
            copy_sctx(buf, kBufOptShiftwidth);
            (*buf).b_p_scbk = p_scbk.get();
            copy_sctx(buf, kBufOptScrollback);
            (*buf).b_p_tw = p_tw.get();
            copy_sctx(buf, kBufOptTextwidth);
            (*buf).b_p_tw_nopaste = p_tw_nopaste.get();
            (*buf).b_p_tw_nobin = p_tw_nobin.get();
            (*buf).b_p_wm = p_wm.get();
            copy_sctx(buf, kBufOptWrapmargin);
            (*buf).b_p_wm_nopaste = p_wm_nopaste.get();
            (*buf).b_p_wm_nobin = p_wm_nobin.get();
            (*buf).b_p_bin = p_bin.get();
            copy_sctx(buf, kBufOptBinary);
            (*buf).b_p_bomb = p_bomb.get();
            copy_sctx(buf, kBufOptBomb);
            (*buf).b_p_et = p_et.get();
            copy_sctx(buf, kBufOptExpandtab);
            (*buf).b_p_fixeol = p_fixeol.get();
            copy_sctx(buf, kBufOptFixendofline);
            (*buf).b_p_et_nobin = p_et_nobin.get();
            (*buf).b_p_et_nopaste = p_et_nopaste.get();
            (*buf).b_p_ml = p_ml.get();
            copy_sctx(buf, kBufOptModeline);
            (*buf).b_p_ml_nobin = p_ml_nobin.get();
            (*buf).b_p_inf = p_inf.get();
            copy_sctx(buf, kBufOptInfercase);

            // `:noswapfile` wins over the global 'swapfile', and leaves the
            // script context alone because nothing set it.
            if (*cmdmod.ptr()).cmod_flags & CMOD_NOSWAPFILE as c_int != 0 {
                (*buf).b_p_swf = 0;
            } else {
                (*buf).b_p_swf = p_swf.get();
                copy_sctx(buf, kBufOptSwapfile);
            }

            (*buf).b_p_cpt = xstrdup(p_cpt.get());
            copy_sctx(buf, kBufOptComplete);
            set_buflocal_cpt_callbacks(buf);
            (*buf).b_p_cfu = xstrdup(p_cfu.get());
            copy_sctx(buf, kBufOptCompletefunc);
            set_buflocal_cfu_callback(buf);
            (*buf).b_p_ofu = xstrdup(p_ofu.get());
            copy_sctx(buf, kBufOptOmnifunc);
            set_buflocal_ofu_callback(buf);
            (*buf).b_p_tfu = xstrdup(p_tfu.get());
            copy_sctx(buf, kBufOptTagfunc);
            set_buflocal_tfu_callback(buf);

            (*buf).b_p_sts = p_sts.get();
            copy_sctx(buf, kBufOptSofttabstop);
            (*buf).b_p_sts_nopaste = p_sts_nopaste.get();
            (*buf).b_p_vsts = xstrdup(p_vsts.get());
            copy_sctx(buf, kBufOptVarsofttabstop);
            (*buf).b_p_vsts_array = if !p_vsts.get().is_null() && p_vsts.get() != unset_string() {
                tabstop_array(p_vsts.get())
            } else {
                ptr::null_mut()
            };
            (*buf).b_p_vsts_nopaste = if p_vsts_nopaste.get().is_null() {
                ptr::null_mut()
            } else {
                xstrdup(p_vsts_nopaste.get())
            };

            (*buf).b_p_com = xstrdup(p_com.get());
            copy_sctx(buf, kBufOptComments);
            (*buf).b_p_cms = xstrdup(p_cms.get());
            copy_sctx(buf, kBufOptCommentstring);
            (*buf).b_p_fo = xstrdup(p_fo.get());
            copy_sctx(buf, kBufOptFormatoptions);
            (*buf).b_p_flp = xstrdup(p_flp.get());
            copy_sctx(buf, kBufOptFormatlistpat);
            (*buf).b_p_nf = xstrdup(p_nf.get());
            copy_sctx(buf, kBufOptNrformats);
            (*buf).b_p_mps = xstrdup(p_mps.get());
            copy_sctx(buf, kBufOptMatchpairs);
            (*buf).b_p_si = p_si.get();
            copy_sctx(buf, kBufOptSmartindent);
            (*buf).b_p_channel = 0 as OptInt;
            (*buf).b_p_ci = p_ci.get();
            copy_sctx(buf, kBufOptCopyindent);
            (*buf).b_p_cin = p_cin.get();
            copy_sctx(buf, kBufOptCindent);
            (*buf).b_p_cink = xstrdup(p_cink.get());
            copy_sctx(buf, kBufOptCinkeys);
            (*buf).b_p_cino = xstrdup(p_cino.get());
            copy_sctx(buf, kBufOptCinoptions);
            (*buf).b_p_cinsd = xstrdup(p_cinsd.get());
            copy_sctx(buf, kBufOptCinscopedecls);
            (*buf).b_p_lop = xstrdup(p_lop.get());
            copy_sctx(buf, kBufOptLispoptions);
            // 'filetype' and 'syntax' start empty: the autocommands that
            // set them have not run for this buffer yet.
            (*buf).b_p_ft = unset_string();
            (*buf).b_p_pi = p_pi.get();
            copy_sctx(buf, kBufOptPreserveindent);
            (*buf).b_p_cinw = xstrdup(p_cinw.get());
            copy_sctx(buf, kBufOptCinwords);
            (*buf).b_p_lisp = p_lisp.get();
            copy_sctx(buf, kBufOptLisp);
            (*buf).b_p_syn = unset_string();
            (*buf).b_p_smc = p_smc.get();
            copy_sctx(buf, kBufOptSynmaxcol);

            (*buf).b_s.b_syn_isk = unset_string();
            (*buf).b_s.b_p_spc = xstrdup(p_spc.get());
            copy_sctx(buf, kBufOptSpellcapcheck);
            compile_cap_prog(&raw mut (*buf).b_s);
            (*buf).b_s.b_p_spf = xstrdup(p_spf.get());
            copy_sctx(buf, kBufOptSpellfile);
            (*buf).b_s.b_p_spl = xstrdup(p_spl.get());
            copy_sctx(buf, kBufOptSpelllang);
            (*buf).b_s.b_p_spo = xstrdup(p_spo.get());
            copy_sctx(buf, kBufOptSpelloptions);
            (*buf).b_s.b_p_spo_flags = spo_flags.get();

            (*buf).b_p_inde = xstrdup(p_inde.get());
            copy_sctx(buf, kBufOptIndentexpr);
            (*buf).b_p_indk = xstrdup(p_indk.get());
            copy_sctx(buf, kBufOptIndentkeys);
            (*buf).b_p_fp = unset_string();
            (*buf).b_p_fex = xstrdup(p_fex.get());
            copy_sctx(buf, kBufOptFormatexpr);
            (*buf).b_p_sua = xstrdup(p_sua.get());
            copy_sctx(buf, kBufOptSuffixesadd);
            (*buf).b_p_keymap = xstrdup(p_keymap.get());
            copy_sctx(buf, kBufOptKeymap);
            (*buf).b_kmap_state = ((*buf).b_kmap_state as c_int | KEYMAP_INIT) as int16_t;
            (*buf).b_p_iminsert = p_iminsert.get();
            copy_sctx(buf, kBufOptIminsert);
            (*buf).b_p_imsearch = p_imsearch.get();
            copy_sctx(buf, kBufOptImsearch);

            // The global-local options start unset, reading through to the
            // global value.
            (*buf).b_p_ac = -1;
            (*buf).b_p_ar = -1;
            (*buf).b_p_fs = -1;
            (*buf).b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;
            for field in [
                &raw mut (*buf).b_p_bkc,
                &raw mut (*buf).b_p_gefm,
                &raw mut (*buf).b_p_gp,
                &raw mut (*buf).b_p_mp,
                &raw mut (*buf).b_p_efm,
                &raw mut (*buf).b_p_ep,
                &raw mut (*buf).b_p_ffu,
                &raw mut (*buf).b_p_kp,
                &raw mut (*buf).b_p_path,
                &raw mut (*buf).b_p_tags,
                &raw mut (*buf).b_p_tc,
                &raw mut (*buf).b_p_def,
                &raw mut (*buf).b_p_inc,
                &raw mut (*buf).b_p_cot,
                &raw mut (*buf).b_p_dict,
                &raw mut (*buf).b_p_dia,
                &raw mut (*buf).b_p_tsr,
                &raw mut (*buf).b_p_tsrfu,
                &raw mut (*buf).b_p_lw,
                &raw mut (*buf).b_p_menc,
            ] {
                *field = unset_string();
            }
            (*buf).b_bkc_flags = 0 as c_uint;
            (*buf).b_tc_flags = 0 as c_uint;
            (*buf).b_cot_flags = 0 as c_uint;
            // 'includeexpr' is buffer-local only, not global-local.
            (*buf).b_p_inex = xstrdup(p_inex.get());
            copy_sctx(buf, kBufOptIncludeexpr);
            (*buf).b_p_qe = xstrdup(p_qe.get());
            copy_sctx(buf, kBufOptQuoteescape);
            (*buf).b_p_udf = p_udf.get();
            copy_sctx(buf, kBufOptUndofile);

            if dont_do_help {
                (*buf).b_p_isk = save_p_isk;
                (*buf).b_p_vts_array = vts_array(buf);
            } else {
                (*buf).b_p_isk = xstrdup(p_isk.get());
                copy_sctx(buf, kBufOptIskeyword);
                did_isk = true;
                (*buf).b_p_ts = p_ts.get();
                copy_sctx(buf, kBufOptTabstop);
                (*buf).b_p_vts = xstrdup(p_vts.get());
                copy_sctx(buf, kBufOptVartabstop);
                (*buf).b_p_vts_array = vts_array(buf);
                (*buf).b_help = false;
                // The buffer is no longer a help buffer, so 'buftype' must
                // not still say "help".
                if *(*buf).b_p_bt as c_int == 'h' as c_int {
                    clear_string_option(&raw mut (*buf).b_p_bt);
                }
                (*buf).b_p_ma = p_ma.get();
                copy_sctx(buf, kBufOptModifiable);
            }
        }

        if should_copy {
            (*buf).b_p_initialized = true;
        }

        check_buf_options(buf);
        if did_isk {
            buf_init_chartab(buf, false);
        }
    }
}

/// The tab-stop array a 'vartabstop'-like value describes.
///
/// # Safety
///
/// `value` must be a non-empty string option value.
unsafe fn tabstop_array(value: *mut c_char) -> *mut colnr_T {
    let mut array: *mut colnr_T = ptr::null_mut();
    // SAFETY: the caller's value.
    unsafe { tabstop_set(value, &raw mut array) };
    array
}

/// The buffer's 'vartabstop' array after a copy: built from the global value
/// when the buffer has none yet, and otherwise **dropped**.
///
/// The drop is upstream behaviour and leaks the old array; it is here rather
/// than inline so that the two identical call sites cannot drift.
///
/// # Safety
///
/// `buf` must be a live buffer.
unsafe fn vts_array(buf: *mut buf_T) -> *mut colnr_T {
    // SAFETY: the caller's buffer, and 'vartabstop' is a string option.
    unsafe {
        let vts = p_vts.get();
        if !vts.is_null() && *vts != NUL as c_char && (*buf).b_p_vts_array.is_null() {
            tabstop_array(vts)
        } else {
            ptr::null_mut()
        }
    }
}

/// `-M`: make every buffer unmodifiable, default included.
pub fn reset_modifiable() {
    // SAFETY: `curbuf` is live.
    unsafe { (*curbuf.get()).b_p_ma = 0 };
    p_ma.set(0);
    change_option_default(
        kOptModifiable,
        OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData { boolean: kFalse },
        },
    );
}

/// Carry a buffer's 'iminsert' back to the global value, so that the next
/// buffer starts where this one left off.
///
/// # Safety
///
/// `buf` must be a live buffer.
pub unsafe fn set_iminsert_global(buf: *mut buf_T) {
    // SAFETY: the caller's buffer.
    p_iminsert.set(unsafe { (*buf).b_p_iminsert });
}

/// As [`set_iminsert_global`], for 'imsearch'.
///
/// # Safety
///
/// `buf` must be a live buffer.
pub unsafe fn set_imsearch_global(buf: *mut buf_T) {
    // SAFETY: the caller's buffer.
    p_imsearch.set(unsafe { (*buf).b_p_imsearch });
}
