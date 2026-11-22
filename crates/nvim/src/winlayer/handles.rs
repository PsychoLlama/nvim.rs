//! The three handle registries, and the allocations an autocommand deferred.
//!
//! The editor hands every window, buffer and tab page a monotone id — a
//! [`Handle`] — and keeps a table from that id to the object, so that an API
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

use core::ffi::c_int;
use core::num::NonZero;

use crate::global_cell::GlobalCell;
use crate::registry::{HandleRegistry, OwnedRegistry, PendingFree};
use crate::slot::{Pooled, SlotBox, SlotPool, SlotRef};
use crate::types::{Buffer, Frame, Handle, Tabpage, Window};
use crate::winlayer::graph::ParkedWindow;
use crate::winlayer::{Buf, FrameRef, TabPage, Win};

// ---------------------------------------------------------------------------
// Where the objects live
//
// Every window, buffer, tab page and frame is a value in a type-stable slot
// of its type's pool (`crate::slot`): the slot's memory is never given back,
// so a handle that outlives its object still points at a slot, whose
// generation word says whether the object it was made for is still there.
// Freeing one ends the value in its slot at the moment the `xfree` used to
// run and moves the generation on.

/// The slots windows live in.
static WINDOW_SLOTS: GlobalCell<SlotPool<Window>> = GlobalCell::new(SlotPool::new());

/// The slots buffers live in, the two unregistered ones included.
static BUFFER_SLOTS: GlobalCell<SlotPool<Buffer>> = GlobalCell::new(SlotPool::new());

/// The slots tab pages live in.
static TABPAGE_SLOTS: GlobalCell<SlotPool<Tabpage>> = GlobalCell::new(SlotPool::new());

/// The slots frames live in, the snapshot trees' included.
static FRAME_SLOTS: GlobalCell<SlotPool<Frame>> = GlobalCell::new(SlotPool::new());

// `inline(never)` on all four: a small leaf method of a trait impl for a
// `pub` type is shippable cross-crate, and a shippable body that names a
// static exports it (`just state-exports`). The pools are reached on
// allocation and free only.
impl Pooled for Window {
    #[inline(never)]
    fn pool() -> &'static GlobalCell<SlotPool<Self>> {
        &WINDOW_SLOTS
    }
}

impl Pooled for Buffer {
    #[inline(never)]
    fn pool() -> &'static GlobalCell<SlotPool<Self>> {
        &BUFFER_SLOTS
    }
}

impl Pooled for Tabpage {
    #[inline(never)]
    fn pool() -> &'static GlobalCell<SlotPool<Self>> {
        &TABPAGE_SLOTS
    }
}

impl Pooled for Frame {
    #[inline(never)]
    fn pool() -> &'static GlobalCell<SlotPool<Self>> {
        &FRAME_SLOTS
    }
}

/// Every live window, by handle.
static WINDOWS: GlobalCell<HandleRegistry<SlotRef<Window>>> =
    GlobalCell::new(HandleRegistry::new());

/// Every live buffer, by number. The registry **owns** them.
static BUFFERS: GlobalCell<OwnedRegistry<Buffer>> = GlobalCell::new(OwnedRegistry::new());

/// Every live tab page, by handle. The registry **owns** them.
static TABPAGES: GlobalCell<OwnedRegistry<Tabpage>> = GlobalCell::new(OwnedRegistry::new());

/// Every live frame, by handle. The registry **owns** them.
///
/// Frames have no handle of their own upstream -- they are reached only by
/// pointer, out of `w_frame`, `tp_topframe`, `tp_snapshot` and each other --
/// and that is exactly why they needed one: `winframe_remove` frees a frame
/// under whoever was holding it, and `close_windows` frees the whole subtree.
/// The registry also holds the *snapshot* trees, which are frames by every
/// other measure and are freed a `:diffsplit` later.
static FRAMES: GlobalCell<OwnedRegistry<Frame>> = GlobalCell::new(OwnedRegistry::new());

/// The window `handle` names, `None` once it has been closed.
#[inline]
pub(crate) fn window(handle: Handle) -> Option<Win> {
    // The borrow ends with the lookup, which cannot re-enter. The handle is
    // the key that found it, so the [`Win`] is assembled without reading the
    // window: a lookup answers about an object it never touches.
    WINDOWS
        .with(|reg| reg.get(handle))
        .map(|slot| Win::at(slot, handle))
}

/// The buffer numbered `handle`, `None` once it has been wiped.
#[inline]
pub(crate) fn buffer(handle: Handle) -> Option<Buf> {
    // As [`window`].
    BUFFERS
        .with(|reg| reg.get(handle))
        .map(|slot| Buf::at(slot, handle))
}

/// The tab page `handle` names, `None` once it has been closed.
#[inline]
pub(crate) fn tabpage(handle: Handle) -> Option<TabPage> {
    // As [`window`].
    TABPAGES
        .with(|reg| reg.get(handle))
        .map(|slot| TabPage::at(slot, handle))
}

/// The frame `handle` names, `None` once it has been freed.
#[inline]
pub(crate) fn frame(handle: Handle) -> Option<FrameRef> {
    // As [`window`].
    FRAMES
        .with(|reg| reg.get(handle))
        .map(|slot| FrameRef::at(slot, handle))
}

/// Record `win` as the live window its handle names.
///
/// Called by the window allocator, and again by `aucmd_prepbuf` when it puts
/// the reused autocommand window back on a list.
pub(crate) fn register_window(win: Win) {
    let slot = win.slot().expect("a window being registered is not null");
    assert!(
        slot.is_live(),
        "a window being registered is live in its slot"
    );
    WINDOWS.with_mut(|reg| reg.register(win.handle(), slot));
}

/// Take the window `handle` names out of the registry while it stays alive:
/// the autocommand window, idle between uses and not to be findable.
pub(crate) fn hide_window(handle: Handle) {
    let _ = WINDOWS.with_mut(|reg| reg.forget(handle));
}

/// Take `win` out of the registry, answering its slot: the free path's half
/// of [`super::graph::forget_window`].
pub(super) fn unregister_window(win: Win) -> Option<SlotRef<Window>> {
    let slot = WINDOWS.with_mut(|reg| reg.forget(win.handle()))?;
    assert!(
        same_value(slot, win.raw(), win.generation()),
        "the window being freed is the one registered under its handle"
    );
    Some(slot)
}

/// Hand `buffer` to the buffer registry, which owns it from here on, and answer
/// the [`Buf`] the caller works through.
///
/// Called by the allocator once the buffer's number is assigned — `handle`
/// is that number, which the caller has already written into the buffer.
pub(crate) fn register_buffer(handle: Handle, buffer: SlotBox<Buffer>) -> Buf {
    Buf::at(BUFFERS.with_mut(|reg| reg.register(handle, buffer)), handle)
}

/// Take `buffer` out of the registry, handing its owner back.
///
/// The first thing a free path does, so that nothing can find the buffer
/// while it is being torn down. Dropping what this answers is the free
/// itself; `free_buffer` holds it until the point the `xfree` used to be,
/// and hands it to [`defer_free_buffer`] when an autocommand is running.
#[must_use = "dropping the answer is the free; ignoring it leaks the buffer"]
pub(crate) fn forget_buffer(buffer: Buf) -> Option<SlotBox<Buffer>> {
    let owned = BUFFERS.with_mut(|reg| reg.forget(buffer.handle()))?;
    assert!(
        same_value(owned.handle(), buffer.raw(), buffer.generation()),
        "the buffer being freed is the one registered under its number"
    );
    Some(owned)
}

/// Whether `slot` is the value a handle of address `raw` and generation
/// `generation` was made for: what the free paths check of the handle they
/// were given against the registry's.
fn same_value<T>(slot: SlotRef<T>, raw: *mut T, generation: u32) -> bool {
    core::ptr::eq(slot.raw(), raw) && slot.generation() == generation
}

/// [`register_buffer`] for a tab page.
pub(crate) fn register_tabpage(handle: Handle, tabpage: SlotBox<Tabpage>) -> TabPage {
    TabPage::at(
        TABPAGES.with_mut(|reg| reg.register(handle, tabpage)),
        handle,
    )
}

/// [`forget_buffer`] for a tab page.
#[must_use = "dropping the answer is the free; ignoring it leaks the tab page"]
pub(crate) fn forget_tabpage(tabpage: TabPage) -> Option<SlotBox<Tabpage>> {
    let owned = TABPAGES.with_mut(|reg| reg.forget(tabpage.handle()))?;
    assert!(
        same_value(owned.handle(), tabpage.raw(), tabpage.generation()),
        "the tab page being freed is the one registered under its handle"
    );
    Some(owned)
}

/// [`register_buffer`] for a frame. `handle` is the number
/// `window::new_frame` took from the frame counter and wrote into it.
pub(crate) fn register_frame(handle: Handle, frame: SlotBox<Frame>) -> FrameRef {
    FrameRef::at(FRAMES.with_mut(|reg| reg.register(handle, frame)), handle)
}

/// [`forget_buffer`] for a frame.
#[must_use = "dropping the answer is the free; ignoring it leaks the frame"]
pub(crate) fn forget_frame(frame: FrameRef) -> Option<SlotBox<Frame>> {
    let owned = FRAMES.with_mut(|reg| reg.forget(frame.handle()))?;
    assert!(
        same_value(owned.handle(), frame.raw(), frame.generation()),
        "the frame being freed is the one registered under its handle"
    );
    Some(owned)
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
// once `AUTOCMD_BUSY` goes false again.
//
// Here the pending set owns its storage ([`PendingFree`]), so those two
// fields have one job. Nothing else changes: `free_buffer`/`win_free` still
// park under exactly the same `AUTOCMD_BUSY` test, `apply_autocmds` still
// drains at exactly the same point, buffers still go before windows, and the
// order within each is still last-deferred-first-freed.

/// Buffers whose allocation is waiting for the outermost autocommand. The
/// set owns them: it took the [`SlotBox`] the registry gave the free path.
static PENDING_FREE_BUFFERS: GlobalCell<PendingFree<SlotBox<Buffer>>> =
    GlobalCell::new(PendingFree::new());

/// Windows whose block is waiting for the outermost autocommand. The set
/// owns them: it took the [`ParkedWindow`] the free path was given.
static PENDING_FREE_WINDOWS: GlobalCell<PendingFree<ParkedWindow>> =
    GlobalCell::new(PendingFree::new());

/// Park `buffer`'s allocation until the outermost autocommand returns.
///
/// Everything else about the buffer is torn down already and its handle is
/// out of the registry; what is left is the memory, which this set owns until
/// [`free_deferred`] drops it. The caller must not use the buffer again.
pub(crate) fn defer_free_buffer(buffer: SlotBox<Buffer>) {
    PENDING_FREE_BUFFERS.with_mut(|pending| pending.park(buffer));
}

/// [`defer_free_buffer`] for a window's block.
pub(crate) fn defer_free_window(block: ParkedWindow) {
    PENDING_FREE_WINDOWS.with_mut(|pending| pending.park(block));
}

/// Give back everything the handlers deferred: the C's two `while` loops at
/// the tail of `apply_autocmds`, run when the outermost firing sees
/// `AUTOCMD_BUSY` false again.
///
/// The set is asked for one allocation at a time rather than drained, so that
/// no borrow of it is held while a free runs — the same reason the C re-reads
/// its list head each time round.
pub(crate) fn free_deferred() {
    // Each allocation was given up by its owner and nothing has reached it
    // since: the handle left the registry before it was parked. Dropping the
    // buffer's `SlotBox` runs `Buffer`'s destructor in its slot and empties
    // it, and dropping a `ParkedWindow` empties its window's; both outside
    // the `with_mut`, so that nothing is borrowed while they run.
    while let Some(buf) = PENDING_FREE_BUFFERS.with_mut(PendingFree::take_next) {
        drop(buf);
    }
    while let Some(block) = PENDING_FREE_WINDOWS.with_mut(PendingFree::take_next) {
        drop(block);
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

/// A window's identity, taken from a live window: the `Handle` that names
/// it, with the address dropped.
///
/// Answering [`WinId::get`] costs a registry lookup and reads nothing that
/// belongs to the window, so it is safe to ask about one an autocommand has
/// closed. [`Win::id`] is the only way to make one.
///
/// # Why the handle is a [`NonZero`]
///
/// Zero names no window, no buffer and no tab page: all three counters
/// (`LAST_WIN_ID`, `TOP_FILE_NUM`, `LAST_TP_HANDLE`) are incremented before
/// they are read, and the editor already spells "no window" as handle `0`
/// (`AcoSave::save_prevwin_handle`). Excluding it buys the niche, so an
/// `Option<WinId>` is four bytes and **all-zero bytes are `None`** — which
/// is what makes these safe to use as the graph's own list links, since a
/// `Window`/`Buffer`/`Tabpage` is born from `xcalloc` or `Box::new_zeroed`
/// and its links have to read as "no neighbour" before anyone writes them.
/// The test at the bottom of this file pins that.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct WinId(pub(super) NonZero<c_int>);

/// A buffer's identity, taken from a live buffer: its number. [`WinId`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BufId(pub(super) NonZero<c_int>);

/// A tab page's identity, taken from a live tab page. [`WinId`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct TabId(pub(super) NonZero<c_int>);

/// A frame's identity, taken from a live frame. [`WinId`].
///
/// This is what the layout tree's own links are made of. A frame is freed
/// under its holders all the time -- `winframe_remove` frees the leaf whose
/// window is closing and `flatten` frees the parent it collapses -- so the
/// links, `w_frame`, `tp_topframe` and `tp_snapshot` all name a frame by
/// identity, and a walk that arrives at a freed one reads `None` rather than
/// whatever the allocator has since put there.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct FrameId(pub(super) NonZero<c_int>);

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
    /// bare `*mut Window` that an autocommand may have freed cannot get here
    /// at all: **building** the [`Win`] reads the window's handle, which is
    /// the very dereference the list walk exists to avoid. `window_at` is
    /// that caller's answer.
    #[inline(always)]
    pub(crate) fn get(self) -> Option<Win> {
        window(Handle::new(self.0.get()))
    }

    /// The bare handle, for the API and RPC edges that speak in numbers.
    #[inline(always)]
    pub(crate) fn handle(self) -> Handle {
        Handle::new(self.0.get())
    }
}

impl BufId {
    /// The buffer again, `None` once it has been wiped. [`WinId::get`].
    #[inline(always)]
    pub(crate) fn get(self) -> Option<Buf> {
        buffer(Handle::new(self.0.get()))
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
        tabpage(Handle::new(self.0.get()))
    }
}

impl FrameId {
    /// The frame again, `None` once it has been freed. [`WinId::get`].
    #[inline(always)]
    pub(crate) fn get(self) -> Option<FrameRef> {
        frame(Handle::new(self.0.get()))
    }
}

// ---------------------------------------------------------------------------
// What a handle carries beside its address

/// A handle's identity and its slot's generation, packed in one word: the
/// generation in the low half, the handle in the high. The generation is
/// what every dereference compares with the slot's header, and in the low
/// half that is one `cmp` of the register's low 32 bits against memory, with
/// no shift.
///
/// One word rather than two fields so that a handle is a pointer and a
/// scalar, which the Rust ABI passes in two registers. With a third field a
/// handle became a memory aggregate passed by reference to every callee
/// that was not inlined, and the hot walks paid for it (+1.2 to +1.6 % of
/// the instructions mlbench, scrbench and spellbench retire).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Stamp(u64);

impl Stamp {
    /// No identity and no slot: the null handle's, and "nothing is current".
    pub(crate) const NONE: Self = Stamp(0);

    #[inline(always)]
    pub(crate) fn new(handle: Handle, generation: u32) -> Self {
        Stamp((u64::from(handle.get().cast_unsigned()) << 32) | u64::from(generation))
    }

    #[inline(always)]
    pub(crate) fn handle(self) -> Handle {
        let high = u32::try_from(self.0 >> 32).unwrap_or_default();
        Handle::new(high.cast_signed())
    }

    #[inline(always)]
    pub(crate) fn generation(self) -> u32 {
        u32::try_from(self.0 & 0xFFFF_FFFF).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The property the graph's list links rest on. `NonZero`'s only niche
    /// is zero, so an `Option` the size of the handle itself is one whose
    /// `None` *is* the all-zero word — which is what lets a `Window`,
    /// `Buffer` or `Tabpage` come out of `xcalloc`/`Box::new_zeroed` with
    /// its links already reading "no neighbour". Nothing in the language
    /// promises the niche is taken, so it is asserted rather than assumed;
    /// were it ever dropped, reading a zeroed link would be UB and Miri
    /// would say so, but this fails first and says why.
    #[test]
    fn an_absent_id_is_the_zero_word() {
        assert_eq!(size_of::<Option<WinId>>(), size_of::<Handle>());
        assert_eq!(size_of::<Option<BufId>>(), size_of::<Handle>());
        assert_eq!(size_of::<Option<TabId>>(), size_of::<Handle>());
        assert_eq!(size_of::<Option<FrameId>>(), size_of::<Handle>());
    }
}
