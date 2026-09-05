//! What an autocommand is being fired for, and what it may not do.
//!
//! `<afile>`, `<abuf>` and `<amatch>` are not arguments: the trigger writes
//! them here (`autocmd_fname`, `autocmd_bufnr`, `autocmd_match`) and the
//! expansion reads them back, so a nested trigger has to save and restore
//! them. Around those sit the "an autocommand is running" flag every
//! re-entrancy check tests (`autocmd_busy`), the two counters that suppress
//! `BufEnter`/`BufLeave` for a switch the user did not ask for, the
//! `CursorHold` and `CursorMoved` bookkeeping, and the queue of events
//! deferred out of a context that could not fire them.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::global_cell::GlobalCell;
use crate::types::{BufferRef, ColNr, LineNr, MultiQueue, Pos, Window};
use core::ffi::{c_char, c_int};

pub(crate) static last_cursormoved_win: GlobalCell<*mut Window> =
    GlobalCell::new(::core::ptr::null_mut::<Window>());
pub(crate) static last_cursormoved: GlobalCell<Pos> = GlobalCell::new(Pos {
    lnum: 0 as LineNr,
    col: 0 as ColNr,
    coladd: 0 as ColNr,
});
pub(crate) static autocmd_busy: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static autocmd_no_enter: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static autocmd_no_leave: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static au_new_curbuf: GlobalCell<BufferRef> = GlobalCell::new(BufferRef::new());
pub(crate) static autocmd_fname: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static autocmd_fname_full: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static autocmd_bufnr: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static autocmd_match: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static did_cursorhold: GlobalCell<bool> = GlobalCell::new(true);
pub(crate) static deferred_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
