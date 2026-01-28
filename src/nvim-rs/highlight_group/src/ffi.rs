//! FFI declarations for C highlight group functions and accessors.
//!
//! This module provides the extern "C" declarations for functions defined in
//! `highlight_group.c` that allow Rust code to read and write highlight group
//! data stored in the C `highlight_ga` array.

use std::ffi::{c_char, c_int};

use crate::types::RgbValue;

// =============================================================================
// External C functions for highlight table access
// =============================================================================

extern "C" {
    // Array length accessor
    pub fn nvim_get_highlight_ga_len() -> c_int;

    // Global color accessors
    pub fn nvim_get_t_colors() -> c_int;
    pub fn nvim_get_normal_fg() -> c_int;
    pub fn nvim_get_normal_bg() -> c_int;
    pub fn nvim_get_normal_sp() -> c_int;
    pub fn nvim_set_normal_fg(val: c_int);
    pub fn nvim_set_normal_bg(val: c_int);
    pub fn nvim_set_normal_sp(val: c_int);
    pub fn nvim_get_cterm_normal_fg_color() -> c_int;
    pub fn nvim_get_cterm_normal_bg_color() -> c_int;
    pub fn nvim_set_cterm_normal_fg_color(val: c_int);
    pub fn nvim_set_cterm_normal_bg_color(val: c_int);
    pub fn nvim_get_p_bg() -> c_char;

    // HlGroup field getters (by index)
    pub fn nvim_hl_table_get_sg_name(idx: c_int) -> *mut c_char;
    pub fn nvim_hl_table_get_sg_name_u(idx: c_int) -> *mut c_char;
    pub fn nvim_hl_table_get_sg_cleared(idx: c_int) -> bool;
    pub fn nvim_hl_table_get_sg_attr(idx: c_int) -> c_int;
    pub fn nvim_hl_table_get_sg_link(idx: c_int) -> c_int;
    pub fn nvim_hl_table_get_sg_deflink(idx: c_int) -> c_int;
    pub fn nvim_hl_table_get_sg_set(idx: c_int) -> c_int;
    pub fn nvim_hl_table_get_sg_cterm(idx: c_int) -> c_int;
    pub fn nvim_hl_table_get_sg_cterm_fg(idx: c_int) -> c_int;
    pub fn nvim_hl_table_get_sg_cterm_bg(idx: c_int) -> c_int;
    pub fn nvim_hl_table_get_sg_cterm_bold(idx: c_int) -> bool;
    pub fn nvim_hl_table_get_sg_gui(idx: c_int) -> c_int;
    pub fn nvim_hl_table_get_sg_rgb_fg(idx: c_int) -> RgbValue;
    pub fn nvim_hl_table_get_sg_rgb_bg(idx: c_int) -> RgbValue;
    pub fn nvim_hl_table_get_sg_rgb_sp(idx: c_int) -> RgbValue;
    pub fn nvim_hl_table_get_sg_rgb_fg_idx(idx: c_int) -> c_int;
    pub fn nvim_hl_table_get_sg_rgb_bg_idx(idx: c_int) -> c_int;
    pub fn nvim_hl_table_get_sg_rgb_sp_idx(idx: c_int) -> c_int;
    pub fn nvim_hl_table_get_sg_blend(idx: c_int) -> c_int;
    pub fn nvim_hl_table_get_sg_parent(idx: c_int) -> c_int;

    // HlGroup field setters (by index)
    pub fn nvim_hl_table_set_sg_cleared(idx: c_int, val: bool);
    pub fn nvim_hl_table_set_sg_attr(idx: c_int, val: c_int);
    pub fn nvim_hl_table_set_sg_link(idx: c_int, val: c_int);
    pub fn nvim_hl_table_set_sg_deflink(idx: c_int, val: c_int);
    pub fn nvim_hl_table_set_sg_set(idx: c_int, val: c_int);
    pub fn nvim_hl_table_set_sg_cterm(idx: c_int, val: c_int);
    pub fn nvim_hl_table_set_sg_cterm_fg(idx: c_int, val: c_int);
    pub fn nvim_hl_table_set_sg_cterm_bg(idx: c_int, val: c_int);
    pub fn nvim_hl_table_set_sg_cterm_bold(idx: c_int, val: bool);
    pub fn nvim_hl_table_set_sg_gui(idx: c_int, val: c_int);
    pub fn nvim_hl_table_set_sg_rgb_fg(idx: c_int, val: RgbValue);
    pub fn nvim_hl_table_set_sg_rgb_bg(idx: c_int, val: RgbValue);
    pub fn nvim_hl_table_set_sg_rgb_sp(idx: c_int, val: RgbValue);
    pub fn nvim_hl_table_set_sg_rgb_fg_idx(idx: c_int, val: c_int);
    pub fn nvim_hl_table_set_sg_rgb_bg_idx(idx: c_int, val: c_int);
    pub fn nvim_hl_table_set_sg_rgb_sp_idx(idx: c_int, val: c_int);
    pub fn nvim_hl_table_set_sg_blend(idx: c_int, val: c_int);
    pub fn nvim_hl_table_set_sg_parent(idx: c_int, val: c_int);

    // Registry functions
    pub fn nvim_highlight_name_lookup(name_u: *const c_char) -> c_int;

    // Window highlight namespace
    pub fn c_curwin_ns_hl_active() -> c_int;

    // Group management functions (called from Rust back into C)
    pub fn c_syn_add_group(name: *const c_char, len: usize) -> c_int;
}

// =============================================================================
// Safe wrapper functions
// =============================================================================

/// Get the number of highlight groups currently defined.
#[inline]
pub fn highlight_group_count() -> c_int {
    unsafe { nvim_get_highlight_ga_len() }
}

/// Get the terminal color count.
#[inline]
pub fn get_t_colors() -> c_int {
    unsafe { nvim_get_t_colors() }
}

/// Get the Normal foreground RGB color.
#[inline]
pub fn get_normal_fg() -> RgbValue {
    unsafe { nvim_get_normal_fg() }
}

/// Get the Normal background RGB color.
#[inline]
pub fn get_normal_bg() -> RgbValue {
    unsafe { nvim_get_normal_bg() }
}

/// Get the Normal special RGB color.
#[inline]
pub fn get_normal_sp() -> RgbValue {
    unsafe { nvim_get_normal_sp() }
}

/// Get the background option ('light' or 'dark').
#[inline]
pub fn get_background_option() -> char {
    unsafe { nvim_get_p_bg() as u8 as char }
}

/// Check if a highlight group index is valid.
#[inline]
pub fn is_valid_index(idx: c_int) -> bool {
    idx >= 0 && idx < highlight_group_count()
}

/// Check if a highlight group ID (1-based) is valid.
#[inline]
pub fn is_valid_id(id: c_int) -> bool {
    id > 0 && id <= highlight_group_count()
}

/// Get the name of a highlight group by index.
///
/// # Safety
/// Caller must ensure `idx` is a valid index (0 to highlight_group_count()-1).
#[inline]
pub unsafe fn get_group_name(idx: c_int) -> *mut c_char {
    nvim_hl_table_get_sg_name(idx)
}

/// Get the uppercase name of a highlight group by index.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_name_upper(idx: c_int) -> *mut c_char {
    nvim_hl_table_get_sg_name_u(idx)
}

/// Get the sg_cleared flag for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_cleared(idx: c_int) -> bool {
    nvim_hl_table_get_sg_cleared(idx)
}

/// Get the screen attribute for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_attr(idx: c_int) -> c_int {
    nvim_hl_table_get_sg_attr(idx)
}

/// Get the link ID for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_link(idx: c_int) -> c_int {
    nvim_hl_table_get_sg_link(idx)
}

/// Get the default link ID for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_deflink(idx: c_int) -> c_int {
    nvim_hl_table_get_sg_deflink(idx)
}

/// Get the sg_set flags for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_set_flags(idx: c_int) -> c_int {
    nvim_hl_table_get_sg_set(idx)
}

/// Get the cterm attributes for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_cterm(idx: c_int) -> c_int {
    nvim_hl_table_get_sg_cterm(idx)
}

/// Get the gui attributes for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_gui(idx: c_int) -> c_int {
    nvim_hl_table_get_sg_gui(idx)
}

/// Get the RGB foreground color for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_rgb_fg(idx: c_int) -> RgbValue {
    nvim_hl_table_get_sg_rgb_fg(idx)
}

/// Get the RGB background color for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_rgb_bg(idx: c_int) -> RgbValue {
    nvim_hl_table_get_sg_rgb_bg(idx)
}

/// Get the RGB special color for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_rgb_sp(idx: c_int) -> RgbValue {
    nvim_hl_table_get_sg_rgb_sp(idx)
}

/// Get the blend level for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_blend(idx: c_int) -> c_int {
    nvim_hl_table_get_sg_blend(idx)
}

/// Get the parent ID for a highlight group (for @nested.groups).
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_parent(idx: c_int) -> c_int {
    nvim_hl_table_get_sg_parent(idx)
}

/// Set the cleared flag for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_cleared(idx: c_int, val: bool) {
    nvim_hl_table_set_sg_cleared(idx, val);
}

/// Set the screen attribute for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_attr(idx: c_int, val: c_int) {
    nvim_hl_table_set_sg_attr(idx, val);
}

/// Set the link ID for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_link(idx: c_int, val: c_int) {
    nvim_hl_table_set_sg_link(idx, val);
}

/// Set the sg_set flags for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_set_flags(idx: c_int, val: c_int) {
    nvim_hl_table_set_sg_set(idx, val);
}

/// Set the cterm attributes for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_cterm(idx: c_int, val: c_int) {
    nvim_hl_table_set_sg_cterm(idx, val);
}

/// Set the gui attributes for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_gui(idx: c_int, val: c_int) {
    nvim_hl_table_set_sg_gui(idx, val);
}

/// Set the RGB foreground color for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_rgb_fg(idx: c_int, val: RgbValue) {
    nvim_hl_table_set_sg_rgb_fg(idx, val);
}

/// Set the RGB background color for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_rgb_bg(idx: c_int, val: RgbValue) {
    nvim_hl_table_set_sg_rgb_bg(idx, val);
}

/// Set the RGB special color for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_rgb_sp(idx: c_int, val: RgbValue) {
    nvim_hl_table_set_sg_rgb_sp(idx, val);
}

/// Set the blend level for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_blend(idx: c_int, val: c_int) {
    nvim_hl_table_set_sg_blend(idx, val);
}

/// Get the current window's active highlight namespace.
#[inline]
pub fn get_curwin_ns_hl_active() -> c_int {
    unsafe { c_curwin_ns_hl_active() }
}
