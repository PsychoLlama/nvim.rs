//! What an option carries that its table row cannot: the state that changes
//! while the editor runs.
//!
//! [`crate::options::options`] is what an option *is* — its name, its type,
//! the scopes it exists in, the flags that drive parsing and redraws, the
//! variable holding its value — and none of that moves after the compiler
//! lays it out. Four things about a row did move, though, which is why the
//! whole table used to be a `GlobalCell` that three dozen call sites reached
//! with `.ptr()`:
//!
//! - the **default**, which startup rewrites once it can expand `$VIMRUNTIME`
//!   and read the environment, and which `alloc_options_default` gives an
//!   allocation of its own so the computed ones can free what they replace;
//! - **`kOptFlagWasSet`**, raised the first time anything sets the option, so
//!   that a computed default does not overwrite what a script chose;
//! - **`kOptFlagInsecure`**, raised when the value came from a modeline, the
//!   sandbox or secure mode — for most options; the ones whose value is
//!   evaluated as an expression keep their own copy per window or buffer,
//!   which is what [`crate::option::insecure_flag`] chooses between;
//! - the **script context**, where the global value was last set from.
//!
//! Those four live here, one record per [`OptIndex`], reached only through
//! the accessors below. The table itself is then a `ConstTable`, indexed
//! rather than dereferenced.
//!
//! Every accessor is a `with`/`with_mut` whose closure reads or writes one
//! field and calls nothing: no autocommand, no Lua callback and no
//! `did_set_*` runs inside a borrow of this array.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_void;
use core::mem::offset_of;

use crate::global_cell::GlobalCell;
use crate::options::{kOptCount, kOptInvalid};
use crate::types::{OptIndex, OptVal, sctx_T, uint32_t, vimoption_T};

use super::{kOptFlagInsecure, kOptFlagWasSet};

/// One option's mutable state. See the module docs.
#[derive(Copy, Clone)]
struct OptionState {
    /// `kOptFlagWasSet` and `kOptFlagInsecure`, in their table positions so
    /// that the shared word and a window's or buffer's own read alike.
    flags: uint32_t,
    /// What `:set opt&` installs — the table's declared default until
    /// startup replaces it with an expanded, owned copy.
    default: OptVal,
    /// Where the global value was last set from.
    script_ctx: sctx_T,
}

/// Every option's state, indexed by `OptIndex`.
static STATE: GlobalCell<[OptionState; kOptCount as usize]> = GlobalCell::new(initial());

/// The state every option starts in: no flags, no script context, and the
/// default its generated row declares.
const fn initial() -> [OptionState; kOptCount as usize] {
    let table: [vimoption_T; kOptCount as usize] = crate::options::table();
    let mut state = [OptionState {
        flags: 0,
        default: table[0].def_val,
        script_ctx: sctx_T {
            sc_sid: 0,
            sc_seq: 0,
            sc_lnum: 0,
            sc_chan: 0,
        },
    }; kOptCount as usize];
    let mut i = 0;
    while i < state.len() {
        state[i].default = table[i].def_val;
        i += 1;
    }
    state
}

/// An option index as a slot in the array above. Every `OptIndex` that
/// reaches this module names a row, so the conversion cannot fail.
fn slot(opt_idx: OptIndex) -> usize {
    debug_assert!(opt_idx != kOptInvalid);
    usize::try_from(opt_idx).expect("an option index is never negative")
}

/// The default `:set opt&` would install, as the option currently holds it.
pub(crate) fn option_default(opt_idx: OptIndex) -> OptVal {
    STATE.with(|state| state[slot(opt_idx)].default)
}

/// Install a default, without freeing the one it replaces —
/// [`crate::option::change_option_default`] is the owning spelling and the
/// one everything outside startup wants.
pub(crate) fn store_option_default(opt_idx: OptIndex, value: OptVal) {
    STATE.with_mut(|state| state[slot(opt_idx)].default = value);
}

/// Where an immutable option reads its value from: its own default, in
/// place. It has no variable of its own, and nothing writes here — the set
/// is refused long before it could.
pub(crate) fn option_default_var(opt_idx: OptIndex) -> *mut c_void {
    // The one place the array's address is taken. Every offset lands inside
    // the element it indexes, so the arithmetic needs no `unsafe`: the walk
    // is `wrapping_*`, exactly as `option/scope.rs` reached the table row's
    // `def_val` before.
    STATE
        .ptr()
        .cast::<OptionState>()
        .wrapping_add(slot(opt_idx))
        .wrapping_byte_add(offset_of!(OptionState, default) + offset_of!(OptVal, data))
        .cast::<c_void>()
}

/// Where the option's global value was last set from.
pub(crate) fn option_last_set(opt_idx: OptIndex) -> sctx_T {
    STATE.with(|state| state[slot(opt_idx)].script_ctx)
}

/// Record where the option's global value was just set from.
pub(crate) fn set_option_last_set(opt_idx: OptIndex, script_ctx: sctx_T) {
    STATE.with_mut(|state| state[slot(opt_idx)].script_ctx = script_ctx);
}

/// Whether anything has ever set the option, in which case a computed
/// default must not overwrite what it chose.
pub(crate) fn option_was_set(opt_idx: OptIndex) -> bool {
    STATE.with(|state| state[slot(opt_idx)].flags & kOptFlagWasSet != 0)
}

/// Remember that the option has been set.
pub(crate) fn mark_option_was_set(opt_idx: OptIndex) {
    STATE.with_mut(|state| state[slot(opt_idx)].flags |= kOptFlagWasSet);
}

/// Forget that anything set the option — what `:set all&` does.
pub(crate) fn reset_option_was_set(opt_idx: OptIndex) {
    STATE.with_mut(|state| state[slot(opt_idx)].flags &= !kOptFlagWasSet);
}

/// Whether the option's shared insecure mark is raised.
pub(crate) fn option_is_insecure(opt_idx: OptIndex) -> bool {
    STATE.with(|state| state[slot(opt_idx)].flags & kOptFlagInsecure != 0)
}

/// Raise or lower the option's shared insecure mark.
pub(crate) fn set_option_insecure(opt_idx: OptIndex, insecure: bool) {
    STATE.with_mut(|state| {
        if insecure {
            state[slot(opt_idx)].flags |= kOptFlagInsecure;
        } else {
            state[slot(opt_idx)].flags &= !kOptFlagInsecure;
        }
    });
}
