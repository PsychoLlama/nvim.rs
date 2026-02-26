//! Initialization functions for the eval subsystem.
//!
//! Migrated from eval_shim.c (Phase 12, Phase 3).
//!
//! - `eval_init`: initialize global and v: variables
//!
//! Note: `eval_clear` is NOT migrated here because the C functions it calls
//! (`evalvars_clear`, `free_scriptnames`, `free_all_functions`, `free_locales`)
//! are all guarded by `#if defined(EXITFREE)` in C. Conditionally linking to
//! EXITFREE-only symbols from Rust requires feature gating that is not yet
//! set up. `eval_clear` remains in C eval_shim.c.

extern "C" {
    fn evalvars_init();
    fn func_init();
}

/// Initialize the global and v: variables.
///
/// Exported as `eval_init` (replaces C wrapper in eval_shim.c, Phase 12).
///
/// # Safety
///
/// Must only be called once at startup, before any eval operations.
#[export_name = "eval_init"]
pub unsafe extern "C" fn rs_eval_init() {
    evalvars_init();
    func_init();
}
