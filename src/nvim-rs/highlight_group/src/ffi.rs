//! FFI declarations for C highlight group functions and accessors.
//!
//! This module provides direct access to C global variables and the
//! `highlight_ga` growing array, eliminating per-field C accessor functions.

use std::ffi::{c_char, c_int, c_void};

use crate::types::{GArray, HlGroup, RgbValue};

// =============================================================================
// External C globals (direct access — no wrapper functions needed)
// =============================================================================

extern "C" {
    /// The highlight group table (was `static garray_T highlight_ga` in C).
    pub static mut highlight_ga: GArray;

    // Global color state
    pub static mut t_colors: c_int;
    pub static mut normal_fg: RgbValue;
    pub static mut normal_bg: RgbValue;
    pub static mut normal_sp: RgbValue;
    pub static mut cterm_normal_fg_color: c_int;
    pub static mut cterm_normal_bg_color: c_int;
    /// `char *p_bg` — points to "light" or "dark"
    pub static p_bg: *const c_char;

    /// Current window (opaque pointer — accessed via accessor below)
    static curwin: *mut c_void;
}

// =============================================================================
// External C functions (non-accessor, kept in C)
// =============================================================================

extern "C" {
    /// Look up a highlight group by its uppercase name.
    pub fn nvim_highlight_name_lookup(name_u: *const c_char) -> c_int;

    /// Get the active highlight namespace for a window.
    pub fn nvim_win_get_ns_hl_active(wp: *mut c_void) -> c_int;

    /// Group management functions (called from Rust back into C)
    pub fn c_syn_add_group(name: *const c_char, len: usize) -> c_int;
}

// =============================================================================
// Inline helpers for direct highlight_ga access
// =============================================================================

/// Get a raw pointer to the HlGroup at the given index (0-based).
///
/// # Safety
/// - `highlight_ga` must be initialized.
/// - `idx` must be in `0..highlight_ga.ga_len`.
#[inline]
unsafe fn hl_table_ptr(idx: c_int) -> *mut HlGroup {
    (highlight_ga.ga_data as *mut HlGroup).add(idx as usize)
}

// =============================================================================
// Safe wrapper functions
// =============================================================================

/// Get the number of highlight groups currently defined.
#[inline]
pub fn highlight_group_count() -> c_int {
    unsafe { highlight_ga.ga_len }
}

/// Get the terminal color count.
#[inline]
pub fn get_t_colors() -> c_int {
    unsafe { t_colors }
}

/// Get the Normal foreground RGB color.
#[inline]
pub fn get_normal_fg() -> RgbValue {
    unsafe { normal_fg }
}

/// Get the Normal background RGB color.
#[inline]
pub fn get_normal_bg() -> RgbValue {
    unsafe { normal_bg }
}

/// Get the Normal special RGB color.
#[inline]
pub fn get_normal_sp() -> RgbValue {
    unsafe { normal_sp }
}

/// Get the background option ('light' or 'dark').
#[inline]
pub fn get_background_option() -> char {
    unsafe { *p_bg as u8 as char }
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

/// Get the current window's active highlight namespace.
#[inline]
pub fn get_curwin_ns_hl_active() -> c_int {
    unsafe { nvim_win_get_ns_hl_active(curwin) }
}

/// Get the name of a highlight group by index.
///
/// # Safety
/// Caller must ensure `idx` is a valid index (0 to highlight_group_count()-1).
#[inline]
pub unsafe fn get_group_name(idx: c_int) -> *mut c_char {
    (*hl_table_ptr(idx)).sg_name
}

/// Get the uppercase name of a highlight group by index.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_name_upper(idx: c_int) -> *mut c_char {
    (*hl_table_ptr(idx)).sg_name_u
}

/// Get the sg_cleared flag for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_cleared(idx: c_int) -> bool {
    (*hl_table_ptr(idx)).sg_cleared
}

/// Get the screen attribute for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_attr(idx: c_int) -> c_int {
    (*hl_table_ptr(idx)).sg_attr
}

/// Get the link ID for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_link(idx: c_int) -> c_int {
    (*hl_table_ptr(idx)).sg_link
}

/// Get the default link ID for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_deflink(idx: c_int) -> c_int {
    (*hl_table_ptr(idx)).sg_deflink
}

/// Get the sg_set flags for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_set_flags(idx: c_int) -> c_int {
    (*hl_table_ptr(idx)).sg_set
}

/// Get the cterm attributes for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_cterm(idx: c_int) -> c_int {
    (*hl_table_ptr(idx)).sg_cterm
}

/// Get the gui attributes for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_gui(idx: c_int) -> c_int {
    (*hl_table_ptr(idx)).sg_gui
}

/// Get the RGB foreground color for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_rgb_fg(idx: c_int) -> RgbValue {
    (*hl_table_ptr(idx)).sg_rgb_fg
}

/// Get the RGB background color for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_rgb_bg(idx: c_int) -> RgbValue {
    (*hl_table_ptr(idx)).sg_rgb_bg
}

/// Get the RGB special color for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_rgb_sp(idx: c_int) -> RgbValue {
    (*hl_table_ptr(idx)).sg_rgb_sp
}

/// Get the blend level for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_blend(idx: c_int) -> c_int {
    (*hl_table_ptr(idx)).sg_blend
}

/// Get the parent ID for a highlight group (for @nested.groups).
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn get_group_parent(idx: c_int) -> c_int {
    (*hl_table_ptr(idx)).sg_parent
}

/// Set the cleared flag for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_cleared(idx: c_int, val: bool) {
    (*hl_table_ptr(idx)).sg_cleared = val;
}

/// Set the screen attribute for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_attr(idx: c_int, val: c_int) {
    (*hl_table_ptr(idx)).sg_attr = val;
}

/// Set the link ID for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_link(idx: c_int, val: c_int) {
    (*hl_table_ptr(idx)).sg_link = val;
}

/// Set the sg_set flags for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_set_flags(idx: c_int, val: c_int) {
    (*hl_table_ptr(idx)).sg_set = val;
}

/// Set the cterm attributes for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_cterm(idx: c_int, val: c_int) {
    (*hl_table_ptr(idx)).sg_cterm = val;
}

/// Set the gui attributes for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_gui(idx: c_int, val: c_int) {
    (*hl_table_ptr(idx)).sg_gui = val;
}

/// Set the RGB foreground color for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_rgb_fg(idx: c_int, val: RgbValue) {
    (*hl_table_ptr(idx)).sg_rgb_fg = val;
}

/// Set the RGB background color for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_rgb_bg(idx: c_int, val: RgbValue) {
    (*hl_table_ptr(idx)).sg_rgb_bg = val;
}

/// Set the RGB special color for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_rgb_sp(idx: c_int, val: RgbValue) {
    (*hl_table_ptr(idx)).sg_rgb_sp = val;
}

/// Set the blend level for a highlight group.
///
/// # Safety
/// Caller must ensure `idx` is a valid index.
#[inline]
pub unsafe fn set_group_blend(idx: c_int, val: c_int) {
    (*hl_table_ptr(idx)).sg_blend = val;
}

// =============================================================================
// Compatibility symbols for callers outside this crate (e.g., nvim-highlight)
//
// These replace the deleted C accessor functions with Rust implementations
// that directly access the global variables. Other Rust crates that declare
// `extern "C" { fn nvim_get_t_colors() -> c_int; }` etc. link to these.
// =============================================================================

#[export_name = "nvim_get_t_colors"]
pub unsafe extern "C" fn compat_get_t_colors() -> c_int {
    t_colors
}

#[export_name = "nvim_get_normal_fg"]
pub unsafe extern "C" fn compat_get_normal_fg() -> c_int {
    normal_fg
}

#[export_name = "nvim_get_normal_bg"]
pub unsafe extern "C" fn compat_get_normal_bg() -> c_int {
    normal_bg
}

#[export_name = "nvim_get_normal_sp"]
pub unsafe extern "C" fn compat_get_normal_sp() -> c_int {
    normal_sp
}

#[export_name = "nvim_set_normal_fg"]
pub unsafe extern "C" fn compat_set_normal_fg(val: c_int) {
    normal_fg = val;
}

#[export_name = "nvim_set_normal_bg"]
pub unsafe extern "C" fn compat_set_normal_bg(val: c_int) {
    normal_bg = val;
}

#[export_name = "nvim_set_normal_sp"]
pub unsafe extern "C" fn compat_set_normal_sp(val: c_int) {
    normal_sp = val;
}

#[export_name = "nvim_get_cterm_normal_fg_color"]
pub unsafe extern "C" fn compat_get_cterm_normal_fg_color() -> c_int {
    cterm_normal_fg_color
}

#[export_name = "nvim_get_cterm_normal_bg_color"]
pub unsafe extern "C" fn compat_get_cterm_normal_bg_color() -> c_int {
    cterm_normal_bg_color
}

#[export_name = "nvim_set_cterm_normal_fg_color"]
pub unsafe extern "C" fn compat_set_cterm_normal_fg_color(val: c_int) {
    cterm_normal_fg_color = val;
}

#[export_name = "nvim_set_cterm_normal_bg_color"]
pub unsafe extern "C" fn compat_set_cterm_normal_bg_color(val: c_int) {
    cterm_normal_bg_color = val;
}

#[export_name = "nvim_get_p_bg"]
pub unsafe extern "C" fn compat_get_p_bg() -> c_char {
    *p_bg
}

#[export_name = "nvim_get_highlight_ga_len"]
pub unsafe extern "C" fn compat_get_highlight_ga_len() -> c_int {
    highlight_ga.ga_len
}

/// Provide `c_curwin_ns_hl_active` for the nvim-highlight crate.
#[export_name = "c_curwin_ns_hl_active"]
pub unsafe extern "C" fn compat_curwin_ns_hl_active() -> c_int {
    nvim_win_get_ns_hl_active(curwin)
}

/// Provide `nvim_hl_table_get_sg_gui` for the nvim-highlight crate.
#[export_name = "nvim_hl_table_get_sg_gui"]
pub unsafe extern "C" fn compat_hl_table_get_sg_gui(idx: c_int) -> c_int {
    (*hl_table_ptr(idx)).sg_gui
}

/// Provide `nvim_hl_table_get_sg_cterm` for the nvim-highlight crate.
#[export_name = "nvim_hl_table_get_sg_cterm"]
pub unsafe extern "C" fn compat_hl_table_get_sg_cterm(idx: c_int) -> c_int {
    (*hl_table_ptr(idx)).sg_cterm
}
