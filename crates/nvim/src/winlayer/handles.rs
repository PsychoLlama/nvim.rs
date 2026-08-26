//! The three handle registries, and the allocations an autocommand deferred.
//!
//! The editor hands every window, buffer and tab page a monotone id — a
//! [`handle_T`] — and keeps a table from that id to the object, so that an API
//! call, an RPC message or a Lua callback can name one without holding a
//! pointer across the call that might free it. Upstream spells the three
//! tables as khash `Map_int_ptr_t`s reached through a raw pointer
//! (`window_handles`, `buffer_handles`, `tabpage_handles`); here they are
//! owned Rust, one [`HandleRegistry`] each.
//!
//! They live in this module and their statics are **private**, so that the
//! two halves of the invariant [`HandleRegistry`] documents — everything in
//! the table is live — are enforced by visibility rather than by review: the
//! only way in is `register_*`, which the allocator calls, and the only way
//! out is `forget_*`, which the free path calls first. That is what makes
//! the three lookups safe functions, and what [`Win::valid`] rests on.
//!
//! A registry does *not* answer "is this window on screen": a hidden window
//! (`win_alloc(_, hidden)`) is registered and on no list, and the autocommand
//! window is unregistered while it is idle. `win_valid` and friends stay list
//! walks — see `window::win_valid`, and [`Win::valid`]'s own docs for which
//! question is which.
//!
//! This is a child of [`crate::winlayer`] so that it can build a [`Win`],
//! [`Buf`] or [`TabPage`] straight from a table entry, whose handle it
//! already knows: a lookup reads nothing out of the object it answers with.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::buffer::free;
use crate::global_cell::GlobalCell;
use crate::registry::{HandleRegistry, PendingFree};
use crate::types::{buf_T, handle_T, tabpage_T, win_T};
use crate::winlayer::{Buf, TabPage, Win};

/// Every live window, by handle.
static WINDOWS: GlobalCell<HandleRegistry<win_T>> = GlobalCell::new(HandleRegistry::new());

/// Every live buffer, by number.
static BUFFERS: GlobalCell<HandleRegistry<buf_T>> = GlobalCell::new(HandleRegistry::new());

/// Every live tab page, by handle.
static TABPAGES: GlobalCell<HandleRegistry<tabpage_T>> = GlobalCell::new(HandleRegistry::new());

/// The window `handle` names, `None` once it has been closed.
pub(crate) fn window(handle: handle_T) -> Option<Win> {
    // The borrow ends with the lookup, which cannot re-enter. The table
    // already knows the handle, so nothing is read out of the window.
    WINDOWS
        .with(|reg| reg.get(handle))
        .map(|wp| Win(wp, handle))
}

/// The buffer numbered `handle`, `None` once it has been wiped.
pub(crate) fn buffer(handle: handle_T) -> Option<Buf> {
    // As [`window`].
    BUFFERS
        .with(|reg| reg.get(handle))
        .map(|buf| Buf(buf, handle))
}

/// The tab page `handle` names, `None` once it has been closed.
pub(crate) fn tabpage(handle: handle_T) -> Option<TabPage> {
    // As [`window`].
    TABPAGES
        .with(|reg| reg.get(handle))
        .map(|tp| TabPage(tp, handle))
}

/// Record `win` as the live window its handle names.
///
/// Called by the window allocator, and again by `aucmd_prepbuf` when it puts
/// the reused autocommand window back on a list.
pub(crate) fn register_window(win: Win) {
    let (handle, raw) = (win.handle(), win.raw());
    WINDOWS.with_mut(|reg| reg.register(handle, raw));
}

/// Forget the window `handle` names, before its memory goes back — or, for
/// the autocommand window, while it is idle and must not be findable.
pub(crate) fn forget_window(handle: handle_T) {
    WINDOWS.with_mut(|reg| reg.forget(handle));
}

/// [`register_window`] for a buffer, called once its number is assigned.
pub(crate) fn register_buffer(buf: Buf) {
    let (handle, raw) = (buf.handle(), buf.raw());
    BUFFERS.with_mut(|reg| reg.register(handle, raw));
}

/// [`forget_window`] for a buffer.
pub(crate) fn forget_buffer(handle: handle_T) {
    BUFFERS.with_mut(|reg| reg.forget(handle));
}

/// [`register_window`] for a tab page.
pub(crate) fn register_tabpage(tp: TabPage) {
    let (handle, raw) = (tp.handle(), tp.raw());
    TABPAGES.with_mut(|reg| reg.register(handle, raw));
}

/// [`forget_window`] for a tab page.
pub(crate) fn forget_tabpage(handle: handle_T) {
    TABPAGES.with_mut(|reg| reg.forget(handle));
}

// ---------------------------------------------------------------------------
// Freed while an autocommand is running
//
// A window or buffer closed from inside an autocommand cannot have its
// allocation given back at once: the handler that closed it, and everything
// below it in the nesting, may still hold the address. Upstream parks the
// object on a chain threaded through the very `b_next`/`w_next` fields the
// editor's own buffer and window lists use (`au_pending_free_buf`,
// `au_pending_free_win`), and the outermost `apply_autocmds` walks the chain
// once `autocmd_busy` goes false again.
//
// Here the pending set owns its storage ([`PendingFree`]), so those two
// fields have one job. Nothing else changes: `free_buffer`/`win_free` still
// park under exactly the same `autocmd_busy` test, `apply_autocmds` still
// drains at exactly the same point, buffers still go before windows, and the
// order within each is still last-deferred-first-freed.

/// Buffers whose allocation is waiting for the outermost autocommand.
static PENDING_FREE_BUFFERS: GlobalCell<PendingFree<buf_T>> = GlobalCell::new(PendingFree::new());

/// Windows whose allocation is waiting for the outermost autocommand.
static PENDING_FREE_WINDOWS: GlobalCell<PendingFree<win_T>> = GlobalCell::new(PendingFree::new());

/// Park `buf`'s allocation until the outermost autocommand returns.
///
/// Everything else about the buffer is torn down already and its handle is
/// out of the registry; what is left is the memory. The caller must not use
/// `buf` again.
pub(crate) fn defer_free_buffer(buf: Buf) {
    let raw = buf.raw();
    PENDING_FREE_BUFFERS.with_mut(|pending| pending.park(raw));
}

/// [`defer_free_buffer`] for a window.
pub(crate) fn defer_free_window(win: Win) {
    let raw = win.raw();
    PENDING_FREE_WINDOWS.with_mut(|pending| pending.park(raw));
}

/// Give back everything the handlers deferred: the C's two `while` loops at
/// the tail of `apply_autocmds`, run when the outermost firing sees
/// `autocmd_busy` false again.
///
/// The set is asked for one allocation at a time rather than drained, so that
/// no borrow of it is held while a free runs — the same reason the C re-reads
/// its list head each time round.
pub(crate) fn free_deferred() {
    // Each allocation was given up by its owner and nothing has reached it
    // since: the handle left the registry before it was parked.
    while let Some(buf) = PENDING_FREE_BUFFERS.with_mut(PendingFree::take_next) {
        free(buf);
    }
    while let Some(win) = PENDING_FREE_WINDOWS.with_mut(PendingFree::take_next) {
        free(win);
    }
}
