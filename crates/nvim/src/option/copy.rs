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

use core::ffi::{CStr, c_char, c_int, c_uint};
use core::mem::offset_of;
use core::ptr;

use crate::buffer::free_buf_options;
use crate::charset::buf_init_chartab;
use crate::ex_docmd::cmdmod_has;
use crate::indent::{briopt_check, tabstop_set};
use crate::insexpand::{
    set_buflocal_cfu_callback, set_buflocal_cpt_callbacks, set_buflocal_ofu_callback,
};
use crate::main::{
    p_ai, p_bin, p_bomb, p_cfu, p_ci, p_cin, p_cink, p_cino, p_cinsd, p_cinw, p_cms, p_com, p_cpo,
    p_cpt, p_et, p_fenc, p_fex, p_ff, p_ffs, p_fixeol, p_flp, p_fo, p_iminsert, p_imsearch, p_inde,
    p_indk, p_inex, p_inf, p_isk, p_keymap, p_lisp, p_lop, p_ma, p_ml, p_mps, p_nf, p_ofu, p_pi,
    p_qe, p_scbk, p_si, p_smc, p_spc, p_spf, p_spl, p_spo, p_sts, p_sua, p_sw, p_swf, p_tfu, p_ts,
    p_tw, p_udf, p_vsts, p_vts, p_wm, spo_flags,
};
use crate::memory::xstrdup;

use super::check::{p_et_nobin, p_ml_nobin, p_tw_nobin, p_wm_nobin};
use super::paste::{
    p_ai_nopaste, p_et_nopaste, p_sts_nopaste, p_tw_nopaste, p_vsts_nopaste, p_wm_nopaste,
};
use crate::global_cell::GlobalCell;
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
    check_buf_options, check_signcolumn, check_string_option, clear_string_option, empty_option,
};
use crate::spell::compile_cap_prog;
use crate::tag::set_buflocal_tfu_callback;
use crate::types::{CmdModFlags, CpoFlag, NUL, OptInt, buf_T, colnr_T, int16_t, win_T, winopt_T};
use crate::window::{check_colorcolumn, set_winbar_win};
use crate::winlayer::{Buf, Live, Win};

use super::{
    BCO_ALWAYS, BCO_ENTER, BCO_NOHELP, KEYMAP_INIT, NO_LOCAL_UNDOLEVEL, boolean_optval,
    change_option_default, check_blending, fill_culopt_flags, kFillchars, kListchars,
    option_last_set, parse_winhl_opt, set_chars_option,
};
use crate::option::cpo_has;

/// One window's set of option values, whose caller has promised it outlives
/// the handle. Construction is the unsafe step; every field access after it
/// is ordinary checked code, and the borrow it hands out lasts only as long
/// as the access that asked for it.
type Wop = Live<winopt_T>;

/// The string every unset string option shares.
fn unset_string() -> *mut c_char {
    empty_option()
}

/// The address of one field of the buffer `$buf` points at, computed rather
/// than read: see [`super::field_ptr`]. The `|b: &buf_T|` argument is never
/// called; it is what ties the answer's type to the field's declaration.
macro_rules! buf_field {
    ($buf:expr, $($field:ident).+) => {
        super::field_ptr(
            $buf,
            offset_of!(buf_T, $($field).+),
            |b: &buf_T| &b.$($field).+,
        )
    };
}

/// [`buf_field`] for a window's set of option values.
macro_rules! wop_field {
    ($wop:expr, $($field:ident).+) => {
        super::field_ptr(
            $wop,
            offset_of!(winopt_T, $($field).+),
            |w: &winopt_T| &w.$($field).+,
        )
    };
}

/// [`buf_field`] for a window.
macro_rules! win_field {
    ($win:expr, $($field:ident).+) => {
        super::field_ptr(
            $win,
            offset_of!(win_T, $($field).+),
            |w: &win_T| &w.$($field).+,
        )
    };
}

/// A duplicate of a global string option's value.
///
/// Every string option holds a live NUL-terminated string from the moment
/// the defaults are set, so naming the option's cell *is* `xstrdup`'s whole
/// precondition — which is why this takes the cell rather than a pointer,
/// and why the promise is paid once here instead of at each of the thirty
/// fields below.
fn dup_global(cell: &GlobalCell<*mut c_char>) -> *mut c_char {
    // SAFETY: a string option's value is a live NUL-terminated string.
    unsafe { xstrdup(cell.get()) }
}

/// [`dup_global`] for one of the compiled-in names.
fn dup_static(name: &CStr) -> *mut c_char {
    // SAFETY: a `CStr` is NUL-terminated by construction.
    unsafe { xstrdup(name.as_ptr()) }
}

/// Give a window's option values to a freshly split one.
///
/// # Safety
///
/// Both windows must be live, and `wp_to`'s option fields uninitialised or
/// already released.
pub(crate) unsafe fn win_copy_options(wp_from: *mut win_T, wp_to: *mut win_T) {
    // SAFETY: the caller's windows; naming a field of one reads nothing,
    // so the four addresses below are ordinary checked code.
    unsafe {
        copy_winopt(
            win_field!(wp_from, w_onebuf_opt),
            win_field!(wp_to, w_onebuf_opt),
        );
        copy_winopt(
            win_field!(wp_from, w_allbuf_opt),
            win_field!(wp_to, w_allbuf_opt),
        );
        didset_window_options(wp_to, true);
    };
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
pub(crate) unsafe fn copy_winopt(from: *mut winopt_T, to: *mut winopt_T) {
    // SAFETY: the caller's structures. Both handles borrow for the one
    // field access that asked and never across a call, so neither ever
    // holds a `&mut winopt_T` the editor could read around.
    let f = unsafe { Wop::new(from) };
    let mut t = unsafe { Wop::new(to) };

    t.wo_arab = f.wo_arab;
    t.wo_list = f.wo_list;
    t.wo_lcs = unsafe { copy_option_val(f.wo_lcs) };
    t.wo_fcs = unsafe { copy_option_val(f.wo_fcs) };
    t.wo_nu = f.wo_nu;
    t.wo_rnu = f.wo_rnu;
    t.wo_ve = unsafe { copy_option_val(f.wo_ve) };
    t.wo_ve_flags = f.wo_ve_flags;
    t.wo_nuw = f.wo_nuw;
    t.wo_rl = f.wo_rl;
    t.wo_rlc = unsafe { copy_option_val(f.wo_rlc) };
    t.wo_sbr = unsafe { copy_option_val(f.wo_sbr) };
    t.wo_stl = unsafe { copy_option_val(f.wo_stl) };
    t.wo_wbr = unsafe { copy_option_val(f.wo_wbr) };
    t.wo_wrap = f.wo_wrap;
    t.wo_wrap_save = f.wo_wrap_save;
    t.wo_lbr = f.wo_lbr;
    t.wo_bri = f.wo_bri;
    t.wo_briopt = unsafe { copy_option_val(f.wo_briopt) };
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
    t.wo_culopt = unsafe { copy_option_val(f.wo_culopt) };
    t.wo_cc = unsafe { copy_option_val(f.wo_cc) };
    t.wo_diff = f.wo_diff;
    t.wo_diff_saved = f.wo_diff_saved;
    t.wo_eiw = unsafe { copy_option_val(f.wo_eiw) };
    t.wo_cocu = unsafe { copy_option_val(f.wo_cocu) };
    t.wo_cole = f.wo_cole;
    t.wo_fdc = unsafe { copy_option_val(f.wo_fdc) };
    // The four `_save` copies only hold anything while `:diffthis` is
    // in effect; otherwise they are the unset string, not a value to
    // duplicate.
    t.wo_fdc_save = if f.wo_diff_saved != 0 {
        unsafe { xstrdup(f.wo_fdc_save) }
    } else {
        unset_string()
    };
    t.wo_fen = f.wo_fen;
    t.wo_fen_save = f.wo_fen_save;
    t.wo_fdi = unsafe { copy_option_val(f.wo_fdi) };
    t.wo_fml = f.wo_fml;
    t.wo_fdl = f.wo_fdl;
    t.wo_fdl_save = f.wo_fdl_save;
    t.wo_fdm = unsafe { copy_option_val(f.wo_fdm) };
    t.wo_fdm_save = if f.wo_diff_saved != 0 {
        unsafe { xstrdup(f.wo_fdm_save) }
    } else {
        unset_string()
    };
    t.wo_fdn = f.wo_fdn;
    t.wo_fde = unsafe { copy_option_val(f.wo_fde) };
    t.wo_fdt = unsafe { copy_option_val(f.wo_fdt) };
    t.wo_fmr = unsafe { copy_option_val(f.wo_fmr) };
    t.wo_scl = unsafe { copy_option_val(f.wo_scl) };
    t.wo_lhi = f.wo_lhi;
    t.wo_winhl = unsafe { copy_option_val(f.wo_winhl) };
    t.wo_winbl = f.wo_winbl;
    t.wo_stc = unsafe { copy_option_val(f.wo_stc) };
    t.wo_wrap_flags = f.wo_wrap_flags;
    t.wo_stl_flags = f.wo_stl_flags;
    t.wo_wbr_flags = f.wo_wbr_flags;
    t.wo_fde_flags = f.wo_fde_flags;
    t.wo_fdt_flags = f.wo_fdt_flags;
    t.wo_script_ctx = f.wo_script_ctx;

    unsafe { check_winopt(to) };
}

/// The window-local string options, as the address of each field. Naming a
/// field reads nothing, so this needs no promise of its own; what the two
/// callers then *do* with the addresses does.
fn winopt_strings(wop: *mut winopt_T) -> [*mut *mut c_char; 23] {
    [
        wop_field!(wop, wo_fdc),
        wop_field!(wop, wo_fdc_save),
        wop_field!(wop, wo_fdi),
        wop_field!(wop, wo_fdm),
        wop_field!(wop, wo_fdm_save),
        wop_field!(wop, wo_fde),
        wop_field!(wop, wo_fdt),
        wop_field!(wop, wo_fmr),
        wop_field!(wop, wo_eiw),
        wop_field!(wop, wo_scl),
        wop_field!(wop, wo_rlc),
        wop_field!(wop, wo_sbr),
        wop_field!(wop, wo_stl),
        wop_field!(wop, wo_culopt),
        wop_field!(wop, wo_cc),
        wop_field!(wop, wo_cocu),
        wop_field!(wop, wo_briopt),
        wop_field!(wop, wo_winhl),
        wop_field!(wop, wo_lcs),
        wop_field!(wop, wo_fcs),
        wop_field!(wop, wo_ve),
        wop_field!(wop, wo_wbr),
        wop_field!(wop, wo_stc),
    ]
}

/// Give a window's two option sets any string values they are still missing.
///
/// # Safety
///
/// `win` must be a live window.
pub(crate) unsafe fn check_win_options(win: *mut win_T) {
    // SAFETY: the caller's window, whose two option sets the addresses
    // below name without reading it.
    unsafe { check_winopt(win_field!(win, w_onebuf_opt)) };
    unsafe { check_winopt(win_field!(win, w_allbuf_opt)) };
}

/// Replace any null string value with the shared unset string.
///
/// # Safety
///
/// `wop` must point at a `winopt_T`.
pub(crate) unsafe fn check_winopt(wop: *mut winopt_T) {
    // SAFETY: the caller's structure, and each address is one of its own
    // string fields.
    for field in winopt_strings(wop) {
        unsafe { check_string_option(field) };
    }
}

/// Release every string value a window's option set owns.
///
/// # Safety
///
/// `wop` must point at a `winopt_T` whose string values are its own.
pub(crate) unsafe fn clear_winopt(wop: *mut winopt_T) {
    // SAFETY: the caller's structure, and each address is one of its own
    // string fields.
    for field in winopt_strings(wop) {
        unsafe { clear_string_option(field) };
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
pub(crate) unsafe fn didset_window_options(wp: *mut win_T, valid_cursor: bool) {
    // SAFETY: the caller's window. The handle borrows it for the one field
    // access that asked and never across a call, so none of the callees
    // below is reached while a `&mut win_T` is live.
    let mut w = unsafe { Win::new(wp) };
    // 'wrap' and 'smoothscroll' scroll in different directions, and only
    // one of the two offsets can be non-zero.
    if w.w_onebuf_opt.wo_wrap != 0 {
        w.w_leftcol = 0 as colnr_T;
    } else {
        w.w_skipcol = 0 as colnr_T;
    }
    let no_err: *mut c_char = ptr::null_mut();
    // SAFETY: the caller's window, which is all any of these needs; the
    // null out-parameters say "report nothing", which each accepts.
    unsafe {
        check_colorcolumn(ptr::null_mut(), wp);
        briopt_check(ptr::null_mut(), wp);
        fill_culopt_flags(None, wp);
    }
    // Read each value where it is used: the calls above parse other
    // options and this one must see whatever they left behind.
    let fcs = w.w_onebuf_opt.wo_fcs;
    // SAFETY: as above; 'fillchars' and 'listchars' are string options.
    unsafe { set_chars_option(wp, fcs, kFillchars, true, no_err, 0) };
    let lcs = w.w_onebuf_opt.wo_lcs;
    unsafe { set_chars_option(wp, lcs, kListchars, true, no_err, 0) };
    // SAFETY: the caller's window.
    unsafe {
        parse_winhl_opt(ptr::null(), wp);
        check_blending(wp);
        set_winbar_win(wp, false, valid_cursor);
        check_signcolumn(ptr::null_mut(), wp);
    }
    w.w_grid_alloc.blending = w.w_onebuf_opt.wo_winbl > 0 as OptInt;
}

/// Attribute a buffer-local option to whatever script set the global value
/// it was just copied from.
///
/// `buf_opt_idx` maps every buffer-local row to a row of the option table.
fn copy_sctx(mut buf: Buf, bv: BufOptIndex) {
    let opt_idx = buf_opt_idx[bv as usize];
    buf.b_p_script_ctx[bv as usize] = option_last_set(opt_idx);
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
pub(crate) unsafe fn buf_copy_options(buf: *mut buf_T, flags: c_int) {
    let mut did_isk = false;
    // SAFETY: the caller's buffer. Every field write below goes through
    // this handle, which borrows the buffer for the one access that asked
    // and never across a call.
    let mut b = unsafe { Buf::new(buf) };

    // Before the defaults exist there is nothing to copy: `main` makes
    // the first buffer that early.
    if p_cpo.get().is_null() {
        unsafe { check_buf_options(buf) };
        return;
    }

    let entering = flags & BCO_ENTER as c_int != 0;
    let keep_global = !cpo_has(CpoFlag::BUFOPTGLOB) || !entering;
    let keep_local = b.b_p_initialized || (!entering && cpo_has(CpoFlag::BUFOPT));
    let should_copy = !(keep_global && keep_local);

    if should_copy || flags & BCO_ALWAYS as c_int != 0 {
        b.b_p_script_ctx = unsafe { core::mem::zeroed() };

        // A help buffer keeps its own settings when it already has them
        // — jumping back to one with CTRL-T or CTRL-O must not reset it.
        let dont_do_help = (flags & BCO_NOHELP as c_int != 0 && b.b_help) || b.b_p_initialized;
        // 'iskeyword' is the one string that survives the free below.
        let save_p_isk = if dont_do_help {
            let saved = b.b_p_isk;
            b.b_p_isk = ptr::null_mut();
            saved
        } else {
            ptr::null_mut()
        };

        if b.b_p_initialized {
            unsafe { free_buf_options(Buf::new(buf), false) };
        } else {
            unsafe { free_buf_options(Buf::new(buf), true) };
            b.b_p_ro = 0;
            b.b_p_fenc = dup_global(&p_fenc);
            // A new buffer takes the *first* of 'fileformats' rather
            // than 'fileformat', since nothing has been read yet.
            b.b_p_ff = match unsafe { *p_ffs.get() } as u8 {
                b'm' => dup_static(c"mac"),
                b'd' => dup_static(c"dos"),
                b'u' => dup_static(c"unix"),
                _ => dup_global(&p_ff),
            };
            b.b_p_bh = unset_string();
            b.b_p_bt = unset_string();
        }

        b.b_p_ai = p_ai.get();
        copy_sctx(b, kBufOptAutoindent);
        b.b_p_ai_nopaste = p_ai_nopaste.get();
        b.b_p_sw = p_sw.get();
        copy_sctx(b, kBufOptShiftwidth);
        b.b_p_scbk = p_scbk.get();
        copy_sctx(b, kBufOptScrollback);
        b.b_p_tw = p_tw.get();
        copy_sctx(b, kBufOptTextwidth);
        b.b_p_tw_nopaste = p_tw_nopaste.get();
        b.b_p_tw_nobin = p_tw_nobin.get();
        b.b_p_wm = p_wm.get();
        copy_sctx(b, kBufOptWrapmargin);
        b.b_p_wm_nopaste = p_wm_nopaste.get();
        b.b_p_wm_nobin = p_wm_nobin.get();
        b.b_p_bin = p_bin.get();
        copy_sctx(b, kBufOptBinary);
        b.b_p_bomb = p_bomb.get();
        copy_sctx(b, kBufOptBomb);
        b.b_p_et = p_et.get();
        copy_sctx(b, kBufOptExpandtab);
        b.b_p_fixeol = p_fixeol.get();
        copy_sctx(b, kBufOptFixendofline);
        b.b_p_et_nobin = p_et_nobin.get();
        b.b_p_et_nopaste = p_et_nopaste.get();
        b.b_p_ml = p_ml.get();
        copy_sctx(b, kBufOptModeline);
        b.b_p_ml_nobin = p_ml_nobin.get();
        b.b_p_inf = p_inf.get();
        copy_sctx(b, kBufOptInfercase);

        // `:noswapfile` wins over the global 'swapfile', and leaves the
        // script context alone because nothing set it.
        if cmdmod_has(CmdModFlags::NOSWAPFILE) {
            b.b_p_swf = 0;
        } else {
            b.b_p_swf = p_swf.get();
            copy_sctx(b, kBufOptSwapfile);
        }

        b.b_p_cpt = dup_global(&p_cpt);
        copy_sctx(b, kBufOptComplete);
        set_buflocal_cpt_callbacks(b);
        b.b_p_cfu = dup_global(&p_cfu);
        copy_sctx(b, kBufOptCompletefunc);
        set_buflocal_cfu_callback(b);
        b.b_p_ofu = dup_global(&p_ofu);
        copy_sctx(b, kBufOptOmnifunc);
        set_buflocal_ofu_callback(b);
        b.b_p_tfu = dup_global(&p_tfu);
        copy_sctx(b, kBufOptTagfunc);
        set_buflocal_tfu_callback(b);

        b.b_p_sts = p_sts.get();
        copy_sctx(b, kBufOptSofttabstop);
        b.b_p_sts_nopaste = p_sts_nopaste.get();
        b.b_p_vsts = dup_global(&p_vsts);
        copy_sctx(b, kBufOptVarsofttabstop);
        b.b_p_vsts_array = if !p_vsts.get().is_null() && p_vsts.get() != unset_string() {
            // SAFETY: 'vartabstop' is a non-empty string option value.
            unsafe { tabstop_array(p_vsts.get()) }
        } else {
            ptr::null_mut()
        };
        b.b_p_vsts_nopaste = if p_vsts_nopaste.get().is_null() {
            ptr::null_mut()
        } else {
            dup_global(&p_vsts_nopaste)
        };

        b.b_p_com = dup_global(&p_com);
        copy_sctx(b, kBufOptComments);
        b.b_p_cms = dup_global(&p_cms);
        copy_sctx(b, kBufOptCommentstring);
        b.b_p_fo = dup_global(&p_fo);
        copy_sctx(b, kBufOptFormatoptions);
        b.b_p_flp = dup_global(&p_flp);
        copy_sctx(b, kBufOptFormatlistpat);
        b.b_p_nf = dup_global(&p_nf);
        copy_sctx(b, kBufOptNrformats);
        b.b_p_mps = dup_global(&p_mps);
        copy_sctx(b, kBufOptMatchpairs);
        b.b_p_si = p_si.get();
        copy_sctx(b, kBufOptSmartindent);
        b.b_p_channel = 0 as OptInt;
        b.b_p_ci = p_ci.get();
        copy_sctx(b, kBufOptCopyindent);
        b.b_p_cin = p_cin.get();
        copy_sctx(b, kBufOptCindent);
        b.b_p_cink = dup_global(&p_cink);
        copy_sctx(b, kBufOptCinkeys);
        b.b_p_cino = dup_global(&p_cino);
        copy_sctx(b, kBufOptCinoptions);
        b.b_p_cinsd = dup_global(&p_cinsd);
        copy_sctx(b, kBufOptCinscopedecls);
        b.b_p_lop = dup_global(&p_lop);
        copy_sctx(b, kBufOptLispoptions);
        // 'filetype' and 'syntax' start empty: the autocommands that
        // set them have not run for this buffer yet.
        b.b_p_ft = unset_string();
        b.b_p_pi = p_pi.get();
        copy_sctx(b, kBufOptPreserveindent);
        b.b_p_cinw = dup_global(&p_cinw);
        copy_sctx(b, kBufOptCinwords);
        b.b_p_lisp = p_lisp.get();
        copy_sctx(b, kBufOptLisp);
        b.b_p_syn = unset_string();
        b.b_p_smc = p_smc.get();
        copy_sctx(b, kBufOptSynmaxcol);

        b.b_s.b_syn_isk = unset_string();
        b.b_s.b_p_spc = dup_global(&p_spc);
        copy_sctx(b, kBufOptSpellcapcheck);
        // SAFETY: `b_s` is the buffer's own syntax block.
        unsafe { compile_cap_prog(buf_field!(buf, b_s)) };
        b.b_s.b_p_spf = dup_global(&p_spf);
        copy_sctx(b, kBufOptSpellfile);
        b.b_s.b_p_spl = dup_global(&p_spl);
        copy_sctx(b, kBufOptSpelllang);
        b.b_s.b_p_spo = dup_global(&p_spo);
        copy_sctx(b, kBufOptSpelloptions);
        b.b_s.b_p_spo_flags = spo_flags.get();

        b.b_p_inde = dup_global(&p_inde);
        copy_sctx(b, kBufOptIndentexpr);
        b.b_p_indk = dup_global(&p_indk);
        copy_sctx(b, kBufOptIndentkeys);
        b.b_p_fp = unset_string();
        b.b_p_fex = dup_global(&p_fex);
        copy_sctx(b, kBufOptFormatexpr);
        b.b_p_sua = dup_global(&p_sua);
        copy_sctx(b, kBufOptSuffixesadd);
        b.b_p_keymap = dup_global(&p_keymap);
        copy_sctx(b, kBufOptKeymap);
        b.b_kmap_state = (b.b_kmap_state as c_int | KEYMAP_INIT) as int16_t;
        b.b_p_iminsert = p_iminsert.get();
        copy_sctx(b, kBufOptIminsert);
        b.b_p_imsearch = p_imsearch.get();
        copy_sctx(b, kBufOptImsearch);

        // The global-local options start unset, reading through to the
        // global value.
        b.b_p_ac = -1;
        b.b_p_ar = -1;
        b.b_p_fs = -1;
        b.b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;
        for field in [
            buf_field!(buf, b_p_bkc),
            buf_field!(buf, b_p_gefm),
            buf_field!(buf, b_p_gp),
            buf_field!(buf, b_p_mp),
            buf_field!(buf, b_p_efm),
            buf_field!(buf, b_p_ep),
            buf_field!(buf, b_p_ffu),
            buf_field!(buf, b_p_kp),
            buf_field!(buf, b_p_path),
            buf_field!(buf, b_p_tags),
            buf_field!(buf, b_p_tc),
            buf_field!(buf, b_p_def),
            buf_field!(buf, b_p_inc),
            buf_field!(buf, b_p_cot),
            buf_field!(buf, b_p_dict),
            buf_field!(buf, b_p_dia),
            buf_field!(buf, b_p_tsr),
            buf_field!(buf, b_p_tsrfu),
            buf_field!(buf, b_p_lw),
            buf_field!(buf, b_p_menc),
        ] {
            unsafe { *field = unset_string() };
        }
        b.b_bkc_flags = 0 as c_uint;
        b.b_tc_flags = 0 as c_uint;
        b.b_cot_flags = 0 as c_uint;
        // 'includeexpr' is buffer-local only, not global-local.
        b.b_p_inex = dup_global(&p_inex);
        copy_sctx(b, kBufOptIncludeexpr);
        b.b_p_qe = dup_global(&p_qe);
        copy_sctx(b, kBufOptQuoteescape);
        b.b_p_udf = p_udf.get();
        copy_sctx(b, kBufOptUndofile);

        if dont_do_help {
            b.b_p_isk = save_p_isk;
            b.b_p_vts_array = vts_array(b);
        } else {
            b.b_p_isk = dup_global(&p_isk);
            copy_sctx(b, kBufOptIskeyword);
            did_isk = true;
            b.b_p_ts = p_ts.get();
            copy_sctx(b, kBufOptTabstop);
            b.b_p_vts = dup_global(&p_vts);
            copy_sctx(b, kBufOptVartabstop);
            b.b_p_vts_array = vts_array(b);
            b.b_help = false;
            // The buffer is no longer a help buffer, so 'buftype' must
            // not still say "help".
            // SAFETY: 'buftype' is a string option, so never null, and
            // its variable is the buffer's own.
            if (unsafe { *b.b_p_bt }) as c_int == 'h' as c_int {
                unsafe { clear_string_option(buf_field!(buf, b_p_bt)) };
            }
            b.b_p_ma = p_ma.get();
            copy_sctx(b, kBufOptModifiable);
        }
    }

    if should_copy {
        b.b_p_initialized = true;
    }

    unsafe { check_buf_options(buf) };
    if did_isk {
        unsafe { buf_init_chartab(buf, false) };
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
fn vts_array(buf: Buf) -> *mut colnr_T {
    let vts = p_vts.get();
    // SAFETY: 'vartabstop' is a string option, so its value is a live
    // NUL-terminated string; the test above is what `tabstop_set` needs.
    if !vts.is_null() && unsafe { *vts } != NUL as c_char && buf.b_p_vts_array.is_null() {
        unsafe { tabstop_array(vts) }
    } else {
        ptr::null_mut()
    }
}

/// `-M`: make every buffer unmodifiable, default included.
pub(crate) fn reset_modifiable() {
    cur_buf().b_p_ma = 0;
    p_ma.set(0);
    change_option_default(kOptModifiable, boolean_optval(Some(false)));
}

/// Carry a buffer's 'iminsert' back to the global value, so that the next
/// buffer starts where this one left off.
///
/// # Safety
///
/// `buf` must be a live buffer.
pub(crate) unsafe fn set_iminsert_global(buf: *mut buf_T) {
    // SAFETY: the caller's buffer.
    p_iminsert.set(unsafe { (*buf).b_p_iminsert });
}

/// As [`set_iminsert_global`], for 'imsearch'.
///
/// # Safety
///
/// `buf` must be a live buffer.
pub(crate) unsafe fn set_imsearch_global(buf: *mut buf_T) {
    // SAFETY: the caller's buffer.
    p_imsearch.set(unsafe { (*buf).b_p_imsearch });
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
