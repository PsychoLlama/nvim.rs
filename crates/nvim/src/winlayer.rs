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
//! The walks at the bottom — [`windows`], [`windows_in_tab`], [`tab_windows`],
//! [`buffers`] and [`frames`], plus [`tabs`] and [`frames_back`] under them —
//! are the C's `FOR_ALL_WINDOWS_IN_TAB`, `FOR_ALL_TAB_WINDOWS`,
//! `FOR_ALL_BUFFERS` and `FOR_ALL_FRAMES`. The lists they walk are the
//! editor's own and live from startup to exit, so the walks are safe
//! functions; each is lazy, as the macro it replaces is.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_char;
use core::ops::{Deref, DerefMut};
use core::{iter, ptr};

use crate::buffer::free;
use crate::drawscreen::redraw_later;
use crate::fold::{has_any_folding, has_folding};
use crate::global_cell::GlobalCell;
use crate::main::{curbuf, curtab, curwin, first_tabpage, firstbuf, firstwin};
use crate::mark::mark_mb_adjustpos;
use crate::mbyte::{utf_ptr2str_char_info, utfc_next};
use crate::memline::{ml_get_buf, ml_get_buf_len, ml_get_buf_mut};
use crate::plines::{getvcol, getvvcol};
use crate::registry::{HandleRegistry, PendingFree};
use crate::types::{
    StrCharInfo, buf_T, colnr_T, frame_T, handle_T, linenr_T, pos_T, tabpage_T, win_T,
};

// ---------------------------------------------------------------------------
// The pointers, wrapped

/// A window the caller has promised is live.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Win(*mut win_T);

/// A buffer the caller has promised is live.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Buf(*mut buf_T);

/// A frame of the window layout tree the caller has promised is live.
///
/// A frame is either a leaf holding one window (`fr_win`) or a row or column
/// of child frames (`fr_child`, chained through `fr_next`); `fr_parent` walks
/// back up. Which of the two a frame is, `fr_layout` says.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Frame(*mut frame_T);

/// A tab page the caller has promised is live.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TabPage(*mut tabpage_T);

/// A cursor or mark position the caller has promised is live.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pos(*mut pos_T);

/// A NUL-terminated buffer line, as `ml_get_buf` hands it back.
#[derive(Clone, Copy)]
pub struct Line(*mut c_char);

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
        Self(wp)
    }

    /// The window `wp` names, `None` for null.
    ///
    /// # Safety
    /// `wp` must be null, or stay a live window for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn from_raw(wp: *mut win_T) -> Option<Self> {
        if wp.is_null() { None } else { Some(Self(wp)) }
    }

    /// The window the editor is working in.
    ///
    /// # Safety
    /// `curwin` must be set, which it is from startup to exit.
    #[inline(always)]
    pub unsafe fn current() -> Self {
        Self(curwin.get())
    }

    #[inline(always)]
    pub fn raw(self) -> *mut win_T {
        self.0
    }

    /// This window's id: the handle the API, `win_getid()` and the registry
    /// all name it by. Identity that survives the address being reused.
    #[inline(always)]
    pub fn handle(self) -> handle_T {
        self.handle
    }

    /// Whether this is the window the editor is working in.
    ///
    /// Safe where [`Win::current`] is not: comparing the two pointers reads
    /// neither of them.
    #[inline(always)]
    pub fn is_current(self) -> bool {
        self.0 == curwin.get()
    }

    #[inline(always)]
    pub fn buffer(self) -> Buf {
        // SAFETY: a live window's buffer is live.
        Buf(unsafe { (*self.0).w_buffer })
    }

    /// The leaf frame this window sits in. Every window has one, floats
    /// included — a float's frame is simply not linked into the layout tree.
    #[inline(always)]
    pub fn frame(self) -> Frame {
        // A live window's `w_frame` is a live frame.
        Frame(self.w_frame)
    }

    #[inline(always)]
    pub fn cursor(self) -> Pos {
        // SAFETY: the cursor is a field of the live window.
        Pos(unsafe { &raw mut (*self.0).w_cursor })
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
        // SAFETY: a live window's `w_next` is a live window or NULL.
        let next = unsafe { (*self.0).w_next };
        (!next.is_null()).then_some(Self(next))
    }

    /// The window before this one in its tab page's list, if any.
    #[inline(always)]
    pub fn prev(self) -> Option<Self> {
        // A live window's `w_prev` is a live window or null.
        let prev = self.w_prev;
        (!prev.is_null()).then_some(Self(prev))
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
        Self(buf)
    }

    /// The buffer `buf` names, `None` for null.
    ///
    /// # Safety
    /// `buf` must be null, or stay a live buffer for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn from_raw(buf: *mut buf_T) -> Option<Self> {
        if buf.is_null() { None } else { Some(Self(buf)) }
    }

    /// The buffer the editor is working in.
    ///
    /// # Safety
    /// `curbuf` must be set, which it is from startup to exit.
    #[inline(always)]
    pub unsafe fn current() -> Self {
        Self(curbuf.get())
    }

    #[inline(always)]
    pub fn raw(self) -> *mut buf_T {
        self.0
    }

    /// This buffer's number: the handle the API and `:ls` show, and what the
    /// registry finds it by. [`Win::handle`] for a buffer.
    #[inline(always)]
    pub fn handle(self) -> handle_T {
        self.handle
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
        (!next.is_null()).then_some(Self(next))
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
    /// `tp` must stay a live tab page for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn new(tp: *mut tabpage_T) -> Self {
        Self(tp)
    }

    /// The tab page `tp` names, `None` for null — which is how the window
    /// family spells "the current one" throughout.
    ///
    /// # Safety
    /// `tp` must be null, or stay a live tab page for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn from_raw(tp: *mut tabpage_T) -> Option<Self> {
        if tp.is_null() { None } else { Some(Self(tp)) }
    }

    /// The tab page the editor is working in.
    ///
    /// # Safety
    /// `curtab` must be set, which it is from startup to exit.
    #[inline(always)]
    pub unsafe fn current() -> Self {
        Self(curtab.get())
    }

    #[inline(always)]
    pub fn raw(self) -> *mut tabpage_T {
        self.0
    }

    /// This tab page's id. [`Win::handle`] for a tab page.
    #[inline(always)]
    pub fn handle(self) -> handle_T {
        self.handle
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
        (!next.is_null()).then_some(Self(next))
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
// Finding one by handle
//
// The editor hands every window, buffer and tab page a monotone id — a
// `handle_T` — and keeps a table from that id to the object, so that an API
// call, an RPC message or a Lua callback can name one without holding a
// pointer across the call that might free it. Upstream spells the three
// tables as khash `Map_int_ptr_t`s reached through a raw pointer
// (`window_handles`, `buffer_handles`, `tabpage_handles`); here they are
// owned Rust, one [`HandleRegistry`] each.
//
// They live in this module and their statics are private, so that the two
// halves of the invariant [`HandleRegistry`] documents — everything in the
// table is live — are enforced by visibility rather than by review: the
// only way in is `register_*`, which the allocator calls, and the only way
// out is `forget_*`, which the free path calls first. That is what makes
// the three lookups safe functions.
//
// A registry does *not* answer "is this window on screen": a hidden window
// (`win_alloc(_, hidden)`) is registered and on no list, and the autocommand
// window is unregistered while it is idle. `win_valid` and friends stay list
// walks — see `window::win_valid`.

/// Every live window, by handle.
static WINDOWS: GlobalCell<HandleRegistry<win_T>> = GlobalCell::new(HandleRegistry::new());

/// Every live buffer, by number.
static BUFFERS: GlobalCell<HandleRegistry<buf_T>> = GlobalCell::new(HandleRegistry::new());

/// Every live tab page, by handle.
static TABPAGES: GlobalCell<HandleRegistry<tabpage_T>> = GlobalCell::new(HandleRegistry::new());

/// The window `handle` names, `None` once it has been closed.
pub(crate) fn window(handle: handle_T) -> Option<Win> {
    // The borrow ends with the lookup, which cannot re-enter.
    WINDOWS.with(|reg| reg.get(handle)).map(Win)
}

/// The buffer numbered `handle`, `None` once it has been wiped.
pub(crate) fn buffer(handle: handle_T) -> Option<Buf> {
    // As [`window`].
    BUFFERS.with(|reg| reg.get(handle)).map(Buf)
}

/// The tab page `handle` names, `None` once it has been closed.
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
    iter::successors((!first.is_null()).then_some(Win(first)), |wp| wp.next())
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
    iter::successors((!first.is_null()).then_some(TabPage(first)), |tp| tp.next())
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
    iter::successors((!first.is_null()).then_some(Buf(first)), |buf| buf.next())
}
