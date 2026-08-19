//! Which copy of a value a scope is looking at — the `varp` plumbing.
//!
//! An option's value lives in a variable, and which variable depends on the
//! scope: a global one in the option table's `var`, a window-local one in
//! `win->w_onebuf_opt`, a buffer-local one in the buffer. A *global-local*
//! option has both, and its local copy carries a sentinel meaning "not set
//! here" — an empty string, a negative number, or `NO_LOCAL_UNDOLEVEL`.
//!
//! [`get_varp_from`] answers "which variable does this option read from
//! right now", following that fallback; [`get_varp_scope_from`] answers the
//! same question for an explicit `:setglobal`/`:setlocal`, and is the one
//! caller that must see the sentinel rather than fall back.
//!
//! The result is a `*mut c_void` because the three value types have three
//! different widths; [`super::value`] is where it gets read or written, and
//! it re-derives the type from the same table row.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::mem::{offset_of, size_of};
use core::ptr;

use crate::main::{curbuf, curwin};
use crate::message::iemsg;
use crate::os::cshim::gettext;
// The generated index enum: 176 of its `kOpt*` constants name an arm below.
use crate::options::*;
use crate::types::{
    OptIndex, OptInt, OptScope, OptVal, OptValType, OptVar, OptionSetFlags, buf_T, ssize_t,
    vimoption_T, win_T, winopt_T,
};

use super::{NO_LOCAL_UNDOLEVEL, get_option, kOptScopeBuf, kOptScopeGlobal, kOptScopeWin};

/// `w_allbuf_opt` follows `w_onebuf_opt` in `win_T`, so the window's global
/// copy of a window-local option is the same field exactly one `winopt_T`
/// further on. `get_varp_scope_from` walks that distance rather than
/// repeating the whole field table for the `:setglobal` case.
const ALLBUF_OFFSET: usize = offset_of!(win_T, w_allbuf_opt) - offset_of!(win_T, w_onebuf_opt);
const _: () = assert!(ALLBUF_OFFSET == size_of::<winopt_T>());

/// Where an option keeps its global value: the variable its row names, or —
/// for an immutable option, which has nowhere to keep one — that row's own
/// `def_val.data`, read in place.
///
/// This is the only place [`OptVar`] becomes an address, and the reason it
/// needs the row rather than the tag alone.
///
/// # Safety
///
/// `p` must point into the option table.
pub(crate) unsafe fn option_var(p: *mut vimoption_T) -> *mut c_void {
    // SAFETY: the caller's `p` is a table row.
    match unsafe { (*p).var } {
        OptVar::NoGlobal => ptr::null_mut(),
        OptVar::Boolean(cell) => cell.ptr().cast(),
        OptVar::Number(cell) => cell.ptr().cast(),
        OptVar::String(cell) => cell.ptr().cast(),
        OptVar::OwnDefault => p
            .wrapping_byte_add(offset_of!(vimoption_T, def_val) + offset_of!(OptVal, data))
            .cast(),
    }
}

/// Whether an option is hidden: immutable, and reading its own default in
/// place, so a write through its variable could not be observed anyway.
pub fn is_option_hidden(opt_idx: OptIndex) -> bool {
    if opt_idx == kOptInvalid {
        return false;
    }
    let opt = get_option(opt_idx);
    // SAFETY: `get_option` hands back a row of the option table.
    unsafe { (*opt).immutable && matches!((*opt).var, OptVar::OwnDefault) }
}

/// Whether the table declares `type_0` as the option's type.
pub fn option_has_type(opt_idx: OptIndex, type_0: OptValType) -> bool {
    // SAFETY: the option table is a plain array; nothing holds a borrow.
    opt_idx != kOptInvalid && unsafe { (*get_option(opt_idx)).type_0 } == type_0
}

/// Whether the option exists in `scope`.
pub fn option_has_scope(opt_idx: OptIndex, scope: OptScope) -> bool {
    assert!(scope <= kOptScopeBuf, "{scope} is not a scope");
    // SAFETY: the option table is a plain array; nothing holds a borrow.
    unsafe { (*get_option(opt_idx)).scope_flags as c_int & 1 << scope != 0 }
}

/// The option's scope mask, or 0 for "no such option".
fn scope_flags(opt_idx: OptIndex) -> u32 {
    if opt_idx == kOptInvalid {
        return 0;
    }
    // SAFETY: the option table is a plain array; nothing holds a borrow.
    unsafe { (*get_option(opt_idx)).scope_flags as u32 }
}

/// Whether the option has both a global value and a local one.
pub(crate) fn option_is_global_local(opt_idx: OptIndex) -> bool {
    opt_idx != kOptInvalid && scope_flags(opt_idx).count_ones() != 1
}

/// Whether the option's only scope is the global one.
pub(crate) fn option_is_global_only(opt_idx: OptIndex) -> bool {
    scope_flags(opt_idx).count_ones() == 1 && option_has_scope(opt_idx, kOptScopeGlobal)
}

/// Whether the option's only scope is a window.
pub(crate) fn option_is_window_local(opt_idx: OptIndex) -> bool {
    scope_flags(opt_idx).count_ones() == 1 && option_has_scope(opt_idx, kOptScopeWin)
}

/// Where in a window's or buffer's array of values this option's sits.
pub fn option_scope_idx(opt_idx: OptIndex, scope: OptScope) -> ssize_t {
    // SAFETY: the option table is a plain array; nothing holds a borrow.
    unsafe { (*get_option(opt_idx)).scope_idx[scope as usize] }
}

/// A global-local string variable, or the global one when the local value is
/// the empty string.
///
/// # Safety
///
/// `local` must point at the option's local string variable.
unsafe fn local_str(local: *mut *mut c_char, global: *mut c_void) -> *mut c_void {
    // SAFETY: an option's string variable is never null.
    if unsafe { **local } != 0 {
        local.cast::<c_void>()
    } else {
        global
    }
}

/// A global-local `int` variable, or the global one when the local value is
/// negative.
///
/// # Safety
///
/// `local` must point at the option's local variable.
unsafe fn local_int(local: *mut c_int, global: *mut c_void) -> *mut c_void {
    // SAFETY: the caller's pointer is the option's local variable.
    if unsafe { *local } >= 0 {
        local.cast::<c_void>()
    } else {
        global
    }
}

/// As [`local_int`], for the wider `OptInt` variables.
///
/// # Safety
///
/// `local` must point at the option's local variable.
unsafe fn local_optint(local: *mut OptInt, global: *mut c_void) -> *mut c_void {
    // SAFETY: the caller's pointer is the option's local variable.
    if unsafe { *local } >= 0 {
        local.cast::<c_void>()
    } else {
        global
    }
}

/// The variable an explicit `:setglobal`/`:setlocal` reaches, given the
/// buffer and window that stand for "local".
///
/// # Safety
///
/// `p` must point into the option table; `buf` and `win` must be live.
pub unsafe fn get_varp_scope_from(
    p: *mut vimoption_T,
    opt_flags: OptionSetFlags,
    buf: *mut buf_T,
    win: *mut win_T,
) -> *mut c_void {
    // SAFETY: the caller's pointers are live, and `p` is a table row.
    unsafe {
        let opt_idx = get_opt_idx(p);
        if opt_flags.has(OptionSetFlags::GLOBAL) && !option_is_global_only(opt_idx) {
            // A window-local option's global copy is its own field in the
            // window's second `winopt_T`, not the table's `var`.
            if option_is_window_local(opt_idx) {
                return get_varp_from(p, buf, win)
                    .cast::<c_char>()
                    .add(ALLBUF_OFFSET)
                    .cast::<c_void>();
            }
            return option_var(p);
        }
        if opt_flags.has(OptionSetFlags::LOCAL) && option_is_global_local(opt_idx) {
            // The local variable itself, sentinel and all.
            return match opt_idx {
                kOptFormatprg => (&raw mut (*buf).b_p_fp).cast(),
                kOptFsync => (&raw mut (*buf).b_p_fs).cast(),
                kOptFindfunc => (&raw mut (*buf).b_p_ffu).cast(),
                kOptErrorformat => (&raw mut (*buf).b_p_efm).cast(),
                kOptGrepformat => (&raw mut (*buf).b_p_gefm).cast(),
                kOptGrepprg => (&raw mut (*buf).b_p_gp).cast(),
                kOptMakeprg => (&raw mut (*buf).b_p_mp).cast(),
                kOptEqualprg => (&raw mut (*buf).b_p_ep).cast(),
                kOptKeywordprg => (&raw mut (*buf).b_p_kp).cast(),
                kOptPath => (&raw mut (*buf).b_p_path).cast(),
                kOptAutocomplete => (&raw mut (*buf).b_p_ac).cast(),
                kOptAutoread => (&raw mut (*buf).b_p_ar).cast(),
                kOptTags => (&raw mut (*buf).b_p_tags).cast(),
                kOptTagcase => (&raw mut (*buf).b_p_tc).cast(),
                kOptSidescrolloff => (&raw mut (*win).w_onebuf_opt.wo_siso).cast(),
                kOptScrolloff => (&raw mut (*win).w_onebuf_opt.wo_so).cast(),
                kOptDefine => (&raw mut (*buf).b_p_def).cast(),
                kOptInclude => (&raw mut (*buf).b_p_inc).cast(),
                kOptCompleteopt => (&raw mut (*buf).b_p_cot).cast(),
                kOptDictionary => (&raw mut (*buf).b_p_dict).cast(),
                kOptDiffanchors => (&raw mut (*buf).b_p_dia).cast(),
                kOptThesaurus => (&raw mut (*buf).b_p_tsr).cast(),
                kOptThesaurusfunc => (&raw mut (*buf).b_p_tsrfu).cast(),
                kOptTagfunc => (&raw mut (*buf).b_p_tfu).cast(),
                kOptShowbreak => (&raw mut (*win).w_onebuf_opt.wo_sbr).cast(),
                kOptStatusline => (&raw mut (*win).w_onebuf_opt.wo_stl).cast(),
                kOptWinbar => (&raw mut (*win).w_onebuf_opt.wo_wbr).cast(),
                kOptUndolevels => (&raw mut (*buf).b_p_ul).cast(),
                kOptLispwords => (&raw mut (*buf).b_p_lw).cast(),
                kOptBackupcopy => (&raw mut (*buf).b_p_bkc).cast(),
                kOptMakeencoding => (&raw mut (*buf).b_p_menc).cast(),
                kOptFillchars => (&raw mut (*win).w_onebuf_opt.wo_fcs).cast(),
                kOptListchars => (&raw mut (*win).w_onebuf_opt.wo_lcs).cast(),
                kOptVirtualedit => (&raw mut (*win).w_onebuf_opt.wo_ve).cast(),
                _ => unreachable!("option {opt_idx} has no local variable"),
            };
        }
        get_varp_from(p, buf, win)
    }
}

/// [`get_varp_scope_from`] for the current buffer and window.
///
/// # Safety
///
/// `p` must point into the option table.
pub unsafe fn get_varp_scope(p: *mut vimoption_T, opt_flags: OptionSetFlags) -> *mut c_void {
    // SAFETY: the caller's `p` is a table row; `curbuf`/`curwin` are live.
    unsafe { get_varp_scope_from(p, opt_flags, curbuf.get(), curwin.get()) }
}

/// [`get_varp_scope_from`] by index.
///
/// # Safety
///
/// `buf` and `win` must be live.
pub unsafe fn get_option_varp_scope_from(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    buf: *mut buf_T,
    win: *mut win_T,
) -> *mut c_void {
    // SAFETY: `opt_idx` indexes the table; the caller's pointers are live.
    unsafe { get_varp_scope_from(get_option(opt_idx), opt_flags, buf, win) }
}

/// The variable the option reads from right now, for the given buffer and
/// window: the local one where it is set, the global one otherwise.
///
/// # Safety
///
/// `p` must point into the option table; `buf` and `win` must be live, and
/// `win->w_s` must be set for the four 'spell*' options.
pub unsafe fn get_varp_from(p: *mut vimoption_T, buf: *mut buf_T, win: *mut win_T) -> *mut c_void {
    // SAFETY: the caller's pointers are live, and `p` is a table row.
    unsafe {
        let opt_idx = get_opt_idx(p);
        let global = option_var(p);
        if is_option_hidden(opt_idx) || option_is_global_only(opt_idx) {
            return global;
        }
        match opt_idx {
            // Global-local: an unset local copy defers to the global one.
            kOptEqualprg => local_str(&raw mut (*buf).b_p_ep, global),
            kOptKeywordprg => local_str(&raw mut (*buf).b_p_kp, global),
            kOptPath => local_str(&raw mut (*buf).b_p_path, global),
            kOptAutocomplete => local_int(&raw mut (*buf).b_p_ac, global),
            kOptAutoread => local_int(&raw mut (*buf).b_p_ar, global),
            kOptTags => local_str(&raw mut (*buf).b_p_tags, global),
            kOptTagcase => local_str(&raw mut (*buf).b_p_tc, global),
            kOptSidescrolloff => local_optint(&raw mut (*win).w_onebuf_opt.wo_siso, global),
            kOptScrolloff => local_optint(&raw mut (*win).w_onebuf_opt.wo_so, global),
            kOptBackupcopy => local_str(&raw mut (*buf).b_p_bkc, global),
            kOptDefine => local_str(&raw mut (*buf).b_p_def, global),
            kOptInclude => local_str(&raw mut (*buf).b_p_inc, global),
            kOptCompleteopt => local_str(&raw mut (*buf).b_p_cot, global),
            kOptDictionary => local_str(&raw mut (*buf).b_p_dict, global),
            kOptDiffanchors => local_str(&raw mut (*buf).b_p_dia, global),
            kOptThesaurus => local_str(&raw mut (*buf).b_p_tsr, global),
            kOptThesaurusfunc => local_str(&raw mut (*buf).b_p_tsrfu, global),
            kOptFormatprg => local_str(&raw mut (*buf).b_p_fp, global),
            kOptFsync => local_int(&raw mut (*buf).b_p_fs, global),
            kOptFindfunc => local_str(&raw mut (*buf).b_p_ffu, global),
            kOptErrorformat => local_str(&raw mut (*buf).b_p_efm, global),
            kOptGrepformat => local_str(&raw mut (*buf).b_p_gefm, global),
            kOptGrepprg => local_str(&raw mut (*buf).b_p_gp, global),
            kOptMakeprg => local_str(&raw mut (*buf).b_p_mp, global),
            kOptShowbreak => local_str(&raw mut (*win).w_onebuf_opt.wo_sbr, global),
            kOptStatusline => local_str(&raw mut (*win).w_onebuf_opt.wo_stl, global),
            kOptWinbar => local_str(&raw mut (*win).w_onebuf_opt.wo_wbr, global),
            // 'undolevels' has a sentinel of its own: 0 is a real value.
            kOptUndolevels => {
                if (*buf).b_p_ul != NO_LOCAL_UNDOLEVEL as OptInt {
                    (&raw mut (*buf).b_p_ul).cast()
                } else {
                    global
                }
            }
            kOptLispwords => local_str(&raw mut (*buf).b_p_lw, global),
            kOptMakeencoding => local_str(&raw mut (*buf).b_p_menc, global),
            kOptFillchars => local_str(&raw mut (*win).w_onebuf_opt.wo_fcs, global),
            kOptListchars => local_str(&raw mut (*win).w_onebuf_opt.wo_lcs, global),
            kOptVirtualedit => local_str(&raw mut (*win).w_onebuf_opt.wo_ve, global),

            // Window-local.
            kOptArabic => (&raw mut (*win).w_onebuf_opt.wo_arab).cast(),
            kOptList => (&raw mut (*win).w_onebuf_opt.wo_list).cast(),
            kOptSpell => (&raw mut (*win).w_onebuf_opt.wo_spell).cast(),
            kOptCursorcolumn => (&raw mut (*win).w_onebuf_opt.wo_cuc).cast(),
            kOptCursorline => (&raw mut (*win).w_onebuf_opt.wo_cul).cast(),
            kOptCursorlineopt => (&raw mut (*win).w_onebuf_opt.wo_culopt).cast(),
            kOptColorcolumn => (&raw mut (*win).w_onebuf_opt.wo_cc).cast(),
            kOptDiff => (&raw mut (*win).w_onebuf_opt.wo_diff).cast(),
            kOptEventignorewin => (&raw mut (*win).w_onebuf_opt.wo_eiw).cast(),
            kOptFoldcolumn => (&raw mut (*win).w_onebuf_opt.wo_fdc).cast(),
            kOptFoldenable => (&raw mut (*win).w_onebuf_opt.wo_fen).cast(),
            kOptFoldignore => (&raw mut (*win).w_onebuf_opt.wo_fdi).cast(),
            kOptFoldlevel => (&raw mut (*win).w_onebuf_opt.wo_fdl).cast(),
            kOptFoldmethod => (&raw mut (*win).w_onebuf_opt.wo_fdm).cast(),
            kOptFoldminlines => (&raw mut (*win).w_onebuf_opt.wo_fml).cast(),
            kOptFoldnestmax => (&raw mut (*win).w_onebuf_opt.wo_fdn).cast(),
            kOptFoldexpr => (&raw mut (*win).w_onebuf_opt.wo_fde).cast(),
            kOptFoldtext => (&raw mut (*win).w_onebuf_opt.wo_fdt).cast(),
            kOptFoldmarker => (&raw mut (*win).w_onebuf_opt.wo_fmr).cast(),
            kOptNumber => (&raw mut (*win).w_onebuf_opt.wo_nu).cast(),
            kOptRelativenumber => (&raw mut (*win).w_onebuf_opt.wo_rnu).cast(),
            kOptNumberwidth => (&raw mut (*win).w_onebuf_opt.wo_nuw).cast(),
            kOptWinfixbuf => (&raw mut (*win).w_onebuf_opt.wo_wfb).cast(),
            kOptWinfixheight => (&raw mut (*win).w_onebuf_opt.wo_wfh).cast(),
            kOptWinfixwidth => (&raw mut (*win).w_onebuf_opt.wo_wfw).cast(),
            kOptPreviewwindow => (&raw mut (*win).w_onebuf_opt.wo_pvw).cast(),
            kOptLhistory => (&raw mut (*win).w_onebuf_opt.wo_lhi).cast(),
            kOptRightleft => (&raw mut (*win).w_onebuf_opt.wo_rl).cast(),
            kOptRightleftcmd => (&raw mut (*win).w_onebuf_opt.wo_rlc).cast(),
            kOptScroll => (&raw mut (*win).w_onebuf_opt.wo_scr).cast(),
            kOptSmoothscroll => (&raw mut (*win).w_onebuf_opt.wo_sms).cast(),
            kOptWrap => (&raw mut (*win).w_onebuf_opt.wo_wrap).cast(),
            kOptLinebreak => (&raw mut (*win).w_onebuf_opt.wo_lbr).cast(),
            kOptBreakindent => (&raw mut (*win).w_onebuf_opt.wo_bri).cast(),
            kOptBreakindentopt => (&raw mut (*win).w_onebuf_opt.wo_briopt).cast(),
            kOptScrollbind => (&raw mut (*win).w_onebuf_opt.wo_scb).cast(),
            kOptCursorbind => (&raw mut (*win).w_onebuf_opt.wo_crb).cast(),
            kOptConcealcursor => (&raw mut (*win).w_onebuf_opt.wo_cocu).cast(),
            kOptConceallevel => (&raw mut (*win).w_onebuf_opt.wo_cole).cast(),
            kOptSigncolumn => (&raw mut (*win).w_onebuf_opt.wo_scl).cast(),
            kOptWinhighlight => (&raw mut (*win).w_onebuf_opt.wo_winhl).cast(),
            kOptWinblend => (&raw mut (*win).w_onebuf_opt.wo_winbl).cast(),
            kOptStatuscolumn => (&raw mut (*win).w_onebuf_opt.wo_stc).cast(),

            // The 'spell*' options belong to the window's syntax block,
            // which a diff or preview window may share with another window.
            kOptSpellcapcheck => (&raw mut (*(*win).w_s).b_p_spc).cast(),
            kOptSpellfile => (&raw mut (*(*win).w_s).b_p_spf).cast(),
            kOptSpelllang => (&raw mut (*(*win).w_s).b_p_spl).cast(),
            kOptSpelloptions => (&raw mut (*(*win).w_s).b_p_spo).cast(),

            // Buffer-local.
            kOptAutoindent => (&raw mut (*buf).b_p_ai).cast(),
            kOptBinary => (&raw mut (*buf).b_p_bin).cast(),
            kOptBomb => (&raw mut (*buf).b_p_bomb).cast(),
            kOptBufhidden => (&raw mut (*buf).b_p_bh).cast(),
            kOptBuftype => (&raw mut (*buf).b_p_bt).cast(),
            kOptBuflisted => (&raw mut (*buf).b_p_bl).cast(),
            kOptBusy => (&raw mut (*buf).b_p_busy).cast(),
            kOptChannel => (&raw mut (*buf).b_p_channel).cast(),
            kOptCopyindent => (&raw mut (*buf).b_p_ci).cast(),
            kOptCindent => (&raw mut (*buf).b_p_cin).cast(),
            kOptCinkeys => (&raw mut (*buf).b_p_cink).cast(),
            kOptCinoptions => (&raw mut (*buf).b_p_cino).cast(),
            kOptCinscopedecls => (&raw mut (*buf).b_p_cinsd).cast(),
            kOptCinwords => (&raw mut (*buf).b_p_cinw).cast(),
            kOptComments => (&raw mut (*buf).b_p_com).cast(),
            kOptCommentstring => (&raw mut (*buf).b_p_cms).cast(),
            kOptComplete => (&raw mut (*buf).b_p_cpt).cast(),
            kOptCompletefunc => (&raw mut (*buf).b_p_cfu).cast(),
            kOptOmnifunc => (&raw mut (*buf).b_p_ofu).cast(),
            kOptEndoffile => (&raw mut (*buf).b_p_eof).cast(),
            kOptEndofline => (&raw mut (*buf).b_p_eol).cast(),
            kOptFixendofline => (&raw mut (*buf).b_p_fixeol).cast(),
            kOptExpandtab => (&raw mut (*buf).b_p_et).cast(),
            kOptFileencoding => (&raw mut (*buf).b_p_fenc).cast(),
            kOptFileformat => (&raw mut (*buf).b_p_ff).cast(),
            kOptFiletype => (&raw mut (*buf).b_p_ft).cast(),
            kOptFormatoptions => (&raw mut (*buf).b_p_fo).cast(),
            kOptFormatlistpat => (&raw mut (*buf).b_p_flp).cast(),
            kOptIminsert => (&raw mut (*buf).b_p_iminsert).cast(),
            kOptImsearch => (&raw mut (*buf).b_p_imsearch).cast(),
            kOptInfercase => (&raw mut (*buf).b_p_inf).cast(),
            kOptIskeyword => (&raw mut (*buf).b_p_isk).cast(),
            kOptIncludeexpr => (&raw mut (*buf).b_p_inex).cast(),
            kOptIndentexpr => (&raw mut (*buf).b_p_inde).cast(),
            kOptIndentkeys => (&raw mut (*buf).b_p_indk).cast(),
            kOptFormatexpr => (&raw mut (*buf).b_p_fex).cast(),
            kOptLisp => (&raw mut (*buf).b_p_lisp).cast(),
            kOptLispoptions => (&raw mut (*buf).b_p_lop).cast(),
            kOptModeline => (&raw mut (*buf).b_p_ml).cast(),
            kOptMatchpairs => (&raw mut (*buf).b_p_mps).cast(),
            kOptModifiable => (&raw mut (*buf).b_p_ma).cast(),
            kOptModified => (&raw mut (*buf).b_changed).cast(),
            kOptNrformats => (&raw mut (*buf).b_p_nf).cast(),
            kOptPreserveindent => (&raw mut (*buf).b_p_pi).cast(),
            kOptQuoteescape => (&raw mut (*buf).b_p_qe).cast(),
            kOptReadonly => (&raw mut (*buf).b_p_ro).cast(),
            kOptScrollback => (&raw mut (*buf).b_p_scbk).cast(),
            kOptSmartindent => (&raw mut (*buf).b_p_si).cast(),
            kOptSofttabstop => (&raw mut (*buf).b_p_sts).cast(),
            kOptSuffixesadd => (&raw mut (*buf).b_p_sua).cast(),
            kOptSwapfile => (&raw mut (*buf).b_p_swf).cast(),
            kOptSynmaxcol => (&raw mut (*buf).b_p_smc).cast(),
            kOptSyntax => (&raw mut (*buf).b_p_syn).cast(),
            kOptShiftwidth => (&raw mut (*buf).b_p_sw).cast(),
            kOptTagfunc => (&raw mut (*buf).b_p_tfu).cast(),
            kOptTabstop => (&raw mut (*buf).b_p_ts).cast(),
            kOptTextwidth => (&raw mut (*buf).b_p_tw).cast(),
            kOptUndofile => (&raw mut (*buf).b_p_udf).cast(),
            kOptWrapmargin => (&raw mut (*buf).b_p_wm).cast(),
            kOptVarsofttabstop => (&raw mut (*buf).b_p_vsts).cast(),
            kOptVartabstop => (&raw mut (*buf).b_p_vts).cast(),
            kOptKeymap => (&raw mut (*buf).b_p_keymap).cast(),

            _ => {
                iemsg(gettext(c"E356: get_varp ERROR".as_ptr()));
                // Upstream falls through to 'wrapmargin' rather than
                // returning null; every caller dereferences the result.
                (&raw mut (*buf).b_p_wm).cast()
            }
        }
    }
}

/// The index of a table row.
///
/// # Safety
///
/// `opt` must point into the option table.
#[inline]
pub(crate) unsafe fn get_opt_idx(opt: *mut vimoption_T) -> OptIndex {
    // SAFETY: the caller's pointer is a row of `options`.
    unsafe { opt.offset_from(options.ptr() as *mut vimoption_T) as OptIndex }
}

/// [`get_varp_from`] for the current buffer and window.
///
/// # Safety
///
/// `p` must point into the option table.
#[inline]
pub(crate) unsafe fn get_varp(p: *mut vimoption_T) -> *mut c_void {
    // SAFETY: the caller's `p` is a table row; `curbuf`/`curwin` are live.
    unsafe { get_varp_from(p, curbuf.get(), curwin.get()) }
}
