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
//!   and read the environment -- `None` until then, which reads as the
//!   literal the table declares;
//! - **`kOptFlagWasSet`**, raised the first time anything sets the option, so
//!   that a computed default does not overwrite what a script chose;
//! - **`kOptFlagInsecure`**, raised when the value came from a modeline, the
//!   sandbox or secure mode — for most options; the ones whose value is
//!   evaluated as an expression keep their own copy per window or buffer,
//!   which is what [`crate::option::insecure_flag`] chooses between;
//! - the **script context**, where the global value was last set from.
//!
//! Those four live here, one record per [`OptIndex`], reached only through
//! the accessors below. An immutable option reads its default *as* its
//! value ([`OptSlot::OwnDefault`](crate::option::OptSlot)), which is why
//! `option_default` is the whole of the read path for one. The table itself is then a `ConstTable`, indexed
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

use crate::global_cell::GlobalCell;
use crate::options::{kOptCount, kOptInvalid, options};
use crate::types::{OptIndex, OptVal, ScriptCtx, uint32_t};

use super::{kOptFlagInsecure, kOptFlagWasSet};

/// One option's mutable state. See the module docs.
struct OptionState {
    /// `kOptFlagWasSet` and `kOptFlagInsecure`, in their table positions so
    /// that the shared word and a window's or buffer's own read alike.
    flags: uint32_t,
    /// What `:set opt&` installs, once something has given the option a
    /// default of its own; `None` is the table's declared one.
    default: Option<OptVal>,
    /// Where the global value was last set from.
    script_ctx: ScriptCtx,
}

impl OptionState {
    /// The state every option starts in: no flags, no script context, and
    /// the default its generated row declares.
    const INITIAL: OptionState = OptionState {
        flags: 0,
        default: None,
        script_ctx: ScriptCtx::NONE,
    };
}

/// Every option's state, indexed by `OptIndex`.
static STATE: GlobalCell<[OptionState; kOptCount as usize]> =
    GlobalCell::new([OptionState::INITIAL; kOptCount as usize]);

/// An option index as a slot in the array above. Every `OptIndex` that
/// reaches this module names a row, so the conversion cannot fail.
fn slot(opt_idx: OptIndex) -> usize {
    debug_assert!(opt_idx != kOptInvalid);
    usize::try_from(opt_idx).expect("an option index is never negative")
}

/// `f` over the default `:set opt&` would install, as the option currently
/// holds it, borrowed for that long. `f` must not touch an option's state.
pub(crate) fn with_option_default<R>(opt_idx: OptIndex, f: impl FnOnce(&OptVal) -> R) -> R {
    STATE.with(|state| match &state[slot(opt_idx)].default {
        Some(default) => f(default),
        None => f(&options[slot(opt_idx)].def_val.value()),
    })
}

/// A copy of the default `:set opt&` would install.
pub(crate) fn option_default(opt_idx: OptIndex) -> OptVal {
    with_option_default(opt_idx, OptVal::clone)
}

/// Give the option a default of its own, and answer the one it replaces.
/// [`crate::option::change_option_default`] is the spelling that drops it.
pub(crate) fn replace_option_default(opt_idx: OptIndex, value: OptVal) -> OptVal {
    let old = STATE.with_mut(|state| state[slot(opt_idx)].default.replace(value));
    old.unwrap_or_else(|| options[slot(opt_idx)].def_val.value())
}

/// Where the option's global value was last set from.
pub(crate) fn option_last_set(opt_idx: OptIndex) -> ScriptCtx {
    STATE.with(|state| state[slot(opt_idx)].script_ctx)
}

/// Record where the option's global value was just set from.
pub(crate) fn set_option_last_set(opt_idx: OptIndex, script_ctx: ScriptCtx) {
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
