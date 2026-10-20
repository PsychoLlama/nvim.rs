//! The editor-wide off switch: [`block_autocmds`] and [`unblock_autocmds`],
//! nestable, and the `TermResponse` owed when a reply arrived while they
//! were off.
//!
//! A file of its own because the two are exported: they are
//! `metrics/abi-ledger.jsonl` rows (`test/functional/terminal/tui_spec.lua`
//! blocks autocommands through the FFI), and `#[unsafe(no_mangle)]` is
//! itself an unsafe attribute, which a file that forbids unsafe code
//! refuses. Nothing here is unchecked.

#![deny(unsafe_op_in_unsafe_fn)]
// The `block_autocmds`/`unblock_autocmds` rows in docs/abi-ledger.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;

/// Turn autocommands off editor-wide, nestably.
#[unsafe(no_mangle)]
pub extern "C" fn block_autocmds() {
    // Remember that we may need to fire `TermResponse` later.
    if !is_autocmd_blocked() {
        termresponse_changed.set(false);
    }
    autocmd_blocked.set(autocmd_blocked.get() + 1);
}

/// Undo one [`block_autocmds`], firing the `TermResponse` that arrived
/// while they were off.
#[unsafe(no_mangle)]
pub extern "C" fn unblock_autocmds() {
    autocmd_blocked.set(autocmd_blocked.get() - 1);
    if !is_autocmd_blocked() && termresponse_changed.get() && has_event(AutoEvent::TermResponse) {
        // The autocommand takes the copy over.
        let sequence = String_0::from_bytes(&vim_var_bytes(Vv::Termresponse));
        do_termresponse_autocmd(sequence);
    }
}

/// Whether [`block_autocmds`] is in effect.
pub fn is_autocmd_blocked() -> bool {
    autocmd_blocked.get() != 0
}
