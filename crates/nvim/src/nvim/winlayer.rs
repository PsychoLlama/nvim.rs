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
//! [`Pos`] and [`Line`] each wrap one pointer and make its **construction**
//! the unsafe step; from there [`Deref`]/[`DerefMut`] give ordinary field
//! access and the handful of accessors below give the projections a bare
//! `&`/`&mut` cannot express — the buffer behind a window, a line of that
//! buffer, the span of a fold. Every one of them rests on the single promise
//! the constructor took, which each `pub unsafe fn` in a consumer restates in
//! its own `# Safety` section.
//!
//! Each family adds the wrappers it needs as its own `impl Win` block (an
//! inherent impl may live in any module of the defining crate), so this module
//! stays the shared minimum rather than growing a method per caller.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_char;
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::src::nvim::drawscreen::redraw_later;
use crate::src::nvim::fold::{hasAnyFolding, hasFolding};
use crate::src::nvim::main::{curbuf, curwin};
use crate::src::nvim::mark::mark_mb_adjustpos;
use crate::src::nvim::mbyte::{utf_ptr2StrCharInfo, utfc_next};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len, ml_get_buf_mut};
use crate::src::nvim::plines::{getvcol, getvvcol};
use crate::src::nvim::types::{StrCharInfo, buf_T, colnr_T, linenr_T, pos_T, win_T};

// ---------------------------------------------------------------------------
// The pointers, wrapped

/// A window the caller has promised is live.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Win(*mut win_T);

/// A buffer the caller has promised is live.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Buf(*mut buf_T);

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

    /// Whether this is the window the editor is working in.
    ///
    /// # Safety
    /// `curwin` must be set, which it is from startup to exit.
    #[inline(always)]
    pub unsafe fn is_current(self) -> bool {
        self.0 == curwin.get()
    }

    #[inline(always)]
    pub fn buffer(self) -> Buf {
        // SAFETY: a live window's buffer is live.
        Buf(unsafe { (*self.0).w_buffer })
    }

    #[inline(always)]
    pub fn cursor(self) -> Pos {
        // SAFETY: the cursor is a field of the live window.
        Pos(unsafe { &raw mut (*self.0).w_cursor })
    }

    /// The next window in this tab page's list, if any.
    #[inline(always)]
    pub fn next(self) -> Option<Self> {
        // SAFETY: a live window's `w_next` is a live window or NULL.
        let next = unsafe { (*self.0).w_next };
        (!next.is_null()).then_some(Self(next))
    }

    /// First line of the fold containing `lnum`, if there is one.
    #[inline(always)]
    pub fn fold_first(self, lnum: linenr_T) -> Option<linenr_T> {
        let mut first = lnum;
        // SAFETY: a live window. `firstp` is written only when the answer is
        // true, so the seed survives a line that is in no fold.
        let folded = unsafe { hasFolding(self.0, lnum, &raw mut first, ptr::null_mut()) };
        folded.then_some(first)
    }

    /// Last line of the fold containing `lnum`, or `lnum` when it is in none.
    #[inline(always)]
    pub fn fold_last(self, lnum: linenr_T) -> linenr_T {
        let mut last = lnum;
        // SAFETY: a live window; `lastp` is written only when folded.
        unsafe { hasFolding(self.0, lnum, ptr::null_mut(), &raw mut last) };
        last
    }

    /// The whole fold containing `lnum`: whether there is one, and its first
    /// and last line (both `lnum` when there is not).
    #[inline(always)]
    pub fn fold_span(self, lnum: linenr_T) -> (bool, linenr_T, linenr_T) {
        let (mut first, mut last) = (lnum, lnum);
        // SAFETY: a live window; both out-params are written only when folded.
        let folded = unsafe { hasFolding(self.0, lnum, &raw mut first, &raw mut last) };
        (folded, first, last)
    }

    #[inline(always)]
    pub fn has_any_folding(self) -> bool {
        // SAFETY: a live window.
        unsafe { hasAnyFolding(self.0) != 0 }
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
        unsafe { utf_ptr2StrCharInfo(self.0) }
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
