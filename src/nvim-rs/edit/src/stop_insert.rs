//! `stop_insert` -- end-of-insert cleanup
//!
//! Ported from `edit.c` `stop_insert()`. The complex cursor-manipulation
//! parts are delegated to composite C helpers; this module owns the control
//! flow and the `did_ai`/`did_si`/`can_si`/`can_si_back` cleanup.
//!
//! Note: `stop_insert` was static in C (only called from within `edit.c`).
//! The C `stop_insert` thin wrapper now delegates to these helpers.

#![allow(clippy::missing_safety_doc)]
// The module-level exports are used via FFI even if Rust doesn't see the calls.
#![allow(dead_code)]

use std::ffi::{c_int, c_void};

extern "C" {
    fn stop_redo_ins();
    fn rs_replace_stack_clear();
    fn nvim_edit_stop_insert_save_text();
    fn nvim_get_arrow_used() -> c_int;
    fn nvim_edit_stop_insert_do_format(end_insert_pos: *mut c_void, esc: c_int, nomove: c_int);
    fn nvim_set_did_ai(val: bool);
    fn nvim_set_did_si(val: bool);
    fn nvim_set_can_si(val: bool);
    fn nvim_set_can_si_back(val: bool);
    fn nvim_edit_set_b_op_marks(end_insert_pos: *mut c_void);
}

/// Handle end-of-insert cleanup.
///
/// Saves inserted text, auto-formats, strips auto-indent whitespace, sets marks.
///
/// # Safety
/// Accesses many global variables via C accessor functions.
/// `end_insert_pos` must be a valid `pos_T*` or NULL.
pub(crate) unsafe fn stop_insert_impl(end_insert_pos: *mut c_void, esc: c_int, nomove: c_int) {
    stop_redo_ins();
    rs_replace_stack_clear();

    nvim_edit_stop_insert_save_text();

    if nvim_get_arrow_used() == 0 && !end_insert_pos.is_null() {
        nvim_edit_stop_insert_do_format(end_insert_pos, esc, nomove);
    }

    nvim_set_did_ai(false);
    nvim_set_did_si(false);
    nvim_set_can_si(false);
    nvim_set_can_si_back(false);

    if !end_insert_pos.is_null() {
        nvim_edit_set_b_op_marks(end_insert_pos);
    }
}
