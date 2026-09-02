#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Clone)]
pub struct AutoCmd {
    pub pat: *mut AutoPat,
    pub id: int64_t,
    pub desc: *mut ::core::ffi::c_char,
    pub handler_cmd: *mut ::core::ffi::c_char,
    pub handler_fn: Callback,
    pub script_ctx: sctx_T,
    pub once: bool,
    pub nested: bool,
}
pub struct AutoPat {
    pub refcount: RefcountSize,
    pub pat: *mut ::core::ffi::c_char,
    pub reg_prog: *mut regprog_T,
    pub group: ::core::ffi::c_int,
    pub patlen: ::core::ffi::c_int,
    pub buflocal_nr: ::core::ffi::c_int,
    pub allow_dirs: ::core::ffi::c_char,
}
pub type AutoPatCmd = AutoPatCmd_S;
pub struct AutoPatCmd_S {
    pub lastpat: *mut AutoPat,
    pub auidx: size_t,
    pub ausize: size_t,
    pub afile_orig: *mut ::core::ffi::c_char,
    pub fname: *mut ::core::ffi::c_char,
    pub sfname: *mut ::core::ffi::c_char,
    pub tail: *mut ::core::ffi::c_char,
    pub group: ::core::ffi::c_int,
    pub event: AutoEvent,
    pub script_ctx: sctx_T,
    pub arg_bufnr: ::core::ffi::c_int,
    pub data: *mut Object,
    pub next: *mut AutoPatCmd,
}
pub struct aco_save_T {
    pub use_aucmd_win_idx: ::core::ffi::c_int,
    pub save_curwin_handle: handle_T,
    pub new_curwin_handle: handle_T,
    pub save_prevwin_handle: handle_T,
    pub new_curbuf: bufref_T,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub globaldir: *mut ::core::ffi::c_char,
    pub save_VIsual_active: bool,
    pub save_prompt_insert: ::core::ffi::c_int,
}

impl Default for aco_save_T {
    /// The zeroed state `aucmd_prepbuf` expects to be handed. Every caller
    /// declares one of these as a local and immediately fills it in.
    fn default() -> Self {
        aco_save_T {
            use_aucmd_win_idx: 0,
            save_curwin_handle: 0,
            new_curwin_handle: 0,
            save_prevwin_handle: 0,
            new_curbuf: bufref_T {
                br_buf: ::core::ptr::null_mut(),
                br_fnum: 0,
                br_buf_free_count: 0,
            },
            tp_localdir: ::core::ptr::null_mut(),
            globaldir: ::core::ptr::null_mut(),
            save_VIsual_active: false,
            save_prompt_insert: 0,
        }
    }
}

pub struct aucmdwin_T {
    pub auc_win: *mut win_T,
    pub auc_win_used: bool,
}

/// An autocommand event.
///
/// c2rust rendered upstream's `auto_event` enumeration as `typedef unsigned
/// int` plus 145 `pub const EVENT_*`, so an event compared equal to any
/// integer in the program and `autocmds[event]` was a bare index.
///
/// **The discriminants are upstream's and are load-bearing**: an event's
/// number is its row in `EVENT_NAMES` -- the name table `event_name2nr`
/// binary searches and `event_nr2name` indexes -- and its slot in the
/// `autocmds` table of per-event command lists. Both come from the ordering
/// upstream's `gen_events.lua` produces (lower-cased name order), so
/// renumbering one without the other silently fires the wrong autocommands.
///
/// A variant is the event's own name, which is the spelling `:autocmd` and
/// `nvim_create_autocmd` take. `Event` was taken by the event loop's queued
/// event, hence `AutoEvent` for upstream's `auto_event`.
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum AutoEvent {
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
    ChanInfo = 23,
    ChanOpen = 24,
    CmdlineChanged = 25,
    CmdlineEnter = 26,
    CmdlineLeave = 27,
    CmdlineLeavePre = 28,
    CmdUndefined = 29,
    CmdwinEnter = 30,
    CmdwinLeave = 31,
    ColorScheme = 32,
    ColorSchemePre = 33,
    CompleteChanged = 34,
    CompleteDone = 35,
    CompleteDonePre = 36,
    CursorHold = 37,
    CursorHoldI = 38,
    CursorMoved = 39,
    CursorMovedC = 40,
    CursorMovedI = 41,
    DiagnosticChanged = 42,
    DiffUpdated = 43,
    DirChanged = 44,
    DirChangedPre = 45,
    EncodingChanged = 46,
    ExitPre = 47,
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
    FocusGained = 66,
    FocusLost = 67,
    FuncUndefined = 68,
    GUIEnter = 69,
    GUIFailed = 70,
    InsertChange = 71,
    InsertCharPre = 72,
    InsertEnter = 73,
    InsertLeave = 74,
    InsertLeavePre = 75,
    LspAttach = 76,
    LspDetach = 77,
    LspNotify = 78,
    LspProgress = 79,
    LspRequest = 80,
    LspTokenUpdate = 81,
    MarkSet = 82,
    MenuPopup = 83,
    ModeChanged = 84,
    OptionSet = 85,
    PackChanged = 86,
    PackChangedPre = 87,
    Progress = 88,
    QuickFixCmdPost = 89,
    QuickFixCmdPre = 90,
    QuitPre = 91,
    RecordingEnter = 92,
    RecordingLeave = 93,
    RemoteReply = 94,
    SafeState = 95,
    SearchWrapped = 96,
    SessionLoadPost = 97,
    SessionLoadPre = 98,
    SessionWritePost = 99,
    ShellCmdPost = 100,
    ShellFilterPost = 101,
    Signal = 102,
    SourceCmd = 103,
    SourcePost = 104,
    SourcePre = 105,
    SpellFileMissing = 106,
    StdinReadPost = 107,
    StdinReadPre = 108,
    SwapExists = 109,
    Syntax = 110,
    TabClosed = 111,
    TabClosedPre = 112,
    TabEnter = 113,
    TabLeave = 114,
    TabNew = 115,
    TabNewEntered = 116,
    TermChanged = 117,
    TermClose = 118,
    TermEnter = 119,
    TermLeave = 120,
    TermOpen = 121,
    TermRequest = 122,
    TermResponse = 123,
    TextChanged = 124,
    TextChangedI = 125,
    TextChangedP = 126,
    TextChangedT = 127,
    TextYankPost = 128,
    UIEnter = 129,
    UILeave = 130,
    User = 131,
    VimEnter = 132,
    VimLeave = 133,
    VimLeavePre = 134,
    VimResized = 135,
    VimResume = 136,
    VimSuspend = 137,
    WinClosed = 138,
    WinEnter = 139,
    WinLeave = 140,
    WinNew = 141,
    WinNewPre = 142,
    WinResized = 143,
    WinScrolled = 144,
}

/// Hand-written rather than derived, and `#[inline(always)]`: the derived
/// `eq` is an ordinary call at `-O0`, which is what both test suites build,
/// and the 'eventignore' walk compares events per name per fired event.
impl PartialEq for AutoEvent {
    #[inline(always)]
    fn eq(&self, other: &AutoEvent) -> bool {
        self.code() == other.code()
    }
}

impl Eq for AutoEvent {}

impl AutoEvent {
    /// How many events there are -- upstream's `NUM_EVENTS`, which it also
    /// spent as the "no such event" marker. That marker is an `Option` here.
    pub const COUNT: usize = 145;

    /// The `unsigned int` upstream stores.
    #[inline(always)]
    pub const fn code(self) -> ::core::ffi::c_uint {
        self as ::core::ffi::c_uint
    }

    /// This event's row in `EVENT_NAMES`, and its slot in `autocmds`.
    #[inline(always)]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The event numbered `index`, or `None` past the last one.
    #[inline(always)]
    pub fn at_row(index: usize) -> Option<AutoEvent> {
        AUTO_EVENTS.get(index).copied()
    }

    /// Every event, in number order.
    pub fn all() -> impl Iterator<Item = AutoEvent> {
        AUTO_EVENTS.iter().copied()
    }
}

/// Every event in number order, which [`AutoEvent::index`] indexes.
#[rustfmt::skip]
static AUTO_EVENTS: [AutoEvent; 145] = [
    AutoEvent::BufAdd, AutoEvent::BufCreate, AutoEvent::BufDelete, AutoEvent::BufEnter,
    AutoEvent::BufFilePost, AutoEvent::BufFilePre, AutoEvent::BufHidden, AutoEvent::BufLeave,
    AutoEvent::BufModifiedSet, AutoEvent::BufNew, AutoEvent::BufNewFile, AutoEvent::BufRead,
    AutoEvent::BufReadCmd, AutoEvent::BufReadPost, AutoEvent::BufReadPre, AutoEvent::BufUnload,
    AutoEvent::BufWinEnter, AutoEvent::BufWinLeave, AutoEvent::BufWipeout, AutoEvent::BufWrite,
    AutoEvent::BufWriteCmd, AutoEvent::BufWritePost, AutoEvent::BufWritePre,
    AutoEvent::ChanInfo, AutoEvent::ChanOpen, AutoEvent::CmdlineChanged,
    AutoEvent::CmdlineEnter, AutoEvent::CmdlineLeave, AutoEvent::CmdlineLeavePre,
    AutoEvent::CmdUndefined, AutoEvent::CmdwinEnter, AutoEvent::CmdwinLeave,
    AutoEvent::ColorScheme, AutoEvent::ColorSchemePre, AutoEvent::CompleteChanged,
    AutoEvent::CompleteDone, AutoEvent::CompleteDonePre, AutoEvent::CursorHold,
    AutoEvent::CursorHoldI, AutoEvent::CursorMoved, AutoEvent::CursorMovedC,
    AutoEvent::CursorMovedI, AutoEvent::DiagnosticChanged, AutoEvent::DiffUpdated,
    AutoEvent::DirChanged, AutoEvent::DirChangedPre, AutoEvent::EncodingChanged,
    AutoEvent::ExitPre, AutoEvent::FileAppendCmd, AutoEvent::FileAppendPost,
    AutoEvent::FileAppendPre, AutoEvent::FileChangedRO, AutoEvent::FileChangedShell,
    AutoEvent::FileChangedShellPost, AutoEvent::FileEncoding, AutoEvent::FileReadCmd,
    AutoEvent::FileReadPost, AutoEvent::FileReadPre, AutoEvent::FileType,
    AutoEvent::FileWriteCmd, AutoEvent::FileWritePost, AutoEvent::FileWritePre,
    AutoEvent::FilterReadPost, AutoEvent::FilterReadPre, AutoEvent::FilterWritePost,
    AutoEvent::FilterWritePre, AutoEvent::FocusGained, AutoEvent::FocusLost,
    AutoEvent::FuncUndefined, AutoEvent::GUIEnter, AutoEvent::GUIFailed,
    AutoEvent::InsertChange, AutoEvent::InsertCharPre, AutoEvent::InsertEnter,
    AutoEvent::InsertLeave, AutoEvent::InsertLeavePre, AutoEvent::LspAttach,
    AutoEvent::LspDetach, AutoEvent::LspNotify, AutoEvent::LspProgress, AutoEvent::LspRequest,
    AutoEvent::LspTokenUpdate, AutoEvent::MarkSet, AutoEvent::MenuPopup,
    AutoEvent::ModeChanged, AutoEvent::OptionSet, AutoEvent::PackChanged,
    AutoEvent::PackChangedPre, AutoEvent::Progress, AutoEvent::QuickFixCmdPost,
    AutoEvent::QuickFixCmdPre, AutoEvent::QuitPre, AutoEvent::RecordingEnter,
    AutoEvent::RecordingLeave, AutoEvent::RemoteReply, AutoEvent::SafeState,
    AutoEvent::SearchWrapped, AutoEvent::SessionLoadPost, AutoEvent::SessionLoadPre,
    AutoEvent::SessionWritePost, AutoEvent::ShellCmdPost, AutoEvent::ShellFilterPost,
    AutoEvent::Signal, AutoEvent::SourceCmd, AutoEvent::SourcePost, AutoEvent::SourcePre,
    AutoEvent::SpellFileMissing, AutoEvent::StdinReadPost, AutoEvent::StdinReadPre,
    AutoEvent::SwapExists, AutoEvent::Syntax, AutoEvent::TabClosed, AutoEvent::TabClosedPre,
    AutoEvent::TabEnter, AutoEvent::TabLeave, AutoEvent::TabNew, AutoEvent::TabNewEntered,
    AutoEvent::TermChanged, AutoEvent::TermClose, AutoEvent::TermEnter, AutoEvent::TermLeave,
    AutoEvent::TermOpen, AutoEvent::TermRequest, AutoEvent::TermResponse,
    AutoEvent::TextChanged, AutoEvent::TextChangedI, AutoEvent::TextChangedP,
    AutoEvent::TextChangedT, AutoEvent::TextYankPost, AutoEvent::UIEnter, AutoEvent::UILeave,
    AutoEvent::User, AutoEvent::VimEnter, AutoEvent::VimLeave, AutoEvent::VimLeavePre,
    AutoEvent::VimResized, AutoEvent::VimResume, AutoEvent::VimSuspend, AutoEvent::WinClosed,
    AutoEvent::WinEnter, AutoEvent::WinLeave, AutoEvent::WinNew, AutoEvent::WinNewPre,
    AutoEvent::WinResized, AutoEvent::WinScrolled,
];
