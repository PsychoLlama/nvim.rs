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
    OptIndex, OptInt, OptScope, OptValType, OptVar, OptionSetFlags, buf_T, ssize_t, synblock_T,
    win_T,
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

/// The address of one field of a live object, computed rather than read.
///
/// A field's address is the object's plus a constant, so naming one needs no
/// dereference: `wrapping_byte_add` produces the address
/// `&raw mut (*base).field` would, in ordinary checked code and with the
/// whole object's provenance rather than the field's.
///
/// `witness` is never called. It is there so the field's *type* comes from
/// the field, which `offset_of!` erases — which is what keeps the
/// [`OptSlot`] arm tied to the declaration and a row whose type disagrees
/// with its option's from compiling.
pub(crate) fn field_ptr<T, F>(base: *mut T, offset: usize, _witness: fn(&T) -> &F) -> *mut F {
    base.wrapping_byte_add(offset).cast::<F>()
}

/// The [`OptSlot`] naming one field of the buffer `$buf` points at.
macro_rules! buf_var {
    ($buf:expr, $($field:ident).+) => {
        OptSlot::from(field_ptr(
            $buf,
            offset_of!(buf_T, $($field).+),
            |b: &buf_T| &b.$($field).+,
        ))
    };
}

/// [`buf_var`] for a window.
macro_rules! win_var {
    ($win:expr, $($field:ident).+) => {
        OptSlot::from(field_ptr(
            $win,
            offset_of!(win_T, $($field).+),
            |w: &win_T| &w.$($field).+,
        ))
    };
}

/// [`buf_var`] for the syntax block the four 'spell*' options live in.
macro_rules! syn_var {
    ($syn:expr, $($field:ident).+) => {
        OptSlot::from(field_ptr(
            $syn,
            offset_of!(synblock_T, $($field).+),
            |s: &synblock_T| &s.$($field).+,
        ))
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
    ///
    /// # Safety
    ///
    /// `local` must be the option's local variable in a live buffer or
    /// window.
    unsafe fn holds(self, local: OptSlot) -> bool {
        // SAFETY: the caller's variable, which every option that reaches
        // here has, and which is never null.
        match (self, local) {
            (Unset::Never, _) | (_, OptSlot::None) => false,
            (Unset::Sentinel, OptSlot::String(var)) => (unsafe { **var }) == 0,
            (Unset::Sentinel, OptSlot::Boolean(var)) => (unsafe { *var }) < 0,
            (Unset::Sentinel, OptSlot::Number(var)) => (unsafe { *var }) < 0,
            (Unset::NoLocalUndolevel, OptSlot::Number(var)) => {
                (unsafe { *var }) == NO_LOCAL_UNDOLEVEL as OptInt
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
    if opt_flags.has(OptionSetFlags::GLOBAL) && !option_is_global_only(opt_idx) {
        // A window-local option's global copy is its own field in the
        // window's second `winopt_T`, not the table's `var`.
        if option_is_window_local(opt_idx) {
            return unsafe { get_varp_from(opt_idx, buf, win) }.byte_offset(ALLBUF_OFFSET);
        }
        return option_var(opt_idx);
    }
    if opt_flags.has(OptionSetFlags::LOCAL) && option_is_global_local(opt_idx) {
        // The local variable itself, sentinel and all.
        return match opt_idx {
            kOptFormatprg => buf_var!(buf, b_p_fp),
            kOptFsync => buf_var!(buf, b_p_fs),
            kOptFindfunc => buf_var!(buf, b_p_ffu),
            kOptErrorformat => buf_var!(buf, b_p_efm),
            kOptGrepformat => buf_var!(buf, b_p_gefm),
            kOptGrepprg => buf_var!(buf, b_p_gp),
            kOptMakeprg => buf_var!(buf, b_p_mp),
            kOptEqualprg => buf_var!(buf, b_p_ep),
            kOptKeywordprg => buf_var!(buf, b_p_kp),
            kOptPath => buf_var!(buf, b_p_path),
            kOptAutocomplete => buf_var!(buf, b_p_ac),
            kOptAutoread => buf_var!(buf, b_p_ar),
            kOptTags => buf_var!(buf, b_p_tags),
            kOptTagcase => buf_var!(buf, b_p_tc),
            kOptSidescrolloff => win_var!(win, w_onebuf_opt.wo_siso),
            kOptScrolloff => win_var!(win, w_onebuf_opt.wo_so),
            kOptDefine => buf_var!(buf, b_p_def),
            kOptInclude => buf_var!(buf, b_p_inc),
            kOptCompleteopt => buf_var!(buf, b_p_cot),
            kOptDictionary => buf_var!(buf, b_p_dict),
            kOptDiffanchors => buf_var!(buf, b_p_dia),
            kOptThesaurus => buf_var!(buf, b_p_tsr),
            kOptThesaurusfunc => buf_var!(buf, b_p_tsrfu),
            kOptTagfunc => buf_var!(buf, b_p_tfu),
            kOptShowbreak => win_var!(win, w_onebuf_opt.wo_sbr),
            kOptStatusline => win_var!(win, w_onebuf_opt.wo_stl),
            kOptWinbar => win_var!(win, w_onebuf_opt.wo_wbr),
            kOptUndolevels => buf_var!(buf, b_p_ul),
            kOptLispwords => buf_var!(buf, b_p_lw),
            kOptBackupcopy => buf_var!(buf, b_p_bkc),
            kOptMakeencoding => buf_var!(buf, b_p_menc),
            kOptFillchars => win_var!(win, w_onebuf_opt.wo_fcs),
            kOptListchars => win_var!(win, w_onebuf_opt.wo_lcs),
            kOptVirtualedit => win_var!(win, w_onebuf_opt.wo_ve),
            _ => unreachable!("option {opt_idx} has no local variable"),
        };
    }
    unsafe { get_varp_from(opt_idx, buf, win) }
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
    let global = option_var(opt_idx);
    if is_option_hidden(opt_idx) || option_is_global_only(opt_idx) {
        return global;
    }
    // Which variable the option keeps its local value in, and what an
    // unset one would look like there. Naming a field reads nothing, so
    // the whole table is checked code; the one read is the sentinel test
    // below.
    let (local, unset) = match opt_idx {
        // Global-local: an unset local copy defers to the global one.
        kOptEqualprg => (buf_var!(buf, b_p_ep), Unset::Sentinel),
        kOptKeywordprg => (buf_var!(buf, b_p_kp), Unset::Sentinel),
        kOptPath => (buf_var!(buf, b_p_path), Unset::Sentinel),
        kOptAutocomplete => (buf_var!(buf, b_p_ac), Unset::Sentinel),
        kOptAutoread => (buf_var!(buf, b_p_ar), Unset::Sentinel),
        kOptTags => (buf_var!(buf, b_p_tags), Unset::Sentinel),
        kOptTagcase => (buf_var!(buf, b_p_tc), Unset::Sentinel),
        kOptSidescrolloff => (win_var!(win, w_onebuf_opt.wo_siso), Unset::Sentinel),
        kOptScrolloff => (win_var!(win, w_onebuf_opt.wo_so), Unset::Sentinel),
        kOptBackupcopy => (buf_var!(buf, b_p_bkc), Unset::Sentinel),
        kOptDefine => (buf_var!(buf, b_p_def), Unset::Sentinel),
        kOptInclude => (buf_var!(buf, b_p_inc), Unset::Sentinel),
        kOptCompleteopt => (buf_var!(buf, b_p_cot), Unset::Sentinel),
        kOptDictionary => (buf_var!(buf, b_p_dict), Unset::Sentinel),
        kOptDiffanchors => (buf_var!(buf, b_p_dia), Unset::Sentinel),
        kOptThesaurus => (buf_var!(buf, b_p_tsr), Unset::Sentinel),
        kOptThesaurusfunc => (buf_var!(buf, b_p_tsrfu), Unset::Sentinel),
        kOptFormatprg => (buf_var!(buf, b_p_fp), Unset::Sentinel),
        kOptFsync => (buf_var!(buf, b_p_fs), Unset::Sentinel),
        kOptFindfunc => (buf_var!(buf, b_p_ffu), Unset::Sentinel),
        kOptErrorformat => (buf_var!(buf, b_p_efm), Unset::Sentinel),
        kOptGrepformat => (buf_var!(buf, b_p_gefm), Unset::Sentinel),
        kOptGrepprg => (buf_var!(buf, b_p_gp), Unset::Sentinel),
        kOptMakeprg => (buf_var!(buf, b_p_mp), Unset::Sentinel),
        kOptShowbreak => (win_var!(win, w_onebuf_opt.wo_sbr), Unset::Sentinel),
        kOptStatusline => (win_var!(win, w_onebuf_opt.wo_stl), Unset::Sentinel),
        kOptWinbar => (win_var!(win, w_onebuf_opt.wo_wbr), Unset::Sentinel),
        // 'undolevels' has a sentinel of its own: 0 is a real value.
        kOptUndolevels => (buf_var!(buf, b_p_ul), Unset::NoLocalUndolevel),
        kOptLispwords => (buf_var!(buf, b_p_lw), Unset::Sentinel),
        kOptMakeencoding => (buf_var!(buf, b_p_menc), Unset::Sentinel),
        kOptFillchars => (win_var!(win, w_onebuf_opt.wo_fcs), Unset::Sentinel),
        kOptListchars => (win_var!(win, w_onebuf_opt.wo_lcs), Unset::Sentinel),
        kOptVirtualedit => (win_var!(win, w_onebuf_opt.wo_ve), Unset::Sentinel),

        // Window-local.
        kOptArabic => (win_var!(win, w_onebuf_opt.wo_arab), Unset::Never),
        kOptList => (win_var!(win, w_onebuf_opt.wo_list), Unset::Never),
        kOptSpell => (win_var!(win, w_onebuf_opt.wo_spell), Unset::Never),
        kOptCursorcolumn => (win_var!(win, w_onebuf_opt.wo_cuc), Unset::Never),
        kOptCursorline => (win_var!(win, w_onebuf_opt.wo_cul), Unset::Never),
        kOptCursorlineopt => (win_var!(win, w_onebuf_opt.wo_culopt), Unset::Never),
        kOptColorcolumn => (win_var!(win, w_onebuf_opt.wo_cc), Unset::Never),
        kOptDiff => (win_var!(win, w_onebuf_opt.wo_diff), Unset::Never),
        kOptEventignorewin => (win_var!(win, w_onebuf_opt.wo_eiw), Unset::Never),
        kOptFoldcolumn => (win_var!(win, w_onebuf_opt.wo_fdc), Unset::Never),
        kOptFoldenable => (win_var!(win, w_onebuf_opt.wo_fen), Unset::Never),
        kOptFoldignore => (win_var!(win, w_onebuf_opt.wo_fdi), Unset::Never),
        kOptFoldlevel => (win_var!(win, w_onebuf_opt.wo_fdl), Unset::Never),
        kOptFoldmethod => (win_var!(win, w_onebuf_opt.wo_fdm), Unset::Never),
        kOptFoldminlines => (win_var!(win, w_onebuf_opt.wo_fml), Unset::Never),
        kOptFoldnestmax => (win_var!(win, w_onebuf_opt.wo_fdn), Unset::Never),
        kOptFoldexpr => (win_var!(win, w_onebuf_opt.wo_fde), Unset::Never),
        kOptFoldtext => (win_var!(win, w_onebuf_opt.wo_fdt), Unset::Never),
        kOptFoldmarker => (win_var!(win, w_onebuf_opt.wo_fmr), Unset::Never),
        kOptNumber => (win_var!(win, w_onebuf_opt.wo_nu), Unset::Never),
        kOptRelativenumber => (win_var!(win, w_onebuf_opt.wo_rnu), Unset::Never),
        kOptNumberwidth => (win_var!(win, w_onebuf_opt.wo_nuw), Unset::Never),
        kOptWinfixbuf => (win_var!(win, w_onebuf_opt.wo_wfb), Unset::Never),
        kOptWinfixheight => (win_var!(win, w_onebuf_opt.wo_wfh), Unset::Never),
        kOptWinfixwidth => (win_var!(win, w_onebuf_opt.wo_wfw), Unset::Never),
        kOptPreviewwindow => (win_var!(win, w_onebuf_opt.wo_pvw), Unset::Never),
        kOptLhistory => (win_var!(win, w_onebuf_opt.wo_lhi), Unset::Never),
        kOptRightleft => (win_var!(win, w_onebuf_opt.wo_rl), Unset::Never),
        kOptRightleftcmd => (win_var!(win, w_onebuf_opt.wo_rlc), Unset::Never),
        kOptScroll => (win_var!(win, w_onebuf_opt.wo_scr), Unset::Never),
        kOptSmoothscroll => (win_var!(win, w_onebuf_opt.wo_sms), Unset::Never),
        kOptWrap => (win_var!(win, w_onebuf_opt.wo_wrap), Unset::Never),
        kOptLinebreak => (win_var!(win, w_onebuf_opt.wo_lbr), Unset::Never),
        kOptBreakindent => (win_var!(win, w_onebuf_opt.wo_bri), Unset::Never),
        kOptBreakindentopt => (win_var!(win, w_onebuf_opt.wo_briopt), Unset::Never),
        kOptScrollbind => (win_var!(win, w_onebuf_opt.wo_scb), Unset::Never),
        kOptCursorbind => (win_var!(win, w_onebuf_opt.wo_crb), Unset::Never),
        kOptConcealcursor => (win_var!(win, w_onebuf_opt.wo_cocu), Unset::Never),
        kOptConceallevel => (win_var!(win, w_onebuf_opt.wo_cole), Unset::Never),
        kOptSigncolumn => (win_var!(win, w_onebuf_opt.wo_scl), Unset::Never),
        kOptWinhighlight => (win_var!(win, w_onebuf_opt.wo_winhl), Unset::Never),
        kOptWinblend => (win_var!(win, w_onebuf_opt.wo_winbl), Unset::Never),
        kOptStatuscolumn => (win_var!(win, w_onebuf_opt.wo_stc), Unset::Never),

        // The 'spell*' options belong to the window's syntax block,
        // which a diff or preview window may share with another window.
        kOptSpellcapcheck => (syn_var!(unsafe { (*win).w_s }, b_p_spc), Unset::Never),
        kOptSpellfile => (syn_var!(unsafe { (*win).w_s }, b_p_spf), Unset::Never),
        kOptSpelllang => (syn_var!(unsafe { (*win).w_s }, b_p_spl), Unset::Never),
        kOptSpelloptions => (syn_var!(unsafe { (*win).w_s }, b_p_spo), Unset::Never),

        // Buffer-local.
        kOptAutoindent => (buf_var!(buf, b_p_ai), Unset::Never),
        kOptBinary => (buf_var!(buf, b_p_bin), Unset::Never),
        kOptBomb => (buf_var!(buf, b_p_bomb), Unset::Never),
        kOptBufhidden => (buf_var!(buf, b_p_bh), Unset::Never),
        kOptBuftype => (buf_var!(buf, b_p_bt), Unset::Never),
        kOptBuflisted => (buf_var!(buf, b_p_bl), Unset::Never),
        kOptBusy => (buf_var!(buf, b_p_busy), Unset::Never),
        kOptChannel => (buf_var!(buf, b_p_channel), Unset::Never),
        kOptCopyindent => (buf_var!(buf, b_p_ci), Unset::Never),
        kOptCindent => (buf_var!(buf, b_p_cin), Unset::Never),
        kOptCinkeys => (buf_var!(buf, b_p_cink), Unset::Never),
        kOptCinoptions => (buf_var!(buf, b_p_cino), Unset::Never),
        kOptCinscopedecls => (buf_var!(buf, b_p_cinsd), Unset::Never),
        kOptCinwords => (buf_var!(buf, b_p_cinw), Unset::Never),
        kOptComments => (buf_var!(buf, b_p_com), Unset::Never),
        kOptCommentstring => (buf_var!(buf, b_p_cms), Unset::Never),
        kOptComplete => (buf_var!(buf, b_p_cpt), Unset::Never),
        kOptCompletefunc => (buf_var!(buf, b_p_cfu), Unset::Never),
        kOptOmnifunc => (buf_var!(buf, b_p_ofu), Unset::Never),
        kOptEndoffile => (buf_var!(buf, b_p_eof), Unset::Never),
        kOptEndofline => (buf_var!(buf, b_p_eol), Unset::Never),
        kOptFixendofline => (buf_var!(buf, b_p_fixeol), Unset::Never),
        kOptExpandtab => (buf_var!(buf, b_p_et), Unset::Never),
        kOptFileencoding => (buf_var!(buf, b_p_fenc), Unset::Never),
        kOptFileformat => (buf_var!(buf, b_p_ff), Unset::Never),
        kOptFiletype => (buf_var!(buf, b_p_ft), Unset::Never),
        kOptFormatoptions => (buf_var!(buf, b_p_fo), Unset::Never),
        kOptFormatlistpat => (buf_var!(buf, b_p_flp), Unset::Never),
        kOptIminsert => (buf_var!(buf, b_p_iminsert), Unset::Never),
        kOptImsearch => (buf_var!(buf, b_p_imsearch), Unset::Never),
        kOptInfercase => (buf_var!(buf, b_p_inf), Unset::Never),
        kOptIskeyword => (buf_var!(buf, b_p_isk), Unset::Never),
        kOptIncludeexpr => (buf_var!(buf, b_p_inex), Unset::Never),
        kOptIndentexpr => (buf_var!(buf, b_p_inde), Unset::Never),
        kOptIndentkeys => (buf_var!(buf, b_p_indk), Unset::Never),
        kOptFormatexpr => (buf_var!(buf, b_p_fex), Unset::Never),
        kOptLisp => (buf_var!(buf, b_p_lisp), Unset::Never),
        kOptLispoptions => (buf_var!(buf, b_p_lop), Unset::Never),
        kOptModeline => (buf_var!(buf, b_p_ml), Unset::Never),
        kOptMatchpairs => (buf_var!(buf, b_p_mps), Unset::Never),
        kOptModifiable => (buf_var!(buf, b_p_ma), Unset::Never),
        kOptModified => (buf_var!(buf, b_changed), Unset::Never),
        kOptNrformats => (buf_var!(buf, b_p_nf), Unset::Never),
        kOptPreserveindent => (buf_var!(buf, b_p_pi), Unset::Never),
        kOptQuoteescape => (buf_var!(buf, b_p_qe), Unset::Never),
        kOptReadonly => (buf_var!(buf, b_p_ro), Unset::Never),
        kOptScrollback => (buf_var!(buf, b_p_scbk), Unset::Never),
        kOptSmartindent => (buf_var!(buf, b_p_si), Unset::Never),
        kOptSofttabstop => (buf_var!(buf, b_p_sts), Unset::Never),
        kOptSuffixesadd => (buf_var!(buf, b_p_sua), Unset::Never),
        kOptSwapfile => (buf_var!(buf, b_p_swf), Unset::Never),
        kOptSynmaxcol => (buf_var!(buf, b_p_smc), Unset::Never),
        kOptSyntax => (buf_var!(buf, b_p_syn), Unset::Never),
        kOptShiftwidth => (buf_var!(buf, b_p_sw), Unset::Never),
        kOptTagfunc => (buf_var!(buf, b_p_tfu), Unset::Never),
        kOptTabstop => (buf_var!(buf, b_p_ts), Unset::Never),
        kOptTextwidth => (buf_var!(buf, b_p_tw), Unset::Never),
        kOptUndofile => (buf_var!(buf, b_p_udf), Unset::Never),
        kOptWrapmargin => (buf_var!(buf, b_p_wm), Unset::Never),
        kOptVarsofttabstop => (buf_var!(buf, b_p_vsts), Unset::Never),
        kOptVartabstop => (buf_var!(buf, b_p_vts), Unset::Never),
        kOptKeymap => (buf_var!(buf, b_p_keymap), Unset::Never),

        _ => {
            iemsg(gettext(c"E356: get_varp ERROR"));
            // Upstream falls through to 'wrapmargin' rather than
            // returning null; every caller dereferences the result.
            (buf_var!(buf, b_p_wm), Unset::Never)
        }
    };
    // SAFETY: `local` names a field of the caller's live buffer or window,
    // or of the syntax block `win->w_s` they promised is set.
    if unsafe { unset.holds(local) } {
        global
    } else {
        local
    }
}

/// [`get_varp_from`] for the current buffer and window.
#[inline]
pub(crate) fn get_varp(opt_idx: OptIndex) -> OptSlot {
    // SAFETY: `curbuf`/`curwin` are live.
    unsafe { get_varp_from(opt_idx, curbuf.get(), curwin.get()) }
}
