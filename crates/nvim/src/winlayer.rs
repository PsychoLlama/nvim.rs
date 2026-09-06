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
//! [`FrameRef`], [`TabPage`], [`PosRef`] and [`Line`] each wrap one pointer and make
//! its **construction** the unsafe step; from there [`Deref`]/[`DerefMut`] give
//! ordinary field access and the handful of accessors below give the
//! projections a bare `&`/`&mut` cannot express — the buffer behind a window, a
//! line of that buffer, the span of a fold. Every one of them rests on the
//! single promise the constructor took, which each `pub unsafe fn` in a
//! consumer restates in its own `# Safety` section.
//!
//! Each family adds the wrappers it needs as its own `impl Win` block (an
//! inherent impl may live in any module of the defining crate), so this module
//! stays the shared minimum rather than growing a method per caller.
//!
//! The three **handle registries** live here too — see "Finding one by
//! handle" below. They are the one place a `Win`/`Buf`/`TabPage` is built
//! from a handle rather than from a pointer a caller already had, and
//! because the registry's own invariant is that everything in it is live,
//! [`window`], [`buffer`] and [`tabpage`] are **safe** functions.
//!
//! Its child [`handles`] holds the three registries and the deferred-free
//! set.
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
//! `Window` until that idle window has a named owner. `registry`'s two types
//! say which is which.
//!
//! **The five list links are handles all the same.** `b_next`/`b_prev`,
//! `w_next`/`w_prev` and `tp_next` — and the six anchors `firstbuf`,
//! `lastbuf`, `firstwin`, `lastwin`, `tp_firstwin`/`tp_lastwin` and
//! `first_tabpage` — hold a [`BufId`]/[`WinId`]/[`TabId`], not an address,
//! so the graph's *lists* are index-shaped whether or not the allocation
//! has moved. Ownership is what a `Vec` inside the object needs; a handle
//! link is what makes a stale link answer `None` instead of pointing into
//! freed memory, and it is why the window list is safe to walk even though
//! nobody owns a `Window` yet. The one thing it asks of the allocator is an
//! order: **a window, buffer or tab page must be in the registry before it
//! is spliced into a list, and must leave the list before it leaves the
//! registry.** `buflist_new` and `aucmd_prepbuf` were both the other way
//! round and were swapped for this.
//!
//! None of that changes what a [`Buf`] or a [`Win`] *is*. Both are still one
//! address and building one still reads nothing: `Owned::address` hands back
//! the pointer it was born with rather than borrowing the table, so the
//! registry's copy, `curbuf` and every `w_buffer` are the same pointer and
//! all of them stay usable. That is the whole reason the table holds an
//! `Owned` rather than a `Box` — see its docs.
//!
//! # The re-entry rule
//!
//! **A `Win`, `Buf` or `TabPage` held across a call that may fire an
//! autocommand or enter Lua or Vimscript is re-derived from its handle
//! afterwards, never reused. No `&mut` reached through one is held across
//! such a call.**
//!
//! Everything below rests on the promise the constructor took — *this object
//! stays live for as long as the value is used* — and an autocommand is
//! exactly what breaks it. `:bwipeout` in a `BufLeave` handler frees the
//! buffer a caller is holding; `WinClosed` closes windows; a Lua callback can
//! do either. The value keeps pointing at memory that has gone back to the
//! allocator, and the next field access is a use-after-free.
//!
//! So the shape of every such caller is: **take the identity before, ask the
//! registry after.** [`WinId`] and [`BufId`] are that identity — a `Handle`
//! with the address dropped. (A `TabPageId` lands with its first caller;
//! `dead_code` is `-D` here.)
//!
//! ```ignore
//! let id = win.id();                  // while the window is provably live
//! apply_autocmds(AutoEvent::BufLeave, ...);
//! let Some(mut win) = id.get() else {
//!     return;                         // it did not survive
//! };
//! win.w_cursor.lnum = 1;              // a fresh value, freshly checked
//! ```
//!
//! [`Win::id`] *reads the window*, which is why it must be taken before the
//! call and not after — and why the identity is a separate value rather than
//! a second field of [`Win`]. **Building a `Win` reads nothing**, and the
//! editor depends on that: `win_valid` and `win_find_tabpage` are handed
//! addresses an autocommand may already have freed, and only compare them.
//! Reading a handle out of one to answer "is it still there?" is the very
//! dereference those functions exist to avoid, so such a caller keeps the raw
//! pointer and keeps the list walk.
//!
//! Four shapes of the rule are in the tree and worth copying:
//!
//! * `BufRef` (`buffer::BufRef`, upstream's `BufferRef`) — `BufRef::of`/`of_opt`
//!   before, `BufRef::valid`/`get` after. `buffer::enter` uses it twice around
//!   `BufLeave`.
//! * A saved `Handle` plus a registry lookup — `autocmd::aucmdwin`'s
//!   `save_curwin_handle`/`save_prevwin_handle`.
//! * [`WinId`] held in a struct that outlives arbitrary re-entry —
//!   `terminal::mode`'s `save_curwin`, restored with `.get()`.
//! * [`BufId::valid`] — the same pair as a question; `buffer::enter` asks it
//!   about a buffer it held across `BufLeave`.
//!
//! **This is not `win_valid()`.** They answer different questions and are
//! not interchangeable — see [`WinId::get`]'s own docs and the comment above
//! `window::win_valid`.
//!
//! On `&mut`: [`DerefMut`] hands out a borrow that lasts exactly as long as
//! the field access asking for it, and nothing here offers a scoped
//! `with_mut` that would stretch one across a callback. Phase 22's ruling 6
//! — nothing an autocommand or Lua callback re-enters holds a `&mut` — is
//! therefore a property of the API rather than of review.
//!
//! # On [`DerefMut`] and raw pointers into the same object
//!
//! **A `&mut` reached through [`DerefMut`] borrows the *whole struct*, not
//! the field.** `win.w_cursor.lnum = 1` asks for `&mut Window` and projects;
//! under Stacked and Tree Borrows that borrow pops every raw pointer
//! previously derived from the same object off the tag stack, so a
//! `*mut Pos` taken earlier from `&raw mut (*wp).w_cursor` — or any other
//! interior pointer the transpiled code is still carrying — is **invalidated
//! by the next write through the handle**, and using it afterwards is UB.
//!
//! Nothing warns. `cargo check`, clippy and a release build are all silent;
//! only Miri sees it, and only if a test happens to walk that path. p23-5
//! found the same edge from the other side, and it is why the sweep that
//! retired the raw `curwin`/`curbuf` reads left `&raw mut (*cur_win().raw())
//! .field` alone rather than writing `&raw mut cur_win().field`: the address
//! would take its provenance from a transient `&mut Window`.
//!
//! So when a body holds an interior raw pointer across writes through a
//! `Win`/`Buf`, one of the two has to go: derive the address with
//! [`Win::cursor`] or [`Live::field_ptr`], which compute it from the base
//! *without* forming a `&mut` and so read nothing, or re-derive the interior
//! pointer after each write. Converting a `*mut Window` parameter to `Win` is
//! not by itself enough — check what else in the body still points inside.
//!
//! The walks — [`windows`], [`windows_in_tab`], [`tab_windows`], [`buffers`]
//! and [`frames`], plus [`tabs`] and [`frames_back`] under them — are the C's
//! `FOR_ALL_WINDOWS_IN_TAB`, `FOR_ALL_TAB_WINDOWS`, `FOR_ALL_BUFFERS` and
//! `FOR_ALL_FRAMES`. They are re-exported from the child [`walk`], which is
//! `forbid(unsafe_code)` because a step is one of the accessors below and
//! needs no promise of its own. **They are not the plain macro's timing**:
//! see that module's "when the link is read".

#![deny(unsafe_op_in_unsafe_fn)]

pub mod graph;
mod handles;
mod live;
mod walk;

pub(crate) use live::{Cc, Ea, Live};

pub(crate) use handles::{
    BufId, TabId, WinId, buffer, defer_free_buffer, defer_free_window, forget_buffer,
    forget_tabpage, forget_window, free_deferred, register_buffer, register_tabpage,
    register_window, tabpage, window,
};

pub(crate) use walk::{
    buffers, buffers_back, first_buffer, first_tab, first_window, frames, frames_back, last_buffer,
    last_window, tab_windows, tabs, windows, windows_back, windows_in_tab,
};

use core::ffi::c_char;
use core::mem::offset_of;
use core::num::NonZero;
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::drawscreen::redraw_later;
use crate::fold::{has_any_folding, has_folding};
use crate::mark::mark_mb_adjustpos;
use crate::mbyte::{utf_ptr2str_char_info, utfc_next};
use crate::memline::{ml_get_buf, ml_get_buf_len, ml_get_buf_mut};
use crate::plines::{getvcol, getvvcol};
use crate::types::{Buffer, ColNr, Frame, Handle, LineNr, Pos, StrCharInfo, Tabpage, Window};
use crate::winlayer::graph::{curtab, curwin};

// ---------------------------------------------------------------------------
// The pointers, wrapped

/// A window the caller has promised is live.
///
/// One pointer, and **building one reads nothing**: the editor passes these
/// addresses around after an autocommand may already have freed them and only
/// compares them (`win_valid`, `win_find_tabpage`). Identity that outlives the
/// address is [`WinId`], taken while the window is live.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Win(*mut Window);

/// A buffer the caller has promised is live. [`Win`]'s shape.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Buf(*mut Buffer);

/// A frame of the window layout tree the caller has promised is live.
///
/// A frame is either a leaf holding one window (`fr_win`) or a row or column
/// of child frames (`fr_child`, chained through `fr_next`); `fr_parent` walks
/// back up. Which of the two a frame is, `fr_layout` says.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FrameRef(*mut Frame);

/// A tab page the caller has promised is live. [`Win`]'s shape.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TabPage(*mut Tabpage);

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
        unsafe { &*self.0 }
    }
}

impl DerefMut for Win {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Window {
        // SAFETY: the constructor's promise — a live window. The borrow lasts
        // only as long as the field access that asked for it.
        unsafe { &mut *self.0 }
    }
}

impl Deref for Buf {
    type Target = Buffer;

    #[inline(always)]
    fn deref(&self) -> &Buffer {
        // SAFETY: the constructor's promise — a live buffer.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Buf {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Buffer {
        // SAFETY: the constructor's promise — a live buffer.
        unsafe { &mut *self.0 }
    }
}

impl Deref for FrameRef {
    type Target = Frame;

    #[inline(always)]
    fn deref(&self) -> &Frame {
        // SAFETY: the constructor's promise — a live frame.
        unsafe { &*self.0 }
    }
}

impl DerefMut for FrameRef {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Frame {
        // SAFETY: the constructor's promise — a live frame.
        unsafe { &mut *self.0 }
    }
}

impl Deref for TabPage {
    type Target = Tabpage;

    #[inline(always)]
    fn deref(&self) -> &Tabpage {
        // SAFETY: the constructor's promise — a live tab page.
        unsafe { &*self.0 }
    }
}

impl DerefMut for TabPage {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Tabpage {
        // SAFETY: the constructor's promise — a live tab page.
        unsafe { &mut *self.0 }
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
    /// # Safety
    /// `raw` must stay a live window for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn new(raw: *mut Window) -> Self {
        Self(raw)
    }

    /// The window `raw` names, `None` for null.
    ///
    /// # Safety
    /// `raw` must be null, or stay a live window for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn from_raw(raw: *mut Window) -> Option<Self> {
        if raw.is_null() { None } else { Some(Self(raw)) }
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
        self.0
    }

    /// This window's id: the handle the API, `win_getid()` and the registry
    /// all name it by. **Reads the window**, so ask it while the window is
    /// live — which is what the re-entry rule asks anyway: before the call
    /// that might close it. [`Win::id`] wraps the answer in a type.
    #[inline(always)]
    pub(crate) fn handle(self) -> Handle {
        self.handle
    }

    /// This window's identity, taken while it is live: the value to hold
    /// across anything that can re-enter the editor. See [`WinId`].
    #[inline(always)]
    pub(crate) fn id(self) -> WinId {
        // A live window's handle is `last_win_id`, which is incremented
        // before it is read, so it is never zero.
        WinId(NonZero::new(self.handle).expect("a live window has a handle"))
    }

    /// Whether this is the window the editor is working in.
    ///
    /// Safe where [`Win::current`] is not: comparing the two pointers reads
    /// neither of them.
    #[inline(always)]
    pub fn is_current(self) -> bool {
        self.0 == curwin.get()
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
        Buf(self.w_buffer)
    }

    /// The leaf frame this window sits in. Every window has one, floats
    /// included — a float's frame is simply not linked into the layout tree.
    #[inline(always)]
    pub fn frame(self) -> FrameRef {
        // A live window's `w_frame` is a live frame.
        FrameRef(self.w_frame)
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
            self.0
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
        (!buf.is_null()).then_some(Buf(buf))
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

    /// First line of the fold containing `lnum`, if there is one.
    #[inline(always)]
    pub fn fold_first(self, lnum: LineNr) -> Option<LineNr> {
        let mut first = lnum;
        // `firstp` is written only when the answer is true, so the seed
        // survives a line that is in no fold.
        let folded = has_folding(self, lnum, Some(&mut first), None);
        folded.then_some(first)
    }

    /// Last line of the fold containing `lnum`, or `lnum` when it is in none.
    #[inline(always)]
    pub fn fold_last(self, lnum: LineNr) -> LineNr {
        let mut last = lnum;
        // `lastp` is written only when folded.
        has_folding(self, lnum, None, Some(&mut last));
        last
    }

    /// Last line of the fold containing `lnum`, `None` when it is in none --
    /// [`Win::fold_first`]'s partner at the other end.
    #[inline(always)]
    pub fn fold_end(self, lnum: LineNr) -> Option<LineNr> {
        let (folded, _, last) = self.fold_span(lnum);
        folded.then_some(last)
    }

    /// The whole fold containing `lnum`: whether there is one, and its first
    /// and last line (both `lnum` when there is not).
    #[inline(always)]
    pub fn fold_span(self, lnum: LineNr) -> (bool, LineNr, LineNr) {
        let (mut first, mut last) = (lnum, lnum);
        // Both out-params are written only when folded.
        let folded = has_folding(self, lnum, Some(&mut first), Some(&mut last));
        (folded, first, last)
    }

    #[inline(always)]
    pub fn has_any_folding(self) -> bool {
        has_any_folding(self) != 0
    }

    /// First and last virtual column of the character at `pos`.
    #[inline(always)]
    pub fn vcol_span(self, pos: PosRef) -> (ColNr, ColNr) {
        let (mut start, mut end) = (0, 0);
        // SAFETY: a live window and a live position in its buffer.
        unsafe { getvcol(self, pos.0, &raw mut start, ptr::null_mut(), &raw mut end) };
        (start, end)
    }

    /// Start, cursor and end virtual column of the character at `pos`.
    #[inline(always)]
    pub fn vcol_triple(self, pos: PosRef) -> (ColNr, ColNr, ColNr) {
        let (mut start, mut cursor, mut end) = (0, 0, 0);
        // SAFETY: a live window and a live position in its buffer.
        unsafe { getvcol(self, pos.0, &raw mut start, &raw mut cursor, &raw mut end) };
        (start, cursor, end)
    }

    /// The first virtual column of the character at `pos`.
    #[inline(always)]
    pub fn vcol(self, pos: PosRef) -> ColNr {
        self.vcol_span(pos).0
    }

    /// [`Win::vcol_span`] with 'virtualedit' taken into account.
    #[inline(always)]
    pub fn virtual_vcol_span(self, pos: PosRef) -> (ColNr, ColNr) {
        let (mut start, mut end) = (0, 0);
        // SAFETY: a live window and a live position in its buffer.
        unsafe { getvvcol(self, pos.0, &raw mut start, ptr::null_mut(), &raw mut end) };
        (start, end)
    }

    /// [`Win::vcol_triple`] with 'virtualedit' taken into account.
    #[inline(always)]
    pub fn virtual_vcol_triple(self, pos: PosRef) -> (ColNr, ColNr, ColNr) {
        let (mut start, mut cursor, mut end) = (0, 0, 0);
        // SAFETY: a live window and a live position in its buffer.
        unsafe { getvvcol(self, pos.0, &raw mut start, &raw mut cursor, &raw mut end) };
        (start, cursor, end)
    }

    /// The first virtual column of the character at `pos`, 'virtualedit'
    /// included.
    #[inline(always)]
    pub fn virtual_vcol(self, pos: PosRef) -> ColNr {
        self.virtual_vcol_span(pos).0
    }

    /// The virtual column the *cursor* shows at within the character at
    /// `pos`, which is not its first column when the character is a tab.
    #[inline(always)]
    pub fn virtual_cursor_vcol(self, pos: PosRef) -> ColNr {
        let mut cursor = 0;
        let (none, c) = (ptr::null_mut(), &raw mut cursor);
        // SAFETY: a live window and a live position in its buffer.
        unsafe { getvvcol(self, pos.0, none, c, none) };
        cursor
    }

    #[inline(always)]
    pub fn redraw_later(self, redraw_type: ::core::ffi::c_int) {
        // SAFETY: a live window.
        redraw_later(unsafe { Win::new(self.0) }, redraw_type);
    }
}

impl Buf {
    /// # Safety
    /// `raw` must stay a live buffer for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn new(raw: *mut Buffer) -> Self {
        Self(raw)
    }

    /// The buffer `raw` names, `None` for null.
    ///
    /// # Safety
    /// `raw` must be null, or stay a live buffer for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn from_raw(raw: *mut Buffer) -> Option<Self> {
        if raw.is_null() { None } else { Some(Self(raw)) }
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
        self.0
    }

    /// This buffer's number: the handle the API and `:ls` show, and what the
    /// registry finds it by. [`Win::handle`] for a buffer — it reads the
    /// buffer, so ask it while the buffer is live.
    #[inline(always)]
    pub(crate) fn handle(self) -> Handle {
        self.handle
    }

    /// This buffer's identity, taken while it is live. [`Win::id`].
    #[inline(always)]
    pub(crate) fn id(self) -> BufId {
        // A live buffer's number is `top_file_num`, which is incremented
        // before it is read, so it is never zero.
        BufId(NonZero::new(self.handle).expect("a live buffer has a number"))
    }

    #[inline(always)]
    pub fn line_count(self) -> LineNr {
        self.b_ml.ml_line_count
    }

    /// # Safety
    /// `lnum` must be a line of this buffer.
    #[inline(always)]
    pub unsafe fn line(self, lnum: LineNr) -> Line {
        Line(unsafe { ml_get_buf(Buf::new(self.0), lnum) })
    }

    /// [`Buf::line`], marking the line dirty so the caller may write to it.
    ///
    /// # Safety
    /// `lnum` must be a line of this buffer.
    #[inline(always)]
    pub unsafe fn line_mut(self, lnum: LineNr) -> Line {
        Line(unsafe { ml_get_buf_mut(Buf::new(self.0), lnum) })
    }

    /// Bytes in line `lnum`, the terminating NUL excluded.
    ///
    /// # Safety
    /// `lnum` must be a line of this buffer.
    #[inline(always)]
    pub unsafe fn line_len(self, lnum: LineNr) -> ColNr {
        unsafe { ml_get_buf_len(Buf::new(self.0), lnum) }
    }

    /// Step `pos` back off a trail byte, so it names a whole character.
    #[inline(always)]
    pub fn snap_to_char(self, pos: PosRef) {
        // SAFETY: a live buffer and a live position in it.
        unsafe { mark_mb_adjustpos(Buf::new(self.0), pos.0) };
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

impl FrameRef {
    /// # Safety
    /// `raw` must stay a live frame for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn new(raw: *mut Frame) -> Self {
        Self(raw)
    }

    /// The frame `raw` names, `None` for null.
    ///
    /// # Safety
    /// `raw` must be null, or stay a live frame for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn from_raw(raw: *mut Frame) -> Option<Self> {
        if raw.is_null() { None } else { Some(Self(raw)) }
    }

    #[inline(always)]
    pub fn raw(self) -> *mut Frame {
        self.0
    }

    /// The window this frame holds — `Some` exactly for a leaf.
    #[inline(always)]
    pub fn win(self) -> Option<Win> {
        // A live leaf frame's `fr_win` is a live window; a row or column's is
        // null.
        let win = self.fr_win;
        (!win.is_null()).then_some(Win(win))
    }

    /// The frame this one is a child of — `None` only for the tab page's
    /// `topframe`, the one frame with no parent.
    #[inline(always)]
    pub fn parent(self) -> Option<Self> {
        // A live frame's `fr_parent` is a live frame or null.
        let parent = self.fr_parent;
        (!parent.is_null()).then_some(Self(parent))
    }

    /// This frame's first child, which every non-leaf frame has.
    #[inline(always)]
    pub fn child(self) -> Option<Self> {
        // A live frame's `fr_child` is a live frame or null.
        let child = self.fr_child;
        (!child.is_null()).then_some(Self(child))
    }

    /// The frame beside this one, if it is not the last of its row or column.
    #[inline(always)]
    pub fn next(self) -> Option<Self> {
        // A live frame's `fr_next` is a live frame or null.
        let next = self.fr_next;
        (!next.is_null()).then_some(Self(next))
    }

    /// The frame before this one, if it is not the first of its row or column.
    #[inline(always)]
    pub fn prev(self) -> Option<Self> {
        // A live frame's `fr_prev` is a live frame or null.
        let prev = self.fr_prev;
        (!prev.is_null()).then_some(Self(prev))
    }

    /// This frame's children, first to last: the C's
    /// `FOR_ALL_FRAMES(frp, topfrp->fr_child)`, which is empty for a leaf.
    #[inline(always)]
    pub fn children(self) -> impl Iterator<Item = Self> {
        frames(self.child())
    }
}

impl TabPage {
    /// # Safety
    /// `raw` must stay a live tab page for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn new(raw: *mut Tabpage) -> Self {
        Self(raw)
    }

    /// The tab page `raw` names, `None` for null — which is how the window
    /// family spells "the current one" throughout.
    ///
    /// # Safety
    /// `raw` must be null, or stay a live tab page for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn from_raw(raw: *mut Tabpage) -> Option<Self> {
        if raw.is_null() { None } else { Some(Self(raw)) }
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
        self.0
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
        Buf(self.tp_diffbuf[idx])
    }

    /// This tab page's id. [`Win::handle`] for a tab page.
    #[inline(always)]
    pub(crate) fn handle(self) -> Handle {
        self.handle
    }

    /// This tab page's identity, for the list links and for holding across
    /// re-entry. [`Win::id`].
    #[inline(always)]
    pub(crate) fn id(self) -> TabId {
        // A live tab page's handle is `LAST_TP_HANDLE`, which is incremented
        // before it is read, so it is never zero.
        TabId(NonZero::new(self.handle).expect("a live tab page has a handle"))
    }

    /// Whether this is the tab page the editor is working in.
    ///
    /// Safe where [`TabPage::current`] is not: comparing the two pointers
    /// reads neither of them.
    #[inline(always)]
    pub fn is_current(self) -> bool {
        self.0 == curtab.get()
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
        // A live tab page's top frame is live.
        FrameRef(self.tp_topframe)
    }

    /// The window this tab page is working in — the one it goes back to when
    /// it is entered again, `tp_curwin` verbatim.
    ///
    /// Stale while the tab page *is* the current one: `curwin` is the answer
    /// then, and `stash_tabpage` writes this field on the way out. Reading it
    /// of the current tab page is what upstream's `tp_curwin` reads there
    /// too, so the two agree — but a caller that wants "the window in use"
    /// wants `Win::current()`.
    #[inline(always)]
    pub(crate) fn current_window(self) -> Win {
        // A live tab page's `tp_curwin` is a live window.
        Win(self.tp_curwin)
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
