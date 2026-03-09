//! Core type definitions for the highlight group system.
//!
//! This module defines the primary data structures used for managing
//! highlight groups in Neovim.

use std::ffi::{c_char, c_int};

/// Flags indicating which parts of a highlight group have been set.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SgSet(pub c_int);

impl SgSet {
    /// No parts have been set
    pub const NONE: SgSet = SgSet(0);
    /// cterm has been set
    pub const CTERM: SgSet = SgSet(2);
    /// gui has been set
    pub const GUI: SgSet = SgSet(4);
    /// link has been set
    pub const LINK: SgSet = SgSet(8);

    /// Check if cterm has been set
    #[inline]
    pub fn has_cterm(self) -> bool {
        (self.0 & Self::CTERM.0) != 0
    }

    /// Check if gui has been set
    #[inline]
    pub fn has_gui(self) -> bool {
        (self.0 & Self::GUI.0) != 0
    }

    /// Check if link has been set
    #[inline]
    pub fn has_link(self) -> bool {
        (self.0 & Self::LINK.0) != 0
    }

    /// Set the cterm flag
    #[inline]
    pub fn set_cterm(&mut self) {
        self.0 |= Self::CTERM.0;
    }

    /// Set the gui flag
    #[inline]
    pub fn set_gui(&mut self) {
        self.0 |= Self::GUI.0;
    }

    /// Set the link flag
    #[inline]
    pub fn set_link(&mut self) {
        self.0 |= Self::LINK.0;
    }
}

/// Special color index values used in highlight groups.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorIdx {
    /// No color set (-1)
    None = -1,
    /// Color specified as hex value (-2)
    Hex = -2,
    /// Use foreground color (-3)
    Fg = -3,
    /// Use background color (-4)
    Bg = -4,
}

impl ColorIdx {
    /// Check if this is a special index (not a normal color index)
    #[inline]
    pub fn is_special(idx: c_int) -> bool {
        idx < 0
    }

    /// Convert from c_int to ColorIdx if it's a special value
    pub fn from_int(idx: c_int) -> Option<Self> {
        match idx {
            -1 => Some(ColorIdx::None),
            -2 => Some(ColorIdx::Hex),
            -3 => Some(ColorIdx::Fg),
            -4 => Some(ColorIdx::Bg),
            _ => None,
        }
    }
}

/// RGB color value type (matches C's RgbValue)
pub type RgbValue = i32;

/// Constant for invalid/unset RGB color
pub const RGB_INVALID: RgbValue = -1;

/// Script ID type (matches C's scid_T)
pub type ScId = c_int;

/// Line number type (matches C's linenr_T)
pub type LineNr = i32;

/// Script context type (matches C's sctx_T).
///
/// Tracks where a highlight group or option was defined/set.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SctxT {
    /// Script ID
    pub sc_sid: ScId,
    /// Sourcing sequence number
    pub sc_seq: c_int,
    /// Line number
    pub sc_lnum: LineNr,
    /// Channel ID (only used when sc_sid is SID_API_CLIENT)
    pub sc_chan: u64,
}

/// Structure that stores information about a highlight group.
///
/// The ID of a highlight group is also called group ID. It is the index in
/// the highlight_ga array PLUS ONE.
///
/// This mirrors the C `HlGroup` struct in `highlight_group.c`.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct HlGroup {
    /// Highlight group name
    pub sg_name: *mut c_char,
    /// Uppercase of sg_name (for case-insensitive lookup)
    pub sg_name_u: *mut c_char,
    /// "hi clear" was used
    pub sg_cleared: bool,
    /// Screen attr (see ATTR_ENTRY)
    pub sg_attr: c_int,
    /// Link to this highlight group ID
    pub sg_link: c_int,
    /// Default link; restored in highlight_clear()
    pub sg_deflink: c_int,
    /// Combination of flags in SG_SET (SG_CTERM, SG_GUI, SG_LINK)
    pub sg_set: c_int,
    /// Script where the default link was set
    pub sg_deflink_sctx: SctxT,
    /// Script in which the group was last set for terminal UIs
    pub sg_script_ctx: SctxT,
    /// "cterm=" highlighting attributes (combination of HlAttrFlags)
    pub sg_cterm: c_int,
    /// Terminal fg color number + 1
    pub sg_cterm_fg: c_int,
    /// Terminal bg color number + 1
    pub sg_cterm_bg: c_int,
    /// Bold attr was set for light color for RGB UIs
    pub sg_cterm_bold: bool,
    /// "gui=" highlighting attributes (combination of HlAttrFlags)
    pub sg_gui: c_int,
    /// RGB foreground color
    pub sg_rgb_fg: RgbValue,
    /// RGB background color
    pub sg_rgb_bg: RgbValue,
    /// RGB special color
    pub sg_rgb_sp: RgbValue,
    /// RGB foreground color index
    pub sg_rgb_fg_idx: c_int,
    /// RGB background color index
    pub sg_rgb_bg_idx: c_int,
    /// RGB special color index
    pub sg_rgb_sp_idx: c_int,
    /// Blend level (0-100 inclusive), -1 if unset
    pub sg_blend: c_int,
    /// Parent of @nested.group
    pub sg_parent: c_int,
}

impl Default for HlGroup {
    fn default() -> Self {
        Self {
            sg_name: std::ptr::null_mut(),
            sg_name_u: std::ptr::null_mut(),
            sg_cleared: true,
            sg_attr: 0,
            sg_link: 0,
            sg_deflink: 0,
            sg_set: 0,
            sg_deflink_sctx: SctxT::default(),
            sg_script_ctx: SctxT::default(),
            sg_cterm: 0,
            sg_cterm_fg: 0,
            sg_cterm_bg: 0,
            sg_cterm_bold: false,
            sg_gui: 0,
            sg_rgb_fg: RGB_INVALID,
            sg_rgb_bg: RGB_INVALID,
            sg_rgb_sp: RGB_INVALID,
            sg_rgb_fg_idx: ColorIdx::None as c_int,
            sg_rgb_bg_idx: ColorIdx::None as c_int,
            sg_rgb_sp_idx: ColorIdx::None as c_int,
            sg_blend: -1,
            sg_parent: 0,
        }
    }
}

/// Growing array (matches C's `garray_T`).
///
/// Used to store the highlight group table (`highlight_ga`).
#[repr(C)]
#[derive(Debug)]
pub struct GArray {
    /// Current number of items used
    pub ga_len: c_int,
    /// Maximum number of items possible
    pub ga_maxlen: c_int,
    /// sizeof(item)
    pub ga_itemsize: c_int,
    /// Number of items to grow each time
    pub ga_growsize: c_int,
    /// Pointer to the first item
    pub ga_data: *mut std::ffi::c_void,
}

/// Maximum length for a syntax name
pub const MAX_SYN_NAME: usize = 200;

/// Maximum value for a highlight ID
pub const MAX_HL_ID: c_int = 20000;

/// Result of looking up a color in the terminal color tables.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LookupColorResult {
    /// The terminal color number
    pub color: c_int,
    /// Bold modifier: -1 = unchanged, 0 = false, 1 = true
    pub bold: c_int,
}

impl LookupColorResult {
    /// Create a new result with the given color and no bold change
    pub const fn new(color: c_int) -> Self {
        LookupColorResult { color, bold: -1 }
    }

    /// Create a new result with the given color and bold value
    pub const fn with_bold(color: c_int, bold: bool) -> Self {
        LookupColorResult {
            color,
            bold: if bold { 1 } else { 0 },
        }
    }
}

/// Color names for terminal colors (16 basic colors + NONE)
pub const COLOR_NAMES: &[&str] = &[
    "Black",
    "DarkBlue",
    "DarkGreen",
    "DarkCyan",
    "DarkRed",
    "DarkMagenta",
    "Brown",
    "DarkYellow",
    "Gray",
    "Grey",
    "LightGray",
    "LightGrey",
    "DarkGray",
    "DarkGrey",
    "Blue",
    "LightBlue",
    "Green",
    "LightGreen",
    "Cyan",
    "LightCyan",
    "Red",
    "LightRed",
    "Magenta",
    "LightMagenta",
    "Yellow",
    "LightYellow",
    "White",
    "NONE",
];

/// Terminal color numbers for 16-color terminals
pub const COLOR_NUMBERS_16: &[c_int] = &[
    0, 1, 2, 3, 4, 5, 6, 6, 7, 7, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 15, -1,
];

/// Terminal color numbers for 88-color terminals
pub const COLOR_NUMBERS_88: &[c_int] = &[
    0, 4, 2, 6, 1, 5, 32, 72, 84, 84, 7, 7, 82, 82, 12, 43, 10, 61, 14, 63, 9, 74, 13, 75, 11, 78,
    15, -1,
];

/// Terminal color numbers for 256-color terminals
pub const COLOR_NUMBERS_256: &[c_int] = &[
    0, 4, 2, 6, 1, 5, 130, 3, 248, 248, 7, 7, 242, 242, 12, 81, 10, 121, 14, 159, 9, 224, 13, 225,
    11, 229, 15, -1,
];

/// Terminal color numbers for 8-color terminals (with bold attribute for bright)
/// Colors 8+ use bold attribute for brightness on some terminals.
pub const COLOR_NUMBERS_8: &[c_int] = &[
    0, 4, 2, 6, 1, 5, 3, 3, 7, 7, 7, 7, 8, 8, 12, 12, 10, 10, 14, 14, 9, 9, 13, 13, 11, 11, 15, -1,
];

// =============================================================================
// FFI Exports
// =============================================================================

/// Get the `RGB_INVALID` constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_hl_rgb_invalid() -> RgbValue {
    RGB_INVALID
}

/// Get the `MAX_HL_ID` constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_hl_max_id() -> c_int {
    MAX_HL_ID
}

/// Check if a color index is a special value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_hl_color_idx_is_special(idx: c_int) -> c_int {
    c_int::from(ColorIdx::is_special(idx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sg_set_flags() {
        let mut flags = SgSet::NONE;
        assert!(!flags.has_cterm());
        assert!(!flags.has_gui());
        assert!(!flags.has_link());

        flags.set_cterm();
        assert!(flags.has_cterm());
        assert!(!flags.has_gui());
        assert!(!flags.has_link());

        flags.set_gui();
        assert!(flags.has_cterm());
        assert!(flags.has_gui());
        assert!(!flags.has_link());

        flags.set_link();
        assert!(flags.has_cterm());
        assert!(flags.has_gui());
        assert!(flags.has_link());
    }

    #[test]
    fn test_color_idx() {
        assert!(ColorIdx::is_special(-1));
        assert!(ColorIdx::is_special(-2));
        assert!(ColorIdx::is_special(-3));
        assert!(ColorIdx::is_special(-4));
        assert!(!ColorIdx::is_special(0));
        assert!(!ColorIdx::is_special(1));

        assert_eq!(ColorIdx::from_int(-1), Some(ColorIdx::None));
        assert_eq!(ColorIdx::from_int(-2), Some(ColorIdx::Hex));
        assert_eq!(ColorIdx::from_int(-3), Some(ColorIdx::Fg));
        assert_eq!(ColorIdx::from_int(-4), Some(ColorIdx::Bg));
        assert_eq!(ColorIdx::from_int(0), None);
    }

    #[test]
    fn test_lookup_color_result() {
        let r1 = LookupColorResult::new(5);
        assert_eq!(r1.color, 5);
        assert_eq!(r1.bold, -1);

        let r2 = LookupColorResult::with_bold(10, true);
        assert_eq!(r2.color, 10);
        assert_eq!(r2.bold, 1);

        let r3 = LookupColorResult::with_bold(15, false);
        assert_eq!(r3.color, 15);
        assert_eq!(r3.bold, 0);
    }

    #[test]
    fn test_color_tables_same_length() {
        assert_eq!(COLOR_NAMES.len(), COLOR_NUMBERS_16.len());
        assert_eq!(COLOR_NAMES.len(), COLOR_NUMBERS_88.len());
        assert_eq!(COLOR_NAMES.len(), COLOR_NUMBERS_256.len());
        assert_eq!(COLOR_NAMES.len(), COLOR_NUMBERS_8.len());
    }

    #[test]
    fn test_sctx_t_default() {
        let sctx = SctxT::default();
        assert_eq!(sctx.sc_sid, 0);
        assert_eq!(sctx.sc_seq, 0);
        assert_eq!(sctx.sc_lnum, 0);
        assert_eq!(sctx.sc_chan, 0);
    }

    #[test]
    fn test_hl_group_default() {
        let group = HlGroup::default();
        assert!(group.sg_name.is_null());
        assert!(group.sg_name_u.is_null());
        assert!(group.sg_cleared);
        assert_eq!(group.sg_attr, 0);
        assert_eq!(group.sg_link, 0);
        assert_eq!(group.sg_deflink, 0);
        assert_eq!(group.sg_set, 0);
        assert_eq!(group.sg_cterm, 0);
        assert_eq!(group.sg_cterm_fg, 0);
        assert_eq!(group.sg_cterm_bg, 0);
        assert!(!group.sg_cterm_bold);
        assert_eq!(group.sg_gui, 0);
        assert_eq!(group.sg_rgb_fg, RGB_INVALID);
        assert_eq!(group.sg_rgb_bg, RGB_INVALID);
        assert_eq!(group.sg_rgb_sp, RGB_INVALID);
        assert_eq!(group.sg_rgb_fg_idx, ColorIdx::None as c_int);
        assert_eq!(group.sg_rgb_bg_idx, ColorIdx::None as c_int);
        assert_eq!(group.sg_rgb_sp_idx, ColorIdx::None as c_int);
        assert_eq!(group.sg_blend, -1);
        assert_eq!(group.sg_parent, 0);
    }
}
