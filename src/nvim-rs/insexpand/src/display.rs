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

    // Completion direction
    fn nvim_get_compl_direction() -> c_int;

    // ins_compl_add wrapper (from insexpand_shim.c)
    fn nvim_ins_compl_add_simple(
        str_: *const c_char,
        len: c_int,
        dir: c_int,
        flags: c_int,
        score: c_int,
    ) -> c_int;

    // FreeWild (from path crate, exported as rs_FreeWild)
    fn rs_FreeWild(count: c_int, files: *mut *mut c_char);

    // Spell back-to-bad-word (already wraps spell_back_to_badword with emsg_off)
    fn nvim_spell_back_safe();
}

// CP flag constants (must match C definitions)
const CP_FAST: c_int = 32;
const CP_ICASE: c_int = 16;
const FUZZY_SCORE_NONE: c_int = -1;
const FORWARD: c_int = 1;

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
/// Iterates the matches array calling nvim_ins_compl_add_simple for each,
/// then frees the array with rs_FreeWild.
///
/// # Safety
/// `matches` must be a valid array of `num_matches` NUL-terminated strings
/// allocated by the expand machinery (will be freed by `FreeWild`).
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_ins_compl_add_matches(
    num_matches: c_int,
    matches: *mut *mut c_char,
    icase: c_int,
) {
    let mut add_r = 0; // OK = 0
    let mut dir = nvim_get_compl_direction();
    let flags = CP_FAST | (if icase != 0 { CP_ICASE } else { 0 });

    let mut i = 0;
    while i < num_matches && add_r != -1 {
        // FAIL = -1
        let str_ = *matches.add(i as usize);
        add_r = nvim_ins_compl_add_simple(str_, -1, dir, flags, FUZZY_SCORE_NONE);
        if add_r == 0 {
            // OK
            dir = FORWARD;
        }
        i += 1;
    }

    rs_FreeWild(num_matches, matches);
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
