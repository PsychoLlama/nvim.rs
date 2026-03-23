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
    fn nvim_get_msg_col() -> c_int;

    /// Set the current message column.
    fn nvim_set_msg_col(col: c_int);

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
        nvim_set_msg_col(col);
        name_col = col;
        endcol = 15;
    } else if (ui_has(K_UI_MESSAGES) || msg_silent != 0) && !force_newline {
        msg_putchar(b' ' as c_int);
        adjust = false;
    } else if nvim_get_msg_col() + outlen + 1 >= Columns || force_newline {
        msg_putchar(b'\n' as c_int);
        if unsafe { got_int } {
            return true;
        }
    } else if nvim_get_msg_col() >= endcol {
        // wrap around is like starting a new line
        newline = false;
    }

    if adjust {
        if nvim_get_msg_col() >= endcol {
            // output at least one space
            endcol = nvim_get_msg_col() + 1;
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
