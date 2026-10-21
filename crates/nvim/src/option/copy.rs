//! Handing a new window or buffer its own copy of the option values.
//!
//! A window's values are copied from the window it was split from; a
//! buffer's are copied from the global values. Both are field-by-field
//! rather than a struct assignment, because a string field has to be
//! duplicated and a few fields are deliberately *not* copied.
//!
//! A string option's local copy owns its bytes, and `None` is the shared
//! empty string every option with no value of its own used to point at — so
//! "duplicate the value" is a `clone` and "give it none" is `None`.

#![forbid(unsafe_code)]

use core::ffi::{CStr, c_int, c_uint};

use crate::buffer::free_buf_options;
use crate::charset::buf_init_chartab;
use crate::ex_docmd::cmdmod_has;
use crate::indent::{TabStopArray, check_window_briopt, tabstop_array_of};
use crate::insexpand::{
    set_buflocal_cfu_callback, set_buflocal_cpt_callbacks, set_buflocal_ofu_callback,
};
use crate::memory::XString;
use crate::option::vars::{
    P_CPO, P_FFS, P_IMINSERT, P_IMSEARCH, P_MA, P_VSTS, p_ai, p_bin, p_bomb, p_cfu, p_ci, p_cin,
    p_cink, p_cino, p_cinsd, p_cinw, p_cms, p_com, p_cpt, p_et, p_fenc, p_fex, p_ff, p_fixeol,
    p_flp, p_fo, p_iminsert, p_imsearch, p_inde, p_indk, p_inex, p_inf, p_isk, p_keymap, p_lisp,
    p_lop, p_ma, p_ml, p_mps, p_nf, p_ofu, p_pi, p_qe, p_scbk, p_si, p_smc, p_spc, p_spf, p_spl,
    p_spo, p_sts, p_sua, p_sw, p_swf, p_tfu, p_ts, p_tw, p_udf, p_vsts, p_vts, p_wm, spo_flags,
};

use super::check::bin_save;
use super::paste::{paste_save, paste_saved_vsts};
use crate::options::{
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
    kBufOptVarsofttabstop, kBufOptVartabstop, kBufOptWrapmargin, kOptModifiable,
};
use crate::optionstr::{
    OptStringRef, check_buf_options, check_window_signcolumn, set_window_chars_option,
};
use crate::spell::compile_cap_prog;
use crate::tag::set_buflocal_tfu_callback;
use crate::types::{CmdModFlags, ColNr, CpoFlag, OptInt, ScriptCtx, WinOpt, int16_t};
use crate::window::{check_window_colorcolumn, set_winbar_win};
use crate::winlayer::{Buf, Win};

use super::{
    BCO_ALWAYS, BCO_ENTER, BCO_NOHELP, KEYMAP_INIT, NO_LOCAL_UNDOLEVEL, boolean_optval,
    change_option_default, check_blending, fill_culopt_flags, kFillchars, kListchars,
    option_last_set, parse_winhl_opt,
};
use crate::option::cpo_has;

/// A buffer-local copy of a value a string option owns.
///
/// This is what a global string option's reader is handed to: the reader
/// projects the option's own string and this makes the copy the buffer's
/// field takes over, so the borrow never leaves the closure. A compiled-in
/// name (`c"mac"`) goes through the same door.
fn dup(value: &CStr) -> Option<XString> {
    Some(XString::from_cstr(value))
}

/// Give a window's option values to a freshly split one.
pub(crate) fn win_copy_options(wp_from: Win, mut wp_to: Win) {
    // Two windows, so the two borrows below are of two objects.
    debug_assert!(wp_from != wp_to);
    copy_winopt(&wp_from.w_onebuf_opt, &mut wp_to.w_onebuf_opt);
    copy_winopt(&wp_from.w_allbuf_opt, &mut wp_to.w_allbuf_opt);
    didset_window_options(wp_to, true);
}

/// Copy one window's worth of option values. A string value is duplicated,
/// and what `t` held is dropped.
pub(crate) fn copy_winopt(f: &WinOpt, t: &mut WinOpt) {
    t.wo_arab = f.wo_arab;
    t.wo_list = f.wo_list;
    t.wo_lcs = f.wo_lcs.clone();
    t.wo_fcs = f.wo_fcs.clone();
    t.wo_nu = f.wo_nu;
    t.wo_rnu = f.wo_rnu;
    t.wo_ve = f.wo_ve.clone();
    t.wo_ve_flags = f.wo_ve_flags;
    t.wo_nuw = f.wo_nuw;
    t.wo_rl = f.wo_rl;
    t.wo_rlc = f.wo_rlc.clone();
    t.wo_sbr = f.wo_sbr.clone();
    t.wo_stl = f.wo_stl.clone();
    t.wo_wbr = f.wo_wbr.clone();
    t.wo_wrap = f.wo_wrap;
    t.wo_wrap_save = f.wo_wrap_save;
    t.wo_lbr = f.wo_lbr;
    t.wo_bri = f.wo_bri;
    t.wo_briopt = f.wo_briopt.clone();
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
    t.wo_culopt = f.wo_culopt.clone();
    t.wo_cc = f.wo_cc.clone();
    t.wo_diff = f.wo_diff;
    t.wo_diff_saved = f.wo_diff_saved;
    t.wo_eiw = f.wo_eiw.clone();
    t.wo_cocu = f.wo_cocu.clone();
    t.wo_cole = f.wo_cole;
    t.wo_fdc = f.wo_fdc.clone();
    // The four `_save` copies only hold anything while `:diffthis` is
    // in effect; otherwise they are the unset string, not a value to
    // duplicate.
    t.wo_fdc_save = if f.wo_diff_saved != 0 {
        f.wo_fdc_save.clone()
    } else {
        None
    };
    t.wo_fen = f.wo_fen;
    t.wo_fen_save = f.wo_fen_save;
    t.wo_fdi = f.wo_fdi.clone();
    t.wo_fml = f.wo_fml;
    t.wo_fdl = f.wo_fdl;
    t.wo_fdl_save = f.wo_fdl_save;
    t.wo_fdm = f.wo_fdm.clone();
    t.wo_fdm_save = if f.wo_diff_saved != 0 {
        f.wo_fdm_save.clone()
    } else {
        None
    };
    t.wo_fdn = f.wo_fdn;
    t.wo_fde = f.wo_fde.clone();
    t.wo_fdt = f.wo_fdt.clone();
    t.wo_fmr = f.wo_fmr.clone();
    t.wo_scl = f.wo_scl.clone();
    t.wo_lhi = f.wo_lhi;
    t.wo_winhl = f.wo_winhl.clone();
    t.wo_winbl = f.wo_winbl;
    t.wo_stc = f.wo_stc.clone();
    t.wo_wrap_flags = f.wo_wrap_flags;
    t.wo_stl_flags = f.wo_stl_flags;
    t.wo_wbr_flags = f.wo_wbr_flags;
    t.wo_fde_flags = f.wo_fde_flags;
    t.wo_fdt_flags = f.wo_fdt_flags;
    t.wo_script_ctx = f.wo_script_ctx;
}

/// Release every string value a window's option set owns.
pub(crate) fn clear_winopt(wop: &mut WinOpt) {
    let WinOpt {
        wo_fdc,
        wo_fdc_save,
        wo_fdi,
        wo_fdm,
        wo_fdm_save,
        wo_fde,
        wo_fdt,
        wo_fmr,
        wo_eiw,
        wo_scl,
        wo_rlc,
        wo_sbr,
        wo_stl,
        wo_culopt,
        wo_cc,
        wo_cocu,
        wo_briopt,
        wo_winhl,
        wo_lcs,
        wo_fcs,
        wo_ve,
        wo_wbr,
        wo_stc,
        ..
    } = wop;
    for field in [
        wo_fdc,
        wo_fdc_save,
        wo_fdi,
        wo_fdm,
        wo_fdm_save,
        wo_fde,
        wo_fdt,
        wo_fmr,
        wo_eiw,
        wo_scl,
        wo_rlc,
        wo_sbr,
        wo_stl,
        wo_culopt,
        wo_cc,
        wo_cocu,
        wo_briopt,
        wo_winhl,
        wo_lcs,
        wo_fcs,
        wo_ve,
        wo_wbr,
        wo_stc,
    ] {
        *field = None;
    }
}

/// Rebuild everything a window derives from its option values, after they
/// were copied or replaced wholesale.
///
/// `valid_cursor` says whether the window's cursor position can be trusted;
/// a window being created does not have one yet.
pub(crate) fn didset_window_options(window: Win, valid_cursor: bool) {
    // SAFETY: the caller's window. The handle borrows it for the one field
    // access that asked and never across a call, so none of the callees
    // below is reached while a `&mut Window` is live.
    let mut w = window;
    // 'wrap' and 'smoothscroll' scroll in different directions, and only
    // one of the two offsets can be non-zero.
    if w.w_onebuf_opt.wo_wrap != 0 {
        w.w_leftcol = 0 as ColNr;
    } else {
        w.w_skipcol = 0 as ColNr;
    }
    let _ = check_window_colorcolumn(window);
    check_window_briopt(window);
    let _ = fill_culopt_flags(None, w);
    // Each value is read where it is used: the calls above parse other
    // options and this one must see whatever they left behind.
    let _ = set_window_chars_option(window, kFillchars, true);
    let _ = set_window_chars_option(window, kListchars, true);
    parse_winhl_opt(None, Some(window));
    check_blending(window);
    set_winbar_win(window, false, valid_cursor);
    let _ = check_window_signcolumn(window);
    w.w_grid_alloc.blending = w.w_onebuf_opt.wo_winbl > 0 as OptInt;
}

/// Attribute a buffer-local option to whatever script set the global value
/// it was just copied from.
///
/// `buf_opt_idx` maps every buffer-local row to a row of the option table.
fn copy_sctx(mut buffer: Buf, bv: BufOptIndex) {
    let opt_idx = buf_opt_idx[bv as usize];
    buffer.b_p_script_ctx[bv as usize] = option_last_set(opt_idx);
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
pub(crate) fn buf_copy_options(buffer: Buf, flags: c_int) {
    let mut did_isk = false;
    // SAFETY: the caller's buffer. Every field write below goes through
    // this handle, which borrows the buffer for the one access that asked
    // and never across a call.
    let mut b = buffer;

    // Before the defaults exist there is nothing to copy: `main` makes
    // the first buffer that early.
    // Upstream's `p_cpo != NULL`: an emptied 'cpoptions' (`SavedCpo` while
    // user code runs) is not this.
    if P_CPO.is_uninit() {
        check_buf_options(buffer);
        return;
    }

    let entering = flags & BCO_ENTER as c_int != 0;
    let keep_global = !cpo_has(CpoFlag::BUFOPTGLOB) || !entering;
    let keep_local = b.b_p_initialized || (!entering && cpo_has(CpoFlag::BUFOPT));
    let should_copy = !(keep_global && keep_local);

    if should_copy || flags & BCO_ALWAYS as c_int != 0 {
        b.b_p_script_ctx.fill(ScriptCtx::NONE);

        // A help buffer keeps its own settings when it already has them
        // — jumping back to one with CTRL-T or CTRL-O must not reset it.
        let dont_do_help = (flags & BCO_NOHELP as c_int != 0 && b.b_help) || b.b_p_initialized;
        // 'iskeyword' is the one string that survives the free below.
        let save_p_isk = if dont_do_help { b.b_p_isk.take() } else { None };

        if b.b_p_initialized {
            free_buf_options(buffer, false);
        } else {
            free_buf_options(buffer, true);
            b.b_p_ro = 0;
            b.b_p_fenc = p_fenc(dup);
            // A new buffer takes the *first* of 'fileformats' rather
            // than 'fileformat', since nothing has been read yet.
            b.b_p_ff = match P_FFS.first_byte() {
                b'm' => dup(c"mac"),
                b'd' => dup(c"dos"),
                b'u' => dup(c"unix"),
                _ => p_ff(dup),
            };
            b.b_p_bh = None;
            b.b_p_bt = None;
        }

        b.b_p_ai = c_int::from(p_ai());
        copy_sctx(b, kBufOptAutoindent);
        b.b_p_ai_nopaste = c_int::from(paste_save().ai);
        b.b_p_sw = p_sw();
        copy_sctx(b, kBufOptShiftwidth);
        b.b_p_scbk = p_scbk();
        copy_sctx(b, kBufOptScrollback);
        b.b_p_tw = p_tw();
        copy_sctx(b, kBufOptTextwidth);
        b.b_p_tw_nopaste = paste_save().tw;
        b.b_p_tw_nobin = bin_save().tw;
        b.b_p_wm = p_wm();
        copy_sctx(b, kBufOptWrapmargin);
        b.b_p_wm_nopaste = paste_save().wm;
        b.b_p_wm_nobin = bin_save().wm;
        b.b_p_bin = c_int::from(p_bin());
        copy_sctx(b, kBufOptBinary);
        b.b_p_bomb = c_int::from(p_bomb());
        copy_sctx(b, kBufOptBomb);
        b.b_p_et = c_int::from(p_et());
        copy_sctx(b, kBufOptExpandtab);
        b.b_p_fixeol = c_int::from(p_fixeol());
        copy_sctx(b, kBufOptFixendofline);
        b.b_p_et_nobin = c_int::from(bin_save().et);
        b.b_p_et_nopaste = c_int::from(paste_save().et);
        b.b_p_ml = c_int::from(p_ml());
        copy_sctx(b, kBufOptModeline);
        b.b_p_ml_nobin = c_int::from(bin_save().ml);
        b.b_p_inf = c_int::from(p_inf());
        copy_sctx(b, kBufOptInfercase);

        // `:noswapfile` wins over the global 'swapfile', and leaves the
        // script context alone because nothing set it.
        if cmdmod_has(CmdModFlags::NOSWAPFILE) {
            b.b_p_swf = 0;
        } else {
            b.b_p_swf = c_int::from(p_swf());
            copy_sctx(b, kBufOptSwapfile);
        }

        b.b_p_cpt = p_cpt(dup);
        copy_sctx(b, kBufOptComplete);
        set_buflocal_cpt_callbacks(b);
        b.b_p_cfu = p_cfu(dup);
        copy_sctx(b, kBufOptCompletefunc);
        set_buflocal_cfu_callback(b);
        b.b_p_ofu = p_ofu(dup);
        copy_sctx(b, kBufOptOmnifunc);
        set_buflocal_ofu_callback(b);
        b.b_p_tfu = p_tfu(dup);
        copy_sctx(b, kBufOptTagfunc);
        set_buflocal_tfu_callback(b);

        b.b_p_sts = p_sts();
        copy_sctx(b, kBufOptSofttabstop);
        b.b_p_sts_nopaste = paste_save().sts;
        b.b_p_vsts = p_vsts(dup);
        copy_sctx(b, kBufOptVarsofttabstop);
        b.b_p_vsts_array = if P_VSTS.is_unset() {
            None
        } else {
            p_vsts(tabstop_array_of)
        };
        b.b_p_vsts_nopaste = paste_saved_vsts();

        b.b_p_com = p_com(dup);
        copy_sctx(b, kBufOptComments);
        b.b_p_cms = p_cms(dup);
        copy_sctx(b, kBufOptCommentstring);
        b.b_p_fo = p_fo(dup);
        copy_sctx(b, kBufOptFormatoptions);
        b.b_p_flp = p_flp(dup);
        copy_sctx(b, kBufOptFormatlistpat);
        b.b_p_nf = p_nf(dup);
        copy_sctx(b, kBufOptNrformats);
        b.b_p_mps = p_mps(dup);
        copy_sctx(b, kBufOptMatchpairs);
        b.b_p_si = c_int::from(p_si());
        copy_sctx(b, kBufOptSmartindent);
        b.b_p_channel = 0 as OptInt;
        b.b_p_ci = c_int::from(p_ci());
        copy_sctx(b, kBufOptCopyindent);
        b.b_p_cin = c_int::from(p_cin());
        copy_sctx(b, kBufOptCindent);
        b.b_p_cink = p_cink(dup);
        copy_sctx(b, kBufOptCinkeys);
        b.b_p_cino = p_cino(dup);
        copy_sctx(b, kBufOptCinoptions);
        b.b_p_cinsd = p_cinsd(dup);
        copy_sctx(b, kBufOptCinscopedecls);
        b.b_p_lop = p_lop(dup);
        copy_sctx(b, kBufOptLispoptions);
        // 'filetype' and 'syntax' start empty: the autocommands that
        // set them have not run for this buffer yet.
        b.b_p_ft = None;
        b.b_p_pi = c_int::from(p_pi());
        copy_sctx(b, kBufOptPreserveindent);
        b.b_p_cinw = p_cinw(dup);
        copy_sctx(b, kBufOptCinwords);
        b.b_p_lisp = c_int::from(p_lisp());
        copy_sctx(b, kBufOptLisp);
        b.b_p_syn = None;
        b.b_p_smc = p_smc();
        copy_sctx(b, kBufOptSynmaxcol);

        b.b_s.b_syn_isk = None;
        b.b_s.b_p_spc = p_spc(dup);
        copy_sctx(b, kBufOptSpellcapcheck);
        let _ = compile_cap_prog(&mut b.b_s);
        b.b_s.b_p_spf = p_spf(dup);
        copy_sctx(b, kBufOptSpellfile);
        b.b_s.b_p_spl = p_spl(dup);
        copy_sctx(b, kBufOptSpelllang);
        b.b_s.b_p_spo = p_spo(dup);
        copy_sctx(b, kBufOptSpelloptions);
        b.b_s.b_p_spo_flags = spo_flags.get();

        b.b_p_inde = p_inde(dup);
        copy_sctx(b, kBufOptIndentexpr);
        b.b_p_indk = p_indk(dup);
        copy_sctx(b, kBufOptIndentkeys);
        b.b_p_fp = None;
        b.b_p_fex = p_fex(dup);
        copy_sctx(b, kBufOptFormatexpr);
        b.b_p_sua = p_sua(dup);
        copy_sctx(b, kBufOptSuffixesadd);
        b.b_p_keymap = p_keymap(dup);
        copy_sctx(b, kBufOptKeymap);
        b.b_kmap_state = (b.b_kmap_state as c_int | KEYMAP_INIT) as int16_t;
        b.b_p_iminsert = p_iminsert();
        copy_sctx(b, kBufOptIminsert);
        b.b_p_imsearch = p_imsearch();
        copy_sctx(b, kBufOptImsearch);

        // The global-local options start unset, reading through to the
        // global value.
        b.b_p_ac = -1;
        b.b_p_ar = -1;
        b.b_p_fs = -1;
        b.b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;
        b.b_p_bkc = None;
        b.b_p_gefm = None;
        b.b_p_gp = None;
        b.b_p_mp = None;
        b.b_p_efm = None;
        b.b_p_ep = None;
        b.b_p_ffu = None;
        b.b_p_kp = None;
        b.b_p_path = None;
        b.b_p_tags = None;
        b.b_p_tc = None;
        b.b_p_def = None;
        b.b_p_inc = None;
        b.b_p_cot = None;
        b.b_p_dict = None;
        b.b_p_dia = None;
        b.b_p_tsr = None;
        b.b_p_tsrfu = None;
        b.b_p_lw = None;
        b.b_p_menc = None;
        b.b_bkc_flags = 0 as c_uint;
        b.b_tc_flags = 0 as c_uint;
        b.b_cot_flags = 0 as c_uint;
        // 'includeexpr' is buffer-local only, not global-local.
        b.b_p_inex = p_inex(dup);
        copy_sctx(b, kBufOptIncludeexpr);
        b.b_p_qe = p_qe(dup);
        copy_sctx(b, kBufOptQuoteescape);
        b.b_p_udf = c_int::from(p_udf());
        copy_sctx(b, kBufOptUndofile);

        if dont_do_help {
            b.b_p_isk = save_p_isk;
            b.b_p_vts_array = vts_array(b);
        } else {
            b.b_p_isk = p_isk(dup);
            copy_sctx(b, kBufOptIskeyword);
            did_isk = true;
            b.b_p_ts = p_ts();
            copy_sctx(b, kBufOptTabstop);
            b.b_p_vts = p_vts(dup);
            copy_sctx(b, kBufOptVartabstop);
            b.b_p_vts_array = vts_array(b);
            b.b_help = false;
            // The buffer is no longer a help buffer, so 'buftype' must
            // not still say "help".
            if b.b_p_bt.bytes().first() == Some(&b'h') {
                b.b_p_bt = None;
            }
            b.b_p_ma = c_int::from(p_ma());
            copy_sctx(b, kBufOptModifiable);
        }
    }

    if should_copy {
        b.b_p_initialized = true;
    }

    check_buf_options(buffer);
    if did_isk {
        buf_init_chartab(buffer, false);
    }
}

/// The buffer's 'vartabstop' array after a copy: built from the global value
/// when the buffer has none yet, and otherwise **dropped**.
///
/// The drop is upstream behaviour (which leaks the old array); it is here
/// rather than inline so that the two identical call sites cannot drift.
fn vts_array(buffer: Buf) -> TabStopArray {
    p_vts(|vts| {
        if !vts.is_empty() && buffer.b_p_vts_array.is_none() {
            tabstop_array_of(vts)
        } else {
            None
        }
    })
}

/// `-M`: make every buffer unmodifiable, default included.
pub(crate) fn reset_modifiable() {
    Buf::current().b_p_ma = 0;
    P_MA.set(false);
    change_option_default(kOptModifiable, boolean_optval(Some(false)));
}

/// Carry a buffer's 'iminsert' back to the global value, so that the next
/// buffer starts where this one left off.
pub(crate) fn set_iminsert_global(buffer: Buf) {
    P_IMINSERT.set(buffer.b_p_iminsert);
}

/// As [`set_iminsert_global`], for 'imsearch'.
pub(crate) fn set_imsearch_global(buffer: Buf) {
    P_IMSEARCH.set(buffer.b_p_imsearch);
}
