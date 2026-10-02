//! The postfix form the parser emits and [`super::build`] consumes.
//!
//! Upstream keeps this as three raw cursors (`post_start`, `post_ptr`,
//! `post_end`) over one `xmalloc`ed `int` array, with the "append, growing
//! if full" dance open-coded at every one of its ~100 uses. It is a stack of
//! `int`s and nothing more: the items are opcodes (the negative `NFA_*`
//! constants), literal code points, and the odd inline operand. This module
//! is that stack behind a checked API — [`Postfix::emit`], [`Postfix::len`],
//! [`Postfix::truncate`], [`Postfix::drop_last`], [`Postfix::items`] — so
//! that the parsers above it hold no pointers at all.
//!
//! The parser rewinds the stack as well as appending to it: `\{n,m}`
//! re-parses its atom and throws away what the speculative pass emitted,
//! which is [`Postfix::len`] plus [`Postfix::truncate`].
//!
//! # Why this is not simply a `Vec`
//!
//! It was, and it cost a factor of three. Appending happens once per emitted
//! item, and the compile-speed test's
//! `\v(((((Nxxxxxxx&&xxxx){179})+)+)+){179}` emits eight million of them:
//! the `\{n,m}` expansion re-parses its atom once per repetition and the
//! repetitions nest. At opt-level 0 — which is what the test suites run —
//! none of `Vec`'s accessors inline, so `push` is a chain of half a dozen
//! calls where upstream's macro was six instructions. `Vec` still owns the
//! allocation; the length and the write pointer are kept beside it as plain
//! fields so the hot path touches neither.
//!
//! The program is a field of the compile's `RegCompiler`, reached by `&mut`:
//! no cell, no borrow table, nothing between `emit` and the store.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::regexp::NfaOp;
use core::ffi::c_int;

/// The postfix program under construction. A field of the compile's
/// [`RegCompiler`](crate::regexp::RegCompiler).
pub(crate) struct Postfix {
    /// Owns the allocation. Its own length stays 0; `len` below is the real
    /// one, so that appending never calls into `Vec`.
    buf: Vec<c_int>,
    /// `buf.as_mut_ptr()`, refreshed whenever `buf` reallocates.
    items: *mut c_int,
    /// `buf.capacity()`, likewise.
    cap: usize,
    len: usize,
}

impl Postfix {
    /// An empty program with nothing reserved.
    pub(crate) const fn new() -> Postfix {
        Postfix {
            buf: Vec::new(),
            items: core::ptr::null_mut(),
            cap: 0,
            len: 0,
        }
    }

    /// Start a fresh program for a pattern `pattern_len` bytes long.
    ///
    /// The reservation is upstream's guess at how many items a pattern of
    /// that length can produce; it is only a capacity, so an underestimate
    /// costs a reallocation rather than a failure.
    pub(crate) fn start(&mut self, pattern_len: usize) {
        self.len = 0;
        self.reserve((pattern_len + 1) * 25 + 1000);
    }

    /// Make room for at least `want` items, keeping the cached pointer and
    /// capacity in step with `buf`.
    fn reserve(&mut self, want: usize) {
        if want <= self.cap {
            return;
        }
        self.buf.reserve_exact(want);
        self.items = self.buf.as_mut_ptr();
        self.cap = self.buf.capacity();
    }

    /// Grow the program. Out of line: appending is the hot path and this
    /// runs a handful of times per compile.
    #[inline(never)]
    #[cold]
    fn grow(&mut self) {
        // Upstream's `realloc_post_list` grows by half again.
        let want = self.cap + self.cap / 2 + 1;
        self.reserve(want);
    }

    /// Append one item. The transpiled form of upstream's `EMIT` macro.
    #[inline(always)]
    pub(crate) fn emit(&mut self, item: c_int) {
        if self.len == self.cap {
            self.grow();
        }
        // SAFETY: `len` is below `cap` now, and `items` addresses `cap`
        // items of the allocation `buf` owns.
        unsafe { *self.items.add(self.len) = item };
        self.len += 1;
    }

    /// Append `item` followed by the `NFA_CONCAT` that joins it to what came
    /// before — the shape most of the collection parser emits in.
    #[inline(always)]
    pub(crate) fn emit_concat(&mut self, item: c_int) {
        self.emit(item);
        self.emit_op(NfaOp::Concat);
    }

    /// Append one opcode, which is where the program's *named* half is
    /// written.
    ///
    /// The `c_int` [`Postfix::emit`] takes is either an opcode or a literal
    /// character; this is the half the type system can hold on to.
    #[inline(always)]
    pub(crate) fn emit_op(&mut self, op: NfaOp) {
        self.emit(op.code());
    }

    /// How many items have been emitted; a handle [`Postfix::truncate`] can
    /// rewind to.
    #[inline(always)]
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    /// Rewind to a [`Postfix::len`] handle, dropping everything emitted
    /// since.
    #[inline(always)]
    pub(crate) fn truncate(&mut self, mark: usize) {
        debug_assert!(mark <= self.len, "a mark from `len`");
        self.len = mark;
    }

    /// Drop the last item. `[a-z]` uses this to reclaim the `NFA_CONCAT`
    /// that followed the range's start character, which it emitted before
    /// it knew a `-` came next.
    #[inline(always)]
    pub(crate) fn drop_last(&mut self) {
        self.len = self.len.saturating_sub(1);
    }

    /// The program as emitted, for the compile's read phase.
    pub(crate) fn items(&self) -> &[c_int] {
        if self.items.is_null() {
            return &[];
        }
        // SAFETY: `items` addresses `len` initialised items of `buf`'s
        // allocation, and the borrow of `self` keeps it from reallocating.
        unsafe { core::slice::from_raw_parts(self.items, self.len) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emitted_items_read_back_in_order_across_growth() {
        let mut post = Postfix::new();
        assert!(post.items().is_empty());
        post.start(0);
        for i in 0..5000 {
            post.emit(i);
        }
        post.emit_concat(-1);
        assert_eq!(post.len(), 5002);
        assert_eq!(post.items()[4999], 4999);
        assert_eq!(post.items()[5001], NfaOp::Concat.code());
    }

    #[test]
    fn truncate_and_drop_last_rewind() {
        let mut post = Postfix::new();
        post.start(1);
        post.emit(1);
        let mark = post.len();
        post.emit(2);
        post.emit_op(NfaOp::Star);
        post.truncate(mark);
        assert_eq!(post.items(), [1]);
        post.drop_last();
        post.drop_last();
        assert!(post.items().is_empty());
    }
}
