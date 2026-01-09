//! Pen (text attributes) implementation for `VTerm`
//!
//! This module provides pen management for terminal emulation,
//! including text attributes (bold, italic, etc.) and color handling.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::trivially_copy_pass_by_ref)]

use std::ffi::{c_int, c_long};

use crate::{
    csi_arg, csi_arg_has_more, csi_arg_is_missing, VTermAttr, VTermColor, VTermUnderline,
    VTermValueType, VTERM_COLOR_DEFAULT_BG, VTERM_COLOR_DEFAULT_FG, VTERM_COLOR_DEFAULT_MASK,
    VTERM_COLOR_TYPE_MASK,
};

// =============================================================================
// Color Constants
// =============================================================================

/// RGB triple without metadata
#[derive(Clone, Copy, Debug)]
struct VTermRgb {
    red: u8,
    green: u8,
    blue: u8,
}

/// Standard ANSI colors (8 normal + 8 high-intensity)
const ANSI_COLORS: [VTermRgb; 16] = [
    // Normal colors
    VTermRgb {
        red: 0,
        green: 0,
        blue: 0,
    }, // black
    VTermRgb {
        red: 224,
        green: 0,
        blue: 0,
    }, // red
    VTermRgb {
        red: 0,
        green: 224,
        blue: 0,
    }, // green
    VTermRgb {
        red: 224,
        green: 224,
        blue: 0,
    }, // yellow
    VTermRgb {
        red: 0,
        green: 0,
        blue: 224,
    }, // blue
    VTermRgb {
        red: 224,
        green: 0,
        blue: 224,
    }, // magenta
    VTermRgb {
        red: 0,
        green: 224,
        blue: 224,
    }, // cyan
    VTermRgb {
        red: 224,
        green: 224,
        blue: 224,
    }, // white == light grey
    // High intensity
    VTermRgb {
        red: 128,
        green: 128,
        blue: 128,
    }, // black
    VTermRgb {
        red: 255,
        green: 64,
        blue: 64,
    }, // red
    VTermRgb {
        red: 64,
        green: 255,
        blue: 64,
    }, // green
    VTermRgb {
        red: 255,
        green: 255,
        blue: 64,
    }, // yellow
    VTermRgb {
        red: 64,
        green: 64,
        blue: 255,
    }, // blue
    VTermRgb {
        red: 255,
        green: 64,
        blue: 255,
    }, // magenta
    VTermRgb {
        red: 64,
        green: 255,
        blue: 255,
    }, // cyan
    VTermRgb {
        red: 255,
        green: 255,
        blue: 255,
    }, // white for real
];

/// 6-level ramp for 216-color cube
const RAMP6: [u8; 6] = [0x00, 0x33, 0x66, 0x99, 0xCC, 0xFF];

/// 24-level greyscale ramp
const RAMP24: [u8; 24] = [
    0x00, 0x0B, 0x16, 0x21, 0x2C, 0x37, 0x42, 0x4D, 0x58, 0x63, 0x6E, 0x79, 0x85, 0x90, 0x9B, 0xA6,
    0xB1, 0xBC, 0xC7, 0xD2, 0xDD, 0xE8, 0xF3, 0xFF,
];

// =============================================================================
// Color Lookup Functions
// =============================================================================

/// Look up a default ANSI color (0-15) and store it in `col`
fn lookup_default_colour_ansi(idx: usize, col: &mut VTermColor) {
    if idx < 16 {
        *col = VTermColor::rgb(
            ANSI_COLORS[idx].red,
            ANSI_COLORS[idx].green,
            ANSI_COLORS[idx].blue,
        );
    }
}

/// Look up a color from the 256-color palette
///
/// - 0-15: ANSI colors (from the state's color palette)
/// - 16-231: 6x6x6 color cube
/// - 232-255: 24-level greyscale
///
/// Returns `true` if the index was valid, `false` otherwise.
pub fn lookup_colour_palette(index: usize, col: &mut VTermColor) -> bool {
    if index < 16 {
        // Normal 8 colours or high intensity
        lookup_default_colour_ansi(index, col);
        true
    } else if index < 232 {
        // 216-colour cube
        let idx = index - 16;
        *col = VTermColor::rgb(RAMP6[(idx / 36) % 6], RAMP6[(idx / 6) % 6], RAMP6[idx % 6]);
        true
    } else if index < 256 {
        // 24 greyscales
        let idx = index - 232;
        let grey = RAMP24[idx];
        *col = VTermColor::rgb(grey, grey, grey);
        true
    } else {
        false
    }
}

/// Look up a color from CSI SGR extended color arguments
///
/// - palette 2: RGB mode (3 args: R, G, B)
/// - palette 5: Indexed 256-color mode (1 arg: index)
///
/// Returns the number of arguments consumed.
pub fn lookup_colour(palette: c_long, args: &[c_long], col: &mut VTermColor) -> usize {
    match palette {
        2 => {
            // RGB mode - 3 args contain colour values directly
            if args.len() < 3 {
                return args.len();
            }
            *col = VTermColor::rgb(
                csi_arg(args[0]) as u8,
                csi_arg(args[1]) as u8,
                csi_arg(args[2]) as u8,
            );
            3
        }
        5 => {
            // XTerm 256-colour mode
            if args.is_empty() || csi_arg_is_missing(args[0]) {
                return usize::from(!args.is_empty());
            }
            *col = VTermColor::indexed(csi_arg(args[0]) as u8);
            usize::from(!args.is_empty())
        }
        _ => 0,
    }
}

// =============================================================================
// Default Colors
// =============================================================================

/// Create the default foreground color (90% grey)
#[inline]
pub fn default_foreground() -> VTermColor {
    let mut col = VTermColor::rgb(240, 240, 240);
    // SAFETY: Setting type flags is safe
    unsafe {
        let t = col.get_type();
        col.set_type(t | VTERM_COLOR_DEFAULT_FG);
    }
    col
}

/// Create the default background color (black)
#[inline]
pub fn default_background() -> VTermColor {
    let mut col = VTermColor::rgb(0, 0, 0);
    // SAFETY: Setting type flags is safe
    unsafe {
        let t = col.get_type();
        col.set_type(t | VTERM_COLOR_DEFAULT_BG);
    }
    col
}

/// Initialize the 16-color ANSI palette with default colors
pub fn init_default_palette(colors: &mut [VTermColor; 16]) {
    for (idx, col) in colors.iter_mut().enumerate() {
        lookup_default_colour_ansi(idx, col);
    }
}

// =============================================================================
// Color Conversion
// =============================================================================

/// Convert an indexed color to RGB using the default palette
///
/// After this function, `col` will be an RGB color (if it was indexed).
/// Other flags stored in `col.type` will be reset.
pub fn convert_indexed_to_rgb(col: &mut VTermColor) {
    if col.is_indexed() {
        // SAFETY: We checked is_indexed() first
        let idx = unsafe { col.indexed.idx as usize };
        lookup_colour_palette(idx, col);
    }
    // Reset any metadata but the type
    // SAFETY: color_type is at the same position in all union variants
    unsafe {
        col.set_type(col.get_type() & VTERM_COLOR_TYPE_MASK);
    }
}

/// Set a color as the default foreground
pub fn set_default_fg(col: &mut VTermColor) {
    // SAFETY: color_type is at the same position in all union variants
    unsafe {
        let t = col.get_type();
        col.set_type((t & !VTERM_COLOR_DEFAULT_MASK) | VTERM_COLOR_DEFAULT_FG);
    }
}

/// Set a color as the default background
pub fn set_default_bg(col: &mut VTermColor) {
    // SAFETY: color_type is at the same position in all union variants
    unsafe {
        let t = col.get_type();
        col.set_type((t & !VTERM_COLOR_DEFAULT_MASK) | VTERM_COLOR_DEFAULT_BG);
    }
}

// =============================================================================
// SGR (Select Graphic Rendition) Parsing
// =============================================================================

/// SGR attribute command codes
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SgrCode {
    Reset = 0,
    Bold = 1,
    Italic = 3,
    Underline = 4,
    Blink = 5,
    Reverse = 7,
    Conceal = 8,
    Strike = 9,
    Font0 = 10,
    Font9 = 19,
    UnderlineDouble = 21,
    BoldOff = 22,
    ItalicOff = 23,
    UnderlineOff = 24,
    BlinkOff = 25,
    ReverseOff = 27,
    ConcealOff = 28,
    StrikeOff = 29,
    FgBlack = 30,
    FgWhite = 37,
    FgExtended = 38,
    FgDefault = 39,
    BgBlack = 40,
    BgWhite = 47,
    BgExtended = 48,
    BgDefault = 49,
    Superscript = 73,
    Subscript = 74,
    SuperSubOff = 75,
    FgBrightBlack = 90,
    FgBrightWhite = 97,
    BgBrightBlack = 100,
    BgBrightWhite = 107,
}

/// Result of processing a single SGR parameter
#[derive(Clone, Debug)]
pub enum SgrResult {
    /// Reset all attributes
    Reset,
    /// Set bold attribute
    SetBold(bool),
    /// Set italic attribute
    SetItalic(bool),
    /// Set underline style
    SetUnderline(u8),
    /// Set blink attribute
    SetBlink(bool),
    /// Set reverse attribute
    SetReverse(bool),
    /// Set conceal attribute
    SetConceal(bool),
    /// Set strike attribute
    SetStrike(bool),
    /// Set font (0-9)
    SetFont(u8),
    /// Set foreground color
    SetForeground(VTermColor),
    /// Reset foreground to default
    ResetForeground,
    /// Set background color
    SetBackground(VTermColor),
    /// Reset background to default
    ResetBackground,
    /// Set small text attribute
    SetSmall(bool),
    /// Set baseline (0=normal, 1=raise, 2=lower)
    SetBaseline(u8),
    /// Unrecognized SGR code
    Unrecognized(c_long),
    /// Need more arguments
    NeedMore,
}

/// Parse a single SGR parameter and return the result
///
/// Returns the result and number of additional arguments consumed.
pub fn parse_sgr_param(args: &[c_long], bold_is_highbright: bool) -> (SgrResult, usize) {
    if args.is_empty() {
        return (SgrResult::Reset, 0);
    }

    let arg = csi_arg(args[0]);

    match arg {
        // Reset
        val if csi_arg_is_missing(args[0]) || val == 0 => (SgrResult::Reset, 0),

        // Bold on
        1 => (SgrResult::SetBold(true), 0),

        // Italic on
        3 => (SgrResult::SetItalic(true), 0),

        // Underline
        4 => {
            if args.len() > 1 && csi_arg_has_more(args[0]) {
                let sub_arg = csi_arg(args[1]);
                let style = match sub_arg {
                    0 => VTermUnderline::Off as u8,
                    2 => VTermUnderline::Double as u8,
                    3 => VTermUnderline::Curly as u8,
                    _ => VTermUnderline::Single as u8, // 1 and others default to single
                };
                (SgrResult::SetUnderline(style), 1)
            } else {
                (SgrResult::SetUnderline(VTermUnderline::Single as u8), 0)
            }
        }

        // Blink
        5 => (SgrResult::SetBlink(true), 0),

        // Reverse on
        7 => (SgrResult::SetReverse(true), 0),

        // Conceal on
        8 => (SgrResult::SetConceal(true), 0),

        // Strikethrough on
        9 => (SgrResult::SetStrike(true), 0),

        // Font selection (10-19)
        10..=19 => (SgrResult::SetFont((arg - 10) as u8), 0),

        // Double underline
        21 => (SgrResult::SetUnderline(VTermUnderline::Double as u8), 0),

        // Bold off
        22 => (SgrResult::SetBold(false), 0),

        // Italic off
        23 => (SgrResult::SetItalic(false), 0),

        // Underline off
        24 => (SgrResult::SetUnderline(VTermUnderline::Off as u8), 0),

        // Blink off
        25 => (SgrResult::SetBlink(false), 0),

        // Reverse off
        27 => (SgrResult::SetReverse(false), 0),

        // Conceal off
        28 => (SgrResult::SetConceal(false), 0),

        // Strikethrough off
        29 => (SgrResult::SetStrike(false), 0),

        // Foreground colour palette (30-37)
        30..=37 => {
            let mut value = (arg - 30) as u8;
            if bold_is_highbright {
                value += 8;
            }
            (SgrResult::SetForeground(VTermColor::indexed(value)), 0)
        }

        // Foreground extended color
        38 => {
            if args.len() < 2 {
                return (SgrResult::NeedMore, 0);
            }
            let palette = csi_arg(args[1]);
            let mut col = VTermColor::default();
            let consumed = lookup_colour(palette, &args[2..], &mut col);
            (SgrResult::SetForeground(col), 1 + consumed)
        }

        // Foreground default
        39 => (SgrResult::ResetForeground, 0),

        // Background colour palette (40-47)
        40..=47 => {
            let value = (arg - 40) as u8;
            (SgrResult::SetBackground(VTermColor::indexed(value)), 0)
        }

        // Background extended color
        48 => {
            if args.len() < 2 {
                return (SgrResult::NeedMore, 0);
            }
            let palette = csi_arg(args[1]);
            let mut col = VTermColor::default();
            let consumed = lookup_colour(palette, &args[2..], &mut col);
            (SgrResult::SetBackground(col), 1 + consumed)
        }

        // Background default
        49 => (SgrResult::ResetBackground, 0),

        // Superscript / Subscript
        73 | 74 => (SgrResult::SetSmall(true), 0),

        // Superscript/subscript off
        75 => (SgrResult::SetSmall(false), 0),

        // Foreground high-intensity (90-97)
        90..=97 => {
            let value = (arg - 90 + 8) as u8;
            (SgrResult::SetForeground(VTermColor::indexed(value)), 0)
        }

        // Background high-intensity (100-107)
        100..=107 => {
            let value = (arg - 100 + 8) as u8;
            (SgrResult::SetBackground(VTermColor::indexed(value)), 0)
        }

        // Unrecognized
        _ => (SgrResult::Unrecognized(arg), 0),
    }
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Convert an indexed color to RGB using the default palette
#[no_mangle]
pub extern "C" fn rs_vterm_color_convert_to_rgb(col: *mut VTermColor) {
    if col.is_null() {
        return;
    }
    // SAFETY: Caller guarantees col is valid
    unsafe {
        convert_indexed_to_rgb(&mut *col);
    }
}

/// Look up a color from the 256-color palette
#[no_mangle]
pub extern "C" fn rs_vterm_lookup_colour_palette(index: c_int, col: *mut VTermColor) -> c_int {
    if col.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees col is valid
    c_int::from(unsafe { lookup_colour_palette(index as usize, &mut *col) })
}

/// Get the ANSI palette default color
#[no_mangle]
pub extern "C" fn rs_vterm_get_ansi_color(index: c_int, col: *mut VTermColor) {
    if col.is_null() || !(0..16).contains(&index) {
        return;
    }
    // SAFETY: Caller guarantees col is valid and index is in range
    unsafe {
        lookup_default_colour_ansi(index as usize, &mut *col);
    }
}

/// Get the value type for an attribute (internal helper)
pub fn get_attr_type(attr: VTermAttr) -> VTermValueType {
    match attr {
        VTermAttr::Bold
        | VTermAttr::Italic
        | VTermAttr::Blink
        | VTermAttr::Reverse
        | VTermAttr::Conceal
        | VTermAttr::Strike
        | VTermAttr::Small => VTermValueType::Bool,

        VTermAttr::Underline | VTermAttr::Font | VTermAttr::Baseline | VTermAttr::Uri => {
            VTermValueType::Int
        }

        VTermAttr::Foreground | VTermAttr::Background => VTermValueType::Color,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_colors() {
        assert_eq!(ANSI_COLORS[0].red, 0); // black
        assert_eq!(ANSI_COLORS[1].red, 224); // red
        assert_eq!(ANSI_COLORS[8].red, 128); // bright black (grey)
        assert_eq!(ANSI_COLORS[15].red, 255); // bright white
    }

    #[test]
    fn test_lookup_default_ansi() {
        let mut col = VTermColor::default();

        lookup_default_colour_ansi(0, &mut col);
        assert!(col.is_rgb());
        unsafe {
            assert_eq!(col.rgb.red, 0);
            assert_eq!(col.rgb.green, 0);
            assert_eq!(col.rgb.blue, 0);
        }

        lookup_default_colour_ansi(1, &mut col);
        unsafe {
            assert_eq!(col.rgb.red, 224);
            assert_eq!(col.rgb.green, 0);
            assert_eq!(col.rgb.blue, 0);
        }
    }

    #[test]
    fn test_lookup_palette_216_cube() {
        let mut col = VTermColor::default();

        // Test first color in cube (index 16)
        assert!(lookup_colour_palette(16, &mut col));
        unsafe {
            assert_eq!(col.rgb.red, RAMP6[0]);
            assert_eq!(col.rgb.green, RAMP6[0]);
            assert_eq!(col.rgb.blue, RAMP6[0]);
        }

        // Test last color in cube (index 231)
        assert!(lookup_colour_palette(231, &mut col));
        unsafe {
            assert_eq!(col.rgb.red, RAMP6[5]);
            assert_eq!(col.rgb.green, RAMP6[5]);
            assert_eq!(col.rgb.blue, RAMP6[5]);
        }
    }

    #[test]
    fn test_lookup_palette_greyscale() {
        let mut col = VTermColor::default();

        // First greyscale (index 232)
        assert!(lookup_colour_palette(232, &mut col));
        unsafe {
            assert_eq!(col.rgb.red, RAMP24[0]);
            assert_eq!(col.rgb.green, RAMP24[0]);
            assert_eq!(col.rgb.blue, RAMP24[0]);
        }

        // Last greyscale (index 255)
        assert!(lookup_colour_palette(255, &mut col));
        unsafe {
            assert_eq!(col.rgb.red, RAMP24[23]);
            assert_eq!(col.rgb.green, RAMP24[23]);
            assert_eq!(col.rgb.blue, RAMP24[23]);
        }
    }

    #[test]
    fn test_lookup_palette_out_of_range() {
        let mut col = VTermColor::default();
        assert!(!lookup_colour_palette(256, &mut col));
    }

    #[test]
    fn test_lookup_colour_rgb() {
        let mut col = VTermColor::default();
        let args: [c_long; 3] = [128, 64, 32];

        let consumed = lookup_colour(2, &args, &mut col);
        assert_eq!(consumed, 3);
        assert!(col.is_rgb());
        unsafe {
            assert_eq!(col.rgb.red, 128);
            assert_eq!(col.rgb.green, 64);
            assert_eq!(col.rgb.blue, 32);
        }
    }

    #[test]
    fn test_lookup_colour_indexed() {
        let mut col = VTermColor::default();
        let args: [c_long; 1] = [42];

        let consumed = lookup_colour(5, &args, &mut col);
        assert_eq!(consumed, 1);
        assert!(col.is_indexed());
        unsafe {
            assert_eq!(col.indexed.idx, 42);
        }
    }

    #[test]
    fn test_default_colors() {
        let fg = default_foreground();
        assert!(fg.is_rgb());
        assert!(fg.is_default_fg());
        unsafe {
            assert_eq!(fg.rgb.red, 240);
            assert_eq!(fg.rgb.green, 240);
            assert_eq!(fg.rgb.blue, 240);
        }

        let bg = default_background();
        assert!(bg.is_rgb());
        assert!(bg.is_default_bg());
        unsafe {
            assert_eq!(bg.rgb.red, 0);
            assert_eq!(bg.rgb.green, 0);
            assert_eq!(bg.rgb.blue, 0);
        }
    }

    #[test]
    fn test_convert_indexed_to_rgb() {
        let mut col = VTermColor::indexed(1); // Red
        convert_indexed_to_rgb(&mut col);
        assert!(col.is_rgb());
        unsafe {
            assert_eq!(col.rgb.red, 224);
            assert_eq!(col.rgb.green, 0);
            assert_eq!(col.rgb.blue, 0);
        }
    }

    #[test]
    fn test_init_default_palette() {
        let mut colors = [VTermColor::default(); 16];
        init_default_palette(&mut colors);

        // Check black
        assert!(colors[0].is_rgb());
        unsafe {
            assert_eq!(colors[0].rgb.red, 0);
        }

        // Check white
        unsafe {
            assert_eq!(colors[15].rgb.red, 255);
            assert_eq!(colors[15].rgb.green, 255);
            assert_eq!(colors[15].rgb.blue, 255);
        }
    }

    #[test]
    fn test_parse_sgr_reset() {
        let args: [c_long; 1] = [0];
        let (result, consumed) = parse_sgr_param(&args, false);
        assert!(matches!(result, SgrResult::Reset));
        assert_eq!(consumed, 0);
    }

    #[test]
    fn test_parse_sgr_bold() {
        let args: [c_long; 1] = [1];
        let (result, consumed) = parse_sgr_param(&args, false);
        assert!(matches!(result, SgrResult::SetBold(true)));
        assert_eq!(consumed, 0);

        let args: [c_long; 1] = [22];
        let (result, _) = parse_sgr_param(&args, false);
        assert!(matches!(result, SgrResult::SetBold(false)));
    }

    #[test]
    fn test_parse_sgr_underline() {
        // Single underline
        let args: [c_long; 1] = [4];
        let (result, _) = parse_sgr_param(&args, false);
        assert!(matches!(
            result,
            SgrResult::SetUnderline(x) if x == VTermUnderline::Single as u8
        ));

        // Double underline
        let args: [c_long; 1] = [21];
        let (result, _) = parse_sgr_param(&args, false);
        assert!(matches!(
            result,
            SgrResult::SetUnderline(x) if x == VTermUnderline::Double as u8
        ));
    }

    #[test]
    fn test_parse_sgr_foreground() {
        // Red (30)
        let args: [c_long; 1] = [31];
        let (result, _) = parse_sgr_param(&args, false);
        if let SgrResult::SetForeground(col) = result {
            assert!(col.is_indexed());
            unsafe {
                assert_eq!(col.indexed.idx, 1);
            }
        } else {
            panic!("Expected SetForeground");
        }

        // Bright red (91)
        let args: [c_long; 1] = [91];
        let (result, _) = parse_sgr_param(&args, false);
        if let SgrResult::SetForeground(col) = result {
            assert!(col.is_indexed());
            unsafe {
                assert_eq!(col.indexed.idx, 9);
            }
        } else {
            panic!("Expected SetForeground");
        }
    }

    #[test]
    fn test_parse_sgr_background() {
        // Blue (44)
        let args: [c_long; 1] = [44];
        let (result, _) = parse_sgr_param(&args, false);
        if let SgrResult::SetBackground(col) = result {
            assert!(col.is_indexed());
            unsafe {
                assert_eq!(col.indexed.idx, 4);
            }
        } else {
            panic!("Expected SetBackground");
        }
    }

    #[test]
    fn test_parse_sgr_font() {
        for font in 0..10 {
            let args: [c_long; 1] = [10 + font];
            let (result, _) = parse_sgr_param(&args, false);
            assert!(
                matches!(result, SgrResult::SetFont(f) if f == font as u8),
                "Font {font}"
            );
        }
    }

    #[test]
    fn test_get_attr_type() {
        assert_eq!(get_attr_type(VTermAttr::Bold), VTermValueType::Bool);
        assert_eq!(get_attr_type(VTermAttr::Underline), VTermValueType::Int);
        assert_eq!(get_attr_type(VTermAttr::Foreground), VTermValueType::Color);
    }
}
