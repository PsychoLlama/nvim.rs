//! `VTerm` - Terminal Emulator Implementation
//!
//! This crate provides a Rust implementation of the `VTerm` terminal emulator,
//! handling VT100/xterm escape sequences, screen state, and terminal operations.
//!
//! The implementation follows the libvterm API for compatibility with Neovim's
//! existing terminal integration.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::similar_names)]
#![allow(clippy::missing_const_for_fn)] // Many setters use &mut which doesn't work well with const
#![allow(clippy::not_unsafe_ptr_arg_deref)] // FFI functions check null before dereferencing

use std::ffi::{c_char, c_int, c_long, c_void};
use std::ptr;

pub mod callbacks;
pub mod encoding;
pub mod keyboard;
pub mod mouse;
pub mod parser;
pub mod pen;
pub mod screen;
pub mod state;

pub use callbacks::*;
pub use encoding::*;
pub use keyboard::*;
pub use mouse::*;
pub use parser::*;
pub use pen::*;
pub use screen::*;
pub use state::*;

// =============================================================================
// Constants
// =============================================================================

/// Version information
pub const VTERM_VERSION_MAJOR: c_int = 0;
pub const VTERM_VERSION_MINOR: c_int = 3;

/// Maximum intermediate characters in escape sequences
pub const VTERM_INTERMED_MAX: usize = 16;

/// Maximum CSI arguments
pub const VTERM_CSI_ARGS_MAX: usize = 32;

/// Maximum CSI leader length
pub const VTERM_CSI_LEADER_MAX: usize = 16;

/// Primary buffer index
pub const VTERM_BUFIDX_PRIMARY: usize = 0;

/// Alternate screen buffer index
pub const VTERM_BUFIDX_ALTSCREEN: usize = 1;

/// Default output buffer length
pub const VTERM_DEFAULT_OUTBUFFER_LEN: usize = 4096;

/// Default temporary buffer length
pub const VTERM_DEFAULT_TMPBUFFER_LEN: usize = 4096;

/// Maximum schar size (from `mbyte_defs.h`)
pub const VTERM_MAX_SCHAR_SIZE: usize = 32;

// =============================================================================
// CSI Argument Handling
// =============================================================================

/// Flag to indicate non-final subparameters in a single CSI parameter.
pub const CSI_ARG_FLAG_MORE: c_long = 1 << 31;

/// Mask to extract the argument value
pub const CSI_ARG_MASK: c_long = !CSI_ARG_FLAG_MORE;

/// Value indicating a missing argument
pub const CSI_ARG_MISSING: c_long = (1 << 31) - 1;

/// Check if a CSI argument has the MORE flag set
#[inline]
pub const fn csi_arg_has_more(a: c_long) -> bool {
    (a & CSI_ARG_FLAG_MORE) != 0
}

/// Extract the CSI argument value
#[inline]
pub const fn csi_arg(a: c_long) -> c_long {
    a & CSI_ARG_MASK
}

/// Check if a CSI argument is missing
#[inline]
pub const fn csi_arg_is_missing(a: c_long) -> bool {
    csi_arg(a) == CSI_ARG_MISSING
}

/// Get CSI argument or default value
#[inline]
pub const fn csi_arg_or(a: c_long, def: c_long) -> c_long {
    if csi_arg_is_missing(a) {
        def
    } else {
        csi_arg(a)
    }
}

/// Get CSI argument count (treating 0 or missing as 1)
#[inline]
pub const fn csi_arg_count(a: c_long) -> c_long {
    let v = csi_arg(a);
    if v == CSI_ARG_MISSING || v == 0 {
        1
    } else {
        v
    }
}

// =============================================================================
// Color Types
// =============================================================================

/// Color type flags
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VTermColorType {
    /// 24-bit RGB color
    Rgb = 0x00,
    /// Indexed color (256 palette)
    Indexed = 0x01,
}

/// Color type mask for extracting RGB/Indexed bit
pub const VTERM_COLOR_TYPE_MASK: u8 = 0x01;

/// Flag indicating default foreground color
pub const VTERM_COLOR_DEFAULT_FG: u8 = 0x02;

/// Flag indicating default background color
pub const VTERM_COLOR_DEFAULT_BG: u8 = 0x04;

/// Mask for extracting default fg/bg flags
pub const VTERM_COLOR_DEFAULT_MASK: u8 = 0x06;

/// Tagged union storing either an RGB color or a palette index.
#[repr(C)]
#[derive(Clone, Copy)]
pub union VTermColor {
    /// Type tag (coincides with rgb.type and indexed.type)
    pub color_type: u8,
    /// RGB color data
    pub rgb: VTermColorRgb,
    /// Indexed color data
    pub indexed: VTermColorIndexed,
}

/// RGB color component
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VTermColorRgb {
    /// Type tag
    pub color_type: u8,
    /// Red component (0-255)
    pub red: u8,
    /// Green component (0-255)
    pub green: u8,
    /// Blue component (0-255)
    pub blue: u8,
}

/// Indexed color
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VTermColorIndexed {
    /// Type tag
    pub color_type: u8,
    /// Index into color palette (0-255)
    pub idx: u8,
}

impl Default for VTermColor {
    fn default() -> Self {
        Self {
            rgb: VTermColorRgb {
                color_type: VTermColorType::Rgb as u8,
                red: 0,
                green: 0,
                blue: 0,
            },
        }
    }
}

impl VTermColor {
    /// Create a new RGB color
    #[inline]
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            rgb: VTermColorRgb {
                color_type: VTermColorType::Rgb as u8,
                red,
                green,
                blue,
            },
        }
    }

    /// Create a new indexed color
    #[inline]
    pub const fn indexed(idx: u8) -> Self {
        Self {
            indexed: VTermColorIndexed {
                color_type: VTermColorType::Indexed as u8,
                idx,
            },
        }
    }

    /// Check if this is an indexed color
    #[inline]
    pub const fn is_indexed(&self) -> bool {
        // SAFETY: color_type field is at the same position in all union variants
        unsafe { (self.color_type & VTERM_COLOR_TYPE_MASK) == VTermColorType::Indexed as u8 }
    }

    /// Check if this is an RGB color
    #[inline]
    pub const fn is_rgb(&self) -> bool {
        // SAFETY: color_type field is at the same position in all union variants
        unsafe { (self.color_type & VTERM_COLOR_TYPE_MASK) == VTermColorType::Rgb as u8 }
    }

    /// Check if this is the default foreground color
    #[inline]
    pub const fn is_default_fg(&self) -> bool {
        // SAFETY: color_type field is at the same position in all union variants
        unsafe { (self.color_type & VTERM_COLOR_DEFAULT_FG) != 0 }
    }

    /// Check if this is the default background color
    #[inline]
    pub const fn is_default_bg(&self) -> bool {
        // SAFETY: color_type field is at the same position in all union variants
        unsafe { (self.color_type & VTERM_COLOR_DEFAULT_BG) != 0 }
    }

    /// Get the type byte
    #[inline]
    pub const fn get_type(&self) -> u8 {
        // SAFETY: color_type field is at the same position in all union variants
        unsafe { self.color_type }
    }

    /// Set the type byte
    ///
    /// # Safety
    /// The caller must ensure that the type value is valid for the current
    /// color representation (RGB or indexed).
    #[inline]
    pub unsafe fn set_type(&mut self, t: u8) {
        // color_type field is at the same position in all union variants
        self.color_type = t;
    }
}

impl std::fmt::Debug for VTermColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: We check the type before accessing the correct variant
        if self.is_indexed() {
            unsafe {
                f.debug_struct("VTermColor::Indexed")
                    .field("idx", &self.indexed.idx)
                    .finish()
            }
        } else {
            unsafe {
                f.debug_struct("VTermColor::Rgb")
                    .field("red", &self.rgb.red)
                    .field("green", &self.rgb.green)
                    .field("blue", &self.rgb.blue)
                    .finish()
            }
        }
    }
}

// =============================================================================
// Position and Rectangle Types
// =============================================================================

/// A position in the terminal (row, column)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VTermPos {
    /// Row position (0-indexed)
    pub row: c_int,
    /// Column position (0-indexed)
    pub col: c_int,
}

impl VTermPos {
    /// Create a new position
    #[inline]
    pub const fn new(row: c_int, col: c_int) -> Self {
        Self { row, col }
    }
}

/// A rectangular region in the terminal
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VTermRect {
    /// Start row (inclusive)
    pub start_row: c_int,
    /// End row (exclusive)
    pub end_row: c_int,
    /// Start column (inclusive)
    pub start_col: c_int,
    /// End column (exclusive)
    pub end_col: c_int,
}

impl VTermRect {
    /// Create a new rectangle
    #[inline]
    pub const fn new(start_row: c_int, end_row: c_int, start_col: c_int, end_col: c_int) -> Self {
        Self {
            start_row,
            end_row,
            start_col,
            end_col,
        }
    }

    /// Move the rectangle by the given delta
    #[inline]
    pub fn move_by(&mut self, row_delta: c_int, col_delta: c_int) {
        self.start_row += row_delta;
        self.end_row += row_delta;
        self.start_col += col_delta;
        self.end_col += col_delta;
    }

    /// Check if position is within the rectangle
    #[inline]
    pub const fn contains(&self, pos: VTermPos) -> bool {
        pos.row >= self.start_row
            && pos.row < self.end_row
            && pos.col >= self.start_col
            && pos.col < self.end_col
    }

    /// Get the width of the rectangle
    #[inline]
    pub const fn width(&self) -> c_int {
        self.end_col - self.start_col
    }

    /// Get the height of the rectangle
    #[inline]
    pub const fn height(&self) -> c_int {
        self.end_row - self.start_row
    }
}

// =============================================================================
// Screen Cell Types
// =============================================================================

/// Screen cell attributes (bitfield)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VTermScreenCellAttrs {
    /// Packed attribute bits
    ///
    /// Layout:
    /// - bit 0: bold
    /// - bits 1-2: underline (0-3)
    /// - bit 3: italic
    /// - bit 4: blink
    /// - bit 5: reverse
    /// - bit 6: conceal
    /// - bit 7: strike
    /// - bits 8-11: font (0-9)
    /// - bit 12: dwl (double width line)
    /// - bits 13-14: dhl (double height line, 0=none, 1=top, 2=bottom)
    /// - bit 15: small
    /// - bits 16-17: baseline (0-2)
    bits: u32,
}

impl VTermScreenCellAttrs {
    // Bit positions
    const BOLD_BIT: u32 = 0;
    const UNDERLINE_SHIFT: u32 = 1;
    const UNDERLINE_MASK: u32 = 0x3;
    const ITALIC_BIT: u32 = 3;
    const BLINK_BIT: u32 = 4;
    const REVERSE_BIT: u32 = 5;
    const CONCEAL_BIT: u32 = 6;
    const STRIKE_BIT: u32 = 7;
    const FONT_SHIFT: u32 = 8;
    const FONT_MASK: u32 = 0xF;
    const DWL_BIT: u32 = 12;
    const DHL_SHIFT: u32 = 13;
    const DHL_MASK: u32 = 0x3;
    const SMALL_BIT: u32 = 15;
    const BASELINE_SHIFT: u32 = 16;
    const BASELINE_MASK: u32 = 0x3;

    /// Get bold attribute
    #[inline]
    pub const fn bold(&self) -> bool {
        (self.bits >> Self::BOLD_BIT) & 1 != 0
    }

    /// Set bold attribute
    #[inline]
    pub fn set_bold(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::BOLD_BIT;
        } else {
            self.bits &= !(1 << Self::BOLD_BIT);
        }
    }

    /// Get underline style (0=none, 1=single, 2=double, 3=curly)
    #[inline]
    pub const fn underline(&self) -> u8 {
        ((self.bits >> Self::UNDERLINE_SHIFT) & Self::UNDERLINE_MASK) as u8
    }

    /// Set underline style
    #[inline]
    pub fn set_underline(&mut self, v: u8) {
        self.bits &= !(Self::UNDERLINE_MASK << Self::UNDERLINE_SHIFT);
        self.bits |= (u32::from(v) & Self::UNDERLINE_MASK) << Self::UNDERLINE_SHIFT;
    }

    /// Get italic attribute
    #[inline]
    pub const fn italic(&self) -> bool {
        (self.bits >> Self::ITALIC_BIT) & 1 != 0
    }

    /// Set italic attribute
    #[inline]
    pub fn set_italic(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::ITALIC_BIT;
        } else {
            self.bits &= !(1 << Self::ITALIC_BIT);
        }
    }

    /// Get blink attribute
    #[inline]
    pub const fn blink(&self) -> bool {
        (self.bits >> Self::BLINK_BIT) & 1 != 0
    }

    /// Set blink attribute
    #[inline]
    pub fn set_blink(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::BLINK_BIT;
        } else {
            self.bits &= !(1 << Self::BLINK_BIT);
        }
    }

    /// Get reverse video attribute
    #[inline]
    pub const fn reverse(&self) -> bool {
        (self.bits >> Self::REVERSE_BIT) & 1 != 0
    }

    /// Set reverse video attribute
    #[inline]
    pub fn set_reverse(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::REVERSE_BIT;
        } else {
            self.bits &= !(1 << Self::REVERSE_BIT);
        }
    }

    /// Get conceal attribute
    #[inline]
    pub const fn conceal(&self) -> bool {
        (self.bits >> Self::CONCEAL_BIT) & 1 != 0
    }

    /// Set conceal attribute
    #[inline]
    pub fn set_conceal(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::CONCEAL_BIT;
        } else {
            self.bits &= !(1 << Self::CONCEAL_BIT);
        }
    }

    /// Get strike attribute
    #[inline]
    pub const fn strike(&self) -> bool {
        (self.bits >> Self::STRIKE_BIT) & 1 != 0
    }

    /// Set strike attribute
    #[inline]
    pub fn set_strike(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::STRIKE_BIT;
        } else {
            self.bits &= !(1 << Self::STRIKE_BIT);
        }
    }

    /// Get font number (0-9)
    #[inline]
    pub const fn font(&self) -> u8 {
        ((self.bits >> Self::FONT_SHIFT) & Self::FONT_MASK) as u8
    }

    /// Set font number
    #[inline]
    pub fn set_font(&mut self, v: u8) {
        self.bits &= !(Self::FONT_MASK << Self::FONT_SHIFT);
        self.bits |= (u32::from(v) & Self::FONT_MASK) << Self::FONT_SHIFT;
    }

    /// Get double-width line attribute
    #[inline]
    pub const fn dwl(&self) -> bool {
        (self.bits >> Self::DWL_BIT) & 1 != 0
    }

    /// Set double-width line attribute
    #[inline]
    pub fn set_dwl(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::DWL_BIT;
        } else {
            self.bits &= !(1 << Self::DWL_BIT);
        }
    }

    /// Get double-height line part (0=none, 1=top, 2=bottom)
    #[inline]
    pub const fn dhl(&self) -> u8 {
        ((self.bits >> Self::DHL_SHIFT) & Self::DHL_MASK) as u8
    }

    /// Set double-height line part
    #[inline]
    pub fn set_dhl(&mut self, v: u8) {
        self.bits &= !(Self::DHL_MASK << Self::DHL_SHIFT);
        self.bits |= (u32::from(v) & Self::DHL_MASK) << Self::DHL_SHIFT;
    }

    /// Get small text attribute
    #[inline]
    pub const fn small(&self) -> bool {
        (self.bits >> Self::SMALL_BIT) & 1 != 0
    }

    /// Set small text attribute
    #[inline]
    pub fn set_small(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::SMALL_BIT;
        } else {
            self.bits &= !(1 << Self::SMALL_BIT);
        }
    }

    /// Get baseline offset (0=normal, 1=raise, 2=lower)
    #[inline]
    pub const fn baseline(&self) -> u8 {
        ((self.bits >> Self::BASELINE_SHIFT) & Self::BASELINE_MASK) as u8
    }

    /// Set baseline offset
    #[inline]
    pub fn set_baseline(&mut self, v: u8) {
        self.bits &= !(Self::BASELINE_MASK << Self::BASELINE_SHIFT);
        self.bits |= (u32::from(v) & Self::BASELINE_MASK) << Self::BASELINE_SHIFT;
    }
}

/// Type alias for screen character (opaque handle to C `schar_T`)
pub type SChar = u64;

/// A screen cell containing character, width, attributes, and colors
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VTermScreenCell {
    /// The character in this cell (`schar_T` from Neovim)
    pub schar: SChar,
    /// Display width of the character
    pub width: c_char,
    /// Cell attributes
    pub attrs: VTermScreenCellAttrs,
    /// Foreground color
    pub fg: VTermColor,
    /// Background color
    pub bg: VTermColor,
    /// URI index (for hyperlinks)
    pub uri: c_int,
}

impl Default for VTermScreenCell {
    fn default() -> Self {
        Self {
            schar: 0,
            width: 1,
            attrs: VTermScreenCellAttrs::default(),
            fg: VTermColor::default(),
            bg: VTermColor::default(),
            uri: 0,
        }
    }
}

// =============================================================================
// Terminal Properties
// =============================================================================

/// Terminal properties
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VTermProp {
    /// Cursor visibility (bool)
    CursorVisible = 1,
    /// Cursor blink (bool)
    CursorBlink = 2,
    /// Alternate screen mode (bool)
    AltScreen = 3,
    /// Terminal title (string)
    Title = 4,
    /// Icon name (string)
    IconName = 5,
    /// Reverse video mode (bool)
    Reverse = 6,
    /// Cursor shape (number)
    CursorShape = 7,
    /// Mouse tracking mode (number)
    Mouse = 8,
    /// Focus reporting (bool)
    FocusReport = 9,
    /// Theme updates (bool)
    ThemeUpdates = 10,
}

/// Number of terminal properties
pub const VTERM_N_PROPS: usize = 11;

/// Cursor shapes
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VTermCursorShape {
    Block = 1,
    Underline = 2,
    BarLeft = 3,
}

/// Mouse tracking modes
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VTermMouseMode {
    None = 0,
    Click = 1,
    Drag = 2,
    Move = 3,
}

// =============================================================================
// Value Types
// =============================================================================

/// Value type enum for property values
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VTermValueType {
    /// Boolean value
    Bool = 1,
    /// Integer value
    Int = 2,
    /// String value
    String = 3,
    /// Color value
    Color = 4,
}

/// String fragment for incremental string delivery
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VTermStringFragment {
    /// Pointer to string data
    pub str_ptr: *const c_char,
    /// Length of the string fragment (30 bits)
    /// Packed with initial and final flags
    len_and_flags: u32,
}

impl VTermStringFragment {
    /// Length mask (30 bits)
    const LEN_MASK: u32 = 0x3FFF_FFFF;
    /// Initial fragment flag (bit 30)
    const INITIAL_BIT: u32 = 30;
    /// Final fragment flag (bit 31)
    const FINAL_BIT: u32 = 31;

    /// Create a new string fragment
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(str_ptr: *const c_char, len: usize, initial: bool, is_final: bool) -> Self {
        // Truncation is intentional - length is capped at 30 bits
        let mut len_and_flags = (len as u32) & Self::LEN_MASK;
        if initial {
            len_and_flags |= 1 << Self::INITIAL_BIT;
        }
        if is_final {
            len_and_flags |= 1 << Self::FINAL_BIT;
        }
        Self {
            str_ptr,
            len_and_flags,
        }
    }

    /// Get the string length
    #[inline]
    pub const fn len(&self) -> usize {
        (self.len_and_flags & Self::LEN_MASK) as usize
    }

    /// Check if the fragment is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if this is the initial fragment
    #[inline]
    pub const fn is_initial(&self) -> bool {
        (self.len_and_flags >> Self::INITIAL_BIT) & 1 != 0
    }

    /// Check if this is the final fragment
    #[inline]
    pub const fn is_final(&self) -> bool {
        (self.len_and_flags >> Self::FINAL_BIT) & 1 != 0
    }
}

/// Tagged union for property values
#[repr(C)]
#[derive(Clone, Copy)]
pub union VTermValue {
    /// Boolean value
    pub boolean: c_int,
    /// Numeric value
    pub number: c_int,
    /// String value
    pub string: VTermStringFragment,
    /// Color value
    pub color: VTermColor,
}

// =============================================================================
// Attribute Types
// =============================================================================

/// Attribute types for pen state
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VTermAttr {
    /// Bold attribute (bool)
    Bold = 1,
    /// Underline style (number: 0-3)
    Underline = 2,
    /// Italic attribute (bool)
    Italic = 3,
    /// Blink attribute (bool)
    Blink = 4,
    /// Reverse video (bool)
    Reverse = 5,
    /// Concealed text (bool)
    Conceal = 6,
    /// Strikethrough (bool)
    Strike = 7,
    /// Font number (number: 0-9)
    Font = 8,
    /// Foreground color (color)
    Foreground = 9,
    /// Background color (color)
    Background = 10,
    /// Small text (bool)
    Small = 11,
    /// Baseline offset (number: 0-2)
    Baseline = 12,
    /// URI/hyperlink (number)
    Uri = 13,
}

/// Number of attributes
pub const VTERM_N_ATTRS: usize = 14;

/// Attribute mask flags
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum VTermAttrMask {
    Bold = 1 << 0,
    Underline = 1 << 1,
    Italic = 1 << 2,
    Blink = 1 << 3,
    Reverse = 1 << 4,
    Strike = 1 << 5,
    Font = 1 << 6,
    Foreground = 1 << 7,
    Background = 1 << 8,
    Conceal = 1 << 9,
    Small = 1 << 10,
    Baseline = 1 << 11,
    Uri = 1 << 12,
}

/// All attributes mask
pub const VTERM_ALL_ATTRS_MASK: u32 = (1 << 13) - 1;

/// Underline styles
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VTermUnderline {
    Off = 0,
    Single = 1,
    Double = 2,
    Curly = 3,
}

/// Baseline positions
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VTermBaseline {
    Normal = 0,
    Raise = 1,
    Lower = 2,
}

// =============================================================================
// Damage Tracking
// =============================================================================

/// Damage reporting granularity
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VTermDamageSize {
    /// Report damage for each cell
    Cell = 0,
    /// Report damage for entire rows
    Row = 1,
    /// Report damage for entire screen
    Screen = 2,
    /// Report damage with scroll information
    Scroll = 3,
}

/// Number of damage sizes
pub const VTERM_N_DAMAGES: usize = 4;

// =============================================================================
// Selection Types
// =============================================================================

/// Selection mask for clipboard operations
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum VTermSelectionMask {
    Clipboard = 1 << 0,
    Primary = 1 << 1,
    Secondary = 1 << 2,
    Select = 1 << 3,
    Cut0 = 1 << 4, // CUT0-CUT7 by bit shifting
}

// =============================================================================
// Glyph and Line Info
// =============================================================================

/// Glyph information for rendering
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VTermGlyphInfo {
    /// The character
    pub schar: SChar,
    /// Display width
    pub width: c_int,
    /// Protected cell flag (DECSCA)
    pub protected_cell: u8,
    /// Double-width line
    pub dwl: u8,
    /// Double-height line (0=none, 1=top, 2=bottom)
    pub dhl: u8,
}

/// Line information
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VTermLineInfo {
    /// Line attributes packed into a single byte
    /// - bit 0: doublewidth (DECDWL or DECDHL line)
    /// - bits 1-2: doubleheight (0=none, 1=top, 2=bottom)
    /// - bit 3: continuation (line is a flow continuation)
    bits: u8,
}

impl VTermLineInfo {
    const DOUBLEWIDTH_BIT: u8 = 0;
    const DOUBLEHEIGHT_SHIFT: u8 = 1;
    const DOUBLEHEIGHT_MASK: u8 = 0x3;
    const CONTINUATION_BIT: u8 = 3;

    /// Get double-width flag
    #[inline]
    pub const fn doublewidth(&self) -> bool {
        (self.bits >> Self::DOUBLEWIDTH_BIT) & 1 != 0
    }

    /// Set double-width flag
    #[inline]
    pub fn set_doublewidth(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::DOUBLEWIDTH_BIT;
        } else {
            self.bits &= !(1 << Self::DOUBLEWIDTH_BIT);
        }
    }

    /// Get double-height part
    #[inline]
    pub const fn doubleheight(&self) -> u8 {
        (self.bits >> Self::DOUBLEHEIGHT_SHIFT) & Self::DOUBLEHEIGHT_MASK
    }

    /// Set double-height part
    #[inline]
    pub fn set_doubleheight(&mut self, v: u8) {
        self.bits &= !(Self::DOUBLEHEIGHT_MASK << Self::DOUBLEHEIGHT_SHIFT);
        self.bits |= (v & Self::DOUBLEHEIGHT_MASK) << Self::DOUBLEHEIGHT_SHIFT;
    }

    /// Get continuation flag
    #[inline]
    pub const fn continuation(&self) -> bool {
        (self.bits >> Self::CONTINUATION_BIT) & 1 != 0
    }

    /// Set continuation flag
    #[inline]
    pub fn set_continuation(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::CONTINUATION_BIT;
        } else {
            self.bits &= !(1 << Self::CONTINUATION_BIT);
        }
    }
}

// =============================================================================
// Parser State
// =============================================================================

/// Parser state machine states
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VTermParserState {
    /// Normal text processing
    Normal = 0,
    /// CSI leader bytes
    CsiLeader = 1,
    /// CSI numeric arguments
    CsiArgs = 2,
    /// CSI intermediate bytes
    CsiIntermed = 3,
    /// DCS command
    DcsCommand = 4,
    /// OSC command number
    OscCommand = 5,
    /// OSC string data
    Osc = 6,
    /// DCS vterm-specific
    DcsVterm = 7,
    /// APC string
    Apc = 8,
    /// PM string
    Pm = 9,
    /// SOS string
    Sos = 10,
}

// =============================================================================
// Key Encoding
// =============================================================================

/// Key encoding flags (Kitty keyboard protocol)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VTermKeyEncodingFlags {
    /// Flags packed into a single byte
    bits: u8,
}

impl VTermKeyEncodingFlags {
    const DISAMBIGUATE_BIT: u8 = 0;
    const REPORT_EVENTS_BIT: u8 = 1;
    const REPORT_ALTERNATE_BIT: u8 = 2;
    const REPORT_ALL_KEYS_BIT: u8 = 3;
    const REPORT_ASSOCIATED_BIT: u8 = 4;

    #[inline]
    pub const fn disambiguate(&self) -> bool {
        (self.bits >> Self::DISAMBIGUATE_BIT) & 1 != 0
    }

    #[inline]
    pub fn set_disambiguate(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::DISAMBIGUATE_BIT;
        } else {
            self.bits &= !(1 << Self::DISAMBIGUATE_BIT);
        }
    }

    #[inline]
    pub const fn report_events(&self) -> bool {
        (self.bits >> Self::REPORT_EVENTS_BIT) & 1 != 0
    }

    #[inline]
    pub fn set_report_events(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::REPORT_EVENTS_BIT;
        } else {
            self.bits &= !(1 << Self::REPORT_EVENTS_BIT);
        }
    }

    #[inline]
    pub const fn report_alternate(&self) -> bool {
        (self.bits >> Self::REPORT_ALTERNATE_BIT) & 1 != 0
    }

    #[inline]
    pub fn set_report_alternate(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::REPORT_ALTERNATE_BIT;
        } else {
            self.bits &= !(1 << Self::REPORT_ALTERNATE_BIT);
        }
    }

    #[inline]
    pub const fn report_all_keys(&self) -> bool {
        (self.bits >> Self::REPORT_ALL_KEYS_BIT) & 1 != 0
    }

    #[inline]
    pub fn set_report_all_keys(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::REPORT_ALL_KEYS_BIT;
        } else {
            self.bits &= !(1 << Self::REPORT_ALL_KEYS_BIT);
        }
    }

    #[inline]
    pub const fn report_associated(&self) -> bool {
        (self.bits >> Self::REPORT_ASSOCIATED_BIT) & 1 != 0
    }

    #[inline]
    pub fn set_report_associated(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::REPORT_ASSOCIATED_BIT;
        } else {
            self.bits &= !(1 << Self::REPORT_ASSOCIATED_BIT);
        }
    }
}

/// Key encoding stack for push/pop support
#[repr(C)]
pub struct VTermKeyEncodingStack {
    /// Stack items
    items: [VTermKeyEncodingFlags; 16],
    /// Current stack size (at least 1)
    size: u8,
}

impl Default for VTermKeyEncodingStack {
    fn default() -> Self {
        Self {
            items: [VTermKeyEncodingFlags::default(); 16],
            size: 1,
        }
    }
}

impl VTermKeyEncodingStack {
    /// Get the current (top) flags
    #[inline]
    pub fn current(&self) -> &VTermKeyEncodingFlags {
        &self.items[self.size as usize - 1]
    }

    /// Get the current (top) flags mutably
    #[inline]
    pub fn current_mut(&mut self) -> &mut VTermKeyEncodingFlags {
        &mut self.items[self.size as usize - 1]
    }

    /// Push a new flags entry
    #[inline]
    pub fn push(&mut self, flags: VTermKeyEncodingFlags) -> bool {
        if self.size as usize >= self.items.len() {
            return false;
        }
        self.items[self.size as usize] = flags;
        self.size += 1;
        true
    }

    /// Pop the top flags entry
    #[inline]
    pub fn pop(&mut self) -> bool {
        if self.size <= 1 {
            return false;
        }
        self.size -= 1;
        true
    }
}

// =============================================================================
// C1 Control Characters
// =============================================================================

/// C1 control character codes
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum C1Control {
    /// Single Shift 3
    Ss3 = 0x8F,
    /// Device Control String
    Dcs = 0x90,
    /// Control Sequence Introducer
    Csi = 0x9B,
    /// String Terminator
    St = 0x9C,
    /// Operating System Command
    Osc = 0x9D,
}

// =============================================================================
// Opaque Handle Types (for C interop)
// =============================================================================

/// Opaque handle to `VTerm` instance
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct VTermHandle(*mut c_void);

impl VTermHandle {
    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(ptr::null_mut())
    }

    /// Create a handle from a raw pointer
    ///
    /// # Safety
    /// The pointer must be a valid `VTerm*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to `VTermState` instance
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct VTermStateHandle(*mut c_void);

impl VTermStateHandle {
    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(ptr::null_mut())
    }

    /// Create a handle from a raw pointer
    ///
    /// # Safety
    /// The pointer must be a valid `VTermState*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to `VTermScreen` instance
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct VTermScreenHandle(*mut c_void);

impl VTermScreenHandle {
    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(ptr::null_mut())
    }

    /// Create a handle from a raw pointer
    ///
    /// # Safety
    /// The pointer must be a valid `VTermScreen*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// VTerm Builder
// =============================================================================

/// Builder for creating `VTerm` instances
#[repr(C)]
pub struct VTermBuilder {
    /// Version (currently unused, reserved for ABI)
    pub ver: c_int,
    /// Number of rows
    pub rows: c_int,
    /// Number of columns
    pub cols: c_int,
    /// Custom allocator (null for default)
    pub allocator: *const VTermAllocatorFunctions,
    /// Allocator user data
    pub allocdata: *mut c_void,
    /// Output buffer length (0 for default 4096)
    pub outbuffer_len: usize,
    /// Temporary buffer length (0 for default 4096)
    pub tmpbuffer_len: usize,
}

impl Default for VTermBuilder {
    fn default() -> Self {
        Self {
            ver: 0,
            rows: 24,
            cols: 80,
            allocator: ptr::null(),
            allocdata: ptr::null_mut(),
            outbuffer_len: 0,
            tmpbuffer_len: 0,
        }
    }
}

/// Allocator function type for malloc
pub type VTermAllocMalloc =
    unsafe extern "C" fn(size: usize, allocdata: *mut c_void) -> *mut c_void;

/// Allocator function type for free
pub type VTermAllocFree = unsafe extern "C" fn(ptr: *mut c_void, allocdata: *mut c_void);

/// Custom allocator functions
#[repr(C)]
pub struct VTermAllocatorFunctions {
    /// Malloc function - memory must be zeroed
    pub malloc: VTermAllocMalloc,
    /// Free function
    pub free: VTermAllocFree,
}

/// Output callback function type
pub type VTermOutputCallback =
    unsafe extern "C" fn(s: *const c_char, len: usize, user: *mut c_void);

// =============================================================================
// State Fields (for resize callback)
// =============================================================================

/// Copies of `VTermState` fields that the resize callback can modify
#[repr(C)]
pub struct VTermStateFields {
    /// Current cursor position
    pub pos: VTermPos,
    /// Line info arrays (primary and altscreen)
    pub lineinfos: [*mut VTermLineInfo; 2],
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Get the value type for an attribute
#[no_mangle]
pub extern "C" fn rs_vterm_get_attr_type(attr: c_int) -> c_int {
    match attr {
        // Boolean attributes: bold, italic, blink, reverse, conceal, strike, small
        1 | 3 | 4 | 5 | 6 | 7 | 11 => VTermValueType::Bool as c_int,
        // Integer attributes: underline, font, baseline, URI
        2 | 8 | 12 | 13 => VTermValueType::Int as c_int,
        // Color attributes: foreground, background
        9 | 10 => VTermValueType::Color as c_int,
        _ => 0,
    }
}

/// Move a rectangle by delta
#[no_mangle]
pub extern "C" fn rs_vterm_rect_move(rect: *mut VTermRect, row_delta: c_int, col_delta: c_int) {
    if rect.is_null() {
        return;
    }
    // SAFETY: Caller guarantees rect is valid
    unsafe {
        (*rect).move_by(row_delta, col_delta);
    }
}

/// Create an RGB color
#[no_mangle]
pub extern "C" fn rs_vterm_color_rgb(col: *mut VTermColor, red: u8, green: u8, blue: u8) {
    if col.is_null() {
        return;
    }
    // SAFETY: Caller guarantees col is valid
    unsafe {
        *col = VTermColor::rgb(red, green, blue);
    }
}

/// Create an indexed color
#[no_mangle]
pub extern "C" fn rs_vterm_color_indexed(col: *mut VTermColor, idx: u8) {
    if col.is_null() {
        return;
    }
    // SAFETY: Caller guarantees col is valid
    unsafe {
        *col = VTermColor::indexed(idx);
    }
}

/// Check if color is indexed
#[no_mangle]
pub extern "C" fn rs_vterm_color_is_indexed(col: *const VTermColor) -> c_int {
    if col.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees col is valid
    c_int::from(unsafe { (*col).is_indexed() })
}

/// Check if color is RGB
#[no_mangle]
pub extern "C" fn rs_vterm_color_is_rgb(col: *const VTermColor) -> c_int {
    if col.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees col is valid
    c_int::from(unsafe { (*col).is_rgb() })
}

/// Check if color is default foreground
#[no_mangle]
pub extern "C" fn rs_vterm_color_is_default_fg(col: *const VTermColor) -> c_int {
    if col.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees col is valid
    c_int::from(unsafe { (*col).is_default_fg() })
}

/// Check if color is default background
#[no_mangle]
pub extern "C" fn rs_vterm_color_is_default_bg(col: *const VTermColor) -> c_int {
    if col.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees col is valid
    c_int::from(unsafe { (*col).is_default_bg() })
}

/// Scroll a rectangular region
///
/// This helper function handles scrolling by calling the provided moverect
/// and eraserect callbacks.
#[no_mangle]
pub extern "C" fn rs_vterm_scroll_rect(
    rect: VTermRect,
    downward: c_int,
    rightward: c_int,
    moverect: Option<
        unsafe extern "C" fn(src: VTermRect, dest: VTermRect, user: *mut c_void) -> c_int,
    >,
    eraserect: Option<
        unsafe extern "C" fn(rect: VTermRect, selective: c_int, user: *mut c_void) -> c_int,
    >,
    user: *mut c_void,
) {
    let height = rect.end_row - rect.start_row;
    let width = rect.end_col - rect.start_col;

    // If scroll amount >= region size, just erase the whole region
    if downward.abs() >= height || rightward.abs() >= width {
        if let Some(erase) = eraserect {
            // SAFETY: Caller guarantees callback and user pointer validity
            unsafe {
                erase(rect, 0, user);
            }
        }
        return;
    }

    // Calculate source and destination rectangles
    let (src, dest) = if rightward >= 0 {
        let src = VTermRect::new(
            rect.start_row,
            rect.end_row,
            rect.start_col + rightward,
            rect.end_col,
        );
        let dest = VTermRect::new(
            rect.start_row,
            rect.end_row,
            rect.start_col,
            rect.end_col - rightward,
        );
        (src, dest)
    } else {
        let leftward = -rightward;
        let src = VTermRect::new(
            rect.start_row,
            rect.end_row,
            rect.start_col,
            rect.end_col - leftward,
        );
        let dest = VTermRect::new(
            rect.start_row,
            rect.end_row,
            rect.start_col + leftward,
            rect.end_col,
        );
        (src, dest)
    };

    let (src, dest) = if downward >= 0 {
        let src = VTermRect::new(
            src.start_row + downward,
            src.end_row,
            src.start_col,
            src.end_col,
        );
        let dest = VTermRect::new(
            dest.start_row,
            dest.end_row - downward,
            dest.start_col,
            dest.end_col,
        );
        (src, dest)
    } else {
        let upward = -downward;
        let src = VTermRect::new(
            src.start_row,
            src.end_row - upward,
            src.start_col,
            src.end_col,
        );
        let dest = VTermRect::new(
            dest.start_row + upward,
            dest.end_row,
            dest.start_col,
            dest.end_col,
        );
        (src, dest)
    };

    // Move the content
    if let Some(mv) = moverect {
        // SAFETY: Caller guarantees callback and user pointer validity
        unsafe {
            mv(dest, src, user);
        }
    }

    // Calculate the region to erase
    let mut erase_rect = rect;
    if downward > 0 {
        erase_rect.start_row = erase_rect.end_row - downward;
    } else if downward < 0 {
        erase_rect.end_row = erase_rect.start_row - downward;
    }

    if rightward > 0 {
        erase_rect.start_col = erase_rect.end_col - rightward;
    } else if rightward < 0 {
        erase_rect.end_col = erase_rect.start_col - rightward;
    }

    // Erase the vacated region
    if let Some(erase) = eraserect {
        // SAFETY: Caller guarantees callback and user pointer validity
        unsafe {
            erase(erase_rect, 0, user);
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vterm_pos() {
        let pos = VTermPos::new(10, 20);
        assert_eq!(pos.row, 10);
        assert_eq!(pos.col, 20);
    }

    #[test]
    fn test_vterm_rect() {
        let mut rect = VTermRect::new(0, 24, 0, 80);
        assert_eq!(rect.width(), 80);
        assert_eq!(rect.height(), 24);
        assert!(rect.contains(VTermPos::new(10, 40)));
        assert!(!rect.contains(VTermPos::new(24, 40)));
        assert!(!rect.contains(VTermPos::new(10, 80)));

        rect.move_by(5, 10);
        assert_eq!(rect.start_row, 5);
        assert_eq!(rect.end_row, 29);
        assert_eq!(rect.start_col, 10);
        assert_eq!(rect.end_col, 90);
    }

    #[test]
    fn test_vterm_color_rgb() {
        let color = VTermColor::rgb(255, 128, 64);
        assert!(color.is_rgb());
        assert!(!color.is_indexed());
        // SAFETY: We know this is an RGB color
        unsafe {
            assert_eq!(color.rgb.red, 255);
            assert_eq!(color.rgb.green, 128);
            assert_eq!(color.rgb.blue, 64);
        }
    }

    #[test]
    fn test_vterm_color_indexed() {
        let color = VTermColor::indexed(42);
        assert!(color.is_indexed());
        assert!(!color.is_rgb());
        // SAFETY: We know this is an indexed color
        unsafe {
            assert_eq!(color.indexed.idx, 42);
        }
    }

    #[test]
    fn test_screen_cell_attrs() {
        let mut attrs = VTermScreenCellAttrs::default();

        assert!(!attrs.bold());
        attrs.set_bold(true);
        assert!(attrs.bold());

        assert_eq!(attrs.underline(), 0);
        attrs.set_underline(2);
        assert_eq!(attrs.underline(), 2);

        assert!(!attrs.italic());
        attrs.set_italic(true);
        assert!(attrs.italic());

        assert_eq!(attrs.font(), 0);
        attrs.set_font(5);
        assert_eq!(attrs.font(), 5);

        assert_eq!(attrs.baseline(), 0);
        attrs.set_baseline(1);
        assert_eq!(attrs.baseline(), 1);

        // Verify all attributes are still correct
        assert!(attrs.bold());
        assert_eq!(attrs.underline(), 2);
        assert!(attrs.italic());
        assert_eq!(attrs.font(), 5);
        assert_eq!(attrs.baseline(), 1);
    }

    #[test]
    fn test_line_info() {
        let mut info = VTermLineInfo::default();

        assert!(!info.doublewidth());
        info.set_doublewidth(true);
        assert!(info.doublewidth());

        assert_eq!(info.doubleheight(), 0);
        info.set_doubleheight(1);
        assert_eq!(info.doubleheight(), 1);

        assert!(!info.continuation());
        info.set_continuation(true);
        assert!(info.continuation());
    }

    #[test]
    fn test_key_encoding_flags() {
        let mut flags = VTermKeyEncodingFlags::default();

        assert!(!flags.disambiguate());
        flags.set_disambiguate(true);
        assert!(flags.disambiguate());

        assert!(!flags.report_events());
        flags.set_report_events(true);
        assert!(flags.report_events());

        assert!(!flags.report_alternate());
        flags.set_report_alternate(true);
        assert!(flags.report_alternate());
    }

    #[test]
    fn test_key_encoding_stack() {
        let mut stack = VTermKeyEncodingStack::default();
        assert_eq!(stack.size, 1);

        let mut flags = VTermKeyEncodingFlags::default();
        flags.set_disambiguate(true);

        assert!(stack.push(flags));
        assert_eq!(stack.size, 2);
        assert!(stack.current().disambiguate());

        assert!(stack.pop());
        assert_eq!(stack.size, 1);
        assert!(!stack.current().disambiguate());

        // Can't pop below 1
        assert!(!stack.pop());
        assert_eq!(stack.size, 1);
    }

    #[test]
    fn test_string_fragment() {
        let frag = VTermStringFragment::new(ptr::null(), 100, true, false);
        assert_eq!(frag.len(), 100);
        assert!(frag.is_initial());
        assert!(!frag.is_final());

        let frag = VTermStringFragment::new(ptr::null(), 50, false, true);
        assert_eq!(frag.len(), 50);
        assert!(!frag.is_initial());
        assert!(frag.is_final());
    }

    #[test]
    fn test_csi_arg_functions() {
        assert!(csi_arg_is_missing(CSI_ARG_MISSING));
        assert!(!csi_arg_is_missing(42));

        assert_eq!(csi_arg_or(CSI_ARG_MISSING, 10), 10);
        assert_eq!(csi_arg_or(42, 10), 42);

        assert_eq!(csi_arg_count(CSI_ARG_MISSING), 1);
        assert_eq!(csi_arg_count(0), 1);
        assert_eq!(csi_arg_count(5), 5);

        let with_more = 42 | CSI_ARG_FLAG_MORE;
        assert!(csi_arg_has_more(with_more));
        assert_eq!(csi_arg(with_more), 42);
    }

    #[test]
    fn test_handles_null() {
        assert!(VTermHandle::null().is_null());
        assert!(VTermStateHandle::null().is_null());
        assert!(VTermScreenHandle::null().is_null());
    }

    #[test]
    fn test_get_attr_type() {
        assert_eq!(rs_vterm_get_attr_type(1), VTermValueType::Bool as c_int); // Bold
        assert_eq!(rs_vterm_get_attr_type(2), VTermValueType::Int as c_int); // Underline
        assert_eq!(rs_vterm_get_attr_type(9), VTermValueType::Color as c_int); // Foreground
        assert_eq!(rs_vterm_get_attr_type(99), 0); // Invalid
    }
}
