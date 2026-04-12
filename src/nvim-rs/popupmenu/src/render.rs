//! Popup menu text rendering utilities.
//!
//! This module provides helper functions for rendering popup menu text
//! with fuzzy match highlighting and proper attribute handling.

use std::ffi::{c_char, c_int, c_uint, c_ulong};

use crate::PUM_STATE;

/// Highlight group IDs used in popup menu rendering.
pub mod hlf {
    use std::ffi::c_int;

    /// Popup menu normal item (unselected).
    pub const HLF_PNI: c_int = 41;
    /// Popup menu selected item.
    pub const HLF_PSI: c_int = 42;
    /// Popup menu normal kind column.
    pub const HLF_PNK: c_int = 45;
    /// Popup menu selected kind column.
    pub const HLF_PSK: c_int = 46;
    /// Popup menu normal extra column.
    pub const HLF_PNX: c_int = 47;
    /// Popup menu selected extra column.
    pub const HLF_PSX: c_int = 48;
    /// Popup menu match highlight (normal).
    pub const HLF_PMNI: c_int = 43;
    /// Popup menu match highlight (selected).
    pub const HLF_PMSI: c_int = 44;
    /// Popup menu scrollbar.
    pub const HLF_PSB: c_int = 49;
    /// Popup menu scrollbar thumb.
    pub const HLF_PST: c_int = 50;
    /// Popup menu border.
    pub const HLF_PBR: c_int = 51;
}

// External C functions for rendering
extern "C" {
    /// Get highlight attribute for a window and highlight group.
    fn nvim_win_hl_attr(wp: *mut crate::display::WinHandle, hlf: c_int) -> c_int;
    /// Combine two highlight attributes.
    fn hl_combine_attr(char_attr: c_int, comb_attr: c_int) -> c_int;
    /// Get cell width of a UTF-8 character.
    fn utf_ptr2cells(p: *const u8) -> c_int;
    /// Get byte length of a UTF-8 character sequence.
    fn utfc_ptr2len(p: *const c_char) -> c_int;
}

// C globals for render.
extern "C" {
    /// C global: `curwin` (current window pointer).
    static mut curwin: *mut crate::display::WinHandle;
}

/// Result of highlight attribute calculation for a cell.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumCellAttr {
    /// Highlight attribute for this cell.
    pub attr: c_int,
    /// Width of this cell in columns.
    pub width: c_int,
}

/// Compute the base highlight attribute for a popup menu item.
///
/// # Arguments
/// * `is_selected` - Whether the item is selected (non-zero = selected)
/// * `column_type` - Column type (0 = abbr, 1 = kind, 2 = extra)
///
/// Returns the combined highlight attribute for the cell.
///
/// # Safety
/// Calls C accessor functions for highlight attributes.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_base_attr(is_selected: c_int, column_type: c_int) -> c_int {
    // Get the highlight group for this column
    let is_sel = is_selected != 0;
    let (selected_hlf, normal_hlf) = match column_type {
        1 => (hlf::HLF_PSK, hlf::HLF_PNK),
        2 => (hlf::HLF_PSX, hlf::HLF_PNX),
        _ => (hlf::HLF_PSI, hlf::HLF_PNI),
    };
    let hlf = if is_sel { selected_hlf } else { normal_hlf };

    let attr = nvim_win_hl_attr(curwin, hlf);
    // Combine with base PNI attribute for consistent styling
    hl_combine_attr(nvim_win_hl_attr(curwin, hlf::HLF_PNI), attr)
}

/// Compute the highlight attribute for a matched character.
///
/// # Arguments
/// * `base_attr` - Base attribute for the item
/// * `is_selected` - Whether the item is selected (non-zero = selected)
///
/// Returns the combined match highlight attribute.
///
/// # Safety
/// Calls C accessor functions for highlight attributes.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_match_attr(base_attr: c_int, is_selected: c_int) -> c_int {
    let match_hlf = if is_selected != 0 {
        hlf::HLF_PMSI
    } else {
        hlf::HLF_PMNI
    };

    let match_attr = nvim_win_hl_attr(curwin, match_hlf);
    let combined = hl_combine_attr(nvim_win_hl_attr(curwin, hlf::HLF_PMNI), match_attr);
    let combined = hl_combine_attr(base_attr, combined);
    hl_combine_attr(nvim_win_hl_attr(curwin, hlf::HLF_PNI), combined)
}

/// Combine a highlight attribute with a user-defined attribute.
///
/// # Arguments
/// * `attr` - Base attribute
/// * `user_attr` - User-defined attribute (0 if none)
///
/// Returns the combined attribute.
///
/// # Safety
/// Calls C combine function.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_combine_user_attr(attr: c_int, user_attr: c_int) -> c_int {
    if user_attr > 0 {
        hl_combine_attr(attr, user_attr)
    } else {
        attr
    }
}

/// Check if match highlighting is needed.
///
/// Match highlighting is only needed when the match highlight attributes
/// differ from the normal/selected attributes.
///
/// # Arguments
/// * `hlf` - Highlight group (`HLF_PSI` or `HLF_PNI`)
///
/// Returns 1 if match highlighting is needed, 0 otherwise.
///
/// # Safety
/// Calls C accessor functions for highlight attributes.
#[no_mangle]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn rs_pum_needs_match_highlight(hlf: c_int) -> c_int {
    // Only apply match highlight for PSI (selected item) or PNI (normal item)
    if hlf != hlf::HLF_PSI && hlf != hlf::HLF_PNI {
        return 0;
    }

    // Check if match highlight attributes differ from normal
    let match_selected_attr = nvim_win_hl_attr(curwin, hlf::HLF_PMSI);
    let selected_attr = nvim_win_hl_attr(curwin, hlf::HLF_PSI);
    let match_normal_attr = nvim_win_hl_attr(curwin, hlf::HLF_PMNI);
    let normal_attr = nvim_win_hl_attr(curwin, hlf::HLF_PNI);

    c_int::from(match_selected_attr != selected_attr || match_normal_attr != normal_attr)
}

/// Get the scrollbar attribute.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_scrollbar_attr() -> c_int {
    nvim_win_hl_attr(curwin, hlf::HLF_PSB)
}

/// Get the scrollbar thumb attribute.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_thumb_attr() -> c_int {
    nvim_win_hl_attr(curwin, hlf::HLF_PST)
}

/// Get the truncation attribute (for truncation indicator).
///
/// # Arguments
/// * `is_selected` - Whether the item is selected (non-zero = selected)
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_trunc_attr(is_selected: c_int) -> c_int {
    if is_selected != 0 {
        nvim_win_hl_attr(curwin, hlf::HLF_PSI)
    } else {
        nvim_win_hl_attr(curwin, hlf::HLF_PNI)
    }
}

/// Result of text width calculation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PumTextWidth {
    /// Total width in cells.
    pub cells: c_int,
    /// Number of characters.
    pub chars: c_int,
}

/// Calculate display width of text.
///
/// # Arguments
/// * `text` - Pointer to UTF-8 text
/// * `max_bytes` - Maximum bytes to examine (-1 for entire string)
///
/// Returns the display width in cells and character count.
///
/// # Safety
/// The caller must ensure `text` is a valid UTF-8 string pointer.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_pum_text_width(text: *const u8, max_bytes: c_int) -> PumTextWidth {
    if text.is_null() {
        return PumTextWidth { cells: 0, chars: 0 };
    }

    let mut ptr = text;
    let mut cells = 0;
    let mut chars = 0;
    let mut bytes_read = 0;

    while *ptr != 0 && (max_bytes < 0 || bytes_read < max_bytes) {
        let char_cells = utf_ptr2cells(ptr);
        let char_len = utfc_ptr2len(ptr.cast());

        cells += char_cells;
        chars += 1;
        bytes_read += char_len;
        // char_len is always positive from utfc_ptr2len
        ptr = ptr.add(char_len as usize);
    }

    PumTextWidth { cells, chars }
}

/// Check if a character position should be highlighted as a match.
///
/// # Arguments
/// * `char_pos` - Character position (0-indexed)
/// * `match_positions` - Array of matching character positions
/// * `match_count` - Number of positions in the array
///
/// Returns 1 if this position should be highlighted, 0 otherwise.
///
/// # Safety
/// The caller must ensure `match_positions` points to an array with at least
/// `match_count` elements.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_pum_is_match_pos(
    char_pos: u32,
    match_positions: *const u32,
    match_count: c_int,
) -> c_int {
    if match_positions.is_null() || match_count <= 0 {
        return 0;
    }

    // match_count is checked to be positive above
    for i in 0..match_count as usize {
        if *match_positions.add(i) == char_pos {
            return 1;
        }
    }
    0
}

/// Calculate the number of cells needed for text with truncation.
///
/// # Arguments
/// * `text_cells` - Total cells needed for text
/// * `available_cells` - Available cells for display
///
/// Returns number of cells to display (may be less than `text_cells` if truncated).
#[no_mangle]
pub const extern "C" fn rs_pum_truncate_width(text_cells: c_int, available_cells: c_int) -> c_int {
    if text_cells <= available_cells {
        text_cells
    } else if available_cells > 1 {
        available_cells - 1 // Leave room for truncation indicator
    } else {
        0
    }
}

/// Check if text needs truncation indicator.
///
/// Returns 1 if truncation indicator should be shown, 0 otherwise.
#[no_mangle]
pub const extern "C" fn rs_pum_needs_truncation(
    text_cells: c_int,
    available_cells: c_int,
) -> c_int {
    (text_cells > available_cells && available_cells > 0) as c_int
}

/// Mode flag for command line.
const MODE_CMDLINE: c_int = 0x08;
/// `kOptCotFlagFuzzy` constant for fuzzy completion detection.
const K_OPT_COT_FLAG_FUZZY: c_uint = 0x80;

// C globals for leader/fuzzy detection.
extern "C" {
    static mut State: c_int;
}

// C functions for leader/fuzzy detection (direct, replacing wrapper accessors).
extern "C" {
    /// Get the cmdline completion pattern (for cmdline mode leader).
    fn cmdline_compl_pattern() -> *mut c_char;
    /// Check if cmdline completion is fuzzy.
    fn cmdline_compl_is_fuzzy() -> c_int;
    /// Get the insert-mode completion leader string.
    fn rs_ins_compl_leader() -> *const c_char;
    /// Get the current 'cot' option flags.
    fn rs_get_cot_flags() -> c_uint;
}

/// `garray_T` C struct layout for fuzzy match positions.
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct GArray {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut std::ffi::c_void,
}

// C accessor functions.
extern "C" {
    /// Fuzzy match a string against a pattern, filling a `GArray` with match positions.
    /// Returns `NULL` on no match. Caller must free with `ga_clear` + `xfree`.
    fn fuzzy_match_str_with_pos(text: *const c_char, leader: *const c_char) -> *mut GArray;
    /// Clear a `GArray` (frees `ga_data` but not the `GArray` itself).
    fn ga_clear(ga: *mut GArray);
    /// Case-insensitive multibyte string comparison.
    fn mb_strnicmp(s1: *const c_char, s2: *const c_char, len: c_ulong) -> c_int;
    /// Allocate memory via xmalloc.
    fn xmalloc(size: usize) -> *mut std::ffi::c_void;
    /// Free memory.
    fn xfree(ptr: *mut std::ffi::c_void);
    /// Get display width of string in cells.
    fn vim_strsize(text: *const c_char) -> c_int;
    /// Display text on popup grid at column with attribute.
    fn grid_line_puts(col: c_int, text: *const c_char, textlen: c_int, attr: c_int) -> c_int;
    /// Free memory allocated by C.
    fn nvim_xfree(ptr: *mut u8);
    /// Get C strlen.
    fn strlen(s: *const c_char) -> c_ulong;
}

/// Get fuzzy match positions for `text` against `leader`.
///
/// Returns a pointer to an xmalloc'd `u32` array of character positions, and
/// sets `*out_len` to the number of positions. Returns NULL on no match.
/// Caller must free with `xfree`.
///
/// # Safety
/// `text` and `leader` must be valid NUL-terminated C strings.
unsafe fn fuzzy_match_positions(
    text: *const c_char,
    leader: *const c_char,
    out_len: &mut c_int,
) -> *mut u32 {
    *out_len = 0;
    let ga = fuzzy_match_str_with_pos(text, leader);
    if ga.is_null() {
        return std::ptr::null_mut();
    }
    let len = (*ga).ga_len;
    if len <= 0 {
        ga_clear(ga);
        xfree(ga.cast());
        return std::ptr::null_mut();
    }
    #[allow(clippy::cast_sign_loss)]
    let len_usize = len as usize;
    let positions = xmalloc(std::mem::size_of::<u32>() * len_usize).cast::<u32>();
    std::ptr::copy_nonoverlapping((*ga).ga_data.cast::<u32>(), positions, len_usize);
    ga_clear(ga);
    xfree(ga.cast());
    *out_len = len;
    positions
}

/// Compute text attributes for a popup menu item.
///
/// Returns a pointer to an array of per-cell attributes (one per display cell),
/// or null if match highlighting is not needed (all cells have the same attribute).
///
/// The caller must free the returned array with `xfree()`.
///
/// # Safety
/// `text` must be a valid NUL-terminated C string.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_pum_compute_text_attrs(
    text: *mut c_char,
    hlf_id: c_int,
    user_hlattr: c_int,
) -> *mut c_int {
    // Early exit: empty text or not a match-highlight-eligible group
    if *text == 0
        || (hlf_id != hlf::HLF_PSI && hlf_id != hlf::HLF_PNI)
        || (nvim_win_hl_attr(curwin, hlf::HLF_PMSI) == nvim_win_hl_attr(curwin, hlf::HLF_PSI)
            && nvim_win_hl_attr(curwin, hlf::HLF_PMNI) == nvim_win_hl_attr(curwin, hlf::HLF_PNI))
    {
        return std::ptr::null_mut();
    }

    // Inline nvim_pum_get_compl_leader: return cmdline pattern or ins-mode leader
    let leader = if (State & MODE_CMDLINE) != 0 {
        cmdline_compl_pattern()
    } else {
        rs_ins_compl_leader().cast_mut()
    };
    if leader.is_null() || *leader == 0 {
        return std::ptr::null_mut();
    }

    let text_cells = vim_strsize(text);
    #[allow(clippy::cast_sign_loss)]
    let attrs = xmalloc(std::mem::size_of::<c_int>() * text_cells as usize).cast::<c_int>();
    // Inline nvim_pum_compl_is_fuzzy: check cmdline or insert-mode fuzzy flag
    let in_fuzzy = if (State & MODE_CMDLINE) != 0 {
        cmdline_compl_is_fuzzy() != 0
    } else {
        (rs_get_cot_flags() & K_OPT_COT_FLAG_FUZZY) != 0
    };
    let leader_len = strlen(leader) as c_ulong;
    let is_select = hlf_id == hlf::HLF_PSI;

    // Get fuzzy match positions if in fuzzy mode
    let mut fuzzy_len: c_int = 0;
    let fuzzy_positions = if in_fuzzy {
        let positions = fuzzy_match_positions(text, leader, &mut fuzzy_len);
        if positions.is_null() {
            nvim_xfree(attrs.cast());
            return std::ptr::null_mut();
        }
        positions
    } else {
        std::ptr::null_mut()
    };

    let mut ptr = text.cast::<u8>();
    let mut cell_idx: c_int = 0;
    let mut char_pos: u32 = 0;
    let mut matched_len: c_int = -1;

    while *ptr != 0 {
        let mut new_attr = nvim_win_hl_attr(curwin, hlf_id);

        if fuzzy_positions.is_null() {
            // Prefix matching
            #[allow(clippy::cast_possible_truncation)]
            if matched_len < 0 && mb_strnicmp(ptr.cast(), leader, leader_len) == 0 {
                matched_len = leader_len as c_int;
            }
            if matched_len > 0 {
                let match_hlf = if is_select {
                    hlf::HLF_PMSI
                } else {
                    hlf::HLF_PMNI
                };
                new_attr = nvim_win_hl_attr(curwin, match_hlf);
                new_attr = hl_combine_attr(nvim_win_hl_attr(curwin, hlf::HLF_PMNI), new_attr);
                new_attr = hl_combine_attr(nvim_win_hl_attr(curwin, hlf_id), new_attr);
                matched_len -= 1;
            }
        } else {
            // Fuzzy matching: check if this character position is a match
            for i in 0..fuzzy_len as usize {
                if char_pos == *fuzzy_positions.add(i) {
                    let match_hlf = if is_select {
                        hlf::HLF_PMSI
                    } else {
                        hlf::HLF_PMNI
                    };
                    new_attr = nvim_win_hl_attr(curwin, match_hlf);
                    new_attr = hl_combine_attr(nvim_win_hl_attr(curwin, hlf::HLF_PMNI), new_attr);
                    new_attr = hl_combine_attr(nvim_win_hl_attr(curwin, hlf_id), new_attr);
                    break;
                }
            }
        }

        new_attr = hl_combine_attr(nvim_win_hl_attr(curwin, hlf::HLF_PNI), new_attr);

        if user_hlattr > 0 {
            new_attr = hl_combine_attr(new_attr, user_hlattr);
        }

        let char_cells = utf_ptr2cells(ptr);
        for i in 0..char_cells {
            *attrs.offset((cell_idx + i) as isize) = new_attr;
        }
        cell_idx += char_cells;

        let char_len = utfc_ptr2len(ptr.cast());
        ptr = ptr.add(char_len as usize);
        char_pos += 1;
    }

    if !fuzzy_positions.is_null() {
        xfree(fuzzy_positions.cast());
    }
    attrs
}

/// Display text on the popup menu grid with per-cell attributes.
///
/// Renders text character by character, looking up the per-cell attribute
/// from the `attrs` array. Handles right-to-left mode by reversing the
/// attribute index.
///
/// # Safety
/// `text` must be a valid NUL-terminated C string. `attrs` must point to
/// an array with at least `cells` elements.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_pum_grid_puts_with_attrs(
    col: c_int,
    cells: c_int,
    text: *const c_char,
    textlen: c_int,
    attrs: *const c_int,
) {
    let col_start = col;
    let mut col = col;
    let mut ptr = text.cast::<u8>();
    let pum_rl = PUM_STATE.rl != 0;

    while *ptr != 0 && (textlen < 0 || (ptr as isize - text as isize) < textlen as isize) {
        let char_len = utfc_ptr2len(ptr.cast());
        let attr_idx = if pum_rl {
            col_start + cells - col - 1
        } else {
            col - col_start
        };
        let attr = *attrs.offset(attr_idx as isize);
        grid_line_puts(col, ptr.cast(), char_len, attr);
        col += utf_ptr2cells(ptr);
        ptr = ptr.add(char_len as usize);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_width_fits() {
        assert_eq!(rs_pum_truncate_width(10, 20), 10);
    }

    #[test]
    fn test_truncate_width_truncated() {
        assert_eq!(rs_pum_truncate_width(20, 10), 9);
    }

    #[test]
    fn test_truncate_width_minimal() {
        assert_eq!(rs_pum_truncate_width(10, 1), 0);
    }

    #[test]
    fn test_needs_truncation() {
        assert_eq!(rs_pum_needs_truncation(10, 20), 0);
        assert_eq!(rs_pum_needs_truncation(20, 10), 1);
        assert_eq!(rs_pum_needs_truncation(10, 0), 0);
    }
}
