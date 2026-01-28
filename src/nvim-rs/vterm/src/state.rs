//! `VTerm` State Module
//!
//! This module implements the terminal state management for `VTerm`.
//! It tracks:
//! - Cursor position and mode
//! - Scroll regions
//! - Terminal modes (origin, autowrap, insert, etc.)
//! - Character encoding state
//! - Mouse state
//! - Pen (text attributes) state

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::trivially_copy_pass_by_ref)] // Consistency with C API and Rust conventions

use std::ffi::{c_int, c_void};

use crate::{
    VTermColor, VTermGlyphInfo, VTermKeyEncodingStack, VTermLineInfo, VTermPos, VTermRect,
    VTERM_BUFIDX_ALTSCREEN, VTERM_BUFIDX_PRIMARY, VTERM_MAX_SCHAR_SIZE,
};

// =============================================================================
// Mouse Protocol
// =============================================================================

/// Mouse protocol modes
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MouseProtocol {
    /// X10 compatibility mode
    #[default]
    X10 = 0,
    /// UTF-8 encoding
    Utf8 = 1,
    /// SGR encoding
    Sgr = 2,
    /// RXVT encoding
    Rxvt = 3,
}

// =============================================================================
// Mouse Flags
// =============================================================================

/// Mouse button tracking flags
pub mod mouse_flags {
    /// Want click events
    pub const WANT_CLICK: i32 = 0x01;
    /// Want drag events
    pub const WANT_DRAG: i32 = 0x02;
    /// Want move events
    pub const WANT_MOVE: i32 = 0x04;
}

// =============================================================================
// Selection State
// =============================================================================

/// Selection operation state
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectionState {
    #[default]
    Initial = 0,
    Selected = 1,
    Query = 2,
    SetInitial = 3,
    Set = 4,
    Invalid = 5,
}

/// Selection data state
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SelectionData {
    /// Selection mask
    pub mask: u16,
    /// Current selection state
    pub state: SelectionState,
    /// Partial receive data
    pub recv_partial: u32,
    /// Partial send data
    pub send_partial: u32,
}

// =============================================================================
// Terminal Modes
// =============================================================================

/// Terminal mode flags
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct TerminalModes {
    /// Packed mode bits
    bits: u16,
}

impl TerminalModes {
    const KEYPAD_BIT: u16 = 0;
    const CURSOR_BIT: u16 = 1;
    const AUTOWRAP_BIT: u16 = 2;
    const INSERT_BIT: u16 = 3;
    const NEWLINE_BIT: u16 = 4;
    const CURSOR_VISIBLE_BIT: u16 = 5;
    const CURSOR_BLINK_BIT: u16 = 6;
    const CURSOR_SHAPE_SHIFT: u16 = 7;
    const CURSOR_SHAPE_MASK: u16 = 0x3;
    const ALT_SCREEN_BIT: u16 = 9;
    const ORIGIN_BIT: u16 = 10;
    const SCREEN_BIT: u16 = 11;
    const LEFTRIGHTMARGIN_BIT: u16 = 12;
    const BRACKETPASTE_BIT: u16 = 13;
    const REPORT_FOCUS_BIT: u16 = 14;
    const THEME_UPDATES_BIT: u16 = 15;

    fn get_bit(&self, bit: u16) -> bool {
        (self.bits >> bit) & 1 != 0
    }

    fn set_bit(&mut self, bit: u16, v: bool) {
        if v {
            self.bits |= 1 << bit;
        } else {
            self.bits &= !(1 << bit);
        }
    }

    /// Keypad application mode
    #[inline]
    pub fn keypad(&self) -> bool {
        self.get_bit(Self::KEYPAD_BIT)
    }

    #[inline]
    pub fn set_keypad(&mut self, v: bool) {
        self.set_bit(Self::KEYPAD_BIT, v);
    }

    /// Cursor keys mode
    #[inline]
    pub fn cursor(&self) -> bool {
        self.get_bit(Self::CURSOR_BIT)
    }

    #[inline]
    pub fn set_cursor(&mut self, v: bool) {
        self.set_bit(Self::CURSOR_BIT, v);
    }

    /// Auto-wrap mode
    #[inline]
    pub fn autowrap(&self) -> bool {
        self.get_bit(Self::AUTOWRAP_BIT)
    }

    #[inline]
    pub fn set_autowrap(&mut self, v: bool) {
        self.set_bit(Self::AUTOWRAP_BIT, v);
    }

    /// Insert mode
    #[inline]
    pub fn insert(&self) -> bool {
        self.get_bit(Self::INSERT_BIT)
    }

    #[inline]
    pub fn set_insert(&mut self, v: bool) {
        self.set_bit(Self::INSERT_BIT, v);
    }

    /// Newline mode
    #[inline]
    pub fn newline(&self) -> bool {
        self.get_bit(Self::NEWLINE_BIT)
    }

    #[inline]
    pub fn set_newline(&mut self, v: bool) {
        self.set_bit(Self::NEWLINE_BIT, v);
    }

    /// Cursor visibility
    #[inline]
    pub fn cursor_visible(&self) -> bool {
        self.get_bit(Self::CURSOR_VISIBLE_BIT)
    }

    #[inline]
    pub fn set_cursor_visible(&mut self, v: bool) {
        self.set_bit(Self::CURSOR_VISIBLE_BIT, v);
    }

    /// Cursor blink
    #[inline]
    pub fn cursor_blink(&self) -> bool {
        self.get_bit(Self::CURSOR_BLINK_BIT)
    }

    #[inline]
    pub fn set_cursor_blink(&mut self, v: bool) {
        self.set_bit(Self::CURSOR_BLINK_BIT, v);
    }

    /// Cursor shape (0-3)
    #[inline]
    pub fn cursor_shape(&self) -> u8 {
        ((self.bits >> Self::CURSOR_SHAPE_SHIFT) & Self::CURSOR_SHAPE_MASK) as u8
    }

    #[inline]
    pub fn set_cursor_shape(&mut self, v: u8) {
        self.bits &= !(Self::CURSOR_SHAPE_MASK << Self::CURSOR_SHAPE_SHIFT);
        self.bits |= (u16::from(v) & Self::CURSOR_SHAPE_MASK) << Self::CURSOR_SHAPE_SHIFT;
    }

    /// Alternate screen mode
    #[inline]
    pub fn alt_screen(&self) -> bool {
        self.get_bit(Self::ALT_SCREEN_BIT)
    }

    #[inline]
    pub fn set_alt_screen(&mut self, v: bool) {
        self.set_bit(Self::ALT_SCREEN_BIT, v);
    }

    /// Origin mode
    #[inline]
    pub fn origin(&self) -> bool {
        self.get_bit(Self::ORIGIN_BIT)
    }

    #[inline]
    pub fn set_origin(&mut self, v: bool) {
        self.set_bit(Self::ORIGIN_BIT, v);
    }

    /// Screen mode
    #[inline]
    pub fn screen(&self) -> bool {
        self.get_bit(Self::SCREEN_BIT)
    }

    #[inline]
    pub fn set_screen(&mut self, v: bool) {
        self.set_bit(Self::SCREEN_BIT, v);
    }

    /// Left/right margin mode
    #[inline]
    pub fn leftrightmargin(&self) -> bool {
        self.get_bit(Self::LEFTRIGHTMARGIN_BIT)
    }

    #[inline]
    pub fn set_leftrightmargin(&mut self, v: bool) {
        self.set_bit(Self::LEFTRIGHTMARGIN_BIT, v);
    }

    /// Bracketed paste mode
    #[inline]
    pub fn bracketpaste(&self) -> bool {
        self.get_bit(Self::BRACKETPASTE_BIT)
    }

    #[inline]
    pub fn set_bracketpaste(&mut self, v: bool) {
        self.set_bit(Self::BRACKETPASTE_BIT, v);
    }

    /// Focus reporting
    #[inline]
    pub fn report_focus(&self) -> bool {
        self.get_bit(Self::REPORT_FOCUS_BIT)
    }

    #[inline]
    pub fn set_report_focus(&mut self, v: bool) {
        self.set_bit(Self::REPORT_FOCUS_BIT, v);
    }

    /// Theme updates
    #[inline]
    pub fn theme_updates(&self) -> bool {
        self.get_bit(Self::THEME_UPDATES_BIT)
    }

    #[inline]
    pub fn set_theme_updates(&mut self, v: bool) {
        self.set_bit(Self::THEME_UPDATES_BIT, v);
    }
}

// =============================================================================
// Saved State
// =============================================================================

/// Saved cursor modes (subset of terminal modes)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SavedModes {
    /// Packed mode bits
    bits: u8,
}

impl SavedModes {
    const CURSOR_VISIBLE_BIT: u8 = 0;
    const CURSOR_BLINK_BIT: u8 = 1;
    const CURSOR_SHAPE_SHIFT: u8 = 2;
    const CURSOR_SHAPE_MASK: u8 = 0x3;

    #[inline]
    pub fn cursor_visible(&self) -> bool {
        (self.bits >> Self::CURSOR_VISIBLE_BIT) & 1 != 0
    }

    #[inline]
    pub fn set_cursor_visible(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::CURSOR_VISIBLE_BIT;
        } else {
            self.bits &= !(1 << Self::CURSOR_VISIBLE_BIT);
        }
    }

    #[inline]
    pub fn cursor_blink(&self) -> bool {
        (self.bits >> Self::CURSOR_BLINK_BIT) & 1 != 0
    }

    #[inline]
    pub fn set_cursor_blink(&mut self, v: bool) {
        if v {
            self.bits |= 1 << Self::CURSOR_BLINK_BIT;
        } else {
            self.bits &= !(1 << Self::CURSOR_BLINK_BIT);
        }
    }

    #[inline]
    pub fn cursor_shape(&self) -> u8 {
        (self.bits >> Self::CURSOR_SHAPE_SHIFT) & Self::CURSOR_SHAPE_MASK
    }

    #[inline]
    pub fn set_cursor_shape(&mut self, v: u8) {
        self.bits &= !(Self::CURSOR_SHAPE_MASK << Self::CURSOR_SHAPE_SHIFT);
        self.bits |= (v & Self::CURSOR_SHAPE_MASK) << Self::CURSOR_SHAPE_SHIFT;
    }
}

// =============================================================================
// Pen State
// =============================================================================

/// Pen (text attribute) state
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Pen {
    /// Foreground color
    pub fg: VTermColor,
    /// Background color
    pub bg: VTermColor,
    /// URI/hyperlink index
    pub uri: c_int,
    /// Packed attribute bits
    attrs: u16,
}

impl Pen {
    const BOLD_BIT: u16 = 0;
    const UNDERLINE_SHIFT: u16 = 1;
    const UNDERLINE_MASK: u16 = 0x3;
    const ITALIC_BIT: u16 = 3;
    const BLINK_BIT: u16 = 4;
    const REVERSE_BIT: u16 = 5;
    const CONCEAL_BIT: u16 = 6;
    const STRIKE_BIT: u16 = 7;
    const FONT_SHIFT: u16 = 8;
    const FONT_MASK: u16 = 0xF;
    const SMALL_BIT: u16 = 12;
    const BASELINE_SHIFT: u16 = 13;
    const BASELINE_MASK: u16 = 0x3;

    fn get_bit(&self, bit: u16) -> bool {
        (self.attrs >> bit) & 1 != 0
    }

    fn set_bit(&mut self, bit: u16, v: bool) {
        if v {
            self.attrs |= 1 << bit;
        } else {
            self.attrs &= !(1 << bit);
        }
    }

    #[inline]
    pub fn bold(&self) -> bool {
        self.get_bit(Self::BOLD_BIT)
    }

    #[inline]
    pub fn set_bold(&mut self, v: bool) {
        self.set_bit(Self::BOLD_BIT, v);
    }

    #[inline]
    pub fn underline(&self) -> u8 {
        ((self.attrs >> Self::UNDERLINE_SHIFT) & Self::UNDERLINE_MASK) as u8
    }

    #[inline]
    pub fn set_underline(&mut self, v: u8) {
        self.attrs &= !(Self::UNDERLINE_MASK << Self::UNDERLINE_SHIFT);
        self.attrs |= (u16::from(v) & Self::UNDERLINE_MASK) << Self::UNDERLINE_SHIFT;
    }

    #[inline]
    pub fn italic(&self) -> bool {
        self.get_bit(Self::ITALIC_BIT)
    }

    #[inline]
    pub fn set_italic(&mut self, v: bool) {
        self.set_bit(Self::ITALIC_BIT, v);
    }

    #[inline]
    pub fn blink(&self) -> bool {
        self.get_bit(Self::BLINK_BIT)
    }

    #[inline]
    pub fn set_blink(&mut self, v: bool) {
        self.set_bit(Self::BLINK_BIT, v);
    }

    #[inline]
    pub fn reverse(&self) -> bool {
        self.get_bit(Self::REVERSE_BIT)
    }

    #[inline]
    pub fn set_reverse(&mut self, v: bool) {
        self.set_bit(Self::REVERSE_BIT, v);
    }

    #[inline]
    pub fn conceal(&self) -> bool {
        self.get_bit(Self::CONCEAL_BIT)
    }

    #[inline]
    pub fn set_conceal(&mut self, v: bool) {
        self.set_bit(Self::CONCEAL_BIT, v);
    }

    #[inline]
    pub fn strike(&self) -> bool {
        self.get_bit(Self::STRIKE_BIT)
    }

    #[inline]
    pub fn set_strike(&mut self, v: bool) {
        self.set_bit(Self::STRIKE_BIT, v);
    }

    #[inline]
    pub fn font(&self) -> u8 {
        ((self.attrs >> Self::FONT_SHIFT) & Self::FONT_MASK) as u8
    }

    #[inline]
    pub fn set_font(&mut self, v: u8) {
        self.attrs &= !(Self::FONT_MASK << Self::FONT_SHIFT);
        self.attrs |= (u16::from(v) & Self::FONT_MASK) << Self::FONT_SHIFT;
    }

    #[inline]
    pub fn small(&self) -> bool {
        self.get_bit(Self::SMALL_BIT)
    }

    #[inline]
    pub fn set_small(&mut self, v: bool) {
        self.set_bit(Self::SMALL_BIT, v);
    }

    #[inline]
    pub fn baseline(&self) -> u8 {
        ((self.attrs >> Self::BASELINE_SHIFT) & Self::BASELINE_MASK) as u8
    }

    #[inline]
    pub fn set_baseline(&mut self, v: u8) {
        self.attrs &= !(Self::BASELINE_MASK << Self::BASELINE_SHIFT);
        self.attrs |= (u16::from(v) & Self::BASELINE_MASK) << Self::BASELINE_SHIFT;
    }
}

// =============================================================================
// Saved State (cursor and pen)
// =============================================================================

/// Saved terminal state under DEC mode 1048/1049
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SavedState {
    /// Saved cursor position
    pub pos: VTermPos,
    /// Saved pen state
    pub pen: Pen,
    /// Saved modes
    pub mode: SavedModes,
}

// =============================================================================
// Temporary State
// =============================================================================

/// Temporary state for DECRQSS and selection parsing
#[repr(C)]
#[derive(Clone, Copy)]
pub union TempState {
    /// DECRQSS buffer
    pub decrqss: [i8; 4],
    /// Selection state
    pub selection: SelectionData,
}

impl Default for TempState {
    fn default() -> Self {
        Self { decrqss: [0; 4] }
    }
}

// =============================================================================
// Selection State
// =============================================================================

/// Selection callbacks
#[repr(C)]
pub struct VTermSelectionCallbacks {
    /// Set selection callback
    pub set: Option<
        unsafe extern "C" fn(
            mask: u32,
            frag: crate::VTermStringFragment,
            user: *mut c_void,
        ) -> c_int,
    >,
    /// Query selection callback
    pub query: Option<unsafe extern "C" fn(mask: u32, user: *mut c_void) -> c_int>,
}

/// Selection state
#[repr(C)]
pub struct SelectionInfo {
    /// Selection callbacks
    pub callbacks: *const VTermSelectionCallbacks,
    /// User data
    pub user: *mut c_void,
    /// Buffer for selection data
    pub buffer: *mut i8,
    /// Buffer length
    pub buflen: usize,
}

impl Default for SelectionInfo {
    fn default() -> Self {
        Self {
            callbacks: std::ptr::null(),
            user: std::ptr::null_mut(),
            buffer: std::ptr::null_mut(),
            buflen: 0,
        }
    }
}

// =============================================================================
// Encoding Instance
// =============================================================================

/// Character encoding instance
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EncodingInstance {
    /// Encoding implementation pointer
    pub enc: *mut c_void, // VTermEncoding*
    /// Encoding-specific state data
    pub data: [u8; 4 * 4], // 4 * sizeof(uint32_t)
}

impl Default for EncodingInstance {
    fn default() -> Self {
        Self {
            enc: std::ptr::null_mut(),
            data: [0; 16],
        }
    }
}

// =============================================================================
// Grapheme State
// =============================================================================

/// Grapheme clustering state
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GraphemeState {
    #[default]
    Initial = 0,
    // Additional states would be added based on Unicode grapheme cluster rules
}

// =============================================================================
// State Callbacks
// =============================================================================

/// Putglyph callback
pub type StatePutglyphCallback =
    unsafe extern "C" fn(info: *const VTermGlyphInfo, pos: VTermPos, user: *mut c_void) -> c_int;

/// Movecursor callback
pub type StateMovecursorCallback = unsafe extern "C" fn(
    pos: VTermPos,
    oldpos: VTermPos,
    visible: c_int,
    user: *mut c_void,
) -> c_int;

/// Scrollrect callback
pub type StateScrollrectCallback = unsafe extern "C" fn(
    rect: VTermRect,
    downward: c_int,
    rightward: c_int,
    user: *mut c_void,
) -> c_int;

/// Moverect callback
pub type StateMoverectCallback =
    unsafe extern "C" fn(dest: VTermRect, src: VTermRect, user: *mut c_void) -> c_int;

/// Erase callback
pub type StateEraseCallback =
    unsafe extern "C" fn(rect: VTermRect, selective: c_int, user: *mut c_void) -> c_int;

/// Initpen callback
pub type StateInitpenCallback = unsafe extern "C" fn(user: *mut c_void) -> c_int;

/// Setpenattr callback
pub type StateSetpenattrCallback =
    unsafe extern "C" fn(attr: c_int, val: *const crate::VTermValue, user: *mut c_void) -> c_int;

/// Settermprop callback
pub type StateSettermrpopCallback =
    unsafe extern "C" fn(prop: c_int, val: *const crate::VTermValue, user: *mut c_void) -> c_int;

/// Bell callback
pub type StateBellCallback = unsafe extern "C" fn(user: *mut c_void) -> c_int;

/// Resize callback
pub type StateResizeCallback = unsafe extern "C" fn(
    rows: c_int,
    cols: c_int,
    fields: *mut crate::VTermStateFields,
    user: *mut c_void,
) -> c_int;

/// Setlineinfo callback
pub type StateSetlineinfoCallback = unsafe extern "C" fn(
    row: c_int,
    newinfo: *const VTermLineInfo,
    oldinfo: *const VTermLineInfo,
    user: *mut c_void,
) -> c_int;

/// State callback function table
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VTermStateCallbacks {
    pub putglyph: Option<StatePutglyphCallback>,
    pub movecursor: Option<StateMovecursorCallback>,
    pub scrollrect: Option<StateScrollrectCallback>,
    pub moverect: Option<StateMoverectCallback>,
    pub erase: Option<StateEraseCallback>,
    pub initpen: Option<StateInitpenCallback>,
    pub setpenattr: Option<StateSetpenattrCallback>,
    pub settermprop: Option<StateSettermrpopCallback>,
    pub bell: Option<StateBellCallback>,
    pub resize: Option<StateResizeCallback>,
    pub setlineinfo: Option<StateSetlineinfoCallback>,
}

/// State fallback callbacks
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VTermStateFallbacks {
    pub control: Option<crate::parser::ParserControlCallback>,
    pub csi: Option<crate::parser::ParserCsiCallback>,
    pub osc: Option<crate::parser::ParserOscCallback>,
    pub dcs: Option<crate::parser::ParserDcsCallback>,
    pub apc: Option<crate::parser::ParserApcCallback>,
    pub pm: Option<crate::parser::ParserPmCallback>,
    pub sos: Option<crate::parser::ParserSosCallback>,
}

// =============================================================================
// Terminal State
// =============================================================================

/// Terminal state
#[repr(C)]
pub struct State {
    /// Parent `VTerm` handle
    pub vt: *mut c_void, // VTerm*

    /// State callbacks
    pub callbacks: *const VTermStateCallbacks,
    /// Callback user data
    pub cbdata: *mut c_void,

    /// Fallback callbacks
    pub fallbacks: *const VTermStateFallbacks,
    /// Fallback user data
    pub fbdata: *mut c_void,

    /// Number of rows
    pub rows: c_int,
    /// Number of columns
    pub cols: c_int,

    /// Current cursor position
    pub pos: VTermPos,

    /// At phantom column (for deferred wrap)
    pub at_phantom: c_int,

    /// Scroll region top
    pub scrollregion_top: c_int,
    /// Scroll region bottom (-1 = unbounded)
    pub scrollregion_bottom: c_int,
    /// Scroll region left
    pub scrollregion_left: c_int,
    /// Scroll region right (-1 = unbounded)
    pub scrollregion_right: c_int,

    /// Tab stops bitvector
    pub tabstops: *mut u8,

    /// Line info arrays (primary and altscreen)
    pub lineinfos: [*mut VTermLineInfo; 2],
    /// Current lineinfo (points to primary or altscreen)
    pub lineinfo: *mut VTermLineInfo,

    /// Mouse column
    pub mouse_col: c_int,
    /// Mouse row
    pub mouse_row: c_int,
    /// Mouse button state
    pub mouse_buttons: c_int,
    /// Mouse flags
    pub mouse_flags: c_int,

    /// Mouse protocol
    pub mouse_protocol: MouseProtocol,

    /// Grapheme buffer
    pub grapheme_buf: [i8; VTERM_MAX_SCHAR_SIZE],
    /// Grapheme buffer length
    pub grapheme_len: usize,
    /// Last grapheme codepoint
    pub grapheme_last: u32,
    /// Grapheme state
    pub grapheme_state: GraphemeState,
    /// Combine width
    pub combine_width: c_int,
    /// Combine position
    pub combine_pos: VTermPos,

    /// Terminal modes
    pub mode: TerminalModes,

    /// Encoding instances (GL, GR, G0-G3)
    pub encoding: [EncodingInstance; 4],
    /// UTF-8 encoding instance
    pub encoding_utf8: EncodingInstance,
    /// Current GL set
    pub gl_set: c_int,
    /// Current GR set
    pub gr_set: c_int,
    /// Single-shift set
    pub gsingle_set: c_int,

    /// Current pen state
    pub pen: Pen,

    /// Default foreground color
    pub default_fg: VTermColor,
    /// Default background color
    pub default_bg: VTermColor,
    /// 16-color palette
    pub colors: [VTermColor; 16],

    /// Bold renders as high-bright
    pub bold_is_highbright: c_int,

    /// Protected cell flag (DECSCA)
    pub protected_cell: u8,

    /// Saved state (DEC 1048/1049)
    pub saved: SavedState,

    /// Temporary state
    pub tmp: TempState,

    /// Selection info
    pub selection: SelectionInfo,

    /// Key encoding stacks (primary and altscreen)
    pub key_encoding_stacks: [VTermKeyEncodingStack; 2],
}

impl State {
    /// Get the effective scroll region bottom
    #[inline]
    pub fn scroll_region_bottom(&self) -> c_int {
        if self.scrollregion_bottom > -1 {
            self.scrollregion_bottom
        } else {
            self.rows
        }
    }

    /// Get the effective scroll region left
    #[inline]
    pub fn scroll_region_left(&self) -> c_int {
        if self.mode.leftrightmargin() {
            self.scrollregion_left
        } else {
            0
        }
    }

    /// Get the effective scroll region right
    #[inline]
    pub fn scroll_region_right(&self) -> c_int {
        if self.mode.leftrightmargin() && self.scrollregion_right > -1 {
            self.scrollregion_right
        } else {
            self.cols
        }
    }

    /// Get the row width (accounting for double-width lines)
    #[inline]
    pub unsafe fn row_width(&self, row: c_int) -> c_int {
        if (*self.lineinfo.add(row as usize)).doublewidth() {
            self.cols / 2
        } else {
            self.cols
        }
    }

    /// Get the current row width
    #[inline]
    pub unsafe fn this_row_width(&self) -> c_int {
        self.row_width(self.pos.row)
    }

    /// Check if using alternate screen
    #[inline]
    pub fn is_alt_screen(&self) -> bool {
        self.mode.alt_screen()
    }

    /// Get the current key encoding flags
    #[inline]
    pub fn current_key_encoding(&self) -> &crate::VTermKeyEncodingFlags {
        let idx = if self.is_alt_screen() {
            VTERM_BUFIDX_ALTSCREEN
        } else {
            VTERM_BUFIDX_PRIMARY
        };
        self.key_encoding_stacks[idx].current()
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Opaque handle to `VTermState`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VTermStateHandle(*mut c_void);

impl VTermStateHandle {
    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

/// FFI export: Get cursor row from state
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_cursorpos_row(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        (*s).pos.row
    }
}

/// FFI export: Get cursor column from state
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_cursorpos_col(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        (*s).pos.col
    }
}

/// FFI export: Get terminal rows from state
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_rows(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        (*s).rows
    }
}

/// FFI export: Get terminal cols from state
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_cols(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        (*s).cols
    }
}

/// FFI export: Check if autowrap is enabled
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_autowrap(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).mode.autowrap())
    }
}

/// FFI export: Check if cursor is visible
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_cursor_visible(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).mode.cursor_visible())
    }
}

/// FFI export: Get cursor shape
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_cursor_shape(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).mode.cursor_shape())
    }
}

/// FFI export: Check if cursor is blinking
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_cursor_blink(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).mode.cursor_blink())
    }
}

/// FFI export: Check if using alternate screen
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_altscreen(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).mode.alt_screen())
    }
}

/// FFI export: Get mouse protocol
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_mouse_protocol(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        (*s).mouse_protocol as c_int
    }
}

/// FFI export: Get mouse flags
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_mouse_flags(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        (*s).mouse_flags
    }
}

/// FFI export: Check if bracketed paste is enabled
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_bracketpaste(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).mode.bracketpaste())
    }
}

/// FFI export: Check if focus reporting is enabled
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_report_focus(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).mode.report_focus())
    }
}

/// FFI export: Get scroll region top
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_scroll_region_top(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        (*s).scrollregion_top
    }
}

/// FFI export: Get scroll region bottom
#[no_mangle]
pub extern "C" fn rs_vterm_state_get_scroll_region_bottom(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        (*s).scroll_region_bottom()
    }
}

/// FFI export: Get pen bold attribute
#[no_mangle]
pub extern "C" fn rs_vterm_state_pen_get_bold(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).pen.bold())
    }
}

/// FFI export: Get pen italic attribute
#[no_mangle]
pub extern "C" fn rs_vterm_state_pen_get_italic(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).pen.italic())
    }
}

/// FFI export: Get pen underline attribute
#[no_mangle]
pub extern "C" fn rs_vterm_state_pen_get_underline(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).pen.underline())
    }
}

/// FFI export: Get pen reverse attribute
#[no_mangle]
pub extern "C" fn rs_vterm_state_pen_get_reverse(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).pen.reverse())
    }
}

/// FFI export: Get pen strike attribute
#[no_mangle]
pub extern "C" fn rs_vterm_state_pen_get_strike(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).pen.strike())
    }
}

/// FFI export: Get pen font
#[no_mangle]
pub extern "C" fn rs_vterm_state_pen_get_font(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees state is valid
    unsafe {
        let s = state.0 as *const State;
        c_int::from((*s).pen.font())
    }
}

// =============================================================================
// C Accessor Function Declarations
// =============================================================================

// These are C accessor functions defined in state.c that provide access to
// VTermState fields. They may not all be used immediately but are declared
// for future use as more state functions are migrated to Rust.
#[allow(dead_code)]
extern "C" {
    // Dimension accessors
    fn nvim_vterm_state_get_rows(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_get_cols(state: VTermStateHandle) -> c_int;

    // Cursor position accessors
    fn nvim_vterm_state_get_pos(state: VTermStateHandle) -> VTermPos;
    fn nvim_vterm_state_set_pos(state: VTermStateHandle, pos: VTermPos);
    fn nvim_vterm_state_get_at_phantom(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_set_at_phantom(state: VTermStateHandle, at_phantom: c_int);

    // Scroll region accessors
    fn nvim_vterm_state_get_scrollregion_top(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_set_scrollregion_top(state: VTermStateHandle, top: c_int);
    fn nvim_vterm_state_get_scrollregion_bottom(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_set_scrollregion_bottom(state: VTermStateHandle, bottom: c_int);
    fn nvim_vterm_state_get_scrollregion_left(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_set_scrollregion_left(state: VTermStateHandle, left: c_int);
    fn nvim_vterm_state_get_scrollregion_right(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_set_scrollregion_right(state: VTermStateHandle, right: c_int);

    // Computed scroll region bounds
    fn nvim_vterm_state_scrollregion_bottom(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_scrollregion_left(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_scrollregion_right(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_row_width(state: VTermStateHandle, row: c_int) -> c_int;
    fn nvim_vterm_state_this_row_width(state: VTermStateHandle) -> c_int;

    // Line info accessors
    fn nvim_vterm_state_get_lineinfo_at(state: VTermStateHandle, row: c_int)
        -> *mut VTermLineInfo;
    fn nvim_vterm_state_set_lineinfo_continuation(
        state: VTermStateHandle,
        row: c_int,
        continuation: c_int,
    );

    // Mode accessors
    fn nvim_vterm_state_mode_autowrap(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_mode_insert(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_mode_newline(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_mode_origin(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_mode_cursor_visible(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_mode_leftrightmargin(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_mode_alt_screen(state: VTermStateHandle) -> c_int;

    // Protected cell accessor
    fn nvim_vterm_state_get_protected_cell(state: VTermStateHandle) -> c_int;

    // Callback accessors
    fn nvim_vterm_state_get_callbacks(state: VTermStateHandle) -> *const VTermStateCallbacks;
    fn nvim_vterm_state_get_cbdata(state: VTermStateHandle) -> *mut c_void;

    // VTerm handle accessor
    fn nvim_vterm_state_get_vt(state: VTermStateHandle) -> *mut c_void;

    // Grapheme buffer accessors
    fn nvim_vterm_state_get_grapheme_buf(state: VTermStateHandle) -> *mut i8;
    fn nvim_vterm_state_get_grapheme_len(state: VTermStateHandle) -> usize;
    fn nvim_vterm_state_set_grapheme_len(state: VTermStateHandle, len: usize);
    fn nvim_vterm_state_get_combine_width(state: VTermStateHandle) -> c_int;
    fn nvim_vterm_state_set_combine_width(state: VTermStateHandle, width: c_int);
    fn nvim_vterm_state_get_combine_pos(state: VTermStateHandle) -> VTermPos;
    fn nvim_vterm_state_set_combine_pos(state: VTermStateHandle, pos: VTermPos);

    // Lineinfo scroll helpers
    fn nvim_vterm_state_lineinfo_scroll_down(
        state: VTermStateHandle,
        start_row: c_int,
        end_row: c_int,
        count: c_int,
    );
    fn nvim_vterm_state_lineinfo_scroll_up(
        state: VTermStateHandle,
        start_row: c_int,
        end_row: c_int,
        count: c_int,
    );
    fn nvim_vterm_state_lineinfo_clear(state: VTermStateHandle, row: c_int);

    // Tabstop accessors
    fn nvim_vterm_state_is_col_tabstop(state: VTermStateHandle, col: c_int) -> c_int;
    fn nvim_vterm_state_set_col_tabstop(state: VTermStateHandle, col: c_int);
    fn nvim_vterm_state_clear_col_tabstop(state: VTermStateHandle, col: c_int);

    // VTerm scroll_rect helper
    fn nvim_vterm_scroll_rect(
        rect: VTermRect,
        downward: c_int,
        rightward: c_int,
        moverect: Option<StateMoverectCallback>,
        erase: Option<StateEraseCallback>,
        user: *mut c_void,
    );

    // VTerm output functions (existing C functions)
    fn vterm_push_output_sprintf_ctrl(vt: *mut c_void, c1: u8, fmt: *const i8, ...);
}

// =============================================================================
// Cursor Movement Functions (Migrated from C)
// =============================================================================

/// Update the cursor position and optionally notify via callback.
///
/// This function handles cursor position changes and calls the movecursor
/// callback if registered.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_updatecursor(
    state: VTermStateHandle,
    oldpos: *const VTermPos,
    cancel_phantom: c_int,
) {
    if state.is_null() {
        return;
    }

    let pos = nvim_vterm_state_get_pos(state);

    if cancel_phantom != 0 {
        nvim_vterm_state_set_at_phantom(state, 0);
    }

    let callbacks = nvim_vterm_state_get_callbacks(state);
    if callbacks.is_null() {
        return;
    }

    if let Some(movecursor) = (*callbacks).movecursor {
        let cbdata = nvim_vterm_state_get_cbdata(state);
        let cursor_visible = nvim_vterm_state_mode_cursor_visible(state);

        if oldpos.is_null() {
            // Use current position as old position
            movecursor(pos, pos, cursor_visible, cbdata);
        } else {
            movecursor(pos, *oldpos, cursor_visible, cbdata);
        }
    }
}

/// Set the cursor position directly.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_setpos(state: VTermStateHandle, pos: VTermPos) {
    if state.is_null() {
        return;
    }
    nvim_vterm_state_set_pos(state, pos);
}

/// Move cursor to absolute position (row, col).
/// Returns true if the cursor was within scroll region bounds.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_cursor_moveto(
    state: VTermStateHandle,
    row: c_int,
    col: c_int,
) -> c_int {
    if state.is_null() {
        return 0;
    }

    let rows = nvim_vterm_state_get_rows(state);
    let cols = nvim_vterm_state_get_cols(state);

    // Clamp position to valid bounds, tracking if we were out of bounds
    let (new_row, row_clamped) = if row < 0 {
        (0, true)
    } else if row >= rows {
        (rows - 1, true)
    } else {
        (row, false)
    };

    let (new_col, col_clamped) = if col < 0 {
        (0, true)
    } else if col >= cols {
        (cols - 1, true)
    } else {
        (col, false)
    };

    let new_pos = VTermPos {
        row: new_row,
        col: new_col,
    };
    nvim_vterm_state_set_pos(state, new_pos);

    c_int::from(!row_clamped && !col_clamped)
}

/// Scroll the terminal region.
/// `downward`: positive = scroll down (content moves up, new lines at bottom)
/// `rightward`: positive = scroll right (content moves left, new columns at right)
///
/// This is the full scroll implementation that:
/// 1. Clamps scroll amounts to region bounds
/// 2. Updates lineinfo for full-width scrolls
/// 3. Calls scrollrect callback if available
/// 4. Falls back to moverect/erase if scrollrect not handled
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_scroll(
    state: VTermStateHandle,
    rect: VTermRect,
    downward: c_int,
    rightward: c_int,
) {
    if state.is_null() || (downward == 0 && rightward == 0) {
        return;
    }

    // Clamp scroll amounts to region bounds
    let rows = rect.end_row - rect.start_row;
    let downward = downward.clamp(-rows, rows);

    let cols = rect.end_col - rect.start_col;
    let rightward = rightward.clamp(-cols, cols);

    // Update lineinfo if full-width vertical scroll
    let state_cols = nvim_vterm_state_get_cols(state);
    if rect.start_col == 0 && rect.end_col == state_cols && rightward == 0 {
        if downward > 0 {
            nvim_vterm_state_lineinfo_scroll_down(state, rect.start_row, rect.end_row, downward);
        } else if downward < 0 {
            nvim_vterm_state_lineinfo_scroll_up(state, rect.start_row, rect.end_row, -downward);
        }
    }

    // Try the scrollrect callback first
    let callbacks = nvim_vterm_state_get_callbacks(state);
    if !callbacks.is_null() {
        if let Some(scrollrect) = (*callbacks).scrollrect {
            let cbdata = nvim_vterm_state_get_cbdata(state);
            if scrollrect(rect, downward, rightward, cbdata) != 0 {
                return; // Callback handled it
            }
        }

        // Fallback to moverect/erase
        nvim_vterm_scroll_rect(
            rect,
            downward,
            rightward,
            (*callbacks).moverect,
            (*callbacks).erase,
            nvim_vterm_state_get_cbdata(state),
        );
    }
}

/// Check if the cursor is within the scroll region.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_cursor_in_scrollregion(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }

    let pos = nvim_vterm_state_get_pos(state);
    let scrollregion_top = nvim_vterm_state_get_scrollregion_top(state);
    let scrollregion_bottom = nvim_vterm_state_scrollregion_bottom(state);
    let scrollregion_left = nvim_vterm_state_scrollregion_left(state);
    let scrollregion_right = nvim_vterm_state_scrollregion_right(state);

    if pos.row < scrollregion_top || pos.row >= scrollregion_bottom {
        return 0;
    }
    if pos.col < scrollregion_left || pos.col >= scrollregion_right {
        return 0;
    }

    1
}

/// Perform a linefeed operation.
///
/// If the cursor is at the bottom of the scroll region, scrolls the region.
/// Otherwise, moves the cursor down one row.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_linefeed(state: VTermStateHandle) {
    if state.is_null() {
        return;
    }

    let pos = nvim_vterm_state_get_pos(state);
    let scrollregion_bottom = nvim_vterm_state_scrollregion_bottom(state);

    if pos.row == scrollregion_bottom - 1 {
        // At bottom of scroll region - scroll the region
        let scrollregion_top = nvim_vterm_state_get_scrollregion_top(state);
        let scrollregion_left = nvim_vterm_state_scrollregion_left(state);
        let scrollregion_right = nvim_vterm_state_scrollregion_right(state);

        let rect = VTermRect {
            start_row: scrollregion_top,
            end_row: scrollregion_bottom,
            start_col: scrollregion_left,
            end_col: scrollregion_right,
        };

        rs_vterm_state_scroll(state, rect, 1, 0);
    } else {
        let rows = nvim_vterm_state_get_rows(state);
        if pos.row < rows - 1 {
            let new_pos = VTermPos {
                row: pos.row + 1,
                col: pos.col,
            };
            nvim_vterm_state_set_pos(state, new_pos);
        }
    }
}

/// Set a column tabstop.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_set_tabstop(state: VTermStateHandle, col: c_int) {
    if state.is_null() {
        return;
    }
    nvim_vterm_state_set_col_tabstop(state, col);
}

/// Clear a column tabstop.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_clear_tabstop(state: VTermStateHandle, col: c_int) {
    if state.is_null() {
        return;
    }
    nvim_vterm_state_clear_col_tabstop(state, col);
}

/// Check if a column is a tabstop.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_is_tabstop(state: VTermStateHandle, col: c_int) -> c_int {
    if state.is_null() {
        return 0;
    }
    nvim_vterm_state_is_col_tabstop(state, col)
}

// Note: The following functions use the existing FFI exports that access
// the Rust State struct directly:
// - rs_vterm_state_get_scroll_region_top
// - rs_vterm_state_get_scroll_region_bottom
// - rs_vterm_state_get_cols
// - rs_vterm_state_get_rows
//
// The C accessor functions (nvim_vterm_state_*) are available for use
// in the scroll/cursor movement functions above.

/// Check if autowrap mode is enabled.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_is_autowrap(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    nvim_vterm_state_mode_autowrap(state)
}

/// Check if insert mode is enabled.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_is_insert_mode(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    nvim_vterm_state_mode_insert(state)
}

/// Check if newline mode is enabled.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_is_newline_mode(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    nvim_vterm_state_mode_newline(state)
}

/// Check if origin mode is enabled.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_is_origin_mode(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    nvim_vterm_state_mode_origin(state)
}

/// Check if the cell at current position is protected.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_is_protected(state: VTermStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    nvim_vterm_state_get_protected_cell(state)
}

// =============================================================================
// CSI Cursor Movement Commands (Migrated from C)
// =============================================================================

/// CUU - Cursor Up (CSI n A) - ECMA-48 8.3.22
///
/// Move cursor up by `count` rows. Clears phantom column.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_csi_cursor_up(state: VTermStateHandle, count: c_int) {
    if state.is_null() {
        return;
    }

    let pos = nvim_vterm_state_get_pos(state);
    let new_pos = VTermPos {
        row: pos.row - count,
        col: pos.col,
    };
    nvim_vterm_state_set_pos(state, new_pos);
    nvim_vterm_state_set_at_phantom(state, 0);
}

/// CUD - Cursor Down (CSI n B) - ECMA-48 8.3.19
///
/// Move cursor down by `count` rows. Clears phantom column.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_csi_cursor_down(state: VTermStateHandle, count: c_int) {
    if state.is_null() {
        return;
    }

    let pos = nvim_vterm_state_get_pos(state);
    let new_pos = VTermPos {
        row: pos.row + count,
        col: pos.col,
    };
    nvim_vterm_state_set_pos(state, new_pos);
    nvim_vterm_state_set_at_phantom(state, 0);
}

/// CUF - Cursor Forward (CSI n C) - ECMA-48 8.3.20
///
/// Move cursor forward (right) by `count` columns. Clears phantom column.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_csi_cursor_forward(state: VTermStateHandle, count: c_int) {
    if state.is_null() {
        return;
    }

    let pos = nvim_vterm_state_get_pos(state);
    let new_pos = VTermPos {
        row: pos.row,
        col: pos.col + count,
    };
    nvim_vterm_state_set_pos(state, new_pos);
    nvim_vterm_state_set_at_phantom(state, 0);
}

/// CUB - Cursor Back (CSI n D) - ECMA-48 8.3.18
///
/// Move cursor back (left) by `count` columns. Clears phantom column.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_csi_cursor_back(state: VTermStateHandle, count: c_int) {
    if state.is_null() {
        return;
    }

    let pos = nvim_vterm_state_get_pos(state);
    let new_pos = VTermPos {
        row: pos.row,
        col: pos.col - count,
    };
    nvim_vterm_state_set_pos(state, new_pos);
    nvim_vterm_state_set_at_phantom(state, 0);
}

/// CNL - Cursor Next Line (CSI n E) - ECMA-48 8.3.12
///
/// Move cursor to start of line `count` lines down. Clears phantom column.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_csi_cursor_next_line(
    state: VTermStateHandle,
    count: c_int,
) {
    if state.is_null() {
        return;
    }

    let pos = nvim_vterm_state_get_pos(state);
    let new_pos = VTermPos {
        row: pos.row + count,
        col: 0,
    };
    nvim_vterm_state_set_pos(state, new_pos);
    nvim_vterm_state_set_at_phantom(state, 0);
}

/// CPL - Cursor Previous Line (CSI n F) - ECMA-48 8.3.13
///
/// Move cursor to start of line `count` lines up. Clears phantom column.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_csi_cursor_prev_line(
    state: VTermStateHandle,
    count: c_int,
) {
    if state.is_null() {
        return;
    }

    let pos = nvim_vterm_state_get_pos(state);
    let new_pos = VTermPos {
        row: pos.row - count,
        col: 0,
    };
    nvim_vterm_state_set_pos(state, new_pos);
    nvim_vterm_state_set_at_phantom(state, 0);
}

/// CHA - Cursor Horizontal Absolute (CSI n G) - ECMA-48 8.3.9
///
/// Move cursor to column `col` (1-based). Clears phantom column.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_csi_cursor_horizontal_absolute(
    state: VTermStateHandle,
    col: c_int,
) {
    if state.is_null() {
        return;
    }

    let pos = nvim_vterm_state_get_pos(state);
    let new_pos = VTermPos {
        row: pos.row,
        col: col - 1, // Convert from 1-based to 0-based
    };
    nvim_vterm_state_set_pos(state, new_pos);
    nvim_vterm_state_set_at_phantom(state, 0);
}

/// CUP - Cursor Position (CSI row ; col H) - ECMA-48 8.3.21
///
/// Move cursor to absolute position (row, col), both 1-based.
/// When origin mode is set, positions are relative to scroll region.
/// Clears phantom column.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_csi_cursor_position(
    state: VTermStateHandle,
    row: c_int,
    col: c_int,
) {
    if state.is_null() {
        return;
    }

    // Convert from 1-based to 0-based
    let mut new_row = row - 1;
    let mut new_col = col - 1;

    // If origin mode is set, positions are relative to scroll region
    if nvim_vterm_state_mode_origin(state) != 0 {
        new_row += nvim_vterm_state_get_scrollregion_top(state);
        new_col += nvim_vterm_state_scrollregion_left(state);
    }

    let new_pos = VTermPos {
        row: new_row,
        col: new_col,
    };
    nvim_vterm_state_set_pos(state, new_pos);
    nvim_vterm_state_set_at_phantom(state, 0);
}

/// VPA - Vertical Line Position Absolute (CSI n d) - ECMA-48 8.3.158
///
/// Move cursor to row `row` (1-based), keeping column. Clears phantom column.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_csi_vertical_position_absolute(
    state: VTermStateHandle,
    row: c_int,
) {
    if state.is_null() {
        return;
    }

    let pos = nvim_vterm_state_get_pos(state);
    let mut new_row = row - 1; // Convert from 1-based to 0-based

    // If origin mode is set, positions are relative to scroll region
    if nvim_vterm_state_mode_origin(state) != 0 {
        new_row += nvim_vterm_state_get_scrollregion_top(state);
    }

    let new_pos = VTermPos {
        row: new_row,
        col: pos.col,
    };
    nvim_vterm_state_set_pos(state, new_pos);
    nvim_vterm_state_set_at_phantom(state, 0);
}

/// HVP - Horizontal and Vertical Position (CSI row ; col f) - ECMA-48 8.3.63
///
/// Same as CUP - move cursor to absolute position (row, col).
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_vterm_state_csi_hvp(
    state: VTermStateHandle,
    row: c_int,
    col: c_int,
) {
    rs_vterm_state_csi_cursor_position(state, row, col);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_modes() {
        let mut modes = TerminalModes::default();

        assert!(!modes.autowrap());
        modes.set_autowrap(true);
        assert!(modes.autowrap());

        assert!(!modes.cursor_visible());
        modes.set_cursor_visible(true);
        assert!(modes.cursor_visible());

        assert_eq!(modes.cursor_shape(), 0);
        modes.set_cursor_shape(2);
        assert_eq!(modes.cursor_shape(), 2);

        // Verify other bits not affected
        assert!(modes.autowrap());
        assert!(modes.cursor_visible());
    }

    #[test]
    fn test_pen() {
        let mut pen = Pen::default();

        assert!(!pen.bold());
        pen.set_bold(true);
        assert!(pen.bold());

        assert_eq!(pen.underline(), 0);
        pen.set_underline(2);
        assert_eq!(pen.underline(), 2);

        assert!(!pen.italic());
        pen.set_italic(true);
        assert!(pen.italic());

        assert_eq!(pen.font(), 0);
        pen.set_font(5);
        assert_eq!(pen.font(), 5);

        // Verify all bits preserved
        assert!(pen.bold());
        assert_eq!(pen.underline(), 2);
        assert!(pen.italic());
    }

    #[test]
    fn test_saved_modes() {
        let mut saved = SavedModes::default();

        assert!(!saved.cursor_visible());
        saved.set_cursor_visible(true);
        assert!(saved.cursor_visible());

        assert_eq!(saved.cursor_shape(), 0);
        saved.set_cursor_shape(1);
        assert_eq!(saved.cursor_shape(), 1);
    }

    #[test]
    fn test_mouse_protocol() {
        assert_eq!(MouseProtocol::default(), MouseProtocol::X10);
    }

    #[test]
    fn test_selection_state() {
        assert_eq!(SelectionState::default(), SelectionState::Initial);
    }
}
