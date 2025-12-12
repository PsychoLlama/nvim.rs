//! Highlight and color manipulation functions for Neovim
//!
//! This crate provides color blending and conversion functions used by the
//! highlight system.

use std::ffi::{c_char, c_int, CStr};

extern "C" {
    /// Get the terminal color count from C globals
    fn nvim_get_t_colors() -> c_int;
    /// Get the normal foreground color
    fn nvim_get_normal_fg() -> c_int;
    /// Get the normal background color
    fn nvim_get_normal_bg() -> c_int;
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
// RGB Color Name Table (from rgb.txt)
// ============================================================================

/// Special color index values
const COLOR_IDX_NONE: c_int = -1;
const COLOR_IDX_HEX: c_int = -2;
const COLOR_IDX_FG: c_int = -3;
const COLOR_IDX_BG: c_int = -4;

/// RGB color name table - 707 entries from rgb.txt
/// Names are sorted alphabetically for binary search
const RGB_COLOR_NAME_TABLE: &[(&str, c_int)] = &[
    ("AliceBlue", 0xf0f8ff),
    ("AntiqueWhite", 0xfaebd7),
    ("AntiqueWhite1", 0xffefdb),
    ("AntiqueWhite2", 0xeedfcc),
    ("AntiqueWhite3", 0xcdc0b0),
    ("AntiqueWhite4", 0x8b8378),
    ("Aqua", 0x00ffff),
    ("Aquamarine", 0x7fffd4),
    ("Aquamarine1", 0x7fffd4),
    ("Aquamarine2", 0x76eec6),
    ("Aquamarine3", 0x66cdaa),
    ("Aquamarine4", 0x458b74),
    ("Azure", 0xf0ffff),
    ("Azure1", 0xf0ffff),
    ("Azure2", 0xe0eeee),
    ("Azure3", 0xc1cdcd),
    ("Azure4", 0x838b8b),
    ("Beige", 0xf5f5dc),
    ("Bisque", 0xffe4c4),
    ("Bisque1", 0xffe4c4),
    ("Bisque2", 0xeed5b7),
    ("Bisque3", 0xcdb79e),
    ("Bisque4", 0x8b7d6b),
    ("Black", 0x000000),
    ("BlanchedAlmond", 0xffebcd),
    ("Blue", 0x0000ff),
    ("Blue1", 0x0000ff),
    ("Blue2", 0x0000ee),
    ("Blue3", 0x0000cd),
    ("Blue4", 0x00008b),
    ("BlueViolet", 0x8a2be2),
    ("Brown", 0xa52a2a),
    ("Brown1", 0xff4040),
    ("Brown2", 0xee3b3b),
    ("Brown3", 0xcd3333),
    ("Brown4", 0x8b2323),
    ("BurlyWood", 0xdeb887),
    ("Burlywood1", 0xffd39b),
    ("Burlywood2", 0xeec591),
    ("Burlywood3", 0xcdaa7d),
    ("Burlywood4", 0x8b7355),
    ("CadetBlue", 0x5f9ea0),
    ("CadetBlue1", 0x98f5ff),
    ("CadetBlue2", 0x8ee5ee),
    ("CadetBlue3", 0x7ac5cd),
    ("CadetBlue4", 0x53868b),
    ("ChartReuse", 0x7fff00),
    ("Chartreuse1", 0x7fff00),
    ("Chartreuse2", 0x76ee00),
    ("Chartreuse3", 0x66cd00),
    ("Chartreuse4", 0x458b00),
    ("Chocolate", 0xd2691e),
    ("Chocolate1", 0xff7f24),
    ("Chocolate2", 0xee7621),
    ("Chocolate3", 0xcd661d),
    ("Chocolate4", 0x8b4513),
    ("Coral", 0xff7f50),
    ("Coral1", 0xff7256),
    ("Coral2", 0xee6a50),
    ("Coral3", 0xcd5b45),
    ("Coral4", 0x8b3e2f),
    ("CornFlowerBlue", 0x6495ed),
    ("Cornsilk", 0xfff8dc),
    ("Cornsilk1", 0xfff8dc),
    ("Cornsilk2", 0xeee8cd),
    ("Cornsilk3", 0xcdc8b1),
    ("Cornsilk4", 0x8b8878),
    ("Crimson", 0xdc143c),
    ("Cyan", 0x00ffff),
    ("Cyan1", 0x00ffff),
    ("Cyan2", 0x00eeee),
    ("Cyan3", 0x00cdcd),
    ("Cyan4", 0x008b8b),
    ("DarkBlue", 0x00008b),
    ("DarkCyan", 0x008b8b),
    ("DarkGoldenrod", 0xb8860b),
    ("DarkGoldenrod1", 0xffb90f),
    ("DarkGoldenrod2", 0xeead0e),
    ("DarkGoldenrod3", 0xcd950c),
    ("DarkGoldenrod4", 0x8b6508),
    ("DarkGray", 0xa9a9a9),
    ("DarkGreen", 0x006400),
    ("DarkGrey", 0xa9a9a9),
    ("DarkKhaki", 0xbdb76b),
    ("DarkMagenta", 0x8b008b),
    ("DarkOliveGreen", 0x556b2f),
    ("DarkOliveGreen1", 0xcaff70),
    ("DarkOliveGreen2", 0xbcee68),
    ("DarkOliveGreen3", 0xa2cd5a),
    ("DarkOliveGreen4", 0x6e8b3d),
    ("DarkOrange", 0xff8c00),
    ("DarkOrange1", 0xff7f00),
    ("DarkOrange2", 0xee7600),
    ("DarkOrange3", 0xcd6600),
    ("DarkOrange4", 0x8b4500),
    ("DarkOrchid", 0x9932cc),
    ("DarkOrchid1", 0xbf3eff),
    ("DarkOrchid2", 0xb23aee),
    ("DarkOrchid3", 0x9a32cd),
    ("DarkOrchid4", 0x68228b),
    ("DarkRed", 0x8b0000),
    ("DarkSalmon", 0xe9967a),
    ("DarkSeaGreen", 0x8fbc8f),
    ("DarkSeaGreen1", 0xc1ffc1),
    ("DarkSeaGreen2", 0xb4eeb4),
    ("DarkSeaGreen3", 0x9bcd9b),
    ("DarkSeaGreen4", 0x698b69),
    ("DarkSlateBlue", 0x483d8b),
    ("DarkSlateGray", 0x2f4f4f),
    ("DarkSlateGray1", 0x97ffff),
    ("DarkSlateGray2", 0x8deeee),
    ("DarkSlateGray3", 0x79cdcd),
    ("DarkSlateGray4", 0x528b8b),
    ("DarkSlateGrey", 0x2f4f4f),
    ("DarkTurquoise", 0x00ced1),
    ("DarkViolet", 0x9400d3),
    ("DarkYellow", 0xbbbb00),
    ("DeepPink", 0xff1493),
    ("DeepPink1", 0xff1493),
    ("DeepPink2", 0xee1289),
    ("DeepPink3", 0xcd1076),
    ("DeepPink4", 0x8b0a50),
    ("DeepSkyBlue", 0x00bfff),
    ("DeepSkyBlue1", 0x00bfff),
    ("DeepSkyBlue2", 0x00b2ee),
    ("DeepSkyBlue3", 0x009acd),
    ("DeepSkyBlue4", 0x00688b),
    ("DimGray", 0x696969),
    ("DimGrey", 0x696969),
    ("DodgerBlue", 0x1e90ff),
    ("DodgerBlue1", 0x1e90ff),
    ("DodgerBlue2", 0x1c86ee),
    ("DodgerBlue3", 0x1874cd),
    ("DodgerBlue4", 0x104e8b),
    ("Firebrick", 0xb22222),
    ("Firebrick1", 0xff3030),
    ("Firebrick2", 0xee2c2c),
    ("Firebrick3", 0xcd2626),
    ("Firebrick4", 0x8b1a1a),
    ("FloralWhite", 0xfffaf0),
    ("ForestGreen", 0x228b22),
    ("Fuchsia", 0xff00ff),
    ("Gainsboro", 0xdcdcdc),
    ("GhostWhite", 0xf8f8ff),
    ("Gold", 0xffd700),
    ("Gold1", 0xffd700),
    ("Gold2", 0xeec900),
    ("Gold3", 0xcdad00),
    ("Gold4", 0x8b7500),
    ("Goldenrod", 0xdaa520),
    ("Goldenrod1", 0xffc125),
    ("Goldenrod2", 0xeeb422),
    ("Goldenrod3", 0xcd9b1d),
    ("Goldenrod4", 0x8b6914),
    ("Gray", 0x808080),
    ("Gray0", 0x000000),
    ("Gray1", 0x030303),
    ("Gray10", 0x1a1a1a),
    ("Gray100", 0xffffff),
    ("Gray11", 0x1c1c1c),
    ("Gray12", 0x1f1f1f),
    ("Gray13", 0x212121),
    ("Gray14", 0x242424),
    ("Gray15", 0x262626),
    ("Gray16", 0x292929),
    ("Gray17", 0x2b2b2b),
    ("Gray18", 0x2e2e2e),
    ("Gray19", 0x303030),
    ("Gray2", 0x050505),
    ("Gray20", 0x333333),
    ("Gray21", 0x363636),
    ("Gray22", 0x383838),
    ("Gray23", 0x3b3b3b),
    ("Gray24", 0x3d3d3d),
    ("Gray25", 0x404040),
    ("Gray26", 0x424242),
    ("Gray27", 0x454545),
    ("Gray28", 0x474747),
    ("Gray29", 0x4a4a4a),
    ("Gray3", 0x080808),
    ("Gray30", 0x4d4d4d),
    ("Gray31", 0x4f4f4f),
    ("Gray32", 0x525252),
    ("Gray33", 0x545454),
    ("Gray34", 0x575757),
    ("Gray35", 0x595959),
    ("Gray36", 0x5c5c5c),
    ("Gray37", 0x5e5e5e),
    ("Gray38", 0x616161),
    ("Gray39", 0x636363),
    ("Gray4", 0x0a0a0a),
    ("Gray40", 0x666666),
    ("Gray41", 0x696969),
    ("Gray42", 0x6b6b6b),
    ("Gray43", 0x6e6e6e),
    ("Gray44", 0x707070),
    ("Gray45", 0x737373),
    ("Gray46", 0x757575),
    ("Gray47", 0x787878),
    ("Gray48", 0x7a7a7a),
    ("Gray49", 0x7d7d7d),
    ("Gray5", 0x0d0d0d),
    ("Gray50", 0x7f7f7f),
    ("Gray51", 0x828282),
    ("Gray52", 0x858585),
    ("Gray53", 0x878787),
    ("Gray54", 0x8a8a8a),
    ("Gray55", 0x8c8c8c),
    ("Gray56", 0x8f8f8f),
    ("Gray57", 0x919191),
    ("Gray58", 0x949494),
    ("Gray59", 0x969696),
    ("Gray6", 0x0f0f0f),
    ("Gray60", 0x999999),
    ("Gray61", 0x9c9c9c),
    ("Gray62", 0x9e9e9e),
    ("Gray63", 0xa1a1a1),
    ("Gray64", 0xa3a3a3),
    ("Gray65", 0xa6a6a6),
    ("Gray66", 0xa8a8a8),
    ("Gray67", 0xababab),
    ("Gray68", 0xadadad),
    ("Gray69", 0xb0b0b0),
    ("Gray7", 0x121212),
    ("Gray70", 0xb3b3b3),
    ("Gray71", 0xb5b5b5),
    ("Gray72", 0xb8b8b8),
    ("Gray73", 0xbababa),
    ("Gray74", 0xbdbdbd),
    ("Gray75", 0xbfbfbf),
    ("Gray76", 0xc2c2c2),
    ("Gray77", 0xc4c4c4),
    ("Gray78", 0xc7c7c7),
    ("Gray79", 0xc9c9c9),
    ("Gray8", 0x141414),
    ("Gray80", 0xcccccc),
    ("Gray81", 0xcfcfcf),
    ("Gray82", 0xd1d1d1),
    ("Gray83", 0xd4d4d4),
    ("Gray84", 0xd6d6d6),
    ("Gray85", 0xd9d9d9),
    ("Gray86", 0xdbdbdb),
    ("Gray87", 0xdedede),
    ("Gray88", 0xe0e0e0),
    ("Gray89", 0xe3e3e3),
    ("Gray9", 0x171717),
    ("Gray90", 0xe5e5e5),
    ("Gray91", 0xe8e8e8),
    ("Gray92", 0xebebeb),
    ("Gray93", 0xededed),
    ("Gray94", 0xf0f0f0),
    ("Gray95", 0xf2f2f2),
    ("Gray96", 0xf5f5f5),
    ("Gray97", 0xf7f7f7),
    ("Gray98", 0xfafafa),
    ("Gray99", 0xfcfcfc),
    ("Green", 0x008000),
    ("Green1", 0x00ff00),
    ("Green2", 0x00ee00),
    ("Green3", 0x00cd00),
    ("Green4", 0x008b00),
    ("GreenYellow", 0xadff2f),
    ("Grey", 0x808080),
    ("Grey0", 0x000000),
    ("Grey1", 0x030303),
    ("Grey10", 0x1a1a1a),
    ("Grey100", 0xffffff),
    ("Grey11", 0x1c1c1c),
    ("Grey12", 0x1f1f1f),
    ("Grey13", 0x212121),
    ("Grey14", 0x242424),
    ("Grey15", 0x262626),
    ("Grey16", 0x292929),
    ("Grey17", 0x2b2b2b),
    ("Grey18", 0x2e2e2e),
    ("Grey19", 0x303030),
    ("Grey2", 0x050505),
    ("Grey20", 0x333333),
    ("Grey21", 0x363636),
    ("Grey22", 0x383838),
    ("Grey23", 0x3b3b3b),
    ("Grey24", 0x3d3d3d),
    ("Grey25", 0x404040),
    ("Grey26", 0x424242),
    ("Grey27", 0x454545),
    ("Grey28", 0x474747),
    ("Grey29", 0x4a4a4a),
    ("Grey3", 0x080808),
    ("Grey30", 0x4d4d4d),
    ("Grey31", 0x4f4f4f),
    ("Grey32", 0x525252),
    ("Grey33", 0x545454),
    ("Grey34", 0x575757),
    ("Grey35", 0x595959),
    ("Grey36", 0x5c5c5c),
    ("Grey37", 0x5e5e5e),
    ("Grey38", 0x616161),
    ("Grey39", 0x636363),
    ("Grey4", 0x0a0a0a),
    ("Grey40", 0x666666),
    ("Grey41", 0x696969),
    ("Grey42", 0x6b6b6b),
    ("Grey43", 0x6e6e6e),
    ("Grey44", 0x707070),
    ("Grey45", 0x737373),
    ("Grey46", 0x757575),
    ("Grey47", 0x787878),
    ("Grey48", 0x7a7a7a),
    ("Grey49", 0x7d7d7d),
    ("Grey5", 0x0d0d0d),
    ("Grey50", 0x7f7f7f),
    ("Grey51", 0x828282),
    ("Grey52", 0x858585),
    ("Grey53", 0x878787),
    ("Grey54", 0x8a8a8a),
    ("Grey55", 0x8c8c8c),
    ("Grey56", 0x8f8f8f),
    ("Grey57", 0x919191),
    ("Grey58", 0x949494),
    ("Grey59", 0x969696),
    ("Grey6", 0x0f0f0f),
    ("Grey60", 0x999999),
    ("Grey61", 0x9c9c9c),
    ("Grey62", 0x9e9e9e),
    ("Grey63", 0xa1a1a1),
    ("Grey64", 0xa3a3a3),
    ("Grey65", 0xa6a6a6),
    ("Grey66", 0xa8a8a8),
    ("Grey67", 0xababab),
    ("Grey68", 0xadadad),
    ("Grey69", 0xb0b0b0),
    ("Grey7", 0x121212),
    ("Grey70", 0xb3b3b3),
    ("Grey71", 0xb5b5b5),
    ("Grey72", 0xb8b8b8),
    ("Grey73", 0xbababa),
    ("Grey74", 0xbdbdbd),
    ("Grey75", 0xbfbfbf),
    ("Grey76", 0xc2c2c2),
    ("Grey77", 0xc4c4c4),
    ("Grey78", 0xc7c7c7),
    ("Grey79", 0xc9c9c9),
    ("Grey8", 0x141414),
    ("Grey80", 0xcccccc),
    ("Grey81", 0xcfcfcf),
    ("Grey82", 0xd1d1d1),
    ("Grey83", 0xd4d4d4),
    ("Grey84", 0xd6d6d6),
    ("Grey85", 0xd9d9d9),
    ("Grey86", 0xdbdbdb),
    ("Grey87", 0xdedede),
    ("Grey88", 0xe0e0e0),
    ("Grey89", 0xe3e3e3),
    ("Grey9", 0x171717),
    ("Grey90", 0xe5e5e5),
    ("Grey91", 0xe8e8e8),
    ("Grey92", 0xebebeb),
    ("Grey93", 0xededed),
    ("Grey94", 0xf0f0f0),
    ("Grey95", 0xf2f2f2),
    ("Grey96", 0xf5f5f5),
    ("Grey97", 0xf7f7f7),
    ("Grey98", 0xfafafa),
    ("Grey99", 0xfcfcfc),
    ("Honeydew", 0xf0fff0),
    ("Honeydew1", 0xf0fff0),
    ("Honeydew2", 0xe0eee0),
    ("Honeydew3", 0xc1cdc1),
    ("Honeydew4", 0x838b83),
    ("HotPink", 0xff69b4),
    ("HotPink1", 0xff6eb4),
    ("HotPink2", 0xee6aa7),
    ("HotPink3", 0xcd6090),
    ("HotPink4", 0x8b3a62),
    ("IndianRed", 0xcd5c5c),
    ("IndianRed1", 0xff6a6a),
    ("IndianRed2", 0xee6363),
    ("IndianRed3", 0xcd5555),
    ("IndianRed4", 0x8b3a3a),
    ("Indigo", 0x4b0082),
    ("Ivory", 0xfffff0),
    ("Ivory1", 0xfffff0),
    ("Ivory2", 0xeeeee0),
    ("Ivory3", 0xcdcdc1),
    ("Ivory4", 0x8b8b83),
    ("Khaki", 0xf0e68c),
    ("Khaki1", 0xfff68f),
    ("Khaki2", 0xeee685),
    ("Khaki3", 0xcdc673),
    ("Khaki4", 0x8b864e),
    ("Lavender", 0xe6e6fa),
    ("LavenderBlush", 0xfff0f5),
    ("LavenderBlush1", 0xfff0f5),
    ("LavenderBlush2", 0xeee0e5),
    ("LavenderBlush3", 0xcdc1c5),
    ("LavenderBlush4", 0x8b8386),
    ("LawnGreen", 0x7cfc00),
    ("LemonChiffon", 0xfffacd),
    ("LemonChiffon1", 0xfffacd),
    ("LemonChiffon2", 0xeee9bf),
    ("LemonChiffon3", 0xcdc9a5),
    ("LemonChiffon4", 0x8b8970),
    ("LightBlue", 0xadd8e6),
    ("LightBlue1", 0xbfefff),
    ("LightBlue2", 0xb2dfee),
    ("LightBlue3", 0x9ac0cd),
    ("LightBlue4", 0x68838b),
    ("LightCoral", 0xf08080),
    ("LightCyan", 0xe0ffff),
    ("LightCyan1", 0xe0ffff),
    ("LightCyan2", 0xd1eeee),
    ("LightCyan3", 0xb4cdcd),
    ("LightCyan4", 0x7a8b8b),
    ("LightGoldenrod", 0xeedd82),
    ("LightGoldenrod1", 0xffec8b),
    ("LightGoldenrod2", 0xeedc82),
    ("LightGoldenrod3", 0xcdbe70),
    ("LightGoldenrod4", 0x8b814c),
    ("LightGoldenrodYellow", 0xfafad2),
    ("LightGray", 0xd3d3d3),
    ("LightGreen", 0x90ee90),
    ("LightGrey", 0xd3d3d3),
    ("LightMagenta", 0xffbbff),
    ("LightPink", 0xffb6c1),
    ("LightPink1", 0xffaeb9),
    ("LightPink2", 0xeea2ad),
    ("LightPink3", 0xcd8c95),
    ("LightPink4", 0x8b5f65),
    ("LightRed", 0xffbbbb),
    ("LightSalmon", 0xffa07a),
    ("LightSalmon1", 0xffa07a),
    ("LightSalmon2", 0xee9572),
    ("LightSalmon3", 0xcd8162),
    ("LightSalmon4", 0x8b5742),
    ("LightSeaGreen", 0x20b2aa),
    ("LightSkyBlue", 0x87cefa),
    ("LightSkyBlue1", 0xb0e2ff),
    ("LightSkyBlue2", 0xa4d3ee),
    ("LightSkyBlue3", 0x8db6cd),
    ("LightSkyBlue4", 0x607b8b),
    ("LightSlateBlue", 0x8470ff),
    ("LightSlateGray", 0x778899),
    ("LightSlateGrey", 0x778899),
    ("LightSteelBlue", 0xb0c4de),
    ("LightSteelBlue1", 0xcae1ff),
    ("LightSteelBlue2", 0xbcd2ee),
    ("LightSteelBlue3", 0xa2b5cd),
    ("LightSteelBlue4", 0x6e7b8b),
    ("LightYellow", 0xffffe0),
    ("LightYellow1", 0xffffe0),
    ("LightYellow2", 0xeeeed1),
    ("LightYellow3", 0xcdcdb4),
    ("LightYellow4", 0x8b8b7a),
    ("Lime", 0x00ff00),
    ("LimeGreen", 0x32cd32),
    ("Linen", 0xfaf0e6),
    ("Magenta", 0xff00ff),
    ("Magenta1", 0xff00ff),
    ("Magenta2", 0xee00ee),
    ("Magenta3", 0xcd00cd),
    ("Magenta4", 0x8b008b),
    ("Maroon", 0x800000),
    ("Maroon1", 0xff34b3),
    ("Maroon2", 0xee30a7),
    ("Maroon3", 0xcd2990),
    ("Maroon4", 0x8b1c62),
    ("MediumAquamarine", 0x66cdaa),
    ("MediumBlue", 0x0000cd),
    ("MediumOrchid", 0xba55d3),
    ("MediumOrchid1", 0xe066ff),
    ("MediumOrchid2", 0xd15fee),
    ("MediumOrchid3", 0xb452cd),
    ("MediumOrchid4", 0x7a378b),
    ("MediumPurple", 0x9370db),
    ("MediumPurple1", 0xab82ff),
    ("MediumPurple2", 0x9f79ee),
    ("MediumPurple3", 0x8968cd),
    ("MediumPurple4", 0x5d478b),
    ("MediumSeaGreen", 0x3cb371),
    ("MediumSlateBlue", 0x7b68ee),
    ("MediumSpringGreen", 0x00fa9a),
    ("MediumTurquoise", 0x48d1cc),
    ("MediumVioletRed", 0xc71585),
    ("MidnightBlue", 0x191970),
    ("MintCream", 0xf5fffa),
    ("MistyRose", 0xffe4e1),
    ("MistyRose1", 0xffe4e1),
    ("MistyRose2", 0xeed5d2),
    ("MistyRose3", 0xcdb7b5),
    ("MistyRose4", 0x8b7d7b),
    ("Moccasin", 0xffe4b5),
    ("NavajoWhite", 0xffdead),
    ("NavajoWhite1", 0xffdead),
    ("NavajoWhite2", 0xeecfa1),
    ("NavajoWhite3", 0xcdb38b),
    ("NavajoWhite4", 0x8b795e),
    ("Navy", 0x000080),
    ("NavyBlue", 0x000080),
    ("NvimDarkBlue", 0x004c73),
    ("NvimDarkCyan", 0x007373),
    ("NvimDarkGray1", 0x07080d),
    ("NvimDarkGray2", 0x14161b),
    ("NvimDarkGray3", 0x2c2e33),
    ("NvimDarkGray4", 0x4f5258),
    ("NvimDarkGreen", 0x005523),
    ("NvimDarkGrey1", 0x07080d),
    ("NvimDarkGrey2", 0x14161b),
    ("NvimDarkGrey3", 0x2c2e33),
    ("NvimDarkGrey4", 0x4f5258),
    ("NvimDarkMagenta", 0x470045),
    ("NvimDarkRed", 0x590008),
    ("NvimDarkYellow", 0x6b5300),
    ("NvimLightBlue", 0xa6dbff),
    ("NvimLightCyan", 0x8cf8f7),
    ("NvimLightGray1", 0xeef1f8),
    ("NvimLightGray2", 0xe0e2ea),
    ("NvimLightGray3", 0xc4c6cd),
    ("NvimLightGray4", 0x9b9ea4),
    ("NvimLightGreen", 0xb3f6c0),
    ("NvimLightGrey1", 0xeef1f8),
    ("NvimLightGrey2", 0xe0e2ea),
    ("NvimLightGrey3", 0xc4c6cd),
    ("NvimLightGrey4", 0x9b9ea4),
    ("NvimLightMagenta", 0xffcaff),
    ("NvimLightRed", 0xffc0b9),
    ("NvimLightYellow", 0xfce094),
    ("OldLace", 0xfdf5e6),
    ("Olive", 0x808000),
    ("OliveDrab", 0x6b8e23),
    ("OliveDrab1", 0xc0ff3e),
    ("OliveDrab2", 0xb3ee3a),
    ("OliveDrab3", 0x9acd32),
    ("OliveDrab4", 0x698b22),
    ("Orange", 0xffa500),
    ("Orange1", 0xffa500),
    ("Orange2", 0xee9a00),
    ("Orange3", 0xcd8500),
    ("Orange4", 0x8b5a00),
    ("OrangeRed", 0xff4500),
    ("OrangeRed1", 0xff4500),
    ("OrangeRed2", 0xee4000),
    ("OrangeRed3", 0xcd3700),
    ("OrangeRed4", 0x8b2500),
    ("Orchid", 0xda70d6),
    ("Orchid1", 0xff83fa),
    ("Orchid2", 0xee7ae9),
    ("Orchid3", 0xcd69c9),
    ("Orchid4", 0x8b4789),
    ("PaleGoldenrod", 0xeee8aa),
    ("PaleGreen", 0x98fb98),
    ("PaleGreen1", 0x9aff9a),
    ("PaleGreen2", 0x90ee90),
    ("PaleGreen3", 0x7ccd7c),
    ("PaleGreen4", 0x548b54),
    ("PaleTurquoise", 0xafeeee),
    ("PaleTurquoise1", 0xbbffff),
    ("PaleTurquoise2", 0xaeeeee),
    ("PaleTurquoise3", 0x96cdcd),
    ("PaleTurquoise4", 0x668b8b),
    ("PaleVioletRed", 0xdb7093),
    ("PaleVioletRed1", 0xff82ab),
    ("PaleVioletRed2", 0xee799f),
    ("PaleVioletRed3", 0xcd6889),
    ("PaleVioletRed4", 0x8b475d),
    ("PapayaWhip", 0xffefd5),
    ("PeachPuff", 0xffdab9),
    ("PeachPuff1", 0xffdab9),
    ("PeachPuff2", 0xeecbad),
    ("PeachPuff3", 0xcdaf95),
    ("PeachPuff4", 0x8b7765),
    ("Peru", 0xcd853f),
    ("Pink", 0xffc0cb),
    ("Pink1", 0xffb5c5),
    ("Pink2", 0xeea9b8),
    ("Pink3", 0xcd919e),
    ("Pink4", 0x8b636c),
    ("Plum", 0xdda0dd),
    ("Plum1", 0xffbbff),
    ("Plum2", 0xeeaeee),
    ("Plum3", 0xcd96cd),
    ("Plum4", 0x8b668b),
    ("PowderBlue", 0xb0e0e6),
    ("Purple", 0x800080),
    ("Purple1", 0x9b30ff),
    ("Purple2", 0x912cee),
    ("Purple3", 0x7d26cd),
    ("Purple4", 0x551a8b),
    ("RebeccaPurple", 0x663399),
    ("Red", 0xff0000),
    ("Red1", 0xff0000),
    ("Red2", 0xee0000),
    ("Red3", 0xcd0000),
    ("Red4", 0x8b0000),
    ("RosyBrown", 0xbc8f8f),
    ("RosyBrown1", 0xffc1c1),
    ("RosyBrown2", 0xeeb4b4),
    ("RosyBrown3", 0xcd9b9b),
    ("RosyBrown4", 0x8b6969),
    ("RoyalBlue", 0x4169e1),
    ("RoyalBlue1", 0x4876ff),
    ("RoyalBlue2", 0x436eee),
    ("RoyalBlue3", 0x3a5fcd),
    ("RoyalBlue4", 0x27408b),
    ("SaddleBrown", 0x8b4513),
    ("Salmon", 0xfa8072),
    ("Salmon1", 0xff8c69),
    ("Salmon2", 0xee8262),
    ("Salmon3", 0xcd7054),
    ("Salmon4", 0x8b4c39),
    ("SandyBrown", 0xf4a460),
    ("SeaGreen", 0x2e8b57),
    ("SeaGreen1", 0x54ff9f),
    ("SeaGreen2", 0x4eee94),
    ("SeaGreen3", 0x43cd80),
    ("SeaGreen4", 0x2e8b57),
    ("SeaShell", 0xfff5ee),
    ("Seashell1", 0xfff5ee),
    ("Seashell2", 0xeee5de),
    ("Seashell3", 0xcdc5bf),
    ("Seashell4", 0x8b8682),
    ("Sienna", 0xa0522d),
    ("Sienna1", 0xff8247),
    ("Sienna2", 0xee7942),
    ("Sienna3", 0xcd6839),
    ("Sienna4", 0x8b4726),
    ("Silver", 0xc0c0c0),
    ("SkyBlue", 0x87ceeb),
    ("SkyBlue1", 0x87ceff),
    ("SkyBlue2", 0x7ec0ee),
    ("SkyBlue3", 0x6ca6cd),
    ("SkyBlue4", 0x4a708b),
    ("SlateBlue", 0x6a5acd),
    ("SlateBlue1", 0x836fff),
    ("SlateBlue2", 0x7a67ee),
    ("SlateBlue3", 0x6959cd),
    ("SlateBlue4", 0x473c8b),
    ("SlateGray", 0x708090),
    ("SlateGray1", 0xc6e2ff),
    ("SlateGray2", 0xb9d3ee),
    ("SlateGray3", 0x9fb6cd),
    ("SlateGray4", 0x6c7b8b),
    ("SlateGrey", 0x708090),
    ("Snow", 0xfffafa),
    ("Snow1", 0xfffafa),
    ("Snow2", 0xeee9e9),
    ("Snow3", 0xcdc9c9),
    ("Snow4", 0x8b8989),
    ("SpringGreen", 0x00ff7f),
    ("SpringGreen1", 0x00ff7f),
    ("SpringGreen2", 0x00ee76),
    ("SpringGreen3", 0x00cd66),
    ("SpringGreen4", 0x008b45),
    ("SteelBlue", 0x4682b4),
    ("SteelBlue1", 0x63b8ff),
    ("SteelBlue2", 0x5cacee),
    ("SteelBlue3", 0x4f94cd),
    ("SteelBlue4", 0x36648b),
    ("Tan", 0xd2b48c),
    ("Tan1", 0xffa54f),
    ("Tan2", 0xee9a49),
    ("Tan3", 0xcd853f),
    ("Tan4", 0x8b5a2b),
    ("Teal", 0x008080),
    ("Thistle", 0xd8bfd8),
    ("Thistle1", 0xffe1ff),
    ("Thistle2", 0xeed2ee),
    ("Thistle3", 0xcdb5cd),
    ("Thistle4", 0x8b7b8b),
    ("Tomato", 0xff6347),
    ("Tomato1", 0xff6347),
    ("Tomato2", 0xee5c42),
    ("Tomato3", 0xcd4f39),
    ("Tomato4", 0x8b3626),
    ("Turquoise", 0x40e0d0),
    ("Turquoise1", 0x00f5ff),
    ("Turquoise2", 0x00e5ee),
    ("Turquoise3", 0x00c5cd),
    ("Turquoise4", 0x00868b),
    ("Violet", 0xee82ee),
    ("VioletRed", 0xd02090),
    ("VioletRed1", 0xff3e96),
    ("VioletRed2", 0xee3a8c),
    ("VioletRed3", 0xcd3278),
    ("VioletRed4", 0x8b2252),
    ("WebGray", 0x808080),
    ("WebGreen", 0x008000),
    ("WebGrey", 0x808080),
    ("WebMaroon", 0x800000),
    ("WebPurple", 0x800080),
    ("Wheat", 0xf5deb3),
    ("Wheat1", 0xffe7ba),
    ("Wheat2", 0xeed8ae),
    ("Wheat3", 0xcdba96),
    ("Wheat4", 0x8b7e66),
    ("White", 0xffffff),
    ("WhiteSmoke", 0xf5f5f5),
    ("X11Gray", 0xbebebe),
    ("X11Green", 0x00ff00),
    ("X11Grey", 0xbebebe),
    ("X11Maroon", 0xb03060),
    ("X11Purple", 0xa020f0),
    ("Yellow", 0xffff00),
    ("Yellow1", 0xffff00),
    ("Yellow2", 0xeeee00),
    ("Yellow3", 0xcdcd00),
    ("Yellow4", 0x8b8b00),
    ("YellowGreen", 0x9acd32),
];

/// Result type for name_to_color function
#[repr(C)]
pub struct NameToColorResult {
    /// RGB color value, or -1 if not found
    pub color: c_int,
    /// Index into color table, or special constant (COLOR_IDX_*)
    pub idx: c_int,
}

/// Check if a character is a hex digit
fn is_hex_digit(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

/// Binary search for a color name in the table (case-insensitive)
fn find_color_name(name: &str) -> Option<(usize, c_int)> {
    let mut lo = 0usize;
    let mut hi = RGB_COLOR_NAME_TABLE.len();

    while lo < hi {
        let mid = (lo + hi) / 2;
        let (table_name, color) = RGB_COLOR_NAME_TABLE[mid];
        match name.to_ascii_lowercase().cmp(&table_name.to_ascii_lowercase()) {
            std::cmp::Ordering::Less => hi = mid,
            std::cmp::Ordering::Greater => lo = mid + 1,
            std::cmp::Ordering::Equal => return Some((mid, color)),
        }
    }
    None
}

/// Convert a color name to its RGB value.
///
/// Handles:
/// - Hex strings like "#RRGGBB"
/// - Special names "fg", "foreground", "bg", "background"
/// - Named colors from rgb.txt (707 entries)
///
/// # Arguments
/// * `name` - Color name (null-terminated C string)
///
/// # Returns
/// NameToColorResult with color value and index
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_name_to_color(name: *const c_char) -> NameToColorResult {
    if name.is_null() {
        return NameToColorResult {
            color: -1,
            idx: COLOR_IDX_NONE,
        };
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_bytes = name_cstr.to_bytes();

    // Check for hex color string "#RRGGBB"
    if name_bytes.len() == 7
        && name_bytes[0] == b'#'
        && is_hex_digit(name_bytes[1])
        && is_hex_digit(name_bytes[2])
        && is_hex_digit(name_bytes[3])
        && is_hex_digit(name_bytes[4])
        && is_hex_digit(name_bytes[5])
        && is_hex_digit(name_bytes[6])
    {
        // Parse hex value
        let hex_str = unsafe { std::str::from_utf8_unchecked(&name_bytes[1..7]) };
        if let Ok(color) = i32::from_str_radix(hex_str, 16) {
            return NameToColorResult {
                color,
                idx: COLOR_IDX_HEX,
            };
        }
    }

    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            return NameToColorResult {
                color: -1,
                idx: COLOR_IDX_NONE,
            }
        }
    };

    // Check for "bg" or "background"
    if name_str.eq_ignore_ascii_case("bg") || name_str.eq_ignore_ascii_case("background") {
        let color = unsafe { nvim_get_normal_bg() };
        return NameToColorResult {
            color,
            idx: COLOR_IDX_BG,
        };
    }

    // Check for "fg" or "foreground"
    if name_str.eq_ignore_ascii_case("fg") || name_str.eq_ignore_ascii_case("foreground") {
        let color = unsafe { nvim_get_normal_fg() };
        return NameToColorResult {
            color,
            idx: COLOR_IDX_FG,
        };
    }

    // Binary search in color table
    if let Some((idx, color)) = find_color_name(name_str) {
        return NameToColorResult {
            color,
            idx: idx as c_int,
        };
    }

    NameToColorResult {
        color: -1,
        idx: COLOR_IDX_NONE,
    }
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
