//! The frame handle's own accessors: the layout tree, walked by identity.
//!
//! A [`FrameRef`] is a frame the caller has promised is live, carrying the
//! frame's handle beside its address ([`crate::winlayer::Win`]'s shape). What
//! is here is everything that can be asked of one without a raw dereference:
//! its identity, and the five edges of the layout tree — the window a leaf
//! holds, the parent it hangs off, and the first child, next and previous
//! sibling that make up a row or a column.
//!
//! A child of [`crate::winlayer`] for the reason [`super::handles`] is: the
//! private fields of the handle are visible to a descendant module, so
//! reading one costs no `unsafe` and this file can `forbid` it. Nothing here
//! dereferences a frame — a [`FrameRef`] is only ever built from a registry
//! entry, whose handle is the key that found it — which is what lets the
//! whole layout tree be walked without one.

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
use crate::global_cell::GlobalCell;
use crate::types::{Frame, Handle};
use crate::winlayer::graph::topframe;
use crate::winlayer::{FrameId, FrameRef, Win, WinId, forget_frame, frames, register_frame};

/// A fresh frame, zeroed and registered, with nothing linked to it yet.
///
/// The C's `xcalloc(1, sizeof(frame_T))`, which each of the three allocation
/// sites spelled for itself: the leaf a window is given, the row or column a
/// split wraps around one, and the copies a layout snapshot is made of. All
/// three come here now, so a [`FrameId`] taken from any of them can be asked
/// whether the frame is still there.
///
/// The allocator lives beside the registry rather than in `window::alloc`,
/// where the window's and the tab page's do, because minting the handle and
/// filing the frame under it is all this is — and written out as a literal
/// rather than as zeroed bytes, it needs no `unsafe` at all.
pub(crate) fn new_frame() -> FrameRef {
    /// The frame handles issued so far. Incremented before it is read, as
    /// `last_win_id` is, so no frame carries handle zero.
    static LAST_FRAME_ID: GlobalCell<Handle> = GlobalCell::new(0);
    LAST_FRAME_ID.set(LAST_FRAME_ID.get() + 1);
    let handle = LAST_FRAME_ID.get();
    let frame = Owned::new(Box::new(Frame {
        handle,
        fr_layout: 0,
        fr_width: 0,
        fr_newwidth: 0,
        fr_height: 0,
        fr_newheight: 0,
        fr_parent: None,
        fr_next: None,
        fr_prev: None,
        fr_child: None,
        fr_win: None,
    }));
    register_frame(handle, frame)
}

/// The root of the current tab page's layout tree.
///
/// The `topframe` global, resolved. It mirrors the current tab page's
/// `tp_topframe` and is set from startup to exit, so this cannot answer
/// nothing — a `None` here would mean the mirror named a freed frame.
#[inline]
pub(crate) fn current_topframe() -> FrameRef {
    topframe
        .get()
        .and_then(FrameId::get)
        .expect("the editor always has a layout tree")
}

/// Give `frp`'s memory back. It must already be out of the tree.
///
/// The C's `xfree(frp)`. Taking it out of the registry is what makes every
/// [`FrameId`] still naming it answer `None` from here on, and dropping what
/// the registry hands back is the free itself — at the point the `xfree` was.
pub(crate) fn free_frame(frp: FrameRef) {
    drop(forget_frame(frp.handle()));
}

impl FrameRef {
    /// [`Win::at`] for a frame.
    #[inline(always)]
    pub(super) const fn at(raw: *mut Frame, handle: Handle) -> Self {
        Self {
            ptr: raw,
            id: handle,
        }
    }

    #[inline(always)]
    pub fn raw(self) -> *mut Frame {
        self.ptr
    }

    /// This frame's handle: its key in the frame registry, as a field load.
    /// [`Win::handle`].
    #[inline(always)]
    pub(crate) fn handle(self) -> Handle {
        self.id
    }

    /// This frame's identity, the value the tree's own links are made of and
    /// the one to hold across anything that can free a frame. [`WinId`].
    ///
    /// # Panics
    ///
    /// When the handle is zero, which no registered frame has.
    #[inline(always)]
    pub(crate) fn id(self) -> FrameId {
        FrameId(NonZero::new(self.id).expect("a live frame has a handle"))
    }

    /// The window this frame holds — `Some` exactly for a leaf, and for a
    /// snapshot's leaf only while the window it remembers is still there.
    #[inline(always)]
    pub fn win(self) -> Option<Win> {
        self.fr_win.and_then(WinId::get)
    }

    /// The frame this one is a child of — `None` only for the tab page's
    /// `topframe`, the one frame with no parent.
    #[inline(always)]
    pub fn parent(self) -> Option<Self> {
        self.fr_parent.and_then(FrameId::get)
    }

    /// This frame's first child, which every non-leaf frame has.
    #[inline(always)]
    pub fn child(self) -> Option<Self> {
        self.fr_child.and_then(FrameId::get)
    }

    /// The frame beside this one, if it is not the last of its row or column.
    #[inline(always)]
    pub fn next(self) -> Option<Self> {
        self.fr_next.and_then(FrameId::get)
    }

    /// The frame before this one, if it is not the first of its row or column.
    #[inline(always)]
    pub fn prev(self) -> Option<Self> {
        self.fr_prev.and_then(FrameId::get)
    }

    /// This frame's children, first to last: the C's
    /// `FOR_ALL_FRAMES(frp, topfrp->fr_child)`, which is empty for a leaf.
    #[inline(always)]
    pub fn children(self) -> impl Iterator<Item = Self> {
        frames(self.child())
    }
}
