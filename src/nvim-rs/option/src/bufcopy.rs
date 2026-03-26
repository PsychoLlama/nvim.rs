//! Buffer option copying (buf_copy_options migration)
//!
//! This module implements `rs_buf_copy_options`, which copies global option
//! values to buffer-local options when creating or entering a buffer.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int};

use crate::buf_opt_index::{
    K_BUF_OPT_AUTOINDENT, K_BUF_OPT_BINARY, K_BUF_OPT_BOMB, K_BUF_OPT_CINDENT, K_BUF_OPT_CINKEYS,
    K_BUF_OPT_CINOPTIONS, K_BUF_OPT_CINSCOPEDECLS, K_BUF_OPT_CINWORDS, K_BUF_OPT_COMMENTS,
    K_BUF_OPT_COMMENTSTRING, K_BUF_OPT_COMPLETE, K_BUF_OPT_COMPLETEFUNC, K_BUF_OPT_COPYINDENT,
    K_BUF_OPT_EXPANDTAB, K_BUF_OPT_FIXENDOFLINE, K_BUF_OPT_FORMATEXPR, K_BUF_OPT_FORMATLISTPAT,
    K_BUF_OPT_FORMATOPTIONS, K_BUF_OPT_IMINSERT, K_BUF_OPT_IMSEARCH, K_BUF_OPT_INCLUDEEXPR,
    K_BUF_OPT_INDENTEXPR, K_BUF_OPT_INDENTKEYS, K_BUF_OPT_INFERCASE, K_BUF_OPT_ISKEYWORD,
    K_BUF_OPT_KEYMAP, K_BUF_OPT_LISP, K_BUF_OPT_LISPOPTIONS, K_BUF_OPT_MATCHPAIRS,
    K_BUF_OPT_MODELINE, K_BUF_OPT_MODIFIABLE, K_BUF_OPT_NRFORMATS, K_BUF_OPT_OMNIFUNC,
    K_BUF_OPT_PRESERVEINDENT, K_BUF_OPT_QUOTEESCAPE, K_BUF_OPT_SCROLLBACK, K_BUF_OPT_SHIFTWIDTH,
    K_BUF_OPT_SMARTINDENT, K_BUF_OPT_SOFTTABSTOP, K_BUF_OPT_SPELLCAPCHECK, K_BUF_OPT_SPELLFILE,
    K_BUF_OPT_SPELLLANG, K_BUF_OPT_SPELLOPTIONS, K_BUF_OPT_SUFFIXESADD, K_BUF_OPT_SWAPFILE,
    K_BUF_OPT_SYNMAXCOL, K_BUF_OPT_TABSTOP, K_BUF_OPT_TAGFUNC, K_BUF_OPT_TEXTWIDTH,
    K_BUF_OPT_UNDOFILE, K_BUF_OPT_VARSOFTTABSTOP, K_BUF_OPT_VARTABSTOP, K_BUF_OPT_WRAPMARGIN,
};
use crate::opt_index::{
    K_OPT_AUTOINDENT, K_OPT_BINARY, K_OPT_BOMB, K_OPT_BUFTYPE, K_OPT_CHANNEL, K_OPT_CINDENT,
    K_OPT_CINKEYS, K_OPT_CINOPTIONS, K_OPT_CINSCOPEDECLS, K_OPT_CINWORDS, K_OPT_COMMENTS,
    K_OPT_COMMENTSTRING, K_OPT_COMPLETE, K_OPT_COMPLETEFUNC, K_OPT_COPYINDENT, K_OPT_DEFINE,
    K_OPT_DICTIONARY, K_OPT_DIFFANCHORS, K_OPT_EQUALPRG, K_OPT_ERRORFORMAT, K_OPT_EXPANDTAB,
    K_OPT_FILEFORMAT, K_OPT_FILETYPE, K_OPT_FINDFUNC, K_OPT_FIXENDOFLINE, K_OPT_FORMATEXPR,
    K_OPT_FORMATLISTPAT, K_OPT_FORMATOPTIONS, K_OPT_FORMATPRG, K_OPT_GREPFORMAT, K_OPT_GREPPRG,
    K_OPT_INCLUDE, K_OPT_INCLUDEEXPR, K_OPT_INDENTEXPR, K_OPT_INDENTKEYS, K_OPT_INFERCASE,
    K_OPT_ISKEYWORD, K_OPT_KEYMAP, K_OPT_KEYWORDPRG, K_OPT_LISP, K_OPT_LISPOPTIONS,
    K_OPT_LISPWORDS, K_OPT_MAKEENCODING, K_OPT_MAKEPRG, K_OPT_MATCHPAIRS, K_OPT_NRFORMATS,
    K_OPT_OMNIFUNC, K_OPT_PATH, K_OPT_PRESERVEINDENT, K_OPT_QUOTEESCAPE, K_OPT_SCROLLBACK,
    K_OPT_SHIFTWIDTH, K_OPT_SMARTINDENT, K_OPT_SOFTTABSTOP, K_OPT_SUFFIXESADD, K_OPT_SWAPFILE,
    K_OPT_SYNMAXCOL, K_OPT_SYNTAX, K_OPT_TABSTOP, K_OPT_TAGFUNC, K_OPT_TAGS, K_OPT_TEXTWIDTH,
    K_OPT_THESAURUS, K_OPT_THESAURUSFUNC, K_OPT_UNDOFILE, K_OPT_VARSOFTTABSTOP, K_OPT_VARTABSTOP,
    K_OPT_WRAPMARGIN,
};
use crate::OptInt;
use crate::{BCO_ALWAYS, BCO_ENTER, BCO_NOHELP, CMOD_NOSWAPFILE, CPO_BUFOPT, CPO_BUFOPTGLOB};

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Global option variables (Phase 1 bool/int)
    static mut p_ai: c_int;
    static mut p_bin: c_int;
    static mut p_bomb: c_int;
    static mut p_ci: c_int;
    static mut p_cin: c_int;
    static mut p_et: c_int;
    static mut p_fixeol: c_int;
    static mut p_lisp: c_int;
    static mut p_ma: c_int;
    static mut p_ml: c_int;
    static mut p_pi: c_int;
    static mut p_si: c_int;
    static mut p_swf: c_int;
    static mut p_udf: c_int;
    // Global option variables (Phase 2 OptInt)
    static mut p_sw: OptInt;
    static mut p_scbk: OptInt;
    static mut p_tw: OptInt;
    static mut p_wm: OptInt;
    static mut p_sts: OptInt;
    static mut p_ts: OptInt;
    static mut p_smc: OptInt;
    static mut p_iminsert: OptInt;
    static mut p_imsearch: OptInt;
}

extern "C" {
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    #[link_name = "nvim_get_cmdmod_cmod_flags"]
    fn nvim_cmdmod_get_cmod_flags() -> c_int;

    fn nvim_buf_get_b_p_initialized(buf: *mut core::ffi::c_void) -> c_int;
    fn nvim_buf_set_b_p_initialized(buf: *mut core::ffi::c_void, val: c_int);
    fn nvim_buf_get_help(buf: *mut core::ffi::c_void) -> c_int;
    fn nvim_buf_set_b_help(buf: *mut core::ffi::c_void, val: c_int);
    fn nvim_buf_clear_b_p_script_ctx(buf: *mut core::ffi::c_void);
    fn nvim_buf_save_and_clear_b_p_isk(buf: *mut core::ffi::c_void) -> *mut c_char;
    fn nvim_buf_restore_b_p_isk(buf: *mut core::ffi::c_void, saved: *mut c_char);
    fn nvim_buf_clear_b_p_ro(buf: *mut core::ffi::c_void);
    fn nvim_buf_get_b_p_bt_is_help(buf: *mut core::ffi::c_void) -> c_int;

    fn free_buf_options(buf: *mut core::ffi::c_void, free_flags: bool);
    fn check_buf_options(buf: *mut core::ffi::c_void);
    fn buf_init_chartab(buf: *mut core::ffi::c_void, global: c_int) -> c_int;
    fn nvim_call_compile_cap_prog_buf(buf: *mut core::ffi::c_void);

    fn nvim_call_tabstop_set_vsts(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_call_tabstop_set_vts(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_get_b_p_vts_array_is_null(buf: *mut core::ffi::c_void) -> c_int;

    fn set_buflocal_cpt_callbacks(buf: *mut core::ffi::c_void);
    fn set_buflocal_cfu_callback(buf: *mut core::ffi::c_void);
    fn set_buflocal_ofu_callback(buf: *mut core::ffi::c_void);
    fn rs_set_buflocal_tfu_callback(buf: *mut core::ffi::c_void);

    fn nvim_buf_kmap_state_set_init(buf: *mut core::ffi::c_void);
    fn clear_string_option(pp: *mut *mut c_char);

    fn nvim_buf_set_b_p_fenc_dup(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_bh_empty(buf: *mut core::ffi::c_void);
    static mut p_ff: *mut c_char;
    static mut p_ffs: *mut c_char;
    fn nvim_buf_set_b_p_bt_empty(buf: *mut core::ffi::c_void);

    // b_s substructure setters (not in the offset table):
    fn nvim_buf_set_b_p_vsts_nopaste_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_s_spc_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_s_spf_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_s_spl_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_s_spo_dup(buf: *mut core::ffi::c_void, s: *const c_char);
    fn nvim_buf_set_b_s_syn_isk_empty(buf: *mut core::ffi::c_void);

    // Global-local setters with flag side-effects (cannot use generic helper):
    fn nvim_buf_set_b_p_bkc_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_tc_empty(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_cot_empty(buf: *mut core::ffi::c_void);

    // Sentinel-value global-local setters:
    fn nvim_buf_set_b_p_ac_minus1(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_ar_minus1(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_ul_no_local(buf: *mut core::ffi::c_void);

    // Generic offset-based field writers:
    fn nvim_buf_set_string_field(buf: *mut core::ffi::c_void, offset: isize, s: *const c_char);
    fn nvim_buf_empty_string_field(buf: *mut core::ffi::c_void, offset: isize);
    fn nvim_buf_set_bool_field(buf: *mut core::ffi::c_void, offset: isize, val: c_int);
    fn nvim_buf_set_optint_field(buf: *mut core::ffi::c_void, offset: isize, val: OptInt);

    // Scalar field setters (variants / substructure not in offset table):
    fn nvim_buf_set_b_p_ai_nopaste(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_tw_nopaste(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_tw_nobin(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_wm_nopaste(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_wm_nobin(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_et_nobin(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_et_nopaste(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_ml(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_ml_nobin(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_sts_nopaste(buf: *mut core::ffi::c_void, v: OptInt);
    fn nvim_buf_set_b_p_iminsert(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_imsearch(buf: *mut core::ffi::c_void, v: c_int);
    fn nvim_buf_set_b_p_ma(buf: *mut core::ffi::c_void, v: c_int);

    fn nvim_buf_copy_opt_sctx(buf: *mut core::ffi::c_void, bv: c_int);
    fn nvim_buf_set_b_s_spo_flags_from_global(buf: *mut core::ffi::c_void);

    // Static local accessors (no global equivalent)
    fn nvim_get_p_ai_nopaste() -> bool;
    fn nvim_get_p_et_nobin() -> c_int;
    fn nvim_get_p_et_nopaste() -> bool;
    fn nvim_get_p_inf() -> c_int;
    fn nvim_get_p_ml_nobin() -> c_int;
    fn nvim_get_p_tw_nobin() -> OptInt;
    fn nvim_get_p_tw_nopaste() -> OptInt;
    fn nvim_get_p_wm_nobin() -> OptInt;
    fn nvim_get_p_wm_nopaste() -> OptInt;
    fn nvim_get_p_sts_nopaste() -> OptInt;
    fn nvim_get_p_vsts_nopaste() -> *const c_char;
}

// NUL character
const NUL: u8 = 0;

/// Check if a C string is non-null and non-empty.
unsafe fn cstr_nonempty(s: *const c_char) -> bool {
    !s.is_null() && (*s as u8) != NUL
}

/// Get the byte offset of a buf_T field for the given OptIndex.
///
/// # Panics
/// Panics (aborts) if `opt_idx` has no buf_T field (offset == -1).
fn field_offset(opt_idx: crate::index::OptIndex) -> isize {
    let offsets = crate::varp::buf_field_offsets();
    let idx = opt_idx as usize;
    let offset = offsets[idx];
    debug_assert!(offset >= 0, "no buf_T field for opt_idx {opt_idx}");
    offset
}

/// Perform the bulk mechanical copy of options from globals to buffer fields.
/// This mirrors all the `buf->b_p_X = p_X` and `COPY_OPT_SCTX` lines in
/// the original C `buf_copy_options`.
///
/// `dont_do_help`: if true, skip the help-buffer-specific fields (isk, ts, vts,
/// b_help, bt, ma). The b_p_isk save/restore is done by the caller.
#[allow(clippy::too_many_lines)]
unsafe fn do_bulk_copy(buf: *mut core::ffi::c_void, dont_do_help: bool) {
    nvim_buf_set_bool_field(buf, field_offset(K_OPT_AUTOINDENT), p_ai);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_AUTOINDENT);
    nvim_buf_set_b_p_ai_nopaste(buf, c_int::from(nvim_get_p_ai_nopaste()));

    nvim_buf_set_optint_field(buf, field_offset(K_OPT_SHIFTWIDTH), p_sw);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SHIFTWIDTH);

    nvim_buf_set_optint_field(buf, field_offset(K_OPT_SCROLLBACK), p_scbk);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SCROLLBACK);

    nvim_buf_set_optint_field(buf, field_offset(K_OPT_TEXTWIDTH), p_tw);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_TEXTWIDTH);
    nvim_buf_set_b_p_tw_nopaste(buf, nvim_get_p_tw_nopaste());
    nvim_buf_set_b_p_tw_nobin(buf, nvim_get_p_tw_nobin());

    nvim_buf_set_optint_field(buf, field_offset(K_OPT_WRAPMARGIN), p_wm);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_WRAPMARGIN);
    nvim_buf_set_b_p_wm_nopaste(buf, nvim_get_p_wm_nopaste());
    nvim_buf_set_b_p_wm_nobin(buf, nvim_get_p_wm_nobin());

    nvim_buf_set_bool_field(buf, field_offset(K_OPT_BINARY), c_int::from(p_bin != 0));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_BINARY);

    nvim_buf_set_bool_field(buf, field_offset(K_OPT_BOMB), c_int::from(p_bomb != 0));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_BOMB);

    nvim_buf_set_bool_field(buf, field_offset(K_OPT_EXPANDTAB), c_int::from(p_et != 0));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_EXPANDTAB);

    nvim_buf_set_bool_field(
        buf,
        field_offset(K_OPT_FIXENDOFLINE),
        c_int::from(p_fixeol != 0),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_FIXENDOFLINE);

    nvim_buf_set_b_p_et_nobin(buf, nvim_get_p_et_nobin());
    nvim_buf_set_b_p_et_nopaste(buf, c_int::from(nvim_get_p_et_nopaste()));

    nvim_buf_set_b_p_ml(buf, p_ml);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_MODELINE);
    nvim_buf_set_b_p_ml_nobin(buf, nvim_get_p_ml_nobin());

    nvim_buf_set_bool_field(buf, field_offset(K_OPT_INFERCASE), nvim_get_p_inf());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_INFERCASE);

    // swapfile: suppress if :noswapfile modifier is active
    if nvim_cmdmod_get_cmod_flags() & CMOD_NOSWAPFILE != 0 {
        nvim_buf_set_bool_field(buf, field_offset(K_OPT_SWAPFILE), 0);
    } else {
        nvim_buf_set_bool_field(buf, field_offset(K_OPT_SWAPFILE), c_int::from(p_swf != 0));
        nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SWAPFILE);
    }

    nvim_buf_set_string_field(buf, field_offset(K_OPT_COMPLETE), crate::p_cpt.cast_const());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_COMPLETE);
    set_buflocal_cpt_callbacks(buf);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_COMPLETEFUNC),
        crate::p_cfu.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_COMPLETEFUNC);
    set_buflocal_cfu_callback(buf);

    nvim_buf_set_string_field(buf, field_offset(K_OPT_OMNIFUNC), crate::p_ofu.cast_const());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_OMNIFUNC);
    set_buflocal_ofu_callback(buf);

    nvim_buf_set_string_field(buf, field_offset(K_OPT_TAGFUNC), crate::p_tfu.cast_const());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_TAGFUNC);
    rs_set_buflocal_tfu_callback(buf);

    nvim_buf_set_optint_field(buf, field_offset(K_OPT_SOFTTABSTOP), p_sts);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SOFTTABSTOP);
    nvim_buf_set_b_p_sts_nopaste(buf, nvim_get_p_sts_nopaste());

    let p_vsts = crate::p_vsts.cast_const();
    nvim_buf_set_string_field(buf, field_offset(K_OPT_VARSOFTTABSTOP), p_vsts);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_VARSOFTTABSTOP);
    if cstr_nonempty(p_vsts) {
        nvim_call_tabstop_set_vsts(buf, p_vsts);
    } else {
        // buf->b_p_vsts_array = NULL (tabstop_set with null does nothing;
        // free_buf_options already zeroed it, but call with null to be safe)
        nvim_call_tabstop_set_vsts(buf, core::ptr::null());
    }
    nvim_buf_set_b_p_vsts_nopaste_dup(buf, nvim_get_p_vsts_nopaste());

    nvim_buf_set_string_field(buf, field_offset(K_OPT_COMMENTS), crate::p_com.cast_const());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_COMMENTS);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_COMMENTSTRING),
        crate::p_cms.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_COMMENTSTRING);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_FORMATOPTIONS),
        crate::p_fo.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_FORMATOPTIONS);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_FORMATLISTPAT),
        crate::p_flp.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_FORMATLISTPAT);

    nvim_buf_set_string_field(buf, field_offset(K_OPT_NRFORMATS), crate::p_nf.cast_const());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_NRFORMATS);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_MATCHPAIRS),
        crate::p_mps.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_MATCHPAIRS);

    nvim_buf_set_bool_field(buf, field_offset(K_OPT_SMARTINDENT), c_int::from(p_si != 0));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SMARTINDENT);

    nvim_buf_set_optint_field(buf, field_offset(K_OPT_CHANNEL), 0);

    nvim_buf_set_bool_field(buf, field_offset(K_OPT_COPYINDENT), c_int::from(p_ci != 0));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_COPYINDENT);

    nvim_buf_set_bool_field(buf, field_offset(K_OPT_CINDENT), c_int::from(p_cin != 0));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_CINDENT);

    nvim_buf_set_string_field(buf, field_offset(K_OPT_CINKEYS), crate::p_cink.cast_const());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_CINKEYS);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_CINOPTIONS),
        crate::p_cino.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_CINOPTIONS);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_CINSCOPEDECLS),
        crate::p_cinsd.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_CINSCOPEDECLS);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_LISPOPTIONS),
        crate::p_lop.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_LISPOPTIONS);

    // Don't copy 'filetype' - it must be detected
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_FILETYPE));

    nvim_buf_set_bool_field(
        buf,
        field_offset(K_OPT_PRESERVEINDENT),
        c_int::from(p_pi != 0),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_PRESERVEINDENT);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_CINWORDS),
        crate::p_cinw.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_CINWORDS);

    nvim_buf_set_bool_field(buf, field_offset(K_OPT_LISP), c_int::from(p_lisp != 0));
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_LISP);

    // Don't copy 'syntax' - it must be set
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_SYNTAX));

    nvim_buf_set_optint_field(buf, field_offset(K_OPT_SYNMAXCOL), p_smc);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SYNMAXCOL);

    nvim_buf_set_b_s_syn_isk_empty(buf);

    nvim_buf_set_b_s_spc_dup(buf, crate::p_spc.cast_const());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SPELLCAPCHECK);
    nvim_call_compile_cap_prog_buf(buf);

    nvim_buf_set_b_s_spf_dup(buf, crate::p_spf.cast_const());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SPELLFILE);

    nvim_buf_set_b_s_spl_dup(buf, crate::p_spl.cast_const());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SPELLLANG);

    nvim_buf_set_b_s_spo_dup(buf, crate::p_spo.cast_const());
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SPELLOPTIONS);
    nvim_buf_set_b_s_spo_flags_from_global(buf);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_INDENTEXPR),
        crate::p_inde.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_INDENTEXPR);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_INDENTKEYS),
        crate::p_indk.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_INDENTKEYS);

    // Don't copy 'formatprg' - no local value by default
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_FORMATPRG));

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_FORMATEXPR),
        crate::p_fex.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_FORMATEXPR);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_SUFFIXESADD),
        crate::p_sua.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_SUFFIXESADD);

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_KEYMAP),
        crate::p_keymap.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_KEYMAP);
    nvim_buf_kmap_state_set_init(buf);

    // Langmap/IME state: copy from current buffer is better than resetting
    // iminsert/imsearch are OptInt globals but b_p_iminsert/b_p_imsearch are int fields
    #[allow(clippy::cast_possible_truncation)]
    nvim_buf_set_b_p_iminsert(buf, p_iminsert as c_int);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_IMINSERT);
    #[allow(clippy::cast_possible_truncation)]
    nvim_buf_set_b_p_imsearch(buf, p_imsearch as c_int);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_IMSEARCH);

    // Global-local options: use global value (no local copy)
    nvim_buf_set_b_p_ac_minus1(buf);
    nvim_buf_set_b_p_ar_minus1(buf);
    nvim_buf_set_b_p_ul_no_local(buf);
    nvim_buf_set_b_p_bkc_empty(buf);
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_GREPFORMAT));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_GREPPRG));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_MAKEPRG));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_ERRORFORMAT));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_EQUALPRG));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_FINDFUNC));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_KEYWORDPRG));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_PATH));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_TAGS));
    nvim_buf_set_b_p_tc_empty(buf);
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_DEFINE));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_INCLUDE));

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_INCLUDEEXPR),
        crate::p_inex.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_INCLUDEEXPR);

    nvim_buf_set_b_p_cot_empty(buf);
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_DICTIONARY));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_DIFFANCHORS));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_THESAURUS));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_THESAURUSFUNC));

    nvim_buf_set_string_field(
        buf,
        field_offset(K_OPT_QUOTEESCAPE),
        crate::p_qe.cast_const(),
    );
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_QUOTEESCAPE);

    nvim_buf_set_bool_field(buf, field_offset(K_OPT_UNDOFILE), p_udf);
    nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_UNDOFILE);

    nvim_buf_empty_string_field(buf, field_offset(K_OPT_LISPWORDS));
    nvim_buf_empty_string_field(buf, field_offset(K_OPT_MAKEENCODING));

    // Help-specific options: iskeyword, tabstop, vartabstop, b_help, buftype, modifiable
    if dont_do_help {
        // b_p_isk was saved and NULLed by caller; here we handle vts_array only
        let vts_global = crate::p_vts.cast_const();
        if cstr_nonempty(vts_global) && nvim_buf_get_b_p_vts_array_is_null(buf) != 0 {
            nvim_call_tabstop_set_vts(buf, vts_global);
        } else {
            nvim_call_tabstop_set_vts(buf, core::ptr::null());
        }
    } else {
        nvim_buf_set_string_field(
            buf,
            field_offset(K_OPT_ISKEYWORD),
            crate::p_isk.cast_const(),
        );
        nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_ISKEYWORD);

        nvim_buf_set_optint_field(buf, field_offset(K_OPT_TABSTOP), p_ts);
        nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_TABSTOP);

        nvim_buf_set_string_field(
            buf,
            field_offset(K_OPT_VARTABSTOP),
            crate::p_vts.cast_const(),
        );
        nvim_buf_copy_opt_sctx(buf, K_BUF_OPT_VARTABSTOP);

        let vts_global = crate::p_vts.cast_const();
        if cstr_nonempty(vts_global) && nvim_buf_get_b_p_vts_array_is_null(buf) != 0 {
            nvim_call_tabstop_set_vts(buf, vts_global);
        } else {
            nvim_call_tabstop_set_vts(buf, core::ptr::null());
        }

        nvim_buf_set_b_help(buf, 0);

        if nvim_buf_get_b_p_bt_is_help(buf) != 0 {
            // clear b_p_bt: xfree(b_p_bt); b_p_bt = empty_string_option
            // The field is properly aligned in buf_T; suppress alignment lint.
            #[allow(clippy::cast_ptr_alignment)]
            let bt_ptr = buf
                .cast::<u8>()
                .offset(field_offset(K_OPT_BUFTYPE))
                .cast::<*mut c_char>();
            clear_string_option(bt_ptr);
        }

        nvim_buf_set_b_p_ma(buf, p_ma);
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
#[export_name = "buf_copy_options"]
pub unsafe extern "C" fn rs_buf_copy_options(buf: *mut core::ffi::c_void, flags: c_int) {
    let p_cpo = crate::p_cpo.cast_const();
    // Skip when option defaults have not been set yet (first buffer allocation).
    if p_cpo.is_null() {
        check_buf_options(buf);
        return;
    }

    let bco_enter = BCO_ENTER;
    let bco_always = BCO_ALWAYS;
    let bco_nohelp = BCO_NOHELP;
    let cpo_s = CPO_BUFOPT;
    let cpo_cap_s = CPO_BUFOPTGLOB;

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
    let should_copy = !((vim_strchr(p_cpo, cpo_cap_s).is_null() || (flags & bco_enter) == 0)
        && (initialized || ((flags & bco_enter) == 0 && !vim_strchr(p_cpo, cpo_s).is_null())));

    if should_copy || (flags & bco_always) != 0 {
        nvim_buf_clear_b_p_script_ctx(buf);

        // Don't copy the options specific to a help buffer when BCO_NOHELP is
        // given or the options were initialized already (jumping back to a help
        // file with CTRL-T or CTRL-O).
        let dont_do_help =
            ((flags & bco_nohelp) != 0 && nvim_buf_get_help(buf) != 0) || initialized;

        // If dont_do_help, save b_p_isk before free_buf_options
        let save_p_isk: *mut c_char = if dont_do_help {
            nvim_buf_save_and_clear_b_p_isk(buf)
        } else {
            core::ptr::null_mut()
        };

        // Free old allocated strings; initialize some fields if first time
        if initialized {
            free_buf_options(buf, false);
        } else {
            free_buf_options(buf, true);
            nvim_buf_clear_b_p_ro(buf);
            nvim_buf_set_b_p_fenc_dup(buf);
            // Set b_p_ff from first char of p_ffs, falling back to p_ff.
            let ff_str: *const c_char = match (*p_ffs) as u8 {
                b'm' => c"mac".as_ptr(),
                b'd' => c"dos".as_ptr(),
                b'u' => c"unix".as_ptr(),
                _ => p_ff,
            };
            nvim_buf_set_string_field(buf, field_offset(K_OPT_FILEFORMAT), ff_str);
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

    check_buf_options(buf);

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
        ((flags & bco_nohelp) != 0 && nvim_buf_get_help(buf) != 0) || initialized;
    let did_isk = did_copy && !dont_do_help_recomputed;

    if did_isk {
        buf_init_chartab(buf, 0); // 0 = false (not global)
    }
}
