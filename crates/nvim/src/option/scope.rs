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
//! The result is an [`OptSlot`] (see [`super::slot`]): which of the three
//! types the option is, and then either the selector naming its field of
//! the global record or a handle to the window, buffer or syntax block
//! holding its own copy plus the selector naming that field.
//! [`super::value`] is where it gets read or written.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use crate::global_cell::field;
use crate::message::iemsg;
use crate::os::cshim::gettext;
// The generated index enum: 176 of its `kOpt*` constants name an arm below.
use crate::options::*;
use crate::types::{
    Buffer, OptIndex, OptInt, OptScope, OptValType, OptVar, OptionSetFlags, SynBlock, WinOpt,
    ssize_t,
};
use crate::winlayer::{Buf, Win};

use super::{
    BoolVar, Local, NO_LOCAL_UNDOLEVEL, NumVar, OptSlot, StrVar, WinOptSet, get_option,
    kOptScopeBuf, kOptScopeGlobal, kOptScopeWin, kOptValTypeBoolean, kOptValTypeNumber,
    kOptValTypeString,
};

/// The address of one field of a live object, computed rather than read.
///
/// A field's address is the object's plus a constant, so naming one needs no
/// dereference: `wrapping_byte_add` produces the address
/// `&raw mut (*base).field` would, in ordinary checked code and with the
/// whole object's provenance rather than the field's.
///
/// `witness` is never called. It is there so the field's *type* comes from
/// the field, which `offset_of!` erases.
pub(crate) fn field_ptr<T, F>(base: *mut T, offset: usize, _witness: fn(&T) -> &F) -> *mut F {
    base.wrapping_byte_add(offset).cast::<F>()
}

/// The [`OptSlot`] naming one field of `$buf`.
macro_rules! buf_var {
    ($buf:expr, $field:ident) => {
        OptSlot::from(Local::Buf($buf, const { field!(Buffer, $field) }))
    };
}

/// [`buf_var`] for a field of a window's `w_onebuf_opt`.
macro_rules! win_var {
    ($win:expr, $field:ident) => {
        OptSlot::from(Local::Win(
            $win,
            WinOptSet::One,
            const { field!(WinOpt, $field) },
        ))
    };
}

/// [`buf_var`] for the syntax block the four 'spell*' options live in,
/// which is reached through the window.
macro_rules! syn_var {
    ($win:expr, $field:ident) => {
        OptSlot::from(Local::Syn($win, const { field!(SynBlock, $field) }))
    };
}

/// What "not set here" looks like in a global-local option's local copy.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Unset {
    /// The option is local only: whatever its variable holds is the value.
    Never,
    /// The usual sentinel — the empty string, or a negative number.
    Sentinel,
    /// 'undolevels' keeps its own, because 0 is a real value there.
    NoLocalUndolevel,
}

impl Unset {
    /// Whether `local` holds this sentinel rather than a value of its own.
    fn holds(self, local: OptSlot) -> bool {
        match (self, local) {
            (Unset::Never, _) | (_, OptSlot::None) => false,
            // An empty local copy is "not set here" whether the field owns
            // the empty string or owns nothing at all.
            (Unset::Sentinel, OptSlot::String(var)) => var.is_empty(),
            (Unset::Sentinel, OptSlot::Boolean(var)) => var.get() < 0,
            (Unset::Sentinel, OptSlot::Number(var)) => var.get() < 0,
            (Unset::NoLocalUndolevel, OptSlot::Number(var)) => {
                var.get() == OptInt::from(NO_LOCAL_UNDOLEVEL)
            }
            (Unset::NoLocalUndolevel, _) => {
                unreachable!("only 'undolevels' carries that sentinel")
            }
        }
    }
}

/// Where an option keeps its global value: the variable its row names, or —
/// for an immutable option, which has nowhere to keep one — its own current
/// default, read in place.
///
/// This is the only place [`OptVar`] becomes an address.
pub(crate) fn option_var(opt_idx: OptIndex) -> OptSlot {
    match get_option(opt_idx).var {
        OptVar::NoGlobal => OptSlot::None,
        OptVar::Boolean(field) => OptSlot::Boolean(BoolVar::Global(field)),
        OptVar::Number(field) => OptSlot::Number(NumVar::Global(field)),
        OptVar::String(field) => OptSlot::String(StrVar::Global(field)),
        // An immutable option has no variable; its own current default is
        // the value, read (and written, if anything ever got that far) in
        // place. Which arm it is still comes from the row's declared type.
        OptVar::OwnDefault => match super::option_get_type(opt_idx) {
            kOptValTypeBoolean => OptSlot::Boolean(BoolVar::OwnDefault(opt_idx)),
            kOptValTypeNumber => OptSlot::Number(NumVar::OwnDefault(opt_idx)),
            kOptValTypeString => OptSlot::String(StrVar::OwnDefault(opt_idx)),
            type_0 => unreachable!("option value type {type_0}"),
        },
    }
}

/// Whether an option is hidden: immutable, and reading its own default in
/// place, so a write through its variable could not be observed anyway.
pub(crate) fn is_option_hidden(opt_idx: OptIndex) -> bool {
    if opt_idx == kOptInvalid {
        return false;
    }
    let opt = get_option(opt_idx);
    opt.immutable && matches!(opt.var, OptVar::OwnDefault)
}

/// Whether the table declares `type_0` as the option's type.
pub(crate) fn option_has_type(opt_idx: OptIndex, type_0: OptValType) -> bool {
    opt_idx != kOptInvalid && get_option(opt_idx).type_0 == type_0
}

/// Whether the option exists in `scope`.
pub(crate) fn option_has_scope(opt_idx: OptIndex, scope: OptScope) -> bool {
    assert!(scope <= kOptScopeBuf, "{scope} is not a scope");
    c_int::from(get_option(opt_idx).scope_flags) & 1 << scope != 0
}

/// The option's scope mask, or 0 for "no such option".
fn scope_flags(opt_idx: OptIndex) -> u32 {
    if opt_idx == kOptInvalid {
        return 0;
    }
    u32::from(get_option(opt_idx).scope_flags)
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
pub(crate) fn option_scope_idx(opt_idx: OptIndex, scope: OptScope) -> ssize_t {
    get_option(opt_idx).scope_idx[scope as usize]
}

/// The variable an explicit `:setglobal`/`:setlocal` reaches, given the
/// buffer and window that stand for "local".
pub(crate) fn get_varp_scope_from(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    buffer: Buf,
    win: Win,
) -> OptSlot {
    if opt_flags.has(OptionSetFlags::GLOBAL) && !option_is_global_only(opt_idx) {
        // A window-local option's global copy is its own field in the
        // window's second `WinOpt`, not the table's `var`.
        if option_is_window_local(opt_idx) {
            return get_varp_from(opt_idx, buffer, win).in_set(WinOptSet::All);
        }
        return option_var(opt_idx);
    }
    if opt_flags.has(OptionSetFlags::LOCAL) && option_is_global_local(opt_idx) {
        // The local variable itself, sentinel and all.
        return match local_var(opt_idx, buffer, win) {
            Some((local, _)) => local,
            None => unreachable!("option {opt_idx} has no local variable"),
        };
    }
    get_varp_from(opt_idx, buffer, win)
}

/// [`get_varp_scope_from`] for the current buffer and window.
pub(crate) fn get_varp_scope(opt_idx: OptIndex, opt_flags: OptionSetFlags) -> OptSlot {
    let (buffer, win) = (Buf::current(), Win::current());
    get_varp_scope_from(opt_idx, opt_flags, buffer, win)
}

/// The variable the option reads from right now, for the given buffer and
/// window: the local one where it is set, the global one otherwise.
///
/// The answer names the holder and the field, not an address, so it reads
/// nothing until it is used and stays a name for the same variable however
/// long it is kept — across a `did_set_*` callback included.
pub(crate) fn get_varp_from(opt_idx: OptIndex, buffer: Buf, win: Win) -> OptSlot {
    let global = option_var(opt_idx);
    if is_option_hidden(opt_idx) || option_is_global_only(opt_idx) {
        return global;
    }
    let (local, unset) = local_var(opt_idx, buffer, win).unwrap_or_else(|| {
        iemsg(gettext(c"E356: get_varp ERROR"));
        // Upstream falls through to 'wrapmargin' rather than returning
        // null; every caller reads the result.
        (buf_var!(buffer, b_p_wm), Unset::Never)
    });
    if unset.holds(local) { global } else { local }
}

/// [`get_varp_from`] for the current buffer and window.
#[inline]
pub(crate) fn get_varp(opt_idx: OptIndex) -> OptSlot {
    get_varp_from(opt_idx, Buf::current(), Win::current())
}

/// Which variable the option keeps its local value in, in `buffer` or
/// `win`, and what an unset one looks like there; `None` for an option with
/// no local value. Naming a field reads nothing.
fn local_var(opt_idx: OptIndex, buffer: Buf, win: Win) -> Option<(OptSlot, Unset)> {
    Some(match opt_idx {
        // Global-local: an unset local copy defers to the global one.
        kOptEqualprg => (buf_var!(buffer, b_p_ep), Unset::Sentinel),
        kOptKeywordprg => (buf_var!(buffer, b_p_kp), Unset::Sentinel),
        kOptPath => (buf_var!(buffer, b_p_path), Unset::Sentinel),
        kOptAutocomplete => (buf_var!(buffer, b_p_ac), Unset::Sentinel),
        kOptAutoread => (buf_var!(buffer, b_p_ar), Unset::Sentinel),
        kOptTags => (buf_var!(buffer, b_p_tags), Unset::Sentinel),
        kOptTagcase => (buf_var!(buffer, b_p_tc), Unset::Sentinel),
        kOptSidescrolloff => (win_var!(win, wo_siso), Unset::Sentinel),
        kOptScrolloff => (win_var!(win, wo_so), Unset::Sentinel),
        kOptBackupcopy => (buf_var!(buffer, b_p_bkc), Unset::Sentinel),
        kOptDefine => (buf_var!(buffer, b_p_def), Unset::Sentinel),
        kOptInclude => (buf_var!(buffer, b_p_inc), Unset::Sentinel),
        kOptCompleteopt => (buf_var!(buffer, b_p_cot), Unset::Sentinel),
        kOptDictionary => (buf_var!(buffer, b_p_dict), Unset::Sentinel),
        kOptDiffanchors => (buf_var!(buffer, b_p_dia), Unset::Sentinel),
        kOptThesaurus => (buf_var!(buffer, b_p_tsr), Unset::Sentinel),
        kOptThesaurusfunc => (buf_var!(buffer, b_p_tsrfu), Unset::Sentinel),
        kOptFormatprg => (buf_var!(buffer, b_p_fp), Unset::Sentinel),
        kOptFsync => (buf_var!(buffer, b_p_fs), Unset::Sentinel),
        kOptFindfunc => (buf_var!(buffer, b_p_ffu), Unset::Sentinel),
        kOptErrorformat => (buf_var!(buffer, b_p_efm), Unset::Sentinel),
        kOptGrepformat => (buf_var!(buffer, b_p_gefm), Unset::Sentinel),
        kOptGrepprg => (buf_var!(buffer, b_p_gp), Unset::Sentinel),
        kOptMakeprg => (buf_var!(buffer, b_p_mp), Unset::Sentinel),
        kOptShowbreak => (win_var!(win, wo_sbr), Unset::Sentinel),
        kOptStatusline => (win_var!(win, wo_stl), Unset::Sentinel),
        kOptWinbar => (win_var!(win, wo_wbr), Unset::Sentinel),
        // 'undolevels' has a sentinel of its own: 0 is a real value.
        kOptUndolevels => (buf_var!(buffer, b_p_ul), Unset::NoLocalUndolevel),
        kOptLispwords => (buf_var!(buffer, b_p_lw), Unset::Sentinel),
        kOptMakeencoding => (buf_var!(buffer, b_p_menc), Unset::Sentinel),
        kOptFillchars => (win_var!(win, wo_fcs), Unset::Sentinel),
        kOptListchars => (win_var!(win, wo_lcs), Unset::Sentinel),
        kOptVirtualedit => (win_var!(win, wo_ve), Unset::Sentinel),

        // Window-local.
        kOptArabic => (win_var!(win, wo_arab), Unset::Never),
        kOptList => (win_var!(win, wo_list), Unset::Never),
        kOptSpell => (win_var!(win, wo_spell), Unset::Never),
        kOptCursorcolumn => (win_var!(win, wo_cuc), Unset::Never),
        kOptCursorline => (win_var!(win, wo_cul), Unset::Never),
        kOptCursorlineopt => (win_var!(win, wo_culopt), Unset::Never),
        kOptColorcolumn => (win_var!(win, wo_cc), Unset::Never),
        kOptDiff => (win_var!(win, wo_diff), Unset::Never),
        kOptEventignorewin => (win_var!(win, wo_eiw), Unset::Never),
        kOptFoldcolumn => (win_var!(win, wo_fdc), Unset::Never),
        kOptFoldenable => (win_var!(win, wo_fen), Unset::Never),
        kOptFoldignore => (win_var!(win, wo_fdi), Unset::Never),
        kOptFoldlevel => (win_var!(win, wo_fdl), Unset::Never),
        kOptFoldmethod => (win_var!(win, wo_fdm), Unset::Never),
        kOptFoldminlines => (win_var!(win, wo_fml), Unset::Never),
        kOptFoldnestmax => (win_var!(win, wo_fdn), Unset::Never),
        kOptFoldexpr => (win_var!(win, wo_fde), Unset::Never),
        kOptFoldtext => (win_var!(win, wo_fdt), Unset::Never),
        kOptFoldmarker => (win_var!(win, wo_fmr), Unset::Never),
        kOptNumber => (win_var!(win, wo_nu), Unset::Never),
        kOptRelativenumber => (win_var!(win, wo_rnu), Unset::Never),
        kOptNumberwidth => (win_var!(win, wo_nuw), Unset::Never),
        kOptWinfixbuf => (win_var!(win, wo_wfb), Unset::Never),
        kOptWinfixheight => (win_var!(win, wo_wfh), Unset::Never),
        kOptWinfixwidth => (win_var!(win, wo_wfw), Unset::Never),
        kOptPreviewwindow => (win_var!(win, wo_pvw), Unset::Never),
        kOptLhistory => (win_var!(win, wo_lhi), Unset::Never),
        kOptRightleft => (win_var!(win, wo_rl), Unset::Never),
        kOptRightleftcmd => (win_var!(win, wo_rlc), Unset::Never),
        kOptScroll => (win_var!(win, wo_scr), Unset::Never),
        kOptSmoothscroll => (win_var!(win, wo_sms), Unset::Never),
        kOptWrap => (win_var!(win, wo_wrap), Unset::Never),
        kOptLinebreak => (win_var!(win, wo_lbr), Unset::Never),
        kOptBreakindent => (win_var!(win, wo_bri), Unset::Never),
        kOptBreakindentopt => (win_var!(win, wo_briopt), Unset::Never),
        kOptScrollbind => (win_var!(win, wo_scb), Unset::Never),
        kOptCursorbind => (win_var!(win, wo_crb), Unset::Never),
        kOptConcealcursor => (win_var!(win, wo_cocu), Unset::Never),
        kOptConceallevel => (win_var!(win, wo_cole), Unset::Never),
        kOptSigncolumn => (win_var!(win, wo_scl), Unset::Never),
        kOptWinhighlight => (win_var!(win, wo_winhl), Unset::Never),
        kOptWinblend => (win_var!(win, wo_winbl), Unset::Never),
        kOptStatuscolumn => (win_var!(win, wo_stc), Unset::Never),

        // The 'spell*' options belong to the window's syntax block,
        // which a diff or preview window may share with another window.
        kOptSpellcapcheck => (syn_var!(win, b_p_spc), Unset::Never),
        kOptSpellfile => (syn_var!(win, b_p_spf), Unset::Never),
        kOptSpelllang => (syn_var!(win, b_p_spl), Unset::Never),
        kOptSpelloptions => (syn_var!(win, b_p_spo), Unset::Never),

        // Buffer-local.
        kOptAutoindent => (buf_var!(buffer, b_p_ai), Unset::Never),
        kOptBinary => (buf_var!(buffer, b_p_bin), Unset::Never),
        kOptBomb => (buf_var!(buffer, b_p_bomb), Unset::Never),
        kOptBufhidden => (buf_var!(buffer, b_p_bh), Unset::Never),
        kOptBuftype => (buf_var!(buffer, b_p_bt), Unset::Never),
        kOptBuflisted => (buf_var!(buffer, b_p_bl), Unset::Never),
        kOptBusy => (buf_var!(buffer, b_p_busy), Unset::Never),
        kOptChannel => (buf_var!(buffer, b_p_channel), Unset::Never),
        kOptCopyindent => (buf_var!(buffer, b_p_ci), Unset::Never),
        kOptCindent => (buf_var!(buffer, b_p_cin), Unset::Never),
        kOptCinkeys => (buf_var!(buffer, b_p_cink), Unset::Never),
        kOptCinoptions => (buf_var!(buffer, b_p_cino), Unset::Never),
        kOptCinscopedecls => (buf_var!(buffer, b_p_cinsd), Unset::Never),
        kOptCinwords => (buf_var!(buffer, b_p_cinw), Unset::Never),
        kOptComments => (buf_var!(buffer, b_p_com), Unset::Never),
        kOptCommentstring => (buf_var!(buffer, b_p_cms), Unset::Never),
        kOptComplete => (buf_var!(buffer, b_p_cpt), Unset::Never),
        kOptCompletefunc => (buf_var!(buffer, b_p_cfu), Unset::Never),
        kOptOmnifunc => (buf_var!(buffer, b_p_ofu), Unset::Never),
        kOptEndoffile => (buf_var!(buffer, b_p_eof), Unset::Never),
        kOptEndofline => (buf_var!(buffer, b_p_eol), Unset::Never),
        kOptFixendofline => (buf_var!(buffer, b_p_fixeol), Unset::Never),
        kOptExpandtab => (buf_var!(buffer, b_p_et), Unset::Never),
        kOptFileencoding => (buf_var!(buffer, b_p_fenc), Unset::Never),
        kOptFileformat => (buf_var!(buffer, b_p_ff), Unset::Never),
        kOptFiletype => (buf_var!(buffer, b_p_ft), Unset::Never),
        kOptFormatoptions => (buf_var!(buffer, b_p_fo), Unset::Never),
        kOptFormatlistpat => (buf_var!(buffer, b_p_flp), Unset::Never),
        kOptIminsert => (buf_var!(buffer, b_p_iminsert), Unset::Never),
        kOptImsearch => (buf_var!(buffer, b_p_imsearch), Unset::Never),
        kOptInfercase => (buf_var!(buffer, b_p_inf), Unset::Never),
        kOptIskeyword => (buf_var!(buffer, b_p_isk), Unset::Never),
        kOptIncludeexpr => (buf_var!(buffer, b_p_inex), Unset::Never),
        kOptIndentexpr => (buf_var!(buffer, b_p_inde), Unset::Never),
        kOptIndentkeys => (buf_var!(buffer, b_p_indk), Unset::Never),
        kOptFormatexpr => (buf_var!(buffer, b_p_fex), Unset::Never),
        kOptLisp => (buf_var!(buffer, b_p_lisp), Unset::Never),
        kOptLispoptions => (buf_var!(buffer, b_p_lop), Unset::Never),
        kOptModeline => (buf_var!(buffer, b_p_ml), Unset::Never),
        kOptMatchpairs => (buf_var!(buffer, b_p_mps), Unset::Never),
        kOptModifiable => (buf_var!(buffer, b_p_ma), Unset::Never),
        kOptModified => (buf_var!(buffer, b_changed), Unset::Never),
        kOptNrformats => (buf_var!(buffer, b_p_nf), Unset::Never),
        kOptPreserveindent => (buf_var!(buffer, b_p_pi), Unset::Never),
        kOptQuoteescape => (buf_var!(buffer, b_p_qe), Unset::Never),
        kOptReadonly => (buf_var!(buffer, b_p_ro), Unset::Never),
        kOptScrollback => (buf_var!(buffer, b_p_scbk), Unset::Never),
        kOptSmartindent => (buf_var!(buffer, b_p_si), Unset::Never),
        kOptSofttabstop => (buf_var!(buffer, b_p_sts), Unset::Never),
        kOptSuffixesadd => (buf_var!(buffer, b_p_sua), Unset::Never),
        kOptSwapfile => (buf_var!(buffer, b_p_swf), Unset::Never),
        kOptSynmaxcol => (buf_var!(buffer, b_p_smc), Unset::Never),
        kOptSyntax => (buf_var!(buffer, b_p_syn), Unset::Never),
        kOptShiftwidth => (buf_var!(buffer, b_p_sw), Unset::Never),
        kOptTagfunc => (buf_var!(buffer, b_p_tfu), Unset::Never),
        kOptTabstop => (buf_var!(buffer, b_p_ts), Unset::Never),
        kOptTextwidth => (buf_var!(buffer, b_p_tw), Unset::Never),
        kOptUndofile => (buf_var!(buffer, b_p_udf), Unset::Never),
        kOptWrapmargin => (buf_var!(buffer, b_p_wm), Unset::Never),
        kOptVarsofttabstop => (buf_var!(buffer, b_p_vsts), Unset::Never),
        kOptVartabstop => (buf_var!(buffer, b_p_vts), Unset::Never),
        kOptKeymap => (buf_var!(buffer, b_p_keymap), Unset::Never),

        _ => return None,
    })
}
