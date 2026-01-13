//! Normal mode state management.
//!
//! This module provides helper functions for managing normal mode state,
//! including operator state, count values, and pending operations.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use super::types::OpArgHandle;

// =============================================================================
// External C Functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Global state accessors
    fn nvim_get_finish_op() -> c_int;
    fn nvim_set_finish_op(val: bool);
    fn nvim_get_no_mapping() -> c_int;
    fn nvim_set_no_mapping(val: c_int);
    fn nvim_get_allow_keys() -> c_int;
    fn nvim_set_allow_keys(val: c_int);
    fn nvim_get_VIsual_active() -> c_int;
    fn nvim_set_VIsual_active(val: bool);
    fn nvim_get_VIsual_reselect() -> c_int;
    fn nvim_set_VIsual_reselect(val: bool);
    fn nvim_get_VIsual_select() -> bool;
    fn nvim_set_VIsual_select(val: bool);
    fn nvim_get_VIsual_mode() -> c_int;
    fn nvim_set_VIsual_mode(val: c_int);
    fn nvim_get_restart_edit() -> c_int;
    fn nvim_set_restart_edit(val: c_int);

    // Current operation accessors
    fn nvim_oap_get_op_type() -> c_int;
    fn nvim_oap_set_op_type(oap: OpArgHandle, val: c_int);
    fn nvim_oap_get_prev_opcount() -> c_int;
    fn nvim_oap_get_prev_count0() -> c_int;
    fn nvim_oap_get_regname() -> c_int;

    // vcount accessors
    fn nvim_get_opcount() -> c_int;
    fn nvim_set_opcount(val: c_int);
}

// =============================================================================
// Finish Operation State
// =============================================================================

/// Check if finish_op is set.
#[inline]
fn is_finish_op_impl() -> bool {
    unsafe { nvim_get_finish_op() != 0 }
}

/// Set finish_op flag.
#[inline]
fn set_finish_op_impl(val: bool) {
    unsafe { nvim_set_finish_op(val) };
}

// =============================================================================
// No Mapping State
// =============================================================================

/// Check if no_mapping is set.
#[inline]
fn is_no_mapping_impl() -> bool {
    unsafe { nvim_get_no_mapping() != 0 }
}

/// Set no_mapping flag.
#[inline]
fn set_no_mapping_impl(val: bool) {
    unsafe { nvim_set_no_mapping(c_int::from(val)) };
}

// =============================================================================
// Allow Keys State
// =============================================================================

/// Check if allow_keys is set.
#[inline]
fn is_allow_keys_impl() -> bool {
    unsafe { nvim_get_allow_keys() != 0 }
}

/// Set allow_keys flag.
#[inline]
fn set_allow_keys_impl(val: bool) {
    unsafe { nvim_set_allow_keys(c_int::from(val)) };
}

// =============================================================================
// Visual Mode State
// =============================================================================

/// Check if visual mode is active.
#[inline]
fn is_visual_active_impl() -> bool {
    unsafe { nvim_get_VIsual_active() != 0 }
}

/// Set visual mode active state.
#[inline]
fn set_visual_active_impl(val: bool) {
    unsafe { nvim_set_VIsual_active(val) };
}

/// Check if visual reselect mode.
#[inline]
fn is_visual_reselect_impl() -> bool {
    unsafe { nvim_get_VIsual_reselect() != 0 }
}

/// Set visual reselect mode.
#[inline]
fn set_visual_reselect_impl(val: bool) {
    unsafe { nvim_set_VIsual_reselect(val) };
}

/// Check if visual select mode.
#[inline]
fn is_visual_select_impl() -> bool {
    unsafe { nvim_get_VIsual_select() }
}

/// Set visual select mode.
#[inline]
fn set_visual_select_impl(val: bool) {
    unsafe { nvim_set_VIsual_select(val) };
}

/// Get visual mode character.
#[inline]
fn get_visual_mode_impl() -> c_int {
    unsafe { nvim_get_VIsual_mode() }
}

/// Set visual mode character.
#[inline]
fn set_visual_mode_impl(val: c_int) {
    unsafe { nvim_set_VIsual_mode(val) };
}

// =============================================================================
// Restart Edit State
// =============================================================================

/// Get restart_edit value.
#[inline]
fn get_restart_edit_impl() -> c_int {
    unsafe { nvim_get_restart_edit() }
}

/// Set restart_edit value.
#[inline]
fn set_restart_edit_impl(val: c_int) {
    unsafe { nvim_set_restart_edit(val) };
}

// =============================================================================
// Operator State Helpers
// =============================================================================

/// Get current operator type from global state.
#[inline]
fn get_op_type_global_impl() -> c_int {
    unsafe { nvim_oap_get_op_type() }
}

/// Get previous operator count.
#[inline]
fn get_prev_opcount_impl() -> c_int {
    unsafe { nvim_oap_get_prev_opcount() }
}

/// Get previous count0.
#[inline]
fn get_prev_count0_impl() -> c_int {
    unsafe { nvim_oap_get_prev_count0() }
}

/// Get current register name from global state.
#[inline]
fn get_regname_global_impl() -> c_int {
    unsafe { nvim_oap_get_regname() }
}

// =============================================================================
// Operation Count Helpers
// =============================================================================

/// Get opcount value.
#[inline]
fn get_opcount_impl() -> c_int {
    unsafe { nvim_get_opcount() }
}

/// Set opcount value.
#[inline]
fn set_opcount_impl(val: c_int) {
    unsafe { nvim_set_opcount(val) };
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Check if finish_op is set.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_is_finish_op() -> c_int {
    c_int::from(is_finish_op_impl())
}

/// FFI: Set finish_op flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_set_finish_op(val: c_int) {
    set_finish_op_impl(val != 0);
}

/// FFI: Check if no_mapping is set.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_is_no_mapping() -> c_int {
    c_int::from(is_no_mapping_impl())
}

/// FFI: Set no_mapping flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_set_no_mapping(val: c_int) {
    set_no_mapping_impl(val != 0);
}

/// FFI: Check if allow_keys is set.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_is_allow_keys() -> c_int {
    c_int::from(is_allow_keys_impl())
}

/// FFI: Set allow_keys flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_set_allow_keys(val: c_int) {
    set_allow_keys_impl(val != 0);
}

/// FFI: Check if visual mode active.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_visual_active() -> c_int {
    c_int::from(is_visual_active_impl())
}

/// FFI: Set visual mode active.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_set_visual_active(val: c_int) {
    set_visual_active_impl(val != 0);
}

/// FFI: Check if visual reselect mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_visual_reselect() -> c_int {
    c_int::from(is_visual_reselect_impl())
}

/// FFI: Set visual reselect mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_set_visual_reselect(val: c_int) {
    set_visual_reselect_impl(val != 0);
}

/// FFI: Check if visual select mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_visual_select() -> c_int {
    c_int::from(is_visual_select_impl())
}

/// FFI: Set visual select mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_set_visual_select(val: c_int) {
    set_visual_select_impl(val != 0);
}

/// FFI: Get visual mode character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_get_visual_mode() -> c_int {
    get_visual_mode_impl()
}

/// FFI: Set visual mode character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_set_visual_mode(val: c_int) {
    set_visual_mode_impl(val);
}

/// FFI: Get restart_edit value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_get_restart_edit() -> c_int {
    get_restart_edit_impl()
}

/// FFI: Set restart_edit value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_set_restart_edit(val: c_int) {
    set_restart_edit_impl(val);
}

/// FFI: Get operator type from global state.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_get_op_type() -> c_int {
    get_op_type_global_impl()
}

/// FFI: Get previous opcount.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_get_prev_opcount() -> c_int {
    get_prev_opcount_impl()
}

/// FFI: Get previous count0.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_get_prev_count0() -> c_int {
    get_prev_count0_impl()
}

/// FFI: Get register name from global state.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_get_regname() -> c_int {
    get_regname_global_impl()
}

/// FFI: Get opcount value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_get_opcount() -> c_int {
    get_opcount_impl()
}

/// FFI: Set opcount value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_state_set_opcount(val: c_int) {
    set_opcount_impl(val);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    // Tests would require runtime state - placeholder for now
}
