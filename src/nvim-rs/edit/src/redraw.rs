//! `ins_redraw`, `init_prompt`, `edit` -- insert-mode redraw, prompt init, and entry point
//!
//! Ported from `edit.c`. These three functions are migrated together because
//! they form the outermost layer of insert mode: `edit` is the entry point,
//! `init_prompt` sets up prompt buffers, and `ins_redraw` handles deferred
//! screen updates triggered on each keystroke.
//!
//! The heavy subsystem work (autocmds, screen updates, `InsertState` setup) is
//! delegated to composite C helpers. The Rust code owns the exported symbol
//! name so the linker resolves all callers to Rust.

#![allow(clippy::missing_safety_doc)]

use std::ffi::c_int;

// ============================================================================
// C composite helpers
// ============================================================================

extern "C" {
    /// Full `ins_redraw()` implementation (composite C helper).
    fn nvim_edit_ins_redraw_impl(ready: c_int);

    /// Full `init_prompt()` implementation (composite C helper).
    fn nvim_edit_init_prompt_impl(cmdchar_todo: c_int);

    /// Full `edit()` implementation (composite C helper).
    /// Returns true iff a CTRL-O caused the return.
    fn nvim_edit_edit_entry(cmdchar: c_int, startln: bool, count: c_int) -> bool;
}

// ============================================================================
// Exported symbols
// ============================================================================

/// Redraw for Insert mode (deferred until next character).
///
/// # Safety
/// Accesses global state via C subsystems.
#[unsafe(export_name = "ins_redraw")]
pub unsafe extern "C" fn rs_ins_redraw(ready: bool) {
    nvim_edit_ins_redraw_impl(c_int::from(ready));
}

/// Prepare prompt buffer for insert mode.
///
/// Ensures the last line has prompt text and moves the cursor to it.
///
/// # Safety
/// Accesses curbuf/curwin globals via C.
#[unsafe(export_name = "init_prompt")]
pub unsafe extern "C" fn rs_init_prompt(cmdchar_todo: c_int) {
    nvim_edit_init_prompt_impl(cmdchar_todo);
}

/// Insert-mode entry point.
///
/// # Safety
/// Accesses many globals via C subsystems.
#[must_use]
#[unsafe(export_name = "edit")]
pub unsafe extern "C" fn rs_edit(cmdchar: c_int, startln: bool, count: c_int) -> bool {
    nvim_edit_edit_entry(cmdchar, startln, count)
}
