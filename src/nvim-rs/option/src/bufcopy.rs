//! Buffer option copying (buf_copy_options migration)
//!
//! This module implements `rs_buf_copy_options`, which copies global option
//! values to buffer-local options when creating or entering a buffer.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int};

use crate::buf_opt_index::{
    K_BUF_OPT_AUTOINDENT, K_BUF_OPT_BINARY, K_BUF_OPT_BOMB, K_BUF_OPT_CINDENT, K_BUF_OPT_CINKEYS,
    K_BUF_OPT_CINOPTIONS, K_BUF_OPT_CINSCOPEDECLS, K_BUF_OPT_CINWORDS, K_BUF_OPT_COMMENTS,
    K_BUF_OPT_COMMENTSTRING, K_BUF_OPT_COMPLETE, K_BUF_OPT_COMPLETEFUNC, K_BUF_OPT_COMPLETESLASH,
    K_BUF_OPT_COPYINDENT, K_BUF_OPT_EXPANDTAB, K_BUF_OPT_FIXENDOFLINE, K_BUF_OPT_FORMATEXPR,
    K_BUF_OPT_FORMATLISTPAT, K_BUF_OPT_FORMATOPTIONS, K_BUF_OPT_IMINSERT, K_BUF_OPT_IMSEARCH,
    K_BUF_OPT_INCLUDEEXPR, K_BUF_OPT_INDENTEXPR, K_BUF_OPT_INDENTKEYS, K_BUF_OPT_INFERCASE,
    K_BUF_OPT_ISKEYWORD, K_BUF_OPT_KEYMAP, K_BUF_OPT_LISP, K_BUF_OPT_LISPOPTIONS,
    K_BUF_OPT_MATCHPAIRS, K_BUF_OPT_MODELINE, K_BUF_OPT_MODIFIABLE, K_BUF_OPT_NRFORMATS,
    K_BUF_OPT_OMNIFUNC, K_BUF_OPT_PRESERVEINDENT, K_BUF_OPT_QUOTEESCAPE, K_BUF_OPT_SCROLLBACK,
    K_BUF_OPT_SHIFTWIDTH, K_BUF_OPT_SMARTINDENT, K_BUF_OPT_SOFTTABSTOP, K_BUF_OPT_SPELLCAPCHECK,
    K_BUF_OPT_SPELLFILE, K_BUF_OPT_SPELLLANG, K_BUF_OPT_SPELLOPTIONS, K_BUF_OPT_SUFFIXESADD,
    K_BUF_OPT_SWAPFILE, K_BUF_OPT_SYNMAXCOL, K_BUF_OPT_TABSTOP, K_BUF_OPT_TAGFUNC,
    K_BUF_OPT_TEXTWIDTH, K_BUF_OPT_UNDOFILE, K_BUF_OPT_VARSOFTTABSTOP, K_BUF_OPT_VARTABSTOP,
    K_BUF_OPT_WRAPMARGIN,
};
use crate::OptInt;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    fn nvim_get_bco_enter() -> c_int;
    fn nvim_get_bco_always() -> c_int;
    fn nvim_get_bco_nohelp() -> c_int;
    fn nvim_get_cpo_bufoptglob() -> c_int;
    fn nvim_get_cpo_bufopt() -> c_int;
    fn nvim_get_cmod_noswapfile() -> c_int;

    fn nvim_option_get_cpo() -> *const c_char;
    fn nvim_call_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
    fn nvim_cmdmod_get_cmod_flags() -> c_int;

    fn nvim_buf_get_b_p_initialized(buf: *mut core::ffi::c_void) -> c_int;
    fn nvim_buf_set_b_p_initialized(buf: *mut core::ffi::c_void, val: c_int);
    fn nvim_buf_get_b_help(buf: *mut core::ffi::c_void) -> c_int;
    fn nvim_buf_set_b_help(buf: *mut core::ffi::c_void, val: c_int);
    fn nvim_buf_clear_b_p_script_ctx(buf: *mut core::ffi::c_void);
    fn nvim_buf_save_and_clear_b_p_isk(buf: *mut core::ffi::c_void) -> *mut c_char;
    fn nvim_buf_restore_b_p_isk(buf: *mut core::ffi::c_void, saved: *mut c_char);
    fn nvim_buf_clear_b_p_ro(buf: *mut core::ffi::c_void);
    fn nvim_buf_get_b_p_bt_is_help(buf: *mut core::ffi::c_void) -> c_int;

    fn nvim_call_free_buf_options(buf: *mut core::ffi::c_void, free_flags: c_int);
    fn nvim_call_check_buf_options(buf: *mut core::ffi::c_void);
    fn nvim_call_buf_init_chartab_buf(buf: *mut core::ffi::c_void);
    fn nvim_call_compile_cap_prog_buf(buf: *mut core::ffi::c_void);

    fn nvim_call_tabstop_set_vsts(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_call_tabstop_set_vts(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_get_b_p_vts_array_is_null(buf: *mut core::ffi::c_void) -> c_int;

    fn nvim_call_set_buflocal_cpt_callbacks(buf: *mut core::ffi::c_void);
    fn nvim_call_set_buflocal_cfu_callback(buf: *mut core::ffi::c_void);
    fn nvim_call_set_buflocal_ofu_callback(buf: *mut core::ffi::c_void);
    fn rs_set_buflocal_tfu_callback(buf: *mut core::ffi::c_void);

    fn nvim_buf_kmap_state_set_init(buf: *mut core::ffi::c_void);
    fn nvim_buf_clear_b_p_bt_if_help(buf: *mut core::ffi::c_void);

    fn nvim_buf_set_b_p_fenc_dup(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_ff_from_ffs(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_bh_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_bt_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_ft_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_syn_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_fp_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_s_syn_isk_empty(buf: *mut core::ffi::c_void);

    fn nvim_buf_set_b_p_cpt_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_cfu_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_ofu_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_tfu_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_vsts_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_vsts_nopaste_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_com_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_cms_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_fo_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_flp_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_nf_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_mps_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_cink_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_cino_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_cinsd_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_lop_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_cinw_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_s_spc_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_s_spf_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_s_spl_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_s_spo_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_inde_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_indk_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_fex_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_sua_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_keymap_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_qe_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_inex_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_isk_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_vts_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_p_csl_dup(buf: *mut core::ffi::c_void, s: *const c_char);

    fn nvim_buf_set_b_p_ai(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_ai_nopaste(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_sw(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_scbk(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_tw(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_tw_nopaste(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_tw_nobin(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_wm(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_wm_nopaste(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_wm_nobin(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_bin(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_bomb(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_et(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_fixeol(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_et_nobin(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_et_nopaste(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_ml(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_ml_nobin(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_inf(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_swf(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_si(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_channel(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_ci(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_cin(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_pi(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_lisp(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_smc(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_sts(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_sts_nopaste(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_udf(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_ts(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_iminsert(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_imsearch(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_ma(buf: *mut core::ffi::c_void, v: c_int);

    fn nvim_buf_set_b_p_ac_minus1(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_ar_minus1(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_ul_no_local(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_bkc_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_gefm_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_gp_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_mp_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_efm_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_ep_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_ffu_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_kp_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_path_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_tags_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_tc_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_def_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_inc_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_cot_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_dict_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_dia_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_tsr_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_tsrfu_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_lw_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_menc_empty(buf: *mut core::ffi::c_void);

    fn nvim_buf_copy_opt_sctx(buf: *mut core::ffi::c_void, bv: c_int);
    fn nvim_buf_set_b_s_spo_flags_from_global(buf: *mut core::ffi::c_void);
    fn nvim_get_backslash_in_filename() -> c_int;

    fn nvim_get_p_ai() -> c_int;
    fn nvim_get_p_ai_nopaste() -> bool;
    fn nvim_get_p_bin() -> bool;
    fn nvim_get_p_bomb() -> bool;
    fn nvim_get_p_ci() -> bool;
    fn nvim_get_p_cin() -> bool;
    fn nvim_get_p_et() -> c_int;
    fn nvim_get_p_et_nobin() -> c_int;
    fn nvim_get_p_et_nopaste() -> bool;
    fn nvim_get_p_fixeol() -> bool;
    fn nvim_get_p_inf() -> c_int;
    fn nvim_get_p_lisp() -> bool;
    fn nvim_get_p_ma() -> c_int;
    fn nvim_get_p_ml() -> c_int;
    fn nvim_get_p_ml_nobin() -> c_int;
    fn nvim_get_p_pi() -> bool;
    fn nvim_get_p_si() -> bool;
    fn nvim_get_p_swf() -> bool;
    fn nvim_get_p_udf() -> c_int;
    fn nvim_get_p_sw() -> OptInt;
    fn nvim_get_p_scbk() -> OptInt;
    fn nvim_get_p_tw() -> OptInt;
    fn nvim_get_p_tw_nobin() -> OptInt;
    fn nvim_get_p_tw_nopaste() -> OptInt;
    fn nvim_get_p_wm() -> OptInt;
    fn nvim_get_p_wm_nobin() -> OptInt;
    fn nvim_get_p_wm_nopaste() -> OptInt;
    fn nvim_get_p_sts() -> OptInt;
    fn nvim_get_p_sts_nopaste() -> OptInt;
    fn nvim_get_p_ts() -> OptInt;
    fn nvim_get_p_smc() -> OptInt;
    fn nvim_get_p_iminsert() -> OptInt;
    fn nvim_get_p_imsearch() -> OptInt;
    fn nvim_get_p_vsts() -> *const c_char;
    fn nvim_get_p_vts() -> *const c_char;
    fn nvim_get_p_vsts_nopaste() -> *const c_char;
    fn nvim_get_p_cpt() -> *const c_char;
    fn nvim_get_p_cfu() -> *const c_char;
    fn nvim_get_p_ofu() -> *const c_char;
    fn nvim_get_p_tfu() -> *const c_char;
    fn nvim_get_p_com() -> *const c_char;
    fn nvim_get_p_cms() -> *const c_char;
    fn nvim_get_p_fo() -> *const c_char;
    fn nvim_get_p_flp() -> *const c_char;
    fn nvim_get_p_nf() -> *const c_char;
    fn nvim_get_p_mps() -> *const c_char;
    fn nvim_get_p_cink() -> *const c_char;
    fn nvim_get_p_cino() -> *const c_char;
    fn nvim_get_p_cinsd() -> *const c_char;
    fn nvim_get_p_lop() -> *const c_char;
    fn nvim_get_p_cinw() -> *const c_char;
    fn nvim_get_p_inde() -> *const c_char;
    fn nvim_get_p_indk() -> *const c_char;
    fn nvim_get_p_fex() -> *const c_char;
    fn nvim_get_p_sua() -> *const c_char;
    fn nvim_get_p_keymap() -> *const c_char;
    fn nvim_get_p_qe() -> *const c_char;
    fn nvim_get_p_inex() -> *const c_char;
    fn nvim_get_p_spc() -> *const c_char;
    fn nvim_get_p_spf() -> *const c_char;
    fn nvim_get_p_spl() -> *const c_char;
    fn nvim_get_p_spo() -> *const c_char;
    fn nvim_get_p_isk() -> *const c_char;
    fn nvim_get_p_csl() -> *const c_char;
}

// NUL character
const NUL: u8 = 0;

/// Check if a C string is non-null and non-empty.
unsafe fn cstr_nonempty(s: *const c_char) -> bool {
    !s.is_null() && (*s as u8) != NUL
}

/// Perform the bulk mechanical copy of options from globals to buffer fields.
/// This mirrors all the `buf->b_p_X = p_X` and `COPY_OPT_SCTX` lines in
/// the original C `buf_copy_options`.
///
/// `dont_do_help`: if true, skip the help-buffer-specific fields (isk, ts, vts,
/// b_help, bt, ma). The b_p_isk save/restore is done by the caller.
#[allow(clippy::too_many_lines)]
unsafe fn do_bulk_copy(buf: *mut core::ffi::c_void, dont_do_help: bool) {
    nvim_buf_set_b_p_ai(buf, c_int::from(nvim_get_p_ai()));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_AUTOINDENT);
    nvim_buf_set_b_p_ai_nopaste(buf, c_int::from(nvim_get_p_ai_nopaste()));

    nvim_buf_set_b_p_sw(buf, nvim_get_p_sw());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SHIFTWIDTH);

    nvim_buf_set_b_p_scbk(buf, nvim_get_p_scbk());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SCROLLBACK);

    nvim_buf_set_b_p_tw(buf, nvim_get_p_tw());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_TEXTWIDTH);
    nvim_buf_set_b_p_tw_nopaste(buf, nvim_get_p_tw_nopaste());
    nvim_buf_set_b_p_tw_nobin(buf, nvim_get_p_tw_nobin());

    nvim_buf_set_b_p_wm(buf, nvim_get_p_wm());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_WRAPMARGIN);
    nvim_buf_set_b_p_wm_nopaste(buf, nvim_get_p_wm_nopaste());
    nvim_buf_set_b_p_wm_nobin(buf, nvim_get_p_wm_nobin());

    nvim_buf_set_b_p_bin(buf, c_int::from(nvim_get_p_bin()));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_BINARY);

    nvim_buf_set_b_p_bomb(buf, c_int::from(nvim_get_p_bomb()));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_BOMB);

    nvim_buf_set_b_p_et(buf, c_int::from(nvim_get_p_et()));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_EXPANDTAB);

    nvim_buf_set_b_p_fixeol(buf, c_int::from(nvim_get_p_fixeol()));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_FIXENDOFLINE);

    nvim_buf_set_b_p_et_nobin(buf, nvim_get_p_et_nobin());
    nvim_buf_set_b_p_et_nopaste(buf, c_int::from(nvim_get_p_et_nopaste()));

    nvim_buf_set_b_p_ml(buf, c_int::from(nvim_get_p_ml()));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_MODELINE);
    nvim_buf_set_b_p_ml_nobin(buf, nvim_get_p_ml_nobin());

    nvim_buf_set_b_p_inf(buf, nvim_get_p_inf());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_INFERCASE);

    // swapfile: suppress if :noswapfile modifier is active
    if nvim_cmdmod_get_cmod_flags() & nvim_get_cmod_noswapfile() != 0 {
        nvim_buf_set_b_p_swf(buf, 0);
    } else {
        nvim_buf_set_b_p_swf(buf, c_int::from(nvim_get_p_swf()));
        nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SWAPFILE);
    }

    nvim_buf_set_b_p_cpt_dup(buf, nvim_get_p_cpt());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_COMPLETE);
    nvim_call_set_buflocal_cpt_callbacks(buf);

    // completeslash only on BACKSLASH_IN_FILENAME platforms
    if nvim_get_backslash_in_filename() != 0 {
        nvim_buf_set_b_p_csl_dup(buf, nvim_get_p_csl());
        nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_COMPLETESLASH);
    }

    nvim_buf_set_b_p_cfu_dup(buf, nvim_get_p_cfu());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_COMPLETEFUNC);
    nvim_call_set_buflocal_cfu_callback(buf);

    nvim_buf_set_b_p_ofu_dup(buf, nvim_get_p_ofu());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_OMNIFUNC);
    nvim_call_set_buflocal_ofu_callback(buf);

    nvim_buf_set_b_p_tfu_dup(buf, nvim_get_p_tfu());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_TAGFUNC);
    rs_set_buflocal_tfu_callback(buf);

    nvim_buf_set_b_p_sts(buf, nvim_get_p_sts());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SOFTTABSTOP);
    nvim_buf_set_b_p_sts_nopaste(buf, nvim_get_p_sts_nopaste());

    let p_vsts = nvim_get_p_vsts();
    nvim_buf_set_b_p_vsts_dup(buf, p_vsts);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_VARSOFTTABSTOP);
    if cstr_nonempty(p_vsts) {
        nvim_call_tabstop_set_vsts(buf, p_vsts);
    } else {
        // buf->b_p_vsts_array = NULL (tabstop_set with null does nothing;
        // free_buf_options already zeroed it, but call with null to be safe)
        nvim_call_tabstop_set_vsts(buf, core::ptr::null());
    }
    nvim_buf_set_b_p_vsts_nopaste_dup(buf, nvim_get_p_vsts_nopaste());

    nvim_buf_set_b_p_com_dup(buf, nvim_get_p_com());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_COMMENTS);

    nvim_buf_set_b_p_cms_dup(buf, nvim_get_p_cms());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_COMMENTSTRING);

    nvim_buf_set_b_p_fo_dup(buf, nvim_get_p_fo());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_FORMATOPTIONS);

    nvim_buf_set_b_p_flp_dup(buf, nvim_get_p_flp());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_FORMATLISTPAT);

    nvim_buf_set_b_p_nf_dup(buf, nvim_get_p_nf());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_NRFORMATS);

    nvim_buf_set_b_p_mps_dup(buf, nvim_get_p_mps());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_MATCHPAIRS);

    nvim_buf_set_b_p_si(buf, c_int::from(nvim_get_p_si()));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SMARTINDENT);

    nvim_buf_set_b_p_channel(buf, 0);

    nvim_buf_set_b_p_ci(buf, c_int::from(nvim_get_p_ci()));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_COPYINDENT);

    nvim_buf_set_b_p_cin(buf, c_int::from(nvim_get_p_cin()));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_CINDENT);

    nvim_buf_set_b_p_cink_dup(buf, nvim_get_p_cink());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_CINKEYS);

    nvim_buf_set_b_p_cino_dup(buf, nvim_get_p_cino());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_CINOPTIONS);

    nvim_buf_set_b_p_cinsd_dup(buf, nvim_get_p_cinsd());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_CINSCOPEDECLS);

    nvim_buf_set_b_p_lop_dup(buf, nvim_get_p_lop());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_LISPOPTIONS);

    // Don't copy 'filetype' - it must be detected
    nvim_buf_set_b_p_ft_empty(buf);

    nvim_buf_set_b_p_pi(buf, c_int::from(nvim_get_p_pi()));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_PRESERVEINDENT);

    nvim_buf_set_b_p_cinw_dup(buf, nvim_get_p_cinw());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_CINWORDS);

    nvim_buf_set_b_p_lisp(buf, c_int::from(nvim_get_p_lisp()));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_LISP);

    // Don't copy 'syntax' - it must be set
    nvim_buf_set_b_p_syn_empty(buf);

    nvim_buf_set_b_p_smc(buf, nvim_get_p_smc());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SYNMAXCOL);

    nvim_buf_set_b_s_syn_isk_empty(buf);

    nvim_buf_set_b_s_spc_dup(buf, nvim_get_p_spc());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SPELLCAPCHECK);
    nvim_call_compile_cap_prog_buf(buf);

    nvim_buf_set_b_s_spf_dup(buf, nvim_get_p_spf());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SPELLFILE);

    nvim_buf_set_b_s_spl_dup(buf, nvim_get_p_spl());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SPELLLANG);

    nvim_buf_set_b_s_spo_dup(buf, nvim_get_p_spo());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SPELLOPTIONS);
    nvim_buf_set_b_s_spo_flags_from_global(buf);

    nvim_buf_set_b_p_inde_dup(buf, nvim_get_p_inde());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_INDENTEXPR);

    nvim_buf_set_b_p_indk_dup(buf, nvim_get_p_indk());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_INDENTKEYS);

    // Don't copy 'formatprg' - no local value by default
    nvim_buf_set_b_p_fp_empty(buf);

    nvim_buf_set_b_p_fex_dup(buf, nvim_get_p_fex());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_FORMATEXPR);

    nvim_buf_set_b_p_sua_dup(buf, nvim_get_p_sua());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SUFFIXESADD);

    nvim_buf_set_b_p_keymap_dup(buf, nvim_get_p_keymap());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_KEYMAP);
    nvim_buf_kmap_state_set_init(buf);

    // Langmap/IME state: copy from current buffer is better than resetting
    // iminsert/imsearch are OptInt globals but b_p_iminsert/b_p_imsearch are int fields
    #[allow(clippy::cast_possible_truncation)]
    nvim_buf_set_b_p_iminsert(buf, nvim_get_p_iminsert() as c_int);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_IMINSERT);
    #[allow(clippy::cast_possible_truncation)]
    nvim_buf_set_b_p_imsearch(buf, nvim_get_p_imsearch() as c_int);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_IMSEARCH);

    // Global-local options: use global value (no local copy)
    nvim_buf_set_b_p_ac_minus1(buf);
    nvim_buf_set_b_p_ar_minus1(buf);
    nvim_buf_set_b_p_ul_no_local(buf);
    nvim_buf_set_b_p_bkc_empty(buf);
    nvim_buf_set_b_p_gefm_empty(buf);
    nvim_buf_set_b_p_gp_empty(buf);
    nvim_buf_set_b_p_mp_empty(buf);
    nvim_buf_set_b_p_efm_empty(buf);
    nvim_buf_set_b_p_ep_empty(buf);
    nvim_buf_set_b_p_ffu_empty(buf);
    nvim_buf_set_b_p_kp_empty(buf);
    nvim_buf_set_b_p_path_empty(buf);
    nvim_buf_set_b_p_tags_empty(buf);
    nvim_buf_set_b_p_tc_empty(buf);
    nvim_buf_set_b_p_def_empty(buf);
    nvim_buf_set_b_p_inc_empty(buf);

    nvim_buf_set_b_p_inex_dup(buf, nvim_get_p_inex());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_INCLUDEEXPR);

    nvim_buf_set_b_p_cot_empty(buf);
    nvim_buf_set_b_p_dict_empty(buf);
    nvim_buf_set_b_p_dia_empty(buf);
    nvim_buf_set_b_p_tsr_empty(buf);
    nvim_buf_set_b_p_tsrfu_empty(buf);

    nvim_buf_set_b_p_qe_dup(buf, nvim_get_p_qe());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_QUOTEESCAPE);

    nvim_buf_set_b_p_udf(buf, nvim_get_p_udf());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_UNDOFILE);

    nvim_buf_set_b_p_lw_empty(buf);
    nvim_buf_set_b_p_menc_empty(buf);

    // Help-specific options: iskeyword, tabstop, vartabstop, b_help, buftype, modifiable
    if dont_do_help {
        // b_p_isk was saved and NULLed by caller; here we handle vts_array only
        let vts_global = nvim_get_p_vts();
        if cstr_nonempty(vts_global) && nvim_buf_get_b_p_vts_array_is_null(buf) != 0 {
            nvim_call_tabstop_set_vts(buf, vts_global);
        } else {
            nvim_call_tabstop_set_vts(buf, core::ptr::null());
        }
    } else {
        nvim_buf_set_b_p_isk_dup(buf, nvim_get_p_isk());
        nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_ISKEYWORD);

        nvim_buf_set_b_p_ts(buf, nvim_get_p_ts());
        nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_TABSTOP);

        nvim_buf_set_b_p_vts_dup(buf, nvim_get_p_vts());
        nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_VARTABSTOP);

        let vts_global = nvim_get_p_vts();
        if cstr_nonempty(vts_global) && nvim_buf_get_b_p_vts_array_is_null(buf) != 0 {
            nvim_call_tabstop_set_vts(buf, vts_global);
        } else {
            nvim_call_tabstop_set_vts(buf, core::ptr::null());
        }

        nvim_buf_set_b_help(buf, 0);

        if nvim_buf_get_b_p_bt_is_help(buf) != 0 {
            nvim_buf_clear_b_p_bt_if_help(buf);
        }

        nvim_buf_set_b_p_ma(buf, c_int::from(nvim_get_p_ma()));
        nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_MODIFIABLE);
    }
}

/// Rust implementation of `buf_copy_options`.
///
/// Copies global option values to buffer-local options when creating a new
/// buffer or entering a buffer.
///
/// # Safety
/// `buf` must be a valid non-null `buf_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_copy_options(buf: *mut core::ffi::c_void, flags: c_int) {
    let p_cpo = nvim_option_get_cpo();
    // Skip when option defaults have not been set yet (first buffer allocation).
    if p_cpo.is_null() {
        nvim_call_check_buf_options(buf);
        return;
    }

    let bco_enter = nvim_get_bco_enter();
    let bco_always = nvim_get_bco_always();
    let bco_nohelp = nvim_get_bco_nohelp();
    let cpo_s = nvim_get_cpo_bufopt();
    let cpo_cap_s = nvim_get_cpo_bufoptglob();

    let initialized = nvim_buf_get_b_p_initialized(buf) != 0;

    // Determine should_copy.
    //
    //    'S'      BCO_ENTER  initialized  's'  should_copy
    //    yes        yes          X         X      true
    //    yes        no          yes        X      false
    //    no          X          yes        X      false
    //     X         no          no        yes     false
    //     X         no          no        no      true
    //    no         yes         no         X      true
    let should_copy = !((nvim_call_vim_strchr(p_cpo, cpo_cap_s).is_null()
        || (flags & bco_enter) == 0)
        && (initialized
            || ((flags & bco_enter) == 0 && !nvim_call_vim_strchr(p_cpo, cpo_s).is_null())));

    if should_copy || (flags & bco_always) != 0 {
        nvim_buf_clear_b_p_script_ctx(buf);

        // Don't copy the options specific to a help buffer when BCO_NOHELP is
        // given or the options were initialized already (jumping back to a help
        // file with CTRL-T or CTRL-O).
        let dont_do_help =
            ((flags & bco_nohelp) != 0 && nvim_buf_get_b_help(buf) != 0) || initialized;

        // If dont_do_help, save b_p_isk before free_buf_options
        let save_p_isk: *mut c_char = if dont_do_help {
            nvim_buf_save_and_clear_b_p_isk(buf)
        } else {
            core::ptr::null_mut()
        };

        // Free old allocated strings; initialize some fields if first time
        if initialized {
            nvim_call_free_buf_options(buf, 0);
        } else {
            nvim_call_free_buf_options(buf, 1);
            nvim_buf_clear_b_p_ro(buf);
            nvim_buf_set_b_p_fenc_dup(buf);
            nvim_buf_set_b_p_ff_from_ffs(buf);
            nvim_buf_set_b_p_bh_empty(buf);
            nvim_buf_set_b_p_bt_empty(buf);
        }

        // Perform the mechanical bulk copy
        do_bulk_copy(buf, dont_do_help);

        // Restore b_p_isk if we saved it
        if dont_do_help {
            nvim_buf_restore_b_p_isk(buf, save_p_isk);
        }
    }

    // Set initialized flag
    if should_copy {
        nvim_buf_set_b_p_initialized(buf, 1);
    }

    nvim_call_check_buf_options(buf);

    // If isk was copied (took the !dont_do_help branch), reinit the chartab.
    //
    // did_isk = true when:
    //   1. We entered the copy block (should_copy || BCO_ALWAYS), AND
    //   2. We took the !dont_do_help path.
    //
    // After do_bulk_copy(!dont_do_help), b_help is set to false.
    // So re-evaluating b_help here gives false for that path, making
    // dont_do_help_recomputed = initialized (which was false). Correct.
    // For the dont_do_help path, b_help is unchanged, so re-evaluation matches original.
    let did_copy = should_copy || (flags & bco_always) != 0;
    let dont_do_help_recomputed =
        ((flags & bco_nohelp) != 0 && nvim_buf_get_b_help(buf) != 0) || initialized;
    let did_isk = did_copy && !dont_do_help_recomputed;

    if did_isk {
        nvim_call_buf_init_chartab_buf(buf);
    }
}
