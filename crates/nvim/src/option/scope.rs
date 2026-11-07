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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::option::kOptValTypeBoolean;
use crate::types::option::kOptValTypeNumber;
use crate::types::option::kOptValTypeString;
use core::ffi::c_int;

use crate::message::iemsg;
use crate::os::cshim::gettext;
// The generated index enum and `local_var`, the ladder naming each local
// option's field.
use crate::options::*;
use crate::optionstr::OptString;
use crate::os::libc::ssize_t;
use crate::types::{OptIndex, OptInt, OptScope, OptValType, OptVar, OptionSetFlags};
use crate::winlayer::{Buf, Win};

use super::{
    BoolVar, NO_LOCAL_UNDOLEVEL, NumVar, OptSlot, StrVar, WinOptSet, get_option, kOptScopeBuf,
    kOptScopeGlobal, kOptScopeWin,
};

/// The [`OptSlot`] naming one field of `$buf`. The generated ladder
/// (`crate::options::local_var`) names every local option's field with these.
macro_rules! buf_var {
    ($buf:expr, $field:ident) => {
        $crate::option::OptSlot::from($crate::option::Local::Buf(
            $buf,
            const { $crate::global_cell::field!($crate::types::Buffer, $field) },
        ))
    };
}
pub(crate) use buf_var;

/// [`buf_var`] for a field of a window's `w_onebuf_opt`.
macro_rules! win_var {
    ($win:expr, $field:ident) => {
        $crate::option::OptSlot::from($crate::option::Local::Win(
            $win,
            $crate::option::WinOptSet::One,
            const { $crate::global_cell::field!($crate::types::WinOpt, $field) },
        ))
    };
}
pub(crate) use win_var;

/// [`buf_var`] for the syntax block the four 'spell*' options live in,
/// which is reached through the window.
macro_rules! syn_var {
    ($win:expr, $field:ident) => {
        $crate::option::OptSlot::from($crate::option::Local::Syn(
            $win,
            const { $crate::global_cell::field!($crate::types::SynBlock, $field) },
        ))
    };
}
pub(crate) use syn_var;

/// What "not set here" looks like in a global-local option's local copy.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Unset {
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
            (Unset::Sentinel, OptSlot::String(var)) => var.first_byte() == 0,
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
/// This is the only place [`OptVar`] becomes a slot.
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
