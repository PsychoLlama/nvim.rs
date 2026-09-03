//! The three handle registries, and the allocations an autocommand deferred.
//!
//! The editor hands every window, buffer and tab page a monotone id — a
//! [`handle_T`] — and keeps a table from that id to the object, so that an API
//! call, an RPC message or a Lua callback can name one without holding a
//! pointer across the call that might free it. Upstream spells the three
//! tables as khash maps reached through a raw pointer
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

use core::num::NonZero;

use crate::allocator::Owned;
use crate::buffer::free;
use crate::global_cell::GlobalCell;
use crate::registry::{HandleRegistry, OwnedRegistry, PendingFree};
use crate::types::{buf_T, handle_T, tabpage_T, win_T};
use crate::winlayer::{Buf, TabPage, Win};

/// Every live window, by handle.
static WINDOWS: GlobalCell<HandleRegistry<win_T>> = GlobalCell::new(HandleRegistry::new());

/// Every live buffer, by number. The registry **owns** them.
static BUFFERS: GlobalCell<OwnedRegistry<buf_T>> = GlobalCell::new(OwnedRegistry::new());

/// Every live tab page, by handle. The registry **owns** them.
static TABPAGES: GlobalCell<OwnedRegistry<tabpage_T>> = GlobalCell::new(OwnedRegistry::new());

/// The window `handle` names, `None` once it has been closed.
#[inline]
pub(crate) fn window(handle: handle_T) -> Option<Win> {
    // The borrow ends with the lookup, which cannot re-enter.
    WINDOWS.with(|reg| reg.get(handle)).map(Win)
}

/// The buffer numbered `handle`, `None` once it has been wiped.
#[inline]
pub(crate) fn buffer(handle: handle_T) -> Option<Buf> {
    // As [`window`].
    BUFFERS.with(|reg| reg.get(handle)).map(Buf)
}

/// The tab page `handle` names, `None` once it has been closed.
#[inline]
pub(crate) fn tabpage(handle: handle_T) -> Option<TabPage> {
    // As [`window`].
    TABPAGES.with(|reg| reg.get(handle)).map(TabPage)
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

/// Hand `buf` to the buffer registry, which owns it from here on, and answer
/// the [`Buf`] the caller works through.
///
/// Called by the allocator once the buffer's number is assigned — `handle`
/// is that number, which the caller has already written into the buffer.
pub(crate) fn register_buffer(handle: handle_T, buf: Owned<buf_T>) -> Buf {
    Buf(BUFFERS.with_mut(|reg| reg.register(handle, buf)))
}

/// Take the buffer `handle` names out of the registry, handing its
/// allocation back.
///
/// The first thing a free path does, so that nothing can find the buffer
/// while it is being torn down. Dropping what this answers is the free
/// itself; `free_buffer` holds it until the point the `xfree` used to be,
/// and hands it to [`defer_free_buffer`] when an autocommand is running.
#[must_use = "dropping the answer is the free; ignoring it leaks the buffer"]
pub(crate) fn forget_buffer(handle: handle_T) -> Option<Owned<buf_T>> {
    BUFFERS.with_mut(|reg| reg.forget(handle))
}

/// [`register_buffer`] for a tab page.
pub(crate) fn register_tabpage(handle: handle_T, tp: Owned<tabpage_T>) -> TabPage {
    TabPage(TABPAGES.with_mut(|reg| reg.register(handle, tp)))
}

/// [`forget_buffer`] for a tab page.
#[must_use = "dropping the answer is the free; ignoring it leaks the tab page"]
pub(crate) fn forget_tabpage(handle: handle_T) -> Option<Owned<tabpage_T>> {
    TABPAGES.with_mut(|reg| reg.forget(handle))
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

/// Buffers whose allocation is waiting for the outermost autocommand. The
/// set owns them: it took the [`Owned`] the registry gave the free path.
static PENDING_FREE_BUFFERS: GlobalCell<PendingFree<Owned<buf_T>>> =
    GlobalCell::new(PendingFree::new());

/// Windows whose allocation is waiting for the outermost autocommand. A bare
/// address, as the window registry still holds — see [`OwnedRegistry`].
static PENDING_FREE_WINDOWS: GlobalCell<PendingFree<*mut win_T>> =
    GlobalCell::new(PendingFree::new());

/// Park `buf`'s allocation until the outermost autocommand returns.
///
/// Everything else about the buffer is torn down already and its handle is
/// out of the registry; what is left is the memory, which this set owns until
/// [`free_deferred`] drops it. The caller must not use the buffer again.
pub(crate) fn defer_free_buffer(buf: Owned<buf_T>) {
    PENDING_FREE_BUFFERS.with_mut(|pending| pending.park(buf));
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
    // since: the handle left the registry before it was parked. Dropping the
    // buffer's `Owned` runs `buf_T`'s destructor and gives the memory back,
    // outside the `with_mut` so that nothing is borrowed while it runs.
    while let Some(buf) = PENDING_FREE_BUFFERS.with_mut(PendingFree::take_next) {
        drop(buf);
    }
    while let Some(win) = PENDING_FREE_WINDOWS.with_mut(PendingFree::take_next) {
        free(win);
    }
}

// ---------------------------------------------------------------------------
// Identity, and validity after re-entry
//
// A `Win`/`Buf`/`TabPage` is an *address* the caller has promised is live, and
// that promise is exactly what an autocommand breaks. The value to carry
// across such a call is not the address but the **handle**, taken while the
// object is still there -- which is what these three are. They are the
// re-entry rule in the type system: you cannot ask "is it still there?" of
// something whose identity you did not take while it was.

/// A window's identity, taken from a live window: the `handle_T` that names
/// it, with the address dropped.
///
/// Answering [`WinId::get`] costs a registry lookup and reads nothing that
/// belongs to the window, so it is safe to ask about one an autocommand has
/// closed. [`Win::id`] is the only way to make one.
///
/// # Why the handle is a [`NonZero`]
///
/// Zero names no window, no buffer and no tab page: all three counters
/// (`last_win_id`, `top_file_num`, `LAST_TP_HANDLE`) are incremented before
/// they are read, and the editor already spells "no window" as handle `0`
/// (`aco_save_T::save_prevwin_handle`). Excluding it buys the niche, so an
/// `Option<WinId>` is four bytes and **all-zero bytes are `None`** — which
/// is what makes these safe to use as the graph's own list links, since a
/// `win_T`/`buf_T`/`tabpage_T` is born from `xcalloc` or `Box::new_zeroed`
/// and its links have to read as "no neighbour" before anyone writes them.
/// The test at the bottom of this file pins that.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct WinId(pub(super) NonZero<handle_T>);

/// A buffer's identity, taken from a live buffer: its number. [`WinId`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct BufId(pub(super) NonZero<handle_T>);

/// A tab page's identity, taken from a live tab page. [`WinId`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct TabId(pub(super) NonZero<handle_T>);

impl WinId {
    /// The window again, `None` once it has been freed.
    ///
    /// This is the second half of the re-entry rule: `let id = win.id()`
    /// before the call, `id.get()` after it, and a fresh [`Win`] or nothing.
    ///
    /// "Still there" here is **not** `win_valid()`, and the two answer
    /// different questions:
    ///
    /// * `win_valid(wp)` asks whether an **address** is on the **current tab
    ///   page's** window list (`win_valid_any_tab` widens that to every tab
    ///   page). It is a list walk, and it says "no" for a hidden window
    ///   (`win_alloc(_, true)`) that is on no list but perfectly alive.
    /// * This asks whether the **object** still exists. It says "yes" for
    ///   that hidden window, and "no" for the autocommand window while it is
    ///   idle, which `aucmd_restbuf` takes out of the registry.
    ///
    /// Ask this one when the question is "did what I was holding survive the
    /// call I just made"; ask `win_valid` when the question is about layout —
    /// "is this window on screen, on this tab page". Reaching for the wrong
    /// one is a behaviour change, not a style choice. And a caller holding a
    /// bare `*mut win_T` that an autocommand may have freed cannot use this
    /// at all: taking a [`WinId`] from one would read the window, the very
    /// dereference the list walk exists to avoid.
    #[inline(always)]
    pub(crate) fn get(self) -> Option<Win> {
        window(self.0.get())
    }

    /// The bare handle, for the API and RPC edges that speak in numbers.
    #[inline(always)]
    pub(crate) fn handle(self) -> handle_T {
        self.0.get()
    }
}

impl BufId {
    /// The buffer again, `None` once it has been wiped. [`WinId::get`].
    #[inline(always)]
    pub(crate) fn get(self) -> Option<Buf> {
        buffer(self.0.get())
    }

    /// Whether the buffer is still registered — [`WinId::get`]'s question,
    /// as a `bool`, with the same warning: this is not `buf_valid`'s walk of
    /// the buffer list, and the two off-list buffers (`ml_recover`'s scratch,
    /// `open_spellbuf`'s dummy) are registered by nobody and answer `false`.
    #[inline(always)]
    pub(crate) fn valid(self) -> bool {
        self.get().is_some()
    }
}

impl TabId {
    /// The tab page again, `None` once it has been closed. [`WinId::get`].
    #[inline(always)]
    pub(crate) fn get(self) -> Option<TabPage> {
        tabpage(self.0.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The property the graph's list links rest on. `NonZero`'s only niche
    /// is zero, so an `Option` the size of the handle itself is one whose
    /// `None` *is* the all-zero word — which is what lets a `win_T`,
    /// `buf_T` or `tabpage_T` come out of `xcalloc`/`Box::new_zeroed` with
    /// its links already reading "no neighbour". Nothing in the language
    /// promises the niche is taken, so it is asserted rather than assumed;
    /// were it ever dropped, reading a zeroed link would be UB and Miri
    /// would say so, but this fails first and says why.
    #[test]
    fn an_absent_id_is_the_zero_word() {
        assert_eq!(size_of::<Option<WinId>>(), size_of::<handle_T>());
        assert_eq!(size_of::<Option<BufId>>(), size_of::<handle_T>());
        assert_eq!(size_of::<Option<TabId>>(), size_of::<handle_T>());
    }
}
