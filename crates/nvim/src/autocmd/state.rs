//! What an autocommand is being fired for, and what it may not do.
//!
//! `<afile>`, `<abuf>` and `<amatch>` are not arguments: the trigger writes
//! them here (`autocmd_fname`, `autocmd_bufnr`, `autocmd_match`) and the
//! expansion reads them back, so a nested trigger has to save and restore
//! them. Around those sit the "an autocommand is running" flag every
//! re-entrancy check tests (`autocmd_busy`), the two counters that suppress
//! `BufEnter`/`BufLeave` for a switch the user did not ask for, the
//! `CursorHold` and `CursorMoved` bookkeeping, the group tables, and the
//! queue of events deferred out of a context that could not fire them.
//!
//! One record behind one cell, reached a field at a time through selectors
//! that keep upstream's names. Nothing holds a borrow of it across a call:
//! every autocommand runs user code, and user code fires autocommands.
//! Two cells stay apart: the per-event autocommand lists and the `aucmd_win`
//! vector, whose *addresses* their families work from.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The selectors keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::{AUGROUP_DEFAULT, SuspendLatch};
use crate::event::multiqueue::ChildQueue;
use crate::global_cell::{GlobalCell, state_record};
use crate::memory::XString;
use crate::registry::{IdMap, SlotTable, id_map};
use crate::types::{BufferRef, ColNr, LineNr, Pos, Timestamp};
use crate::winlayer::WinId;
use core::ffi::c_int;

state_record! {
    /// The autocommand machinery's state: upstream's `autocmd.c` globals
    /// and the function-scope statics of the triggers.
    pub(crate) struct AutocmdState in AUTOCMD as AutocmdField;

    /// The window `CursorMoved` last fired in, with [`last_cursormoved`]
    /// its cursor then.
    pub(crate) last_cursormoved_win: Option<WinId> = None;
    pub(crate) last_cursormoved: Pos = Pos {
        lnum: 0 as LineNr,
        col: 0 as ColNr,
        coladd: 0 as ColNr,
    };
    pub(crate) autocmd_busy: bool = false;
    pub(crate) autocmd_no_enter: c_int = 0;
    pub(crate) autocmd_no_leave: c_int = 0;
    pub(crate) au_new_curbuf: BufferRef = BufferRef::new();
    /// `<afile>`, room for a full path: the expansion resolves it in place.
    pub(crate) autocmd_fname: Option<XString> = None;
    pub(crate) autocmd_fname_full: bool = false;
    pub(crate) autocmd_bufnr: c_int = 0;
    /// `<amatch>`.
    pub(crate) autocmd_match: Option<XString> = None;
    pub(crate) did_cursorhold: bool = true;
    /// The buffer number each running autocommand walk matches
    /// `<buffer=N>` patterns against, innermost last; the walk keeps its
    /// index. Upstream threads the walks' own `AutoPatCmd`s into
    /// `active_apc_list` so that freeing a buffer can clear the number.
    pub(super) active_walk_bufnrs: Vec<c_int> = Vec::new();
    pub(super) next_augroup_id: c_int = 1;
    pub(super) current_augroup: c_int = AUGROUP_DEFAULT;
    pub(super) au_need_clean: bool = false;
    pub(super) autocmd_blocked: c_int = 0;
    pub(super) autocmd_nested: bool = false;
    pub(super) autocmd_include_groups: bool = false;
    pub(super) termresponse_changed: bool = false;
    /// Group name -> id, in creation order.
    ///
    /// khash, which this was, is insertion-ordered with a swap-remove, and
    /// `:augroup`'s listing walks it directly (F-P21-9). A [`SlotTable`] is
    /// that order; a key is the name plus a NUL, because the listing prints
    /// one as a C string (`groups::group_key`).
    pub(super) map_augroup_name_to_id: SlotTable<Box<[u8]>, c_int> = SlotTable::new();
    /// Id -> group name. Point lookups only, so a plain map will do; the
    /// name is NUL-terminated because `augroup_name` answers a `*mut c_char`
    /// into it.
    pub(super) map_augroup_id_to_name: IdMap<c_int, Box<[u8]>> = id_map();
    pub(super) pending_vimresume: SuspendLatch = SuspendLatch::Idle;
    /// `UIEnter`/`UILeave` is being fired.
    pub(super) uienter_busy: bool = false;
    /// `FocusGained`/`FocusLost` is being fired.
    pub(super) focusgained_busy: bool = false;
    /// When a `FocusGained` last checked file timestamps.
    pub(super) focusgained_last_time: Timestamp = 0;
    /// How deep `FileType` firings nest.
    pub(super) ft_recursive: c_int = 0;
    /// How deep autocommand firings nest.
    pub(super) autocmd_nesting: c_int = 0;
    /// `FileChangedShell` is being fired.
    pub(super) filechangeshell_busy: bool = false;
}

/// Events deferred out of a context that could not fire them.
pub(crate) static deferred_events: GlobalCell<Option<ChildQueue>> = GlobalCell::new(None);
