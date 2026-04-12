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
    static Columns: c_int;
    static mut msg_silent: c_int;
    static mut got_int: bool;
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

}

// C accessors used by the Phase 1 Rust implementation of syn_add_group.
extern "C" {
    fn nvim_hlg_alloc_entry(id_out: *mut c_int) -> *mut c_void;
    fn nvim_hlg_arena_memdupz(name: *const c_char, len: usize) -> *mut c_char;
    fn nvim_hlg_vim_strup(s: *mut c_char);
    fn nvim_hlg_unames_put(name_u: *const c_char, id: c_int);
    fn nvim_hlg_emsg(msg: *const c_char);
    fn nvim_hlg_msg_source();
    fn nvim_hlg_vim_isprintc(c: c_int) -> c_int;
    fn nvim_hlg_xmemrchr(s: *const c_void, c: c_int, n: usize) -> *mut c_void;
}

// C accessors used by the Phase 2 Rust implementation of set_hl_group.
extern "C" {
    /// Extract Dict(highlight) fields into HlGroupSetInfo bridge struct.
    fn nvim_hlg_extract_set_info(dict: *mut c_void) -> crate::types::HlGroupSetInfo;
    /// Get current_sctx value.
    fn nvim_hlg_get_current_sctx() -> crate::types::SctxT;
    /// Get SOURCING_LNUM value.
    fn nvim_hlg_get_sourcing_lnum() -> c_int;
    /// Call nlua_set_sctx.
    fn nvim_hlg_nlua_set_sctx(sctx: *mut crate::types::SctxT);
    /// Call name_to_color.
    fn nvim_hlg_name_to_color(name: *const c_char, idx: *mut c_int) -> c_int;
    /// Call highlight_attr_set_all().
    fn nvim_hlg_highlight_attr_set_all();
    /// Call ui_default_colors_set().
    fn nvim_hlg_ui_default_colors_set();
    /// Call redraw_all_later(UPD_NOT_VALID).
    fn nvim_hlg_redraw_all_later();
    /// Get updating_screen global.
    fn nvim_hlg_updating_screen() -> bool;
    /// Set need_highlight_changed = true.
    fn nvim_hlg_set_need_highlight_changed();
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

// =============================================================================
// Accessor functions migrated from highlight_group.c (Phase 2)
//
// These replace the C implementations with direct Rust access to highlight_ga.
// Callers in C and Rust link to these symbols identically.
// =============================================================================

/// Returns the name of a highlight group (0-based index).
#[export_name = "highlight_group_name"]
pub unsafe extern "C" fn rs_highlight_group_name(id: c_int) -> *mut c_char {
    (*hl_table_ptr(id)).sg_name
}

/// Returns the link ID of a highlight group (0-based index).
#[export_name = "highlight_link_id"]
pub unsafe extern "C" fn rs_highlight_link_id(id: c_int) -> c_int {
    (*hl_table_ptr(id)).sg_link
}

/// Returns the screen attribute of a highlight group (0-based index).
/// Returns 0 if `id` is out of bounds.
#[export_name = "highlight_group_attr"]
pub unsafe extern "C" fn rs_highlight_group_attr(id: c_int) -> c_int {
    if !is_valid_index(id) {
        return 0;
    }
    (*hl_table_ptr(id)).sg_attr
}

/// Returns whether a highlight group has been cleared (0-based index).
/// Returns false if `id` is out of bounds.
#[export_name = "highlight_group_cleared"]
pub unsafe extern "C" fn rs_highlight_group_cleared(id: c_int) -> bool {
    if !is_valid_index(id) {
        return false;
    }
    (*hl_table_ptr(id)).sg_cleared
}

/// Returns the sg_set flags of a highlight group (0-based index).
/// Returns 0 if `id` is out of bounds.
#[export_name = "highlight_group_set"]
pub unsafe extern "C" fn rs_highlight_group_set(id: c_int) -> c_int {
    if !is_valid_index(id) {
        return 0;
    }
    (*hl_table_ptr(id)).sg_set
}

/// Returns the parent ID of a highlight group (0-based index).
/// Returns 0 if `id` is out of bounds.
#[export_name = "highlight_group_parent"]
pub unsafe extern "C" fn rs_highlight_group_parent(id: c_int) -> c_int {
    if !is_valid_index(id) {
        return 0;
    }
    (*hl_table_ptr(id)).sg_parent
}

// =============================================================================
// init_highlight migrated from highlight_group.c (Phase 2)
// =============================================================================

extern "C" {
    /// Get the value of a global Vimscript variable (e.g. "g:colors_name").
    /// Returns NULL if the variable is not set.
    fn get_var_value(name: *const c_char) -> *mut c_char;

    /// Load a colorscheme file by name. Returns 1 (OK) on success, 0 on failure.
    fn load_colors(name: *mut c_char) -> c_int;

    /// Process one `:highlight` command line (init path).
    fn do_highlight(line: *const c_char, forceit: bool, init: bool);

    /// Initialize the cmdline syntax highlight colors.
    fn syn_init_cmdline_highlight(both: bool, reset: bool);

    /// Duplicate a C string (xstrdup).
    fn xstrdup(s: *const c_char) -> *mut c_char;

    /// Free memory allocated by xmalloc/xstrdup.
    fn xfree(ptr: *mut c_void);
}

use std::sync::atomic::{AtomicBool, Ordering};

// =============================================================================
// Utility functions migrated from highlight_group.c (Phase 3)
//
// hl_has_settings, highlight_clear, set_hl_attr
// =============================================================================

extern "C" {
    /// Compute and store the screen attribute for a highlight group.
    /// (hl_get_syn_attr is in the nvim-highlight crate)
    fn hl_get_syn_attr(ns_id: c_int, idx: c_int, attrs: crate::HlAttrs) -> c_int;

    /// Check if any cursor mode entry uses the given syntax ID.
    fn cursor_mode_uses_syn_id(syn_id: c_int) -> bool;

    /// Notify the UI about a mode info change.
    fn ui_mode_info_set();
}

/// Returns true if highlight group `idx` has any settings.
///
/// `check_link`: if true, also check for an existing link target.
/// Was `static bool hl_has_settings(int idx, bool check_link)` in C.
#[export_name = "hl_has_settings"]
pub unsafe extern "C" fn rs_hl_has_settings(idx: c_int, check_link: bool) -> bool {
    let sg = &*hl_table_ptr(idx);
    !sg.sg_cleared
        && (sg.sg_attr != 0
            || sg.sg_cterm_fg != 0
            || sg.sg_cterm_bg != 0
            || sg.sg_rgb_fg_idx != crate::types::ColorIdx::None as c_int
            || sg.sg_rgb_bg_idx != crate::types::ColorIdx::None as c_int
            || sg.sg_rgb_sp_idx != crate::types::ColorIdx::None as c_int
            || (check_link && (sg.sg_set & crate::types::SgSet::LINK.0) != 0))
}

/// Clear all highlight settings for group `idx`.
///
/// Restores default link/context if they are set.
/// Was `static void highlight_clear(int idx)` in C.
#[export_name = "highlight_clear"]
pub unsafe extern "C" fn rs_highlight_clear(idx: c_int) {
    let sg = &mut *hl_table_ptr(idx);
    sg.sg_cleared = true;
    sg.sg_attr = 0;
    sg.sg_cterm = 0;
    sg.sg_cterm_bold = false;
    sg.sg_cterm_fg = 0;
    sg.sg_cterm_bg = 0;
    sg.sg_gui = 0;
    sg.sg_rgb_fg = -1;
    sg.sg_rgb_bg = -1;
    sg.sg_rgb_sp = -1;
    sg.sg_rgb_fg_idx = crate::types::ColorIdx::None as c_int;
    sg.sg_rgb_bg_idx = crate::types::ColorIdx::None as c_int;
    sg.sg_rgb_sp_idx = crate::types::ColorIdx::None as c_int;
    sg.sg_blend = -1;
    // Restore default link and context if they exist.
    sg.sg_link = sg.sg_deflink;
    sg.sg_script_ctx = sg.sg_deflink_sctx;
}

/// Compute and set the screen attribute for highlight group `idx`.
///
/// Builds an `HlAttrs` from the group fields and calls `hl_get_syn_attr`.
/// Was `static void set_hl_attr(int idx)` in C.
#[export_name = "set_hl_attr"]
pub unsafe extern "C" fn rs_set_hl_attr(idx: c_int) {
    let sg = &*hl_table_ptr(idx);
    let none = crate::types::ColorIdx::None as c_int;

    let at_en = crate::HlAttrs {
        cterm_ae_attr: sg.sg_cterm as i16,
        cterm_fg_color: sg.sg_cterm_fg as i16,
        cterm_bg_color: sg.sg_cterm_bg as i16,
        rgb_ae_attr: sg.sg_gui as i16,
        rgb_fg_color: if sg.sg_rgb_fg_idx != none {
            sg.sg_rgb_fg
        } else {
            -1
        },
        rgb_bg_color: if sg.sg_rgb_bg_idx != none {
            sg.sg_rgb_bg
        } else {
            -1
        },
        rgb_sp_color: if sg.sg_rgb_sp_idx != none {
            sg.sg_rgb_sp
        } else {
            -1
        },
        hl_blend: sg.sg_blend,
        url: -1,
    };

    (*hl_table_ptr(idx)).sg_attr = hl_get_syn_attr(0, idx + 1, at_en);

    // A cursor style uses this syn_id — make sure its attribute is updated.
    if cursor_mode_uses_syn_id(idx + 1) {
        ui_mode_info_set();
    }
}

/// Tracks whether `init_highlight(both=true, ...)` has been called yet.
static HAD_BOTH: AtomicBool = AtomicBool::new(false);

/// Load colors from a file if "g:colors_name" is set, otherwise load
/// compiled-in defaults.
///
/// - `both`: if true, apply groups that apply to both backgrounds and set the
///   `had_both` flag so subsequent calls with `both=false` proceed.
/// - `reset`: if true, clear groups before reapplying.
#[export_name = "init_highlight"]
pub unsafe extern "C" fn rs_init_highlight(both: bool, reset: bool) {
    // Try finding a color scheme file. Used when a color file was loaded
    // and 'background' or 't_Co' is changed.
    let p = get_var_value(c"g:colors_name".as_ptr());
    if !p.is_null() {
        // Value of g:colors_name could be freed inside load_colors(), so copy it.
        let copy_p = xstrdup(p);
        let okay = load_colors(copy_p) != 0;
        xfree(copy_p as *mut c_void);
        if okay {
            return;
        }
    }

    // Didn't use a color file; use the compiled-in defaults.
    if both {
        HAD_BOTH.store(true, Ordering::Relaxed);
        let mut i = 0;
        while !crate::init_tables::highlight_init_both.0[i].is_null() {
            do_highlight(crate::init_tables::highlight_init_both.0[i], reset, true);
            i += 1;
        }
    } else if !HAD_BOTH.load(Ordering::Relaxed) {
        // Don't do anything before the call with both == true from main().
        // Not everything has been set up then, and that call will overrule
        // everything anyway.
        return;
    }

    // Apply background-specific defaults.
    let table = if *p_bg == b'l' as i8 {
        &crate::init_tables::highlight_init_light.0[..]
    } else {
        &crate::init_tables::highlight_init_dark.0[..]
    };
    let mut i = 0;
    while !table[i].is_null() {
        do_highlight(table[i], reset, true);
        i += 1;
    }

    syn_init_cmdline_highlight(false, false);
}

// =============================================================================
// Phase 4: highlight_color and highlight_list Functions
//
// highlight_color, highlight_list_one, highlight_list_arg, syn_list_header
// =============================================================================

extern "C" {
    /// Check whether any RGB-capable UI is attached.
    fn ui_rgb_attached() -> bool;

    /// Check whether a UI extension is enabled.
    /// ext=4 is kUIMessages.
    fn ui_has(ext: c_int) -> bool;

    /// Output string with highlight.
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);

    /// Output a character to the message area.
    fn msg_putchar(c: c_int);

    /// Output a string, translating special characters.
    /// Returns msg_col after the string.
    fn msg_outtrans(s: *const c_char, hl_id: c_int, hist: bool) -> c_int;

    /// Move the message cursor to column `col`.
    fn msg_advance(col: c_int);

    /// Get the current message column.
    static mut msg_col: c_int;

    /// Set the current message column.

    /// Get the Columns (screen width) global.

    /// Get the unsafe { got_int } global (returns c_int; non-zero means interrupted).

    /// Get the msg_silent global.

    /// Get the p_verbose global.
    fn nvim_get_p_verbose() -> c_int;

    /// Check if a message would be filtered (not displayed).
    fn message_filtered(msg: *const c_char) -> bool;

    /// Show the "last set" message for a script context.
    fn last_set_msg(script_ctx: crate::types::SctxT);

    /// Return the display width of a string in columns.
    fn vim_strsize(s: *const c_char) -> c_int;

    /// Return color name string for a color index.
    fn coloridx_to_name(idx: c_int, val: c_int, hexbuf: *mut c_char) -> *const c_char;
}

// kUIMessages = 4 (UIExtension enum in ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

/// Return color name of the given highlight group.
///
/// @param id Highlight group ID (1-based)
/// @param what What to return: "font", "fg", "bg", "sp", "fg#", "bg#", "sp#"
/// @param modec 'g' for GUI, 'c' for cterm, 't' for term
///
/// # Safety
/// `what` must be a valid nul-terminated C string.
#[export_name = "highlight_color"]
pub unsafe extern "C" fn rs_highlight_color(
    id: c_int,
    what: *const c_char,
    modec: c_int,
) -> *const c_char {
    // Static buffer for cterm number and hex color string.
    // SAFETY: this matches the C `static char name[20]` pattern.
    // Neovim is single-threaded for message processing.
    static mut NAME_BUF: [c_char; 20] = [0; 20];

    if id <= 0 || id > highlight_ga.ga_len {
        return std::ptr::null();
    }

    // Read the first 4 bytes, lowercase for case-insensitive comparison
    let w0 = (*what as u8).to_ascii_lowercase();
    let w1 = (*what.add(1) as u8).to_ascii_lowercase();
    let w2 = (*what.add(2) as u8).to_ascii_lowercase();
    let w3 = (*what.add(3) as u8).to_ascii_lowercase();
    // also keep raw byte 2 for '#' check
    let raw2 = *what.add(2) as u8;

    // Determine what kind of color is requested
    let is_fg = w0 == b'f' && w1 == b'g';
    let is_sp = w0 == b's' && w1 == b'p';
    let is_font = w0 == b'f' && w1 == b'o' && w2 == b'n' && w3 == b't';
    let is_bg = w0 == b'b' && w1 == b'g';

    if !is_fg && !is_sp && !is_font && !is_bg {
        return std::ptr::null();
    }

    let sg = &*hl_table_ptr(id - 1);

    // Raw pointer to NAME_BUF — never create a reference to it
    let name_buf_ptr = std::ptr::addr_of_mut!(NAME_BUF) as *mut c_char;

    if modec == b'g' as c_int {
        // GUI colors
        if raw2 == b'#' && ui_rgb_attached() {
            let n = if is_fg {
                sg.sg_rgb_fg
            } else if is_sp {
                sg.sg_rgb_sp
            } else {
                sg.sg_rgb_bg
            };
            if !(0..=0x00ff_ffff).contains(&n) {
                return std::ptr::null();
            }
            // Format as "#rrggbb" directly into the static buffer
            let s = std::format!("#{:06x}\0", n as u32);
            let bytes = s.as_bytes();
            for (i, &b) in bytes.iter().enumerate().take(20) {
                *name_buf_ptr.add(i) = b as c_char;
            }
            return name_buf_ptr as *const c_char;
        }
        // Return color name
        return if is_fg {
            coloridx_to_name(sg.sg_rgb_fg_idx, sg.sg_rgb_fg, name_buf_ptr)
        } else if is_sp {
            coloridx_to_name(sg.sg_rgb_sp_idx, sg.sg_rgb_sp, name_buf_ptr)
        } else {
            coloridx_to_name(sg.sg_rgb_bg_idx, sg.sg_rgb_bg, name_buf_ptr)
        };
    }

    // Non-GUI: font and sp not supported
    if is_font || is_sp {
        return std::ptr::null();
    }

    if modec == b'c' as c_int {
        let n = if is_fg {
            sg.sg_cterm_fg - 1
        } else {
            sg.sg_cterm_bg - 1
        };
        if n < 0 {
            return std::ptr::null();
        }
        // Format number directly into the static buffer
        let s = std::format!("{}\0", n);
        let bytes = s.as_bytes();
        for (i, &b) in bytes.iter().enumerate().take(20) {
            *name_buf_ptr.add(i) = b as c_char;
        }
        return name_buf_ptr as *const c_char;
    }

    // term doesn't have color
    std::ptr::null()
}

/// Output the syntax list header.
///
/// @param did_header  did header already
/// @param outlen      length of string that comes after
/// @param id          highlight group id (1-based)
/// @param force_newline  always start a new line
/// @return true when started a new line
#[export_name = "syn_list_header"]
pub unsafe extern "C" fn rs_syn_list_header(
    did_header: bool,
    outlen: c_int,
    id: c_int,
    force_newline: bool,
) -> bool {
    let mut endcol: c_int = 19;
    let mut newline = true;
    let mut name_col: c_int = 0;
    let mut adjust = true;

    if !did_header {
        msg_putchar(b'\n' as c_int);
        if unsafe { got_int } {
            return true;
        }
        let col = msg_outtrans((*hl_table_ptr(id - 1)).sg_name, 0, false);
        msg_col = col;
        name_col = col;
        endcol = 15;
    } else if (ui_has(K_UI_MESSAGES) || msg_silent != 0) && !force_newline {
        msg_putchar(b' ' as c_int);
        adjust = false;
    } else if msg_col + outlen + 1 >= Columns || force_newline {
        msg_putchar(b'\n' as c_int);
        if unsafe { got_int } {
            return true;
        }
    } else if msg_col >= endcol {
        // wrap around is like starting a new line
        newline = false;
    }

    if adjust {
        if msg_col >= endcol {
            // output at least one space
            endcol = msg_col + 1;
        }
        msg_advance(endcol);
    }

    // Show "xxx" with the attributes.
    if !did_header {
        if endcol == Columns - 1 && endcol <= name_col {
            msg_putchar(b' ' as c_int);
        }
        msg_puts_hl(c"xxx".as_ptr(), id, false);
        msg_putchar(b' ' as c_int);
    }

    newline
}

// HL_UNDERLINE_MASK as c_int for use in highlight_list_arg
const HL_UNDERLINE_MASK_INT: c_int = 0x38;

// LIST_XXX constants matching C
const LIST_ATTR: c_int = 1;
const LIST_STRING: c_int = 2;
const LIST_INT: c_int = 3;

/// Outputs a highlight when doing ":hi MyHighlight".
///
/// @param id       highlight group id (1-based)
/// @param didh     whether the header was already output
/// @param type     one of LIST_ATTR, LIST_STRING, LIST_INT
/// @param iarg     integer argument (for LIST_INT and LIST_ATTR)
/// @param sarg     string argument (for LIST_STRING)
/// @param name     attribute name (e.g. "cterm", "guifg")
/// @return new value of didh
#[export_name = "highlight_list_arg"]
pub unsafe extern "C" fn rs_highlight_list_arg(
    id: c_int,
    didh: bool,
    r#type: c_int,
    mut iarg: c_int,
    sarg: *const c_char,
    name: *const c_char,
) -> bool {
    if unsafe { got_int } {
        return false;
    }

    // If nothing to show, return unchanged didh
    let is_empty = if r#type == LIST_STRING {
        sarg.is_null()
    } else {
        iarg == 0
    };
    if is_empty {
        return didh;
    }

    // Build the value string.
    // For LIST_INT: the value is iarg-1 (stored as +1)
    // For LIST_STRING: the value is sarg directly
    // For LIST_ATTR: build a comma-separated list of matching attribute names
    let ts: *const c_char;
    // We use a fixed-size stack buffer for INT and ATTR cases.
    // Safety: 100 bytes is enough for any attribute list or integer.
    let mut buf = [0u8; 100];

    if r#type == LIST_INT {
        let s = std::format!("{}", iarg - 1);
        let bytes = s.as_bytes();
        let len = bytes.len().min(99);
        buf[..len].copy_from_slice(&bytes[..len]);
        buf[len] = 0;
        ts = buf.as_ptr() as *const c_char;
    } else if r#type == LIST_STRING {
        ts = sarg;
    } else {
        // LIST_ATTR: build comma-separated list
        buf[0] = 0;
        let mut pos = 0usize;
        for &(attr_name, flag) in crate::parse::HL_NAME_TABLE {
            let flag_int = flag as c_int;
            if flag_int == 0 {
                continue; // skip NONE entry
            }
            let matches = if flag_int & HL_UNDERLINE_MASK_INT != 0 {
                (iarg & HL_UNDERLINE_MASK_INT) == flag_int
            } else {
                (iarg & flag_int) != 0
            };
            if matches {
                if pos > 0 && pos < 99 {
                    buf[pos] = b',';
                    pos += 1;
                }
                let name_bytes = attr_name.as_bytes();
                let copy_len = name_bytes.len().min(99 - pos);
                buf[pos..pos + copy_len].copy_from_slice(&name_bytes[..copy_len]);
                pos += copy_len;
                if flag_int & HL_UNDERLINE_MASK_INT == 0 {
                    iarg &= !flag_int; // don't want "inverse" twice
                }
            }
        }
        buf[pos.min(99)] = 0;
        ts = buf.as_ptr() as *const c_char;
    }

    rs_syn_list_header(didh, vim_strsize(ts) + vim_strsize(name) + 1, id, false);
    if !unsafe { got_int } {
        // if *name != NUL
        if *name != 0 {
            msg_puts_hl(name, HLF_D, false);
            msg_puts_hl(c"=".as_ptr(), HLF_D, false);
        }
        msg_outtrans(ts, 0, false);
    }
    true
}

// HLF_D = 5 (same as syntax/src/listing.rs)
const HLF_D: c_int = 5;

/// Outputs a highlight when doing ":hi MyHighlight".
///
/// @param id highlight group ID (1-based)
#[export_name = "highlight_list_one"]
pub unsafe extern "C" fn rs_highlight_list_one(id: c_int) {
    let sgp = &*hl_table_ptr(id - 1);
    let mut didh = false;

    if message_filtered(sgp.sg_name) {
        return;
    }

    // don't list specialized groups if a parent is used instead
    if sgp.sg_parent != 0 && sgp.sg_cleared {
        return;
    }

    didh = rs_highlight_list_arg(
        id,
        didh,
        LIST_ATTR,
        sgp.sg_cterm,
        std::ptr::null(),
        c"cterm".as_ptr(),
    );
    didh = rs_highlight_list_arg(
        id,
        didh,
        LIST_INT,
        sgp.sg_cterm_fg,
        std::ptr::null(),
        c"ctermfg".as_ptr(),
    );
    didh = rs_highlight_list_arg(
        id,
        didh,
        LIST_INT,
        sgp.sg_cterm_bg,
        std::ptr::null(),
        c"ctermbg".as_ptr(),
    );

    didh = rs_highlight_list_arg(
        id,
        didh,
        LIST_ATTR,
        sgp.sg_gui,
        std::ptr::null(),
        c"gui".as_ptr(),
    );

    let mut hexbuf = [0u8; 8];
    let hbuf = hexbuf.as_mut_ptr() as *mut c_char;
    let fg_name = coloridx_to_name(sgp.sg_rgb_fg_idx, sgp.sg_rgb_fg, hbuf);
    didh = rs_highlight_list_arg(id, didh, LIST_STRING, 0, fg_name, c"guifg".as_ptr());

    let mut hexbuf2 = [0u8; 8];
    let hbuf2 = hexbuf2.as_mut_ptr() as *mut c_char;
    let bg_name = coloridx_to_name(sgp.sg_rgb_bg_idx, sgp.sg_rgb_bg, hbuf2);
    didh = rs_highlight_list_arg(id, didh, LIST_STRING, 0, bg_name, c"guibg".as_ptr());

    let mut hexbuf3 = [0u8; 8];
    let hbuf3 = hexbuf3.as_mut_ptr() as *mut c_char;
    let sp_name = coloridx_to_name(sgp.sg_rgb_sp_idx, sgp.sg_rgb_sp, hbuf3);
    didh = rs_highlight_list_arg(id, didh, LIST_STRING, 0, sp_name, c"guisp".as_ptr());

    didh = rs_highlight_list_arg(
        id,
        didh,
        LIST_INT,
        sgp.sg_blend + 1,
        std::ptr::null(),
        c"blend".as_ptr(),
    );

    if sgp.sg_link != 0 && !unsafe { got_int } {
        rs_syn_list_header(didh, 0, id, true);
        // didh = true (but we don't need to use it further)
        msg_puts_hl(c"links to".as_ptr(), HLF_D, false);
        msg_putchar(b' ' as c_int);
        msg_outtrans(
            (*hl_table_ptr((*hl_table_ptr(id - 1)).sg_link - 1)).sg_name,
            0,
            false,
        );
    } else if !didh {
        rs_highlight_list_arg(id, false, LIST_STRING, 0, c"cleared".as_ptr(), c"".as_ptr());
    }

    if nvim_get_p_verbose() as i64 > 0 {
        last_set_msg(sgp.sg_script_ctx);
    }
}

// =============================================================================
// Phase 1: Completion and Display Helpers
//
// set_context_in_highlight_cmd, get_highlight_name_ext, get_highlight_name,
// highlight_list, highlight_list_two
// =============================================================================

extern "C" {
    // expand_T field accessors (already provided by option_shim.c)
    fn nvim_xp_set_context(xp: *mut c_void, val: c_int);
    fn nvim_xp_set_pattern(xp: *mut c_void, val: *mut c_char);

    // include_link/default/none setters/getters (in syntax_accessors.c)
    fn nvim_syn_set_include_link(val: c_int);
    fn nvim_syn_set_include_default(val: c_int);
    fn nvim_syn_get_include_none() -> c_int;
    fn nvim_syn_get_include_default() -> c_int;
    fn nvim_syn_get_include_link() -> c_int;

    fn skiptowhite(s: *const c_char) -> *mut c_char;
    fn skipwhite(s: *const c_char) -> *mut c_char;

    fn msg_clr_eos();
    fn ui_flush();
    fn os_delay(msec: u64, ignoreinput: bool);
}

// EXPAND_* constants (from cmdexpand_defs.h)
const EXPAND_NOTHING: c_int = 0;
const EXPAND_HIGHLIGHT: c_int = 13;

/// Handle command line completion for :highlight command.
///
/// # Safety
/// `xp` must be a valid expand_T pointer. `arg` must be a valid C string.
#[export_name = "set_context_in_highlight_cmd"]
pub unsafe extern "C" fn rs_set_context_in_highlight_cmd(xp: *mut c_void, arg: *const c_char) {
    // Default: expand group names.
    nvim_xp_set_context(xp, EXPAND_HIGHLIGHT);
    nvim_xp_set_pattern(xp, arg as *mut c_char);
    nvim_syn_set_include_link(2);
    nvim_syn_set_include_default(1);

    if *arg == 0 {
        return;
    }

    // (part of) subcommand already typed
    let p = skiptowhite(arg);
    if *p == 0 {
        return;
    }

    // past "default" or group name
    nvim_syn_set_include_default(0);
    let p_len = p.offset_from(arg) as usize;
    if p_len == 7 && libc_strncmp(arg, c"default".as_ptr(), 7) == 0 {
        let arg2 = skipwhite(p);
        nvim_xp_set_pattern(xp, arg2);
        let p2 = skiptowhite(arg2);
        if *p2 == 0 {
            return;
        }
        // fall through with updated arg and p
        let p2_len = p2.offset_from(arg2) as usize;
        // past group name
        nvim_syn_set_include_link(0);
        if (*arg2.add(1) == b'i' as c_char) && (*arg2 == b'N' as c_char) {
            rs_highlight_list_internal();
        }
        if (p2_len == 4 && libc_strncmp(arg2, c"link".as_ptr(), 4) == 0)
            || (p2_len == 5 && libc_strncmp(arg2, c"clear".as_ptr(), 5) == 0)
        {
            let pat = skipwhite(p2);
            nvim_xp_set_pattern(xp, pat);
            let p3 = skiptowhite(pat);
            if *p3 != 0 {
                // past first group name
                let pat2 = skipwhite(p3);
                nvim_xp_set_pattern(xp, pat2);
                let p4 = skiptowhite(pat2);
                if *p4 != 0 {
                    nvim_xp_set_context(xp, EXPAND_NOTHING);
                }
            }
        } else if *p2 != 0 {
            // past group name(s)
            nvim_xp_set_context(xp, EXPAND_NOTHING);
        }
        return;
    }

    // past group name
    nvim_syn_set_include_link(0);
    if (*arg.add(1) == b'i' as c_char) && (*arg == b'N' as c_char) {
        rs_highlight_list_internal();
    }
    if (p_len == 4 && libc_strncmp(arg, c"link".as_ptr(), 4) == 0)
        || (p_len == 5 && libc_strncmp(arg, c"clear".as_ptr(), 5) == 0)
    {
        let pat = skipwhite(p);
        nvim_xp_set_pattern(xp, pat);
        let p2 = skiptowhite(pat);
        if *p2 != 0 {
            // past first group name
            let pat2 = skipwhite(p2);
            nvim_xp_set_pattern(xp, pat2);
            let p3 = skiptowhite(pat2);
            if *p3 != 0 {
                nvim_xp_set_context(xp, EXPAND_NOTHING);
            }
        }
    } else if *p != 0 {
        // past group name(s)
        nvim_xp_set_context(xp, EXPAND_NOTHING);
    }
}

// Thin wrapper over libc strncmp for use in this module.
#[inline]
unsafe fn libc_strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int {
    extern "C" {
        fn strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;
    }
    strncmp(a, b, n)
}

/// Internal highlight list animation.
unsafe fn rs_highlight_list_internal() {
    for i in (0..10).rev() {
        rs_highlight_list_two_internal(i, HLF_D);
    }
    for _ in (0..40).rev() {
        rs_highlight_list_two_internal(99, 0);
    }
}

unsafe fn rs_highlight_list_two_internal(cnt: c_int, id: c_int) {
    // "N \bI \b!  \b" indexed by cnt/11
    let s = b"N \x08I \x08!  \x08\0";
    let idx = (cnt / 11) as usize;
    let ptr = s[idx..].as_ptr() as *const c_char;
    msg_puts_hl(ptr, id, false);
    msg_clr_eos();
    ui_flush();
    let delay_ms: u64 = if cnt == 99 { 40 } else { cnt as u64 * 50 };
    os_delay(delay_ms, false);
}

/// Function given to ExpandGeneric() to obtain the list of group names.
///
/// # Safety
/// `xp` must be valid (unused but required by callback signature).
#[export_name = "get_highlight_name"]
pub unsafe extern "C" fn rs_get_highlight_name(xp: *mut c_void, idx: c_int) -> *mut c_char {
    rs_get_highlight_name_ext(xp, idx, true) as *mut c_char
}

/// Obtain a highlight group name.
///
/// @param skip_cleared  if true don't return a cleared entry.
///
/// # Safety
/// `xp` must be valid (unused but required by callback signature).
#[export_name = "get_highlight_name_ext"]
pub unsafe extern "C" fn rs_get_highlight_name_ext(
    _xp: *mut c_void,
    idx: c_int,
    skip_cleared: bool,
) -> *const c_char {
    if idx < 0 {
        return std::ptr::null();
    }

    let ga_len = highlight_ga.ga_len;

    // Items are never removed from the table, skip the ones that were cleared.
    if skip_cleared && idx < ga_len && (*hl_table_ptr(idx)).sg_cleared {
        return c"".as_ptr();
    }

    let include_none = nvim_syn_get_include_none();
    let include_default = nvim_syn_get_include_default();
    let include_link = nvim_syn_get_include_link();

    if idx == ga_len && include_none != 0 {
        return c"none".as_ptr();
    } else if idx == ga_len + include_none && include_default != 0 {
        return c"default".as_ptr();
    } else if idx == ga_len + include_none + include_default && include_link != 0 {
        return c"link".as_ptr();
    } else if idx == ga_len + include_none + include_default + 1 && include_link != 0 {
        return c"clear".as_ptr();
    } else if idx >= ga_len {
        return std::ptr::null();
    }
    (*hl_table_ptr(idx)).sg_name
}

// =============================================================================
// Phase 5: highlight_attr_set_all and load_colors
// =============================================================================

extern "C" {
    /// Get curbuf as an opaque pointer (buf_T *).
    fn nvim_get_curbuf() -> *mut c_void;

    /// Get curbuf->b_fname (the current buffer's file name).
    fn nvim_get_curbuf_b_fname() -> *const c_char;

    /// Apply autocommands for event `event`, with pattern `fname`, I/O file `fname_io`.
    /// Returns true if the autocommands were applied (at least one matched).
    fn apply_autocmds(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: *mut c_void,
    ) -> bool;

    /// Source a runtime file matching `name` with the given `flags`.
    /// Returns OK (0) on success.
    fn source_runtime_vim_lua(name: *mut c_char, flags: c_int) -> c_int;

    /// Allocate memory. Panics (abort) on OOM, like xmalloc.
    fn xmalloc(size: usize) -> *mut c_void;
}

// Autocmd event IDs (from auevents_enum.generated.h)
const EVENT_COLORSCHEME: c_int = 32;
const EVENT_COLORSCHEMEPRE: c_int = 33;

// DIP flags for source_runtime_vim_lua
const DIP_START: c_int = 0x08;
const DIP_OPT: c_int = 0x10;

// OK return value (matches C's OK == 0)
const OK: c_int = 0;

/// Refresh the actual RGB colors for all highlight groups that use
/// "fg" or "bg" as color values.
///
/// Called after `normal_fg`/`normal_bg` change.
/// Was `void highlight_attr_set_all(void)` in C.
#[export_name = "highlight_attr_set_all"]
pub unsafe extern "C" fn rs_highlight_attr_set_all() {
    let color_idx_fg = crate::types::ColorIdx::Fg as c_int;
    let color_idx_bg = crate::types::ColorIdx::Bg as c_int;

    for idx in 0..highlight_ga.ga_len {
        let sg = &mut *hl_table_ptr(idx);

        if sg.sg_rgb_bg_idx == color_idx_fg {
            sg.sg_rgb_bg = normal_fg;
        } else if sg.sg_rgb_bg_idx == color_idx_bg {
            sg.sg_rgb_bg = normal_bg;
        }

        if sg.sg_rgb_fg_idx == color_idx_fg {
            sg.sg_rgb_fg = normal_fg;
        } else if sg.sg_rgb_fg_idx == color_idx_bg {
            sg.sg_rgb_fg = normal_bg;
        }

        if sg.sg_rgb_sp_idx == color_idx_fg {
            sg.sg_rgb_sp = normal_fg;
        } else if sg.sg_rgb_sp_idx == color_idx_bg {
            sg.sg_rgb_sp = normal_bg;
        }

        rs_set_hl_attr(idx);
    }
}

/// Load a color scheme file by name.
///
/// Applies ColorSchemePre and ColorScheme autocommands.
/// Returns OK (0) on success.
/// Was `int load_colors(char *name)` in C.
#[export_name = "load_colors"]
pub unsafe extern "C" fn rs_load_colors(name: *mut c_char) -> c_int {
    // Track recursion: if load_colors is called while already loading
    // (e.g., from setting 'background'), return OK immediately.
    static RECURSIVE: AtomicBool = AtomicBool::new(false);
    if RECURSIVE.load(Ordering::Relaxed) {
        return OK;
    }

    RECURSIVE.store(true, Ordering::Relaxed);

    // Build "colors/<name>.*" pattern
    let name_len = {
        let mut p = name;
        while *p != 0 {
            p = p.add(1);
        }
        p.offset_from(name) as usize
    };
    let buflen = name_len + 12; // "colors/" + name + ".*" + NUL
    let buf = xmalloc(buflen) as *mut c_char;

    // Build the string: "colors/<name>.*\0"
    let prefix = b"colors/";
    for (i, &b) in prefix.iter().enumerate() {
        *buf.add(i) = b as c_char;
    }
    for i in 0..name_len {
        *buf.add(7 + i) = *name.add(i);
    }
    let suffix = b".*\0";
    for (i, &b) in suffix.iter().enumerate() {
        *buf.add(7 + name_len + i) = b as c_char;
    }

    let curbuf = nvim_get_curbuf();
    let b_fname = nvim_get_curbuf_b_fname();

    apply_autocmds(EVENT_COLORSCHEMEPRE, name, b_fname, false, curbuf);

    let retval = source_runtime_vim_lua(buf, DIP_START + DIP_OPT);
    xfree(buf as *mut c_void);

    if retval == OK {
        apply_autocmds(EVENT_COLORSCHEME, name, b_fname, false, curbuf);
    }

    RECURSIVE.store(false, Ordering::Relaxed);
    retval
}

// =============================================================================
// Phase 2: do_highlight
// =============================================================================

/// Result of rs_lookup_color (matches C LookupColorResult).
#[repr(C)]
struct LookupColorResult {
    color: c_int,
    bold: c_int, // -1 = unchanged, 0 = false, 1 = true
}

extern "C" {
    pub static mut current_sctx: crate::types::SctxT;
    pub static mut updating_screen: bool;
    pub static mut need_highlight_changed: bool;
    pub static mut starting: c_int;
    fn nvim_get_sourcing_lnum() -> i32;
    fn nvim_get_sourcing_name() -> *const c_char;
    fn nlua_set_sctx(ctx: *mut crate::types::SctxT);
    fn syn_name2id_len(name: *const c_char, len: usize) -> c_int;
    fn syn_check_group(name: *const c_char, len: usize) -> c_int;
    fn ends_excmd(c: u8) -> bool;
    fn msg_ext_set_kind(kind: *const c_char);
    fn highlight_list_one(id: c_int);
    fn semsg(fmt: *const c_char, ...);
    fn emsg(msg: *const c_char) -> bool;
    fn name_to_color(name: *const c_char, idx: *mut c_int) -> crate::types::RgbValue;
    fn do_unlet(name: *const c_char, name_len: usize, forceit: bool) -> c_int;
    fn restore_cterm_colors();
    fn redraw_all_later(r#type: c_int);
    fn ui_refresh();
    fn ui_default_colors_set();
    fn option_was_set(opt_idx: c_int) -> bool;
    fn set_option_value_give_err(opt_idx: c_int, value: DhOptVal, opt_flags: c_int);
    fn reset_option_was_set(opt_idx: c_int);
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn strcasecmp(a: *const c_char, b: *const c_char) -> c_int;
    fn strncasecmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;
    fn strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    fn atoi(s: *const c_char) -> c_int;
    fn strtol(s: *const c_char, end: *mut *mut c_char, base: c_int) -> i64;
    fn vim_memcpy_up(dst: *mut c_char, src: *const c_char, n: usize);
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    fn rs_lookup_color(idx: c_int, foreground: bool) -> LookupColorResult;
    fn init_highlight(both: bool, reset: bool);
}

/// OptVal for passing string to set_option_value_give_err.
/// Must match the C OptVal layout (option_defs.h).
#[repr(C)]
#[derive(Clone, Copy)]
struct DhOptValString {
    data: *mut c_char,
    size: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
union DhOptValData {
    boolean: c_int,
    number: i64,
    string: DhOptValString,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DhOptVal {
    type_: c_int, // kOptValTypeString = 2
    data: DhOptValData,
}

const K_OPT_VAL_TYPE_STRING: c_int = 2;
const K_OPT_BACKGROUND: c_int = 13; // kOptBackground index
const SG_CTERM: c_int = 2;
const SG_GUI: c_int = 4;
const SG_LINK: c_int = 8;
const K_COLOR_IDX_NONE: c_int = -1; // kColorIdxNone
const UPD_NOT_VALID: c_int = 40;
const UPD_SOME_VALID: c_int = 35;
const K_UI_LINEGRID: c_int = 5; // kUILinegrid

use nvim_highlight::hl_attr_flags::{
    HL_ALTFONT, HL_BOLD, HL_INVERSE, HL_ITALIC, HL_NOCOMBINE, HL_STANDOUT, HL_STRIKETHROUGH,
    HL_UNDERCURL, HL_UNDERDASHED, HL_UNDERDOTTED, HL_UNDERDOUBLE, HL_UNDERLINE, HL_UNDERLINE_MASK,
};

// Matches hl_name_table[] and hl_attr_table[] in C (14 entries).
const HL_NAME_ATTR: [(&[u8], i16); 14] = [
    (b"bold\0", HL_BOLD),
    (b"standout\0", HL_STANDOUT),
    (b"underline\0", HL_UNDERLINE),
    (b"undercurl\0", HL_UNDERCURL),
    (b"underdouble\0", HL_UNDERDOUBLE),
    (b"underdotted\0", HL_UNDERDOTTED),
    (b"underdashed\0", HL_UNDERDASHED),
    (b"italic\0", HL_ITALIC),
    (b"reverse\0", HL_INVERSE),
    (b"inverse\0", HL_INVERSE),
    (b"strikethrough\0", HL_STRIKETHROUGH),
    (b"altfont\0", HL_ALTFONT),
    (b"nocombine\0", HL_NOCOMBINE),
    (b"NONE\0", 0),
];

// color_names[28] from C
static COLOR_NAMES_28: [&[u8]; 28] = [
    b"Black\0",
    b"DarkBlue\0",
    b"DarkGreen\0",
    b"DarkCyan\0",
    b"DarkRed\0",
    b"DarkMagenta\0",
    b"Brown\0",
    b"DarkYellow\0",
    b"Gray\0",
    b"Grey\0",
    b"LightGray\0",
    b"LightGrey\0",
    b"DarkGray\0",
    b"DarkGrey\0",
    b"Blue\0",
    b"LightBlue\0",
    b"Green\0",
    b"LightGreen\0",
    b"Cyan\0",
    b"LightCyan\0",
    b"Red\0",
    b"LightRed\0",
    b"Magenta\0",
    b"LightMagenta\0",
    b"Yellow\0",
    b"LightYellow\0",
    b"White\0",
    b"NONE\0",
];

/// Parse attr string for TERM/CTERM/GUI key. Returns (attr, error).
/// Iterates through comma-separated attribute names, matching HL_NAME_ATTR table.
unsafe fn parse_attr_string(arg: *const c_char) -> (c_int, bool) {
    let mut attr: c_int = 0;
    let mut off: usize = 0;
    loop {
        if *arg.add(off) == 0 {
            break;
        }
        let mut found = false;
        // Iterate in reverse like C (so later entries take priority)
        let mut i = HL_NAME_ATTR.len();
        while i > 0 {
            i -= 1;
            let (name_bytes, flag) = HL_NAME_ATTR[i];
            let name_len = name_bytes.len() - 1; // strip NUL
            if strncasecmp(arg.add(off), name_bytes.as_ptr() as *const c_char, name_len) == 0 {
                if flag & HL_UNDERLINE_MASK != 0 {
                    attr &= !(HL_UNDERLINE_MASK as c_int);
                }
                attr |= flag as c_int;
                off += name_len;
                found = true;
                break;
            }
        }
        if !found {
            return (attr, true);
        }
        if *arg.add(off) == b',' as c_char {
            off += 1;
        }
    }
    (attr, false)
}

/// Handle ":highlight" command.
///
/// # Safety
/// `line` must be a valid NUL-terminated C string.
#[export_name = "do_highlight"]
pub unsafe extern "C" fn rs_do_highlight(line: *const c_char, forceit: bool, init: bool) {
    // If no argument, list current highlighting.
    if !init && ends_excmd(*line as u8) {
        msg_ext_set_kind(c"list_cmd".as_ptr());
        let mut i = 1;
        while i <= highlight_ga.ga_len && !got_int {
            highlight_list_one(i);
            i += 1;
        }
        return;
    }

    let mut dodefault = false;
    let mut line = line;

    // Isolate the name.
    let mut name_end = skiptowhite(line);
    let mut linep = skipwhite(name_end);

    // Check for "default" argument.
    // C uses strncmp(line, "default", name_len) which allows abbreviated prefixes
    // (e.g. "def" matches "default"). We match that behavior.
    let name_len = name_end.offset_from(line) as usize;
    if name_len > 0 && name_len <= 7 && strncmp(line, c"default".as_ptr(), name_len) == 0 {
        dodefault = true;
        line = linep;
        name_end = skiptowhite(line);
        linep = skipwhite(name_end);
    }

    let mut doclear = false;
    let mut dolink = false;

    // Check for "clear" or "link" argument (prefix matching, same as C).
    let name_len = name_end.offset_from(line) as usize;
    if name_len > 0 && name_len <= 5 && strncmp(line, c"clear".as_ptr(), name_len) == 0 {
        doclear = true;
    } else if name_len > 0 && name_len <= 4 && strncmp(line, c"link".as_ptr(), name_len) == 0 {
        dolink = true;
    }

    // ":highlight {group-name}": list highlighting for one group.
    if !doclear && !dolink && ends_excmd(*linep as u8) {
        let id = syn_name2id_len(line, name_len);
        if id == 0 {
            semsg(c"E411: Highlight group not found: %s".as_ptr(), line);
        } else {
            msg_ext_set_kind(c"list_cmd".as_ptr());
            highlight_list_one(id);
        }
        return;
    }

    // Handle ":highlight link {from} {to}" command.
    if dolink {
        let from_start = linep;
        let from_end = skiptowhite(from_start);
        let to_start = skipwhite(from_end);
        let to_end = skiptowhite(to_start);

        if ends_excmd(*from_start as u8) || ends_excmd(*to_start as u8) {
            semsg(
                c"E412: Not enough arguments: \":highlight link %s\"".as_ptr(),
                from_start,
            );
            return;
        }

        if !ends_excmd(*skipwhite(to_end) as u8) {
            semsg(
                c"E413: Too many arguments: \":highlight link %s\"".as_ptr(),
                from_start,
            );
            return;
        }

        let from_id = syn_check_group(from_start, from_end.offset_from(from_start) as usize);
        let to_id = if strncmp(to_start, c"NONE".as_ptr(), 4) == 0 {
            0
        } else {
            syn_check_group(to_start, to_end.offset_from(to_start) as usize)
        };

        if from_id > 0 {
            let hlgroup = &mut *hl_table_ptr(from_id - 1);
            if dodefault && (forceit || hlgroup.sg_deflink == 0) {
                hlgroup.sg_deflink = to_id;
                hlgroup.sg_deflink_sctx = current_sctx;
                hlgroup.sg_deflink_sctx.sc_lnum += nvim_get_sourcing_lnum();
                nlua_set_sctx(&mut hlgroup.sg_deflink_sctx);
            }

            if !init || hlgroup.sg_set == 0 {
                if to_id > 0 && !forceit && !init && rs_hl_has_settings(from_id - 1, dodefault) {
                    if nvim_get_sourcing_name().is_null() && !dodefault {
                        emsg(c"E414: Group has settings, highlight link ignored".as_ptr());
                    }
                } else if hlgroup.sg_link != to_id
                    || hlgroup.sg_script_ctx.sc_sid != current_sctx.sc_sid
                    || hlgroup.sg_cleared
                {
                    if !init {
                        hlgroup.sg_set |= SG_LINK;
                    }
                    hlgroup.sg_link = to_id;
                    hlgroup.sg_script_ctx = current_sctx;
                    hlgroup.sg_script_ctx.sc_lnum += nvim_get_sourcing_lnum();
                    nlua_set_sctx(&mut hlgroup.sg_script_ctx);
                    hlgroup.sg_cleared = false;
                    redraw_all_later(UPD_SOME_VALID);
                    need_highlight_changed = true;
                }
            }
        }

        return;
    }

    if doclear {
        // ":highlight clear [group]" command.
        line = linep;
        if ends_excmd(*line as u8) {
            do_unlet(c"g:colors_name".as_ptr(), 13, true);
            restore_cterm_colors();
            let mut j = 0;
            while j < highlight_ga.ga_len {
                rs_highlight_clear(j);
                j += 1;
            }
            init_highlight(true, true);
            rs_highlight_changed();
            redraw_all_later(UPD_NOT_VALID);
            return;
        }
        name_end = skiptowhite(line);
        linep = skipwhite(name_end);
    }

    // Find the group name in the table. If it does not exist yet, add it.
    let name_len = name_end.offset_from(line) as usize;
    let id = syn_check_group(line, name_len);
    if id == 0 {
        return;
    }
    let idx = id - 1;

    // Return if "default" was used and the group already has settings.
    if dodefault && rs_hl_has_settings(idx, true) {
        return;
    }

    // Make a copy so we can check if any attribute actually changed.
    let item_before = (*hl_table_ptr(idx)).clone();
    let sg_name_u = (*hl_table_ptr(idx)).sg_name_u;
    let is_normal_group = strcmp(sg_name_u, c"NORMAL".as_ptr()) == 0;

    // Clear the highlighting for ":hi clear {group}" and ":hi clear".
    if doclear || (forceit && init) {
        rs_highlight_clear(idx);
        if !doclear {
            (*hl_table_ptr(idx)).sg_set = 0;
        }
    }

    let mut did_change = false;
    let mut error = false;

    if !doclear {
        while !ends_excmd(*linep as u8) {
            let key_start = linep;
            if *linep == b'=' as c_char {
                semsg(c"E415: Unexpected equal sign: %s".as_ptr(), key_start);
                error = true;
                break;
            }

            // Isolate the key and uppercase it.
            while *linep != 0 && !is_ascii_white(*linep as u8) && *linep != b'=' as c_char {
                linep = linep.add(1);
            }
            let key_len = linep.offset_from(key_start) as usize;
            if key_len > 63 {
                emsg(c"E423: Illegal argument".as_ptr());
                error = true;
                break;
            }
            let mut key_buf = [0u8; 64];
            vim_memcpy_up(key_buf.as_mut_ptr() as *mut c_char, key_start, key_len);
            key_buf[key_len] = 0;
            linep = skipwhite(linep);

            // Handle "NONE" keyword.
            if key_len == 4 && &key_buf[..4] == b"NONE" {
                if !init || (*hl_table_ptr(idx)).sg_set == 0 {
                    if !init {
                        (*hl_table_ptr(idx)).sg_set |= SG_CTERM + SG_GUI;
                    }
                    rs_highlight_clear(idx);
                }
                continue;
            }

            // Check for '='.
            if *linep != b'=' as c_char {
                semsg(c"E416: Missing equal sign: %s".as_ptr(), key_start);
                error = true;
                break;
            }
            linep = linep.add(1);

            // Isolate the argument.
            linep = skipwhite(linep);
            let arg_start;
            let arg_len;
            if *linep == b'\'' as c_char {
                linep = linep.add(1);
                arg_start = linep;
                let end = strchr(linep, b'\'' as c_int);
                if end.is_null() {
                    semsg(c"E475: Invalid argument: %s".as_ptr(), key_start);
                    error = true;
                    break;
                }
                arg_len = end.offset_from(arg_start) as usize;
            } else {
                arg_start = linep;
                linep = skiptowhite(linep);
                arg_len = linep.offset_from(arg_start) as usize;
            }

            if arg_len == 0 {
                semsg(c"E417: Missing argument: %s".as_ptr(), key_start);
                error = true;
                break;
            }
            if arg_len > 511 {
                emsg(c"E423: Illegal argument".as_ptr());
                error = true;
                break;
            }

            // Copy arg to NUL-terminated buffer.
            let mut arg_buf = [0u8; 512];
            memcpy(
                arg_buf.as_mut_ptr() as *mut c_void,
                arg_start as *const c_void,
                arg_len,
            );
            arg_buf[arg_len] = 0;
            let arg = arg_buf.as_ptr() as *const c_char;

            // Skip closing quote.
            if *linep == b'\'' as c_char {
                linep = linep.add(1);
            }

            // Dispatch on uppercased key.
            if (key_len == 4 && &key_buf[..4] == b"TERM")
                || (key_len == 5 && &key_buf[..5] == b"CTERM")
                || (key_len == 3 && &key_buf[..3] == b"GUI")
            {
                let (parsed_attr, parse_err) = parse_attr_string(arg);
                if parse_err {
                    semsg(c"E418: Illegal value: %s".as_ptr(), arg);
                    error = true;
                    break;
                }
                if key_buf[0] == b'C' {
                    // CTERM
                    if !init || ((*hl_table_ptr(idx)).sg_set & SG_CTERM == 0) {
                        if !init {
                            (*hl_table_ptr(idx)).sg_set |= SG_CTERM;
                        }
                        (*hl_table_ptr(idx)).sg_cterm = parsed_attr;
                        (*hl_table_ptr(idx)).sg_cterm_bold = false;
                    }
                } else if key_buf[0] == b'G' {
                    // GUI
                    if !init || ((*hl_table_ptr(idx)).sg_set & SG_GUI == 0) {
                        if !init {
                            (*hl_table_ptr(idx)).sg_set |= SG_GUI;
                        }
                        (*hl_table_ptr(idx)).sg_gui = parsed_attr;
                    }
                }
                // TERM is ignored.
            } else if key_len == 4 && &key_buf[..4] == b"FONT" {
                // Fonts ignored in non-GUI.
            } else if key_len == 7 && (&key_buf[..7] == b"CTERMFG" || &key_buf[..7] == b"CTERMBG") {
                if !init || ((*hl_table_ptr(idx)).sg_set & SG_CTERM == 0) {
                    if !init {
                        (*hl_table_ptr(idx)).sg_set |= SG_CTERM;
                    }

                    let is_fg = key_buf[5] == b'F';

                    // Reset bold when setting foreground color if it was set for a light color.
                    if is_fg && (*hl_table_ptr(idx)).sg_cterm_bold {
                        (*hl_table_ptr(idx)).sg_cterm &= !(HL_BOLD as c_int);
                        (*hl_table_ptr(idx)).sg_cterm_bold = false;
                    }

                    let color;
                    if is_ascii_digit(*arg as u8) {
                        color = atoi(arg);
                    } else if strcasecmp(arg, c"fg".as_ptr()) == 0 {
                        if cterm_normal_fg_color != 0 {
                            color = cterm_normal_fg_color - 1;
                        } else {
                            emsg(c"E419: FG color unknown".as_ptr());
                            error = true;
                            break;
                        }
                    } else if strcasecmp(arg, c"bg".as_ptr()) == 0 {
                        if cterm_normal_bg_color > 0 {
                            color = cterm_normal_bg_color - 1;
                        } else {
                            emsg(c"E420: BG color unknown".as_ptr());
                            error = true;
                            break;
                        }
                    } else {
                        // Look up in color_names[].
                        let first_upper = (*arg as u8).to_ascii_uppercase() as c_char;
                        let mut found_idx: c_int = -1;
                        let mut ci = COLOR_NAMES_28.len();
                        while ci > 0 {
                            ci -= 1;
                            let cname = COLOR_NAMES_28[ci].as_ptr() as *const c_char;
                            if first_upper == (*cname as u8).to_ascii_uppercase() as c_char
                                && strcasecmp(arg.add(1), cname.add(1)) == 0
                            {
                                found_idx = ci as c_int;
                                break;
                            }
                        }
                        if found_idx < 0 {
                            semsg(
                                c"E421: Color name or number not recognized: %s".as_ptr(),
                                key_start,
                            );
                            error = true;
                            break;
                        }
                        let result = rs_lookup_color(found_idx, is_fg);
                        color = result.color;
                        if result.bold == 1 {
                            (*hl_table_ptr(idx)).sg_cterm |= HL_BOLD as c_int;
                            (*hl_table_ptr(idx)).sg_cterm_bold = true;
                        } else if result.bold == 0 {
                            (*hl_table_ptr(idx)).sg_cterm &= !(HL_BOLD as c_int);
                            (*hl_table_ptr(idx)).sg_cterm_bold = false;
                        }
                    }

                    // Add one to avoid zero (zero means NONE).
                    if is_fg {
                        (*hl_table_ptr(idx)).sg_cterm_fg = color + 1;
                        if is_normal_group {
                            cterm_normal_fg_color = color + 1;
                        }
                    } else {
                        (*hl_table_ptr(idx)).sg_cterm_bg = color + 1;
                        if is_normal_group {
                            cterm_normal_bg_color = color + 1;
                            if !ui_rgb_attached() && color >= 0 {
                                let dark: c_int = if t_colors < 16 {
                                    if color == 0 || color == 4 {
                                        1
                                    } else {
                                        0
                                    }
                                } else if color < 16 {
                                    if color < 7 || color == 8 {
                                        1
                                    } else {
                                        0
                                    }
                                } else {
                                    -1
                                };
                                if dark != -1
                                    && (dark != 0) != (*p_bg == b'd' as c_char)
                                    && !option_was_set(K_OPT_BACKGROUND)
                                {
                                    let bg_ptr: *const c_char = if dark != 0 {
                                        c"dark".as_ptr()
                                    } else {
                                        c"light".as_ptr()
                                    };
                                    let opt_val = DhOptVal {
                                        type_: K_OPT_VAL_TYPE_STRING,
                                        data: DhOptValData {
                                            string: DhOptValString {
                                                data: bg_ptr as *mut c_char,
                                                size: if dark != 0 { 4 } else { 5 },
                                            },
                                        },
                                    };
                                    set_option_value_give_err(K_OPT_BACKGROUND, opt_val, 0);
                                    reset_option_was_set(K_OPT_BACKGROUND);
                                }
                            }
                        }
                    }
                }
            } else if key_len == 5
                && (&key_buf[..5] == b"GUIFG"
                    || &key_buf[..5] == b"GUIBG"
                    || &key_buf[..5] == b"GUISP")
            {
                let indexp = if key_buf[3] == b'F' {
                    &mut (*hl_table_ptr(idx)).sg_rgb_fg_idx
                } else if key_buf[3] == b'B' {
                    &mut (*hl_table_ptr(idx)).sg_rgb_bg_idx
                } else {
                    &mut (*hl_table_ptr(idx)).sg_rgb_sp_idx
                };

                if !init || ((*hl_table_ptr(idx)).sg_set & SG_GUI == 0) {
                    if !init {
                        (*hl_table_ptr(idx)).sg_set |= SG_GUI;
                    }

                    if key_buf[3] == b'F' {
                        let old_color = (*hl_table_ptr(idx)).sg_rgb_fg;
                        let old_idx = (*hl_table_ptr(idx)).sg_rgb_fg_idx;
                        if strcmp(arg, c"NONE".as_ptr()) != 0 {
                            (*hl_table_ptr(idx)).sg_rgb_fg = name_to_color(arg, indexp);
                        } else {
                            (*hl_table_ptr(idx)).sg_rgb_fg = -1;
                            (*hl_table_ptr(idx)).sg_rgb_fg_idx = K_COLOR_IDX_NONE;
                        }
                        did_change = (*hl_table_ptr(idx)).sg_rgb_fg != old_color
                            || (*hl_table_ptr(idx)).sg_rgb_fg_idx != old_idx;
                    } else if key_buf[3] == b'B' {
                        let old_color = (*hl_table_ptr(idx)).sg_rgb_bg;
                        let old_idx = (*hl_table_ptr(idx)).sg_rgb_bg_idx;
                        if strcmp(arg, c"NONE".as_ptr()) != 0 {
                            (*hl_table_ptr(idx)).sg_rgb_bg = name_to_color(arg, indexp);
                        } else {
                            (*hl_table_ptr(idx)).sg_rgb_bg = -1;
                            (*hl_table_ptr(idx)).sg_rgb_bg_idx = K_COLOR_IDX_NONE;
                        }
                        did_change = (*hl_table_ptr(idx)).sg_rgb_bg != old_color
                            || (*hl_table_ptr(idx)).sg_rgb_bg_idx != old_idx;
                    } else {
                        let old_color = (*hl_table_ptr(idx)).sg_rgb_sp;
                        let old_idx = (*hl_table_ptr(idx)).sg_rgb_sp_idx;
                        if strcmp(arg, c"NONE".as_ptr()) != 0 {
                            (*hl_table_ptr(idx)).sg_rgb_sp = name_to_color(arg, indexp);
                        } else {
                            (*hl_table_ptr(idx)).sg_rgb_sp = -1;
                            (*hl_table_ptr(idx)).sg_rgb_sp_idx = K_COLOR_IDX_NONE;
                        }
                        did_change = (*hl_table_ptr(idx)).sg_rgb_sp != old_color
                            || (*hl_table_ptr(idx)).sg_rgb_sp_idx != old_idx;
                    }
                }

                // Normal group fg/bg/sp update is outside the init check in C.
                if is_normal_group {
                    if key_buf[3] == b'F' {
                        normal_fg = (*hl_table_ptr(idx)).sg_rgb_fg;
                    } else if key_buf[3] == b'B' {
                        normal_bg = (*hl_table_ptr(idx)).sg_rgb_bg;
                    } else {
                        normal_sp = (*hl_table_ptr(idx)).sg_rgb_sp;
                    }
                }
            } else if (key_len == 5 && &key_buf[..5] == b"START")
                || (key_len == 4 && &key_buf[..4] == b"STOP")
            {
                // Ignored.
            } else if key_len == 5 && &key_buf[..5] == b"BLEND" {
                if strcmp(arg, c"NONE".as_ptr()) != 0 {
                    (*hl_table_ptr(idx)).sg_blend = strtol(arg, std::ptr::null_mut(), 10) as c_int;
                } else {
                    (*hl_table_ptr(idx)).sg_blend = -1;
                }
            } else {
                semsg(c"E423: Illegal argument: %s".as_ptr(), key_start);
                error = true;
                break;
            }

            (*hl_table_ptr(idx)).sg_cleared = false;

            // When highlighting has been given for a group, don't link it.
            if !init || ((*hl_table_ptr(idx)).sg_set & SG_LINK == 0) {
                (*hl_table_ptr(idx)).sg_link = 0;
            }

            linep = skipwhite(linep);
        }
    }

    let did_highlight_changed;

    if !error && is_normal_group {
        rs_highlight_attr_set_all();
        if !ui_has(K_UI_LINEGRID) && starting == 0 {
            ui_refresh();
        } else {
            ui_default_colors_set();
        }
        did_highlight_changed = true;
        redraw_all_later(UPD_NOT_VALID);
    } else {
        rs_set_hl_attr(idx);
        did_highlight_changed = false;
    }

    (*hl_table_ptr(idx)).sg_script_ctx = current_sctx;
    (*hl_table_ptr(idx)).sg_script_ctx.sc_lnum += nvim_get_sourcing_lnum();
    nlua_set_sctx(&mut (*hl_table_ptr(idx)).sg_script_ctx);

    if (did_change
        || memcmp(
            &item_before as *const _ as *const c_void,
            hl_table_ptr(idx) as *const c_void,
            std::mem::size_of::<crate::types::HlGroup>(),
        ) != 0)
        && !did_highlight_changed
    {
        if !updating_screen {
            redraw_all_later(UPD_NOT_VALID);
        }
        need_highlight_changed = true;
    }
}

#[inline]
fn is_ascii_white(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

#[inline]
fn is_ascii_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

// =============================================================================
// Phase 3: syn_name2id, combine_stl_hlt, highlight_changed
// =============================================================================

// HLF_* constants matching C enum hlf_T values in highlight_defs.h.
const HLF_NONE: c_int = 0;
const HLF_S: c_int = 19; // status lines
const HLF_SNC: c_int = 20; // status lines of not-current windows
const HLF_INACTIVE: c_int = 60; // NormalNC
const HLF_MSG: c_int = 63; // Message area
const HLF_COUNT: c_int = 76;

extern "C" {
    /// highlight_attr[] -- attribute per HLF_ index.
    static mut highlight_attr: [c_int; 76]; // [HLF_COUNT]
    /// highlight_attr_last[] -- previous values for change detection.
    static mut highlight_attr_last: [c_int; 76];
    /// User[1-9] highlight attributes.
    static mut highlight_user: [c_int; 9];
    /// User[1-9] highlights on top of StatusLineNC.
    static mut highlight_stlnc: [c_int; 9];
    /// Name strings for each HLF_ index.
    static hlf_names: [*const c_char; 76]; // [HLF_COUNT]
    /// bool: the command line must be cleared.
    static mut clear_cmdline: bool;
    /// The message area grid.
    static mut msg_grid: MsgGrid;

    fn strlen(s: *const c_char) -> usize;
    fn syn_attr2entry(attr: c_int) -> crate::HlAttrs;
    fn hl_get_ui_attr(ns_id: c_int, idx: c_int, final_id: c_int, optional: bool) -> c_int;
    fn ui_call_hl_group_set(name: nvim_api::NvimString, id: i64);
    fn syn_id2attr(hl_id: c_int) -> c_int;
    fn ga_grow(gap: *mut GArray, n: c_int);
    fn decor_provider_invalidate_hl();
    fn syn_ns_get_final_id(ns_id: *mut c_int, hl_idp: *mut c_int) -> bool;
}

/// Partial layout of ScreenGrid matching C struct grid_defs.h.
/// Only fields up to `blending` are modeled; the rest are opaque.
#[repr(C)]
struct MsgGrid {
    handle: c_int,
    _pad: c_int, // alignment padding before pointers
    chars: *mut u32,
    attrs: *mut i32,
    vcols: *mut c_int,
    line_offset: *mut usize,
    dirty_col: *mut c_int,
    rows: c_int,
    cols: c_int,
    valid: bool,
    throttled: bool,
    pub blending: bool,
    // remaining fields omitted (mouse_enabled, zindex, ...)
}

/// Replace C `int syn_name2id(const char *name)`.
///
/// If the name starts with '@', calls `syn_check_group` (which may create the
/// group); otherwise calls `syn_name2id_len`.
#[unsafe(export_name = "syn_name2id")]
pub unsafe extern "C" fn rs_syn_name2id(name: *const c_char) -> c_int {
    if *name as u8 == b'@' {
        syn_check_group(name, strlen(name))
    } else {
        syn_name2id_len(name, strlen(name))
    }
}

/// Apply the difference between User[1-9] and HLF_S to HLF_SNC.
///
/// Equivalent to the static C `combine_stl_hlt()`.
///
/// # Safety
/// `highlight_ga` and the hl_table must be valid.
unsafe fn rs_combine_stl_hlt(
    id: c_int,
    id_s: c_int,
    id_alt: c_int,
    hlcnt: c_int,
    i: c_int,
    hlf: c_int,
    table: *mut c_int,
) {
    let dst = hl_table_ptr(hlcnt + i);
    let src_s = hl_table_ptr(id_s - 1);
    let src_id = hl_table_ptr(id - 1);

    if id_alt == 0 {
        // Zero the destination slot and copy current HLF attrs.
        std::ptr::write_bytes(dst, 0, 1);
        (*dst).sg_cterm = highlight_attr[hlf as usize];
        (*dst).sg_gui = highlight_attr[hlf as usize];
    } else {
        let src_alt = hl_table_ptr(id_alt - 1);
        std::ptr::copy_nonoverlapping(src_alt, dst, 1);
    }

    (*dst).sg_link = 0;

    (*dst).sg_cterm ^= (*src_id).sg_cterm ^ (*src_s).sg_cterm;
    if (*src_id).sg_cterm_fg != (*src_s).sg_cterm_fg {
        (*dst).sg_cterm_fg = (*src_id).sg_cterm_fg;
    }
    if (*src_id).sg_cterm_bg != (*src_s).sg_cterm_bg {
        (*dst).sg_cterm_bg = (*src_id).sg_cterm_bg;
    }
    (*dst).sg_gui ^= (*src_id).sg_gui ^ (*src_s).sg_gui;
    if (*src_id).sg_rgb_fg != (*src_s).sg_rgb_fg {
        (*dst).sg_rgb_fg = (*src_id).sg_rgb_fg;
    }
    if (*src_id).sg_rgb_bg != (*src_s).sg_rgb_bg {
        (*dst).sg_rgb_bg = (*src_id).sg_rgb_bg;
    }
    if (*src_id).sg_rgb_sp != (*src_s).sg_rgb_sp {
        (*dst).sg_rgb_sp = (*src_id).sg_rgb_sp;
    }

    highlight_ga.ga_len = hlcnt + i + 1;
    rs_set_hl_attr(hlcnt + i);
    *table.add(i as usize) = syn_id2attr(hlcnt + i + 1);
}

/// Translate highlight groups into attributes in `highlight_attr[]` and set up
/// the User1..9 statusline highlights.
///
/// Replaces C `void highlight_changed(void)`.
#[unsafe(export_name = "highlight_changed")]
pub unsafe extern "C" fn rs_highlight_changed() {
    need_highlight_changed = false;

    // sentinel: no UI highlight active
    highlight_attr[HLF_NONE as usize] = 0;

    let mut id_s: c_int = -1;
    let mut id_snc: c_int = 0;

    // Translate builtin highlight groups into attributes for quick lookup.
    for hlf in 1..HLF_COUNT {
        let name = hlf_names[hlf as usize];
        let id = syn_check_group(name, strlen(name));
        if id == 0 {
            std::process::abort();
        }
        let mut ns_id: c_int = -1;
        let mut final_id = id;
        syn_ns_get_final_id(&mut ns_id, &mut final_id);
        if hlf == HLF_SNC {
            id_snc = final_id;
        } else if hlf == HLF_S {
            id_s = final_id;
        }

        let attr = hl_get_ui_attr(ns_id, hlf, final_id, hlf == HLF_INACTIVE);
        highlight_attr[hlf as usize] = attr;

        if attr != highlight_attr_last[hlf as usize] {
            if hlf == HLF_MSG {
                clear_cmdline = true;
                let attrs = syn_attr2entry(attr);
                msg_grid.blending = attrs.hl_blend > -1;
            }
            ui_call_hl_group_set(
                nvim_api::NvimString {
                    data: hlf_names[hlf as usize] as *mut c_char,
                    size: strlen(hlf_names[hlf as usize]),
                },
                attr as i64,
            );
            highlight_attr_last[hlf as usize] = attr;
        }
    }

    // Setup User[1-9] highlights.
    // Temporarily use 10 extra hl entries (9 for User combined with SNC, 1 for S default).
    ga_grow(&raw mut highlight_ga, 10);
    let hlcnt = highlight_ga.ga_len;

    if id_s == -1 {
        // Make sure id_s is always valid to simplify code below.
        std::ptr::write_bytes(hl_table_ptr(hlcnt + 9), 0, 1);
        id_s = hlcnt + 10;
    }

    for i in 0..9i32 {
        // Build "User<n>" name in a stack buffer.
        let n = (i + 1) as u8;
        let userhl: [u8; 8] = [b'U', b's', b'e', b'r', b'0' + n, 0, 0, 0];
        let id = syn_name2id_len(userhl.as_ptr() as *const c_char, 5);
        if id == 0 {
            highlight_user[i as usize] = 0;
            highlight_stlnc[i as usize] = 0;
        } else {
            highlight_user[i as usize] = syn_id2attr(id);
            rs_combine_stl_hlt(
                id,
                id_s,
                id_snc,
                hlcnt,
                i,
                HLF_SNC,
                (&raw mut highlight_stlnc).cast::<c_int>(),
            );
        }
    }

    highlight_ga.ga_len = hlcnt;
    decor_provider_invalidate_hl();
}

// =============================================================================
// Phase 4: free_highlight
// =============================================================================

/// MapHash layout (matches C struct in map_defs.h).
/// sizeof(MapHash) == 6*sizeof(u32) + sizeof(*void) = 32 bytes on 64-bit.
#[repr(C)]
struct MapHash {
    n_buckets: u32,
    size: u32,
    n_occupied: u32,
    upper_bound: u32,
    n_keys: u32,
    keys_capacity: u32,
    hash: *mut u32,
}

/// Set(cstr_t) layout (matches C struct in map_defs.h).
#[repr(C)]
struct SetCstrT {
    h: MapHash,
    keys: *mut *const c_char,
}

/// Map(cstr_t, int) layout (matches C struct in map_defs.h).
#[repr(C)]
struct MapCstrTInt {
    set: SetCstrT,
    values: *mut c_int,
}

/// Arena layout (matches C struct in memory_defs.h).
#[repr(C)]
struct CArena {
    cur_blk: *mut c_char,
    pos: usize,
    size: usize,
}

extern "C" {
    /// The highlight group name map (Map(cstr_t, int) in C).
    static mut highlight_unames: MapCstrTInt;
    /// The highlight group arena allocator.
    static mut highlight_arena: CArena;

    fn ga_clear(gap: *mut GArray);
    fn arena_finish(arena: *mut CArena) -> *mut c_void; // returns ArenaMem
    fn arena_mem_free(mem: *mut c_void);
}

/// Free all highlight group data (called on exit via EXITFREE).
///
/// Replaces C `void free_highlight(void)`.
#[unsafe(export_name = "free_highlight")]
pub unsafe extern "C" fn rs_free_highlight() {
    ga_clear(&raw mut highlight_ga);

    // Equivalent to map_destroy(cstr_t, &highlight_unames):
    //   set_destroy(cstr_t, &set)  =>  xfree(keys), xfree(h.hash), zero set
    //   XFREE_CLEAR(values)
    xfree(highlight_unames.set.keys as *mut c_void);
    xfree(highlight_unames.set.h.hash as *mut c_void);
    highlight_unames.set.h = MapHash {
        n_buckets: 0,
        size: 0,
        n_occupied: 0,
        upper_bound: 0,
        n_keys: 0,
        keys_capacity: 0,
        hash: std::ptr::null_mut(),
    };
    highlight_unames.set.keys = std::ptr::null_mut();
    xfree(highlight_unames.values as *mut c_void);
    highlight_unames.values = std::ptr::null_mut();

    arena_mem_free(arena_finish(&raw mut highlight_arena));
}

// =============================================================================
// Phase 1: syn_add_group migrated to Rust from highlight_group.c
// =============================================================================

/// Add new highlight group and return its 1-based ID.
///
/// Exported as `c_syn_add_group` so the nvim-highlight crate's
/// rs_syn_check_group can call it without any changes on its end.
///
/// # Safety
/// `name` must be valid for at least `len` bytes.
#[export_name = "c_syn_add_group"]
pub unsafe extern "C" fn rs_syn_add_group(name: *const c_char, len: usize) -> c_int {
    // Validate every character in the name.
    for i in 0..len {
        let c = (*name.add(i) as u8) as c_int;
        if nvim_hlg_vim_isprintc(c) == 0 {
            nvim_hlg_emsg(c"E669: Unprintable character in group name".as_ptr());
            return 0;
        }
        let b = c as u8;
        if !b.is_ascii_alphanumeric() && b != b'_' && b != b'.' && b != b'@' && b != b'-' {
            nvim_hlg_msg_source();
            nvim_hlg_emsg(c"E5248: Invalid character in group name".as_ptr());
            return 0;
        }
    }

    // Look up the scoped parent for @foo.bar style names.
    let scoped_parent: c_int = if len > 1 && *name == b'@' as c_char {
        let delim = nvim_hlg_xmemrchr(name as *const c_void, b'.' as c_int, len) as *const c_char;
        if !delim.is_null() {
            let parent_len = (delim as usize) - (name as usize);
            syn_check_group(name, parent_len)
        } else {
            0
        }
    } else {
        0
    };

    // Allocate a new entry in highlight_ga (already zeroed by nvim_hlg_alloc_entry).
    let mut id: c_int = 0;
    let hlgp = nvim_hlg_alloc_entry(&raw mut id) as *mut crate::types::HlGroup;
    if hlgp.is_null() {
        nvim_hlg_emsg(c"E849: Too many highlight and syntax groups".as_ptr());
        return 0;
    }

    (*hlgp).sg_name = nvim_hlg_arena_memdupz(name, len);
    (*hlgp).sg_rgb_bg = -1;
    (*hlgp).sg_rgb_fg = -1;
    (*hlgp).sg_rgb_sp = -1;
    (*hlgp).sg_rgb_bg_idx = crate::types::ColorIdx::None as c_int;
    (*hlgp).sg_rgb_fg_idx = crate::types::ColorIdx::None as c_int;
    (*hlgp).sg_rgb_sp_idx = crate::types::ColorIdx::None as c_int;
    (*hlgp).sg_blend = -1;
    (*hlgp).sg_name_u = nvim_hlg_arena_memdupz(name, len);
    (*hlgp).sg_parent = scoped_parent;
    (*hlgp).sg_cleared = true;
    nvim_hlg_vim_strup((*hlgp).sg_name_u);

    nvim_hlg_unames_put((*hlgp).sg_name_u, id);

    id
}

// =============================================================================
// Phase 2: set_hl_group migrated to Rust from highlight_group.c
// =============================================================================

/// Set highlight group attributes from an API call.
///
/// This directly provides the C symbol `set_hl_group`, replacing the C implementation.
///
/// # Safety
/// `dict` must be a valid `Dict(highlight)*` pointer.
#[export_name = "set_hl_group"]
pub unsafe extern "C" fn rs_set_hl_group(
    id: c_int,
    attrs: crate::HlAttrs,
    dict: *mut c_void,
    link_id: c_int,
) {
    use nvim_highlight::hl_attr_flags::{HL_BOLD, HL_DEFAULT};

    // SG_LINK flag value (matches C enum in highlight_group.c)
    const SG_LINK: c_int = 8;

    let idx = id - 1;
    let is_default = (attrs.rgb_ae_attr & HL_DEFAULT) != 0;

    let info = nvim_hlg_extract_set_info(dict);

    // Return if "default" was used and the group already has settings.
    if is_default && rs_hl_has_settings(idx, true) && !info.force {
        return;
    }

    let sg = &mut *hl_table_ptr(idx);
    sg.sg_cleared = false;

    if link_id > 0 {
        sg.sg_link = link_id;
        sg.sg_script_ctx = nvim_hlg_get_current_sctx();
        sg.sg_script_ctx.sc_lnum += nvim_hlg_get_sourcing_lnum();
        nvim_hlg_nlua_set_sctx(&raw mut sg.sg_script_ctx);
        sg.sg_set |= SG_LINK;
        if is_default {
            sg.sg_deflink = link_id;
            sg.sg_deflink_sctx = nvim_hlg_get_current_sctx();
            sg.sg_deflink_sctx.sc_lnum += nvim_hlg_get_sourcing_lnum();
            nvim_hlg_nlua_set_sctx(&raw mut sg.sg_deflink_sctx);
        }
    } else {
        sg.sg_link = 0;
    }

    sg.sg_gui = (attrs.rgb_ae_attr & !HL_DEFAULT) as c_int;

    sg.sg_rgb_fg = attrs.rgb_fg_color;
    sg.sg_rgb_bg = attrs.rgb_bg_color;
    sg.sg_rgb_sp = attrs.rgb_sp_color;

    // Resolve color index for each channel.
    let channels: [(*mut c_int, crate::types::RgbValue, *const c_char); 3] = [
        (&raw mut sg.sg_rgb_fg_idx, sg.sg_rgb_fg, info.fg_name),
        (&raw mut sg.sg_rgb_bg_idx, sg.sg_rgb_bg, info.bg_name),
        (&raw mut sg.sg_rgb_sp_idx, sg.sg_rgb_sp, info.sp_name),
    ];
    for (dest, val, name) in channels {
        if val < 0 {
            *dest = crate::types::ColorIdx::None as c_int;
        } else if !name.is_null() {
            nvim_hlg_name_to_color(name, dest);
        } else {
            *dest = crate::types::ColorIdx::Hex as c_int;
        }
    }

    sg.sg_cterm = (attrs.cterm_ae_attr & !HL_DEFAULT) as c_int;
    sg.sg_cterm_bg = attrs.cterm_bg_color as c_int;
    sg.sg_cterm_fg = attrs.cterm_fg_color as c_int;
    sg.sg_cterm_bold = (sg.sg_cterm & HL_BOLD as c_int) != 0;
    sg.sg_blend = attrs.hl_blend;

    sg.sg_script_ctx = nvim_hlg_get_current_sctx();
    sg.sg_script_ctx.sc_lnum += nvim_hlg_get_sourcing_lnum();
    nvim_hlg_nlua_set_sctx(&raw mut sg.sg_script_ctx);

    sg.sg_attr = hl_get_syn_attr(0, id, attrs);

    // Normal group is special.
    let is_normal = {
        let s = std::ffi::CStr::from_ptr(sg.sg_name_u);
        s.to_bytes() == b"NORMAL"
    };

    if is_normal {
        cterm_normal_fg_color = sg.sg_cterm_fg;
        cterm_normal_bg_color = sg.sg_cterm_bg;
        let did_changed =
            normal_bg != sg.sg_rgb_bg || normal_fg != sg.sg_rgb_fg || normal_sp != sg.sg_rgb_sp;
        normal_fg = sg.sg_rgb_fg;
        normal_bg = sg.sg_rgb_bg;
        normal_sp = sg.sg_rgb_sp;
        if did_changed {
            nvim_hlg_highlight_attr_set_all();
        }
        nvim_hlg_ui_default_colors_set();
    } else if cursor_mode_uses_syn_id(id) {
        ui_mode_info_set();
    }

    if !nvim_hlg_updating_screen() {
        nvim_hlg_redraw_all_later();
    }
    nvim_hlg_set_need_highlight_changed();
}
