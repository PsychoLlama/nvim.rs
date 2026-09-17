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
//! The result is an [`OptSlot`]: which of the three types the option is, and
//! then either the selector naming its field of the global record or the
//! address of a window's, buffer's or syntax block's own copy. [`super::value`]
//! is where it gets read or written.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};
use core::mem::offset_of;

use crate::message::iemsg;
use crate::os::cshim::gettext;
// The generated index enum: 176 of its `kOpt*` constants name an arm below.
use crate::memory::XString;
use crate::options::vars::{BoolOpt, NumOpt, StrOpt};
use crate::options::*;
use crate::optionstr::{empty_option, is_empty_option};
use crate::types::{
    Buffer, OptIndex, OptInt, OptScope, OptStr, OptVal, OptValType, OptVar, OptionSetFlags,
    SynBlock, Window, ssize_t,
};

use super::{
    NO_LOCAL_UNDOLEVEL, get_option, kOptScopeBuf, kOptScopeGlobal, kOptScopeWin,
    kOptValTypeBoolean, kOptValTypeNumber, kOptValTypeString, option_default, store_option_default,
};

/// The signed distance from a field of `w_onebuf_opt` to the same field of
/// `w_allbuf_opt`. The two are the same type, so their fields sit at the
/// same offsets *within* a `WinOpt`; the distance between the two copies
/// is therefore the distance between the copies themselves, whichever order
/// `Window` happens to store them in. `get_varp_scope_from` walks it rather
/// than repeating the whole field table for the `:setglobal` case.
///
/// It is deliberately not `size_of::<WinOpt>()`: `Window` has no
/// guaranteed layout, so the two copies need be neither adjacent nor in
/// declaration order.
const ALLBUF_OFFSET: isize = {
    let one = offset_of!(Window, w_onebuf_opt).cast_signed();
    let all = offset_of!(Window, w_allbuf_opt).cast_signed();
    all - one
};

/// One variable of one kind, wherever it lives.
///
/// A global value is a field of `Options`, named by the selector the table's
/// row holds; a local one is a field of a window, a buffer or a syntax block,
/// and is still an address — `crate::types::buffer`'s fields are a later
/// slice's. The pair is what lets a read or a write be one operation with one
/// `# Safety` clause instead of a match at every call site.
///
/// [`NumVar`], [`StrVar`] and [`BoolVar`] are the three; the third is spelled
/// out separately because its two halves disagree about the type.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum NumVar {
    /// The global value: a field of the option record.
    Global(NumOpt),
    /// A window's or buffer's own copy, or a negative sentinel meaning "not
    /// set here".
    Local(*mut OptInt),
    /// An immutable option has nowhere to keep a value, so it reads its own
    /// current default in place. See [`BoolVar::OwnDefault`].
    OwnDefault(OptIndex),
}

impl NumVar {
    /// What the variable holds.
    ///
    /// # Safety
    ///
    /// A `Local` must name a field of a live window, buffer or syntax block
    /// — what `get_varp`/`get_varp_scope` hand out.
    pub(crate) unsafe fn get(self) -> OptInt {
        match self {
            NumVar::Global(field) => field.get(),
            NumVar::OwnDefault(idx) => option_default(idx)
                .as_number()
                .expect("an immutable number option's default is a number"),
            // SAFETY: the caller's live field.
            NumVar::Local(var) => unsafe { *var },
        }
    }

    /// Overwrite the variable.
    ///
    /// # Safety
    ///
    /// As [`get`](Self::get).
    pub(crate) unsafe fn set(self, value: OptInt) {
        match self {
            NumVar::Global(field) => field.set(value),
            NumVar::OwnDefault(idx) => store_option_default(idx, OptVal::Number(value)),
            // SAFETY: the caller's live field.
            NumVar::Local(var) => unsafe { *var = value },
        }
    }

    /// The same field of the object `delta` bytes away — a window's other
    /// `WinOpt`. Only a local copy has one.
    fn byte_offset(self, delta: isize) -> Self {
        match self {
            NumVar::Global(_) | NumVar::OwnDefault(_) => {
                unreachable!("a global value has no second copy")
            }
            NumVar::Local(var) => NumVar::Local(var.wrapping_byte_offset(delta)),
        }
    }
}

/// A string option's variable.
///
/// The global value is a field of the option record and the option owns it;
/// a local copy is still a raw `char *` field of a window, a buffer or a
/// syntax block, which is the next slice's. This pair of operations is the
/// seam between the two: above it a string option's value changes hands as
/// an owned allocation, below it as the `char *` the option protocol and
/// [`OptVal`] still speak.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum StrVar {
    /// The global value: a field of the option record.
    Global(StrOpt),
    /// A window's, buffer's or syntax block's own copy.
    Local(*mut *mut c_char),
    /// An immutable option reads its own default in place. See
    /// [`BoolVar::OwnDefault`].
    OwnDefault(OptIndex),
}

impl StrVar {
    /// The value's bytes, as the pointer the option protocol reads.
    ///
    /// A variable that owns nothing answers the shared empty string, which
    /// is what upstream kept in the variable itself; the pointer is live
    /// until the variable is written.
    ///
    /// # Safety
    ///
    /// A `Local` must name a field of a live window, buffer or syntax block.
    pub(crate) unsafe fn get(self) -> *mut c_char {
        match self {
            // The option owns the buffer and goes on owning it; the caller
            // reads it and must not free it. Writing the variable is what
            // ends the pointer's life, and the option protocol's
            // `free_oldval` is where that is decided.
            StrVar::Global(field) => field.value_ptr(),
            StrVar::OwnDefault(idx) => option_default(idx)
                .as_string()
                .expect("an immutable string option's default is a string")
                .data(),
            // SAFETY: the caller's live field.
            StrVar::Local(var) => unsafe { *var },
        }
    }

    /// Overwrite the variable, taking over `value`, and answer the block
    /// that was there — which the caller now owns and must release.
    ///
    /// # Safety
    ///
    /// As [`get`](Self::get), and `value` must be the shared empty string or
    /// an allocation with one owner, which this takes over.
    pub(crate) unsafe fn replace(self, value: *mut c_char) -> *mut c_char {
        match self {
            StrVar::Global(field) => {
                // SAFETY: the caller's promise -- one owner. The shared empty
                // string is nobody's allocation, so it becomes the option
                // owning nothing rather than a block to adopt, and comes
                // back out as itself.
                let owned = (!is_empty_option(value)).then(|| unsafe { XString::from_raw(value) });
                field
                    .swap(owned)
                    .map_or_else(empty_option, XString::into_raw)
            }
            StrVar::OwnDefault(idx) => {
                // SAFETY: an option value is NUL-terminated.
                let len = unsafe { crate::cstr::bytes_at(value) }.len();
                let old = option_default(idx)
                    .as_string()
                    .expect("an immutable string option's default is a string")
                    .data();
                store_option_default(idx, OptVal::String(OptStr::from_raw_parts(value, len)));
                old
            }
            // SAFETY: the caller's live field.
            StrVar::Local(var) => unsafe {
                let old = *var;
                *var = value;
                old
            },
        }
    }

    /// See [`NumVar::byte_offset`].
    fn byte_offset(self, delta: isize) -> Self {
        match self {
            StrVar::Global(_) | StrVar::OwnDefault(_) => {
                unreachable!("a global value has no second copy")
            }
            StrVar::Local(var) => StrVar::Local(var.wrapping_byte_offset(delta)),
        }
    }
}

/// A boolean option's variable.
///
/// The two halves disagree about the type, which is why this one is written
/// out rather than taken from `variable!`: the global value is a `bool`, and
/// a local copy is the tri-state word upstream gave every boolean — 0 false,
/// 1 true, **-1 not set in this scope**.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum BoolVar {
    /// The global value: a field of the option record.
    Global(BoolOpt),
    /// A window's or buffer's own tri-state copy.
    Local(*mut c_int),
    /// An immutable option has nowhere to keep a value, so it reads its own
    /// current default in place — the `def_val` of its own row, as
    /// `crate::option::state` currently holds it. Upstream reached the same
    /// bytes through a `void *` into the defaults table.
    OwnDefault(OptIndex),
}

impl BoolVar {
    /// What the variable holds, as the tri-state word: 0, 1 or -1.
    ///
    /// # Safety
    ///
    /// A `Local` must name a field of a live window or buffer.
    pub(crate) unsafe fn get(self) -> c_int {
        match self {
            BoolVar::Global(field) => c_int::from(field.get()),
            BoolVar::OwnDefault(idx) => option_default(idx)
                .tristate()
                .expect("an immutable boolean option's default is a boolean"),
            // SAFETY: the caller's live field.
            BoolVar::Local(var) => unsafe { *var },
        }
    }

    /// Overwrite the variable with a tri-state word. A global value has no
    /// third state: -1 there would read back as true.
    ///
    /// # Safety
    ///
    /// As [`get`](Self::get).
    pub(crate) unsafe fn set(self, word: c_int) {
        match self {
            BoolVar::Global(field) => {
                debug_assert!(word == 0 || word == 1, "a global boolean is not tri-state");
                field.set(word != 0);
            }
            BoolVar::OwnDefault(idx) => {
                store_option_default(idx, OptVal::Boolean(super::tristate(word)));
            }
            // SAFETY: the caller's live field.
            BoolVar::Local(var) => unsafe { *var = word },
        }
    }

    /// See [`NumVar::byte_offset`].
    fn byte_offset(self, delta: isize) -> Self {
        match self {
            BoolVar::Global(_) | BoolVar::OwnDefault(_) => {
                unreachable!("a global value has no second copy")
            }
            BoolVar::Local(var) => BoolVar::Local(var.wrapping_byte_offset(delta)),
        }
    }
}

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
    /// A boolean option's variable.
    Boolean(BoolVar),
    /// A number option's variable.
    Number(NumVar),
    /// A string option's variable.
    String(StrVar),
}

impl From<*mut c_int> for OptSlot {
    fn from(var: *mut c_int) -> Self {
        OptSlot::Boolean(BoolVar::Local(var))
    }
}

impl From<*mut OptInt> for OptSlot {
    fn from(var: *mut OptInt) -> Self {
        OptSlot::Number(NumVar::Local(var))
    }
}

impl From<*mut *mut c_char> for OptSlot {
    fn from(var: *mut *mut c_char) -> Self {
        OptSlot::String(StrVar::Local(var))
    }
}

impl OptSlot {
    /// Whether the option has no variable in this scope.
    pub(crate) fn is_none(self) -> bool {
        matches!(self, OptSlot::None)
    }

    /// A boolean option's variable. The callers that reach for one have
    /// already established the option's type — from `option_has_type`, from
    /// the `did_set_*` they are, or from the row itself — and the table's
    /// compile-time assertion is what ties that type to this arm.
    pub(crate) fn boolean_var(self) -> BoolVar {
        match self {
            OptSlot::Boolean(var) => var,
            _ => unreachable!("the option is not a boolean option"),
        }
    }

    /// A number option's variable. See [`boolean_var`](Self::boolean_var).
    pub(crate) fn number_var(self) -> NumVar {
        match self {
            OptSlot::Number(var) => var,
            _ => unreachable!("the option is not a number option"),
        }
    }

    /// A string option's variable. See [`boolean_var`](Self::boolean_var).
    pub(crate) fn string_var(self) -> StrVar {
        match self {
            OptSlot::String(var) => var,
            _ => unreachable!("the option is not a string option"),
        }
    }

    /// The same field of the window's *other* `WinOpt`. See
    /// [`ALLBUF_OFFSET`].
    fn byte_offset(self, delta: isize) -> Self {
        match self {
            OptSlot::None => OptSlot::None,
            OptSlot::Boolean(var) => OptSlot::Boolean(var.byte_offset(delta)),
            OptSlot::Number(var) => OptSlot::Number(var.byte_offset(delta)),
            OptSlot::String(var) => OptSlot::String(var.byte_offset(delta)),
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
            offset_of!(Buffer, $($field).+),
            |b: &Buffer| &b.$($field).+,
        ))
    };
}

/// [`buf_var`] for a window.
macro_rules! win_var {
    ($win:expr, $($field:ident).+) => {
        OptSlot::from(field_ptr(
            $win,
            offset_of!(Window, $($field).+),
            |w: &Window| &w.$($field).+,
        ))
    };
}

/// [`buf_var`] for the syntax block the four 'spell*' options live in.
macro_rules! syn_var {
    ($syn:expr, $($field:ident).+) => {
        OptSlot::from(field_ptr(
            $syn,
            offset_of!(SynBlock, $($field).+),
            |s: &SynBlock| &s.$($field).+,
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
            (Unset::Sentinel, OptSlot::String(var)) => (unsafe { *var.get() }) == 0,
            (Unset::Sentinel, OptSlot::Boolean(var)) => (unsafe { var.get() }) < 0,
            (Unset::Sentinel, OptSlot::Number(var)) => (unsafe { var.get() }) < 0,
            (Unset::NoLocalUndolevel, OptSlot::Number(var)) => {
                (unsafe { var.get() }) == OptInt::from(NO_LOCAL_UNDOLEVEL)
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
    // SAFETY: the caller's pointers are live.
    if opt_flags.has(OptionSetFlags::GLOBAL) && !option_is_global_only(opt_idx) {
        // A window-local option's global copy is its own field in the
        // window's second `WinOpt`, not the table's `var`.
        if option_is_window_local(opt_idx) {
            return unsafe { get_varp_from(opt_idx, buffer, win) }.byte_offset(ALLBUF_OFFSET);
        }
        return option_var(opt_idx);
    }
    if opt_flags.has(OptionSetFlags::LOCAL) && option_is_global_local(opt_idx) {
        // The local variable itself, sentinel and all.
        return match opt_idx {
            kOptFormatprg => buf_var!(buffer.raw(), b_p_fp),
            kOptFsync => buf_var!(buffer.raw(), b_p_fs),
            kOptFindfunc => buf_var!(buffer.raw(), b_p_ffu),
            kOptErrorformat => buf_var!(buffer.raw(), b_p_efm),
            kOptGrepformat => buf_var!(buffer.raw(), b_p_gefm),
            kOptGrepprg => buf_var!(buffer.raw(), b_p_gp),
            kOptMakeprg => buf_var!(buffer.raw(), b_p_mp),
            kOptEqualprg => buf_var!(buffer.raw(), b_p_ep),
            kOptKeywordprg => buf_var!(buffer.raw(), b_p_kp),
            kOptPath => buf_var!(buffer.raw(), b_p_path),
            kOptAutocomplete => buf_var!(buffer.raw(), b_p_ac),
            kOptAutoread => buf_var!(buffer.raw(), b_p_ar),
            kOptTags => buf_var!(buffer.raw(), b_p_tags),
            kOptTagcase => buf_var!(buffer.raw(), b_p_tc),
            kOptSidescrolloff => win_var!(win.raw(), w_onebuf_opt.wo_siso),
            kOptScrolloff => win_var!(win.raw(), w_onebuf_opt.wo_so),
            kOptDefine => buf_var!(buffer.raw(), b_p_def),
            kOptInclude => buf_var!(buffer.raw(), b_p_inc),
            kOptCompleteopt => buf_var!(buffer.raw(), b_p_cot),
            kOptDictionary => buf_var!(buffer.raw(), b_p_dict),
            kOptDiffanchors => buf_var!(buffer.raw(), b_p_dia),
            kOptThesaurus => buf_var!(buffer.raw(), b_p_tsr),
            kOptThesaurusfunc => buf_var!(buffer.raw(), b_p_tsrfu),
            kOptTagfunc => buf_var!(buffer.raw(), b_p_tfu),
            kOptShowbreak => win_var!(win.raw(), w_onebuf_opt.wo_sbr),
            kOptStatusline => win_var!(win.raw(), w_onebuf_opt.wo_stl),
            kOptWinbar => win_var!(win.raw(), w_onebuf_opt.wo_wbr),
            kOptUndolevels => buf_var!(buffer.raw(), b_p_ul),
            kOptLispwords => buf_var!(buffer.raw(), b_p_lw),
            kOptBackupcopy => buf_var!(buffer.raw(), b_p_bkc),
            kOptMakeencoding => buf_var!(buffer.raw(), b_p_menc),
            kOptFillchars => win_var!(win.raw(), w_onebuf_opt.wo_fcs),
            kOptListchars => win_var!(win.raw(), w_onebuf_opt.wo_lcs),
            kOptVirtualedit => win_var!(win.raw(), w_onebuf_opt.wo_ve),
            _ => unreachable!("option {opt_idx} has no local variable"),
        };
    }
    unsafe { get_varp_from(opt_idx, buffer, win) }
}

/// [`get_varp_scope_from`] for the current buffer and window.
pub(crate) fn get_varp_scope(opt_idx: OptIndex, opt_flags: OptionSetFlags) -> OptSlot {
    let (buffer, win) = (Buf::current(), Win::current());
    get_varp_scope_from(opt_idx, opt_flags, buffer, win)
}

/// The variable the option reads from right now, for the given buffer and
/// window: the local one where it is set, the global one otherwise.
///
/// # Safety
///
/// `buffer` and `win` must be live, and `win->w_s` must be set for the four
/// 'spell*' options.
pub(crate) unsafe fn get_varp_from(opt_idx: OptIndex, buffer: Buf, win: Win) -> OptSlot {
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
        kOptEqualprg => (buf_var!(buffer.raw(), b_p_ep), Unset::Sentinel),
        kOptKeywordprg => (buf_var!(buffer.raw(), b_p_kp), Unset::Sentinel),
        kOptPath => (buf_var!(buffer.raw(), b_p_path), Unset::Sentinel),
        kOptAutocomplete => (buf_var!(buffer.raw(), b_p_ac), Unset::Sentinel),
        kOptAutoread => (buf_var!(buffer.raw(), b_p_ar), Unset::Sentinel),
        kOptTags => (buf_var!(buffer.raw(), b_p_tags), Unset::Sentinel),
        kOptTagcase => (buf_var!(buffer.raw(), b_p_tc), Unset::Sentinel),
        kOptSidescrolloff => (win_var!(win.raw(), w_onebuf_opt.wo_siso), Unset::Sentinel),
        kOptScrolloff => (win_var!(win.raw(), w_onebuf_opt.wo_so), Unset::Sentinel),
        kOptBackupcopy => (buf_var!(buffer.raw(), b_p_bkc), Unset::Sentinel),
        kOptDefine => (buf_var!(buffer.raw(), b_p_def), Unset::Sentinel),
        kOptInclude => (buf_var!(buffer.raw(), b_p_inc), Unset::Sentinel),
        kOptCompleteopt => (buf_var!(buffer.raw(), b_p_cot), Unset::Sentinel),
        kOptDictionary => (buf_var!(buffer.raw(), b_p_dict), Unset::Sentinel),
        kOptDiffanchors => (buf_var!(buffer.raw(), b_p_dia), Unset::Sentinel),
        kOptThesaurus => (buf_var!(buffer.raw(), b_p_tsr), Unset::Sentinel),
        kOptThesaurusfunc => (buf_var!(buffer.raw(), b_p_tsrfu), Unset::Sentinel),
        kOptFormatprg => (buf_var!(buffer.raw(), b_p_fp), Unset::Sentinel),
        kOptFsync => (buf_var!(buffer.raw(), b_p_fs), Unset::Sentinel),
        kOptFindfunc => (buf_var!(buffer.raw(), b_p_ffu), Unset::Sentinel),
        kOptErrorformat => (buf_var!(buffer.raw(), b_p_efm), Unset::Sentinel),
        kOptGrepformat => (buf_var!(buffer.raw(), b_p_gefm), Unset::Sentinel),
        kOptGrepprg => (buf_var!(buffer.raw(), b_p_gp), Unset::Sentinel),
        kOptMakeprg => (buf_var!(buffer.raw(), b_p_mp), Unset::Sentinel),
        kOptShowbreak => (win_var!(win.raw(), w_onebuf_opt.wo_sbr), Unset::Sentinel),
        kOptStatusline => (win_var!(win.raw(), w_onebuf_opt.wo_stl), Unset::Sentinel),
        kOptWinbar => (win_var!(win.raw(), w_onebuf_opt.wo_wbr), Unset::Sentinel),
        // 'undolevels' has a sentinel of its own: 0 is a real value.
        kOptUndolevels => (buf_var!(buffer.raw(), b_p_ul), Unset::NoLocalUndolevel),
        kOptLispwords => (buf_var!(buffer.raw(), b_p_lw), Unset::Sentinel),
        kOptMakeencoding => (buf_var!(buffer.raw(), b_p_menc), Unset::Sentinel),
        kOptFillchars => (win_var!(win.raw(), w_onebuf_opt.wo_fcs), Unset::Sentinel),
        kOptListchars => (win_var!(win.raw(), w_onebuf_opt.wo_lcs), Unset::Sentinel),
        kOptVirtualedit => (win_var!(win.raw(), w_onebuf_opt.wo_ve), Unset::Sentinel),

        // Window-local.
        kOptArabic => (win_var!(win.raw(), w_onebuf_opt.wo_arab), Unset::Never),
        kOptList => (win_var!(win.raw(), w_onebuf_opt.wo_list), Unset::Never),
        kOptSpell => (win_var!(win.raw(), w_onebuf_opt.wo_spell), Unset::Never),
        kOptCursorcolumn => (win_var!(win.raw(), w_onebuf_opt.wo_cuc), Unset::Never),
        kOptCursorline => (win_var!(win.raw(), w_onebuf_opt.wo_cul), Unset::Never),
        kOptCursorlineopt => (win_var!(win.raw(), w_onebuf_opt.wo_culopt), Unset::Never),
        kOptColorcolumn => (win_var!(win.raw(), w_onebuf_opt.wo_cc), Unset::Never),
        kOptDiff => (win_var!(win.raw(), w_onebuf_opt.wo_diff), Unset::Never),
        kOptEventignorewin => (win_var!(win.raw(), w_onebuf_opt.wo_eiw), Unset::Never),
        kOptFoldcolumn => (win_var!(win.raw(), w_onebuf_opt.wo_fdc), Unset::Never),
        kOptFoldenable => (win_var!(win.raw(), w_onebuf_opt.wo_fen), Unset::Never),
        kOptFoldignore => (win_var!(win.raw(), w_onebuf_opt.wo_fdi), Unset::Never),
        kOptFoldlevel => (win_var!(win.raw(), w_onebuf_opt.wo_fdl), Unset::Never),
        kOptFoldmethod => (win_var!(win.raw(), w_onebuf_opt.wo_fdm), Unset::Never),
        kOptFoldminlines => (win_var!(win.raw(), w_onebuf_opt.wo_fml), Unset::Never),
        kOptFoldnestmax => (win_var!(win.raw(), w_onebuf_opt.wo_fdn), Unset::Never),
        kOptFoldexpr => (win_var!(win.raw(), w_onebuf_opt.wo_fde), Unset::Never),
        kOptFoldtext => (win_var!(win.raw(), w_onebuf_opt.wo_fdt), Unset::Never),
        kOptFoldmarker => (win_var!(win.raw(), w_onebuf_opt.wo_fmr), Unset::Never),
        kOptNumber => (win_var!(win.raw(), w_onebuf_opt.wo_nu), Unset::Never),
        kOptRelativenumber => (win_var!(win.raw(), w_onebuf_opt.wo_rnu), Unset::Never),
        kOptNumberwidth => (win_var!(win.raw(), w_onebuf_opt.wo_nuw), Unset::Never),
        kOptWinfixbuf => (win_var!(win.raw(), w_onebuf_opt.wo_wfb), Unset::Never),
        kOptWinfixheight => (win_var!(win.raw(), w_onebuf_opt.wo_wfh), Unset::Never),
        kOptWinfixwidth => (win_var!(win.raw(), w_onebuf_opt.wo_wfw), Unset::Never),
        kOptPreviewwindow => (win_var!(win.raw(), w_onebuf_opt.wo_pvw), Unset::Never),
        kOptLhistory => (win_var!(win.raw(), w_onebuf_opt.wo_lhi), Unset::Never),
        kOptRightleft => (win_var!(win.raw(), w_onebuf_opt.wo_rl), Unset::Never),
        kOptRightleftcmd => (win_var!(win.raw(), w_onebuf_opt.wo_rlc), Unset::Never),
        kOptScroll => (win_var!(win.raw(), w_onebuf_opt.wo_scr), Unset::Never),
        kOptSmoothscroll => (win_var!(win.raw(), w_onebuf_opt.wo_sms), Unset::Never),
        kOptWrap => (win_var!(win.raw(), w_onebuf_opt.wo_wrap), Unset::Never),
        kOptLinebreak => (win_var!(win.raw(), w_onebuf_opt.wo_lbr), Unset::Never),
        kOptBreakindent => (win_var!(win.raw(), w_onebuf_opt.wo_bri), Unset::Never),
        kOptBreakindentopt => (win_var!(win.raw(), w_onebuf_opt.wo_briopt), Unset::Never),
        kOptScrollbind => (win_var!(win.raw(), w_onebuf_opt.wo_scb), Unset::Never),
        kOptCursorbind => (win_var!(win.raw(), w_onebuf_opt.wo_crb), Unset::Never),
        kOptConcealcursor => (win_var!(win.raw(), w_onebuf_opt.wo_cocu), Unset::Never),
        kOptConceallevel => (win_var!(win.raw(), w_onebuf_opt.wo_cole), Unset::Never),
        kOptSigncolumn => (win_var!(win.raw(), w_onebuf_opt.wo_scl), Unset::Never),
        kOptWinhighlight => (win_var!(win.raw(), w_onebuf_opt.wo_winhl), Unset::Never),
        kOptWinblend => (win_var!(win.raw(), w_onebuf_opt.wo_winbl), Unset::Never),
        kOptStatuscolumn => (win_var!(win.raw(), w_onebuf_opt.wo_stc), Unset::Never),

        // The 'spell*' options belong to the window's syntax block,
        // which a diff or preview window may share with another window.
        kOptSpellcapcheck => (syn_var!(win.w_s, b_p_spc), Unset::Never),
        kOptSpellfile => (syn_var!(win.w_s, b_p_spf), Unset::Never),
        kOptSpelllang => (syn_var!(win.w_s, b_p_spl), Unset::Never),
        kOptSpelloptions => (syn_var!(win.w_s, b_p_spo), Unset::Never),

        // Buffer-local.
        kOptAutoindent => (buf_var!(buffer.raw(), b_p_ai), Unset::Never),
        kOptBinary => (buf_var!(buffer.raw(), b_p_bin), Unset::Never),
        kOptBomb => (buf_var!(buffer.raw(), b_p_bomb), Unset::Never),
        kOptBufhidden => (buf_var!(buffer.raw(), b_p_bh), Unset::Never),
        kOptBuftype => (buf_var!(buffer.raw(), b_p_bt), Unset::Never),
        kOptBuflisted => (buf_var!(buffer.raw(), b_p_bl), Unset::Never),
        kOptBusy => (buf_var!(buffer.raw(), b_p_busy), Unset::Never),
        kOptChannel => (buf_var!(buffer.raw(), b_p_channel), Unset::Never),
        kOptCopyindent => (buf_var!(buffer.raw(), b_p_ci), Unset::Never),
        kOptCindent => (buf_var!(buffer.raw(), b_p_cin), Unset::Never),
        kOptCinkeys => (buf_var!(buffer.raw(), b_p_cink), Unset::Never),
        kOptCinoptions => (buf_var!(buffer.raw(), b_p_cino), Unset::Never),
        kOptCinscopedecls => (buf_var!(buffer.raw(), b_p_cinsd), Unset::Never),
        kOptCinwords => (buf_var!(buffer.raw(), b_p_cinw), Unset::Never),
        kOptComments => (buf_var!(buffer.raw(), b_p_com), Unset::Never),
        kOptCommentstring => (buf_var!(buffer.raw(), b_p_cms), Unset::Never),
        kOptComplete => (buf_var!(buffer.raw(), b_p_cpt), Unset::Never),
        kOptCompletefunc => (buf_var!(buffer.raw(), b_p_cfu), Unset::Never),
        kOptOmnifunc => (buf_var!(buffer.raw(), b_p_ofu), Unset::Never),
        kOptEndoffile => (buf_var!(buffer.raw(), b_p_eof), Unset::Never),
        kOptEndofline => (buf_var!(buffer.raw(), b_p_eol), Unset::Never),
        kOptFixendofline => (buf_var!(buffer.raw(), b_p_fixeol), Unset::Never),
        kOptExpandtab => (buf_var!(buffer.raw(), b_p_et), Unset::Never),
        kOptFileencoding => (buf_var!(buffer.raw(), b_p_fenc), Unset::Never),
        kOptFileformat => (buf_var!(buffer.raw(), b_p_ff), Unset::Never),
        kOptFiletype => (buf_var!(buffer.raw(), b_p_ft), Unset::Never),
        kOptFormatoptions => (buf_var!(buffer.raw(), b_p_fo), Unset::Never),
        kOptFormatlistpat => (buf_var!(buffer.raw(), b_p_flp), Unset::Never),
        kOptIminsert => (buf_var!(buffer.raw(), b_p_iminsert), Unset::Never),
        kOptImsearch => (buf_var!(buffer.raw(), b_p_imsearch), Unset::Never),
        kOptInfercase => (buf_var!(buffer.raw(), b_p_inf), Unset::Never),
        kOptIskeyword => (buf_var!(buffer.raw(), b_p_isk), Unset::Never),
        kOptIncludeexpr => (buf_var!(buffer.raw(), b_p_inex), Unset::Never),
        kOptIndentexpr => (buf_var!(buffer.raw(), b_p_inde), Unset::Never),
        kOptIndentkeys => (buf_var!(buffer.raw(), b_p_indk), Unset::Never),
        kOptFormatexpr => (buf_var!(buffer.raw(), b_p_fex), Unset::Never),
        kOptLisp => (buf_var!(buffer.raw(), b_p_lisp), Unset::Never),
        kOptLispoptions => (buf_var!(buffer.raw(), b_p_lop), Unset::Never),
        kOptModeline => (buf_var!(buffer.raw(), b_p_ml), Unset::Never),
        kOptMatchpairs => (buf_var!(buffer.raw(), b_p_mps), Unset::Never),
        kOptModifiable => (buf_var!(buffer.raw(), b_p_ma), Unset::Never),
        kOptModified => (buf_var!(buffer.raw(), b_changed), Unset::Never),
        kOptNrformats => (buf_var!(buffer.raw(), b_p_nf), Unset::Never),
        kOptPreserveindent => (buf_var!(buffer.raw(), b_p_pi), Unset::Never),
        kOptQuoteescape => (buf_var!(buffer.raw(), b_p_qe), Unset::Never),
        kOptReadonly => (buf_var!(buffer.raw(), b_p_ro), Unset::Never),
        kOptScrollback => (buf_var!(buffer.raw(), b_p_scbk), Unset::Never),
        kOptSmartindent => (buf_var!(buffer.raw(), b_p_si), Unset::Never),
        kOptSofttabstop => (buf_var!(buffer.raw(), b_p_sts), Unset::Never),
        kOptSuffixesadd => (buf_var!(buffer.raw(), b_p_sua), Unset::Never),
        kOptSwapfile => (buf_var!(buffer.raw(), b_p_swf), Unset::Never),
        kOptSynmaxcol => (buf_var!(buffer.raw(), b_p_smc), Unset::Never),
        kOptSyntax => (buf_var!(buffer.raw(), b_p_syn), Unset::Never),
        kOptShiftwidth => (buf_var!(buffer.raw(), b_p_sw), Unset::Never),
        kOptTagfunc => (buf_var!(buffer.raw(), b_p_tfu), Unset::Never),
        kOptTabstop => (buf_var!(buffer.raw(), b_p_ts), Unset::Never),
        kOptTextwidth => (buf_var!(buffer.raw(), b_p_tw), Unset::Never),
        kOptUndofile => (buf_var!(buffer.raw(), b_p_udf), Unset::Never),
        kOptWrapmargin => (buf_var!(buffer.raw(), b_p_wm), Unset::Never),
        kOptVarsofttabstop => (buf_var!(buffer.raw(), b_p_vsts), Unset::Never),
        kOptVartabstop => (buf_var!(buffer.raw(), b_p_vts), Unset::Never),
        kOptKeymap => (buf_var!(buffer.raw(), b_p_keymap), Unset::Never),

        _ => {
            iemsg(gettext(c"E356: get_varp ERROR"));
            // Upstream falls through to 'wrapmargin' rather than
            // returning null; every caller dereferences the result.
            (buf_var!(buffer.raw(), b_p_wm), Unset::Never)
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
    unsafe { get_varp_from(opt_idx, Buf::current(), Win::current()) }
}
