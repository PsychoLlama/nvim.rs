//! UI event protocol
//!
//! This module provides UI event serialization infrastructure
//! for communication between Neovim and UI clients.

use std::ffi::c_int;

// =============================================================================
// UI Event Type
// =============================================================================

/// Type of UI event.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiEventType {
    /// Unknown/invalid event
    #[default]
    Unknown = 0,
    /// Mode info set
    ModeInfoSet = 1,
    /// Update menu
    UpdateMenu = 2,
    /// Busy start
    BusyStart = 3,
    /// Busy stop
    BusyStop = 4,
    /// Mouse on
    MouseOn = 5,
    /// Mouse off
    MouseOff = 6,
    /// Mode change
    ModeChange = 7,
    /// Bell
    Bell = 8,
    /// Visual bell
    VisualBell = 9,
    /// Flush
    Flush = 10,
    /// Suspend
    Suspend = 11,
    /// Set title
    SetTitle = 12,
    /// Set icon
    SetIcon = 13,
    /// Screenshot
    Screenshot = 14,
    /// Option set
    OptionSet = 15,
    /// Default colors set
    DefaultColorsSet = 16,
    /// Highlight attr define
    HlAttrDefine = 17,
    /// Highlight group set
    HlGroupSet = 18,
    /// Grid resize
    GridResize = 19,
    /// Grid clear
    GridClear = 20,
    /// Grid cursor goto
    GridCursorGoto = 21,
    /// Grid line
    GridLine = 22,
    /// Grid scroll
    GridScroll = 23,
    /// Grid destroy
    GridDestroy = 24,
    /// Window position
    WinPos = 25,
    /// Window float position
    WinFloatPos = 26,
    /// Window external position
    WinExternalPos = 27,
    /// Window hide
    WinHide = 28,
    /// Window close
    WinClose = 29,
    /// Msg set position
    MsgSetPos = 30,
    /// Win viewport
    WinViewport = 31,
    /// Win viewport margins
    WinViewportMargins = 32,
    /// Win extmark
    WinExtmark = 33,
    /// Popup menu show
    PopupmenuShow = 34,
    /// Popup menu select
    PopupmenuSelect = 35,
    /// Popup menu hide
    PopupmenuHide = 36,
    /// Tabline update
    TablineUpdate = 37,
    /// Cmdline show
    CmdlineShow = 38,
    /// Cmdline position
    CmdlinePos = 39,
    /// Cmdline special char
    CmdlineSpecialChar = 40,
    /// Cmdline hide
    CmdlineHide = 41,
    /// Cmdline block show
    CmdlineBlockShow = 42,
    /// Cmdline block append
    CmdlineBlockAppend = 43,
    /// Cmdline block hide
    CmdlineBlockHide = 44,
    /// Wildmenu show
    WildmenuShow = 45,
    /// Wildmenu select
    WildmenuSelect = 46,
    /// Wildmenu hide
    WildmenuHide = 47,
    /// Message show
    MsgShow = 48,
    /// Message clear
    MsgClear = 49,
    /// Message show cursor
    MsgShowcursor = 50,
    /// Message history show
    MsgHistoryShow = 51,
    /// Message history clear
    MsgHistoryClear = 52,
    /// Message ruler
    MsgRuler = 53,
    /// Message showcmd
    MsgShowcmd = 54,
    /// Raw data (for forward compatibility)
    RawData = 100,
}

impl UiEventType {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::ModeInfoSet,
            2 => Self::UpdateMenu,
            3 => Self::BusyStart,
            4 => Self::BusyStop,
            5 => Self::MouseOn,
            6 => Self::MouseOff,
            7 => Self::ModeChange,
            8 => Self::Bell,
            9 => Self::VisualBell,
            10 => Self::Flush,
            11 => Self::Suspend,
            12 => Self::SetTitle,
            13 => Self::SetIcon,
            14 => Self::Screenshot,
            15 => Self::OptionSet,
            16 => Self::DefaultColorsSet,
            17 => Self::HlAttrDefine,
            18 => Self::HlGroupSet,
            19 => Self::GridResize,
            20 => Self::GridClear,
            21 => Self::GridCursorGoto,
            22 => Self::GridLine,
            23 => Self::GridScroll,
            24 => Self::GridDestroy,
            25 => Self::WinPos,
            26 => Self::WinFloatPos,
            27 => Self::WinExternalPos,
            28 => Self::WinHide,
            29 => Self::WinClose,
            30 => Self::MsgSetPos,
            31 => Self::WinViewport,
            32 => Self::WinViewportMargins,
            33 => Self::WinExtmark,
            34 => Self::PopupmenuShow,
            35 => Self::PopupmenuSelect,
            36 => Self::PopupmenuHide,
            37 => Self::TablineUpdate,
            38 => Self::CmdlineShow,
            39 => Self::CmdlinePos,
            40 => Self::CmdlineSpecialChar,
            41 => Self::CmdlineHide,
            42 => Self::CmdlineBlockShow,
            43 => Self::CmdlineBlockAppend,
            44 => Self::CmdlineBlockHide,
            45 => Self::WildmenuShow,
            46 => Self::WildmenuSelect,
            47 => Self::WildmenuHide,
            48 => Self::MsgShow,
            49 => Self::MsgClear,
            50 => Self::MsgShowcursor,
            51 => Self::MsgHistoryShow,
            52 => Self::MsgHistoryClear,
            53 => Self::MsgRuler,
            54 => Self::MsgShowcmd,
            100 => Self::RawData,
            _ => Self::Unknown,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this is a grid event.
    #[must_use]
    pub const fn is_grid_event(self) -> bool {
        matches!(
            self,
            Self::GridResize
                | Self::GridClear
                | Self::GridCursorGoto
                | Self::GridLine
                | Self::GridScroll
                | Self::GridDestroy
        )
    }

    /// Check if this is a window event.
    #[must_use]
    pub const fn is_window_event(self) -> bool {
        matches!(
            self,
            Self::WinPos
                | Self::WinFloatPos
                | Self::WinExternalPos
                | Self::WinHide
                | Self::WinClose
                | Self::WinViewport
                | Self::WinViewportMargins
                | Self::WinExtmark
        )
    }

    /// Check if this is a popup menu event.
    #[must_use]
    pub const fn is_popupmenu_event(self) -> bool {
        matches!(
            self,
            Self::PopupmenuShow | Self::PopupmenuSelect | Self::PopupmenuHide
        )
    }

    /// Check if this is a cmdline event.
    #[must_use]
    pub const fn is_cmdline_event(self) -> bool {
        matches!(
            self,
            Self::CmdlineShow
                | Self::CmdlinePos
                | Self::CmdlineSpecialChar
                | Self::CmdlineHide
                | Self::CmdlineBlockShow
                | Self::CmdlineBlockAppend
                | Self::CmdlineBlockHide
        )
    }

    /// Check if this is a message event.
    #[must_use]
    pub const fn is_message_event(self) -> bool {
        matches!(
            self,
            Self::MsgShow
                | Self::MsgClear
                | Self::MsgShowcursor
                | Self::MsgHistoryShow
                | Self::MsgHistoryClear
                | Self::MsgRuler
                | Self::MsgShowcmd
        )
    }
}

/// FFI: Check if grid event.
#[no_mangle]
pub extern "C" fn rs_ui_event_is_grid(event_type: c_int) -> c_int {
    c_int::from(UiEventType::from_c_int(event_type).is_grid_event())
}

/// FFI: Check if window event.
#[no_mangle]
pub extern "C" fn rs_ui_event_is_window(event_type: c_int) -> c_int {
    c_int::from(UiEventType::from_c_int(event_type).is_window_event())
}

// =============================================================================
// Event Priority
// =============================================================================

/// Priority of UI events.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiEventPriority {
    /// Low priority (decorations, etc.)
    Low = 0,
    /// Normal priority (most events)
    #[default]
    Normal = 1,
    /// High priority (cursor, mode changes)
    High = 2,
    /// Immediate (flush, suspend)
    Immediate = 3,
}

impl UiEventPriority {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Low,
            2 => Self::High,
            3 => Self::Immediate,
            _ => Self::Normal,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Event Header
// =============================================================================

/// Header for UI events.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UiEventHeader {
    /// Event type
    pub event_type: c_int,
    /// Priority
    pub priority: c_int,
    /// Sequence number
    pub seq: u64,
    /// Target grid (0 for global)
    pub grid: c_int,
    /// Timestamp (microseconds)
    pub timestamp_us: i64,
}

impl UiEventHeader {
    /// Create new event header.
    #[must_use]
    pub const fn new(event_type: UiEventType) -> Self {
        Self {
            event_type: event_type as c_int,
            priority: UiEventPriority::Normal as c_int,
            seq: 0,
            grid: 0,
            timestamp_us: 0,
        }
    }

    /// Create header for grid event.
    #[must_use]
    pub const fn for_grid(event_type: UiEventType, grid: c_int) -> Self {
        Self {
            event_type: event_type as c_int,
            priority: UiEventPriority::Normal as c_int,
            seq: 0,
            grid,
            timestamp_us: 0,
        }
    }

    /// Get event type.
    #[must_use]
    pub const fn get_event_type(&self) -> UiEventType {
        UiEventType::from_c_int(self.event_type)
    }

    /// Get priority.
    #[must_use]
    pub const fn get_priority(&self) -> UiEventPriority {
        UiEventPriority::from_c_int(self.priority)
    }
}

/// FFI: Create event header.
#[no_mangle]
pub extern "C" fn rs_ui_event_header_new(event_type: c_int) -> UiEventHeader {
    UiEventHeader::new(UiEventType::from_c_int(event_type))
}

/// FFI: Create event header for grid.
#[no_mangle]
pub extern "C" fn rs_ui_event_header_for_grid(event_type: c_int, grid: c_int) -> UiEventHeader {
    UiEventHeader::for_grid(UiEventType::from_c_int(event_type), grid)
}

// =============================================================================
// Grid Line Data
// =============================================================================

/// Grid line cell data.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GridLineCell {
    /// Character (UTF-8 encoded, up to 4 bytes)
    pub chars: [u8; 4],
    /// Character byte length
    pub char_len: u8,
    /// Highlight attribute ID
    pub hl_id: c_int,
    /// Repeat count (0 = no repeat)
    pub repeat: c_int,
}

impl GridLineCell {
    /// Create new cell with ASCII character.
    #[must_use]
    pub const fn ascii(ch: u8, hl_id: c_int) -> Self {
        Self {
            chars: [ch, 0, 0, 0],
            char_len: 1,
            hl_id,
            repeat: 0,
        }
    }

    /// Create new cell with repeat.
    #[must_use]
    pub const fn repeated(ch: u8, hl_id: c_int, repeat: c_int) -> Self {
        Self {
            chars: [ch, 0, 0, 0],
            char_len: 1,
            hl_id,
            repeat,
        }
    }

    /// Create space cell.
    #[must_use]
    pub const fn space(hl_id: c_int) -> Self {
        Self::ascii(b' ', hl_id)
    }
}

/// FFI: Create ASCII cell.
#[no_mangle]
pub extern "C" fn rs_grid_line_cell_ascii(ch: u8, hl_id: c_int) -> GridLineCell {
    GridLineCell::ascii(ch, hl_id)
}

/// FFI: Create repeated cell.
#[no_mangle]
pub extern "C" fn rs_grid_line_cell_repeated(ch: u8, hl_id: c_int, repeat: c_int) -> GridLineCell {
    GridLineCell::repeated(ch, hl_id, repeat)
}

// =============================================================================
// Scroll Region
// =============================================================================

/// Scroll region for grid scroll events.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ScrollRegion {
    /// Top row (inclusive)
    pub top: c_int,
    /// Bottom row (exclusive)
    pub bot: c_int,
    /// Left column (inclusive)
    pub left: c_int,
    /// Right column (exclusive)
    pub right: c_int,
    /// Rows to scroll (positive = down, negative = up)
    pub rows: c_int,
    /// Columns to scroll (positive = right, negative = left)
    pub cols: c_int,
}

impl ScrollRegion {
    /// Create new scroll region.
    #[must_use]
    pub const fn new(
        top: c_int,
        bot: c_int,
        left: c_int,
        right: c_int,
        rows: c_int,
        cols: c_int,
    ) -> Self {
        Self {
            top,
            bot,
            left,
            right,
            rows,
            cols,
        }
    }

    /// Create vertical scroll region.
    #[must_use]
    pub const fn vertical(top: c_int, bot: c_int, rows: c_int) -> Self {
        Self {
            top,
            bot,
            left: 0,
            right: 0,
            rows,
            cols: 0,
        }
    }

    /// Check if scroll is vertical only.
    #[must_use]
    pub const fn is_vertical(&self) -> bool {
        self.cols == 0
    }

    /// Check if region is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.top < self.bot && self.rows != 0
    }
}

/// FFI: Create scroll region.
#[no_mangle]
pub extern "C" fn rs_scroll_region_new(
    top: c_int,
    bot: c_int,
    left: c_int,
    right: c_int,
    rows: c_int,
    cols: c_int,
) -> ScrollRegion {
    ScrollRegion::new(top, bot, left, right, rows, cols)
}

/// FFI: Create vertical scroll region.
#[no_mangle]
pub extern "C" fn rs_scroll_region_vertical(top: c_int, bot: c_int, rows: c_int) -> ScrollRegion {
    ScrollRegion::vertical(top, bot, rows)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_event_type() {
        assert!(UiEventType::GridLine.is_grid_event());
        assert!(UiEventType::GridScroll.is_grid_event());
        assert!(!UiEventType::ModeChange.is_grid_event());

        assert!(UiEventType::WinPos.is_window_event());
        assert!(UiEventType::WinFloatPos.is_window_event());

        assert!(UiEventType::PopupmenuShow.is_popupmenu_event());
        assert!(UiEventType::CmdlineShow.is_cmdline_event());
        assert!(UiEventType::MsgShow.is_message_event());
    }

    #[test]
    fn test_event_header() {
        let header = UiEventHeader::new(UiEventType::GridLine);
        assert_eq!(header.get_event_type(), UiEventType::GridLine);
        assert_eq!(header.get_priority(), UiEventPriority::Normal);
        assert_eq!(header.grid, 0);

        let grid_header = UiEventHeader::for_grid(UiEventType::GridClear, 5);
        assert_eq!(grid_header.grid, 5);
    }

    #[test]
    fn test_grid_line_cell() {
        let cell = GridLineCell::ascii(b'A', 1);
        assert_eq!(cell.chars[0], b'A');
        assert_eq!(cell.char_len, 1);
        assert_eq!(cell.hl_id, 1);

        let repeated = GridLineCell::repeated(b' ', 0, 10);
        assert_eq!(repeated.repeat, 10);
    }

    #[test]
    fn test_scroll_region() {
        let region = ScrollRegion::new(5, 20, 0, 80, -3, 0);
        assert!(region.is_valid());
        assert!(region.is_vertical());

        let vert = ScrollRegion::vertical(0, 24, 5);
        assert!(vert.is_vertical());
        assert_eq!(vert.rows, 5);
    }
}
