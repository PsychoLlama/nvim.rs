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
    VTermValue, VTermValueType, VTERM_COLOR_DEFAULT_BG, VTERM_COLOR_DEFAULT_FG,
    VTERM_COLOR_DEFAULT_MASK, VTERM_COLOR_TYPE_MASK,
};

use crate::state::VTermStateHandle;

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

// =============================================================================
// Internal helpers for state-aware pen operations
// =============================================================================

/// Dispatch setpenattr callback with a boolean value
///
/// # Safety
/// The state handle must be valid.
unsafe fn setpenattr_bool(state: VTermStateHandle, attr: VTermAttr, boolean: bool) {
    let mut val = VTermValue {
        boolean: c_int::from(boolean),
    };
    nvim_vterm_state_call_setpenattr(state, attr as c_int, std::ptr::addr_of_mut!(val));
}

/// Dispatch setpenattr callback with an integer value
///
/// # Safety
/// The state handle must be valid.
unsafe fn setpenattr_int(state: VTermStateHandle, attr: VTermAttr, number: c_int) {
    let mut val = VTermValue { number };
    nvim_vterm_state_call_setpenattr(state, attr as c_int, std::ptr::addr_of_mut!(val));
}

/// Dispatch setpenattr callback with a color value
///
/// # Safety
/// The state handle must be valid.
unsafe fn setpenattr_col(state: VTermStateHandle, attr: VTermAttr, color: VTermColor) {
    let mut val = VTermValue { color };
    nvim_vterm_state_call_setpenattr(state, attr as c_int, std::ptr::addr_of_mut!(val));
}

// =============================================================================
// Phase 2 FFI Functions: simple pen state functions
// =============================================================================

/// Initialize pen state for a new `VTermState`
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_newpen(state: VTermStateHandle) {
    // 90% grey so that pure white is brighter
    let mut fg = VTermColor::rgb(240, 240, 240);
    let mut bg = VTermColor::rgb(0, 0, 0);
    // set_default_colors logic inline
    {
        let t = fg.get_type();
        fg.set_type((t & !VTERM_COLOR_DEFAULT_MASK) | VTERM_COLOR_DEFAULT_FG);
    }
    {
        let t = bg.get_type();
        bg.set_type((t & !VTERM_COLOR_DEFAULT_MASK) | VTERM_COLOR_DEFAULT_BG);
    }
    nvim_vterm_state_set_default_fg(state, fg);
    nvim_vterm_state_set_default_bg(state, bg);

    for col_idx in 0..16i32 {
        let mut col = VTermColor::default();
        lookup_default_colour_ansi(col_idx as usize, &mut col);
        nvim_vterm_state_set_color(state, col_idx, col);
    }
}

/// Set the default foreground and background colors
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_set_default_colors(
    state: VTermStateHandle,
    default_fg: *const VTermColor,
    default_bg: *const VTermColor,
) {
    if !default_fg.is_null() {
        let mut fg = *default_fg;
        let t = fg.get_type();
        fg.set_type((t & !VTERM_COLOR_DEFAULT_MASK) | VTERM_COLOR_DEFAULT_FG);
        nvim_vterm_state_set_default_fg(state, fg);
    }
    if !default_bg.is_null() {
        let mut bg = *default_bg;
        let t = bg.get_type();
        bg.set_type((t & !VTERM_COLOR_DEFAULT_MASK) | VTERM_COLOR_DEFAULT_BG);
        nvim_vterm_state_set_default_bg(state, bg);
    }
}

/// Set a single palette color (indices 0-15)
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_set_palette_color(
    state: VTermStateHandle,
    index: c_int,
    col: *const VTermColor,
) {
    if (0..16).contains(&index) && !col.is_null() {
        nvim_vterm_state_set_color(state, index, *col);
    }
}

/// Convert a color to RGB using the state's palette for indexed colors 0-15
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_convert_color_to_rgb(
    state: VTermStateHandle,
    col: *mut VTermColor,
) {
    if col.is_null() {
        return;
    }
    let c = &mut *col;
    if c.is_indexed() {
        let idx = c.indexed.idx as usize;
        if idx < 16 {
            // Use the state's palette for indices 0-15
            // idx < 16 so fits in c_int
            #[allow(clippy::cast_possible_wrap)]
            let idx_i = idx as c_int;
            *c = nvim_vterm_state_get_color(state, idx_i);
        } else {
            // Use default palette for 16-255
            lookup_colour_palette(idx, c);
        }
    }
    // Reset any metadata but the type
    let t = c.get_type();
    c.set_type(t & VTERM_COLOR_TYPE_MASK);
}

/// Reset all pen attributes to defaults
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_resetpen(state: VTermStateHandle) {
    nvim_vterm_state_set_pen_bold(state, 0);
    setpenattr_bool(state, VTermAttr::Bold, false);

    nvim_vterm_state_set_pen_underline(state, 0);
    setpenattr_int(state, VTermAttr::Underline, 0);

    nvim_vterm_state_set_pen_italic(state, 0);
    setpenattr_bool(state, VTermAttr::Italic, false);

    nvim_vterm_state_set_pen_blink(state, 0);
    setpenattr_bool(state, VTermAttr::Blink, false);

    nvim_vterm_state_set_pen_reverse(state, 0);
    setpenattr_bool(state, VTermAttr::Reverse, false);

    nvim_vterm_state_set_pen_conceal(state, 0);
    setpenattr_bool(state, VTermAttr::Conceal, false);

    nvim_vterm_state_set_pen_strike(state, 0);
    setpenattr_bool(state, VTermAttr::Strike, false);

    nvim_vterm_state_set_pen_font(state, 0);
    setpenattr_int(state, VTermAttr::Font, 0);

    nvim_vterm_state_set_pen_small(state, 0);
    setpenattr_bool(state, VTermAttr::Small, false);

    nvim_vterm_state_set_pen_baseline(state, 0);
    setpenattr_int(state, VTermAttr::Baseline, 0);

    let fg = nvim_vterm_state_get_default_fg(state);
    nvim_vterm_state_set_pen_fg(state, fg);
    setpenattr_col(state, VTermAttr::Foreground, fg);

    let bg = nvim_vterm_state_get_default_bg(state);
    nvim_vterm_state_set_pen_bg(state, bg);
    setpenattr_col(state, VTermAttr::Background, bg);

    nvim_vterm_state_set_pen_uri(state, 0);
    setpenattr_int(state, VTermAttr::Uri, 0);
}

/// Convert a C integer to a `VTermAttr` enum value, or None if out of range
fn vterm_attr_from_int(attr: c_int) -> Option<VTermAttr> {
    match attr {
        1 => Some(VTermAttr::Bold),
        2 => Some(VTermAttr::Underline),
        3 => Some(VTermAttr::Italic),
        4 => Some(VTermAttr::Blink),
        5 => Some(VTermAttr::Reverse),
        6 => Some(VTermAttr::Conceal),
        7 => Some(VTermAttr::Strike),
        8 => Some(VTermAttr::Font),
        9 => Some(VTermAttr::Foreground),
        10 => Some(VTermAttr::Background),
        11 => Some(VTermAttr::Small),
        12 => Some(VTermAttr::Baseline),
        13 => Some(VTermAttr::Uri),
        _ => None,
    }
}

// =============================================================================
// Phase 3 FFI Functions: savepen and set_penattr
// =============================================================================

/// Save or restore the pen state
///
/// If `save` is nonzero, saves the current pen to `saved.pen`.
/// If `save` is zero, restores the pen from `saved.pen` and dispatches callbacks.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_savepen(state: VTermStateHandle, save: c_int) {
    if save != 0 {
        nvim_vterm_state_save_pen(state);
    } else {
        nvim_vterm_state_restore_pen(state);

        // After restore, dispatch callbacks for all pen attributes with the restored values
        let bold = nvim_vterm_state_get_saved_pen_bold(state);
        setpenattr_bool(state, VTermAttr::Bold, bold != 0);

        let underline = nvim_vterm_state_get_saved_pen_underline(state);
        setpenattr_int(state, VTermAttr::Underline, underline);

        let italic = nvim_vterm_state_get_saved_pen_italic(state);
        setpenattr_bool(state, VTermAttr::Italic, italic != 0);

        let blink = nvim_vterm_state_get_saved_pen_blink(state);
        setpenattr_bool(state, VTermAttr::Blink, blink != 0);

        let reverse = nvim_vterm_state_get_saved_pen_reverse(state);
        setpenattr_bool(state, VTermAttr::Reverse, reverse != 0);

        let conceal = nvim_vterm_state_get_saved_pen_conceal(state);
        setpenattr_bool(state, VTermAttr::Conceal, conceal != 0);

        let strike = nvim_vterm_state_get_saved_pen_strike(state);
        setpenattr_bool(state, VTermAttr::Strike, strike != 0);

        let font = nvim_vterm_state_get_saved_pen_font(state);
        setpenattr_int(state, VTermAttr::Font, font);

        let small = nvim_vterm_state_get_saved_pen_small(state);
        setpenattr_bool(state, VTermAttr::Small, small != 0);

        let baseline = nvim_vterm_state_get_saved_pen_baseline(state);
        setpenattr_int(state, VTermAttr::Baseline, baseline);

        let fg = nvim_vterm_state_get_saved_pen_fg(state);
        setpenattr_col(state, VTermAttr::Foreground, fg);

        let bg = nvim_vterm_state_get_saved_pen_bg(state);
        setpenattr_col(state, VTermAttr::Background, bg);

        let uri = nvim_vterm_state_get_saved_pen_uri(state);
        setpenattr_int(state, VTermAttr::Uri, uri);
    }
}

/// Set an individual pen attribute by type
///
/// Returns 1 on success, 0 on failure (null val, type mismatch, or unknown attr).
///
/// # Safety
/// The state handle must be valid. `val` must be a valid pointer to `VTermValue`.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_set_penattr(
    state: VTermStateHandle,
    attr: c_int,
    type_: c_int,
    val: *mut VTermValue,
) -> c_int {
    if val.is_null() {
        return 0;
    }

    // Validate attr is in range and type matches
    let Some(attr_enum) = vterm_attr_from_int(attr) else {
        return 0;
    };

    let expected_type = get_attr_type(attr_enum) as c_int;
    if type_ != expected_type {
        return 0;
    }

    let v = &*val;

    match attr_enum {
        VTermAttr::Bold => nvim_vterm_state_set_pen_bold(state, v.boolean),
        VTermAttr::Underline => nvim_vterm_state_set_pen_underline(state, v.number),
        VTermAttr::Italic => nvim_vterm_state_set_pen_italic(state, v.boolean),
        VTermAttr::Blink => nvim_vterm_state_set_pen_blink(state, v.boolean),
        VTermAttr::Reverse => nvim_vterm_state_set_pen_reverse(state, v.boolean),
        VTermAttr::Conceal => nvim_vterm_state_set_pen_conceal(state, v.boolean),
        VTermAttr::Strike => nvim_vterm_state_set_pen_strike(state, v.boolean),
        VTermAttr::Font => nvim_vterm_state_set_pen_font(state, v.number),
        VTermAttr::Foreground => nvim_vterm_state_set_pen_fg(state, v.color),
        VTermAttr::Background => nvim_vterm_state_set_pen_bg(state, v.color),
        VTermAttr::Small => nvim_vterm_state_set_pen_small(state, v.boolean),
        VTermAttr::Baseline => nvim_vterm_state_set_pen_baseline(state, v.number),
        VTermAttr::Uri => nvim_vterm_state_set_pen_uri(state, v.number),
    }

    nvim_vterm_state_call_setpenattr(state, attr, val);

    1
}

// Pull in the C accessor functions declared in state.rs
// All imports are needed across phases 2-4.
#[allow(unused_imports)]
use crate::state::{
    nvim_vterm_state_call_setpenattr, nvim_vterm_state_get_bold_is_highbright,
    nvim_vterm_state_get_color, nvim_vterm_state_get_default_bg, nvim_vterm_state_get_default_fg,
    nvim_vterm_state_get_pen_baseline, nvim_vterm_state_get_pen_bg, nvim_vterm_state_get_pen_blink,
    nvim_vterm_state_get_pen_bold, nvim_vterm_state_get_pen_conceal, nvim_vterm_state_get_pen_fg,
    nvim_vterm_state_get_pen_font, nvim_vterm_state_get_pen_italic,
    nvim_vterm_state_get_pen_reverse, nvim_vterm_state_get_pen_small,
    nvim_vterm_state_get_pen_strike, nvim_vterm_state_get_pen_underline,
    nvim_vterm_state_get_pen_uri, nvim_vterm_state_get_saved_pen_baseline,
    nvim_vterm_state_get_saved_pen_bg, nvim_vterm_state_get_saved_pen_blink,
    nvim_vterm_state_get_saved_pen_bold, nvim_vterm_state_get_saved_pen_conceal,
    nvim_vterm_state_get_saved_pen_fg, nvim_vterm_state_get_saved_pen_font,
    nvim_vterm_state_get_saved_pen_italic, nvim_vterm_state_get_saved_pen_reverse,
    nvim_vterm_state_get_saved_pen_small, nvim_vterm_state_get_saved_pen_strike,
    nvim_vterm_state_get_saved_pen_underline, nvim_vterm_state_get_saved_pen_uri,
    nvim_vterm_state_restore_pen, nvim_vterm_state_save_pen, nvim_vterm_state_set_color,
    nvim_vterm_state_set_default_bg, nvim_vterm_state_set_default_fg,
    nvim_vterm_state_set_pen_baseline, nvim_vterm_state_set_pen_bg, nvim_vterm_state_set_pen_blink,
    nvim_vterm_state_set_pen_bold, nvim_vterm_state_set_pen_conceal, nvim_vterm_state_set_pen_fg,
    nvim_vterm_state_set_pen_font, nvim_vterm_state_set_pen_italic,
    nvim_vterm_state_set_pen_reverse, nvim_vterm_state_set_pen_small,
    nvim_vterm_state_set_pen_strike, nvim_vterm_state_set_pen_underline,
    nvim_vterm_state_set_pen_uri,
};

// =============================================================================
// Phase 4 FFI Functions: setpen (SGR dispatcher) and getpen
// =============================================================================

/// Set an ANSI palette foreground or background color on the pen, then dispatch callback
///
/// # Safety
/// The state handle must be valid.
unsafe fn set_pen_col_ansi(state: VTermStateHandle, attr: VTermAttr, col_idx: u8) {
    let col = VTermColor::indexed(col_idx);
    match attr {
        VTermAttr::Foreground => nvim_vterm_state_set_pen_fg(state, col),
        VTermAttr::Background => nvim_vterm_state_set_pen_bg(state, col),
        _ => {}
    }
    setpenattr_col(state, attr, col);
}

/// Process SGR (Select Graphic Rendition) parameters and apply to pen state
///
/// # Safety
/// The state handle must be valid. `args` must point to a valid array of `argcount` elements.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_vterm_state_setpen(
    state: VTermStateHandle,
    args: *const c_long,
    argcount: c_int,
) {
    use crate::{csi_arg, csi_arg_has_more, csi_arg_is_missing};

    if args.is_null() || argcount <= 0 {
        return;
    }

    let args = std::slice::from_raw_parts(args, argcount as usize);
    let mut argi: usize = 0;

    while argi < args.len() {
        let mut done = true;
        let arg = csi_arg(args[argi]);

        match arg {
            // Reset
            _ if csi_arg_is_missing(args[argi]) || arg == 0 => {
                rs_vterm_state_resetpen(state);
            }

            // Bold on
            1 => {
                let fg = nvim_vterm_state_get_pen_fg(state);
                nvim_vterm_state_set_pen_bold(state, 1);
                setpenattr_bool(state, VTermAttr::Bold, true);
                // bold_is_highbright: promote fg colour 0-7 to 8-15
                if !fg.is_default_fg()
                    && fg.is_indexed()
                    && fg.indexed.idx < 8
                    && nvim_vterm_state_get_bold_is_highbright(state) != 0
                {
                    set_pen_col_ansi(state, VTermAttr::Foreground, fg.indexed.idx + 8);
                }
            }

            // Italic on
            3 => {
                nvim_vterm_state_set_pen_italic(state, 1);
                setpenattr_bool(state, VTermAttr::Italic, true);
            }

            // Underline (with optional sub-parameter)
            4 => {
                let underline = if csi_arg_has_more(args[argi]) && argi + 1 < args.len() {
                    argi += 1;
                    match csi_arg(args[argi]) {
                        0 => VTermUnderline::Off as c_int,
                        2 => VTermUnderline::Double as c_int,
                        3 => VTermUnderline::Curly as c_int,
                        _ => VTermUnderline::Single as c_int, // 1 and others
                    }
                } else {
                    VTermUnderline::Single as c_int
                };
                nvim_vterm_state_set_pen_underline(state, underline);
                setpenattr_int(state, VTermAttr::Underline, underline);
            }

            // Blink
            5 => {
                nvim_vterm_state_set_pen_blink(state, 1);
                setpenattr_bool(state, VTermAttr::Blink, true);
            }

            // Reverse on
            7 => {
                nvim_vterm_state_set_pen_reverse(state, 1);
                setpenattr_bool(state, VTermAttr::Reverse, true);
            }

            // Conceal on
            8 => {
                nvim_vterm_state_set_pen_conceal(state, 1);
                setpenattr_bool(state, VTermAttr::Conceal, true);
            }

            // Strikethrough on
            9 => {
                nvim_vterm_state_set_pen_strike(state, 1);
                setpenattr_bool(state, VTermAttr::Strike, true);
            }

            // Select font (10-19)
            10..=19 => {
                let font = (arg - 10) as c_int;
                nvim_vterm_state_set_pen_font(state, font);
                setpenattr_int(state, VTermAttr::Font, font);
            }

            // Double underline
            21 => {
                nvim_vterm_state_set_pen_underline(state, VTermUnderline::Double as c_int);
                setpenattr_int(state, VTermAttr::Underline, VTermUnderline::Double as c_int);
            }

            // Bold off
            22 => {
                nvim_vterm_state_set_pen_bold(state, 0);
                setpenattr_bool(state, VTermAttr::Bold, false);
            }

            // Italic off
            23 => {
                nvim_vterm_state_set_pen_italic(state, 0);
                setpenattr_bool(state, VTermAttr::Italic, false);
            }

            // Underline off
            24 => {
                nvim_vterm_state_set_pen_underline(state, 0);
                setpenattr_int(state, VTermAttr::Underline, 0);
            }

            // Blink off
            25 => {
                nvim_vterm_state_set_pen_blink(state, 0);
                setpenattr_bool(state, VTermAttr::Blink, false);
            }

            // Reverse off
            27 => {
                nvim_vterm_state_set_pen_reverse(state, 0);
                setpenattr_bool(state, VTermAttr::Reverse, false);
            }

            // Conceal off
            28 => {
                nvim_vterm_state_set_pen_conceal(state, 0);
                setpenattr_bool(state, VTermAttr::Conceal, false);
            }

            // Strikethrough off
            29 => {
                nvim_vterm_state_set_pen_strike(state, 0);
                setpenattr_bool(state, VTermAttr::Strike, false);
            }

            // Foreground colour palette (30-37)
            30..=37 => {
                let mut value = (arg - 30) as u8;
                if nvim_vterm_state_get_pen_bold(state) != 0
                    && nvim_vterm_state_get_bold_is_highbright(state) != 0
                {
                    value += 8;
                }
                set_pen_col_ansi(state, VTermAttr::Foreground, value);
            }

            // Foreground colour alternative palette
            38 => {
                if args.len() - argi < 2 {
                    return;
                }
                let palette = csi_arg(args[argi + 1]);
                let mut col = VTermColor::default();
                let consumed = lookup_colour(palette, &args[argi + 2..], &mut col);
                argi += 1 + consumed;
                nvim_vterm_state_set_pen_fg(state, col);
                setpenattr_col(state, VTermAttr::Foreground, col);
            }

            // Foreground colour default
            39 => {
                let fg = nvim_vterm_state_get_default_fg(state);
                nvim_vterm_state_set_pen_fg(state, fg);
                setpenattr_col(state, VTermAttr::Foreground, fg);
            }

            // Background colour palette (40-47)
            40..=47 => {
                let value = (arg - 40) as u8;
                set_pen_col_ansi(state, VTermAttr::Background, value);
            }

            // Background colour alternative palette
            48 => {
                if args.len() - argi < 2 {
                    return;
                }
                let palette = csi_arg(args[argi + 1]);
                let mut col = VTermColor::default();
                let consumed = lookup_colour(palette, &args[argi + 2..], &mut col);
                argi += 1 + consumed;
                nvim_vterm_state_set_pen_bg(state, col);
                setpenattr_col(state, VTermAttr::Background, col);
            }

            // Background colour default
            49 => {
                let bg = nvim_vterm_state_get_default_bg(state);
                nvim_vterm_state_set_pen_bg(state, bg);
                setpenattr_col(state, VTermAttr::Background, bg);
            }

            // Superscript (73) / Subscript (74) / off (75)
            73..=75 => {
                let small = arg != 75;
                let baseline = match arg {
                    73 => crate::VTermBaseline::Raise as c_int,
                    74 => crate::VTermBaseline::Lower as c_int,
                    _ => crate::VTermBaseline::Normal as c_int,
                };
                nvim_vterm_state_set_pen_small(state, c_int::from(small));
                nvim_vterm_state_set_pen_baseline(state, baseline);
                setpenattr_bool(state, VTermAttr::Small, small);
                setpenattr_int(state, VTermAttr::Baseline, baseline);
            }

            // Foreground high-intensity (90-97)
            90..=97 => {
                let value = (arg - 90 + 8) as u8;
                set_pen_col_ansi(state, VTermAttr::Foreground, value);
            }

            // Background high-intensity (100-107)
            100..=107 => {
                let value = (arg - 100 + 8) as u8;
                set_pen_col_ansi(state, VTermAttr::Background, value);
            }

            _ => {
                done = false;
            }
        }

        let _ = done; // suppress unused warning

        // Advance past any sub-parameters with the HAS_MORE flag
        while csi_arg_has_more(args[argi]) {
            argi += 1;
            if argi >= args.len() {
                return;
            }
        }
        argi += 1;
    }
}

/// Serialize pen state to SGR parameter array
///
/// Returns the number of arguments written to `args`.
///
/// # Safety
/// The state handle must be valid. `args` must point to a valid array of at least `argcount`
/// elements.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_getpen(
    state: VTermStateHandle,
    args: *mut c_long,
    argcount: c_int,
) -> c_int {
    use crate::CSI_ARG_FLAG_MORE;

    if args.is_null() || argcount <= 0 {
        return 0;
    }

    let args = std::slice::from_raw_parts_mut(args, argcount as usize);
    let mut argi: usize = 0;

    if nvim_vterm_state_get_pen_bold(state) != 0 {
        args[argi] = 1;
        argi += 1;
    }

    if nvim_vterm_state_get_pen_italic(state) != 0 {
        args[argi] = 3;
        argi += 1;
    }

    let underline = nvim_vterm_state_get_pen_underline(state);
    if underline == VTermUnderline::Single as c_int {
        args[argi] = 4;
        argi += 1;
    }
    if underline == VTermUnderline::Curly as c_int {
        args[argi] = 4 | CSI_ARG_FLAG_MORE;
        argi += 1;
        args[argi] = 3;
        argi += 1;
    }

    if nvim_vterm_state_get_pen_blink(state) != 0 {
        args[argi] = 5;
        argi += 1;
    }

    // Note: reverse, conceal, strike read from pen but aren't exposed via scalar accessors yet.
    // We read them using the existing Phase 1 accessors.
    if nvim_vterm_state_get_pen_reverse(state) != 0 {
        args[argi] = 7;
        argi += 1;
    }

    if nvim_vterm_state_get_pen_conceal(state) != 0 {
        args[argi] = 8;
        argi += 1;
    }

    if nvim_vterm_state_get_pen_strike(state) != 0 {
        args[argi] = 9;
        argi += 1;
    }

    let font = nvim_vterm_state_get_pen_font(state);
    if font != 0 {
        args[argi] = c_long::from(10 + font);
        argi += 1;
    }

    if underline == VTermUnderline::Double as c_int {
        args[argi] = 21;
        argi += 1;
    }

    // Foreground color
    let fg = nvim_vterm_state_get_pen_fg(state);
    argi = getpen_color(&fg, argi, args, true);

    // Background color
    let bg = nvim_vterm_state_get_pen_bg(state);
    argi = getpen_color(&bg, argi, args, false);

    // Small / baseline
    let small = nvim_vterm_state_get_pen_small(state);
    if small != 0 {
        let baseline = nvim_vterm_state_get_pen_baseline(state);
        if baseline == crate::VTermBaseline::Raise as c_int {
            args[argi] = 73;
            argi += 1;
        } else if baseline == crate::VTermBaseline::Lower as c_int {
            args[argi] = 74;
            argi += 1;
        }
    }

    // argi <= argcount (c_int) so this is safe
    #[allow(clippy::cast_possible_wrap)]
    let result = argi as c_int;
    result
}

/// Serialize a single color to SGR args (helper for getpen)
///
/// Returns updated `argi`.
fn getpen_color(col: &VTermColor, mut argi: usize, args: &mut [c_long], fg: bool) -> usize {
    use crate::CSI_ARG_FLAG_MORE;

    // Do nothing if it's the default color
    if fg && col.is_default_fg() || !fg && col.is_default_bg() {
        return argi;
    }

    if col.is_indexed() {
        // SAFETY: We checked is_indexed()
        let idx = c_long::from(unsafe { col.indexed.idx });
        if idx < 8 {
            args[argi] = idx + if fg { 30 } else { 40 };
        } else if idx < 16 {
            args[argi] = idx - 8 + if fg { 90 } else { 100 };
        } else {
            args[argi] = CSI_ARG_FLAG_MORE | if fg { 38 } else { 48 };
            argi += 1;
            args[argi] = CSI_ARG_FLAG_MORE | 5;
            argi += 1;
            args[argi] = idx;
        }
        argi += 1;
    } else if col.is_rgb() {
        // SAFETY: We checked is_rgb()
        let (r, g, b) = unsafe { (col.rgb.red, col.rgb.green, col.rgb.blue) };
        args[argi] = CSI_ARG_FLAG_MORE | if fg { 38 } else { 48 };
        argi += 1;
        args[argi] = CSI_ARG_FLAG_MORE | 2;
        argi += 1;
        args[argi] = CSI_ARG_FLAG_MORE | c_long::from(r);
        argi += 1;
        args[argi] = CSI_ARG_FLAG_MORE | c_long::from(g);
        argi += 1;
        args[argi] = c_long::from(b);
        argi += 1;
    }

    argi
}

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
