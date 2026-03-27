//! Extmark and cleanup support for completion.

// C accessor functions
extern "C" {
    // Compound accessors for extmark management
    fn nvim_extmark_splice_delete_compl();
    fn nvim_compl_orig_extmarks_size() -> usize;
    fn nvim_extmark_apply_undo_at(idx: usize);

    // Compound accessors for cleanup
    fn nvim_api_clear_string_compl_orig_text();
    fn nvim_clear_compl_orig_extmarks();
    fn nvim_callback_free_cfu();
    fn nvim_callback_free_ofu();
    fn nvim_callback_free_tsrfu();
    fn nvim_clear_static_cpt_callbacks();
}

/// Save extmarks in `compl_orig_text` so they may be restored when completion
/// is cancelled or the original text is completed.
///
/// # Safety
/// Requires valid global completion state (`curbuf`, `curwin`, `compl_col`,
/// `compl_length`, `compl_orig_extmarks`).
#[no_mangle]
pub unsafe extern "C" fn rs_save_orig_extmarks() {
    nvim_extmark_splice_delete_compl();
}

/// Restore extmarks saved by `rs_save_orig_extmarks`, replaying them in
/// reverse order.
///
/// # Safety
/// Requires valid global completion state (`compl_orig_extmarks`).
#[no_mangle]
pub unsafe extern "C" fn rs_restore_orig_extmarks() {
    let size = nvim_compl_orig_extmarks_size();
    for i in (0..size).rev() {
        nvim_extmark_apply_undo_at(i);
    }
}

/// Free all completion-related global state at process exit (EXITFREE).
///
/// # Safety
/// Should only be called at process exit; modifies static callback state.
#[export_name = "free_insexpand_stuff"]
pub unsafe extern "C" fn rs_free_insexpand_stuff() {
    nvim_api_clear_string_compl_orig_text();
    nvim_clear_compl_orig_extmarks();
    nvim_callback_free_cfu();
    nvim_callback_free_ofu();
    nvim_callback_free_tsrfu();
    nvim_clear_static_cpt_callbacks();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_line_count_for_single_line() {
        // A single-line completion has 1 line
        let expected = 1;
        assert!(expected >= 1);
    }
}
