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
use crate::memory::XString;
use crate::winlayer::Win;

/// A pattern's slot in its event's [`EventCmds::pats`] slab.
///
/// An id, not an address: a handler running under a walk can define and
/// delete autocommands, and a walk holds its last pattern across that. A
/// slot is cleared when its last command goes and is only reused after the
/// slab compacts, which waits for the last walk to end -- so an id names one
/// pattern for as long as any walk can hold it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct PatId(u32);

impl PatId {
    /// The slot this id names.
    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

/// What an autocommand runs.
pub(crate) enum AutoHandler {
    /// An Ex command line.
    Cmd(XString),
    /// A callback -- a Lua function, a funcref or a partial. A deleted
    /// command holds `Callback::None`.
    Fn(Callback),
}

pub struct AutoCmd {
    /// The command's pattern, `None` once [`aucmd_del`] has deleted it --
    /// the mark compaction removes it on.
    ///
    /// [`aucmd_del`]: crate::autocmd::aucmd_del
    pub(crate) pat: Option<PatId>,
    pub id: int64_t,
    pub desc: Option<XString>,
    pub(crate) handler: AutoHandler,
    pub script_ctx: ScriptCtx,
    pub once: bool,
    pub nested: bool,
}

pub struct AutoPat {
    /// How many commands name this pattern; the slot is cleared at zero.
    pub refcount: usize,
    pub pat: XString,
    /// The pattern compiled, for a pattern that is not buffer-local.
    pub reg_prog: Option<crate::regexp::OwnedProg>,
    pub group: ::core::ffi::c_int,
    pub buflocal_nr: ::core::ffi::c_int,
    pub allow_dirs: bool,
}

/// One event's autocommands: the commands in definition order, which is the
/// order they fire and list in, over a slab of the patterns they share.
///
/// A deleted command stays in place, its `pat` cleared, until no walk is
/// running; then [`compact`](Self::compact) removes the deleted rows and
/// the cleared slots together.
pub(crate) struct EventCmds {
    pub(crate) pats: Vec<Option<AutoPat>>,
    pub(crate) cmds: Vec<AutoCmd>,
}

impl EventCmds {
    /// No autocommands, and nothing allocated.
    pub(crate) const EMPTY: EventCmds = EventCmds {
        pats: Vec::new(),
        cmds: Vec::new(),
    };

    /// How many command rows there are, the deleted ones included -- the
    /// bound a walk snapshots.
    pub(crate) fn len(&self) -> usize {
        self.cmds.len()
    }

    /// Whether there are no rows at all, deleted or not.
    pub(crate) fn is_empty(&self) -> bool {
        self.cmds.is_empty()
    }

    /// The pattern `id` names, which must be live: a command that has a
    /// pattern holds a count of it.
    pub(crate) fn pat(&self, id: PatId) -> &AutoPat {
        self.pats[id.index()]
            .as_ref()
            .expect("a command's pattern is live while the command is")
    }

    /// [`pat`](Self::pat), to write.
    pub(crate) fn pat_mut(&mut self, id: PatId) -> &mut AutoPat {
        self.pats[id.index()]
            .as_mut()
            .expect("a command's pattern is live while the command is")
    }

    /// The commands that are not deleted, each with its pattern, in order.
    pub(crate) fn live(&self) -> impl Iterator<Item = (&AutoCmd, &AutoPat)> {
        self.cmds
            .iter()
            .filter_map(|cmd| cmd.pat.map(|id| (cmd, self.pat(id))))
    }

    /// Put `pat` in the slab, holding no count yet.
    pub(crate) fn add_pat(&mut self, pat: AutoPat) -> PatId {
        let id = PatId(u32::try_from(self.pats.len()).expect("fewer than 2^32 patterns"));
        self.pats.push(Some(pat));
        id
    }

    /// Delete command `idx`: give up its count of its pattern, clearing the
    /// slot at zero, and release what the command owns. The row stays
    /// where a running walk's index can still reach it. Answers whether the
    /// row was live.
    pub(crate) fn delete(&mut self, idx: usize) -> bool {
        let cmd = &mut self.cmds[idx];
        let was_live = cmd.pat.is_some();
        if let Some(id) = cmd.pat.take() {
            let slot = &mut self.pats[id.index()];
            let pat = slot
                .as_mut()
                .expect("a command's pattern is live while the command is");
            pat.refcount -= 1;
            if pat.refcount == 0 {
                *slot = None;
            }
        }
        let cmd = &mut self.cmds[idx];
        if let AutoHandler::Fn(callback) = &mut cmd.handler {
            callback.clear();
        }
        cmd.handler = AutoHandler::Fn(Callback::None);
        cmd.desc = None;
        was_live
    }

    /// Remove the deleted rows and the cleared pattern slots, renumbering
    /// the patterns that stay. Only for when no walk is running: a walk
    /// holds row indices and a pattern id.
    pub(crate) fn compact(&mut self) {
        self.cmds.retain(|cmd| cmd.pat.is_some());
        if self.cmds.is_empty() {
            // An event with nothing left gives its blocks back.
            *self = EventCmds::EMPTY;
            return;
        }
        if self.pats.iter().all(Option::is_some) {
            return;
        }
        let mut renumbered = vec![PatId(u32::MAX); self.pats.len()];
        let mut kept = Vec::with_capacity(self.pats.len());
        for (slot, pat) in core::mem::take(&mut self.pats).into_iter().enumerate() {
            if let Some(pat) = pat {
                renumbered[slot] =
                    PatId(u32::try_from(kept.len()).expect("fewer than 2^32 patterns"));
                kept.push(Some(pat));
            }
        }
        self.pats = kept;
        for cmd in &mut self.cmds {
            if let Some(id) = &mut cmd.pat {
                *id = renumbered[id.index()];
            }
        }
    }
}

/// The cursor of one firing walk: where it is in its event's commands, and
/// the names its patterns are matched against.
pub struct AutoPatCmd {
    /// The pattern of the command the walk last stopped at; `None` when it
    /// has not started, or has run out.
    pub(crate) lastpat: Option<PatId>,
    pub auidx: usize,
    /// The row count when the walk started: commands a handler adds for
    /// the same event do not extend the walk.
    pub ausize: usize,
    /// `<afile>` as the caller gave it, for a callback's `file`.
    pub afile_orig: Option<XString>,
    /// The name the patterns match: a full path unless the event's names
    /// are not file names.
    pub fname: XString,
    pub sfname: Option<XString>,
    /// Where `fname`'s tail starts.
    pub tail: usize,
    pub group: ::core::ffi::c_int,
    pub event: AutoEvent,
    pub script_ctx: ScriptCtx,
    /// The buffer number `<buffer=N>` patterns match against, until the
    /// walk is running; then its slot in the running walks' numbers, which
    /// a buffer being freed clears.
    pub arg_bufnr: ::core::ffi::c_int,
    pub(crate) walk: Option<usize>,
}
pub struct AcoSave {
    pub use_aucmd_win_idx: ::core::ffi::c_int,
    pub save_curwin_handle: Handle,
    pub new_curwin_handle: Handle,
    pub save_prevwin_handle: Handle,
    pub new_curbuf: BufferRef,
    /// The tab page's and the global directory, held aside while the
    /// autocommand window is current so `win_enter` cannot `chdir`.
    pub tp_localdir: Option<XString>,
    pub globaldir: Option<XString>,
    pub save_visual_active: bool,
    pub save_prompt_insert: ::core::ffi::c_int,
}

impl Default for AcoSave {
    /// The zeroed state `aucmd_prepbuf` expects to be handed. Every caller
    /// declares one of these as a local and immediately fills it in.
    fn default() -> Self {
        AcoSave {
            use_aucmd_win_idx: 0,
            save_curwin_handle: 0,
            new_curwin_handle: 0,
            save_prevwin_handle: 0,
            new_curbuf: BufferRef::new(),
            tp_localdir: None,
            globaldir: None,
            save_visual_active: false,
            save_prompt_insert: 0,
        }
    }
}

pub struct AucmdWin {
    /// The slot's window, `None` until `win_alloc_aucmd_win` makes it. The
    /// slot owns it -- it is unregistered while idle, so no id finds it --
    /// and nothing closes it but the exit path, so the handle stays good.
    pub auc_win: Option<Win>,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A pattern row holding no compiled program.
    fn pattern(text: &str) -> AutoPat {
        AutoPat {
            refcount: 0,
            pat: XString::from(text),
            reg_prog: None,
            group: -1,
            buflocal_nr: 0,
            allow_dirs: false,
        }
    }

    /// Append a command on `pat` to `cmds`, as `autocmd_register` does.
    fn push(cmds: &mut EventCmds, pat: PatId, line: &str) {
        cmds.pat_mut(pat).refcount += 1;
        cmds.cmds.push(AutoCmd {
            pat: Some(pat),
            id: 0,
            desc: None,
            handler: AutoHandler::Cmd(XString::from(line)),
            script_ctx: ScriptCtx::NONE,
            once: false,
            nested: false,
        });
    }

    /// The live commands' lines with their patterns, in order.
    fn listing(cmds: &EventCmds) -> Vec<(String, String)> {
        cmds.live()
            .map(|(cmd, pat)| {
                let AutoHandler::Cmd(line) = &cmd.handler else {
                    unreachable!("the cases only push commands")
                };
                (
                    String::from_utf8_lossy(&pat.pat).into_owned(),
                    String::from_utf8_lossy(line).into_owned(),
                )
            })
            .collect()
    }

    /// A deletion only marks: the rows keep their indices, which a walk is
    /// standing on, and the pattern goes with its last command.
    #[test]
    fn delete_marks_in_place_and_frees_the_last_users_pattern() {
        let mut cmds = EventCmds::EMPTY;
        let a = cmds.add_pat(pattern("*.c"));
        push(&mut cmds, a, "echo 1");
        push(&mut cmds, a, "echo 2");
        let b = cmds.add_pat(pattern("*.h"));
        push(&mut cmds, b, "echo 3");

        assert!(cmds.delete(0));
        assert_eq!(cmds.len(), 3, "the row stays");
        assert!(cmds.pats[a.index()].is_some(), "one command still names it");
        assert!(cmds.delete(1));
        assert!(cmds.pats[a.index()].is_none(), "its last command went");
        assert!(!cmds.delete(1), "a deleted row is not live");
        assert_eq!(listing(&cmds), [("*.h".into(), "echo 3".into())]);
    }

    /// Compaction drops the deleted rows and the cleared slots, renumbers
    /// the patterns that stay, and keeps the order the list fires in.
    #[test]
    fn compact_keeps_order_and_renumbers_patterns() {
        let mut cmds = EventCmds::EMPTY;
        let a = cmds.add_pat(pattern("a"));
        push(&mut cmds, a, "1");
        let b = cmds.add_pat(pattern("b"));
        push(&mut cmds, b, "2");
        let c = cmds.add_pat(pattern("c"));
        push(&mut cmds, c, "3");
        push(&mut cmds, b, "4");
        cmds.delete(0);

        cmds.compact();
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds.pats.len(), 2);
        assert_eq!(
            listing(&cmds),
            [
                ("b".into(), "2".into()),
                ("c".into(), "3".into()),
                ("b".into(), "4".into()),
            ]
        );
        // The two commands on `b` still share one slot, with both counts.
        assert_eq!(cmds.cmds[0].pat, cmds.cmds[2].pat);
        assert_eq!(cmds.pat(cmds.cmds[0].pat.unwrap()).refcount, 2);

        // Deleting everything gives the blocks back.
        for idx in 0..cmds.len() {
            cmds.delete(idx);
        }
        cmds.compact();
        assert!(cmds.is_empty());
        assert_eq!(cmds.pats.capacity(), 0);
        assert_eq!(cmds.cmds.capacity(), 0);
    }
}
