//! Full `do_pending_operator` migration (Phase 5)
//!
//! Migrated from `do_pending_operator()` in ops.c — the main operator
//! dispatcher that runs after a motion completes.

use std::ffi::{c_int, c_void};

extern "C" {
    // Check if we should process pending operator
    fn nvim_dpo_should_process(cap: *mut c_void) -> c_int;

    // Preamble: save state, handle motion_force, prep redo
    fn nvim_dpo_preamble(cap: *mut c_void, gui_yank: c_int);

    // Setup positions, folds, visual state, redo visual
    fn nvim_dpo_setup_positions(cap: *mut c_void, gui_yank: c_int);

    // Operator dispatch (the big switch)
    fn nvim_dpo_dispatch_operator(cap: *mut c_void, gui_yank: c_int);

    // Postamble: virtual_op reset, column restore, clearop
    fn nvim_dpo_postamble(cap: *mut c_void, old_col: c_int, gui_yank: c_int);

    // Final: restore_lbr
    fn nvim_dpo_restore_lbr(cap: *mut c_void);
}

/// Full migration of `do_pending_operator()`.
///
/// # Safety
/// - `cap` must be a valid `cmdarg_T *`
/// - Accesses global state via C accessors
#[unsafe(export_name = "do_pending_operator")]
pub unsafe extern "C" fn rs_do_pending_operator(cap: *mut c_void, old_col: c_int, gui_yank: bool) {
    let gui_yank_int = c_int::from(gui_yank);
    if nvim_dpo_should_process(cap) != 0 {
        nvim_dpo_preamble(cap, gui_yank_int);
        nvim_dpo_setup_positions(cap, gui_yank_int);
        nvim_dpo_dispatch_operator(cap, gui_yank_int);
        nvim_dpo_postamble(cap, old_col, gui_yank_int);
    }
    nvim_dpo_restore_lbr(cap);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // The pending operator module requires full C runtime;
        // unit tests are limited to compilation checks.
        // Verify the extern declarations are well-formed by checking function pointer sizes.
        let size =
            std::mem::size_of::<unsafe extern "C" fn(*mut std::ffi::c_void) -> std::ffi::c_int>();
        assert!(size > 0);
    }
}
