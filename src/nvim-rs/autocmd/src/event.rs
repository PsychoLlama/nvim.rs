//! Autocommand event management helpers
//!
//! This module provides helpers for working with autocommand events,
//! including event classification, matching, and execution control.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int};

// C accessors for event name resolution and eventignore
extern "C" {
    fn nvim_event_name2nr(start: *const c_char, len: usize) -> c_int;
    fn nvim_get_event_sign(event: c_int) -> c_int;
    fn nvim_get_p_ei() -> *const c_char;
    #[link_name = "xmemdupz"]
    fn nvim_autocmd_xmemdupz(src: *const c_char, len: usize) -> *mut c_char;
    #[link_name = "xstrnsave"]
    fn nvim_autocmd_xstrnsave(src: *const c_char, len: usize) -> *mut c_char;
    #[link_name = "xfree"]
    fn nvim_autocmd_xfree(ptr: *mut c_char);
    fn nvim_autocmd_set_option_eventignore(val: *const c_char);
}

/// Result of event name resolution, returning both the event number
/// and a pointer past the parsed event name.
#[repr(C)]
pub struct EventNameResult {
    pub event: c_int,
    pub end_ptr: *const c_char,
}

// =============================================================================
// Event Categories
// =============================================================================

/// Categories of autocommand events.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventCategory {
    /// Unknown or invalid event
    #[default]
    Unknown = 0,
    /// Buffer-related events (BufEnter, BufLeave, etc.)
    Buffer = 1,
    /// File-related events (FileRead, FileWrite, etc.)
    File = 2,
    /// Window-related events (WinEnter, WinLeave, etc.)
    Window = 3,
    /// Tab-related events (TabEnter, TabLeave, etc.)
    Tab = 4,
    /// Cursor-related events (CursorMoved, CursorHold, etc.)
    Cursor = 5,
    /// Insert mode events (InsertEnter, InsertLeave, etc.)
    Insert = 6,
    /// Command-line events (CmdlineEnter, CmdlineLeave, etc.)
    Cmdline = 7,
    /// Terminal events (TermOpen, TermClose, etc.)
    Terminal = 8,
    /// UI events (ColorScheme, VimResized, etc.)
    Ui = 9,
    /// Session events (VimEnter, VimLeave, etc.)
    Session = 10,
    /// Text change events (TextChanged, TextYankPost, etc.)
    TextChange = 11,
    /// Completion events (CompleteChanged, CompleteDone, etc.)
    Completion = 12,
    /// User-defined events
    User = 13,
}

impl EventCategory {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Buffer,
            2 => Self::File,
            3 => Self::Window,
            4 => Self::Tab,
            5 => Self::Cursor,
            6 => Self::Insert,
            7 => Self::Cmdline,
            8 => Self::Terminal,
            9 => Self::Ui,
            10 => Self::Session,
            11 => Self::TextChange,
            12 => Self::Completion,
            13 => Self::User,
            _ => Self::Unknown,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Event Types (Complete Enum)
// =============================================================================

/// Total number of autocommand events.
pub const NUM_EVENTS: c_int = 141;

/// Autocommand event types.
///
/// This enum contains all autocommand event types supported by Neovim.
/// Values must match the C enum in `auevents_enum.generated.h`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    // Buffer events (0-22)
    BufAdd = 0,
    BufCreate = 1,
    BufDelete = 2,
    BufEnter = 3,
    BufFilePost = 4,
    BufFilePre = 5,
    BufHidden = 6,
    BufLeave = 7,
    BufModifiedSet = 8,
    BufNew = 9,
    BufNewFile = 10,
    BufRead = 11,
    BufReadCmd = 12,
    BufReadPost = 13,
    BufReadPre = 14,
    BufUnload = 15,
    BufWinEnter = 16,
    BufWinLeave = 17,
    BufWipeout = 18,
    BufWrite = 19,
    BufWriteCmd = 20,
    BufWritePost = 21,
    BufWritePre = 22,

    // Channel events (23-24)
    ChanInfo = 23,
    ChanOpen = 24,

    // Command-line events (25-31)
    CmdlineChanged = 25,
    CmdlineEnter = 26,
    CmdlineLeave = 27,
    CmdlineLeavePre = 28,
    CmdUndefined = 29,
    CmdWinEnter = 30,
    CmdWinLeave = 31,

    // Color/UI events (32-33)
    ColorScheme = 32,
    ColorSchemePre = 33,

    // Completion events (34-36)
    CompleteChanged = 34,
    CompleteDone = 35,
    CompleteDonePre = 36,

    // Cursor events (37-41)
    CursorHold = 37,
    CursorHoldI = 38,
    CursorMoved = 39,
    CursorMovedC = 40,
    CursorMovedI = 41,

    // Diagnostic events (42)
    DiagnosticChanged = 42,

    // Diff events (43)
    DiffUpdated = 43,

    // Directory events (44-45)
    DirChanged = 44,
    DirChangedPre = 45,

    // Encoding events (46)
    EncodingChanged = 46,

    // Exit events (47)
    ExitPre = 47,

    // File events (48-65)
    FileAppendCmd = 48,
    FileAppendPost = 49,
    FileAppendPre = 50,
    FileChangedRO = 51,
    FileChangedShell = 52,
    FileChangedShellPost = 53,
    FileEncoding = 54,
    FileReadCmd = 55,
    FileReadPost = 56,
    FileReadPre = 57,
    FileType = 58,
    FileWriteCmd = 59,
    FileWritePost = 60,
    FileWritePre = 61,
    FilterReadPost = 62,
    FilterReadPre = 63,
    FilterWritePost = 64,
    FilterWritePre = 65,

    // Focus events (66-67)
    FocusGained = 66,
    FocusLost = 67,

    // Function events (68)
    FuncUndefined = 68,

    // GUI events (69-70)
    GUIEnter = 69,
    GUIFailed = 70,

    // Insert mode events (71-75)
    InsertChange = 71,
    InsertCharPre = 72,
    InsertEnter = 73,
    InsertLeave = 74,
    InsertLeavePre = 75,

    // LSP events (76-81)
    LspAttach = 76,
    LspDetach = 77,
    LspNotify = 78,
    LspProgress = 79,
    LspRequest = 80,
    LspTokenUpdate = 81,

    // Menu events (82)
    MenuPopup = 82,

    // Mode events (83)
    ModeChanged = 83,

    // Option events (84)
    OptionSet = 84,

    // Pack events (85-86)
    PackChanged = 85,
    PackChangedPre = 86,

    // Progress events (87)
    Progress = 87,

    // Quickfix events (88-89)
    QuickFixCmdPost = 88,
    QuickFixCmdPre = 89,

    // Quit events (90)
    QuitPre = 90,

    // Recording events (91-92)
    RecordingEnter = 91,
    RecordingLeave = 92,

    // Remote events (93)
    RemoteReply = 93,

    // Safe state events (94)
    SafeState = 94,

    // Search events (95)
    SearchWrapped = 95,

    // Session events (96-97)
    SessionLoadPost = 96,
    SessionWritePost = 97,

    // Shell events (98-99)
    ShellCmdPost = 98,
    ShellFilterPost = 99,

    // Signal events (100)
    Signal = 100,

    // Source events (101-103)
    SourceCmd = 101,
    SourcePost = 102,
    SourcePre = 103,

    // Spell events (104)
    SpellFileMissing = 104,

    // Stdin events (105-106)
    StdinReadPost = 105,
    StdinReadPre = 106,

    // Swap events (107)
    SwapExists = 107,

    // Syntax events (108)
    Syntax = 108,

    // Tab events (109-113)
    TabClosed = 109,
    TabEnter = 110,
    TabLeave = 111,
    TabNew = 112,
    TabNewEntered = 113,

    // Terminal events (114-120)
    TermChanged = 114,
    TermClose = 115,
    TermEnter = 116,
    TermLeave = 117,
    TermOpen = 118,
    TermRequest = 119,
    TermResponse = 120,

    // Text events (121-125)
    TextChanged = 121,
    TextChangedI = 122,
    TextChangedP = 123,
    TextChangedT = 124,
    TextYankPost = 125,

    // UI events (126-127)
    UIEnter = 126,
    UILeave = 127,

    // User events (128)
    User = 128,

    // Vim events (129-134)
    VimEnter = 129,
    VimLeave = 130,
    VimLeavePre = 131,
    VimResized = 132,
    VimResume = 133,
    VimSuspend = 134,

    // Window events (135-140)
    WinClosed = 135,
    WinEnter = 136,
    WinLeave = 137,
    WinNew = 138,
    WinResized = 139,
    WinScrolled = 140,
}

impl Event {
    /// Create from raw C enum value.
    #[must_use]
    #[allow(clippy::missing_transmute_annotations)]
    pub const fn from_raw(value: c_int) -> Option<Self> {
        if value < 0 || value >= NUM_EVENTS {
            return None;
        }
        // SAFETY: We checked bounds above, and the repr(i32) ensures layout matches
        Some(unsafe { std::mem::transmute::<c_int, Self>(value) })
    }

    /// Convert to raw C enum value.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this event is valid.
    #[must_use]
    pub const fn is_valid(self) -> bool {
        (self as c_int) >= 0 && (self as c_int) < NUM_EVENTS
    }

    /// Get the category of this event.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub const fn category(self) -> EventCategory {
        match self {
            // Buffer events
            Self::BufAdd
            | Self::BufCreate
            | Self::BufDelete
            | Self::BufEnter
            | Self::BufFilePost
            | Self::BufFilePre
            | Self::BufHidden
            | Self::BufLeave
            | Self::BufModifiedSet
            | Self::BufNew
            | Self::BufNewFile
            | Self::BufRead
            | Self::BufReadCmd
            | Self::BufReadPost
            | Self::BufReadPre
            | Self::BufUnload
            | Self::BufWinEnter
            | Self::BufWinLeave
            | Self::BufWipeout
            | Self::BufWrite
            | Self::BufWriteCmd
            | Self::BufWritePost
            | Self::BufWritePre => EventCategory::Buffer,

            // File events
            Self::FileAppendCmd
            | Self::FileAppendPost
            | Self::FileAppendPre
            | Self::FileChangedRO
            | Self::FileChangedShell
            | Self::FileChangedShellPost
            | Self::FileEncoding
            | Self::FileReadCmd
            | Self::FileReadPost
            | Self::FileReadPre
            | Self::FileType
            | Self::FileWriteCmd
            | Self::FileWritePost
            | Self::FileWritePre
            | Self::FilterReadPost
            | Self::FilterReadPre
            | Self::FilterWritePost
            | Self::FilterWritePre => EventCategory::File,

            // Window events
            Self::WinClosed
            | Self::WinEnter
            | Self::WinLeave
            | Self::WinNew
            | Self::WinResized
            | Self::WinScrolled => EventCategory::Window,

            // Tab events
            Self::TabClosed
            | Self::TabEnter
            | Self::TabLeave
            | Self::TabNew
            | Self::TabNewEntered => EventCategory::Tab,

            // Cursor events
            Self::CursorHold
            | Self::CursorHoldI
            | Self::CursorMoved
            | Self::CursorMovedC
            | Self::CursorMovedI => EventCategory::Cursor,

            // Insert mode events
            Self::InsertChange
            | Self::InsertCharPre
            | Self::InsertEnter
            | Self::InsertLeave
            | Self::InsertLeavePre => EventCategory::Insert,

            // Cmdline events
            Self::CmdlineChanged
            | Self::CmdlineEnter
            | Self::CmdlineLeave
            | Self::CmdlineLeavePre
            | Self::CmdUndefined
            | Self::CmdWinEnter
            | Self::CmdWinLeave => EventCategory::Cmdline,

            // Terminal events
            Self::TermChanged
            | Self::TermClose
            | Self::TermEnter
            | Self::TermLeave
            | Self::TermOpen
            | Self::TermRequest
            | Self::TermResponse => EventCategory::Terminal,

            // UI events
            Self::ColorScheme
            | Self::ColorSchemePre
            | Self::FocusGained
            | Self::FocusLost
            | Self::GUIEnter
            | Self::GUIFailed
            | Self::UIEnter
            | Self::UILeave
            | Self::VimResized => EventCategory::Ui,

            // Session events
            Self::VimEnter
            | Self::VimLeave
            | Self::VimLeavePre
            | Self::VimResume
            | Self::VimSuspend
            | Self::ExitPre
            | Self::QuitPre
            | Self::SessionLoadPost
            | Self::SessionWritePost
            | Self::SafeState => EventCategory::Session,

            // Text change events
            Self::TextChanged
            | Self::TextChangedI
            | Self::TextChangedP
            | Self::TextChangedT
            | Self::TextYankPost => EventCategory::TextChange,

            // Completion events
            Self::CompleteChanged | Self::CompleteDone | Self::CompleteDonePre => {
                EventCategory::Completion
            }

            // User events
            Self::User => EventCategory::User,

            // Everything else categorized as Unknown
            _ => EventCategory::Unknown,
        }
    }

    /// Get the event name as a static string.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub const fn name(self) -> &'static str {
        match self {
            Self::BufAdd => "BufAdd",
            Self::BufCreate => "BufCreate",
            Self::BufDelete => "BufDelete",
            Self::BufEnter => "BufEnter",
            Self::BufFilePost => "BufFilePost",
            Self::BufFilePre => "BufFilePre",
            Self::BufHidden => "BufHidden",
            Self::BufLeave => "BufLeave",
            Self::BufModifiedSet => "BufModifiedSet",
            Self::BufNew => "BufNew",
            Self::BufNewFile => "BufNewFile",
            Self::BufRead => "BufRead",
            Self::BufReadCmd => "BufReadCmd",
            Self::BufReadPost => "BufReadPost",
            Self::BufReadPre => "BufReadPre",
            Self::BufUnload => "BufUnload",
            Self::BufWinEnter => "BufWinEnter",
            Self::BufWinLeave => "BufWinLeave",
            Self::BufWipeout => "BufWipeout",
            Self::BufWrite => "BufWrite",
            Self::BufWriteCmd => "BufWriteCmd",
            Self::BufWritePost => "BufWritePost",
            Self::BufWritePre => "BufWritePre",
            Self::ChanInfo => "ChanInfo",
            Self::ChanOpen => "ChanOpen",
            Self::CmdlineChanged => "CmdlineChanged",
            Self::CmdlineEnter => "CmdlineEnter",
            Self::CmdlineLeave => "CmdlineLeave",
            Self::CmdlineLeavePre => "CmdlineLeavePre",
            Self::CmdUndefined => "CmdUndefined",
            Self::CmdWinEnter => "CmdWinEnter",
            Self::CmdWinLeave => "CmdWinLeave",
            Self::ColorScheme => "ColorScheme",
            Self::ColorSchemePre => "ColorSchemePre",
            Self::CompleteChanged => "CompleteChanged",
            Self::CompleteDone => "CompleteDone",
            Self::CompleteDonePre => "CompleteDonePre",
            Self::CursorHold => "CursorHold",
            Self::CursorHoldI => "CursorHoldI",
            Self::CursorMoved => "CursorMoved",
            Self::CursorMovedC => "CursorMovedC",
            Self::CursorMovedI => "CursorMovedI",
            Self::DiagnosticChanged => "DiagnosticChanged",
            Self::DiffUpdated => "DiffUpdated",
            Self::DirChanged => "DirChanged",
            Self::DirChangedPre => "DirChangedPre",
            Self::EncodingChanged => "EncodingChanged",
            Self::ExitPre => "ExitPre",
            Self::FileAppendCmd => "FileAppendCmd",
            Self::FileAppendPost => "FileAppendPost",
            Self::FileAppendPre => "FileAppendPre",
            Self::FileChangedRO => "FileChangedRO",
            Self::FileChangedShell => "FileChangedShell",
            Self::FileChangedShellPost => "FileChangedShellPost",
            Self::FileEncoding => "FileEncoding",
            Self::FileReadCmd => "FileReadCmd",
            Self::FileReadPost => "FileReadPost",
            Self::FileReadPre => "FileReadPre",
            Self::FileType => "FileType",
            Self::FileWriteCmd => "FileWriteCmd",
            Self::FileWritePost => "FileWritePost",
            Self::FileWritePre => "FileWritePre",
            Self::FilterReadPost => "FilterReadPost",
            Self::FilterReadPre => "FilterReadPre",
            Self::FilterWritePost => "FilterWritePost",
            Self::FilterWritePre => "FilterWritePre",
            Self::FocusGained => "FocusGained",
            Self::FocusLost => "FocusLost",
            Self::FuncUndefined => "FuncUndefined",
            Self::GUIEnter => "GUIEnter",
            Self::GUIFailed => "GUIFailed",
            Self::InsertChange => "InsertChange",
            Self::InsertCharPre => "InsertCharPre",
            Self::InsertEnter => "InsertEnter",
            Self::InsertLeave => "InsertLeave",
            Self::InsertLeavePre => "InsertLeavePre",
            Self::LspAttach => "LspAttach",
            Self::LspDetach => "LspDetach",
            Self::LspNotify => "LspNotify",
            Self::LspProgress => "LspProgress",
            Self::LspRequest => "LspRequest",
            Self::LspTokenUpdate => "LspTokenUpdate",
            Self::MenuPopup => "MenuPopup",
            Self::ModeChanged => "ModeChanged",
            Self::OptionSet => "OptionSet",
            Self::PackChanged => "PackChanged",
            Self::PackChangedPre => "PackChangedPre",
            Self::Progress => "Progress",
            Self::QuickFixCmdPost => "QuickFixCmdPost",
            Self::QuickFixCmdPre => "QuickFixCmdPre",
            Self::QuitPre => "QuitPre",
            Self::RecordingEnter => "RecordingEnter",
            Self::RecordingLeave => "RecordingLeave",
            Self::RemoteReply => "RemoteReply",
            Self::SafeState => "SafeState",
            Self::SearchWrapped => "SearchWrapped",
            Self::SessionLoadPost => "SessionLoadPost",
            Self::SessionWritePost => "SessionWritePost",
            Self::ShellCmdPost => "ShellCmdPost",
            Self::ShellFilterPost => "ShellFilterPost",
            Self::Signal => "Signal",
            Self::SourceCmd => "SourceCmd",
            Self::SourcePost => "SourcePost",
            Self::SourcePre => "SourcePre",
            Self::SpellFileMissing => "SpellFileMissing",
            Self::StdinReadPost => "StdinReadPost",
            Self::StdinReadPre => "StdinReadPre",
            Self::SwapExists => "SwapExists",
            Self::Syntax => "Syntax",
            Self::TabClosed => "TabClosed",
            Self::TabEnter => "TabEnter",
            Self::TabLeave => "TabLeave",
            Self::TabNew => "TabNew",
            Self::TabNewEntered => "TabNewEntered",
            Self::TermChanged => "TermChanged",
            Self::TermClose => "TermClose",
            Self::TermEnter => "TermEnter",
            Self::TermLeave => "TermLeave",
            Self::TermOpen => "TermOpen",
            Self::TermRequest => "TermRequest",
            Self::TermResponse => "TermResponse",
            Self::TextChanged => "TextChanged",
            Self::TextChangedI => "TextChangedI",
            Self::TextChangedP => "TextChangedP",
            Self::TextChangedT => "TextChangedT",
            Self::TextYankPost => "TextYankPost",
            Self::UIEnter => "UIEnter",
            Self::UILeave => "UILeave",
            Self::User => "User",
            Self::VimEnter => "VimEnter",
            Self::VimLeave => "VimLeave",
            Self::VimLeavePre => "VimLeavePre",
            Self::VimResized => "VimResized",
            Self::VimResume => "VimResume",
            Self::VimSuspend => "VimSuspend",
            Self::WinClosed => "WinClosed",
            Self::WinEnter => "WinEnter",
            Self::WinLeave => "WinLeave",
            Self::WinNew => "WinNew",
            Self::WinResized => "WinResized",
            Self::WinScrolled => "WinScrolled",
        }
    }
}

// Note: Default is explicitly implemented rather than derived to document
// that BufAdd (0) is the default event value
#[allow(clippy::derivable_impls)]
impl Default for Event {
    fn default() -> Self {
        Self::BufAdd
    }
}

// =============================================================================
// Event FFI Exports
// =============================================================================

/// Check if an event value is valid.
#[unsafe(no_mangle)]
#[allow(clippy::manual_range_contains)]
pub extern "C" fn rs_event_valid(event: c_int) -> c_int {
    c_int::from(event >= 0 && event < NUM_EVENTS)
}

/// Get the category of an event.
#[unsafe(no_mangle)]
pub extern "C" fn rs_event_category(event: c_int) -> c_int {
    Event::from_raw(event).map_or(EventCategory::Unknown.to_raw(), |e| e.category().to_raw())
}

/// Get the number of events.
#[unsafe(no_mangle)]
pub extern "C" fn rs_num_events() -> c_int {
    NUM_EVENTS
}

// =============================================================================
// Event Flags
// =============================================================================

/// Flags for autocommand event execution.
pub mod event_flags {
    use std::ffi::c_int;

    /// Event is currently being executed
    pub const AU_EXECUTING: c_int = 0x01;
    /// Event should be nested (allow triggering more autocmds)
    pub const AU_NESTED: c_int = 0x02;
    /// Event execution was blocked
    pub const AU_BLOCKED: c_int = 0x04;
    /// Event is from a pattern match
    pub const AU_PATTERN: c_int = 0x08;
    /// Event is buffer-local
    pub const AU_BUFLOCAL: c_int = 0x10;
    /// Event is once-only (delete after execution)
    pub const AU_ONCE: c_int = 0x20;
}

/// Check if event flags have a specific flag set.
#[must_use]
#[inline]
pub const fn has_event_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set an event flag.
#[must_use]
#[inline]
pub const fn set_event_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear an event flag.
#[must_use]
#[inline]
pub const fn clear_event_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

// =============================================================================
// Event State
// =============================================================================

/// State for autocommand event execution.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct EventState {
    /// Current event number
    pub event: c_int,
    /// Event execution flags
    pub flags: c_int,
    /// Nesting level (how many autocmds deep)
    pub nesting_level: c_int,
    /// Buffer number for buffer-local events
    pub bufnr: c_int,
}

impl EventState {
    /// Create a new event state.
    #[must_use]
    pub const fn new(event: c_int) -> Self {
        Self {
            event,
            flags: 0,
            nesting_level: 0,
            bufnr: 0,
        }
    }

    /// Check if event is executing.
    #[must_use]
    pub const fn is_executing(&self) -> bool {
        has_event_flag(self.flags, event_flags::AU_EXECUTING)
    }

    /// Check if event is nested.
    #[must_use]
    pub const fn is_nested(&self) -> bool {
        has_event_flag(self.flags, event_flags::AU_NESTED)
    }

    /// Check if event is blocked.
    #[must_use]
    pub const fn is_blocked(&self) -> bool {
        has_event_flag(self.flags, event_flags::AU_BLOCKED)
    }

    /// Check if event is buffer-local.
    #[must_use]
    pub const fn is_buflocal(&self) -> bool {
        has_event_flag(self.flags, event_flags::AU_BUFLOCAL)
    }

    /// Check if event is once-only.
    #[must_use]
    pub const fn is_once(&self) -> bool {
        has_event_flag(self.flags, event_flags::AU_ONCE)
    }

    /// Set executing flag.
    pub fn set_executing(&mut self, executing: bool) {
        if executing {
            self.flags = set_event_flag(self.flags, event_flags::AU_EXECUTING);
        } else {
            self.flags = clear_event_flag(self.flags, event_flags::AU_EXECUTING);
        }
    }

    /// Set nested flag.
    pub fn set_nested(&mut self, nested: bool) {
        if nested {
            self.flags = set_event_flag(self.flags, event_flags::AU_NESTED);
        } else {
            self.flags = clear_event_flag(self.flags, event_flags::AU_NESTED);
        }
    }

    /// Increment nesting level.
    pub fn push_nesting(&mut self) {
        self.nesting_level += 1;
    }

    /// Decrement nesting level.
    pub fn pop_nesting(&mut self) {
        if self.nesting_level > 0 {
            self.nesting_level -= 1;
        }
    }
}

// =============================================================================
// Nesting Control
// =============================================================================

/// Maximum allowed nesting depth for autocommands.
pub const MAX_NESTING_DEPTH: c_int = 10;

/// Check if more nesting is allowed.
#[must_use]
pub const fn can_nest(current_depth: c_int) -> bool {
    current_depth < MAX_NESTING_DEPTH
}

/// Result of nesting check.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestingResult {
    /// Nesting is allowed
    Allowed = 0,
    /// Nesting is not allowed (++nested flag not set)
    NotNested = 1,
    /// Nesting depth exceeded
    TooDeep = 2,
    /// Event is blocked
    Blocked = 3,
}

impl NestingResult {
    /// Check if nesting is allowed.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }
}

/// Check if an event can trigger nested autocommands.
#[must_use]
pub const fn check_nesting(state: &EventState) -> NestingResult {
    if state.is_blocked() {
        return NestingResult::Blocked;
    }
    if !state.is_nested() {
        return NestingResult::NotNested;
    }
    if !can_nest(state.nesting_level) {
        return NestingResult::TooDeep;
    }
    NestingResult::Allowed
}

// =============================================================================
// Event Matching
// =============================================================================

/// Result of event matching.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct EventMatch {
    /// Whether a match was found
    pub matched: bool,
    /// Event number that matched (if any)
    pub event: c_int,
    /// Match score (higher is better)
    pub score: c_int,
}

impl EventMatch {
    /// Create a non-match.
    #[must_use]
    pub const fn no_match() -> Self {
        Self {
            matched: false,
            event: -1,
            score: 0,
        }
    }

    /// Create a match.
    #[must_use]
    pub const fn matched(event: c_int, score: c_int) -> Self {
        Self {
            matched: true,
            event,
            score,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get event category from event number.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_event_category(event: c_int) -> c_int {
    // Event ranges based on auevents_enum.generated.h
    // Buffer events: 0-20
    // File events: 21-35
    // Cursor events: 36-40
    // etc.
    let category = if event < 0 {
        EventCategory::Unknown
    } else if event <= 20 {
        EventCategory::Buffer
    } else if event <= 35 {
        EventCategory::File
    } else if event <= 40 {
        EventCategory::Cursor
    } else if event <= 50 {
        EventCategory::Insert
    } else if event <= 60 {
        EventCategory::Cmdline
    } else if event <= 70 {
        EventCategory::Window
    } else if event <= 80 {
        EventCategory::Tab
    } else if event <= 90 {
        EventCategory::Terminal
    } else if event <= 100 {
        EventCategory::TextChange
    } else if event <= 110 {
        EventCategory::Completion
    } else if event <= 130 {
        EventCategory::Session
    } else if event <= 140 {
        EventCategory::Ui
    } else {
        EventCategory::User
    };
    category.to_raw()
}

/// Check if event flags have a specific flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_event_has_flag(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_event_flag(flags, flag))
}

/// Set an event flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_event_set_flag(flags: c_int, flag: c_int) -> c_int {
    set_event_flag(flags, flag)
}

/// Clear an event flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_event_clear_flag(flags: c_int, flag: c_int) -> c_int {
    clear_event_flag(flags, flag)
}

/// Check if more nesting is allowed.
#[unsafe(no_mangle)]
pub extern "C" fn rs_can_nest(current_depth: c_int) -> c_int {
    c_int::from(can_nest(current_depth))
}

/// Get maximum nesting depth.
#[unsafe(no_mangle)]
pub extern "C" fn rs_max_nesting_depth() -> c_int {
    MAX_NESTING_DEPTH
}

// =============================================================================
// Phase 2: Event Name Resolution + EventIgnore
// =============================================================================

/// Helper: check if a byte is ASCII whitespace.
#[inline]
fn is_ascii_white(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Scan to find the end of an event name token.
/// Returns the length of the token (stops at NUL, whitespace, comma, or pipe).
unsafe fn event_name_len(start: *const c_char) -> usize {
    let mut len = 0usize;
    loop {
        let c = *start.add(len) as u8;
        if c == 0 || is_ascii_white(c) || c == b',' || c == b'|' {
            break;
        }
        len += 1;
    }
    len
}

/// Resolve an event name string to its event number.
///
/// Returns an `EventNameResult` with the event number (or `NUM_EVENTS` if not found)
/// and a pointer past the parsed name (after comma if present).
///
/// # Safety
/// `start` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_event_name2nr(start: *const c_char) -> EventNameResult {
    let len = event_name_len(start);
    let event = nvim_event_name2nr(start, len);

    // Advance past the name
    let mut end = start.add(len);
    // Skip comma if present
    if *end == b',' as c_char {
        end = end.add(1);
    }

    EventNameResult {
        event,
        end_ptr: end,
    }
}

/// Resolve an event name from a data+size pair (API String).
///
/// Returns the event number, or `NUM_EVENTS` if not found.
///
/// # Safety
/// `data` must be valid for `size` bytes.
#[unsafe(export_name = "event_name2nr_str")]
pub unsafe extern "C" fn rs_event_name2nr_str(data: *const c_char, size: usize) -> c_int {
    nvim_event_name2nr(data, size)
}

/// Check if an event is included in an eventignore string.
///
/// Handles `-` prefix (unignore), `all` keyword, and comma-separated event names.
/// The `all` keyword only covers window-level events if `ei` is `p_ei` (global eventignore).
///
/// # Safety
/// `ei` must be a valid NUL-terminated C string.
#[export_name = "event_ignored"]
pub unsafe extern "C" fn rs_event_ignored(event: c_int, ei: *const c_char) -> bool {
    let p_ei = nvim_get_p_ei();
    let mut p = ei;
    let mut ignored = false;

    while *p != 0 {
        let unignore = *p == b'-' as c_char;
        if unignore {
            p = p.add(1);
        }

        // Check for "all" keyword
        let remaining = p;
        if strnicmp_prefix(remaining, b"all") && {
            let after = *remaining.add(3) as u8;
            after == 0 || after == b','
        } {
            // "all" in global p_ei ignores all events;
            // "all" in window-local ei only ignores window-level events (sign <= 0)
            ignored = p == p_ei || nvim_get_event_sign(event) <= 0;
            p = remaining.add(3);
            if *p == b',' as c_char {
                p = p.add(1);
            }
        } else {
            // Parse event name
            let result = rs_event_name2nr(p);
            p = result.end_ptr;
            if result.event == event {
                if unignore {
                    return false;
                }
                ignored = true;
            }
        }
    }

    ignored
}

/// Validate the contents of 'eventignore' or 'eventignorewin'.
///
/// Returns 0 (OK) if valid, -1 (FAIL) if invalid.
///
/// # Safety
/// `ei` must be a valid NUL-terminated C string.
#[unsafe(export_name = "check_ei")]
pub unsafe extern "C" fn rs_check_ei(ei: *const c_char) -> c_int {
    let p_ei = nvim_get_p_ei();
    let win = ei != p_ei;
    let mut p = ei;

    while *p != 0 {
        // Check for "all" keyword
        if strnicmp_prefix(p, b"all") && {
            let after = *p.add(3) as u8;
            after == 0 || after == b','
        } {
            p = p.add(3);
            if *p == b',' as c_char {
                p = p.add(1);
            }
        } else {
            // Skip optional '-' prefix
            if *p == b'-' as c_char {
                p = p.add(1);
            }
            let result = rs_event_name2nr(p);
            p = result.end_ptr;
            if result.event == NUM_EVENTS {
                return -1; // FAIL
            }
            // Window-local ei can only have window-level events (sign <= 0)
            if win && nvim_get_event_sign(result.event) > 0 {
                return -1; // FAIL
            }
        }
    }

    0 // OK
}

/// Add "what" to 'eventignore' to skip loading syntax highlighting.
///
/// "what" must start with a comma. Returns the old value of 'eventignore'
/// in allocated memory (caller must pass it to `rs_au_event_restore`).
///
/// # Safety
/// `what` must be a valid NUL-terminated C string starting with ','.
#[unsafe(export_name = "au_event_disable")]
pub unsafe extern "C" fn rs_au_event_disable(what: *const c_char) -> *mut c_char {
    let p_ei = nvim_get_p_ei();
    let p_ei_len = c_strlen(p_ei);
    let what_len = c_strlen(what);

    // Save old value
    let save_ei = nvim_autocmd_xmemdupz(p_ei, p_ei_len);

    // Create new value: p_ei + what
    let new_ei = nvim_autocmd_xstrnsave(p_ei, p_ei_len + what_len);

    if *what == b',' as c_char && *p_ei == 0 {
        // p_ei is empty, skip the leading comma in what
        std::ptr::copy_nonoverlapping(what.add(1), new_ei, what_len); // includes NUL from what
    } else {
        std::ptr::copy_nonoverlapping(what, new_ei.add(p_ei_len), what_len + 1);
    }

    nvim_autocmd_set_option_eventignore(new_ei);
    nvim_autocmd_xfree(new_ei);

    save_ei
}

/// Restore 'eventignore' from a previously saved value.
///
/// # Safety
/// `old_ei` must be a value returned by `rs_au_event_disable`, or null.
#[unsafe(export_name = "au_event_restore")]
pub unsafe extern "C" fn rs_au_event_restore(old_ei: *mut c_char) {
    if !old_ei.is_null() {
        nvim_autocmd_set_option_eventignore(old_ei);
        nvim_autocmd_xfree(old_ei);
    }
}

/// Get the length of a NUL-terminated C string.
unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

/// Case-insensitive prefix check against a known ASCII lowercase byte pattern.
unsafe fn strnicmp_prefix(s: *const c_char, prefix: &[u8]) -> bool {
    for (i, &expected) in prefix.iter().enumerate() {
        let c = *s.add(i) as u8;
        if c.to_ascii_lowercase() != expected {
            return false;
        }
    }
    true
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_category() {
        assert_eq!(EventCategory::from_raw(0), EventCategory::Unknown);
        assert_eq!(EventCategory::from_raw(1), EventCategory::Buffer);
        assert_eq!(EventCategory::from_raw(13), EventCategory::User);
        assert_eq!(EventCategory::from_raw(99), EventCategory::Unknown);

        assert_eq!(EventCategory::Buffer.to_raw(), 1);
        assert_eq!(EventCategory::User.to_raw(), 13);
    }

    #[test]
    fn test_event_flags() {
        let flags = 0;
        assert!(!has_event_flag(flags, event_flags::AU_EXECUTING));

        let flags = set_event_flag(flags, event_flags::AU_EXECUTING);
        assert!(has_event_flag(flags, event_flags::AU_EXECUTING));

        let flags = set_event_flag(flags, event_flags::AU_NESTED);
        assert!(has_event_flag(flags, event_flags::AU_EXECUTING));
        assert!(has_event_flag(flags, event_flags::AU_NESTED));

        let flags = clear_event_flag(flags, event_flags::AU_EXECUTING);
        assert!(!has_event_flag(flags, event_flags::AU_EXECUTING));
        assert!(has_event_flag(flags, event_flags::AU_NESTED));
    }

    #[test]
    fn test_event_state() {
        let mut state = EventState::new(10);
        assert_eq!(state.event, 10);
        assert!(!state.is_executing());
        assert!(!state.is_nested());
        assert_eq!(state.nesting_level, 0);

        state.set_executing(true);
        assert!(state.is_executing());

        state.set_nested(true);
        assert!(state.is_nested());

        state.push_nesting();
        assert_eq!(state.nesting_level, 1);
        state.push_nesting();
        assert_eq!(state.nesting_level, 2);

        state.pop_nesting();
        assert_eq!(state.nesting_level, 1);
    }

    #[test]
    fn test_nesting() {
        assert!(can_nest(0));
        assert!(can_nest(9));
        assert!(!can_nest(10));
        assert!(!can_nest(100));
    }

    #[test]
    fn test_check_nesting() {
        let mut state = EventState::new(0);

        // Not nested by default
        assert_eq!(check_nesting(&state), NestingResult::NotNested);

        // Enable nesting
        state.set_nested(true);
        assert_eq!(check_nesting(&state), NestingResult::Allowed);

        // Too deep
        state.nesting_level = MAX_NESTING_DEPTH;
        assert_eq!(check_nesting(&state), NestingResult::TooDeep);

        // Blocked
        state.flags = set_event_flag(state.flags, event_flags::AU_BLOCKED);
        assert_eq!(check_nesting(&state), NestingResult::Blocked);
    }

    #[test]
    fn test_event_match() {
        let no_match = EventMatch::no_match();
        assert!(!no_match.matched);
        assert_eq!(no_match.event, -1);

        let matched = EventMatch::matched(42, 100);
        assert!(matched.matched);
        assert_eq!(matched.event, 42);
        assert_eq!(matched.score, 100);
    }

    #[test]
    fn test_event_from_raw() {
        // Valid events
        assert_eq!(Event::from_raw(0), Some(Event::BufAdd));
        assert_eq!(Event::from_raw(3), Some(Event::BufEnter));
        assert_eq!(Event::from_raw(37), Some(Event::CursorHold));
        assert_eq!(Event::from_raw(128), Some(Event::User));
        assert_eq!(Event::from_raw(140), Some(Event::WinScrolled));

        // Invalid events
        assert_eq!(Event::from_raw(-1), None);
        assert_eq!(Event::from_raw(141), None);
        assert_eq!(Event::from_raw(1000), None);
    }

    #[test]
    fn test_event_to_raw() {
        assert_eq!(Event::BufAdd.to_raw(), 0);
        assert_eq!(Event::BufEnter.to_raw(), 3);
        assert_eq!(Event::CursorHold.to_raw(), 37);
        assert_eq!(Event::User.to_raw(), 128);
        assert_eq!(Event::WinScrolled.to_raw(), 140);
    }

    #[test]
    fn test_event_enum_category() {
        assert_eq!(Event::BufEnter.category(), EventCategory::Buffer);
        assert_eq!(Event::FileType.category(), EventCategory::File);
        assert_eq!(Event::WinEnter.category(), EventCategory::Window);
        assert_eq!(Event::TabEnter.category(), EventCategory::Tab);
        assert_eq!(Event::CursorHold.category(), EventCategory::Cursor);
        assert_eq!(Event::InsertEnter.category(), EventCategory::Insert);
        assert_eq!(Event::CmdlineEnter.category(), EventCategory::Cmdline);
        assert_eq!(Event::TermOpen.category(), EventCategory::Terminal);
        assert_eq!(Event::ColorScheme.category(), EventCategory::Ui);
        assert_eq!(Event::VimEnter.category(), EventCategory::Session);
        assert_eq!(Event::TextChanged.category(), EventCategory::TextChange);
        assert_eq!(Event::CompleteDone.category(), EventCategory::Completion);
        assert_eq!(Event::User.category(), EventCategory::User);
    }

    #[test]
    fn test_event_name() {
        assert_eq!(Event::BufAdd.name(), "BufAdd");
        assert_eq!(Event::BufEnter.name(), "BufEnter");
        assert_eq!(Event::CursorHold.name(), "CursorHold");
        assert_eq!(Event::TextYankPost.name(), "TextYankPost");
        assert_eq!(Event::WinScrolled.name(), "WinScrolled");
    }

    #[test]
    fn test_event_is_valid() {
        assert!(Event::BufAdd.is_valid());
        assert!(Event::WinScrolled.is_valid());
    }

    #[test]
    fn test_num_events() {
        assert_eq!(NUM_EVENTS, 141);
    }

    #[test]
    fn test_event_roundtrip() {
        // Test that all events can round-trip through raw conversion
        for i in 0..NUM_EVENTS {
            let event = Event::from_raw(i).expect("valid event");
            assert_eq!(event.to_raw(), i);
        }
    }
}
