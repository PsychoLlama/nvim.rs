//! Screen buffer implementation for `VTerm`
//!
//! This module provides the screen buffer management for terminal emulation,
//! including cell storage, damage tracking, and scrollback buffer support.

#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_int, c_void};

use crate::{
    SChar, VTermColor, VTermDamageSize, VTermPos, VTermProp, VTermRect, VTermScreenCell, VTermValue,
};

// =============================================================================
// Buffer Indices
// =============================================================================

/// Index for primary screen buffer
pub const BUFIDX_PRIMARY: usize = 0;

/// Index for alternate screen buffer
pub const BUFIDX_ALTSCREEN: usize = 1;

// =============================================================================
// Screen Pen
// =============================================================================

/// State of the pen at some moment in time, also used in a cell.
///
/// This is the internal representation used by the screen buffer.
/// It differs from `VTermScreenCellAttrs` in that it includes
/// additional state like URI and protected cell info.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ScreenPen {
    /// Foreground color
    pub fg: VTermColor,
    /// Background color
    pub bg: VTermColor,
    /// URI index (for hyperlinks)
    pub uri: c_int,
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
    /// - bit 12: small
    /// - bits 13-14: baseline (0-2)
    /// - bit 15: `protected_cell`
    /// - bit 16: dwl (double width line)
    /// - bits 17-18: dhl (double height line)
    bits: u32,
}

impl ScreenPen {
    // Bit positions for attributes
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
    const SMALL_BIT: u32 = 12;
    const BASELINE_SHIFT: u32 = 13;
    const BASELINE_MASK: u32 = 0x3;
    const PROTECTED_CELL_BIT: u32 = 15;
    const DWL_BIT: u32 = 16;
    const DHL_SHIFT: u32 = 17;
    const DHL_MASK: u32 = 0x3;

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

    /// Get protected cell flag
    #[inline]
    pub const fn protected_cell(&self) -> bool {
        (self.bits >> Self::PROTECTED_CELL_BIT) & 1 != 0
    }

    /// Set protected cell flag
    #[inline]
    pub fn set_protected_cell(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::PROTECTED_CELL_BIT;
        } else {
            self.bits &= !(1 << Self::PROTECTED_CELL_BIT);
        }
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
}

// =============================================================================
// Screen Cell
// =============================================================================

/// Internal representation of a screen cell.
///
/// This is the cell type used in the screen buffer, which is simpler than
/// `VTermScreenCell` (used for external API).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ScreenCell {
    /// The character in this cell (`schar_T` from Neovim)
    pub schar: SChar,
    /// Cell pen (attributes and colors)
    pub pen: ScreenPen,
}

impl ScreenCell {
    /// Create a new empty cell
    #[inline]
    pub const fn new() -> Self {
        Self {
            schar: 0,
            pen: ScreenPen {
                fg: VTermColor {
                    color_type: 0, // RGB black
                },
                bg: VTermColor {
                    color_type: 0, // RGB black
                },
                uri: 0,
                bits: 0,
            },
        }
    }

    /// Check if this is a continuation cell (part of a wide character)
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub const fn is_continuation(&self) -> bool {
        self.schar == u64::MAX
    }

    /// Mark this cell as a continuation cell
    #[inline]
    pub fn mark_continuation(&mut self) {
        self.schar = u64::MAX;
    }
}

// =============================================================================
// Screen Callbacks
// =============================================================================

/// Callback function types for screen events
pub type DamageCallback = unsafe extern "C" fn(rect: VTermRect, user: *mut c_void) -> c_int;

pub type MoverectCallback =
    unsafe extern "C" fn(dest: VTermRect, src: VTermRect, user: *mut c_void) -> c_int;

pub type MovecursorCallback = unsafe extern "C" fn(
    pos: VTermPos,
    oldpos: VTermPos,
    visible: c_int,
    user: *mut c_void,
) -> c_int;

pub type SettermpropCallback =
    unsafe extern "C" fn(prop: VTermProp, val: *mut VTermValue, user: *mut c_void) -> c_int;

pub type BellCallback = unsafe extern "C" fn(user: *mut c_void) -> c_int;

pub type ResizeCallback =
    unsafe extern "C" fn(rows: c_int, cols: c_int, user: *mut c_void) -> c_int;

pub type ThemeCallback = unsafe extern "C" fn(dark: *mut bool, user: *mut c_void) -> c_int;

pub type SbPushlineCallback =
    unsafe extern "C" fn(cols: c_int, cells: *const VTermScreenCell, user: *mut c_void) -> c_int;

pub type SbPoplineCallback =
    unsafe extern "C" fn(cols: c_int, cells: *mut VTermScreenCell, user: *mut c_void) -> c_int;

pub type SbClearCallback = unsafe extern "C" fn(user: *mut c_void) -> c_int;

/// Screen callback functions
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VTermScreenCallbacks {
    /// Called when screen damage occurs
    pub damage: Option<DamageCallback>,
    /// Called when a rectangle is moved/scrolled
    pub moverect: Option<MoverectCallback>,
    /// Called when cursor moves
    pub movecursor: Option<MovecursorCallback>,
    /// Called when a terminal property changes
    pub settermprop: Option<SettermpropCallback>,
    /// Called when bell is triggered
    pub bell: Option<BellCallback>,
    /// Called when screen is resized
    pub resize: Option<ResizeCallback>,
    /// Called when theme is requested
    pub theme: Option<ThemeCallback>,
    /// Called to push a line to scrollback buffer
    pub sb_pushline: Option<SbPushlineCallback>,
    /// Called to pop a line from scrollback buffer
    pub sb_popline: Option<SbPoplineCallback>,
    /// Called to clear the scrollback buffer
    pub sb_clear: Option<SbClearCallback>,
}

// =============================================================================
// Screen Structure
// =============================================================================

/// Screen buffer manager
///
/// Manages the screen buffer for terminal emulation, including
/// primary and alternate screen buffers, damage tracking, and scrolling.
#[repr(C)]
pub struct Screen {
    /// Pointer to the parent `VTerm` instance (opaque)
    pub vt: *mut c_void,
    /// Pointer to the terminal state (opaque)
    pub state: *mut c_void,

    /// Screen callbacks
    pub callbacks: *const VTermScreenCallbacks,
    /// Callback user data
    pub cbdata: *mut c_void,

    /// Damage merge mode
    pub damage_merge: VTermDamageSize,
    /// Current damaged region (`start_row` == -1 means no damage)
    pub damaged: VTermRect,
    /// Pending scroll rectangle
    pub pending_scrollrect: VTermRect,
    /// Pending scroll downward amount
    pub pending_scroll_downward: c_int,
    /// Pending scroll rightward amount
    pub pending_scroll_rightward: c_int,

    /// Number of rows
    pub rows: c_int,
    /// Number of columns
    pub cols: c_int,

    /// Global reverse video flag
    pub global_reverse: bool,
    /// Reflow mode enabled
    pub reflow: bool,

    /// Screen buffers (primary and altscreen)
    /// `buffers[1]` is lazily allocated when altscreen is enabled
    pub buffers: [*mut ScreenCell; 2],

    /// Current active buffer (points to `buffers[0]` or `buffers[1]`)
    pub buffer: *mut ScreenCell,

    /// Buffer for a single screen row used in scrollback storage callbacks
    pub sb_buffer: *mut VTermScreenCell,

    /// Current pen state
    pub pen: ScreenPen,
}

impl Screen {
    /// Create an uninitialized damaged rect marker
    #[inline]
    pub const fn no_damage_rect() -> VTermRect {
        VTermRect {
            start_row: -1,
            end_row: 0,
            start_col: 0,
            end_col: 0,
        }
    }

    /// Check if there is pending damage
    #[inline]
    pub const fn has_damage(&self) -> bool {
        self.damaged.start_row != -1
    }

    /// Check if there is a pending scroll
    #[inline]
    pub const fn has_pending_scroll(&self) -> bool {
        self.pending_scrollrect.start_row != -1
    }

    /// Check if altscreen is currently active
    #[inline]
    pub fn is_altscreen_active(&self) -> bool {
        !self.buffers[BUFIDX_ALTSCREEN].is_null() && self.buffer == self.buffers[BUFIDX_ALTSCREEN]
    }

    /// Get a cell at the given position
    ///
    /// Returns `None` if the position is out of bounds.
    ///
    /// # Safety
    /// The buffer must be valid and properly allocated.
    #[inline]
    pub unsafe fn get_cell(&self, row: c_int, col: c_int) -> Option<&ScreenCell> {
        if row < 0 || row >= self.rows || col < 0 || col >= self.cols {
            return None;
        }
        let idx = (row as usize) * (self.cols as usize) + (col as usize);
        Some(&*self.buffer.add(idx))
    }

    /// Get a mutable cell at the given position
    ///
    /// Returns `None` if the position is out of bounds.
    ///
    /// # Safety
    /// The buffer must be valid and properly allocated.
    #[inline]
    pub unsafe fn get_cell_mut(&mut self, row: c_int, col: c_int) -> Option<&mut ScreenCell> {
        if row < 0 || row >= self.rows || col < 0 || col >= self.cols {
            return None;
        }
        let idx = (row as usize) * (self.cols as usize) + (col as usize);
        Some(&mut *self.buffer.add(idx))
    }

    /// Clear a cell to the default state using current pen colors
    ///
    /// # Safety
    /// The cell pointer must be valid.
    #[inline]
    pub unsafe fn clear_cell(&self, cell: *mut ScreenCell) {
        (*cell).schar = 0;
        (*cell).pen = ScreenPen {
            fg: self.pen.fg,
            bg: self.pen.bg,
            uri: 0,
            bits: 0,
        };
    }
}

// =============================================================================
// Rectangle Utility Functions
// =============================================================================

/// Expand `dst` to contain `src` as well
#[inline]
pub fn rect_expand(dst: &mut VTermRect, src: &VTermRect) {
    if dst.start_row > src.start_row {
        dst.start_row = src.start_row;
    }
    if dst.start_col > src.start_col {
        dst.start_col = src.start_col;
    }
    if dst.end_row < src.end_row {
        dst.end_row = src.end_row;
    }
    if dst.end_col < src.end_col {
        dst.end_col = src.end_col;
    }
}

/// Clip `dst` to ensure it does not step outside of `bounds`
#[inline]
pub fn rect_clip(dst: &mut VTermRect, bounds: &VTermRect) {
    if dst.start_row < bounds.start_row {
        dst.start_row = bounds.start_row;
    }
    if dst.start_col < bounds.start_col {
        dst.start_col = bounds.start_col;
    }
    if dst.end_row > bounds.end_row {
        dst.end_row = bounds.end_row;
    }
    if dst.end_col > bounds.end_col {
        dst.end_col = bounds.end_col;
    }
    // Ensure it doesn't end up negatively-sized
    if dst.end_row < dst.start_row {
        dst.end_row = dst.start_row;
    }
    if dst.end_col < dst.start_col {
        dst.end_col = dst.start_col;
    }
}

/// Check if two rectangles are equal
#[inline]
pub const fn rect_equal(a: &VTermRect, b: &VTermRect) -> bool {
    a.start_row == b.start_row
        && a.start_col == b.start_col
        && a.end_row == b.end_row
        && a.end_col == b.end_col
}

/// Check if `big` entirely contains `small`
#[inline]
pub const fn rect_contains(big: &VTermRect, small: &VTermRect) -> bool {
    small.start_row >= big.start_row
        && small.start_col >= big.start_col
        && small.end_row <= big.end_row
        && small.end_col <= big.end_col
}

/// Check if two rectangles overlap at all
#[inline]
pub const fn rect_intersects(a: &VTermRect, b: &VTermRect) -> bool {
    !(a.start_row > b.end_row
        || b.start_row > a.end_row
        || a.start_col > b.end_col
        || b.start_col > a.end_col)
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Expand a rectangle to include another
#[no_mangle]
pub extern "C" fn rs_vterm_rect_expand(dst: *mut VTermRect, src: *const VTermRect) {
    if dst.is_null() || src.is_null() {
        return;
    }
    // SAFETY: Caller guarantees pointers are valid
    unsafe {
        rect_expand(&mut *dst, &*src);
    }
}

/// Clip a rectangle to bounds
#[no_mangle]
pub extern "C" fn rs_vterm_rect_clip(dst: *mut VTermRect, bounds: *const VTermRect) {
    if dst.is_null() || bounds.is_null() {
        return;
    }
    // SAFETY: Caller guarantees pointers are valid
    unsafe {
        rect_clip(&mut *dst, &*bounds);
    }
}

/// Check if two rectangles are equal
#[no_mangle]
pub extern "C" fn rs_vterm_rect_equal(a: *const VTermRect, b: *const VTermRect) -> c_int {
    if a.is_null() || b.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees pointers are valid
    c_int::from(unsafe { rect_equal(&*a, &*b) })
}

/// Check if big contains small
#[no_mangle]
pub extern "C" fn rs_vterm_rect_contains(big: *const VTermRect, small: *const VTermRect) -> c_int {
    if big.is_null() || small.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees pointers are valid
    c_int::from(unsafe { rect_contains(&*big, &*small) })
}

/// Check if two rectangles intersect
#[no_mangle]
pub extern "C" fn rs_vterm_rect_intersects(a: *const VTermRect, b: *const VTermRect) -> c_int {
    if a.is_null() || b.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees pointers are valid
    c_int::from(unsafe { rect_intersects(&*a, &*b) })
}

/// Get a cell from a screen at the given position
#[no_mangle]
pub extern "C" fn rs_vterm_screen_get_cell(
    screen: *const Screen,
    pos: VTermPos,
    cell: *mut VTermScreenCell,
) -> c_int {
    if screen.is_null() || cell.is_null() {
        return 0;
    }

    // SAFETY: Caller guarantees screen and cell are valid
    unsafe {
        let screen_ref = &*screen;
        let Some(int_cell) = screen_ref.get_cell(pos.row, pos.col) else {
            return 0;
        };

        // Copy internal cell to external representation
        let out = &mut *cell;

        // Handle continuation cells
        out.schar = if int_cell.is_continuation() {
            0
        } else {
            int_cell.schar
        };

        // Copy attributes with global_reverse XOR
        out.attrs.set_bold(int_cell.pen.bold());
        out.attrs.set_underline(int_cell.pen.underline());
        out.attrs.set_italic(int_cell.pen.italic());
        out.attrs.set_blink(int_cell.pen.blink());
        out.attrs
            .set_reverse(int_cell.pen.reverse() ^ screen_ref.global_reverse);
        out.attrs.set_conceal(int_cell.pen.conceal());
        out.attrs.set_strike(int_cell.pen.strike());
        out.attrs.set_font(int_cell.pen.font());
        out.attrs.set_small(int_cell.pen.small());
        out.attrs.set_baseline(int_cell.pen.baseline());
        out.attrs.set_dwl(int_cell.pen.dwl());
        out.attrs.set_dhl(int_cell.pen.dhl());

        out.fg = int_cell.pen.fg;
        out.bg = int_cell.pen.bg;
        out.uri = int_cell.pen.uri;

        // Determine cell width
        if pos.col < (screen_ref.cols - 1) {
            if let Some(next_cell) = screen_ref.get_cell(pos.row, pos.col + 1) {
                if next_cell.is_continuation() {
                    out.width = 2;
                } else {
                    out.width = 1;
                }
            } else {
                out.width = 1;
            }
        } else {
            out.width = 1;
        }

        1
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_pen_attributes() {
        let mut pen = ScreenPen::default();

        // Test all boolean attributes
        assert!(!pen.bold());
        pen.set_bold(true);
        assert!(pen.bold());

        assert!(!pen.italic());
        pen.set_italic(true);
        assert!(pen.italic());

        assert!(!pen.blink());
        pen.set_blink(true);
        assert!(pen.blink());

        assert!(!pen.reverse());
        pen.set_reverse(true);
        assert!(pen.reverse());

        assert!(!pen.conceal());
        pen.set_conceal(true);
        assert!(pen.conceal());

        assert!(!pen.strike());
        pen.set_strike(true);
        assert!(pen.strike());

        assert!(!pen.small());
        pen.set_small(true);
        assert!(pen.small());

        assert!(!pen.protected_cell());
        pen.set_protected_cell(true);
        assert!(pen.protected_cell());

        assert!(!pen.dwl());
        pen.set_dwl(true);
        assert!(pen.dwl());

        // Test multi-bit attributes
        assert_eq!(pen.underline(), 0);
        pen.set_underline(3);
        assert_eq!(pen.underline(), 3);

        assert_eq!(pen.font(), 0);
        pen.set_font(9);
        assert_eq!(pen.font(), 9);

        assert_eq!(pen.baseline(), 0);
        pen.set_baseline(2);
        assert_eq!(pen.baseline(), 2);

        assert_eq!(pen.dhl(), 0);
        pen.set_dhl(2);
        assert_eq!(pen.dhl(), 2);

        // Verify all attributes are still set correctly
        assert!(pen.bold());
        assert!(pen.italic());
        assert!(pen.blink());
        assert!(pen.reverse());
        assert_eq!(pen.underline(), 3);
        assert_eq!(pen.font(), 9);
    }

    #[test]
    fn test_screen_cell() {
        let cell = ScreenCell::new();
        assert_eq!(cell.schar, 0);
        assert!(!cell.is_continuation());

        let mut cell = ScreenCell::default();
        cell.mark_continuation();
        assert!(cell.is_continuation());
    }

    #[test]
    fn test_rect_expand() {
        let mut dst = VTermRect::new(5, 10, 5, 10);
        let src = VTermRect::new(3, 12, 2, 15);
        rect_expand(&mut dst, &src);
        assert_eq!(dst.start_row, 3);
        assert_eq!(dst.end_row, 12);
        assert_eq!(dst.start_col, 2);
        assert_eq!(dst.end_col, 15);
    }

    #[test]
    fn test_rect_clip() {
        let mut dst = VTermRect::new(0, 30, 0, 100);
        let bounds = VTermRect::new(5, 25, 10, 80);
        rect_clip(&mut dst, &bounds);
        assert_eq!(dst.start_row, 5);
        assert_eq!(dst.end_row, 25);
        assert_eq!(dst.start_col, 10);
        assert_eq!(dst.end_col, 80);
    }

    #[test]
    fn test_rect_equal() {
        let a = VTermRect::new(0, 10, 0, 20);
        let b = VTermRect::new(0, 10, 0, 20);
        let c = VTermRect::new(1, 10, 0, 20);

        assert!(rect_equal(&a, &b));
        assert!(!rect_equal(&a, &c));
    }

    #[test]
    fn test_rect_contains() {
        let big = VTermRect::new(0, 100, 0, 100);
        let small = VTermRect::new(10, 20, 10, 20);
        let outside = VTermRect::new(90, 110, 90, 110);

        assert!(rect_contains(&big, &small));
        assert!(!rect_contains(&big, &outside));
        assert!(!rect_contains(&small, &big));
    }

    #[test]
    fn test_rect_intersects() {
        let a = VTermRect::new(0, 10, 0, 10);
        let b = VTermRect::new(5, 15, 5, 15);
        let c = VTermRect::new(20, 30, 20, 30);

        assert!(rect_intersects(&a, &b));
        assert!(!rect_intersects(&a, &c));
    }

    #[test]
    fn test_no_damage_rect() {
        let rect = Screen::no_damage_rect();
        assert_eq!(rect.start_row, -1);
    }
}
