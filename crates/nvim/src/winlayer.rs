//! The window, buffer and position pointers the editor works through, wrapped
//! so that dereferencing one is not an unsafe operation at every use.
//!
//! The transpiled editor passes `*mut Window` / `*mut Buffer` / `*mut Pos`
//! everywhere, and the pointers have to stay raw: callers interleave these
//! calls with reads of the `curwin`/`curbuf` globals — which alias the same
//! objects — and many of them re-enter through autocommands, so a long-lived
//! `&mut` would invalidate a pointer the caller still holds.
//!
//! What does not have to stay raw is the *dereference*. [`Win`], [`Buf`],
//! [`FrameRef`], [`TabPage`], [`PosRef`] and [`Line`] each wrap one pointer and
//! make its **construction** the unsafe step; from there [`Deref`]/[`DerefMut`]
//! give ordinary field access and the accessors below give the projections a
//! bare `&`/`&mut` cannot express — the buffer behind a window, a line of that
//! buffer. Every one rests on the single promise the constructor took, which
//! each `pub unsafe fn` in a consumer restates in its own `# Safety` section.
//! A family adds the wrappers it needs as its own `impl Win` block (an
//! inherent impl may live in any module of the defining crate) — the fold
//! projections are in `fold`, the ones a registry needs in [`handles`] — so
//! this module stays the shared minimum.
//!
//! The three **handle registries** are that child [`handles`], along with the
//! deferred-free set. They are the one place a `Win`/`Buf`/`TabPage` is built
//! from a handle rather than from a pointer a caller already had, and because
//! the registry's own invariant is that everything in it is live, [`window`],
//! [`buffer`] and [`tabpage`] are **safe** functions.
//!
//! # Who owns the object
//!
//! **The buffer and tab page registries own what they hold.** A `Buffer` and
//! a `Tabpage` are `Box`es the registry took (`allocator::Owned`), so the
//! free path takes the allocation back with the handle and *drops* it where
//! the `xfree` used to be — which is what lets a `Buffer` hold a `Vec`.
//! **Windows are not there yet**: `aucmd_restbuf` takes the autocommand
//! window out of the registry while it stays alive and `aucmd_prepbuf` puts
//! it back, so "registered" and "owned" are different lifetimes for a
//! `Window`. `registry`'s two types say which is which.
//!
//! **The list links are handles all the same.** `b_next`/`b_prev`,
//! `w_next`/`w_prev` and `tp_next`, and the anchors `firstbuf`/`lastbuf`,
//! `firstwin`/`lastwin`, `tp_firstwin`/`tp_lastwin` and `first_tabpage`, hold
//! a [`BufId`]/[`WinId`]/[`TabId`] rather than an address, so a stale link
//! answers `None` instead of pointing into freed memory and the window list
//! is safe to walk even though nobody owns a `Window` yet. The one thing that
//! asks of the allocator is an order: **a window, buffer or tab page must be
//! in the registry before it is spliced into a list, and must leave the list
//! before it leaves the registry.** `buflist_new` and `aucmd_prepbuf` were
//! both the other way round and were swapped for this.
//!
//! None of that moves a [`Buf`] or a [`Win`]: `Owned::address` hands back the
//! pointer it was born with rather than borrowing the table, so the
//! registry's copy, `curbuf` and every `w_buffer` are the same pointer. That
//! is why the table holds an `Owned` rather than a `Box` — see its docs.
//!
//! # The re-entry rule
//!
//! **A `Win`, `Buf` or `TabPage` held across a call that may fire an
//! autocommand or enter Lua or Vimscript is re-derived from its handle
//! afterwards, never reused. No `&mut` reached through one is held across
//! such a call.** `:bwipeout` in a `BufLeave` handler frees the buffer a
//! caller is holding; `WinClosed` closes windows; a Lua callback can do
//! either. The value keeps pointing at memory that has gone back to the
//! allocator, and the next field access is a use-after-free.
//!
//! ```ignore
//! let id = win.id();                  // a field load; the address is dropped
//! apply_autocmds(AutoEvent::BufLeave, ...);
//! let Some(mut win) = id.get() else {
//!     return;                         // it did not survive
//! };
//! win.w_cursor.lnum = 1;              // a fresh value, freshly checked
//! ```
//!
//! **The handle carries its own identity.** [`Win::new`] reads the window's
//! handle at the one moment the caller promised the window is live and stores
//! it beside the address, so [`Win::id`] is a field load and the validity
//! predicates — `win_valid`, `win_valid_any_tab`, `win_find_tabpage`,
//! `buf_valid`, `valid_tabpage` — walk the live lists comparing copies,
//! reading nothing that may have been freed. Taking the id *ahead* of the
//! call is therefore a narrowing rather than a safety step. While the
//! identity lived only in the object it was neither: p28-6 asked `win.id()`
//! at the check, and `just asan functionaltest` answered with 169
//! heap-use-after-free reports.
//!
//! The other half of that hazard remains. A caller holding a bare
//! `*mut Window` an autocommand may have freed still may not wrap it —
//! [`Win::new`] is exactly the read a list walk exists to avoid — so
//! [`window_at`], [`buffer_at`] and [`tabpage_at`] compare the address
//! against the live lists instead. Shapes worth copying: `buffer::BufRef`
//! (upstream's `BufferRef`), a saved `Handle` plus a registry lookup
//! (`autocmd::aucmdwin`), a [`WinId`] in a struct (`terminal::mode`), and
//! [`BufId::valid`].
//!
//! **This is not `win_valid()`.** They answer different questions — see
//! [`WinId::get`]'s docs and the comment above `window::win_valid`.
//!
//! # On [`DerefMut`] and raw pointers into the same object
//!
//! **A `&mut` reached through [`DerefMut`] borrows the *whole struct*, not
//! the field.** `win.w_cursor.lnum = 1` asks for `&mut Window` and projects;
//! under Stacked and Tree Borrows that borrow pops every raw pointer
//! previously derived from the same object, so an interior `*mut Pos` taken
//! earlier is **invalidated by the next write through the handle**. Nothing
//! warns but Miri, and only if a test walks that path (p23-5). So when a body
//! holds an interior raw pointer across writes through a `Win`/`Buf`, derive
//! the address with [`Win::cursor`] or [`Live::field_ptr`], which compute it
//! from the base without forming a `&mut`, or re-derive it after each write.
//! Retyping a `*mut Window` parameter to `Win` is not by itself enough.
//!
//! Nothing here offers a scoped `with_mut` that would stretch a borrow across
//! a callback, so phase 22's ruling 6 — nothing an autocommand re-enters
//! holds a `&mut` — is a property of the API rather than of review.
//!
//! The walks — [`windows`], [`windows_in_tab`], [`tab_windows`], [`buffers`]
//! and [`frames`], plus [`tabs`] and [`frames_back`] under them — are the C's
//! `FOR_ALL_WINDOWS_IN_TAB`, `FOR_ALL_TAB_WINDOWS`, `FOR_ALL_BUFFERS` and
//! `FOR_ALL_FRAMES`. They are re-exported from the child [`walk`], which is
//! `forbid(unsafe_code)` because a step is one of the accessors below and
//! needs no promise of its own. **They are not the plain macro's timing**:
//! see that module's "when the link is read".

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `winlayer.rs` row in docs/perimeter.md.
#![allow(unsafe_code)]

mod frame;
pub mod graph;
mod handles;
mod live;
mod walk;

pub(crate) use frame::{current_topframe, free_frame, new_frame};
pub(crate) use live::{Cc, Ea, Live};

pub use handles::BufId;

pub(crate) use handles::{
    FrameId, TabId, WinId, buffer, defer_free_buffer, defer_free_window, forget_buffer,
    forget_frame, forget_tabpage, forget_window, free_deferred, register_buffer, register_frame,
    register_tabpage, register_window, tabpage, window,
};

pub(crate) use walk::{
    buffer_at, buffers, buffers_back, cmdline_window, cmdwin_window, first_buffer, first_tab,
    first_window, frames, frames_back, last_buffer, last_used_tab, last_window, prev_window,
    tab_windows, tabs, window_at, windows, windows_back, windows_in_tab,
};

use core::ffi::c_char;
use core::mem::offset_of;
use core::num::NonZero;
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::drawscreen::redraw_later;
use crate::mark::mark_mb_adjustpos;
use crate::mbyte::{utf_ptr2str_char_info, utfc_next};
use crate::memline::{ml_get_buf, ml_get_buf_len, ml_get_buf_mut};
use crate::types::{Buffer, ColNr, Frame, Handle, LineNr, Pos, StrCharInfo, Tabpage, Window};
use crate::winlayer::graph::{curtab, curwin};

// ---------------------------------------------------------------------------
// The pointers, wrapped

/// A window the caller has promised is live: **its address and its identity,
/// both taken at construction**.
///
/// [`Win::new`] is the one moment the window is promised live, so it is the
/// one moment reading it is sound — and it is where the handle is read.
/// Everything after that reads the copy: [`Win::id`] is a field load, so
/// asking "is this window still there?" — `WinId::get`, then a compare of the
/// address it answers with — touches nothing that may already have been
/// freed. While the identity was a field *of the object*, every validity
/// check read the very window it was asking about; p28-6 counted 169 heap
/// use-after-frees from that shape.
///
/// Two handles are equal when they name the same **address**: a hand-written
/// `PartialEq` rather than a derive, so that the comparison stays the C's
/// `wp == curwin`.
#[derive(Clone, Copy)]
pub struct Win {
    ptr: *mut Window,
    /// The window's handle, read while it was live. Named `id` rather than
    /// `handle` so that it cannot be confused with `Window`'s own field of
    /// that name, which [`Deref`] still reaches: `win.handle` is a read of
    /// the *window*, [`Win::handle`] a read of this copy.
    ///
    /// Zero means "no identity" — a null [`Win`], or a window whose handle
    /// the allocator has not assigned yet — which is what [`Win::id`]
    /// refuses.
    id: Handle,
}

/// A buffer the caller has promised is live. [`Win`]'s shape.
///
/// A `Buf` may be **null**: `w_buffer` is null for the moment between a
/// window losing a buffer and being given another, and an empty `tp_diffbuf`
/// slot is null too. A null one carries handle zero and may only be compared
/// or tested — see [`Win::buffer_or_none`].
#[derive(Clone, Copy)]
pub struct Buf {
    ptr: *mut Buffer,
    /// The buffer's number, read while it was live. [`Win`]'s `id`.
    id: Handle,
}

/// A frame of the window layout tree the caller has promised is live.
/// [`Win`]'s shape.
///
/// A frame is either a leaf holding one window (`fr_win`) or a row or column
/// of child frames (`fr_child`, chained through `fr_next`); `fr_parent` walks
/// back up. Which of the two a frame is, `fr_layout` says.
#[derive(Clone, Copy)]
pub struct FrameRef {
    ptr: *mut Frame,
    /// The frame's handle, read while it was live. [`Win`]'s `id`.
    id: Handle,
}

/// A tab page the caller has promised is live. [`Win`]'s shape.
#[derive(Clone, Copy)]
pub struct TabPage {
    ptr: *mut Tabpage,
    /// The tab page's handle, read while it was live. [`Win`]'s `id`.
    id: Handle,
}

// Address equality for all three, hand-written. A derive would compare the
// cached handle too, making a copy taken before a reallocation unequal to one
// taken after it at the same address — a distinction no caller asks for.
// `wp == curwin` is an address test in the C and stays one here.
macro_rules! address_eq {
    ($($ty:ty),+) => { $(
        impl PartialEq for $ty {
            #[inline(always)]
            fn eq(&self, other: &Self) -> bool {
                ptr::eq(self.ptr, other.ptr)
            }
        }

        impl Eq for $ty {}
    )+ };
}

address_eq!(Win, Buf, FrameRef, TabPage);

/// A cursor or mark position the caller has promised is live.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PosRef(*mut Pos);

/// A NUL-terminated buffer line, as `ml_get_buf` hands it back.
#[derive(Clone, Copy)]
pub struct Line(*mut c_char);

impl Deref for Win {
    type Target = Window;

    #[inline(always)]
    fn deref(&self) -> &Window {
        // SAFETY: the constructor's promise — a live window.
        unsafe { &*self.ptr }
    }
}

impl DerefMut for Win {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Window {
        // SAFETY: the constructor's promise — a live window. The borrow lasts
        // only as long as the field access that asked for it.
        unsafe { &mut *self.ptr }
    }
}

impl Deref for Buf {
    type Target = Buffer;

    #[inline(always)]
    fn deref(&self) -> &Buffer {
        // SAFETY: the constructor's promise — a live buffer.
        unsafe { &*self.ptr }
    }
}

impl DerefMut for Buf {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Buffer {
        // SAFETY: the constructor's promise — a live buffer.
        unsafe { &mut *self.ptr }
    }
}

impl Deref for FrameRef {
    type Target = Frame;

    #[inline(always)]
    fn deref(&self) -> &Frame {
        // SAFETY: the constructor's promise — a live frame.
        unsafe { &*self.ptr }
    }
}

impl DerefMut for FrameRef {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Frame {
        // SAFETY: the constructor's promise — a live frame.
        unsafe { &mut *self.ptr }
    }
}

impl Deref for TabPage {
    type Target = Tabpage;

    #[inline(always)]
    fn deref(&self) -> &Tabpage {
        // SAFETY: the constructor's promise — a live tab page.
        unsafe { &*self.ptr }
    }
}

impl DerefMut for TabPage {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Tabpage {
        // SAFETY: the constructor's promise — a live tab page.
        unsafe { &mut *self.ptr }
    }
}

impl Deref for PosRef {
    type Target = Pos;

    #[inline(always)]
    fn deref(&self) -> &Pos {
        // SAFETY: the constructor's promise — a live position.
        unsafe { &*self.0 }
    }
}

impl DerefMut for PosRef {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Pos {
        // SAFETY: the constructor's promise — a live position.
        unsafe { &mut *self.0 }
    }
}

impl Win {
    /// The window at `raw`, with its identity taken from it.
    ///
    /// **This reads the window** — its handle — which is exactly what the
    /// caller's promise makes sound, and is the only read the value will ever
    /// need. A null `raw` is tolerated and carries handle zero: the family's
    /// "or null" spellings route through here.
    ///
    /// # Safety
    /// `raw` must be null, or stay a live window for as long as the value is
    /// used.
    #[inline(always)]
    pub unsafe fn new(raw: *mut Window) -> Self {
        // SAFETY: the caller's promise, and the null test that precedes it.
        let id = if raw.is_null() {
            0
        } else {
            unsafe { (*raw).handle }
        };
        Self { ptr: raw, id }
    }

    /// The window `raw` names, `None` for null.
    ///
    /// # Safety
    /// `raw` must be null, or stay a live window for as long as the value is
    /// used.
    #[inline(always)]
    pub unsafe fn from_raw(raw: *mut Window) -> Option<Self> {
        // SAFETY: the caller's promise, narrowed by the null test.
        (!raw.is_null()).then(|| unsafe { Self::new(raw) })
    }

    /// The window at `raw` with `handle` already known — a registry entry —
    /// so **nothing is read**.
    #[inline(always)]
    pub(super) const fn at(raw: *mut Window, handle: Handle) -> Self {
        Self {
            ptr: raw,
            id: handle,
        }
    }

    /// The window the editor is working in.
    ///
    /// Safe, and checked: the editor's idea of "current" is the *identity*
    /// of a window ([`WinId`]), so this is a registry lookup rather than a
    /// raw pointer nobody promised. [`Win::current_or_none`] is the same
    /// question where the answer may be "none".
    ///
    /// `inline(always)` on all three, as the `unsafe fn` this replaced was:
    /// the draw benches read them thousands of times per redraw, and letting
    /// the ordinary `inline` heuristic decide cost ~0.5% of `scrbench`.
    ///
    /// # Panics
    ///
    /// When there is no current window — before `win_alloc_first` and after
    /// the last one goes — or when the window it names has been freed
    /// without anything being made current in its place. Both are a bug in
    /// whatever last called `Win::make_current`, which is the only writer.
    #[inline(always)]
    pub fn current() -> Self {
        Self::current_or_none().expect("no current window: nothing called Win::make_current")
    }

    /// The window the editor is working in, `None` where there is none.
    ///
    /// The `curwin != NULL` test, as a question with an answer: a caller
    /// that means to cope with having no window asks this one.
    #[inline(always)]
    pub fn current_or_none() -> Option<Self> {
        graph::CURRENT_WIN.get().and_then(WinId::get)
    }

    /// [`Win::current_or_none`] as the pointer the C compared against: the
    /// window's address, or null where there is none.
    ///
    /// The transpiled tree is full of `wp == curwin` and of callees that
    /// still take a `*mut Window` and mean "or NULL" by it. Those are the
    /// sites this exists for, and it retires with them: a signature that
    /// takes `Option<Win>` wants [`Win::current_or_none`], and a body that
    /// reads a field wants [`Win::current`].
    #[inline(always)]
    pub fn current_raw() -> *mut Window {
        Self::current_or_none().map_or(ptr::null_mut(), Self::raw)
    }

    #[inline(always)]
    pub fn raw(self) -> *mut Window {
        self.ptr
    }

    /// This window's id: the handle the API, `win_getid()` and the registry
    /// all name it by. A **field load** — the handle was read once, at
    /// construction, while the window was provably live.
    #[inline(always)]
    pub(crate) fn handle(self) -> Handle {
        self.id
    }

    /// This window's identity: the value to hold across anything that can
    /// re-enter the editor. See [`WinId`].
    ///
    /// # Panics
    ///
    /// When the handle is zero — a null [`Win`], or one built before the
    /// allocator assigned a handle. Neither names a window, so neither has
    /// an identity to hold.
    #[inline(always)]
    pub(crate) fn id(self) -> WinId {
        // A live window's handle is `last_win_id`, which is incremented
        // before it is read, so it is never zero.
        WinId(NonZero::new(self.id).expect("a live window has a handle"))
    }

    /// Give this window its handle, at the one moment it has none: the
    /// allocator's, before anything can name it.
    ///
    /// Writes the object **and** the copy, which is why it is a method:
    /// `win.handle = h` reaches `Window`'s own field through [`DerefMut`]
    /// and would leave the copy at zero, so the next `win.id()` would name
    /// nothing.
    #[inline]
    pub(crate) fn set_handle(&mut self, handle: Handle) {
        DerefMut::deref_mut(self).handle = handle;
        self.id = handle;
    }

    /// Whether this is the window the editor is working in.
    ///
    /// Safe where [`Win::current`] is not: comparing the two pointers reads
    /// neither of them.
    #[inline(always)]
    pub fn is_current(self) -> bool {
        self.ptr == curwin.get()
    }

    /// The buffer this window shows, or a null [`Buf`] for the moment between
    /// losing one and being given another — [`Win::buffer_or_none`] separates
    /// the two, and the callers that do not care only pass the address on.
    ///
    /// **Never write `Some(win.buffer())`.** An `Option<Buf>` parameter means
    /// "the buffer, or none", and wrapping this in `Some` promises a buffer
    /// that may not exist: the callee then reads through a null `Buf`. That
    /// shipped once in this slice and crashed `window::close::is_prompt`
    /// (`Test_BufUnload_close_other`). [`Win::buffer_or_none`] is the one to
    /// hand an `Option<Buf>`, and the two names differ so the grep is easy.
    #[inline(always)]
    pub fn buffer(self) -> Buf {
        Buf::at_field(self.w_buffer)
    }

    /// The leaf frame this window sits in. Every window has one, floats
    /// included — a float's frame is simply not linked into the layout tree.
    #[inline(always)]
    pub fn frame(self) -> FrameRef {
        self.frame_or_none()
            .expect("a live window has a frame: `win_alloc` gives it one")
    }

    /// The leaf frame this window sits in, `None` for a float whose frame
    /// `win_float_split` has already given back.
    #[inline(always)]
    pub fn frame_or_none(self) -> Option<FrameRef> {
        self.w_frame.and_then(FrameId::get)
    }

    /// The window's cursor, which lives inside the window.
    #[inline(always)]
    pub fn cursor(self) -> PosRef {
        // A field's address is the object's plus a constant, and computing it
        // that way needs no dereference: `wrapping_byte_add` keeps the whole
        // `Window`'s provenance, exactly as `&raw mut (*self.0).w_cursor`
        // would, without asking the window to be readable to say where its
        // cursor is.
        PosRef(
            self.ptr
                .wrapping_byte_add(offset_of!(Window, w_cursor))
                .cast(),
        )
    }

    /// The buffer this window shows, `None` for the moment between losing one
    /// and being given another.
    #[inline(always)]
    pub fn buffer_or_none(self) -> Option<Buf> {
        // A live window's `w_buffer` is a live buffer or null.
        let buf = self.w_buffer;
        (!buf.is_null()).then(|| Buf::at_field(buf))
    }

    /// The next window in this tab page's list, if any.
    #[inline(always)]
    pub fn next(self) -> Option<Self> {
        self.w_next.and_then(WinId::get)
    }

    /// The window before this one in its tab page's list, if any.
    #[inline(always)]
    pub fn prev(self) -> Option<Self> {
        self.w_prev.and_then(WinId::get)
    }

    #[inline(always)]
    pub fn redraw_later(self, redraw_type: ::core::ffi::c_int) {
        // SAFETY: a live window.
        redraw_later(self, redraw_type);
    }
}

impl Buf {
    /// The buffer at `raw`, with its number taken from it. [`Win::new`],
    /// including the null case.
    ///
    /// # Safety
    /// `raw` must be null, or stay a live buffer for as long as the value is
    /// used.
    #[inline(always)]
    pub unsafe fn new(raw: *mut Buffer) -> Self {
        // SAFETY: the caller's promise, and the null test that precedes it.
        let id = if raw.is_null() {
            0
        } else {
            unsafe { (*raw).handle }
        };
        Self { ptr: raw, id }
    }

    /// The buffer `raw` names, `None` for null.
    ///
    /// # Safety
    /// `raw` must be null, or stay a live buffer for as long as the value is
    /// used.
    #[inline(always)]
    pub unsafe fn from_raw(raw: *mut Buffer) -> Option<Self> {
        // SAFETY: the caller's promise, narrowed by the null test.
        (!raw.is_null()).then(|| unsafe { Self::new(raw) })
    }

    /// [`Win::at`] for a buffer.
    #[inline(always)]
    pub(super) const fn at(raw: *mut Buffer, handle: Handle) -> Self {
        Self {
            ptr: raw,
            id: handle,
        }
    }

    /// The buffer a window's `w_buffer` or a tab page's `tp_diffbuf` slot
    /// names, or a null [`Buf`]. The window or tab page promised it, so
    /// reading it is sound; a caller whose *own* object may already be gone
    /// holds a bare address and asks [`buffer_at`] instead.
    #[inline(always)]
    fn at_field(raw: *mut Buffer) -> Self {
        // SAFETY: a live window's `w_buffer`, or a live tab page's diff
        // slot, is a live buffer or null.
        unsafe { Self::new(raw) }
    }

    /// The buffer the editor is working in. [`Win::current`].
    ///
    /// # Panics
    ///
    /// When there is none — the few statements after `leave_curbuf`, and
    /// startup before the first buffer exists. [`Buf::current_or_none`] is
    /// the form for the callers that mean to be there.
    #[inline(always)]
    pub fn current() -> Self {
        Self::current_or_none().expect("no current buffer: see winlayer::graph::leave_curbuf")
    }

    /// The buffer the editor is working in, `None` where there is none.
    /// [`Win::current_or_none`].
    #[inline(always)]
    pub fn current_or_none() -> Option<Self> {
        graph::CURRENT_BUF.get().and_then(BufId::get)
    }

    /// The buffer's address, or null where there is none.
    /// [`Win::current_raw`].
    #[inline(always)]
    pub fn current_raw() -> *mut Buffer {
        Self::current_or_none().map_or(ptr::null_mut(), Self::raw)
    }

    #[inline(always)]
    pub fn raw(self) -> *mut Buffer {
        self.ptr
    }

    /// This buffer's number: the handle the API and `:ls` show, and what the
    /// registry finds it by. [`Win::handle`] for a buffer — a field load.
    #[inline(always)]
    pub(crate) fn handle(self) -> Handle {
        self.id
    }

    /// This buffer's identity. [`Win::id`], panic included.
    #[inline(always)]
    pub fn id(self) -> BufId {
        // A live buffer's number is `top_file_num`, which is incremented
        // before it is read, so it is never zero.
        BufId(NonZero::new(self.id).expect("a live buffer has a number"))
    }

    /// Give this buffer its number. [`Win::set_handle`].
    #[inline]
    pub(crate) fn set_handle(&mut self, handle: Handle) {
        DerefMut::deref_mut(self).handle = handle;
        self.id = handle;
    }

    #[inline(always)]
    pub fn line_count(self) -> LineNr {
        self.b_ml.ml_line_count
    }

    /// # Safety
    /// `lnum` must be a line of this buffer.
    #[inline(always)]
    pub unsafe fn line(self, lnum: LineNr) -> Line {
        Line(unsafe { ml_get_buf(self, lnum) })
    }

    /// [`Buf::line`], marking the line dirty so the caller may write to it.
    ///
    /// # Safety
    /// `lnum` must be a line of this buffer.
    #[inline(always)]
    pub unsafe fn line_mut(self, lnum: LineNr) -> Line {
        Line(unsafe { ml_get_buf_mut(self, lnum) })
    }

    /// Bytes in line `lnum`, the terminating NUL excluded.
    ///
    /// # Safety
    /// `lnum` must be a line of this buffer.
    #[inline(always)]
    pub unsafe fn line_len(self, lnum: LineNr) -> ColNr {
        unsafe { ml_get_buf_len(self, lnum) }
    }

    /// Step `pos` back off a trail byte, so it names a whole character.
    #[inline(always)]
    pub fn snap_to_char(self, pos: PosRef) {
        // SAFETY: a live buffer and a live position in it.
        unsafe { mark_mb_adjustpos(self, pos.0) };
    }

    /// The next buffer in the editor's buffer list, if any.
    #[inline(always)]
    pub fn next(self) -> Option<Self> {
        self.b_next.and_then(BufId::get)
    }

    /// The buffer before this one in the editor's buffer list, if any.
    #[inline(always)]
    pub fn prev(self) -> Option<Self> {
        self.b_prev.and_then(BufId::get)
    }
}

impl TabPage {
    /// The tab page at `raw`, with its handle taken from it. [`Win::new`].
    ///
    /// # Safety
    /// `raw` must be null, or stay a live tab page for as long as the value
    /// is used.
    #[inline(always)]
    pub unsafe fn new(raw: *mut Tabpage) -> Self {
        // SAFETY: the caller's promise, and the null test that precedes it.
        let id = if raw.is_null() {
            0
        } else {
            unsafe { (*raw).handle }
        };
        Self { ptr: raw, id }
    }

    /// [`Win::at`] for a tab page.
    #[inline(always)]
    pub(super) const fn at(raw: *mut Tabpage, handle: Handle) -> Self {
        Self {
            ptr: raw,
            id: handle,
        }
    }

    /// The tab page `raw` names, `None` for null — which is how the window
    /// family spells "the current one" throughout.
    ///
    /// # Safety
    /// `raw` must be null, or stay a live tab page for as long as the value is
    /// used.
    #[inline(always)]
    pub unsafe fn from_raw(raw: *mut Tabpage) -> Option<Self> {
        // SAFETY: the caller's promise, narrowed by the null test.
        (!raw.is_null()).then(|| unsafe { Self::new(raw) })
    }

    /// The tab page the editor is working in. [`Win::current`].
    ///
    /// # Panics
    ///
    /// When there is none, which is startup before `win_alloc_first` and
    /// nowhere else. [`TabPage::current_or_none`] for the two callers that
    /// run there.
    #[inline(always)]
    pub fn current() -> Self {
        Self::current_or_none().expect("no current tab page: nothing called TabPage::make_current")
    }

    /// The tab page the editor is working in, `None` where there is none.
    /// [`Win::current_or_none`].
    #[inline(always)]
    pub fn current_or_none() -> Option<Self> {
        graph::CURRENT_TAB.get().and_then(TabId::get)
    }

    /// The tab page's address, or null where there is none.
    /// [`Win::current_raw`].
    #[inline(always)]
    pub fn current_raw() -> *mut Tabpage {
        Self::current_or_none().map_or(ptr::null_mut(), Self::raw)
    }

    #[inline(always)]
    pub fn raw(self) -> *mut Tabpage {
        self.ptr
    }

    /// One of the up-to-eight buffers this tab page is diffing, or a null
    /// [`Buf`] for an empty slot -- [`Win::buffer`]'s shape, and the same
    /// caveat: only a caller that has already ruled the slot out may read
    /// through it.
    ///
    /// Safe for the reason [`Win::buffer`] is: the slot is read out of a tab
    /// page the handle already promised is live.
    ///
    /// # Panics
    ///
    /// When `idx` is not a diff slot.
    #[inline(always)]
    pub fn diffbuf(self, idx: usize) -> Buf {
        Buf::at_field(self.tp_diffbuf[idx])
    }

    /// This tab page's id. [`Win::handle`] for a tab page — a field load.
    #[inline(always)]
    pub(crate) fn handle(self) -> Handle {
        self.id
    }

    /// This tab page's identity, for the list links and for holding across
    /// re-entry. [`Win::id`], panic included.
    #[inline(always)]
    pub(crate) fn id(self) -> TabId {
        // A live tab page's handle is `LAST_TP_HANDLE`, which is incremented
        // before it is read, so it is never zero.
        TabId(NonZero::new(self.id).expect("a live tab page has a handle"))
    }

    /// Give this tab page its handle. [`Win::set_handle`].
    #[inline]
    pub(crate) fn set_handle(&mut self, handle: Handle) {
        DerefMut::deref_mut(self).handle = handle;
        self.id = handle;
    }

    /// Whether this is the tab page the editor is working in.
    ///
    /// Safe where [`TabPage::current`] is not: comparing the two pointers
    /// reads neither of them.
    #[inline(always)]
    pub fn is_current(self) -> bool {
        self.ptr == curtab.get()
    }

    /// This tab page as the window family takes it in an argument: `None` when
    /// it is the current one, which every such entry point reads as "no tab
    /// page given, use the current".
    #[inline(always)]
    pub fn into_other(self) -> Option<Self> {
        (!self.is_current()).then_some(self)
    }

    /// The next tab page in the editor's list, if any.
    #[inline(always)]
    pub fn next(self) -> Option<Self> {
        self.tp_next.and_then(TabId::get)
    }

    /// The root of this tab page's layout tree, `tp_topframe` verbatim.
    ///
    /// Unlike `tp_firstwin`, upstream reads this field for the current tab page
    /// too (`min_rows`, `win_vert_neighbor`), so this does not switch to the
    /// `topframe` global the way [`windows_in_tab`] switches to `firstwin`.
    #[inline(always)]
    pub fn topframe(self) -> FrameRef {
        self.tp_topframe
            .and_then(FrameId::get)
            .expect("a live tab page has a layout tree")
    }

    /// The window this tab page is working in — the one it goes back to when
    /// it is entered again, `tp_curwin` resolved. `None` once that window
    /// has been closed, which an autocommand can do between the write and
    /// the read.
    ///
    /// Stale while the tab page *is* the current one: `curwin` is the answer
    /// then, and `stash_tabpage` writes this field on the way out. Reading it
    /// of the current tab page is what upstream's `tp_curwin` reads there
    /// too, so the two agree — but a caller that wants "the window in use"
    /// wants `Win::current()`.
    #[inline(always)]
    pub(crate) fn current_window(self) -> Option<Win> {
        self.tp_curwin.and_then(WinId::get)
    }
}

impl PosRef {
    /// # Safety
    /// `pos` must stay a live position for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn new(pos: *mut Pos) -> Self {
        Self(pos)
    }

    #[inline(always)]
    pub fn raw(self) -> *mut Pos {
        self.0
    }
}

impl Line {
    /// # Safety
    /// `line` must stay a live NUL-terminated buffer line for as long as the
    /// value is used.
    #[inline(always)]
    pub const unsafe fn new(line: *mut c_char) -> Self {
        Self(line)
    }

    #[inline(always)]
    pub fn raw(self) -> *mut c_char {
        self.0
    }

    /// The byte `idx` bytes into the line.
    ///
    /// # Safety
    /// `idx` must be within the line, the terminating NUL included.
    #[inline(always)]
    pub unsafe fn byte(self, idx: ::core::ffi::c_int) -> c_char {
        unsafe { *self.0.offset(idx as isize) }
    }

    /// The first character of the line, and the walk state to step it with.
    #[inline(always)]
    pub fn first_char(self) -> StrCharInfo {
        // SAFETY: a NUL-terminated line.
        unsafe { utf_ptr2str_char_info(self.0) }
    }

    /// The character after `ci`.
    ///
    /// # Safety
    /// `ci` must be a character of this line, and not its terminating NUL.
    #[inline(always)]
    pub unsafe fn next_char(self, ci: StrCharInfo) -> StrCharInfo {
        unsafe { utfc_next(ci) }
    }

    /// Whether `ci` has reached the line's terminating NUL.
    ///
    /// # Safety
    /// `ci` must be a character of this line.
    #[inline(always)]
    pub unsafe fn ended(self, ci: StrCharInfo) -> bool {
        unsafe { *ci.ptr == 0 }
    }

    /// How many bytes into the line `ci` sits.
    #[inline(always)]
    pub fn index_of(self, ci: StrCharInfo) -> ::core::ffi::c_int {
        ci.ptr.addr().wrapping_sub(self.0.addr()) as ::core::ffi::c_int
    }
}
