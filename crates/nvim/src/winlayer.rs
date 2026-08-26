//! The window, buffer and position pointers the editor works through, wrapped
//! so that dereferencing one is not an unsafe operation at every use.
//!
//! The transpiled editor passes `*mut win_T` / `*mut buf_T` / `*mut pos_T`
//! everywhere, and the pointers have to stay raw: callers interleave these
//! calls with reads of the `curwin`/`curbuf` globals — which alias the same
//! objects — and many of them re-enter through autocommands, so a long-lived
//! `&mut` would invalidate a pointer the caller still holds.
//!
//! What does not have to stay raw is the *dereference*. [`Win`], [`Buf`],
//! [`Frame`], [`TabPage`], [`Pos`] and [`Line`] each wrap one pointer and make
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
//! So the shape of every such caller is: **take the handle before, ask the
//! registry after.**
//!
//! ```ignore
//! let handle = win.handle();          // while the window is provably live
//! apply_autocmds(EVENT_BUFLEAVE, ...);
//! let Some(mut win) = winlayer::window(handle) else {
//!     return;                         // it did not survive
//! };
//! win.w_cursor.lnum = 1;              // a fresh value, freshly checked
//! ```
//!
//! [`Win::handle`] is deliberately cheap and reads nothing: the handle is
//! carried in the value, put there when it was derived. That is what makes
//! [`Win::valid`] — "is what I was holding still registered?" — answerable
//! *without* touching the object, and it is why `handle()` still answers
//! correctly for a window that has since been closed. Nothing else on these
//! types may be called on a value whose object may be gone.
//!
//! Three shapes of this are already in the tree and are worth copying:
//!
//! * `BufRef` (`buffer::BufRef`, upstream's `bufref_T`) — `set_bufref` before,
//!   `bufref_valid`/`BufRef::get` after. `buffer::enter` uses it twice around
//!   `BufLeave`.
//! * A saved `handle_T` plus a registry lookup — `terminal::mode`'s
//!   `save_curwin_handle` and `win_for_handle`, `autocmd::aucmdwin`'s
//!   `save_curwin_handle`/`save_prevwin_handle`.
//! * [`Buf::valid`] on a value derived while the buffer was live, which is
//!   the same question with the handle already in hand.
//!
//! **`valid()` is not `win_valid()`.** They answer different questions and
//! are not interchangeable — see [`Win::valid`]'s own docs and the comment
//! above `window::win_valid`. A caller holding a bare `*mut win_T` that an
//! autocommand may have freed *cannot* use `valid()` at all: deriving a
//! `Win` to ask reads the window's handle, which is the very dereference the
//! question exists to avoid. Such a caller keeps the pointer raw and keeps
//! the list walk.
//!
//! On `&mut`: [`DerefMut`] hands out a borrow that lasts exactly as long as
//! the field access asking for it, and nothing here offers a scoped
//! `with_mut` that would stretch one across a callback. Phase 22's ruling 6
//! — nothing an autocommand or Lua callback re-enters holds a `&mut` — is
//! therefore a property of the API rather than of review.
//!
//! The walks at the bottom — [`windows`], [`windows_in_tab`], [`tab_windows`],
//! [`buffers`] and [`frames`], plus [`tabs`] and [`frames_back`] under them —
//! are the C's `FOR_ALL_WINDOWS_IN_TAB`, `FOR_ALL_TAB_WINDOWS`,
//! `FOR_ALL_BUFFERS` and `FOR_ALL_FRAMES`. The lists they walk are the
//! editor's own and live from startup to exit, so the walks are safe
//! functions; each is lazy, as the macro it replaces is.

#![deny(unsafe_op_in_unsafe_fn)]

mod handles;

pub(crate) use handles::{
    buffer, defer_free_buffer, defer_free_window, forget_buffer, forget_tabpage, forget_window,
    free_deferred, register_buffer, register_tabpage, register_window, tabpage, window,
};

use core::ffi::c_char;
use core::mem::offset_of;
use core::ops::{Deref, DerefMut};
use core::{iter, ptr};

use crate::drawscreen::redraw_later;
use crate::fold::{has_any_folding, has_folding};
use crate::main::{curbuf, curtab, curwin, first_tabpage, firstbuf, firstwin};
use crate::mark::mark_mb_adjustpos;
use crate::mbyte::{utf_ptr2str_char_info, utfc_next};
use crate::memline::{ml_get_buf, ml_get_buf_len, ml_get_buf_mut};
use crate::plines::{getvcol, getvvcol};
use crate::types::{
    StrCharInfo, buf_T, colnr_T, frame_T, handle_T, linenr_T, pos_T, tabpage_T, win_T,
};

// ---------------------------------------------------------------------------
// The pointers, wrapped

/// A window the caller has promised is live.
///
/// Two words: the address, and the `handle_T` that named the window when the
/// value was derived. See "Identity, and validity after re-entry" below for
/// why the handle rides along instead of being read back out of the window.
#[derive(Clone, Copy, Eq)]
pub struct Win(*mut win_T, handle_T);

/// A buffer the caller has promised is live. [`Win`]'s shape: address and
/// buffer number.
#[derive(Clone, Copy, Eq)]
pub struct Buf(*mut buf_T, handle_T);

/// A frame of the window layout tree the caller has promised is live.
///
/// A frame is either a leaf holding one window (`fr_win`) or a row or column
/// of child frames (`fr_child`, chained through `fr_next`); `fr_parent` walks
/// back up. Which of the two a frame is, `fr_layout` says.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Frame(*mut frame_T);

/// A tab page the caller has promised is live. [`Win`]'s shape.
#[derive(Clone, Copy, Eq)]
pub struct TabPage(*mut tabpage_T, handle_T);

/// A cursor or mark position the caller has promised is live.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pos(*mut pos_T);

/// A NUL-terminated buffer line, as `ml_get_buf` hands it back.
#[derive(Clone, Copy)]
pub struct Line(*mut c_char);

/// Two values are the same window when they hold the same address, which is
/// what the comparison meant while this was a bare pointer. The handle is
/// derived from the address and cannot disagree with it — handles are handed
/// out by a monotone counter and never reused — so comparing it as well would
/// change no answer, and leaving it out keeps the two representations
/// interchangeable while phase 23 moves between them. As [`Buf`], [`TabPage`].
impl PartialEq for Win {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq for Buf {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq for TabPage {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Deref for Win {
    type Target = win_T;

    #[inline(always)]
    fn deref(&self) -> &win_T {
        // SAFETY: the constructor's promise — a live window.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Win {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut win_T {
        // SAFETY: the constructor's promise — a live window. The borrow lasts
        // only as long as the field access that asked for it.
        unsafe { &mut *self.0 }
    }
}

impl Deref for Buf {
    type Target = buf_T;

    #[inline(always)]
    fn deref(&self) -> &buf_T {
        // SAFETY: the constructor's promise — a live buffer.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Buf {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut buf_T {
        // SAFETY: the constructor's promise — a live buffer.
        unsafe { &mut *self.0 }
    }
}

impl Deref for Frame {
    type Target = frame_T;

    #[inline(always)]
    fn deref(&self) -> &frame_T {
        // SAFETY: the constructor's promise — a live frame.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Frame {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut frame_T {
        // SAFETY: the constructor's promise — a live frame.
        unsafe { &mut *self.0 }
    }
}

impl Deref for TabPage {
    type Target = tabpage_T;

    #[inline(always)]
    fn deref(&self) -> &tabpage_T {
        // SAFETY: the constructor's promise — a live tab page.
        unsafe { &*self.0 }
    }
}

impl DerefMut for TabPage {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut tabpage_T {
        // SAFETY: the constructor's promise — a live tab page.
        unsafe { &mut *self.0 }
    }
}

impl Deref for Pos {
    type Target = pos_T;

    #[inline(always)]
    fn deref(&self) -> &pos_T {
        // SAFETY: the constructor's promise — a live position.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Pos {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut pos_T {
        // SAFETY: the constructor's promise — a live position.
        unsafe { &mut *self.0 }
    }
}

impl Win {
    /// # Safety
    /// `wp` must stay a live window for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn new(wp: *mut win_T) -> Self {
        Self::wrap(wp)
    }

    /// Wrap a window this module has just established is live, reading the
    /// handle that becomes the value's identity.
    ///
    /// Private, and the module's only way in: every call site is on the same
    /// footing as a caller of [`Win::new`] and says so where it is not
    /// obvious.
    ///
    /// A **null** is carried with handle `0`, which names no window, so
    /// [`Win::valid`] answers `false` for it and nothing is read. That is not
    /// a nicety: the editor really does hand these around — `curbuf` is null
    /// for the moment `free_buffer` clears it, and a window being closed has
    /// a null `w_buffer` — and while this was a bare pointer such a value
    /// was made and passed on freely. Wrapping one must therefore stay as
    /// harmless as it was.
    #[inline(always)]
    const fn wrap(wp: *mut win_T) -> Self {
        match wp.is_null() {
            true => Self(wp, 0),
            // SAFETY: non-null here is a live window, per the contract above.
            false => Self(wp, unsafe { (*wp).handle }),
        }
    }

    /// The window `wp` names, `None` for null.
    ///
    /// # Safety
    /// `wp` must be null, or stay a live window for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn from_raw(wp: *mut win_T) -> Option<Self> {
        if wp.is_null() {
            None
        } else {
            Some(Self::wrap(wp))
        }
    }

    /// The window the editor is working in.
    ///
    /// # Safety
    /// `curwin` must be set, which it is from startup to exit.
    #[inline(always)]
    pub unsafe fn current() -> Self {
        Self::wrap(curwin.get())
    }

    #[inline(always)]
    pub fn raw(self) -> *mut win_T {
        self.0
    }

    /// This window's id: the handle the API, `win_getid()` and the registry
    /// all name it by. Identity that survives the address being reused.
    ///
    /// Read when the value was derived, not now, so this answers correctly
    /// even for a window an autocommand has since closed — which is what
    /// makes [`Win::valid`] able to ask about one.
    #[inline(always)]
    pub fn handle(self) -> handle_T {
        self.1
    }

    /// Give a freshly allocated window the handle that names it from here on.
    ///
    /// The one place `w_handle` is written, and the only thing that keeps the
    /// two halves of this value in step: the allocator wraps the raw memory
    /// before it has a handle to read, so it must hand the handle back
    /// through here rather than storing it through [`DerefMut`]. Copies of
    /// `self` taken before this call keep the old identity, which is why the
    /// three allocators pass their value on by `&mut`.
    #[inline(always)]
    pub(crate) fn assign_handle(&mut self, handle: handle_T) {
        self.handle = handle;
        self.1 = handle;
    }

    /// Whether the window this value was derived from is **still
    /// registered**: allocated, and not yet freed.
    ///
    /// This is *not* `win_valid()`, and the two answer different questions:
    ///
    /// * `win_valid(wp)` asks whether an **address** is on the **current tab
    ///   page's** window list (`win_valid_any_tab` widens that to every tab
    ///   page). It is a list walk, and it says "no" for a hidden window
    ///   (`win_alloc(_, true)`) that is on no list but perfectly alive.
    /// * This asks whether the **object** this value names still exists,
    ///   by the handle read when the value was made. It says "yes" for that
    ///   hidden window, and "no" for the autocommand window while it is idle,
    ///   which `aucmd_restbuf` takes out of the registry.
    ///
    /// Ask this one when the question is "did what I was holding survive the
    /// call I just made"; ask `win_valid` when the question is about layout —
    /// "is this window on screen, on this tab page". Reaching for the wrong
    /// one is a behaviour change, not a style choice.
    ///
    /// No memory belonging to the window is read, so this is safe to ask of a
    /// value whose window may already be gone — the one thing the rest of the
    /// type is not.
    #[inline(always)]
    pub fn valid(self) -> bool {
        window(self.1) == Some(self)
    }

    /// Whether this is the window the editor is working in.
    ///
    /// Safe where [`Win::current`] is not: comparing the two pointers reads
    /// neither of them.
    #[inline(always)]
    pub fn is_current(self) -> bool {
        self.0 == curwin.get()
    }

    /// The buffer this window shows.
    ///
    /// A live window's `w_buffer` is a live buffer — except for the moment
    /// between losing one and being given another, when it is null and this
    /// answers a null [`Buf`] rather than reading it. Callers that care use
    /// [`Win::buffer_or_none`]; the rest only pass the address on, as they
    /// did while these were bare pointers.
    #[inline(always)]
    pub fn buffer(self) -> Buf {
        Buf::wrap(self.w_buffer)
    }

    /// The leaf frame this window sits in. Every window has one, floats
    /// included — a float's frame is simply not linked into the layout tree.
    #[inline(always)]
    pub fn frame(self) -> Frame {
        // A live window's `w_frame` is a live frame.
        Frame(self.w_frame)
    }

    /// The window's cursor, which lives inside the window.
    #[inline(always)]
    pub fn cursor(self) -> Pos {
        // A field's address is the object's plus a constant, and computing it
        // that way needs no dereference: `wrapping_byte_add` keeps the whole
        // `win_T`'s provenance, exactly as `&raw mut (*self.0).w_cursor`
        // would, without asking the window to be readable to say where its
        // cursor is.
        Pos(self.0.wrapping_byte_add(offset_of!(win_T, w_cursor)).cast())
    }

    /// The buffer this window shows, `None` for the moment between losing one
    /// and being given another.
    #[inline(always)]
    pub fn buffer_or_none(self) -> Option<Buf> {
        // A live window's `w_buffer` is a live buffer or null.
        let buf = self.w_buffer;
        (!buf.is_null()).then(|| Buf::wrap(buf))
    }

    /// The next window in this tab page's list, if any.
    #[inline(always)]
    pub fn next(self) -> Option<Self> {
        // A live window's `w_next` is a live window or null.
        let next = self.w_next;
        (!next.is_null()).then(|| Self::wrap(next))
    }

    /// The window before this one in its tab page's list, if any.
    #[inline(always)]
    pub fn prev(self) -> Option<Self> {
        // A live window's `w_prev` is a live window or null.
        let prev = self.w_prev;
        (!prev.is_null()).then(|| Self::wrap(prev))
    }

    /// First line of the fold containing `lnum`, if there is one.
    #[inline(always)]
    pub fn fold_first(self, lnum: linenr_T) -> Option<linenr_T> {
        let mut first = lnum;
        // SAFETY: a live window. `firstp` is written only when the answer is
        // true, so the seed survives a line that is in no fold.
        let folded = unsafe { has_folding(self.0, lnum, &raw mut first, ptr::null_mut()) };
        folded.then_some(first)
    }

    /// Last line of the fold containing `lnum`, or `lnum` when it is in none.
    #[inline(always)]
    pub fn fold_last(self, lnum: linenr_T) -> linenr_T {
        let mut last = lnum;
        // SAFETY: a live window; `lastp` is written only when folded.
        unsafe { has_folding(self.0, lnum, ptr::null_mut(), &raw mut last) };
        last
    }

    /// The whole fold containing `lnum`: whether there is one, and its first
    /// and last line (both `lnum` when there is not).
    #[inline(always)]
    pub fn fold_span(self, lnum: linenr_T) -> (bool, linenr_T, linenr_T) {
        let (mut first, mut last) = (lnum, lnum);
        // SAFETY: a live window; both out-params are written only when folded.
        let folded = unsafe { has_folding(self.0, lnum, &raw mut first, &raw mut last) };
        (folded, first, last)
    }

    #[inline(always)]
    pub fn has_any_folding(self) -> bool {
        // SAFETY: a live window.
        unsafe { has_any_folding(self.0) != 0 }
    }

    /// First and last virtual column of the character at `pos`.
    #[inline(always)]
    pub fn vcol_span(self, pos: Pos) -> (colnr_T, colnr_T) {
        let (mut start, mut end) = (0, 0);
        // SAFETY: a live window and a live position in its buffer.
        unsafe { getvcol(self.0, pos.0, &raw mut start, ptr::null_mut(), &raw mut end) };
        (start, end)
    }

    /// Start, cursor and end virtual column of the character at `pos`.
    #[inline(always)]
    pub fn vcol_triple(self, pos: Pos) -> (colnr_T, colnr_T, colnr_T) {
        let (mut start, mut cursor, mut end) = (0, 0, 0);
        // SAFETY: a live window and a live position in its buffer.
        unsafe { getvcol(self.0, pos.0, &raw mut start, &raw mut cursor, &raw mut end) };
        (start, cursor, end)
    }

    /// [`Win::vcol_span`] with 'virtualedit' taken into account.
    #[inline(always)]
    pub fn virtual_vcol_span(self, pos: Pos) -> (colnr_T, colnr_T) {
        let (mut start, mut end) = (0, 0);
        // SAFETY: a live window and a live position in its buffer.
        unsafe { getvvcol(self.0, pos.0, &raw mut start, ptr::null_mut(), &raw mut end) };
        (start, end)
    }

    /// [`Win::vcol_triple`] with 'virtualedit' taken into account.
    #[inline(always)]
    pub fn virtual_vcol_triple(self, pos: Pos) -> (colnr_T, colnr_T, colnr_T) {
        let (mut start, mut cursor, mut end) = (0, 0, 0);
        // SAFETY: a live window and a live position in its buffer.
        unsafe { getvvcol(self.0, pos.0, &raw mut start, &raw mut cursor, &raw mut end) };
        (start, cursor, end)
    }

    /// The first virtual column of the character at `pos`, 'virtualedit'
    /// included.
    #[inline(always)]
    pub fn virtual_vcol(self, pos: Pos) -> colnr_T {
        self.virtual_vcol_span(pos).0
    }

    /// The virtual column the *cursor* shows at within the character at
    /// `pos`, which is not its first column when the character is a tab.
    #[inline(always)]
    pub fn virtual_cursor_vcol(self, pos: Pos) -> colnr_T {
        let mut cursor = 0;
        let (none, c) = (ptr::null_mut(), &raw mut cursor);
        // SAFETY: a live window and a live position in its buffer.
        unsafe { getvvcol(self.0, pos.0, none, c, none) };
        cursor
    }

    #[inline(always)]
    pub fn redraw_later(self, redraw_type: ::core::ffi::c_int) {
        // SAFETY: a live window.
        unsafe { redraw_later(self.0, redraw_type) };
    }
}

impl Buf {
    /// # Safety
    /// `buf` must stay a live buffer for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn new(buf: *mut buf_T) -> Self {
        Self::wrap(buf)
    }

    /// [`Win::wrap`] for a buffer, nulls and all.
    #[inline(always)]
    const fn wrap(buf: *mut buf_T) -> Self {
        match buf.is_null() {
            true => Self(buf, 0),
            // SAFETY: non-null here is a live buffer.
            false => Self(buf, unsafe { (*buf).handle }),
        }
    }

    /// The buffer `buf` names, `None` for null.
    ///
    /// # Safety
    /// `buf` must be null, or stay a live buffer for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn from_raw(buf: *mut buf_T) -> Option<Self> {
        if buf.is_null() {
            None
        } else {
            Some(Self::wrap(buf))
        }
    }

    /// The buffer the editor is working in.
    ///
    /// # Safety
    /// `curbuf` must be set, which it is from startup to exit.
    #[inline(always)]
    pub unsafe fn current() -> Self {
        Self::wrap(curbuf.get())
    }

    #[inline(always)]
    pub fn raw(self) -> *mut buf_T {
        self.0
    }

    /// This buffer's number: the handle the API and `:ls` show, and what the
    /// registry finds it by. [`Win::handle`] for a buffer.
    #[inline(always)]
    pub fn handle(self) -> handle_T {
        self.1
    }

    /// [`Win::assign_handle`] for a buffer, whose handle is its number.
    #[inline(always)]
    pub(crate) fn assign_handle(&mut self, handle: handle_T) {
        self.handle = handle;
        self.1 = handle;
    }

    /// Whether the buffer this value was derived from is still registered.
    /// [`Win::valid`] for a buffer, with the same warning attached: this is
    /// not `buflist_findnr`, and it is not `bufref_valid` either — a
    /// `bufref_T` also insists the buffer has not been freed *and* reallocated
    /// under the same number, which cannot happen, and it is checked against a
    /// buffer that was on the list. The two off-list buffers
    /// (`ml_recover`'s scratch, `open_spellbuf`'s dummy) are registered by
    /// nobody and answer `false` here.
    #[inline(always)]
    pub fn valid(self) -> bool {
        buffer(self.1) == Some(self)
    }

    #[inline(always)]
    pub fn line_count(self) -> linenr_T {
        self.b_ml.ml_line_count
    }

    /// # Safety
    /// `lnum` must be a line of this buffer.
    #[inline(always)]
    pub unsafe fn line(self, lnum: linenr_T) -> Line {
        Line(unsafe { ml_get_buf(self.0, lnum) })
    }

    /// [`Buf::line`], marking the line dirty so the caller may write to it.
    ///
    /// # Safety
    /// `lnum` must be a line of this buffer.
    #[inline(always)]
    pub unsafe fn line_mut(self, lnum: linenr_T) -> Line {
        Line(unsafe { ml_get_buf_mut(self.0, lnum) })
    }

    /// Bytes in line `lnum`, the terminating NUL excluded.
    ///
    /// # Safety
    /// `lnum` must be a line of this buffer.
    #[inline(always)]
    pub unsafe fn line_len(self, lnum: linenr_T) -> colnr_T {
        unsafe { ml_get_buf_len(self.0, lnum) }
    }

    /// Step `pos` back off a trail byte, so it names a whole character.
    #[inline(always)]
    pub fn snap_to_char(self, pos: Pos) {
        // SAFETY: a live buffer and a live position in it.
        unsafe { mark_mb_adjustpos(self.0, pos.0) };
    }

    /// The next buffer in the editor's buffer list, if any.
    #[inline(always)]
    pub fn next(self) -> Option<Self> {
        // A live buffer's `b_next` is a live buffer or null.
        let next = self.b_next;
        (!next.is_null()).then(|| Self::wrap(next))
    }
}

impl Frame {
    /// # Safety
    /// `fp` must stay a live frame for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn new(fp: *mut frame_T) -> Self {
        Self(fp)
    }

    /// The frame `fp` names, `None` for null.
    ///
    /// # Safety
    /// `fp` must be null, or stay a live frame for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn from_raw(fp: *mut frame_T) -> Option<Self> {
        if fp.is_null() { None } else { Some(Self(fp)) }
    }

    #[inline(always)]
    pub fn raw(self) -> *mut frame_T {
        self.0
    }

    /// The window this frame holds — `Some` exactly for a leaf.
    #[inline(always)]
    pub fn win(self) -> Option<Win> {
        // A live leaf frame's `fr_win` is a live window; a row or column's is
        // null.
        let win = self.fr_win;
        (!win.is_null()).then(|| Win::wrap(win))
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
    /// `tp` must stay a live tab page for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn new(tp: *mut tabpage_T) -> Self {
        Self::wrap(tp)
    }

    /// [`Win::wrap`] for a tab page, nulls and all.
    #[inline(always)]
    const fn wrap(tp: *mut tabpage_T) -> Self {
        match tp.is_null() {
            true => Self(tp, 0),
            // SAFETY: non-null here is a live tab page.
            false => Self(tp, unsafe { (*tp).handle }),
        }
    }

    /// The tab page `tp` names, `None` for null — which is how the window
    /// family spells "the current one" throughout.
    ///
    /// # Safety
    /// `tp` must be null, or stay a live tab page for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn from_raw(tp: *mut tabpage_T) -> Option<Self> {
        if tp.is_null() {
            None
        } else {
            Some(Self::wrap(tp))
        }
    }

    /// The tab page the editor is working in.
    ///
    /// # Safety
    /// `curtab` must be set, which it is from startup to exit.
    #[inline(always)]
    pub unsafe fn current() -> Self {
        Self::wrap(curtab.get())
    }

    #[inline(always)]
    pub fn raw(self) -> *mut tabpage_T {
        self.0
    }

    /// This tab page's id. [`Win::handle`] for a tab page.
    #[inline(always)]
    pub fn handle(self) -> handle_T {
        self.1
    }

    /// [`Win::assign_handle`] for a tab page.
    #[inline(always)]
    pub(crate) fn assign_handle(&mut self, handle: handle_T) {
        self.handle = handle;
        self.1 = handle;
    }

    /// Whether the tab page this value was derived from is still registered.
    /// [`Win::valid`] for a tab page — and *not* `valid_tabpage()`, which
    /// walks the editor's tab page list looking for an address.
    #[inline(always)]
    pub fn valid(self) -> bool {
        tabpage(self.1) == Some(self)
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
        // A live tab page's `tp_next` is a live tab page or null.
        let next = self.tp_next;
        (!next.is_null()).then(|| Self::wrap(next))
    }

    /// The root of this tab page's layout tree, `tp_topframe` verbatim.
    ///
    /// Unlike `tp_firstwin`, upstream reads this field for the current tab page
    /// too (`min_rows`, `win_vert_neighbor`), so this does not switch to the
    /// `topframe` global the way [`windows_in_tab`] switches to `firstwin`.
    #[inline(always)]
    pub fn topframe(self) -> Frame {
        // A live tab page's top frame is live.
        Frame(self.tp_topframe)
    }
}

impl Pos {
    /// # Safety
    /// `pos` must stay a live position for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn new(pos: *mut pos_T) -> Self {
        Self(pos)
    }

    #[inline(always)]
    pub fn raw(self) -> *mut pos_T {
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

// ---------------------------------------------------------------------------
// The lists, walked
//
// Each of these is one of the C's `FOR_ALL_*` macros. The lists are the
// editor's own: they are built before the first window is drawn and torn down
// only at exit, and every link ends at a null, so producing the head and
// stepping the chain needs no promise from the caller — which is what makes
// these safe functions rather than `unsafe fn`s. A walk that its own body can
// invalidate is a different matter and stays the caller's problem: none of
// these re-reads the head, exactly as the macros do not.

/// The windows hanging off `first`, in list order.
fn win_chain(first: *mut win_T) -> impl Iterator<Item = Win> {
    // The chain is a live window list ending at a null `w_next`.
    iter::successors((!first.is_null()).then(|| Win::wrap(first)), |wp| wp.next())
}

/// Every window of the current tab page, in list order: the C's
/// `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)`, whose `curtab == curtab` test always
/// picks `firstwin`.
pub fn windows() -> impl Iterator<Item = Win> {
    win_chain(firstwin.get())
}

/// Every window of tab page `tp`, in list order: `FOR_ALL_WINDOWS_IN_TAB`.
///
/// The current tab page's windows hang off the `firstwin` global rather than
/// off its own `tp_firstwin`, which is stale while it is current — that is
/// what the macro's first arm reads.
pub fn windows_in_tab(tp: TabPage) -> impl Iterator<Item = Win> {
    win_chain(if tp.is_current() {
        firstwin.get()
    } else {
        tp.tp_firstwin
    })
}

/// Every tab page, in list order: the C's `FOR_ALL_TABS`.
pub fn tabs() -> impl Iterator<Item = TabPage> {
    // The chain is the editor's tab page list, ending at a null `tp_next`.
    let first = first_tabpage.get();
    iter::successors((!first.is_null()).then(|| TabPage::wrap(first)), |tp| {
        tp.next()
    })
}

/// Every window of every tab page: `FOR_ALL_TAB_WINDOWS`, which is exactly
/// [`tabs`] with [`windows_in_tab`] inside it. `tp_next` is read after the tab
/// page's own windows are exhausted, as the macro's outer `for` reads it.
pub fn tab_windows() -> impl Iterator<Item = Win> {
    tabs().flat_map(windows_in_tab)
}

/// `first` and every frame after it in its row or column: the C's
/// `FOR_ALL_FRAMES(frp, first)`, whose head is usually a `fr_child`.
pub fn frames(first: Option<Frame>) -> impl Iterator<Item = Frame> {
    iter::successors(first, |fr| fr.next())
}

/// [`frames`] the other way, following `fr_prev`. The C spells this out as a
/// `while` loop each time it needs it (`frame_setheight`'s second run, say).
pub fn frames_back(first: Option<Frame>) -> impl Iterator<Item = Frame> {
    iter::successors(first, |fr| fr.prev())
}

/// Every buffer, in list order: the C's `FOR_ALL_BUFFERS`.
pub fn buffers() -> impl Iterator<Item = Buf> {
    // The chain is the editor's buffer list, ending at a null `b_next`.
    let first = firstbuf.get();
    iter::successors((!first.is_null()).then(|| Buf::wrap(first)), |buf| {
        buf.next()
    })
}
