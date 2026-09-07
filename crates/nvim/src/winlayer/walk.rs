//! The editor's lists, walked: the C's `FOR_ALL_*` macros.
//!
//! Each of these is one of the C's `FOR_ALL_*` macros. The lists are the
//! editor's own: they are built before the first window is drawn and torn down
//! only at exit, and every chain ends — at a `None` link for the buffer list,
//! at a null pointer for the window and frame ones — so producing the head and
//! stepping the chain needs no promise from the caller, which is what makes
//! these safe functions rather than `unsafe fn`s. A walk that its own body can
//! invalidate is a different matter and stays the caller's problem: none of
//! these re-reads the head, exactly as the macros do not.
//!
//! # When the link is read
//!
//! **Every walk here reads the next link *before* the body runs, not after.**
//! `iter::successors` calls its closure at yield time — `let item =
//! self.next.take()?; self.next = (self.succ)(&item); Some(item)` — so these
//! are the C's `FOR_ALL_*_SAFE` shape, not `FOR_ALL_*`'s. The macro's
//! `buf = buf->b_next` increment runs *after* its body.
//!
//! The difference is only visible to a body that touches the element it is
//! standing on, and it cuts one way each: freeing that element is safe here
//! and a use-after-free in the macro, while *relinking* it is followed by the
//! macro and ignored here, because the neighbour was read already. Neither
//! showed in the suites, but a caller that relinks under itself must not use
//! these — `buffer::info`'s `:ls` walk is the one in the tree that needs the
//! macro's timing and spells its own `step` out to get it.
//!
//! # The links are handles
//!
//! `b_next`/`b_prev` are `Option<BufId>`, and `firstbuf`/`lastbuf` with them:
//! a step is a registry lookup, not a load. That is what makes the object
//! graph index-shaped rather than pointer-shaped — a buffer can move, and a
//! link can never name one that has been freed, because the free path takes
//! the handle out of the registry (after the unlink — see `registry`). The
//! cost is one `HandleMap` probe per step, a subtract and a load, which is
//! why that type exists at all.
//! Every walk here is safe, and the module is `forbid(unsafe_code)` to keep it
//! that way: a step is a `Win`/`Buf`/`TabPage`/`FrameRef` accessor, and those
//! carry the promise. [`winlayer`](super) itself cannot take the attribute --
//! it is where the promise is *made* -- which is why this is a file of its own
//! rather than a section of that one.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::iter;

use super::graph::{
    cmdline_win, cmdwin_win, first_tabpage, firstbuf, firstwin, lastbuf, lastused_tabpage, lastwin,
    prevwin,
};
use super::{Buf, BufId, FrameRef, TabId, TabPage, Win, WinId};
use crate::types::{Buffer, Window};

/// `first` and every window after it in its tab page's list.
pub(crate) fn windows_from(first: Option<Win>) -> impl Iterator<Item = Win> {
    iter::successors(first, |wp| wp.next())
}

/// The head of the current tab page's window list, `None` only before the
/// first window exists and while the editor is tearing the last one down.
#[inline]
pub(crate) fn first_window() -> Option<Win> {
    firstwin.get().and_then(WinId::get)
}

/// The tail of the current tab page's window list. [`first_window`].
#[inline]
pub(crate) fn last_window() -> Option<Win> {
    lastwin.get().and_then(WinId::get)
}

/// The window `CTRL-W p` goes back to -- the C's `prevwin` -- `None` when
/// there is none or it has been closed since it was named.
///
/// The five below are the same shape: a global that remembers one window,
/// buffer or tab page across a call that may free it, and answers `None`
/// rather than a dangling address when it did.
#[inline]
pub(crate) fn prev_window() -> Option<Win> {
    prevwin.get().and_then(WinId::get)
}

/// The tab page `:tab` and `g<Tab>` go back to -- `lastused_tabpage`.
#[inline]
pub(crate) fn last_used_tab() -> Option<TabPage> {
    lastused_tabpage.get().and_then(TabId::get)
}

/// The command-line window, while one is open -- `cmdwin_win`.
#[inline]
pub(crate) fn cmdwin_window() -> Option<Win> {
    cmdwin_win.get().and_then(WinId::get)
}

/// The window the cmdline is drawn in when `'cmdheight'` is zero and a float
/// stands in for the message area -- `cmdline_win`.
#[inline]
pub(crate) fn cmdline_window() -> Option<Win> {
    cmdline_win.get().and_then(WinId::get)
}

/// Every window of the current tab page, in list order: the C's
/// `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)`, whose `curtab == curtab` test always
/// picks `firstwin`.
pub(crate) fn windows() -> impl Iterator<Item = Win> {
    windows_from(first_window())
}

/// Every window of tab page `tabpage`, in list order: `FOR_ALL_WINDOWS_IN_TAB`.
///
/// The current tab page's windows hang off the `firstwin` global rather than
/// off its own `tp_firstwin`, which is stale while it is current — that is
/// what the macro's first arm reads.
pub(crate) fn windows_in_tab(tabpage: TabPage) -> impl Iterator<Item = Win> {
    windows_from(if tabpage.is_current() {
        first_window()
    } else {
        tabpage.tp_firstwin.and_then(WinId::get)
    })
}

/// The window at address `raw`, if it is on the current tab page's list.
///
/// The one lookup that still speaks in addresses, and it lives here for the
/// reason `winlayer`'s docs give: the caller is holding a `Window *` an
/// autocommand may already have freed (a layout snapshot's `fr_win`), so the
/// address can only be *compared*, never read. Everything else asks by
/// [`WinId`].
pub(crate) fn window_at(raw: *const Window) -> Option<Win> {
    (!raw.is_null())
        .then(|| windows().find(|wp| wp.raw().cast_const() == raw))
        .flatten()
}

/// The buffer at address `raw`, if it is still on the buffer list -- the C's
/// `buf_valid()`.
///
/// The buffer twin of [`window_at`], and here for the same reason: a caller
/// holding a `Buffer *` out of a window's `w_buffer`, a tab page's
/// `tp_diffbuf` or a saved scan state is holding an address an autocommand
/// may already have freed, so it can only be *compared*. A caller that still
/// has its buffer live asks [`BufId::get`] instead.
pub(crate) fn buffer_at(raw: *const Buffer) -> Option<Buf> {
    (!raw.is_null())
        .then(|| buffers_back().find(|buf| buf.raw().cast_const() == raw))
        .flatten()
}

/// Every tab page, in list order: the C's `FOR_ALL_TABS`.
pub(crate) fn tabs() -> impl Iterator<Item = TabPage> {
    iter::successors(first_tab(), |tp| tp.next())
}

/// The head of the editor's tab page list, `None` only before the first one
/// is made. [`first_buffer`].
#[inline]
pub(crate) fn first_tab() -> Option<TabPage> {
    first_tabpage.get().and_then(TabId::get)
}

/// Every window of every tab page: `FOR_ALL_TAB_WINDOWS`, which is exactly
/// [`tabs`] with [`windows_in_tab`] inside it. `tp_next` is read after the tab
/// page's own windows are exhausted, as the macro's outer `for` reads it.
pub(crate) fn tab_windows() -> impl Iterator<Item = Win> {
    tabs().flat_map(windows_in_tab)
}

/// `first` and every frame after it in its row or column: the C's
/// `FOR_ALL_FRAMES(frp, first)`, whose head is usually a `fr_child`.
pub(crate) fn frames(first: Option<FrameRef>) -> impl Iterator<Item = FrameRef> {
    iter::successors(first, |fr| fr.next())
}

/// [`frames`] the other way, following `fr_prev`. The C spells this out as a
/// `while` loop each time it needs it (`frame_setheight`'s second run, say).
pub(crate) fn frames_back(first: Option<FrameRef>) -> impl Iterator<Item = FrameRef> {
    iter::successors(first, |fr| fr.prev())
}

/// Every buffer, in list order: the C's `FOR_ALL_BUFFERS`.
pub(crate) fn buffers() -> impl Iterator<Item = Buf> {
    iter::successors(first_buffer(), |buf| buf.next())
}

/// The head of the editor's buffer list, `None` before the first buffer is
/// created and again once the last one is gone.
#[inline]
pub(crate) fn first_buffer() -> Option<Buf> {
    firstbuf.get().and_then(BufId::get)
}

/// The tail of the editor's buffer list. [`first_buffer`].
#[inline]
pub(crate) fn last_buffer() -> Option<Buf> {
    lastbuf.get().and_then(BufId::get)
}

/// Every buffer, last to first: the C's `FOR_ALL_BUFFERS_BACKWARDS`.
pub(crate) fn buffers_back() -> impl Iterator<Item = Buf> {
    iter::successors(last_buffer(), |buf| buf.prev())
}

/// Every window of the current tab page, last to first. The C spells this
/// out as a `while` loop each time it needs it — the float walks in
/// `winfloat` and `window::size` are the two.
pub(crate) fn windows_back() -> impl Iterator<Item = Win> {
    iter::successors(last_window(), |wp| wp.prev())
}
