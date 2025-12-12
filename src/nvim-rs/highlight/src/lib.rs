//! Highlight and color manipulation functions for Neovim
//!
//! This crate provides color blending and conversion functions used by the
//! highlight system.

use std::ffi::{c_char, c_int, CStr};

extern "C" {
    /// Get the terminal color count from C globals
    fn nvim_get_t_colors() -> c_int;
}

// ============================================================================
// Color Blending
// ============================================================================

/// Blend two RGB colors together based on a ratio.
///
/// # Arguments
/// * `ratio` - Blend ratio (0-100). 100 means full rgb1, 0 means full rgb2.
/// * `rgb1` - First RGB color (0xRRGGBB format)
/// * `rgb2` - Second RGB color (0xRRGGBB format)
///
/// # Returns
/// Blended RGB color in 0xRRGGBB format
#[no_mangle]
pub extern "C" fn rs_rgb_blend(ratio: c_int, rgb1: c_int, rgb2: c_int) -> c_int {
    let a = ratio;
    let b = 100 - ratio;

    let r1 = (rgb1 >> 16) & 0xFF;
    let g1 = (rgb1 >> 8) & 0xFF;
    let b1 = rgb1 & 0xFF;

    let r2 = (rgb2 >> 16) & 0xFF;
    let g2 = (rgb2 >> 8) & 0xFF;
    let b2 = rgb2 & 0xFF;

    let mr = (a * r1 + b * r2) / 100;
    let mg = (a * g1 + b * g2) / 100;
    let mb = (a * b1 + b * b2) / 100;

    (mr << 16) + (mg << 8) + mb
}

// ============================================================================
// Color Conversion Tables
// ============================================================================

/// xterm 6x6x6 color cube values
const CUBE_VALUE: [c_int; 6] = [0x00, 0x5F, 0x87, 0xAF, 0xD7, 0xFF];

/// xterm grey ramp values (colors 232-255)
const GREY_RAMP: [c_int; 24] = [
    0x08, 0x12, 0x1C, 0x26, 0x30, 0x3A, 0x44, 0x4E, 0x58, 0x62, 0x6C, 0x76, 0x80, 0x8A, 0x94, 0x9E,
    0xA8, 0xB2, 0xBC, 0xC6, 0xD0, 0xDA, 0xE4, 0xEE,
];

/// ANSI 16-color table: [R, G, B, idx]
const ANSI_TABLE: [[u8; 4]; 16] = [
    [0, 0, 0, 1],         // black
    [224, 0, 0, 2],       // dark red
    [0, 224, 0, 3],       // dark green
    [224, 224, 0, 4],     // dark yellow / brown
    [0, 0, 224, 5],       // dark blue
    [224, 0, 224, 6],     // dark magenta
    [0, 224, 224, 7],     // dark cyan
    [224, 224, 224, 8],   // light grey
    [128, 128, 128, 9],   // dark grey
    [255, 64, 64, 10],    // light red
    [64, 255, 64, 11],    // light green
    [255, 255, 64, 12],   // light yellow
    [64, 64, 255, 13],    // light blue
    [255, 64, 255, 14],   // light magenta
    [64, 255, 255, 15],   // light cyan
    [255, 255, 255, 16],  // white
];

/// Convert 8-bit color (0-255) to RGB color.
/// This is compatible with xterm.
///
/// # Arguments
/// * `nr` - 8-bit color number (0-255)
///
/// # Returns
/// RGB color in 0xRRGGBB format
#[no_mangle]
pub extern "C" fn rs_hl_cterm2rgb_color(nr: c_int) -> c_int {
    if nr < 16 {
        // ANSI colors
        let entry = &ANSI_TABLE[nr as usize];
        return (c_int::from(entry[0]) << 16) | (c_int::from(entry[1]) << 8) | c_int::from(entry[2]);
    }

    if nr < 232 {
        // 6x6x6 color cube (colors 16-231)
        let idx = nr - 16;
        let r_idx = idx / 36;
        let g_idx = (idx % 36) / 6;
        let b_idx = idx % 6;
        let r = CUBE_VALUE[r_idx as usize];
        let g = CUBE_VALUE[g_idx as usize];
        let b = CUBE_VALUE[b_idx as usize];
        return (r << 16) | (g << 8) | b;
    }

    // Grey ramp (colors 232-255)
    let grey = GREY_RAMP[(nr - 232) as usize];
    (grey << 16) | (grey << 8) | grey
}

/// Convert RGB color to 8-bit color (0-255).
/// Uses the 6x6x6 color cube portion of the xterm 256-color palette.
///
/// # Arguments
/// * `rgb` - RGB color in 0xRRGGBB format
///
/// # Returns
/// 8-bit color number (16-231, the color cube range)
#[no_mangle]
pub extern "C" fn rs_hl_rgb2cterm_color(rgb: c_int) -> c_int {
    let r = (rgb >> 16) & 0xFF;
    let g = (rgb >> 8) & 0xFF;
    let b = rgb & 0xFF;

    // Map to 6x6x6 cube indices and add offset 16
    (r * 6 / 256) * 36 + (g * 6 / 256) * 6 + (b * 6 / 256) + 16
}

/// Blend two cterm colors together based on a ratio.
///
/// 1. Converts cterm color numbers to RGB.
/// 2. Blends the RGB colors.
/// 3. Converts the RGB result back to a cterm color.
///
/// # Arguments
/// * `ratio` - Blend ratio (0-100). 100 means full c1, 0 means full c2.
/// * `c1` - First cterm color (0-255)
/// * `c2` - Second cterm color (0-255)
///
/// # Returns
/// Blended cterm color number
#[no_mangle]
pub extern "C" fn rs_cterm_blend(ratio: c_int, c1: i16, c2: i16) -> c_int {
    let rgb1 = rs_hl_cterm2rgb_color(c_int::from(c1));
    let rgb2 = rs_hl_cterm2rgb_color(c_int::from(c2));
    let rgb_blended = rs_rgb_blend(ratio, rgb1, rgb2);
    rs_hl_rgb2cterm_color(rgb_blended)
}

// ============================================================================
// Color Name Lookup
// ============================================================================

/// Color names for terminal colors (28 entries)
const COLOR_NAMES: [&str; 28] = [
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

/// Color numbers for 16-color terminals
const COLOR_NUMBERS_16: [c_int; 28] = [
    0, 1, 2, 3, 4, 5, 6, 6, 7, 7, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 15, -1,
];

/// Color numbers for 8-color terminals
const COLOR_NUMBERS_8: [c_int; 28] = [
    0, 4, 2, 6, 1, 5, 3, 3, 7, 7, 7, 7, 8, 8, 12, 12, 10, 10, 14, 14, 9, 9, 13, 13, 11, 11, 15, -1,
];

/// Color numbers for xterm with 88 colors
const COLOR_NUMBERS_88: [c_int; 28] = [
    0, 4, 2, 6, 1, 5, 32, 72, 84, 84, 7, 7, 82, 82, 12, 43, 10, 61, 14, 63, 9, 74, 13, 75, 11, 78,
    15, -1,
];

/// Color numbers for xterm with 256 colors
const COLOR_NUMBERS_256: [c_int; 28] = [
    0, 4, 2, 6, 1, 5, 130, 3, 248, 248, 7, 7, 242, 242, 12, 81, 10, 121, 14, 159, 9, 224, 13, 225,
    11, 229, 15, -1,
];

/// Lookup the "cterm" value for a color index based on terminal color count.
fn lookup_color(idx: usize, t_colors: c_int) -> c_int {
    // Use the _16 table to check if it's a valid color name.
    let color = COLOR_NUMBERS_16[idx];
    if color < 0 {
        return -1;
    }

    // Select appropriate color table based on terminal color count
    if t_colors == 8 {
        COLOR_NUMBERS_8[idx] & 7 // truncate to 8 colors
    } else if t_colors == 16 {
        COLOR_NUMBERS_8[idx]
    } else if t_colors == 88 {
        COLOR_NUMBERS_88[idx]
    } else if t_colors >= 256 {
        COLOR_NUMBERS_256[idx]
    } else {
        color
    }
}

/// Case-insensitive comparison for ASCII strings
fn str_icmp(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .all(|(c1, c2)| c1.eq_ignore_ascii_case(&c2))
}

/// Convert a color name to its cterm color number.
///
/// # Arguments
/// * `name` - Color name (null-terminated C string)
///
/// # Returns
/// cterm color number, or -1 if not found
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_name_to_ctermcolor(name: *const c_char) -> c_int {
    if name.is_null() {
        return -1;
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    if name_str.is_empty() {
        return -1;
    }

    // Find matching color name (case-insensitive)
    for (idx, color_name) in COLOR_NAMES.iter().enumerate().rev() {
        if str_icmp(name_str, color_name) {
            let t_colors = unsafe { nvim_get_t_colors() };
            return lookup_color(idx, t_colors);
        }
    }

    -1
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_blend_full_first() {
        // 100% of rgb1
        assert_eq!(rs_rgb_blend(100, 0xFF0000, 0x00FF00), 0xFF0000);
    }

    #[test]
    fn test_rgb_blend_full_second() {
        // 0% of rgb1 = 100% of rgb2
        assert_eq!(rs_rgb_blend(0, 0xFF0000, 0x00FF00), 0x00FF00);
    }

    #[test]
    fn test_rgb_blend_half() {
        // 50% blend of red and green
        let result = rs_rgb_blend(50, 0xFF0000, 0x00FF00);
        let r = (result >> 16) & 0xFF;
        let g = (result >> 8) & 0xFF;
        let b = result & 0xFF;
        // Should be roughly half of each
        assert_eq!(r, 127); // 255 * 0.5 = 127
        assert_eq!(g, 127); // 255 * 0.5 = 127
        assert_eq!(b, 0);
    }

    #[test]
    fn test_cterm2rgb_ansi_black() {
        assert_eq!(rs_hl_cterm2rgb_color(0), 0x000000);
    }

    #[test]
    fn test_cterm2rgb_ansi_red() {
        assert_eq!(rs_hl_cterm2rgb_color(1), 0xE00000);
    }

    #[test]
    fn test_cterm2rgb_ansi_white() {
        assert_eq!(rs_hl_cterm2rgb_color(15), 0xFFFFFF);
    }

    #[test]
    fn test_cterm2rgb_cube_start() {
        // Color 16 is the first color cube entry (0x00, 0x00, 0x00)
        assert_eq!(rs_hl_cterm2rgb_color(16), 0x000000);
    }

    #[test]
    fn test_cterm2rgb_cube_red() {
        // Color 196 is pure red in the cube (5*36 + 16 = 196)
        assert_eq!(rs_hl_cterm2rgb_color(196), 0xFF0000);
    }

    #[test]
    fn test_cterm2rgb_grey_ramp() {
        // Color 232 is first grey (0x08)
        assert_eq!(rs_hl_cterm2rgb_color(232), 0x080808);
        // Color 255 is last grey (0xEE)
        assert_eq!(rs_hl_cterm2rgb_color(255), 0xEEEEEE);
    }

    #[test]
    fn test_rgb2cterm_black() {
        // Pure black should map to color cube entry 16
        assert_eq!(rs_hl_rgb2cterm_color(0x000000), 16);
    }

    #[test]
    fn test_rgb2cterm_white() {
        // Pure white should map to highest color cube entry
        // 5*36 + 5*6 + 5 + 16 = 231
        assert_eq!(rs_hl_rgb2cterm_color(0xFFFFFF), 231);
    }

    #[test]
    fn test_cterm_blend() {
        // Blending same color should return same color
        let c = rs_cterm_blend(50, 196, 196);
        // Result might not be exactly 196 due to conversion losses
        // but should be close (pure red area)
        assert!(c >= 190 && c <= 200);
    }

    // Tests for color name lookup (unit tests that don't depend on C)
    #[test]
    fn test_str_icmp_equal() {
        assert!(str_icmp("Black", "black"));
        assert!(str_icmp("BLACK", "black"));
        assert!(str_icmp("DarkBlue", "DARKBLUE"));
    }

    #[test]
    fn test_str_icmp_not_equal() {
        assert!(!str_icmp("Black", "White"));
        assert!(!str_icmp("Black", "Blac"));
        assert!(!str_icmp("Black", "Blackx"));
    }

    #[test]
    fn test_lookup_color_256() {
        // Black at index 0 should be 0 for 256 colors
        assert_eq!(lookup_color(0, 256), 0);
        // Blue at index 14 should be 12 for 256 colors
        assert_eq!(lookup_color(14, 256), 12);
        // NONE at index 27 should be -1
        assert_eq!(lookup_color(27, 256), -1);
    }

    #[test]
    fn test_lookup_color_16() {
        // Black at index 0 should be 0 for 16 colors
        assert_eq!(lookup_color(0, 16), 0);
        // DarkBlue at index 1 should be 4 for 16 colors (from _8 table)
        assert_eq!(lookup_color(1, 16), 4);
    }

    #[test]
    fn test_lookup_color_8() {
        // Black at index 0 should be 0 for 8 colors
        assert_eq!(lookup_color(0, 8), 0);
        // DarkGray at index 12 should be 0 (8 & 7 = 0) for 8 colors
        assert_eq!(lookup_color(12, 8), 0);
    }
}
