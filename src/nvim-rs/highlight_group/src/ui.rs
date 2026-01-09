//! UI integration for highlight groups.
//!
//! This module provides types and utilities for integrating the highlight
//! system with Neovim's UI layer, including:
//! - Terminal color capability detection
//! - Color mode selection (RGB, 256, 16, 8 colors)
//! - UI update notifications
//!
//! The actual UI communication happens in C code, but this module provides
//! the Rust-side types and logic.

use std::ffi::c_int;

use crate::types::{COLOR_NUMBERS_16, COLOR_NUMBERS_256, COLOR_NUMBERS_8, COLOR_NUMBERS_88};

// FFI declarations
extern "C" {
    fn nvim_get_t_colors() -> c_int;
}

/// Terminal color modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// 8 colors (basic ANSI)
    Colors8,
    /// 16 colors (extended ANSI)
    Colors16,
    /// 88 colors (xterm-88color)
    Colors88,
    /// 256 colors (xterm-256color)
    Colors256,
    /// True color (24-bit RGB)
    TrueColor,
}

impl ColorMode {
    /// Get the number of colors for this mode.
    pub const fn color_count(&self) -> c_int {
        match self {
            ColorMode::Colors8 => 8,
            ColorMode::Colors16 => 16,
            ColorMode::Colors88 => 88,
            ColorMode::Colors256 => 256,
            ColorMode::TrueColor => 16777216, // 2^24
        }
    }

    /// Check if this mode supports RGB colors directly.
    #[inline]
    pub const fn supports_rgb(&self) -> bool {
        matches!(self, ColorMode::TrueColor)
    }

    /// Check if this mode supports at least 256 colors.
    #[inline]
    pub const fn supports_256(&self) -> bool {
        matches!(self, ColorMode::Colors256 | ColorMode::TrueColor)
    }

    /// Detect the color mode from the terminal color count.
    pub fn from_t_colors(t_colors: c_int) -> ColorMode {
        if t_colors >= 16777216 {
            ColorMode::TrueColor
        } else if t_colors >= 256 {
            ColorMode::Colors256
        } else if t_colors >= 88 {
            ColorMode::Colors88
        } else if t_colors >= 16 {
            ColorMode::Colors16
        } else {
            ColorMode::Colors8
        }
    }
}

/// Get the current terminal color mode.
///
/// # Safety
/// This function accesses global state through FFI.
pub fn get_color_mode() -> ColorMode {
    let t_colors = unsafe { nvim_get_t_colors() };
    ColorMode::from_t_colors(t_colors)
}

/// Get the terminal color count.
///
/// # Safety
/// This function accesses global state through FFI.
pub fn get_t_colors() -> c_int {
    unsafe { nvim_get_t_colors() }
}

/// Get the appropriate color table for the current terminal.
pub fn get_color_table(mode: ColorMode) -> &'static [c_int] {
    match mode {
        ColorMode::Colors8 => COLOR_NUMBERS_8,
        ColorMode::Colors16 => COLOR_NUMBERS_16,
        ColorMode::Colors88 => COLOR_NUMBERS_88,
        ColorMode::Colors256 | ColorMode::TrueColor => COLOR_NUMBERS_256,
    }
}

/// Map a basic color index to a terminal color number.
///
/// # Arguments
/// * `idx` - Color index (0-27, matching COLOR_NAMES)
/// * `mode` - Terminal color mode
///
/// # Returns
/// The terminal color number, or -1 for NONE/invalid
pub fn map_color_index(idx: usize, mode: ColorMode) -> c_int {
    let table = get_color_table(mode);
    if idx < table.len() {
        table[idx]
    } else {
        -1
    }
}

/// UI capability flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct UiCapabilities {
    /// UI supports RGB colors
    pub rgb: bool,
    /// UI supports 256 colors
    pub ext_colors_256: bool,
    /// UI is a terminal
    pub is_terminal: bool,
    /// UI supports semantic tokens
    pub semantic_tokens: bool,
}

impl UiCapabilities {
    /// Create capabilities from terminal color count.
    pub fn from_t_colors(t_colors: c_int, is_tgc: bool) -> Self {
        UiCapabilities {
            rgb: is_tgc || t_colors >= 16777216,
            ext_colors_256: t_colors >= 256,
            is_terminal: true,
            semantic_tokens: false,
        }
    }

    /// Get the best color mode for these capabilities.
    pub fn best_color_mode(&self) -> ColorMode {
        if self.rgb {
            ColorMode::TrueColor
        } else if self.ext_colors_256 {
            ColorMode::Colors256
        } else {
            ColorMode::Colors16
        }
    }
}

/// Actions to request from the UI layer.
#[derive(Debug, Clone, Copy, Default)]
pub struct UiUpdateRequest {
    /// Update default colors
    pub update_default_colors: bool,
    /// Update mode info (cursor styles)
    pub update_mode_info: bool,
    /// Flush pending updates
    pub flush: bool,
}

impl UiUpdateRequest {
    /// No updates needed.
    pub const NONE: UiUpdateRequest = UiUpdateRequest {
        update_default_colors: false,
        update_mode_info: false,
        flush: false,
    };

    /// Request default colors update.
    pub fn default_colors() -> Self {
        UiUpdateRequest {
            update_default_colors: true,
            update_mode_info: false,
            flush: false,
        }
    }

    /// Request mode info update.
    pub fn mode_info() -> Self {
        UiUpdateRequest {
            update_default_colors: false,
            update_mode_info: true,
            flush: false,
        }
    }

    /// Request both updates.
    pub fn all() -> Self {
        UiUpdateRequest {
            update_default_colors: true,
            update_mode_info: true,
            flush: true,
        }
    }

    /// Check if any update is needed.
    #[inline]
    pub fn any_needed(&self) -> bool {
        self.update_default_colors || self.update_mode_info || self.flush
    }

    /// Merge with another request.
    pub fn merge(self, other: UiUpdateRequest) -> UiUpdateRequest {
        UiUpdateRequest {
            update_default_colors: self.update_default_colors || other.update_default_colors,
            update_mode_info: self.update_mode_info || other.update_mode_info,
            flush: self.flush || other.flush,
        }
    }
}

/// Convert an RGB color to the nearest terminal color.
///
/// This uses a simple algorithm to find the closest match in the
/// 256-color palette.
///
/// # Arguments
/// * `rgb` - RGB color value (0xRRGGBB)
/// * `mode` - Terminal color mode
///
/// # Returns
/// The terminal color number
pub fn rgb_to_terminal_color(rgb: i32, mode: ColorMode) -> c_int {
    if rgb < 0 {
        return -1;
    }

    let r = ((rgb >> 16) & 0xFF) as u8;
    let g = ((rgb >> 8) & 0xFF) as u8;
    let b = (rgb & 0xFF) as u8;

    match mode {
        ColorMode::TrueColor => rgb as c_int,
        ColorMode::Colors256 => rgb_to_256(r, g, b),
        ColorMode::Colors88 => rgb_to_88(r, g, b),
        ColorMode::Colors16 | ColorMode::Colors8 => rgb_to_16(r, g, b),
    }
}

/// Convert RGB to 256-color palette index.
fn rgb_to_256(r: u8, g: u8, b: u8) -> c_int {
    // Check for grayscale
    if r == g && g == b {
        if r < 8 {
            return 16; // black
        }
        if r > 248 {
            return 231; // white
        }
        // Grayscale: 232-255 (24 shades)
        return (((r as u32 - 8) * 24 / 240) + 232) as c_int;
    }

    // 6x6x6 color cube: 16-231
    let ri = color_component_to_cube(r);
    let gi = color_component_to_cube(g);
    let bi = color_component_to_cube(b);

    (16 + 36 * ri + 6 * gi + bi) as c_int
}

/// Convert a color component (0-255) to a 6-level cube index (0-5).
#[inline]
fn color_component_to_cube(c: u8) -> u32 {
    if c < 48 {
        0
    } else if c < 115 {
        1
    } else {
        ((c as u32 - 35) / 40).min(5)
    }
}

/// Convert RGB to 88-color palette index.
fn rgb_to_88(r: u8, g: u8, b: u8) -> c_int {
    // 88-color has a 4x4x4 color cube (16-79) and 8 grays (80-87)
    if r == g && g == b {
        if r < 47 {
            return 16;
        }
        if r > 230 {
            return 79;
        }
        return (80 + ((r as u32 - 47) * 8 / 183)) as c_int;
    }

    let ri = (r as u32 * 4 / 256).min(3);
    let gi = (g as u32 * 4 / 256).min(3);
    let bi = (b as u32 * 4 / 256).min(3);

    (16 + 16 * ri + 4 * gi + bi) as c_int
}

/// Convert RGB to 16-color palette index.
fn rgb_to_16(r: u8, g: u8, b: u8) -> c_int {
    // Simple brightness-based mapping
    let brightness = (r as u32 + g as u32 + b as u32) / 3;

    // Determine if it's a light or dark color
    let is_bright = brightness > 127;

    // Find dominant color component
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let saturation = if max > 0 {
        (max - min) as u32 * 255 / max as u32
    } else {
        0
    };

    if saturation < 50 {
        // Low saturation = gray/white/black
        if brightness < 64 {
            return 0; // Black
        } else if brightness < 192 {
            return if is_bright { 7 } else { 8 }; // Gray
        } else {
            return 15; // White
        }
    }

    // Map to basic colors based on RGB components
    let base = if r >= g && r >= b {
        if g > b {
            3
        } else {
            1
        } // Yellow/Red
    } else if g >= r && g >= b {
        if r > b {
            3
        } else {
            2
        } // Yellow/Green
    } else if r > g {
        5 // Magenta
    } else if g > r {
        6 // Cyan
    } else {
        4 // Blue
    };

    if is_bright {
        base + 8
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mode() {
        assert_eq!(ColorMode::Colors8.color_count(), 8);
        assert_eq!(ColorMode::Colors256.color_count(), 256);
        assert!(ColorMode::TrueColor.supports_rgb());
        assert!(!ColorMode::Colors256.supports_rgb());
        assert!(ColorMode::Colors256.supports_256());
    }

    #[test]
    fn test_color_mode_from_t_colors() {
        assert_eq!(ColorMode::from_t_colors(8), ColorMode::Colors8);
        assert_eq!(ColorMode::from_t_colors(16), ColorMode::Colors16);
        assert_eq!(ColorMode::from_t_colors(88), ColorMode::Colors88);
        assert_eq!(ColorMode::from_t_colors(256), ColorMode::Colors256);
        assert_eq!(ColorMode::from_t_colors(16777216), ColorMode::TrueColor);
    }

    #[test]
    fn test_map_color_index() {
        // Black is always 0
        assert_eq!(map_color_index(0, ColorMode::Colors256), 0);
        // NONE is always -1
        assert_eq!(map_color_index(27, ColorMode::Colors256), -1);
    }

    #[test]
    fn test_ui_capabilities() {
        let caps = UiCapabilities::from_t_colors(256, false);
        assert!(!caps.rgb);
        assert!(caps.ext_colors_256);
        assert_eq!(caps.best_color_mode(), ColorMode::Colors256);

        let caps_tgc = UiCapabilities::from_t_colors(256, true);
        assert!(caps_tgc.rgb);
        assert_eq!(caps_tgc.best_color_mode(), ColorMode::TrueColor);
    }

    #[test]
    fn test_ui_update_request() {
        assert!(!UiUpdateRequest::NONE.any_needed());
        assert!(UiUpdateRequest::default_colors().any_needed());
        assert!(UiUpdateRequest::all().update_default_colors);
        assert!(UiUpdateRequest::all().update_mode_info);
    }

    #[test]
    fn test_rgb_to_256() {
        // Black
        assert_eq!(rgb_to_256(0, 0, 0), 16);
        // White
        assert_eq!(rgb_to_256(255, 255, 255), 231);
        // Pure red
        assert_eq!(rgb_to_256(255, 0, 0), 196);
        // Pure green
        assert_eq!(rgb_to_256(0, 255, 0), 46);
        // Pure blue
        assert_eq!(rgb_to_256(0, 0, 255), 21);
    }

    #[test]
    fn test_rgb_to_terminal_color() {
        // True color returns the RGB value
        assert_eq!(
            rgb_to_terminal_color(0xFF0000, ColorMode::TrueColor),
            0xFF0000
        );
        // Invalid returns -1
        assert_eq!(rgb_to_terminal_color(-1, ColorMode::Colors256), -1);
    }

    #[test]
    fn test_rgb_to_16() {
        // Black
        assert_eq!(rgb_to_16(0, 0, 0), 0);
        // White
        assert_eq!(rgb_to_16(255, 255, 255), 15);
        // Red
        assert!(rgb_to_16(255, 0, 0) == 9 || rgb_to_16(255, 0, 0) == 1);
    }
}
