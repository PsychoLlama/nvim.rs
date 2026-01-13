//! Color conversion and manipulation utilities for highlight groups.
//!
//! This module provides utilities for converting between color formats,
//! parsing color strings, and working with the color name table.
//!
//! Note: The main color conversion FFI functions (`rs_name_to_color`,
//! `rs_coloridx_to_name`, etc.) are in the `nvim-highlight` crate since
//! they were migrated earlier. This module provides additional utilities.

use std::ffi::c_int;

use crate::types::{ColorIdx, RgbValue, RGB_INVALID};

/// Parse a hex color string to RGB value.
///
/// Supports formats:
/// - `#RRGGBB` (6 digits)
/// - `#RGB` (3 digits, expanded to 6)
///
/// # Arguments
/// * `s` - Color string (with or without leading '#')
///
/// # Returns
/// The RGB value or `RGB_INVALID` if parsing fails
pub fn parse_hex_color(s: &str) -> RgbValue {
    let s = s.strip_prefix('#').unwrap_or(s);

    match s.len() {
        6 => {
            // #RRGGBB format
            if let Ok(color) = i32::from_str_radix(s, 16) {
                return color;
            }
        }
        3 => {
            // #RGB format - expand each digit to two
            let chars: Vec<char> = s.chars().collect();
            if chars.iter().all(|c| c.is_ascii_hexdigit()) {
                let r = chars[0].to_digit(16).unwrap_or(0);
                let g = chars[1].to_digit(16).unwrap_or(0);
                let b = chars[2].to_digit(16).unwrap_or(0);
                // Expand: 0xF -> 0xFF
                return (((r << 4) | r) << 16 | ((g << 4) | g) << 8 | ((b << 4) | b)) as RgbValue;
            }
        }
        _ => {}
    }
    RGB_INVALID
}

/// Extract the red component from an RGB value.
#[inline]
pub const fn rgb_red(color: RgbValue) -> u8 {
    ((color >> 16) & 0xFF) as u8
}

/// Extract the green component from an RGB value.
#[inline]
pub const fn rgb_green(color: RgbValue) -> u8 {
    ((color >> 8) & 0xFF) as u8
}

/// Extract the blue component from an RGB value.
#[inline]
pub const fn rgb_blue(color: RgbValue) -> u8 {
    (color & 0xFF) as u8
}

/// Create an RGB value from red, green, blue components.
#[inline]
pub const fn rgb_from_components(r: u8, g: u8, b: u8) -> RgbValue {
    ((r as i32) << 16) | ((g as i32) << 8) | (b as i32)
}

/// Check if an RGB color is a valid (not invalid/unset).
#[inline]
pub const fn is_valid_rgb(color: RgbValue) -> bool {
    color >= 0
}

/// Determine if an RGB color is "dark" (for background option).
///
/// This uses a simple luminance calculation based on human perception:
/// - Green contributes most to perceived brightness
/// - Red contributes moderately
/// - Blue contributes least
///
/// # Arguments
/// * `color` - RGB color value
///
/// # Returns
/// `true` if the color is dark, `false` if light
pub fn is_dark_color(color: RgbValue) -> bool {
    if !is_valid_rgb(color) {
        return true; // Default to dark if invalid
    }

    let r = rgb_red(color) as i32;
    let g = rgb_green(color) as i32;
    let b = rgb_blue(color) as i32;

    // Using sRGB luminance formula (simplified):
    // Y = 0.299*R + 0.587*G + 0.114*B
    // Threshold at ~127 (half of 255)
    let luminance = (299 * r + 587 * g + 114 * b) / 1000;
    luminance < 128
}

/// Determine if a cterm color number is "dark".
///
/// For the standard 16-color palette:
/// - Colors 0-6 are dark (black, dark colors)
/// - Color 7 (white) is light
/// - Color 8 is dark (bright black/grey)
/// - Colors 9-15 are light (bright colors)
///
/// # Arguments
/// * `color` - cterm color number (0-15 for standard palette)
/// * `t_colors` - Terminal color count
///
/// # Returns
/// Some(true) if dark, Some(false) if light, None if unknown
pub fn is_dark_cterm_color(color: c_int, t_colors: c_int) -> Option<bool> {
    if color < 0 {
        return None;
    }

    if t_colors < 16 {
        // Limited palette: 0 and 4 (black and blue) are dark
        Some(color == 0 || color == 4)
    } else if color < 16 {
        // Standard 16 colors
        Some(color < 7 || color == 8)
    } else {
        // Extended colors - can't determine
        None
    }
}

/// Format an RGB color as a hex string.
///
/// # Arguments
/// * `color` - RGB color value
/// * `buffer` - Buffer to write to (must be at least 8 bytes: "#RRGGBB\0")
///
/// # Returns
/// Slice of the buffer containing the formatted string (without null terminator)
pub fn format_hex_color(color: RgbValue, buffer: &mut [u8; 8]) -> &str {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

    buffer[0] = b'#';
    buffer[1] = HEX_DIGITS[((color >> 20) & 0xF) as usize];
    buffer[2] = HEX_DIGITS[((color >> 16) & 0xF) as usize];
    buffer[3] = HEX_DIGITS[((color >> 12) & 0xF) as usize];
    buffer[4] = HEX_DIGITS[((color >> 8) & 0xF) as usize];
    buffer[5] = HEX_DIGITS[((color >> 4) & 0xF) as usize];
    buffer[6] = HEX_DIGITS[(color & 0xF) as usize];
    buffer[7] = 0;

    // SAFETY: We only wrote ASCII characters
    unsafe { std::str::from_utf8_unchecked(&buffer[0..7]) }
}

/// Get the color index type description.
///
/// # Arguments
/// * `idx` - Color index value
///
/// # Returns
/// String describing the color index type
pub fn color_idx_description(idx: c_int) -> &'static str {
    match ColorIdx::from_int(idx) {
        Some(ColorIdx::None) => "NONE",
        Some(ColorIdx::Hex) => "hex color",
        Some(ColorIdx::Fg) => "fg (foreground)",
        Some(ColorIdx::Bg) => "bg (background)",
        None => {
            if idx >= 0 {
                "color table index"
            } else {
                "unknown"
            }
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

use std::ffi::{c_char, CStr};

/// Parse a hex color string to RGB value.
///
/// # Safety
/// `s` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_hl_parse_hex_color(s: *const c_char) -> RgbValue {
    if s.is_null() {
        return RGB_INVALID;
    }
    let s_str = match unsafe { CStr::from_ptr(s) }.to_str() {
        Ok(s) => s,
        Err(_) => return RGB_INVALID,
    };
    parse_hex_color(s_str)
}

/// Extract the red component from an RGB value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_hl_rgb_red(color: RgbValue) -> c_int {
    rgb_red(color) as c_int
}

/// Extract the green component from an RGB value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_hl_rgb_green(color: RgbValue) -> c_int {
    rgb_green(color) as c_int
}

/// Extract the blue component from an RGB value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_hl_rgb_blue(color: RgbValue) -> c_int {
    rgb_blue(color) as c_int
}

/// Create an RGB value from red, green, blue components.
#[unsafe(no_mangle)]
pub extern "C" fn rs_hl_rgb_from_components(r: c_int, g: c_int, b: c_int) -> RgbValue {
    rgb_from_components(r as u8, g as u8, b as u8)
}

/// Check if an RGB color is valid (not invalid/unset).
#[unsafe(no_mangle)]
pub extern "C" fn rs_hl_is_valid_rgb(color: RgbValue) -> c_int {
    c_int::from(is_valid_rgb(color))
}

/// Check if an RGB color is dark.
#[unsafe(no_mangle)]
pub extern "C" fn rs_hl_is_dark_color(color: RgbValue) -> c_int {
    c_int::from(is_dark_color(color))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_6digit() {
        assert_eq!(parse_hex_color("#ff0000"), 0xff0000);
        assert_eq!(parse_hex_color("#00ff00"), 0x00ff00);
        assert_eq!(parse_hex_color("#0000ff"), 0x0000ff);
        assert_eq!(parse_hex_color("#ffffff"), 0xffffff);
        assert_eq!(parse_hex_color("#000000"), 0x000000);
        assert_eq!(parse_hex_color("#abcdef"), 0xabcdef);
        // Without #
        assert_eq!(parse_hex_color("ff0000"), 0xff0000);
    }

    #[test]
    fn test_parse_hex_color_3digit() {
        assert_eq!(parse_hex_color("#f00"), 0xff0000);
        assert_eq!(parse_hex_color("#0f0"), 0x00ff00);
        assert_eq!(parse_hex_color("#00f"), 0x0000ff);
        assert_eq!(parse_hex_color("#fff"), 0xffffff);
        assert_eq!(parse_hex_color("#000"), 0x000000);
        assert_eq!(parse_hex_color("#abc"), 0xaabbcc);
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert_eq!(parse_hex_color(""), RGB_INVALID);
        assert_eq!(parse_hex_color("#"), RGB_INVALID);
        assert_eq!(parse_hex_color("#gg0000"), RGB_INVALID);
        assert_eq!(parse_hex_color("#12345"), RGB_INVALID); // 5 digits
        assert_eq!(parse_hex_color("#1234567"), RGB_INVALID); // 7 digits
    }

    #[test]
    fn test_rgb_components() {
        let color = 0xaabbcc;
        assert_eq!(rgb_red(color), 0xaa);
        assert_eq!(rgb_green(color), 0xbb);
        assert_eq!(rgb_blue(color), 0xcc);

        assert_eq!(rgb_from_components(0xaa, 0xbb, 0xcc), color);
    }

    #[test]
    fn test_is_valid_rgb() {
        assert!(is_valid_rgb(0x000000));
        assert!(is_valid_rgb(0xffffff));
        assert!(!is_valid_rgb(RGB_INVALID));
        assert!(!is_valid_rgb(-2));
    }

    #[test]
    fn test_is_dark_color() {
        assert!(is_dark_color(0x000000)); // Black
        assert!(!is_dark_color(0xffffff)); // White
        assert!(is_dark_color(0x000080)); // Dark blue
        assert!(!is_dark_color(0xffff00)); // Yellow
        assert!(is_dark_color(RGB_INVALID)); // Invalid defaults to dark
    }

    #[test]
    fn test_is_dark_cterm_color() {
        // 16 color mode
        assert_eq!(is_dark_cterm_color(0, 16), Some(true)); // black
        assert_eq!(is_dark_cterm_color(1, 16), Some(true)); // dark red
        assert_eq!(is_dark_cterm_color(7, 16), Some(false)); // white
        assert_eq!(is_dark_cterm_color(8, 16), Some(true)); // bright black
        assert_eq!(is_dark_cterm_color(15, 16), Some(false)); // bright white

        // 8 color mode
        assert_eq!(is_dark_cterm_color(0, 8), Some(true)); // black
        assert_eq!(is_dark_cterm_color(4, 8), Some(true)); // blue (dark)
        assert_eq!(is_dark_cterm_color(3, 8), Some(false)); // not 0 or 4

        // Extended colors (256)
        assert_eq!(is_dark_cterm_color(16, 256), None);

        // Invalid color
        assert_eq!(is_dark_cterm_color(-1, 16), None);
    }

    #[test]
    fn test_format_hex_color() {
        let mut buf = [0u8; 8];

        assert_eq!(format_hex_color(0xff0000, &mut buf), "#ff0000");
        assert_eq!(format_hex_color(0x00ff00, &mut buf), "#00ff00");
        assert_eq!(format_hex_color(0x0000ff, &mut buf), "#0000ff");
        assert_eq!(format_hex_color(0xffffff, &mut buf), "#ffffff");
        assert_eq!(format_hex_color(0x000000, &mut buf), "#000000");
        assert_eq!(format_hex_color(0xabcdef, &mut buf), "#abcdef");
    }

    #[test]
    fn test_color_idx_description() {
        assert_eq!(color_idx_description(-1), "NONE");
        assert_eq!(color_idx_description(-2), "hex color");
        assert_eq!(color_idx_description(-3), "fg (foreground)");
        assert_eq!(color_idx_description(-4), "bg (background)");
        assert_eq!(color_idx_description(0), "color table index");
        assert_eq!(color_idx_description(100), "color table index");
        assert_eq!(color_idx_description(-5), "unknown");
    }
}
