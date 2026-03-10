//! Popup menu item access and manipulation.
//!
//! This module provides functions for accessing popup menu item data
//! and combining highlight attributes.

use std::ffi::{c_char, c_int};

use crate::PUM_STATE;

/// Completion item type for abbreviation/text.
pub const CPT_ABBR: c_int = 0;
/// Completion item type for kind.
pub const CPT_KIND: c_int = 1;
/// Completion item type for menu/extra text.
pub const CPT_MENU: c_int = 2;

/// Opaque handle to a popup menu item array.
#[repr(C)]
pub struct PumItemArray {
    _private: [u8; 0],
}

// FFI declarations for item access
extern "C" {
    /// Get item text field from array at index.
    pub fn nvim_pum_item_get_text(array: *const PumItemArray, index: c_int) -> *const c_char;
    /// Get item kind field from array at index.
    pub fn nvim_pum_item_get_kind(array: *const PumItemArray, index: c_int) -> *const c_char;
    /// Get item extra field from array at index.
    pub fn nvim_pum_item_get_extra(array: *const PumItemArray, index: c_int) -> *const c_char;
    /// Get item user abbr highlight attr from array at index.
    pub fn nvim_pum_item_get_user_abbr_hlattr(array: *const PumItemArray, index: c_int) -> c_int;
    /// Get item user kind highlight attr from array at index.
    pub fn nvim_pum_item_get_user_kind_hlattr(array: *const PumItemArray, index: c_int) -> c_int;
    /// Combine highlight attributes (`hl_combine_attr`).
    fn hl_combine_attr(char_attr: c_int, comb_attr: c_int) -> c_int;
    /// Get the `pumborder` option value.
    fn nvim_get_p_pumborder() -> *const c_char;
    /// Check if two strings are equal.
    fn strequal(s1: *const c_char, s2: *const c_char) -> c_int;
    /// Get the completion item align flags global.
    fn nvim_get_cia_flags() -> c_int;
    /// Compute the display width of a string.
    fn vim_strsize(s: *const c_char) -> c_int;
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
/// * `cia_flags` - The completion item align flags (0 for default)
///
/// Returns struct with [first, second, third] column types.
#[no_mangle]
pub const extern "C" fn rs_pum_align_order(cia_flags: c_int) -> PumAlignOrder {
    if cia_flags == 0 {
        // Default order: abbr, kind, menu
        PumAlignOrder {
            first: CPT_ABBR,
            second: CPT_KIND,
            third: CPT_MENU,
        }
    } else {
        // Parse flags: hundreds = first, tens = second, units = third
        PumAlignOrder {
            first: cia_flags / 100,
            second: (cia_flags / 10) % 10,
            third: cia_flags % 10,
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
    let flags = nvim_get_cia_flags();
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
pub unsafe extern "C" fn rs_pum_get_item(
    array: *const PumItemArray,
    index: c_int,
    item_type: c_int,
) -> *const c_char {
    if array.is_null() {
        return std::ptr::null();
    }

    match item_type {
        CPT_ABBR => nvim_pum_item_get_text(array, index),
        CPT_KIND => nvim_pum_item_get_kind(array, index),
        CPT_MENU => nvim_pum_item_get_extra(array, index),
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

    let user_attr = if item_type == 0 {
        nvim_pum_item_get_user_abbr_hlattr(array, idx)
    } else {
        nvim_pum_item_get_user_kind_hlattr(array, idx)
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
    let p_pumborder = nvim_get_p_pumborder();

    // Check if empty string
    if p_pumborder.is_null() || *p_pumborder == 0 {
        return 0;
    }

    // Check if "none"
    if strequal(p_pumborder, opt_winborder_none) != 0 {
        return 0;
    }

    // Check if "shadow" - only has right+bottom
    if strequal(p_pumborder, opt_winborder_shadow) != 0 {
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
        let text = nvim_pum_item_get_text(array, i);
        if !text.is_null() {
            let w = vim_strsize(text);
            if base_width < w {
                base_width = w;
            }
        }
        let kind = nvim_pum_item_get_kind(array, i);
        if !kind.is_null() {
            let w = vim_strsize(kind) + 1;
            if kind_width < w {
                kind_width = w;
            }
        }
        let extra = nvim_pum_item_get_extra(array, i);
        if !extra.is_null() {
            let w = vim_strsize(extra) + 1;
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
