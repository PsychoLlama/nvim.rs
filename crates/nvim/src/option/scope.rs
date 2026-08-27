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
use core::mem::offset_of;

use crate::main::{curbuf, curwin};
use crate::message::iemsg;
use crate::os::cshim::gettext;
// The generated index enum: 176 of its `kOpt*` constants name an arm below.
use crate::options::*;
use crate::types::{
    OptIndex, OptInt, OptScope, OptValType, OptVar, OptionSetFlags, buf_T, ssize_t, win_T,
};

use super::{
    NO_LOCAL_UNDOLEVEL, get_option, kOptScopeBuf, kOptScopeGlobal, kOptScopeWin,
    kOptValTypeBoolean, kOptValTypeNumber, kOptValTypeString, option_default_var,
};

/// The signed distance from a field of `w_onebuf_opt` to the same field of
/// `w_allbuf_opt`. The two are the same type, so their fields sit at the
/// same offsets *within* a `winopt_T`; the distance between the two copies
/// is therefore the distance between the copies themselves, whichever order
/// `win_T` happens to store them in. `get_varp_scope_from` walks it rather
/// than repeating the whole field table for the `:setglobal` case.
///
/// It is deliberately not `size_of::<winopt_T>()`: `win_T` has no
/// guaranteed layout, so the two copies need be neither adjacent nor in
/// declaration order.
const ALLBUF_OFFSET: isize = {
    let one = offset_of!(win_T, w_onebuf_opt) as isize;
    let all = offset_of!(win_T, w_allbuf_opt) as isize;
    all - one
};

/// An option's storage in one scope, with the type its row declares.
///
/// The plumbing used to answer a bare `*mut c_void` and let every reader
/// re-derive the type from the table — the storage indirection that pinned
/// an option's field to a raw scalar whatever its struct's `repr`, and made
/// "which variable is this" an address comparison. The arm carries the
/// type, so the field addresses below need no `.cast()` and a field whose
/// type disagrees with its option's does not compile.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum OptSlot {
    /// The option has no variable in this scope.
    None,
    /// A boolean option's tri-state `int`: 0 false, 1 true, -1 unset here.
    Boolean(*mut c_int),
    /// A number option's `OptInt`.
    Number(*mut OptInt),
    /// A string option's `char *`. Never null; an unset one holds the
    /// shared empty string.
    String(*mut *mut c_char),
}

impl From<*mut c_int> for OptSlot {
    fn from(var: *mut c_int) -> Self {
        OptSlot::Boolean(var)
    }
}

impl From<*mut OptInt> for OptSlot {
    fn from(var: *mut OptInt) -> Self {
        OptSlot::Number(var)
    }
}

impl From<*mut *mut c_char> for OptSlot {
    fn from(var: *mut *mut c_char) -> Self {
        OptSlot::String(var)
    }
}

impl OptSlot {
    /// The slot a raw `varp` is, given the option whose variable it is. The
    /// last untyped edge: [`option_default_var`] answers the address of a
    /// hidden option's own default, which is a `void *` because the defaults
    /// table is one union per row.
    fn from_raw(opt_idx: OptIndex, varp: *mut c_void) -> Self {
        if varp.is_null() {
            return OptSlot::None;
        }
        match super::option_get_type(opt_idx) {
            kOptValTypeBoolean => OptSlot::Boolean(varp.cast::<c_int>()),
            kOptValTypeNumber => OptSlot::Number(varp.cast::<OptInt>()),
            kOptValTypeString => OptSlot::String(varp.cast::<*mut c_char>()),
            type_0 => unreachable!("option value type {type_0}"),
        }
    }

    /// Whether the option has no variable in this scope.
    pub(crate) fn is_none(self) -> bool {
        matches!(self, OptSlot::None)
    }

    /// A boolean option's variable. The callers that reach for one have
    /// already established the option's type — from `option_has_type`, from
    /// the `did_set_*` they are, or from the row itself — and the table's
    /// compile-time assertion is what ties that type to this arm.
    pub(crate) fn boolean_var(self) -> *mut c_int {
        match self {
            OptSlot::Boolean(var) => var,
            _ => unreachable!("the option is not a boolean option"),
        }
    }

    /// A number option's variable. See [`boolean_var`](Self::boolean_var).
    pub(crate) fn number_var(self) -> *mut OptInt {
        match self {
            OptSlot::Number(var) => var,
            _ => unreachable!("the option is not a number option"),
        }
    }

    /// A string option's variable. See [`boolean_var`](Self::boolean_var).
    pub(crate) fn string_var(self) -> *mut *mut c_char {
        match self {
            OptSlot::String(var) => var,
            _ => unreachable!("the option is not a string option"),
        }
    }

    /// The same field of the window's *other* `winopt_T`. See
    /// [`ALLBUF_OFFSET`].
    fn byte_offset(self, delta: isize) -> Self {
        match self {
            OptSlot::None => OptSlot::None,
            OptSlot::Boolean(var) => OptSlot::Boolean(var.wrapping_byte_offset(delta)),
            OptSlot::Number(var) => OptSlot::Number(var.wrapping_byte_offset(delta)),
            OptSlot::String(var) => OptSlot::String(var.wrapping_byte_offset(delta)),
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
        OptVar::Boolean(cell) => OptSlot::Boolean(cell.ptr()),
        OptVar::Number(cell) => OptSlot::Number(cell.ptr()),
        OptVar::String(cell) => OptSlot::String(cell.ptr()),
        OptVar::OwnDefault => OptSlot::from_raw(opt_idx, option_default_var(opt_idx)),
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

/// A global-local string variable, or the global one when the local value is
/// the empty string.
///
/// # Safety
///
/// `local` must point at the option's local string variable.
unsafe fn local_str(local: *mut *mut c_char, global: OptSlot) -> OptSlot {
    // SAFETY: an option's string variable is never null.
    if unsafe { **local } != 0 {
        OptSlot::String(local)
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
unsafe fn local_int(local: *mut c_int, global: OptSlot) -> OptSlot {
    // SAFETY: the caller's pointer is the option's local variable.
    if unsafe { *local } >= 0 {
        OptSlot::Boolean(local)
    } else {
        global
    }
}

/// As [`local_int`], for the wider `OptInt` variables.
///
/// # Safety
///
/// `local` must point at the option's local variable.
unsafe fn local_optint(local: *mut OptInt, global: OptSlot) -> OptSlot {
    // SAFETY: the caller's pointer is the option's local variable.
    if unsafe { *local } >= 0 {
        OptSlot::Number(local)
    } else {
        global
    }
}

/// The variable an explicit `:setglobal`/`:setlocal` reaches, given the
/// buffer and window that stand for "local".
///
/// # Safety
///
/// `buf` and `win` must be live.
pub(crate) unsafe fn get_varp_scope_from(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    buf: *mut buf_T,
    win: *mut win_T,
) -> OptSlot {
    // SAFETY: the caller's pointers are live.
    unsafe {
        if opt_flags.has(OptionSetFlags::GLOBAL) && !option_is_global_only(opt_idx) {
            // A window-local option's global copy is its own field in the
            // window's second `winopt_T`, not the table's `var`.
            if option_is_window_local(opt_idx) {
                return get_varp_from(opt_idx, buf, win).byte_offset(ALLBUF_OFFSET);
            }
            return option_var(opt_idx);
        }
        if opt_flags.has(OptionSetFlags::LOCAL) && option_is_global_local(opt_idx) {
            // The local variable itself, sentinel and all.
            return match opt_idx {
                kOptFormatprg => OptSlot::from(&raw mut (*buf).b_p_fp),
                kOptFsync => OptSlot::from(&raw mut (*buf).b_p_fs),
                kOptFindfunc => OptSlot::from(&raw mut (*buf).b_p_ffu),
                kOptErrorformat => OptSlot::from(&raw mut (*buf).b_p_efm),
                kOptGrepformat => OptSlot::from(&raw mut (*buf).b_p_gefm),
                kOptGrepprg => OptSlot::from(&raw mut (*buf).b_p_gp),
                kOptMakeprg => OptSlot::from(&raw mut (*buf).b_p_mp),
                kOptEqualprg => OptSlot::from(&raw mut (*buf).b_p_ep),
                kOptKeywordprg => OptSlot::from(&raw mut (*buf).b_p_kp),
                kOptPath => OptSlot::from(&raw mut (*buf).b_p_path),
                kOptAutocomplete => OptSlot::from(&raw mut (*buf).b_p_ac),
                kOptAutoread => OptSlot::from(&raw mut (*buf).b_p_ar),
                kOptTags => OptSlot::from(&raw mut (*buf).b_p_tags),
                kOptTagcase => OptSlot::from(&raw mut (*buf).b_p_tc),
                kOptSidescrolloff => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_siso),
                kOptScrolloff => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_so),
                kOptDefine => OptSlot::from(&raw mut (*buf).b_p_def),
                kOptInclude => OptSlot::from(&raw mut (*buf).b_p_inc),
                kOptCompleteopt => OptSlot::from(&raw mut (*buf).b_p_cot),
                kOptDictionary => OptSlot::from(&raw mut (*buf).b_p_dict),
                kOptDiffanchors => OptSlot::from(&raw mut (*buf).b_p_dia),
                kOptThesaurus => OptSlot::from(&raw mut (*buf).b_p_tsr),
                kOptThesaurusfunc => OptSlot::from(&raw mut (*buf).b_p_tsrfu),
                kOptTagfunc => OptSlot::from(&raw mut (*buf).b_p_tfu),
                kOptShowbreak => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_sbr),
                kOptStatusline => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_stl),
                kOptWinbar => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_wbr),
                kOptUndolevels => OptSlot::from(&raw mut (*buf).b_p_ul),
                kOptLispwords => OptSlot::from(&raw mut (*buf).b_p_lw),
                kOptBackupcopy => OptSlot::from(&raw mut (*buf).b_p_bkc),
                kOptMakeencoding => OptSlot::from(&raw mut (*buf).b_p_menc),
                kOptFillchars => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_fcs),
                kOptListchars => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_lcs),
                kOptVirtualedit => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_ve),
                _ => unreachable!("option {opt_idx} has no local variable"),
            };
        }
        get_varp_from(opt_idx, buf, win)
    }
}

/// [`get_varp_scope_from`] for the current buffer and window.
pub(crate) fn get_varp_scope(opt_idx: OptIndex, opt_flags: OptionSetFlags) -> OptSlot {
    // SAFETY: `curbuf`/`curwin` are live.
    unsafe { get_varp_scope_from(opt_idx, opt_flags, curbuf.get(), curwin.get()) }
}

/// The variable the option reads from right now, for the given buffer and
/// window: the local one where it is set, the global one otherwise.
///
/// # Safety
///
/// `buf` and `win` must be live, and `win->w_s` must be set for the four
/// 'spell*' options.
pub(crate) unsafe fn get_varp_from(opt_idx: OptIndex, buf: *mut buf_T, win: *mut win_T) -> OptSlot {
    // SAFETY: the caller's pointers are live.
    unsafe {
        let global = option_var(opt_idx);
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
                    OptSlot::from(&raw mut (*buf).b_p_ul)
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
            kOptArabic => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_arab),
            kOptList => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_list),
            kOptSpell => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_spell),
            kOptCursorcolumn => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_cuc),
            kOptCursorline => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_cul),
            kOptCursorlineopt => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_culopt),
            kOptColorcolumn => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_cc),
            kOptDiff => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_diff),
            kOptEventignorewin => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_eiw),
            kOptFoldcolumn => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_fdc),
            kOptFoldenable => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_fen),
            kOptFoldignore => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_fdi),
            kOptFoldlevel => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_fdl),
            kOptFoldmethod => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_fdm),
            kOptFoldminlines => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_fml),
            kOptFoldnestmax => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_fdn),
            kOptFoldexpr => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_fde),
            kOptFoldtext => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_fdt),
            kOptFoldmarker => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_fmr),
            kOptNumber => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_nu),
            kOptRelativenumber => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_rnu),
            kOptNumberwidth => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_nuw),
            kOptWinfixbuf => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_wfb),
            kOptWinfixheight => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_wfh),
            kOptWinfixwidth => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_wfw),
            kOptPreviewwindow => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_pvw),
            kOptLhistory => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_lhi),
            kOptRightleft => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_rl),
            kOptRightleftcmd => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_rlc),
            kOptScroll => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_scr),
            kOptSmoothscroll => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_sms),
            kOptWrap => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_wrap),
            kOptLinebreak => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_lbr),
            kOptBreakindent => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_bri),
            kOptBreakindentopt => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_briopt),
            kOptScrollbind => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_scb),
            kOptCursorbind => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_crb),
            kOptConcealcursor => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_cocu),
            kOptConceallevel => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_cole),
            kOptSigncolumn => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_scl),
            kOptWinhighlight => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_winhl),
            kOptWinblend => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_winbl),
            kOptStatuscolumn => OptSlot::from(&raw mut (*win).w_onebuf_opt.wo_stc),

            // The 'spell*' options belong to the window's syntax block,
            // which a diff or preview window may share with another window.
            kOptSpellcapcheck => OptSlot::from(&raw mut (*(*win).w_s).b_p_spc),
            kOptSpellfile => OptSlot::from(&raw mut (*(*win).w_s).b_p_spf),
            kOptSpelllang => OptSlot::from(&raw mut (*(*win).w_s).b_p_spl),
            kOptSpelloptions => OptSlot::from(&raw mut (*(*win).w_s).b_p_spo),

            // Buffer-local.
            kOptAutoindent => OptSlot::from(&raw mut (*buf).b_p_ai),
            kOptBinary => OptSlot::from(&raw mut (*buf).b_p_bin),
            kOptBomb => OptSlot::from(&raw mut (*buf).b_p_bomb),
            kOptBufhidden => OptSlot::from(&raw mut (*buf).b_p_bh),
            kOptBuftype => OptSlot::from(&raw mut (*buf).b_p_bt),
            kOptBuflisted => OptSlot::from(&raw mut (*buf).b_p_bl),
            kOptBusy => OptSlot::from(&raw mut (*buf).b_p_busy),
            kOptChannel => OptSlot::from(&raw mut (*buf).b_p_channel),
            kOptCopyindent => OptSlot::from(&raw mut (*buf).b_p_ci),
            kOptCindent => OptSlot::from(&raw mut (*buf).b_p_cin),
            kOptCinkeys => OptSlot::from(&raw mut (*buf).b_p_cink),
            kOptCinoptions => OptSlot::from(&raw mut (*buf).b_p_cino),
            kOptCinscopedecls => OptSlot::from(&raw mut (*buf).b_p_cinsd),
            kOptCinwords => OptSlot::from(&raw mut (*buf).b_p_cinw),
            kOptComments => OptSlot::from(&raw mut (*buf).b_p_com),
            kOptCommentstring => OptSlot::from(&raw mut (*buf).b_p_cms),
            kOptComplete => OptSlot::from(&raw mut (*buf).b_p_cpt),
            kOptCompletefunc => OptSlot::from(&raw mut (*buf).b_p_cfu),
            kOptOmnifunc => OptSlot::from(&raw mut (*buf).b_p_ofu),
            kOptEndoffile => OptSlot::from(&raw mut (*buf).b_p_eof),
            kOptEndofline => OptSlot::from(&raw mut (*buf).b_p_eol),
            kOptFixendofline => OptSlot::from(&raw mut (*buf).b_p_fixeol),
            kOptExpandtab => OptSlot::from(&raw mut (*buf).b_p_et),
            kOptFileencoding => OptSlot::from(&raw mut (*buf).b_p_fenc),
            kOptFileformat => OptSlot::from(&raw mut (*buf).b_p_ff),
            kOptFiletype => OptSlot::from(&raw mut (*buf).b_p_ft),
            kOptFormatoptions => OptSlot::from(&raw mut (*buf).b_p_fo),
            kOptFormatlistpat => OptSlot::from(&raw mut (*buf).b_p_flp),
            kOptIminsert => OptSlot::from(&raw mut (*buf).b_p_iminsert),
            kOptImsearch => OptSlot::from(&raw mut (*buf).b_p_imsearch),
            kOptInfercase => OptSlot::from(&raw mut (*buf).b_p_inf),
            kOptIskeyword => OptSlot::from(&raw mut (*buf).b_p_isk),
            kOptIncludeexpr => OptSlot::from(&raw mut (*buf).b_p_inex),
            kOptIndentexpr => OptSlot::from(&raw mut (*buf).b_p_inde),
            kOptIndentkeys => OptSlot::from(&raw mut (*buf).b_p_indk),
            kOptFormatexpr => OptSlot::from(&raw mut (*buf).b_p_fex),
            kOptLisp => OptSlot::from(&raw mut (*buf).b_p_lisp),
            kOptLispoptions => OptSlot::from(&raw mut (*buf).b_p_lop),
            kOptModeline => OptSlot::from(&raw mut (*buf).b_p_ml),
            kOptMatchpairs => OptSlot::from(&raw mut (*buf).b_p_mps),
            kOptModifiable => OptSlot::from(&raw mut (*buf).b_p_ma),
            kOptModified => OptSlot::from(&raw mut (*buf).b_changed),
            kOptNrformats => OptSlot::from(&raw mut (*buf).b_p_nf),
            kOptPreserveindent => OptSlot::from(&raw mut (*buf).b_p_pi),
            kOptQuoteescape => OptSlot::from(&raw mut (*buf).b_p_qe),
            kOptReadonly => OptSlot::from(&raw mut (*buf).b_p_ro),
            kOptScrollback => OptSlot::from(&raw mut (*buf).b_p_scbk),
            kOptSmartindent => OptSlot::from(&raw mut (*buf).b_p_si),
            kOptSofttabstop => OptSlot::from(&raw mut (*buf).b_p_sts),
            kOptSuffixesadd => OptSlot::from(&raw mut (*buf).b_p_sua),
            kOptSwapfile => OptSlot::from(&raw mut (*buf).b_p_swf),
            kOptSynmaxcol => OptSlot::from(&raw mut (*buf).b_p_smc),
            kOptSyntax => OptSlot::from(&raw mut (*buf).b_p_syn),
            kOptShiftwidth => OptSlot::from(&raw mut (*buf).b_p_sw),
            kOptTagfunc => OptSlot::from(&raw mut (*buf).b_p_tfu),
            kOptTabstop => OptSlot::from(&raw mut (*buf).b_p_ts),
            kOptTextwidth => OptSlot::from(&raw mut (*buf).b_p_tw),
            kOptUndofile => OptSlot::from(&raw mut (*buf).b_p_udf),
            kOptWrapmargin => OptSlot::from(&raw mut (*buf).b_p_wm),
            kOptVarsofttabstop => OptSlot::from(&raw mut (*buf).b_p_vsts),
            kOptVartabstop => OptSlot::from(&raw mut (*buf).b_p_vts),
            kOptKeymap => OptSlot::from(&raw mut (*buf).b_p_keymap),

            _ => {
                iemsg(gettext(c"E356: get_varp ERROR".as_ptr()));
                // Upstream falls through to 'wrapmargin' rather than
                // returning null; every caller dereferences the result.
                OptSlot::from(&raw mut (*buf).b_p_wm)
            }
        }
    }
}

/// [`get_varp_from`] for the current buffer and window.
#[inline]
pub(crate) fn get_varp(opt_idx: OptIndex) -> OptSlot {
    // SAFETY: `curbuf`/`curwin` are live.
    unsafe { get_varp_from(opt_idx, curbuf.get(), curwin.get()) }
}
