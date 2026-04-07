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
    /// Full `init_prompt()` implementation (composite C helper).
    fn nvim_edit_init_prompt_impl(cmdchar_todo: c_int);

    /// Full `edit()` implementation (composite C helper).
    /// Returns true iff a CTRL-O caused the return.
    fn nvim_edit_edit_entry(cmdchar: c_int, startln: bool, count: c_int) -> bool;

    // --- ins_redraw composites ---

    fn char_avail() -> bool;
    fn pum_visible() -> bool;
    fn rs_ins_compl_active() -> c_int;
    fn may_trigger_win_scrolled_resized();
    fn may_trigger_safestate(safe: bool);

    /// True when `CursorMovedI` should fire.
    fn nvim_ins_redraw_cursormoved_pending() -> bool;
    /// True when syntax highlighting is present and `must_redraw` is set.
    fn nvim_ins_redraw_syntax_must_redraw() -> bool;
    /// Trigger `CursorMovedI` and update `last_cursormoved` tracking.
    fn nvim_ins_redraw_trigger_cursormovedi();
    /// True when `TextChangedI` should fire.
    fn nvim_curbuf_textchangedi_pending() -> bool;
    /// Apply `TextChangedI` autocmds, sync changedtick, `u_save` if needed.
    fn nvim_edit_apply_textchangedi();
    /// True when `TextChangedP` should fire.
    fn nvim_curbuf_textchangedp_pending() -> bool;
    /// Apply `TextChangedP` autocmds, sync changedtick, `u_save` if needed.
    fn nvim_edit_apply_textchangedp();
    /// True when `BufModifiedSet` should fire.
    fn nvim_curbuf_bufmodifiedset_pending() -> bool;
    /// Apply `BufModifiedSet` autocmds and clear `b_changed_invalid`.
    fn nvim_edit_apply_bufmodifiedset();
    /// Run the final screen-update sequence (`pum_check_clear`, `update_screen`, etc.).
    fn nvim_ins_redraw_screen_update();
    /// `update_screen()` wrapper (used for `CursorMovedI` pre-update).
    fn update_screen() -> c_int;
}

// ============================================================================
// ins_redraw implementation (ported from nvim_edit_ins_redraw_impl)
// ============================================================================

/// Redraw for Insert mode (deferred until next character).
///
/// Triggers `CursorMovedI`, `TextChangedI`, `TextChangedP`, `BufModifiedSet`,
/// and `SafeState` autocmds, then performs the screen redraw.
///
/// # Safety
/// Accesses global state via C subsystems.
pub(crate) unsafe fn ins_redraw_impl(ready: bool) {
    if char_avail() {
        return;
    }

    // Trigger CursorMovedI if the cursor moved and popup is not visible.
    if ready && nvim_ins_redraw_cursormoved_pending() {
        // Update screen first so syntax highlighting is correct.
        if nvim_ins_redraw_syntax_must_redraw() {
            update_screen();
        }
        nvim_ins_redraw_trigger_cursormovedi();
    }

    // Trigger TextChangedI if changedtick_i differs and popup is not visible.
    if ready && nvim_curbuf_textchangedi_pending() {
        nvim_edit_apply_textchangedi();
    }

    // Trigger TextChangedP if changedtick_pum differs and popup is visible.
    if ready && nvim_curbuf_textchangedp_pending() {
        nvim_edit_apply_textchangedp();
    }

    if ready {
        may_trigger_win_scrolled_resized();
    }

    // Trigger BufModifiedSet if b_changed_invalid is set and popup is not visible.
    if ready && nvim_curbuf_bufmodifiedset_pending() {
        nvim_edit_apply_bufmodifiedset();
    }

    // Trigger SafeState if nothing is pending.
    may_trigger_safestate(ready && rs_ins_compl_active() == 0 && !pum_visible());

    nvim_ins_redraw_screen_update();
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
    ins_redraw_impl(ready);
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
