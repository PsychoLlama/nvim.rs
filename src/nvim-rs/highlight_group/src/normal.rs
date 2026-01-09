//! Normal highlight group management.
//!
//! This module handles the special "Normal" highlight group which defines
//! the default foreground/background/special colors for the UI.
//!
//! The Normal group is special because:
//! - Its colors are cached in global variables for fast access
//! - Other groups can reference "fg" or "bg" to inherit from Normal
//! - Changes to Normal trigger recomputation of all dependent groups
//! - It affects the default UI colors

use std::ffi::c_int;

use crate::types::RgbValue;

// FFI declarations for accessing C global state
extern "C" {
    fn nvim_get_normal_fg() -> c_int;
    fn nvim_get_normal_bg() -> c_int;
    fn nvim_get_normal_sp() -> c_int;
    fn nvim_set_normal_fg(val: c_int);
    fn nvim_set_normal_bg(val: c_int);
    fn nvim_set_normal_sp(val: c_int);
    fn nvim_get_cterm_normal_fg_color() -> c_int;
    fn nvim_get_cterm_normal_bg_color() -> c_int;
    fn nvim_set_cterm_normal_fg_color(val: c_int);
    fn nvim_set_cterm_normal_bg_color(val: c_int);
}

/// The Normal highlight colors (RGB).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NormalColors {
    /// Foreground color (-1 if unset)
    pub fg: RgbValue,
    /// Background color (-1 if unset)
    pub bg: RgbValue,
    /// Special color for undercurl, etc. (-1 if unset)
    pub sp: RgbValue,
}

impl NormalColors {
    /// Invalid/unset color value
    pub const INVALID: RgbValue = -1;

    /// Create a new `NormalColors` with all colors unset.
    #[inline]
    pub const fn new() -> Self {
        NormalColors {
            fg: Self::INVALID,
            bg: Self::INVALID,
            sp: Self::INVALID,
        }
    }

    /// Create from explicit values.
    #[inline]
    pub const fn from_rgb(fg: RgbValue, bg: RgbValue, sp: RgbValue) -> Self {
        NormalColors { fg, bg, sp }
    }

    /// Check if foreground is set.
    #[inline]
    pub const fn has_fg(&self) -> bool {
        self.fg >= 0
    }

    /// Check if background is set.
    #[inline]
    pub const fn has_bg(&self) -> bool {
        self.bg >= 0
    }

    /// Check if special color is set.
    #[inline]
    pub const fn has_sp(&self) -> bool {
        self.sp >= 0
    }

    /// Check if any color is set.
    #[inline]
    pub const fn has_any(&self) -> bool {
        self.has_fg() || self.has_bg() || self.has_sp()
    }

    /// Check if all colors are set.
    #[inline]
    pub const fn has_all(&self) -> bool {
        self.has_fg() && self.has_bg() && self.has_sp()
    }
}

/// The Normal highlight cterm colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NormalCtermColors {
    /// Foreground terminal color (0 if unset, 1-based when set)
    pub fg: c_int,
    /// Background terminal color (0 if unset, 1-based when set)
    pub bg: c_int,
}

impl NormalCtermColors {
    /// Create with all colors unset.
    #[inline]
    pub const fn new() -> Self {
        NormalCtermColors { fg: 0, bg: 0 }
    }

    /// Create from explicit values.
    #[inline]
    pub const fn from_colors(fg: c_int, bg: c_int) -> Self {
        NormalCtermColors { fg, bg }
    }

    /// Check if foreground is set (non-zero).
    #[inline]
    pub const fn has_fg(&self) -> bool {
        self.fg != 0
    }

    /// Check if background is set (non-zero).
    #[inline]
    pub const fn has_bg(&self) -> bool {
        self.bg != 0
    }

    /// Get the actual foreground color value (0-based), or None if unset.
    #[inline]
    pub const fn fg_color(&self) -> Option<c_int> {
        if self.fg > 0 {
            Some(self.fg - 1)
        } else {
            None
        }
    }

    /// Get the actual background color value (0-based), or None if unset.
    #[inline]
    pub const fn bg_color(&self) -> Option<c_int> {
        if self.bg > 0 {
            Some(self.bg - 1)
        } else {
            None
        }
    }
}

/// Get the current Normal highlight RGB colors from global state.
///
/// # Safety
/// This function accesses global state through FFI.
#[inline]
pub fn get_normal_colors() -> NormalColors {
    unsafe {
        NormalColors {
            fg: nvim_get_normal_fg(),
            bg: nvim_get_normal_bg(),
            sp: nvim_get_normal_sp(),
        }
    }
}

/// Set the Normal highlight RGB colors in global state.
///
/// # Safety
/// This function modifies global state through FFI.
#[inline]
pub fn set_normal_colors(colors: &NormalColors) {
    unsafe {
        nvim_set_normal_fg(colors.fg);
        nvim_set_normal_bg(colors.bg);
        nvim_set_normal_sp(colors.sp);
    }
}

/// Get the current Normal highlight cterm colors from global state.
///
/// # Safety
/// This function accesses global state through FFI.
#[inline]
pub fn get_cterm_normal_colors() -> NormalCtermColors {
    unsafe {
        NormalCtermColors {
            fg: nvim_get_cterm_normal_fg_color(),
            bg: nvim_get_cterm_normal_bg_color(),
        }
    }
}

/// Set the Normal highlight cterm colors in global state.
///
/// # Safety
/// This function modifies global state through FFI.
#[inline]
pub fn set_cterm_normal_colors(colors: &NormalCtermColors) {
    unsafe {
        nvim_set_cterm_normal_fg_color(colors.fg);
        nvim_set_cterm_normal_bg_color(colors.bg);
    }
}

/// Reset all Normal colors to their default (unset) values.
///
/// This resets:
/// - RGB colors (fg, bg, sp) to -1
/// - Cterm colors to 0
///
/// # Safety
/// This function modifies global state through FFI.
pub fn reset_normal_colors() {
    unsafe {
        nvim_set_normal_fg(-1);
        nvim_set_normal_bg(-1);
        nvim_set_normal_sp(-1);
        nvim_set_cterm_normal_fg_color(0);
        nvim_set_cterm_normal_bg_color(0);
    }
}

/// Check if the Normal highlight group colors have changed.
///
/// This is used to determine if dependent highlight groups need to be
/// refreshed when the Normal group is updated.
///
/// # Arguments
/// * `old` - The previous Normal colors
/// * `new` - The new Normal colors
///
/// # Returns
/// `true` if any color value changed
#[inline]
pub fn colors_changed(old: &NormalColors, new: &NormalColors) -> bool {
    old.fg != new.fg || old.bg != new.bg || old.sp != new.sp
}

/// Resolve a color reference to "fg" or "bg".
///
/// When a highlight group specifies `guifg=fg` or `guibg=bg`, this function
/// resolves it to the actual Normal group color.
///
/// # Arguments
/// * `color_ref` - The color reference from ColorIdx
/// * `normal` - The current Normal colors
///
/// # Returns
/// The resolved RGB color value
pub fn resolve_color_ref(color_ref: crate::types::ColorIdx, normal: &NormalColors) -> RgbValue {
    match color_ref {
        crate::types::ColorIdx::Fg => normal.fg,
        crate::types::ColorIdx::Bg => normal.bg,
        _ => NormalColors::INVALID,
    }
}

/// Default fallback colors when Normal is not set.
///
/// These are used when the Normal highlight group hasn't been explicitly
/// defined (e.g., during startup before colorscheme is loaded).
#[derive(Debug, Clone, Copy)]
pub struct DefaultColors {
    /// Default foreground (typically white or black depending on background)
    pub fg: RgbValue,
    /// Default background (typically black or white)
    pub bg: RgbValue,
}

impl DefaultColors {
    /// Dark theme defaults (white on black)
    pub const DARK: DefaultColors = DefaultColors {
        fg: 0xFFFFFF, // White
        bg: 0x000000, // Black
    };

    /// Light theme defaults (black on white)
    pub const LIGHT: DefaultColors = DefaultColors {
        fg: 0x000000, // Black
        bg: 0xFFFFFF, // White
    };

    /// Get the foreground, falling back to default if normal is unset.
    #[inline]
    pub const fn get_fg(&self, normal: &NormalColors) -> RgbValue {
        if normal.fg >= 0 {
            normal.fg
        } else {
            self.fg
        }
    }

    /// Get the background, falling back to default if normal is unset.
    #[inline]
    pub const fn get_bg(&self, normal: &NormalColors) -> RgbValue {
        if normal.bg >= 0 {
            normal.bg
        } else {
            self.bg
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_colors_new() {
        let colors = NormalColors::new();
        assert_eq!(colors.fg, -1);
        assert_eq!(colors.bg, -1);
        assert_eq!(colors.sp, -1);
        assert!(!colors.has_fg());
        assert!(!colors.has_bg());
        assert!(!colors.has_sp());
        assert!(!colors.has_any());
    }

    #[test]
    fn test_normal_colors_from_rgb() {
        let colors = NormalColors::from_rgb(0xFF0000, 0x00FF00, 0x0000FF);
        assert_eq!(colors.fg, 0xFF0000);
        assert_eq!(colors.bg, 0x00FF00);
        assert_eq!(colors.sp, 0x0000FF);
        assert!(colors.has_fg());
        assert!(colors.has_bg());
        assert!(colors.has_sp());
        assert!(colors.has_all());
    }

    #[test]
    fn test_normal_colors_partial() {
        let colors = NormalColors::from_rgb(0xFF0000, -1, -1);
        assert!(colors.has_fg());
        assert!(!colors.has_bg());
        assert!(!colors.has_sp());
        assert!(colors.has_any());
        assert!(!colors.has_all());
    }

    #[test]
    fn test_cterm_colors() {
        let colors = NormalCtermColors::new();
        assert_eq!(colors.fg, 0);
        assert_eq!(colors.bg, 0);
        assert!(!colors.has_fg());
        assert!(!colors.has_bg());
        assert_eq!(colors.fg_color(), None);
        assert_eq!(colors.bg_color(), None);

        let colors = NormalCtermColors::from_colors(8, 16);
        assert!(colors.has_fg());
        assert!(colors.has_bg());
        assert_eq!(colors.fg_color(), Some(7));
        assert_eq!(colors.bg_color(), Some(15));
    }

    #[test]
    fn test_colors_changed() {
        let old = NormalColors::from_rgb(0xFF0000, 0x00FF00, 0x0000FF);
        let same = NormalColors::from_rgb(0xFF0000, 0x00FF00, 0x0000FF);
        let different = NormalColors::from_rgb(0xFF0000, 0x00FF00, 0xFFFFFF);

        assert!(!colors_changed(&old, &same));
        assert!(colors_changed(&old, &different));
    }

    #[test]
    fn test_resolve_color_ref() {
        let normal = NormalColors::from_rgb(0xFF0000, 0x00FF00, 0x0000FF);

        assert_eq!(
            resolve_color_ref(crate::types::ColorIdx::Fg, &normal),
            0xFF0000
        );
        assert_eq!(
            resolve_color_ref(crate::types::ColorIdx::Bg, &normal),
            0x00FF00
        );
        assert_eq!(resolve_color_ref(crate::types::ColorIdx::None, &normal), -1);
    }

    #[test]
    fn test_default_colors() {
        let normal_unset = NormalColors::new();
        let normal_set = NormalColors::from_rgb(0xAAAAAA, 0x555555, -1);

        // Dark theme with unset normal
        assert_eq!(DefaultColors::DARK.get_fg(&normal_unset), 0xFFFFFF);
        assert_eq!(DefaultColors::DARK.get_bg(&normal_unset), 0x000000);

        // Dark theme with set normal
        assert_eq!(DefaultColors::DARK.get_fg(&normal_set), 0xAAAAAA);
        assert_eq!(DefaultColors::DARK.get_bg(&normal_set), 0x555555);

        // Light theme with unset normal
        assert_eq!(DefaultColors::LIGHT.get_fg(&normal_unset), 0x000000);
        assert_eq!(DefaultColors::LIGHT.get_bg(&normal_unset), 0xFFFFFF);
    }
}
