//! Popup menu item access and manipulation.
//!
//! This module provides functions for accessing popup menu item data
//! and combining highlight attributes.

use std::ffi::{c_char, c_int, c_uint};

use crate::PUM_STATE;

/// Completion item type for abbreviation/text.
pub const CPT_ABBR: c_int = 0;
/// Completion item type for kind.
pub const CPT_KIND: c_int = 1;
/// Completion item type for menu/extra text.
pub const CPT_MENU: c_int = 2;

/// Popup menu item, matching C `pumitem_T` layout (sizeof=48).
///
/// Layout (verified via C static assertions):
/// - `pum_text` at offset 0 (8 bytes)
/// - `pum_kind` at offset 8 (8 bytes)
/// - `pum_extra` at offset 16 (8 bytes)
/// - `pum_info` at offset 24 (8 bytes)
/// - `pum_cpt_source_idx` at offset 32 (4 bytes)
/// - `pum_user_abbr_hlattr` at offset 36 (4 bytes)
/// - `pum_user_kind_hlattr` at offset 40 (4 bytes)
/// - 4 bytes padding
#[repr(C)]
pub struct PumItemArray {
    /// Main menu text (abbr).
    pub pum_text: *mut c_char,
    /// Extra kind text (may be truncated).
    pub pum_kind: *mut c_char,
    /// Extra menu text (may be truncated).
    pub pum_extra: *mut c_char,
    /// Extra info text.
    pub pum_info: *mut c_char,
    /// Index of completion source in 'cpt'.
    pub pum_cpt_source_idx: c_int,
    /// Highlight attribute for abbr.
    pub pum_user_abbr_hlattr: c_int,
    /// Highlight attribute for kind.
    pub pum_user_kind_hlattr: c_int,
    /// Padding to match C struct size.
    _pad: c_int,
}

// FFI declarations for item operations
extern "C" {
    /// Combine highlight attributes (`hl_combine_attr`).
    fn hl_combine_attr(char_attr: c_int, comb_attr: c_int) -> c_int;
    /// Check if two strings are equal.
    fn strequal(s1: *const c_char, s2: *const c_char) -> bool;
    /// Compute the display width of a string.
    fn vim_strsize(s: *const c_char) -> c_int;
}

// C globals for item operations.
extern "C" {
    /// C global: `p_pumborder` option value.
    static p_pumborder: *const c_char;
    /// C global: `cia_flags` (completion item align flags, unsigned).
    static cia_flags: c_uint;
}

// Static string constants for border comparison
extern "C" {
    /// `opt_winborder_values[3]` - "shadow"
    static opt_winborder_shadow: *const c_char;
    /// `opt_winborder_values[7]` - "none"
    static opt_winborder_none: *const c_char;
}

/// Result of align order parsing.
#[repr(C)]
pub struct PumAlignOrder {
    /// First column type (`CPT_ABBR`, `CPT_KIND`, or `CPT_MENU`).
    pub first: c_int,
    /// Second column type.
    pub second: c_int,
    /// Third column type.
    pub third: c_int,
}

/// Parse the completion item align flags into column order.
///
/// # Arguments
/// * `flags` - The completion item align flags (0 for default)
///
/// Returns struct with [first, second, third] column types.
#[no_mangle]
pub const extern "C" fn rs_pum_align_order(flags: c_int) -> PumAlignOrder {
    if flags == 0 {
        // Default order: abbr, kind, menu
        PumAlignOrder {
            first: CPT_ABBR,
            second: CPT_KIND,
            third: CPT_MENU,
        }
    } else {
        // Parse flags: hundreds = first, tens = second, units = third
        PumAlignOrder {
            first: flags / 100,
            second: (flags / 10) % 10,
            third: flags % 10,
        }
    }
}

/// Get the column ordering flags and return the order array.
///
/// This is a convenience function that reads the global `cia_flags`
/// and returns the order.
///
/// # Safety
/// Calls C accessor for global `cia_flags`.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_get_current_align_order() -> PumAlignOrder {
    #[allow(clippy::cast_possible_wrap)]
    let flags = cia_flags as c_int;
    rs_pum_align_order(flags)
}

/// Get item text by type from the popup menu array.
///
/// # Arguments
/// * `array` - Pointer to the popup menu item array
/// * `index` - Index of the item
/// * `item_type` - Type of text to get (`CPT_ABBR`, `CPT_KIND`, or `CPT_MENU`)
///
/// Returns pointer to the text, or null if not available.
///
/// # Safety
/// The caller must ensure `array` is a valid pointer to a `pumitem_T` array
/// with at least `index + 1` elements.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_pum_get_item(
    array: *const PumItemArray,
    index: c_int,
    item_type: c_int,
) -> *const c_char {
    if array.is_null() {
        return std::ptr::null();
    }

    let item = &*array.offset(index as isize);
    match item_type {
        CPT_ABBR => item.pum_text,
        CPT_KIND => item.pum_kind,
        CPT_MENU => item.pum_extra,
        _ => std::ptr::null(),
    }
}

/// Combine user highlight attribute with the given attribute.
///
/// For item types 0 (abbr) and 1 (kind), combines the user-defined
/// highlight attribute with the provided attribute.
///
/// # Arguments
/// * `array` - Pointer to the popup menu item array
/// * `idx` - Index of the item
/// * `item_type` - Type of item (0 = abbr, 1 = kind)
/// * `attr` - Base attribute to combine with
///
/// Returns combined attribute, or original attr if no user highlight.
///
/// # Safety
/// The caller must ensure `array` is a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_user_attr_combine(
    array: *const PumItemArray,
    idx: c_int,
    item_type: c_int,
    attr: c_int,
) -> c_int {
    if array.is_null() || item_type > 1 {
        return attr;
    }

    let item = &*array.offset(idx as isize);
    let user_attr = if item_type == 0 {
        item.pum_user_abbr_hlattr
    } else {
        item.pum_user_kind_hlattr
    };

    if user_attr > 0 {
        hl_combine_attr(attr, user_attr)
    } else {
        attr
    }
}

/// Calculate the border width for the popup menu.
///
/// Returns:
/// - 0 if no border or "none"
/// - 1 if "shadow" border (only right+bottom)
/// - 2 for full border
///
/// # Safety
/// Calls C accessor functions and string comparisons.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_border_width() -> c_int {
    // Check if empty string
    if p_pumborder.is_null() || *p_pumborder == 0 {
        return 0;
    }

    // Check if "none"
    if strequal(p_pumborder, opt_winborder_none) {
        return 0;
    }

    // Check if "shadow" - only has right+bottom
    if strequal(p_pumborder, opt_winborder_shadow) {
        return 1;
    }

    // All other borders have full border (2)
    2
}

/// Check if an item at the given index is empty (null or empty string).
///
/// # Arguments
/// * `array` - Pointer to the popup menu item array
/// * `index` - Index of the item
/// * `item_type` - Type of text to check
///
/// Returns 1 if empty/null, 0 otherwise.
///
/// # Safety
/// The caller must ensure `array` is a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_item_is_empty(
    array: *const PumItemArray,
    index: c_int,
    item_type: c_int,
) -> c_int {
    let ptr = rs_pum_get_item(array, index, item_type);
    if ptr.is_null() {
        return 1;
    }
    // Check if first byte is NUL
    (*ptr == 0) as c_int
}

/// Compute the widths of the widest match, kind, and extra text.
///
/// Writes results directly to `PUM_STATE.base_width`, `.kind_width`,
/// `.extra_width`. Iterates over all items in the popup menu array and
/// measures each field's display width using `vim_strsize`.
///
/// # Arguments
/// * `array` - Pointer to the popup menu item array
///
/// # Safety
/// The caller must ensure `array` is a valid pointer to a `pumitem_T` array
/// with at least `pum_size` elements.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_compute_size(array: *const PumItemArray) {
    let size = PUM_STATE.size;
    let mut base_width: c_int = 0;
    let mut kind_width: c_int = 0;
    let mut extra_width: c_int = 0;

    for i in 0..size {
        let item = &*array.offset(i as isize);
        if !item.pum_text.is_null() {
            let w = vim_strsize(item.pum_text);
            if base_width < w {
                base_width = w;
            }
        }
        if !item.pum_kind.is_null() {
            let w = vim_strsize(item.pum_kind) + 1;
            if kind_width < w {
                kind_width = w;
            }
        }
        if !item.pum_extra.is_null() {
            let w = vim_strsize(item.pum_extra) + 1;
            if extra_width < w {
                extra_width = w;
            }
        }
    }

    PUM_STATE.base_width = base_width;
    PUM_STATE.kind_width = kind_width;
    PUM_STATE.extra_width = extra_width;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_order_default() {
        let order = rs_pum_align_order(0);
        assert_eq!(order.first, CPT_ABBR);
        assert_eq!(order.second, CPT_KIND);
        assert_eq!(order.third, CPT_MENU);
    }

    #[test]
    fn test_align_order_custom() {
        // flags = 210 means first=2 (menu), second=1 (kind), third=0 (abbr)
        let order = rs_pum_align_order(210);
        assert_eq!(order.first, 2);
        assert_eq!(order.second, 1);
        assert_eq!(order.third, 0);
    }

    #[test]
    fn test_align_order_reversed() {
        // flags = 012 means first=0 (abbr), second=1 (kind), third=2 (menu)
        let order = rs_pum_align_order(12);
        assert_eq!(order.first, 0);
        assert_eq!(order.second, 1);
        assert_eq!(order.third, 2);
    }
}
