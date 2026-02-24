//! Display helpers: show_pum, ins_compl_add_matches, spell_back_to_badword.
//!
//! This module provides Rust implementations for completion display helpers
//! that orchestrate popup menu display and match addition.

use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    // RedrawingDisabled
    fn nvim_get_redrawing_disabled() -> c_int;
    fn nvim_set_redrawing_disabled(val: c_int);

    // Cursor position
    fn nvim_setcursor();
    fn nvim_get_curwin_w_wrow() -> c_int;
    fn nvim_get_curwin_w_leftcol() -> c_int;

    // Popup menu helpers (from pum.rs)
    fn rs_ins_compl_del_pum();
    fn rs_ins_compl_show_pum();

    // Compound accessor: runs the ins_compl_add loop + FreeWild
    fn nvim_ins_compl_add_matches_impl(num_matches: c_int, matches: *mut *mut c_char, icase: c_int);

    // Spell back-to-bad-word (already wraps spell_back_to_badword with emsg_off)
    fn nvim_spell_back_safe();
}

/// Remove (if needed) and show the popup menu.
///
/// Rust port of the C `show_pum()` function.
///
/// # Safety
/// Must be called from insert mode with valid completion/window state.
#[no_mangle]
pub unsafe extern "C" fn rs_show_pum(prev_w_wrow: c_int, prev_w_leftcol: c_int) {
    // RedrawingDisabled may be set when invoked through complete().
    let n = nvim_get_redrawing_disabled();
    nvim_set_redrawing_disabled(0);

    // If the cursor moved or the display scrolled we need to remove the pum first.
    nvim_setcursor();
    if prev_w_wrow != nvim_get_curwin_w_wrow() || prev_w_leftcol != nvim_get_curwin_w_leftcol() {
        rs_ins_compl_del_pum();
    }

    rs_ins_compl_show_pum();
    nvim_setcursor();
    nvim_set_redrawing_disabled(n);
}

/// Add an array of matches to the completion list and free the array.
///
/// Rust port of the C `ins_compl_add_matches()` function.
/// The actual looping is delegated to a C compound accessor so that the
/// `ins_compl_add` / `FreeWild` calls remain in C.
///
/// # Safety
/// `matches` must be a valid array of `num_matches` NUL-terminated strings
/// allocated by the expand machinery (will be freed by `FreeWild`).
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_add_matches(
    num_matches: c_int,
    matches: *mut *mut c_char,
    icase: c_int,
) {
    nvim_ins_compl_add_matches_impl(num_matches, matches, icase);
}

/// Move cursor to the previous badly-spelled word when starting spell completion.
///
/// Rust port of C `spell_back_to_badword()` (called through `nvim_spell_back_safe`
/// which already suppresses error messages via `emsg_off`).
///
/// # Safety
/// Must be called from insert mode with a valid window state.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_back_to_badword() {
    nvim_spell_back_safe();
}
