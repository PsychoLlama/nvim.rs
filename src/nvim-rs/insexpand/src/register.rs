//! Register completion support.
//!
//! This module provides Rust implementation for register-based completion
//! (CTRL-X CTRL-R). The core register access and infercase addition remain
//! in C via compound accessor, but Rust orchestrates the call.

extern "C" {
    /// Compound accessor: performs the full register completion scan in C.
    /// Iterates all registers, extracts words, and calls ins_compl_add_infercase
    /// for each match.
    fn nvim_get_register_completion_impl();
}

/// Perform register-based completion.
///
/// Iterates all named registers and adds their contents as completion matches.
/// For each register entry, extracts individual words (unless in adding mode,
/// where the whole entry string is used) and calls ins_compl_add_infercase.
///
/// # Safety
/// Requires valid completion state (compl_orig_text, compl_direction, etc.)
#[no_mangle]
pub unsafe extern "C" fn rs_get_register_completion() {
    nvim_get_register_completion_impl();
}
